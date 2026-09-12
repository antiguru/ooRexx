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

//! The differential corpus runner: every program named in [`SUBSET_FILES`] --
//! every phase subset file `rust/corpus/` has, read as a union -- run under
//! both interpreters, compared byte for byte on stdout and exit code, and on
//! stderr up to DEVIATION 0's own narrow indent normalisation (see the
//! "DEVIATION 0" section below).

mod support;
mod watchdog;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rexx_exec::Outcome;
use support::oracle::{
    Oracle, StderrComparison, StdoutComparison, descriptor_diffs_modes, did_not_finish,
    wrapped_exit_code,
};

/// Env var that flips this test from a progress report into the phase gate.
/// See the module doc's "REPORT vs STRICT" section.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program. Neither the list nor its count is assumed anywhere
/// else in this file; both come from the files themselves, so the subset can
/// grow or shrink with no change here.
fn read_subset(list_paths: &[&Path]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut union = Vec::new();
    for list_path in list_paths {
        let text = fs::read_to_string(list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if seen.insert(line.to_string()) {
                union.push(line.to_string());
            }
        }
    }
    union
}

/// The directory one corpus program runs in: its own, under the target
/// directory, named by the program's corpus path with `/` spelled `__`. That
/// spelling collapses the path to a single component, so no entry can name a
/// directory outside this root.
fn run_directory(rel_path: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("corpus-run")
        .join(rel_path.replace('/', "__"))
}

/// Empties `dir`, creating it when it is not there.
fn empty_run_directory(dir: &Path) {
    if dir.exists() {
        fs::remove_dir_all(dir).unwrap_or_else(|e| panic!("cannot empty {}: {e}", dir.display()));
    }
    fs::create_dir_all(dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
}

/// Runs the executor in process, on `path`, from `directory`.
fn run_rust(path: &Path, directory: &Path) -> Outcome {
    let text = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let path_str = path
        .to_str()
        .unwrap_or_else(|| panic!("corpus path {} is not valid UTF-8", path.display()));
    // **The same working directory the oracle gets**, which is why the caller
    // chooses it rather than this function taking the program's own: the
    // process's directory is shared between the interpreters this harness runs
    // on threads, and a program naming a relative path must see one state.
    watchdog::run_bounded(
        path_str,
        text,
        rexx_exec::Invocation::none().with_directory(directory.to_path_buf()),
    )
}

/// One corpus program that disagreed with the oracle, or one where either
/// side's run did not finish.
struct Mismatch {
    rel_path: String,
    /// The construct named in `rexx-exec: X is not implemented`, when the
    /// stderr has that shape. `None` for a genuine divergence and for a
    /// structural failure alike.
    owner: Option<String>,
    reason: String,
    /// `true` for a run of either side that did not finish, `false` for an
    /// ordinary byte divergence. Kept as its own field rather than inferred from
    /// `reason`'s text, because a structural failure is red in every mode
    /// and a divergence is red only under the gate -- `corpus_differential`
    /// asserts on this field unconditionally, before the gated assertion
    /// over the whole list, and a bit that says which kind a `Mismatch` is
    /// cannot be conflated with prose the way a substring match could be.
    structural: bool,
}

/// Pulls `X` out of a `rexx-exec: X is not implemented` line, the same
/// pattern the hand-run shell loop grepped for to partition failures by task.
fn owner_from_stderr(stderr: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stderr);
    const MARKER: &str = "rexx-exec: ";
    const SUFFIX: &str = " is not implemented";
    let after_marker = &text[text.find(MARKER)? + MARKER.len()..];
    let end = after_marker.find(SUFFIX)?;
    Some(after_marker[..end].to_string())
}

/// Bounds a byte string to a short, readable excerpt, so an UNCLASSIFIED
/// divergence's report stays diagnosable without reprinting a program's
/// entire output.
fn excerpt(bytes: &[u8]) -> String {
    const BOUND: usize = 200;
    let text = String::from_utf8_lossy(bytes);
    if text.len() > BOUND {
        format!("{}...", &text[..BOUND])
    } else {
        text.into_owned()
    }
}

/// Corpus programs compared byte-for-byte on `stderr` rather than through
/// DEVIATION 0's normalisation. See the module doc's "Opting a program out
/// of DEVIATION 0".
const RAW_STDERR_COMPARISON: &[&str] = &[
    // Phase 5a (2026-08-17 plan) Task 11: what a list expression and a
    // `DO OVER` trace. Every value line here is indented, and the indents are
    // what the program is for -- the `>A>` per written list position against
    // the `>>>` for the list itself, and the `>K>` a `DO OVER` header emits
    // against the `>=>` each pass emits two columns further in.
    "lang/array_trace.rex",
    // Fix round 1, finding B2: RAISE's own keyword value lines. Each renders an
    // object through `stringValue()` and each is indented, which is what the
    // three programs exist to pin; the two `ADDITIONAL`/`ARRAY` spellings also
    // differ from each other only in their traced element lines.
    "lang/raise_additional_array.rex",
    "lang/raise_array_spelling.rex",
    "lang/raise_keyword_object_traces.rex",
    // Fix round 2: a nested array as a substitution item. Its traced element
    // lines are indented and its report reads the item's own object name, both
    // of which the normaliser would blur.
    "lang/raise_array_nested.rex",
    "lang/raise_additional_nested.rex",
    // Phase 5a Task 7's two traced method activations. The indent a
    // `::METHOD` body's clauses echo at is the thing under test -- 0,
    // whatever the sending clause's own indent was -- and normalisation
    // erases exactly that difference, so these two are the first programs
    // that would pass the default comparison while being wrong.
    "lang/method_trace_invocation.rex",
    "lang/method_trace_nested.rex",
    // The `::ATTRIBUTE` accessor pair. Its `>I>`/`<I<` lines sit at the indent
    // the entries above are here for, and they carry the resolved message
    // name -- `"B"` against `"B="` -- which is the only place a program can
    // read that name itself rather than infer it from which body ran.
    "lang/method_attribute_set_body.rex",
    // Phase 5a (2026-08-17 plan) Task 6: the operator-forwarded native-method
    // frame. Raw mode is what asserts the frame line's own bytes -- its
    // leading whitespace and its absent line number included -- rather than
    // resting on whatever DEVIATION 0's normaliser happens to do to them.
    // Measured, od-verified: the gap after the frame line's own `*-*` marker
    // is exactly one space on both sides today, so normalisation is
    // currently a no-op on every entry below's stderr; raw mode is the
    // comparison that still asserts those bytes rather than one that would
    // pass by coincidence if the gap ever widened.
    "lang/operator_frame_stem_plus.rex",
    "lang/operator_frame_stem_power.rex",
    "lang/operator_frame_stem_prefix_minus.rex",
    // Fix round 1, finding 3: the same forwarded frame reached from a
    // controlled `DO` header's numeric position rather than from an
    // operator directly, one program per position. `Interp::header_number`
    // rounds each position through a real unary `+`, so a stem position
    // carries the identical frame -- and the same raw-mode reasoning above
    // applies to it unchanged.
    "lang/operator_frame_stem_do_initial.rex",
    "lang/operator_frame_stem_do_to.rex",
    "lang/operator_frame_stem_do_by.rex",
    // Fix round 2, finding 1: the frame belongs to the receiver of a
    // forwarded operator whatever step of evaluating it raises, not only its
    // own conversion. Each program below reaches the frame from a different
    // failing step past that conversion -- arithmetic overflow, the power
    // exponent's own range check, a non-logical operand on `&` and on prefix
    // `\`, and a DO header's own range check -- and the same raw-mode
    // reasoning above applies to each unchanged.
    "lang/operator_frame_stem_divide_by_zero.rex",
    "lang/operator_frame_stem_power_exponent_range.rex",
    "lang/operator_frame_stem_logical_and.rex",
    "lang/operator_frame_stem_prefix_not.rex",
    "lang/operator_frame_stem_do_exponent_range.rex",
    // Fix round 3, finding 1: the same forwarded frame reached through a
    // non-strict comparison's own numeric conversion, the operator family
    // whose need for a number -- not whether it is arithmetic -- is what
    // decides whether it reaches this frame at all.
    "lang/operator_frame_stem_compare_overflow.rex",
    // Phase 5a (2026-08-17 plan) Task 14: the frame the required-string
    // protocol's own `REQUEST` activation contributes when a `makeString`
    // reached through it raises. The pair is one frame from a language context
    // against two from a method argument, and the frame lines are indented and
    // carry no line number, which is what raw mode asserts.
    "lang/required_string_make_string_raises.rex",
    "lang/required_string_argument_make_string_raises.rex",
    // Phase 5a (2026-08-17 plan) Task 16: what a `REPLY` leaves on stderr.
    // Both programs are traceback-only, and both carry the two-space-and-more
    // gap of a `*-*` clause line at the indent an owed body reports at -- the
    // run of spaces normalisation collapses. `method_reply`'s 98.936 is
    // reported from the resumed half of the body and `method_reply_twice`'s
    // 98.935 from a clause the first half never reached, so the indent is the
    // only thing separating either report from one raised in the sender.
    "lang/method_reply.rex",
    "lang/method_reply_twice.rex",
];

/// Corpus programs whose `stderr` is compared as a multiset of lines, per
/// DEVIATION 7: `REPLY` under a package trace setting has two threads writing
/// trace lines and their interleaving is not a specified observable.
const CONCURRENTLY_TRACED: &[&str] = &["lang/directive_options_trace_reply.rex"];

/// Corpus programs whose `stdout` is compared as a multiset of lines, per
/// DEVIATION 8: what they print is the contents of a hash-ordered
/// `StringTable`, whose iteration order is a bucket walk on both sides and
/// reproduces on neither.
const HASH_ORDERED_STDOUT: &[&str] = &["lang/package_writes.rex"];

/// The `stdout` comparison one corpus entry gets.
fn stdout_mode(rel_path: &str) -> StdoutComparison {
    if HASH_ORDERED_STDOUT.contains(&rel_path) {
        StdoutComparison::Multiset
    } else {
        StdoutComparison::Raw
    }
}

/// The `stderr` comparison one corpus entry gets, and no entry may ask for two.
fn stderr_mode(rel_path: &str) -> StderrComparison {
    match (
        RAW_STDERR_COMPARISON.contains(&rel_path),
        CONCURRENTLY_TRACED.contains(&rel_path),
    ) {
        (true, true) => panic!(
            "{rel_path} is on both RAW_STDERR_COMPARISON and CONCURRENTLY_TRACED, which ask \
             for a stricter and a weaker comparison of the same channel"
        ),
        (true, false) => StderrComparison::Raw,
        (false, true) => StderrComparison::Multiset,
        (false, false) => StderrComparison::Normalized,
    }
}

/// Every entry in [`RAW_STDERR_COMPARISON`], [`CONCURRENTLY_TRACED`] and
/// [`HASH_ORDERED_STDOUT`] is a line some phase subset file actually names.
#[test]
fn every_opted_in_comparison_names_a_program_the_subset_runs() {
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    for (list, opted_into) in [
        (RAW_STDERR_COMPARISON, "raw"),
        (CONCURRENTLY_TRACED, "multiset"),
        (HASH_ORDERED_STDOUT, "sorted stdout"),
    ] {
        for listed in list {
            assert!(
                subset.iter().any(|entry| entry == listed),
                "{listed} is listed for the {opted_into} stderr comparison but is not a \
                 line in any phase subset file -- stderr_mode would silently give it the \
                 normalised comparison instead of the one it is supposed to opt into"
            );
        }
    }
}

/// [`stdout_mode`] answers the multiset mode for exactly the licensed list.
#[test]
fn the_multiset_stdout_mode_is_selected_for_exactly_the_licensed_list() {
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    assert!(
        !HASH_ORDERED_STDOUT.is_empty() && HASH_ORDERED_STDOUT.len() < subset.len(),
        "the licensed list is empty or is the whole subset, so this control          separates nothing"
    );
    for entry in &subset {
        let listed = HASH_ORDERED_STDOUT.contains(&entry.as_str());
        let mode = stdout_mode(entry);
        assert_eq!(
            mode == StdoutComparison::Multiset,
            listed,
            "{entry} is compared as {mode:?} and is {} on HASH_ORDERED_STDOUT",
            if listed { "" } else { "not" }
        );
    }
}

/// DEVIATION 8's own control: for every program on [`HASH_ORDERED_STDOUT`],
/// the two sides' `stdout` **differs** raw and **agrees** sorted.
#[test]
fn the_sorted_stdout_licence_covers_an_ordering_difference_and_nothing_else() {
    let oracle = support::oracle::locate();
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    assert!(
        !HASH_ORDERED_STDOUT.is_empty(),
        "the licensed list is empty, so this control iterates over nothing"
    );
    for listed in HASH_ORDERED_STDOUT {
        let abs = fs::canonicalize(corpus_dir.join(listed))
            .unwrap_or_else(|e| panic!("cannot resolve {listed}: {e}"));
        let dir = run_directory(listed);
        empty_run_directory(&dir);
        let cpp = oracle.run_in(&abs, &dir, &[]);
        empty_run_directory(&dir);
        let rust = run_rust(&abs, &dir);
        assert!(
            !cpp.stdout.is_empty(),
            "{listed}: on HASH_ORDERED_STDOUT and the oracle wrote no stdout, so \
the sorted comparison would agree with anything"
        );
        // Both compared as booleans rather than with `assert_ne!`/`assert_eq!`
        // on the transcripts: the message is the useful part, and those macros
        // would print two whole `Vec<u8>`s as byte arrays.
        assert!(
            rust.stdout != cpp.stdout,
            "{listed}: agrees on stdout byte for byte, so it does not need \
Deviation 8 and must not carry a relaxation that hides an ordering defect"
        );
        assert!(
            support::oracle::stdout_multiset(&rust.stdout)
                == support::oracle::stdout_multiset(&cpp.stdout),
            "{listed}: differs on stdout as a MULTISET of lines, which \
Deviation 8 does not license -- the two sides printed different content, not \
the same content in a different order"
        );
    }
}

/// Runs one corpus entry under both interpreters and compares all three
/// observable channels. `None` when they agree.
fn check_case(oracle: &Oracle, corpus_dir: &Path, rel_path: &str) -> Option<Mismatch> {
    let abs = fs::canonicalize(corpus_dir.join(rel_path))
        .unwrap_or_else(|e| panic!("cannot resolve corpus entry {rel_path}: {e}"));

    // One directory per program, the same absolute path for both interpreters
    // and emptied between them: a program that writes files leaves nothing in
    // `corpus/`, and neither side ever reads what the other left behind.
    let dir = run_directory(rel_path);
    empty_run_directory(&dir);
    let cpp = oracle.run_in(&abs, &dir, &[]);
    empty_run_directory(&dir);
    let rust = run_rust(&abs, &dir);

    // Checked before `descriptor_diffs_modes` calls `cpp.expect_exit_code()`
    // itself: that panic has no `rel_path` in it and fires from inside
    // `support/oracle.rs`, while this file's own report is built from the
    // `Mismatch`es this function returns and printed only after the whole
    // sweep finishes -- so the panic form is not merely less informative
    // here, it never reaches a reader at all. Naming the path is this
    // caller's job because it is the one thing `expect_exit_code` cannot
    // know.
    if did_not_finish(&cpp) {
        return Some(Mismatch {
            rel_path: rel_path.to_string(),
            owner: None,
            reason: format!(
                "the oracle did not finish: {:?} -- a structural failure, not a byte \
                 comparison",
                cpp.termination
            ),
            structural: true,
        });
    }

    // The same reading of the crate side, which `watchdog::run_bounded` gives
    // it. **Structural rather than a divergence**, because a run that did not
    // finish produced no answer to compare: leaving it to the byte comparison
    // would make it red only under the gate, where the oracle's own
    // non-finish above is red in every mode.
    if watchdog::did_not_finish(&rust) {
        return Some(Mismatch {
            rel_path: rel_path.to_string(),
            owner: None,
            reason: format!(
                "the crate did not finish: {:?} -- a structural failure, not a byte comparison",
                String::from_utf8_lossy(&rust.stderr).trim_end()
            ),
            structural: true,
        });
    }

    let rust_exit = wrapped_exit_code(rust.exit_code);

    let diffs = descriptor_diffs_modes(&rust, &cpp, stdout_mode(rel_path), stderr_mode(rel_path));
    if diffs.is_empty() {
        return None;
    }

    let owner = owner_from_stderr(&rust.stderr);
    let reason = match &owner {
        Some(construct) => format!(
            "{} differ (loud failure: {construct} is not implemented; rust rc {rust_exit}, \
             oracle rc {})",
            diffs.join(", "),
            cpp.expect_exit_code()
        ),
        // No loud-failure marker: a real divergence rather than a known gap,
        // so give enough of all three channels to diagnose it from the
        // report alone.
        None => format!(
            "{} differ\n      rust:   stdout={:?} stderr={:?} exit={rust_exit}\n      \
             oracle: stdout={:?} stderr={:?} exit={}",
            diffs.join(", "),
            excerpt(&rust.stdout),
            excerpt(&rust.stderr),
            excerpt(&cpp.stdout),
            excerpt(&cpp.stderr),
            cpp.expect_exit_code()
        ),
    };

    Some(Mismatch {
        rel_path: rel_path.to_string(),
        owner,
        reason,
        structural: false,
    })
}

/// Builds the report text. A `String`, not a `println!` stream: the whole
/// point is that this text crosses to [`emit_uncaptured`] as one payload
/// rather than going through libtest's per-call capture at all.
fn build_report(matched: usize, total: usize, mismatches: &[Mismatch], gate: bool) -> String {
    let banner = "=".repeat(78);
    let mut report = String::new();
    let w = &mut report;
    writeln!(w, "{banner}").unwrap();
    writeln!(
        w,
        "rexx-exec differential corpus report -- rust/corpus/{}",
        SUBSET_FILES.join(" + ")
    )
    .unwrap();
    if gate {
        writeln!(w, "mode: STRICT (the gate) -- {GATE_ENV} is set").unwrap();
    } else {
        writeln!(
            w,
            "*** REPORT MODE -- NOT THE GATE. Set {GATE_ENV}=1 to run this as the gate. ***"
        )
        .unwrap();
    }
    let not_the_gate = if gate {
        ""
    } else {
        " -- REPORT MODE, NOT THE GATE"
    };
    writeln!(w, "{matched} of {total} matching{not_the_gate}").unwrap();

    if !mismatches.is_empty() {
        writeln!(w, "mismatches ({}):", mismatches.len()).unwrap();
        for mismatch in mismatches {
            let owner = mismatch.owner.as_deref().unwrap_or("UNCLASSIFIED");
            writeln!(
                w,
                "  [{owner:<10}] {}: {}",
                mismatch.rel_path, mismatch.reason
            )
            .unwrap();
        }

        let mut by_owner: BTreeMap<&str, usize> = BTreeMap::new();
        for mismatch in mismatches {
            let owner = mismatch.owner.as_deref().unwrap_or("UNCLASSIFIED");
            *by_owner.entry(owner).or_insert(0) += 1;
        }
        writeln!(w, "by owner:").unwrap();
        for (owner, count) in &by_owner {
            writeln!(w, "  {owner}: {count}").unwrap();
        }
    }

    if !gate {
        writeln!(
            w,
            "*** REPORT MODE -- NOT THE GATE. {matched} of {total} matching means {} \
             programs still disagree with the oracle; it is a progress signal for the \
             tasks still landing, not a claim that the phase is done. ***",
            total - matched
        )
        .unwrap();
    }
    writeln!(w, "{banner}").unwrap();
    report
}

/// Writes `text` to the real, process-level stderr, so it reaches the
/// terminal under a plain `cargo test` with no `--nocapture` -- see the
/// module doc's "REPORT vs STRICT" section for why `println!`/`eprintln!`
/// cannot do this from inside a `#[test]`.
fn emit_uncaptured(text: &str) {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("cat >&2")
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawning `sh -c 'cat >&2'` to bypass libtest's output capture");
    child
        .stdin
        .take()
        .expect("stdin was requested as piped")
        .write_all(text.as_bytes())
        .expect("writing the report to the uncaptured-output child's stdin");
    let status = child
        .wait()
        .expect("waiting for the uncaptured-output child");
    assert!(
        status.success(),
        "the uncaptured-output child (`sh -c 'cat >&2'`) exited abnormally: {status}"
    );
}

/// The runner itself. See the module doc for REPORT vs STRICT and how the
/// oracle and the memory limit are handled.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
    "phase-5b.txt",
    "phase-5c.txt",
    "phase-5d.txt",
    "phase-5j.txt",
    "phase-7.txt",
];

/// The phase subset files that exist in the corpus directory, sorted.
fn phase_subset_files_on_disk() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"))
        .collect();
    names.sort();
    names
}

/// The differential reads **every** phase subset file the corpus has.
#[test]
fn the_differential_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the corpus differential does not read every phase subset file in \
         rust/corpus/. A file missing from SUBSET_FILES is a phase whose \
         programs are never run against the oracle, and the run stays green \
         over whatever is left -- the headline shrinks and nothing asserts on \
         it"
    );
}

/// The directory whose programs [`every_lang_program_is_run_or_named_unfiled`]
/// accounts for.
const LANG_SUBDIR: &str = "lang";

/// The companion file naming the ones it does not run, and why.
const UNFILED_FILE: &str = "unfiled.txt";

/// Every `path<TAB>reason` in `corpus/unfiled.txt`.
fn read_unfiled(corpus_dir: &Path) -> Vec<(String, String)> {
    let path = corpus_dir.join(UNFILED_FILE);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (entry, reason) = line.split_once('\t').unwrap_or_else(|| {
                panic!("{}: {line:?} is not a `path<TAB>reason`", path.display())
            });
            assert!(
                !reason.trim().is_empty(),
                "{}: {entry} is named with no reason",
                path.display()
            );
            (entry.to_string(), reason.to_string())
        })
        .collect()
}

/// Every `corpus/lang/*.rex` on disk is either named by a phase subset file or
/// named by [`UNFILED_FILE`], and never both.
#[test]
fn every_lang_program_is_run_or_named_unfiled() {
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let dir = corpus_dir.join(LANG_SUBDIR);
    let mut on_disk: BTreeSet<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".rex"))
        .map(|name| format!("{LANG_SUBDIR}/{name}"))
        .collect();
    assert!(
        !on_disk.is_empty(),
        "{} holds no programs, so this test compared two empty sets",
        dir.display()
    );

    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let filed: BTreeSet<String> =
        read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>())
            .into_iter()
            .filter(|entry| entry.starts_with(&format!("{LANG_SUBDIR}/")))
            .collect();
    let unfiled: Vec<(String, String)> = read_unfiled(&corpus_dir);

    let both: Vec<&String> = unfiled
        .iter()
        .map(|(entry, _)| entry)
        .filter(|entry| filed.contains(*entry))
        .collect();
    assert!(
        both.is_empty(),
        "{both:?} are named by a phase subset file and by {UNFILED_FILE}. The second says \
         nothing runs them against the oracle, and the first says something does"
    );

    for (entry, _) in &unfiled {
        assert!(
            on_disk.contains(entry),
            "{UNFILED_FILE} names {entry}, which is not a program in {}. An exemption for a \
             path that does not exist protects nothing and hides the one that does",
            dir.display()
        );
    }
    for entry in &filed {
        assert!(
            on_disk.contains(entry),
            "a phase subset file names {entry}, which is not a program in {}",
            dir.display()
        );
    }

    for entry in filed.iter().chain(unfiled.iter().map(|(entry, _)| entry)) {
        on_disk.remove(entry);
    }
    assert!(
        on_disk.is_empty(),
        "{on_disk:?} are in {} and named by no phase subset file, so nothing compares them \
         against the oracle and the differential's headline does not count them. File each \
         one, or name it in {UNFILED_FILE} with the reason it is not run",
        dir.display()
    );
}

#[test]
fn corpus_differential() {
    let oracle = support::oracle::locate();
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    // Covers the empty *union* -- every named file present and every one of
    // them naming nothing -- and nothing beyond it. A union that lost a whole
    // file is still non-empty while the others stand, which is why the pin
    // above exists rather than this guard being widened.
    assert!(
        !subset.is_empty(),
        "the phase subset files named no programs -- that is a corpus defect, \
         not an empty pass"
    );

    let mut mismatches = Vec::new();
    let mut matched = 0usize;
    for rel_path in &subset {
        match check_case(&oracle, &corpus_dir, rel_path) {
            None => matched += 1,
            Some(mismatch) => mismatches.push(mismatch),
        }
    }

    let total = subset.len();
    let gate = gate_mode();
    emit_uncaptured(&build_report(matched, total, &mismatches, gate));

    // Unconditional, and checked before the gated assertion below: a
    // structural failure is red in every mode, per the global constraints,
    // and `gate_mode()` exists to relax a *verdict* comparison, not to
    // decide whether a run that never produced bytes to compare gets
    // noticed. Named separately from `mismatches.is_empty()` so that a
    // report-mode run -- most runs -- cannot let a program that stopped
    // finishing pass as a fully-matching corpus just because the corpus
    // gate itself was not requested.
    let structural: Vec<&str> = mismatches
        .iter()
        .filter(|m| m.structural)
        .map(|m| m.rel_path.as_str())
        .collect();
    assert!(
        structural.is_empty(),
        "{} corpus program(s) did not finish, a structural failure in every \
         mode rather than a verdict `{GATE_ENV}` could relax: {structural:?}. \
         See the report above for which side each one stopped on.",
        structural.len()
    );

    assert!(
        !gate || mismatches.is_empty(),
        "STRICT ({GATE_ENV}) mode: {} of {total} corpus programs disagree with \
         the oracle; see the report above for which and why.",
        mismatches.len()
    );
}

/// Not a corpus case. Exists only to be re-executed, alone, by
/// [`demonstrate_the_report_reaches_a_plain_cargo_test`], which is the reason
/// it is `#[ignore]`d: a normal test run must not run it as a case of its
/// own, only as the child process the demonstration spawns.
#[test]
#[ignore = "run only by demonstrate_the_report_reaches_a_plain_cargo_test, as a child process"]
fn probe_emit_uncaptured_marker() {
    emit_uncaptured("EMIT-UNCAPTURED-PROBE-MARKER\n");
}

/// Proves `emit_uncaptured` reaches the terminal under a plain `cargo test`,
/// rather than asserting it should. See the module doc's "REPORT vs STRICT"
/// section for why this cannot observe its own process's fd 2 and instead
/// re-executes this test binary as a child running only
/// [`probe_emit_uncaptured_marker`], with no `--nocapture` -- the same
/// invocation `cargo test` (workspace or single-crate) uses on every test
/// binary. `--include-ignored` is required for exactly one reason: the
/// probe's own `#[ignore]`, which exists so *this* process's normal test run
/// does not also execute it directly.
#[test]
fn demonstrate_the_report_reaches_a_plain_cargo_test() {
    let exe = env::current_exe().expect("a running test binary knows its own path");
    let output = Command::new(exe)
        .args([
            "--exact",
            "probe_emit_uncaptured_marker",
            "--include-ignored",
        ])
        .output()
        .expect("re-executing this test binary against itself");

    let mut combined = output.stdout;
    combined.extend_from_slice(&output.stderr);
    let text = String::from_utf8_lossy(&combined);
    assert!(
        text.contains("EMIT-UNCAPTURED-PROBE-MARKER"),
        "the probe's marker did not reach the child test binary's own captured \
         output under a plain (no --nocapture) run, so emit_uncaptured is not \
         bypassing libtest's capture. Full child output:\n{text}"
    );
}
