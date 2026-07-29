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

//! The directive grammar, pinned against `build/bin/rexxc`.
//!
//! Every accepted case below was checked with `rexxc`, which is a parse-only
//! oracle, and every rejected case carries the number and sub-number `rexxc`
//! reported. Both directions are tested for every gate: a test that only checks
//! the accepted cases catches one error and misses its opposite.
//!
//! In-crate rather than under `tests/`, because `ParseCtx`, `ClauseCursor` and
//! `parse_directive` are all `pub(crate)` and an integration test is a separate
//! crate. Task 3.6's tests live here for the same reason.
//!
//! # What an rc of 0 does and does not prove
//!
//! `rexxc` runs more than the parse: it installs the package, which resolves
//! external libraries and checks duplicates. Two rows below therefore cite a
//! NON-zero rc as their evidence, and cite it deliberately, because reaching a
//! run-time failure proves the parse succeeded: measured,
//! `::ROUTINE r EXTERNAL "LIBRARY x"` is Error 98.903 at rc 158 and
//! `::METHOD m CLASS` with no `::CLASS` above it is 99.905. Neither is a parse
//! error and neither is raised here.

use crate::ast::{
    Access, AnnotationTarget, AttributeStyle, CodeBody, ConditionOption, ConstantValue, Directive,
    DirectiveKind, ExternalSpec, GuardOption, OptionsForm, PackageOption, Protection,
};
use crate::block::translate_block;
use crate::clause::{ClauseCursor, split_clauses};
use crate::token::{Keywords, ParseCtx, ParseError, SymbolTable};
use crate::{ProgramSource, SourceKind, scan};

use super::{
    DIR_ANNOTATE, DIR_ATTRIBUTE, DIR_CLASS, DIR_CONSTANT, DIR_METHOD, DIR_OPTIONS, DIR_REQUIRES,
    DIR_RESOURCE, DIR_ROUTINE, SUBDIR_ABSTRACT, SUBDIR_ALL, SUBDIR_ATTRIBUTE, SUBDIR_CLASS,
    SUBDIR_CONDITION, SUBDIR_CONSTANT, SUBDIR_DELEGATE, SUBDIR_DIGITS, SUBDIR_END, SUBDIR_ERROR,
    SUBDIR_EXTERNAL, SUBDIR_FAILURE, SUBDIR_FORM, SUBDIR_FUZZ, SUBDIR_GET, SUBDIR_GUARDED,
    SUBDIR_INHERIT, SUBDIR_LIBRARY, SUBDIR_LOSTDIGITS, SUBDIR_METACLASS, SUBDIR_METHOD,
    SUBDIR_MIXINCLASS, SUBDIR_NAMESPACE, SUBDIR_NOPROLOG, SUBDIR_NOSTRING, SUBDIR_NOTREADY,
    SUBDIR_NOVALUE, SUBDIR_NUMERIC, SUBDIR_PACKAGE, SUBDIR_PRIVATE, SUBDIR_PROLOG,
    SUBDIR_PROTECTED, SUBDIR_PUBLIC, SUBDIR_ROUTINE, SUBDIR_SET, SUBDIR_SUBCLASS, SUBDIR_SYNTAX,
    SUBDIR_TRACE, SUBDIR_UNGUARDED, SUBDIR_UNPROTECTED, SUBKEY_ENGINEERING, SUBKEY_INHERIT,
    SUBKEY_NOINHERIT, SUBKEY_SCIENTIFIC, parse_directive,
};

/// Parses `text` whole, the way `translate` does: one code body, then every `::`
/// clause through `parse_directive` with that directive's own body after it.
///
/// The same composition the public entry point uses, minus building a `Program`,
/// because the directives are what is asserted here. Driving `parse_directive`
/// over every `::` clause of a real file is what makes the `CoreClasses.orx`
/// test possible.
fn parse_with_symbols(
    text: &str,
    kind: SourceKind,
) -> Result<(Vec<Directive>, SymbolTable), ParseError> {
    let source = ProgramSource::new(text.as_bytes().to_vec(), kind);
    let scanned = scan(&source)?;
    let result = {
        let ctx = ParseCtx {
            source: &source,
            tokens: &scanned.tokens,
            symbols: &scanned.symbols,
            keywords: &scanned.keywords,
            resources: &scanned.resources,
        };
        let mut cursor = ClauseCursor::new(split_clauses(ctx.tokens)?);
        let mut directives = Vec::new();
        let mut failure = None;
        if let Err(e) = translate_block(&ctx, &mut cursor) {
            failure = Some(e);
        }
        while failure.is_none() && cursor.peek().is_some() {
            match parse_directive(&ctx, &mut cursor) {
                Ok(mut directive) => {
                    let wants_body = match &directive.kind {
                        DirectiveKind::Method(m) => m.body.is_some(),
                        DirectiveKind::Attribute(a) => a.body.is_some(),
                        DirectiveKind::Routine(r) => r.body.is_some(),
                        _ => false,
                    };
                    if wants_body {
                        match translate_block(&ctx, &mut cursor) {
                            Ok(body) => set_body(&mut directive.kind, body),
                            Err(e) => failure = Some(e),
                        }
                    }
                    directives.push(directive);
                }
                Err(e) => failure = Some(e),
            }
        }
        match failure {
            Some(e) => Err(e),
            None => Ok(directives),
        }
    };
    result.map(|directives| (directives, scanned.symbols))
}

/// Installs an assembled body into whichever directive kind can hold one.
fn set_body(kind: &mut DirectiveKind, body: CodeBody) {
    match kind {
        DirectiveKind::Method(m) => m.body = Some(body),
        DirectiveKind::Attribute(a) => a.body = Some(body),
        DirectiveKind::Routine(r) => r.body = Some(body),
        other => panic!("{other:?} cannot hold a body"),
    }
}

fn parse(text: &str) -> Result<Vec<Directive>, ParseError> {
    parse_with_symbols(text, SourceKind::Program).map(|(directives, _)| directives)
}

/// The directives of `text`, which must parse.
fn ok(text: &str) -> Vec<Directive> {
    parse(text).unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"))
}

/// The error `text` raises, as `(code, sub)`.
fn err(text: &str) -> (u16, u16) {
    match parse(text) {
        Ok(directives) => panic!(
            "{text:?} parsed into {:?} but an error was expected",
            keywords_of(&directives)
        ),
        Err(e) => (e.code, e.sub),
    }
}

/// The byte `text`'s error is reported against, which Task 3.8 turns into a
/// line number.
fn err_byte(text: &str) -> usize {
    parse(text).expect_err("an error was expected").byte
}

fn keywords_of(directives: &[Directive]) -> Vec<&'static str> {
    directives.iter().map(|d| d.kind.keyword()).collect()
}

/// A canonical rendering of every directive in `text`, one string each.
///
/// Space-separated so that no two different nodes render alike: every optional
/// field either contributes a token or contributes nothing, and no field's token
/// can be produced by another field.
fn shapes(text: &str) -> Vec<String> {
    let (directives, symbols) = parse_with_symbols(text, SourceKind::Program)
        .unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
    directives
        .iter()
        .map(|d| shape(d, &symbols, text))
        .collect()
}

fn shape(directive: &Directive, symbols: &SymbolTable, text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    match &directive.kind {
        DirectiveKind::Class(class) => {
            out.push(format!("class {}", string(&class.name)));
            push_access(&mut out, class.access);
            if class.abstract_ {
                out.push("abstract".into());
            }
            if let Some(reference) = &class.subclass {
                let keyword = if class.mixin {
                    "mixinclass"
                } else {
                    "subclass"
                };
                out.push(format!("{keyword} {}", class_ref(reference, symbols)));
            }
            if let Some(reference) = &class.metaclass {
                out.push(format!("metaclass {}", class_ref(reference, symbols)));
            }
            for reference in &class.inherit {
                out.push(format!("inherit {}", class_ref(reference, symbols)));
            }
        }
        DirectiveKind::Method(method) => {
            out.push(format!("method {}", string(&method.name)));
            if method.class_method {
                out.push("class".into());
            }
            if method.attribute {
                out.push("attribute".into());
            }
            if method.abstract_ {
                out.push("abstract".into());
            }
            push_access(&mut out, method.access);
            push_protection(&mut out, method.protection);
            push_guard(&mut out, method.guard);
            push_external(&mut out, method.external.as_ref());
            if let Some(delegate) = method.delegate {
                out.push(format!("delegate {}", symbols.name(delegate)));
            }
            if method.body.is_some() {
                out.push("body".into());
            }
        }
        DirectiveKind::Attribute(attribute) => {
            out.push(format!("attribute {}", string(&attribute.name)));
            out.push(
                match attribute.style {
                    AttributeStyle::Both => "both",
                    AttributeStyle::Get => "get",
                    AttributeStyle::Set => "set",
                }
                .into(),
            );
            if attribute.class_method {
                out.push("class".into());
            }
            if attribute.abstract_ {
                out.push("abstract".into());
            }
            push_access(&mut out, attribute.access);
            push_protection(&mut out, attribute.protection);
            push_guard(&mut out, attribute.guard);
            push_external(&mut out, attribute.external.as_ref());
            if let Some(delegate) = attribute.delegate {
                out.push(format!("delegate {}", symbols.name(delegate)));
            }
            if attribute.body.is_some() {
                out.push("body".into());
            }
        }
        DirectiveKind::Constant(constant) => {
            out.push(format!("constant {}", string(&constant.name)));
            out.push(match &constant.value {
                ConstantValue::Name => "<name>".to_string(),
                ConstantValue::Text(value) => format!("text {}", string(value)),
                ConstantValue::Expression(expr) => format!("expr {}", expr.shape(symbols)),
            });
        }
        DirectiveKind::Annotate(annotate) => {
            out.push(match &annotate.target {
                AnnotationTarget::Package => "annotate package".to_string(),
                AnnotationTarget::Class(name) => format!("annotate class {}", string(name)),
                AnnotationTarget::Routine(name) => format!("annotate routine {}", string(name)),
                AnnotationTarget::Method(name) => format!("annotate method {}", string(name)),
                AnnotationTarget::Attribute(name) => {
                    format!("annotate attribute {}", string(name))
                }
                AnnotationTarget::Constant(name) => format!("annotate constant {}", string(name)),
            });
            for annotation in &annotate.annotations {
                out.push(format!(
                    "{}={}",
                    symbols.name(annotation.name),
                    string(&annotation.value)
                ));
            }
        }
        DirectiveKind::Options(options) => {
            out.push("options".into());
            for option in options {
                out.push(match option {
                    PackageOption::Digits(digits) => format!("digits {digits}"),
                    PackageOption::Fuzz(fuzz) => format!("fuzz {fuzz}"),
                    PackageOption::Form(OptionsForm::Scientific) => "form scientific".into(),
                    PackageOption::Form(OptionsForm::Engineering) => "form engineering".into(),
                    PackageOption::Trace(setting) => format!("trace {}", string(setting)),
                    PackageOption::Condition { which, syntax } => format!(
                        "{}={}",
                        match which {
                            ConditionOption::All => "all",
                            ConditionOption::Error => "error",
                            ConditionOption::Failure => "failure",
                            ConditionOption::LostDigits => "lostdigits",
                            ConditionOption::NoString => "nostring",
                            ConditionOption::NotReady => "notready",
                            ConditionOption::NoValue => "novalue",
                        },
                        if *syntax { "syntax" } else { "condition" }
                    ),
                    PackageOption::Prolog(true) => "prolog".into(),
                    PackageOption::Prolog(false) => "noprolog".into(),
                    PackageOption::NumericInherit(true) => "numeric inherit".into(),
                    PackageOption::NumericInherit(false) => "numeric noinherit".into(),
                });
            }
        }
        DirectiveKind::Requires(requires) => {
            out.push(format!("requires {}", string(&requires.name)));
            if requires.library {
                out.push("library".into());
            }
            if let Some(namespace) = requires.namespace {
                out.push(format!("namespace {}", symbols.name(namespace)));
            }
        }
        DirectiveKind::Resource(resource) => {
            out.push(format!("resource {}", string(&resource.name)));
            out.push(format!("end {}", string(&resource.end_marker)));
            for line in &resource.lines {
                out.push(format!("line {:?}", &text[line.clone()]));
            }
        }
        DirectiveKind::Routine(routine) => {
            out.push(format!("routine {}", string(&routine.name)));
            push_access(&mut out, routine.access);
            push_external(&mut out, routine.external.as_ref());
            if routine.body.is_some() {
                out.push("body".into());
            }
        }
    }
    out.join(" ")
}

fn class_ref(reference: &crate::ast::ClassRef, symbols: &SymbolTable) -> String {
    match reference.namespace {
        Some(namespace) => format!("{}:{}", symbols.name(namespace), string(&reference.name)),
        None => string(&reference.name),
    }
}

fn push_access(out: &mut Vec<String>, access: Access) {
    match access {
        Access::Default => {}
        Access::Private => out.push("private".into()),
        Access::Public => out.push("public".into()),
        Access::Package => out.push("package".into()),
    }
}

fn push_protection(out: &mut Vec<String>, protection: Protection) {
    match protection {
        Protection::Default => {}
        Protection::Protected => out.push("protected".into()),
        Protection::Unprotected => out.push("unprotected".into()),
    }
}

fn push_guard(out: &mut Vec<String>, guard: GuardOption) {
    match guard {
        GuardOption::Default => {}
        GuardOption::Guarded => out.push("guarded".into()),
        GuardOption::Unguarded => out.push("unguarded".into()),
    }
}

fn push_external(out: &mut Vec<String>, external: Option<&ExternalSpec>) {
    if let Some(external) = external {
        let keyword = if external.registered {
            "registered"
        } else {
            "library"
        };
        let entry = match &external.entry {
            Some(entry) => format!(":{}", string(entry)),
            None => String::new(),
        };
        out.push(format!("{keyword} {}{entry}", string(&external.library)));
    }
}

/// Bytes as a quoted, escaped string, so that no two byte strings render alike.
fn string(bytes: &[u8]) -> String {
    format!("{:?}", String::from_utf8_lossy(bytes))
}

#[test]
fn the_directive_tables_are_nine_and_forty_entries() {
    // Asserted so that a mis-extraction fails loudly rather than silently
    // narrowing the task. NOT nine and thirty-six: that split is an artefact of
    // an unanchored grep over the enum, and it would leave the sub-directive
    // table four entries short.
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    assert_eq!(keywords.directives.len(), 9, "the directive table is not 9");
    assert_eq!(
        keywords.sub_directives.len(),
        40,
        "the sub-directive table is not 40"
    );
}

#[test]
fn directive_indices_still_name_their_own_spellings() {
    // The index constants are positions in a table whose order is load bearing,
    // so each is pinned against the spelling it stands for. A reordering of
    // `DIRECTIVES` fails here rather than silently making `::CLASS` parse as
    // `::CONSTANT`.
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    for (index, spelling) in [
        (DIR_ANNOTATE, "ANNOTATE"),
        (DIR_ATTRIBUTE, "ATTRIBUTE"),
        (DIR_CLASS, "CLASS"),
        (DIR_CONSTANT, "CONSTANT"),
        (DIR_METHOD, "METHOD"),
        (DIR_OPTIONS, "OPTIONS"),
        (DIR_REQUIRES, "REQUIRES"),
        (DIR_RESOURCE, "RESOURCE"),
        (DIR_ROUTINE, "ROUTINE"),
    ] {
        let id = symbols.intern(spelling);
        assert_eq!(
            keywords.directives.index_of(id),
            Some(index),
            "DIRECTIVES no longer holds {spelling} at {index}"
        );
    }
}

#[test]
fn sub_directive_indices_still_name_their_own_spellings() {
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    let table = [
        (SUBDIR_ABSTRACT, "ABSTRACT"),
        (SUBDIR_ALL, "ALL"),
        (SUBDIR_ATTRIBUTE, "ATTRIBUTE"),
        (SUBDIR_CLASS, "CLASS"),
        (SUBDIR_CONDITION, "CONDITION"),
        (SUBDIR_CONSTANT, "CONSTANT"),
        (SUBDIR_DELEGATE, "DELEGATE"),
        (SUBDIR_DIGITS, "DIGITS"),
        (SUBDIR_END, "END"),
        (SUBDIR_ERROR, "ERROR"),
        (SUBDIR_EXTERNAL, "EXTERNAL"),
        (SUBDIR_FAILURE, "FAILURE"),
        (SUBDIR_FORM, "FORM"),
        (SUBDIR_FUZZ, "FUZZ"),
        (SUBDIR_GET, "GET"),
        (SUBDIR_GUARDED, "GUARDED"),
        (SUBDIR_INHERIT, "INHERIT"),
        (SUBDIR_LIBRARY, "LIBRARY"),
        (SUBDIR_LOSTDIGITS, "LOSTDIGITS"),
        (SUBDIR_METACLASS, "METACLASS"),
        (SUBDIR_METHOD, "METHOD"),
        (SUBDIR_MIXINCLASS, "MIXINCLASS"),
        (SUBDIR_NAMESPACE, "NAMESPACE"),
        (SUBDIR_NOPROLOG, "NOPROLOG"),
        (SUBDIR_NOSTRING, "NOSTRING"),
        (SUBDIR_NOTREADY, "NOTREADY"),
        (SUBDIR_NOVALUE, "NOVALUE"),
        (SUBDIR_NUMERIC, "NUMERIC"),
        (SUBDIR_PACKAGE, "PACKAGE"),
        (SUBDIR_PRIVATE, "PRIVATE"),
        (SUBDIR_PROLOG, "PROLOG"),
        (SUBDIR_PROTECTED, "PROTECTED"),
        (SUBDIR_PUBLIC, "PUBLIC"),
        (SUBDIR_ROUTINE, "ROUTINE"),
        (SUBDIR_SET, "SET"),
        (SUBDIR_SUBCLASS, "SUBCLASS"),
        (SUBDIR_SYNTAX, "SYNTAX"),
        (SUBDIR_TRACE, "TRACE"),
        (SUBDIR_UNGUARDED, "UNGUARDED"),
        (SUBDIR_UNPROTECTED, "UNPROTECTED"),
    ];
    // Every row of the C++ table is named here, not just the ones a test happens
    // to reach.
    assert_eq!(table.len(), keywords.sub_directives.len());
    for (index, spelling) in table {
        let id = symbols.intern(spelling);
        assert_eq!(
            keywords.sub_directives.index_of(id),
            Some(index),
            "SUB_DIRECTIVES no longer holds {spelling} at {index}"
        );
    }
}

#[test]
fn the_sub_keyword_indices_used_here_name_their_own_spellings() {
    // These four come from `SUB_KEYWORDS`, a different table, and `INHERIT` is a
    // row of both at different positions. Pinning them here is what keeps
    // `::OPTIONS NUMERIC INHERIT` from silently resolving against the wrong
    // table.
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    for (index, spelling) in [
        (SUBKEY_ENGINEERING, "ENGINEERING"),
        (SUBKEY_INHERIT, "INHERIT"),
        (SUBKEY_NOINHERIT, "NOINHERIT"),
        (SUBKEY_SCIENTIFIC, "SCIENTIFIC"),
    ] {
        let id = symbols.intern(spelling);
        assert_eq!(
            keywords.sub_keywords.index_of(id),
            Some(index),
            "SUB_KEYWORDS no longer holds {spelling} at {index}"
        );
    }
    // And the pair that proves the tables are not interchangeable: NOINHERIT is
    // a sub-keyword and NOT a sub-directive, while SYNTAX is the reverse.
    let noinherit = symbols.intern("NOINHERIT");
    let syntax = symbols.intern("SYNTAX");
    assert_eq!(keywords.sub_directives.index_of(noinherit), None);
    assert_eq!(keywords.sub_keywords.index_of(syntax), None);
}

#[test]
fn every_directive_keyword_reaches_its_node() {
    // One minimal legal instance per row of `directives[]`. Measured: all nine
    // are rc 0.
    for (text, expected) in [
        ("::annotate package\n", "ANNOTATE"),
        ("::attribute a\n", "ATTRIBUTE"),
        ("::class c\n", "CLASS"),
        ("::constant k 1\n", "CONSTANT"),
        ("::method m\n  return 1\n", "METHOD"),
        ("::options noprolog\n", "OPTIONS"),
        ("::requires \"nosuch\"\n", "REQUIRES"),
        ("::resource d\nbody\n::END\n", "RESOURCE"),
        ("::routine r\n  return 1\n", "ROUTINE"),
    ] {
        assert_eq!(keywords_of(&ok(text)), vec![expected], "{text:?}");
    }
}

/// The spelling after `::` resolves against `directives[]` and everything after
/// it against `subDirectives[]`, so a spelling in both tables means different
/// things in the two positions. Five spellings are in both.
#[test]
fn the_five_shared_spellings_mean_different_things_by_position() {
    // `CLASS` at the top level names a class; `CLASS` as an option of
    // `::METHOD` makes a class method. Measured: both files are rc 0.
    assert_eq!(
        shapes("::class c\n::method m class\n  return 1\n"),
        vec!["class \"C\"", "method \"M\" class body"]
    );
    assert_eq!(
        shapes("::attribute a\n::method m attribute\n"),
        vec!["attribute \"A\" both", "method \"M\" attribute"]
    );
    assert_eq!(
        shapes("::class c\n::constant k 1\n::annotate constant k x 1\n"),
        vec![
            "class \"C\"",
            "constant \"K\" text \"1\"",
            "annotate constant \"K\" X=\"1\"",
        ]
    );
    assert_eq!(
        shapes("::method m\n  return\n::annotate method m k 1\n"),
        vec!["method \"M\" body", "annotate method \"M\" K=\"1\""]
    );
    assert_eq!(
        shapes("::routine r\n  return\n::annotate routine r k 1\n"),
        vec!["routine \"R\" body", "annotate routine \"R\" K=\"1\""]
    );
    // And a directive spelling used as an OPTION where it is not one is
    // rejected. Measured: `::method m subclass object` is 25.902 and
    // `::routine r class` is 25.903.
    assert_eq!(err("::method m subclass object\n"), (25, 902));
    assert_eq!(err("::routine r class\n"), (25, 903));
}

/// A table pairing every sub-directive with a directive that carries it, and a
/// second pairing every one with a directive that does not.
///
/// Both directions, because a table of accepted cases alone would pass with an
/// option loop that accepted everything.
#[test]
fn every_sub_directive_is_reachable_and_every_one_is_refusable() {
    // Each row was run through `rexxc`. Thirty-eight are rc 0; the two that are
    // not are marked, and their non-zero rc is what proves the parse got past
    // this module.
    let accepted: [(&str, &str); 40] = [
        ("ABSTRACT", "::class c abstract\n"),
        ("ALL", "::options all syntax\n"),
        ("ATTRIBUTE", "::method m attribute\n"),
        // rc 157, Error 99.905 without the leading `::class`: the CLASS option
        // needs an active class, which the caller tracks and this module does
        // not.
        ("CLASS", "::class c\n::method m class\n  return 1\n"),
        ("CONDITION", "::options novalue condition\n"),
        (
            "CONSTANT",
            "::class c\n::constant k 1\n::annotate constant k x 1\n",
        ),
        ("DELEGATE", "::method m delegate p\n"),
        ("DIGITS", "::options digits 12\n"),
        ("END", "::resource d end stop\nline\nSTOP\n"),
        ("ERROR", "::options error syntax\n"),
        // rc 158, Error 98.903: the library does not exist, which is a run-time
        // failure of a program that parsed.
        ("EXTERNAL", "::routine r external \"LIBRARY x\"\n"),
        ("FAILURE", "::options failure syntax\n"),
        ("FORM", "::options form scientific\n"),
        ("FUZZ", "::options fuzz 3\n"),
        ("GET", "::attribute a get\n"),
        ("GUARDED", "::method m guarded\n  return 1\n"),
        (
            "INHERIT",
            "::class m mixinclass object\n::class c inherit m\n",
        ),
        ("LIBRARY", "::requires x library\n"),
        ("LOSTDIGITS", "::options lostdigits syntax\n"),
        ("METACLASS", "::class c metaclass class\n"),
        ("METHOD", "::method m\n  return\n::annotate method m k 1\n"),
        ("MIXINCLASS", "::class c mixinclass object\n"),
        ("NAMESPACE", "::requires \"nosuch\" namespace ns\n"),
        ("NOPROLOG", "::options noprolog\n"),
        ("NOSTRING", "::options nostring syntax\n"),
        ("NOTREADY", "::options notready syntax\n"),
        ("NOVALUE", "::options novalue syntax\n"),
        ("NUMERIC", "::options numeric inherit\n"),
        ("PACKAGE", "::annotate package k 1\n"),
        ("PRIVATE", "::class c private\n"),
        ("PROLOG", "::options prolog\n"),
        ("PROTECTED", "::method m protected\n  return 1\n"),
        ("PUBLIC", "::class c public\n"),
        (
            "ROUTINE",
            "::routine r\n  return\n::annotate routine r k 1\n",
        ),
        ("SET", "::attribute a set\n"),
        ("SUBCLASS", "::class c subclass object\n"),
        ("SYNTAX", "::options novalue syntax\n"),
        ("TRACE", "::options trace r\n"),
        ("UNGUARDED", "::method m unguarded\n  return 1\n"),
        ("UNPROTECTED", "::method m unprotected\n  return 1\n"),
    ];
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    // A row per table entry, so a sub-directive added to the table without a
    // test fails here rather than going unexercised.
    assert_eq!(accepted.len(), keywords.sub_directives.len());
    for (spelling, text) in accepted {
        let id = symbols.intern(spelling);
        assert!(
            keywords.sub_directives.index_of(id).is_some(),
            "{spelling} is not a sub-directive"
        );
        ok(text);
    }

    // The other direction. `::REQUIRES` accepts exactly two sub-directives, so
    // every other one is refused there with 25.904; the two it accepts are
    // refused by `::CLASS` with 25.901. Measured, all forty.
    for (spelling, _) in accepted {
        let (text, expected) = match spelling {
            "NAMESPACE" | "LIBRARY" => (format!("::class c {spelling}\n"), (25, 901)),
            _ => (format!("::requires x {spelling}\n"), (25, 904)),
        };
        assert_eq!(err(&text), expected, "{text:?}");
    }
}

#[test]
fn class_carries_its_options_and_rejects_a_second_of_each() {
    assert_eq!(shapes("::class c\n"), vec!["class \"C\""]);
    assert_eq!(shapes("::class \"c\"\n"), vec!["class \"c\""]);
    assert_eq!(shapes("::class c public\n"), vec!["class \"C\" public"]);
    assert_eq!(shapes("::class c private\n"), vec!["class \"C\" private"]);
    assert_eq!(shapes("::class c abstract\n"), vec!["class \"C\" abstract"]);
    assert_eq!(
        shapes("::class c subclass object\n"),
        vec!["class \"C\" subclass \"OBJECT\""]
    );
    assert_eq!(
        shapes("::class c mixinclass object\n"),
        vec!["class \"C\" mixinclass \"OBJECT\""]
    );
    assert_eq!(
        shapes("::class c metaclass class\n"),
        vec!["class \"C\" metaclass \"CLASS\""]
    );
    assert_eq!(
        shapes(
            "::class m1 mixinclass object\n::class m2 mixinclass object\n::class c inherit m1 m2\n"
        ),
        vec![
            "class \"M1\" mixinclass \"OBJECT\"",
            "class \"M2\" mixinclass \"OBJECT\"",
            "class \"C\" inherit \"M1\" inherit \"M2\"",
        ]
    );
    // A namespace-qualified reference, and a literal that is NOT one because
    // `parseClassReference` returns before it can look for a colon. Measured:
    // `::class c subclass rexx:object` is rc 0.
    assert_eq!(
        shapes("::class c subclass rexx:object\n"),
        vec!["class \"C\" subclass REXX:\"OBJECT\""]
    );
    assert_eq!(
        shapes("::class c subclass \"rexx:object\"\n"),
        vec!["class \"C\" subclass \"REXX:OBJECT\""]
    );

    // Every duplicate and every conflict, all 25.901. Measured, each.
    for text in [
        "::class c public private\n",
        "::class c public public\n",
        "::class c private public\n",
        "::class c abstract abstract\n",
        "::class c subclass object subclass object\n",
        "::class c subclass object mixinclass object\n",
        "::class c mixinclass object subclass object\n",
        "::class c metaclass class metaclass class\n",
        "::class c junk\n",
        "::class c package\n",
        "::class c guarded\n",
        "::class c, public\n",
    ] {
        assert_eq!(err(text), (25, 901), "{text:?}");
    }
    // And the "nothing there" errors, one number per keyword.
    assert_eq!(err("::class\n"), (19, 901));
    assert_eq!(err("::class c metaclass\n"), (19, 906));
    assert_eq!(err("::class c subclass\n"), (19, 907));
    assert_eq!(err("::class c mixinclass\n"), (19, 913));
    assert_eq!(err("::class c inherit\n"), (19, 908));
    assert_eq!(err("::class c subclass rexx:\n"), (20, 921));
}

#[test]
fn method_carries_its_options_and_decides_whether_a_body_follows() {
    assert_eq!(
        shapes("::method m\n  return 1\n"),
        vec!["method \"M\" body"]
    );
    // The name is NOT upcased, because it names the method object. Measured:
    // `::method "abc"` is rc 0.
    assert_eq!(
        shapes("::method \"abc\"\n  return 1\n"),
        vec!["method \"abc\" body"]
    );
    assert_eq!(
        shapes("::method m private\n  return 1\n"),
        vec!["method \"M\" private body"]
    );
    assert_eq!(
        shapes("::method m package\n  return 1\n"),
        vec!["method \"M\" package body"]
    );
    assert_eq!(
        shapes("::method m public protected unguarded\n  return 1\n"),
        vec!["method \"M\" public protected unguarded body"]
    );
    // Each generating option turns the body off, which is what makes a
    // following clause an error.
    assert_eq!(
        shapes("::method m attribute\n"),
        vec!["method \"M\" attribute"]
    );
    assert_eq!(
        shapes("::method m abstract\n"),
        vec!["method \"M\" abstract"]
    );
    assert_eq!(
        shapes("::method m delegate p\n"),
        vec!["method \"M\" delegate P"]
    );
    assert_eq!(
        shapes("::method m external \"LIBRARY foo bar\"\n"),
        vec!["method \"M\" library \"foo\":\"bar\""]
    );
    assert_eq!(
        shapes("::method m external \"LIBRARY foo\"\n"),
        vec!["method \"M\" library \"foo\""]
    );
    // The combinations that ARE legal. Measured: both rc 0.
    assert_eq!(
        shapes("::method m attribute abstract\n"),
        vec!["method \"M\" attribute abstract"]
    );
    assert_eq!(
        shapes("::method m delegate p attribute\n"),
        vec!["method \"M\" attribute delegate P"]
    );

    // Duplicates and conflicts, all 25.902.
    for text in [
        "::method m class class\n",
        "::method m public private\n",
        "::method m protected unprotected\n",
        "::method m guarded unguarded\n",
        "::method m attribute attribute\n",
        "::method m abstract abstract\n",
        "::method m abstract external \"LIBRARY x\"\n",
        "::method m external \"LIBRARY x\" abstract\n",
        "::method m delegate p delegate q\n",
        "::method m delegate p abstract\n",
        "::method m junk\n",
    ] {
        assert_eq!(err(text), (25, 902), "{text:?}");
    }
    assert_eq!(err("::method\n"), (19, 902));
    assert_eq!(err("::method m external nostring\n"), (19, 905));
    assert_eq!(err("::method m delegate \"p\"\n"), (20, 926));

    // A body where the shape generates the method, one number per shape.
    assert_eq!(err("::method m delegate p\n  return 1\n"), (99, 946));
    assert_eq!(err("::method m attribute\n  return 1\n"), (99, 934));
    assert_eq!(err("::method m abstract\n  return 1\n"), (99, 933));
    assert_eq!(
        err("::method m external \"LIBRARY x\"\n  return 1\n"),
        (99, 936)
    );
    // A following DIRECTIVE is fine, which is the other direction of the same
    // gate. Measured: rc 0.
    assert_eq!(
        keywords_of(&ok("::method m abstract\n::method n\n  return 1\n")),
        vec!["METHOD", "METHOD"]
    );
}

/// `::METHOD`'s plain external shape decodes the specification BEFORE checking
/// the body and its attribute shape checks the body first, so the two report
/// different errors for the same mistake. That ordering is measured, and it is
/// the kind of thing a single-error test cannot see.
#[test]
fn the_external_decode_happens_where_each_shape_does_it() {
    // Measured: 99.917 on line 1.
    assert_eq!(err("::method m external \"junk\"\n  return 1\n"), (99, 917));
    // Measured: 99.934 on line 2, the SAME source with ATTRIBUTE added.
    assert_eq!(
        err("::method m attribute external \"junk\"\n  return 1\n"),
        (99, 934)
    );
    // With no body the attribute shape reaches the decode after all.
    assert_eq!(err("::method m attribute external \"junk\"\n"), (99, 917));
    // `::ATTRIBUTE` checks the body first in every shape. Measured: 99.935 on
    // line 2.
    assert_eq!(
        err("::attribute a get external \"junk\"\n  return 1\n"),
        (99, 935)
    );
    assert_eq!(err("::attribute a get external \"junk\"\n"), (99, 917));
    // `::ROUTINE` decodes first, like `::METHOD`'s plain shape.
    assert_eq!(
        err("::routine r external \"junk\"\n  return 1\n"),
        (99, 917)
    );
}

#[test]
fn an_external_specification_is_one_keyword_and_one_or_two_names() {
    // The first word is upcased and the rest are not, and blanks or tabs
    // separate them. Measured: `"  library   x  "` loads the library `x`.
    assert_eq!(
        shapes("::method m external \"  library   x  \"\n"),
        vec!["method \"M\" library \"x\""]
    );
    assert_eq!(
        shapes("::method m external \"\tLIBRARY\tx\"\n"),
        vec!["method \"M\" library \"x\""]
    );
    // `REGISTERED` is a routine spelling only. Measured:
    // `::routine r external "registered x"` gets past the parse to 90.999 while
    // `::method m external "REGISTERED x"` is 99.917.
    assert_eq!(
        shapes("::routine r external \"registered x\"\n"),
        vec!["routine \"R\" registered \"x\""]
    );
    assert_eq!(
        shapes("::routine r external \"REGISTERED x y\"\n"),
        vec!["routine \"R\" registered \"x\":\"y\""]
    );
    assert_eq!(err("::method m external \"REGISTERED x\"\n"), (99, 917));
    // Every rejected shape, all 99.917. Measured, each.
    for text in [
        "::method m external \"\"\n",
        "::method m external \"LIBRARY\"\n",
        "::method m external \"LIBRARY a b c\"\n",
        "::method m external \"junk\"\n",
        "::method m external \"libraryx x\"\n",
        "::routine r external \"REGISTERED\"\n",
        "::routine r external \"REGISTERED a b c\"\n",
    ] {
        assert_eq!(err(text), (99, 917), "{text:?}");
    }
}

#[test]
fn attribute_asks_whether_a_body_follows_only_for_get_and_set() {
    assert_eq!(shapes("::attribute a\n"), vec!["attribute \"A\" both"]);
    assert_eq!(shapes("::attribute a get\n"), vec!["attribute \"A\" get"]);
    assert_eq!(shapes("::attribute a set\n"), vec!["attribute \"A\" set"]);
    // The one place a directive's parse depends on the clause AFTER it: both
    // spellings are rc 0 and they mean different things.
    assert_eq!(
        shapes("::attribute a get\n  return 1\n"),
        vec!["attribute \"A\" get body"]
    );
    assert_eq!(
        shapes("::attribute a set\n  return 1\n"),
        vec!["attribute \"A\" set body"]
    );
    // And a following directive leaves the body off, which is the other
    // direction.
    assert_eq!(
        shapes("::attribute a get\n::attribute b set\n"),
        vec!["attribute \"A\" get", "attribute \"B\" set"]
    );
    // BOTH never takes a body, whatever follows. Measured: 99.937.
    assert_eq!(err("::attribute a\n  return 1\n"), (99, 937));
    assert_eq!(
        err("::attribute a external \"junk\"\n  return 1\n"),
        (99, 937)
    );
    // A generating option on GET/SET takes the body away again, one number per
    // option. Measured, each.
    assert_eq!(err("::attribute a get abstract\n  return 1\n"), (99, 940));
    assert_eq!(err("::attribute a get delegate p\n  return 1\n"), (99, 947));
    assert_eq!(
        err("::attribute a set external \"LIBRARY x\"\n  return 1\n"),
        (99, 935)
    );
    // The full option set.
    assert_eq!(
        shapes("::class c\n::attribute a get class private protected unguarded\n"),
        vec![
            "class \"C\"",
            "attribute \"A\" get class private protected unguarded",
        ]
    );
    assert_eq!(
        shapes("::attribute a package\n"),
        vec!["attribute \"A\" both package"]
    );
    // Duplicates and conflicts, all 25.925.
    for text in [
        "::attribute a get set\n",
        "::attribute a set get\n",
        "::attribute a get get\n",
        "::attribute a class class\n",
        "::attribute a public private\n",
        "::attribute a protected unprotected\n",
        "::attribute a guarded unguarded\n",
        "::attribute a abstract abstract\n",
        "::attribute a abstract delegate p\n",
        "::attribute a inherit x\n",
        "::attribute a junk\n",
    ] {
        assert_eq!(err(text), (25, 925), "{text:?}");
    }
    assert_eq!(err("::attribute\n"), (19, 914));
}

#[test]
fn a_constant_takes_a_name_a_value_or_an_expression() {
    // No value at all, whose value is the name as written.
    assert_eq!(shapes("::constant c\n"), vec!["constant \"C\" <name>"]);
    assert_eq!(
        shapes("::constant c 5\n"),
        vec!["constant \"C\" text \"5\""]
    );
    assert_eq!(
        shapes("::constant c \"x\"\n"),
        vec!["constant \"C\" text \"x\""]
    );
    assert_eq!(
        shapes("::constant c abc\n"),
        vec!["constant \"C\" text \"ABC\""]
    );
    // The signed form, with the blank dropped by the concatenation. Measured:
    // `::constant c + 5` is rc 0.
    assert_eq!(
        shapes("::constant c -5\n"),
        vec!["constant \"C\" text \"-5\""]
    );
    assert_eq!(
        shapes("::constant c + 5\n"),
        vec!["constant \"C\" text \"+5\""]
    );
    assert_eq!(
        shapes("::constant c -.5\n"),
        vec!["constant \"C\" text \"-.5\""]
    );
    assert_eq!(
        shapes("::constant c -1e2\n"),
        vec!["constant \"C\" text \"-1E2\""]
    );
    // The parenthesised form, which admits a comma list because it is a
    // REQUIRED expression. Measured: `(1+2)` and `(1,2)` both get past the parse
    // to 99.906, which needs an active `::CLASS` this module does not keep.
    assert_eq!(
        shapes("::class d\n::constant c (1+2)\n"),
        vec!["class \"D\"", "constant \"C\" expr (+ 1 2)"]
    );
    assert_eq!(
        shapes("::class d\n::constant c (1,2)\n"),
        vec!["class \"D\"", "constant \"C\" expr (list 1 2)"]
    );
    // Every rejection, one number each. Measured.
    assert_eq!(err("::constant\n"), (19, 915));
    assert_eq!(err("::constant c *5\n"), (19, 916));
    assert_eq!(err("::constant c -abc\n"), (19, 916));
    assert_eq!(err("::constant c -\"5\"\n"), (19, 916));
    assert_eq!(err("::constant c -.true\n"), (19, 916));
    assert_eq!(err("::constant c -5x\n"), (19, 916));
    assert_eq!(err("::constant c -1e\n"), (19, 916));
    assert_eq!(err("::constant c 5 6\n"), (21, 913));
    assert_eq!(err("::constant c ()\n"), (35, 936));
    assert_eq!(err("::constant c (1+2\n"), (36, 901));
    assert_eq!(err("::constant c 5\n  return 1\n"), (99, 938));
}

#[test]
fn an_annotation_is_a_target_and_a_list_of_symbol_value_pairs() {
    assert_eq!(shapes("::annotate package\n"), vec!["annotate package"]);
    assert_eq!(
        shapes("::annotate package a 1 b \"x\"\n"),
        vec!["annotate package A=\"1\" B=\"x\""]
    );
    // The signed value form, shared with `::CONSTANT` but without the
    // parenthesised one.
    assert_eq!(
        shapes("::annotate package a -1\n"),
        vec!["annotate package A=\"-1\""]
    );
    // Every target, each with a real object to annotate so that the oracle's
    // rc 0 covers the whole file.
    assert_eq!(
        shapes("::class c\n::annotate class c k 1\n"),
        vec!["class \"C\"", "annotate class \"C\" K=\"1\""]
    );
    assert_eq!(
        shapes("::attribute a\n::annotate attribute a k 1\n"),
        vec!["attribute \"A\" both", "annotate attribute \"A\" K=\"1\""]
    );
    // A lower-case target name is upcased, unlike a `::METHOD` name.
    assert_eq!(
        shapes("::method \"m\"\n  return\n::annotate method \"m\" k 1\n"),
        vec!["method \"m\" body", "annotate method \"M\" K=\"1\""]
    );
    // Rejections. Measured, each.
    assert_eq!(err("::annotate\n"), (20, 924));
    assert_eq!(err("::annotate \"package\"\n"), (20, 924));
    assert_eq!(err("::annotate junk k 1\n"), (25, 928));
    // A symbol that IS a sub-directive but not one of the six targets takes the
    // same arm, which is the other side of the 20.924 gate.
    assert_eq!(err("::annotate public k 1\n"), (25, 928));
    assert_eq!(err("::annotate class\n"), (19, 925));
    assert_eq!(err("::annotate package \"a\" 1\n"), (20, 919));
    // The missing-value and bad-value errors are DIFFERENT numbers, which is
    // the pair a single test would have conflated.
    assert_eq!(err("::annotate package a\n"), (19, 924));
    assert_eq!(err("::annotate package a *\n"), (19, 923));
    assert_eq!(err("::annotate package a -abc\n"), (19, 923));
    assert_eq!(err("::annotate package a -5x\n"), (19, 923));
}

#[test]
fn every_package_option_reaches_its_own_variant() {
    assert_eq!(
        shapes("::options digits 12 fuzz 3\n"),
        vec!["options digits 12 fuzz 3"]
    );
    // A bare `::OPTIONS` is legal and carries nothing. Measured: rc 0.
    assert_eq!(shapes("::options\n"), vec!["options"]);
    assert_eq!(
        shapes("::options form scientific\n"),
        vec!["options form scientific"]
    );
    assert_eq!(
        shapes("::options form engineering\n"),
        vec!["options form engineering"]
    );
    assert_eq!(shapes("::options trace r\n"), vec!["options trace \"R\""]);
    assert_eq!(shapes("::options prolog\n"), vec!["options prolog"]);
    assert_eq!(shapes("::options noprolog\n"), vec!["options noprolog"]);
    assert_eq!(
        shapes("::options numeric inherit\n"),
        vec!["options numeric inherit"]
    );
    assert_eq!(
        shapes("::options numeric noinherit\n"),
        vec!["options numeric noinherit"]
    );
    // The seven condition options, both settings each. Measured: all fourteen
    // are rc 0.
    for keyword in [
        "all",
        "error",
        "failure",
        "lostdigits",
        "nostring",
        "notready",
        "novalue",
    ] {
        for setting in ["syntax", "condition"] {
            let text = format!("::options {keyword} {setting}\n");
            assert_eq!(
                shapes(&text),
                vec![format!("options {keyword}={setting}")],
                "{text:?}"
            );
        }
    }
    // `NOVALUE ERROR` is a spelling of `NOVALUE SYNTAX`, kept for backwards
    // compatibility, and it is the ONLY one: measured, `::options novalue error`
    // is rc 0 while `::options error error` and `::options all error` are
    // 25.927.
    assert_eq!(
        shapes("::options novalue error\n"),
        vec!["options novalue=syntax"]
    );
    for keyword in [
        "all",
        "error",
        "failure",
        "lostdigits",
        "nostring",
        "notready",
    ] {
        assert_eq!(err(&format!("::options {keyword} error\n")), (25, 927));
    }
    // The order is kept, because a later option overrides an earlier one:
    // measured with `build/bin/rexx`, `::options digits 12` then
    // `::options digits 5` makes `digits()` report 5.
    assert_eq!(
        shapes("::options digits 12 digits 5\n"),
        vec!["options digits 12 digits 5"]
    );
}

#[test]
fn the_options_arguments_are_gated_the_way_the_oracle_gates_them() {
    // DIGITS must exceed zero and FUZZ need not, which is one number apart.
    assert_eq!(shapes("::options fuzz 0\n"), vec!["options fuzz 0"]);
    assert_eq!(err("::options digits 0\n"), (26, 5));
    // Eighteen digits is the boundary, `Numerics::ARGUMENT_DIGITS` on a 64-bit
    // build.
    assert_eq!(
        shapes("::options digits 123456789012345678\n"),
        vec!["options digits 123456789012345678"]
    );
    assert_eq!(err("::options digits 1234567890123456789\n"), (26, 5));
    // Every numeric rejection.
    for text in [
        "::options digits abc\n",
        "::options digits \"9.5\"\n",
        "::options digits \"\"\n",
        "::options digits 1E18\n",
        "::options digits \"1e-2\"\n",
    ] {
        assert_eq!(err(text), (26, 5), "{text:?}");
    }
    assert_eq!(err("::options fuzz abc\n"), (26, 6));
    assert_eq!(err("::options fuzz \"-1\"\n"), (26, 6));
    // A number written any way Rexx writes one, including with blanks around it.
    // Measured: `::options digits " 9 "` is rc 0.
    for text in [
        "::options digits \" 9 \"\n",
        "::options digits \"+9\"\n",
        "::options digits \"9.0\"\n",
        "::options digits \"9.\"\n",
        "::options digits \"0009\"\n",
        "::options digits 1e2\n",
    ] {
        ok(text);
    }
    // A `-` is neither a symbol nor a literal, so it never reaches the number
    // test: measured, `::options digits -1` is 19.917 and not 26.5.
    assert_eq!(err("::options digits -1\n"), (19, 917));
    assert_eq!(err("::options digits\n"), (19, 917));
    assert_eq!(err("::options fuzz -1\n"), (19, 918));
    assert_eq!(err("::options trace -1\n"), (19, 919));
    assert_eq!(err("::options trace\n"), (19, 919));
    assert_eq!(err("::options trace zzz\n"), (24, 1));
    // FORM and NUMERIC resolve their argument against `subKeywords[]`, which
    // makes two things observable: NOINHERIT works though it is not a
    // sub-directive, and SYNTAX fails though it is.
    assert_eq!(err("::options numeric syntax\n"), (25, 935));
    assert_eq!(err("::options numeric \"inherit\"\n"), (20, 935));
    assert_eq!(err("::options numeric\n"), (20, 935));
    assert_eq!(err("::options form value\n"), (25, 11));
    assert_eq!(err("::options form \"scientific\"\n"), (20, 925));
    assert_eq!(err("::options form\n"), (20, 925));
    assert_eq!(err("::options novalue\n"), (20, 929));
    assert_eq!(err("::options novalue \"syntax\"\n"), (20, 929));
    assert_eq!(err("::options junk\n"), (25, 924));
    assert_eq!(err("::options \"digits\" 9\n"), (25, 924));
}

#[test]
fn requires_takes_a_name_and_one_of_two_mutually_exclusive_options() {
    assert_eq!(
        shapes("::requires \"nosuch\"\n"),
        vec!["requires \"nosuch\""]
    );
    // A symbol name arrives upcased and a literal does not.
    assert_eq!(shapes("::requires x\n"), vec!["requires \"X\""]);
    assert_eq!(
        shapes("::requires x library\n"),
        vec!["requires \"X\" library"]
    );
    assert_eq!(
        shapes("::requires \"nosuch\" namespace ns\n"),
        vec!["requires \"nosuch\" namespace NS"]
    );
    // Both directions of the exclusion, in both orders. Measured: 25.904.
    for text in [
        "::requires x library namespace ns\n",
        "::requires x namespace ns library\n",
        "::requires x library library\n",
        "::requires x namespace a namespace b\n",
        "::requires x junk\n",
    ] {
        assert_eq!(err(text), (25, 904), "{text:?}");
    }
    assert_eq!(err("::requires\n"), (19, 904));
    assert_eq!(err("::requires \"x\" namespace \"ns\"\n"), (20, 920));
    // REXX is reserved, whatever its case, because the symbol is upcased first.
    assert_eq!(err("::requires \"x\" namespace rexx\n"), (99, 944));
    assert_eq!(err("::requires \"x\" namespace REXX\n"), (99, 944));
}

#[test]
fn routine_takes_an_access_option_and_an_external_specification() {
    assert_eq!(
        shapes("::routine r\n  return 1\n"),
        vec!["routine \"R\" body"]
    );
    // The name is not upcased, because a quoted routine name is looked up
    // case-sensitively.
    assert_eq!(
        shapes("::routine \"r\"\n  return 1\n"),
        vec!["routine \"r\" body"]
    );
    assert_eq!(
        shapes("::routine r public\n  return 1\n"),
        vec!["routine \"R\" public body"]
    );
    assert_eq!(
        shapes("::routine r private\n  return 1\n"),
        vec!["routine \"R\" private body"]
    );
    // An external routine has no body.
    assert_eq!(
        shapes("::routine r external \"LIBRARY x\"\n"),
        vec!["routine \"R\" library \"x\""]
    );
    assert_eq!(
        err("::routine r external \"LIBRARY x\"\n  return 1\n"),
        (99, 939)
    );
    for text in [
        "::routine r public private\n",
        "::routine r public public\n",
        "::routine r package\n",
        "::routine r class\n",
        "::routine r junk\n",
    ] {
        assert_eq!(err(text), (25, 903), "{text:?}");
    }
    assert_eq!(err("::routine\n"), (19, 903));
    assert_eq!(err("::routine r external nostring\n"), (19, 905));
    // A SECOND external on a `::ROUTINE` reports the `::CLASS` number, because
    // `routineDirective` passes the wrong error code. Measured: 25.901, not
    // 25.903. The interpreter defines the behaviour.
    assert_eq!(
        err("::routine r external \"LIBRARY x\" external \"LIBRARY y\"\n"),
        (25, 901)
    );
}

#[test]
fn a_resource_carries_its_body_verbatim() {
    assert_eq!(
        shapes("::resource d\nfirst\nsecond\n::END\n"),
        vec!["resource \"D\" end \"::END\" line \"first\" line \"second\""]
    );
    // An empty body is legal.
    assert_eq!(
        shapes("::resource d\n::END\n"),
        vec!["resource \"D\" end \"::END\""]
    );
    // A named marker, upcased when it comes from a symbol and verbatim when it
    // comes from a literal. Measured: `end stop` is closed by `STOP` and not by
    // `stop`.
    assert_eq!(
        shapes("::resource d end stop\nbody\nSTOP\n"),
        vec!["resource \"D\" end \"STOP\" line \"body\""]
    );
    assert_eq!(
        shapes("::resource d end \"%%\"\nbody\n%%\n"),
        vec!["resource \"D\" end \"%%\" line \"body\""]
    );
    // The body is NOT Rexx and must not be scanned as Rexx. Measured: rc 0.
    assert_eq!(
        shapes("::resource d\nthis is 'unmatched and /* unclosed\n::END\n"),
        vec!["resource \"D\" end \"::END\" line \"this is 'unmatched and /* unclosed\""]
    );
    // A `;` ends the directive clause and the rest of that line is skipped
    // entirely rather than parsed. Measured: rc 0 and a one-line resource.
    assert_eq!(
        shapes("::resource d; say 'x'\nbody\n::END\n"),
        vec!["resource \"D\" end \"::END\" line \"body\""]
    );
    // Two resources in one file, each finding its own body by the index of its
    // own `::` token.
    assert_eq!(
        shapes("::resource a\nfirst\n::END\n::resource b\nsecond\n::END\n"),
        vec![
            "resource \"A\" end \"::END\" line \"first\"",
            "resource \"B\" end \"::END\" line \"second\"",
        ]
    );
    // And a directive after the body is reached normally.
    assert_eq!(
        keywords_of(&ok("::resource d\nbody\n::END\n::method m\n  return 1\n")),
        vec!["RESOURCE", "METHOD"]
    );

    // Rejections. Measured, each.
    assert_eq!(err("::resource\n"), (19, 920));
    assert_eq!(err("::resource d junk\nbody\n::END\n"), (25, 926));
    // A sub-directive that IS in the table but is not END, which is the other
    // half of the same gate: a test using only an unknown symbol passes with a
    // check that merely requires a sub-directive. Measured: 25.926.
    assert_eq!(err("::resource d public x\nbody\n::END\n"), (25, 926));
    assert_eq!(err("::resource d \"end\" x\nbody\n::END\n"), (25, 926));
    assert_eq!(err("::resource d end\nbody\n::END\n"), (19, 921));
    assert_eq!(err("::resource d end \"x\" extra\nbody\nx\n"), (21, 914));
    // The missing marker is the scanner's error, 99.943, and it is a whole-file
    // failure rather than one directive's. Both the missing and the mis-cased
    // marker reach it.
    assert_eq!(err("::resource d\nbody\n"), (99, 943));
    assert_eq!(err("::resource d\nbody\n::end\n"), (99, 943));
    assert_eq!(err("::resource d end stop\nbody\nstop\n"), (99, 943));
}

#[test]
fn a_clause_that_is_not_a_directive_and_a_directive_that_is_not_known() {
    // `nextDirective`'s own two guards, both measured.
    assert_eq!(err("::\n"), (20, 916));
    assert_eq!(err(":: \"x\"\n"), (20, 916));
    // A symbol that is not a directive keyword is 99.916 and NOT 20.916, which
    // is the pair a test of one number would have conflated. Measured: `::junk`
    // and `:: 5` both report `Unrecognized directive instruction`.
    assert_eq!(err("::junk\n"), (99, 916));
    assert_eq!(err(":: 5\n"), (99, 916));
    // A single colon is not a directive at all. Measured: `:junk` is 35.1, from
    // the expression grammar, which is what proves the DCOLON test is on the
    // token class and not on the character.
    assert_eq!(err("nop\n:junk\n"), (35, 1));
}

/// `getRetriever`'s name check, 99.925, at all four of its call sites.
///
/// It is a purely local check on the name's own text, and it is easy to miss
/// because the `syntaxError` lives in `LanguageParser.cpp` rather than in the
/// 2,867 lines of `DirectiveParser.cpp`. Both directions, and the controls that
/// say where the check is NOT applied.
#[test]
fn an_attribute_name_must_be_a_variable_name() {
    // The three kinds a variable name can be, all rc 0.
    ok("::attribute a\n");
    ok("::attribute a.\n");
    ok("::attribute a.b\n");
    ok("::attribute \"aB\"\n");
    // And the kinds it cannot be. Measured: all 99.925.
    for text in [
        "::attribute 3\n",
        "::attribute .a\n",
        "::attribute \"3\"\n",
        "::attribute \"a b\"\n",
        "::attribute \"a-b\"\n",
        "::attribute \"1e+5\"\n",
        "::attribute 3 get\n",
        "::attribute 3 abstract\n",
    ] {
        assert_eq!(err(text), (99, 925), "{text:?}");
    }
    // `scanSymbol` looks for an exponent before giving up, so a sign after an
    // `E` survives and the compound test is reached anyway. Measured: rc 0.
    ok("::attribute \"a.e+5\"\n");
    // The length bound. Measured: 250 bytes is rc 0 and 251 is 99.925.
    let name = "a".repeat(250);
    ok(&format!("::attribute \"{name}\"\n"));
    let name = "a".repeat(251);
    assert_eq!(err(&format!("::attribute \"{name}\"\n")), (99, 925));

    // A DELEGATE target is checked too, on both directives.
    assert_eq!(err("::method m delegate 5\n"), (99, 925));
    assert_eq!(err("::method m delegate .p\n"), (99, 925));
    assert_eq!(err("::attribute a delegate 5\n"), (99, 925));
    assert_eq!(err("::attribute a get delegate 5\n"), (99, 925));
    ok("::method m delegate p\n");
    ok("::attribute a delegate p.\n");

    // A plain `::METHOD` name is NOT checked, which is the control that says
    // this is the attribute rule and not a name rule. Measured: rc 0.
    ok("::method 3\n  return 1\n");
    ok("::method .a\n  return 1\n");
    // With ATTRIBUTE it is checked, but only where the accessor pair is
    // generated. Measured: `::method 3 attribute` is 99.925, while the same with
    // ABSTRACT is rc 0 and with EXTERNAL reaches 98.903, a library-load failure
    // past the parse.
    assert_eq!(err("::method 3 attribute\n"), (99, 925));
    assert_eq!(err("::method .a attribute\n"), (99, 925));
    ok("::method 3 attribute abstract\n");
    ok("::method 3 attribute external \"LIBRARY x\"\n");
}

/// Where the name check sits relative to the body check, which differs per shape
/// and is only visible when the two disagree.
#[test]
fn the_name_check_happens_where_each_shape_does_it() {
    // `::ATTRIBUTE` checks the name before everything. Measured: 99.925 on
    // line 1, not 99.937 on line 2.
    assert_eq!(err("::attribute 3\n  return 1\n"), (99, 925));
    assert_eq!(err("::attribute 3 external \"LIBRARY x\"\n"), (99, 925));
    // `::METHOD DELEGATE` too. Measured: 99.925 on line 1, not 99.946.
    assert_eq!(err("::method m delegate 5\n  return 1\n"), (99, 925));
    // `::METHOD ATTRIBUTE` is the other way round. Measured: 99.934 on line 2.
    assert_eq!(err("::method 3 attribute\n  return 1\n"), (99, 934));
    // And so is `::ATTRIBUTE`'s BOTH style for its DELEGATE target, where
    // GET/SET is not. Measured: 99.937 on line 2 against 99.925 on line 1.
    assert_eq!(err("::attribute a delegate 5\n  return 1\n"), (99, 937));
    assert_eq!(err("::attribute a get delegate 5\n  return 1\n"), (99, 925));
}

#[test]
fn a_directive_spans_its_own_clause() {
    // The span is the clause, so an explicit `;` is inside it and the blanks
    // after it are not, exactly as for an instruction.
    let text = "::class c;\n::class d   \n";
    let directives = ok(text);
    let spans: Vec<&str> = directives
        .iter()
        .map(|d| &text[d.clause_span.clone()])
        .collect();
    assert_eq!(spans, vec!["::class c;", "::class d   "]);
}

/// A `checkDirective` error is reported against the OFFENDING clause and every
/// other directive error against the directive's own clause. Two positions, and
/// only a source with blank lines between them can tell them apart.
#[test]
fn a_body_error_is_reported_against_the_body_and_not_the_directive() {
    // `checkDirective` saves `clauseLocation` and restores it only AFTER the
    // error, so `nextClause()` has already moved it. The oracle prints the
    // WHOLE message, and it names line 5:
    //
    //   5 *-* return 1
    //   Error 99 running ... line 5:  Translation error.
    //   Error 99.933:  Abstract methods cannot have a method body.
    //
    // The blank lines are what make the two positions distinguishable: without
    // them the directive's byte and the body's byte sit on adjacent lines and
    // moving one moves the other.
    let text = "::method m abstract\n\n\n\n  return 1\n";
    assert_eq!(err(text), (99, 933));
    let offender = text.find("  return").expect("the body clause is there");
    assert_eq!(
        err_byte(text),
        offender + 2,
        "reported against the directive rather than the body"
    );
    // And the directive's own errors go the other way, against the directive's
    // first byte. Measured: `::method m junk` on line 5 reports line 5.
    let text = "::class c\n\n\n\n::method m junk\n";
    assert_eq!(err(text), (25, 902));
    assert_eq!(err_byte(text), text.find("::method").expect("line 5"));
}

/// `CoreClasses.orx` is the acceptance test for this task: 347 of its 4,193
/// lines start with `::`, and every method body in the file sits inside one, so
/// nothing in the file parses until the directives do.
#[test]
fn core_classes_parses() {
    const CORE_CLASSES: &str =
        include_str!("../../../../../interpreter/RexxClasses/CoreClasses.orx");
    let directives =
        parse(CORE_CLASSES).unwrap_or_else(|e| panic!("CoreClasses.orx failed to parse: {e:?}"));
    // The pinned counts, so a change that silently stops producing directives
    // fails here rather than passing with fewer nodes.
    assert_eq!(directives.len(), 347);
    let mut classes = 0;
    let mut methods = 0;
    let mut attributes = 0;
    for directive in &directives {
        match directive.kind.keyword() {
            "CLASS" => classes += 1,
            "METHOD" => methods += 1,
            "ATTRIBUTE" => attributes += 1,
            other => panic!("CoreClasses.orx holds a {other} directive"),
        }
        // Every span is inside the file and non-empty, so a mis-split cannot
        // pass by producing the right COUNT of wrong nodes.
        assert!(
            directive.clause_span.start < directive.clause_span.end
                && directive.clause_span.end <= CORE_CLASSES.len(),
            "span {:?} is empty or out of range",
            directive.clause_span
        );
    }
    assert_eq!((classes, methods, attributes), (32, 303, 12));
}

/// The other shipped package, so that the acceptance test is not one file's
/// habits. `StreamClasses.orx` adds `::CONSTANT`, which `CoreClasses.orx` never
/// uses: 7 `::CLASS`, 139 `::METHOD`, 5 `::ATTRIBUTE` and 2 `::CONSTANT`.
#[test]
fn the_other_shipped_packages_parse() {
    /// How many of each directive a package holds, in the order
    /// `::CLASS`, `::METHOD`, `::ATTRIBUTE`, `::CONSTANT`.
    struct Counts {
        classes: usize,
        methods: usize,
        attributes: usize,
        constants: usize,
    }

    // Asserted rather than described: an earlier version of this test carried
    // these numbers in its doc comment and asserted only that the list was
    // non-empty.
    const PACKAGES: &[(&str, &str, Counts)] = &[(
        "StreamClasses.orx",
        include_str!("../../../../../interpreter/RexxClasses/StreamClasses.orx"),
        Counts {
            classes: 7,
            methods: 139,
            attributes: 5,
            constants: 2,
        },
    )];
    for (name, source, expected) in PACKAGES {
        let directives = parse(source).unwrap_or_else(|e| panic!("{name} failed to parse: {e:?}"));
        let mut classes = 0;
        let mut methods = 0;
        let mut attributes = 0;
        let mut constants = 0;
        for directive in &directives {
            match directive.kind.keyword() {
                "CLASS" => classes += 1,
                "METHOD" => methods += 1,
                "ATTRIBUTE" => attributes += 1,
                "CONSTANT" => constants += 1,
                other => panic!("{name} holds a {other} directive"),
            }
        }
        assert_eq!(
            (classes, methods, attributes, constants),
            (
                expected.classes,
                expected.methods,
                expected.attributes,
                expected.constants
            ),
            "{name} decomposed differently"
        );
    }
}
