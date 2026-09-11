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

use rexx_exec::{Invocation, run_program};

/// A program with a large live set, dropped, then a long tail of churn.
const TRANSIENT_THEN_CHURN: &str = "\
s. = 0
do i = 1 to 200000
  s.i = 'vvvvvvvv' || i
end
drop s.
do j = 1 to 1000000
  yy = 'abcdefghij' || j
end
say 'ok' yy
";

/// The same program with the `DROP` removed, so the live set stays live.
const LIVE_STAYS_LIVE: &str = "\
s. = 0
do i = 1 to 200000
  s.i = 'vvvvvvvv' || i
end
do j = 1 to 1000000
  yy = 'abcdefghij' || j
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
#[test]
fn a_dead_transients_slots_are_reused_before_the_collector_runs_again() {
    let outcome = run(TRANSIENT_THEN_CHURN);
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "ok abcdefghij1000000\n"
    );
    assert!(
        outcome.collections > 0,
        "the program allocates over a million values and collected nothing, so \
         the trigger is not firing at all"
    );
    assert!(
        outcome.collections <= 12,
        "collected {} times after the transient died. A collector that fires \
         on the live count alone reads 17 here, because it collects while the \
         slots the transient freed are still unused; one that waits for the \
         arena to need to grow reads 6",
        outcome.collections
    );
}

/// The adjacent case: with the live set still live, there are no free slots
/// to wait for and the count is the growth allowance's, not the free list's.
#[test]
fn a_live_set_that_does_not_die_still_collects_as_it_grows() {
    let outcome = run(LIVE_STAYS_LIVE);
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "ok abcdefghij1000000 vvvvvvvv199999\n",
        "the stem must still be readable, which is what says the collector \
         did not sweep a live tail"
    );
    assert!(
        outcome.collections >= 5,
        "collected only {} times while the arena grew past a million \
         allocations with a 200,000-entry stem live throughout, where both \
         policies measure 7",
        outcome.collections
    );
}
