**NOT COPIED, ruled by the controller 2026-10-19 under Moritz's standing rule that oracle bugs are never copied.** Upstream report only; no suite test depends on either shape.

# Upstream: refused inserts that change their receiver

These are for the ooRexx project, not for this crate. Found 2026-10-19 by Phase 9d Task 6.7
(`task-6.7-report.md`, Concerns).
* **A refused `Queue~insert` extends the queue.** `insertRexx` validates under `IndexUpdate`, which
  extends the array to the position before `checkInsertIndex` refuses it. After a failed
  `insert('a', 99)` on an empty queue, `put('m', 50)` raises 93.966 on the oracle, where a fresh
  queue raises 93.918 (`docs/superpowers/records/2026-10-18-phase-9d/t6.7/probes/queue-insert-extends/`).
* **An Array index on an empty, unfixed Array.** `.Array~new~insert('a', .array~of(1,2))` answers
  3 on the oracle and makes the receiver two-dimensional, a shape `insert` itself refuses.
  `.Array~new~section(.array~of(1,2))` reads a start position `validateIndex` never set
  (`t6.7/probes/index-outside-cluster/q2.diff`, its last two lines). The crate raises 93.907 for both.
