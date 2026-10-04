# Parallel concurrency-gate rows: report

Commits:
- `d63a7e8ae` adds the pool, keeps WALL_CLOCK rows serial, and removes ONE_GROUP.
- `69d16a2eb` runs a group's WALL_CLOCK rows beside each other.

`69d16a2eb` departs from requirement 2. It is a separate commit, so reverting it restores the serial behaviour as specified.

## Design

`tests/support/group_runner.rs` gains `rows_in_parallel(name, rows, quiet, row)`.
- **Scratch dirs.** Each row runs in its own copy, `CARGO_TARGET_TMPDIR/<name>-<pid>-<index>/run`. The copy is removed after the row, when it exists; a row that reaches rxapi never creates one.
- **Order.** Results come back in `rows`' order, and the table loops consume them unchanged.
- **Pooled rows.** One rayon pool, shared by the whole test binary, runs them. It is sized by `REXX_GROUP_ROWS`, default 6, so the bound holds across tests, not just within one.
- **Quiet rows.** The rows `quiet` selects (WALL_CLOCK) run after the pooled ones, under `on_a_quiet_machine`.
- **The two-class gate** (`RowTurn`): a quiet row waits until no pooled row runs anywhere in the binary, and while a pooled row waits. A pooled row waits while a quiet row runs.
  - Pooled rows have priority. An earlier version gave quiet rows priority, and it starved the other tests' pooled rows for the whole S2 wall-clock chain: GUARD and Alarm/Ticker finished at 634 s and 688 s (runs `new6-1` and `timing6`).
- **Callers.**
  - `outcome_table` is used by Message, GUARD, Object, TRACE_TraceObject and Alarm/Ticker. It runs one `run_tests` per test through `rows_in_parallel`. A test is quiet when `"<dir>/<group>.testGroup <test>"` is in WALL_CLOCK.
  - `rows_in_both_modes`' per-row body moved, unchanged, into `row_in_both_modes`. That includes the oracle-deadline `catch_unwind`, the P46 race rule, and the P48 single rerun with its "(P48 rerun)" suffix.
  - The timer test's P48 rerun runs `on_a_quiet_machine`.
- **ONE_GROUP is removed.** Its memory argument was the Object run reaching TEST_UNINIT and TEST_UNINIT_CLASS, which allocate until the cap. `66368f3a7` restricted that run to the start tests, and the pool now bounds the rows in flight binary-wide. The measured peaks below are at most 1.31 GB, against the 8 GiB cap.
- **`69d16a2eb`.** A group's quiet rows each get a thread, still after its pooled rows and still never beside a pooled row.

## Why `69d16a2eb` exists

Per-row timing (instrumented binary `timing6b`, not committed) shows where S2's time goes:
- **Pooled rows** take about 0.2 s each. The exceptions are the Message START/REPLY rows at 24 s and METHOD TESTGUARDEDACCESS at 10.6 s.
- **WALL_CLOCK rows:**
  - TIME TEST_2, TEST_3, TEST_4, TEST_5, TEST_8, TEST_9, TEST_10 and TEST_11, at 29 to 75 s each;
  - Alarm TEST_BASE_ALARM, 61 s;
  - MutexSemaphore TEST_EXCLUSION, 60 s;
  - Message TEST_HALT_START, 24 s;
  - SysSleep TEST_SLEEP_CONCURRENT, 20 s.
  - Run serially, these rows sum to about 575 s of S2's 656 s, and almost all of it is SysSleep.
- **Effect on requirement 2.** With WALL_CLOCK rows serial (`d63a7e8ae`), the binary took 663 s against the base's 656 s, so the change as specified gains nothing.

I asked team-lead before committing `69d16a2eb` and had no answer when I committed.

## Bound

- **Per-row memory.** At `REXX_GROUP_ROWS=1`, with one pooled row at a time across the binary, the cgroup's anon peak was 292,016,128 bytes for the whole process (run `new-rows1`). That figure bounds one row plus the process's own footprint.
- **Default of 6.**
  - In the worst case, six such rows come to 1.75 GB, well under the 8 GiB memcap.
  - The measured peaks at 6 were 0.72 to 1.21 GB anon.
  - A wider pool would buy little, because the rows that dominate are not pooled.
- **Peak RSS.** It is the cgroup's `memory.peak` and the sampled maximum of `memory.stat` anon. Both come from `memcap-peak`, which is `memcap` plus that read-out.

## Wall time (release, `REXX_CORPUS_GATE=1`, under an 8G cgroup cap)

| run | binary wall | S2 | S3 | Alarm/Ticker | cgroup peak | anon peak |
|---|---|---|---|---|---|---|
| base, run 1 | 656.3 s | 655.8 | 11.9 | 163.4 | 311.7 MB | 259.0 MB |
| base, run 2 | 658.3 s | | | | 295.0 MB | 257.8 MB |
| `d63a7e8ae` (serial WALL_CLOCK) | 663.3 s | 663.2 | 86.2 | 211.5 | 793.3 MB | 719.6 MB |
| `69d16a2eb`, run 1 | 197.7 s | 159.4 | 85.0 | 197.1 | 1305.7 MB | 1205.1 MB |
| `69d16a2eb`, run 2 | 194.7 s | | | | 1003.9 MB | 974.5 MB |
| `69d16a2eb`, run 3 | 226.8 s | | | | 1149.1 MB | 1115.3 MB |
| `69d16a2eb`, under `cargo test -p rexx-exec --release --test concurrency_tests` | 221 s | | | | 1134.3 MB | 1041.1 MB |
| `69d16a2eb`, loaded | 172.1 s | | | | 1257.4 MB | 1201.9 MB |

How the figures were taken:
- **Per-test columns** are the completion times printed by libtest within the binary run.
- **Alarm/Ticker in base, run 1** includes waiting on ONE_GROUP.
- **S3 inside the binary** waits for every pooled row of the binary before its quiet rows start, so its in-binary time is not its own cost.

Each test run alone (`--exact`):

| test | base | `69d16a2eb` |
|---|---|---|
| S2 | 655.8 (in-binary) | 125.4 |
| S3 | 12.0 | 3.0 |
| Alarm/Ticker | 88.3 | 67.2 |

How the runs were made:
- `$SP/run.sh LABEL BIN [args]` runs the copied test binary from `rust/crates/rexx-exec` with the gate and table variables, under `memcap-peak 8G`. `$SP` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/par-rows`.
- **Base** is `bddcd480e`'s binary (`target-base`).
- **The parallel runs before `d63a7e8ae` was committed** used `REXX_GROUP_ROWS=6` with the then-default of 4 compiled in. Their code is otherwise identical to the commits: the diff was checked with the doc lines stripped.

## Table identity

Every run diffed each table against base run 1: `diff runs/base1/<t>.txt runs/<run>/<t>.txt` for `t` in `s2`, `s3` and `timer`.
- **S3 and timer** came out byte-identical in every run, serial and parallel.
- **S2, between the two serial base runs** (base1 against base2), three rows differ:
  - STREAM TEST_QUERYDIR_EXISTS: oracle 2 assertions against ours 10, then a pass;
  - REPLY TEST_REPLY_RETURN_CODE_REPLYASSERT: a pass, then oracle 1 against ours 0;
  - SysSleep TEST_SLEEP_DURATION: "same", then "same (P48 rerun)".
- **Other serial-to-serial variance:**
  - `bg/ec829d171` g4 against g6, both serial gate runs: REPLY TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT.
  - base1 against `new-rows1` (one pooled row at a time): STREAM TEST_QUERYDIR_EXISTS and REPLY TEST_REPLY_SAME_REPLYASSERT.
- **Parallel runs against base1:**
  - STREAM TEST_QUERYDIR_EXISTS differs in every parallel run.
  - The other differing rows are all MODE_DIFFERING REPLY rows: RETURN_CODE_SAME, EXIT_CODE, STACK and SAME. In each, the oracle's assertion count after the REPLY varies by one (the P41 race).
  - EXIT_CODE and STACK were not seen to vary in a serial run here; their siblings were.
- **Column 4** ("every against normal") is identical in every run apart from the P48 suffix.
- **Unchanged.**
  - Assertions are untouched, and every run passed.
  - "(P48 rerun)" still appears where a rerun happens: base2 shows one, and no parallel run needed one.

## Stability and load

- **Three runs of `69d16a2eb`**: 32 passed each, with the tables as above.
- **Loaded run.**
  - Setup: `cargo test --workspace --release --no-run` into a fresh target dir (`target-load`), started 15 s before the test binary.
  - The build ended 11 s before the tests did, so the two overlapped for about 160 s of the 172 s run.
  - Load average went from 7.65 to 13.11 (1 min).
  - Result: 32 passed. The tables differed only in STREAM TEST_QUERYDIR_EXISTS and REPLY RETURN_CODE_SAME, EXIT_CODE and STACK, all from the variable set above.

## Checks at `69d16a2eb`

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings`: exit 0, run in its own target dir.
- `REXX_CORPUS_GATE=1` under an 8G cgroup cap, `cargo test -p rexx-exec --release --test concurrency_tests`: exit 0, 32 passed, 221.25 s.
- `memcap 8G cargo test -p rexx-exec --features pinning --test concurrency_tests -- measured::`, in its own target dir: exit 0, 18 passed.
- Not run: the debug-profile gate.
