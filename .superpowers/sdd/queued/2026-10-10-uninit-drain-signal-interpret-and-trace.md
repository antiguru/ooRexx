# UNINIT drain misses an INTERPRET left by SIGNAL, and trace lines

The Task 11b re-review (Phase 6.1) found both of these on 2026-10-10.

* **An INTERPRET left by SIGNAL does not drain.** The oracle treats that exit as a normal return and drains there. A loop driven only by `interpret 'signal top'` prints `done 0` at 356 MB in the crate, and `done 998` at 24 MB on the oracle.
* **A trace line's direct write does not drain.** The oracle writes trace output through `.traceoutput` to a native `lineout`, whose return drains. A loop run under `trace r` prints `done 0` at 356 MB in the crate, and `done 998` at 24 MB on the oracle.

See the "## Re-review 1" section (R1 and R2) of `.superpowers/sdd/2026-10-07-phase-6-1/task-11b-review.md`. The probes are under `/tmp/claude-1000/p61/t11br/`.
