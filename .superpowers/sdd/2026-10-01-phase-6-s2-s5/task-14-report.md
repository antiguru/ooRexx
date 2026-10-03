# Task 14 report: Alarm and Ticker end to end; the ooTest ticker; close S3

Base `adc6c900f`. Commits: `7266ae03c` (code, witnesses, tests), and the docs/S3-test commit that
carries this report (sha in the reply).

## Changes

- `ir/drive.rs` `Interp::drive`: a nested driver lends out the caller's pushed call arguments
  (`mem::take` of `value_buffer`, restored after). Every clause of the nested body empties the
  buffer (`enter_clause`), so a Rexx `STRING` method run by a concatenation in a non-first argument
  dropped the arguments before it: the queued "a compiled call op does not name a call of its own
  body" (`TEST_BASE_ALARM`'s `self~assertTrue(1, 'a' d)`). Queued note removed with `git rm`.
- Corpus witnesses (`phase-8.txt`, sourceline files from the documented driver):
  `alarm_cancel_waits_for_the_timer`, `alarm_message_target`, `ticker_fires_on_its_replied_activity`,
  `call_argument_concatenates_a_rexx_string_method`.
- `scheduler/tests.rs` `an_uncancelled_ticker_keeps_the_programs_end_waiting` (Review Focus 2).
- `support/oracle.rs`: `Oracle::run_within` (deadline as a parameter; `run_with` passes
  `ORACLE_DEADLINE`). `group_runner.rs`: `ORACLE_TEST_DEADLINE` 30 s for every group-runner oracle
  run; TEST_BASE_ALARM takes 14.3 s on the oracle.
- `concurrency_tests.rs`: `the_alarm_and_ticker_groups_pass_in_both_modes` (gate-only, every test
  of both groups passes against the oracle, unswitched and every); `the_s3_rows_of_the_derived_list_in_both_modes`
  (rows naming an S3 feature and no S2 one), sharing `rows_in_both_modes` with the S2 test, each in
  its own scratch directory.
- `api_group_tests.rs` `the_framework_ticker_runs_without_dash_u` (criterion 2, gate-only).
- `method_bodies.rs` `RECEIVER_OVERRIDES`: Alarm drops the `SysSleep(0.2)` before its cancel (the
  cancel's `guard on when timerStarted` now waits); the sleep after the cancel stays (a second
  cancel would post the oracle's freed semaphore, `oracle-crashes.txt` 25), as does Ticker's.
  `REXX_METHOD_BODIES_REFRESH=1`: "regressions this run: 0. other drift from the committed table:
  0", table unchanged.
- Docs: `phase-6-gate.md` S3 close and the S3-owned owner cells of the S2 table and the
  MethodArgs paragraph; `phase-6-pinning.md` S3 close; L2 rows in `phase-4-exclusions.txt`, the
  roadmap's row 8 (Rung `L2 -> 10`) and `phase-8-gate.md`.

Step 2's comment edit: `api_group_tests.rs` no longer carries a Phase 6 clause (pruned earlier);
`-U` stays where it is. Nothing edited for it.

## Witnesses

Run counts: oracle `( ulimit -v 1048576; LD_LIBRARY_PATH=... timeout -k 5 20 .../rexx FILE )` from
a fresh directory per run; ours `rexx-run` (release, `7266ae03c` code) unswitched and with
`REXX_SWITCH_MODE=every`, fresh directory per run. Each line below: one distinct outcome in every
run, stderr empty, rc 0.

| program | oracle | ours unswitched | ours every | stdout |
|---|---|---|---|---|
| alarm_cancel_waits_for_the_timer | 30/30 | 30/30 | 30/30 | `cancel notified 1`, `alarm 0 1 1` |
| alarm_message_target | 30/30 | 30/30 | 30/30 | `main 0`, `ring 1 x 1` |
| ticker_fires_on_its_replied_activity | 30/30 | 30/30 | 30/30 | `tick 1 1`, `tick 2 1`, `cancel notified 1`, `main 1` |
| call_argument_concatenates_a_rexx_string_method | 30/30 | 30/30 | 30/30 | `string`/`3 1 a S 3` and three more pairs |

Under `collect_stress` (one run each, `phase-8.txt`): passes (below). The call-argument program at
the base: `rexx-exec: a compiled call op does not name a call of its own body`, rc 120.

Uncancelled Ticker on the oracle: `main done`, `ticked`, killed at 3 s with rc 137, 3 of 3. Ours:
2 clock ticks of CPU (utime 1, stime 1, CLK_TCK 100) over 3 s, killed, same stdout.

No oracle crash met.

## Commands and results

At `7266ae03c` (`checks/run.sh` in scratch, statuses unpiped):

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; `cargo clippy -p rexx-exec
  --all-targets --features pinning -- -D warnings`: exit 0 (both re-run after the S3 test, exit 0).
- `memcap 8G cargo test -p rexx-exec --lib`: 941 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: `787 of 787
  matching`; with `REXX_CORPUS_SWITCH=every`: 787 of 787; debug with every: 787 of 787.
- `collect_stress` release 36 passed, debug 36 passed.
- `refusal_sites` 5, `method_bodies` 23, `gate_table_c` 22, `dispatch_seam` 6 (release, exit 0);
  `rexx-parse` `sourceline_oracle` 1 passed.
- `REXX_CORPUS_GATE=1 ... --test api_group_tests`: 24 passed.
- `REXX_CORPUS_GATE=1 ... --test concurrency_tests -- --test-threads=1`: 31 passed, 829 s
  (`the_alarm_and_ticker_groups_pass_in_both_modes` and `the_s2_rows...` among them).
- After the S3 test: `REXX_CORPUS_GATE=1 ... -- group_runs::the_s3_rows`: 1 passed, 12 s; and
  release `concurrency_tests owners licensed_divergences closed_phases coverage keyword_assertions
  bif_assertions builtin_status api_group_tests` (non-gate): all passed.
- Pinning: `RAYON_NUM_THREADS=4 CARGO_TARGET_DIR=<scratch> memcap 8G cargo test --release -p
  rexx-exec --features pinning --test concurrency_tests -- measured:: --nocapture
  --test-threads=1`: 18 passed, 127 s. Tables in `phase-6-pinning.md` S3 close.
- "It works": every `rust/bench-programs/*.rex`, release, head against a base `adc6c900f` build in
  its own target dir (`Compiling rexx-exec` seen): stdout and rc identical but `heapshape.rex`,
  whose differing lines are its timing figures (`build_seconds=`, `gc_pause_seconds=`), which also
  differ between two head runs.

## Tables

In `phase-6-gate.md` S3 close (criterion 1, S2 rows and S3 rows, with owners) and
`phase-6-pinning.md` S3 close (arrivals per park and waits by kind, both modes). Changes against the
S2 close: `TIME` TEST_3/8/9 and `Alarm` TEST_BASE_ALARM now compared (30 s deadline) and pass;
`TIME` TEST_5/11 differ like TEST_4 (elapsed clock, queued); `TEST_REPLY_SAME_REPLYASSERT` under
every counts 6 (S2: 5), an oracle-observed count. No immovable REPLY and no inverted wait in either
pinning table; per-test outcomes equal across the modes but TEST_EXCLUSION (P46).

## Concerns

- `STREAM` TEST_QUERYDIR_EXISTS failed the S2 rows test in bggates `7ac8abb5f` G4 (every run rc 1);
  it passed in both modes here. The test waits out a wall-clock second boundary
  (`STREAM.testGroup:866`) and compares a directory timestamp, so it can flake under load. Not
  traced.
- Ticker TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE (three triggers of 270 ms within 1 s)
  is wall-clock sensitive and now gated in both modes by the Alarm/Ticker test; it passed in every
  run here.
- The 30 s runner deadline applies to every group-runner oracle run (api_group_tests included);
  it only lets slower tests finish.
- The `the_framework_ticker_runs_without_dash_u` test runs on this crate only (no oracle), as the
  plan specifies.

## Fix round 1

Review `task-14-review.md`; ruling P48. Commit `b461bf174`.

- P48 / I1: `concurrency_tests.rs` `WALL_CLOCK`, the review's group 1, each row with its group
  lines in a comment. `rows_in_both_modes`: a listed row whose two crate runs differ (or one is
  stuck) runs both again once and the second result is judged; the cell ends `(P48 rerun)` and
  `P48 rerun: <row>: ...` goes to stderr. `the_alarm_and_ticker_groups_pass_in_both_modes`: a
  listed row that is not `Pass` in a mode runs again once through `run_tests` (oracle and ours in
  that mode); it fails only if the rerun is not `Pass`; the rerun is logged. Unlisted rows strict.
  Group 2 recorded in `phase-6-gate.md` S3 close, "Wall-clock rows (ruling P48)".
  The `P48 rerun:` line is on stderr, which `cargo test` shows only for a failing test or under
  `--nocapture`; the table cell carries it for the row tests in any case.
- Live proof, temporary env-gated injections (removed; `/bin/grep -c T14_` on both files 0):
  - `the_s3_rows` with a mismatch injected on the first judgement of TEST_TICKER_TWO_ARGS_STRING_CANCEL
    (listed): exit 0, `P48 rerun: ... injected []`, cell `same (P48 rerun)`.
  - Injected on both judgements of that row: exit 101, `tests differing between the modes:
    [... TEST_TICKER_TWO_ARGS_STRING_CANCEL]`, cell `injected again (P48 rerun)`.
  - Injected on TEST_TICKER_NO_ARGS (unlisted): exit 101, no rerun.
  - `the_alarm_and_ticker_groups...` with TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE (listed)
    treated as not passing on its first run in each mode: exit 0, rerun logged `pass` in both modes,
    89 s. TEST_TICKER_NO_ARGS (unlisted): exit 101, `not passing: ["Ticker timer-table
    TEST_TICKER_NO_ARGS pass", "Ticker timer-table-switched TEST_TICKER_NO_ARGS pass"]` (the label is
    the real, injected-over outcome).
- M1: `alarm_message_target.rex` alarm 0.5 s, sourceline file regenerated. Oracle 30/30, ours
  unswitched 30/30, every 30/30: `main 0`, `ring 1 x 1`, rc 0, stderr empty.
- M2: the S2 MethodArgs paragraph restored to its S2 text. M3: `phase-4-exclusions.txt` line
  wrapped (gate line 271 went with M2). M4: "each ticker line carries dots"; "waits out its
  alarms, about two seconds each". M5: "a fresh deadline".
- Uncancelled ticker: deadline 2 s (three 50 ms triggers needed). Early-end mutation
  (`run_started_activities` returning at once, env-gated, from a copy, restored): fails at
  `scheduler/tests.rs:1162` (the elapsed assertion) in 0.02 s; restored, passes in 2.01 s.

Checks at `b461bf174`: fmt 0; clippy workspace and pinning 0; lib 941 passed; corpus 787 of 787
(release unswitched, release every, debug every); collect_stress 36/36 release and debug;
`dispatch_seam` 6, `gate_table_c` 22, `method_bodies` 23, `refusal_sites` 5, `sourceline_oracle`
1; `api_group_tests` gate 24; `concurrency_tests` gate `--test-threads=1 --nocapture` 32 passed,
840 s, no `P48 rerun` line, S2 and S3 tables equal to the committed S3 close.
