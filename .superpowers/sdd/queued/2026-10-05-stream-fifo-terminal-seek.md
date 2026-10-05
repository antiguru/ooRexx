# Named FIFOs and terminals do not work through the stream class

Found by the Phase 6 S2-S5 Task 21 re-review 3 (F-2). Pre-existing, outside Phase 6.

`read_from` and `write_at` (`dispatch/stream.rs:684-697`, `:1314-1317`) seek before every read and
write; a FIFO or terminal answers ESPIPE. A read answers nothing at once (`x=[]`, `ERROR:0`), a write
fails `ERROR:29 Illegal seek`, and `stream('f', 'c', 'open read')` on a FIFO is Error 91.999. The
oracle reads and writes both (`charin('f',,3)` with a writer at 0.3 s: `x=[abc]`, `READY:`).

When fixed: the read's `File::read` will meet EINTR (SA_RESTART is off since Task 21). Answer
`ERROR:4` and let the halt run, as the oracle does; do not add a retry. A halt does not end such a
read (DEVIATIONS 12) unless P60 is extended. Full scenario: `.superpowers/sdd/2026-10-01-phase-6-s2-s5/task-21-rereview-3.md` F-2.
