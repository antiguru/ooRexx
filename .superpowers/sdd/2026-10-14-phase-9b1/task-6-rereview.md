# Phase 9b1 Task 6 fix round 1: re-review

Verdict: resolved, no open findings. Reviewed 13c5c2390..a8ba623b2 at HEAD a8ba623b2. Binary:
`rust/target/release/rexx-run` (cargo reported it fresh). Each loop ran from its own fresh dir,
`/usr/bin/time -f %M` directly on rexx-run under `memcap 4G`.

## (1) merged_from leak
Peak kB, 20000 / 80000 iterations:

| loop | 20000 | 80000 | change |
|---|---|---|---|
| `call ext`, `return 1` | 423104 | 424232 | +0.27% |
| `call ext`, with `::routine pubr public` | 448140 | 448512 | +0.08% |
| `exta`/`extb`, same public name (loser never written) | 446948 | 448136 | +0.27% |
| 3 files, different public names | 448136 | 448920 | +0.18% |
| `call ext`, public `::class Foo` | 589052 | 602000 | +2.2% |
| `exta`/`extb`, same public class name | 594252 | 615664 | +3.6% |

All within 10%. Before the fix the first row was 640 MB / 2.5 GB. The absolute gap to the oracle
(~20 MB) is the collector trigger, ruled outside this task. `merged_public_classes` is first-wins,
so a class path holds one package per name; measured flat.
e5/e6/e7 (`collect_stress -- external parked`, release): 9 passed. Mutant (archive copy, own target):
`MergedRoutine::Installed(_, object)` removed from `PackageData::trace` -> the three
`an_external_routines_merged_routine_*` tests fail.

## (2) parked.rex
`a_parked_activitys_frame_keeps_its_program` passes unmutated and FAILS with
`Activities::running_programs` emptied (same mutant build). Withdrawn exception is reflected in the doc.

## (3)-(6)
- special_methods: pruned in `prune_program_rows`, trailing None popped; `special_method_row_mut`
  resizes on write and ids are monotonic, so the pop is safe. The `is_empty` debug_assert in
  `install_object_model` runs once at start. Asserted in the stale-id test.
- Prune assertions: `executable_objects` and global-root names added. Fine.
- M2b (delete `executable_package()` from `Body::trace`) is a lifetime mutant; the report shows the same
  mutant binary passing a plain run, so the red is the collection. I did not re-run M2b.
- expect: gone; `unattached_method_id` takes the caller's `&Rc<Program>`.
- New behaviour check: `added_routine` now returns None when the routine's program has no package.
  Probe `addRoutine` of a `Routine~new` routine and of a `Package~new` routine, with forced allocation
  between: oracle and crate identical (`7 8` / `7` / `8` / `7 8` / `9`, rc 0).

## (7) inline(always)
`callgrind.sh -B 0.5 -p "sendloop startup"` vs the pin, exit 0, no OVER/SPREAD: sendloop +0.0715%,
startup +0.3372% (matches the report's figures to the digit).

## Observations (not findings)
- `install_directives` now builds the package object eagerly for any program with routines. Startup
  is unchanged by the measure above.
- I did not re-run the full lib/corpus suites; I ran collect_stress under the mutant (50 pass, 5 fail:
  3 external, parked, and `the_l0_subset...`, all consequences of the two mutants).
