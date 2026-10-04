# Task 19 report: API callbacks from activities on other threads

Base `7ab2691e7`. Commits: `50151f530` (code), `9f21b4020` (`refusal-sites.tsv` re-derived, line
numbers only), `398417c89`, `4592cc4f9`, `9ef5a4c37` (tests and the mutation follow-ups below); this
report is committed after them.

## What was built

### The thread check (Step 2)

- **`CALLING`** (`rexx-api/src/ffi.rs`): a `thread_local!` holding the native call entered last on
  this OS thread, as an `Entered` (the guard `ThreadContext::enter` keeps on its stack). Each
  `Entered` links the call entered before it on the same thread (`outer`), so a thread's calls form
  a chain, innermost first (`calls_here`). This replaces the table-wide `Running` cell and the
  per-context `home`/`runner` thread ids.
- **The rule** (`reached`, `addressed`): a callback through a context reaches that context's
  activity's innermost call (`Here`) when that call is the one calling on this thread, or a call
  of another activity entered above it here holds its conversion state (the old busy guard).
  Otherwise it is `Elsewhere`: 98.983 goes to this thread's own call, or, on a thread running no
  native call, to the context's innermost call (the oracle's target, `Activity::validateThread`).
  A method or call context whose own activation is the calling one, and not busy, is the fast
  path: one TLS read and a pointer compare.
- **The baton before anything is read.** A thread with a call takes the baton through its own
  call's activation (a pool thread recalls it, `Recalling`). A thread running no call takes it
  through the thread table's requester (`ThreadTable::requesting`, `Requester` in
  `dispatch/library.rs`): it posts `Recall { by: Recaller::Context(address), base: None }` and
  waits for a lend; the holder serves it at its next drain with the context's activity switched in
  (`Interp::context_owner`) and `stack_room` 0, so no Rexx code can run there.
- **`AttachThread` and `AddCommandEnvironment`** accept a thread running a call of the instance's
  activity (`runs_here` walks `calls_here`), instead of the home thread or the Task 18 runner.
- **`Host` checks the baton**: unchanged from Task 17 (P50): a guarded `HostRef` asserts it in
  debug builds; `reached` asserts `baton_held` on the innermost call.
- `kept_numeric`/`kept_executable` (the oracle's non-blocking members) also read a frame of the
  running activity (`Interp::with_native_owner`), which a foreign callback switches in.

### Step 3: what existed and what was added

Existed (Task 18, P52): a recall is a post to the inbox; the holder serves it at every drain: the
scheduler entry (`next_runnable`), every cold visit (`serve_requests`, every 1024 clauses, pinned or
not) and its idle on the inbox. That is the forced switch at the next cold visit and the nested
pinned loop draining delegated callbacks (a recall served by a loop nested in a callback is a
nested lend). Witnessed now by `a_callback_is_served_at_a_pinned_holders_next_cold_visit` and
`a_callback_while_the_holder_is_pinned_is_answered`.

Added:
- **Requeue before a lend** (`file_completions`, `Inbox::requeue`). The holder drained a batch and
  served its recalls one at a time; a lendee whose callback waited for a later post of that batch
  (another recall) waited for ever, since the batch sat on the holder's stack. Now what follows a
  recall goes back to the front of the inbox before the lend, and the holder drains again after
  it. Found by running: the first version of the interleaved test hung at its deadline. Loom model:
  `a_requeued_post_is_drained_again_ahead_of_later_posts`.
- **Timer arming for a pending request: nothing added.** A pooled call's recall implies a call in
  flight, which `serve_requests` already arms for. A foreign thread's recall needs no slice: the
  holder sees the `INBOX` bit at its next cold visit, or is woken by the post when idle. Arming
  would change nothing observable.

## Carries

1. **Elsewhere hold (T17).** Every addressing helper now answers a `Reached`, which owns the
   `BatonHold`; `Addressed::Here`/`Elsewhere` carry it, so `raise_invalid_thread`, `kept_numeric`
   and `kept_executable` run under the hold that classified them. A member takes the baton once
   for its whole length (the rexx-api test `a_thread_member_off_the_baton_takes_it_before_reading_the_table`
   now expects one take, not two). `raise_invalid_thread` on a call whose conversion state is held
   (a foreign callback's target, its own thread waiting in a callback) records 98.983 as pending
   rather than borrowing it.
2. **Completed identity (R1 minor A).** Restored: `Completed` carries the call's frame, and
   `file_completions` completes a record only when `call.frame()` matches. **The path is reachable,
   and reaching it crashed first**: `yield_after_notifier_failure` ran its round unburied, so a
   deeper loop re-ran the yielding activity, whose `root_then` was already taken:
   `unreachable!("a started activity records its outcome")`. Ten lines do it (three started
   messages with a failing notifier); the oracle prints the same stdout in 30 of 30 runs, rc 0, and
   its stderr interleaving varies (13 forms in 30 runs), of which ours, 30 of 30, is the most
   common. Fixed by burying the yielding activity (`run_round(true)`): its end runs on below the
   round. Test: `notifier_failures_in_several_started_activities_each_end_their_own`. With that
   fixed, `an_abandoned_calls_completion_does_not_complete_the_next_call` reaches the scenario:
   an interpreter thread of 33 MiB; another activity starts 3000 messages with failing notifiers
   while main is parked in `NAPLONG` (1.5 s); their yields nest until `run_round`'s stack check
   raises 11.1 into main's root wait, which abandons main's call; main traps it, drains, and makes
   `NAPLONGER` (3 s). Without the filter the first call's completion completes the second:
   `waited 0`.
3. **Panicked before the give-back (R1 minor B).** `Lent`'s drop no longer gives the baton back
   while a panic unwinds; `pool::posting_panics` (now given the baton) posts `Panicked`, then gives
   back a lend the thread still holds. Every lend in this tree is followed by a drain before the
   lender runs a clause (`file_completions` drains again after a recall's lend; after
   `exit_for_native`'s lend the next step is `next_runnable`'s entry drain), so the post is the
   whole fix: a check on the lend's return was written, its mutant survived for that reason, and it
   was removed (`4592cc4f9`). The pool thread's remaining locals (its `ThreadContext` clone among
   them) now drop on the baton. Test: `a_panic_under_a_lend_reaches_the_lender_before_it_runs_on`,
   over a test-only injection (`Interp::panic_at_call_end`): the pool thread appends `end` to a
   file and panics under the call-end lend while another activity appends `tick` lines; the file
   must end with `end`. Under `cfg(test)`, `posting_panics` sleeps 100 ms before it posts, which
   widens the window a give-back during the unwind would open. Loom:
   `a_panic_posted_under_a_lend_is_drained_when_the_lend_returns`.
4. **Running non-LIFO.** `Running` is gone (see Step 2): the per-thread chain is LIFO by
   construction, since a thread's calls nest on its stack (`debug_assert` in `Entered::drop`). The
   per-context innermost cell is unlinked by identity: a call ending while a later call on its
   context is in flight (an abandoned call, carry 2) rewrites that later call's `below` instead of
   restoring its own predecessor. Tests: `two_activities_in_native_calls_with_interleaved_callbacks`
   and the rexx-api unit test `a_call_ending_below_a_later_one_leaves_the_later_one_innermost`.

## Tests

`scheduler/tests/callbacks.rs` (new), over routines defined in the test and `rexx_api::load`'s
`doc(hidden)` natives (`send_twice`, `send_await_send`, `send_from_another_thread`,
`send_keeping`, `send_through`; `unsafe` stays in `load.rs`):

| Test | Shows |
|---|---|
| `a_callback_from_the_calls_own_thread_runs` | Step 1a: `b`, the callback on a pool thread |
| `a_callback_from_another_thread_takes_the_baton_and_raises_98_983` | Step 1b: `trapped 98.983`, no `spoke`, one take, served while another activity runs |
| `a_kept_context_used_from_another_threads_call_raises_98_983_there` | a context kept from a call on one pool thread, used from another's: 98.983 in the user |
| `a_callback_while_the_holder_is_pinned_is_answered` | Step 1c: delegated to the pinned waiter's loop |
| `two_activities_in_native_calls_with_interleaved_callbacks` | Step 1d and carry 4: rendezvous in callbacks, calls end in entry order |
| `a_callback_waits_for_a_contended_guard_on_its_own_thread` | Step 1e: a pinned guard wait on the callback's thread |
| `a_callback_is_served_at_a_pinned_holders_next_cold_visit` | Step 3: a callback 200 ms into its call reaches a pinned busy-wait with nothing ready |
| `recalls_drained_together_are_each_served` | Step 3: two recalls in one batch (posted while the holder keeps the baton for a call at a pool bound of 2) |
| `an_abandoned_calls_completion_does_not_complete_the_next_call` | carry 2 |

Also: `pool.rs` `a_panic_under_a_lend_reaches_the_lender_before_it_runs_on` (carry 3);
`scheduler/tests.rs` `notifier_failures_in_several_started_activities_each_end_their_own`; rexx-api
`a_call_ending_below_a_later_one_leaves_the_later_one_innermost`; loom
`a_requeued_post_is_drained_again_ahead_of_later_posts` and
`a_panic_posted_under_a_lend_is_drained_when_the_lend_returns`. New natives: `nap_then_send` and
`await_send_twice` besides those above. The pool harness (`Shape`) gained the
offered library, the interpreter stack and the injection file; `Ran` gained the take count.

**Oracle, Step 1b.** A forged `libforeign.so` (g++ against the worktree's `api/`, NEEDs no
interpreter) whose routine sends from a `std::thread` it joins, run under the same program from a
fresh directory: the oracle aborts, rc 134, `terminate called after throwing an instance of
'NativeActivation*'`, 30 of 30 (`validateThread` throws outside any handler on the foreign
thread). So 98.983 at the call is this crate's answer where the oracle has none.

## Mutation evidence

Harness `p6-scratch/t19/mut/run.py`: exact-string apply (one occurrence asserted), restore from a
saved copy (asserted equal), `memcap 8G cargo test --profile mutation -p <crate> --lib -- <test>
--exact`, the pass/fail counts read from the `test result` line (never 0 run). Logs beside it.
Round 1 ran at `398417c89`; the survivors were answered by `4592cc4f9` and `9ef5a4c37`, then
rerun (round 2, `round2/`). `git status` after each round showed only the controller's
`progress.md`.

| Mutant | Test | Result |
|---|---|---|
| M1 `Completed` frame filter removed | abandoned-call test | red: `waited 0` (rerun at `9ef5a4c37`: red, `abandoned 11.1` then the `waited 1` assertion) |
| M2 `Lent` gives back while panicking | `a_panic_under_a_lend_...` | round 1 green (window too narrow); round 2 red: `end` at line 30629, about 15000 `tick` lines after it |
| M3 no re-raise after a recall's lend | `a_panic_under_a_lend_...` | green; the re-raise was removed (carry 3) |
| M4 no requeue (serve the batch in place) | `recalls_drained_together_are_each_served` | red: deadline (round 2) |
| M4 | `two_activities_..._interleaved_callbacks` | green in that run, so the batch test was added |
| M5 notifier yield unburied | notifier test; abandoned-call test | red: `unreachable!` panic, both |
| M6 hold dropped before `Elsewhere` handling | foreign-thread test | red: takes 3, not 1 |
| M7 foreign callback takes no baton | foreign-thread test | red: takes 2, not 1 |
| M8 foreign recall switches no activity in | foreign-thread test | red: "a native activation is running", abort |
| M9 `Here` when the innermost call is not busy (the old rule) | kept-context test | red: `spoke`, `answered` |
| M10 a thread running no call is `Here` | foreign-thread test | red: `trapped 11.1` (Rexx code on the foreign thread meets its zero stack room) |
| M11 innermost restored LIFO | rexx-api unlink test | red: `None` against the later call |
| M12 `CALLING` not restored to the outer call | interleaved test | red: rc 158, `Error 98.983` on stderr |
| M13 no drain at the cold visit | cold-visit test | round 1 green (the callback came within the round's entry drain); round 2, with `NAPTHENSEND`, red: deadline |
| M14 requeue appends instead of prepending | loom `a_requeued_post_...` | red |

M14 was run by hand: `RUSTFLAGS="--cfg loom" memcap 8G cargo test --release -p rexx-exec --test
loom -- a_requeued`, `timer.rs` restored from a copy after.

## Checks

At `9ef5a4c37`, from `rust/`, `CARGO_TARGET_DIR` in the task's scratch:
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- At `4592cc4f9` (the last commit touching non-test code): `cargo clippy -p rexx-exec
  --all-targets --features pinning -- -D warnings`: exit 0; `RUSTFLAGS="--cfg loom" cargo clippy -p
  rexx-exec --all-targets -- -D warnings`: exit 0; `RUSTFLAGS="--cfg loom" memcap 8G cargo test
  --release -p rexx-exec --test loom`: exit 0, 12 passed (303.53 s).
- `cargo test --workspace --release --no-run` (exit 0), then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, 141 `test result` lines, 2967 passed, 0 failed (summed over
  those lines).
- `memcap 8G cargo test -q -p rexx-exec --release --lib scheduler::tests`, ten times at
  `9ef5a4c37`'s tree: 10 of 10 at 84 passed. Before that commit the abandoned-call test failed 9
  of 10 this way (see Concerns), though the workspace run at `4592cc4f9` had passed it.
- `corpus/refusal-sites.tsv` re-derived with `REXX_REFUSAL_SITES_REFRESH=1`; the diff changes the
  definition column only (checked by diffing the rows with that column cut).

## Departures

- **A `thread_local!` (`CALLING`) in `rexx-api/src/ffi.rs`.** The global-state rule counts
  process-wide state; this is per OS thread, belongs to one native call's own stack, and follows
  `REFUSED` and `HOOK_THREW`, which spec 2.5 keeps per OS thread for the same reason. The alternative,
  a table-wide list of (thread, call) under the baton, costs a `thread::current()` and a vector
  push and remove per native call and still needs the baton before it can be read, which a
  foreign thread does not have.
- **Error 98.983's target from a thread running no call** is the context's innermost call (the
  oracle's), while a thread running its own call keeps Task 17's target (its own call). The
  gate-only `outer_context` tests, which pin Task 17's targets against forged extensions, were not
  run under P51.
- **Outside the brief**: the notifier-yield burial (carry 2's crash), the requeue (Step 3's
  deadlock), `with_native_owner`.
- **No check on a lend's return** for a posted panic (carry 3 offered either); see carry 3.

## Concerns

- **A foreign callback during a call that keeps the baton** (a lone activity's call, P43, or the
  fallback at the pool bound, P55) waits for the holder's next drain, which comes only after that
  call returns; a native that joins the foreign thread hangs. Recorded with P43/P55's divergences;
  the oracle aborts there anyway (above).
- **Inverted waits across callbacks**: a callback that waits for an activity whose continuation is
  pinned below the loop serving it is refused loudly (P30/P31 class). The first draft of the
  interleaved test did this (`B`'s second callback waited for `A`'s call to end while `A`'s
  callback was below), and was redesigned rather than the scheduler changed.
- **A failure from a nested round reaching a sliced main ends main** (`root_step` with a slice
  and a failure answers `Err`, which becomes main's end, untrapped, rc 245). Seen while the
  abandoned-call test started its messages from main: under parallel load main was sliced in its
  start loop when the nested Error 11 arrived. Pre-existing; the test now starts them from another
  activity.
- **Carry 2's crash took ten lines**, measured with this task's other changes in place (the yield
  is untouched by them), not a 480 MiB construction.
- The pool harness's `NATIVE_EXITS` counts the interpreter thread's exits only: an exit made by a
  pool thread holding a lend (its nested loop running another activity) is not counted, so the
  interleaved test does not assert `exits`.
