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

//! `ADDRESS` and the redirections of its `WITH` clause.

use crate::ast::{Address, AddressIo, OutputOption, Redirection};
use crate::expr::{Terminators, parse_constant_expression};
use crate::token::{ParseError, SymbolClass, Tag, TokenKind};

use super::{
    Inst, SUB_APPEND, SUB_ERROR, SUB_INPUT, SUB_NORMAL, SUB_OUTPUT, SUB_REPLACE, SUB_STEM,
    SUB_STREAM, SUB_USING, SUB_VALUE, SUB_WITH,
};

impl<'a> Inst<'a> {
    /// `addressNew` (`InstructionParser.cpp:563`).
    pub(super) fn address(&mut self) -> Result<Address, ParseError> {
        let mut address = Address::default();
        let Some(index) = self.peek_real_index() else {
            return Ok(address);
        };
        let token = &self.ctx.tokens[index];
        if !matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) {
            // An implicit `ADDRESS VALUE`, with the token left in place.
            // Measured: `address (e)` is rc 0.
            address.dynamic = self.opt_expr(Terminators::PARSE_WITH)?;
        } else if self.sub_keyword(token) == Some(SUB_VALUE) {
            self.seek(index + 1);
            // `Error_Invalid_expression_address`, measured as 35.914 for
            // `address value`.
            address.dynamic = Some(self.expr(Terminators::PARSE_WITH, 914)?);
        } else {
            self.seek(index + 1);
            address.environment = Some(self.value_of(token));
            if !self.at_end() {
                // The command expression stops at WITH, so what follows is
                // either that keyword or the end of the clause. This can come
                // back empty, which is `ADDRESS env WITH ...`: a configuration
                // with no command.
                address.command = self.opt_expr(Terminators::PARSE_WITH)?;
            }
        }
        let with = self
            .peek_real()
            .filter(|token| self.sub_keyword(token) == Some(SUB_WITH));
        if with.is_some() {
            self.next_real();
            address.io = Some(Box::new(self.address_with()?));
        }
        Ok(address)
    }

    /// `parseAddressWith` (`InstructionParser.cpp:670`): the `INPUT`, `OUTPUT`
    /// and `ERROR` redirections, each at most once and in any order.
    fn address_with(&mut self) -> Result<AddressIo, ParseError> {
        let mut io = AddressIo::default();
        // `Error_Symbol_expected_address_with`, measured as 20.933 for
        // `address system with`.
        if self.at_end() {
            return Err(self.error(20, 933));
        }
        let mut seen = [false; 3];
        while let Some(token) = self.next_real() {
            if token.kind.tag() != Tag::Symbol {
                return Err(self.error(20, 933));
            }
            // The three duplicate errors are 25.930, 25.931 and 25.932, one per
            // stream. Measured: `with input normal input normal` is 25.930.
            let stream = match self.sub_keyword(token) {
                Some(SUB_INPUT) => 0,
                Some(SUB_OUTPUT) => 1,
                Some(SUB_ERROR) => 2,
                // `Error_Invalid_subkeyword_address_with_option`, measured as
                // 25.934 for `address system with foo` and for `with 1`.
                _ => return Err(self.error(25, 934)),
            };
            if seen[stream] {
                return Err(self.error(25, 930 + u16::try_from(stream).expect("0, 1 or 2")));
            }
            seen[stream] = true;
            // `NORMAL` resets the stream and takes no target.
            let normal = self
                .peek_real()
                .filter(|token| self.sub_keyword(token) == Some(SUB_NORMAL));
            if normal.is_some() {
                self.next_real();
                match stream {
                    0 => io.input = Redirection::Normal,
                    1 => io.output = Redirection::Normal,
                    _ => io.error = Redirection::Normal,
                }
                continue;
            }
            // Only an output stream takes APPEND or REPLACE, and both are
            // optional.
            if stream != 0 {
                let option = self.output_option();
                if stream == 1 {
                    io.output_option = option;
                } else {
                    io.error_option = option;
                }
            }
            let target = self.redirect_target()?;
            match stream {
                0 => io.input = target,
                1 => io.output = target,
                _ => io.error = target,
            }
        }
        Ok(io)
    }

    /// `parseRedirectOutputOptions` (`InstructionParser.cpp:812`): `APPEND` or
    /// `REPLACE`, or neither, in which case nothing is consumed.
    fn output_option(&mut self) -> OutputOption {
        let Some(token) = self.peek_real() else {
            return OutputOption::Default;
        };
        let option = match self.sub_keyword(token) {
            Some(SUB_REPLACE) => OutputOption::Replace,
            Some(SUB_APPEND) => OutputOption::Append,
            // Probably one of the target keywords, so leave it in place.
            _ => return OutputOption::Default,
        };
        self.next_real();
        option
    }

    /// `parseRedirectOptions` (`InstructionParser.cpp:834`): where one stream
    /// goes.
    fn redirect_target(&mut self) -> Result<Redirection, ParseError> {
        // `Error_Invalid_subkeyword_address_with_io_option`, measured as 25.933
        // for `address system with input` and for `with input foo`.
        let Some(token) = self.next_real() else {
            return Err(self.error(25, 933));
        };
        if token.kind.tag() != Tag::Symbol {
            return Err(self.error(25, 933));
        }
        match self.sub_keyword(token) {
            Some(SUB_STEM) => {
                // `Error_Symbol_expected_after_stem_keyword`, measured as
                // 20.932 for `with input stem a`, because `a` is a variable and
                // not a stem.
                let Some(name) = self.next_real() else {
                    return Err(self.error(20, 932));
                };
                let TokenKind::Symbol {
                    id,
                    class: SymbolClass::Stem,
                } = name.kind
                else {
                    return Err(self.error(20, 932));
                };
                Ok(Redirection::Stem(id))
            }
            // Both take the constant-expression form, so a bare variable is
            // 35.1: measured, `with error using x` is rc 221 while
            // `with error using (x)` would be accepted.
            Some(index @ (SUB_STREAM | SUB_USING)) => {
                match parse_constant_expression(self.ctx, &mut self.cursor)? {
                    // `Error_Invalid_expression_missing_general`, measured as
                    // 35.935 for `with input stream`.
                    None => Err(self.error(35, 935)),
                    Some(value) => Ok(if index == SUB_STREAM {
                        Redirection::Stream(value)
                    } else {
                        Redirection::Using(value)
                    }),
                }
            }
            _ => Err(self.error(25, 933)),
        }
    }
}
