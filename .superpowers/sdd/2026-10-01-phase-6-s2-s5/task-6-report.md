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
  `INTERPRET` cannot hold a REPLY (99.924 and rc on both sides; the oracle's traceback has one
  more first line, the interpreted clause, a recorded pre-existing divergence) and a `CALL ON`
  handler is not a method activation (99.919, both sides). Without the check such a REPLY would run the rest of its
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

## Fix round 1

By s2-t6b, base 939fdace1. Code at ff3637e2a. Probes and scripts: scratchpad `t6b/` (`run.sh`
runs one mode from a fresh empty dir, `stab.sh` hashes N runs, `rss.sh` peak RSS through
`/usr/bin/time -f %M`, `grow.sh` samples `VmRSS` at 1..5 s). "939fdace1" below is a release
build of a `git archive` of that commit in its own target dir (`Compiling rexx-exec` seen).

### I1: the owed split is the activity's

`Activity::splits_owed` counts activations that replied and wait for their split: incremented by
`exec_suspends`' Split arm, decremented by `split_level` and by `release_method_activation`'s
no-split arm. `serve_requests` keeps the countdown at 1 while it is non-zero and serves the
split only where the running activation owes it, so a `CALL ON` handler's clauses no longer
reload the countdown. Witnesses (oracle 30/30, ours 30/30 unswitched and `every`, one hash
each, the oracle's): `reply_split_after_a_trap_at_the_reply.rex` (`trapatreply`),
`reply_split_after_a_trap_takes_the_failure.rex` (`traperr`), `reply_split_after_a_trap_does_
not_wait.rex` (`trapdelay`). Red at 939fdace1: `after reply on main 1`; rc 5 `sender caught`;
`rest done` before `got v after 0`. Mutation (the `splits_owed > 0` test put back to
`split_owed()`): all three red the same way; restored, green.

### I2: an empty rest numbers the replier

`release_method_activation`'s no-split arm calls `activity_number()` and `rotate_pooled()`
(the oldest pooled number to the tail, under `MAX_POOLED`). Witness
`reply_as_the_last_clause_numbers_the_replier.rex` (`r1spawner`): oracle 30/30 `t 3` / `s 2`,
ours the same 30/30 unswitched and `every`; 939fdace1 `t 2` / `s 3`. The pool rotation half
(`r1pool`) is racy on the oracle and has no witness.

### I3 (P34): sender first

`spawn_continuation` no longer sets `SLICE`. Its comment now names the oracle's
`hasWaiters` gate (`ActivityManager.hpp:293`). It keeps `clause_countdown = 1`: the split's own
countdown visit returns before serving any other request, so with REPLYs closer together than
`CLAUSES_PER_CHECK` clauses no visit ever armed the timer. Measured with the line removed:
`memreply100000` (100k REPLYs in a loop) never switched (6000 REPLYs, 48 ms, no continuation
ran) and aborted at 0.10 s, rc 134, allocating an arena block. With it the continuations run at
each 24 ms slice.
- `a_reply_yields_to_its_continuation_at_the_senders_next_clause` became
  `a_reply_leaves_its_sender_running_until_an_ordinary_switch` (`caller 1, caller 2, rest 1,
  rest 2`). The switch-mode interleaving test is unchanged and green.
- `reply_seen_by_a_guarded_send.rex` and its sourceline file left the corpus, to
  `.superpowers/sdd/queued/2026-10-02-reply-guard-transfer-witness.md` with the program, the
  oracle's output, ours (`after none`), and the review's `init.rex` shape. R-T6-4 no longer
  holds: the sender now reads `none`.
- `p04b` unswitched now prints the sender's lines first, the oracle's majority order (23/30 in
  the review).

### I4 (P35): one table, contexts made on first use

- `rexx_api::ffi::ThreadTable`: one heap `RexxThreadInterface`, shared by `Rc`.
  `ThreadContext::linking(&table)` points the context's `functions` at it and keeps it alive;
  `ThreadContext::new()` links a table of its own. The data members are still written at the
  first `enter` while null, now once per table.
- `Activity::thread` is `Option<ThreadContext>`, made by `Interp::thread_context()` (the four
  native-call sites in `dispatch/library.rs` and the two `keep_thread_context` sites in
  `install.rs`) from the table on `Activities`. An ended activity's context is retained only if
  it was made (`retired.extend(thread.take())`).
- Tests: `an_activity_without_a_native_call_retains_no_thread_context` (a started send and a
  REPLY continuation end, `retired()` is empty; red when retirement pushes a default context for
  an unmade one); `contexts_linking_one_table_share_its_constants` (rexx-api).
  `an_ended_activitys_thread_context_is_kept` stays green.

RSS, release, `ulimit -v 8388608`, probes in `t6b/p/` (the review's), two runs each:

| program | 939fdace1 | ff3637e2a |
|---|---|---|
| `memstart100000` (100k `~start~wait`) | 168,008 / 167,828 kB | 41,932 / 42,184 kB |
| `memstartnowait100000` (100k `~start`) | 187,560 kB | 74,052 / 73,920 kB |
| `memreply100000` (100k REPLYs) | 145,608 / 145,632 kB | 901,628 / 881,164 kB |
| `memreply100000`, `REXX_SWITCH_MODE=every` | | 20,136 kB |
| REPLY+FORWARD self-loop, `VmRSS` at 1..5 s | 96, 168, 239, 311, 387 MB | 21, 21, 21, 21, 21 MB |

Commands: `t6b/rss.sh BIN FILE` and `t6b/grow.sh BIN t6b/p/selfloop.rex`.

The REPLY row is worse, and the cause is I3, not I4. Sender-first leaves a slice's worth of
continuations pending, about 1,300 at 24 ms and roughly 18 µs per REPLY. Each pending
continuation holds the arena block `split_level` opened for its registers. The block is
`FrameBlock::DEFAULT` + `GUARD` cells, 1 MB, zeroed. The `every` row shows the contexts are
not the cost: there each continuation runs at once and the peak is 20 MB. The oracle
accumulates the same way. It cannot run this program: `Error 48.1: ERROR CREATING THREAD`,
rc 208, at 3.7 s under its 1 GB `ulimit -v`. `~start` does not pay this, because a started
activity opens its block when it first runs. This makes Task 26's `open_block` item larger,
and it gains a second remedy: the continuation could open its block when it first runs.

### G4: moved-handle refusal in release

`a_moved_frames_old_handle_is_refused` is now `#[cfg(debug_assertions)]` with
`expected = "the cached top record disagrees"`. The moved frame keeps its serial, so in the old
record's position the top-frame fast path of `frame_slot` matches on serial alone, and only
`debug_assert_top` checks the depth. A release refusal would add a depth compare to every
fast-path slot access. rexx-core tests: debug, roots 20 passed; release, roots 18 passed (two
debug-gated).

### Minors

- `2026-10-01-reply-continuation-state-visibility.md` rewritten: the continuation is its own
  activity, and the remaining divergence is the guard, carried to Task 11. Now tracked.
- Queued `2026-10-02-uninit-reply-at-termination.md`: the oracle gave the third line 3/3 this
  round (the review measured it 2/3); ours gives the first two lines, rc 0.
- `oracle-crashes.txt`: the self-loop entry re-measured (flat RSS).

### P28 checks (at ff3637e2a)

- `cargo fmt --all --check`: 0. Clippy `-D warnings`, `--all-targets`: `-p rexx-exec`,
  `-p rexx-exec --features pinning`, `-p rexx-core`, `-p rexx-api`: clean (`Checking` seen).
- `memcap 8G cargo test -p rexx-exec --lib`: 902 passed. Under `ulimit -v 8388608` instead of
  memcap, many lib tests fail at once and pass alone: virtual address space across parallel
  interpreter threads. This is not a regression, the same as Task 5's note.
- `cargo test -p rexx-core`, debug and `--release`: green. `cargo test -p rexx-api`: green.
- `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`: 722 of 722;
  `REXX_CORPUS_SWITCH=every`: 722 of 722.
- `collect_stress` (release, gate): 34 passed.
- `concurrency_tests` (gate): 27 passed. `--features pinning ... measured::`: 16 passed.
- `refusal_sites` 5 passed, `sourceline_oracle` (rexx-parse) passed.
- Corpus (release) while collect_stress (release) ran alongside: 722 of 722; collect_stress 34.
- Bench "it works": every `rust/bench-programs/*.rex` has the same stdout+stderr+rc as
  939fdace1, except `heapshape`, which prints its own timings.

## Fix round 1b

Code at a8173be99. A pending continuation no longer holds an arena block.
- `split_level` parks the replying level's register values on the new record
  (`ActivityRoots::park`) inside a `RepliedLevel` (level, parked values, temps frame) on
  `Activity::replied_level`.
- The continuation's first step (`First::Reply`) calls `Interp::open_replied_level`. It takes
  the values back (`ActivityRoots::take_parked`, new in rexx-core), reserves the frame in the
  continuation's own arena, and pushes the `ParkedLevel` the slice resumes. Opening the block
  there matches what a started activity does at its first frame.
- The frames moved at the split are unchanged: the slot frame still moves in
  `spawn_continuation`, and only the register copy is deferred.

RSS, release, `t6b/rss.sh BIN t6b/p/FILE` (`/usr/bin/time -f %M`, `ulimit -v 8388608`):

| program | 939fdace1 | ff3637e2a | a8173be99 |
|---|---|---|---|
| `memreply100000` (100k REPLYs, unswitched) | 145,608 kB | 881,164 / 901,628 kB | 89,944 / 91,460 / 89,568 kB |
| `memstartnowait100000` (100k `~start`) | 187,560 kB | 73,920 / 74,052 kB | 75,632 kB |
| `memstart100000` (100k `~start~wait`) | 168,008 kB | 41,932 kB | 41,896 / 41,960 kB |
| REPLY+FORWARD self-loop, `VmRSS` at 1..5 s (`t6b/grow.sh`) | 96 .. 387 MB | 21 MB flat | 19 MB flat |

Rooting while pending: `a_pending_reply_keeps_its_registers_across_a_collection`
(collect_stress) replies inside a `DO OVER` whose array only a register holds, and the sender
allocates under collect-every-allocation before the continuation runs. It is green. With the
values kept in the `RepliedLevel` but not parked, it is red: "a message send to a value whose
object is no longer live", rc 120. The existing collect_stress suite stayed green under that
mutation, so this test is the only one that sees the pending window.
`taken_parked_values_stop_being_roots` (rexx-core) covers `take_parked`.

Debug arena assertions under collect-every-allocation: a scratch driver
(`t6b/stressdrv`, debug build of this tree, `run_program_collect_every_alloc`) ran each corpus
REPLY program. Those are `reply*.rex`, `method_reply_chain.rex`,
`started_waited_in_a_replied_body.rex` and the round-1 witnesses. It also ran the review's
frame-move probes: `deep`, `loops`, `ref`, `settings`, `nested`, `fwd`, `started`,
`whileside`, `untilside`, `loopstep`, `parkedin`, `pinwait` and `treesend`. Every run had the
oracle's stdout and rc, with no panic or assertion on stderr and a non-zero collection count.

P28 checks at a8173be99:
- fmt 0.
- Clippy `-D warnings --all-targets` is clean for `-p rexx-exec`, for `-p rexx-exec`
  `--features pinning`, and for `-p rexx-core`.
- Lib tests: 902 passed. rexx-core is green in debug and release.
- Corpus (release, gate): 722/722 unswitched and 722/722 under `every`.
- collect_stress (release): 35 passed. Run alongside it, the corpus gave 722/722.
- Other suites: concurrency_tests 27 passed, pinning `measured::` 16 passed, refusal_sites 5
  passed, sourceline_oracle passed.
- Bench "it works": every program matches 939fdace1 except `heapshape`, which prints its own
  timings.

## Fix round 2

Code at 6d5f6f116.

### O1: a slice inside an unpinned nested driver

The cause is not the continuation. A nested driver (`Interp::drive`, never parkable) can only
defer a slice when a pinned frame sits above it. `resolve_stream`'s `.STREAM~new` send had no
pin, so a stream builtin naming a new stream ran the `Stream` class's `NEW` on a driver with
`pin_depth` 0. The same panic occurs on a started activity and on main whenever another
activity is ready. The review's control only had no ready activity. Measured with temporary
logging, debug build, `REXX_SWITCH_MODE=every` (scratch `t6b/p2/` and inline):
`DBG slice pin=0 drv=0 ready=1` on the continuation, on a started activity with main busy, and
on main with a started activity ready, each followed by the assertion.

To find every such site, I logged a backtrace at each `drive` entry with `pin_depth == 0` over
the debug corpus (instrumentation not committed). The runtime sites:
- `resolve_stream` (`lineout`, `linein`, `lines`, `stream`);
- the `NAME=` send of a message-term assignment target (`assign_expr_target`, from PARSE);
- `security_send` (security-manager checkpoints);
- `run_stored_method` (a directory's `setMethod` entry run by a lookup).

The other logged sites are bootstrap (`mint_local_entries`) and the main program's install
(`install_directives` from `run_loaded`), both before any second activity exists.

Fix:
- **Pins.** `pinned!` around the four sends, with kinds StreamWrapper, TreeSend, the new
  `PinKind::SecurityManager`, and `PinKind::native(name)`.
- **Probe.** `FRAME_PROBES` gains a `SecurityManager` case. It is red with the pin removed,
  because the park is then reported under `Unknown` only.
- **Debug guard.** `Interp::drive` now asserts in debug builds that it runs under a pinned
  frame once any activity has been spawned (`Interp::activities_spawned`). A missing pin then
  fails on every debug run of a multi-activity program, not only where a slice lands.
- **Rooting.** The new pins expose a rooting defect in the stream builtins. A pinned frame
  takes a pinned yield (P29), and other activities allocate during it. `stream()`'s COMMAND
  argument, `lines()`'s argument, and the resolved stream were not rooted. Under debug,
  stress and `every`, the review's `lineoutcont2` panicked "a live value". The stream builtins
  now `push_temp` the stream and the argument.

Tests:
- `a_stream_a_builtin_builds_defers_the_slice_under_the_switch_mode` (lib, EveryOpportunity, a
  REPLY continuation and a started activity, each with another activity ready). Red with the
  `NEW` pin and the drive guard removed: "a slice ended in a driver no pinned frame counts".
- `a_stream_builtins_values_survive_a_pinned_yield` (collect_stress, `every`, collect every
  allocation; the oracle's stdout). Red without the new `push_temp`s, in release and debug:
  "a live value".
- Debug, stress and `every` through the scratch driver `t6b/stressdrv`:
  - All of these match the oracle (stdout and rc, or sorted stdout where the order is
    scheduling-dependent) with no panic: every `reply*.rex` witness; the review's `lineoutcont`,
    `lineoutcont2` and `lineoutcont3`; `settings`, `deep`, `loops`, `ref`, `nested`, `fwd`,
    `started`, `parkedin`, `pinwait` and `treesend`; `pend1`, `hsend` and `two`; and the
    round's `secsleep`, `streamstart` and `streamcont`.
  - `lineoutcont3` and `settings` panicked before the fix; `lineoutcont2` panicked on the
    missing rooting until the `push_temp`s.

### N1 (P36): yield after a REPLY only where another activity is ready

`spawn_continuation` sets `SLICE` when the ready queue was non-empty before the continuation
is filed (`Interp::any_ready`), so the sender yields at its next clause boundary. With nothing
ready it runs on as before. Its comment names both rulings.
`a_reply_yields_at_the_next_boundary_only_where_another_activity_is_ready`:
- It runs under `AtClause(1_000_000)`, which keeps the timer out.
- `two.rex` gives `ra, rb, a rest, b rest, main 1, main 2`. The first REPLY does not yield and
  the second does.
- `REPLIED` keeps the sender-first order.
- With the yield removed it is red (`ra, rb, main 1, main 2, a rest, b rest`).

The oracle's majority orders for `two` and `hsend` are not reached: its relinquish happens
inside the REPLY, before the sender's clause shows the value, and this crate's yield is at the
sender's next boundary as ruled. These are crate tests only.

Side effect: `memreply100000` (100k REPLYs, `t6b/rss.sh`) now peaks at 21,080 / 21,028 kB. Each
REPLY after the first finds the previous continuation ready, so the sender yields and the
queue does not grow.

### Witness made order-fixed

During the debug corpus run under `every`, `reply_inside_constructs.rex` failed once with the
**oracle** side printing the OTHERWISE continuation before the sender's `otherwise: other`.
The oracle's order there is a race it almost always wins, not a fixed order. Each method's
rest now waits (`call hold d`) until the sender has shown the reply value. The output is
unchanged:
- quiet: oracle 30/30 and ours 30/30, unswitched and `every`, all the same hash as before the
  edit;
- under load: oracle 32/32 (eight parallel runners of four each).

The sourceline file was regenerated. The other REPLY witnesses held one hash on the oracle
under the same load: 24 runs each, eight runners of three.

### Minors

- M4: R-T6-2's "99.924, both sides" corrected: code and rc agree, and the oracle's traceback has
  the interpreted clause as an extra first line.
- Queued with probe and both sides' output: `2026-10-02-exit-in-handler-at-reply.md` (O2) and
  `2026-10-02-halt-after-replied-send.md` (O3).

### P28 checks (at 6d5f6f116)

- **Format and lint.** fmt is 0. Clippy `-D warnings --all-targets` is clean for `-p rexx-exec`
  and for `--features pinning`.
- **Full debug suite.** `REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --no-fail-fast`
  (debug, every integration test, with the new drive guard): exit 0. The corpus gave 722/722.
  The run predates the witness edit.
- **Corpus.** Debug under `every`: 722/722, after the witness edit. Release: 722/722
  unswitched and 722/722 under `every`.
- **Stress.** collect_stress (release): 36 passed. The corpus run alongside it gave 722/722.
- **Other suites.** Lib tests 904 passed. Pinning `measured::` 16 passed. rexx-core is green in
  debug and release. sourceline_oracle passed.
- **Bench "it works".** Every program matches 939fdace1 except `heapshape`.

## Fix round 3

- R2-2: `reply_inside_constructs.rex`'s `indo` is `if i = 2 then reply i` again, a REPLY as an
  IF arm's sole instruction inside a loop. The order fix moved to a second clause,
  `if i = 2 then call hold d` (the re-review's `p2/indo_old.rex`).
  - Oracle 30/30, ours 30/30 unswitched and 30/30 under `every`, all one hash
    (`t6b/stab.sh`), the same as before the edit.
  - Debug, collect every allocation (`t6b/stressdrv`): unswitched and `every`, 3 runs each,
    each with the oracle's stdout, rc 0 and no panic.
  - The sourceline file was regenerated.
- R2-3: `queued/2026-10-02-halt-after-replied-send.md` now says only rc 252 and the 4.1 at line 5
  are fixed, and that the order of the `1` and `rest 1` varies.
- `oracle-crashes.txt` entry 24: the re-review's `p2/traceobj.rex` (`.traceOutput` redirected
  to a Rexx object under `trace r`, with a started activity running). Reproduced SIGSEGV rc 139
  3 of 3 under the standard wrapper. The entry also records what this crate prints.
- Checks: corpus (release, gate) 722/722 unswitched and 722/722 under `every`;
  `sourceline_oracle` passed.
