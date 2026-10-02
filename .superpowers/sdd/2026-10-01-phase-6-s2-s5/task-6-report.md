# Task 6 report: REPLY as a split with frame moves

Status: DONE. Base 88bad9ebc. Code at 6599f7882.

## Design

- `exec_reply` answers `ExecOutcome::Split(value)`. The driver's cold `exec_suspends` arm records
  the value on the replying activation (`Activation::replied`) and loads the clause countdown, so
  the level's next countdown visit (its next `Op::Clause`/`Op::CallingClause`, or `Op::LoopNext`)
  serves the split from `serve_requests` as a slice-shaped exit. No new branch in the
  `Op::Clause` arm; everything added is in cold functions or behind the existing slice exit.
- `drive_levels`' slice arm, finding a split owed on the running activation, runs `split_level`:
  a new activity's record (`Interp::new_activity`) gets the level's register frame (copied into
  its own arena), the constructs it has open from the level's base with their flat loops, its
  clause and level state, and a parked level resumed at the clause op (`DriveStart::Sliced`).
  The level then ends as a body end. The caller's `finish_call` traces `<I<` (the oracle traces
  it at the REPLY), pops the activation, and `release_method_activation` moves the slot frame into
  the continuation's roots with `ActivityRoots::move_frame` (same serial, record rewritten), puts
  the activation (same id, same `invocation`) on the record as its running activation, roots its
  convention there, files it through `Scheduler::spawn` (now taking a record), and requests a
  slice (`RexxActivation.cpp:776`). `finish_call` hands the caller the reply value.
- The continuation's first step (`First::Reply`) announces `>I>` where the first half had, then
  resumes the parked level. Its end goes through `run_started` with `Then::Pass`: an untrapped
  condition settles failed sends and is reported on `.traceOutput` like any started activity's.
- Removed: `DeferredReply`, `Interp::deferred`, `park_reply`, `run_deferred_replies`,
  `resume_reply`, `begin_resume_reply`, `top_level_clause`, `Loud::reply_inside_construct`,
  `PinKind::DeferredReply` (and its `FRAME_PROBES` row), the program-end drain in `execute_on`,
  the scripted `Split` outcome and `TailKind::Resumed`.
- Added: `Loud::immovable_reply` (owner none), `PinReport::immovable_replies` with
  `immovable_reply!`, `Activity::driver_pins`, `ActivityRoots::move_frame`.
- Task 5 carry-in: `measure_stack` asserts the thread stack exceeds `STACK_MARGIN`
  (`a_stack_within_the_margin_is_refused`).

## Rulings

- R-T6-1: the split is served at the replier level's next countdown visit. A boundary that closes
  a construct's branch (an `IF` arm's `Op::EndBranch`, a `SELECT` branch end) is a non-yielding
  visit; it keeps the split owed for the next clause. A body that ends before any such clause
  (REPLY as the last clause it runs) hands the caller the value and spawns no activity, since
  nothing is left to run. Cost if wrong: an empty continuation does not take a pooled activity
  number as the oracle's `spawnReply` does, which only a later `.context~thread` could observe.
- R-T6-2: a REPLY is movable iff the only pin above its driver's entry is its own `Op::Exec`
  (`pin_depth == driver_pins + 1`, `driver_pins` recorded per `drive_from`). The reachable
  immovable shape is a REPLY in a block run on a nested Rust frame (a labelled `DO` block):
  `INTERPRET` cannot hold a REPLY (99.924, both sides) and a `CALL ON` handler is not a method
  activation (99.919, both sides). Without the check such a REPLY would run the rest of its
  construct in the sender's time and split at the next clause after the frames unwind (measured
  by mutation); the spec's loud refusal is kept.
- R-T6-3: the pending-trap hand-over at the split is an assertion, not a move: a trap the REPLY
  clause queued is delivered at that clause's end, before the split.
- R-T6-4: the value a continuation's guarded state shows a later sender (`p04` shape) is right
  here because the REPLY's yield runs the continuation's first clauses first; the oracle's
  reason is the transferred guard lock, which is Task 11's.

## Commits

- 6599f7882 Move a REPLY's continuation to a new activity (Task 6): code, tests, corpus witnesses,
  sourceline files, refusal-sites.tsv, oracle-crashes.txt.
- This report, committed after it.

## Tests and evidence

Red before (base 88bad9ebc, release binary from `git archive`, `Compiling rexx-exec` seen):
`reply_then_wait`, `reply_continuation_error_trace`, `reply_continuation_error_stderr`,
`reply_continuation_failed_send` hang (rc 137 at 20 s: the continuation ran only at program end);
`reply_inside_constructs` refuses (rc 120, "a REPLY inside a DO, SELECT or IF");
`reply_seen_by_a_guarded_send` prints `after none`.

Mutations (each restored from a copy, binary rebuilt):
- continuation convention not parked: `a_parked_reply_keeps_its_variables_across_a_collection`
  (collect_stress, release) panics "a live value". The test was first green under this mutation
  (the argument stayed in the sender's register); it now sends from a routine that returns while
  the continuation sleeps.
- no slice request after filing: `a_reply_yields_to_its_continuation_at_the_senders_next_clause`
  and the interleaving test red.
- `immovable_reply!` removed: `an_immovable_reply_is_counted_with_its_frames` red (0 vs 1).
- pin check disabled: `the_guard_instructions_answers_and_the_phase_6_refusals` red.
- value dropped when the body ends before its split: corpus 718/718 stayed green (no witness);
  `reply_as_the_last_clause.rex` added, red under the mutation (91.999).
- `settle_failed_sends` removed from `run_started`: `reply_continuation_failed_send` differs
  (rc 159, no condition object). This is Task 3's M3: a failed send in the continuation settles
  on its message.

New crate tests: `a_reply_splits_once` (drive), `a_reply_yields_to_its_continuation_at_the_
senders_next_clause`, `a_reply_continuation_interleaves_with_its_sender_under_the_switch_mode`
(EveryOpportunity and AtClause 1..=12: each side's order kept, more than one interleaving),
`a_stack_within_the_margin_is_refused` (scheduler), `an_immovable_reply_is_counted_with_its_frames`
(pinning), three `move_frame` tests (rexx-core). The pinning table gains an `immovable` row.

## Oracle stability

30 runs each, one (stdout, stderr, rc) hash per side: oracle, ours unswitched, ours
EveryOpportunity, all one hash and the same hash, for `reply_then_wait`,
`reply_inside_constructs`, `reply_continuation_error_trace`, `reply_continuation_error_stderr`,
`reply_continuation_failed_send`, `reply_seen_by_a_guarded_send`, `reply_as_the_last_clause`.
`p04_reply_in_arg` also under AtClause 1..10: one hash.

Ordering without a synchronising wait is scheduling-dependent on the oracle: `p1` (REPLY, then
both sides print) gave two outputs over 30 runs (21/9); `p04b` (continuation prints) is 30/30 on
the oracle with the sender first and differs here (continuation first, R-T6-1's yield). These are
crate-side tests only.

## Queued items

- `2026-10-01-reply-continuation-state-visibility`: witnessed, `reply_seen_by_a_guarded_send.rex`
  (the `p04_reply_in_arg` probe), oracle 30/30; see R-T6-4. File left in `queued/`.
- reply2 ordering (S1 Task 10 re-review, final review B9): order is scheduling-dependent on the
  oracle; crate-side `a_reply_continuation_interleaves_with_its_sender_under_the_switch_mode`.
- No program in the corpus reads a context across a REPLY (corpus green), so Task 7 Step 2 stays.

## Corpus and records

- Seven witnesses added to `corpus/phase-8.txt`, sourceline files generated with the sanctioned
  driver. Comments rewritten (and sourceline files regenerated) where they described the
  deferral: `started_waited_in_a_replied_body.rex`, `class_context_reply.rex`,
  `method_reply_chain.rex`; list comments in `phase-5a.txt`, `phase-8.txt`.
- `oracle-crashes.txt`: the REPLY-then-FORWARD self-loop re-measured; it still does not
  terminate, and its RSS now grows (ended activities' thread contexts are kept until
  termination).
- `docs/superpowers/plans/phase-6-gate.md:89` ("this crate runs it after the caller") is the S1
  gate record's input list and was left as recorded.

## refusal-sites.tsv

Re-derived with `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites`:
one row removed (`reply_inside_construct`), one added (`immovable_reply`, lib.rs:717).

## Bench "it works"

`rust/bench-programs/*.rex`, release head against the base release binary: stdout and rc
identical for every program but `heapshape`, which prints its own timings.

## P28 checks

- `cargo fmt --all --check`: 0.
- `cargo clippy -p rexx-exec --all-targets -- -D warnings`, and with `--features pinning`, and
  `-p rexx-core`: clean (`Checking rexx-exec` seen after a touch).
- `cargo test -p rexx-exec --lib`: 901 passed. `cargo test -p rexx-core`: green (roots 20).
- `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`: 719 of 719 matching;
  with `REXX_CORPUS_SWITCH=every`: 719 of 719.
- `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test collect_stress`: 34 passed.
- `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test concurrency_tests`: 27 passed; Message
  tables (unswitched and switched) identical to Task 5's raw tables.
- `cargo test -p rexx-exec --features pinning --test concurrency_tests measured::`: 16 passed.
- `refusal_sites`, `deadline`, `coverage`, `sourceline_oracle`: green.
- Corpus (release) while collect_stress (release) ran alongside: 719 of 719; collect_stress 34
  passed. Debug corpus (`cargo test -p rexx-exec --test corpus`, gate): 719 of 719.
