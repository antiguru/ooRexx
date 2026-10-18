# Phase 9d Task 4 review: `json`, `yaml` and `dateparser`

Reviewed commits `b6f6cd64c` (fix, witnesses, rows) and `2f626a5a5` (records, report). All probes ran
in a detached worktree at `2f626a5a5`, release builds, crate under `memcap 2G` with `timeout 20`,
oracle under the standard wrapper from fresh empty directories.

## Spec compliance: compliant

* Step 1: two pure-Rexx witnesses, one per refusal text, listed in `rust/corpus/phase-9.txt`.
  Both are rc 120 with the two refusal texts on a base-equivalent build, and oracle-identical
  (stdout, stderr, rc) on the fix.
* Step 2: the fix is in the driver (`rust/crates/rexx-exec/src/run/loops.rs:1858-1905`); the
  compiler is untouched, which matches the root cause. The `op_not_driven` GUARD row in
  `rust/corpus/refusal-dispositions.tsv` is reworded, not deleted, which the brief allows because
  `Loud::op_not_driven` has other sites.
* Step 3: `phase_10_group` answers `None` under the three directories
  (`tests/support/suite_rows.rs:277-295`), backed by global constraint 2. The fixture's json row
  moved to `FAILING`, and `extensions/rxmath/RxMath TEST_NAN` took its `PASSING` place and is a
  real row of `phase-9-exclusions.txt`. The exclusions file lost 453 rows: 31 DateFormatter,
  112 DateParser, 33 json, 227 json_02, 50 yaml, which are the group sizes the report's table
  gives. No row under the three directories remains (`grep -cE` answers 0).
* Step 4: groups run, `yaml.cls` hand run, perf check, all recorded.

## Mutation claim: confirmed

Each arm restored to its base line by exact edit, rebuilt, and both witnesses compared with the
oracle:

| build | `loop_test_callee_loop.rex` | `loop_test_callee_loop_chunk.rex` |
|---|---|---|
| base (all three arms restored; diff vs `671fd0489` is comments and the unused helper only) | DIFF rc 120 | DIFF rc 120 |
| A: `Body` assigns `Some(top)` | DIFF rc 120 | SAME |
| B: `Done` pops unconditionally | DIFF rc 120 | DIFF rc 120 |
| C: `Err` does not cover | DIFF rc 120 | SAME |
| fix | SAME | SAME |

This is the report's table exactly.

## Defect-class hunt: no new defect

Seventeen probes, each a stepping loop inside an enclosing loop whose `WHILE`/`UNTIL` test reaches
a loop in other code. All are oracle-identical on the fix. All but one (`p13`) are rc 120 on the
base build, so they reach the defect.

| probe | shape | base | fix |
|---|---|---|---|
| p01 | callee loop ends by `LEAVE` | DIFF | SAME |
| p02 | callee loop ends by `SIGNAL` out of it | DIFF | SAME |
| p03 | `RETURN` from inside two nested callee loops | DIFF | SAME |
| p04 | error inside callee loop trapped by `SIGNAL ON SYNTAX` in the callee | DIFF | SAME |
| p05 | `CALL ON NOTREADY` in the caller, handler loops, raised in the test's callee | DIFF | SAME |
| p06 | callee runs three nested loops, one `DO OVER` | DIFF | SAME |
| p07 | `DO UNTIL g()` enclosing `DO i = 1 TO 5 WHILE f()`, `g` itself runs a `DO ... WHILE` | DIFF | SAME |
| p08 | `DO OVER ... WHILE` and `DO WITH INDEX ITEM OVER ... UNTIL` stepping | DIFF | SAME |
| p09 | callee loops inside `INTERPRET` | DIFF | SAME |
| p10 | method rather than routine, `DO OVER` inside | DIFF | SAME |
| p11 | callee `SIGNAL`s to a label holding `ITERATE` (oracle 28.2, trapped by the caller) | DIFF | SAME |
| p12 | callee's own loop has a `WHILE` that calls a looping routine (recursion of the case) | DIFF | SAME |
| p13 | error from the test caught by `SIGNAL ON SYNTAX` three loops out, handler loops | SAME | SAME |
| p14 | `ITERATE j` / `LEAVE j` of the encloser after a looping test | DIFF | SAME |
| p15 | `CALL ON NOTREADY` in the callee, handler loops | DIFF | SAME |
| p16 | method `REPLY`s from inside its loop during the test | DIFF | SAME |
| p17 | `ITERATE` of the stepping loop (the `flat_loop_step_escaped` path) with a pending `CALL ON` delivered after the step | DIFF | SAME |

A `WHILE` and an `UNTIL` on one `DO` is Error 27.1 on the oracle and on the crate, so p07 nests
them instead.

## Driver and performance

* No change in `Op::Clause`; the diff touches only `flat_loop_step_top`. The new per-pass work is
  `Option::replace` plus a `Some` test in `Body`, and an `is_none` test in `Done`. The push is in
  `cover_enclosing_loop`, which is `#[cold] #[inline(never)]` (`loops.rs:1901-1905`).
* `bench-programs/callgrind.sh -B 0.5 -j 6 -p "startup sayloop" pin=<p9 pin> base=<base build>
  head=<fix build>`, exit 0:

| program | head d% vs pin (this run) | head d% vs pin (report) | base d% vs pin |
|---|---|---|---|
| startup | +0.0169 | +0.0188 | +0.0168 |
| sayloop | +0.0327 | +0.0336 | +0.1441 |

  Head agrees with the report within 0.002 points. The sayloop drop against base (-0.11) is a
  layout effect, not a saving: sayloop never reaches the cold path.

## Findings

* Minor: the `Err` arm is witnessed by one program only. Mutant C passes every probe above,
  including p04 and p13, which raise from inside the test. Only `loop_test_callee_loop.rex`'s
  `trapped` section separates C from the fix. A second witness is not required.

No Critical or Important findings.

## Task quality: approved

The fix restores the stack invariant on all three exits, behind a cold helper. The witnesses and
the mutation matrix reproduce exactly as reported. The row, fixture and exclusion changes match the
brief and the 2026-10-17 ruling.
