# Task 5 report: sleepers, SysSleep as a park, parkable-native composition

Status: DONE. Base 157da9660. Code commit c0ded9538; this report in the commit after it.

## What landed

* `ParkReason::Sleep { deadline }`; `Activities::sleepers`, a deadline min-heap (ties in park order).
  `next_runnable` moves due sleepers to the ready queue and, with nothing ready, idles until the
  earliest is due (`Interp::idle_until`, `clause.rs`); a run's deadline bounds the idle
  (`Failure::Deadline`). `serve_requests` moves due sleepers at every countdown visit, so a running
  interpreter wakes them within the countdown's reload and the timer then slices it.
* `timer.rs`: `Registration::idle_until(at)` blocks on a per-interpreter condvar under the registry
  lock; the timer thread wakes it when `at` is due, alongside its slice work.
* `SysSleep` (`builtin/rexxutil.rs`) parks through `Interp::park_routine` (`dispatch.rs`), which is
  `park_native` with the routine's answer as the continuation: a pinned wait under a pinned frame,
  else the park recorded on the activity. `park_point!` stays.
* Composition: `run_internal`/`run_internal_as` answer `Started<ObjRef>` (type-level: every caller
  handles `Entered`). Function calls (`Op::CallArgs`, `Op::CallExpr`) and `CALL` (`Op::Call`,
  `Op::CallNamed`) reached from a resumable entry park at the driver; `resume_woken` now delivers a
  woken value into the register (`Deliver::Register`) or settles `RESULT` (`Deliver::Flow`), where it
  refused loudly before. `lend_stack` does not lend the argument run to a callee tail when the call
  parked (`entered_body`). Non-resumable callers (`complete_function`, `complete_subroutine`,
  `run_call_args`, `run_call_expr`) refuse a recorded park (`refuse_unrooted_park`), as
  `complete_send` already did. `Routine~call` of an internal row composes as a parked native
  (`NativeStarted::Entered(Then::Pass)`); not reachable for SysSleep (see concerns).
* Pinning report: `PinReport::pinned_parks` (pinned waits by park kind and frames, counted in
  `pinned_wait`); `ParkKind::of(reason)` shared with `inverted` (Sleep maps to `SysSleep`).
  `pinned_parks_over_the_derived_list`'s table gains a pinned/inverted waits-by-kind section.
* Carried 3 (nested-loop depth guard): `stack_depth()` = activation depth + the depth of every
  buried activity whose loop is on the stack; the three `MAX_ACTIVATION_DEPTH` guards use it. And a
  loop nested more than `INTERPRETER_STACK_BYTES / 2` of Rust stack above the outermost raises 11.1
  (`run_round`).

## Rulings recorded (spec does not settle)

* R-T5-1: a running interpreter finds due sleepers at its countdown visits (at most 1024 clauses),
  not through a timer request bit; the timer thread wakes idle interpreters only. One `Instant::now()`
  per visit and only while sleepers exist.
* R-T5-2: the inverted refusal names the wait's kind (`a message's completion`, Task 2's text), not
  the pinned re-entry kind: the default build tracks pin depth only; the kinds are in the pinning
  report. Cost if wrong: one message text.
* R-T5-3: `Message~halt` does not wake a sleeping target; the halt lands when its sleep ends. The
  oracle's `SysSleep` is `SysThread::longSleep` (`RexxUtilCommon.cpp:1922`), which `Message~halt`
  does not interrupt. TEST_HALT_START passes on that basis.
* R-T5-4: the nested-loop bound is a Rust-stack span (half the interpreter stack) plus the buried
  activation share, both raising 11.1. Cost: dev-profile (`target/debug`) nesting capacity drops;
  measured chain (each started method waits pinned in `INTERPRET` on the next): 2000 completes,
  3000/4000/10000 raise 11.1 (rc 245), where the base overflowed at 7000 (5000 completed). Release:
  6000 completes, 12000 and 20000 raise 11.1 (9999 reports each). Every-mode
  `deep.rex` (Task 4 review) dev profile: 2000 completes, 5000 raises 11.1 (it completed before).
* `SysSleep 0` parks with a due deadline: the activity goes to the back of the ready queue.
* No inbox exists (Task 15), so the nested loop drains nothing.

## Tests and red-before evidence

New lib tests (`scheduler/tests.rs`): `sleepers_wake_in_deadline_order_and_overlap` (unswitched and
EveryOpportunity; 2 s <= wall < 2.9 s, sum 3 s), `a_sleep_in_a_sort_comparator_runs_the_others_meanwhile`,
`a_sleeper_wakes_while_main_is_busy`, `a_deadline_ends_a_wait_on_a_later_sleeper`,
`an_inverted_pinned_wait_is_refused_under_every_opportunity`,
`nested_pinned_waits_are_bounded_by_insufficient_stack` (chain 20000; rc 245, 11.1, empty stdout).
New measured test (`--features pinning`): `a_sleep_is_a_pinned_wait_only_under_a_pinned_frame`
(IF/DO, loop, function argument, assignment: frames empty and no pinned park; comparator: pinned
parks equal the SysSleep arrivals, all under `SortComparator`).

Mutations (applied to a copy, restored by copying back; diffs checked after):
* M1 SysSleep back to `std::thread::sleep`: red `sleepers_wake_in_deadline_order_and_overlap`,
  `a_sleep_in_a_sort_comparator_runs_the_others_meanwhile`, `a_deadline_ends_a_wait_on_a_later_sleeper`;
  corpus `sleepers_wake_in_deadline_order` (A before B, `below the sum 0`) and
  `sleep_parks_in_if_do_and_argument` (ticks after main) differ from the oracle;
  `sleeper_wakes_busy_main` stays green (it witnesses M2).
* M2 no sleeper wake in `serve_requests`: red `a_sleeper_wakes_while_main_is_busy` (deadline);
  corpus `sleeper_wakes_busy_main` hangs (timeout rc 124).
* M3 bounds: removing the stack-span bound alone leaves the test green in the test profile (opt 3),
  where the buried activation share fires; removing both reddens it. In the dev profile the span
  bound is the one that fires: with only the activation share, chain 7000 overflowed (rc 134).
* Group runner: TEST_START (EveryOpportunity) and TEST_HALT_START (unswitched) were listed differing
  at base (Task 4 tables); now `pass`, lists emptied.

## Oracle and our-side stability

Oracle, 30 runs each, one stdout/stderr/rc hash per program: `sleepers_wake_in_deadline_order` 30/30,
`sleep_parks_in_if_do_and_argument` 30/30, `sleeper_wakes_busy_main` 30/30, and Task 3's 0.3 s
witnesses `message_start_error_condition` 30/30, `started_primitive_error_condition` 30/30,
`message_result_reraise` 30/30. Ours (release), 30 runs unswitched and 30 under EveryOpportunity for
each of those six: always the oracle's hash. The other sleep-using witnesses
(`main_fails_in_pinned_yield`, `message_halt_untrapped`, `message_halt_wait`,
`message_primitive_send_fails_main`, `message_error_condition_after_main`, `message_halt_returning`,
`message_result_relayed`, `message_send_fails_main_frames`): 10 runs each mode, always the oracle's
hash. collect_stress (release) green twice over the corpus including the new witnesses.

Also checked against the oracle, one run: `TRACE I` over `call SysSleep`, `SysSleep()` in an
expression, an `IF` condition and an argument (stdout, stderr, rc identical); SysSleep argument
errors under `SIGNAL ON SYNTAX`.

## Message table delta

Unswitched: 52 pass / 16 refused (Task 4: 51 / 1 differ / 16); TEST_HALT_START differ -> pass.
EveryOpportunity start tests: 18 pass / 1 refused (TEST_STARTWITH_NOT_ARRAY); TEST_START differ -> pass.
Object start tests: 40 pass. Raw tables: `task-5-message-table.raw.md`,
`task-5-message-switched-table.raw.md`.

## refusal-sites.tsv

No `Loud` constructor added or removed (one new `scheduler_inconsistency` call site, one removed);
`refusal_sites` green, table unchanged.

## Bench "it works"

`rust/bench-programs/*.rex`, release head against a base release built from `git archive 157da9660`
in its own target dir (`Compiling rexx-exec` seen): stdout and rc identical for every program but
`heapshape`, whose two lines are timings (base differs from itself across three runs).

## P28 checks (commands run, exit statuses read)

* `cargo fmt --all --check`: 0.
* `cargo clippy -p rexx-exec --all-targets -- -D warnings`: 0; with `--features pinning`: 0 (both
  re-run after touching sources, `Checking rexx-exec` printed).
* `cargo test -p rexx-exec --lib`: 897 passed.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`: 711 of 711; with
  `REXX_CORPUS_SWITCH=every`: 711 of 711.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test collect_stress`: 34 passed.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --features pinning --test concurrency_tests`:
  40 passed (group runs and `measured::` included).
* `refusal_sites`: 5 passed. `rexx-parse --test sourceline_oracle`: 1 passed (three new
  expectation files generated with the module's driver).

## Concerns

* Single-activity `INTERPRET` recursion still overflows the stack in the dev profile (measured:
  `f` recursing through `interpret 'r = f(n + 1)'`, rc 134 debug, 11.1 rc 245 release). Pre-existing
  and outside nested loops; the activation guard's 10000 does not fit interpret frames in debug.
* Release chains of several thousand started activities exhaust address space under `ulimit -v 8G`
  (chain 8000 aborts with a failed 1 MB allocation at 495 MB resident), the licensed OOM divergence
  Task 2's review recorded.
* `SysSleep` cannot be reached as a `Routine` object or a `LIBRARY REXXUTIL` external here
  (98.903; `findRoutine('SysSleep')` is `.nil` on both sides), so the `Routine~call` composition
  arm of `executable.rs` has no witness.
* A sleeper far in the future delays an inverted or unsatisfiable refusal until it wakes.
