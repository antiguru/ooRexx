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

//! `Method`'s and `Routine`'s own readers -- `classes/MethodClass.cpp`,
//! `classes/RoutineClass.cpp` and the `BaseExecutable` rows both share, bound
//! by `memory/Setup.cpp:1089`-`:1113` and `:1128`-`:1143`.
//!
//! A child of [`super`] for the reason [`super::rexx_info`] is one: the rows
//! point at [`super::NativeMethod`]s, whose parameter list names types only
//! that module can.
//!
//! # Both classes read one thing, and it is not the object
//!
//! `BaseExecutable::source` and `BaseExecutable::getPackage` are
//! `code->getSource()` and `code->getPackage()`, and the seven `is*` flags are
//! a word on the method object that the *directive* filled in. So every reader
//! here starts from [`crate::ExecutableSource`] -- which directive declared
//! this object -- and the answer for an object with no directive is
//! `BaseCode`'s: an empty `Array` (`execution/BaseCode.cpp:120`) and the
//! `REXX` package.

use super::{Arity, Cleared, Failure, Interp, Loud, NativeMethod, ObjRef, Raised, array_of_texts};
use crate::plan::Package;
use crate::{ExecutableSource, ProgramId};
use rexx_parse::{Access, DirectiveKind, GuardOption, Program, Protection};

/// `Method`'s and `Routine`'s instance methods. Chained into
/// `ObjectModel::build` beside [`super::NATIVE_METHODS`].
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("Method", "ISABSTRACT", Arity::Fixed(0), is_abstract),
    ("Method", "ISATTRIBUTE", Arity::Fixed(0), is_attribute),
    ("Method", "ISCONSTANT", Arity::Fixed(0), is_constant),
    ("Method", "ISGUARDED", Arity::Fixed(0), is_guarded),
    ("Method", "ISPACKAGE", Arity::Fixed(0), is_package),
    ("Method", "ISPRIVATE", Arity::Fixed(0), is_private),
    ("Method", "ISPROTECTED", Arity::Fixed(0), is_protected),
    ("Method", "PACKAGE", Arity::Fixed(0), package),
    ("Method", "SETGUARDED", Arity::Fixed(0), set_guarded),
    ("Method", "SETPRIVATE", Arity::Fixed(0), set_private),
    ("Method", "SETPROTECTED", Arity::Fixed(0), set_protected),
    (
        "Method",
        "SETSECURITYMANAGER",
        Arity::Fixed(1),
        set_security_manager,
    ),
    ("Method", "SETUNGUARDED", Arity::Fixed(0), set_unguarded),
    ("Method", "SOURCE", Arity::Fixed(0), source),
    ("Routine", "[]", Arity::Counted, call),
    ("Routine", "CALL", Arity::Counted, call),
    ("Routine", "CALLWITH", Arity::Fixed(1), call_with),
    ("Routine", "PACKAGE", Arity::Fixed(0), package),
    (
        "Routine",
        "SETSECURITYMANAGER",
        Arity::Fixed(1),
        set_security_manager,
    ),
    ("Routine", "SOURCE", Arity::Fixed(0), source),
];

/// Which flag one of the `Method` readers answers.
///
/// `UNGUARDED_FLAG` is the stored bit and `isGuarded()` is its negation
/// (`classes/MethodClass.hpp:113`), which is why the default answer is `1`
/// for this one flag and `0` for the other six.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Flag {
    Abstract,
    Attribute,
    Constant,
    Unguarded,
    Package,
    Private,
    Protected,
}

/// What the four setters have changed on one `Method` object, over what its
/// directive declared.
///
/// Three fields and not four: `setGuarded` and `setUnguarded` write the one
/// `UNGUARDED_FLAG` in both directions, while `setPrivate` and `setProtected`
/// only ever set their bit -- there is no `setPublic` or `setUnprotected` row
/// on `Method` (`memory/Setup.cpp:1097`-`:1108`).
#[derive(Copy, Clone, Default)]
pub(crate) struct MethodFlagWrites {
    pub(crate) unguarded: Option<bool>,
    pub(crate) private: bool,
    pub(crate) protected: bool,
}

/// The `ExecutableSource` this receiver reports on, or the refusal for a
/// receiver this crate did not build.
///
/// The refusal is an internal inconsistency rather than a program's doing: a
/// receiver reaches here only by resolving one of these names at `Method` or
/// `Routine`, and every object of either class this crate hands out is
/// recorded as it is built.
fn source_of(interp: &Interp, receiver: ObjRef) -> Result<ExecutableSource, Failure> {
    interp
        .executable_sources
        .get(&receiver)
        .map(|record| record.source)
        .ok_or_else(|| Loud::receiver_class("an executable this crate did not build").into())
}

/// One flag's value for this receiver: what the directive declared, with any
/// write the setters have made over it.
fn flag(interp: &Interp, receiver: ObjRef, which: Flag) -> Result<bool, Failure> {
    let declared = declared_flag(interp, source_of(interp, receiver)?, which);
    let written = interp.method_flag_writes.get(&receiver).copied();
    Ok(match (which, written) {
        (Flag::Unguarded, Some(writes)) => writes.unguarded.unwrap_or(declared),
        (Flag::Private, Some(writes)) => declared || writes.private,
        (Flag::Protected, Some(writes)) => declared || writes.protected,
        _ => declared,
    })
}

/// One flag as the declaring directive set it -- `setAttributes` and the
/// `setAbstract`/`setAttribute`/`setConstant` calls beside it
/// (`parser/DirectiveParser.cpp:2413`-`:2525`).
fn declared_flag(interp: &Interp, source: ExecutableSource, which: Flag) -> bool {
    let ExecutableSource::Directive { program, directive } = source else {
        // A primitive carries the flag word `new MethodClass` left it: every
        // bit clear, which `isGuarded` reads as guarded. Measured, oracle rc
        // 0: `.Object~method('objectName')` answers `0 0 0 1 0 0 0` for the
        // seven readers in the order they are declared here.
        return false;
    };
    let Some(directive) = interp
        .programs
        .get(program.0)
        .and_then(|program| program.directives.get(directive))
    else {
        return false;
    };
    match &directive.kind {
        DirectiveKind::Method(method) => match which {
            Flag::Abstract => method.abstract_,
            Flag::Attribute => method.attribute,
            Flag::Constant => false,
            Flag::Unguarded => method.guard == GuardOption::Unguarded,
            Flag::Package => method.access == Access::Package,
            Flag::Private => method.access == Access::Private,
            Flag::Protected => method.protection == Protection::Protected,
        },
        DirectiveKind::Attribute(attribute) => match which {
            Flag::Abstract => attribute.abstract_,
            // `createAttributeGetterMethod` and the `createMethod` an
            // `::ATTRIBUTE ... GET` with a body takes both mark the
            // attribute state (`:2416`, `:2472`, `:2502`). Measured, oracle
            // rc 0: `::attribute ATT` and `::attribute ATT get` with a body
            // each answer `isAttribute` `1`.
            Flag::Attribute => true,
            Flag::Constant => false,
            Flag::Unguarded => attribute.guard == GuardOption::Unguarded,
            Flag::Package => attribute.access == Access::Package,
            Flag::Private => attribute.access == Access::Private,
            Flag::Protected => attribute.protection == Protection::Protected,
        },
        // `createConstantGetterMethod` sets `CONSTANT_METHOD` and calls
        // `setUnguarded()` outright (`parser/DirectiveParser.cpp:2523`,
        // `:2525`), taking no access or protection option. Measured, oracle
        // rc 0: `::constant CON 5` answers `0 0 1 0 0 0 0`, and it is the one
        // directive whose `isGuarded` is `0` without an `UNGUARDED` keyword.
        DirectiveKind::Constant(_) => matches!(which, Flag::Constant | Flag::Unguarded),
        // A `::ROUTINE` is not a `Method` and answers none of these; the rows
        // are not bound at `Routine` (`memory/Setup.cpp:1134`-`:1143`).
        _ => false,
    }
}

/// `MethodClass::isAbstractRexx`.
fn is_abstract(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    answer_flag(interp, receiver, Flag::Abstract, false)
}

/// `MethodClass::isAttributeRexx`.
fn is_attribute(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    answer_flag(interp, receiver, Flag::Attribute, false)
}

/// `MethodClass::isConstantRexx`.
fn is_constant(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    answer_flag(interp, receiver, Flag::Constant, false)
}

/// `MethodClass::isGuardedRexx`, which is `!methodFlags[UNGUARDED_FLAG]`.
fn is_guarded(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    answer_flag(interp, receiver, Flag::Unguarded, true)
}

/// `MethodClass::isPackageRexx`.
fn is_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    answer_flag(interp, receiver, Flag::Package, false)
}

/// `MethodClass::isPrivateRexx`.
fn is_private(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    answer_flag(interp, receiver, Flag::Private, false)
}

/// `MethodClass::isProtectedRexx`.
fn is_protected(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    answer_flag(interp, receiver, Flag::Protected, false)
}

/// One flag reader's answer. `negated` is `isGuarded`'s, the one reader whose
/// question is the stored bit's complement.
fn answer_flag(
    interp: &mut Interp,
    receiver: ObjRef,
    which: Flag,
    negated: bool,
) -> Result<Option<ObjRef>, Failure> {
    let set = flag(interp, receiver, which)?;
    Ok(Some(interp.counted(usize::from(set != negated))))
}

/// `MethodClass::setUnguardedRexx`, `setGuardedRexx`, `setPrivateRexx` and
/// `setProtectedRexx`: each writes one bit and returns `OREF_NULL`.
///
/// **They answer no result at all**, which the C++'s `return OREF_NULL` makes
/// them and which is measured: oracle rc 165, `say m~setGuarded` is
/// `91.999 Message "SETGUARDED" did not return a result.` So a witness sends
/// them as statements, and each is read back through its own `is*`.
fn set_guarded(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    write_flag(interp, receiver, |writes| writes.unguarded = Some(false))
}

/// See [`set_guarded`].
fn set_unguarded(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    write_flag(interp, receiver, |writes| writes.unguarded = Some(true))
}

/// See [`set_guarded`]. **This one changes how a send resolves**, because the
/// object `Class~method` answers is the dictionary's own method: measured,
/// oracle, `o~mm` answers `1`, then `.K~method('MM')~setPrivate`, and the
/// same `o~mm` is 97 at rc 159. [`Interp::make_method_private`] is the half
/// that does it.
fn set_private(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    write_flag(interp, receiver, |writes| writes.private = true)?;
    interp.make_method_private(receiver);
    Ok(None)
}

/// See [`set_guarded`]. `PROTECTED` is read only by the security-manager
/// seam, which nothing in this phase installs, so the readback is the whole
/// of what changes here -- D12 owns the rest.
fn set_protected(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    write_flag(interp, receiver, |writes| writes.protected = true)
}

/// Records one setter's write, after checking the receiver is an executable
/// this crate built.
fn write_flag(
    interp: &mut Interp,
    receiver: ObjRef,
    write: impl FnOnce(&mut MethodFlagWrites),
) -> Result<Option<ObjRef>, Failure> {
    source_of(interp, receiver)?;
    write(interp.method_flag_writes.entry(receiver).or_default());
    Ok(None)
}

/// `MethodClass::setSecurityManager` and `RoutineClass::setSecurityManager`,
/// each of which is `code->setSecurityManager(manager)`
/// (`classes/MethodClass.cpp:209`, `classes/RoutineClass.cpp:275`).
///
/// **The answer is the code object's kind and not a constant.**
/// `RexxCode::setSecurityManager` puts the manager on the package and returns
/// true (`execution/RexxCode.cpp:245`-`:249`); every other code object takes
/// `BaseCode::setSecurityManager`, which returns false unconditionally
/// (`execution/BaseCode.cpp:133`). Measured, oracle rc 0, one program: a
/// written `::method`, an `::attribute ... GET` with a body and a
/// `::routine` answer `1`; a primitive, a generated accessor, a `::constant`
/// getter, an `abstract` method and an `EXTERNAL` method answer `0`.
///
/// **The argument-taking form is refused, and the refusal is the ruling.**
/// The manager it installs is consulted at the next environment-symbol
/// lookup -- measured, oracle rc 159: `m~setSecurityManager(.Object~new)`
/// then `.routines~rr~class~id` is
/// `97.1 Object "an Object" does not understand message "LOCAL".`, and the
/// same program without that line is rc 0. Storing the manager and answering
/// here would run on at rc 0 where the oracle raises, so the interception
/// points D12 owns are named instead.
///
/// **The no-argument form is provably inert**, which is what makes it
/// answerable: measured, `m~setSecurityManager` with no argument leaves
/// `.routines~rr` answering. `.nil` is an argument and not an absence --
/// measured, it installs a manager and the next lookup is `97.1` against
/// `The NIL object`.
fn set_security_manager(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let source = source_of(interp, receiver)?;
    if args.first().copied().flatten().is_some() {
        return Err(Loud::security_manager().into());
    }
    let rexx_code = is_rexx_code(interp, source);
    Ok(Some(interp.counted(usize::from(rexx_code))))
}

/// Whether this executable's body is a `RexxCode` -- the distinction
/// `setSecurityManager` answers and `~source` reads.
///
/// A `RexxCode` is what `translateBlock` produced, so it is exactly the
/// directives that own a [`rexx_parse::CodeBody`] and carry no `EXTERNAL`.
fn is_rexx_code(interp: &Interp, source: ExecutableSource) -> bool {
    let (program, directive) = match source {
        ExecutableSource::Main { .. } => return true,
        ExecutableSource::Native => return false,
        ExecutableSource::Directive { program, directive } => (program, directive),
    };
    let Some(directive) = interp
        .programs
        .get(program.0)
        .and_then(|program| program.directives.get(directive))
    else {
        return false;
    };
    match &directive.kind {
        DirectiveKind::Method(method) => method.body.is_some() && method.external.is_none(),
        DirectiveKind::Attribute(attribute) => {
            attribute.body.is_some() && attribute.external.is_none()
        }
        DirectiveKind::Routine(routine) => routine.body.is_some() && routine.external.is_none(),
        _ => false,
    }
}

/// `BaseExecutable::getPackage`: the package the declaring directive belongs
/// to, and `REXX` for an executable no directive declared.
///
/// Measured, oracle rc 0: `.Object~method('objectName')~package~name` is
/// `REXX`, a `::method` in a file answers that file's own path, and
/// `.Routine~new('NEWR', 'return 42')~package~name` is `NEWR` -- the
/// executable's own name, which is what [`Interp::package_path`] answers for a
/// program compiled from source text.
fn package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let package = match source_of(interp, receiver)? {
        ExecutableSource::Directive { program, .. } | ExecutableSource::Main { program } => {
            Package::Program(program)
        }
        ExecutableSource::Native => Package::Rexx,
    };
    Ok(Some(interp.package_object(package)))
}

/// `BaseExecutable::source`: the body's own source lines as an `Array`.
fn source(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let lines = match source_of(interp, receiver)? {
        ExecutableSource::Directive { program, directive } => {
            block_source_lines(interp, program, Some(directive))
        }
        ExecutableSource::Main { program } => block_source_lines(interp, program, None),
        ExecutableSource::Native => Vec::new(),
    };
    array_of_texts(interp, lines).map(Some)
}

/// The physical lines `RexxCode::getSource` answers for one code block:
/// `Some(i)` is directive `i`'s own body and `None` is the program's main
/// section.
///
/// `RexxCode::getSource` is `package->extractSource(location)` over the block
/// location `LanguageParser::translateBlock` recorded: it **starts where the
/// parser stood when the block began and ends just before the clause that
/// terminated it** (`parser/LanguageParser.cpp:1190`-`:1193` and
/// `:1626`-`:1671`), so the answer is whole physical lines from the one after
/// the directive -- line 1, for a main section -- to the one before the next
/// directive, blank lines and whole-line comments included. Measured, oracle
/// rc 0: a `::method` followed by a blank line, a comment line,
/// `  return 1  /* trailing */`, a blank line and a second comment line
/// answers all five, in order.
///
/// **A directive with no body of its own answers nothing**, which is
/// `BaseCode::getSource`'s empty array and not an empty range: an `ABSTRACT`
/// method, a generated accessor, a `::CONSTANT` getter and an `EXTERNAL`
/// binding each carry a `BaseCode` that never saw the source.
fn block_source_lines(
    interp: &Interp,
    program: ProgramId,
    directive: Option<usize>,
) -> Vec<Vec<u8>> {
    let Some(source) = interp.programs.get(program.0) else {
        return Vec::new();
    };
    let text = &source.source;
    let (first, from) = match directive {
        None => (1, 0),
        Some(directive) => match block_start(source, directive) {
            Some(start) => block_first_line(text, start),
            None => return Vec::new(),
        },
    };
    let next = directive.map_or(0, |directive| directive + 1);
    let (last, upto) = match source.directives.get(next) {
        // A synthetic directive covers no source: it is the one
        // `Interp::record_compiled_body` files over a text that had none, and
        // a written directive's clause always spans at least `::method x`.
        Some(next) if !next.clause_span.is_empty() => block_last_line(text, next.clause_span.start),
        _ => {
            let last = text.line_count();
            (last, text.line_span(last).map_or(0, |span| span.end))
        }
    };
    // `extractSourceLines`' own step back over an end that sits at the start
    // of a line (`parser/ProgramSource.cpp:224`-`:233`): the block's end
    // offset is that line's length, so an **empty** last line drops out.
    // Measured, oracle rc 0, counting the items of a body followed by `n`
    // blank lines: one blank answers the body alone and two answer the body
    // and one blank, at the end of the file and before a following directive
    // alike; and a `::method` with nothing after it but one blank line
    // answers `Array(0)`.
    let (last, upto) = match text.line_span(last) {
        Some(span) if span.is_empty() => {
            let stepped = last.saturating_sub(1);
            (stepped, text.line_span(stepped).map_or(0, |span| span.end))
        }
        _ => (last, upto),
    };
    (first..=last)
        .map_while(|line| {
            let span = text.line_span(line)?;
            let cut = span.start.max(from)..span.end.min(upto);
            text.span_bytes(cut.clone())
                .map(<[u8]>::to_vec)
                .or_else(|| Some(Vec::new()))
        })
        .collect()
}

/// The line a block begins on, and the byte within the text it begins at.
///
/// **A clause ended by `;` leaves the block starting on its own line**, where
/// one ended by the line itself leaves it starting on the next -- the two
/// positions `nextClause` can leave the scanner in, and
/// `blockLocation.setStart(lineNumber, lineOffset)`
/// (`parser/LanguageParser.cpp:1193`) records whichever it is. Measured,
/// oracle rc 0: `::method MSEMI; return 7` answers `Array(1)` holding
/// `< return 7>`, an `::attribute ... get` whose first body line is
/// `  a = 1;   b = 2` answers `<   b = 2>` and the body's remaining lines,
/// and one whose first body line ends `a = 1;` answers an **empty** first
/// line rather than dropping it.
fn block_first_line(text: &rexx_parse::ProgramSource, start: usize) -> (usize, usize) {
    // The `;` may be inside the clause's own span or just past it, which is a
    // difference between a directive clause and a body clause and not one the
    // answer turns on.
    let after = match text.span_bytes(start..start + 1) {
        Some(b";") => start + 1,
        _ => start,
    };
    match text.span_bytes(after.saturating_sub(1)..after) {
        Some(b";") => (text.line_of(after.saturating_sub(1)), after),
        _ => (text.line_of(start) + 1, 0),
    }
}

/// The line a block ends on, and the byte it ends at, for a block terminated
/// by the directive clause beginning at `next`.
///
/// **A directive that does not begin its line cuts that line short**, which is
/// `translateBlock`'s `else` arm (`parser/LanguageParser.cpp:1657`) against
/// the `getOffset() == 0` arm above it. Measured, oracle rc 0:
/// `  return 1; ::method B` answers `Array(1)` holding `<  return 1; >`, the
/// bytes up to the `::` and no further.
fn block_last_line(text: &rexx_parse::ProgramSource, next: usize) -> (usize, usize) {
    let line = text.line_of(next);
    match text.line_span(line) {
        Some(span) if span.start < next => (line, next),
        _ => {
            let previous = line.saturating_sub(1);
            (
                previous,
                text.line_span(previous).map_or(0, |span| span.end),
            )
        }
    }
}

/// The byte the block whose source is being extracted begins after, or `None`
/// for a directive that owns no `RexxCode` at all.
///
/// **An `::ATTRIBUTE` with a body begins one clause late, and that is the
/// oracle's own answer rather than an approximation of it.**
/// `LanguageParser::attributeDirective` decides between a generated accessor
/// and a real method by calling `hasBody()`
/// (`parser/DirectiveParser.cpp:1771`), which consumes the next clause and
/// `reclaimClause()`s it -- reclaiming the clause but not the scanner's line
/// position, so `translateBlock` records a start past that first clause.
/// Measured, oracle rc 0: `::attribute ATTMULTI get` over `  a = 1`,
/// `  b = 2`, `  return a + b` answers `Array(2)` beginning at `  b = 2`,
/// while the `::method` beside it answers both of its own lines.
fn block_start(program: &Program, directive: usize) -> Option<usize> {
    let clause_end = program.directives.get(directive)?.clause_span.end;
    match &program.directives[directive].kind {
        DirectiveKind::Method(method) => method.body.as_ref().map(|_| clause_end),
        DirectiveKind::Routine(routine) => routine.body.as_ref().map(|_| clause_end),
        DirectiveKind::Attribute(attribute) => attribute
            .body
            .as_ref()
            .and_then(|body| body.instructions.first())
            .map(|first| first.clause_span.end),
        _ => None,
    }
}

/// `RoutineClass::callRexx` and the `[]` bound to the same body
/// (`memory/Setup.cpp:1137`-`:1138`): run the routine over the arguments
/// given, and answer what it returned.
///
/// **A `SUBROUTINE` call, in the routine's own package.** Measured, oracle rc
/// 0: `parse source` inside a routine reached through `~call` answers
/// `LINUX SUBROUTINE <the declaring file>`, `arg()` is the count `~call`
/// passed, and a routine calling a second routine of its own package resolves
/// it.
///
/// **A routine that returns nothing is 91.999**, which is
/// [`Interp::call_over_values`]' own answer for a function call: measured,
/// oracle, `say .routines~noret~call` on a bare `return` is 91 at rc 165.
fn call(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    enter_routine(interp, receiver, args.to_vec())
}

/// `RoutineClass::callWithRexx`: the same call over an argument array.
///
/// **`arrayArgument(args, ARG_ONE)` and not the named overload**, which is
/// the whole of the difference and is measured: oracle rc 163,
/// `.routines~rr~callWith` is `93.903 Missing argument in method; argument 1
/// is required.` where the named overload would report `argument arguments`,
/// and `~callWith(.nil)` is `98.913 Unable to convert object "The NIL object"
/// to a single-dimensional array value.`
fn call_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(array)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let values = super::array_argument(interp, array, super::ArrayArgument::Positional)?;
    enter_routine(interp, receiver, values)
}

/// The half [`call`] and [`call_with`] share, once the arguments are values.
fn enter_routine(
    interp: &mut Interp,
    receiver: ObjRef,
    values: Vec<Option<ObjRef>>,
) -> Result<Option<ObjRef>, Failure> {
    let body = interp
        .executable_sources
        .get(&receiver)
        .and_then(|record| record.routine);
    let Some((program, directive)) = body else {
        return Err(
            Loud::method_from_source("a routine whose body this crate does not hold").into(),
        );
    };
    interp.enter_installed_routine(program, directive, values)
}
