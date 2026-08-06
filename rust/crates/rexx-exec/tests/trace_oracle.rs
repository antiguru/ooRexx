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

//! Criterion 3 (D17): `TRACE` output matches the oracle byte for byte, over a
//! committed table mapping each reachable prefix to a witness program.
//!
//! **This is the opposite strategy from `tests/corpus.rs`, deliberately.**
//! `corpus.rs` shells out to the oracle *live*, every run, because it exists
//! to track progress across tasks and a committed expectation would need
//! regenerating on every task that changes behaviour. A trace witness is a
//! fixed artefact instead, because its whole value is that it cannot drift
//! silently, and capturing it needs a running oracle that a machine checking
//! out this tree may not have. Neither is wrong; they answer different
//! questions, and `rexx-parse/tests/sourceline_oracle.rs` is this design's
//! own precedent -- unrelated to `TRACE` but the same "committed expectation,
//! regenerate by a driver script" shape.
//!
//! **The prefix table**, all 19 from `RexxActivation.hpp:90`-`110`, and which
//! witness (if any) below reaches it. Which of them a witness reaches, and who
//! owns each of the rest, is [`PREFIX_COVERAGE`]'s -- asserted, not described
//! here. One correction to the design spec's own "measured reachable from
//! pure-4a code" list is worth keeping, because it was got wrong once: `>E>`
//! is not on that list and is nevertheless reachable
//! (`dotvariable_beyond_the_list.rex`).
//!
//! | prefix | witness |
//! |---|---|
//! | `*-*` | every witness below (asserted, not claimed -- see below) |
//! | `>>>` | every witness below **except `trace_labels.rex`**, whose whole
//!   output is three `*-*` lines (asserted) |
//! | `>=>` | `compound_read_write.rex` (and `trace_output.rex`'s simple form) |
//! | `>L>` | `trace_output.rex` |
//! | `>V>` | `trace_output.rex`, `compound_read_write.rex` |
//! | `>O>` | `trace_output.rex` |
//! | `>K>` | `keyword_while.rex` (`WHILE`), `pull_queue.rex` (`PULL`/`LINEIN`) |
//! | `>C>` | `compound_read_write.rex` |
//! | `>P>` | `prefix_operators.rex` |
//! | `>E>` (bonus, not required) | `dotvariable_beyond_the_list.rex` |
//! | `>A>` | `call_arguments.rex`, `function_call.rex` |
//! | `>F>` | `function_call.rex` |
//! | `>R>` | `use_arg_alias.rex` |
//! | `>.>` | `parse_placeholder.rex` |
//!
//! **Several witnesses below claim no prefix the table above does not
//! already cover, and are here for a *content* difference instead** --
//! something no prefix table can express. No count is given, deliberately:
//! the sentence that stood here carried one, and it was stale within a
//! commit of being written (re-review NEW-4). [`WITNESS_PREFIXES`] is the
//! list, and it is the one this file's own test reads. The kinds, which is
//! what a reader actually needs:
//!
//! * a value line that was **absent** -- `controlled_loop.rex` (a
//!   `Controlled` loop's re-tested pass) and `exit_value.rex`
//!   (`EXIT <expr>`'s own `>>>`);
//! * a value line that was present and **wrong** --
//!   `control_variable_reread.rex` (with the trip count that produced it);
//! * a whole mode that emitted **nothing** -- `trace_labels.rex`;
//! * a **condition** that was not raised -- `control_variable_novalue.rex`;
//! * a value line that carried the **wrong stage of a transform** --
//!   `pull_queue.rex`, where a bare `PULL`'s `>K>` is the line before the
//!   upcase and its `>>>` the line after, and where `ARG` alone among the
//!   sources emits no `>K>` at all.
//!
//! Every one of them is listed in [`WITNESS_PREFIXES`] for the prefixes it
//! does emit, which is what keeps it inside this file's own drift check.
//!
//! **This table used to be prose only, and a branch review (H3,
//! `branch-review-harness.md`) showed exactly what that cost**: replacing
//! `keyword_while.rex` with a straight-line program emitting no `>K>` at
//! all, regenerating its `.expected` from the live oracle with this file's
//! own documented recipe, still passed all five tests that existed then --
//! the byte-for-byte
//! check compares this crate's output to the committed file and nothing
//! else, so a witness that stopped witnessing its own prefix went
//! unnoticed. [`WITNESS_PREFIXES`] and
//! [`every_witness_still_emits_every_prefix_it_is_named_for`] turn the
//! table above into an assertion: each witness's committed `.expected`
//! stderr must contain every prefix this table claims for it, checked as a
//! byte substring, and the union across every witness must be exactly
//! [`CLAIMED_PREFIXES`] -- named rather than counted, because a count here is
//! falsified by the next witness that lands and the name never is. A witness
//! can still be swapped for a better one, but not for one that silently
//! covers less.
//!
//! **NOTHING IN THIS FILE PINS A TRACE LINE'S INDENT, and no witness here
//! can** (review round 1, F1). `check_witness` compares both sides through
//! `support::normalize_stderr` (DEVIATION 0), which collapses exactly the
//! space run between a trace line's marker and its content -- so a value
//! line emitted at the wrong indent compares equal to one emitted at the
//! right one. Measured rather than reasoned: replacing the `do_indent`/
//! `loop_indent` split in `run.rs`'s own `LoopState::Controlled` arm with
//! `loop_indent` alone leaves this file green while diverging from the
//! oracle byte for byte, and the same holds for `>F>` emitted two columns
//! in. **This file green, not the workspace**: the paragraph below names
//! the unit test that catches both, and it is in the same workspace, so a
//! sentence claiming the workspace stays green would contradict it twelve
//! lines later. (It did, with a gate number that was stale as well --
//! re-review NEW-3.)
//!
//! What *does* hold trace indents to the oracle: the `run.rs`/`lib.rs`/
//! `error.rs` unit tests that assert an exact stderr string, which is
//! outside either harness's comparison function, plus raw A/B probes in each
//! task's own report. For Task 9's own two indents specifically that is
//! `run.rs`'s `task_9s_two_new_indents_are_the_oracles_own_and_
//! normalisation_cannot_see_them`, which asserts both transcripts byte for
//! byte and goes red under exactly the mutation that leaves this file green.
//! **A comment in this file must not claim otherwise**; two did, and this
//! paragraph replaces them.
//!
//! **The gap this file used to disclose here is closed, and
//! `controlled_loop.rex` is the witness** (Task 9). A `Controlled`
//! (`TO`-style) `DO`/`LOOP`'s own re-tested pass traces two more `>>>` lines
//! (the control variable's pre- and post-increment value, `DoBlock::
//! checkControl`, `DoBlock.cpp:182`-`205`) plus, under `TRACE I`, the `>V>`
//! that reads it and the `>=>` that writes it back. That witness covers all
//! four, in both modes, across an `ITERATE` and a negative `BY`, and it
//! carries a `DO OVER` beside them as the neighbouring *passing* case --
//! which pins that a `DO OVER` traces its own `>=>` at all, not the indent
//! it traces it at. `keyword_while.rex` remains the `>K>` witness on its own
//! merits (a real repeating construct, re-echoing its clause every pass,
//! `>K>` re-firing every pass), not because it once dodged this gap.
//!
//! **Regeneration.** Every `.expected` file was captured with:
//!
//! ```bash
//! ( ulimit -v 1048576; \
//!   LD_LIBRARY_PATH=/path/to/ooRexx/build/lib \
//!   /path/to/ooRexx/build/bin/rexx PROGRAM.rex </dev/null ) \
//!   1>/tmp/out 2>/tmp/err; rc=$?
//! { echo "RC $rc"; echo "===STDOUT==="; cat /tmp/out; \
//!   echo "===STDERR==="; cat /tmp/err; } > PROGRAM.expected
//! ```
//!
//! The `</dev/null` is not decoration: `pull_queue.rex` reads the console, and
//! without it the capture blocks on a terminal and consumes whatever a pipe
//! happens to hold. `run_program`'s own default (`Invocation::none`, hence
//! `ProgramInput::Nothing`) is the empty console, so a capture taken with any
//! other input would be an expectation this crate can never reproduce.
//!
//! `trace_output.rex` and `pull_queue.rex` live in `rust/corpus/lang/` (both
//! corpus subset members) rather than being duplicated here
//! -- this test reads them from there by relative path, and only their own
//! `.expected` files live in this directory.
//!
//! **DEVIATION 0**: `check_witness`'s own stderr comparison runs both sides
//! through `support::normalize_stderr` (`tests/support/mod.rs`, shared with
//! `tests/corpus.rs`) before comparing -- collapsing only the run of spaces
//! between a trace line's own marker and its content, never anything else.
//! This applies to the *comparison* alone. **Regeneration stays exact**:
//! the recipe above captures the oracle's own raw bytes with no
//! normalisation applied, so a committed `.expected` file remains real
//! oracle output, indent counter defect included, for anyone regenerating
//! one later.

mod support;

use rexx_exec::run_program;
use std::path::Path;

/// One witness's fixed oracle answer: exit code, stdout, stderr.
struct Expected {
    rc: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Parses an `.expected` file (`RC n` / `===STDOUT===` / bytes /
/// `===STDERR===` / bytes) -- a from-scratch format, not `sourceline_
/// oracle.rs`'s `count N` shape, because a witness needs three fields
/// (exit code, stdout, stderr) where a source-line expectation needs only
/// a count and lines. The two marker lines are chosen never to collide with
/// anything a trace prefix or Rexx `SAY` output could produce.
fn parse_expected(bytes: &[u8], path: &str) -> Expected {
    let text = std::str::from_utf8(bytes)
        .unwrap_or_else(|_| panic!("{path}: expectation file is not valid UTF-8"));
    let after_rc = text
        .strip_prefix("RC ")
        .unwrap_or_else(|| panic!("{path}: expectation file does not start with `RC `"));
    let (rc_text, rest) = after_rc
        .split_once('\n')
        .unwrap_or_else(|| panic!("{path}: no newline after the RC line"));
    let rc: i32 = rc_text
        .parse()
        .unwrap_or_else(|_| panic!("{path}: `{rc_text}` is not an exit code"));
    let rest = rest
        .strip_prefix("===STDOUT===\n")
        .unwrap_or_else(|| panic!("{path}: missing `===STDOUT===` marker"));
    let (stdout, rest) = rest
        .split_once("===STDERR===\n")
        .unwrap_or_else(|| panic!("{path}: missing `===STDERR===` marker"));
    Expected {
        rc,
        stdout: stdout.as_bytes().to_vec(),
        stderr: rest.as_bytes().to_vec(),
    }
}

/// Runs `program_path` through this crate's own public entry point and
/// asserts it against `<oracle_dir>/<name>.expected` on all three of
/// stdout, stderr and exit code -- never a substring or a loose bound,
/// matching criterion 2's own "byte for byte, never numerically" rule for
/// the same reason: a numeric or partial comparison here would hide
/// exactly the quoting and value-content divergences this task exists to
/// catch. stdout and exit code are byte-exact; stderr is byte-exact up to
/// DEVIATION 0's own narrow indent normalisation (this file's module doc
/// has the scope).
fn check_witness(name: &str, program_path: &Path) {
    let oracle_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle");
    let expected_path = oracle_dir.join(format!("{name}.expected"));
    let expected_bytes = std::fs::read(&expected_path).unwrap_or_else(|e| {
        panic!(
            "{}: no committed expectation for witness {name} ({e}); \
             regenerate per this file's own module comment",
            expected_path.display()
        )
    });
    let expected = parse_expected(&expected_bytes, name);

    let source = std::fs::read(program_path)
        .unwrap_or_else(|e| panic!("{}: unreadable ({e})", program_path.display()));
    let outcome = run_program(
        &program_path.to_string_lossy(),
        source,
        rexx_exec::Invocation::none(),
    );

    assert_eq!(outcome.stdout, expected.stdout, "{name}: stdout");
    // DEVIATION 0: normalised on this comparison only, never on the
    // committed `.expected` bytes themselves -- see this file's own module
    // doc.
    assert_eq!(
        support::normalize_stderr(&outcome.stderr),
        support::normalize_stderr(&expected.stderr),
        "{name}: stderr"
    );
    assert_eq!(outcome.exit_code, expected.rc, "{name}: exit code");
}

/// `>L>`/`>V>`/`>O>`/`>>>`/`>=>`/`*-*`: `TRACE I` over two assignments and an
/// `IF`/`THEN`/`SAY`, read from `rust/corpus/` rather than duplicated here.
#[test]
fn trace_output_covers_clause_result_assignment_literal_variable_and_operator() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang/trace_output.rex");
    check_witness("trace_output", &path);
}

/// `>K>`: a `DO WHILE` loop, re-echoing its own clause and `END` every pass
/// and re-firing `>K> "WHILE"` every pass too -- a complete answer for a
/// real repeating construct, not a single-shot stand-in.
#[test]
fn keyword_while_covers_a_re_evaluated_keyword_across_every_pass() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/keyword_while.rex");
    check_witness("keyword_while", &path);
}

/// `>C>`: a compound variable's own resolved name, announced before `>V>`
/// on a read and before `>=>` on a write, both under `TRACE I`.
#[test]
fn compound_read_write_covers_the_resolved_compound_name() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/compound_read_write.rex");
    check_witness("compound_read_write", &path);
}

/// `>P>`: the prefix operators this witness covers, `+` and `\`.
#[test]
fn prefix_operators_covers_plus_and_backslash() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/prefix_operators.rex");
    check_witness("prefix_operators", &path);
}

/// `>E>`, a bonus beyond the design spec's own nine-prefix "measured
/// reachable from pure-4a code" list -- a correction worth keeping because it
/// was got wrong once (`.nil` is one of `ExprKind::DotVariable`'s own three
/// admissible names, D15, so it is reachable), not required by criterion 3's
/// own table but kept because it is real and cheap to pin.
#[test]
fn dotvariable_beyond_the_list_covers_the_spec_correction() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/trace_oracle/dotvariable_beyond_the_list.rex");
    check_witness("dotvariable_beyond_the_list", &path);
}

/// `>A>` and the `USE ARG` value lines: a `CALL` whose four argument
/// positions are an expression, an omission, a literal and a variable
/// reference, into a `USE ARG` that binds three of them.
#[test]
fn call_arguments_covers_the_argument_prefix_at_every_position_shape() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/call_arguments.rex");
    check_witness("call_arguments", &path);
}

/// `>F>`: two expression-form calls in one clause, which pins that the
/// second one is emitted **at all** after the first callee's own lines --
/// not the column it lands in. An earlier version of this comment said the
/// second line "has to come back to the caller's indent"; nothing here makes
/// it, because DEVIATION 0 normalises that difference away (review round 1,
/// F1, and the module doc's own paragraph on what does pin an indent).
#[test]
fn function_call_covers_the_function_prefix_after_the_callees_own_lines() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/function_call.rex");
    check_witness("function_call", &path);
}

/// `>R>`, deliberately under `TRACE R` rather than `TRACE I`: it is a
/// RESULTS-level prefix, so a witness that ran under `I` would pass just as
/// well with the gate written wrongly as `intermediates`. Nothing else in
/// this file's transcript is an intermediates line, which is what makes the
/// distinction visible.
#[test]
fn use_arg_alias_covers_the_alias_prefix_and_its_results_level_gate() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/use_arg_alias.rex");
    check_witness("use_arg_alias", &path);
}

/// A `Controlled` loop's own control-variable value lines, in both modes --
/// see the module doc's own paragraph on the gap this closes and on the
/// `DO OVER` case beside it, which is here as the adjacent passing shape
/// rather than as extra coverage.
#[test]
fn controlled_loop_covers_the_control_variables_own_value_lines() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/controlled_loop.rex");
    check_witness("controlled_loop", &path);
}

/// The re-test is an **evaluation**, so it raises `NOVALUE` when the body
/// has dropped the control variable -- the hole the first version of the
/// re-read left, which used a `NOVALUE`-blind reader. The program's own
/// header has the pair and says what the first loop is doing there.
#[test]
fn control_variable_novalue_covers_a_dropped_control_variable_under_a_trap() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/trace_oracle/control_variable_novalue.rex");
    check_witness("control_variable_novalue", &path);
}

/// `TRACE L`: every executed `LABEL` clause echoes and nothing else does.
/// The program's own header says which routes to a label it probes, why
/// they are examples rather than an enumeration, and why the silent
/// constructs between them are the other half of the claim.
#[test]
fn trace_labels_covers_the_labels_only_mode() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/trace_labels.rex");
    check_witness("trace_labels", &path);
}

/// The control variable is **re-read** on every re-tested pass, not taken
/// from the loop's own saved value (review round 1, F2). The program's own
/// header has the measured pair; the visible difference is a trip count and
/// a `>V>` value, neither of which normalisation touches.
#[test]
fn control_variable_reread_covers_a_body_that_writes_the_control_variable() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/trace_oracle/control_variable_reread.rex");
    check_witness("control_variable_reread", &path);
}

/// `EXIT <expr>`'s own `>>>`, which this crate did not emit at all before
/// Task 9 -- see the program's own header for why its adjacent passing case
/// (a bare `EXIT`, which traces nothing) lives in three other witnesses
/// rather than here.
#[test]
fn exit_value_covers_the_exit_instructions_own_result_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/exit_value.rex");
    check_witness("exit_value", &path);
}

/// `>.>`, the `PARSE` template placeholder's own line, **under both `TRACE I`
/// and `TRACE R` in one program**. Two modes rather than one because the fact
/// worth pinning is a *choice* of prefix: an assigned target traces `>=>`
/// under `I` and `>>>` under `R`, never both, and the placeholder's `>.>`
/// appears under `I` only. Either mode alone passes against an engine with two
/// independent gates. The program's own header has the C++ site and says why
/// it contains no `parse source`.
#[test]
fn parse_placeholder_covers_the_dummy_prefix_and_the_two_modes_disagreement() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/parse_placeholder.rex");
    check_witness("parse_placeholder", &path);
}

/// The two `PARSE` sources that read a line, in **both** modes, and the one
/// fact about them that only a trace can show: for a bare `PULL`, the `>K>`
/// line carries the value **before** the upcase while the `>>>` line after it
/// carries the value after, so the two lines disagree on a single instruction.
///
/// Three claims, all held by `check_witness`'s byte-exact comparison rather
/// than by the substring table below:
///
/// * `>K>` is emitted for `PULL` and for `LINEIN` under `TRACE R` **and**
///   under `TRACE I`, so it is a `results` prefix. A witness in one mode alone
///   passes against an engine that gated it on the other.
/// * `ARG` and `PARSE ARG` emit **no** `>K>` in either mode -- the only
///   sources that do not. An engine that emitted one adds a line the committed
///   bytes do not have.
/// * the pre-upcase split above, which an engine that traced the transformed
///   value in `>K>` gets wrong while still printing the right answer on
///   stdout.
///
/// Read from `rust/corpus/lang/` rather than duplicated here, the same way
/// `trace_output.rex` is: that program is a `phase-4c.txt` member and will run
/// live against the oracle once Task 15 wires that file into `tests/corpus.rs`,
/// where today only `coverage.rs` parses it. Committing its expectation here is
/// what makes its trace half checked *now*, offline, in the ungated suite.
#[test]
fn pull_queue_covers_the_line_reading_sources_in_both_modes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang/pull_queue.rex");
    check_witness("pull_queue", &path);
}

/// The module doc's own table, as data: which prefixes each witness is
/// claimed to cover. See the module doc's own note on why this exists --
/// found missing by a branch review (H3), which swapped `keyword_while.rex`
/// for a program with no `>K>` at all and watched every test stay green.
const WITNESS_PREFIXES: &[(&str, &[&str])] = &[
    ("trace_output", &["*-*", ">>>", ">=>", ">L>", ">V>", ">O>"]),
    ("keyword_while", &["*-*", ">>>", ">K>"]),
    (
        "compound_read_write",
        &["*-*", ">>>", ">=>", ">L>", ">V>", ">C>"],
    ),
    ("prefix_operators", &["*-*", ">>>", ">L>", ">P>"]),
    ("dotvariable_beyond_the_list", &["*-*", ">>>", ">E>"]),
    (
        "call_arguments",
        &["*-*", ">>>", ">=>", ">L>", ">V>", ">O>", ">A>"],
    ),
    (
        "function_call",
        &["*-*", ">>>", ">=>", ">L>", ">V>", ">O>", ">A>", ">F>"],
    ),
    ("use_arg_alias", &["*-*", ">>>", ">R>"]),
    ("exit_value", &["*-*", ">>>"]),
    (
        "control_variable_reread",
        &["*-*", ">>>", ">=>", ">L>", ">V>", ">K>"],
    ),
    // `TRACE L`'s whole output is three `*-*` lines: no value line of any
    // kind can appear, so this witness claims exactly one prefix and that
    // is not an oversight.
    ("trace_labels", &["*-*"]),
    ("control_variable_novalue", &["*-*", ">>>", ">K>"]),
    (
        "controlled_loop",
        &["*-*", ">>>", ">=>", ">L>", ">V>", ">K>", ">P>"],
    ),
    (
        "parse_placeholder",
        &["*-*", ">>>", ">=>", ">L>", ">K>", ">.>"],
    ),
    ("pull_queue", &["*-*", ">>>", ">=>", ">K>"]),
];

/// Every prefix a witness below is expected to reach, between them.
/// `WITNESS_PREFIXES`'s union must equal this set exactly: not a subset (a
/// prefix could otherwise be claimed by the module doc's own table and never
/// checked at all) and not a superset (a typo'd prefix that no witness could
/// ever really emit would go unnoticed otherwise).
const CLAIMED_PREFIXES: &[&str] = &[
    "*-*", ">>>", ">=>", ">L>", ">V>", ">O>", ">K>", ">C>", ">P>", ">E>", ">A>", ">F>", ">R>",
    ">.>",
];

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len().max(1)).any(|w| w == needle)
}

/// Turns the module doc's prefix-to-witness table from prose into a check.
/// For every witness, its committed `.expected` file's stderr must contain
/// every prefix `WITNESS_PREFIXES` claims for it, as a byte substring --
/// the same shape `check_witness` itself uses for the full comparison, at
/// a coarser grain. A witness that stops witnessing its own prefix (the
/// exact H3 attack: `keyword_while.rex` replaced with a straight-line
/// program, `.expected` regenerated from the live oracle, both still
/// "correct" in the sense that they agree with each other) now fails here
/// instead of passing silently.
#[test]
fn every_witness_still_emits_every_prefix_it_is_named_for() {
    let oracle_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle");
    let mut failures = String::new();
    let mut covered: Vec<&str> = Vec::new();

    for (name, prefixes) in WITNESS_PREFIXES {
        let expected_path = oracle_dir.join(format!("{name}.expected"));
        let bytes = std::fs::read(&expected_path)
            .unwrap_or_else(|e| panic!("{}: unreadable ({e})", expected_path.display()));
        let expected = parse_expected(&bytes, name);
        for prefix in *prefixes {
            if contains_bytes(&expected.stderr, prefix.as_bytes()) {
                covered.push(prefix);
            } else {
                use std::fmt::Write as _;
                writeln!(
                    failures,
                    "{name}: claimed to cover {prefix:?} but its committed \
                     `.expected` stderr does not contain it"
                )
                .unwrap();
            }
        }
    }
    assert!(
        failures.is_empty(),
        "a witness stopped witnessing a prefix this file's own table \
         claims for it:\n{failures}"
    );

    // **The module doc's two *universal* rows, turned into assertions** --
    // "every witness below" is exactly the shape of claim that went stale
    // the moment `trace_labels.rex` landed (re-review NEW-4), and prose
    // cannot notice that. `grep -c '>>>' tests/trace_oracle/*.expected` is
    // the command behind the second one: 0 for `trace_labels.expected` and
    // non-zero for all twelve others.
    let without_clause: Vec<&str> = WITNESS_PREFIXES
        .iter()
        .filter(|(_, prefixes)| !prefixes.contains(&"*-*"))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        without_clause.is_empty(),
        "the module doc's `*-*` row says every witness; these do not claim it:          {without_clause:?}"
    );
    let without_result: Vec<&str> = WITNESS_PREFIXES
        .iter()
        .filter(|(_, prefixes)| !prefixes.contains(&">>>"))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        without_result,
        ["trace_labels"],
        "the module doc's `>>>` row names exactly one exception; the set of          witnesses with no `>>>` has changed"
    );

    covered.sort_unstable();
    covered.dedup();
    let mut expected_union = CLAIMED_PREFIXES.to_vec();
    expected_union.sort_unstable();
    assert_eq!(
        covered, expected_union,
        "WITNESS_PREFIXES' union no longer matches CLAIMED_PREFIXES -- both \
         must be updated together, the module doc's own table with them"
    );
}

/// What criterion 3 can say about one of the oracle's nineteen prefixes.
enum Coverage {
    /// A committed witness above emits it, and `check_witness` compares it
    /// byte for byte.
    Witnessed,
    /// Not reachable from the code this crate runs yet, and the phase named
    /// is where it becomes reachable. The string is spelled exactly as
    /// `phase-4-exclusions.txt`'s own owner column spells it, because that
    /// file and this table are the two places an owner is recorded and a
    /// third spelling would hide a disagreement between them.
    Owned(&'static str),
}

/// **Criterion 3's coverage measure** (D14 amendment 3). Without this table
/// the honest statement is that the witnesses verify what they cover and
/// *how much of the trace surface that is* is measured by nothing.
///
/// Every one of the oracle's nineteen prefixes appears exactly once, either
/// as [`Coverage::Witnessed`] or with the phase that owns it. The owners:
///
/// * `+++` -- Phase 7. Four producers in the C++, and every one of them is
///   behind something D18 defers. `RexxActivation.cpp:4468` is a command's
///   non-zero `RC`, reached through `RexxActivation::command`, whose only
///   two callers are `AddressInstruction.cpp:163` and
///   `CommandInstruction.cpp:89` -- command dispatch, which D18 assigns to
///   Phase 7 even though the `ADDRESS` *instruction* itself is not. The
///   other three are interactive debug: `:4024` (`traceSourceString`, whose
///   caller is guarded by `inDebug()` at `:4305`), `:4237` (the debug
///   prompt, whose message text begins `+++`), and `Activity.cpp:1496`
///   (`displayDebug`, likewise). So an `ADDRESS` instruction alone does not
///   bring this prefix into reach; issuing a command does, and that is the
///   half of `ADDRESS` that is not 4c's. See `phase-4-exclusions.txt`'s own
///   `+++` row for the transcripts.
/// * `>M>` -- Phase 5. Message sends; `ExprKind::Message` is Phase 5's in
///   the exclusions file's own ownership table.
/// * `>N>` -- Phase 5. `traceClassResolution`, a namespace-qualified name,
///   which needs `::REQUIRES`; `ExprKind::QualifiedCall` is Phase 5's there.
/// * `>I>`/`<I<` -- 4c, alongside `::routine` dispatch itself. Measured
///   rather than assumed, and the exclusions file's own row carries the
///   transcripts: the gate is `tracingLabels() && isMethodOrRoutine()`
///   (`RexxActivation.cpp:3655`), so **both** halves are needed, and this
///   crate reaches neither -- `::routine` is deferred to 4c by decision, not
///   by unreachability.
const PREFIX_COVERAGE: &[(&str, Coverage)] = &[
    ("*-*", Coverage::Witnessed),
    ("+++", Coverage::Owned("Phase 7")),
    (">>>", Coverage::Witnessed),
    (">.>", Coverage::Witnessed),
    (">V>", Coverage::Witnessed),
    (">E>", Coverage::Witnessed),
    (">L>", Coverage::Witnessed),
    (">F>", Coverage::Witnessed),
    (">P>", Coverage::Witnessed),
    (">O>", Coverage::Witnessed),
    (">C>", Coverage::Witnessed),
    (">M>", Coverage::Owned("Phase 5")),
    (">A>", Coverage::Witnessed),
    (">=>", Coverage::Witnessed),
    (">I>", Coverage::Owned("4c")),
    (">N>", Coverage::Owned("Phase 5")),
    (">K>", Coverage::Witnessed),
    (">R>", Coverage::Witnessed),
    ("<I<", Coverage::Owned("4c")),
];

/// The coverage number itself, committed so that a change to it is a change
/// to this file rather than a change to a printed line nobody reads.
const WITNESSED_PREFIX_COUNT: usize = 14;

/// The other five, each with an owner. `WITNESSED_PREFIX_COUNT` plus this is
/// asserted to be the whole table, so neither number can drift on its own.
const OUT_OF_SCOPE_PREFIX_COUNT: usize = 5;

/// The phases an owner may name. A phase that has finished cannot own a
/// prefix -- whatever it owned is witnessed by then -- so a finished phase's
/// name does not appear here.
const OWNER_PHASES: &[&str] = &["4c", "Phase 5", "Phase 7"];

/// Criterion 3's coverage measure, asserted rather than printed.
///
/// **A printed number that no assertion reads cannot fail**, which is the
/// whole reason this test exists in the shape it does: the number is a
/// committed literal, and the four checks below are what make it mean
/// something.
///
/// 1. The prefix set equals `support::TRACE_PREFIXES` -- the nineteen read
///    from `RexxActivation.cpp`'s own `trace_prefix_table`. So this table
///    cannot quietly drop a prefix (which would make the coverage fraction
///    look better) or invent one (which would make it look worse).
/// 2. The `Witnessed` subset equals `CLAIMED_PREFIXES`, which
///    [`every_witness_still_emits_every_prefix_it_is_named_for`] has already
///    tied to what the committed `.expected` files actually contain. That
///    chain is what stops "witnessed" from being a claim this file makes
///    about itself.
/// 3. Both counts match their committed literals and add up to the whole
///    table.
/// 4. Every owner names a phase from [`OWNER_PHASES`].
#[test]
fn the_trace_surfaces_coverage_is_fourteen_of_nineteen_with_owners_for_the_rest() {
    let mut listed: Vec<&str> = PREFIX_COVERAGE.iter().map(|(prefix, _)| *prefix).collect();
    listed.sort_unstable();
    let before_dedup = listed.len();
    listed.dedup();
    assert_eq!(
        before_dedup,
        listed.len(),
        "PREFIX_COVERAGE names a prefix twice"
    );
    let mut from_oracle: Vec<String> = support::TRACE_PREFIXES
        .iter()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect();
    from_oracle.sort_unstable();
    assert_eq!(
        listed, from_oracle,
        "PREFIX_COVERAGE is no longer the oracle's own nineteen \
         (`support::TRACE_PREFIXES`)"
    );

    let mut witnessed: Vec<&str> = PREFIX_COVERAGE
        .iter()
        .filter(|(_, coverage)| matches!(coverage, Coverage::Witnessed))
        .map(|(prefix, _)| *prefix)
        .collect();
    witnessed.sort_unstable();
    let mut claimed = CLAIMED_PREFIXES.to_vec();
    claimed.sort_unstable();
    assert_eq!(
        witnessed, claimed,
        "PREFIX_COVERAGE's `Witnessed` rows disagree with CLAIMED_PREFIXES"
    );

    let owned: Vec<&str> = PREFIX_COVERAGE
        .iter()
        .filter_map(|(_, coverage)| match coverage {
            Coverage::Witnessed => None,
            Coverage::Owned(phase) => Some(*phase),
        })
        .collect();
    assert_eq!(witnessed.len(), WITNESSED_PREFIX_COUNT, "witnessed count");
    assert_eq!(owned.len(), OUT_OF_SCOPE_PREFIX_COUNT, "out-of-scope count");
    assert_eq!(
        WITNESSED_PREFIX_COUNT + OUT_OF_SCOPE_PREFIX_COUNT,
        PREFIX_COVERAGE.len(),
        "the two counts no longer add up to the whole table"
    );
    for phase in &owned {
        assert!(
            OWNER_PHASES.contains(phase),
            "{phase:?} is not one of the phases an owner may name"
        );
    }
}
