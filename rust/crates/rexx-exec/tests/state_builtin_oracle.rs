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

//! A differential sweep over `builtin/state.rs`'s eleven names: every case
//! runs through both interpreters and all three descriptors must agree,
//! except for the declared gaps in [`DECLARED_GAPS`].
//!
//! # Why this exists as a file rather than as a paragraph in a report
//!
//! 4c Task 10 ran a sweep of this shape by hand, reported "60 cases, 3
//! differ", and shipped a wrong answer in three error messages that were
//! inside the swept surface. The sweep was not wrong about the sixty
//! programs it ran; it was **blind in one direction and its own published
//! alphabet hid that**, which is the failure this file is built to make
//! impossible to repeat. So it is committed, its alphabet is the table
//! itself rather than a prose summary of it, and the set that differs is
//! asserted in both directions.
//!
//! # The axis crossing, stated because it is what the hand sweep missed
//!
//! The alphabet a case list needs here is **two-dimensional**, and holding
//! either axis at its safe value hides everything at the intersection:
//!
//! * *what an argument looks like* -- an integer literal, a value with an
//!   exponent, one with surrounding blanks, one with a redundant sign, one
//!   with a fractional part, a `NUMERIC`-produced value, a byte `>= 0x80`, a
//!   control byte, the null string;
//! * *what the interpreter's own settings are when it is read* -- the
//!   default `NUMERIC DIGITS`, and one small enough that a value's captured
//!   rendering (D15) stops being its digits.
//!
//! The hand sweep varied the first, held the second at its default, and
//! published an alphabet naming `1.0`, `1.5` and `99.0` -- which reads as
//! coverage of the numeric axis and is not, because none of those values
//! reaches a message that substitutes anything. Every case below whose name
//! ends `_crossed` is at the intersection, and each one was a live
//! divergence when this file was written.
//!
//! # Both directions, and why the gap list is a set rather than a count
//!
//! A case that starts differing is red because it is not in
//! [`DECLARED_GAPS`]. A declared gap that starts agreeing is red too,
//! because the list is compared as a set: closing one is a change to what
//! this crate claims, and it should show up in a diff rather than being
//! absorbed. A count would let one open while another closed.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use support::oracle::{Oracle, descriptor_diffs, did_not_finish};

/// One swept program: a name for the failure message, and the whole
/// single-clause-or-two source.
struct Case {
    name: &'static str,
    source: &'static str,
}

/// The cases whose three descriptors must **not** agree, each because this
/// crate refuses an answer it cannot build (`Loud::builtin_option_object`).
///
/// Named rather than counted: see the module doc.
const DECLARED_GAPS: &[&str] = &[
    "arg_option_array",
    "condition_additional",
    "condition_object",
];

const CASES: &[Case] = &[
    // ---- ADDRESS: the default, the forms, byte transparency ----
    Case {
        name: "address_default",
        source: "say '['address()']'",
    },
    Case {
        name: "address_set",
        source: "address zork; say address()",
    },
    Case {
        name: "address_blank_in_name",
        source: "address 'a b'; say '['address()']'",
    },
    Case {
        name: "address_toggle_first",
        source: "address ; say '['address()']'",
    },
    Case {
        name: "address_high_byte",
        source: "address 'ff80'x; say c2x(address())",
    },
    Case {
        name: "address_control_byte",
        source: "address '00'x; say c2x(address())",
    },
    Case {
        name: "address_default_bytes",
        source: "say c2x(address())",
    },
    Case {
        name: "address_arity",
        source: "say address(1)",
    },
    // ---- the NUMERIC trio ----
    Case {
        name: "numeric_trio",
        source: "say digits() fuzz() form()",
    },
    Case {
        name: "digits_after_set",
        source: "numeric digits 100; say digits()",
    },
    Case {
        name: "fuzz_after_set",
        source: "numeric fuzz 5; say fuzz()",
    },
    Case {
        name: "form_after_set",
        source: "numeric form engineering; say form()",
    },
    Case {
        name: "digits_arity",
        source: "say digits(1)",
    },
    Case {
        name: "fuzz_arity",
        source: "say fuzz('x')",
    },
    Case {
        name: "form_arity",
        source: "say form(1)",
    },
    // ---- QUEUED ----
    Case {
        name: "queued_empty",
        source: "say queued()",
    },
    Case {
        name: "queued_null_line",
        source: "queue ''; say queued() '['queued()']'",
    },
    Case {
        name: "queued_high_byte",
        source: "push '80'x; say queued(); pull v; say c2x(v)",
    },
    Case {
        name: "queued_control_byte",
        source: "queue '09'x; say queued(); pull v; say c2x(v)",
    },
    Case {
        name: "queued_arity",
        source: "say queued(1)",
    },
    // ---- GC: the argument is a spelling, not an option letter ----
    Case {
        name: "gc_spellings",
        source: "say gc() gc('F') gc('f') gc('force') gc('Force') gc('FORCE') gc('fx') gc('Fnord')",
    },
    Case {
        name: "gc_empty_option",
        source: "say gc('')",
    },
    Case {
        name: "gc_bad_option",
        source: "say gc('x')",
    },
    Case {
        name: "gc_high_byte_option",
        source: "say gc('80'x)",
    },
    Case {
        name: "gc_control_byte_option",
        source: "say gc('09'x)",
    },
    Case {
        name: "gc_numeric_option",
        source: "say gc(1)",
    },
    Case {
        name: "gc_arity",
        source: "say gc(1,2)",
    },
    // ---- ERRORTEXT ----
    Case {
        name: "errortext_range",
        source: "say errortext(0) '|' errortext(1) '|' errortext(40) '|' errortext(93) '|' errortext(99)",
    },
    Case {
        name: "errortext_no_entry",
        source: "say '['errortext(98)']'",
    },
    Case {
        name: "errortext_whole_float",
        source: "say errortext(99.0)",
    },
    Case {
        name: "errortext_not_whole",
        source: "say errortext(1.5)",
    },
    Case {
        name: "errortext_too_wide",
        source: "say errortext(99999999999999999999)",
    },
    Case {
        name: "errortext_high_byte",
        source: "say errortext('80'x)",
    },
    Case {
        name: "errortext_control_byte",
        source: "say errortext('00'x)",
    },
    Case {
        name: "errortext_missing",
        source: "say errortext()",
    },
    Case {
        name: "errortext_arity",
        source: "say errortext(1,2)",
    },
    // The range message's substitution. `_crossed` marks the intersection
    // the hand sweep held constant; every one of these was a live
    // divergence.
    Case {
        name: "errortext_exponent_crossed",
        source: "say errortext(1e2)",
    },
    Case {
        name: "errortext_blanks_crossed",
        source: "say errortext(' 100 ')",
    },
    Case {
        name: "errortext_trailing_zero_crossed",
        source: "say errortext(100.0)",
    },
    Case {
        name: "errortext_digits_crossed",
        source: "numeric digits 3; say errortext(999999+1)",
    },
    Case {
        name: "errortext_digits_restored_crossed",
        source: "numeric digits 3; z = 999999+1; numeric digits 9; say errortext(z)",
    },
    Case {
        name: "errortext_not_whole_digits_crossed",
        source: "numeric digits 3; z = 1/3; numeric digits 9; say errortext(z)",
    },
    // ---- SOURCELINE ----
    Case {
        name: "sourceline_count",
        source: "say sourceline()",
    },
    Case {
        name: "sourceline_first",
        source: "say '['sourceline(1)']'",
    },
    Case {
        name: "sourceline_not_whole",
        source: "say sourceline('80'x)",
    },
    Case {
        name: "sourceline_control_byte",
        source: "say sourceline('09'x)",
    },
    Case {
        name: "sourceline_arity",
        source: "say sourceline(1,2)",
    },
    Case {
        name: "sourceline_huge",
        source: "say sourceline(2000000000000)",
    },
    Case {
        name: "sourceline_zero_crossed",
        source: "say sourceline(0.0)",
    },
    Case {
        name: "sourceline_padded_zero_crossed",
        source: "say sourceline('0000')",
    },
    Case {
        name: "sourceline_exponent_crossed",
        source: "say sourceline(1e1)",
    },
    Case {
        name: "sourceline_digits_crossed",
        source: "numeric digits 3; say sourceline(999999+1)",
    },
    // ---- TRACE ----
    Case {
        name: "trace_initial",
        source: "say trace()",
    },
    Case {
        name: "trace_set_and_read",
        source: "say '['trace('a')']['trace('O')']'",
    },
    Case {
        name: "trace_labels",
        source: "say '['trace('L')']['trace('O')']'",
    },
    Case {
        name: "trace_bad_letter",
        source: "say trace('80'x)",
    },
    Case {
        name: "trace_control_byte",
        source: "say trace('00'x)",
    },
    Case {
        name: "trace_empty",
        source: "say '['trace('')']['trace('O')']'",
    },
    // A digit string is 24.1 to the builtin and 24.901 to the instruction.
    Case {
        name: "trace_digit_string",
        source: "say trace('5')",
    },
    Case {
        name: "trace_negative_digit_string",
        source: "say trace('-3')",
    },
    Case {
        name: "trace_numeric_instruction",
        source: "trace value 5",
    },
    // ---- ARG ----
    Case {
        name: "arg_none",
        source: "say arg() '['arg(1)']' arg(1,'E') arg(1,'O')",
    },
    Case {
        name: "arg_option_array",
        source: "say arg(1,'A')",
    },
    Case {
        name: "arg_high_byte_option",
        source: "say arg(1,'80'x)",
    },
    Case {
        name: "arg_control_byte_option",
        source: "say arg(1,'00'x)",
    },
    Case {
        name: "arg_arity",
        source: "say arg(1,2,3)",
    },
    // The three checks in order, plus the substitution crossing.
    Case {
        name: "arg_bad_type",
        source: "say arg('x','q')",
    },
    Case {
        name: "arg_bad_range_with_option",
        source: "say arg(0,'q')",
    },
    Case {
        name: "arg_bad_range_empty_option",
        source: "say arg(0,'')",
    },
    Case {
        name: "arg_empty_option",
        source: "say arg(1,'')",
    },
    Case {
        name: "arg_no_position_empty_option",
        source: "say arg(,'')",
    },
    Case {
        name: "arg_no_position_option",
        source: "say arg(,'x')",
    },
    Case {
        name: "arg_zero_float_crossed",
        source: "say arg(0.0)",
    },
    Case {
        name: "arg_signed_zero_crossed",
        source: "say arg('+0')",
    },
    Case {
        name: "arg_negative_float_crossed",
        source: "say arg(-1.0)",
    },
    Case {
        name: "arg_digits_crossed",
        source: "numeric digits 3; say arg(-(999999+1))",
    },
    // ---- CONDITION ----
    Case {
        name: "condition_outside",
        source: "say condition() '|' condition('c') '|' condition('S') '|' condition('E') '|' condition('D') '|' condition('R')",
    },
    Case {
        name: "condition_additional",
        source: "say condition('A')",
    },
    Case {
        name: "condition_object",
        source: "say condition('O')",
    },
    Case {
        name: "condition_bad_option",
        source: "say condition('80'x)",
    },
    Case {
        name: "condition_control_byte",
        source: "say condition('09'x)",
    },
    Case {
        name: "condition_empty_option",
        source: "say condition('')",
    },
    Case {
        name: "condition_numeric_option",
        source: "say condition(1)",
    },
    Case {
        name: "condition_arity",
        source: "say condition('c','x')",
    },
    // Inside a live handler, which is the only place the options answer
    // anything at all -- and the re-arm that separates the stored `I` from
    // the live `S`.
    Case {
        name: "condition_in_signal_handler",
        source: "signal on syntax name h\nsay 1/0\nexit\nh:\nsay '['condition()']['condition('C')']['condition('D')']['condition('E')']['condition('I')']['condition('S')']'\nsignal on syntax name h\nsay condition('I') condition('S')",
    },
    Case {
        name: "condition_in_call_handler",
        source: "call on user uc name uh\ncall r1\nsay '['condition()']['condition('C')']'\nexit\nuh:\nsay condition('I') condition('S') condition('C') '['condition('D')']'\nreturn\nr1:\nraise user uc description 'cd' return 1",
    },
    Case {
        name: "condition_reset_is_activation_local",
        source: "signal on syntax name h\nsay 1/0\nexit\nh:\ncall clr\nsay '['condition('C')']'\nexit\nclr:\nsay '['condition('C')']['condition('R')']['condition('C')']'\nreturn",
    },
];

/// A directory of this run's own, for the reason `Oracle::run`'s doc gives:
/// an unresolved name searches the working directory for an external
/// routine, so a swept program must not share a directory with stale ones.
fn fresh_run_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("state-builtin-{}-{nanos}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)
            .unwrap_or_else(|e| panic!("cannot clear {}: {e}", root.display()));
    }
    fs::create_dir_all(&root).unwrap_or_else(|e| panic!("cannot create {}: {e}", root.display()));
    root
}

/// Runs one case through both interpreters and reports which descriptors
/// disagree, with the transcript.
fn sweep_one(oracle: &Oracle, run_root: &Path, case: &Case) -> Option<String> {
    let dir = run_root.join(case.name);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    let file = dir.join("case.rex");
    fs::write(&file, format!("{}\n", case.source))
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));
    let path = abs
        .to_str()
        .unwrap_or_else(|| panic!("case path {} is not valid UTF-8", abs.display()));

    let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
    let rust = rexx_exec::run_program(path, text, rexx_exec::Invocation::none());
    let cpp = oracle.run(&abs);

    // An unconditional `assert!`, not a `Some(String)` return: this
    // function's caller matches its `Some` against `DECLARED_GAPS`
    // (`every_state_builtin_case_matches_the_oracle_except_the_declared_gaps`),
    // and a case already in that set would have a non-finish silently
    // absorbed as the declared gap it names, in every mode, rather than
    // reddening as the structural failure it actually is. Checked before
    // `descriptor_diffs`, whose own `expect_exit_code()` panic has no
    // `case.name` in it.
    assert!(
        !did_not_finish(&cpp),
        "{}: the oracle did not finish: {:?} -- a structural failure, not a byte \
         comparison, and never a declared gap",
        case.name,
        cpp.termination
    );

    let diffs = descriptor_diffs(&rust, &cpp);
    if diffs.is_empty() {
        return None;
    }
    Some(format!(
        "  {} [{}]\n    source: {:?}\n    rust:   stdout={:?} stderr={:?} exit={}\n    \
         oracle: stdout={:?} stderr={:?} exit={}",
        case.name,
        diffs.join(", "),
        case.source,
        String::from_utf8_lossy(&rust.stdout),
        String::from_utf8_lossy(&rust.stderr),
        rust.exit_code,
        String::from_utf8_lossy(&cpp.stdout),
        String::from_utf8_lossy(&cpp.stderr),
        cpp.expect_exit_code()
    ))
}

/// The set of cases that differ is exactly [`DECLARED_GAPS`].
#[test]
fn every_state_builtin_case_matches_the_oracle_except_the_declared_gaps() {
    let oracle = support::oracle::locate();
    let run_root = fresh_run_root();

    let mut differing = Vec::new();
    let mut transcripts = String::new();
    for case in CASES {
        if let Some(report) = sweep_one(&oracle, &run_root, case) {
            differing.push(case.name);
            transcripts.push_str(&report);
            transcripts.push('\n');
        }
    }

    let unexpected: Vec<&str> = differing
        .iter()
        .copied()
        .filter(|name| !DECLARED_GAPS.contains(name))
        .collect();
    let closed: Vec<&str> = DECLARED_GAPS
        .iter()
        .copied()
        .filter(|name| !differing.contains(name))
        .collect();

    assert!(
        unexpected.is_empty() && closed.is_empty(),
        "the state builtins' differential surface moved.\n\
         differ but are not declared gaps: {unexpected:?}\n\
         are declared gaps but now agree (remove them from DECLARED_GAPS): {closed:?}\n\
         transcripts:\n{transcripts}"
    );

    // Only on success: a failing run's case directories are the evidence.
    let _ = fs::remove_dir_all(&run_root);
    assert_eq!(
        oracle.invocations(),
        CASES.len(),
        "one oracle run per case, and every case run"
    );
}

/// Every declared gap names a real case, and every gap is loud rather than
/// merely different.
///
/// Without the first half a typo in [`DECLARED_GAPS`] would forgive nothing
/// and be invisible, since the set test above would then report it as
/// "closed" only if a case of that name existed. Without the second, a
/// silent wrong answer could be parked in the list beside the two honest
/// refusals.
#[test]
fn every_declared_gap_names_a_case_and_fails_loudly() {
    for name in DECLARED_GAPS {
        assert!(
            CASES.iter().any(|case| case.name == *name),
            "{name} is a declared gap with no case of that name"
        );
    }
    let run_root = fresh_run_root();
    for name in DECLARED_GAPS {
        let case = CASES
            .iter()
            .find(|case| case.name == *name)
            .expect("checked immediately above");
        let dir = run_root.join(case.name);
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
        let file = dir.join("case.rex");
        fs::write(&file, format!("{}\n", case.source))
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
        let text =
            fs::read(&file).unwrap_or_else(|e| panic!("cannot read {}: {e}", file.display()));
        let rust = rexx_exec::run_program(
            file.to_str().expect("a UTF-8 path"),
            text,
            rexx_exec::Invocation::none(),
        );
        assert_eq!(
            rust.exit_code,
            rexx_exec::NOT_IMPLEMENTED_EXIT,
            "{name} is a declared gap but does not fail loudly; stderr: {}",
            String::from_utf8_lossy(&rust.stderr)
        );
    }
    let _ = fs::remove_dir_all(&run_root);
}
