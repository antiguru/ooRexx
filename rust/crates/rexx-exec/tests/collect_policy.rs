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

//! What makes the collector fire, on an **ordinary** run.
//!
//! `collect_stress.rs` next door is about the opposite mode -- collect on
//! every allocation, to find a missed root. Nothing there can see the
//! production trigger at all, because that mode overrides it: every corpus
//! program is far too small to reach the growth allowance, so under
//! `run_program` they collect zero times and pass either way.
//!
//! # Why this file exists rather than a paragraph in a doc comment
//!
//! `Interp::collect_at` is a policy, and the reason it is *this* policy is a
//! measurement on a program shape no benchmark axis has: a large live set
//! that dies, followed by a long tail of short-lived values. A watermark on
//! the live count alone goes on collecting there while thousands of swept
//! slots sit unused, because the live count has fallen back to the floor.
//! Adding `Heap::will_grow` to the condition is what stops it.
//!
//! That reason is prose everywhere else, and prose about a number rots. The
//! assertion below is the one form of it that gets re-run.

use rexx_exec::{Invocation, run_program};

/// A program with a large live set, dropped, then a long tail of churn.
///
/// The tails are `'v' || i` rather than `i` because a small integer is a
/// tagged immediate and never reaches the heap at all -- a stem of 200,000 of
/// those would occupy one slot for the stem and none for its contents, and
/// this file would be measuring nothing.
const TRANSIENT_THEN_CHURN: &str = "\
s. = 0
do i = 1 to 200000
  s.i = 'v' || i
end
drop s.
do 1000000
  yy = 'abc'
end
say 'ok' yy
";

/// The same program with the `DROP` removed, so the live set stays live.
///
/// The adjacent case, and it is what pins the assertion below to "swept slots
/// are going unused" rather than to "the program is large". Here the arena
/// has no reusable slots -- everything in it is reachable -- so the two
/// conditions coincide and the collection count is whatever the growth
/// allowance dictates, on any policy.
const LIVE_STAYS_LIVE: &str = "\
s. = 0
do i = 1 to 200000
  s.i = 'v' || i
end
do 1000000
  yy = 'abc'
end
say 'ok' yy s.199999
";

fn run(text: &str) -> rexx_exec::Outcome {
    run_program(
        "<collect-policy>",
        text.as_bytes().to_vec(),
        Invocation::none(),
    )
}

/// A collector that waits for the free list to run out does far less work
/// after a transient than one that watches the live count.
///
/// **The bound is between two measured values, not a target.** With
/// `Heap::will_grow` in the condition this program collects **8** times; with
/// the live count alone it collects **19**, for the same peak resident set
/// (60,656 KB against 60,860 KB) and the same output. 12 sits between them,
/// so the assertion fails for the policy this one replaced and has room for
/// the ordinary drift of a few collections either way.
///
/// **It is not an emptiness check**: the lower bound is asserted too, because
/// a trigger that stopped firing entirely would satisfy an upper bound alone
/// and would be the defect this whole area exists to fix.
#[test]
fn a_dead_transients_slots_are_reused_before_the_collector_runs_again() {
    let outcome = run(TRANSIENT_THEN_CHURN);
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stdout), "ok abc\n");
    assert!(
        outcome.collections > 0,
        "the program allocates 1.2 million values and collected nothing, so \
         the trigger is not firing at all"
    );
    assert!(
        outcome.collections <= 12,
        "collected {} times after the transient died. A collector that fires \
         on the live count alone reads about 19 here, because it collects \
         while the slots the transient freed are still unused; one that waits \
         for the arena to need to grow reads about 8",
        outcome.collections
    );
}

/// The adjacent case: with the live set still live, there are no free slots
/// to wait for and the count is the growth allowance's, not the free list's.
///
/// This is the case that must **not** be quiet. A policy change that made the
/// collector fire less often everywhere would pass the test above and fail
/// here, which is what stops that test being satisfied by simply collecting
/// less.
#[test]
fn a_live_set_that_does_not_die_still_collects_as_it_grows() {
    let outcome = run(LIVE_STAYS_LIVE);
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "ok abc v199999\n",
        "the stem must still be readable, which is what says the collector \
         did not sweep a live tail"
    );
    assert!(
        outcome.collections >= 8,
        "collected only {} times while the arena grew past 1.2 million \
         allocations with a 200,000-entry stem live throughout",
        outcome.collections
    );
}
