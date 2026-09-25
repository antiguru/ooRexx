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

//! The loop header of `DO` and `LOOP`: `createLoop` and the loop forms it
//! builds.

use crate::ast::{ControlExpr, Controlled, Expr, Loop, LoopConditional, LoopKind};
use crate::expr::{Terminators, need_variable};
use crate::token::{Operator, ParseError, SymbolId, Tag, Token, TokenKind};

use super::{
    Inst, SUB_BY, SUB_COUNTER, SUB_FOR, SUB_FOREVER, SUB_INDEX, SUB_ITEM, SUB_LABEL, SUB_OVER,
    SUB_TO, SUB_UNTIL, SUB_WHILE, SUB_WITH, UNREACHABLE_SWITCH,
};

impl<'a> Inst<'a> {
    /// `createLoop` (`InstructionParser.cpp:1994`), for both `DO` and `LOOP`.
    pub(super) fn create_loop(&mut self, is_loop: bool) -> Result<Loop, ParseError> {
        let mut label = None;
        let mut counter = None;

        // `LABEL name` and `COUNTER name` come first, in either order, and each
        // only once. The loop stops on anything else, including a second
        // occurrence of one already seen, which is how `do label a label b`
        // becomes a DO whose count expression is `label b`.
        while let Some(token) = self.peek_real() {
            if token.kind.tag() != Tag::Symbol {
                break;
            }
            let which = match self.sub_keyword(token) {
                Some(SUB_LABEL) if label.is_none() => SUB_LABEL,
                Some(SUB_COUNTER) if counter.is_none() => SUB_COUNTER,
                _ => break,
            };
            let name = self.nth_real(1);
            match name.map(|token| &token.kind) {
                Some(TokenKind::Symbol { id, class }) => {
                    let (id, class) = (*id, *class);
                    if which == SUB_LABEL {
                        label = Some(id);
                    } else {
                        need_variable(self.ctx, id, class, self.clause_byte)?;
                        counter = Some(id);
                    }
                    let consumed = self.nth_real_index(1).expect("just matched it");
                    self.seek(consumed + 1);
                    if label.is_some() && counter.is_some() {
                        break;
                    }
                }
                // `do label = 1 to 2` is a controlled loop over a variable
                // named LABEL, not a labelled loop. Any `=` triggers this,
                // even one that is part of a larger operator, which is why
                // `==` is an expression error rather than a controlled loop.
                Some(TokenKind::Operator(Operator::Equal)) => {
                    let control = self.controlled(label, counter, token)?;
                    return Ok(control);
                }
                Some(TokenKind::Operator(Operator::StrictEqual)) => {
                    return Err(self.error(35, 1));
                }
                // `Error_Symbol_expected_LABEL` is 20.918 and
                // `Error_Symbol_expected_counter` is 20.934.
                _ => {
                    return Err(self.error(20, if which == SUB_LABEL { 918 } else { 934 }));
                }
            }
        }

        // Just the keyword and its options.
        if self.at_end() {
            if is_loop {
                return self.plain_loop(label, counter, LoopKind::Forever);
            }
            // `newSimpleDo` takes no counter, because a block does not iterate.
            if counter.is_some() {
                return Err(self.error(27, 905));
            }
            return Ok(Loop {
                label,
                counter,
                kind: LoopKind::Simple,
                conditional: None,
                end: None,
            });
        }

        let Some(first) = self.peek_real() else {
            unreachable!("at_end was just false")
        };
        if first.kind.tag() != Tag::Symbol {
            // `DO expr` where the expression does not start with a symbol.
            return self.count_loop(label, counter);
        }
        let second = self.nth_real(1);
        match second.map(|token| &token.kind) {
            Some(TokenKind::Operator(Operator::StrictEqual)) => Err(self.error(35, 1)),
            Some(TokenKind::Operator(Operator::Equal)) => self.controlled(label, counter, first),
            _ => {
                // `DO name OVER expr`. This test comes BEFORE the WITH one, so
                // `do with over x` is a DO OVER whose control variable is named
                // WITH: measured, rc 0 under rexxc.
                if second.and_then(|token| self.sub_keyword(token)) == Some(SUB_OVER) {
                    return self.do_over(label, counter, first);
                }
                if self.sub_keyword(first) == Some(SUB_WITH)
                    && matches!(
                        second.and_then(|token| self.sub_keyword(token)),
                        Some(SUB_INDEX | SUB_ITEM)
                    )
                {
                    // Step past WITH; the INDEX or ITEM keyword starts the
                    // options.
                    self.next_real();
                    return self.do_with(label, counter);
                }
                match self.sub_keyword(first) {
                    Some(SUB_FOREVER) => {
                        self.next_real();
                        // `Error_Invalid_do_forever` is 27.901, and it is the
                        // one reachable use of `parseLoopConditional`'s error
                        // argument: measured, `do forever x` is rc 229.
                        self.plain_loop(label, counter, LoopKind::Forever)
                    }
                    // `DO WHILE` and `DO UNTIL` are a FOREVER loop with the
                    // conditional attached, which is what `newLoopWhile` and
                    // `newLoopUntil` build.
                    Some(SUB_WHILE | SUB_UNTIL) => {
                        self.plain_loop(label, counter, LoopKind::Forever)
                    }
                    // Not a loop keyword, so this is `DO expr`.
                    _ => self.count_loop(label, counter),
                }
            }
        }
    }

    /// `DO FOREVER`, `DO WHILE` and `DO UNTIL`, which differ only in the
    /// conditional that follows.
    fn plain_loop(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
        kind: LoopKind,
    ) -> Result<Loop, ParseError> {
        let conditional = self.loop_conditional((27, 901))?;
        Ok(Loop {
            label,
            counter,
            kind,
            conditional,
            end: None,
        })
    }

    /// `parseCountLoop` (`InstructionParser.cpp:1916`): `DO expr`, with an
    /// optional trailing conditional.
    fn count_loop(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
    ) -> Result<Loop, ParseError> {
        let count = self.opt_expr(Terminators::COND)?;
        let conditional = self.loop_conditional(UNREACHABLE_SWITCH)?;
        Ok(Loop {
            label,
            counter,
            kind: LoopKind::Count(count),
            conditional,
            end: None,
        })
    }

    /// `newControlledLoop` (`InstructionParser.cpp:1265`):
    /// `DO i = initial TO t BY b FOR f`.
    fn controlled(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
        name: &Token,
    ) -> Result<Loop, ParseError> {
        let TokenKind::Symbol { id, class } = name.kind else {
            unreachable!("a controlled loop's control token is a symbol")
        };
        need_variable(self.ctx, id, class, self.clause_byte)?;
        // Step past the control variable and the `=`.
        let equals = self.nth_real_index(1).expect("the `=` that got us here");
        self.seek(equals + 1);

        let initial = self.expr(Terminators::CONTROL, 904)?;
        let mut control = Controlled {
            control: id,
            initial,
            to: None,
            by: None,
            for_count: None,
            order: Vec::new(),
        };
        let mut conditional = None;
        while let Some(token) = self.peek_real() {
            let (slot, missing, entry) = match self.sub_keyword(token) {
                Some(SUB_BY) => (&mut control.by, 905, ControlExpr::By),
                Some(SUB_TO) => (&mut control.to, 906, ControlExpr::To),
                Some(SUB_FOR) => (&mut control.for_count, 907, ControlExpr::For),
                Some(SUB_WHILE | SUB_UNTIL) => {
                    // `parseLoopConditional` allows nothing after itself, so
                    // this ends the clause.
                    conditional = self.loop_conditional(UNREACHABLE_SWITCH)?;
                    break;
                }
                _ => return Err(self.error(UNREACHABLE_SWITCH.0, UNREACHABLE_SWITCH.1)),
            };
            if slot.is_some() {
                // `Error_Invalid_do_duplicate`. Measured: `do i = 1 to 3 to 4`
                // is rc 229, Error 27.902.
                return Err(self.error(27, 902));
            }
            self.next_real();
            let value = self.expr(Terminators::CONTROL, missing)?;
            match entry {
                ControlExpr::By => control.by = Some(value),
                ControlExpr::To => control.to = Some(value),
                ControlExpr::For => control.for_count = Some(value),
            }
            control.order.push(entry);
        }
        Ok(Loop {
            // With no LABEL clause the control variable's name is the loop's
            // name, which is what `LEAVE i` matches against.
            label: label.or(Some(id)),
            counter,
            kind: LoopKind::Controlled(Box::new(control)),
            conditional,
            end: None,
        })
    }

    /// `newDoOverLoop` (`InstructionParser.cpp:1432`): `DO name OVER expr`,
    /// with an optional `FOR` and an optional conditional.
    fn do_over(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
        name: &Token,
    ) -> Result<Loop, ParseError> {
        let TokenKind::Symbol { id, class } = name.kind else {
            unreachable!("a DO OVER control token is a symbol")
        };
        need_variable(self.ctx, id, class, self.clause_byte)?;
        // Step past the control variable and the OVER keyword.
        let over = self.nth_real_index(1).expect("the OVER that got us here");
        self.seek(over + 1);
        let target = self.expr(Terminators::OVER, 911)?;
        let (for_count, conditional) = self.for_and_conditional()?;
        Ok(Loop {
            label: label.or(Some(id)),
            counter,
            kind: LoopKind::Over {
                control: id,
                target,
                for_count,
            },
            conditional,
            end: None,
        })
    }

    /// `newDoWithLoop` (`InstructionParser.cpp:1582`):
    /// `DO WITH INDEX i ITEM v OVER expr`.
    fn do_with(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
    ) -> Result<Loop, ParseError> {
        let mut index = None;
        let mut item = None;
        while let Some(token) = self.peek_real() {
            if token.kind.tag() != Tag::Symbol {
                break;
            }
            let slot = match self.sub_keyword(token) {
                Some(SUB_INDEX) => &mut index,
                Some(SUB_ITEM) => &mut item,
                _ => break,
            };
            if slot.is_some() {
                return Err(self.error(27, 902));
            }
            self.next_real();
            // `requiredVariable` then `addVariable`, so a non-symbol is
            // 20.929 and a symbol that is not a variable is `needVariable`'s
            // own number: measured, `do with index 1 over x` is 31.2 and
            // `do with index .a over x` is 31.3.
            let Some(name) = self.next_real() else {
                return Err(self.error(20, 929));
            };
            let TokenKind::Symbol { id, class } = name.kind else {
                return Err(self.error(20, 929));
            };
            need_variable(self.ctx, id, class, self.clause_byte)?;
            match self.sub_keyword(token) {
                Some(SUB_INDEX) => index = Some(id),
                _ => item = Some(id),
            }
        }
        if index.is_none() && item.is_none() {
            // `Error_Invalid_do_with_no_control`. Unreachable through
            // `createLoop`, which only comes here when the token after WITH is
            // INDEX or ITEM, and kept because `newDoWithLoop` checks it.
            return Err(self.error(27, 903));
        }
        // `Error_Invalid_do_with_no_over`. Measured: `do with index i x` is
        // rc 229, Error 27.904.
        let over = self
            .peek_real()
            .filter(|token| token.kind.tag() == Tag::Symbol);
        if over.and_then(|token| self.sub_keyword(token)) != Some(SUB_OVER) {
            return Err(self.error(27, 904));
        }
        self.next_real();
        let target = self.expr(Terminators::OVER, 911)?;
        let (for_count, conditional) = self.for_and_conditional()?;
        Ok(Loop {
            label,
            counter,
            kind: LoopKind::With {
                index,
                item,
                target,
                for_count,
            },
            conditional,
            end: None,
        })
    }

    /// The `FOR n` and `WHILE`/`UNTIL` tail that `DO OVER` and `DO WITH`
    /// share.
    fn for_and_conditional(
        &mut self,
    ) -> Result<(Option<Expr>, Option<LoopConditional>), ParseError> {
        let mut for_count = None;
        let mut conditional = None;
        while let Some(token) = self.peek_real() {
            match self.sub_keyword(token) {
                Some(SUB_FOR) => {
                    if for_count.is_some() {
                        return Err(self.error(27, 902));
                    }
                    self.next_real();
                    for_count = Some(self.expr(Terminators::CONTROL, 907)?);
                }
                Some(SUB_WHILE | SUB_UNTIL) => {
                    conditional = self.loop_conditional(UNREACHABLE_SWITCH)?;
                    break;
                }
                _ => return Err(self.error(UNREACHABLE_SWITCH.0, UNREACHABLE_SWITCH.1)),
            }
        }
        Ok((for_count, conditional))
    }

    /// `parseLoopConditional` (`InstructionParser.cpp:4600`): an optional
    /// trailing `WHILE` or `UNTIL`, and nothing after it.
    fn loop_conditional(
        &mut self,
        unexpected: (u16, u16),
    ) -> Result<Option<LoopConditional>, ParseError> {
        let Some(token) = self.next_real() else {
            return Ok(None);
        };
        let until = match self.sub_keyword(token) {
            Some(SUB_WHILE) => false,
            Some(SUB_UNTIL) => true,
            _ => return Err(self.error(unexpected.0, unexpected.1)),
        };
        // 929 is `Error_Invalid_expression_logical_list`, for the same reason the
        // `IF` arm passes it: `parseLogical` raises it itself as soon as a
        // sub-expression comes back null, so `requiredLogicalExpression`'s
        // per-caller `Error_Invalid_expression_while` (35.908) and
        // `Error_Invalid_expression_until` (35.909) are unreachable for every
        // caller. Measured: `do while` then `end` and `do until` then `end` both
        // give `Error 35.929` under `rexxc`, not 908 or 909.
        let condition = self.logical(Terminators::COND, 929)?;
        // `Error_Invalid_do_whileuntil`. Measured:
        // `do i = 1 to 3 while 1 until 2` is rc 229, Error 27.1.
        self.required_end(27, 1)?;
        Ok(Some(LoopConditional { until, condition }))
    }
}
