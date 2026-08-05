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

//! `Raised`: the payload of a real Rexx condition.
//!
//! **Only the payload exists here.** Task 12 ("Errors, the message
//! catalogue, and the exit code") owns the message catalogue, the
//! oracle's exact two-line stderr format, the clause echo, and the
//! `256 - number` exit-code mapping -- none of that is built in this
//! task. What exists is enough to assert *which* condition was raised
//! (the condition name, the number and sub-number, and the substitution
//! values), not to reproduce what the oracle prints for it. A test
//! against this type checks "did `1/0` raise 42.3", never "does stderr
//! read `Error 42.3: ...` and does the process exit 214".
//!
//! `Failure` is the other half: `step` and everything above it can fail
//! either because 4a does not implement a construct (`Loud`, `lib.rs`) or
//! because a real condition was raised (`Raised`), and a clause containing
//! an expression can do either, so the propagation type has to carry both.

use crate::Loud;
use rexx_core::ObjRef;
use rexx_num::ArithError;
use rexx_parse::ParseError;
use std::borrow::Cow;

/// Which activations may trap one raise, walking outward from the one that
/// raised it.
///
/// **Three answers, not one, and the differences are measured rather than
/// derived from the grammar.** Nothing about `RAISE`'s syntax says that its
/// tail decides who is allowed to catch it, and every one of these was found
/// by running the three-level shape that tells them apart -- a two-level
/// program gives the same bytes for all three.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Search {
    /// From the activation that raised, outward level by level. Every
    /// ordinary condition (`say 1/0`, an unset variable under `SIGNAL ON
    /// NOVALUE`), and `RAISE SYNTAX ... RETURN`.
    ///
    /// Measured for the ordinary case: `say 1/0` inside a `PROCEDURE`d
    /// routine, with `SIGNAL ON SYNTAX` enabled only in its caller, traps
    /// *in the routine* -- the handler reads the routine's own isolated
    /// pool, and `SIGL` is the routine's raising line. Turning the trap off
    /// in the routine makes the same condition trap in the caller instead,
    /// with `SIGL` set to the caller's `call` clause, which is the outward
    /// half.
    #[default]
    Here,
    /// From the **caller** of the activation that raised, outward -- the
    /// raising activation's own (inherited) trap is skipped.
    ///
    /// `RAISE <anything but SYNTAX> ... RETURN`. Measured: `raise user foo
    /// return 'RETVAL'` inside `fun`, with `signal on user foo` in the main
    /// body, traps with `SIGL` set to the main body's own clause -- not to
    /// `fun`'s `raise` line, which is what `Here` gives and which is exactly
    /// what the identical program with `raise syntax 40.4 return` does
    /// report. The two spellings differ in nothing but the condition.
    ///
    /// **"Outward" stops at the caller, and that is measured rather than a
    /// limitation.** Task 7's report listed one-level-only as a residual --
    /// "if the grandparent has a trap and the parent does not, we ignore the
    /// condition where the oracle might propagate to it". Built and run: with
    /// `signal on user foo` in the main body, `signal off user foo` in
    /// `lev1`, and `raise user foo return` in `lev2`, the oracle does **not**
    /// reach the main body's handler either -- `lev1` simply resumes. So the
    /// search really does end at the caller, and this is a behaviour rather
    /// than a residual.
    Caller,
    /// The **outermost** activation only; every level it unwinds through
    /// skips its own trap check.
    ///
    /// `RAISE SYNTAX` with no `RETURN`/`EXIT` tail, and `RAISE SYNTAX ...
    /// EXIT`. Measured at three levels: with `SIGNAL ON SYNTAX` enabled in
    /// the *middle* routine and nowhere else, a tail-less `raise syntax
    /// 40.4` in the innermost is **not** trapped -- it is the ordinary fatal
    /// report at rc 216. Enable it in the main body as well and the main
    /// body's trap is the one that fires, with `SIGL` set to the main body's
    /// own `call` clause, the middle routine's trap still untouched.
    Top,
    /// No activation at all may trap this -- it is already the condition's
    /// default action, on its way to the report.
    ///
    /// Two producers, both measured. `RAISE HALT` with no `RETURN` tail:
    /// `signal on halt` on the line immediately above it does **not** fire,
    /// and the program gets the fatal `Error 4.1` at rc 252 -- which is what
    /// separates this from [`Search::Top`], since at top level `Top` would
    /// have offered it to exactly that trap. And `RAISE PROPAGATE`, which is
    /// trapped by no enclosing handler at either of the two depths it was
    /// measured at.
    Nobody,
}

/// How one raise in flight must be delivered, beside the payload it carries.
///
/// Every field is at its default for every condition an *expression* or an
/// ordinary instruction raises, which is every raiser in the crate except
/// `RAISE` itself -- so [`Raised::syntax`] supplies the default and the
/// thirty-odd raiser functions never mention it.
///
/// **Three functions in `run.rs` write these fields, not one** (fix round 1's
/// finding 6, which corrects a line here that named only the first):
/// `exec_raise` sets `search` from the tail and the condition,
/// `exec_raise_propagate` sets both fields, and `offer_to_trap` performs the
/// load-bearing [`Search::Caller`] -> [`Search::Here`] rewrite as a raise
/// declines its first level. That last one is the writer a reader most needs
/// to know about, since it is the only one that mutates a `Delivery` already
/// in flight.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Delivery {
    pub(crate) search: Search,
    /// Render the major line as `Error 42:  ...` rather than
    /// `Error 42 running <path> line 8:  ...`.
    ///
    /// `RAISE PROPAGATE`'s own form, measured at two nesting depths: the
    /// clause echoes above it are unchanged (one per level, innermost
    /// first), and only the ` running <path> line <n>` span is dropped.
    pub(crate) positionless: bool,
}

/// One value a catalogue message interpolates: **bytes, not text**.
///
/// A Rexx string is a byte string, and a substitution is usually one --
/// `left('ab', zz)` puts `zz`'s own rendering into 40.12's `found "&3"`, and
/// nothing constrains it to UTF-8. Passing it through `String` costs the
/// bytes that are not: measured, `say copies('ab','FF'x)` reports
/// `found "\377"` from the oracle, where a lossy conversion reports
/// `found "\357\277\275"` -- U+FFFD, three bytes for one, on a channel the
/// differential harness compares byte for byte.
///
/// The sanitising the oracle *does* apply is a different thing and happens
/// later, at [`displayable`], on the whole line rather than on the value.
type Substitution = Vec<u8>;

/// The substitutions a sibling crate's error carries, as bytes.
///
/// `rexx-num` and `rexx-parse` build their own substitution lists as
/// `String`, which is right for them: every value they interpolate is a
/// number's rendering or a catalogue-supplied fragment, never arbitrary
/// program data. This is the one-way widening at the boundary, so the field
/// itself can stay [`Substitution`]s.
pub(crate) fn into_substitutions(values: Vec<String>) -> Vec<Substitution> {
    values.into_iter().map(String::into_bytes).collect()
}

/// A real Rexx condition raised during evaluation.
#[derive(Clone, Debug)]
pub(crate) struct Raised {
    /// The condition name a trapped Rexx program would see from
    /// `condition('c')`, and the exact bytes an activation's trap table is
    /// keyed by (`Activation::traps`). It is carried as a field rather than
    /// hardcoded at each call site because the spec's own shape includes it
    /// and 4b's `NOVALUE` and `RAISE` do set it to something else.
    ///
    /// **`Cow` rather than `&'static str` since 4b's Task 7**, and for
    /// exactly one condition family: `RAISE USER foo` names the condition
    /// `USER FOO`, built from the program's own text, where every other
    /// condition name in the language is one of a fixed set. The borrowed
    /// case stays allocation-free, which is every raise the crate makes on
    /// its own.
    ///
    /// The `#[expect(dead_code)]` that used to sit here is **deleted rather
    /// than moved**: `SIGNAL ON`/`CALL ON` read this field to decide whether
    /// a trap matches, which is the reader it was waiting for.
    pub(crate) condition: Cow<'static, str>,
    pub(crate) number: u16,
    pub(crate) sub: u16,
    /// What `&1`, `&2`, ... in this error's catalogue entry stand for.
    ///
    /// See [`Substitution`] for why these are bytes.
    pub(crate) additional: Vec<Substitution>,
    /// What a trapping handler reads back from `RC`, or `None` to leave `RC`
    /// alone.
    ///
    /// Measured, three rows, and the third is what makes this a field rather
    /// than "always the major":
    ///
    /// ```text
    /// signal on syntax  ; say 1/0            -> handler sees RC = 42
    /// signal on syntax  ; raise syntax 40.4  -> handler sees RC = 40  (not 40.4)
    /// signal on novalue ; say zunset         -> handler sees RC = RC  (untouched)
    /// ```
    ///
    /// So it is the *major* for every `SYNTAX` condition however it arose --
    /// which is why [`Raised::syntax`] fills this in from `number` and no
    /// raiser has to think about it -- and it is untouched for a condition
    /// with no catalogue number. `RAISE ERROR n`/`RAISE FAILURE n` is the one
    /// case that is neither: `n` is the value, measured at `rc= 5` for `raise
    /// error 5`, and `exec_raise` sets it there.
    pub(crate) rc: Option<Vec<u8>>,
    pub(crate) delivery: Delivery,
}

/// Which of the two grouped digit notations a validation error is about.
///
/// The oracle carries the same distinction as a `bool hex` parameter threaded
/// through `StringUtil::validateGroupedSet` (`classes/support/StringUtil.cpp`),
/// which picks between paired catalogue entries at each of its three
/// failures. It is a type here so that a caller validating a binary string
/// cannot name the hexadecimal message: the three raisers below take this and
/// choose the sub-code themselves, rather than each call site writing a
/// number down.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Notation {
    Hex,
    Binary,
}

impl Raised {
    /// A `SYNTAX` condition with an ordinary delivery -- the shape every
    /// raiser outside `RAISE` itself has.
    ///
    /// `pub(crate)` since 4b's Task 7, which is when `Raised` gained a field
    /// no raiser cares about: `run.rs` and `trace.rs` each built their own
    /// `Raised { condition: "SYNTAX", .. }` literals, twenty-one copies of
    /// the same two constant fields, and every one of them would have had to
    /// name `delivery` too. Calling this instead means a field added here is
    /// free for all of them, which is the same argument `ClauseState` makes
    /// one level up.
    pub(crate) fn syntax(number: u16, sub: u16, additional: Vec<Substitution>) -> Raised {
        Raised {
            condition: Cow::Borrowed("SYNTAX"),
            number,
            sub,
            additional,
            rc: Some(number.to_string().into_bytes()),
            delivery: Delivery::default(),
        }
    }

    /// A condition that is not `SYNTAX`, carrying no catalogue entry.
    ///
    /// **`number`/`sub` are `0` and can never be rendered**, which is a
    /// property of how these are raised rather than a hope. Measured, the
    /// untrapped default action for `USER`, `ERROR`, `FAILURE`, `NOVALUE`,
    /// `NOSTRING`, `NOTREADY` and `LOSTDIGITS` is to *ignore* the condition
    /// -- `raise error 5` at top level with no trap prints nothing and exits
    /// 0 -- so `exec_raise` applies that default itself and one of these
    /// never reaches `execute`'s reporting arm. `HALT` is the one non-syntax
    /// condition whose default *is* fatal (`Error 4.1`, rc 252, measured),
    /// and it is built through [`Raised::syntax`]'s numbered path instead,
    /// precisely so it has a catalogue entry to render.
    pub(crate) fn condition(name: Cow<'static, str>) -> Raised {
        Raised {
            condition: name,
            number: 0,
            sub: 0,
            additional: Vec::new(),
            rc: None,
            delivery: Delivery::default(),
        }
    }

    /// Whether this condition would *report* if nothing traps it.
    ///
    /// `false` is exactly [`Raised::condition`]'s own zero-numbered kind,
    /// whose untrapped default action is measured to be silence. The check
    /// is spelled on `number` rather than on the condition name because the
    /// name is open-ended (`USER anything`) while the numbering is not, and
    /// because `HALT` -- the one non-`SYNTAX` condition whose default action
    /// *is* a report -- is built through the numbered path precisely so this
    /// answers `true` for it.
    pub(crate) fn reportable(&self) -> bool {
        self.number != 0
    }

    /// 4.1, `HALT`'s own untrapped default action: "Program interrupted with
    /// HALT condition", measured at rc 252 for `raise halt` with no trap
    /// enabled, and measured *again* with `signal on halt` enabled in the
    /// same (top-level) activation -- the tail-less `RAISE` terminates that
    /// activation before its own trap can see the condition, so the trap
    /// does not fire.
    pub(crate) fn halt() -> Raised {
        Raised {
            condition: Cow::Borrowed("HALT"),
            number: 4,
            sub: 1,
            // The `(4, 1)` catalogue entry is "Program interrupted with &1
            // condition.", so the condition's own name is the substitution
            // -- measured, the oracle prints `HALT`, and a version with no
            // substitution prints the literal `&1`.
            additional: vec![b"HALT".to_vec()],
            // `None` rather than `4`: `RC` is measured to carry the major
            // only for `SYNTAX` (42 for `say 1/0`, 40 for `raise syntax
            // 40.4`) and to be left untouched for a trapped `NOVALUE`. A
            // trapped `HALT` is not measured either way, so this follows the
            // non-`SYNTAX` row rather than inventing a third rule.
            rc: None,
            delivery: Delivery::default(),
        }
    }

    /// 41.1: a nonnumeric value used in arithmetic. `value` is the
    /// operand's own text, verbatim -- measured, `say 'abc' + 1` reports
    /// `Nonnumeric value ("abc")`, the operand as it renders, not upcased
    /// or otherwise transformed.
    pub(crate) fn nonnumeric(value: &[u8]) -> Raised {
        Raised::syntax(41, 1, vec![value.to_vec()])
    }

    /// 26.8: `**`'s right operand is not a whole number, **including not
    /// being a number at all**. Measured: `2 ** 'x'` and `2 ** 2.5` both
    /// give 26.8 ("found \"x\""/"found \"2.5\""), while the identical
    /// failure on the *left* operand is the ordinary 41.1 (`'y' ** 2` is
    /// 41.1, `'y' ** 'x'` is still 41.1 -- the base is checked first).
    /// This is deliberately not routed through `nonnumeric`: the oracle's
    /// own asymmetry between the two operands is the fact being
    /// reproduced, not an implementation shortcut. `found` is the
    /// exponent's own text; used when the exponent does not even parse as
    /// a number, so there is no `Number` for `rexx-num`'s own
    /// `ArithError::PowerExponentNotWhole` to carry.
    pub(crate) fn power_exponent_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(26, 8, vec![found.to_vec()])
    }

    /// 34.901: the prefix `\` operator's operand is not a logical value.
    /// A logical value is *exactly* the one-byte string `0` or `1`, no
    /// coercion -- this is a text check, never a numeric one, which is
    /// why the caller passes the operand's own rendered text rather than
    /// anything from `to_number`. Measured: `say \'abc'` gives 34.901,
    /// `Logical value must be exactly "0" or "1"; found "abc"`.
    pub(crate) fn not_logical(found: &[u8]) -> Raised {
        Raised::syntax(34, 901, vec![found.to_vec()])
    }

    /// 11.1: "Insufficient control stack space" -- D19's evaluation-depth
    /// limit (`eval.rs`'s own `MAX_EVAL_DEPTH`). No substitution: measured
    /// against the oracle's own parse-side 11.1 (nested parens/calls,
    /// `phase-4-exclusions.txt`'s Deviation 2), the catalogue's `(11, 1)`
    /// entry carries none either.
    pub(crate) fn insufficient_stack() -> Raised {
        Raised::syntax(11, 1, Vec::new())
    }

    /// 34.6: one element of a comma-separated logical list
    /// (`ExprKind::Logical`, `if a, b then` and friends) is not a logical
    /// value. A distinct sub-number from `not_logical`'s 34.901, and
    /// deliberately not shared with it even though the underlying check is
    /// identical (exactly `0` or `1`, text not numeric) -- measured, `if 1,
    /// 'x' then` gives 34.6 ("Value of logical list expression element
    /// must be exactly \"0\" or \"1\"; found \"x\""), a different message
    /// from `&`'s 34.901 for the identical bad value. `IF`/`WHEN`/`WHILE`/
    /// `UNTIL`'s own 34.1/34.2/34.3/34.4 are for when the *whole* condition
    /// is a single expression, not a list, and are Tasks 9-11's to raise
    /// when they exist; this crate has no instruction context yet to
    /// prefer one of those over 34.6, so 34.6 is `ExprKind::Logical`'s own
    /// answer regardless of which keyword built the list.
    pub(crate) fn logical_list_element(found: &[u8]) -> Raised {
        Raised::syntax(34, 6, vec![found.to_vec()])
    }

    /// 44.1: an internal routine reached through `ExprKind::Call`'s
    /// expression form (`f(...)`, Task 4, `eval.rs`'s `eval_call`) ran to
    /// completion without a value to hand back -- a bare `RETURN`, measured
    /// against the oracle in a clean directory: `say f(1)` into `f: return`
    /// gives rc 212 and
    ///
    /// ```text
    /// Error 44 running .../f.rex line 1:  Function or message did not return data.
    /// Error 44.1:  No data returned from function "F".
    /// ```
    ///
    /// `name` is the resolved label's own spelling (already upcased for a
    /// `CallTarget::Symbol`, which is the only target this can ever fire
    /// for -- a `CallTarget::Literal` never resolves at all in this phase,
    /// see `eval_call`'s own doc). **Not** the same path as running off the
    /// end of the routine with no `RETURN` at all: that is `Ended::Exited`,
    /// measured to end the whole program silently (rc 0, no stdout) rather
    /// than raise anything, exactly as falling off the end of a `CALL`ed
    /// routine already does (`resolve_and_run_call`'s own doc, `run.rs`).
    pub(crate) fn no_data_returned(name: &[u8]) -> Raised {
        Raised::syntax(44, 1, vec![name.to_vec()])
    }

    /// 16.1: `SIGNAL`/`SIGNAL VALUE` named a target that matches no label in
    /// the running activation's own body. `name` is the resolved target's
    /// own bytes -- already upcased for a bare symbol, verbatim for a quoted
    /// literal or a `SIGNAL VALUE` expression's rendered text.
    ///
    /// **No fallback exists, unlike `CALL`'s own builtin/external search**:
    /// a `SIGNAL` target is only ever a label, so this is the oracle's real
    /// answer and not a placeholder for a table a later phase owns. Measured
    /// in a clean directory: `signal nowhere` gives rc 240 and
    /// `Label "NOWHERE" not found.`; `signal "sub"` (quoted, lowercase) with
    /// `sub:` present gives the same error naming `"sub"` verbatim, because
    /// the label itself is stored upcased and a quoted target is matched
    /// case-sensitively rather than upcased on the way in -- `signal Sub`
    /// (a bare, mixed-case symbol) and `signal "SUB"` both resolve.
    pub(crate) fn label_not_found(name: &[u8]) -> Raised {
        Raised::syntax(16, 1, vec![name.to_vec()])
    }

    /// 17.1: a `PROCEDURE` that is not the first instruction executed after
    /// an internal `CALL` or function invocation. No substitutions.
    ///
    /// Measured at all four shapes, in a clean directory, all rc 239: at top
    /// level; as the first instruction of a label *fallen into* rather than
    /// called; after a `NOP` in a called routine; and inside `interpret
    /// "procedure"` as a called routine's first clause. Two labels between
    /// the call and the `PROCEDURE` do **not** raise it --
    /// `Activation::first_instruction_pending` carries that whole table.
    pub(crate) fn procedure_out_of_place() -> Raised {
        Raised::syntax(17, 1, Vec::new())
    }

    /// 40.3: `USE STRICT ARG` with fewer arguments than it has targets
    /// without defaults. `routine` is the callee's own resolved name, upcased,
    /// and `minimum` counts the targets that must be supplied.
    ///
    /// Measured: `call sub2 1` into `use strict arg p, q` gives rc 216 and
    /// `Not enough arguments in invocation of SUB2; minimum expected is 2.`
    /// A target carrying a default satisfies the minimum -- `use strict arg
    /// p, q = 'dflt'` with one argument runs and prints `[1][dflt]`.
    pub(crate) fn not_enough_arguments(routine: &[u8], minimum: usize) -> Raised {
        Raised::syntax(
            40,
            3,
            vec![routine.to_vec(), minimum.to_string().into_bytes()],
        )
    }

    /// 40.4: `USE STRICT ARG` with more arguments than it has targets, and
    /// no trailing `...`.
    ///
    /// Measured: `call sub2 1,2,3` into `use strict arg p` gives rc 216 and
    /// `Too many arguments in invocation of SUB2; maximum expected is 1.`
    /// With `use strict arg p, q, ...` the same three arguments run clean, so
    /// `allow_optionals` suppresses this check and not the 40.3 one.
    pub(crate) fn too_many_arguments(routine: &[u8], maximum: usize) -> Raised {
        Raised::syntax(
            40,
            4,
            vec![routine.to_vec(), maximum.to_string().into_bytes()],
        )
    }

    /// 40.5: a required argument was **omitted in place** rather than left
    /// off the end -- `substr('abc',,2)`. `routine` is the callee's own name,
    /// upcased, and `position` is 1-based.
    ///
    /// Measured, rc 216: `say substr('abc',,2)` gives
    /// `Missing argument in invocation of SUBSTR; argument 2 is required.`
    ///
    /// **A distinct answer from 40.3, and the two are told apart by where the
    /// omission is, not by how many arguments were written.** Measured, an
    /// omission at the *end* of the list is not an argument at all: `q(1,)`
    /// into `q: return arg()` answers 1, `q(1,,2,,)` answers 3, and `q(,)`
    /// answers 0 -- so `say length('abc',)` runs and prints 3 where `say
    /// length(,)` is 40.3 with a minimum of 1, not 40.5. `rexx-parse` already
    /// drops those trailing positions (`ExprKind::List`'s own doc comment,
    /// citing `parseArgList`'s `realcount`), so an argument list arriving here
    /// has interior omissions only.
    pub(crate) fn missing_argument(routine: &[u8], position: usize) -> Raised {
        Raised::syntax(
            40,
            5,
            vec![routine.to_vec(), position.to_string().into_bytes()],
        )
    }

    /// 40.12: a builtin argument that has to be a whole number is not one.
    /// `routine` is the builtin's own name as its table row spells it,
    /// `position` is 1-based **in the call's own argument list**, and `found`
    /// is the argument's own **rendered value**.
    ///
    /// That `found` is the rendered value -- not the source spelling, and not
    /// a re-rendering under the `DIGITS` in force at the call -- is measured
    /// rather than assumed, with a program in which all three differ:
    ///
    /// ```text
    /// numeric digits 3 ; zz = 2 / 3 ; numeric digits 9 ; say left('ab', zz)
    /// ->  40.12  LEFT argument 2 must be a whole number; found "0.667".
    /// ```
    ///
    /// The spelling is `zz`; the current-`DIGITS` rendering would be
    /// `0.666666667`; `0.667` is the value's own rendering, fixed by the
    /// `DIGITS 3` in force when the division created it (D15). The same
    /// program with the same value in a *pad* position reports `found
    /// "0.667"` under [`argument_not_a_pad`]'s 40.23, so the two sub-codes
    /// answer the question the same way.
    ///
    /// **What counts as a whole number here is not the current `NUMERIC
    /// DIGITS`.** The oracle converts through `Numerics::ARGUMENT_DIGITS`,
    /// which is 18 on a 64-bit build, and measured in both directions:
    /// `numeric digits 2 ; left('ab','1.0000001')` is 40.12 where a
    /// two-digit conversion would have rounded it to a whole `1`, and
    /// `left('ab','1.0000000000000000000004')` succeeds, because rounding
    /// *that* to 18 digits leaves `1`. A value needing more than 18 digits is
    /// rejected however it is spelled -- `left('ab','1E18')` is 40.12.
    ///
    /// [`argument_not_a_pad`]: Raised::argument_not_a_pad
    pub(crate) fn argument_not_whole(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            12,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.23: a builtin's pad argument is not exactly one character.
    /// Substitutions as [`argument_not_whole`]'s, and `found` is the
    /// rendered value for the same measured reason.
    ///
    /// **A pad is checked whether or not it could ever be used**, measured:
    /// `left('',0,'xx')` and `right('',0,'xx')` are both 40.23 though the
    /// result is the null string either way, and `substr('abc',0,5,'xx')` is
    /// 40.23 rather than the 93.924 its zero position would otherwise give.
    /// The null string is not a pad either -- `space('a b c',1,'')` is 40.23
    /// with `found ""`.
    ///
    /// [`argument_not_whole`]: Raised::argument_not_whole
    pub(crate) fn argument_not_a_pad(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            23,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 93.923: a length argument converted to a whole number but is
    /// negative. No routine name and no position in the message, only the
    /// value.
    ///
    /// **A different major from the 40.x family, and a different exit code**:
    /// measured, `say substr('abc',2,-1)` is `Error 93 ... Incorrect call to
    /// method.` / `Error 93.923:  Invalid length argument specified; found
    /// "-1".` at **rc 163**, where every 40.x above is rc 216.
    ///
    /// `found` is the value **after** conversion to a whole number, not the
    /// argument's own text, and that is measured with the two spellings
    /// apart: `left('ab','-1.0')`, `left('ab',' -1 ')` and
    /// `left('ab','-1e0')` all report `found "-1"`. The 40.12 above reports
    /// the argument's text instead, so the two families genuinely disagree
    /// about what they name.
    pub(crate) fn invalid_length(found: &[u8]) -> Raised {
        Raised::syntax(93, 923, vec![found.to_vec()])
    }

    /// 93.924: a position argument converted to a whole number but is zero
    /// or negative. Same shape and same rc 163 as [`invalid_length`], and
    /// `found` is likewise the converted value -- measured,
    /// `substr('abc','0.0')` reports `found "0"`.
    ///
    /// Which of the two a builtin raises is per argument, not per
    /// constraint: measured, `substr('abc',0)` is 93.924 while
    /// `substr('abc',2,-1)` is 93.923, and `insert('-','abc',-1)` is neither
    /// (see [`argument_not_non_negative`]).
    ///
    /// [`invalid_length`]: Raised::invalid_length
    /// [`argument_not_non_negative`]: Raised::argument_not_non_negative
    pub(crate) fn invalid_position(found: &[u8]) -> Raised {
        Raised::syntax(93, 924, vec![found.to_vec()])
    }

    /// 93.906: a count argument converted to a whole number but is negative.
    /// `position` is 1-based **in the underlying method's argument list**,
    /// which is the builtin's own list less the positions the method takes
    /// as its receiver and its earlier operands -- measured,
    /// `copies('ab',-1)` reports `Method argument 1`, `insert('-','abc',-1)`
    /// reports `Method argument 2` and `changestr('a','banana','X',-1)`
    /// reports `Method argument 3`, from builtin positions 2, 3 and 4.
    ///
    /// The third of the trio with [`invalid_length`] and
    /// [`invalid_position`], at the same rc 163 and with `found` likewise
    /// the converted value: `copies('ab','-1.0')` reports `found "-1"`.
    ///
    /// [`invalid_length`]: Raised::invalid_length
    /// [`invalid_position`]: Raised::invalid_position
    pub(crate) fn argument_not_non_negative(position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            93,
            906,
            vec![position.to_string().into_bytes(), found.to_vec()],
        )
    }

    /// 93.915: an option argument's first letter is not one of the ones the
    /// builtin accepts. `valid` is the accepted set as the oracle spells it
    /// in the message, and `found` is the **whole option string**, not the
    /// letter that was rejected.
    ///
    /// Measured, both parts: `strip('ab','Xyz')` gives `Method option must
    /// be one of "BLT"; found "Xyz".` and `verify('a','b','Xyz')` gives the
    /// same shape with `"MN"`. The null string is rejected too, with `found
    /// ""` -- `strip('ab','')` and `verify('abcde','abc','')` are both
    /// 93.915 -- so an empty option is not "omitted".
    ///
    /// Only the first letter is examined, and case-insensitively: measured,
    /// `strip('  ab  ','Leading')` and `strip('  ab  ','l')` both strip
    /// leading blanks only, and `verify('abcde','abc','Nope')` is 4, the
    /// same as `'N'`.
    pub(crate) fn invalid_option(valid: &str, found: &[u8]) -> Raised {
        Raised::syntax(93, 915, vec![valid.as_bytes().to_vec(), found.to_vec()])
    }

    /// 5: a result string too large to allocate. No sub-number and no
    /// substitution, which is why this is the one raiser here built with a
    /// sub of `0`: measured, `say left('ab','999999999999999999')` prints
    ///
    /// ```text
    ///      1 *-* say left('ab','999999999999999999')
    /// Error 5 running /abs/p.rex line 1:  System resources exhausted.
    /// ```
    ///
    /// at rc 251, with **no** `Error 5.x:` second line -- exactly what
    /// `Raised::report` already writes for a zero sub.
    ///
    /// The oracle reaches this by asking the allocator and being refused,
    /// not by testing the requested size against a limit: `right`, `center`,
    /// `space`, `substr`, `copies`, `insert` and `overlay` were each
    /// measured reporting it for a length argument of `123456789012345678`,
    /// the same value at which `left` above succeeds in converting the
    /// argument and fails to allocate. Reproducing the *mechanism* rather than a
    /// threshold is why the call sites ask `Vec::try_reserve_exact` -- a
    /// chosen cut-off would be a number this project could not measure, and
    /// an unguarded allocation of that size aborts the process rather than
    /// raising anything.
    pub(crate) fn system_resources() -> Raised {
        Raised::syntax(5, 0, Vec::new())
    }

    /// 40.28: an argument that has to be either a character class name or a
    /// single character is neither. Substitutions as [`argument_not_whole`]'s.
    ///
    /// The neighbour of [`argument_not_a_pad`]'s 40.23, and the two really do
    /// split by argument position rather than by value: measured,
    /// `xrange('a','zz')` is 40.23 naming argument 2 where `xrange('zz','a')`
    /// is this one naming argument 1, for the same offending string.
    ///
    /// The null string reaches it too -- `xrange('')` is 40.28 with
    /// `found ""` -- because the oracle tests for a length of exactly one and
    /// treats everything else as a class name to look up.
    ///
    /// [`argument_not_whole`]: Raised::argument_not_whole
    /// [`argument_not_a_pad`]: Raised::argument_not_a_pad
    pub(crate) fn argument_not_a_pad_or_class_name(
        routine: &[u8],
        position: usize,
        found: &[u8],
    ) -> Raised {
        Raised::syntax(
            40,
            28,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 93.927: `D2X`/`D2C` were asked to convert a negative value without a
    /// length to hold the sign extension. No substitutions.
    ///
    /// Measured, both at rc 163: `say d2x(-1)` and `say d2c(-1)` are
    /// `Length must be specified to convert a negative value.`, and the same
    /// calls with any length at all succeed -- `d2x(-1,1)` is `F`.
    pub(crate) fn length_required_for_negative() -> Raised {
        Raised::syntax(93, 927, Vec::new())
    }

    /// 93.928: `D2X`'s value argument is not a whole number the current
    /// `NUMERIC DIGITS` can hold. `found` is the argument's own **rendered
    /// value**, which is the pair with [`argument_not_whole`]'s measurement:
    /// `numeric digits 3 ; zz = 2 / 3 ; numeric digits 9 ; say d2x(zz)`
    /// reports `found "0.667"`.
    ///
    /// **The setting bounds the value, not the text**, measured in both
    /// directions at `DIGITS 3`: `d2x('000123')` is `7B`, since the leading
    /// zeros are not digits of the value, while `d2x('1E3')` and `d2x(1000)`
    /// are both this error -- one thousand needs four digits however it is
    /// spelled.
    ///
    /// [`argument_not_whole`]: Raised::argument_not_whole
    pub(crate) fn d2x_value_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(93, 928, vec![found.to_vec()])
    }

    /// 93.929: [`d2x_value_not_whole`]'s twin for `D2C`, measured to be the
    /// same rule with a different number -- `d2c('abc')` and `d2x('abc')`
    /// differ only in the sub-code and the routine the text names.
    ///
    /// [`d2x_value_not_whole`]: Raised::d2x_value_not_whole
    pub(crate) fn d2c_value_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(93, 929, vec![found.to_vec()])
    }

    /// 93.935: `X2D`'s *result* does not fit the current `NUMERIC DIGITS`.
    /// The substitution is the setting itself, not the value.
    ///
    /// **The bound is on the result and not on how many bytes went in**,
    /// which is what separates this from [`d2x_value_not_whole`]'s check.
    /// Measured at `DIGITS 3`: `x2d('ff')` is 255 and `x2d('ffff')` is this
    /// error naming 3.
    ///
    /// [`d2x_value_not_whole`]: Raised::d2x_value_not_whole
    pub(crate) fn x2d_result_too_large(digits: u64) -> Raised {
        Raised::syntax(93, 935, vec![digits.to_string().into_bytes()])
    }

    /// 93.936: [`x2d_result_too_large`]'s twin for `C2D`.
    ///
    /// The pair of measurements that shows the bound is the result's:
    /// `numeric digits 9 ; c2d(copies('00'x,10)||'01'x)` is `1` from eleven
    /// bytes, while `numeric digits 9 ; c2d('ffffffff'x)` is this error from
    /// four.
    ///
    /// [`x2d_result_too_large`]: Raised::x2d_result_too_large
    pub(crate) fn c2d_result_too_large(digits: u64) -> Raised {
        Raised::syntax(93, 936, vec![digits.to_string().into_bytes()])
    }

    /// 93.931/93.932: a hexadecimal or binary string carries whitespace where
    /// it may not -- at the very start, or at the very end. `position` is
    /// 1-based.
    ///
    /// Measured, all rc 163: `x2c(' 4142')` names position 1, `x2c('4142 ')`
    /// names 5, and `x2c('41 42  ')` names 7 -- the *last* of a trailing run,
    /// not the first. The binary twin is the same shape: `b2x(' 1010')` names
    /// position 1 and `b2x('1010 ')` names 5.
    pub(crate) fn misplaced_whitespace(notation: Notation, position: usize) -> Raised {
        let sub = match notation {
            Notation::Hex => 931,
            Notation::Binary => 932,
        };
        Raised::syntax(93, sub, vec![position.to_string().into_bytes()])
    }

    /// 93.933/93.934: a byte that is neither a digit of the notation nor one
    /// of the two bytes that may separate its groups. The substitution is the
    /// offending byte itself.
    ///
    /// It is a byte and not text: measured, `x2c('41'||'ff'x)` reports the
    /// raw `0xff` and `x2c('41'||'01'x)` reports `?`, which is
    /// [`displayable`]'s rule applied to the finished line rather than
    /// anything this raiser does.
    ///
    /// [`displayable`]: crate::error::displayable
    pub(crate) fn invalid_digit(notation: Notation, character: u8) -> Raised {
        let sub = match notation {
            Notation::Hex => 933,
            Notation::Binary => 934,
        };
        Raised::syntax(93, sub, vec![vec![character]])
    }

    /// 93.976/93.977: the groups of a hexadecimal or binary string are not
    /// sized as the notation requires. No substitutions.
    ///
    /// Measured: `x2c('414 243')` is 93.976, where `x2c('414 2434')` and
    /// `b2x('101 0000')` both convert. The rule those three share is written
    /// out where it is enforced, in `builtin/convert.rs`'s module doc.
    pub(crate) fn invalid_grouping(notation: Notation) -> Raised {
        let sub = match notation {
            Notation::Hex => 976,
            Notation::Binary => 977,
        };
        Raised::syntax(93, sub, Vec::new())
    }

    /// 88.928: `USE ARG >name` where the caller did not pass a variable
    /// reference. `position` is 1-based; `found` is the argument's own
    /// **rendered value**.
    ///
    /// That `found` is the value and not the argument's spelling is measured
    /// rather than assumed, because the obvious probe cannot tell them apart:
    /// a variable named `caller` reports `found "caller"` whichever rule
    /// holds. Three programs that do discriminate, all rc 168 -- `zebra =
    /// 'orig'; call sub2 zebra` reports `found "orig"`, a literal argument
    /// `'literal-value'` reports its own text, and passing it second reports
    /// `The 2 argument`.
    pub(crate) fn not_a_variable_reference(position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            88,
            928,
            vec![position.to_string().into_bytes(), found.to_vec()],
        )
    }

    /// 88.931: `USE ARG >name` where the caller omitted that position
    /// entirely. `position` is 1-based.
    ///
    /// A different sub-number from [`not_a_variable_reference`]'s 88.928, and
    /// measured rather than assumed to be the same: `call sub2 1` into `use
    /// arg p, >q` gives rc 168 and `Argument 2 was omitted. A
    /// VariableReference argument is required.`, where passing a *wrong-kind*
    /// value in that position gives 88.928 instead. An omission and a bad
    /// value are two different complaints here.
    ///
    /// [`not_a_variable_reference`]: Raised::not_a_variable_reference
    pub(crate) fn variable_reference_omitted(position: usize) -> Raised {
        Raised::syntax(88, 931, vec![position.to_string().into_bytes()])
    }

    /// 88.929: `USE ARG >name` where the target is a **stem** and the caller
    /// passed a reference to a **simple** variable.
    ///
    /// `reference` is the *caller's* variable name, not the target's -- the
    /// opposite of 98.995 below, and measured rather than assumed with a
    /// variable whose value differs from its name: `p = 'value-not-name'`
    /// passed as `>p` into `use arg >q.` reports rc 168 and `The 1 argument
    /// must be a VariableReference for a Stem variable; found "P".` The same
    /// position under 88.928 reports `value-not-name`, so the two families
    /// genuinely disagree about what they name.
    pub(crate) fn not_a_stem_variable_reference(position: usize, reference: &[u8]) -> Raised {
        Raised::syntax(
            88,
            929,
            vec![position.to_string().into_bytes(), reference.to_vec()],
        )
    }

    /// 88.930: the mirror of [`not_a_stem_variable_reference`] -- the target
    /// is a **simple** variable and the caller passed a **stem** reference.
    ///
    /// Measured: `p.1 = 'value-not-name'` passed as `>p.` into `use arg >q`
    /// gives rc 168 and `... must be a VariableReference for a simple
    /// variable; found "P.".` The name carries the trailing period, which is
    /// the stem's own spelling and needs no shaping here.
    ///
    /// [`not_a_stem_variable_reference`]: Raised::not_a_stem_variable_reference
    pub(crate) fn not_a_simple_variable_reference(position: usize, reference: &[u8]) -> Raised {
        Raised::syntax(
            88,
            930,
            vec![position.to_string().into_bytes(), reference.to_vec()],
        )
    }

    /// 98.995: `USE ARG >name` whose target is not currently unset. `name` is
    /// the target's own spelling.
    ///
    /// Measured, rc 158: `p = 'p-orig'; q = 'q-orig'; call sub >p` into `use
    /// arg >q` gives `Unable to reference variable "Q"; it must be an
    /// uninitialized local variable.` A stem target reports its own spelling
    /// including the period -- `Q.` -- which is what `use_target_name`
    /// already produces, so neither case needs shaping here.
    ///
    /// **The message's "local" is not the condition.** An exposed target
    /// raises this when it holds a value and does not when it is unset;
    /// `run.rs`'s `target_is_uninitialised` has that pair and is where the
    /// rule lives.
    pub(crate) fn variable_reference_not_uninitialised(name: &[u8]) -> Raised {
        Raised::syntax(98, 995, vec![name.to_vec()])
    }

    /// 98.993: `USE LOCAL` as the first instruction executed of a top-level
    /// program. No substitutions.
    ///
    /// Measured, rc 158: `use local outer` on line 1 of a program gives `The
    /// USE LOCAL instruction may only be used from method invocations.` This
    /// crate has no method invocations at all, so `USE LOCAL` is never legal
    /// here -- implementing it means implementing which of its two refusals
    /// applies, and [`use_local_not_first`] is the other one.
    ///
    /// [`use_local_not_first`]: Raised::use_local_not_first
    pub(crate) fn use_local_outside_method() -> Raised {
        Raised::syntax(98, 993, Vec::new())
    }

    /// 99.910: `USE LOCAL` anywhere other than the first instruction executed
    /// of a top-level program. No substitutions.
    ///
    /// Measured, rc 157, at three shapes: as the *second* instruction of a
    /// program, as the first instruction of a called routine, and after a
    /// `PROCEDURE` in a called routine. The second of those is the one that
    /// makes this the wider case -- a called routine's first instruction is
    /// still "not first after a *method* invocation", so it lands here rather
    /// than on 98.993.
    ///
    /// The shape that would separate "is a method" from "was entered by a
    /// call" cannot be written in this phase, since no method invocation
    /// exists to write it with; both errors are reproduced from the four
    /// shapes that can be.
    pub(crate) fn use_local_not_first() -> Raised {
        Raised::syntax(99, 910, Vec::new())
    }
}

/// Converts a `rexx-num` arithmetic failure into a `Raised`.
///
/// The `(major, sub)` pair comes from `ArithError::sub_code`, made `pub` in
/// `rexx-num` for exactly this caller (`4a320f1c`) rather than hand-copied
/// here: this task originally flagged that `sub_code` was private and
/// shipped a two-variant stopgap covering only what its own tests had
/// independently verified against the oracle (`DivideByZero`,
/// `PowerExponentNotWhole`), sub `0` elsewhere. The accessor landing
/// retires that stopgap -- every `ArithError` variant now gets its real
/// sub-number, not only the two this task happened to exercise.
impl From<ArithError> for Raised {
    fn from(error: ArithError) -> Raised {
        // `additional()` and `sub_code()` both borrow, so either can run
        // first; ordered to match the doc comment's own telling.
        let additional = error.additional();
        let (number, sub) = error.sub_code();
        Raised::syntax(number, sub, into_substitutions(additional))
    }
}

/// Converts a `rexx-parse` translation failure into the condition the oracle
/// raises for it.
///
/// **Measured, and the reason this exists** (4b Task 2, Step 5b): `interpret
/// "do forever then"` on line 2 of a two-line program gives the oracle
///
/// ```text
///      2 *-* do forever then
///      2 *-* interpret "do forever then"
/// Error 27 running /abs/p1.rex line 2:  Invalid DO or LOOP syntax.
/// Error 27.901:  Incorrect data following FOREVER keyword on the loop; found "THEN".
/// ```
///
/// at rc 229 -- a real, trappable SYNTAX condition, not a translation-time
/// refusal. Before this, a fragment that did not parse was a `Loud` failure
/// at rc 120, which was correct-but-loud while `INTERPRET` was unreachable
/// and a live divergence once 4b's Task 1 made it reachable.
///
/// **What this does not carry, and it is not an oversight.** `ParseError`
/// has a major, a sub and the clause's start byte, and deliberately no
/// substitution values -- `rexx-parse`'s own `error.rs` module note has the
/// measurement behind that decision and what it owes Phase 4. So the sub
/// line renders its catalogue template with `&1` passed through where the
/// oracle writes `found "THEN"`. Everything else matches: the condition, the
/// major, the sub, the exit code, and the enclosing clause echoes. That is
/// the same bound `execute`'s own top-level parse arm already states, and
/// closing it is the same job in both places.
impl From<&ParseError> for Raised {
    fn from(error: &ParseError) -> Raised {
        Raised::syntax(error.code, error.sub, Vec::new())
    }
}

/// Either kind of failure a clause can produce: a construct 4a does not
/// implement (`Loud`) or a real Rexx condition (`Raised`). `step` and
/// everything above it propagate this rather than either alone, since a
/// clause containing an expression can fail either way -- `eval`'s own
/// `ExprKind::Message` arm is `Loud` (not implemented, Phase 5's), its
/// `1 / 0` arm is `Raised` (implemented, and this is what it does).
#[derive(Debug)]
pub(crate) enum Failure {
    Loud(Loud),
    Raised(Raised),
    /// **Not a failure at all** -- `EXIT` inside a routine reached through
    /// `ExprKind::Call`'s expression form (Task 4), or that routine falling
    /// off its own end, either of which ends the whole program exactly as
    /// the same event does when reached through `CALL` (`resolve_and_run_
    /// call`'s own doc, `run.rs`). `CALL`'s own instruction form carries
    /// this through `Flow::Exit`/`Ended::Exited` instead, entirely through
    /// `Ok` returns, because `step` and `run_activation` both return a
    /// `Flow`/`Ended` that has room for "the program is exiting" as a
    /// successful outcome. `eval`'s own return type is a plain `ObjRef`, with
    /// no such room, so this variant is what lets the same event travel
    /// through an expression instead: constructed once, in `eval_call`
    /// (`eval.rs`), and then propagated by every intervening `?` completely
    /// unremarked -- `step_in_temps_frame`'s and `resolve_and_run_call`'s own
    /// generic "an `Err` escaped, record a site and re-throw" paths do not
    /// need to know this variant exists, because sealing a site nothing
    /// prints is harmless (`execute`, `lib.rs`, never calls `Raised::report`
    /// for it) and re-throwing is exactly what unwinding every nested `CALL`
    /// to end the whole program needs regardless of how many levels deep the
    /// `EXIT` was. `execute`'s own top-level match is the one place this is
    /// finally read, and there it is handled exactly like an ordinary
    /// `Ok(value)`: same `exit_code_for`, no stderr report.
    Exited(Option<ObjRef>),
}

impl From<Loud> for Failure {
    fn from(loud: Loud) -> Failure {
        Failure::Loud(loud)
    }
}

impl From<Raised> for Failure {
    fn from(raised: Raised) -> Failure {
        Failure::Raised(raised)
    }
}

/// Where a failing clause was found -- `Interp::failure_site`'s own type
/// (`lib.rs`), and what `run.rs`'s `record_failure_site` fills in.
///
/// A named struct rather than a `(usize, Vec<u8>, usize)` tuple **on
/// purpose**: `line` and `indent` are both bare `usize`s, and a position-only
/// tuple lets the two transpose with nothing to catch it -- the failure mode
/// would be plausible-looking, wrong stderr, not a compile error or a panic.
/// Naming the fields removes that whole class rather than trusting call-site
/// order.
///
/// **One of these per *activation-like level*, not per nesting level.** 4b's
/// Task 2 turned the single site into a stack of them (`ClauseSite::sites`),
/// and the unit the stack counts is measured: an error inside three nested
/// `DO`s echoes once, not four times, while the same error inside an
/// `INTERPRET` fragment echoes twice and inside a fragment inside a fragment
/// three times. `Interp::failure_site` stays first-wins *within* a level and
/// `run.rs`'s own `seal_site_level` is what closes one off and starts the
/// next.
#[derive(Clone)]
pub(crate) struct FailureSite {
    pub(crate) line: usize,
    pub(crate) text: Vec<u8>,
    /// Spaces to prefix `text` with on the echo line, Task 11's own
    /// nesting-depth quantity. **Computed statically from the AST** (`run.rs`'s
    /// `static_indent`), never carried on a running counter: Task 10's own
    /// report concluded the depth is derivable from the instruction list
    /// alone with no runtime block stack, and this task's own oracle
    /// measurements confirm it for the ordinary case and for one
    /// LEAVE/ITERATE error family (28.5) besides -- see `static_indent`'s
    /// own doc comment and the report for the transcripts. A mutable
    /// per-`Interp` counter was the first design tried here and was
    /// abandoned once it became clear it would need perfect symmetric
    /// bookkeeping on every exit path out of every construct, including the
    /// error paths and the `run_bounded` `Goto`-absorption case `Flow`'s own
    /// doc comment warns about -- exactly the class of defect this crate's
    /// skipped-`pop_frame` discussion elsewhere already flags. A pure
    /// function of `(instructions, index)` cannot desync, because there is
    /// nothing stateful to desync.
    pub(crate) indent: usize,
}

/// Where the failing clause is, which is everything the report needs from
/// outside this module.
///
/// Passed in rather than reached for: `error.rs` owns the *format*, and the
/// instruction loop owns knowing which clause failed. That split is why this
/// module needs no access to `Interp`, the program or the source. Built from
/// the `FailureSite` stack plus the one thing it does not carry, the
/// program's own path -- `execute` (`lib.rs`) is the one place both are in
/// hand together.
pub(crate) struct ClauseSite<'a> {
    /// The program's path **as the oracle prints it**, absolute. Measured:
    /// the major line carries the full path, and `rexx-oracle`'s `normalize`
    /// masks the cwd, so an absolute path is comparable across machines.
    pub(crate) path: &'a str,
    /// One entry per activation-like level the condition escaped through,
    /// **innermost first** -- 4b's Task 2, and the whole reason this is a
    /// slice rather than the single site 4a carried.
    ///
    /// Measured against the oracle (`interpret "say 2 & 1"` on line 2 of a
    /// two-line program):
    ///
    /// ```text
    ///      2 *-* say 2 & 1
    ///      2 *-* interpret "say 2 & 1"
    /// ```
    ///
    /// Each entry carries its own line and its own **absolute** printed
    /// indent, which is why this module does no arithmetic on either: an
    /// inner level's line is not derivable from an outer one's (measured,
    /// every echo of a fragment carries the *enclosing* `INTERPRET` clause's
    /// line, not the fragment's own), and its indent is not derivable from
    /// the depth of the stack (measured, a fragment's own clauses sit at the
    /// enclosing clause's indent plus whatever nests them *inside* the
    /// fragment: `interpret "do jj = 1 to 1; say 2 & 1; end"` at top level
    /// echoes the inner clause at 2 and the `INTERPRET` at 0).
    ///
    /// Empty only in the "nothing recorded" case `execute` guards, which
    /// prints no echo at all rather than a blank one.
    pub(crate) sites: &'a [FailureSite],
}

impl Raised {
    /// `256 - major`, the whole rule.
    ///
    /// Verified across nine majors rather than the four the plan recorded:
    /// 7 -> 249, 24 -> 232, 25 -> 231, 26 -> 230, 33 -> 223, 34 -> 222,
    /// 41 -> 215, 42 -> 214, 98 -> 158.
    ///
    /// This is also why `NOT_IMPLEMENTED_EXIT` must stay outside 157..=253:
    /// majors 3 to 99 fill that band, so a loud failure inside it would be
    /// indistinguishable from a raised condition and a program *expecting*
    /// that condition would pass against a gap.
    pub(crate) fn exit_code(&self) -> i32 {
        256 - i32::from(self.number)
    }

    /// The exact bytes the oracle writes to stderr for this condition.
    ///
    /// Three lines, and every part of the shape is measured rather than
    /// inferred (`say 1` then a `SELECT` with no true `WHEN`, `cat -A`):
    ///
    /// ```text
    ///      4 *-* end
    /// Error 7 running /abs/path/f.rex line 4:  WHEN or OTHERWISE expected.
    /// Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.
    /// ```
    ///
    /// * The **clause echo appears with trace off**, which is the part that
    ///   surprises: this is not trace output and is not suppressed by
    ///   `TRACE OFF`.
    /// * The line number is **right-aligned in a six-character field**,
    ///   measured at one, two and three digits: `     4`, `    12`, `   105`.
    /// * **Two spaces after each colon**, on both error lines.
    /// * The major line's text is the catalogue's `(major, 0)` entry and the
    ///   sub line's is `(major, sub)`.
    ///
    /// **There is one echo line per entry in `site.sites`, innermost first**
    /// (4b's Task 2), so the three-line shape above is the one-entry case and
    /// not a special case in the code. The **line the major line names is the
    /// innermost entry's**, measured: `interpret "say 2 & 1"` on line 2 names
    /// line 2, and a raise on line 8 of a routine called from line 3 names
    /// line 8, not 3.
    ///
    /// **One raise renders the major line without its position span**, and
    /// only one: `RAISE PROPAGATE`, whose report reads `Error 42:  ...` where
    /// every other raise reads `Error 42 running <path> line 8:  ...`. See
    /// [`Delivery::positionless`], which is the flag, and the loop above,
    /// which is unaffected -- the echo lines are the same either way.
    ///
    /// `SAY` output goes to stdout and all of this to stderr, so their
    /// relative order is not observable (D17).
    pub(crate) fn report(&self, site: &ClauseSite<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        // `trace::push_clause` rather than a second copy of the same four
        // lines: the two used to be written out separately and documented as
        // byte-identical, and 4b's Task 2 needed the 40-column clamp on both.
        // Calling the one formatter is what makes "one quantity" true in the
        // code rather than only in a comment -- `push_clause` owns the
        // clamp, the six-wide line field and the indent, and this loop owns
        // only the order.
        for entry in site.sites {
            crate::trace::push_clause(&mut out, entry.line, entry.indent, &entry.text);
        }
        // The innermost entry's line, or `0` when nothing was recorded at all
        // -- `execute`'s own guard already substitutes a visible placeholder
        // entry for that case, so this fallback is unreachable from there and
        // exists so this function has no panic on the error path.
        let line = site.sites.first().map_or(0, |entry| entry.line);
        // `RAISE PROPAGATE` drops the position span and nothing else
        // (`Delivery::positionless`). Measured against the same program with
        // and without the `raise propagate`: the echo lines, the sub line and
        // the exit code are identical, and only ` running <path> line <n>`
        // goes.
        let position = if self.delivery.positionless {
            String::new()
        } else {
            format!(" running {} line {line}", site.path)
        };
        out.extend_from_slice(format!("Error {}{}:  ", self.number, position).as_bytes());
        out.extend_from_slice(&self.message(self.number, 0));
        out.push(b'\n');
        // **Sub `0` prints no second line at all**, measured: `raise syntax
        // 40` gives the major line and stops, where `raise syntax 40.4`
        // gives both. Reachable only through `RAISE` -- every raiser in the
        // crate names a real sub -- which is why 4a never had to know.
        if self.sub != 0 {
            out.extend_from_slice(format!("Error {}.{}:  ", self.number, self.sub).as_bytes());
            out.extend_from_slice(&self.message(self.number, self.sub));
            out.push(b'\n');
        }
        // **Applied once, to the whole report, and that is the oracle's own
        // shape rather than a shortcut.** `Activity::display` sends each
        // traceback echo and each `Error ...` line through
        // `displayUsingTraceOutput`, which sanitises the line it is handed;
        // the rule is per byte and leaves `\n` alone, so sanitising the
        // concatenation is the same bytes as sanitising each line. It
        // therefore covers the clause echoes too, which carry the program's
        // own source and can hold any byte -- measured, a raw `0x01` inside a
        // source literal echoes as `?`.
        displayable(&mut out);
        out
    }

    /// One catalogue entry with this error's substitutions applied.
    ///
    /// The text comes from `rexx-inventory`'s generated table, derived from
    /// `interpreter/messages/rexxmsg.xml`, never hand-transcribed here: 704
    /// messages the tree already generates, and criterion 1 compares these
    /// bytes exactly.
    ///
    /// A miss renders visibly rather than panicking or rendering empty. The
    /// catalogue and the oracle come from one source, so a miss is a bug in
    /// this crate's numbering, and the error path is the worst possible place
    /// to abort: it would turn a reportable condition into a crash, which is
    /// the outcome the whole failing-loudly rule exists to prevent.
    fn message(&self, major: u16, sub: u16) -> Vec<u8> {
        match rexx_inventory::errors::lookup(major, sub) {
            Some(entry) => substitute(entry.text, &self.additional),
            None => format!("<no message {major}.{sub} in the catalogue>").into_bytes(),
        }
    }
}

/// Replaces `&1`, `&2`, ... with the raiser's substitution values.
///
/// The catalogue spells substitutions the way `rexxmsg.xml` does, so this is
/// the one piece of message rendering that is ours rather than generated.
/// Scans rather than chaining `replace`, so a substitution value that itself
/// contains `&2` cannot be re-substituted -- a real risk here, since these
/// values are arbitrary Rexx data (`say '&1' + 1` puts `&1` in the message).
///
/// An `&` not followed by a digit, and a digit with no matching value, are
/// both passed through unchanged rather than swallowed.
///
/// Bytes out, not text: see [`Substitution`]. The catalogue's own template is
/// `&str` because `rexxmsg.xml` is, and only the values can be arbitrary.
fn substitute(text: &str, values: &[Substitution]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut bytes = text.as_bytes().iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        if byte != b'&' {
            out.push(byte);
            continue;
        }
        match bytes.peek().copied() {
            Some(digit @ b'1'..=b'9') => {
                bytes.next();
                match values.get(usize::from(digit - b'1')) {
                    Some(value) => out.extend_from_slice(value),
                    None => out.extend_from_slice(&[b'&', digit]),
                }
            }
            _ => out.push(b'&'),
        }
    }
    out
}

/// The oracle's own rule for putting arbitrary Rexx bytes on a report line.
///
/// **A byte below `0x20` other than tab, carriage return and line feed
/// becomes `?`; every other byte, including every byte at or above `0x80`,
/// is written through unchanged.** That is `RexxString::stringTrace`
/// (`classes/StringClass.cpp`), which the oracle applies to *whole output
/// lines* rather than to the values inside them: `Activity::display` sends
/// every traceback echo, the major line and the secondary line through
/// `displayUsingTraceOutput` -> `processTraceInfo`, whose first act is
/// `traceLine->stringTrace()`.
///
/// Measured independently of the source, by driving all 256 byte values
/// through a builtin that reports the offending argument
/// (`say copies('ab','NN'x)` for the ones that are not valid counts, and
/// `say left('ab',5,'NNNN'x)` for the digits, which are):
///
/// ```text
/// rendered as ?  :  00-08  0b-0c  0e-1f
/// rendered raw   :  09-0a  0d     20-ff
/// ```
///
/// The echo obeys the same rule, measured with a raw `0x01` inside a source
/// literal: the oracle echoes `say copies('a?b','x')`.
///
/// # The two callers, and the one C++ function they both correspond to
///
/// `processTraceInfo` is the single sink, and the oracle reaches it two ways.
/// This crate has one application per way, so the pairing can be checked
/// rather than taken on trust:
///
/// | this crate | oracle |
/// |---|---|
/// | [`Raised::report`] | `Activity::display` (`concurrency/Activity.cpp:1414`) -> `RexxActivation::displayUsingTraceOutput` (`execution/RexxActivation.cpp:5262`) -> `processTraceInfo` |
/// | `trace.rs`'s line formatters | `RexxActivation::processTraceInfo` (`execution/RexxActivation.cpp:5249`) directly, for every live `TRACE` line |
///
/// Applying it twice to the same bytes is harmless and happens on a report's
/// clause echoes, which `push_clause` has already sanitised: `?` is `0x3f`,
/// above the threshold, so a second pass is the identity.
///
/// [`Raised::report`]: Raised::report
pub(crate) fn displayable(bytes: &mut [u8]) {
    for byte in bytes {
        if *byte < 0x20 && !matches!(*byte, b'\t' | b'\n' | b'\r') {
            *byte = b'?';
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One `FailureSite`, for the tests that predate the stack.
    ///
    /// Every assertion below that used to build a `ClauseSite { line, text,
    /// indent }` directly now builds a one-entry stack through this, and the
    /// **expected bytes in those tests are unchanged**: that is the check
    /// that the stack's one-element case is byte-identical to what 4a
    /// shipped, and it is worth more as an untouched expectation than as a
    /// new test asserting the same thing.
    fn one(line: usize, text: &[u8], indent: usize) -> Vec<FailureSite> {
        vec![FailureSite {
            line,
            text: text.to_vec(),
            indent,
        }]
    }

    /// The 7.3 transcript, captured from `build/bin/rexx` with `cat -A` so the
    /// trailing bytes are the oracle's and not a guess.
    ///
    /// Program: `say 1` / `select` / `when 1=0 then nop` / `end`. Stdout gets
    /// `1`; all three lines below go to stderr; rc is 249.
    #[test]
    fn the_7_3_report_matches_the_oracle_byte_for_byte() {
        let raised = Raised::syntax(7, 3, vec![]);
        let sites = one(4, b"end", 0);
        let site = ClauseSite {
            path: "/abs/path/f.rex",
            sites: &sites,
        };
        assert_eq!(
            String::from_utf8(raised.report(&site)).unwrap(),
            "     4 *-* end\n\
             Error 7 running /abs/path/f.rex line 4:  WHEN or OTHERWISE expected.\n\
             Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.\n"
        );
        assert_eq!(raised.exit_code(), 249);
    }

    /// A substituted message, and a clause echo that keeps its trailing space.
    ///
    /// Captured: `if 'x' then nop` on line 12 of a twelve-line program echoes
    /// `    12 *-* if 'x' ` -- the span stops at the start of `then`, so the
    /// space before it belongs to the clause. Trimming it would diverge on
    /// every `IF`.
    #[test]
    fn a_substituted_message_and_a_clause_echo_that_keeps_its_trailing_space() {
        let raised = Raised::not_logical(b"x");
        let sites = one(12, b"if 'x' ", 0);
        let site = ClauseSite {
            path: "/abs/w.rex",
            sites: &sites,
        };
        let report = String::from_utf8(raised.report(&site)).unwrap();
        assert_eq!(
            report,
            "    12 *-* if 'x' \n\
             Error 34 running /abs/w.rex line 12:  Logical value not 0 or 1.\n\
             Error 34.901:  Logical value must be exactly \"0\" or \"1\"; found \"x\".\n"
        );
        assert_eq!(raised.exit_code(), 222);
    }

    /// The line number is right-aligned in a six-character field, measured at
    /// one, two and three digits against the oracle: `     4`, `    12`,
    /// `   105`.
    #[test]
    fn the_line_number_field_is_six_wide() {
        for (line, expected) in [(4usize, "     4"), (12, "    12"), (105, "   105")] {
            let sites = one(line, b"nop", 0);
            let site = ClauseSite {
                path: "/p",
                sites: &sites,
            };
            let report = Raised::syntax(7, 3, vec![]).report(&site);
            let first = String::from_utf8(report).unwrap();
            let first = first.lines().next().unwrap().to_string();
            assert_eq!(&first[..6], expected, "line {line}");
        }
    }

    /// `256 - major`, over every major 4a is measured to raise.
    ///
    /// Nine, not the four the plan recorded, each confirmed by running the
    /// construct under the oracle and reading `$?`.
    #[test]
    fn the_exit_code_is_256_minus_the_major() {
        for (major, sub, rc) in [
            (7u16, 3u16, 249i32),
            (11, 1, 245),
            (24, 901, 232),
            (25, 11, 231),
            (26, 5, 230),
            (28, 3, 228),
            (33, 1, 223),
            (34, 1, 222),
            (41, 1, 215),
            (42, 3, 214),
            (98, 913, 158),
        ] {
            assert_eq!(
                Raised::syntax(major, sub, vec![]).exit_code(),
                rc,
                "{major}"
            );
        }
    }

    /// Every raiser family 4a is measured to produce has catalogue text for
    /// both its lines.
    ///
    /// This is the test that would have caught a hand-transcribed catalogue
    /// going stale, and it is why the text is looked up rather than written
    /// here: it asserts the entries *exist* and are non-empty, never what they
    /// say, so it cannot drift from `rexxmsg.xml` the way a copy would.
    #[test]
    fn every_measured_family_has_catalogue_text() {
        for (major, sub) in [
            (7u16, 3u16),
            (11, 1),
            (24, 1),
            (24, 901),
            (25, 11),
            (26, 2),
            (26, 3),
            (26, 5),
            (26, 6),
            (26, 8),
            (33, 1),
            (34, 1),
            (34, 2),
            (34, 3),
            (34, 4),
            (28, 1),
            (28, 2),
            (28, 3),
            (28, 4),
            (28, 5),
            (34, 6),
            (34, 901),
            (41, 1),
            (42, 3),
            (42, 901),
            (98, 913),
        ] {
            for (m, s) in [(major, 0), (major, sub)] {
                let entry = rexx_inventory::errors::lookup(m, s)
                    .unwrap_or_else(|| panic!("no catalogue entry for {m}.{s}"));
                assert!(!entry.text.is_empty(), "{m}.{s} has empty text");
            }
        }
    }

    /// A substitution value containing `&1` is not re-substituted.
    ///
    /// Reachable from a Rexx program: `say '&1' + 1` raises 41.1 with the
    /// operand text `&1`, so a `replace`-chaining implementation would expand
    /// the value into itself. Scanning once is what makes that impossible.
    #[test]
    fn a_substitution_value_containing_an_ampersand_digit_is_left_alone() {
        let raised = Raised::nonnumeric(b"&1");
        assert_eq!(
            raised.message(41, 1),
            b"Nonnumeric value (\"&1\") used in arithmetic operation."
        );
    }

    /// An `&` that is not a substitution, and a missing value, both pass
    /// through rather than being swallowed.
    #[test]
    fn a_bare_ampersand_and_a_missing_value_pass_through() {
        assert_eq!(substitute("a & b", &[]), b"a & b");
        assert_eq!(substitute("x &1 y", &[]), b"x &1 y");
        assert_eq!(substitute("&1 and &2", &[b"one".to_vec()]), b"one and &2");
    }

    /// A catalogue miss renders visibly instead of panicking or rendering
    /// empty: the error path is the worst place to abort, since it would turn
    /// a reportable condition into a crash.
    #[test]
    fn a_catalogue_miss_is_visible_rather_than_silent() {
        let raised = Raised::syntax(999, 999, vec![]);
        assert_eq!(
            raised.message(999, 999),
            b"<no message 999.999 in the catalogue>"
        );
    }

    /// Task 11's own addition: `site.indent` prefixes the clause echo with
    /// that many spaces, and nothing else on the report moves.
    ///
    /// Captured against the oracle: `do i = 1 to 3 / say 1/0 / end`
    /// reports `     2 *-*   say 1/0` -- two spaces for the one enclosing
    /// `DO`. Kills a mutation that applies the indent to the wrong line (the
    /// `Error 42 running ...` line, say), one that appends it after `text`
    /// instead of before, and one that never applies it at all (which the
    /// pre-existing `indent: 0` tests above would not catch, since they are
    /// silent about anything `indent` does when it is nonzero).
    #[test]
    fn the_indent_field_prefixes_the_clause_echo_with_that_many_spaces() {
        let raised = Raised::syntax(42, 3, vec![]);
        let sites = one(2, b"say 1/0", 2);
        let site = ClauseSite {
            path: "/abs/do1.rex",
            sites: &sites,
        };
        let report = String::from_utf8(raised.report(&site)).unwrap();
        assert_eq!(
            report.lines().next().unwrap(),
            "     2 *-*   say 1/0",
            "two spaces before the clause text, none anywhere else on the line"
        );
    }

    /// 4b Task 2: one echo line per entry, innermost first, each carrying its
    /// own line and its own absolute indent -- and the major line naming the
    /// **innermost** entry's line, not the outermost.
    ///
    /// The expected bytes are the oracle's, from a program whose two levels
    /// disagree on both quantities at once, which is what makes the assertion
    /// able to fail. Captured (4b Task 2's report, `c2.rex`): a `CALL` two
    /// `DO`s deep, at printed indent 4 on line 3, into a flat routine whose
    /// `say 1/0` is on line 8 and prints at indent 6.
    ///
    /// A one-entry implementation fails this (one echo instead of two); an
    /// outermost-first walk fails it (the two echoes swap); reading the line
    /// from the *last* entry fails it (`line 3` instead of `line 8`); and
    /// deriving either entry's indent from its position in the stack fails it
    /// (nothing about `[6, 4]` follows from `[inner, outer]`).
    #[test]
    fn the_report_echoes_one_line_per_level_innermost_first() {
        let raised = Raised::syntax(42, 3, vec![]);
        let sites = vec![
            FailureSite {
                line: 8,
                text: b"say 1/0".to_vec(),
                indent: 6,
            },
            FailureSite {
                line: 3,
                text: b"call sub1".to_vec(),
                indent: 4,
            },
        ];
        let site = ClauseSite {
            path: "/abs/c2.rex",
            sites: &sites,
        };
        assert_eq!(
            String::from_utf8(raised.report(&site)).unwrap(),
            concat!(
                "     8 *-*       say 1/0\n",
                "     3 *-*     call sub1\n",
                "Error 42 running /abs/c2.rex line 8:  Arithmetic overflow/underflow.\n",
                "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
            )
        );
    }

    /// The clause echo saturates at 40 columns, and the two error lines do
    /// not move when it does.
    ///
    /// Measured against the oracle with nested `DO`s and no call at all: 18
    /// levels print 36, 19 print 38, 20 print 40, and 21, 25 and 30 all print
    /// 40. The 19/20/21 rows are the ones that pin the boundary; 25 is there
    /// because a clamp written as `if indent == 42` would pass 21 and fail
    /// it. `trace.rs`'s own `MAX_CLAUSE_INDENT` doc has the value-line half
    /// of the measurement, which is what keeps the clamp out of
    /// `static_indent`.
    #[test]
    fn the_clause_echo_saturates_at_forty_columns() {
        for (indent, expected) in [(36usize, 36usize), (38, 38), (40, 40), (42, 40), (50, 40)] {
            let sites = one(9, b"say 1/0", indent);
            let site = ClauseSite {
                path: "/p",
                sites: &sites,
            };
            let report = String::from_utf8(Raised::syntax(42, 3, vec![]).report(&site)).unwrap();
            let echo = report.lines().next().unwrap();
            let after_field = &echo[11..];
            assert_eq!(
                after_field.len() - after_field.trim_start().len(),
                expected,
                "indent {indent}"
            );
            assert!(
                report.contains("Error 42 running /p line 9:  "),
                "the clamp moved something other than the echo's indent: {report:?}"
            );
        }
    }

    /// A `ParseError` becomes the SYNTAX condition the oracle raises for it,
    /// with the parser's own major and sub and the matching `256 - major`
    /// exit code.
    ///
    /// The pair is the oracle's, measured through `INTERPRET` rather than
    /// invented here: `interpret "do forever then"` is 27.901 at rc 229 and
    /// `interpret "if"` is 35.929 at rc 221. Asserting the exit code as well
    /// as the numbers is what makes this fail for an implementation that
    /// kept the loud path's `NOT_IMPLEMENTED_EXIT`.
    ///
    /// `condition` is deliberately not asserted: it is still `expect(dead_
    /// code)` until 4b's `SIGNAL ON` reads it for real, and a test reading it
    /// would fulfil that expectation in `cfg(test)` builds only, turning the
    /// annotation into a warning under `--all-targets` without giving the
    /// field the genuine reader its own doc comment is waiting for.
    #[test]
    fn a_parse_error_becomes_the_condition_the_oracle_raises() {
        for (code, sub, rc) in [(27u16, 901u16, 229i32), (35, 929, 221)] {
            let raised: Raised = (&ParseError::new(code, sub, 0)).into();
            assert_eq!((raised.number, raised.sub), (code, sub));
            assert_eq!(raised.exit_code(), rc);
        }
    }

    // The new Task 11 raisers themselves -- 26.2/26.3/28.1-28.5/34.3/34.4 --
    // live in `run.rs` as local `fn raised_*` free functions, matching that
    // file's own established convention for every other instruction-
    // specific raiser (`raised_if_not_logical`, `raised_select_no_when`,
    // `raised_symbol_expected`, ...), not as `Raised::` methods here: this
    // module holds only the raisers `eval.rs` also needs (cross-module), and
    // `insufficient_stack` is the one member of this task's own set that
    // qualifies. Their wording is exercised end to end by `run.rs`'s own
    // tests (`run_source` against a real program, checking `raised.number`/
    // `.sub`/`.additional`), not spot-checked again here -- `Raised::message`
    // is private to this module and `every_measured_family_has_catalogue_text`
    // above already proves every one of their catalogue entries exists.
}
