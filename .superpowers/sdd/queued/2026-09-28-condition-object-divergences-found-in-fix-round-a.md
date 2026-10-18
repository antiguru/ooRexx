**OPEN: ruled 9d, cluster call-on-trap-argument (Phase 9d inventory).** callarg still 1 1 against 0 0 (the cluster); coorder still lacks RESULT and ADDITIONAL, which stream-notready-names touches.

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
