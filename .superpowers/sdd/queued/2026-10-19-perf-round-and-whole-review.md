**RULED: later, by Moritz 2026-10-19.** A targeted performance round, and a manual whole-implementation review.

# A performance round and a whole-implementation review

Moritz, 2026-10-19, accepting rexxcps at +0.53% against the Phase 9 pin (libc and ld-linux
subtracted) at the Phase 9d close: "We'll need a targeted performance round eventually to improve
it again. But then we also need to do a (manual) whole-implementation review at some point, so
it'll come." Inputs for the performance round include the drift recorded per task in the Phase 9d
records and `.superpowers/sdd/queued/2026-10-19-drive-rs-macros.md`.

Moritz, 2026-10-20, as an FYI: cREXX's benchmarks,
https://github.com/adesutherland/CREXX/tree/develop/tests/benchmarks (Are-We-Fast-Yet ports,
RexxCPS level B, JSON, base64, PARSE). The top-level files are cREXX syntax; `cross-runtime/`
reportedly holds ooRexx versions. Check their licences (AWFY, SOM MIT, RexxCPS CPL-1.0) before
copying any into this repository.

The performance round also owns define-benchmark m5 from [the 9b1 review minors](2026-10-15-9b1-task-8b-review-minors.md). Task 8.3 measured `free_unheld_behaviours` dictionary traversal at 7.38% inclusive and rejected a sorted-list alternative that measured 7.33%. Those historical measurements precede `a2a4f78b7`, which changed the benchmark to `n = 3000`, and must be reproduced against the selected baseline before judging an optimization. The benchmark is already registered, and [the residual audit](../2026-10-18-phase-9d/task-8.8-residual-inventory-audit.md) links the measured source and evidence.
