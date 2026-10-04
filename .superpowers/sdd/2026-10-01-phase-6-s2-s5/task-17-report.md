# Task 17 report: the driver exit for native calls

Base `f13f15122`. Commits:
- `69b90067e`: the exit, the guarded host, held rows, the tests;
- `4401cc265`: `HostRef` reworked into an address with an optional baton, plus inline attributes;
  this is the performance follow-up;
- this report.

## Design

### When a call leaves its driver

`Interp::exits_for_native` (`dispatch/library.rs`) decides this before the call's frame is pushed.
A call exits when all three hold:
- it comes from a resumable entry that nothing pins (`pin_depth == 0`);
- no park continuation is running (`Activity::resuming`, which `resume_native_park` sets around the
  continuation; that code runs outside every driver);
- another activity is alive (`Interp::others_live`), or a test mode is set (a switch mode, or
  collect stress).

Every other call runs back to back on the baton, as at base. Those are pinned calls (spec 2.1),
calls made from a park continuation, and a lone activity's calls. The lone-activity case is the P43
shortcut.

### The exit sequence

1. **Prepare.** `run_library_method` and `run_library_routine` push and pin the native frame as
   before. Their shared body is `Interp::native_call`. On the exit path it runs the signature and
   `prepare` under the baton, pinned. It then unpins and records two things:
   - `Activity::native_call`, a boxed `NativeInFlight`. This holds an `OffBaton` part (the held row,
     the `NativeCall` and the call's `CStringPool`), the frame token, the condition recorded so far,
     and a stage: `Prepared`, `Left` or `Completed`.
   - `Activity::native_park`, with `ParkReason::Native` and the continuation `resume_library_method`
     or `resume_library_routine`.

   It answers `Started::Entered`. The routine call sites (`run/call.rs`, through `library_begun`,
   and `dispatch/executable.rs`) now take a `Started<Option<ObjRef>>`.
2. **Park.** The driver parks the activity. `park(Native)` files nothing. `run_until_park` and
   `run_activity_root` call `exit_if_awaited` immediately, before anything else runs or switches.
3. **Exit.** `Scheduler::exit_for_native` does the following on the baton:
   - collects under collect stress;
   - moves the activity's park into the record (`NativeInFlight::leave`), where it is rooted, so
     that a park recorded by a callback's Rexx code is not mistaken for it;
   - takes the `OffBaton` part out;
   - increments `in_flight` and `runs_below`;
   - pins the activity with `NativeApiCallback`.

   Then `OffBaton::run`:
   - builds the call's `Activation` over `HostRef::guarded(interp, &baton)`;
   - enters the activity's thread context while still on the baton;
   - releases the baton;
   - calls `invoke::call_held_method` or `call_held_routine`;
   - posts the `Completion` through `Scheduler::post_completion`;
   - takes the baton back, and leaves the thread context.

   Finally `exit_for_native` unpins, decrements `runs_below`, and puts the part and the park back
   (`NativeInFlight::back`).
4. **Drain.** The holder drains completions at its next scheduler entry (`next_runnable`, when
   `INBOX` is set) and at each cold visit (`serve_requests`). `file_completions` stores the
   completion on the record (stage `Completed`), decrements `in_flight` and readies the activity.
5. **Resume.** The activity's root driver resumes the park. In order:
   - `resume_native_call` pins and collects under stress;
   - `finish_native_call` runs `invoke::finish` as that activity;
   - `end_library_method` or `end_library_routine` pops the frame, releases a guard the frame still
     held, then raises or settles;
   - the method or routine is blamed, as on the inline path.

   A function call whose routine answered nothing gets 44.1 under the call's spelling
   (`no_data_from_parked_call`, `ir/drive.rs`). Where a failure ends the park instead,
   `abandon_native_call` pops the frame.

**What the post carries:** `Completed { activity: ActivityId, completion: Completion }`. This is
the inbox's `Posted` type (`u32` under `loom`). `Completion` is Task 16's; the condition stays in
the native frame.

### The nested loop

- `Activities::in_flight` counts calls that have exited and whose completion has not been drained.
- `Activities::runs_below` counts those whose run is on this thread's stack, under a callback.

When `next_runnable` finds nothing ready and no sleeper:
- if `in_flight == runs_below`, it answers `None`, giving the existing refusals (inverted, or
  nothing can end);
- otherwise it blocks on the inbox through `Interp::idle_for_posts`, which uses the deadline-less
  `Registration::idle`, or `idle_until` the run's deadline, and files what arrives.

`runs_below` matters already in Task 17: a callback's Rexx wait that nothing can end would
otherwise block forever on a completion that can only be posted after the callback returns.

The run's end idles for good only when `in_flight == runs_below`.

Other changes:
- The timer is armed while `in_flight > 0`, in both `serve_requests` and `switch_to`.
- `Interp::idle_until` files any posts. Its deadline branch loops on the clock, and returns `Ok`
  when something was posted.

### The baton-guarded Host accessor (P50)

- **The baton trait.** `rexx_api::values::Baton` has `take_unless_held`, `release` and
  `held_here`. `crate::baton::Baton` implements it (in `dispatch/library.rs`) and gains
  `take_unless_held`. `release` is no longer test-only. `Interp::baton` is an `Arc<Baton>`.
- **The host reference.** `Conversion::host` is now a `HostRef` (`rexx-api/src/ffi.rs`). It holds:
  - the host's address, taken from a `&'a mut` that it keeps borrowed for `'a`;
  - an optional baton.

  Each `Deref` or `DerefMut` derives a fresh borrow. For a guarded host it first debug-asserts "a
  callback reached the host without the baton". `HostRef::lent` serves the calls that keep the
  baton; `HostRef::guarded` serves the off-baton call. So the call's `Activation` holds no
  `&mut dyn Host`.
- **Taking the baton.** `Activation::conversion` returns a `Converting` guard. For a guarded host
  it takes the baton unless this thread already holds it, and gives it back when the guard drops,
  after the `RefCell` borrow ends.

Derivation of every path from an API callback into the interpreter, run at `4401cc265`:
```
grep -rn "conversion\.borrow\|conversion\.try_borrow" rust/crates/rexx-api/src
  values.rs:851 (Activation::conversion), values.rs:865 (is_busy)
grep -rn "dyn Host\b" rust/crates/rexx-api/src --include=*.rs | grep -v "/tests\.rs:\|/tests/"
  ffi.rs:119-185, all inside HostRef (field, lent, guarded, From, Deref, DerefMut)
grep -rn "impl Host for\|impl Surface for" rust/crates/rexx-exec/src
  dispatch/library/surface.rs:32, dispatch/library.rs:1061 (Interp)
grep -rn "HostRef::guarded\|host: self\.into()\|host: host\.into()" rust/crates/rexx-exec/src
  dispatch/library.rs:249 back-to-back call, :267 prepare, :349 finish, :399 package hook,
  :445 command handler (lent); :998 OffBaton::run (guarded)
```
- Every callback reaches an `Activation`, either through a context's owner pointer or through the
  thread context's innermost call.
- An `Activation` reaches the host only through `conversion()`.
- `Surface` is reached only through `Host::surface` on a derived host.

### Rc and thread-context clones (Step 2)

What changed:
- The method path's `Rc::clone(&binding.library)` is gone.
- The routine path's `Rc` clone and its per-call clone of the code key are gone.
- A row is now held for the call by `NativeMethodEntry::held` or `NativeRoutineEntry::held`
  (`load.rs`). The answer holds the stub address and an `Arc` of the mapping; `Mapping` now uses an
  `Arc` and atomics. While it lives, the call counts as in flight, so a close is refused. It is
  `Send`, and on the exit path it lives in the activity record.

**Departure:** the `ThreadContext` clones remain in `native_call`, `exit_for_native`,
`run_package_hook` and `run_command_handler`.
- Borrowing the record's field across the call conflicts with the host borrow the call's
  `Activation` takes.
- As `enter`'s `&self` argument, that borrow would also be protected while a callback derives
  `&mut Interp`.
- A raw handle would need `unsafe` in rexx-exec, or a safe `ffi.rs` handle whose validity no type
  states.
- The clone is made and dropped on the baton.

### Collect stress

Collect stress collects at every driver exit and again before `finish`. Under stress every
unpinned call exits, so the collect-stress corpus runs exercise the exit. Pinned calls keep Task
16's between-halves hook.

### Other departures

- There is no pool yet, so `exit_for_native` runs the call on the releasing thread at once.
- While the activity's park is kept on the record, `message_runner` does not see it. This matches
  the inline path, where nothing records a message's runner during the call.

## Tests

`src/scheduler/tests/native.rs`. Each run executes on an interpreter thread of its own and counts
driver exits and callback takes through test-only thread-locals.

| Test | What it shows |
|---|---|
| `a_native_call_where_another_activity_lives_leaves_its_driver` | `5\n7\n`, 2 exits |
| `a_lone_activitys_native_call_keeps_its_driver` | 0 exits (P43) |
| `collect_stress_across_a_native_calls_driver_exit` | `RxCalcSqrt` builds its answer in a callback; plain and stressed both `4\n1.4142\n`, 2 exits, at least 2 takes; more collections when stressed |
| `a_callback_during_an_off_baton_call_takes_the_baton` | `TestSendMessageScoped` runs a Rexx method in a callback, under stress: `base\n7\n`, takes at least 1. The same call under `INTERPRET` (pinned): 0 exits and 0 takes |
| `a_pinned_wait_is_satisfied_by_an_activity_returning_from_a_native_call` | comparator waits on `m~result`; `7\nsorted 1,2\n`, unswitched and under every |
| `a_callbacks_wait_on_a_call_below_it_is_refused` | kept and exited runs both give the refusal at rc 120, within a 5 s deadline |
| `a_function_whose_call_left_its_driver_and_answered_nothing_is_44_1` | identical stderr in both modes |
| `a_guarded_library_method_runs_after_its_guard_wait` | `5\nheld\n`, both modes |
| `the_timer_is_armed_for_a_call_in_flight` | arming follows `in_flight` |

The expected outputs of the oracle-runnable programs were checked against the oracle from fresh
directories: c, e, g and d match, including the 44.1 report.

rexx-api tests (`ffi.rs`):
- `a_callback_through_a_guarded_host_takes_the_baton_for_its_length`: one take and one release; no
  take when the baton is already held.
- `a_guarded_host_reached_without_the_baton_fails`: should panic, debug builds only.

`tests/loom.rs`:
- `a_callback_during_an_off_baton_call_takes_the_baton_first`: covers the release, the callback's
  take, touching state, the post and the reacquire, against a thread that takes the baton
  meanwhile. Exclusivity is checked with an atomic `touch`.
- `a_loop_with_a_call_in_flight_waits_for_its_completion`: a deadline-less `Registration::idle`
  against a post from the call's thread.

## Mutation evidence

Each mutant was applied to a copy of `4401cc265` with `scratchpad/t17/mut/run.py` and restored from
a saved copy. The command was `cargo test --profile mutation -p rexx-exec --lib
scheduler::tests::native` with an own target directory; the collect run used `--test
collect_stress` with `REXX_CORPUS_GATE=1`.

| mutant | result |
|---|---|
| `exits_for_native` answers false | exit 101, 6 native tests fail |
| without `others_live` | exit 101: the exit test, the pinned-wait test and the stress test fail (`exits` 0) |
| exits for a lone activity | exit 101: the lone test, the 44.1 test and the refusal test fail |
| exits while resuming | exit 101: guarded test, "a guarded method that parked again after its guard wait" |
| exits while pinned | exit 101: callback test, "a wait outside every root driver and pinned frame" |
| park left on the activity during the call | exit 101: callback test, "a woken activity with no wait recorded"; refusal test |
| completion stored but activity not readied | exit 101: 4 tests fail, "a wait that nothing left to run can end" |
| no drain at the scheduler entry | exit 0: the inbox block files the completion |
| no inbox block (answers `None`) | exit 0: the entry drain files it |
| both of the previous | exit 101: the pinned-wait test and 3 others fail, "a wait that nothing left to run can end" |
| `runs_below` ignored (`in_flight == 0`) | exit 101: refusal test fails (the run hits the 5 s deadline) |
| 44.1 replaced by a scheduler inconsistency | exit 101: 44.1 test |
| timer not armed for a call in flight | exit 101: arming test |
| callback takes no baton (`values.rs`) | exit 101: callback test ("0 callback takes") and stress test |
| native frame locals unrooted | native tests exit 0; `collect_stress` exit 101, `the_l0_subset_passes_again_under_collect_on_every_allocation`, panic `dispatch.rs:1660` |
| same, and no collection before `finish` | `collect_stress` exit 0, 36 passed |
| loom: `take_unless_held` answers false without taking | `a_callback_during_an_off_baton_call_takes_the_baton_first` panics "two threads touch interpreter state" |
| loom: `Inbox::post` without `notify_one` | `a_loop_with_a_call_in_flight_waits_for_its_completion`: loom deadlock |

What these show:
- The entry drain and the inbox block each cover the other in Task 17, because the completion is
  always posted before `next_runnable` runs. Only removing both is caught. The block on its own is
  witnessed by the loom model.
- The collection before `finish` is the only one that sees unrooted native locals.
- My stress test does not catch that mutant; `collect_stress` does.

## Checks

All runs are at `4401cc265`, from `rust/`, with statuses written unpiped by
`scratchpad/t17/checks.sh` (the same set also passed at `69b90067e`):
- `cargo fmt --all --check`: 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: 0.
- `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings`: 0.
- `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=<own> cargo clippy -p rexx-exec --all-targets -- -D
  warnings`: 0.
- `memcap 8G cargo test -p rexx-api`: 0.
  - lib 66;
  - context 14, handles 5, invoke 16, layout 23, load 11, values 83;
  - doctests 1 and 4.
- `memcap 8G cargo test -p rexx-exec --lib`: 954 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: 787 of 787.
- The same with `REXX_CORPUS_SWITCH=every`: 787 of 787.
- Debug with `every`: 787 of 787.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test collect_stress`: 36
  passed.
- `memcap 8G cargo test --release -p rexx-exec --test refusal_sites --test method_bodies --test
  gate_table_c --test dispatch_seam --no-fail-fast`: 0.
  - refusal_sites 5, method_bodies 23, gate_table_c 22, dispatch_seam 6.
  - `corpus/refusal-sites.tsv`: one row re-derived, `Raised no_data_returned` surface `body` to
    `body+ir`. No line numbers moved.
- `cargo test --release -p rexx-parse --test sourceline_oracle`: 1 passed.
- `cargo test -p rexx-core --test unsafe_sites`: 2 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test api_group_tests`: 24
  passed.
- `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=<own> memcap 8G cargo test -p rexx-exec --test loom`:
  8 passed.
- `RAYON_NUM_THREADS=4 CARGO_TARGET_DIR=<own> memcap 8G cargo test --release -p rexx-exec
  --features pinning --test concurrency_tests -- measured:: --nocapture --test-threads=1`: 18
  passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test concurrency_tests --
  --test-threads=1`: 32 passed.
- "It works": every `bench-programs/*.rex` plus `rexxcps`, release builds of base and head, with
  `LD_LIBRARY_PATH` at the oracle's lib.
  - Exit status is 0 and identical on both.
  - stdout and stderr are identical except the timing lines of `heapshape` (`build_seconds=`,
    `gc_pause_seconds=`) and `rexxcps` (`Performance:`).
  - extcall prints `3000000`.

## Callgrind (Ir, libc and ld-linux excluded)

Command: `bench-programs/callgrind.sh -r 1 -j 4 -p "extcall dispatch fibcall rexxcps"
base=<base> head=<head>`.
- Each `rexx-run` is built `--release` from `git archive <sha> rust interpreter api` into its own
  `CARGO_TARGET_DIR`.
- All files were touched first.
- Both builds printed `Compiling rexx-api` and `Compiling rexx-exec`.

| program | base `f13f15122` | head `4401cc265` | delta |
|---|---|---|---|
| extcall | 8155604566 | 8422639499 | +3.2743% |
| dispatch | 21264385157 | 21264501787 | +0.0005% |
| fibcall | 8494020443 | 8494090674 | +0.0008% |
| rexxcps | 17800617029 | 17800744307 | +0.0007% |

At `69b90067e` the same command gave extcall +6.2538%.

**extcall above +2%.** The program is a single activity, so no call exits. Its whole delta is the
inline path.
- **Including libc, extcall fell.** Base 9445712814 against head 9262753180, which is −1.94%.
  - The base cloned the library code key on every call, with `malloc`, `memcpy` and `free` in
    libc. That is −450M libc Ir (1289697740 against 839700500), and the instrument subtracts it.
  - In its place there is about 89 Ir per call of interpreter code:
    - the held row's `Arc` and in-flight count;
    - the `exits_for_native` test;
    - the baton test per conversion;
    - the `Started`/`Option` plumbing;
    - `ThreadContext::enter` and other halves out of line.
- **Already applied (`4401cc265`).** `HostRef` became one struct instead of an enum, which removed
  the branch per host access. `#[inline(always)]` went on `native_call`, the `end_library_*`
  halves and the held signature functions. Together these took extcall from +6.25% to +3.27%.
- **Tried and gave nothing.** Inlining `enter` saved 1 Ir per call. Inlining `signature_of`,
  `call_stub` and the held `call` gave an identical total.
- **Remedy.** `Library::routine` scans the routine table by name on every call: 1.27G Ir, which is
  425 Ir per call and 13% of the program, at base and head alike. Resolving the row once per
  library code, and keeping its index in `library_codes`, would more than repay the delta. That is
  for Task 26.

## Concerns

- **Task 18/19: protected `&mut self`.** `exit_for_native` holds `&mut self` (a protected argument)
  across the release, and `HostRef::guarded` takes its address from it. With one thread that is
  sound. Before another thread takes the baton, the exit must hold only the island root, and
  `guarded` must take its address from that root.
- **Task 18: `OffBaton` is not `Send`.** Its `CStringPool` keys are `ObjRef`s.
- **Task 19: foreign-thread callbacks under P43.** The shortcut keeps a lone activity's call on the
  baton, so a callback from a foreign thread during that call would wait for it.
- **Untested paths.**
  - `abandon_native_call` (a failure ending a native park) has no test that observes it.
  - `Interp::idle_until` filing posts is reached only by the loom model at the `Registration`
    level.
- **The thread-context clones remain** (Step 2 departure, above).
- **extcall +3.27%** under spec 7's instrument (−1.94% with libc counted); remedy above.

## Fix round 1

Commit `94266cb68`, under `task-17-review.md`.

### I1 (P50): a callback takes the baton before it reads shared state

- **Recovering the activation.** `innermost_or_running` (`ffi.rs`) first recovers the activation
  from its own context's `Innermost` cell. That thread wrote the cell on the baton, before the
  release.
- **Taking the baton.** It then calls `Activation::hold_baton`, which takes the baton unless this
  thread holds it and gives it back on drop. Only after that does it read the table's `Running`
  cell or `is_busy` of another call. `addressed` likewise holds the baton before `own.is_busy()`.
- **Where the baton is kept.** `Activation` keeps the baton in a field of its own (`baton`, next to
  the conversion state), so it can be read while a conversion is held.
- **Enforcement.** A debug assertion, "a callback read the interpreter's state without the baton",
  stands before the `Running` read.
- **Test.** `a_thread_member_off_the_baton_takes_it_before_reading_the_table` (rexx-api `ffi.rs`)
  enters a guarded activation and releases the baton. It then calls `WholeNumberToObject` through
  the thread table and gets a handle, with takes/releases `(2, 3)`: one take for the table read, one
  for the conversion, and the test's own release.
- **Mutant.** With the hold removed, that test panics at the assertion.
- **No new loom model.** These are rexx-api functions over `Activation`, which `tests/loom.rs`
  cannot compile. The existing model covers the baton protocol they use.

### I2: routine and method rows resolved once

The change is based on the reviewer's prototype.
- **New lookups.** `Library::routine_index` / `routine_at` and `method_index` / `method_at`
  (`load.rs`). `routine` and `method` now go through them.
- **Routine rows.** `Interp::library_code_routines` (`Vec<Option<u32>>`, alongside
  `library_code_keys`) holds each code row's routine index. `Interp::library_code_routine` resolves
  it by name on the first call.
- **Method rows.** `LibraryBinding` keeps `method: Option<usize>` in place of the procedure name,
  resolved where the binding is built (`install.rs`, `environment/identities.rs`).

**What keeps the cached row valid:**
- `Libraries::hold` writes a name's library once, and later holds answer the earlier library.
  Nothing removes or replaces it (`libraries.rs`).
- A `Library`'s tables are private and never change after load.
- A code row always names the same library and procedure; a binding holds its own `Rc<Library>`.
- So no library reload or package change can make a cached index name another row.

**Tests:**
- `libraries::tests::a_held_name_keeps_its_first_library`: a second hold answers the first.
- `library_code_routine` debug-asserts at every call that the cached index is the one the name
  finds. The debug corpus run, the debug lib tests, and
  `a_library_routine_called_again_reaches_the_row_its_name_finds` (`1\n2\n3\nTESTNAMEARG\n4\n`,
  which equals the oracle's output) all exercise it.
- Mutant: caching `index + 1` makes that test fail with the assertion.

**Callgrind (Ir, libc and ld-linux excluded).** Base and head were each built from `git archive
<sha> rust interpreter api` into their own `CARGO_TARGET_DIR`, with every file touched; both printed
`Compiling rexx-api` and `Compiling rexx-exec`. The command:
`bench-programs/callgrind.sh -r 1 -j 4 -p "extcall dispatch fibcall rexxcps" base=<base> head=<head>`

| program | base `f13f15122` | head `94266cb68` | delta |
|---|---|---|---|
| extcall | 8155604627 | 7309638996 | −10.3728% |
| dispatch | 21264384971 | 21224499964 | −0.1876% |
| fibcall | 8494020504 | 8494088858 | +0.0008% |
| rexxcps | 17800626282 | 17800743146 | +0.0007% |

Notes on the table:
- The script compared stdout with base's and exited 0.
- Spread was 0.0000%.
- No benchmark sends a library method.
- dispatch makes no native call, and the two commits between `7790acc96` and `94266cb68` are
  plan and docs only. So its −0.19% is a codegen shift; it is within the ~4% per-axis layout band
  and is not claimed as a gain.

### M1: the abandon path has a test

`a_failure_ending_a_native_park_abandons_the_call` runs the review's program:
- it gets rc 120, an empty stdout, and the stderr line `rexx-exec: a method source that is neither
  a string nor an array is not implemented (Phase 5)`;
- it counts `exits == 1`.

A mutant that panics in `abandon_native_call` makes this test fail, so the test reaches the path.

**The missing `5` / `after` lines.** Base printed `5\nafter\n` before the refusal. The oracle prints
`5`, `after`, then the started activity's Error 93.974, at rc 0. Head prints only the refusal.
- This task caused it. Main's call is now a switch point, and the started activity, ready since
  `~start`, runs while main waits for its completion.
- On the oracle that is also a possible schedule, because the native call releases the kernel lock.
- What keeps main's lines from appearing is that this crate's refusal is loud and ends the run. A
  condition in the started activity would not end it.
- So it is an artifact of an unimplemented path, not of the exit. Changing the schedule to hide it
  would undo the switch point the spec asks for, so I left it.
- The report's line "no test observes it" now reads: no failure that ends a native park lets the
  run continue, so the frame pop and the guard release are clean-up.

### M2: the held row's order

- `Held::new` now takes a closure. It counts the call first, then resolves the stub, which reads
  `open`.
- `Mapping::close` clears `open` first, then reads `calls`. If it finds a call, it restores `open`
  and refuses.
- `run_hook` now checks `open` inside `Mapping::hold`, after the count.
- So of a close and a call that race, one always sees the other. A call that races a refused close
  may see the mapping closed and call nothing. That race needs two threads, and none exist yet.
- Test: `load::tests::a_held_row_keeps_its_mapping_open`.

### Checks at `94266cb68`

`scratchpad/t17/checks.sh`, statuses unpiped, all exit 0:
- fmt; clippy for the workspace, `--features pinning`, and `--cfg loom`;
- `cargo test -p rexx-api`:
  - lib 68;
  - context 14, handles 5, invoke 16, layout 23, load 11, values 83;
  - doctests 1 and 4;
- `cargo test -p rexx-exec --lib`: 957 passed;
- corpus, with `REXX_CORPUS_GATE=1`: release 787/787, release under every 787/787, debug under
  every 787/787;
- collect_stress: 36;
- refusal_sites 5, method_bodies 23, gate_table_c 22, dispatch_seam 6;
- sourceline_oracle 1; unsafe_sites 2; api_group_tests 24; loom 8;
- pinning `measured::`: 18; concurrency_tests: 32.

"It works": every `bench-programs/*.rex` and `rexxcps` on the two release builds above.
- Every exit status is 0 on both.
- stdout and stderr are identical except the timing lines of `heapshape` and `rexxcps`.
- extcall prints `3000000`.
