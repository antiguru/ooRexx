# Task 2 report: `tests/suite.rs`, the harness

Base `bee47022f`. Status: done.

## What changed

* `rust/crates/rexx-exec/tests/suite.rs` (new): `GroupRun` (the brief's fields, plus `tests_ran`, the
  framework's `Tests ran:` figure, and `took`), `CapKind` (`Memcap`, `UlimitFallback`, plus
  `OracleUlimit` for the oracle side), `discover_groups`, `run_group_on_oracle`, `run_group_on_crate`,
  and beside them `run_group` (oracle, then crate under `crate_deadline` = max(floor, 4 x oracle)),
  `run_group_on_crate_within` with a `CrateSettings` (groups root, `memcap` lookup name, floor, run
  cap), `crate_run_once`, `check_path`, `resolve`, `suite_revision`, the table and the
  oracle-difference report. The gate test `every_group_on_both_sides` runs under `REXX_SUITE_GATE=1`,
  eight groups at once, restricted by `REXX_SUITE_GROUPS`; it writes `REXX_SUITE_TABLE` or
  `target/suite/table.tsv`, and `oracle-differences.tsv` beside it.
* `support/group_runner.rs`: `fresh_copy_from` and `reaching_rxapi_in` (a groups root other than
  `ootest/ooRexx`, for the fixtures); `renamed` (skip's rewrite without the assertion, used to test
  whether a test can be renamed out); `run_process` (the process-group runner `run_crate_process`
  now calls) and `set_environment`; `crate_environment_with_path`; `interpreter_bin` and
  `groups_root` made public; `mark_starts_in_file` and `started_in_file`; `failing` and `refusal`
  moved here from `whole_groups` unchanged.
* `concurrency_tests.rs`: `whole_groups` imports `failing` and `refusal` instead of defining them.
* `rust/corpus/suite-revision.txt`: `13178`, which `svn info --show-item revision ootest` answered.
* Fixtures `tests/suite/fixtures/`: `RefusedCall`, `RefusedProlog`, `Hangs`, `HangsEverywhere`,
  `AddressSpace`. The refused routine is `SysQueryRexxMacro` (macrospace, stays Phase 10, spec
  section 4). Its name is built at run time (`'SysQueryRexx' || 'Macro'` through `INTERPRET`),
  because a literal `rexxmacro` makes `reaching_rxapi` rename the test out on both sides.

## Departure from spec section 2 (controller ruling (b))

The crate hands its stdout and stderr to the descriptors only before a read, before the thread
idles, and at exit (`input.rs` `hand_over_output`). Measured: `.error~say('started X'); do forever;
end` under `timeout -s KILL 2` writes nothing to stderr on `rexx-run` (rc 137), and `started X` on the
oracle. So a timeout or OOM kill loses every stderr marker, and `SIGTERM` instead raises HALT, which
`doTheTest`'s `signal on any` traps as an error in the hung test before the run goes on, so "last
started" would name the wrong test. Scout A's port had the same blind spot.

Ruling (b): the suite marks starts with `mark_starts_in_file`, which appends `started NAME` to
`<copy>/../started.txt` and closes the stream after each line, on both sides. `whole_groups` keeps the
stderr marker. Spec section 2 says ran is "the set of names in the `started` markers, as
`whole_groups`' `mark_starts` writes them"; the suite's markers are the same lines in a file. Fixing
the crate's buffering is queued for 9c by the controller.

## Interpretations

* The crate side's `bin` is `interpreter_bin`'s, `<copy>/../bin-rexx`, beside the copy and not
  inside it. That way a test that lists its directory sees the same files on both sides. Its `PATH`
  is that `bin`, then `/usr/local/bin:/usr/bin:/bin`. `check_path` runs before every crate run and
  panics unless `rexx` resolves to `bin/rexx` and `rexxc` resolves nowhere.
* Crate runs use `timeout -s KILL <secs>`, not scout A's `-k 10` (TERM first): TERM becomes HALT and
  the framework traps it (above). `timeout`'s 124 and 137 are both read as a timeout, after
  the refusal, memcap OOM, allocation-failure and panic texts are checked.
* The fallback without `memcap` is `sh -c 'ulimit -v 4194304 && exec "$0" "$@"'`, the shape
  `support/oracle.rs` already uses for the oracle. It adds no unsafe site.
* `REXX_SUITE_CRATE_FLOOR` is read by `CrateSettings::from_env` through `floor_from`. The fixture tests
  set the 5 s floor in `CrateSettings` directly, because `set_var` is `unsafe` in edition 2024 and
  the tests run threaded. `floor_from` itself has a test.
* A run that ends normally fills `passed` and `failed` from its `-V 2` detail for the tests it
  started. In a capped or blocked final run, the tests started besides the blocked one get no verdict.
* Iteration stops at the cap (60), at a run with no test started (blocked at load), or at a test the
  group file does not define (it cannot be renamed out).
* A run whose framework prints a summary with `Tests ran: 0` (for example a group file without the
  metadata comment, met while writing the fixtures) counts as ended normally with nothing started.
  The denominator check is what catches that.

## Step 1: fixture tests failing before the code

The tests were run against `todo!()` stubs of the harness functions
(`cargo test --release -p rexx-exec --test suite`, exit 101):

```
test fixtures::discovery_finds_groups_at_every_depth_in_path_order ... FAILED
test fixtures::a_refusal_ends_the_run_and_the_next_run_goes_past_it ... FAILED
test fixtures::the_crate_side_finds_its_own_rexx_and_no_rexxc ... FAILED
test fixtures::the_run_cap_stops_a_group_that_hangs_on_every_test ... FAILED
test fixtures::without_memcap_the_crate_run_is_under_the_ulimit_fallback ... FAILED
test fixtures::a_test_that_hangs_is_renamed_out_and_the_next_run_proceeds ... FAILED
test fixtures::a_refusal_at_load_is_blocked_at_load_after_one_run ... FAILED
test fixtures::the_crate_floor_is_read_from_its_variable ... FAILED
test fixtures::the_suite_revision_is_the_recorded_one ... FAILED
test result: FAILED. 20 passed; 9 failed; ...
```

With the stderr marker still in place (before ruling (b)), the two hang tests failed with
`iterations` 1 against 2. The KILL had left no marker, so the run counted as blocked at load. That
run is the control for `the_marker_file_names_the_test_a_kill_at_the_bound_ended_in`, added after
the ruling, which asserts status 137 and a marker file naming `TEST_A`, `TEST_B`.

The fallback witness: `AddressSpace.TEST_LIMIT` reads `Max address space` from `/proc/self/limits`.
It passes under the fallback (4294967296) and fails under `memcap` (unlimited), and the test asserts
both.

## Checks (each exit read unpiped)

* `cargo fmt -p rexx-exec`, then `cargo fmt --all --check`: exit 0.
* `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`, after
  touching `suite.rs`, `concurrency_tests.rs`, `api_group_tests.rs` and `group_runner.rs` so they
  were re-linted (one `Checking rexx-exec` line): exit 0. This also compiles `concurrency_tests` and
  `api_group_tests`.
* `CARGO_INCREMENTAL=0 memcap 8G cargo test --release -j 4 -p rexx-exec --test suite`: exit 0,
  `31 passed; 0 failed`, 10.08 s for the tests (28 s wall with the build).
* Not run: the `concurrency_tests` or `api_group_tests` binaries. Their `group_runner` changes are
  extractions with unchanged behaviour (`run_crate_process` builds the same command and calls
  `run_process`; `fresh_copy` and `reaching_rxapi` delegate with `groups_root()`; `skip` asserts the
  same set over `renamed`'s output). This is inferred, not run.

## Step 5: three groups against `scout-a-groups.tsv`

`REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/bif/C2X,base/class/Package_Options,samples/samples
REXX_SUITE_TABLE=/tmp/claude-1000/p9/t2/step5/table2.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test
--release -j 4 -p rexx-exec --test suite every_group_on_both_sides`: exit 0, 56.24 s. The table was
identical in every column but `failed` (empty on all rows in both runs) to the first run, which used
the stderr marker.

| group | side | started | passed | blocked | at load | iterations | ended normally | tests ran |
|---|---|---|---|---|---|---|---|---|
| base/bif/C2X | oracle | 34 | 34 | 0 | - | 1 | yes | 34 |
| base/bif/C2X | crate | 34 | 34 | 0 | - | 1 | yes | 34 |
| base/class/Package_Options | oracle | 54 | 54 | 0 | - | 1 | yes | 54 |
| base/class/Package_Options | crate | 54 | 0 | 54 | - | 55 | yes | 0 |
| samples/samples | oracle | 29 | 29 | 0 | - | 1 | yes | 29 |
| samples/samples | crate | 0 | 0 | 0 | RexxInfo EXECUTABLE (Phase 9) | 1 | no | - |

Against scout A:

* `C2X`: same (34 ran and passed on both, one crate iteration).
* `samples/samples`: same (oracle 29 ran, crate blocked before any test with `rexx-exec: method
  "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9)`, one iteration).
* `Package_Options`: the oracle row is the same (54 ran). On the crate, scout A stopped at its own
  40-iteration cap with 40 tests blocked and 14 unreached. The harness's cap is the spec's 60, so all
  54 tests were blocked one per run, each with `rexx-exec: a package settings write is not
  implemented (Phase 10)`. The 55th run ended normally with no test left. The 40 that scout A named
  are among the 54.
* `oracle-differences.tsv` has 83 rows: 54 `passed` against `blocked: ...` (Package_Options) and 29
  `passed` against `blocked at load: ...` (samples).

## Concerns

* A group that hangs on every test costs up to 60 x its bound (spec section 2's cap). The fixture
  test witnesses that the cap stops it, at `max_runs` 2.
* An oracle run that hits its 300 s bound (the socket samples) is recorded as `blocked` against its
  last started test with `timeout after 300 s`, not iterated. That path has not been run here.
* The 40 tests scout A blocked in `Package_Options` were checked to be a subset of the 54 here
  (`comm -23` over the two lists: 0 lines).
