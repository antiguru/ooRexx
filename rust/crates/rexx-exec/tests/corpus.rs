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

//! The differential corpus runner: every program named in the phase subset
//! files -- `rust/corpus/phase-4a.txt` and `rust/corpus/phase-4b.txt`, read as
//! a union -- run under both interpreters, compared byte for byte on stdout,
//! stderr and exit code.
//!
//! **Every phase's subset file is read, not only the current phase's.** A
//! construct a later phase implements cannot have its witness in
//! `phase-4a.txt`, whose own header excludes those constructs by definition,
//! so each phase adds a file and this runner unions them. 4b's Task 1 is the
//! first task to make that true here; before it, this call site was the one
//! `read_subset` caller of four still pinned to a single file, which would
//! have left a 4b witness enumerated by `coverage.rs` and never actually run
//! against the oracle by anything.
//!
//! **This is a repeatable progress instrument, not a once-at-the-end gate.**
//! It replaces the hand-run shell loop this phase has used after every task
//! (3 of 26 programs matching before Task 9, 9 of 26 after), and each of the
//! two tasks still to land should be able to see its own effect by re-running
//! it. See [`REPORT vs STRICT`](#report-vs-strict) below for how the same run
//! serves both that daily use and the phase gate.
//!
//! # The oracle
//!
//! `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, hardcoded rather than made
//! configurable: the entire point of this test is "does the executor agree
//! with *this* build", and an env var that could point it at a different one
//! would let a stale binary answer for the current oracle with nothing to
//! notice. If that binary is missing, the test **fails**, loudly, rather than
//! skipping: a machine without the oracle reporting "0 of 0 matching" and
//! going green would be indistinguishable from a machine with the oracle and
//! a fully-passing corpus, which is exactly the silent-vacuous-harness shape
//! this project keeps finding in its own instruments. A failure names the
//! missing path and what to do about it; nothing here can go green by
//! accident.
//!
//! # The memory limit
//!
//! Every oracle invocation is wrapped as `sh -c 'ulimit -v <KiB> && exec "$0"
//! "$@"' <binary> <args...>`, matching the `( ulimit -v 1048576; ... )` this
//! project runs by hand everywhere else it touches the oracle (the
//! sourceline-oracle regeneration recipe, this phase's own ad hoc corpus
//! loop). `std::process::Command` has no direct rlimit hook; the alternative
//! is an `unsafe` `pre_exec` closure calling `setrlimit`, which the workspace
//! forbids (`unsafe_code = "forbid"`) and which buys nothing a shell builtin
//! does not already do for free. Verified directly, outside this test: `sh -c
//! 'ulimit -v 1048576 && exec "$0" "$@"' python3 -c 'bytearray(2 * 1024 *
//! 1024 * 1024)'` raises `MemoryError` under the limit and does not without
//! it, and the same wrapper still runs an ordinary corpus program (`say
//! 1/3`-shaped `arith_digits.rex`) to rc 0. The `"$0" "$@"` form passes the
//! binary and its arguments as separate `argv` entries rather than
//! interpolating them into the shell string, so no path needs escaping.
//!
//! # Owner grouping
//!
//! Every mismatch today is a *clean loud failure*: nothing produces a wrong
//! answer, an unimplemented construct exits [`rexx_exec::NOT_IMPLEMENTED_EXIT`]
//! with `rexx-exec: X is not implemented` on stderr, naming the construct.
//! This is exactly the string the hand-run shell loop grepped for to
//! partition failures by which task owns them, and doing the same thing here
//! is what turns "17 mismatches" into an actionable list rather than a wall of
//! diffs. A mismatch whose stderr does not have that shape is a genuine
//! divergence rather than a gap, and is reported as `UNCLASSIFIED` with a
//! bounded excerpt of all three channels, since that is the case a reader
//! cannot otherwise diagnose from the summary alone.
//!
//! # REPORT vs STRICT
//!
//! [`GATE_ENV`] chosen as the switch, because that is what this phase's own
//! ledger calls the distinction ("the strict switch, so it runs in report
//! mode with a flag the gate flips"): unset or empty or `"0"` is REPORT mode,
//! anything else is STRICT.
//!
//! REPORT (the default) always exits 0, however many programs disagree with
//! the oracle, and prints the count, the full mismatch list and the
//! owner breakdown. **The summary line itself carries the caveat that this is
//! not the gate**, top and bottom, rather than leaving it to a doc comment
//! nobody reads at the moment they see green: a `cargo test` line that means
//! "17 of 26 disagree" is exactly the failure this project keeps finding in
//! its own harnesses, and the worst place to introduce it is the instrument
//! that measures the others.
//!
//! **The report does not rely on `--nocapture`.** `println!`/`eprintln!`
//! inside a `#[test]` write through a *thread-local* sink libtest swaps in for
//! the duration of the test, not through the process's real file descriptor
//! 2 -- so a passing test's own prints are invisible under a plain `cargo
//! test`, `--nocapture` or not, is a flag the reader has to already know to
//! reach for, and documenting that flag as the answer is the same
//! reader-has-to-already-know-to-ask shape as the silent pass this report
//! exists to prevent. `emit_uncaptured` instead pipes the finished report into
//! a `cat >&2` child process whose *stderr is inherited*: a child's inherited
//! descriptor is dup'd from this process's own real fd 2 at spawn time, which
//! the thread-local swap never touches, so the write reaches the terminal (or
//! whatever `cargo test`'s own stderr is connected to) regardless of capture
//! state. Verified, not assumed: a throwaway two-test crate
//! (`a_captured_println_is_invisible` / `a_subprocess_write_bypasses_capture`)
//! run under plain `cargo test`, no flags, showed the `println!` line nowhere
//! in the terminal and the subprocess-written line printed inline between the
//! two `test ... ok` lines.
//!
//! `demonstrate_the_report_reaches_a_plain_cargo_test` below is that same
//! proof, kept in the tree rather than left as a one-off experiment. It cannot
//! observe its *own* real fd 2 from inside itself with no `unsafe` (the only
//! way to intercept a process's own inherited descriptor is a `dup2`-shaped
//! syscall), so it instead re-executes **this same test binary**
//! (`std::env::current_exe`) as a child, asking libtest for exactly one
//! `#[ignore]`d probe test and *not* passing `--nocapture` -- the identical
//! invocation `cargo test` itself uses on every test binary in the workspace.
//! Piping that child's stdout and stderr back (`Command::output`) is legitimate
//! here in a way it would not be for the real report: this process is the
//! child's *parent*, so reading its pipes is not "capturing your own tests'
//! output" but observing a separate process from outside, exactly what a
//! human at a terminal does. Finding the probe's marker in that captured
//! output demonstrates the mechanism survives an ordinary, flagless test run;
//! not finding it would mean this file's whole premise is wrong.
//!
//! STRICT (`REXX_CORPUS_GATE=1 cargo test ...`) runs the identical comparison
//! and fails the test if any program mismatches. The report is written the
//! same way either way, so a gate failure and a report-mode run are equally
//! visible; the assertion failure is what turns the mismatch into a non-zero
//! exit, not what makes it legible.

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rexx_exec::Outcome;

/// Address-space ceiling imposed on every oracle invocation, in KiB. 1 GiB:
/// the figure this project has used by hand throughout Phase 4a, restated
/// here as a named constant rather than a magic number in the format string.
const ORACLE_MEMORY_LIMIT_KIB: u64 = 1_048_576;

/// Root of the built C++ oracle. See the module doc for why this is
/// hardcoded rather than read from an env var.
fn oracle_root() -> PathBuf {
    PathBuf::from("/home/moritz/dev/repos/ooRexx/build")
}

/// Env var that flips this test from a progress report into the phase gate.
/// See the module doc's "REPORT vs STRICT" section.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// The oracle binary and the library directory it needs on `LD_LIBRARY_PATH`.
struct Oracle {
    binary: PathBuf,
    lib_dir: PathBuf,
}

/// Locates the oracle, or fails the test naming exactly what is missing.
///
/// A failure here, not a skip: see the module doc's "The oracle" section for
/// why a missing binary must never let this test go green having compared
/// nothing.
fn oracle() -> Oracle {
    let root = oracle_root();
    let binary = root.join("bin/rexx");
    let lib_dir = root.join("lib");
    assert!(
        binary.is_file(),
        "the oracle binary is missing at {}. This test compares the executor \
         against a built ooRexx C++ interpreter; without it there is nothing \
         to compare against, and a machine reporting \"0 of 0 matching\" here \
         would look identical to one where every program actually passed. \
         Build ooRexx there first.",
        binary.display()
    );
    Oracle { binary, lib_dir }
}

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program. Neither the list nor its count is assumed anywhere
/// else in this file; both come from the files themselves, so the subset can
/// grow or shrink with no change here.
///
/// **Task 0's Step 4.** Was a single-file reader (`&Path`); widened to `&[&Path]`
/// so a later task's own subset file can run *alongside* `phase-4a.txt`
/// rather than replacing it -- see `coverage.rs`'s own copy of this function
/// for the fuller argument. The caller below passes `phase-4a.txt` and
/// `phase-4b.txt` since 4b's Task 1.
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

/// Runs the executor in process, on `path`.
///
/// `path` is passed through as-is, already canonicalised by the caller: a
/// raised condition's report names the program by its absolute,
/// dot-normalised path, the oracle prints exactly that, and `rexx-run`'s own
/// `std::fs::canonicalize` is what makes the two agree. Passing anything else
/// here would make every raising program mismatch on stderr regardless of
/// whether the executor is right.
fn run_rust(path: &Path) -> Outcome {
    let text = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let path_str = path
        .to_str()
        .unwrap_or_else(|| panic!("corpus path {} is not valid UTF-8", path.display()));
    rexx_exec::run_program(path_str, text)
}

/// What one oracle run produced. Deliberately not `rexx_exec::Outcome`: that
/// type carries a `stack: StackSpan` field this process never measures, and
/// reusing it would invite comparing a field that was never filled in.
struct CppOutcome {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
}

/// Runs the oracle under the memory limit. See the module doc for the
/// mechanism and how it was verified.
fn run_oracle(oracle: &Oracle, path: &Path) -> CppOutcome {
    let cwd = path.parent().unwrap_or(Path::new("."));
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" \"$@\""
        ))
        .arg(&oracle.binary)
        .arg(path)
        .current_dir(cwd)
        .env("LD_LIBRARY_PATH", &oracle.lib_dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn the oracle for {}: {e}", path.display()));
    CppOutcome {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.status.code().unwrap_or(-1),
    }
}

/// Truncates an in-process exit code to the single byte a real process's
/// status would carry.
///
/// `rexx_exec::Outcome::exit_code` can be wider than a byte (`EXIT`'s own
/// expression result, before `rexx-run`'s `as u8` wraps it for the OS), while
/// the oracle subprocess's exit status is already a byte by construction --
/// `std::process::ExitStatus::code` only ever returns what `WEXITSTATUS`
/// gives. Comparing the two without this would make `exit 256` (in-process
/// `256`, real process `0`) look like a divergence that `rexx-run`'s own
/// wrapping already resolves.
fn wrapped_exit_code(code: i32) -> i32 {
    i32::from(code as u8)
}

/// One corpus program that disagreed with the oracle.
struct Mismatch {
    rel_path: String,
    /// The construct named in `rexx-exec: X is not implemented`, when the
    /// stderr has that shape. `None` for a genuine divergence.
    owner: Option<String>,
    reason: String,
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

/// Runs one corpus entry under both interpreters and compares all three
/// observable channels. `None` when they agree.
fn check_case(oracle: &Oracle, corpus_dir: &Path, rel_path: &str) -> Option<Mismatch> {
    let abs = fs::canonicalize(corpus_dir.join(rel_path))
        .unwrap_or_else(|e| panic!("cannot resolve corpus entry {rel_path}: {e}"));

    let rust = run_rust(&abs);
    let cpp = run_oracle(oracle, &abs);
    let rust_exit = wrapped_exit_code(rust.exit_code);

    let mut diffs = Vec::new();
    if rust.stdout != cpp.stdout {
        diffs.push("stdout");
    }
    if rust.stderr != cpp.stderr {
        diffs.push("stderr");
    }
    let exit_differs = rust_exit != cpp.exit_code;
    if exit_differs {
        diffs.push("exit code");
    }
    if diffs.is_empty() {
        return None;
    }

    let owner = owner_from_stderr(&rust.stderr);
    let reason = match &owner {
        Some(construct) => format!(
            "{} differ (loud failure: {construct} is not implemented; rust rc {rust_exit}, \
             oracle rc {})",
            diffs.join(", "),
            cpp.exit_code
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
            cpp.exit_code
        ),
    };

    Some(Mismatch {
        rel_path: rel_path.to_string(),
        owner,
        reason,
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
        "rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt"
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
///
/// `sh -c 'cat >&2'` rather than `Command::new("cat")` directly: `cat`'s own
/// stdout has to land on *this process's* real fd 2, and redirecting a
/// child's stdout to a specific existing descriptor is exactly what a shell's
/// `>&2` does; reaching for the same effect through `std::process::Stdio`
/// alone would need a raw-fd constructor this workspace's `unsafe_code =
/// "forbid"` lint rules out. Setting the `Command`'s own `stderr` to
/// `Stdio::inherit()` is what makes that `>&2` resolve to the *real* fd 2:
/// a child's inherited descriptor is dup'd from the parent's at spawn time,
/// upstream of libtest's thread-local capture.
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
///
/// Today's expected result, at commit `e0e57825`: 9 of 26 matching, the
/// remaining 17 partitioned as `DO` 10, `TRACE` 4, `SELECT` 2, `IF` 1 --
/// reproduced with a standalone shell loop before this test was written.
/// Tasks implementing `IF` and `SELECT` may move this number out from under a
/// later run; that is expected, not a regression, and the fix is to re-run
/// and record which commit was measured, not to adjust this comment to match
/// a stale number.
#[test]
fn corpus_differential() {
    let oracle = oracle();
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let subset = read_subset(&[
        &corpus_dir.join("phase-4a.txt"),
        &corpus_dir.join("phase-4b.txt"),
    ]);
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
