# Task 8.3 fix round 1

Review: task-8.3-review.md (spec COMPLIANT, quality CHANGES REQUESTED). Fix:

1. **I1:** add `define` to `PROGRAMS` in rust/bench-programs/callgrind.sh, run the script end to end
   at HEAD against BASE efe8d3f40 (all programs, define included), and replace the report's perf
   section with that run's table. Own deltas within +0.05% libc subtracted, as before.
2. **M1:** correct the report's frame claim to what was built (primitives entered through a
   `NativeTail` begin half), or delete it.
3. **M2:** wrap the two over-width comment lines (executable.rs:596, install.rs:2724) to 100 columns.

The review's queue candidates Q1-Q3 are queued by the controller; not this round. Same rules as the
first dispatch. Records under t8.3/r1/. Append "Fix round 1" to task-8.3-report.md. Commit; send
status, commits, one-line test summary, concerns.
