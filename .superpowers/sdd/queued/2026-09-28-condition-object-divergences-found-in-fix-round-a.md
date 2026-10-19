**CLOSED 2026-10-19 by Phase 9d Task 6.9 (94b09da09).** `queued-probes/callarg` and `queued-probes/coorder` are identical to the oracle on stdout, stderr and rc (`docs/superpowers/records/2026-10-18-phase-9d/t6.9/queued/`).

# Condition-object divergences found by Task 8 fix round A, not fixed (owner: not assigned)

Both pre-exist `2b26b6970`; neither is in `phase-4-exclusions.txt`. Probes in the fix round's
scratch (`fix-s8a/p/`), each from a fresh empty directory.

1. **A NOTREADY condition object's entries.** `call on notready name h3` then
   `x = linein('/nonexistent/zz')`, the handler printing `condition('O')~allIndexes`: the oracle's
   object has `RESULT` and `ADDITIONAL` and no `POSITION`; this crate's has `POSITION` and neither
   of the others (`coorder2.rex`, line 2 of its output). Same at HEAD's binary.
2. **A CALL ON handler's argument.** `call on error name h; 'exit 3'` with `h: say arg()
   arg(1, 'E')`: oracle `1 1`, here `0 0` (`callarg.rex`). `RexxActivation::internalCallTrap`
   (`execution/RexxActivation.cpp:3329-3343`) passes the condition object itself as argument 1.
   If it is implemented, the object handed over must be the store-backed form a program can use:
   a trapped condition's object is a native `Directory` until `condition_copy` copies it.

Also: `.context~condition` still refuses (refusal-sites `builtin_option_object`, `diverges`),
where `Interp::condition_copy` now builds exactly the copy `RexxContext::getCondition` answers.
