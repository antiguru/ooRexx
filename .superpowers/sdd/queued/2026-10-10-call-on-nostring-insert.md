# `call on nostring` reports 25.1 with an unsubstituted insert

The Task 11a reviewer (Phase 6.1) found this on 2026-10-10. It reproduces on base `fbdf615e8` and on `f8d7049e8`.

`call on nostring name h` gives Error 25.1 with `found "&1"`. The oracle gives `found "NOSTRING"`, so the message insert is never substituted. See `.superpowers/sdd/2026-10-07-phase-6-1/task-11a-review.md`, under "Pre-existing, outside the task".
