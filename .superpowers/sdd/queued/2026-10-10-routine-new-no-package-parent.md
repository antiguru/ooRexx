# Routine~new without directives cannot find the caller's routines

Found by Task 11a's implementer (Phase 6.1) on 2026-10-10. It is outside Task 11a's items, and it reproduces at `fbdf615e8`.

Probe (`/tmp/claude-1000/p61/t11a/probes/s6/ctx3.rex`; `ctx2.rex` is a variant of it):

```rexx
r = .routine~new('g', 'return helper()')
say r[]
::routine helper public
return 'helper found'
```

Results:

* The oracle prints `helper found` and exits with rc 0.
* The crate exits with rc 213 and reports `Error 43.1: Could not find routine "HELPER"`. The traceback is `1 *-* return helper()`, `Compiled method "[]" with scope "Routine"`.
* The result is the same without `public`, and when `.context~package` is passed as the context argument.

The suspected cause is `record_compiled_routine` in `install.rs` (about line 1770 at `fbdf615e8`). It records a fresh program with no package parent. Only the directives path goes through `install_executable` with a parent. This has not been verified yet. The fix should state the oracle's rule for which package a source-text routine inherits, and test it with and without directives and with the context argument.
