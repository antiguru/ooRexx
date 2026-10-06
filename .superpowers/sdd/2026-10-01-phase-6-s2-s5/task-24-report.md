# Task 24 report (implementer s5-t24)

Base 4936f24c3. Scratch and target dirs under `/tmp/claude-1000/p6-t24/`.

## Commits

- 53e6b9bd6: the task's code, rows, tests, corpus move, enumeration scripts.
- (second commit): gate record criterion 8 subsection, enumeration `output.txt` at 742913850,
  this report and the mutation evidence (`task-24-mutations/`).

## What was done

- P78: `run.rs` LEGALITY comment and `handle.rs` `ObjRef` doc reworded, no Phase 6 owner. The
  doc now says only `Islanded` carries a handle to another thread; checked against
  `grep -rn "unsafe impl.*Send" rust/crates --include=*.rs` (island.rs:56 `Islanded`; ffi.rs
  `ValueDescriptor`, `Value`, a test's `Shared`, none holding an `ObjRef`).
- The test `the_guard_instructions_answers_and_the_phase_6_refusals` renamed
  `..._and_their_refusals` (its name was false once Phase 6 closed).
- Divergence rows 14-23 (`phase-4-exclusions.txt:1574-1748`), owner none:
  14 inverted pinned wait, 15 immovable REPLY, 16 wrapper keeps the baton, 17 HALT under
  nesting, 18 pinned busy-waiter, 19 callback waits for a baton-keeping call (P55/P57),
  20 a wait nothing left can end (P27/P65/P77), 21 program end (P39/P40/P75, u5),
  22 one schedule where the oracle races (P32/P36/P38/P41), 23 thread migration and Error 11
  on a pool thread (spec section 11). Row texts from `s5-find-refusals.md` with these changes:
  row 16 drops "Counted by the pinning report" (the report counts parks and yields, not a
  wrapper's block; `pinning.rs` `PinReport`), and rows 16, 17 gained a measured program and a
  witness.
- New witnesses (P76), each oracle-measured 30 runs per side, probes and counts in
  `docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-24-divergences/`:
  - `scheduler::tests::a_halt_of_an_activity_below_a_pinned_region_waits_for_the_region`
    (row 17; oracle `halt sent|A halted|B done` 30/30, ours `halt sent|B done|A halted` 30/30).
  - `scheduler::tests::pool::a_redirected_command_stops_every_other_activity` (row 16; oracle
    `w 1..3` then `main 0` 30/30, ours `main 0` first 30/30).
  - `scheduler::tests::a_notifier_failing_inside_a_pinned_notifier_wait_replaces_the_outcome`
    (row 22, P38; y1 oracle 28/30 `outer waited 1 42.3`, 2/30 ours' `97.1`; y2 oracle rc 0
    30/30, ours rc 214 30/30).
  - `scheduler::tests::a_termination_uninits_reply_continuation_is_not_waited_for` (row 21,
    P75; u5 oracle prints the third line 23/30, ours 0/30).
  - Existing witnesses cited for rows 14, 15, 18-23 (names in the rows).
  - Rows with a part that has no witness say why: row 19's hang (blocks the test thread in
    the native's join), row 22's HALT landing site (P32, a per-activation count this crate
    does not keep), row 23 is not oracle-measured.
- GUARDED row (`:4253`): DELIVERED note citing the findings' 30-run measurement and the Task
  11 corpus witnesses. Layout row: "or Phase 6" deleted (`REFUSING_MEMBERS` names Phase 9 only).
- P79: the false pass shown first. With `CLOSED` gaining `"Phase 6"` and the old tokenizer,
  `cargo test --release -p rexx-exec --test closed_phases`: 5 passed
  (`/tmp/claude-1000/p6-t24/closed-a.log`), with the Alarm/Ticker row's `OWNER: Phase 6`
  unresolved. With `_` joined: `no_open_exclusions_row_names_a_closed_phase` FAILED naming
  that row (`closed-b.log`). Case H added to `the_row_check_tells_an_open_owner_from_a_resolved_one`.
  Then the row's `CLOSED_PHASES` sentence (false once 6 closes) deleted and a DELIVERED note
  added citing `task-24-gate-table-c.txt` (gate table C at the Task 24 tree: `Alarm instance
  ... agree=7`, `Ticker instance ... agree=6`, 22 passed).
- `closed_phases`: CLOSED gains Phase 6, the negative control counts `"Phase 10"` alone;
  `internal_routines` PHASES is `["Phase 10"]`; gate table C `CLOSED_PHASES` gains `"6"`.
- P17: `phase-8.txt:366-557` moved to the new `phase-6.txt` (the "Filed here because ..."
  sentence dropped, a header written); same program list
  (`diff <(awk '!/^#/ && NF' phase-6.txt) <(awk 'NR>=366 && !/^#/ && NF' old phase-8.txt)`
  empty). `"phase-6.txt"` added after `"phase-5j.txt"` to SUBSET_FILES in corpus.rs,
  coverage.rs, ir_recorded.rs, trace_oracle.rs, collect_stress.rs (`grep -rn 'phase-7\.txt'`
  over rust/ and p6-gates found only these five). `phase-8.txt` stays last, so
  `dispatch/library/tests.rs:177`'s sentence about its order is unchanged.
- Enumeration scripts: `docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-24-enumerations/`
  (`run.sh`, `methods.py`, `probe.sh`, `method-probes/`, `control/`); `output.txt` from
  `bash .../run.sh <release rexx-run> /home/moritz/dev/repos/ooRexx` at 742913850 (the lead's
  ledger commit on top of 53e6b9bd6, which changed only progress.md): S0/S1 commands empty
  (refusal-sites `0`), method join no MISSING (30 rows), method probes print nothing, control
  prints the MAKEARRAY refusal. `method-probes/` are the refusals agent's `probes/` with its
  four `probes2/` rewrites (msg_reply, msg_replywith, ev_uninit, mx_uninit).
- Gate record: `### Criterion 8` appended to `## S5` of `phase-6-gate.md`.

## Witnesses shown red (P76, report only)

A `git archive` of 53e6b9bd6 at `/tmp/claude-1000/p6-t24/mut` with env-gated mutations
(`T24_MUT=<name>`, patch `task-24-mutations/mutations.patch`), release build in its own target
dir (Compiling rexx-exec seen). Each test run with the variable unset (green) and set; logs
`task-24-mutations/mutations-{1..5}.log`.

| mutation | what it reverts | red |
|---|---|---|
| inv | an inverted wait refused as "nothing left to run" | an_inverted_pinned_wait_is_refused, an_inversion_the_refusing_loop_set_aside_is_refused_as_inverted |
| reply | the immovable-REPLY check skipped | the_guard_instructions_answers_and_their_refusals; with `--features pinning`, an_immovable_reply_is_counted_with_its_frames |
| redir | a redirected command leaves the baton like an unredirected one | a_redirected_command_stops_every_other_activity |
| halt | `Message~halt` delivered to the running activity | a_halt_of_an_activity_below_a_pinned_region_waits_for_the_region |
| busy | inverted yields not counted (`--features pinning`) | pinned_busy_waiters_that_need_each_other_count_inverted_yields |
| lone | a lone native call leaves its driver | a_lone_call_keeps_the_baton_after_its_callback_starts_an_activity |
| unsat | the wait nothing can end answers 11.1 | the_result_of_a_message_never_sent_is_refused, a_pinned_wait_nothing_can_end_is_refused, the_guard_instructions_answers_and_their_refusals |
| term | program end waits for activities termination UNINITs start | a_termination_uninits_reply_continuation_is_not_waited_for, an_activity_a_termination_uninit_starts_is_not_waited_for |
| p40 | program end waits after a refusal in main | a_refusal_in_main_does_not_wait_for_the_other_activities |
| replace | a failing notifier does not replace the started send's failure | a_notifier_failing_inside_a_pinned_notifier_wait_replaces_the_outcome |
| p36 / p36y | the REPLY sender never / always yields | a_reply_yields_at_the_next_boundary_only_where_another_activity_is_ready / a_reply_leaves_its_sender_running_until_an_ordinary_switch |
| stack | the stack-room check off | deep_pinned_recursion_on_a_pool_thread_raises_11, nested_pinned_waits_are_bounded_by_the_stack_remaining (both abort: stack overflow; mutations-3.log) |
| tok | `_` not joined in `closed_phases`' tokenizer | the_row_check_tells_an_open_owner_from_a_resolved_one (case H) |

Two first attempts stayed green and were replaced: `notif` (no yield after a notifier failure)
left the y1/y2 test green, and skipping only the `message_outcomes` insert did too (the message
is also told of the replacement); both in mutations-1/2.log. Under `tok`,
`no_open_exclusions_row_names_a_closed_phase` stays green because the Alarm/Ticker row now
carries DELIVERED; the false pass itself is closed-a.log / closed-b.log.

## Checks (P51), at 53e6b9bd6

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0 (Checking rexx-exec seen).
- `memcap 8G cargo test --workspace --release --no-fail-fast` (test binaries built first without
  memcap; the first attempt with memcap around the build was OOM-killed at 8G while compiling):
  144 test results, 3040 passed, 0 failed, 4 ignored, exit 0.
- `cargo test -p rexx-core --doc`: 3 passed, 5 compile_fail passed.

## Concerns

- Rows 19 and 23 rest partly on rulings and code, not on a fresh oracle run: row 19's rc 134 is
  P57's figure (not re-run), row 23 says it is not oracle-measured.
- Row 22 adds P32 and P36 beside P77's P38/P41 (same licence class, both observable); P44 left
  out (its witness pins agreement, not the cadence difference).
- The probe scripts in `task-24-divergences/` name `/tmp/claude-1000/p6-t24/` paths for the
  binaries; the counts file's header says what they ran.
- `mutations-2.log` shows the `stack` runs with no test lines: the process aborted, which
  `mutations-3.log` shows with the abort message.
