# Scout B: the silent-wrong-answer items of 6.1

Read `scout-common.md` beside this file first. Your name for paths: `sb`.

Items (each `.superpowers/sdd/queued/<name>.md`):
- `2026-10-02-elapsed-clock-per-routine-and-reset`, `2026-10-07-setlocal-scope` (and the RANDOM seed): the
  Activity vs activation scope class. Memory note: the clock, RANDOM seed and SETLOCAL list sit on
  `Activity`, where the oracle keeps them per activation. P89 (`ir/drive.rs` `split_level`, ~2984) copies
  them into a REPLY continuation.
- `2026-10-07-debug-pause-skipped-after-flowed-clause`, `2026-10-07-debug-input-does-not-reach-pauses`.
- `2026-10-07-interpret-syntax-traceback-line`.

For each:
1. Re-run the item's probes at HEAD against ours and the oracle; record the output (one run each,
   5 for anything concurrent).
2. Find the oracle's mechanism in the C++ source (file:line) and the exact observable rule. For the
   scope class: list EVERY field of `Activity` (`rexx-exec/src/activity.rs` and wherever it is defined)
   and classify each as activity-scoped or activation-scoped in the oracle (cite `RexxActivity.hpp` /
   `RexxActivation.hpp` / `ActivationSettings`). Name any field beyond the three already known that sits
   on the wrong side, with a probe that shows it.
3. Locate where an activation's state lives in our IR engine (frames, `ir/drive.rs`) and sketch the
   change: which struct gains the fields, where they are initialized for a new activation (internal
   call vs routine vs method vs external program vs INTERPRET vs started activity vs REPLY continuation),
   and what inherits from the caller (the oracle: internal calls inherit settings; routines and
   methods start fresh; check each). Files and size estimate (S/M/L as lines <50, <300, more).
4. Performance risk: does the change put work on the per-clause or per-call hot path? Name the path.
5. ooTest rows each fix unlocks (`docs/superpowers/records/2026-10-01-phase-6-s2-s5/whole-groups/table.txt`,
   `alone.txt`).

Write `.superpowers/sdd/2026-10-07-phase-6-1/scout-b-report.md`: per item: probe results, oracle rule
with citation, fix sketch, risk, rows. Then the full Activity field table. Probes in an appendix.
