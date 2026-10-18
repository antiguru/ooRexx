**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** probe with stdout closed: lineout 0 READY: here, 1 ERROR:9 on the oracle.

# LINEOUT to a closed stdout answers 0 READY:

Source: 9b Task 9 review M1 (.superpowers/sdd/2026-10-13-phase-9b/task-9-review.md), pre-existing.
With stdout closed (>&-) or a closed pipe, the crate answers lineout 0 / READY: for STDOUT and
HANDLE:1; the oracle's STDOUT answers 1 with ERROR:9 (EBADF) / ERROR:32 (EPIPE). The crate swallows
the write error. Fix: surface the OS error in the stream state and the residual count, with a witness
run under a closed descriptor and a closed pipe on both sides (SIGPIPE disposition matters: check the
oracle's).
