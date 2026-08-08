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

//! The 4a exit gate's criterion 4: the named L0 subset -- the union of every
//! `rust/corpus/phase-*.txt` file, which is what [`read_subset`]'s caller
//! reads -- passes again under collect-on-every-allocation, and the mode is
//! proved to do something before its pass is believed.
//!
//! # The mode did not exist before this task
//!
//! `Heap::alloc_with` (as it was named before this task) never collected,
//! and `Heap::collect` had no caller outside `rexx-core`'s own tests --
//! this criterion was written as though the mode already existed, but it
//! had never once run against `rexx-exec`. Task 16 built it:
//! `rexx-core::Heap::alloc_with_uncollected` is the renamed, never-collects
//! primitive (renamed so a *new* allocation site written the natural way
//! announces at the call site that it bypasses the stress hook, rather
//! than silently doing so); `rexx_exec::Interp::alloc_with` (`lib.rs`) is
//! the one production entry point every value/stem constructor now calls,
//! and it collects when [`run_program_collect_every_alloc`] enabled it and
//! does nothing extra otherwise. **This means criterion 4's pass here is
//! the first time this mode has ever run against this crate's rooting
//! discipline, not a re-confirmation of something exercised throughout the
//! phase.** See the gate document for why that matters and what it does
//! and does not prove.
//!
//! # Collect BEFORE the allocation, not after -- and this was not the first
//! design tried
//!
//! `Interp::alloc_with`'s own doc comment has the full account: an earlier
//! version collected immediately *after* allocating, which cannot work --
//! the caller has not had a chance to root the value the allocation is
//! about to return, so every single allocation swept its own result,
//! unconditionally, on every program including ones with no rooting
//! question at stake. Measured at the time: 29 of 29 subset programs
//! panicked, even `say 1`. Collecting *before* the allocation asks the
//! right question instead -- is everything already rooted by an *earlier*
//! call's `push_temp` still reachable now that a *new* allocation is about
//! to happen -- and that is what this file actually exercises.
//!
//! # The negative control
//!
//! **Verified by hand, not by an assertion in this file**, because it
//! means deleting a line of production code, which nothing here should do
//! on every run. With `eval.rs`'s `eval_arithmetic`'s
//! `self.roots.push_temp(left_value);` removed (the call that roots the
//! left operand while the right operand's own evaluation runs and can
//! allocate arbitrarily), **7 of the 29 subset programs panic** under
//! [`run_program_collect_every_alloc`] with `to_text`'s "a live value"
//! (`arith_digits.rex`, `trace_output.rex`, `notation_thresholds.rex`,
//! `number_identity.rex`, `deep_nested_expr.rex`, `trace_results.rex`,
//! `mutation_digits_at_render.rex`) -- rebuilt and re-run against a clean
//! tree afterward, all 29 pass again. A different site, the analogous
//! `push_temp(right_value)` two lines below, turned out **not** to be a
//! useful control at this particular call shape: `right_value` is read
//! exactly once, immediately, by `arith_operand`, with no allocation
//! between its creation and that read, so nothing here ever asks whether
//! it survived -- deleting its root is inert for this reason alone, not
//! because rooting does not matter for it in general. That is why
//! `left_value`'s is the site named as this criterion's control, not
//! `right_value`'s: the criterion asks for a site whose deletion a subset
//! program actually catches, and this is the one that does.
//!
//! # Why comparison is against `run_program`, not the oracle, directly
//!
//! `tests/corpus.rs` (criterion 1) already establishes, byte for byte,
//! that `run_program`'s output matches the oracle for every program named
//! in `phase-4a.txt`. So `stress_output == plain_output` combined with
//! that already-established fact gives `stress_output == oracle_output`
//! transitively, without this file needing its own oracle invocation (and
//! its own `ulimit` wrapper, `LD_LIBRARY_PATH`, missing-binary handling --
//! all of which `corpus.rs` already owns). It is also the more direct
//! question this criterion is actually asking: does turning the mode on
//! change what the interpreter produces, not does the interpreter still
//! agree with a second program.

use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::{run_program, run_program_collect_every_alloc};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program. Duplicated from `corpus.rs`/`coverage.rs` rather
/// than shared: see either file's own module doc for why an integration
/// test cannot `mod` another test binary.
///
/// Takes a slice rather than a single path so that a later phase's own subset
/// file runs *alongside* the earlier ones rather than replacing them -- see
/// `coverage.rs`'s own copy of this function for the fuller argument. The
/// caller below reads every phase subset file the corpus has, which is what
/// puts a phase's own constructs under the collector at all: a program calling
/// a builtin lives only in `phase-4c.txt`, and until it was read here every
/// allocation a builtin makes was outside this harness's reach.
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

/// The subset files this harness reads, in union order.
///
/// A named constant rather than a literal at the call site so that
/// [`the_stress_subset_reads_every_phase_subset_file`] can assert it against
/// the corpus directory itself. Which files a harness reads is not something
/// any other check here can see: `coverage.rs`'s
/// `phase_*_subset_matches_the_committed_list` tests pin each file's
/// **contents**, and every assertion in
/// [`the_l0_subset_passes_again_under_collect_on_every_allocation`] holds just
/// as well over a smaller union. Measured by deleting the subject: with
/// `phase-4c.txt` removed from this list, the whole workspace stays green and
/// byte-identical, and the builtins leave the collector's reach silently.
const SUBSET_FILES: &[&str] = &["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"];

/// The subset programs that allocate nothing, so collect-on-every-allocation
/// has nothing to fire on.
///
/// Every value these four produce is a literal spelling a canonical small
/// integer, and `Interp::literal` inlines those into the handle rather than
/// the heap. `deep_nested_expr.rex` is the clearest case: three thousand
/// terms, all of them the literal `1`, and not one allocation between them.
///
/// A program belongs here because of what it contains, not because it was
/// inconvenient -- see the both-directions assertion at the use site.
const NO_ALLOCATION_PROGRAMS: &[&str] = &[
    "lang/deep_nested_expr.rex",
    "lang/mutation_controlled_order.rex",
    "lang/no_trailing_newline.rex",
    "lang/trace_numeric_request.rex",
];

/// The phase subset files that exist in the corpus directory, sorted.
///
/// Read from the directory rather than listed a second time, so the assertion
/// below cannot be satisfied by a copy of [`SUBSET_FILES`] that was edited in
/// the same change -- and so a subset file added later and forgotten here is
/// red rather than silently unread.
fn phase_subset_files_on_disk() -> Vec<String> {
    let dir = corpus_dir();
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

/// The stress run reads **every** phase subset file the corpus has.
///
/// This is the pin on *which files* the harness reads, which is a different
/// question from what any of them contains. Without it the third entry of
/// [`SUBSET_FILES`] can be dropped -- by an edit, or by a merge -- and nothing
/// in the workspace moves: the run still passes, still reports a non-zero
/// collection count for every program it did read, and still exercises no
/// builtin at all, because no program in `phase-4a.txt` or `phase-4b.txt`
/// calls one.
#[test]
fn the_stress_subset_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the collect-on-every-allocation run does not read every phase subset \
         file in rust/corpus/. A file missing from SUBSET_FILES is a phase \
         whose programs never reach this harness, and nothing else in the \
         workspace can see that -- the run passes over whatever it was given"
    );
}

#[test]
fn the_l0_subset_passes_again_under_collect_on_every_allocation() {
    let corpus_dir = corpus_dir();
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    assert!(
        !subset.is_empty(),
        "the phase subset files named no programs -- that is a corpus defect, \
         not an empty pass"
    );

    let mut total_collections: u64 = 0;
    let mut mismatches = Vec::new();
    let mut zero_collection_programs = Vec::new();

    for rel_path in &subset {
        let abs = corpus_dir.join(rel_path);
        let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
        let path_str = abs
            .to_str()
            .unwrap_or_else(|| panic!("corpus path {} is not valid UTF-8", abs.display()));

        let plain = run_program(path_str, text.clone(), rexx_exec::Invocation::none());
        let stress = run_program_collect_every_alloc(path_str, text, rexx_exec::Invocation::none());

        if stress.collections == 0 {
            zero_collection_programs.push(rel_path.clone());
        }
        total_collections += stress.collections;

        if plain.exit_code != stress.exit_code
            || plain.stdout != stress.stdout
            || plain.stderr != stress.stderr
        {
            mismatches.push(format!(
                "{rel_path}: plain exit={} stdout={:?} stderr={:?}; \
                 stress exit={} stdout={:?} stderr={:?}",
                plain.exit_code,
                String::from_utf8_lossy(&plain.stdout),
                String::from_utf8_lossy(&plain.stderr),
                stress.exit_code,
                String::from_utf8_lossy(&stress.stdout),
                String::from_utf8_lossy(&stress.stderr),
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "the subset diverges under collect-on-every-allocation:\n{}",
        mismatches.join("\n")
    );

    // Anti-vacuity device 1: a mode that never collects would pass the
    // check above by construction (nothing to sweep, nothing to diverge),
    // exactly the defect this criterion was rewritten to close. Checked
    // per program, not only in aggregate, so one silent program cannot
    // hide behind the rest of the subset's counts.
    //
    // The set is committed data rather than an emptiness check, because a
    // program that allocates nothing is a legitimate state: a program whose
    // only values are literals spelling canonical small integers allocates
    // nothing at all, since `Interp::literal` inlines those into the handle
    // instead of the heap.
    //
    // **Both directions.** A program joining this set has had an allocation
    // silently removed; a program leaving it has gained one. Either is a
    // change a human should look at, which a one-directional exemption
    // would not force -- and an exemption nobody can fail is how a
    // shrinking subset goes unnoticed.
    let mut observed: Vec<&str> = zero_collection_programs
        .iter()
        .map(String::as_str)
        .collect();
    observed.sort_unstable();
    let mut expected: Vec<&str> = NO_ALLOCATION_PROGRAMS.to_vec();
    expected.sort_unstable();
    assert_eq!(
        observed, expected,
        "the set of programs performing zero collections under the stress \
         mode has drifted from the committed list; a program that gained an \
         allocation and one that lost one are both worth deciding about"
    );
    assert!(
        total_collections > 0,
        "collect-on-every-allocation performed zero collections across the \
         whole subset -- the mode is a no-op and this criterion cannot tell \
         a real pass from a vacuous one"
    );
}

/// **Fix round 4's NEW-4.** The value a clause resolves to is rooted across
/// the `CALL ON` handler its boundary runs, and *this* is what fails when
/// that rooting goes: the corpus subset above does not contain the shape,
/// which is why deleting the `push_temp` left 970 tests and this whole file
/// green while panicking on `a live value` under a hand-run program.
///
/// The shape is a clause whose value is created *and consumed* across an
/// activation boundary: `return bb() || 'TAIL'`, where `bb` queues the
/// condition. By the time the boundary runs, the concatenation's own
/// one-clause temps frame is already popped, and the handler is a nested
/// activation that allocates -- so the only thing keeping the returned
/// `ObjRef` alive is the boundary's own `push_temp`.
///
/// Three rows because the crate has two `Flow` variants that carry a value
/// and both must be rooted: a `RETURN` at top level, the same inside a `DO`
/// body (a different `run_bounded` drives it), and an `EXIT`.
///
/// **Checked by deleting its subject**, which is the only thing that makes
/// this a test rather than a re-run of the plain interpreter: with
/// `ClauseValue for Flow` returning `None`, all three rows panic on
/// `a live value` at `value.rs`, and the plain (non-stress) run of the same
/// three programs still passes, so nothing but the collector sees it.
#[test]
fn a_clause_value_survives_the_handler_its_boundary_runs() {
    let rows: [(&str, &str, &str); 3] = [
        (
            "return-at-an-activation-boundary",
            "call on user foo name uh\nzmark = 'NOMARK'\nzr = aa()\nsay zr zmark\nexit\naa:\nreturn bb() || 'TAIL'\nbb:\nraise user foo return 'HEAD'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            "HEADTAIL HANDLER-AT 7\n",
        ),
        (
            "the same inside a DO body",
            "call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to 1\nzr = aa()\nend\nsay zr zmark\nexit\naa:\nreturn bb() || 'TAIL'\nbb:\nraise user foo return 'HEAD'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            "HEADTAIL HANDLER-AT 9\n",
        ),
        (
            "EXIT rather than RETURN",
            "call on user foo name uh\nzmark = 'NOMARK'\nsay 'go'\nexit bb() || 'TAIL'\nbb:\nraise user foo return 'HEAD'\nuh:\nzmark = 'HANDLER-AT' sigl\nsay 'handler' zmark\nreturn\n",
            "go\nhandler HANDLER-AT 4\n",
        ),
    ];
    let mut total_collections: u64 = 0;
    for (name, text, expected) in rows {
        let path = format!("<clause-value-rooting: {name}>");
        let stress = run_program_collect_every_alloc(
            &path,
            text.as_bytes().to_vec(),
            rexx_exec::Invocation::none(),
        );
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            expected,
            "{name}, under collect-on-every-allocation"
        );
        // Anti-vacuity: a program that never collects cannot observe a
        // dropped root, so a green row with zero collections would prove
        // nothing at all.
        assert!(
            stress.collections > 0,
            "{name} performed zero collections, so it cannot see a dropped root"
        );
        total_collections += stress.collections;
    }
    assert!(total_collections > 0);
}

/// The command line's argument string survives every allocation the program
/// makes.
///
/// It is an `ObjRef` created before the first clause runs and read by a clause
/// that may be the program's last, and `Interp::call_context` is **not** walked
/// by the collector -- so the only thing keeping it reachable is the
/// `push_temp` `execute` takes before `Interp::run`. Nothing else in the tree
/// can see that: the plain interpreter never collects, and every differential
/// harness runs a program short enough that a swept value would still be
/// sitting in freed-but-untouched memory.
///
/// **Checked by deleting its subject.** With `execute`'s `push_temp(value)`
/// removed, this test panics at `value.rs`'s `a live value` while
/// `tests/input_oracle.rs` and the whole plain suite stay green.
///
/// The program allocates repeatedly before reading the argument, and reads it
/// last, so a collect between the two has somewhere to happen.
#[test]
fn a_command_line_argument_survives_collect_on_every_allocation() {
    let text = "do i = 1 to 20\n  zj = 'filler' i\nend\nparse arg zp\nsay '['zp']'\n";
    let stress = run_program_collect_every_alloc(
        "<argument-rooting>",
        text.as_bytes().to_vec(),
        rexx_exec::Invocation::with_argument(b"the-argument".to_vec()),
    );
    assert_eq!(
        String::from_utf8_lossy(&stress.stdout),
        "[the-argument]\n",
        "the argument string did not survive collect-on-every-allocation"
    );
    // Anti-vacuity: a run that never collected cannot observe a dropped root.
    assert!(
        stress.collections > 0,
        "zero collections, so this row cannot see a dropped root"
    );
}

/// `VALUE`'s compound-write path (`builtin/datatype.rs`) holds the *old*
/// value across the allocation its own write performs.
///
/// `Interp::stem_get`'s "no object at all" branch (`stem.rs`) derives a
/// fresh, slot-less `Body::Text` for the read-before-write answer when the
/// stem has never been touched, and `Interp::stem_set` then allocates that
/// stem's *first* object on the identical branch -- so without rooting the
/// old value first, that second allocation's own pre-sweep can collect the
/// first with nothing left pointing at it.
///
/// **Checked by deleting its subject**, the `push_temp` `value`'s
/// `SymbolKind::CompoundName` arm now takes: with it removed, the fourth
/// row panics at `value.rs`'s `a live value`, and the plain (non-stress)
/// run of the same program still passes, so nothing but the collector
/// sees it. The first three rows are the adjacent successes that pin the
/// defect to *this* write path rather than to compounds, `VALUE`, or
/// reads in general -- a direct compound assignment, a stem already
/// carrying a default, and `VALUE`'s own read-only form all stay green
/// under the mutant.
#[test]
fn values_compound_write_roots_the_old_value_before_the_stems_first_allocation() {
    let rows: [(&str, &str, &str); 4] = [
        (
            "a direct compound assignment, no VALUE involved",
            "j=3\na.j='new'\nsay a.3\n",
            "new\n",
        ),
        (
            "a stem already carrying a default, read directly",
            "s.='d'\nsay s.9\n",
            "d\n",
        ),
        (
            "VALUE's read-only form on the same never-touched compound",
            "j=3\nsay value('a.j')\n",
            "A.3\n",
        ),
        (
            "the failing shape: VALUE's write on a never-touched compound",
            "j=3\nsay value('a.j','new')\n",
            "A.3\n",
        ),
    ];
    let mut total_collections: u64 = 0;
    for (name, text, expected) in rows {
        let path = format!("<value-compound-write-rooting: {name}>");
        let stress = run_program_collect_every_alloc(
            &path,
            text.as_bytes().to_vec(),
            rexx_exec::Invocation::none(),
        );
        assert_eq!(
            String::from_utf8_lossy(&stress.stdout),
            expected,
            "{name}, under collect-on-every-allocation"
        );
        assert!(
            stress.collections > 0,
            "{name} performed zero collections, so it cannot see a dropped root"
        );
        total_collections += stress.collections;
    }
    assert!(total_collections > 0);
}
