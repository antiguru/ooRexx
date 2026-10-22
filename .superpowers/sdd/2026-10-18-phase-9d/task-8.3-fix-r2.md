# Task 8.3 fix round 2

Re-review r1 (task-8.3-rereview-r1.md) APPROVED round 1, with one Minor and one queue candidate.
Controller ruling: Q-A is in this task's area (call-site keeps) and is a collector defect whose
outcome can be a wrong body once a freed slot is reused, so it is fixed here, not queued.

1. **Q-A:** a call site's kept routine is not traced by the collector. Probe in the re-review,
   section "Queue candidates" (Q-A): the oracle prints `f 1 / f 2 / f 3`, rc 0; the crate fails
   with "an activation's body selector names no routine body", rc 120, at HEAD, efe8d3f40 and
   eeab7f128. Trace (or root) whatever a call-site slot holds for as long as the slot holds it, in
   both engines and in the main program, internal-function, and CALL forms. Witness: the probe as a
   corpus program (oracle-identical) plus a collect_stress program, each shown red at 9963948e3.
   Check that a dropped chunk (an INTERPRET string, a routine no longer reachable) releases its
   slots: no growth over 100000 iterations of the re-review's g1.rex shapes.
2. **M1:** restore a tripwire for a keep left unclaimed: a debug_assert at the filing in
   `site_call` (run/call.rs, where `entered_keep`'s assert was), as the re-review describes.

Perf check: callgrind.sh on every program (no -p) against efe8d3f40, own delta at most +0.05%
libc subtracted (task-8-batches.md header). Same rules otherwise. Records under t8.3/r2/. Append
"Fix round 2" to task-8.3-report.md. Commit; send status, commits, one-line test summary, concerns.
