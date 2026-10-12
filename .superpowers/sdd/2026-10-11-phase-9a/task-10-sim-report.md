# Phase 9a Task 10 sim fix report

Base `fccf0d832`. Commits `0ad23af25` (rows), `dca9c3ac8` (early-end test), this report.

## Mechanism

* `sim_gate::judge` reds a failing test unless an oracle outcome in `corpus/sim-oracle/<group>.<part>.tsv` or a normal-mode `DIFFERING` key of the same group and part (`failing_unswitched`) names it.
* A `rest` row exists only where the whole part refuses: `calibration` and `whole_groups::rows_of` rename each refusing test out and rerun. Its `left_out` column is that refusing set.
* No whole part refuses now, so each rest row's left-out tests ran, `TEST_INSTANCEMETHOD`, `TEST_SETMETHOD` and `TESTRS01` failed in `rest` parts, and no rest `DIFFERING` row lists them. The whole parts' `DIFFERING` rows already list them.

## Changed rows

`rust/corpus/sim-gate.tsv`, derived by `calibration` (seed 0 under fifo). Calibration produced no rest row, so all seven go, with their oracle sets:

| row | before | after |
|---|---|---|
| Message rest | `1192 70 0 61 71 - TEST_REPLYWITH_NOT_ARRAY TEST_SENDWITH_NOT_ARRAY TEST_STARTWITH_NOT_ARRAY` | removed |
| Method rest | `0 70 0 66 71 - TEST_NEW_CONTEXT_* ... TEST_ROUTINE_SOURCE_WHITESPACE` | removed |
| Object rest | `320 70 0 81 91 - TEST_RUN_ARRAY_ARGUMENT TEST_RUN_NO_METHOD TEST_SENDWITH_NOT_ARRAY` | removed |
| RexxContext rest | `7 70 0 61 66 - TESTCOPY01` | removed |
| CONSTANT rest | `0 70 0 56 61 - TEST_EXPRESSION_ACTIVATE TEST_EXPRESSION_SELF` | removed |
| METHOD rest | `8 70 0 66 71 - TESTPACKAGE` | removed |
| Section1 rest | `15 70 0 50 56 - TEST_OBJECT_OBJECTNAMEEQUALS` | removed |

Rows whose calibrated k changed take the calibration's k and release_ms (k, release_ms):

| row | before | after |
|---|---|---|
| Message whole | 409, 56 | 1192, 66 |
| Message derived | 409, 55 | 1192, 61 |
| MethodArgs whole | 48479, 142 | 48486, 152 |
| MethodArgs derived | 48468, 141 | 48485, 163 |
| Object whole | 2, 66 | 320, 81 |
| RexxContext whole | 0, 50 | 7, 61 |
| Section1 whole | 0, 51 | 15, 50 |

Not re-derived: `debug_ms` (calibration was run in release only); the MutexSemaphore whole and derived k and the STREAM rows, since `calibration` runs every part without its `left_out` and knobs (`TEST_EXCLUSION`, `clock=real`), so its figures for them are of a different part.

Removed `rust/corpus/sim-oracle/*.rest.tsv` for the seven groups above.

`DIFFERING` in `crates/rexx-exec/tests/concurrency_tests.rs`, derived by `each_group_of_the_derived_list_in_one_run_in_both_modes` over the full list:

| row (normal and every) | before | after |
|---|---|---|
| TIME whole | `failure, assertions 506, rc 1 ... failing [TEST_VALIDOPT_BIGCHAR_R TEST_VALIDOPT_LITTLECHAR_R TEST_10 TEST_11 TEST_4 TEST_5]` | removed: agrees with the oracle, `pass, assertions 524` |
| TIME derived | `failure, assertions 65, rc 1 ... failing [TEST_10 TEST_11 TEST_4 TEST_5]` | removed: agrees, `pass, assertions 81` |
| CALL derived | `failure, assertions 7, rc 1 ... failing [TEST_4]` | removed: agrees, `pass, assertions 18`, in two runs |
| Section1 whole | `rexx-exec: method "OBJECTNAME=" ... rc 120 ...` | removed: agrees, `pass, assertions 17` |
| Method rest | `error, assertions 85, rc 2 ...` | removed: no rest part is produced |
| CONSTANT rest | `error, assertions 170, rc 2 ...` | removed: no rest part is produced |

Kept although listed as agreeing in one run: REPLY whole and derived, every mode. The oracle's 30 runs of each gave several outcomes, so one agreement does not show the row stale.

Tests changed with the rows: `the_differing_tests_are_read_from_every_key` reads TRACE's `TEST_TRACE_DROP` in place of the removed TIME key. `a_run_ending_where_no_listed_outcome_ends_is_red` used Section1 whole's refusing key, the only normal-mode `DIFFERING` key ending in a refusal; `early_end` now takes the part's `DIFFERING` keys, the test passes its listed key directly, and it checks `judge` reads `DIFFERING` through Object whole's rc 1 key.

## Commands

From `rust/`, each `CARGO_INCREMENTAL=0 ... memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests`:

* `REXX_CORPUS_GATE=1 REXX_SIM_CALIBRATION=<f> REXX_SIM_JOBS=4 ... sim_gate::calibration`: exit 0, no rest row and no refusal among the parts.
* `REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_ONLY=<batch> REXX_WHOLE_GROUPS_TABLE=<f> ... each_group_of_the_derived_list_in_one_run_in_both_modes`, before the edit in three batches covering every group: exit 0, exit 0, exit 0.
* `cargo fmt --all --check`: exit 0. `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* the whole binary without the gate: exit 0, 54 passed.
* At `dca9c3ac8`, `REXX_CORPUS_GATE=1 ... each_group_of_the_derived_list_in_one_run_in_both_modes` over the full list: exit 0, 1 passed, no rest row; listed as differing and agreeing: REPLY whole and derived every only.
* At `dca9c3ac8`, `REXX_CORPUS_GATE=1 REXX_SIM_REPORT=<f> ... sim_gate::`: exit 0, 11 passed; `the_seeded_gate` 13230 runs, 0 reds, 34 exempted.

Not run: the full close command.
