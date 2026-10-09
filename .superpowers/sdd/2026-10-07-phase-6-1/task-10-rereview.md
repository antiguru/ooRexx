# Task 10 fix round 1: re-review

Range `11d768777..6ac9a8c1a`. Read-only; nothing run. Lines at `705dfb767`, under `rust/crates/rexx-exec/`.

### Finding Verdicts

* Important 1 (early end): **ADDRESSED**. `judge` calls `early_end` (`tests/concurrency_tests.rs:4171-4176`, fn `:4223`),
  kind `Abort`. Reds an unlisted refusal, a missing ooTest summary (not under `pct`), and a status no oracle
  outcome or `DIFFERING` key has when no failing test is named (programs: always). Section1's refusal passes
  through its `DIFFERING` normal key, not a wildcard. Self-test below.
* Minor 1 (false doc comment, `group_runner.rs:536-538`): **ADDRESSED**. Sentence deleted and replaced by a true
  one. The leak stays unfixed in code, as the review allowed (bounded, red path only).
* Minor 2 (WALL_CLOCK rerun, `concurrency_tests.rs` `run_unit`, `Row::real_clock`): **ADDRESSED**. A passing rerun
  on a row without `clock=real` is a `determinism` red. The check reads the exact knob (`split(',')`, equality).
* Minor 3 (scratch, `unit_dir`): **ADDRESSED**. Profile in the dir name, full path in the replay line.
* Minor 4 (`REXX_SIM_*` passthrough): **ADDRESSED**. Dropped in `crate_environment` and the program path. The
  justification (a sim child gets its variables from `command.rs`) matches the review's own reading of
  `sim.rs:1256-1260`.
* Minor 5 (stdin writer): **ADDRESSED**. Writer thread, `group_runner.rs:590-597`. The thread is detached and the
  pipe closes on the group kill, so it ends.
* Minor 6 (N1/M11 comments): **ADDRESSED**. Header comments added.

### New Breakage in the Fix Diff

None Critical or Important. Two doubts, both Minor:

1. (a) Refusal acceptance is a substring match, `keys.iter().any(|key| key.contains(line))`
   (`:4231`-ish in `early_end`). `line` is a whole `rexx-exec: ...` stderr line and a key is
   `<refusal-or-summary>, rc N, last started ..., failing [...]`, so a different refusal passes only if its full
   text is a substring of a listed key. That means a refusal that is a strict prefix of a listed one (e.g. the
   same message without a trailing `(Phase 9)`), or text that appears inside the `failing [..]` / `last started`
   part. Narrow in practice, not a hole of the kind Important 1 was. Tightening is cheap:
   `key.starts_with(line) && key[line.len()..].starts_with(", rc ")`. The refusal branch also never checks the
   status; an accepted refusal at an unlisted rc passes.
2. A design-limit refusal whose oracle twin hangs now returns no reds without reaching the sim-line check
   (`:4163`). Acceptable (that is what the previous `twin_hangs` guard meant), noting it skips `NoReport`.

(b) The self-test `a_run_ending_where_no_listed_outcome_ends_is_red` (`:4877`) calls `judge` itself with
`Some(&seen)`, the function and argument shape the gate uses, and uses the real `DIFFERING` table (Section1).
It covers rc 1 with a pass summary (red; green at rc 0 and under `pct`), empty stdout, the listed refusal (green),
and an unlisted refusal in three contexts. Not covered: the "program" status route and the rerun determinism
red. The report records that disabling the call makes it fail, which I did not rerun.

### Out-of-Scope Observations

None.

### Verdict

**Fix round:** All findings addressed, no new Critical/Important breakage

Open (Minor, optional): refusal match in `early_end` is `contains`, not an exact refusal-field match; no self-test
for the program-status abort route or the determinism red.
