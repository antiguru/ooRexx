# Task 6.2 report: integer representation (Deviation 44)

Status: DONE. No crate source changed. All 19 tests of `rexxinteger-min-representation` differ
from the oracle only through the group's `isInteger` routine, so each is licensed under
Deviation 44.

## Commits

* `9fdf5bd99` Phase 9d Task 6.2: the known gap beside Deviation 41 sits inside the deviations section
* `63c459bbc` Phase 9d Task 6.2: license the RexxInteger tests under Deviation 44
* the records and report commit that follows them

### Why the first commit

The first licence-check run failed every new row with
`docs/superpowers/plans/phase-4-exclusions.txt has no Deviation 44 that is not withdrawn`.
`suite_rows::deviations` reads the `DEVIATIONS -- permanent, chosen` section up to the next
line at the margin. The `KNOWN GAP BESIDE DEVIATION 41` heading (added in 9c Task 9, `240508580`)
started at the margin at line 2295, so the scan stopped there, before entries 42 to 46. The commit
indents its two heading lines to match its body. No wording changed. The same fix lets the
instances for Deviations 45 and 46 write their rows. `phase-4-exclusions.txt` is one of the
shared serial files.

## Method

The patched group is `docs/superpowers/records/2026-10-18-phase-9d/t6.2/isinteger-patch.diff`,
a diff against `ootest/ooRexx/base/class/RexxInteger.testGroup`. The original routine is renamed
`isIntegerOriginal`, unchanged. The new `isInteger` has two modes:

* `ISINT_MODE=log` runs the original and logs, per call, the caller's line
  (`.context~stackFrames[2]~line`), the argument, `dataType("W")` under `NUMERIC DIGITS 18`, and
  the answer.
* Otherwise it replays the oracle's answer from a table keyed by `line|argument`. For a call
  site where every oracle call answered alike, it falls back to a key of the line alone, which
  `DATE` and `TIME` need because their arguments change between runs. It also compares
  `dataType("W")` with the oracle's. A missing key or a `dataType` mismatch answers `.nil`, which
  every assertion and `\` refuse.

The replay keeps the routine's non-representation half, `dataType("W")`, checked on the crate.
It replaces only the `+ 20` and `~min` steps, which are the Deviation.

Commands (scratchpad copy of the framework laid out as `fresh_copy` does, `rxfuncquery` lines
removed from `ooTest.frm`):

* `t6.2/rungroup.sh SIDE COPY OUT` runs `testOORexx.rex -f <group> -U -V 2`. The oracle runs
  under `ulimit -v 1048576`, the crate binary (release, `d96b243a8` source) under `memcap 2G`, each
  under `timeout`.
* The oracle log is `t6.2/runs/log-oracle.tsv`. `t6.2/derive-replay.sh runs/log-oracle.tsv OUT`
  derives `t6.2/replay.tsv`, re-run and `cmp`-identical to the committed table.
* `t6.2/oracle-calls-by-site.txt` counts the oracle's calls per site. Every oracle call answered
  the same as its `dataType("W")`. Only the power loop's site (line 256) mixes answers, and every
  call there has its exact key.

## Results

| run | side | tests ran | assertions | failures | errors | status |
| --- | --- | --- | --- | --- | --- | --- |
| original group | oracle | 20 | 38761 | 0 | 0 | 0 |
| original group | crate | 20 | 1181 | 19 | 0 | 1 |
| replay | oracle | 20 | 38761 | 0 | 0 | 0 |
| replay | crate | 20 | 38761 | 0 | 0 | 0 |

The two replay runs' stdout, with the four timing lines removed, is identical. Both stderrs are
empty. Files: `t6.2/runs/{base,replay}-{oracle,crate}.{stdout,status}`.

Controls (the replay is live on the crate, not a vacuous pass):

* `runs/control.diff` sets the oracle's `dataType` for `256|1000` to 0. The crate run prints
  `ISINT-WHOLE 256 1000 1 0` and errors in `TEST_INTEGER_POWER` (`runs/control-crate.stdout`).
* `runs/control2.diff` flips the exact key `139|5` and the line-only key `391` to answer 0. The
  crate fails `TEST_INTEGER_CONSTANT` at line 139 ("5 should be a RexxInteger") and
  `TEST_INTEGER_TIME` at line 391 ("time('F') ..."), so the exact keys and the line-only fallback
  are both used (`runs/control2-crate.stdout`).

The crate's original routine answers 0 for every whole number. In log mode its first call in
each test logs `dataType("W")` 1 and answer 0 (`runs/log-crate.tsv`).

## Per-test table

Each test's first failure on the original group is an `assertTrue(isInteger(...))`. Under replay
every assertion of every test passes on the crate, with the oracle's assertion total.

| test | first failure, original (line: message) | verdict |
| --- | --- | --- |
| TEST_INTEGER_ADDITION | 168: `0 + 0` | detection only |
| TEST_INTEGER_CONSTANT | 139: `0 should be a RexxInteger` | detection only |
| TEST_INTEGER_D2X | 281: `0~d2x should be a RexxInteger` | detection only |
| TEST_INTEGER_DATE | 379: `date('B') should return a RexxInteger` | detection only |
| TEST_INTEGER_DIVISION | 203: `0 / 1 should be a RexxInteger` | detection only |
| TEST_INTEGER_INTEGERDIVIDE | 215: `0 % 1 should be a RexxInteger` | detection only |
| TEST_INTEGER_MODULO | 240: `0~modulo(1) should be a RexxInteger` | detection only |
| TEST_INTEGER_MULTIPLICATION | 190: `0 * 0 should be a RexxInteger` | detection only |
| TEST_INTEGER_POWER | 258: `-1000 ** 0 NumberString should be > ...`, after `until \isInteger` ended the loop at power 0 | detection only |
| TEST_INTEGER_REMAINDER | 226: `0 // 1 should be a RexxInteger` | detection only |
| TEST_INTEGER_REXXINFO | 367: `999999999999999999 should be a RexxInteger` | detection only |
| TEST_INTEGER_SUBTRACTION | 179: `0 - 0` | detection only |
| TEST_INTEGER_TIME | 391: `time('F') should return a RexxInteger` | detection only |
| TEST_INTEGER_UNARY_MINUS | 158: `-0 should be a RexxInteger` | detection only |
| TEST_INTEGER_UNARY_PLUS | 149: `+0 should be a RexxInteger` | detection only |
| TEST_NUMBERSTRING_CEIL_FLOOR_ROUND_TRUNC | 301: `0.~trunc should be a RexxInteger` | detection only |
| TEST_NUMBERSTRING_INTEGERDIVIDE | 320: `0. % 1 should be a RexxInteger` | detection only |
| TEST_STRING_C2D | 339: ` ~c2d should be a RexxInteger` | detection only |
| TEST_STRING_X2D | 358: `0~x2d should be a RexxInteger` | detection only |

The group's 20th test, `TEST_INTEGER_D2C`, calls no `isInteger` and already passes.

## Fixes and witnesses

None. No assertion outside the detection differs, so there is no real divergence to fix and no
corpus witness. Deviation 44 carries no `LICENSED DIVERGENCE WITNESS:` marker, and this task adds
none.

## Licence rows

`rust/corpus/suite-licences.txt` gains 19 rows of the form
`base/class/RexxInteger<TAB><TEST><TAB>Deviation 44<TAB>the group's isInteger reads the oracle's integer representation through n~min`,
sorted into place by group. The 19 `base/class/RexxInteger` lines leave
`rust/corpus/suite-9d-set.txt`.

The licence check passes:
`CARGO_INCREMENTAL=0 timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite`,
exit 0, 52 passed, including `the_row_files_hold_only_checked_rows`. Before the first commit, the
same test failed on all 19 rows (see above).

## Whole-suite changed rows

Run: `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.2/suite/table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`
(with `timeout 600` and the task's `CARGO_TARGET_DIR`), exit 101 as expected, 402 groups, 36 red.

Against `t6.1/suite-r2/`:

* `base/class/RexxInteger`: verdict red to green. In `tests.tsv` its 19 tests' row column moves
  from `-` to `Deviation(44)` (still oracle passed, crate failed). In `table.tsv` the expected
  count moves from 20 to 1 on both sides.
* `base/rexxutil/SysFileTree TEST_TREE_DIRECTORY`: the oracle failed it in this run (crate
  passed). Not caused by this task, which changed no interpreter code. Re-running
  `REXX_SUITE_GROUPS=base/rexxutil/SysFileTree,base/class/RexxInteger` three times
  (`t6.2/suite-sysfiletree-{1,2,3}/`) gave passed/passed, passed/passed, then passed on the oracle
  and failed on the crate. Earlier records show it failing on either side
  (`2026-10-11-phase-9a/suite-baseline`, `suite-close`, `2026-10-13-phase-9b/task-2-baseline`). It is
  flaky on both sides and has no row.

No other row changed.

## Perf

Not run. The task changes no file under `src/`.

## Checks

| command (from `rust/`, task `CARGO_TARGET_DIR`, `CARGO_INCREMENTAL=0`) | exit |
| --- | --- |
| `cargo build -j 4 --release -p rexx-exec --bin rexx` under `memcap 8G` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy` | not run, no Rust file changed |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite` (non-gated, row checks) | 0 |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test licensed_divergences` | 0 |
| `REXX_CORPUS_GATE=1 timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0 (29 passed, 1 ignored) |
| `REXX_SUITE_GATE=1 ... l3_core_suite` (whole suite) | 101 (expected) |
| same, two groups, three times | 0, 0, 101 (the SysFileTree flake) |

## Concerns

* The exclusions-file indentation is a change the brief did not name. Without it no Deviation
  numbered above 41 can be licensed.
* `TEST_TREE_DIRECTORY` is flaky on both sides with no `Flaky` row. It will keep showing up in
  later tasks' changed rows.
