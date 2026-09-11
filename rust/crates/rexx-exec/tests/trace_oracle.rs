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
//! ```bash
//! ( ulimit -v 1048576; \
//!   LD_LIBRARY_PATH=/path/to/ooRexx/build/lib \
//!   /path/to/ooRexx/build/bin/rexx PROGRAM.rex </dev/null ) \
//!   1>/tmp/out 2>/tmp/err; rc=$?
//! { echo "RC $rc"; echo "===STDOUT==="; cat /tmp/out; \
//!   echo "===STDERR==="; cat /tmp/err; } > PROGRAM.expected
//! ```

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
#[test]
fn pull_queue_covers_the_line_reading_sources_in_both_modes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang/pull_queue.rex");
    check_witness("pull_queue", &path);
}

/// `>M>`: a message send's own result line, at four positions -- inside a
/// `SAY`, at the right of an assignment, as a whole clause, and in the `~~`
/// form whose line shows the target rather than the method's result. The
/// argument-bearing send is what puts an `>A>` between the receiver's `>L>`
/// and the `>M>`.
#[test]
fn message_send_covers_the_message_result_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/message_send.rex");
    check_witness("message_send", &path);
}

/// `>N>`: a namespace-qualified class lookup, `w:NsWidget` against a
/// `::REQUIRES ... NAMESPACE w`.
#[test]
fn namespace_lookup_covers_the_class_resolution_line() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/namespace_lookup.rex");
    check_witness("namespace_lookup", &path);
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
    ("message_send", &["*-*", ">>>", ">=>", ">L>", ">A>", ">M>"]),
    ("namespace_lookup", &["*-*", ">>>", ">N>"]),
];

/// Every prefix a witness below is expected to reach, between them.
/// `WITNESS_PREFIXES`'s union must equal this set exactly: not a subset (a
/// prefix could otherwise be claimed by the module doc's own table and never
/// checked at all) and not a superset (a typo'd prefix that no witness could
/// ever really emit would go unnoticed otherwise).
const CLAIMED_PREFIXES: &[&str] = &[
    "*-*", ">>>", ">=>", ">L>", ">V>", ">O>", ">K>", ">C>", ">P>", ">E>", ">A>", ">F>", ">R>",
    ">.>", ">M>", ">N>",
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
    /// A **live corpus** program emits it, named here by its path relative to
    /// `rust/corpus/`. `tests/corpus.rs` runs that program under both
    /// interpreters on every run and compares all three channels, so the
    /// comparison is as strict as `check_witness`'s -- it is the
    /// *expectation* that cannot be committed, not the check.
    WitnessedLive(&'static str),
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
    (">M>", Coverage::Witnessed),
    (">A>", Coverage::Witnessed),
    (">=>", Coverage::Witnessed),
    (">I>", Coverage::WitnessedLive(LIVE_INVOCATION_WITNESS)),
    (">N>", Coverage::Witnessed),
    (">K>", Coverage::Witnessed),
    (">R>", Coverage::Witnessed),
    ("<I<", Coverage::WitnessedLive(LIVE_INVOCATION_WITNESS)),
];

/// The phase subset files this file reads, in union order.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
    "phase-5b.txt",
    "phase-5c.txt",
    "phase-5d.txt",
    "phase-5j.txt",
];

/// The phase subset files that exist in the corpus directory, sorted.
fn phase_subset_files_on_disk() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let entries =
        std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"))
        .collect();
    names.sort();
    names
}

/// This file reads **every** phase subset file the corpus has.
#[test]
fn the_prefix_table_reads_every_phase_subset_file() {
    let on_disk = phase_subset_files_on_disk();
    // The listing is the half that can come back empty, and a comparison of
    // two empty lists is a pass with nothing behind it. Asserted here rather
    // than left to the non-empty literal below to carry, which is the same
    // reason `gate_table_d.rs`'s owner check asserts its own enumeration.
    assert!(
        !on_disk.is_empty(),
        "no phase-*.txt in rust/corpus/, so the assertion below compared the list against \
         nothing"
    );
    assert_eq!(
        SUBSET_FILES, on_disk,
        "this file does not read every phase subset file in rust/corpus/. A file missing \
         from SUBSET_FILES leaves the union below as it stood before that file landed, and \
         a WitnessedLive row whose program only the missing file names then rests on this \
         file's own output rather than on the oracle"
    );
}

/// Each [`Coverage::WitnessedLive`] row's own chain, the analogue of
/// [`every_witness_still_emits_every_prefix_it_is_named_for`] for a witness
/// whose expectation lives in the live corpus.
#[test]
fn every_live_witness_emits_its_prefix_and_is_run_by_the_corpus() {
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let mut listed = String::new();
    for name in SUBSET_FILES {
        let path = corpus_dir.join(name);
        listed.push_str(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: unreadable ({e})", path.display())),
        );
    }
    for (prefix, coverage) in PREFIX_COVERAGE {
        let Coverage::WitnessedLive(rel_path) = coverage else {
            continue;
        };
        assert!(
            listed
                .lines()
                .any(|line| line.trim() == *rel_path && !line.trim_start().starts_with('#')),
            "{rel_path} witnesses {prefix:?} but is in no phase subset file, so \
             tests/corpus.rs never runs it against the oracle"
        );
        let path = corpus_dir.join(rel_path);
        let source =
            std::fs::read(&path).unwrap_or_else(|e| panic!("{}: unreadable ({e})", path.display()));
        let outcome = run_program(
            &path.to_string_lossy(),
            source,
            rexx_exec::Invocation::none(),
        );
        assert!(
            contains_bytes(&outcome.stderr, prefix.as_bytes()),
            "{rel_path} is this table's witness for {prefix:?} and emitted no \
             such line: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
    }
}

/// The one corpus program that witnesses a prefix no committed file can
/// hold, relative to `rust/corpus/`.
const LIVE_INVOCATION_WITNESS: &str = "lang/routine_dispatch.rex";

/// The coverage number itself, committed so that a change to it is a change
/// to this file rather than a change to a printed line nobody reads. Counts
/// `Witnessed` and `WitnessedLive` together: both are compared against the
/// oracle byte for byte, and the split between them is about where the
/// expectation lives, not about how strict the check is.
const WITNESSED_PREFIX_COUNT: usize = 18;

/// The rest, each with an owner. `WITNESSED_PREFIX_COUNT` plus this is
/// asserted to be the whole table, so neither number can drift on its own.
const OUT_OF_SCOPE_PREFIX_COUNT: usize = 1;

/// The phases an owner may name. A phase that has finished cannot own a
/// prefix -- whatever it owned is witnessed by then -- so a finished phase's
/// name does not appear here.
const OWNER_PHASES: &[&str] = &["Phase 7"];

/// Criterion 3's coverage measure, asserted rather than printed.
#[test]
fn the_trace_surfaces_coverage_is_eighteen_of_nineteen_with_an_owner_for_the_rest() {
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
            Coverage::Witnessed | Coverage::WitnessedLive(_) => None,
            Coverage::Owned(phase) => Some(*phase),
        })
        .collect();
    let live = PREFIX_COVERAGE
        .iter()
        .filter(|(_, coverage)| matches!(coverage, Coverage::WitnessedLive(_)))
        .count();
    assert_eq!(
        witnessed.len() + live,
        WITNESSED_PREFIX_COUNT,
        "witnessed count, committed and live together"
    );
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
