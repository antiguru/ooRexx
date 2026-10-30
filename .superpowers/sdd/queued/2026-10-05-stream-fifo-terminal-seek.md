**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).** FIFO read, terminal and closing-LINEOUT subparts are verified under the recorded rulings.

# Named FIFOs and terminals do not work through the stream class

Found by the Phase 6 S2-S5 Task 21 re-review 3 (F-2). Pre-existing, outside Phase 6.

`read_from` and `write_at` (`dispatch/stream.rs:684-697`, `:1314-1317`) seek before every read and
write; a FIFO or terminal answers ESPIPE. A read answers nothing at once (`x=[]`, `ERROR:0`), a write
fails `ERROR:29 Illegal seek`, and `stream('f', 'c', 'open read')` on a FIFO is Error 91.999. The
oracle reads and writes both (`charin('f',,3)` with a writer at 0.3 s: `x=[abc]`, `READY:`).

When fixed: the read's `File::read` will meet EINTR (SA_RESTART is off since Task 21). Answer
`ERROR:4` and let the halt run, as the oracle does; do not add a retry. A halt does not end such a
read (DEVIATIONS 12) unless P60 is extended. Full scenario: `.superpowers/sdd/2026-10-01-phase-6-s2-s5/task-21-rereview-3.md` F-2.


## Task 8.8e1 evidence

The bounded PTY control reads abc and writes xyz with matching descriptors and terminal bytes. Closing LINEOUT differs with or without a state query, and through the Stream method itself. The oracle syscall trace shows ESPIPE followed by a one-byte read that removes h from hello before close. This is an error-path side effect, not evidence that LINEOUT should consume FIFO data, and Deviation 46 is not broadened. See `task-8.8e1-report.md` for the retained EINTR diagnostic and controller handoff.

## Task 8.8f evidence

Named transient reads propagate EINTR and discard accumulated partial data from the failed operation. Owned blocked-thread traces verify ERROR:4, no retry or writer unblock, and NOTREADY before HALT. HALT placement relative to the outer read-result print varies with timer posting, preserving the existing clause-boundary policy and Deviation 12. Closing LINEOUT preserves bytes and closes normally under approved Deviation 82, and the raw PTY control agrees. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
