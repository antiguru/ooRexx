# Phase 9c Task 6 report: the `rexx-run` switch

Implementer: c9c-impl-6. Base `c8f0420c4`.

## Status

Done. `rexx-run.rs` is deleted and every caller uses `rexx`. Both sample client groups stay in
`suite-9d-set.txt` (cause below).

## Enumeration

`git grep -l 'rexx-run' -- rust` at the start listed 23 files: `rust/CLAUDE.md`,
`bench-baselines/PINNED.md`, `bench-programs/wallclock.sh`, `corpus/README.md`,
`corpus/phase-4c.txt`, the five `rexx-bench` sources (`rexx-arms.rs`, `rexx-bench-band.rs`,
`rexx-bench-suite.rs`, `rexx-bench-suite/report.rs`, `child.rs`), `rexx-exec/src/bin/rexx-run.rs`,
`rexx-exec/src/lib.rs`, fifteen files under `rexx-exec/tests/` (`buffer_growth_error_exit`,
`datetime_zone`, `input_oracle`, `instance_copy_memory`, `library_routine_memory`,
`method_bodies`, `package_requires`, `program_end`, `prompt_before_read`,
`release_local_reference`, `rexxutil_process`, `signals`, `sim_processes`, `stdin_contention`,
`support/arity.rs`, `support/group_runner.rs`) and `rexx-parse/tests/deep.rs`. `suite-9d-set.txt`
holds no `rexx-run` text. Count: 92 matching lines outside the deleted source.

After: `git grep -n 'rexx-run' -- rust` prints 10 lines, all in `bench-baselines/PINNED.md`
(lines 3, 8, 20, 51, 65, 69, 79, 80, 81, 82).

## Kept PINNED.md lines

All ten name a pin that was a `rexx-run` binary (`rexx-run-f558ea501`, `rexx-run-15a1ffa98`,
`rexx-run-pre-phase-5`) or say `pinned/` holds `rexx-run` binaries. They record which binary
a historical pin was. The perf pin outside the repo keeps its name.

## Step 1

`git rm` of the source; `sed s/rexx-run/rexx/g` over the other 21 files (every
`CARGO_BIN_EXE_rexx-run`, `target/release/rexx-run`, `--bin rexx-run`, the wallclock stage name,
and the prose). `cargo fmt` then joined one array in `rexx-bench-suite.rs`.

## Step 2 audit

Command: `git grep -n 'usage\|Some(2)\|status.code' -- crates/rexx-exec/tests crates/rexx-bench crates/rexx-parse/tests`.

| Site | Relied on | Result |
|---|---|---|
| `sim_processes.rs:124-132` | rc 2 and the text `rexx-run: REXX_SWITCH_MODE: ...` | `rexx` returns 2 on that path and prints `rexx: REXX_SWITCH_MODE: ...`; text changed by the rename, test passes |
| `signals.rs:1163-1180` | the main thread's name, `/proc` comm of the binary, `rexx-run` | now `rexx`; the list is sorted by name and `rexx` sorts first, so the expected order was rewritten (this test failed once before the fix) |
| `sim_processes.rs:147` | a symlink named `rexx` to the binary | target now `CARGO_BIN_EXE_rexx` |
| `arity.rs:399`, `rexx-bench-band.rs:143`, `rexx-bench-suite.rs:601-658` | literal `target/release/rexx-run` | `target/release/rexx` |
| `rexx-parse/tests/program.rs:234` `Some(2)` | not `rexx-run` | unaffected |
| other `status.code()` readers | program exit codes | unaffected |

No assertion relied on rexx-run's usage text or on rc 2 for a missing argument.

## Step 3: samples groups

Each run: `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=samples/G REXX_SUITE_TABLE=<own dir>/t.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`.

| Group | exit | oracle | crate |
|---|---|---|---|
| samples/scserver | 0 | 2 of 2 | 2 of 2 |
| samples/sfserver | 0 | 2 of 2 | 2 of 2 |
| samples/scclient | 101 | 2 of 2 | TEST_04_SCCLIENT.REX failed |
| samples/sfclient | 101 | 2 of 2 | TEST_04_SFCLIENT.REX failed |

Cause of both: the client test starts `rexx scserver`/`rexx scclient`, which `::requires
'socket.cls'`, which does `::requires "rxsock" LIBRARY`. The crate answers `Error 98.903: Unable
to load library "rxsock"` (reproduced by hand with the crate `rexx` and `REXX_PATH` at
`build/bin`). The crate has no rxsock library. This is independent of the switch.
`suite-9d-set.txt` is unchanged: the two client lines stay, the servers were never in it.

## Checks

All from `rust/`, `CARGO_INCREMENTAL=0`, runs under `memcap 8G`, unpiped status.

| Command | Exit | Result |
|---|---|---|
| `cargo fmt --all --check` | 0 (after `cargo fmt --all`) | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| `cargo build -p rexx-bench` | 0 | |
| `cargo test --release -p rexx-exec` with the 15 touched tests plus `rexx_command` | 101, then 0 | one failure in `signals` (above); after the fix `signals` 37, `sim_processes` 4, `stdin_contention` 1, `collection_arity` 24, `introspection_arity` 26 pass |
| other touched binaries | 0 | `buffer_growth_error_exit` 1, `datetime_zone` 22, `input_oracle` 22, `instance_copy_memory` 21, `library_routine_memory` 21, `method_bodies` 23, `package_requires` 24, `program_end` 2, `prompt_before_read` 2, `release_local_reference` 3, `rexxutil_process` 3, `rexx_command` 1 |
| `cargo test --release -p rexx-exec --lib` | 0 | 1141 passed |
| `--test refusal_sites --test refusal_dispositions` | 0 | 3 and 5 passed |
| `cargo test --release -p rexx-parse --test deep` | 0 | 6 passed |
| `cargo test --release -p rexx-exec --test suite` | 0 | 52 passed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 1065 of 1065 matching, 29 passed |

`rexx_command` passes as one test here because the group gate is off; the group's
`TEST_PASSINGARGS7` red is Task 7's and was not touched.

## Concerns

None beyond the two client groups. `.superpowers/sdd/2026-10-15-phase-9c/progress.md` was already
modified when I started and is not in my commit.
