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

use std::collections::HashSet;
use std::sync::OnceLock;

use rexx_core::ObjRef;

use crate::error::{Failure, Raised};
use crate::{Interp, Loud};

mod string;

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
        // A minimum of 1, not 2: `DELSTR`'s start position is optional and
        // defaults to 1, so measured, `say delstr('abcdef')` deletes the
        // whole string rather than raising.
        name: b"DELSTR",
        min: 1,
        max: Some(3),
        run: string::delstr,
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
        // Six, not four: `start` and `range` are ooRexx's own extension to
        // the classic four-argument `TRANSLATE`, and measured, a seventh
        // argument is 40.4 naming a maximum of 6.
        name: b"TRANSLATE",
        min: 1,
        max: Some(6),
        run: string::translate,
    },
    Builtin {
        name: b"UPPER",
        min: 1,
        max: Some(3),
        run: string::upper,
    },
    Builtin {
        name: b"VERIFY",
        min: 2,
        max: Some(5),
        run: string::verify,
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
    if !is_builtin(name) {
        return None;
    }
    let Some(builtin) = IMPLEMENTED.iter().find(|builtin| builtin.name == name) else {
        // A builtin name this crate has no code for. It reads as the same
        // declared gap an unresolved name gets, and deliberately so: a
        // builtin *is* a routine as far as the message is concerned, the
        // owning phase is the same one, and `tests/keyword_assertions.rs`
        // reads the owner out of exactly this shape.
        return Some(Err(Loud::unresolved_call(name).into()));
    };
    if let Err(failure) = check_arity(builtin, args) {
        return Some(Err(failure));
    }
    Some((builtin.run)(interp, builtin.name, args))
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
