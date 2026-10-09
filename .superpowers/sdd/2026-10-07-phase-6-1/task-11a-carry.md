# Task 11a carry-ins (controller)

* **Step 0: Task 11's review Minors.** These are test-only; see `task-11-review.md`.
  * In `src/scheduler/tests/mutants.rs`, make M9's test check the halt-dependent specs
    (`pre:2,k=40`, `uniform:0.3`) before the first failure stops the loop, or collect every
    failure before asserting. The kill must come through the halt route. Correct the M9 row in
    `task-11-report.md` to match.
  * Change M7's comment to say the one-clause comparator is needed for an odd pinned-visit count.
  * Make the dead-handle counter use `heap.peek` instead of `heap.get`.
* **Sites and probes.** Each item's probe text, both engines' output and its crate site at
  `f7170918c` are in `scout-e1-report.md` (items 2, 3, 4) and `scout-e2-report.md` (items 7, 8, 9,
  P1, P2, P3). Find each site by name, because the line numbers have moved since.
* **Heapshape round (just before you).** `dispatch/buffer.rs` and the heap's byte accounting
  changed. Live body bytes are now a running figure. Any growth or shrink you write in Steps 5, 7
  and 8 (MutableBuffer, `Array` and `List` slots, COPIES output) must keep the charge in step with
  the bytes held: use the same charge path the heapshape round uses, never `get_mut` whole-body
  replacement. The debug assertion in `Heap::collect` checks this; a test that reaches a
  collection after your change proves it.
* **R11's deviation row** (Step 4): add a row next to Deviation 28 in the same file, in the same
  style. The new refusal is a `Loud` constructor, so re-derive `refusal-sites.tsv` and keep
  `refusal-dispositions.tsv` green.
* **Step 5 widened by the scout.** Message `START` and `REPLY` refuse the same way as `SEND`. A
  MutableBuffer subclass whose INIT does not forward still answers normally: leave it, and record
  it.
* **Step 6.** If the interpreter half lands, the `EXEMPT` rows' `unblocked_by: "Phase 9"` go
  stale. Remove those rows, or re-point them.
* **Commit per step**, each with its test, so the review can read the steps one at a time. The
  per-task check now includes `cargo clippy -p rexx-exec --all-targets --features pinning,sharing
  -- -D warnings`.
* **Defects found beyond the items:** message the controller (SendMessage to "main") with the
  evidence before widening scope.
