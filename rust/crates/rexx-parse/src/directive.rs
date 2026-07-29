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

//! The directive grammar: one `::` clause in, one directive out.
//!
//! Ported from `LanguageParser::nextDirective` (`DirectiveParser.cpp:64`) and
//! the nine functions it dispatches to.
//!
//! # Two tables, resolved by position
//!
//! The token after `::` resolves against `RexxToken::directives[]`, nine rows,
//! and every token after that against `RexxToken::subDirectives[]`, forty rows.
//! Five spellings are rows of both -- `ATTRIBUTE`, `CLASS`, `CONSTANT`,
//! `METHOD` and `ROUTINE` -- so `::CLASS c SUBCLASS d` uses `CLASS` at the top
//! level while `::METHOD m CLASS` uses it as an option, and only position tells
//! them apart. That is the same positional rule the instruction grammar uses,
//! and for the same reason: a directive keyword is not a reserved word.
//!
//! # A third table, at two points
//!
//! Two option arguments resolve against `subKeywords[]` instead, because the
//! C++ calls `token->subKeyword()` there: `::OPTIONS FORM` takes `SCIENTIFIC`
//! or `ENGINEERING` (`DirectiveParser.cpp:1007`) and `::OPTIONS NUMERIC` takes
//! `INHERIT` or `NOINHERIT` (`DirectiveParser.cpp:1339`). This is not a
//! detail that could be papered over with `subDirectives[]`: `NOINHERIT`,
//! `SCIENTIFIC` and `ENGINEERING` are rows of `subKeywords[]` alone, and
//! measured, `::OPTIONS NUMERIC NOINHERIT` and `::OPTIONS FORM ENGINEERING`
//! are both rc 0.
//!
//! # What this does not do
//!
//! A directive's *body* is not parsed here. `::METHOD` and `::ROUTINE` hand a
//! body to `translateBlock`, which is the task that assembles the instruction
//! chain; this module only decides whether a body belongs to the directive and
//! records that in the node. The one thing it does look at is the clause that
//! FOLLOWS, and only through `checkDirective` and `hasBody`, which need nothing
//! but that clause's first token.
//!
//! Nothing that needs the accumulated package is done here either, because this
//! function returns one directive and the accumulator is the caller's. That
//! leaves these to the caller, each measured:
//!
//! * The duplicate checks. `::CLASS c` twice is 99.901, `::ROUTINE r` twice is
//!   99.903, `::RESOURCE d` twice is 99.942, and the method, attribute and
//!   constant duplicates are 99.902, 99.931 and 99.932.
//! * 99.906, a `::CONSTANT` with an expression outside any `::CLASS`. Measured,
//!   `::CONSTANT c (1+2)` alone in a file is 99.906 and the same directive
//!   under a `::CLASS` is rc 0.
//! * 99.905, a `::METHOD ... CLASS` with no `::CLASS` above it.
//! * 99.945, a `::ANNOTATE` naming a target that does not exist yet.
//! * 33.1, a `::OPTIONS FUZZ` that is not less than the package's `DIGITS`.
//!   Measured, `::options fuzz 9` alone is 33.1 because the default `DIGITS` is
//!   9, and it is a `reportException` rather than a `syntaxError`.
//! * 98.903 and 90.998/90.999, resolving an `EXTERNAL` library or entry point.
//!   Those are run-time failures of a program that parsed: measured,
//!   `::METHOD m EXTERNAL "LIBRARY nosuch"` is rc 158, not a parse error.
//! * 99.916 for a non-directive clause where a directive was due, which is what
//!   the next call to `nextDirective` raises, and 99.915 for a directive inside
//!   `INTERPRET` text, which `translate` raises before any directive is parsed
//!   (`LanguageParser.cpp:762`).

use crate::ast::{
    Access, Annotate, Annotation, AnnotationTarget, AttributeDirective, AttributeStyle,
    ClassDirective, ClassRef, ConditionOption, ConstantDirective, ConstantValue, Directive,
    DirectiveKind, Expr, ExternalSpec, GuardOption, MethodDirective, OptionsForm, PackageOption,
    Protection, Requires, Resource, RoutineDirective,
};
use crate::clause::{Clause, ClauseCursor};
use crate::convert::{check_trace_setting, is_number, whole_number};
use crate::expr::{Terminators, parse_expr};
use crate::token::{
    Operator, ParseCtx, ParseError, SymbolClass, SymbolId, Tag, Token, TokenCursor, TokenKind,
};

// Positions in the `DIRECTIVES` table, which `KeywordSet::index_of` returns.
// An entry's position is its meaning, so these are indices and not spellings,
// and `tests::directive_indices_still_name_their_own_spellings` pins every one
// against the table.
const DIR_ANNOTATE: usize = 0;
const DIR_ATTRIBUTE: usize = 1;
const DIR_CLASS: usize = 2;
const DIR_CONSTANT: usize = 3;
const DIR_METHOD: usize = 4;
const DIR_OPTIONS: usize = 5;
const DIR_REQUIRES: usize = 6;
const DIR_RESOURCE: usize = 7;
const DIR_ROUTINE: usize = 8;

// Positions in the `SUB_DIRECTIVES` table, pinned the same way. All forty are
// named, and `tests::every_sub_directive_is_reachable` pairs each with a
// directive that accepts it.
const SUBDIR_ABSTRACT: usize = 0;
const SUBDIR_ALL: usize = 1;
const SUBDIR_ATTRIBUTE: usize = 2;
const SUBDIR_CLASS: usize = 3;
const SUBDIR_CONDITION: usize = 4;
const SUBDIR_CONSTANT: usize = 5;
const SUBDIR_DELEGATE: usize = 6;
const SUBDIR_DIGITS: usize = 7;
const SUBDIR_END: usize = 8;
const SUBDIR_ERROR: usize = 9;
const SUBDIR_EXTERNAL: usize = 10;
const SUBDIR_FAILURE: usize = 11;
const SUBDIR_FORM: usize = 12;
const SUBDIR_FUZZ: usize = 13;
const SUBDIR_GET: usize = 14;
const SUBDIR_GUARDED: usize = 15;
const SUBDIR_INHERIT: usize = 16;
const SUBDIR_LIBRARY: usize = 17;
const SUBDIR_LOSTDIGITS: usize = 18;
const SUBDIR_METACLASS: usize = 19;
const SUBDIR_METHOD: usize = 20;
const SUBDIR_MIXINCLASS: usize = 21;
const SUBDIR_NAMESPACE: usize = 22;
const SUBDIR_NOPROLOG: usize = 23;
const SUBDIR_NOSTRING: usize = 24;
const SUBDIR_NOTREADY: usize = 25;
const SUBDIR_NOVALUE: usize = 26;
const SUBDIR_NUMERIC: usize = 27;
const SUBDIR_PACKAGE: usize = 28;
const SUBDIR_PRIVATE: usize = 29;
const SUBDIR_PROLOG: usize = 30;
const SUBDIR_PROTECTED: usize = 31;
const SUBDIR_PUBLIC: usize = 32;
const SUBDIR_ROUTINE: usize = 33;
const SUBDIR_SET: usize = 34;
const SUBDIR_SUBCLASS: usize = 35;
const SUBDIR_SYNTAX: usize = 36;
const SUBDIR_TRACE: usize = 37;
const SUBDIR_UNGUARDED: usize = 38;
const SUBDIR_UNPROTECTED: usize = 39;

// Positions in the `SUB_KEYWORDS` table, for the two option arguments that
// resolve against it. `INHERIT` is a row of BOTH tables, at 22 here and at 16
// in `SUB_DIRECTIVES`, which is exactly why an index is never shared between
// them.
const SUBKEY_ENGINEERING: usize = 12;
const SUBKEY_INHERIT: usize = 22;
const SUBKEY_NOINHERIT: usize = 29;
const SUBKEY_SCIENTIFIC: usize = 37;

/// The precision `::OPTIONS DIGITS` and `::OPTIONS FUZZ` convert under.
///
/// `Numerics::ARGUMENT_DIGITS` (`Numerics.hpp:90`), 18 on a 64-bit build and 9
/// on a 32-bit one. The platform dependence is reproduced here, unlike the
/// scanner's `INTEGER_CONSTANT` flag, because this one is observable: measured
/// on this 64-bit build, `::options digits 123456789012345678` is rc 0 and
/// `1234567890123456789` is Error 26.5, so the boundary sits at eighteen.
const ARGUMENT_DIGITS: usize = 18;

/// The marker that ends a `::RESOURCE` body when the directive names none.
///
/// `GlobalNames::DEFAULT_RESOURCE_END`. Compared verbatim, so the case matters:
/// measured, a body closed by `::end` instead of `::END` is Error 99.943.
const DEFAULT_RESOURCE_END: &[u8] = b"::END";

/// Parses the `::` clause the cursor is sitting on, advancing it past that
/// clause.
///
/// This is `nextDirective` (`DirectiveParser.cpp:64`) including its own two
/// guards: a clause that does not start with `::` is 99.916 and a `::` not
/// followed by a symbol is 20.916, so a caller may hand this any clause.
///
/// Panics on an exhausted cursor, which is `noClauseAvailable()` and is the
/// caller's loop condition rather than an error.
#[allow(dead_code)] // deleted by Task 3.7b
pub(crate) fn parse_directive(
    ctx: &ParseCtx,
    cursor: &mut ClauseCursor,
) -> Result<Directive, ParseError> {
    let clause = cursor
        .next_clause()
        .expect("parse_directive on an exhausted cursor");
    let mut parser = Dir::new(ctx, clause);
    let kind = parser.dispatch(cursor)?;
    Ok(Directive {
        kind,
        clause_span: parser.clause.span.clone(),
    })
}

/// One directive clause's parse in progress.
///
/// Shaped like the instruction grammar's `Inst`, with one difference that
/// matters: the clause is already CONSUMED from the `ClauseCursor` by the time
/// this exists, because `checkDirective` and `hasBody` both look at the clause
/// that follows and could not see it otherwise.
struct Dir<'a> {
    ctx: &'a ParseCtx<'a>,
    /// Position inside `clause.tokens`.
    cursor: TokenCursor,
    clause: Clause,
    /// The byte offset every error is reported against: the start of the
    /// clause, not of the offending token.
    ///
    /// The exception is `checkDirective`, which reports against the FOLLOWING
    /// clause and builds its error without this field. See `check_directive`.
    clause_byte: usize,
}

impl<'a> Dir<'a> {
    fn new(ctx: &'a ParseCtx<'a>, clause: Clause) -> Self {
        let clause_byte = ctx
            .tokens
            .get(clause.tokens.start)
            .map_or(clause.span.start, |token| token.span.start);
        Dir {
            ctx,
            cursor: TokenCursor::new(clause.tokens.clone()),
            clause,
            clause_byte,
        }
    }

    fn error(&self, code: u16, sub: u16) -> ParseError {
        ParseError::new(code, sub, self.clause_byte)
    }

    /// `nextReal` without consuming.
    fn peek_real(&self) -> Option<&'a Token> {
        self.cursor
            .peek_real(self.ctx.tokens)
            .map(|i| &self.ctx.tokens[i])
    }

    /// `nextReal`: the next token that is not a blank, consumed.
    fn next_real(&mut self) -> Option<&'a Token> {
        self.cursor
            .advance_real(self.ctx.tokens)
            .map(|i| &self.ctx.tokens[i])
    }

    /// `isEndOfClause()` on the next real token.
    fn at_end(&self) -> bool {
        self.peek_real().is_none()
    }

    /// `requiredEndOfClause`: nothing may follow.
    fn required_end(&self, code: u16, sub: u16) -> Result<(), ParseError> {
        if self.at_end() {
            return Ok(());
        }
        Err(self.error(code, sub))
    }

    /// The `SUB_DIRECTIVES` index of `token`, or `None` when it is not a symbol
    /// or not in that table.
    ///
    /// Both misses are the same error at every call site, because the C++ tests
    /// `!token->isSymbol()` and the switch's `default` with one error code each
    /// time. Measured on `::CLASS`: `::class c,` and `::class c junk` are both
    /// 25.901.
    fn sub_directive(&self, token: &Token) -> Option<usize> {
        match &token.kind {
            TokenKind::Symbol { id, .. } => self.ctx.keywords.sub_directives.index_of(*id),
            _ => None,
        }
    }

    /// The `SUB_KEYWORDS` index of `token`, for the two option arguments that
    /// resolve against that table instead.
    fn sub_keyword(&self, token: &Token) -> Option<usize> {
        match &token.kind {
            TokenKind::Symbol { id, .. } => self.ctx.keywords.sub_keywords.index_of(*id),
            _ => None,
        }
    }

    /// `RexxToken::value()`: the upcased spelling of a symbol, or a literal's
    /// bytes verbatim.
    fn value_of(&self, token: &Token) -> Box<[u8]> {
        match &token.kind {
            TokenKind::Symbol { id, .. } => Box::from(self.ctx.symbols.name(*id).as_bytes()),
            TokenKind::Literal { value } => value.clone(),
            other => panic!("value_of on {other:?}"),
        }
    }

    /// `RexxToken::upperValue()`: like `value_of`, but a literal is upcased too.
    ///
    /// ASCII-only, which is exactly `SymbolTable::intern`'s rule and is safe for
    /// the same reason: a symbol cannot hold a non-ASCII byte at all. A literal
    /// can, and there this under-upcases the way `RexxString::upper` does.
    fn upper_value_of(&self, token: &Token) -> Box<[u8]> {
        let mut value = self.value_of(token).into_vec();
        value.make_ascii_uppercase();
        value.into_boxed_slice()
    }

    /// A required `isSymbolOrLiteral()` name, taken as written.
    fn require_name(&mut self, code: u16, sub: u16) -> Result<Box<[u8]>, ParseError> {
        let Some(token) = self.next_real() else {
            return Err(self.error(code, sub));
        };
        if !matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) {
            return Err(self.error(code, sub));
        }
        Ok(self.value_of(token))
    }

    /// A required `isSymbolOrLiteral()` name, upcased.
    fn require_upper_name(&mut self, code: u16, sub: u16) -> Result<Box<[u8]>, ParseError> {
        let Some(token) = self.next_real() else {
            return Err(self.error(code, sub));
        };
        if !matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) {
            return Err(self.error(code, sub));
        }
        Ok(self.upper_value_of(token))
    }

    /// A required `isSymbol()` token, yielding its interned id.
    fn require_symbol(&mut self, code: u16, sub: u16) -> Result<SymbolId, ParseError> {
        match self.next_real().map(|token| &token.kind) {
            Some(TokenKind::Symbol { id, .. }) => Ok(*id),
            _ => Err(self.error(code, sub)),
        }
    }

    /// A required `isLiteral()` token, yielding its decoded bytes.
    fn require_literal(&mut self, code: u16, sub: u16) -> Result<Box<[u8]>, ParseError> {
        match self.next_real().map(|token| &token.kind) {
            Some(TokenKind::Literal { value }) => Ok(value.clone()),
            _ => Err(self.error(code, sub)),
        }
    }

    // ---- what follows the directive ----

    /// `checkDirective` (`DirectiveParser.cpp:154`): if a clause follows, it
    /// must be a directive.
    ///
    /// Reported against the OFFENDING clause and not against the directive.
    /// `checkDirective` saves `clauseLocation` and restores it only AFTER the
    /// error, so `nextClause()` has already moved it: measured, `::method m
    /// abstract` on line 1 with `return 1` on line 2 reports
    /// `line 2: Translation error` with `99.933`.
    fn check_directive(
        &self,
        cursor: &ClauseCursor,
        code: u16,
        sub: u16,
    ) -> Result<(), ParseError> {
        let Some(next) = cursor.peek() else {
            return Ok(());
        };
        if self.ctx.tokens[next.tokens.start].kind.tag() == Tag::DColon {
            return Ok(());
        }
        Err(ParseError::new(code, sub, next.span.start))
    }

    /// `hasBody` (`DirectiveParser.cpp:189`): whether a non-directive clause
    /// follows.
    ///
    /// The one place a directive's parse depends on what comes after it rather
    /// than only rejecting it. `::ATTRIBUTE a GET` with a body is a method
    /// written in Rexx and without one is a generated getter, and both are rc 0.
    fn has_body(&self, cursor: &ClauseCursor) -> bool {
        cursor
            .peek()
            .is_some_and(|next| self.ctx.tokens[next.tokens.start].kind.tag() != Tag::DColon)
    }

    // ---- dispatch ----

    fn dispatch(&mut self, cursor: &mut ClauseCursor) -> Result<DirectiveKind, ParseError> {
        // `Error_Translation_bad_directive`. Measured on the other side of this
        // gate too: a lone `:` is not a directive at all but an expression, and
        // `:junk` is 35.1.
        match self.next_real() {
            Some(token) if token.kind.tag() == Tag::DColon => {}
            _ => return Err(self.error(99, 916)),
        }
        // `Error_Symbol_expected_directive`. Measured: `::` alone and `:: "x"`
        // are both 20.916.
        let Some(token) = self.next_real() else {
            return Err(self.error(20, 916));
        };
        let TokenKind::Symbol { id, .. } = token.kind else {
            return Err(self.error(20, 916));
        };
        // A symbol that is not a directive keyword is 99.916 and not 20.916:
        // measured, `::junk` and `:: 5` both report `Unrecognized directive
        // instruction`.
        let Some(index) = self.ctx.keywords.directives.index_of(id) else {
            return Err(self.error(99, 916));
        };
        match index {
            DIR_ANNOTATE => self.annotate(),
            DIR_ATTRIBUTE => self.attribute(cursor),
            DIR_CLASS => self.class(),
            DIR_CONSTANT => self.constant(cursor),
            DIR_METHOD => self.method(cursor),
            DIR_OPTIONS => self.options(),
            DIR_REQUIRES => self.requires(),
            DIR_RESOURCE => self.resource(),
            DIR_ROUTINE => self.routine(cursor),
            other => panic!("directive index {other} has no arm"),
        }
    }

    // ---- ::CLASS ----

    /// `classDirective` (`DirectiveParser.cpp:334`).
    fn class(&mut self) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(19, 901)?;
        let mut class = ClassDirective {
            name,
            access: Access::Default,
            abstract_: false,
            subclass: None,
            mixin: false,
            metaclass: None,
            inherit: Vec::new(),
        };
        while let Some(token) = self.next_real() {
            // Every rejection on this directive is 25.901, whether the token is
            // not a symbol, not a sub-directive, or a sub-directive already
            // given. Measured all three: `::class c,`, `::class c junk` and
            // `::class c abstract abstract`.
            let bad = self.error(25, 901);
            match self.sub_directive(token) {
                Some(SUBDIR_METACLASS) if class.metaclass.is_none() => {
                    class.metaclass = Some(self.class_reference(19, 906)?);
                }
                Some(SUBDIR_PUBLIC) if class.access == Access::Default => {
                    class.access = Access::Public;
                }
                Some(SUBDIR_PRIVATE) if class.access == Access::Default => {
                    class.access = Access::Private;
                }
                // `SUBCLASS` and `MIXINCLASS` fill one slot, because
                // `setMixinClass` sets the subclass too, so either one already
                // present rejects the other. Measured in both orders.
                Some(SUBDIR_SUBCLASS) if class.subclass.is_none() => {
                    class.subclass = Some(self.class_reference(19, 907)?);
                }
                Some(SUBDIR_MIXINCLASS) if class.subclass.is_none() => {
                    class.subclass = Some(self.class_reference(19, 913)?);
                    class.mixin = true;
                }
                Some(SUBDIR_INHERIT) => {
                    // `INHERIT` consumes every remaining token of the clause,
                    // and requires at least one: measured, `::class c inherit`
                    // is 19.908.
                    if self.at_end() {
                        return Err(self.error(19, 908));
                    }
                    while !self.at_end() {
                        class.inherit.push(self.class_reference(19, 908)?);
                    }
                }
                Some(SUBDIR_ABSTRACT) if !class.abstract_ => class.abstract_ = true,
                _ => return Err(bad),
            }
        }
        Ok(DirectiveKind::Class(Box::new(class)))
    }

    /// `parseClassReference` (`DirectiveParser.cpp:287`).
    ///
    /// `code`/`sub` is the caller's "nothing there" error, which differs per
    /// keyword: 19.906 for `METACLASS`, 19.907 for `SUBCLASS`, 19.913 for
    /// `MIXINCLASS` and 19.908 for `INHERIT`.
    fn class_reference(&mut self, code: u16, sub: u16) -> Result<ClassRef, ParseError> {
        let Some(token) = self.next_real() else {
            return Err(self.error(code, sub));
        };
        match &token.kind {
            // A literal is the whole reference: `parseClassReference` returns
            // before it can look for a colon, so `"rexx:object"` is one name
            // and not a qualified one.
            TokenKind::Literal { .. } => Ok(ClassRef {
                namespace: None,
                name: self.upper_value_of(token),
            }),
            TokenKind::Symbol { id, .. } => {
                let first = *id;
                let name = self.upper_value_of(token);
                // A `:` here makes this `ns:name`. Peeked rather than consumed,
                // because the C++ puts the token back when it is not a colon.
                if self.peek_real().map(|next| next.kind.tag()) != Some(Tag::Colon) {
                    return Ok(ClassRef {
                        namespace: None,
                        name,
                    });
                }
                self.next_real();
                // `Error_Symbol_expected_namespace_class`. Measured:
                // `::class c subclass rexx:` is 20.921.
                let qualified = self.require_symbol(20, 921)?;
                Ok(ClassRef {
                    namespace: Some(first),
                    name: Box::from(self.ctx.symbols.name(qualified).as_bytes()),
                })
            }
            _ => Err(self.error(code, sub)),
        }
    }

    // ---- ::METHOD and ::ATTRIBUTE ----

    /// `methodDirective` (`DirectiveParser.cpp:629`).
    fn method(&mut self, cursor: &mut ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(19, 902)?;
        let mut method = MethodDirective {
            name,
            class_method: false,
            attribute: false,
            abstract_: false,
            access: Access::Default,
            protection: Protection::Default,
            guard: GuardOption::Default,
            external: None,
            delegate: None,
            body: false,
        };
        // The EXTERNAL string is kept undecoded until the shape is known,
        // because the shapes decode it at different points and the error order
        // is observable. Measured: `::method m external "junk"` with a body is
        // 99.917 on line 1, while the same with ATTRIBUTE added is 99.934 on
        // line 2, so the attribute shape checks the body FIRST and the plain
        // external shape decodes first.
        let mut external: Option<Box<[u8]>> = None;
        while let Some(token) = self.next_real() {
            let bad = self.error(25, 902);
            match self.sub_directive(token) {
                Some(SUBDIR_CLASS) if !method.class_method => method.class_method = true,
                Some(SUBDIR_EXTERNAL)
                    if external.is_none() && method.delegate.is_none() && !method.abstract_ =>
                {
                    // `Error_Symbol_or_string_external`, and a symbol is not
                    // enough: measured, `::method m external nostring` is
                    // 19.905.
                    external = Some(self.require_literal(19, 905)?);
                }
                Some(SUBDIR_PRIVATE) if method.access == Access::Default => {
                    method.access = Access::Private;
                }
                Some(SUBDIR_PACKAGE) if method.access == Access::Default => {
                    method.access = Access::Package;
                }
                Some(SUBDIR_PUBLIC) if method.access == Access::Default => {
                    method.access = Access::Public;
                }
                Some(SUBDIR_PROTECTED) if method.protection == Protection::Default => {
                    method.protection = Protection::Protected;
                }
                Some(SUBDIR_UNPROTECTED) if method.protection == Protection::Default => {
                    method.protection = Protection::Unprotected;
                }
                Some(SUBDIR_UNGUARDED) if method.guard == GuardOption::Default => {
                    method.guard = GuardOption::Unguarded;
                }
                Some(SUBDIR_GUARDED) if method.guard == GuardOption::Default => {
                    method.guard = GuardOption::Guarded;
                }
                Some(SUBDIR_ATTRIBUTE) if !method.attribute => method.attribute = true,
                Some(SUBDIR_ABSTRACT)
                    if !method.abstract_ && external.is_none() && method.delegate.is_none() =>
                {
                    method.abstract_ = true;
                }
                Some(SUBDIR_DELEGATE)
                    if external.is_none() && method.delegate.is_none() && !method.abstract_ =>
                {
                    // `Error_Symbol_expected_delegate`. Measured:
                    // `::method m delegate "p"` is 20.926.
                    method.delegate = Some(self.require_symbol(20, 926)?);
                }
                _ => return Err(bad),
            }
        }

        // The shapes, in the C++'s own order, because they are not disjoint:
        // ATTRIBUTE combines with each of EXTERNAL and ABSTRACT, and DELEGATE
        // combines with ATTRIBUTE. Measured, `::method m delegate p attribute`
        // and `::method m attribute abstract` are both rc 0.
        if method.delegate.is_some() {
            self.check_directive(cursor, 99, 946)?;
        } else if method.attribute {
            self.check_directive(cursor, 99, 934)?;
            method.external = self.decode_external(external.as_deref(), false)?;
        } else if method.abstract_ {
            self.check_directive(cursor, 99, 933)?;
        } else if external.is_none() {
            method.body = true;
        } else {
            method.external = self.decode_external(external.as_deref(), false)?;
            self.check_directive(cursor, 99, 936)?;
        }
        Ok(DirectiveKind::Method(Box::new(method)))
    }

    /// `attributeDirective` (`DirectiveParser.cpp:1457`).
    fn attribute(&mut self, cursor: &mut ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(19, 914)?;
        let mut attribute = AttributeDirective {
            name,
            style: AttributeStyle::Both,
            class_method: false,
            abstract_: false,
            access: Access::Default,
            protection: Protection::Default,
            guard: GuardOption::Default,
            external: None,
            delegate: None,
            body: false,
        };
        let mut external: Option<Box<[u8]>> = None;
        while let Some(token) = self.next_real() {
            let bad = self.error(25, 925);
            match self.sub_directive(token) {
                // GET and SET share one slot, so a second of either is 25.925.
                // Measured: `::attribute a get set`.
                Some(SUBDIR_GET) if attribute.style == AttributeStyle::Both => {
                    attribute.style = AttributeStyle::Get;
                }
                Some(SUBDIR_SET) if attribute.style == AttributeStyle::Both => {
                    attribute.style = AttributeStyle::Set;
                }
                Some(SUBDIR_CLASS) if !attribute.class_method => attribute.class_method = true,
                Some(SUBDIR_PRIVATE) if attribute.access == Access::Default => {
                    attribute.access = Access::Private;
                }
                Some(SUBDIR_PUBLIC) if attribute.access == Access::Default => {
                    attribute.access = Access::Public;
                }
                Some(SUBDIR_PACKAGE) if attribute.access == Access::Default => {
                    attribute.access = Access::Package;
                }
                Some(SUBDIR_PROTECTED) if attribute.protection == Protection::Default => {
                    attribute.protection = Protection::Protected;
                }
                Some(SUBDIR_UNPROTECTED) if attribute.protection == Protection::Default => {
                    attribute.protection = Protection::Unprotected;
                }
                Some(SUBDIR_UNGUARDED) if attribute.guard == GuardOption::Default => {
                    attribute.guard = GuardOption::Unguarded;
                }
                Some(SUBDIR_GUARDED) if attribute.guard == GuardOption::Default => {
                    attribute.guard = GuardOption::Guarded;
                }
                Some(SUBDIR_EXTERNAL)
                    if external.is_none()
                        && attribute.delegate.is_none()
                        && !attribute.abstract_ =>
                {
                    external = Some(self.require_literal(19, 905)?);
                }
                Some(SUBDIR_ABSTRACT)
                    if !attribute.abstract_
                        && external.is_none()
                        && attribute.delegate.is_none() =>
                {
                    attribute.abstract_ = true;
                }
                Some(SUBDIR_DELEGATE)
                    if external.is_none()
                        && attribute.delegate.is_none()
                        && !attribute.abstract_ =>
                {
                    attribute.delegate = Some(self.require_symbol(20, 926)?);
                }
                _ => return Err(bad),
            }
        }

        // Every shape here checks the body BEFORE decoding the external
        // string, unlike `::METHOD`'s plain external shape. Measured:
        // `::attribute a get external "junk"` with a body is 99.935 on line 2,
        // where the same on `::method` is 99.917 on line 1.
        match attribute.style {
            // Both methods are generated, so a body can never belong here.
            AttributeStyle::Both => {
                self.check_directive(cursor, 99, 937)?;
            }
            AttributeStyle::Get | AttributeStyle::Set => {
                if external.is_some() {
                    self.check_directive(cursor, 99, 935)?;
                } else if attribute.abstract_ {
                    self.check_directive(cursor, 99, 940)?;
                } else if attribute.delegate.is_some() {
                    self.check_directive(cursor, 99, 947)?;
                } else {
                    attribute.body = self.has_body(cursor);
                }
            }
        }
        attribute.external = self.decode_external(external.as_deref(), false)?;
        Ok(DirectiveKind::Attribute(Box::new(attribute)))
    }

    /// `decodeExternalMethod` (`DirectiveParser.cpp:1403`) and the routine
    /// form (`DirectiveParser.cpp:2649`-`2749`).
    ///
    /// `registered` admits the `REGISTERED` spelling, which only `::ROUTINE`
    /// accepts. Measured: `::method m external "junk"` and
    /// `::routine r external "junk"` are both 99.917, and
    /// `::routine r external "registered x"` gets past the parse to 90.999
    /// while there is no method spelling that does.
    fn decode_external(
        &self,
        spec: Option<&[u8]>,
        registered: bool,
    ) -> Result<Option<ExternalSpec>, ParseError> {
        let Some(spec) = spec else {
            return Ok(None);
        };
        // `words()` splits on blanks and upcases the FIRST word only, which is
        // why the library name keeps its case. Measured: tabs separate words
        // too, and `"  library   x  "` resolves the library `x`.
        let words: Vec<&[u8]> = spec
            .split(|&byte| byte == b' ' || byte == b'\t')
            .filter(|word| !word.is_empty())
            .collect();
        // `Error_Translation_bad_external`. Measured for the empty string, for
        // one word, for four words and for a first word that is neither
        // keyword.
        let bad = self.error(99, 917);
        let Some((&keyword, rest)) = words.split_first() else {
            return Err(bad);
        };
        let mut keyword = keyword.to_vec();
        keyword.make_ascii_uppercase();
        let is_registered = registered && keyword == b"REGISTERED";
        if keyword != b"LIBRARY" && !is_registered {
            return Err(bad);
        }
        let (library, entry) = match rest {
            [library] => (*library, None),
            [library, entry] => (*library, Some(Box::from(*entry))),
            _ => return Err(bad),
        };
        Ok(Some(ExternalSpec {
            registered: is_registered,
            library: Box::from(library),
            entry,
        }))
    }

    // ---- ::CONSTANT ----

    /// `constantDirective` (`DirectiveParser.cpp:1854`).
    fn constant(&mut self, cursor: &mut ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(19, 915)?;
        let value = match self.peek_real() {
            // No value at all, whose value is the name as written. Measured:
            // `::constant c` is rc 0.
            None => ConstantValue::Name,
            Some(token) if token.kind.tag() == Tag::LeftParen => {
                self.next_real();
                ConstantValue::Expression(self.constant_expression()?)
            }
            Some(token) if matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) => {
                self.next_real();
                ConstantValue::Text(self.value_of(token))
            }
            Some(_) => ConstantValue::Text(self.signed_constant(19, 916)?),
        };
        // `Error_Invalid_data_constant_dir`. Measured: `::constant c 5 6` is
        // 21.913.
        self.required_end(21, 913)?;
        // `Error_Translation_constant_body`. Measured: 99.938 on line 2.
        self.check_directive(cursor, 99, 938)?;
        Ok(DirectiveKind::Constant(Box::new(ConstantDirective {
            name,
            value,
        })))
    }

    /// `translateConstantExpression` (`LanguageParser.cpp:1725`), entered with
    /// the `(` already consumed.
    ///
    /// A comma list is allowed, because this is `requiredExpression(TERM_RIGHT)`
    /// and a required expression admits one: measured, `::constant c (1,2)`
    /// gets past the parse to 99.906, the same error `(1+2)` gets.
    fn constant_expression(&mut self) -> Result<Expr, ParseError> {
        // `Error_Invalid_expression_missing_constant`. Measured:
        // `::constant c ()` is 35.936.
        let expr = parse_expr(self.ctx, &mut self.cursor, Terminators::RIGHT, 936)?;
        // `Error_Unmatched_parenthesis_paren`. Measured: `::constant c (1+2`
        // is 36.901. No blank can sit before a `)`, so `nextToken` and
        // `nextReal` agree here.
        match self.next_real() {
            Some(token) if token.kind.tag() == Tag::RightParen => Ok(expr),
            _ => Err(self.error(36, 901)),
        }
    }

    /// The signed-number value form that `::CONSTANT` and `::ANNOTATE` share
    /// (`DirectiveParser.cpp:1886`-`1907` and `2230`-`2251`).
    ///
    /// A `+` or `-` followed by a CONSTANT symbol whose concatenation is a
    /// number. All three conditions are separate: measured, `::constant c *5`
    /// fails on the operator, `-.true` on the symbol's class, and `-5x` and
    /// `-1e` on the number test, and all four report the same error.
    fn signed_constant(&mut self, code: u16, sub: u16) -> Result<Box<[u8]>, ParseError> {
        let Some(token) = self.next_real() else {
            return Err(self.error(code, sub));
        };
        let sign: &[u8] = match token.kind {
            TokenKind::Operator(Operator::Plus) => b"+",
            TokenKind::Operator(Operator::Subtract) => b"-",
            _ => return Err(self.error(code, sub)),
        };
        let Some(second) = self.next_real() else {
            return Err(self.error(code, sub));
        };
        let TokenKind::Symbol {
            class: SymbolClass::Constant,
            ..
        } = second.kind
        else {
            return Err(self.error(code, sub));
        };
        let mut value = sign.to_vec();
        value.extend_from_slice(&self.value_of(second));
        if !is_number(&value) {
            return Err(self.error(code, sub));
        }
        Ok(value.into_boxed_slice())
    }

    // ---- ::ANNOTATE ----

    /// `annotateDirective` (`DirectiveParser.cpp:1940`).
    fn annotate(&mut self) -> Result<DirectiveKind, ParseError> {
        // `Error_Symbol_expected_annotation_type`, and a literal is not enough:
        // measured, `::annotate "package"` is 20.924.
        let Some(token) = self.next_real() else {
            return Err(self.error(20, 924));
        };
        let bad = self.error(25, 928);
        // `Error_Symbol_or_string_directive_option`, one error for every named
        // target. Measured: `::annotate class` is 19.925.
        let target = match self.sub_directive(token) {
            Some(SUBDIR_PACKAGE) => AnnotationTarget::Package,
            Some(SUBDIR_CLASS) => AnnotationTarget::Class(self.require_upper_name(19, 925)?),
            Some(SUBDIR_ROUTINE) => AnnotationTarget::Routine(self.require_upper_name(19, 925)?),
            Some(SUBDIR_METHOD) => AnnotationTarget::Method(self.require_upper_name(19, 925)?),
            Some(SUBDIR_ATTRIBUTE) => {
                AnnotationTarget::Attribute(self.require_upper_name(19, 925)?)
            }
            Some(SUBDIR_CONSTANT) => AnnotationTarget::Constant(self.require_upper_name(19, 925)?),
            // A non-symbol reaches 20.924 above, so this is a symbol that is
            // not one of the six. Measured: `::annotate junk k 1` is 25.928.
            _ => {
                if token.kind.tag() != Tag::Symbol {
                    return Err(self.error(20, 924));
                }
                return Err(bad);
            }
        };
        let mut annotations = Vec::new();
        while !self.at_end() {
            annotations.push(self.annotation()?);
        }
        Ok(DirectiveKind::Annotate(Box::new(Annotate {
            target,
            annotations,
        })))
    }

    /// `processAnnotation` (`DirectiveParser.cpp:2209`): one `name value` pair.
    fn annotation(&mut self) -> Result<Annotation, ParseError> {
        // `Error_Symbol_expected_annotation_attribute`. Measured:
        // `::annotate package "a" 1` is 20.919.
        let name = self.require_symbol(20, 919)?;
        let value = match self.peek_real() {
            // `Error_Symbol_or_string_package_attribute_missing`, a DIFFERENT
            // number from the bad-value one. Measured: `::annotate package a`
            // is 19.924 and `::annotate package a *` is 19.923.
            None => return Err(self.error(19, 924)),
            Some(token) if matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) => {
                self.next_real();
                self.value_of(token)
            }
            // No parenthesised form here, unlike `::CONSTANT`.
            Some(_) => self.signed_constant(19, 923)?,
        };
        Ok(Annotation { name, value })
    }

    // ---- ::OPTIONS ----

    /// `optionsDirective` (`DirectiveParser.cpp:948`).
    fn options(&mut self) -> Result<DirectiveKind, ParseError> {
        let mut options = Vec::new();
        while let Some(token) = self.next_real() {
            // `Error_Invalid_subkeyword_options`. Measured: `::options junk`
            // and `::options "digits" 9` are both 25.924.
            let bad = self.error(25, 924);
            match self.sub_directive(token) {
                Some(SUBDIR_DIGITS) => {
                    // `Error_Symbol_or_string_digits_value`, then
                    // `Error_Invalid_whole_number_digits`. Measured:
                    // `::options digits -1` is 19.917 because a `-` is neither
                    // a symbol nor a literal, and `::options digits 0` is 26.5
                    // because the value must exceed zero.
                    let value = self.require_name(19, 917)?;
                    let digits = whole_number(&value, ARGUMENT_DIGITS)
                        .filter(|&digits| digits >= 1)
                        .and_then(|digits| usize::try_from(digits).ok())
                        .ok_or_else(|| self.error(26, 5))?;
                    options.push(PackageOption::Digits(digits));
                }
                Some(SUBDIR_FORM) => {
                    // `Error_Symbol_expected_form`, then
                    // `Error_Invalid_subkeyword_form`. This argument resolves
                    // against `subKeywords[]`: measured, `::options form value`
                    // is 25.11 where `VALUE` is a row of that table, and
                    // `::options form "scientific"` is 20.925.
                    let Some(token) = self.next_real() else {
                        return Err(self.error(20, 925));
                    };
                    if token.kind.tag() != Tag::Symbol {
                        return Err(self.error(20, 925));
                    }
                    let form = match self.sub_keyword(token) {
                        Some(SUBKEY_SCIENTIFIC) => OptionsForm::Scientific,
                        Some(SUBKEY_ENGINEERING) => OptionsForm::Engineering,
                        _ => return Err(self.error(25, 11)),
                    };
                    options.push(PackageOption::Form(form));
                }
                Some(SUBDIR_FUZZ) => {
                    // `Error_Symbol_or_string_fuzz_value`, then
                    // `Error_Invalid_whole_number_fuzz`. Zero is legal here
                    // where it is not for DIGITS: measured, `::options fuzz 0`
                    // is rc 0 and `::options fuzz "-1"` is 26.6.
                    let value = self.require_name(19, 918)?;
                    // `try_from` also rejects a negative, which is what
                    // `requestUnsignedNumber` does and is why FUZZ has no
                    // lower-bound filter of its own.
                    let fuzz = whole_number(&value, ARGUMENT_DIGITS)
                        .and_then(|fuzz| usize::try_from(fuzz).ok())
                        .ok_or_else(|| self.error(26, 6))?;
                    options.push(PackageOption::Fuzz(fuzz));
                }
                Some(SUBDIR_TRACE) => {
                    // `Error_Symbol_or_string_trace_value`, then
                    // `Error_Invalid_trace_trace`. Measured: `::options trace
                    // zzz` is 24.1 and `::options trace r` is rc 0.
                    let value = self.require_name(19, 919)?;
                    check_trace_setting(&value).map_err(|()| self.error(24, 1))?;
                    options.push(PackageOption::Trace(value));
                }
                // The seven condition options, each taking SYNTAX or
                // CONDITION. NOVALUE additionally accepts ERROR, which the C++
                // marks as backwards compatibility, and nothing else does:
                // measured, `::options novalue error` is rc 0 while
                // `::options error error` and `::options all error` are 25.927.
                Some(SUBDIR_ALL) => options.push(self.condition_option(ConditionOption::All)?),
                Some(SUBDIR_ERROR) => options.push(self.condition_option(ConditionOption::Error)?),
                Some(SUBDIR_FAILURE) => {
                    options.push(self.condition_option(ConditionOption::Failure)?);
                }
                Some(SUBDIR_LOSTDIGITS) => {
                    options.push(self.condition_option(ConditionOption::LostDigits)?);
                }
                Some(SUBDIR_NOSTRING) => {
                    options.push(self.condition_option(ConditionOption::NoString)?);
                }
                Some(SUBDIR_NOTREADY) => {
                    options.push(self.condition_option(ConditionOption::NotReady)?);
                }
                Some(SUBDIR_NOVALUE) => {
                    options.push(self.condition_option(ConditionOption::NoValue)?);
                }
                Some(SUBDIR_NOPROLOG) => options.push(PackageOption::Prolog(false)),
                Some(SUBDIR_PROLOG) => options.push(PackageOption::Prolog(true)),
                Some(SUBDIR_NUMERIC) => {
                    // The second argument that resolves against
                    // `subKeywords[]`. Measured: `::options numeric noinherit`
                    // is rc 0 even though NOINHERIT is not a sub-directive at
                    // all, and `::options numeric syntax` is 25.935 even
                    // though SYNTAX is.
                    let Some(token) = self.next_real() else {
                        return Err(self.error(20, 935));
                    };
                    if token.kind.tag() != Tag::Symbol {
                        return Err(self.error(20, 935));
                    }
                    let inherit = match self.sub_keyword(token) {
                        Some(SUBKEY_INHERIT) => true,
                        Some(SUBKEY_NOINHERIT) => false,
                        _ => return Err(self.error(25, 935)),
                    };
                    options.push(PackageOption::NumericInherit(inherit));
                }
                _ => return Err(bad),
            }
        }
        Ok(DirectiveKind::Options(options))
    }

    /// One of the seven condition options and its `SYNTAX`/`CONDITION`
    /// argument.
    fn condition_option(&mut self, which: ConditionOption) -> Result<PackageOption, ParseError> {
        // `Error_Symbol_expected_after_keyword`, the same number for all seven.
        // Measured: `::options novalue` with nothing after it and
        // `::options novalue "syntax"` are both 20.929.
        let Some(token) = self.next_real() else {
            return Err(self.error(20, 929));
        };
        if token.kind.tag() != Tag::Symbol {
            return Err(self.error(20, 929));
        }
        let syntax = match self.sub_directive(token) {
            Some(SUBDIR_SYNTAX) => true,
            Some(SUBDIR_CONDITION) => false,
            // `SUBDIRECTIVE_ERROR` falls through to the SYNTAX arm for NOVALUE
            // alone (`DirectiveParser.cpp:1091`).
            Some(SUBDIR_ERROR) if which == ConditionOption::NoValue => true,
            // `Error_Invalid_subkeyword_following`.
            _ => return Err(self.error(25, 927)),
        };
        Ok(PackageOption::Condition { which, syntax })
    }

    // ---- ::REQUIRES ----

    /// `requiresDirective` (`DirectiveParser.cpp:2779`).
    fn requires(&mut self) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(19, 904)?;
        let mut requires = Requires {
            name,
            library: false,
            namespace: None,
        };
        while let Some(token) = self.next_real() {
            // `Error_Invalid_subkeyword_requires`. LIBRARY and NAMESPACE are
            // mutually exclusive as well as each being once-only, and both
            // rejections are this error: measured in both orders and doubled.
            let bad = self.error(25, 904);
            let taken = requires.library || requires.namespace.is_some();
            match self.sub_directive(token) {
                Some(SUBDIR_NAMESPACE) if !taken => {
                    // `Error_Symbol_expected_namespace`. Measured:
                    // `::requires "x" namespace "ns"` is 20.920.
                    let namespace = self.require_symbol(20, 920)?;
                    // `Error_Translation_reserved_namespace`. The symbol is
                    // already upcased, so this catches every spelling of REXX.
                    if self.ctx.symbols.name(namespace) == "REXX" {
                        return Err(self.error(99, 944));
                    }
                    requires.namespace = Some(namespace);
                }
                Some(SUBDIR_LIBRARY) if !taken => requires.library = true,
                _ => return Err(bad),
            }
        }
        Ok(DirectiveKind::Requires(Box::new(requires)))
    }

    // ---- ::RESOURCE ----

    /// `resourceDirective` (`DirectiveParser.cpp:2266`).
    ///
    /// The body itself was copied out by `scan`, which had to: the lines are not
    /// Rexx and tokenising them would invent errors the interpreter does not
    /// raise. Measured, a body holding `this is 'unmatched and /* unclosed` gets
    /// rc 0. So this validates the directive and picks up the body `scan`
    /// already found.
    fn resource(&mut self) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(19, 920)?;
        let mut end_marker: Box<[u8]> = Box::from(DEFAULT_RESOURCE_END);
        if !self.at_end() {
            // `Error_Invalid_subkeyword_resource`, for a non-symbol and for
            // any sub-directive but END. Measured: `::resource data junk` is
            // 25.926.
            let token = self.next_real().expect("at_end was false");
            if self.sub_directive(token) != Some(SUBDIR_END) {
                return Err(self.error(25, 926));
            }
            // `Error_Symbol_or_string_resource_end`, and the marker is UPCASED
            // when it comes from a symbol because that is what
            // `RexxToken::value()` gives. Measured: `::resource data end stop`
            // is closed by `STOP` and not by `stop`.
            end_marker = self.require_name(19, 921)?;
            // `Error_Invalid_data_resource_dir`. Measured:
            // `::resource data end "x" extra` is 21.914.
            self.required_end(21, 914)?;
        }

        // Every shape `scan` copies a body for is a shape that reaches here
        // with no error, and every shape it skips raises one of the four errors
        // above, so a body is always present by now. `scan` also owns the
        // missing-marker error, 99.943, which fails the whole scan.
        let body = self
            .ctx
            .resources
            .iter()
            .find(|body| body.directive == self.clause.tokens.start)
            .expect("scan copies a body for every well-formed ::RESOURCE");
        Ok(DirectiveKind::Resource(Box::new(Resource {
            name,
            end_marker,
            lines: body.lines.clone(),
        })))
    }

    // ---- ::ROUTINE ----

    /// `routineDirective` (`DirectiveParser.cpp:2565`).
    fn routine(&mut self, cursor: &mut ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(19, 903)?;
        let mut routine = RoutineDirective {
            name,
            access: Access::Default,
            external: None,
            body: false,
        };
        let mut external: Option<Box<[u8]>> = None;
        while let Some(token) = self.next_real() {
            let bad = self.error(25, 903);
            match self.sub_directive(token) {
                Some(SUBDIR_EXTERNAL) => {
                    // A SECOND external is 25.901, the ::CLASS number, because
                    // `routineDirective` passes `Error_Invalid_subkeyword_class`
                    // there (`DirectiveParser.cpp:2606`). Measured:
                    // `::routine r external "LIBRARY x" external "LIBRARY y"`
                    // reports `Unknown keyword on ::CLASS directive`. The
                    // interpreter defines the behaviour, so the oddity is
                    // reproduced rather than corrected.
                    if external.is_some() {
                        return Err(self.error(25, 901));
                    }
                    external = Some(self.require_literal(19, 905)?);
                }
                Some(SUBDIR_PUBLIC) if routine.access == Access::Default => {
                    routine.access = Access::Public;
                }
                Some(SUBDIR_PRIVATE) if routine.access == Access::Default => {
                    routine.access = Access::Private;
                }
                _ => return Err(bad),
            }
        }
        match external {
            // `REGISTERED` is accepted here and nowhere else.
            Some(spec) => {
                routine.external = self.decode_external(Some(&spec), true)?;
                // `Error_Translation_external_routine`.
                self.check_directive(cursor, 99, 939)?;
            }
            None => routine.body = true,
        }
        Ok(DirectiveKind::Routine(Box::new(routine)))
    }
}

#[cfg(test)]
mod tests;
