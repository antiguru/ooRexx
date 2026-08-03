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

//! The 4a exit gate's criterion 4: the named L0 subset (`rust/corpus/phase-4a.txt`)
//! passes again under collect-on-every-allocation, and the mode is proved to
//! do something before its pass is believed.
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
/// **Task 0's Step 4.** Was a single-file reader (`&Path`); widened to `&[&Path]`
/// so a later task's own subset file can run *alongside* `phase-4a.txt`
/// rather than replacing it -- see `coverage.rs`'s own copy of this function
/// for the fuller argument. Today's caller passes a one-element slice
/// containing only `phase-4a.txt`, so the union is that file's own content
/// unchanged.
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

#[test]
fn the_l0_subset_passes_again_under_collect_on_every_allocation() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-4a.txt")]);
    assert!(
        !subset.is_empty(),
        "phase-4a.txt named no programs -- that is a corpus defect, not an \
         empty pass"
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

        let plain = run_program(path_str, text.clone());
        let stress = run_program_collect_every_alloc(path_str, text);

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
    // hide behind the other 28's counts.
    assert!(
        zero_collection_programs.is_empty(),
        "these programs performed zero collections under the stress mode, \
         which the aggregate total alone would not have caught: {}",
        zero_collection_programs.join(", ")
    );
    assert!(
        total_collections > 0,
        "collect-on-every-allocation performed zero collections across the \
         whole subset -- the mode is a no-op and this criterion cannot tell \
         a real pass from a vacuous one"
    );
}
