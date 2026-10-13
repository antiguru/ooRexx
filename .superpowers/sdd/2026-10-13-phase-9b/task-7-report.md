# Phase 9b Task 7 report: the RexxUtil process and session routines

## Status

DONE_WITH_CONCERNS. Every routine but `SysFork` runs. `SysFork` keeps a loud refusal naming the
grant it needs (concern 1). `base/security.manager/SecurityManager` is 23 of 29, not the 24 the
dispatch expected: `TEST_REQUIRES_HANDLE` was blocked on `SysDropFuncs` and now fails on a
security-manager `REQUIRES` difference outside RexxUtil (concern 2). Commits:

- `6220bd401` Phase 9b Task 7: the RexxUtil process and session routines
- `022590321` Phase 9b Task 7: PGID pinned in a process group of its own
- the records commit that carries this report (sha in the reply)

## Classification (Step 1)

| routine | class | evidence |
|---|---|---|
| `SysUtilVersion`, `SysLoadFuncs`, `SysDropFuncs` | observable | witness |
| `SysQueryProcess` | observable through `/proc` | witness, plus `tests/rexxutil_process.rs` for `PGID` |
| `SysSetPriority` | observable through `PPRIO` and `/proc/thread-self/stat` | witness |
| `SysCls` | observable: `clear` writes its bytes to standard output with no terminal | witness |
| `SysGetKey` | terminal-only, except the not-a-terminal answer and argument errors | unit tests on a pseudo-terminal, witness for the rest |
| `SysCreatePipe` | observable through `/proc/self/fdinfo` and a child's inherited descriptors | witness |
| `SysWait` | observable, but reaps any child of the process | `tests/rexxutil_process.rs` (own process) |
| `SysFork` | needs `unsafe` | measured, not written |

None needs the daemon.

## What changed

New `builtin/rexxutil/process.rs` and `process/tests.rs`. The ten rows in `internal_routines.rs` run.

- **`SysUtilVersion`**: `version::VERSION_NUMBER`. **`SysLoadFuncs`/`SysDropFuncs`**: one body, `0`.
  All three are `RexxRoutine0`: 88.922 for any argument (measured).
- **`SysQueryProcess`**: `PID`, `PPID`, `PGID` (the parent's group, `getpgid(getppid())` as the
  C++ has it), `PPRIO` (`getpriority`), and `PTIME`/`PMEM`/`PSWAPS`/`PRCVDSIG`; the selector caseless
  and cut at a NUL; anything else 40.1. `getrusage` has no safe binding (rustix has none, libc is
  `unsafe`), so the usage comes from `/proc/self/stat` (`utime`, `stime`) and `/proc/self/status`
  (`VmHWM`). `ru_nswap` and `ru_nsignals` are never set on Linux (`getrusage(2)`), so those two
  answer `0`. `PTIME` ports the C++ formatting and its millisecond carry exactly (unit test).
- **`SysSetPriority`**: optional `int32` class (checked, ignored), required `int32` priority,
  `setpriority(PRIO_PROCESS, 0, -prio)` through `rustix::process::setpriority_process`, errno or `0`.
- **`SysCls`**: `/bin/sh -c clear` with `argv[0]` `sh` (as `system` runs it: the shell's own
  "not found" message names `sh`), the program's environment and directory, its output into this
  interpreter's sinks, answering the raw wait status (`256` for `clear` failing, `32512` for no
  `clear`, measured).
- **`SysGetKey`**: `ECHO`/`NOECHO` caseless, else 40.920. The null string where `.input` is not the
  process's standard input or that is not a terminal. Otherwise the output is handed over, `/dev/tty`
  goes out of canonical mode (echo off for `NOECHO`, `VMIN` 1, `VTIME` 0), one byte is read from
  standard input, and the settings are restored. A NUL is the null string, end of input `'FF'x`
  (`getchar`'s `EOF` stored in a `char`). New `Interp::input_is_process_stdin`.
- **`SysCreatePipe`**: `std::io::pipe`, then `FD_CLOEXEC` cleared on both ends with
  `rustix::io::fcntl_setfd` (`pipe(2)` leaves it clear, and children inherit both ends),
  `O_NONBLOCK` on the read end for a first character `n`/`N` via `rustix::fs::fcntl_setfl`, and the
  descriptors leaked with `into_raw_fd` as the C++ leaks them. 40.920 for another first character,
  the empty string included. A failure writes `perror`'s line to standard error and answers `''`.
- **`SysWait`**: `rustix::process::wait`, safe, answering the raw status; `-1` where there is no
  child or the wait fails (entry 45).
- **`SysFork`**: 88.922 for arguments, then `Loud::unsafe_grant`, a new constructor:
  `routine "SYSFORK" without an unsafe grant for fork(2) is not implemented (Phase 9)`.
- `directives.rs`'s refusal example `SysGetKey` became `SysFork`; the body-less refusal is still
  covered there by `SysOpenEventSem`.
- `rustix` gains `termios` (library) and `pty` (dev-dependency only). No new crate, `Cargo.lock`
  unchanged.

## SysFork (not implemented)

`nix::unistd::fork` and `rustix::runtime::kernel_fork` are both `unsafe fn`, and `std` has no fork
without exec, so no safe API gives `fork` semantics. The closest safe form is
`std::process::Command`, which spawns a new program and cannot continue the Rexx program in a child.
Every run here has at least two threads, the caller's and the interpreter thread `run_program`
spawns (`lib.rs:3424`), so a forked child would hold one thread and whatever locks the others held.

Measured on the oracle (`probes/fork-plain.rex`, `probes/fork-threaded.rex`):

- plain: 3 runs of 3 rc 0, the child runs and `exit 3`, the parent's `SysWait` answers `768`;
- with a second activity spinning: 5 runs of 5 killed at the 20 s deadline (rc 124). The child
  printed its line in 3 and nothing in 2, and never ended; all 5 children outlived their parents
  and were killed by hand. Recorded as `oracle-crashes.txt` entry 46 (INDEFINITE BLOCK).

The grant question is the controller's to take to Moritz.

## Review Focus 4

- **`SysCreatePipe` from two activities at once**: the witness's last section starts two activities,
  each creating a pipe and running a command that writes a line into the write end and reads it
  back from the read end through `/proc/self/fd/N` (which needs the inherited descriptors). Oracle
  3 runs identical; crate identical normally and under eight simulation runs (fifo, uniform 0.2,
  0.5 with `gc=0.5`, 1, pct 2 and 3, pre 1 and 2): `witness-sim.txt`.
- **`SysWait` from two activities at once**: `tests/rexxutil_process.rs` runs main plus two started
  activities each calling `SysWait` with no child, normally and under `sim:1`, `sim:2,uniform:1`,
  `sim:3,pct:3,k=4`: `-1` three times each. It runs `rexx-run` as a process because `SysWait` would
  reap the children of other tests in a shared test binary. For the same reason `SysWait` is not in
  the corpus witness: the corpus harness runs programs on threads of one process.
- **`SysWait` reaping a command's child** (`probes/wait-steals.rex`, `probe-runs.txt`): the oracle
  is undefined both ways (entry 45). The crate answers the wait's status `1280` and the command
  `rc 127` with a `FAILURE` (2 runs of 2). Under `sim:1` the command finishes on the baton first,
  so the wait answers `-1` and the command `5`.
- `concurrency_tests`' S2 list: none of these routines is an S2 feature (`S2_FEATURES`), so it does
  not apply.

## Tests and results

Commands from `rust/`, foreground, statuses unpiped. Records in
`docs/superpowers/records/2026-10-13-phase-9b/task-7/`.

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 at `022590321`'s tree |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 |
| lib + tables | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib --test closed_phases --test internal_routines --test owners --test phase_ten_reasons --test refusal_dispositions --test refusal_sites --test no_process_state --test rexxutil_wait` | exit 0 at `6220bd401`'s tree: lib 1115, then 8, 2, 6, 5, 3, 5, 1, 1 passed |
| refusal sites | `REXX_REFUSAL_SITES_REFRESH=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test refusal_sites` | exit 0; one row added (`unsafe_grant`, `body`), the other `lib.rs` rows' line numbers moved |
| process tests | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test rexxutil_process` | 2 passed (after the rename) |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 (`sourceline.txt`) |
| corpus | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0, `1040 of 1040 matching` (`corpus.txt`) |
| collect_stress | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test collect_stress` | exit 0, 37 passed |
| suite group | `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/security.manager/SecurityManager REXX_SUITE_TABLE=<records>/suite-table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | exit 101 at `6220bd401`, wall 23 s: 23 of 29 passed; failed the five 9d-set tests and `TEST_REQUIRES_HANDLE` |

Not run, per the dispatch: the full suite and callgrind. The change adds routine bodies, one input
accessor and one refusal constructor, and touches no dispatch, allocation or per-call path.

Derived rows: `grep -n -a -i "security\|rexxutil" corpus/sim-gate.tsv corpus/sim-exempt.tsv` prints
nothing, and `concurrency_tests.rs` names `SecurityManager` only inside a hand-written program
(line 1400), not as a whole-group row. `suite-9d-set.txt` is unchanged (concern 2).

### Before and after

RED: Task 2's baseline (`task-2-baseline/tests.tsv`) had SecurityManager at 17 passed, 7 blocked
(six on `SYSDROPFUNCS`, one on `SysSearchPath`), 5 failed. GREEN: 23 passed; of the six this task
unblocked, five pass and `TEST_REQUIRES_HANDLE` fails.

### Witness

`rust/corpus/lang/rexxutil_process.rex`, in `phase-9.txt`, sourceline file generated by the test's
driver from an empty directory. From fresh directories, stdout identical and stderr identical but
for the run directory in the error line, rc 216 on both (`witness.oracle.out`/`.err`). The oracle's
stdout was read: each line prints what its label says, among them `5.3.0 1`, `pid 1 1 1`,
`ppid 1`, `pgid of the parent 1`, `pprio 1`, the masked `PTIME` line, the priority steps
(`0 1`, `thread 1`, `0 1`, `0 19`), the `clear` bytes between `before` and `after 0`,
`dumb 256`, `unknown 256`, `no clear 32512`, `[] [] [] []`, the descriptor flags `00 01` and
`04000 01` (no `O_CLOEXEC`), `0 1 first through a pipe` and `0 1 second through a pipe`, and every
argument error.

### Mutations

Each applied by an exact edit, reversed from a copy, confirmed with `cmp` and `git diff --stat`:

| mutation | caught by |
|---|---|
| `ICANON` left on | both pseudo-terminal tests ("the mode never changed") |
| `ECHO` left on for `NOECHO` | `noecho_reads_one_key_and_echoes_nothing` |
| settings not restored | both pseudo-terminal tests |
| no millisecond carry in `PTIME` | `ptime_carries_milliseconds_and_splits_the_clock` |
| `FD_CLOEXEC` kept (built `--profile mutation`) | witness: flags `02000000 02000001` |
| `PGID` as the program's own group | not the witness (the program and its parent share a group there); `pgid_is_the_parents_group`, which starts `rexx-run` in a group of its own |

## Concerns

1. **`SysFork` needs an `unsafe` grant** (above). Until then it refuses loudly.
2. **`TEST_REQUIRES_HANDLE` fails, and the cause is not RexxUtil.** The security manager changes the
   short name's `NAME`; the oracle's `PackageManager::loadRequires` (`package/PackageManager.cpp:723`
   and `:756`) resolves the file from the original name and sends the second `REQUIRES` event with
   that resolved name, where this crate resolves from the changed name (`probes/requires-handle.rex`:
   the oracle's second `NAME` is `.../R_handle.cls`, the crate's `.../R_replaced.cls`; both load the
   replaced file). The test asserts the last event's name ends with the original. The fix is in the
   shared package-loading path, so per "mechanical batches" it is reported, not widened. Either a
   fix round or a `suite-9d-set.txt` line closes the group.
3. **Entry 45**: `SysWait` with no child answers `-1` where the oracle answers stack contents (a
   different number each run). Not licensed yet; it wants a ruling.
4. **`PTIME` and `PMEM` come from `/proc`**, not `getrusage`. `PTIME` therefore moves in clock ticks
   (10 ms here) where the oracle's moves in microseconds, and `PMEM` is `VmHWM`, which leaves out the
   high-water mark the kernel carries over from before `exec`. Neither value repeats between runs,
   so no program can compare them with the oracle's.
5. **`SysSetPriority` and `PPRIO` act on the calling thread** (Linux's `PRIO_PROCESS` with pid 0),
   as in the oracle, whose activities each run on a thread of their own. Whether a priority set in
   one activity is read back in another therefore depends on which threads the two activities run
   on, on each side. Not measured.
6. **`SysGetKey` holds the baton while it waits for a key**, and a halt cannot end the wait. The
   oracle holds its kernel lock there too. It also installs signal handlers on every call that
   re-raise into themselves. This crate installs none.
7. **Found, not this task's**: `call o~string('x')` is 35.1 on both sides, but the crate prints
   `Incorrect expression detected at "&1".` where the oracle prints `at "~"` (an unsubstituted
   insert). Also, a `.Stream` opened on `/proc/self/fd/N` for a pipe behaves differently on the two
   sides (`probes/proc-fd-stream.rex`: `lineout` answers `0` on the oracle and `1` here, and the
   oracle's read after a failed open answers `llo`). The witness avoids both.

## Fix round 1

Commit `ad69540d2`. Records in `docs/superpowers/records/2026-10-13-phase-9b/task-7/fix-1/`.

### SysFork: not written, reported

Measured before any code was written (`fix-1/threads.rex`, which lists `/proc/self/task` and runs
no command): under `rexx-run` every program runs with three threads, `rexx-run` (main, joining),
`rexx-interp` and `rexx-timer`, at start and after a `SysSleep`. The oracle runs one, `rexx`.
"Refuse if threaded" would refuse every call. I sent the controller the options (quiesce and
restart the timer around the fork and give the child its own end path, or change when the timer
starts, or keep refusing), and `SysFork` still refuses until it rules.

### REQUIRES under a renaming security manager

`Interp::load_requires` (`install.rs`) binds `original` before asking the manager. The search
(`resolve_requires`, `resolve_search`), every 43.901, and the cache entries use it. The manager's
short-name answer is only looked up in the cache. The cache key follows
`InterpreterInstance::loadRequires` (`runtime/InterpreterInstance.cpp:1060`, which caches the
original short name). Without it, a later load of the replaced name found the first load's entry,
where the oracle finds nothing and answers 43.901. The first version of the fix (search and
errors only) left that case wrong in the witness.

Witness `rust/corpus/lang/requires_manager_rename.rex`, in `phase-9.txt`, with its sourceline file
generated by the driver from an empty directory. Two cases: an existing file renamed to another
(the replaced file loads, and both events name the original), and a missing file renamed to an
existing one (43.901 naming the original, with one event). The oracle's stdout
(`requires-witness.oracle.out`) was read, and every line prints what its label says. The crate is
byte-identical on stdout and stderr, rc 0, and leaves its run directory with only the program.
RED: the pre-fix release binary printed `event name ends with the original 0` and loaded
`rq_replaced.cls` for the missing name.

Not changed: the oracle consults its instance cache under the original names **before** asking
the manager (`InterpreterInstance.cpp:1024`, `:1036`). The crate asks the manager first, so a
repeated load of a cached package sends `REQUIRES` events where the oracle sends none. That was
not measured against the suite and is outside this round.

### Docs and tests

- `wait`'s doc says it reaps any child, another activity's command or another interpreter's, as
  the oracle's `wait` does.
- `cls`'s doc lost the ` -- ` clause and its measurement. `rexxutil.rs`'s module doc no longer
  ends a line in `--`.
- New `a_failed_read_answers_none_and_restores_the_settings`: `read_key` with a pty for the
  terminal and a pipe's write end as input, which fails. It asserts `None` and restored modes.
  Mutation: restore only after a successful read turns it red (and only it). Reversed from a
  copy, confirmed with `cmp`.

### Checks

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 |
| lib + tables | as in the first round, `--test rexxutil_process` for the renamed binary | exit 0: lib 1116, then 8, 2, 1, 6, 5, 3, 5, 2 (`fix-1/tables.txt`) |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 |
| corpus | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0, `1041 of 1041 matching` (`fix-1/corpus.txt`) |
| package_requires | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test package_requires` | exit 0, 24 passed |
| suite group | as in the first round, table in `fix-1/` | exit 101 (the group stays red on the 9d set): **24 of 29**, failing exactly the five `suite-9d-set.txt` tests |

### SysFork deferred (Moritz, 2026-10-13)

Commit `5a403729f`. `SysFork` stays a loud refusal with no `unsafe` and no fork site.
`Loud::unsafe_grant` became `Loud::fork_child`, which says `routine "SYSFORK", whose forked child
cannot continue without the timer thread and the joining main thread, is not implemented (Phase 9)`.
Its doc points at `.superpowers/sdd/queued/2026-10-13-sysfork-design.md`, and so do `fork`'s doc
and `oracle-crashes.txt` entry 46. `refusal-sites.tsv` was refreshed (one row renamed), the
`directives.rs` example was updated, and fmt, clippy, the lib (1116) and the table binaries
(closed_phases, internal_routines, owners, phase_ten_reasons, refusal_dispositions, refusal_sites,
no_process_state, rexxutil_process) all exit 0.

### The cache before the manager

Commit `fb6dbddeb`. Records in `docs/superpowers/records/2026-10-13-phase-9b/task-7/fix-1b/`.
`load_requires` now looks up the name as written, then the file that name resolves to, before it
asks the security manager. A hit sends no `REQUIRES` event, as in
`InterpreterInstance::loadRequires` (`runtime/InterpreterInstance.cpp:1021`-`:1045`). The resolved
original is cached beside the final resolved name. The change is 32 lines added and 14 removed in
`install.rs`.

The witness `lang/requires_manager_rename.rex` gains a section that loads `rq_twice.cls` twice
and then by its absolute path under one counting manager. The oracle prints `2 events` after
each load. The pre-fix binary (built `--profile mutation` from the copy before this change)
printed `2`, `3`, `4`. The crate is now byte-identical on stdout and stderr. The sourceline file was
regenerated by the driver.

Checks at `fb6dbddeb` (`fix-1b/`):
- fmt and clippy exit 0.
- The lib (1116), closed_phases, owners, refusal_sites and rexxutil_process tests exit 0.
- sourceline exits 0, and package_requires exits 0 with 24 passed.
- The corpus is `1041 of 1041 matching`.
- SecurityManager is 24 of 29, failing exactly the five `suite-9d-set.txt` tests.
