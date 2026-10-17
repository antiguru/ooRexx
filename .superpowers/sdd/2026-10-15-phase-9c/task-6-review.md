# Phase 9c Task 6 review (c8f0420c4..257a88fd8)

Reviewer: c9c-review-6. Own target dir, memcap on every interpreter run, worktree and target dirs removed.

## Verdicts

Spec compliance: PASS. Task quality: PASS with Minor findings only.

## Checks run

1. `git grep -n -e 'rexx-run' -e 'rexx_run' -- rust`: `rexx-run` appears only at bench-baselines/PINNED.md:3,8,20,51,65,69,79,80,81,82, each naming a historical pin binary (`rexx-run-f558ea501`, `-15a1ffa98`, `-pre-phase-5`) or saying `pinned/` holds `rexx-run` binaries. `rexx_run` hits are two unrelated helpers (`internal_routines.rs:81` const fn, `rexxutil_process.rs:24` test fn), both pre-existing names. No Cargo.toml `[[bin]]` entry (auto-discovered, `src/bin` holds `rexx.rs` and `rexx-ir.rs`), no script or env-var default builds a `rexx-run` path.
2. Step 2 audit: re-ran the greps for `Some(2)`, `code(), 2`, `status 2`, `exit_code, 2`, `ExitCode::from(2)`, `usage` over the three test trees. Only `sim_processes.rs:124` asserts rc 2; `rexx.rs:78,90` still return 2 for the REXX_SWITCH_MODE refusal, and the test passes. `rexx-parse/tests/program.rs:234` and `rexx-time.rs:43` are unrelated. No assertion relies on rexx-run's usage text (`rexx` answers usage with 255, `launcher::USAGE_STATUS`). Complete.
3. Bench tools, release build of `rexx-exec` and `rexx-bench`: `rexx-arms` runs (refuses with no `--build`, as designed); `rexx-bench-band` finds `target/release/rexx` and produces rust-ir rows (rc 101 was my `head` closing the pipe); `rexx-bench-suite` starts and stops at its offset axis because `perf` counters are unavailable here (environmental, before any `rexx` path); `wallclock.sh -r 1` with `nop` and `callgrind.sh -r 1 -p nop` both ran green with the staged `rexx` copy.
4. Signals test: the main-thread name is `rexx`, the list is sorted so `rexx` sorts first and the three expected vectors were reordered correctly. Passes on the commit; with SIGHUP removed from `HALTING` in `signal.rs` (mutation in a detached worktree copy, separate target dir) it fails.
5. Samples: `samples/scserver` green (exit 0, 16 s) through the harness; `samples/scclient` red ("not passed: TEST_04_SCCLIENT.REX", oracle equal). `suite-9d-set.txt:210-211` keeps scclient and sfclient. The cause (no `rxsock` library) is in the report only; the set file does not name it, which the brief does not require.
6. Prose: the diff adds no em-dashes and no set sizes. Comments are current-state apart from the two Minors below.

## Findings

No Critical. No Important.

Minor:
- `crates/rexx-bench/src/bin/rexx-bench-suite/report.rs:185`: "because that is what `rexx` defaulted to at the time" is now a false claim about the binary: `rexx` did not exist as the benchmark target then; the rename turned a true sentence about `rexx-run` into a false one. Say "the tree-walker was the engine then" or delete the clause.
- `crates/rexx-bench/src/bin/rexx-bench-suite.rs:206`: "An unrecognised value is refused rather than defaulted, exactly as `rexx` refuses one." `rexx` reads no engine value (`rexx.rs` has none, and the next comment says so), so the clause is false after the rename. Delete it. (It was already stale for `rexx-run`; the sed made it a claim about a different binary.)
- Report line "92 matching lines outside the deleted source" and "fifteen files" are set sizes in a record, not in code or CLAUDE.md; harmless.
