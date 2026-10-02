# Task 8 report: the native timers as parks

Status: DONE. Base 18d0f9a4c. Code, tests, tables and this report in one commit (sha in the ledger).

## Design

- `ParkReason::Timer { deadline, cancel: TimerId }` (`scheduler.rs`). A timer park is a sleeper
  (same heap, same idle and countdown wake) whose `TimerId` names a per-interpreter `Timer` in
  `Activities::timers`: the oracle's `SysSemaphore` (posted until reset) plus the waiter and its
  sleeper's park order. `post_timer` is the early wake: it removes that sleeper by its order and
  queues the waiter. `cancel_wait` clears a failed wait's waiter.
- Natives (`dispatch/time_support.rs`), rows in `dispatch/native.rs` as a new
  `ExternalBody::Scoped { arity, begin }`: a begin-style external that gets the method's scope
  (`resolution.scope`), so `EVENTSEMHANDLE`/`TIMERSTARTED` are set and `CANCELED` read in the
  defining class's scope as `SetObjectVariable`/`GetObjectVariable` do. The begin is not wrapped
  in `pinned!`: it runs no Rexx, and a park from it composes through resumable entries
  (`Started::Entered` with the continuation on `native_park`), or is a pinned wait under a pinned
  frame (`park_native`). Argument conversion is the oracle's: 88.901 missing, 88.907 range for
  `wholenumber_t`, 88.914 for a `POINTER` that is not a `.Pointer`, 88.922 too many.
- `EVENTSEMHANDLE` is a `.Pointer` whose address is the timer number (`TimerId::address`); the
  continuation (`alarm_woken`/`ticker_woken`) is handed that pointer as its receiver, so it needs no
  extra state. Alarm's timer ends with its call (the oracle's semaphore is a stack local); a
  ticker's ends when a post that was a cancel wakes it (the oracle's `delete sem`).
- The stores go through `Interp::set_pool_variable`, the path attribute setters and the library
  `SetObjectVariable` take (spec 2.5's barrier list).
- Day loop, as `TimeSupport.cpp`: a post that is a cancel ends a wait at once; a post that is not
  a cancel, during the whole days, is reset and ends the current day (the sleeper's deadline moves
  to now plus the days not begun plus the remainder); in the remainder any post ends it. A ticker
  wait that finds its timer already posted ends at once as a cancel, else ends at once with no
  whole days, else resets the post and drops the first day.

## Rulings

- R-T8-1: `CANCELED` counts as cancelled where its value is `1` (a tagged 1 or the string `1`).
  The oracle tests identity with `.true`; a string's identity here is its bytes (D15), so
  `canceled = 1` also cancels here and not on the oracle. Unreachable from `CoreClasses.orx`,
  which assigns `.true`. Cost if wrong: one divergence for a native bound by user code.
- R-T8-2: a stop on a timer that has ended (an alarm's after its wait, a ticker's after its
  cancel) does nothing, and a ticker wait on one answers 0 at once. The oracle posts or waits on a
  freed semaphore (undefined behaviour). Cost if wrong: none observable short of the oracle's UB.
- R-T8-3: a wait longer than an `Instant` reaches ends `u32::MAX` seconds out.
- R-T8-4: the program's end stops at the run's deadline. `run_started_to_end` looped on
  `run_started_activities` while it failed, and a sleeper due after an expired deadline made every
  pass fail with `Failure::Deadline`: the failure list grew without bound (measured, an abort on a
  40 GiB allocation under `ulimit -v 32G`). Reachable at base with `SysSleep` in a started method
  and a deadline (test `a_deadline_ends_the_programs_wait_on_a_started_sleeper`); the method-body
  table's Alarm rows reached it here. A deadline failure now ends the loop.
- R-T8-5: the method-body table's `Alarm` and `Ticker` receivers are `RECEIVER_OVERRIDES` that
  `SysSleep` 0.2 s around the cancel, so the timer has started before it and its activity has
  ended after it. With `class-set.txt`'s `~~cancel` straight after `~new`, the oracle waits in
  `guard on when timerStarted` and this crate refuses that wait (Task 12), then the program's end
  waits a day for the alarm (rows went loud -> diverge at rc 121); the `cancel` row's second
  cancel could post the oracle's freed stack semaphore (one oracle run `Signaled`); the Ticker
  `init` row raced the ticker's loop (oracle stderr interleaved two threads' tracebacks). With the
  overrides: oracle 20/20 one hash for the `cancel` and `init` probes; 13 rows loud -> answers or
  unstable (`scheduledTime`, a clock reading, unstable on this crate), 0 regressions.

## Tests

Lib (`dispatch/time_support/tests.rs`), each run unswitched, under `EveryOpportunity` and
collecting at every allocation, with a 30 s deadline, asserting rc 0, stdout and the run time
(lower bound from the wait, upper bound 10 s against 20 s waits). The caller is a class whose
unguarded methods are the five natives (`::method ... unguarded external 'LIBRARY REXX ...'`):
`an_alarm_timer_waits_its_time` (sets `TIMERSTARTED` and a `.Pointer` `EVENTSEMHANDLE`),
`a_cancel_before_due_ends_an_alarm_wait` (remainder and whole days),
`a_post_ends_the_remainder_or_the_current_day`, `a_cancel_after_due_does_nothing`,
`a_ticker_post_stays_until_a_cancel`, `a_post_before_a_wait_of_days_ends_its_first_day`,
`a_cancel_ends_a_ticker_wait`, `a_timer_wait_in_a_sort_comparator_runs_the_others_meanwhile`
(pinned wait), and through the classes `an_alarm_fires_on_its_replied_activity`,
`an_alarm_cancel_ends_its_wait`, `a_ticker_triggers_until_cancelled`.
`scheduler/tests.rs`: `a_deadline_ends_the_programs_wait_on_a_started_sleeper` (R-T8-4).

The minimal caller's paths were checked against the oracle with the same natives bound in a user
class (scratch `n1.rex`: start, cancel before due in the remainder and in the days, a post that is
not a cancel in the remainder and in the days, ticker wait, a post staying, a pre-post with days,
ticker cancel): oracle 10/10 one hash, ours 10/10 that hash unswitched and 10/10 under every. Not
run on the oracle: a stop or wait on an ended timer (R-T8-2, the oracle's UB).

Red before (mutations applied to the file, restored by copy, `cmp` checked; `cargo test -p
rexx-exec --lib`, `ulimit -v 16G`):
- M0, the scoped arm answers the base's deferred refusal: all 11 timer tests red, each
  `the LIBRARY REXX entry point ... is not implemented (Phase 6)`. On a release build with M0 the
  four corpus witnesses each exit 120 with that refusal (`alarm_startTimer`, `ticker_createTimer`,
  `alarm_startTimer`, `alarm_stopTimer`).
- M1, a post does not queue its waiter: red `a_cancel_before_due_ends_an_alarm_wait`,
  `a_post_ends_the_remainder_or_the_current_day`, `a_cancel_ends_a_ticker_wait`,
  `an_alarm_cancel_ends_its_wait`.
- M2, no day-phase branch (every post wakes): red `a_post_ends_the_remainder_or_the_current_day`.
- M3, a day-phase post moves no deadline: red the same (deadline).
- M4, `CANCELED` never counts: red `a_cancel_before_due_ends_an_alarm_wait`,
  `an_alarm_cancel_ends_its_wait`. A ticker's timer removal on a cancel is unobservable from Rexx
  (a wait on a removed timer and on a posted one both answer 0 at once); no test reddens.
- M5, the program's end ignores the deadline: the lib test binary aborts (SIGABRT, a 20 GiB
  allocation) in `a_deadline_ends_the_programs_wait_on_a_started_sleeper`.
- M6, a pre-posted ticker wait keeps its first day: red
  `a_post_before_a_wait_of_days_ends_its_first_day` (deadline).

Corpus witnesses (`rust/corpus/lang/`, listed in `phase-8.txt`, sourceline expectations generated
with the module's driver): `alarm_fires_after_reply` (REPLY then `!startTimer`; fires on the
replied activity after main's last line), `ticker_cancelled_by_its_target`,
`timer_cancel_in_whole_days` (Alarm two days out and Ticker of a day, cancelled once started),
`timer_native_arguments` (88.901, 88.907, 88.914, 88.922 through a subclass, trapped; a direct
200 ms wait). Every path each claims was read in its stdout.

## Oracle and our-side stability

One stdout/stderr/rc hash per run (`stab.sh` in the scratchpad), release head binary:
- oracle, 30 runs each: `alarm_fires_after_reply` 30/30, `ticker_cancelled_by_its_target` 30/30,
  `timer_cancel_in_whole_days` 30/30, `timer_native_arguments` 30/30, one hash each.
- ours unswitched 30 runs each and `REXX_SWITCH_MODE=every` 30 runs each: always the oracle's hash.
- collect_stress (release) green, and green run alongside the corpus differential.

## Remaining Alarm/Ticker paths for Tasks 12/14

- `Alarm~cancel` before the replied activity has started the timer: `guard on when timerStarted`
  with `timerStarted` 0 is the GUARD WHEN refusal (Task 12). The program's end then waits for the
  alarm's timer (`.alarm~new(5, t)~cancel` refuses, then fires 5 s later). Every `Alarm~cancel`
  in this task's witnesses and tests comes after a `SysSleep`.
- Guard locks (Task 11): Alarm `init`'s `guard off`/`guard on` around the wait, Ticker's loop
  guards and the guarded `triggered` attribute run as today's no-ops.
- ooTest (Task 14): over the derived list's `base/class/Alarm` and `base/class/Ticker` rows,
  crate-side in-process outcomes in the pinning table (`concurrency_tests` measured, `grep
  '^| base/class/\(Alarm\|Ticker\)' target/tmp/pinning-table.md`): every test passes but
  `Alarm TEST_BASE_ALARM`, refused at GUARD WHEN; no oracle comparison of these groups was run here.
  `MethodArgs TEST_REQUEST_STRING_*` rows reach a Timer park and the same GUARD WHEN refusal.

## refusal-sites.tsv

Re-derived with `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test
refusal_sites`: one row moved (`deferred_send`'s line, `native.rs:694` -> `:735`); no `Loud`
constructor added or removed. `deferred_send` stays: stream and queue rows still defer. The five
`deferred(.., "Phase 6")` rows are gone; `every_deferred_entry_point_names_an_open_phase` now admits
only Phase 10, `native_entries`' `IMPLEMENTED` lists the five, and the timer family's probe
(`gate-tables/native-entries/timer.rex`) carries the built-family comment `file.rex` carries.
`method-bodies.txt` re-derived (`REXX_METHOD_BODIES_REFRESH=1`): Alarm and Ticker rows as R-T8-5.

## Bench "it works"

`rust/bench-programs/*.rex`, release head against a release built from `git archive 18d0f9a4c`
(with `interpreter`, `common`, `api`) in its own target dir, `Compiling rexx-exec` seen: stdout and
rc identical for every program but `heapshape`, whose lines are timings. `extcall` is rc 158 on
both (its library is not on the run directory's path).

## P28 checks (commands run, exit statuses read)

- `cargo fmt --all --check`: 0.
- `cargo clippy -p rexx-exec --all-targets -- -D warnings`: 0; with `--features pinning`: 0
  (`Checking rexx-exec` printed in both).
- `cargo test -p rexx-exec --lib`: 920 passed.
- `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`: 737 of 737 matching; with
  `REXX_CORPUS_SWITCH=every`: 737 of 737; debug (`cargo test -p rexx-exec --test corpus`) under
  every: 737 of 737.
- `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test collect_stress`: 36 passed; again
  36 passed with the corpus binary (737 of 737) running alongside.
- `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --features pinning --test
  concurrency_tests`: 43 passed (`measured::` and `group_runs` included).
- `refusal_sites` 5 passed; `native_entries` 24, `closed_phases` 5, `owners` 6, `loud` 9 passed;
  `rexx-parse --test sourceline_oracle` 1 passed; `method_bodies` (refresh run) 23 passed,
  0 regressions.

## Concerns

- Pre-existing, also at base for the stream natives: a `LIBRARY REXX` native's untrapped error
  names the program's path in `Error NN running ...:` where the oracle names `REXX` (measured with
  `.stream~new(..)~lines(1,2,3)` and with `!stopTimer('abc')` from an Alarm subclass;
  `external_package_path` answers no program for the bootstrap packages). The corpus witness traps
  its errors.
- A loud refusal in main no longer ends quickly where a timer is pending: the program's end waits
  for it (spec's end rule; Task 9 owns it).
