# Task 3 report: the row files, their checks, the first whole run

Base `8a149e8b0`. Status: done, with concerns below. Commits, read back with `git log --oneline`:

* `717ab354b` the row files, their owner checks and the L3-core gate (and Task 2's minors 1, 2, 3, 6)
* `2415e1733` enumerate as the framework does; copy STREAM's neighbours (found by the first run)
* `50b483a81` the baseline suite run, the 9d set, the gate record, `rust/CLAUDE.md:13`
* `f82bca323` unit tests of D1's judgement

## What changed

* `support/group_runner.rs`: `source_test_names` widened and rebuilt on `source_classes` /
  `test_classes` (Step 1, plus what the first run found): a class is a test class when its
  directive names `ooTestCase` or it subclasses, mixes in from or inherits a test class of the same
  file (fixpoint, any declaration order; `INHERIT` takes every later word, as
  `DirectiveParser.cpp:461-469` does); block comments (nesting) and `--` comments are skipped
  outside quotes; a quoted method name keeps its blanks; `::CONSTANT` and `::ATTRIBUTE` count as
  methods (`getTestMethods` takes every instance method named `TEST...`); `class` methods do not.
  `source_test_names_in` takes a root.
* `support/suite_rows.rs` (new, `#[path]` in `suite.rs` and `concurrency_tests.rs`): row parser,
  `Owner` and `Phase10Reason` (D3's closed set), the owner checks, `derived_rows`, `table_path`
  and `TESTS_FILE` (shared with the DIFFERING cross-check). Reason formats:
  `reaching_rxapi` | `group` | `refusal: TEXT` for Phase 10; `common.txt: ENTRY` for Flaky;
  `record: PATH; ruled: Moritz YYYY-MM-DD` for Environment (a reason starting `hangs` is renamed
  out on both sides); `restrictOS line N` | `test type NAME` for NotRun. Each file carries one
  `# ootest revision N` line, checked against `suite-revision.txt`.
  **The Phase 10 refusal check is `refusal_carries(text, reason)`, one function**: today it asks
  only that the text names Phase 10 (controller ruling); Task 4 tightens it there.
* `suite.rs`: `expected_set(_in)`; `l3_core_suite` (gate-only) **replaces**
  `every_group_on_both_sides` rather than running the suite twice: it runs, fills `expected` (the
  enumeration less the group's rows), writes `table.tsv`, `oracle-differences.tsv`, `tests.tsv`
  (per test: row owner, oracle verdict, crate verdict) and `verdicts.tsv`, then asserts every group
  green. `judge` is D1 plus the row cross-checks (a recorded refusal must equal the captured one; a
  NotRun test must not start on the oracle); `oracle_denominator` reports the check on the oracle
  side. `the_row_files_hold_only_checked_rows` (plain `cargo test`) runs every owner check over
  every group and asserts each derivable row is present. `NEIGHBOURS` (below). The table gains an
  `ended before a test` column.
* `concurrency_tests.rs`: `differing_tests()` and `every_differing_test_is_fixed_or_has_a_row`
  (gate-only; "fixed" = `passed` on the crate in the suite's last `tests.tsv`), and a plain test
  that the parser reads the keys.
* Row files: `docs/superpowers/plans/phase-9-exclusions.txt`, `rust/corpus/suite-licences.txt`,
  `rust/corpus/suite-not-run.txt`; `rust/corpus/phase-9d-set.txt`;
  `rust/corpus/suite-oracle-runs/samples-{scclient,scserver,sfclient,sfserver}.txt`;
  `docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/` (the run's four outputs);
  `docs/superpowers/plans/phase-9a-gate.md` (sections Row files, Baseline suite run, Environment
  candidates, DIFFERING cross-check, The 9d set); `rust/CLAUDE.md:13`.
* Fixtures: `ChildRefuses`, `Grandchild`, `ChildPath`, `RefusedAfterFirst`, `Inherits`,
  `Enumerated`.

### Task 2 minors

1. `end_reason`: a run is early only if killed, signalled, rc 124 or >= 128, or without a summary;
   stderr text (OOM, refusal, panic) only labels an early run.
2. `timeout --foreground -s KILL`. Measured before the change: without `--foreground` GNU `timeout`
   already kills its own group at expiry (the fixture with a foreground `sleep` at a 5 s bound
   stayed green), so the escape is a process a test leaves running in the background after a
   normal end: it sat in `timeout`'s group and outlived the run.
3. `a_command_the_crate_side_runs_finds_its_rexx_and_no_rexxc`: `address system` runs
   `command -v rexx` and `command -v rexxc` and the test reads both answers (`rexxc`: `127`, dash's
   not-found status).
6. `blocked_at_load` only on the first run; a later run that starts no test sets the new
   `ended_before_a_test`.

## Failing-first evidence

All with `CARGO_INCREMENTAL=0 memcap 8G cargo test --release -j 4 -p rexx-exec --test suite -- <filter>`:

* Minor 1, 6: exit 101 before the fix. `end_reason` answered
  `Some("rexx-exec: routine \"SYSQUERYREXXMACRO\" is not implemented (Phase 10)")` for a run that
  ended with its summary; `RefusedAfterFirst` set `blocked_at_load` to that text on the second run.
* Minor 2: exit 101 before the fix, "the command outlived the run" (a `sleep 41.5` was still in
  `/proc`, killed by hand). An earlier fixture shape showed this crate's `address system` waits for
  a background child holding its output pipe (the run took 41.6 s); not compared with the oracle.
* Minor 3: new test, green; the mutant that sets the child's `PATH` to `/usr/bin:/bin` after the
  environment is built (so `check_path` still passes) reddens it: `left: ""`, right the `bin/rexx`
  path. Reversed; `grep -c MUTANT` 0.
* Step 1 (inheritance): `Inherits` red before the widening (`left: {"TEST_BASE"}` against five);
  green after, and equal to the oracle's started set. The first oracle run of the fixture refused
  `INHERIT ... PUBLIC` ("Class PUBLIC not found"), which is how the INHERIT rule was found.
* Enumeration (after run 1): `Enumerated` red before the comment/quote/constant/attribute fix
  (`left` had `TEST_CLASS_SIDE`, `TEST_COMMENTED`, `TEST_QUOTED`), green after and equal to the
  oracle's started set.
* Owner checks: the fixture rows were written beside the code, not before it. Evidence that each
  check has a failing row is a mutant per check (`/tmp/claude-1000/p9/t3/mutants.py`, restored by
  copy, `cmp` clean): disabling the Phase 10, Deviation, Environment, NotRun, refusal, owner-file,
  Rxapi-completeness or duplicate check each reddens a `row_checks` test. The Flaky mutant first
  stayed green; a row (`TEST_SIMPLE` citing its own non-entry) was added and it then went red.
* NEIGHBOURS: with the copy loop disabled,
  `the_relative_path_tests_pass_on_the_oracle_with_their_neighbours_copied` fails
  (`STREAM TEST_RELATIVE_FILE_EXISTS`); green with it.
* `judging` (D1): six mutants, one per reason `judge` gives, each reddens a test.

## Checks (exit status unpiped)

At `f82bca323`'s tree:

* `cargo fmt -p rexx-exec`: 0; `cargo fmt --all --check`: 0 (at `50b483a81`).
* `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: 0.
* `CARGO_INCREMENTAL=0 memcap 8G cargo test --release -j 4 -p rexx-exec --test suite --test concurrency_tests`:
  0; `suite` 47 passed, `concurrency_tests` 54 passed (gated parts return early).
* The first `cargo build --tests` after the support module landed took 7 min 16 s; later check runs
  reused the build and finished well inside 5 minutes.

## The whole run

Run 1 at `717ab354b` (status `/tmp/claude-1000/p9/t3/run1/status.txt`): 189 s, exit 101, 86 red.
It found the enumeration errors and the four STREAM/Stream oracle failures, fixed in `2415e1733`.

Run 2, the recorded one, at `2415e1733` (status file's first line the sha, `finished` last; tree
untouched until then):

```sh
cd rust
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=/tmp/claude-1000/p9/t3/run2/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture
```

328 s wall, exit 101, `78 of 402 groups are not green`. Expected (enumeration less rows) 23003 on
each side; oracle started 23935, passed 23928; crate started 23421, passed 22850 (both counts
include rowed tests). Of the expected tests the crate passed 22472, failed 251, blocked 243, blocked
at load 37. **The oracle-side denominator check is `equal` on all 402 groups.** 658 crate runs, 38
groups iterated, `Package_Options` 55 times. Per-group verdicts are in the gate record and
`suite-baseline/verdicts.tsv`.

## Row counts

```sh
cat docs/superpowers/plans/phase-9-exclusions.txt rust/corpus/suite-licences.txt rust/corpus/suite-not-run.txt \
  | grep -v '^#' | awk -F'\t' '{r=$4; if (r ~ /^refusal: /) r="refusal"; if (r ~ /^restrictOS line/) r="restrictOS";
    if (r ~ /^common.txt/) r="common.txt"; print $3" / "r}' | sort | uniq -c
```

274 Phase 10 Rxapi / reaching_rxapi; 18 Phase 10 Rxapi / group (R1 utilities); 4 Phase 10 Rxapi /
refusal; 973 Phase 10 Extension / group; 63 Phase 10 Row10NativeApi / group; 4 Flaky; 183 NotRun /
restrictOS; 1 NotRun / test type. No Deviation, no Environment row.

The four refusal rows: `base/bif/LINEOUT` `TEST_QUEUE` (`.STDQUE`), `base/class/RexxQueue`
`TEST_BUG_1379` (`rexx_query_queue`), `base/directives/ROUTINE` `TESTEXTERNALREGISTEREDBADLIBRARYNAME`
and `TESTEXTERNALREGISTEREDBADNAME` (`::ROUTINE EXTERNAL naming REGISTERED`). I gave them `Rxapi`
by reading the refusal text; Task 4's classification decides, and a relabel to Phase 9 would fail
the tightened check. Not rowed, because D3/R2 make them Phase 9's: the package settings write and
every non-macrospace RexxUtil routine, though their text says Phase 10 today.

## Environment candidates

Not added; listed in the gate record with records in `rust/corpus/suite-oracle-runs/`. The
`TEST_04` of `samples/scclient`, `scserver`, `sfclient`, `sfserver`: alone, one after another, all
four pass on the oracle; at eight groups at once they fail or hang in a different combination in
each of three runs (scout A's, run 1, run 2), with `EADDRINUSE` from a bind in scout A's records.
They use fixed ports. A harness rule running them one at a time is the alternative to rows; both
are Moritz's call. No other test without a Phase 10 row failed on the oracle in run 2.

## The STREAM finding

The harness copy caused all four. `STREAM.TEST_RELATIVE_FILE_EXISTS`/`_EXISTS2` open
`../class/Array.testGroup` and `../../../ooRexx/API/oo/CONVERSION.testGroup` from the group's
directory (their `setup` changes to it); `Stream.TEST_RELATIVE_EXISTS2`/`_EXISTS3` open
`base/bif/STREAM.testGroup`. Scratch oracle runs: `base/bif` alone, 2 failures; with the two
neighbour files, 0. `base/class` alone, 2 failures; with `STREAM.testGroup`, 0. `NEIGHBOURS` now
copies them; run 2 has all four passing on the oracle. `whole_groups` keeps its own copy and its
`DIFFERING` key naming them is unchanged.

## DIFFERING cross-check and the 9d set

Against run 2's record the cross-check exits 101 with 41 tests neither passing nor rowed (listed in
the gate record). Expected red, as `l3_core_suite` is.

`rust/corpus/phase-9d-set.txt`: run 2's crate-failed, oracle-passed, unrowed tests less the 52
witnesses, by the `awk` in its header (re-run against the committed `tests.tsv`: identical). Against
the TSV: the TSV's `tests_wrong` has 249 distinct (group, test) pairs, all in run 2's set; run 2 adds
`base/class/Package` `TEST_PROLOG` and `TEST_PACKAGE_FINDPROGRAM_REXXTRY`.

## Inferred, not run

* `TEST_PACKAGE_FINDPROGRAM_REXXTRY` fails because the crate side's `PATH` holds no `mime.cls`
  (the oracle's `bin`, on its `PATH`, does): read from the test and an `ls`, not run with a
  changed `PATH`.
* The socket groups' failures come from the port contention between them: inferred from
  `EADDRINUSE` in scout A's record and the alone runs passing, not from a controlled pairing.
* `.STDQUE`, `rexx_query_queue` and `REGISTERED` needing the daemon: read from the names.

## Concerns

* `l3_core_suite` replaced `every_group_on_both_sides` (one run, not two); Task 2's report and any
  later dispatch naming the old test need the new name.
* Spec section 2 does not say the row reason formats, `NEIGHBOURS`, or that "fixed" for the
  DIFFERING cross-check means `passed` in the last gate run's `tests.tsv`. I did not amend the
  spec or plan; say if the controller wants that.
* `derived_rows` asserts derivable rows are present, not that every present row is still derivable;
  a stale Phase 10 or NotRun row passes as long as its owner check does. `judge` does not flag a
  rowed test that passes.
* The TSV's `tests_wrong` is 249 tests in 49 groups (`awk -F'\t' 'NR>1 && $25!=""'`), as the spec
  says; scout A's report says 36 groups. Not reconciled.
* The socket candidates make the gate's verdict for four groups depend on scheduling until ruled.
