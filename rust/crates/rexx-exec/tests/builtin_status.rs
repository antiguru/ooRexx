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

//! Where the builtin implemented/not-implemented boundary sits, measured
//! rather than described, and committed as `rust/corpus/builtin-status.txt`.
//!
//! For every name in `rexx_inventory::builtins::NAMES` this derives one of
//! four statuses and asserts the derivation equals the committed file, in
//! both directions. A task that implements a builtin re-runs this, sees
//! exactly which rows flipped, and commits them.
//!
//! # Why the boundary is a file and not prose
//!
//! `rust/CLAUDE.md`'s own rule: phase status is a mutable aggregate, and it
//! is the one that rots. "Owned by 4c", "the twelve builtins so far" and
//! "not yet implemented" are all claims about where this boundary sits, and
//! the boundary moves every time a task lands. The measured evidence for the
//! rule is a contrast between two media -- a 796-row derived-and-policed
//! table needed no correction across thirteen tasks, while a one-line count
//! comment stating the same kind of fact in prose rotted four times. This
//! file is the derived-and-policed medium for the builtins, which is what
//! makes asserting the boundary cheaper than writing about it.
//!
//! # The four statuses
//!
//! * `excluded` -- the name is excluded outright from Phase 4
//!   (`rexx_inventory::builtins::wholly_excluded`, which is
//!   `docs/superpowers/plans/phase-4-exclusions.txt`'s fifteen). Nothing is
//!   run for it: there is no probe and no oracle invocation.
//! * `loud` -- the executor exited [`rexx_exec::NOT_IMPLEMENTED_EXIT`]. The
//!   gap is declared rather than silent, which is the contract every
//!   unimplemented construct in this crate holds to.
//! * `implemented` -- stdout, stderr and exit status all matched the oracle
//!   on this name's probe.
//! * `divergent` -- neither. A wrong answer, not a missing one.
//!
//! # Three statuses are outcomes; `divergent` is a defect
//!
//! A divergence is the failure mode this whole project is organised to
//! prevent: an implementation that runs, returns, and is wrong. Committing a
//! `divergent` row is therefore not "recording a status" but recording a
//! known-wrong answer, and it requires a `KNOWN GAP: <NAME>` marker in the
//! exclusions file naming it, which [`every_divergent_row_has_a_known_gap`]
//! enforces. Without that, a divergence could be absorbed into this file by
//! the same one-line edit that records a legitimate flip.
//!
//! # Why the oracle invocation count is asserted
//!
//! Consider a classifier of the shape `if EXCLUDED.contains(n) { excluded }
//! else if DISPATCHED.contains(n) { implemented } else { loud }`. It
//! satisfies set equality against the committed file, every count, and the
//! `divergent`-is-empty rule -- while running no program at all and
//! measuring nothing. The one thing it cannot do is start a subprocess, so
//! [`support::oracle::Oracle`] counts its own runs and
//! [`the_status_file_matches_a_live_differential_run`] asserts the total
//! equals the number of in-scope names. That assertion is the difference
//! between this file being a measurement and being a second copy of the
//! exclusion list.
//!
//! # Each probe gets a directory of its own
//!
//! A Rexx call to an unresolved name searches the current directory for an
//! external routine, so a directory holding another probe's `.rex` file is a
//! directory where a call can silently run the wrong program. Measured on
//! this host, the same program reported error 44.1 rc 212 from a directory
//! of stale probes and 43.1 rc 213 from a fresh empty one. Every probe here
//! is written as `probe.rex` into a freshly created directory named for its
//! builtin, and both interpreters are pointed at that same absolute path --
//! which also matters for the comparison itself, since a raised condition's
//! report names the program by its path and the two sides must print the
//! same one.

mod support;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rexx_exec::NOT_IMPLEMENTED_EXIT;
use support::oracle::{Oracle, descriptor_diffs};

/// One row's classification. The spelling of each is the token that appears
/// in `builtin-status.txt`, so the file and this enum cannot disagree about
/// what a status is called.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Implemented,
    Loud,
    Divergent,
    Excluded,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Implemented => "implemented",
            Status::Loud => "loud",
            Status::Divergent => "divergent",
            Status::Excluded => "excluded",
        }
    }

    fn parse(token: &str) -> Option<Status> {
        match token {
            "implemented" => Some(Status::Implemented),
            "loud" => Some(Status::Loud),
            "divergent" => Some(Status::Divergent),
            "excluded" => Some(Status::Excluded),
            _ => None,
        }
    }
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

fn probes_path() -> PathBuf {
    corpus_dir().join("builtin-probes.txt")
}

fn status_path() -> PathBuf {
    corpus_dir().join("builtin-status.txt")
}

fn exclusions_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/superpowers/plans/phase-4-exclusions.txt")
}

/// Splits a `NAME<TAB>rest` file into pairs, skipping `#` comments and blank
/// lines. Both committed files this test reads have that shape, and reading
/// them the same way is what keeps "a row exists" meaning the same thing in
/// each.
fn read_tab_rows(path: &Path) -> Vec<(String, String)> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, rest) = line.split_once('\t').unwrap_or_else(|| {
            panic!(
                "{}:{} has no tab separator: {line:?}",
                path.display(),
                index + 1
            )
        });
        rows.push((name.to_string(), rest.to_string()));
    }
    rows
}

/// A run directory nothing else has written to, under Cargo's own per-target
/// temporary directory. Named with the pid and the clock so two concurrent
/// runs of this binary cannot collide -- `cargo test` runs test binaries in
/// parallel and a shared fixed path would let one run's probes appear in
/// another's search path, which is precisely what the module doc says must
/// not happen.
fn fresh_run_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("builtin-status-{}-{nanos}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)
            .unwrap_or_else(|e| panic!("cannot clear {}: {e}", root.display()));
    }
    fs::create_dir_all(&root).unwrap_or_else(|e| panic!("cannot create {}: {e}", root.display()));
    root
}

/// What one probe produced, beyond its status: kept so the assertions below
/// can say *why* a row reads the way it does without running anything twice.
struct Measured {
    status: Status,
    /// The executor's stderr. Only read for a `loud` row, where it must name
    /// the builtin the row is about.
    rust_stderr: Vec<u8>,
    /// A bounded rendering of both sides and which channels disagreed, for
    /// the report a flipped or divergent row prints.
    detail: String,
}

/// Bounds a byte string to a short, readable excerpt, so a divergence's
/// report stays diagnosable without reprinting a program's entire output.
fn excerpt(bytes: &[u8]) -> String {
    const BOUND: usize = 200;
    let text = String::from_utf8_lossy(bytes);
    if text.len() > BOUND {
        format!("{}...", &text[..BOUND])
    } else {
        text.into_owned()
    }
}

/// Runs one probe under both interpreters and classifies the result.
fn measure(oracle: &Oracle, run_root: &Path, name: &str, program: &str) -> Measured {
    let dir = run_root.join(name);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    let file = dir.join("probe.rex");
    fs::write(&file, format!("{program}\n"))
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));
    let path_str = abs
        .to_str()
        .unwrap_or_else(|| panic!("probe path {} is not valid UTF-8", abs.display()));

    let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
    let rust = rexx_exec::run_program(path_str, text);
    let cpp = oracle.run(&abs);

    let diffs = descriptor_diffs(&rust, &cpp);
    // Order matters: a loud failure is a loud failure whatever the oracle
    // did, and asking "did all three match" first would classify a builtin
    // whose probe the oracle also rejects as `implemented`.
    let status = if rust.exit_code == NOT_IMPLEMENTED_EXIT {
        Status::Loud
    } else if diffs.is_empty() {
        Status::Implemented
    } else {
        Status::Divergent
    };

    let detail = format!(
        "differing: [{}]\n        rust:   stdout={:?} stderr={:?} exit={}\n        \
         oracle: stdout={:?} stderr={:?} exit={}",
        diffs.join(", "),
        excerpt(&rust.stdout),
        excerpt(&rust.stderr),
        rust.exit_code,
        excerpt(&cpp.stdout),
        excerpt(&cpp.stderr),
        cpp.exit_code
    );

    Measured {
        status,
        rust_stderr: rust.stderr,
        detail,
    }
}

/// Whether `haystack` contains `name` as a whole word -- neither preceded nor
/// followed by an ASCII alphanumeric.
///
/// The boundary check is the whole point. `WORD` is a prefix of `WORDS`,
/// `WORDINDEX`, `WORDLENGTH` and `WORDPOS`, so a plain substring test would
/// let `routine "WORDS" is not implemented` pass as evidence that the `WORD`
/// row failed on `WORD`.
fn mentions_as_word(haystack: &[u8], name: &str) -> bool {
    let text = String::from_utf8_lossy(haystack);
    let bytes = text.as_bytes();
    let needle = name.as_bytes();
    bytes.windows(needle.len()).enumerate().any(|(at, window)| {
        window == needle
            && !at
                .checked_sub(1)
                .is_some_and(|i| bytes[i].is_ascii_alphanumeric())
            && !bytes
                .get(at + needle.len())
                .is_some_and(u8::is_ascii_alphanumeric)
    })
}

/// The measurement itself, plus the derived table it produces.
struct Run {
    derived: Vec<(String, Status)>,
    measured: BTreeMap<String, Measured>,
    oracle_invocations: usize,
    in_scope: Vec<&'static str>,
}

/// Classifies every name in `NAMES`, running a probe for each in-scope one.
///
/// Called by each test below rather than shared through a `static`: the runs
/// are seconds apart at most, and a lazily-shared oracle handle would make
/// the invocation-count assertion depend on which tests libtest chose to run
/// and in what order -- an assertion that changes meaning under `--exact` is
/// not an assertion.
fn classify() -> Run {
    let probes: BTreeMap<String, String> = read_tab_rows(&probes_path()).into_iter().collect();
    let in_scope = rexx_inventory::builtins::in_scope();
    let whole = rexx_inventory::builtins::wholly_excluded();

    // Both directions between the probe file and the in-scope set, before
    // anything runs: a missing probe would otherwise surface as a confusing
    // panic inside the loop, and a spurious one would never be noticed.
    for name in &in_scope {
        assert!(
            probes.contains_key(*name),
            "{name} is in scope but has no probe in {}",
            probes_path().display()
        );
    }
    for name in probes.keys() {
        assert!(
            in_scope.contains(&name.as_str()),
            "{} has a probe for {name}, which is not an in-scope builtin",
            probes_path().display()
        );
    }

    let oracle = support::oracle::locate();
    let run_root = fresh_run_root();

    let mut derived = Vec::new();
    let mut measured = BTreeMap::new();
    for name in rexx_inventory::builtins::NAMES {
        if whole.contains(name) {
            derived.push(((*name).to_string(), Status::Excluded));
            continue;
        }
        let program = &probes[*name];
        let result = measure(&oracle, &run_root, name, program);
        derived.push(((*name).to_string(), result.status));
        measured.insert((*name).to_string(), result);
    }

    // Only on success: a failing run's probe directories are the evidence.
    let _ = fs::remove_dir_all(&run_root);

    Run {
        derived,
        measured,
        oracle_invocations: oracle.invocations(),
        in_scope,
    }
}

/// Renders the derived table in `builtin-status.txt`'s data format, so a
/// task whose change flipped a row can see exactly what to commit.
fn render(derived: &[(String, Status)]) -> String {
    let mut out = String::new();
    for (name, status) in derived {
        writeln!(out, "{name}\t{}", status.as_str()).expect("writing to a String cannot fail");
    }
    out
}

/// The whole derivation, against the committed file, both directions, plus
/// the counts and the invocation total. See the module doc for why the
/// invocation total is the assertion that stops a name-table classifier.
#[test]
fn the_status_file_matches_a_live_differential_run() {
    let run = classify();
    let committed = read_tab_rows(&status_path());

    for (name, token) in &committed {
        assert!(
            Status::parse(token).is_some(),
            "{}: {name} has status {token:?}, which is not one of implemented/loud/\
             divergent/excluded",
            status_path().display()
        );
    }
    let committed: Vec<(String, Status)> = committed
        .into_iter()
        .map(|(name, token)| (name, Status::parse(&token).expect("checked just above")))
        .collect();

    // Set equality, said in both directions with a message that names which
    // way it went -- "the sets differ" leaves a reader to work out whether
    // they must re-run the harness or fix the file.
    let derived_by_name: BTreeMap<&str, Status> = run
        .derived
        .iter()
        .map(|(name, status)| (name.as_str(), *status))
        .collect();
    let committed_by_name: BTreeMap<&str, Status> = committed
        .iter()
        .map(|(name, status)| (name.as_str(), *status))
        .collect();

    let derived_text = render(&run.derived);
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("builtin-status.derived.txt");
    let _ = fs::write(&scratch, &derived_text);
    let how_to_fix = format!(
        "The measured table has been written to {}; copy its rows into {} if \
         the change was intended.",
        scratch.display(),
        status_path().display()
    );

    let missing: Vec<&str> = derived_by_name
        .keys()
        .copied()
        .filter(|name| !committed_by_name.contains_key(name))
        .collect();
    assert!(
        missing.is_empty(),
        "measured but absent from {}: {missing:?}. {how_to_fix}",
        status_path().display()
    );
    let extra: Vec<&str> = committed_by_name
        .keys()
        .copied()
        .filter(|name| !derived_by_name.contains_key(name))
        .collect();
    assert!(
        extra.is_empty(),
        "committed in {} but not produced by the run: {extra:?} -- either the \
         name left BuiltinFunctions.cpp's table or the row is a typo",
        status_path().display()
    );

    let mut flipped = String::new();
    for (name, derived_status) in &derived_by_name {
        let committed_status = committed_by_name[name];
        if *derived_status != committed_status {
            let detail = run
                .measured
                .get(*name)
                .map(|m| format!("\n        {}", m.detail))
                .unwrap_or_default();
            writeln!(
                flipped,
                "  {name}: committed {}, measured {}{detail}",
                committed_status.as_str(),
                derived_status.as_str()
            )
            .expect("writing to a String cannot fail");
        }
    }
    assert!(
        flipped.is_empty(),
        "rows whose measured status differs from {}:\n{flipped}{how_to_fix}",
        status_path().display()
    );

    // The counts. `NAMES.len()` and the size of the whole-exclusion set are
    // both stable facts about the C++ table and the exclusions file, not
    // facts about how much is implemented, so asserting them here cannot rot
    // as a task lands.
    assert_eq!(
        run.derived.len(),
        rexx_inventory::builtins::NAMES.len(),
        "one row per builtin"
    );
    let excluded = run
        .derived
        .iter()
        .filter(|(_, status)| *status == Status::Excluded)
        .count();
    assert_eq!(
        excluded,
        rexx_inventory::builtins::wholly_excluded().len(),
        "the excluded rows are exactly the names excluded outright"
    );
    assert_eq!(excluded, 15, "phase-4-exclusions.txt's whole exclusions");
    assert_eq!(
        run.derived.len() - excluded,
        66,
        "66 of the 81 builtins are in scope, three of them partially"
    );

    // The assertion a classifier that consults only name tables cannot pass.
    assert_eq!(
        run.oracle_invocations,
        run.in_scope.len(),
        "the oracle was invoked {} times for {} in-scope builtins; every \
         in-scope name must be measured by running its probe, and no excluded \
         name may run anything",
        run.oracle_invocations,
        run.in_scope.len()
    );
    assert_eq!(
        run.oracle_invocations, 66,
        "one oracle run per in-scope name"
    );
}

/// A `loud` row must be loud about *its own* builtin.
///
/// The failure this catches is a probe that reaches some other unimplemented
/// name first, which makes the row answer for that name instead: measured,
/// a draft that rendered the bit operations with `c2x(...)` reported
/// `routine "C2X" is not implemented` for BITAND, BITOR, BITXOR, D2C and
/// XRANGE, so five rows would have stayed `loud` for as long as C2X was --
/// long after their own builtin worked. Nothing in the status file itself
/// distinguishes that from an honest gap.
#[test]
fn every_loud_row_is_loud_about_its_own_builtin() {
    let run = classify();
    let mut wrong = String::new();
    for (name, result) in &run.measured {
        if result.status != Status::Loud {
            continue;
        }
        if !mentions_as_word(&result.rust_stderr, name) {
            writeln!(
                wrong,
                "  {name}: {:?}",
                String::from_utf8_lossy(&result.rust_stderr)
            )
            .expect("writing to a String cannot fail");
        }
    }
    assert!(
        wrong.is_empty(),
        "these rows exited NOT_IMPLEMENTED_EXIT without their own builtin's \
         name in the message, so the row is answering for something else:\n{wrong}"
    );
}

/// Committing a `divergent` row requires a `KNOWN GAP: <NAME>` marker in the
/// exclusions file. See the module doc: a divergence is a wrong answer, and
/// absorbing one into the status file must cost more than a one-line edit.
#[test]
fn every_divergent_row_has_a_known_gap() {
    let committed = read_tab_rows(&status_path());
    let exclusions = fs::read_to_string(exclusions_path())
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", exclusions_path().display()));
    for (name, token) in &committed {
        if Status::parse(token) != Some(Status::Divergent) {
            continue;
        }
        assert!(
            exclusions.contains(&format!("KNOWN GAP: {name}")),
            "{name} is committed as divergent, which records a wrong answer \
             rather than a missing one. That needs a row reading \
             \"KNOWN GAP: {name}\" in {}, naming it and saying what is wrong.",
            exclusions_path().display()
        );
    }
}

/// The word-boundary rule [`mentions_as_word`] rests on, pinned in both
/// directions. Without the negative case the function could return `true`
/// unconditionally and every caller would still look satisfied.
#[test]
fn a_name_is_only_mentioned_when_it_stands_alone() {
    assert!(mentions_as_word(
        br#"rexx-exec: routine "WORD" is not implemented (4c)"#,
        "WORD"
    ));
    assert!(mentions_as_word(
        b"rexx-exec: ADDRESS is not implemented (4c)",
        "ADDRESS"
    ));
    assert!(!mentions_as_word(
        br#"rexx-exec: routine "WORDS" is not implemented (4c)"#,
        "WORD"
    ));
    assert!(!mentions_as_word(
        br#"rexx-exec: routine "C2X" is not implemented (4c)"#,
        "BITAND"
    ));
    assert!(!mentions_as_word(b"", "LENGTH"));
}
