# 2026-10-07-setlocal-scope

Found by the Phase 6 final review (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/final-review.md` M1).

`SETLOCAL`'s list is `Activity::locals` (`activity.rs:33`). The oracle keeps it on the top-level-call
activation (`RexxActivation::pushEnvironment`, `RexxActivation.cpp:4640-4657`) and restores it when
that activation ends (`:1494-1500`); REPLY migrates the activation.

- One activity: a method calls `setlocal()` and returns; main's `endlocal()`: oracle `0`, the
  environment restored; ours `1`, not restored.
- Across REPLY: a method calls `setlocal()`, sets `P6X`, replies, then `endlocal()`; main later
  `endlocal()`. Oracle `cont endlocal 1`, `main endlocal 0`; ours the reverse (3/3 each).

Moving the list to the top-level-call activation fixes both.
