/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! `PARSE`, `ARG` and `PULL`: the options, the source and the template.

use crate::ast::{Expr, Parse, ParseSource, ParseTrigger, TriggerKind};
use crate::expr::{
    Terminators, need_variable, parse_paren_expression, parse_variable_or_message_term, symbol_kind,
};
use crate::token::{Operator, ParseError, SymbolClass, Tag, Token, TokenKind};

use super::{
    Inst, POPT_ARG, POPT_CASELESS, POPT_LINEIN, POPT_LOWER, POPT_PULL, POPT_SOURCE, POPT_UPPER,
    POPT_VALUE, POPT_VAR, POPT_VERSION, SUB_WITH,
};

impl<'a> Inst<'a> {
    /// `parseNew` (`InstructionParser.cpp:3102`), shared by `PARSE`, `ARG` and
    /// `PULL`.
    pub(super) fn parse_instruction_body(
        &mut self,
        short_form: Option<ParseSource>,
    ) -> Result<Parse, ParseError> {
        let mut upper = short_form.is_some();
        let mut lower = false;
        let mut caseless = false;
        let source = match short_form {
            Some(source) => source,
            None => {
                // The option modifiers come first and each only once. A
                // repeat falls through to the source-keyword switch, where it
                // is an unknown source: measured, `parse upper upper arg a` is
                // rc 231, Error 25.12.
                let option = loop {
                    let Some(token) = self.next_real() else {
                        return Err(self.error(20, 903));
                    };
                    if token.kind.tag() != Tag::Symbol {
                        return Err(self.error(20, 903));
                    }
                    let option = self.parse_option(token);
                    match option {
                        Some(POPT_UPPER) if !upper && !lower => upper = true,
                        Some(POPT_LOWER) if !upper && !lower => lower = true,
                        Some(POPT_CASELESS) if !caseless => caseless = true,
                        _ => break option,
                    }
                };
                match option {
                    Some(POPT_ARG) => ParseSource::Arg,
                    Some(POPT_LINEIN) => ParseSource::LineIn,
                    Some(POPT_PULL) => ParseSource::Pull,
                    Some(POPT_SOURCE) => ParseSource::Source,
                    Some(POPT_VERSION) => ParseSource::Version,
                    Some(POPT_VAR) => {
                        // Measured: `parse var` is 20.904 and `parse var 1 a`
                        // is 31.2, so the symbol test and the variable test are
                        // separate.
                        let Some(name) = self.next_real() else {
                            return Err(self.error(20, 904));
                        };
                        let TokenKind::Symbol { id, class } = name.kind else {
                            return Err(self.error(20, 904));
                        };
                        need_variable(self.ctx, id, class, self.clause_byte)?;
                        ParseSource::Var(id)
                    }
                    Some(POPT_VALUE) => {
                        // The expression is optional and defaults to the null
                        // string: measured, `parse value with a` is rc 0.
                        let value = self.opt_expr(Terminators::PARSE_WITH)?;
                        // `Error_Invalid_template_with`. Measured:
                        // `parse value "x" a` is rc 218, Error 38.3.
                        let with = self
                            .next_real()
                            .filter(|token| self.sub_keyword(token) == Some(SUB_WITH));
                        if with.is_none() {
                            return Err(self.error(38, 3));
                        }
                        ParseSource::Value(value)
                    }
                    // `Error_Invalid_subkeyword_parse`.
                    _ => return Err(self.error(25, 12)),
                }
            }
        };
        let template = self.parse_template(caseless)?;
        Ok(Parse {
            source,
            upper,
            lower,
            caseless,
            template,
        })
    }

    /// `RexxToken::parseOption` (`KeywordConstants.cpp:551`), which is a table
    /// of its own and not the sub-keyword table.
    fn parse_option(&self, token: &Token) -> Option<usize> {
        match &token.kind {
            TokenKind::Symbol { id, .. } => self.ctx.keywords.parse_options.index_of(*id),
            _ => None,
        }
    }

    /// The template grammar (`InstructionParser.cpp:3239`-`3418`), shared by
    /// all three spellings.
    fn parse_template(&mut self, caseless: bool) -> Result<Vec<Option<ParseTrigger>>, ParseError> {
        let string_kind = if caseless {
            TriggerKind::Mixed
        } else {
            TriggerKind::String
        };
        let mut template: Vec<Option<ParseTrigger>> = Vec::new();
        let mut targets: Vec<Option<Expr>> = Vec::new();
        loop {
            let Some(index) = self.peek_real_index() else {
                if !targets.is_empty() {
                    template.push(Some(ParseTrigger {
                        kind: TriggerKind::End,
                        value: None,
                        targets,
                    }));
                }
                break;
            };
            let token = &self.ctx.tokens[index];
            self.seek(index + 1);
            match &token.kind {
                TokenKind::Comma => {
                    if !targets.is_empty() {
                        template.push(Some(ParseTrigger {
                            kind: TriggerKind::End,
                            value: None,
                            targets: std::mem::take(&mut targets),
                        }));
                    }
                    template.push(None);
                }
                TokenKind::Operator(op) => {
                    let kind = match op {
                        Operator::Plus => TriggerKind::Plus,
                        Operator::Subtract => TriggerKind::Minus,
                        Operator::Equal => TriggerKind::Absolute,
                        Operator::LessThan => TriggerKind::MinusLength,
                        Operator::GreaterThan => TriggerKind::PlusLength,
                        // `Error_Invalid_template_trigger`. Measured:
                        // `parse arg *3` is rc 218, Error 38.1.
                        _ => return Err(self.error(38, 1)),
                    };
                    let value = self.trigger_position()?;
                    template.push(Some(ParseTrigger {
                        kind,
                        value: Some(value),
                        targets: std::mem::take(&mut targets),
                    }));
                }
                TokenKind::LeftParen => {
                    let Some(value) = parse_paren_expression(self.ctx, &mut self.cursor)? else {
                        // `Error_Invalid_expression_parse`. Measured:
                        // `parse arg +()` is rc 221, Error 35.931.
                        return Err(self.error(35, 931));
                    };
                    template.push(Some(ParseTrigger {
                        kind: string_kind,
                        value: Some(value),
                        targets: std::mem::take(&mut targets),
                    }));
                }
                TokenKind::Literal { value } => {
                    let literal = Expr::new(
                        crate::ast::ExprKind::Literal(value.clone()),
                        token.span.clone(),
                    );
                    template.push(Some(ParseTrigger {
                        kind: string_kind,
                        value: Some(literal),
                        targets: std::mem::take(&mut targets),
                    }));
                }
                TokenKind::Symbol { id, class } => {
                    let (id, class) = (*id, *class);
                    match class {
                        // A bare number is an absolute column.
                        SymbolClass::Constant => {
                            template.push(Some(ParseTrigger {
                                kind: TriggerKind::Absolute,
                                value: Some(Expr::new(symbol_kind(id, class), token.span.clone())),
                                targets: std::mem::take(&mut targets),
                            }));
                        }
                        // A lone period consumes a field and assigns nothing.
                        SymbolClass::Dummy => targets.push(None),
                        _ => {
                            // Step back onto the symbol and parse a target,
                            // which may be a message term: measured,
                            // `parse arg q~x` is rc 0.
                            let mut trial = self.trial_from(index);
                            match parse_variable_or_message_term(self.ctx, &mut trial)? {
                                Some(target) => {
                                    self.cursor = trial;
                                    targets.push(Some(target));
                                }
                                // `Error_Variable_expected_PARSE`, 89.2.
                                None => return Err(self.error(89, 2)),
                            }
                        }
                    }
                }
                _ => return Err(self.error(38, 1)),
            }
        }
        Ok(template)
    }

    /// The column a `+`, `-`, `=`, `<` or `>` trigger moves to.
    fn trigger_position(&mut self) -> Result<Expr, ParseError> {
        let Some(token) = self.next_real() else {
            // `Error_Invalid_template_missing`. Measured: `parse arg +` is
            // rc 218, Error 38.901.
            return Err(self.error(38, 901));
        };
        match &token.kind {
            TokenKind::LeftParen => match parse_paren_expression(self.ctx, &mut self.cursor)? {
                Some(value) => Ok(value),
                None => Err(self.error(35, 931)),
            },
            TokenKind::Symbol { id, class } => {
                // `Error_Invalid_template_position`, for a variable, a stem or
                // a compound.
                if matches!(
                    class,
                    SymbolClass::Variable | SymbolClass::Stem | SymbolClass::Compound
                ) {
                    return Err(self.error(38, 2));
                }
                Ok(Expr::new(symbol_kind(*id, *class), token.span.clone()))
            }
            _ => Err(self.error(38, 2)),
        }
    }
}
