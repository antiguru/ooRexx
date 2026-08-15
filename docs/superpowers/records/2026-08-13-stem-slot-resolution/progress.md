# SDD ledger -- plan: docs/superpowers/plans/2026-08-13-stem-slot-resolution.md

## Task 1: the stem's slot -- COMPLETE, PLAN CLOSED
5eaa3c8e8 the change, 73d470474 entry 30, 4bfabd00f + 0e3659c8c the comment corrections. 1481 passed, 0 failed.
Review: spec PASS, quality PASS WITH CHANGES, all in comments. Everything the implementer reported was reproduced independently, including the tripwire's opposite result -- NOT an artifact: a wrong tail-piece slot always changes a printed byte, while a wrong stem slot often lands on an empty slot that auto-vivifies to the same derived name.
Measured: compound -9.88%, alloc4c -3.16%, rexxcps -2.17% instructions. Hashing bucket on rexxcps 7.7% -> 5.3%, ranges non-overlapping on the bucket and on each of its three largest members.
The extra question came back YES a third time, by running: `do za.zi = 1 to 3` grows ZA., `interpret "zq.1 = 7"` grows ZQ., `zn='ZR.1'; drop (zn)` grows ZR., with a control that resolves through the plan.

Controller verification of the final round, done by me rather than a re-review: entry 31 appended at 40 insertions / 0 deletions, control_slot's doc now states the true relation, the cardinality is gone, tree clean. A comments-only round that produced its own closed set did not warrant another opus pass.

FILED, NOT TAKEN: f431ffc05, the bare stem's own slot -- three sites, one of which (bind_control's stem arm) pays per ITERATION of a stem-controlled DO. Found by the review, not the task. No bench axis can see it, so step 1 of that task is a workload.
