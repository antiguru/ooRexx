# Task 11 carry-ins (controller)

* **Stale line numbers.** The brief's line numbers predate Tasks 8 to 10. Find each site by name.
  At `f7170918c`:
  * `switch_to` is at `scheduler.rs:996` and `cancel_wait` at `scheduler.rs:1186`.
  * `sleepers.retain` appears at `scheduler.rs:1361` and `:1377`. Decide which one is M11's, by
    reading scout C's mutant table, and record the choice.
  * `wake_for_halt` is at `scheduler.rs:2371` and `object_roots` at `activity.rs:433`. Check
    whether `failed_sends` is still rooted there.
  * The dead-handle no-op is in `environment.rs` `set_native_entry`.

  Re-derive each mutant's site from scout C's description in `scout-c-report.md` (mutant table
  rows M6, M7, M9 to M12, recommendations 8 and 9) and `scout-c-dst.md`.
* **Sim mode as built (Tasks 8 to 10).** The brief's knobs exist:
  * `REXX_SWITCH_MODE=sim:SEED[,policy][,knobs]`, with `fail=wait:K`, `halt@K`, `gc=q` and
    `order=fifo`;
  * the `pre`, `pct` and `uniform` policies;
  * `trace=` and `replay=`.

  Read the formats in `task-9-report.md` and `task-9-fix1-report.md`. The seeded gate from
  Task 10 is `the_seeded_gate` in `tests/concurrency_tests.rs`, and its M11 row under
  `fail=wait:K` already exists (Task 10 Step 5). Reuse that program for Step 1's crate test
  where it fits, and do not duplicate it.
* **Mutants run in the tree, then are restored.** Do each mutation as an uncommitted edit:
  1. run;
  2. restore with your editor, by reversing the exact edit;
  3. confirm `git diff --stat` shows only your intended changes;
  4. then commit.

  Never `git checkout`, `restore`, `stash` or `reset`. `git checkout --` has destroyed
  uncommitted work in this project before.
* **A defect found** (Step 3) is fixed in this task with a crate test using explicit switch points
  and an assertion that the test reaches the defect's site. Report it to the controller first
  (SendMessage to "main") with the evidence, then fix it.
* **The dead-handle write counter** is `cfg(test)` only, and costs nothing in release. Show that
  with a release build that does not contain it: grep the symbol in the binary, or rely on the
  cfg.
