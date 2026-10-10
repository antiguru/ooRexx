# A failed pool-thread spawn still runs a blocking call inline

The Task 11b review (Phase 6.1) found this on 2026-10-10. It predates the task.

When the native pool cannot spawn a thread, a blocking command or native still runs inline on the baton and can hang the process. Under `ulimit -v` 3 GB the scout's full-pool probe is killed at rc 137, where the oracle raises Error 48.1. Each pool thread reserves 512 MB of stack, so the address-space limit is reached quickly. Task 11b also grows the pool with no cap: 1000 concurrent commands ran with 2003 threads and 545 GB reserved address space. See `.superpowers/sdd/2026-10-07-phase-6-1/task-11b-review.md` (M2 and the answer to question (c)).
