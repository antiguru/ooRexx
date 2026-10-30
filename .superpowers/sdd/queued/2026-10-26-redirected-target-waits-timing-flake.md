**CLOSED 2026-10-30 by Phase 9d Task 8.8c (d7581cc4a).**

**OPEN, queued 2026-10-26 by the controller; proposed 9d (Task 8.8).**

# collect_stress the_l0_subset flakes on command_redirected_target_waits_elsewhere

Found by Phase 9d Task 8.3 fix round 2 (task-8.3-report.md, concern 4): the program's
`time('e') > 0.9` check failed in 2 of 3 runs at 3e6dc0426 and 3 of 3 at efe8d3f40 under
collect_stress (which programs failed the first runs was not captured). Related to the queued
redirected-target flake row. Open: make the wait observable without a wall-clock threshold.

## Task 8.8c evidence

The shared witness records completion after SysSleep returns and checks the full callback transcript under collection on every allocation. Sleep-duration discrimination now belongs to a separate deterministic virtual-clock control, which reports advancement for the real call and none for its omission. The late-notification regression, semaphore omission and guard-holder controls pass with the ordinary, stress, seeded simulation and oracle transcripts. The program remains in phase-9.txt and the collector inclusion, with the related 2026-10-21 item closed separately.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
