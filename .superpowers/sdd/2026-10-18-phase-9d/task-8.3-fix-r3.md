# Task 8.3 fix round 3 (last; two exact fixes)

Re-review r2 (task-8.3-rereview-r2.md) APPROVED round 2 with two Minors. Controller ruling: both are
small and exact, so one bounded round fixes them; the controller verifies by running and the
phase's final whole-branch review covers the code.

1. **M1:** `prune_program_rows` (rust/crates/rexx-exec/src/lib.rs, about 3589-3599) drops a dead
   program's chunks without subtracting their holds from `site_hold_count`, so the count only grows
   (and could wrap in release). In its retain, also collect `chunk.site_holds()` and subtract their
   count, so the count is exact; make `lib.rs:1387-1388`'s doc true. Witness: a unit test that runs
   a program which dies holding sites and asserts the count returns to the live holds (red without
   the fix). Use checked or saturating arithmetic only if the count can legitimately be exceeded;
   otherwise a debug_assert that it never goes below zero.
2. **M2:** add the re-review's `interp2` shape (section 1 of the verdict) as a collect_stress form,
   shown red at 9963948e3 and green at HEAD, and correct concern 1 in the report's "Fix round 2".

Checks: fmt, clippy, --lib, collect_stress, signals, the gated corpus (the DST program
rexxutil_file_date_time.rex is a known base failure, queued), and callgrind.sh on nop interpret
intcall against efe8d3f40 (own delta at most +0.05% libc subtracted). Records under t8.3/r3/. Append
"Fix round 3" to task-8.3-report.md. Commit; send status, commits, one-line test summary, concerns.
