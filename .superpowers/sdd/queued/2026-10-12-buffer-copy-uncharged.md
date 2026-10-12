# A Buffer copy is not charged to the collector

Found 2026-10-12 by the Phase 9a Task 8 re-review (`.superpowers/sdd/2026-10-11-phase-9a/task-8-rereview.md`).
`copy_object` charges a String subclass's text (Task 8 fix round, `object_protocol.rs:661-671`), but a
`Buffer` body (`NativeState::Data`) copied the same way is not charged; the hole predates Task 8.
Witness shape: Task 8's `tests/instance_copy_memory.rs` loop with a large Buffer under `ulimit -v`.
Owner: unassigned.
