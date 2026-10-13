# Phase 9b gate: RexxUtil

This record holds what Phase 9b measures. Sections are added as the plan's tasks land.

## Starting verdicts (Task 2)

The gate groups are `rust/corpus/suite-9b-groups.txt`. The starting run is the suite harness over
those groups at `a0abcc77c`, the commit that adds the file, before any 9b routine exists. Run from
`rust/`:

```sh
REXX_SUITE_GATE=1 \
REXX_SUITE_GROUPS="$(grep -v '^#' corpus/suite-9b-groups.txt | paste -sd,)" \
REXX_SUITE_TABLE=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-13-phase-9b/task-2-baseline/table.tsv \
CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite
```

Exit status (101, red as expected) and wall time (2 s, the release build already cached) are the last two lines of `run-output.txt`. The raw output, the table,
the verdicts and the per-test record are in
`docs/superpowers/records/2026-10-13-phase-9b/task-2-baseline/` (`run-output.txt`, `table.tsv`,
`verdicts.tsv`, `tests.tsv`, `oracle-differences.tsv`). The oracle denominator is `equal` for every
group. 18 groups ran and 16 are red.

Per group, from `tests.tsv` (`crate` columns: passed, blocked by a "not implemented (Phase 9)"
refusal, failed, not started because a row owns the test):

| group | tests | passed | blocked | failed | not started |
|---|---|---|---|---|---|
| base/class/File | 64 | 62 | 2 | 0 | 0 |
| base/keyword/ADDRESS | 80 | 78 | 1 | 0 | 1 |
| base/rexxutil/SysDumpVariables | 4 | 0 | 4 | 0 | 0 |
| base/rexxutil/SysFileDateTime | 8 | 0 | 8 | 0 | 0 |
| base/rexxutil/SysFileSearch | 14 | 0 | 14 | 0 | 0 |
| base/rexxutil/SysFileTree | 26 | 9 | 0 | 17 | 0 |
| base/rexxutil/SysFileXXX | 43 | 19 | 23 | 1 | 0 |
| base/rexxutil/SysFormatMessage | 5 | 0 | 5 | 0 | 0 |
| base/rexxutil/SysSearchPath | 10 | 0 | 10 | 0 | 0 |
| base/rexxutil/SysStemCopy | 12 | 0 | 12 | 0 | 0 |
| base/rexxutil/SysStemDelete | 5 | 0 | 5 | 0 | 0 |
| base/rexxutil/SysStemInsert | 5 | 0 | 5 | 0 | 0 |
| base/rexxutil/SysStemSort | 18 | 0 | 18 | 0 | 0 |
| base/rexxutil/platform/unix/SysGetMessage | 11 | 0 | 9 | 0 | 2 |
| base/rexxutil/platform/unix/tilde | 1 | 0 | 1 | 0 | 0 |
| base/rexxutil/platform/windows/SysFileTree | 13 | 0 | 0 | 0 | 13 |
| base/rexxutil/platform/windows/SysIsFileDirectory | 6 | 0 | 0 | 0 | 6 |
| base/security.manager/SecurityManager | 29 | 17 | 7 | 5 | 0 |

The two `platform/windows` groups are green already: every test carries a `NotRun` row (`restrictOS`),
so they stay in the file and never move.

The 9b close criterion (controller ruling, 2026-10-13, ledger .superpowers/sdd/2026-10-13-phase-9b/progress.md) is that every test in a group of the file passes, or
is listed in `rust/corpus/suite-9d-set.txt`. The 9d set is the carried list, so 9b judges only what it
owns. `base/security.manager/SecurityManager`'s 5 failed tests (`TEST_ENVIRONMENT_HANDLE`,
`TEST_METHOD_HANDLE_NO_RESULT`, `TEST_METHOD_HANDLE_RESULT`, `TEST_METHOD_PASS`,
`TEST_STREAM_PASS_BIF_STREAM_OPEN`) are in `suite-9d-set.txt:200-204`, owned by 9d. Its 7 blocked tests
are 9b's. `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID` is 9b's although the 9d set also lists it (line 198):
Task 4 removes that line once the test is green.

Owning 9b task of each red group (plan `2026-10-13-phase-9b.md`):

| group | task |
|---|---|
| base/rexxutil/SysStemCopy, SysStemDelete, SysStemInsert, SysStemSort, SysDumpVariables | Task 3 |
| base/rexxutil/SysFileXXX, SysFileDateTime, SysSearchPath, SysFileSearch, platform/unix/tilde, base/class/File | Task 4 |
| base/rexxutil/SysFileTree | Task 5 |
| base/rexxutil/platform/unix/SysGetMessage, SysFormatMessage, base/keyword/ADDRESS | Task 6 |
| base/security.manager/SecurityManager, 6 tests blocked on `SYSDROPFUNCS` | Task 7 |
| base/security.manager/SecurityManager, `TEST_CALL_PASS_THREE_ARGS` (blocked on `SysSearchPath`) | Task 4 |

SecurityManager's 5 failed tests are 9d's, as above.
