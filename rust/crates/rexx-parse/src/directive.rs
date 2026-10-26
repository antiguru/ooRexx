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

use rexx_inventory::errors::{
    E19_901, E19_902, E19_903, E19_904, E19_905, E19_906, E19_907, E19_908, E19_913, E19_914,
    E19_915, E19_916, E19_917, E19_918, E19_919, E19_920, E19_921, E19_923, E19_924, E19_925,
    E20_916, E20_919, E20_920, E20_921, E20_924, E20_925, E20_926, E20_929, E20_935, E21_913,
    E21_914, E24_1, E25_11, E25_901, E25_902, E25_903, E25_904, E25_924, E25_925, E25_926, E25_927,
    E25_928, E25_935, E26_5, E26_6, E33_1, E35_936, E36_901, E99_901, E99_902, E99_903, E99_905,
    E99_906, E99_916, E99_917, E99_925, E99_931, E99_932, E99_933, E99_934, E99_935, E99_936,
    E99_937, E99_938, E99_939, E99_940, E99_942, E99_943, E99_944, E99_945, E99_946, E99_947, Msg,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::ast::{
    Access, Annotate, Annotation, AnnotationTarget, AttributeDirective, AttributeStyle,
    ClassDirective, ClassRef, CodeBody, ConditionOption, ConstantDirective, ConstantValue,
    Directive, DirectiveKind, Expr, ExternalSpec, GuardOption, MethodDirective, OptionsForm,
    PackageOption, Protection, Requires, Resource, RoutineDirective,
};
use crate::clause::{Clause, ClauseCursor};
use crate::convert::{ARGUMENT_DIGITS, check_trace_setting, is_number, whole_number};
use crate::expr::{Terminators, display_value, parse_expr};
use crate::scanner::{MAX_SYMBOL_LENGTH, is_symbol_char};
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

/// The marker that ends a `::RESOURCE` body when the directive names none.
const DEFAULT_RESOURCE_END: &[u8] = b"::END";

/// What the directives parsed so far declare, which a later directive is
/// checked against while it is parsed: `LanguageParser`'s `activeClass`,
/// `classDependencies`, `routines`, `resources`, the member tables and the
/// package's numeric settings.
pub(crate) struct Declared {
    /// Whether a `::CLASS` directive came before (`activeClass`,
    /// `DirectiveParser.cpp:355`).
    in_class: bool,
    classes: FxHashSet<Box<[u8]>>,
    routines: FxHashSet<Box<[u8]>>,
    resources: FxHashSet<Box<[u8]>>,
    /// The active class's methods by side (`true` for the class side) and
    /// upcased name, or before any `::CLASS` the file's `unattachedMethods`.
    members: FxHashMap<(bool, Box<[u8]>), Member>,
    digits: usize,
    fuzz: usize,
    /// The directives whose `EXTERNAL` the oracle resolves while translating
    /// (`createNativeMethod`, `PackageManager::resolveRoutine`), in the order
    /// the parse reached that point.
    pub(crate) binds: Vec<Directive>,
}

/// What `::ANNOTATE` asks of a method it names: `isConstant()` and
/// `isAttribute()`.
#[derive(Copy, Clone)]
struct Member {
    constant: bool,
    attribute: bool,
}

impl Default for Declared {
    fn default() -> Self {
        Declared {
            in_class: false,
            classes: FxHashSet::default(),
            routines: FxHashSet::default(),
            resources: FxHashSet::default(),
            members: FxHashMap::default(),
            // `Numerics::DEFAULT_DIGITS` and a fuzz of zero.
            digits: 9,
            fuzz: 0,
            binds: Vec::new(),
        }
    }
}

/// Parses the `::` clause the cursor is sitting on, advancing it past that
/// clause, and records what it declares in `declared`.
pub(crate) fn parse_directive(
    ctx: &ParseCtx,
    cursor: &mut ClauseCursor,
    declared: &mut Declared,
) -> Result<Directive, ParseError> {
    let clause = cursor
        .next_clause()
        .expect("parse_directive on an exhausted cursor");
    let mut parser = Dir::new(ctx, clause, declared);
    let kind = parser.dispatch(cursor)?;
    // A body check that read past the last clause has met a scanner error
    // there first, if there is one, before the oracle resolves anything.
    let bound = parser.bound && !cursor.read_past_end();
    let directive = Directive {
        kind,
        clause_span: parser.clause.span.clone(),
    };
    if bound {
        declared.binds.push(directive.clone());
    }
    Ok(directive)
}

/// One directive clause's parse in progress.
struct Dir<'a, 'd> {
    ctx: &'a ParseCtx<'a>,
    /// Position inside `clause.tokens`.
    cursor: TokenCursor,
    clause: Clause,
    /// The byte offset every error is reported against: the start of the
    /// clause, not of the offending token.
    clause_byte: usize,
    declared: &'d mut Declared,
    /// Whether the parse reached the point where the oracle resolves this
    /// directive's `EXTERNAL`.
    bound: bool,
}

impl<'a, 'd> Dir<'a, 'd> {
    fn new(ctx: &'a ParseCtx<'a>, clause: Clause, declared: &'d mut Declared) -> Self {
        let clause_byte = ctx
            .tokens
            .get(clause.tokens.start)
            .map_or(clause.span.start, |token| token.span.start);
        Dir {
            ctx,
            cursor: TokenCursor::new(clause.tokens.clone()),
            clause,
            clause_byte,
            declared,
            bound: false,
        }
    }

    /// The CLASS-keyword half of `checkDuplicateMethod`
    /// (`DirectiveParser.cpp:510`-`516`): a class method needs a `::CLASS`
    /// above it.
    fn require_class(&self, class_method: bool) -> Result<(), ParseError> {
        if class_method && !self.declared.in_class {
            return Err(self.error(E99_905, []));
        }
        Ok(())
    }

    /// The duplicate half of `checkDuplicateMethod`
    /// (`DirectiveParser.cpp:518`-`529`): `msg` if the active class, or the
    /// unattached table, already holds `name` on that side, which `name` then
    /// claims.
    fn claim(
        &mut self,
        name: Box<[u8]>,
        class_side: bool,
        member: Member,
        msg: Msg<0>,
    ) -> Result<(), ParseError> {
        match self.declared.members.entry((class_side, name)) {
            std::collections::hash_map::Entry::Occupied(_) => Err(self.error(msg, [])),
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(member);
                Ok(())
            }
        }
    }

    /// `findMethod`: the active class's instance method `name`, else its class
    /// method.
    fn find_member(&self, name: &[u8]) -> Option<Member> {
        [false, true]
            .into_iter()
            .find_map(|side| self.declared.members.get(&(side, Box::from(name))).copied())
    }

    /// `msg` reported against this clause, with `inserts` for its `&1`,
    /// `&2`, ...
    #[cold]
    #[inline(never)]
    fn error<const N: usize>(&self, msg: Msg<N>, inserts: [Vec<u8>; N]) -> ParseError {
        ParseError::raise(msg, self.clause_byte, inserts)
    }

    /// `msg` naming `token` as `RexxToken::displayValue` shows it, `None`
    /// being the end of the clause.
    #[cold]
    #[inline(never)]
    fn found(&self, msg: Msg<1>, token: Option<&Token>) -> ParseError {
        self.error(msg, [display_value(self.ctx, token)])
    }

    /// 36.901 for the `(` `opener` with no `)`: `errorPosition`
    /// (`LanguageParser.cpp:4114`) inserts its 1-based offset in its line,
    /// then the line.
    #[cold]
    #[inline(never)]
    fn unmatched_paren(&self, opener: &Token) -> ParseError {
        let source = self.ctx.source;
        let line = source.line_of(opener.span.start);
        let line_start = source.line_span(line).map_or(0, |span| span.start);
        let position = opener.span.start - line_start + 1;
        self.error(
            E36_901,
            [
                position.to_string().into_bytes(),
                source.message_line(line).to_string().into_bytes(),
            ],
        )
    }

    /// 24.1 for a `TRACE` value `check_trace_setting` refused, naming its first
    /// byte that is not `?` (`TraceSetting::parseTraceSetting`'s `badOption`).
    #[cold]
    #[inline(never)]
    fn bad_trace_setting(&self, value: &[u8]) -> ParseError {
        let bad = value.iter().copied().find(|&byte| byte != b'?');
        self.error(E24_1, [bad.into_iter().collect()])
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

    /// `requiredEndOfClause`: nothing may follow, and `msg` names what does.
    fn required_end(&self, msg: Msg<1>) -> Result<(), ParseError> {
        match self.peek_real() {
            None => Ok(()),
            token => Err(self.found(msg, token)),
        }
    }

    /// The `SUB_DIRECTIVES` index of `token`, or `None` when it is not a symbol
    /// or not in that table.
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
    fn upper_value_of(&self, token: &Token) -> Box<[u8]> {
        let mut value = self.value_of(token).into_vec();
        value.make_ascii_uppercase();
        value.into_boxed_slice()
    }

    /// The next real token when it is a symbol or a literal, or the token
    /// found instead.
    fn next_name(&mut self) -> Result<&'a Token, Option<&'a Token>> {
        match self.next_real() {
            Some(token) if matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) => Ok(token),
            found => Err(found),
        }
    }

    /// A required `isSymbolOrLiteral()` name, taken as written, `msg` naming
    /// the token found instead.
    fn require_name(&mut self, msg: Msg<1>) -> Result<Box<[u8]>, ParseError> {
        match self.next_name() {
            Ok(token) => Ok(self.value_of(token)),
            Err(found) => Err(self.found(msg, found)),
        }
    }

    /// The next real token's interned id when it is a symbol, or the token
    /// found instead.
    fn next_symbol(&mut self) -> Result<SymbolId, Option<&'a Token>> {
        match self.next_real() {
            Some(Token {
                kind: TokenKind::Symbol { id, .. },
                ..
            }) => Ok(*id),
            found => Err(found),
        }
    }

    /// A required `isLiteral()` token, yielding its decoded bytes, `msg`
    /// naming the token found instead.
    fn require_literal(&mut self, msg: Msg<1>) -> Result<Box<[u8]>, ParseError> {
        match self.next_real() {
            Some(Token {
                kind: TokenKind::Literal { value },
                ..
            }) => Ok(value.clone()),
            found => Err(self.found(msg, found)),
        }
    }

    /// `getRetriever` (`LanguageParser.cpp:2507`): a name that an attribute
    /// method will read and write must be a variable name.
    fn require_variable_name(&self, name: &[u8]) -> Result<(), ParseError> {
        if is_variable_name(name) {
            return Ok(());
        }
        Err(self.error(E99_925, [name.to_vec()]))
    }

    // ---- what follows the directive ----

    /// `checkDirective` (`DirectiveParser.cpp:154`): if a clause follows, it
    /// must be a directive.
    fn check_directive(&self, cursor: &ClauseCursor, msg: Msg<0>) -> Result<(), ParseError> {
        let Some(next) = cursor.peek() else {
            return Ok(());
        };
        if self.ctx.tokens[next.tokens.start].kind.tag() == Tag::DColon {
            return Ok(());
        }
        Err(ParseError::raise(msg, next.span.start, []))
    }

    /// `hasBody` (`DirectiveParser.cpp:189`): whether a non-directive clause
    /// follows.
    fn has_body(&self, cursor: &ClauseCursor) -> bool {
        cursor
            .peek()
            .is_some_and(|next| self.ctx.tokens[next.tokens.start].kind.tag() != Tag::DColon)
    }

    // ---- dispatch ----

    fn dispatch(&mut self, cursor: &ClauseCursor) -> Result<DirectiveKind, ParseError> {
        // `Error_Translation_bad_directive`. Measured on the other side of this
        // gate too: a lone `:` is not a directive at all but an expression, and
        // `:junk` is 35.1.
        match self.next_real() {
            Some(token) if token.kind.tag() == Tag::DColon => {}
            _ => return Err(self.error(E99_916, [])),
        }
        // `Error_Symbol_expected_directive`. Measured: `::` alone and `:: "x"`
        // are both 20.916.
        let Some(token) = self.next_real() else {
            return Err(self.error(E20_916, []));
        };
        let TokenKind::Symbol { id, .. } = token.kind else {
            return Err(self.error(E20_916, []));
        };
        // A symbol that is not a directive keyword is 99.916 and not 20.916:
        // measured, `::junk` and `:: 5` both report `Unrecognized directive
        // instruction`.
        let Some(index) = self.ctx.keywords.directives.index_of(id) else {
            return Err(self.error(E99_916, []));
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
        // `Error_Symbol_or_string_class`, raised with no insert
        // (`DirectiveParser.cpp:340`).
        let name = match self.next_name() {
            Ok(token) => self.value_of(token),
            Err(_) => return Err(self.error(E19_901, [])),
        };
        // `isDuplicateClass` on the upcased name, then the new `activeClass`
        // with empty method tables (`DirectiveParser.cpp:349`-`355`).
        if !self
            .declared
            .classes
            .insert(name.to_ascii_uppercase().into())
        {
            return Err(self.error(E99_901, []));
        }
        self.declared.in_class = true;
        self.declared.members.clear();
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
            match self.sub_directive(token) {
                Some(SUBDIR_METACLASS) if class.metaclass.is_none() => {
                    class.metaclass = Some(self.class_reference(E19_906)?);
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
                    class.subclass = Some(self.class_reference(E19_907)?);
                }
                Some(SUBDIR_MIXINCLASS) if class.subclass.is_none() => {
                    class.subclass = Some(self.class_reference(E19_913)?);
                    class.mixin = true;
                }
                Some(SUBDIR_INHERIT) => {
                    // `INHERIT` consumes every remaining token of the clause,
                    // and requires at least one: measured, `::class c inherit`
                    // is 19.908.
                    if self.at_end() {
                        return Err(self.found(E19_908, None));
                    }
                    while !self.at_end() {
                        class.inherit.push(self.class_reference(E19_908)?);
                    }
                }
                Some(SUBDIR_ABSTRACT) if !class.abstract_ => class.abstract_ = true,
                _ => return Err(self.found(E25_901, Some(token))),
            }
        }
        Ok(DirectiveKind::Class(Box::new(class)))
    }

    /// `parseClassReference` (`DirectiveParser.cpp:287`).
    fn class_reference(&mut self, msg: Msg<1>) -> Result<ClassRef, ParseError> {
        let Some(token) = self.next_real() else {
            return Err(self.found(msg, None));
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
                let qualified = self.next_symbol().map_err(|_| self.error(E20_921, []))?;
                Ok(ClassRef {
                    namespace: Some(first),
                    name: Box::from(self.ctx.symbols.name(qualified).as_bytes()),
                })
            }
            _ => Err(self.found(msg, Some(token))),
        }
    }

    // ---- ::METHOD and ::ATTRIBUTE ----

    /// `methodDirective` (`DirectiveParser.cpp:629`).
    fn method(&mut self, cursor: &ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(E19_902)?;
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
            body: None,
        };
        // The EXTERNAL string is kept undecoded until the shape is known,
        // because the shapes decode it at different points and the error order
        // is observable. Measured: `::method m external "junk"` with a body is
        // 99.917 on line 1, while the same with ATTRIBUTE added is 99.934 on
        // line 2, so the attribute shape checks the body FIRST and the plain
        // external shape decodes first.
        let mut external: Option<Box<[u8]>> = None;
        while let Some(token) = self.next_real() {
            match self.sub_directive(token) {
                Some(SUBDIR_CLASS) if !method.class_method => method.class_method = true,
                Some(SUBDIR_EXTERNAL)
                    if external.is_none() && method.delegate.is_none() && !method.abstract_ =>
                {
                    // `Error_Symbol_or_string_external`, and a symbol is not
                    // enough: measured, `::method m external nostring` is
                    // 19.905.
                    external = Some(self.require_literal(E19_905)?);
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
                    method.delegate = Some(
                        self.next_symbol()
                            .map_err(|found| self.found(E20_926, found))?,
                    );
                }
                _ => return Err(self.found(E25_902, Some(token))),
            }
        }

        // `checkDuplicateMethod` runs ahead of every shape
        // (`DirectiveParser.cpp:822`). Measured: `::method m class` with a
        // body holding a syntax error and no `::CLASS` above it is 99.905.
        self.require_class(method.class_method)?;
        let upper: Box<[u8]> = method.name.to_ascii_uppercase().into();
        let member = Member {
            constant: false,
            attribute: method.attribute,
        };
        let setter = || {
            let mut setter = upper.to_vec();
            setter.push(b'=');
            setter.into_boxed_slice()
        };
        self.claim(upper.clone(), method.class_method, member, E99_902)?;

        // The shapes, in the C++'s own order, because they are not disjoint:
        // ATTRIBUTE combines with each of EXTERNAL and ABSTRACT, and DELEGATE
        // combines with ATTRIBUTE. Measured, `::method m delegate p attribute`
        // and `::method m attribute abstract` are both rc 0.
        if let Some(delegate) = method.delegate {
            // `getRetriever(delegateName)` runs BEFORE `checkDirective` here
            // (`DirectiveParser.cpp:831`-`834`). Measured:
            // `::method m delegate 5` with a body is 99.925 on line 1, not
            // 99.946 on line 2.
            self.require_variable_name(self.ctx.symbols.name(delegate).as_bytes())?;
            self.check_directive(cursor, E99_946)?;
            if method.attribute {
                self.claim(setter(), method.class_method, member, E99_902)?;
            }
        } else if method.attribute {
            // The setter's duplicate check comes before the body check here
            // (`DirectiveParser.cpp:855`).
            self.claim(setter(), method.class_method, member, E99_902)?;
            self.check_directive(cursor, E99_934)?;
            method.external = self.decode_external(external.as_deref(), false)?;
            self.bound = method.external.is_some();
            // And here it runs AFTER, and only in the sub-branch that generates
            // the accessor pair (`DirectiveParser.cpp:892`). Measured:
            // `::method 3 attribute` with a body is 99.934 on line 2, while
            // `::method 3 attribute abstract` and
            // `::method 3 attribute external "LIBRARY x"` never reach the check
            // at all and are rc 0 and 98.903.
            if external.is_none() && !method.abstract_ {
                self.require_variable_name(&method.name)?;
            }
        } else if method.abstract_ {
            self.check_directive(cursor, E99_933)?;
        } else if external.is_none() {
            method.body = Some(CodeBody::default());
        } else {
            method.external = self.decode_external(external.as_deref(), false)?;
            self.check_directive(cursor, E99_936)?;
            self.bound = true;
        }
        Ok(DirectiveKind::Method(Box::new(method)))
    }

    /// `attributeDirective` (`DirectiveParser.cpp:1457`).
    fn attribute(&mut self, cursor: &ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(E19_914)?;
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
            body: None,
        };
        let mut external: Option<Box<[u8]>> = None;
        while let Some(token) = self.next_real() {
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
                    external = Some(self.require_literal(E19_905)?);
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
                    attribute.delegate = Some(
                        self.next_symbol()
                            .map_err(|found| self.found(E20_926, found))?,
                    );
                }
                _ => return Err(self.found(E25_925, Some(token))),
            }
        }

        // `getRetriever(name)` runs before the style switch and so before every
        // body check (`DirectiveParser.cpp:1656`, whose own comment says it is
        // deliberately first "so errors get diagnosed on the correct line").
        // Measured: `::attribute 3` with a body is 99.925 on line 1, not 99.937
        // on line 2, and `::attribute 3 external "LIBRARY x"` is 99.925 too.
        self.require_variable_name(&attribute.name)?;
        // Each style's `checkDuplicateMethod` comes next, ahead of its body
        // check (`DirectiveParser.cpp:1664`, `:1725`, `:1790`). Measured:
        // `::attribute 'foo' class` with a body and no `::CLASS` above it is
        // 99.905, not 99.937.
        self.require_class(attribute.class_method)?;
        let upper: Box<[u8]> = attribute.name.to_ascii_uppercase().into();
        let mut setter = upper.to_vec();
        setter.push(b'=');
        let member = Member {
            constant: false,
            attribute: true,
        };
        let side = attribute.class_method;
        match attribute.style {
            AttributeStyle::Both => {
                self.claim(upper, side, member, E99_931)?;
                self.claim(setter.into(), side, member, E99_931)?;
            }
            AttributeStyle::Get => self.claim(upper, side, member, E99_931)?,
            AttributeStyle::Set => self.claim(setter.into(), side, member, E99_931)?,
        }

        // Every shape here checks the body BEFORE decoding the external
        // string, unlike `::METHOD`'s plain external shape. Measured:
        // `::attribute a get external "junk"` with a body is 99.935 on line 2,
        // where the same on `::method` is 99.917 on line 1.
        match attribute.style {
            // Both methods are generated, so a body can never belong here.
            AttributeStyle::Both => {
                self.check_directive(cursor, E99_937)?;
                // The delegate's own name check, which for this style comes
                // AFTER the body check where for GET/SET it comes before.
                // Measured: `::attribute a delegate 5` with a body is 99.937 on
                // line 2, and `::attribute a get delegate 5` with a body is
                // 99.925 on line 1.
                if let Some(delegate) = attribute.delegate {
                    self.require_variable_name(self.ctx.symbols.name(delegate).as_bytes())?;
                }
            }
            AttributeStyle::Get | AttributeStyle::Set => {
                if external.is_some() {
                    self.check_directive(cursor, E99_935)?;
                } else if attribute.abstract_ {
                    self.check_directive(cursor, E99_940)?;
                } else if let Some(delegate) = attribute.delegate {
                    self.require_variable_name(self.ctx.symbols.name(delegate).as_bytes())?;
                    self.check_directive(cursor, E99_947)?;
                } else {
                    attribute.body = self.has_body(cursor).then(CodeBody::default);
                }
            }
        }
        attribute.external = self.decode_external(external.as_deref(), false)?;
        self.bound = attribute.external.is_some();
        Ok(DirectiveKind::Attribute(Box::new(attribute)))
    }

    /// `decodeExternalMethod` (`DirectiveParser.cpp:1403`) and the routine
    /// form (`DirectiveParser.cpp:2649`-`2749`).
    fn decode_external(
        &self,
        spec: Option<&[u8]>,
        registered: bool,
    ) -> Result<Option<ExternalSpec>, ParseError> {
        let Some(spec) = spec else {
            return Ok(None);
        };
        // `words()` splits on blanks and upcases the FIRST word only, which is
        // why the library name keeps its case. A blank is a space or a tab and
        // nothing else: `words` goes through `subWords`, which drives
        // `RexxString::WordIterator`, whose `skipBlanks` and `skipNonBlanks`
        // both test `*scan != ' ' && *scan != '\t'` (`StringClass.hpp:155` and
        // `:178`). Measured: tabs do separate words, and `"  library   x  "`
        // resolves the library `x`.
        let words: Vec<&[u8]> = spec
            .split(|&byte| byte == b' ' || byte == b'\t')
            .filter(|word| !word.is_empty())
            .collect();
        // `Error_Translation_bad_external`. Measured for the empty string, for
        // one word, for four words and for a first word that is neither
        // keyword.
        let bad = || self.error(E99_917, [spec.to_vec()]);
        let Some((&keyword, rest)) = words.split_first() else {
            return Err(bad());
        };
        let mut keyword = keyword.to_vec();
        keyword.make_ascii_uppercase();
        let is_registered = registered && keyword == b"REGISTERED";
        if keyword != b"LIBRARY" && !is_registered {
            return Err(bad());
        }
        let (library, entry) = match rest {
            [library] => (*library, None),
            [library, entry] => (*library, Some(Box::from(*entry))),
            _ => return Err(bad()),
        };
        Ok(Some(ExternalSpec {
            registered: is_registered,
            library: Box::from(library),
            entry,
        }))
    }

    // ---- ::CONSTANT ----

    /// `constantDirective` (`DirectiveParser.cpp:1854`).
    fn constant(&mut self, cursor: &ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(E19_915)?;
        let value = match self.peek_real() {
            // No value at all, whose value is the name as written. Measured:
            // `::constant c` is rc 0.
            None => ConstantValue::Name,
            Some(token) if token.kind.tag() == Tag::LeftParen => {
                self.next_real();
                ConstantValue::Expression(self.constant_expression(token)?)
            }
            Some(token) if matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) => {
                self.next_real();
                ConstantValue::Text(self.value_of(token))
            }
            Some(_) => ConstantValue::Text(
                self.signed_constant()
                    .map_err(|refused| self.error(E19_916, [refused.token]))?,
            ),
        };
        // `Error_Invalid_data_constant_dir`. Measured: `::constant c 5 6` is
        // 21.913.
        self.required_end(E21_913)?;
        // `Error_Translation_constant_body`. Measured: 99.938 on line 2.
        self.check_directive(cursor, E99_938)?;
        // The duplicate checks, the class side only under a `::CLASS`, then
        // `createConstantGetterMethod`'s 99.906 (`DirectiveParser.cpp:1926`-`1933`).
        let upper: Box<[u8]> = name.to_ascii_uppercase().into();
        let member = Member {
            constant: true,
            attribute: false,
        };
        self.claim(upper.clone(), false, member, E99_932)?;
        if self.declared.in_class {
            self.claim(upper, true, member, E99_932)?;
        } else if matches!(value, ConstantValue::Expression(_)) {
            return Err(self.error(E99_906, []));
        }
        Ok(DirectiveKind::Constant(Box::new(ConstantDirective {
            name,
            value,
        })))
    }

    /// `translateConstantExpression` (`LanguageParser.cpp:1725`), entered with
    /// the `(` `opener` already consumed.
    fn constant_expression(&mut self, opener: &Token) -> Result<Expr, ParseError> {
        // `Error_Invalid_expression_missing_constant`. Measured:
        // `::constant c ()` is 35.936.
        let expr = parse_expr(self.ctx, &mut self.cursor, Terminators::RIGHT, E35_936)?;
        // `Error_Unmatched_parenthesis_paren`. Measured: `::constant c (1+2`
        // is 36.901. No blank can sit before a `)`, so `nextToken` and
        // `nextReal` agree here.
        match self.next_real() {
            Some(token) if token.kind.tag() == Tag::RightParen => Ok(expr),
            _ => Err(self.unmatched_paren(opener)),
        }
    }

    /// The signed-number value form that `::CONSTANT` and `::ANNOTATE` share
    /// (`DirectiveParser.cpp:1886`-`1907` and `2230`-`2251`).
    fn signed_constant(&mut self) -> Result<Box<[u8]>, NotSigned> {
        let token = self.next_real();
        let sign: &[u8] = match token.map(|token| &token.kind) {
            Some(TokenKind::Operator(Operator::Plus)) => b"+",
            Some(TokenKind::Operator(Operator::Subtract)) => b"-",
            _ => {
                return Err(NotSigned {
                    token: display_value(self.ctx, token),
                    value: None,
                });
            }
        };
        let refused = |value| NotSigned {
            token: sign.to_vec(),
            value,
        };
        let second = match self.next_real() {
            Some(second)
                if matches!(
                    second.kind,
                    TokenKind::Symbol {
                        class: SymbolClass::Constant,
                        ..
                    }
                ) =>
            {
                second
            }
            _ => return Err(refused(None)),
        };
        let mut value = sign.to_vec();
        value.extend_from_slice(&self.value_of(second));
        if !is_number(&value) {
            return Err(refused(Some(value)));
        }
        Ok(value.into_boxed_slice())
    }

    // ---- ::ANNOTATE ----

    /// `annotateDirective` (`DirectiveParser.cpp:1940`).
    fn annotate(&mut self) -> Result<DirectiveKind, ParseError> {
        // `Error_Symbol_expected_annotation_type`, and a literal is not enough:
        // measured, `::annotate "package"` is 20.924.
        let Some(token) = self.next_real() else {
            return Err(self.found(E20_924, None));
        };
        // `Error_Symbol_or_string_directive_option`, one error for every named
        // target, naming the directive and the target. Measured:
        // `::annotate class` is 19.925.
        let target = match self.sub_directive(token) {
            Some(SUBDIR_PACKAGE) => AnnotationTarget::Package,
            Some(SUBDIR_CLASS) => AnnotationTarget::Class(self.annotated_name(b"CLASS")?),
            Some(SUBDIR_ROUTINE) => AnnotationTarget::Routine(self.annotated_name(b"ROUTINE")?),
            Some(SUBDIR_METHOD) => AnnotationTarget::Method(self.annotated_name(b"METHOD")?),
            Some(SUBDIR_ATTRIBUTE) => {
                AnnotationTarget::Attribute(self.annotated_name(b"ATTRIBUTE")?)
            }
            Some(SUBDIR_CONSTANT) => AnnotationTarget::Constant(self.annotated_name(b"CONSTANT")?),
            // A non-symbol reaches 20.924 above, so this is a symbol that is
            // not one of the six. Measured: `::annotate junk k 1` is 25.928.
            _ => {
                if token.kind.tag() != Tag::Symbol {
                    return Err(self.found(E20_924, Some(token)));
                }
                return Err(self.found(E25_928, Some(token)));
            }
        };
        self.require_annotation_target(&target)?;
        let mut annotations = Vec::new();
        while !self.at_end() {
            annotations.push(self.annotation()?);
        }
        Ok(DirectiveKind::Annotate(Box::new(Annotate {
            target,
            annotations,
        })))
    }

    /// 99.945 for a target the directives so far do not declare, which
    /// `annotateDirective` checks before reading any pair
    /// (`DirectiveParser.cpp:1983`-`2090`, `processAttributeAnnotations`).
    fn require_annotation_target(&self, target: &AnnotationTarget) -> Result<(), ParseError> {
        let attribute = |name: &[u8], side: bool| {
            let mut setter = name.to_vec();
            setter.push(b'=');
            [name, &setter[..]].into_iter().any(|half| {
                self.declared
                    .members
                    .get(&(side, Box::from(half)))
                    .is_some_and(|member| member.attribute)
            })
        };
        let (kind, name): (&[u8], &[u8]) = match target {
            AnnotationTarget::Package => return Ok(()),
            AnnotationTarget::Class(name) if self.declared.classes.contains(name) => return Ok(()),
            AnnotationTarget::Class(name) => (b"class", name),
            AnnotationTarget::Routine(name) if self.declared.routines.contains(name) => {
                return Ok(());
            }
            AnnotationTarget::Routine(name) => (b"routine", name),
            AnnotationTarget::Method(name) if self.find_member(name).is_some() => return Ok(()),
            AnnotationTarget::Method(name) => (b"method", name),
            AnnotationTarget::Constant(name)
                if self.find_member(name).is_some_and(|member| member.constant) =>
            {
                return Ok(());
            }
            AnnotationTarget::Constant(name) => (b"constant", name),
            AnnotationTarget::Attribute(name)
                if attribute(name, false) || attribute(name, true) =>
            {
                return Ok(());
            }
            AnnotationTarget::Attribute(name) => (b"attribute", name),
        };
        Err(self.error(E99_945, [kind.to_vec(), name.to_vec()]))
    }

    /// The upcased name an `::ANNOTATE` of the `target` kind requires.
    fn annotated_name(&mut self, target: &'static [u8]) -> Result<Box<[u8]>, ParseError> {
        match self.next_name() {
            Ok(token) => Ok(self.upper_value_of(token)),
            Err(_) => Err(self.error(E19_925, [b"::ANNOTATE".to_vec(), target.to_vec()])),
        }
    }

    /// `processAnnotation` (`DirectiveParser.cpp:2209`): one `name value` pair.
    fn annotation(&mut self) -> Result<Annotation, ParseError> {
        // `Error_Symbol_expected_annotation_attribute`. Measured:
        // `::annotate package "a" 1` is 20.919.
        let name = self
            .next_symbol()
            .map_err(|found| self.found(E20_919, found))?;
        let value = match self.peek_real() {
            // `Error_Symbol_or_string_package_attribute_missing`, a DIFFERENT
            // number from the bad-value one. Measured: `::annotate package a`
            // is 19.924 and `::annotate package a *` is 19.923.
            None => return Err(self.error(E19_924, [])),
            Some(token) if matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) => {
                self.next_real();
                self.value_of(token)
            }
            // No parenthesised form here, unlike `::CONSTANT`.
            Some(_) => self
                .signed_constant()
                .map_err(|refused| self.error(E19_923, [refused.value.unwrap_or(refused.token)]))?,
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
            match self.sub_directive(token) {
                Some(SUBDIR_DIGITS) => {
                    // `Error_Symbol_or_string_digits_value`, then
                    // `Error_Invalid_whole_number_digits`. Measured:
                    // `::options digits -1` is 19.917 because a `-` is neither
                    // a symbol nor a literal, and `::options digits 0` is 26.5
                    // because the value must exceed zero.
                    let value = self.require_name(E19_917)?;
                    let digits = whole_number(&value, ARGUMENT_DIGITS)
                        .filter(|&digits| digits >= 1)
                        .and_then(|digits| usize::try_from(digits).ok())
                        .ok_or_else(|| self.error(E26_5, [value.to_vec()]))?;
                    if digits <= self.declared.fuzz {
                        return Err(self.numeric_conflict(digits, self.declared.fuzz));
                    }
                    self.declared.digits = digits;
                    options.push(PackageOption::Digits(digits));
                }
                Some(SUBDIR_FORM) => {
                    // `Error_Symbol_expected_form`, then
                    // `Error_Invalid_subkeyword_form`. This argument resolves
                    // against `subKeywords[]`: measured, `::options form value`
                    // is 25.11 where `VALUE` is a row of that table, and
                    // `::options form "scientific"` is 20.925.
                    let token = match self.next_real() {
                        Some(token) if token.kind.tag() == Tag::Symbol => token,
                        found => return Err(self.found(E20_925, found)),
                    };
                    let form = match self.sub_keyword(token) {
                        Some(SUBKEY_SCIENTIFIC) => OptionsForm::Scientific,
                        Some(SUBKEY_ENGINEERING) => OptionsForm::Engineering,
                        _ => return Err(self.found(E25_11, Some(token))),
                    };
                    options.push(PackageOption::Form(form));
                }
                Some(SUBDIR_FUZZ) => {
                    // `Error_Symbol_or_string_fuzz_value`, then
                    // `Error_Invalid_whole_number_fuzz`. Zero is legal here
                    // where it is not for DIGITS: measured, `::options fuzz 0`
                    // is rc 0 and `::options fuzz "-1"` is 26.6.
                    let value = self.require_name(E19_918)?;
                    // `try_from` also rejects a negative, which is what
                    // `requestUnsignedNumber` does and is why FUZZ has no
                    // lower-bound filter of its own.
                    let fuzz = whole_number(&value, ARGUMENT_DIGITS)
                        .and_then(|fuzz| usize::try_from(fuzz).ok())
                        .ok_or_else(|| self.error(E26_6, [value.to_vec()]))?;
                    if fuzz >= self.declared.digits {
                        return Err(self.numeric_conflict(self.declared.digits, fuzz));
                    }
                    self.declared.fuzz = fuzz;
                    options.push(PackageOption::Fuzz(fuzz));
                }
                Some(SUBDIR_TRACE) => {
                    // `Error_Symbol_or_string_trace_value`, then
                    // `Error_Invalid_trace_trace`. Measured: `::options trace
                    // zzz` is 24.1 and `::options trace r` is rc 0.
                    let value = self.require_name(E19_919)?;
                    check_trace_setting(&value).map_err(|()| self.bad_trace_setting(&value))?;
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
                    let token = match self.next_real() {
                        Some(token) if token.kind.tag() == Tag::Symbol => token,
                        found => return Err(self.found(E20_935, found)),
                    };
                    let inherit = match self.sub_keyword(token) {
                        Some(SUBKEY_INHERIT) => true,
                        Some(SUBKEY_NOINHERIT) => false,
                        _ => return Err(self.found(E25_935, Some(token))),
                    };
                    options.push(PackageOption::NumericInherit(inherit));
                }
                _ => return Err(self.found(E25_924, Some(token))),
            }
        }
        Ok(DirectiveKind::Options(options))
    }

    /// 33.1 for a package `digits` that does not exceed its `fuzz`
    /// (`Error_Expression_result_digits`, `DirectiveParser.cpp:988`, `:1047`).
    #[cold]
    #[inline(never)]
    fn numeric_conflict(&self, digits: usize, fuzz: usize) -> ParseError {
        self.error(
            E33_1,
            [
                digits.to_string().into_bytes(),
                fuzz.to_string().into_bytes(),
            ],
        )
    }

    /// One of the seven condition options and its `SYNTAX`/`CONDITION`
    /// argument.
    fn condition_option(&mut self, which: ConditionOption) -> Result<PackageOption, ParseError> {
        // `Error_Symbol_expected_after_keyword`, the same number for all seven.
        // Measured: `::options novalue` with nothing after it and
        // `::options novalue "syntax"` are both 20.929.
        let name = || condition_option_name(which).to_vec();
        let Some(token) = self.next_real() else {
            return Err(self.error(E20_929, [name()]));
        };
        if token.kind.tag() != Tag::Symbol {
            return Err(self.error(E20_929, [name()]));
        }
        let syntax = match self.sub_directive(token) {
            Some(SUBDIR_SYNTAX) => true,
            Some(SUBDIR_CONDITION) => false,
            // `SUBDIRECTIVE_ERROR` falls through to the SYNTAX arm for NOVALUE
            // alone (`DirectiveParser.cpp:1091`).
            Some(SUBDIR_ERROR) if which == ConditionOption::NoValue => true,
            // `Error_Invalid_subkeyword_following`.
            _ => return Err(self.error(E25_927, [name(), self.value_of(token).into_vec()])),
        };
        Ok(PackageOption::Condition { which, syntax })
    }

    // ---- ::REQUIRES ----

    /// `requiresDirective` (`DirectiveParser.cpp:2779`).
    fn requires(&mut self) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(E19_904)?;
        let mut requires = Requires {
            name,
            library: false,
            namespace: None,
        };
        while let Some(token) = self.next_real() {
            // `Error_Invalid_subkeyword_requires`. LIBRARY and NAMESPACE are
            // mutually exclusive as well as each being once-only, and both
            // rejections are this error: measured in both orders and doubled.
            let taken = requires.library || requires.namespace.is_some();
            match self.sub_directive(token) {
                Some(SUBDIR_NAMESPACE) if !taken => {
                    // `Error_Symbol_expected_namespace`. Measured:
                    // `::requires "x" namespace "ns"` is 20.920.
                    let namespace = self.next_symbol().map_err(|_| self.error(E20_920, []))?;
                    // `Error_Translation_reserved_namespace`. The symbol is
                    // already upcased, so this catches every spelling of REXX.
                    if self.ctx.symbols.name(namespace) == "REXX" {
                        return Err(self.error(E99_944, []));
                    }
                    requires.namespace = Some(namespace);
                }
                Some(SUBDIR_LIBRARY) if !taken => requires.library = true,
                _ => return Err(self.found(E25_904, Some(token))),
            }
        }
        Ok(DirectiveKind::Requires(Box::new(requires)))
    }

    // ---- ::RESOURCE ----

    /// `resourceDirective` (`DirectiveParser.cpp:2266`).
    fn resource(&mut self) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(E19_920)?;
        let mut end_marker: Box<[u8]> = Box::from(DEFAULT_RESOURCE_END);
        if !self.at_end() {
            // `Error_Invalid_subkeyword_resource`, for a non-symbol and for
            // any sub-directive but END. Measured: `::resource data junk` is
            // 25.926.
            let token = self.next_real().expect("at_end was false");
            if self.sub_directive(token) != Some(SUBDIR_END) {
                return Err(self.found(E25_926, Some(token)));
            }
            // `Error_Symbol_or_string_resource_end`, and the marker is UPCASED
            // when it comes from a symbol because that is what
            // `RexxToken::value()` gives. Measured: `::resource data end stop`
            // is closed by `STOP` and not by `stop`.
            end_marker = self.require_name(E19_921)?;
            // `Error_Invalid_data_resource_dir`. Measured:
            // `::resource data end "x" extra` is 21.914.
            self.required_end(E21_914)?;
        }
        // `Error_Translation_duplicate_resource`, before the body is read
        // (`DirectiveParser.cpp:2316`).
        if !self
            .declared
            .resources
            .insert(name.to_ascii_uppercase().into())
        {
            return Err(self.error(E99_942, []));
        }

        // `scan_resource_if_directive` copies a body for exactly the clause
        // shape enforced above, so a missing one means the marker never came.
        let Some(body) = self
            .ctx
            .resources
            .iter()
            .find(|body| body.directive == self.clause.tokens.start)
        else {
            return Err(self.error(E99_943, [end_marker.into_vec(), name.into_vec()]));
        };
        Ok(DirectiveKind::Resource(Box::new(Resource {
            name,
            end_marker,
            lines: body.lines.clone(),
        })))
    }

    // ---- ::ROUTINE ----

    /// `routineDirective` (`DirectiveParser.cpp:2565`).
    fn routine(&mut self, cursor: &ClauseCursor) -> Result<DirectiveKind, ParseError> {
        let name = self.require_name(E19_903)?;
        // `isDuplicateRoutine`, ahead of the options (`DirectiveParser.cpp:2578`).
        if !self
            .declared
            .routines
            .insert(name.to_ascii_uppercase().into())
        {
            return Err(self.error(E99_903, []));
        }
        let mut routine = RoutineDirective {
            name,
            access: Access::Default,
            external: None,
            body: None,
        };
        let mut external: Option<Box<[u8]>> = None;
        while let Some(token) = self.next_real() {
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
                        return Err(self.found(E25_901, Some(token)));
                    }
                    external = Some(self.require_literal(E19_905)?);
                }
                Some(SUBDIR_PUBLIC) if routine.access == Access::Default => {
                    routine.access = Access::Public;
                }
                Some(SUBDIR_PRIVATE) if routine.access == Access::Default => {
                    routine.access = Access::Private;
                }
                _ => return Err(self.found(E25_903, Some(token))),
            }
        }
        match external {
            // `REGISTERED` is accepted here and nowhere else.
            Some(spec) => {
                routine.external = self.decode_external(Some(&spec), true)?;
                // `Error_Translation_external_routine`.
                self.check_directive(cursor, E99_939)?;
                self.bound = true;
            }
            None => routine.body = Some(CodeBody::default()),
        }
        Ok(DirectiveKind::Routine(Box::new(routine)))
    }
}

/// What `signed_constant` names when it refuses: `token`, the token the
/// oracle's own `token` points at there (the one found where a sign was
/// wanted, otherwise the sign), and `value`, the signed value when only the
/// number test failed.
struct NotSigned {
    token: Vec<u8>,
    value: Option<Vec<u8>>,
}

/// The keyword a condition option is spelled with, as 20.929 and 25.927 name it.
fn condition_option_name(which: ConditionOption) -> &'static [u8] {
    match which {
        ConditionOption::All => b"ALL",
        ConditionOption::Error => b"ERROR",
        ConditionOption::Failure => b"FAILURE",
        ConditionOption::LostDigits => b"LOSTDIGITS",
        ConditionOption::NoString => b"NOSTRING",
        ConditionOption::NotReady => b"NOTREADY",
        ConditionOption::NoValue => b"NOVALUE",
    }
}

/// Whether `name` spells a variable: a simple name, a stem, or a compound.
fn is_variable_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > MAX_SYMBOL_LENGTH {
        return false;
    }
    let mut scan = 0;
    while scan < name.len() && is_symbol_char(name[scan]) {
        scan += 1;
    }
    if scan < name.len() {
        // A non-symbol character stops the walk. The only one that can still
        // leave a name is an exponent's sign: not last, preceded by an `E`, and
        // followed by digits alone.
        if scan + 1 >= name.len() || scan == 0 {
            return false;
        }
        if name[scan] != b'-' && name[scan] != b'+' {
            return false;
        }
        if !name[scan - 1].eq_ignore_ascii_case(&b'E') {
            return false;
        }
        if !name[scan + 1..].iter().all(u8::is_ascii_digit) {
            return false;
        }
    }
    !(name[0] == b'.' || name[0].is_ascii_digit())
}

#[cfg(test)]
mod tests;
