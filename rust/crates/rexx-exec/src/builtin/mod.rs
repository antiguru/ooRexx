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

//! The builtin functions: which names are builtins, how many arguments each
//! takes, and the one dispatch every implementation hangs off.
//!
//! # The name set is read, never copied
//!
//! [`is_builtin`] answers from `rexx_inventory::builtins::in_scope()`, which
//! is `NAMES` (generated from `BuiltinFunctions.cpp`) less the names
//! `docs/superpowers/plans/phase-4-exclusions.txt` excludes **outright**.
//! Three of that file's rows -- `VALUE`, `ADDRESS` and `QUEUED` -- are
//! partial: excluded in one form and in scope in the other, so they are
//! builtin names here. Subtracting `EXCLUDED` instead of `wholly_excluded()`
//! gives a set three names short, and a name missing from this set is not a
//! quiet gap: [`dispatch`] answers `None` for it, which is the answer that
//! means "try a `::routine` next". Measured on the oracle, `::routine max`
//! alongside `call max 1,2` still calls the builtin, so a name that dropped
//! out of this set would silently run the wrong routine.
//!
//! # Arity and implementation are one row
//!
//! [`IMPLEMENTED`] carries the `(min, max)` pair beside the function pointer
//! rather than in a table of its own. That is what makes the count check
//! structural: [`dispatch`] runs [`check_arity`] from the same row it is
//! about to call, so a new builtin cannot be added without an arity.
//! `max` is `None` for a variadic builtin.
//!
//! **The guarantee is a count, not a shape, and the difference is real.**
//! `(min, max)` can say how many arguments are acceptable; it cannot say
//! *which positions* must be filled, because required-ness is conditional on
//! what comes after. Measured: `date()` and `date('S')` both succeed, so
//! `DATE`'s minimum is 0 -- yet `date('S',,'S')` is `40.5`, "argument 2 is
//! required", because supplying position 3 makes position 2 mandatory.
//! A model in which the required positions are a prefix of length `min`
//! cannot express that, so an implementation *can* still be reached with an
//! interior omission it would reject, and each one that cares must check its
//! own positions.
//!
//! # What the builtin path does *not* do
//!
//! `resolve_and_run_call` (`run.rs`) evaluates the arguments and then comes
//! here **without pushing an activation**, and that is measured rather than
//! assumed. Each of the three is a thing the label path does:
//!
//! * **`SIGL` is not set.** Measured: with `say sigl` before and after,
//!   `n = length('abc')` leaves `SIGL` at its uninitialised `SIGL`, where the
//!   `call sub` two lines later sets it to that clause's own line number.
//! * **`>A>` argument lines *do* fire**, exactly as they do for a label.
//!   Measured under `trace i`, `n = length('abc')` traces
//!   `>L>   "abc"` / `>A>   "abc"` / `>F>   LENGTH => "3"` / `>>>   "3"`,
//!   which is the same argument shape `n = sub('abc')` traces.
//! * **No activation level is added.** Two observables, both measured. Under
//!   `trace i` the builtin's `>F>` and `>>>` sit at the *calling* clause's own
//!   indent, where a label callee's clauses echo two columns further in. And
//!   a condition raised by a builtin echoes one clause per enclosing
//!   activation and none for the builtin: `say substr('abc')` at the top level
//!   echoes one line, the same call inside `sub:` echoes two (the failing
//!   clause, then `call sub`).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use rexx_core::ObjRef;

use crate::error::{Failure, Raised};
use crate::value::Rendered;
use crate::{Interp, Loud};

mod convert;
mod datatype;
mod datetime;
mod numeric;
mod state;
mod string;
mod word;

/// What a builtin's code looks like: the interpreter, the row's own name and
/// the already-evaluated arguments.
///
/// A named type rather than the signature spelled inline, because it is
/// spelled in three places -- [`Builtin::run`], every implementation in
/// `string.rs`, and the tests' own stand-in -- and those three cannot drift
/// while they name this.
type Run = fn(&mut Interp, &'static [u8], &[Option<ObjRef>]) -> Result<ObjRef, Failure>;

/// One builtin this crate runs: its name, its arity, and the code.
struct Builtin {
    /// The name as `BuiltinFunctions.cpp` spells it, which is upper case.
    /// Compared against the call site's own bytes without upcasing either
    /// side -- see [`dispatch`] for the measurement that makes that the rule
    /// rather than a shortcut.
    name: &'static [u8],
    /// The fewest arguments the oracle accepts, which is also the number of
    /// leading positions that may not be omitted (40.3 and 40.5 below).
    min: usize,
    /// The most the oracle accepts, or `None` for a variadic builtin.
    max: Option<usize>,
    /// The code, taking this row's own [`name`] as its second argument.
    ///
    /// **The name is passed rather than written down again inside the
    /// implementation**, and that is what lets `CENTER` and `CENTRE` be one
    /// function: the oracle's two bodies are identical except for the name
    /// they report, and measured, they really do report differently --
    /// `centre('ab',6,'--')` is `CENTRE argument 3 must be a single
    /// character` where `center('ab',6,'--')` is `CENTER argument 3`. An
    /// implementation naming itself would be a second copy of the string in
    /// this row, free to disagree with it.
    ///
    /// [`name`]: Builtin::name
    run: Run,
}

/// Every builtin this crate runs, with the arity `check_arity` enforces
/// before the code is entered.
///
/// A name that is a builtin but has no row here fails loudly rather than
/// being answered wrongly; `corpus/builtin-status.txt` is where the
/// implemented/not-implemented boundary is recorded and policed, so this
/// table does not describe it in prose.
///
/// Every `(min, max)` pair below is the oracle's own, taken from the
/// `x_Min`/`x_Max` constants each `BUILTIN(x)` body opens with
/// (`interpreter/expression/BuiltinFunctions.cpp`) and confirmed against the
/// interpreter at both ends -- one argument short is 40.3 naming that
/// minimum and one too many is 40.4 naming that maximum.
const IMPLEMENTED: &[Builtin] = &[
    Builtin {
        name: b"ABBREV",
        min: 2,
        max: Some(3),
        run: string::abbrev,
    },
    Builtin {
        name: b"ABS",
        min: 1,
        max: Some(1),
        run: numeric::abs,
    },
    Builtin {
        // Zero arguments, not one: `check_args` with a maximum of 0, so
        // measured, `say address(1)` is 40.4 naming a maximum of 0 rather
        // than ignoring the extra.
        name: b"ADDRESS",
        min: 0,
        max: Some(0),
        run: state::address,
    },
    Builtin {
        name: b"ARG",
        min: 0,
        max: Some(2),
        run: state::arg,
    },
    Builtin {
        name: b"B2X",
        min: 1,
        max: Some(1),
        run: convert::b2x,
    },
    Builtin {
        // A minimum of 1, not 2: the second string is optional and the pad
        // does the work on its own, so measured, `c2x(bitand('ffff'x))` is
        // `FFFF`.
        name: b"BITAND",
        min: 1,
        max: Some(3),
        run: convert::bitand,
    },
    Builtin {
        name: b"BITOR",
        min: 1,
        max: Some(3),
        run: convert::bitor,
    },
    Builtin {
        name: b"BITXOR",
        min: 1,
        max: Some(3),
        run: convert::bitxor,
    },
    Builtin {
        name: b"C2D",
        min: 1,
        max: Some(2),
        run: convert::c2d,
    },
    Builtin {
        name: b"C2X",
        min: 1,
        max: Some(1),
        run: convert::c2x,
    },
    Builtin {
        // Two rows, one implementation: see `Builtin::run` for why the name
        // travels as an argument and what a program can see of the
        // difference.
        name: b"CENTER",
        min: 2,
        max: Some(3),
        run: string::center,
    },
    Builtin {
        name: b"CENTRE",
        min: 2,
        max: Some(3),
        run: string::center,
    },
    Builtin {
        name: b"CHANGESTR",
        min: 3,
        max: Some(4),
        run: string::changestr,
    },
    Builtin {
        name: b"COMPARE",
        min: 2,
        max: Some(3),
        run: string::compare,
    },
    Builtin {
        // A minimum of 0 and a maximum of 1: the bare form is
        // `CONDITION('I')`, measured -- `BUILTIN(CONDITION)` opens
        // `int style = 'I'`.
        name: b"CONDITION",
        min: 0,
        max: Some(1),
        run: state::condition,
    },
    Builtin {
        name: b"COPIES",
        min: 2,
        max: Some(2),
        run: string::copies,
    },
    Builtin {
        name: b"COUNTSTR",
        min: 2,
        max: Some(2),
        run: string::countstr,
    },
    Builtin {
        // A minimum of 1, not 2: the second argument is the type option,
        // and it is optional -- measured, `datatype('12.5')` succeeds with
        // no second argument at all, answering `NUM`/`CHAR` rather than
        // 40.3. `DATATYPE_Min`/`DATATYPE_Max`, `BuiltinFunctions.cpp:780`.
        name: b"DATATYPE",
        min: 1,
        max: Some(2),
        run: datatype::datatype,
    },
    Builtin {
        // A minimum of 0 and a maximum of 5: `date()` and `date('S')` both
        // succeed with no other arguments at all, and `DATE_Min`/
        // `DATE_Max` (`BuiltinFunctions.cpp:1003`-`1004`) name 5 as the top
        // -- `osep`/`isep`, positions 4 and 5, following the input/output
        // style pair. This module's own doc comment on the count check
        // names the conditional-required quirk `(min, max)` cannot express
        // for this exact builtin: supplying `option2` (position 3) without
        // `indate` (position 2) is 40.5, not legal, and `date::date` checks
        // that itself.
        name: b"DATE",
        min: 0,
        max: Some(5),
        run: datetime::date,
    },
    Builtin {
        // A minimum of 1, not 2: `DELSTR`'s start position is optional and
        // defaults to 1, so measured, `say delstr('abcdef')` deletes the
        // whole string rather than raising.
        name: b"DELSTR",
        min: 1,
        max: Some(3),
        run: string::delstr,
    },
    Builtin {
        name: b"D2C",
        min: 1,
        max: Some(2),
        run: convert::d2c,
    },
    Builtin {
        name: b"D2X",
        min: 1,
        max: Some(2),
        run: convert::d2x,
    },
    Builtin {
        name: b"DIGITS",
        min: 0,
        max: Some(0),
        run: state::digits,
    },
    Builtin {
        // A minimum of 2 where `DELSTR`'s is 1: `DELWORD`'s start word is
        // required, so measured, `say delword('a b')` is 40.3 naming a
        // minimum of 2 where `say delstr('abcdef')` succeeds.
        name: b"DELWORD",
        min: 2,
        max: Some(3),
        run: word::delword,
    },
    Builtin {
        name: b"ERRORTEXT",
        min: 1,
        max: Some(1),
        run: state::errortext,
    },
    Builtin {
        name: b"FORM",
        min: 0,
        max: Some(0),
        run: state::form,
    },
    Builtin {
        name: b"FORMAT",
        min: 1,
        max: Some(5),
        run: numeric::format,
    },
    Builtin {
        name: b"FUZZ",
        min: 0,
        max: Some(0),
        run: state::fuzz,
    },
    Builtin {
        // A maximum of 1 where its four neighbours here take none: the
        // argument is the `"force"` spelling, and `BUILTIN(GC)` rejects
        // anything whose first byte is not `f` or `F`.
        name: b"GC",
        min: 0,
        max: Some(1),
        run: state::gc,
    },
    Builtin {
        name: b"INSERT",
        min: 2,
        max: Some(5),
        run: string::insert,
    },
    Builtin {
        name: b"LASTPOS",
        min: 2,
        max: Some(4),
        run: string::lastpos,
    },
    Builtin {
        name: b"LEFT",
        min: 2,
        max: Some(3),
        run: string::left,
    },
    Builtin {
        // `say length()` is 40.3 with a minimum of 1 and `say
        // length('abc','x')` is 40.4 with a maximum of 1, both measured, rc
        // 216.
        name: b"LENGTH",
        min: 1,
        max: Some(1),
        run: string::length,
    },
    Builtin {
        name: b"LOWER",
        min: 1,
        max: Some(3),
        run: string::lower,
    },
    Builtin {
        // `max: None` for the same reason `XRANGE`'s row gives -- `MAX_Max`
        // is `argcount` (`BUILTIN(MAX)`) -- and measured,
        // `max(1,2,3,4,5,6,7,8)` is 8 rather than 40.4.
        name: b"MAX",
        min: 1,
        max: None,
        run: numeric::max,
    },
    Builtin {
        name: b"MIN",
        min: 1,
        max: None,
        run: numeric::min,
    },
    Builtin {
        name: b"OVERLAY",
        min: 2,
        max: Some(5),
        run: string::overlay,
    },
    Builtin {
        name: b"POS",
        min: 2,
        max: Some(4),
        run: string::pos,
    },
    Builtin {
        name: b"QUEUED",
        min: 0,
        max: Some(0),
        run: state::queued,
    },
    Builtin {
        // A minimum of 0: `random()` is a call with no arguments at all and
        // answers a number in 0..999.
        name: b"RANDOM",
        min: 0,
        max: Some(3),
        run: numeric::random,
    },
    Builtin {
        name: b"REVERSE",
        min: 1,
        max: Some(1),
        run: string::reverse,
    },
    Builtin {
        name: b"RIGHT",
        min: 2,
        max: Some(3),
        run: string::right,
    },
    Builtin {
        name: b"SIGN",
        min: 1,
        max: Some(1),
        run: numeric::sign,
    },
    Builtin {
        name: b"SOURCELINE",
        min: 0,
        max: Some(1),
        run: state::sourceline,
    },
    Builtin {
        name: b"SPACE",
        min: 1,
        max: Some(3),
        run: string::space,
    },
    Builtin {
        name: b"STRIP",
        min: 1,
        max: Some(3),
        run: string::strip,
    },
    Builtin {
        name: b"SUBSTR",
        min: 2,
        max: Some(4),
        run: string::substr,
    },
    Builtin {
        name: b"SUBWORD",
        min: 2,
        max: Some(3),
        run: word::subword,
    },
    Builtin {
        name: b"SYMBOL",
        min: 1,
        max: Some(1),
        run: datatype::symbol,
    },
    Builtin {
        // A minimum of 0 and a maximum of 3: `TIME_Min`/`TIME_Max`
        // (`BuiltinFunctions.cpp:1315`-`1316`). `option2` (position 3)
        // requires `intime` (position 2) the same way `DATE`'s own
        // conditional pair does, measured, `time(,,'n')` is 40.5 naming
        // argument 2.
        name: b"TIME",
        min: 0,
        max: Some(3),
        run: datetime::time,
    },
    Builtin {
        // A maximum of 1, and the argument *sets* the mode while the answer
        // is the mode that was in force -- measured, `trace l` then `say
        // trace('O')` prints `L`.
        name: b"TRACE",
        min: 0,
        max: Some(1),
        run: state::trace,
    },
    Builtin {
        // Six, not four: `start` and `range` are ooRexx's own extension to
        // the classic four-argument `TRANSLATE`, and measured, a seventh
        // argument is 40.4 naming a maximum of 6.
        name: b"TRANSLATE",
        min: 1,
        max: Some(6),
        run: string::translate,
    },
    Builtin {
        name: b"TRUNC",
        min: 1,
        max: Some(2),
        run: numeric::trunc,
    },
    Builtin {
        name: b"UPPER",
        min: 1,
        max: Some(3),
        run: string::upper,
    },
    Builtin {
        // `VALUE_Min`/`VALUE_Max`, `BuiltinFunctions.cpp:1812`-`1813`: the
        // new-value and selector positions are both optional, so
        // `value('name')` alone is legal.
        name: b"VALUE",
        min: 1,
        max: Some(3),
        run: datatype::value,
    },
    Builtin {
        name: b"VAR",
        min: 1,
        max: Some(1),
        run: datatype::var,
    },
    Builtin {
        name: b"VERIFY",
        min: 2,
        max: Some(5),
        run: string::verify,
    },
    Builtin {
        name: b"WORD",
        min: 2,
        max: Some(2),
        run: word::word,
    },
    Builtin {
        name: b"WORDINDEX",
        min: 2,
        max: Some(2),
        run: word::word_index,
    },
    Builtin {
        name: b"WORDLENGTH",
        min: 2,
        max: Some(2),
        run: word::word_length,
    },
    Builtin {
        // The search phrase is argument 1 and the string searched is argument
        // 2, which is the reverse of every other builtin here that takes both
        // (`BUILTIN(WORDPOS)`, `expression/BuiltinFunctions.cpp`). Measured,
        // the message numbers them the same way round: `wordpos('a','a b','q')`
        // is `WORDPOS argument 3 must be a whole number`.
        name: b"WORDPOS",
        min: 2,
        max: Some(3),
        run: word::word_pos,
    },
    Builtin {
        name: b"WORDS",
        min: 1,
        max: Some(1),
        run: word::words,
    },
    Builtin {
        name: b"X2B",
        min: 1,
        max: Some(1),
        run: convert::x2b,
    },
    Builtin {
        name: b"X2C",
        min: 1,
        max: Some(1),
        run: convert::x2c,
    },
    Builtin {
        name: b"X2D",
        min: 1,
        max: Some(2),
        run: convert::x2d,
    },
    Builtin {
        // `max: None` because `XRANGE_Max` is `argcount` itself
        // (`BUILTIN(XRANGE)`), so no call is ever too long -- measured,
        // `xrange('a','b','c','d','e','f','g','h')` is eight bytes rather
        // than 40.4. Its minimum is 0, so `xrange()` is a call with no
        // arguments at all and answers all 256 bytes.
        name: b"XRANGE",
        min: 0,
        max: None,
        run: convert::xrange,
    },
];

/// The builtin names Phase 4 dispatches, as a set built once.
///
/// `in_scope()` allocates a fresh `Vec` per call and a call reaches it once
/// per named call clause, so the set is built on first use and kept. It is
/// still *derived*: the rows come from `rexx_inventory` on every process,
/// never from a list written down here.
fn in_scope() -> &'static HashSet<&'static str> {
    static NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| rexx_inventory::builtins::in_scope().into_iter().collect())
}

/// Whether `name` is a builtin function name.
///
/// `name` arrives as the call site spells it: already upcased for a symbol
/// target, verbatim for a quoted literal. Both are compared against the
/// table's own upper-case spelling with no further folding, which is the
/// oracle's own rule and is measured in both directions -- `say
/// "LENGTH"('abc')` prints 3, and `say "length"('abc')` is Error 43.1
/// ("Could not find routine") at rc 213.
///
/// A name that is not UTF-8 is not a builtin. `NAMES` is generated from C++
/// identifiers, so every entry is ASCII; a `CallTarget::Literal` carrying
/// arbitrary bytes simply matches none of them.
pub(crate) fn is_builtin(name: &[u8]) -> bool {
    std::str::from_utf8(name).is_ok_and(|name| in_scope().contains(name))
}

/// The builtin names Phase 4 excludes **outright**, as a set built once --
/// the same derivation [`in_scope`] uses, from the complement
/// `rexx_inventory::builtins::wholly_excluded()`. A *partially* excluded name
/// is not here: its in-scope form still dispatches, and its excluded form is
/// loud from inside the builtin's own code.
fn wholly_excluded() -> &'static HashSet<&'static str> {
    static NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        rexx_inventory::builtins::wholly_excluded()
            .into_iter()
            .collect()
    })
}

/// Whether `name` is a builtin function name Phase 4 runs nothing for.
///
/// **Its own resolution step, in front of the `::ROUTINE` lookup and behind
/// [`is_builtin`].** The oracle's builtin table is one table and a builtin
/// always beats a `::ROUTINE` of the same name, so a name in it must never
/// reach the routine step -- a `::routine charin` would otherwise run here
/// where the oracle runs `CHARIN`. And it must never reach 43.1 either: the
/// oracle answers this name, so a condition here would let a program
/// *expecting* that condition pass against a gap, which is the whole reason
/// an excluded construct fails loudly instead.
///
/// Case-sensitive on the same argument [`is_builtin`]'s doc makes: a quoted
/// lower-case target reaches no builtin on the oracle either.
pub(crate) fn is_excluded_builtin(name: &[u8]) -> bool {
    std::str::from_utf8(name).is_ok_and(|name| wholly_excluded().contains(name))
}

/// Runs the builtin `name` over already-evaluated arguments.
///
/// `None` means **`name` is not a builtin**, which is the answer that lets
/// resolution carry on to a `::routine` and then to external resolution.
/// `Some(Err(..))` is a raised condition -- the 40.x incorrect-call family
/// among them -- or this crate's own declared gap for a builtin it does not
/// run yet.
///
/// **Arguments arrive evaluated, and an omitted interior position arrives as
/// `None`.** A trailing omission is not a position at all by the time it
/// gets here: `rexx-parse` drops those (`ExprKind::List`'s own doc comment),
/// matching the oracle, where `q(1,,2,,)` reports `arg()` as 3.
///
/// See the module doc for what this path deliberately does *not* do that the
/// label path does -- `SIGL`, and the activation level.
pub(crate) fn dispatch(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Option<Result<ObjRef, Failure>> {
    let target = resolve(name)?;
    Some(run(interp, name, target, args))
}

/// Which builtin a resolved name runs, decided once where the name is
/// resolved rather than found again on every call.
///
/// **A row index, and not a copy of the row.** [`Resolved::Builtin`] used to
/// carry nothing, and its doc gave the reason: the arity check and the code
/// live on one row, so carrying "which builtin" beside the resolution would be
/// a second copy free to drift from it. That argument is about the row's
/// *contents*. An index names the one row and cannot disagree with it, and it
/// is what lets a call site keep its answer.
///
/// [`Resolved::Builtin`]: crate::Resolved::Builtin
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum BuiltinTarget {
    /// The row of [`IMPLEMENTED`] that runs this name.
    Row(u16),
    /// A name this crate declares in scope and runs nothing for.
    ///
    /// **A resolution rather than a miss**, for exactly
    /// [`is_excluded_builtin`]'s reason: the oracle answers this name, so it
    /// must reach neither a `::ROUTINE` nor 43.1. It reaches the loud declared
    /// gap instead, and [`run`] is where that happens.
    Gap,
}

/// Every implemented builtin's row, keyed by the bytes a call site spells.
///
/// Built once, from [`IMPLEMENTED`] itself, so it cannot name a row the table
/// does not have or miss one it does.
fn rows() -> &'static HashMap<&'static [u8], u16> {
    static ROWS: OnceLock<HashMap<&'static [u8], u16>> = OnceLock::new();
    ROWS.get_or_init(|| {
        IMPLEMENTED
            .iter()
            .enumerate()
            .map(|(at, builtin)| {
                let at = u16::try_from(at).expect("the builtin table is far under u16");
                (builtin.name, at)
            })
            .collect()
    })
}

/// Resolves `name` to what will run it, or `None` when it is not a builtin at
/// all and resolution should carry on to a `::ROUTINE`.
///
/// **This is the whole of the lookup, and it used to be two.** Resolution
/// asked [`is_builtin`] -- one hash of the name -- and then [`dispatch`]
/// hashed it a second time and walked [`IMPLEMENTED`] comparing byte slices
/// until it matched. Entry 11 of the phase 4f record measured the pair at
/// **17.4% of the `strings` axis**, 9.0% in the set lookup and 8.4% in the
/// scan, and the scan is the half that grows with the table.
pub(crate) fn resolve(name: &[u8]) -> Option<BuiltinTarget> {
    // **The row map is asked first, and that ordering is the second half of
    // the saving.** Every row's name is in scope -- asserted by
    // `every_implemented_row_names_an_in_scope_builtin`, which is what makes a hit
    // here conclusive without asking the set as well. So a call to a builtin this
    // crate runs, which is every call any benchmark here makes, hashes the
    // name once rather than twice.
    if let Some(row) = rows().get(name) {
        return Some(BuiltinTarget::Row(*row));
    }
    // A miss is not yet an answer: the name may still be in scope with no row
    // here, and that difference is `Gap`. This is the cold half.
    is_builtin(name).then_some(BuiltinTarget::Gap)
}

/// Runs the builtin `target` names over already-evaluated arguments.
///
/// `name` is the call site's own spelling and is used only to report the
/// declared gap; the row's own [`Builtin::name`] is what a running builtin is
/// handed, which is what keeps `CENTER` and `CENTRE` one function.
pub(crate) fn run(
    interp: &mut Interp,
    name: &[u8],
    target: BuiltinTarget,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let BuiltinTarget::Row(row) = target else {
        // A builtin name this crate has no code for. It reads as the same
        // declared gap an unresolved name gets, and deliberately so: a
        // builtin *is* a routine as far as the message is concerned, the
        // owning phase is the same one, and `tests/keyword_assertions.rs`
        // reads the owner out of exactly this shape.
        return Err(Loud::unresolved_call(name).into());
    };
    let builtin = &IMPLEMENTED[row as usize];
    check_arity(builtin, args)?;
    (builtin.run)(interp, builtin.name, args)
}

/// The 40.x incorrect-call checks every builtin shares, in the order the
/// oracle applies them.
///
/// **The order is measured, not chosen.** `say substr(,2,3,'p','q')` has both
/// too many arguments and a missing required first one, and the oracle
/// answers 40.4 -- so the maximum is checked before anything about which
/// positions were supplied. All three, rc 216 in every case:
///
/// ```text
/// say substr('abc')             40.3  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
/// say length('abc','x')         40.4  Too many arguments in invocation of LENGTH; maximum expected is 1.
/// say substr('abc',,2)          40.5  Missing argument in invocation of SUBSTR; argument 2 is required.
/// ```
///
/// The routine name is **upcased in the message while the clause echo keeps
/// the source spelling**, measured with a mixed-case call: `say
/// SuBsTr('abc','x')` echoes `*-* say SuBsTr('abc','x')` above a secondary
/// line naming `SUBSTR`. That falls out of `name` already being the table's
/// own spelling and the echo being the clause's own bytes; nothing here
/// upcases anything.
///
/// The name is interpolated rather than fixed: the same bad argument gives
/// `COPIES argument 2 must be a whole number` for `say copies('abc','x')`.
fn check_arity(builtin: &Builtin, args: &[Option<ObjRef>]) -> Result<(), Failure> {
    if let Some(max) = builtin.max
        && args.len() > max
    {
        return Err(Raised::too_many_arguments(builtin.name, max).into());
    }
    if args.len() < builtin.min {
        return Err(Raised::not_enough_arguments(builtin.name, builtin.min).into());
    }
    match args[..builtin.min].iter().position(Option::is_none) {
        Some(index) => Err(Raised::missing_argument(builtin.name, index + 1).into()),
        None => Ok(()),
    }
}

// ---- reading arguments ----
//
// The conversions every family of builtins shares, in the shape the oracle's
// `required_*`/`optional_*` macros have: one per argument *kind* a builtin can
// declare, each naming the routine and the call's own argument position in the
// 40.x message it raises.

/// The precision the oracle converts a builtin's numeric arguments under.
///
/// `Numerics::ARGUMENT_DIGITS` (`runtime/Numerics.hpp`), 18 on a 64-bit
/// build and deliberately not the current `NUMERIC DIGITS` --
/// `Raised::argument_not_whole` carries the pair of measurements that
/// separates the two.
const ARGUMENT_DIGITS: usize = 18;

/// The argument at 1-based `position`, or `None` if the call did not supply
/// one there.
///
/// The two ways a position can be absent are one answer here on purpose: a
/// list shorter than `position` and an interior `None` mean the same thing to
/// every builtin, since the oracle's `optional_*` macros test
/// `argcount >= position` and then read a slot that may itself be null.
/// [`check_arity`] guarantees positions `1..=min` are all `Some`, having
/// turned any omission there into 40.5, so those are the positions the
/// `expect`ing helpers below may be asked about -- and only those.
fn arg(args: &[Option<ObjRef>], position: usize) -> Option<ObjRef> {
    args.get(position - 1).copied().flatten()
}

/// The rendered bytes of the argument at 1-based `position`, which the
/// caller knows is present.
fn required_string(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Vec<u8> {
    let value = arg(args, position).expect("check_arity admitted this required argument");
    interp.to_text(value).into_owned()
}

/// The argument at 1-based `position` prepared for a shared borrow, for a
/// builtin that reads more than one string or reads one across a further call
/// on the interpreter.
///
/// The counterpart to [`required_string`], and the reason to prefer it: that
/// one copies the bytes out, and the copy is not what the caller wanted -- it
/// is what the caller had to buy to release `to_text`'s `&mut`. Reading
/// through [`Rendered::text`] instead costs nothing for a string argument,
/// which is nearly all of them.
///
/// **Every `&mut` call the builtin makes has to happen before this one.**
/// That is a real constraint on the call sites and it reorders them: the
/// numeric and pad arguments are converted first, then the strings are read.
/// The reordering is not observable, because reading a string cannot fail --
/// [`Interp::to_text`] is total -- so no error can change place, and the two
/// lazy caches it fills are pure.
fn required_render(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Rendered {
    let value = arg(args, position).expect("check_arity admitted this required argument");
    interp.render(value)
}

/// The rendered bytes of an optional argument.
fn optional_string(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Option<Vec<u8>> {
    let value = arg(args, position)?;
    Some(interp.to_text(value).into_owned())
}

/// An argument the builtin declared as an integer, converted the way the
/// oracle's `optional_integer`/`required_integer` macros do.
///
/// `Ok(None)` is "the call supplied nothing here"; the caller then applies
/// that argument's own default, which differs per builtin and is never a
/// single shared value.
fn whole_number(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<i64>, Failure> {
    let Some(value) = arg(args, position) else {
        return Ok(None);
    };
    // `to_number` hands back an owned `Number`, so the borrow of `interp` is
    // over before `to_text` below needs its own.
    if let Ok(number) = interp.to_number(value)
        && let Some(whole) = number.whole_value(ARGUMENT_DIGITS)
    {
        return Ok(Some(whole));
    }
    let found = interp.to_text(value).into_owned();
    Err(Raised::argument_not_whole(name, position, &found).into())
}

/// An argument the builtin declared as a pad, which must be exactly one
/// byte.
fn pad_byte(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<u8>, Failure> {
    let Some(value) = arg(args, position) else {
        return Ok(None);
    };
    let found = interp.to_text(value).into_owned();
    match found.as_slice() {
        [byte] => Ok(Some(*byte)),
        _ => Err(Raised::argument_not_a_pad(name, position, &found).into()),
    }
}

// ---- range-checking converted arguments ----
//
// The operation layer's 93.9xx checks, which run after every conversion above
// and name neither the routine nor the call position. `string.rs`'s own module
// doc carries the three oracle transcripts that fix the order between the two
// layers.

/// A converted argument used as a length: zero or positive.
fn length_of(value: i64) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| Raised::invalid_length(value.to_string().as_bytes()).into())
}

/// A converted argument used as a position: strictly positive.
fn position_of(value: i64) -> Result<usize, Failure> {
    match usize::try_from(value) {
        Ok(position) if position > 0 => Ok(position),
        _ => Err(Raised::invalid_position(value.to_string().as_bytes()).into()),
    }
}

/// A converted argument used as a repetition or replacement count: zero or
/// positive. `method_position` is the position the oracle's message names,
/// which is the *operation's* own numbering rather than the call's.
fn count_of(value: i64, method_position: usize) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| {
        Raised::argument_not_non_negative(method_position, value.to_string().as_bytes()).into()
    })
}

// ---- building results ----

/// A result buffer of exactly `len` bytes' capacity, or the condition the
/// oracle raises when the allocator refuses.
///
/// See `Raised::system_resources` for why the refusal is asked of the
/// allocator rather than of a size limit.
/// A buffer that is not the lent one.
///
/// Two callers need this. `pack_hex` has no `Interp` to lend from, and
/// `padding_width` uses the reservation *itself* as the resource check and then
/// drops it -- a lent buffer that already had the capacity would make that
/// check succeed without asking the allocator anything, which is the one place
/// where reuse would be a defect rather than a saving.
fn fresh_buffer(len: usize) -> Result<Vec<u8>, Failure> {
    let mut out = Vec::new();
    out.try_reserve_exact(len)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    Ok(out)
}

/// **`try_reserve`, not `try_reserve_exact`, and the difference is the whole
/// point of lending the buffer.** An exact reservation resizes the shared
/// buffer to precisely one call's need, so two callers wanting different
/// lengths reallocate on every alternation and the lending buys nothing:
/// measured on `bench-programs/strings.rex`, whose loop asks `changestr` for
/// 43 bytes and then joins 46, the buffer was reallocated twice per iteration
/// and that was the whole of what the program still allocated. Growing
/// amortised lets the capacity settle at the longest result a program asks
/// for and stay there.
///
/// The guarantee `fresh_buffer` beside this exists for is untouched: this
/// still reserves fallibly, so a result sized from user input raises 5.1
/// rather than aborting, and a request larger than the buffer's own capacity
/// is still a single reservation of what was asked for.
fn buffer(interp: &Interp, len: usize) -> Result<Vec<u8>, Failure> {
    let mut out = interp.take_result_buffer();
    out.try_reserve(len)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three partial rows are builtin names, and the whole exclusions are
    /// not.
    ///
    /// This is the 63-against-66 trap in both directions. `EXCLUDED` has
    /// eighteen rows and only fifteen of them are exclusions; taking the
    /// whole list out would leave `VALUE`, `ADDRESS` and `QUEUED` answering
    /// `None` from `dispatch`, which is the answer reserved for "this is not
    /// a builtin, try a `::routine`".
    #[test]
    fn the_partial_exclusions_are_builtin_names_and_the_whole_ones_are_not() {
        for name in rexx_inventory::builtins::PARTIALLY_EXCLUDED {
            assert!(
                is_builtin(name.as_bytes()),
                "{name} is excluded only in part, so its in-scope form must dispatch"
            );
        }
        for name in rexx_inventory::builtins::wholly_excluded() {
            assert!(
                !is_builtin(name.as_bytes()),
                "{name} is excluded outright, so nothing here may claim it"
            );
        }
        assert!(is_builtin(b"LENGTH"));
        assert!(!is_builtin(b"length"), "the table is not case-folded");
        assert!(!is_builtin(b"ZORKOLO"));
        assert!(!is_builtin(&[0xff, 0xfe]), "and does not need valid UTF-8");
    }

    /// [`resolve`] partitions the in-scope set exactly, and answers `None`
    /// for everything outside it.
    ///
    /// **The partition is what the reordering inside `resolve` depends on.**
    /// That function asks the row map before the in-scope set and treats a hit
    /// as conclusive, which is only sound if every row is in scope -- the test
    /// below -- and only *complete* if every in-scope name without a row still
    /// answers `Gap`, which is this one. Derived from the two collections
    /// rather than from a list written here, so a name added to either is
    /// covered without this test being edited -- which is how it came to
    /// record that the gap set is currently empty.
    #[test]
    fn resolve_partitions_the_in_scope_set_into_rows_and_gaps() {
        let mut rows_seen = 0;
        let mut gaps_seen = 0;
        for name in in_scope() {
            let bytes = name.as_bytes();
            match (rows().get(bytes), resolve(bytes)) {
                (Some(row), Some(BuiltinTarget::Row(answered))) => {
                    assert_eq!(*row, answered, "{name} resolved to the wrong row");
                    rows_seen += 1;
                }
                (None, Some(BuiltinTarget::Gap)) => gaps_seen += 1,
                (row, answered) => {
                    panic!("{name}: row map says {row:?}, resolve says {answered:?}")
                }
            }
        }
        assert_eq!(
            rows_seen,
            IMPLEMENTED.len(),
            "every row is reachable by name"
        );

        // **`BuiltinTarget::Gap` is unreachable today, and this is where that
        // is written down.** Phase 4's in-scope set and this crate's table are
        // currently the same 66 names, so no call can produce it -- found by
        // this test failing an earlier assertion that demanded a gap exist.
        //
        // The arm stays regardless, and the reason is that the two sets are
        // not tied statically: `in_scope()` is derived per process from
        // `rexx_inventory`, so widening it without adding a row makes `Gap`
        // live again. Without the arm, such a name would fall past the builtin
        // step to the `::ROUTINE` lookup and then to 43.1 -- exactly what
        // `is_excluded_builtin`'s own doc gives the measurement against. This
        // assertion is what turns that widening into a failure here rather
        // than into a wrong answer at run time.
        assert_eq!(
            gaps_seen,
            0,
            "an in-scope builtin has no row: {} in scope against {} rows",
            in_scope().len(),
            IMPLEMENTED.len()
        );

        assert_eq!(resolve(b"ZORKOLO"), None, "not a builtin at all");
        assert_eq!(resolve(b"length"), None, "the table is not case-folded");
        assert_eq!(
            resolve(&[0xff, 0xfe]),
            None,
            "and does not need valid UTF-8"
        );
    }

    /// Every implemented row names a real in-scope builtin.
    ///
    /// Without this a typo in [`IMPLEMENTED`] is invisible: `is_builtin`
    /// would answer `false`, `dispatch` would answer `None`, and the row
    /// would simply never run.
    #[test]
    fn every_implemented_row_names_an_in_scope_builtin() {
        for builtin in IMPLEMENTED {
            let name = std::str::from_utf8(builtin.name).expect("a builtin name is ASCII");
            assert!(
                is_builtin(builtin.name),
                "{name} has an implementation but is not an in-scope builtin name"
            );
            assert!(
                builtin.max.is_none_or(|max| max >= builtin.min),
                "{name} has a maximum below its minimum"
            );
        }
    }

    /// `dispatch` answers `None` for a name that is not a builtin, which is
    /// the answer resolution needs to carry on past this step.
    #[test]
    fn dispatch_declines_a_name_that_is_not_a_builtin() {
        let mut interp = Interp::new();
        assert!(dispatch(&mut interp, b"ZORKOLO", &[]).is_none());
        assert!(
            dispatch(&mut interp, b"CHARIN", &[]).is_none(),
            "a whole exclusion is not a builtin name here either"
        );
    }

    fn never_run(
        _: &mut Interp,
        _: &'static [u8],
        _: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        unreachable!("check_arity never runs the builtin")
    }

    /// A builtin with a required argument in a middle position, which is the
    /// only shape that can reach 40.5. `LENGTH` takes one argument and a lone
    /// omitted argument is a trailing omission that never arrives, so its own
    /// row cannot produce that sub-code.
    ///
    /// The arity is [`IMPLEMENTED`]'s own row rather than a copy of its
    /// numbers, so this test cannot go on asserting a `(2, 4)` the table has
    /// stopped saying; only `run` is replaced, since `check_arity` must never
    /// reach it.
    fn substr_arity() -> Builtin {
        let row = IMPLEMENTED
            .iter()
            .find(|builtin| builtin.name == b"SUBSTR")
            .expect("SUBSTR has a row");
        Builtin {
            run: never_run,
            ..*row
        }
    }

    /// The three incorrect-call sub-codes and their substitutions, against
    /// the oracle transcripts in [`check_arity`]'s own doc.
    #[test]
    fn the_arity_checks_answer_the_oracles_own_sub_codes() {
        let value = ObjRef::small_int(1).expect("1 is a small int");

        let failure =
            check_arity(&substr_arity(), &[Some(value)]).expect_err("one argument is too few");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 3));
        assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"2".to_vec()]);

        let five = [Some(value); 5];
        let failure = check_arity(&substr_arity(), &five).expect_err("five arguments are too many");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 4));
        assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"4".to_vec()]);

        let failure = check_arity(&substr_arity(), &[Some(value), None, Some(value)])
            .expect_err("argument 2 is required");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 5));
        assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"2".to_vec()]);

        // The adjacent success: an omission *past* the required positions is
        // not an error at all -- measured, `say substr('abc',2,)` prints
        // `bc`.
        check_arity(&substr_arity(), &[Some(value), Some(value), None])
            .expect("that call is legal");
    }

    /// The maximum is checked before the required positions, which is the one
    /// ordering a program can tell apart.
    ///
    /// Measured: `say substr(,2,3,'p','q')` is 40.4, not 40.5, even though
    /// argument 1 is both required and omitted.
    #[test]
    fn too_many_arguments_wins_over_a_missing_required_one() {
        let value = ObjRef::small_int(1).expect("1 is a small int");
        let failure = check_arity(
            &substr_arity(),
            &[None, Some(value), Some(value), Some(value), Some(value)],
        )
        .expect_err("five arguments are too many");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 4));
    }
}
