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
`TEST_STREAM_PASS_BIF_STREAM_OPEN`) are in `suite-9d-set.txt`, owned by 9d. Its 7 blocked tests
are 9b's. `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID` is 9b's although the 9d set listed it: Task 4
removed its line once the test was green.

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

## 9b close

The record's run is close run 2 at `07cf14bde`, by
`docs/superpowers/records/2026-10-13-phase-9b/close2/run.sh`:

```sh
cd rust
REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../close2/suite/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture
```

Its status file (`close2/status.txt`): start 2026-10-13T20:28:22Z, workspace exit 0 at 20:54:07Z,
suite exit 101 at 20:58:41Z, `finished`. Logs beside it: `workspace.log`, `suite.log`; the suite's
table, verdicts, per-test record and oracle differences are in `close2/suite/`.

Close run 1 at `afc3b895b` (`close/`) had the same exit statuses and an identical per-test record. It
predates the perf fix (`f3efac9a1`) and the final-review fixes (`3da644f47`, `7d4743be9`, `1590a1cea`).

**Criterion: every test in a group of `suite-9b-groups.txt` passes, or is in `suite-9d-set.txt`.**
Met. Every group of the file is green under the harness except `base/security.manager/SecurityManager`,
whose 5 failed tests (`TEST_ENVIRONMENT_HANDLE`, `TEST_METHOD_HANDLE_NO_RESULT`,
`TEST_METHOD_HANDLE_RESULT`, `TEST_METHOD_PASS`, `TEST_STREAM_PASS_BIF_STREAM_OPEN`) are
`suite-9d-set.txt` lines. The 7 tests it had blocked now pass.

Whole suite against the 9a close (`2026-10-11-phase-9a/suite-close/`): 356 of 402 groups green (340
at 9a). The groups that turned green are `base/class/File`, `base/keyword/ADDRESS`, `samples/samples`
and every `base/rexxutil` group of the file that was red; the list is the join of the two
`verdicts.tsv` on group where the verdict changed. Per test, 124 went from blocked to passed and 19
from failed to passed. No test that passed at 9a fails here; the comparison is a join of the two
`tests.tsv` on group and test.

**Criterion: no `"Phase 10"` site in RexxUtil lacks a D3 reason.** Met. The only `Owner::Phase10` rows
in `internal_routines.rs` are the seven macrospace routines, each `Deferral::Rxapi`
(`internal_routines.rs:114-206`), and `phase_ten_reasons` passed in the workspace run.

**Close command exit status:** 0.

**Spec scope not built, by Moritz's rulings (2026-10-13, ledger):**
- `SysFork` stays a loud refusal. A Rust process here always runs a timer thread and a joining main
  thread, so a forked child cannot continue; the design is
  `.superpowers/sdd/queued/2026-10-13-sysfork-design.md`. No ooTest file mentions `SysFork`.
- Named semaphores (`SysOpenEventSem`, `SysOpenMutexSem`, the named create) stay a loud refusal. On
  Unix they are POSIX `sem_open` semaphores, shared across processes and never unlinked; the design is
  `.superpowers/sdd/queued/2026-10-13-named-semaphores-design.md`. No ooTest file mentions them.

Both rows carry `Owner::Phase9`. Spec section 9 adds "Phase 9" to `closed_phases`' CLOSED at 9d's
close, which these rows would block, so their owner is open for Moritz before 9d plans.

**Licensed this phase:** Deviation 34 (`QUALIFY` and every path the shared normaliser serves keep
lexical POSIX `..`, where the oracle's `normalizePathName` mis-tracks a component that starts with `.`)
and Deviation 35 (`HANDLE:0/1/2` read and write the standard streams, where the oracle's
`handleOpen` drops the descriptor). New oracle-crashes entries 37 to 48.

**Plan constraint overridden:** the plan's Global Constraints say "No new dependency". Task 4 added
`tz-rs` 0.7.3 (`forbid(unsafe_code)`, no dependencies of its own) so `SysGetFileDateTime` and
`SysSetFileDateTime` read the program's `TZ` and resolve a skipped local time as glibc's `mktime`
does. The alternative was a hand-written TZif reader or a licensed divergence. Open for Moritz to
ratify.

**Perf budget: whole-process callgrind instructions, libc and ld included, within +0.5% of the pin.**
Met. `rust/bench-programs/callgrind.sh -r 3 -B 0.5 pin=<pin> head=<07cf14bde build>` exited 0 with no
`OVER` and no `SPREAD` row; pin `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`, sha256 `c4b4e8c4ce7f1f12e4a73fa37cb15eca262717092c2c1aed727febf437523c67` (`phase-9a-gate.md`), head binary
`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b-close2`. Log and table:
`docs/superpowers/records/2026-10-13-phase-9b/close2/callgrind.log` and `close2/callgrind/`; the raw
`.cg` outputs are in `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b-close/close2-run/`. Worst four
against the pin: hashnew +0.3065%, startup +0.2930%, pingguard +0.2696%, alloc +0.2272%.

Close run 1's callgrind (`close/callgrind.log`) flagged dirread at +0.5180%; `f3efac9a1` took it
under (`.superpowers/sdd/2026-10-13-phase-9b/close-perf-report.md`). The pin's own totals move
between invocations by up to about 0.2% on alloc and dispatch, with zero spread inside one run, so
deltas below that on those programs are not attributable.
