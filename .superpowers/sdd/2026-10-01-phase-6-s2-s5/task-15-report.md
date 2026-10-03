# Task 15 report: the synchronisation shim, the baton and the inbox under `loom`

Commits: `1a5e70c4a` (code and tests), then this report with the re-derived
`corpus/refusal-sites.tsv`. Base `92af930a0`.

## Design decisions

**Shim (`src/sync.rs`).** Re-exports `Arc`, `Mutex`, `MutexGuard`, `Condvar`, `AtomicU32`,
`Ordering` and `thread` from `std`, or from `loom` under `cfg(all(loom, test))`, plus `lock`,
`wait`, `wait_timeout` (poison-tolerant) and `now()`. Under `loom`, `now()` is a model clock
(`advance` moves it), since `loom` models no time. The condition is `all(loom, test)`, not `loom`
alone: `tests/loom.rs` compiles `baton.rs`, `sync.rs` and `timer.rs` into the test crate with
`#[path]`, so `loom` checks the shipped source files, while the library itself (built without
`cfg(test)` for an integration test) keeps `std`. `loom` therefore stays a plain dev-dependency;
nothing in `src/` ever links it. `cfg(loom)` is declared in the workspace `unexpected_cfgs`.

**Baton (`src/baton.rs`, new file).** `Mutex<Option<ThreadId>>` plus a `Condvar` notified at each
release. States: free (`None`), held by thread T (`Some(T)`). `acquire` waits in a `while` loop
until free, then records the caller; `release` asserts the caller holds it, clears, notifies one;
`held_here` compares with `thread::current().id()`. `Interp::new` acquires it, so the creating
thread holds the baton for the interpreter's life; there is no release in shipped code, because
the spec allows a release only at a driver exit (P6-3) and none exists yet. `release` is
`#[cfg(test)]` until its first caller (Task 17). `Baton` is its own file so the `loom` test can
compile it; it is not in `scheduler.rs`.

**`Scheduler::stop_the_world`** (trait method, first caller `Interp::collect_now`, the one
collection site, `lib.rs`): asserts `self.baton.held_here()` with the message "a collection on a
thread not holding the interpreter's baton".

**Inbox (`src/timer.rs`, created here per R-T4-1).** `Inbox<T>` holds the request word
(`Requests`, bits `SLICE` and the new `INBOX`), a `Mutex<Queue<T>>` (`posted: VecDeque<T>`, `woken:
bool`) and a `Condvar` `arrived`. `post` (test-only until Task 17) pushes, sets `INBOX` and
notifies, all under the lock but the notify. `drain` (test-only until Task 17) returns nothing
without locking while `INBOX` is clear, else takes the queue and clears `INBOX` under the lock.
`idle` blocks until `woken` or a non-empty queue, then takes. The interpreter's inbox type is
`Inbox<Posted>` with `Posted = Infallible`; `Interp::idle_until` matches what `idle_until`
answers with `match posted {}`.

**Timer on the inbox.** The holder now idles on its inbox, not on a per-entry condvar under the
registry lock: `Registration::idle_until` forgets a stale wake (`expect_wake`), publishes
`idle_until` and notifies `changed` under the registry lock, idles on the inbox, then clears its
deadline. The timer's `tick` sets `SLICE` and then wakes a due idler through `Inbox::wake` (lock
order registry, then inbox; the holder never holds both). One park point serves both wake sources,
which is where the nested pinned loop blocks on completions. The registry is no longer a `const`
static (loom's `Mutex::new` is not `const`): `registry()` is a `LazyLock` in shipped builds and a
`loom::lazy_static` per model execution. The timer thread ends when the registry is empty
(`Registration::drop` notifies `changed`), and the next arm or idle starts a new one; a model must
join every thread, so under `loom` the `JoinHandle` is kept for `join_timer`.

**Memory orderings.** Unchanged from Task 4: `set` is `fetch_or(Release)`, `pending` is
`load(Acquire)`, `clear` is `fetch_and(AcqRel)`. No data is published through a request bit: a
post's item, the `woken` flag and the baton's holder are all read under a mutex, so the bits only
need to be eventually visible. Mutation M9 confirms this: all three weakened to `Relaxed`, every
model stays green. They are kept as they were (no cost change, and Release/Acquire stays right if
a bit ever does publish data).

## `loom` models (`tests/loom.rs`)

| Model | Proves |
|---|---|
| `the_baton_passes_to_a_waiting_thread` | a pool thread blocked in `acquire` takes the baton after the holder's release, never while the holder has it |
| `the_baton_passes_to_one_waiting_thread_at_a_time` | with two waiters, each release admits exactly one; no double holder, no stuck waiter |
| `completions_posted_while_the_holder_drains_arrive_once_in_order` | posts racing `drain` and `idle` reach the holder once each, in order, and `INBOX` is clear once all are taken |
| `the_timer_sets_slice_while_the_holder_idles` | an armed interpreter idling on a deadline sees `SLICE` pending when the timer's wake ends the idle; arm/disarm reach the registry |
| `an_idle_deadline_is_never_lost` | an idle whose deadline the timer finds due ends whether the wake lands before or after the holder blocks; twice in a row (stale wake); and after the timer thread ended with the registry empty and was started anew |

## Mutation evidence

Run by `scratchpad/t15/mut/mutate.py` (copy, mutate, `RUSTFLAGS="--cfg loom" cargo test --release
--test loom --no-fail-fast`, restore from the copy and assert byte equality).

| Mutation | Caught by |
|---|---|
| M1 `Baton::release` without `notify_one` | `the_baton_passes_to_a_waiting_thread`: loom deadlock |
| M2 `Baton::acquire` `if` instead of `while` | `the_baton_passes_to_one_waiting_thread_at_a_time`: release assertion (two holders) |
| M3 `post` sets `INBOX` after unlocking | `completions_posted...`: `INBOX` left set over an empty queue (`loom.rs:110`) |
| M4 `post` without `notify_one` | `completions_posted...`: loom deadlock |
| M5 `idle` waits once, ignoring its predicate | `completions_posted...`: loom deadlock |
| M6 `expect_wake` after publishing the deadline | `an_idle_deadline_is_never_lost`: loom deadlock (timer's wake erased) |
| M7 `idle_until` without notifying `changed` | `an_idle_deadline_is_never_lost`: loom deadlock |
| M8 `Inbox::wake` without `notify_one` | `an_idle_deadline_is_never_lost`: loom deadlock |
| M9 every request-bit ordering `Relaxed` | not caught, as expected (see orderings) |
| M10 `Registration::drop` without notifying `changed` | `an_idle_deadline_is_never_lost` and `the_timer_sets_slice...`: loom deadlock (timer never ends) |
| removing `stop_the_world` from `collect_now` | lib test `a_collection_without_the_baton_fails_its_assertion` fails (debug, `cargo test -p rexx-exec --lib baton`) |

## The `loom` gate command

From `rust/`, empty target dir:

```
RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=<own dir> memcap 8G cargo test -p rexx-exec --test loom
```

Measured: 33.7 s wall from an empty target dir (`/usr/bin/time`), of which the models take 1.75 s;
5 passed. Debug, not release: release builds the workspace's fat-LTO binaries (about 70 s) for a
1.4 s test run. `RUSTFLAGS` rebuilds every dependency, hence the own target dir.

## Checks (at `1a5e70c4a`)

From `rust/`, statuses written unpiped by `scratchpad/t15/checks/run.sh` (`CARGO_TARGET_DIR` in
scratch), at `1a5e70c4a`:

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; `cargo clippy -p rexx-exec
  --all-targets --features pinning -- -D warnings`: exit 0; `RUSTFLAGS="--cfg loom" cargo clippy -p
  rexx-exec --test loom -- -D warnings`: exit 0.
- `memcap 8G cargo test -p rexx-exec --lib`: 945 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: `787 of 787
  matching`; with `REXX_CORPUS_SWITCH=every`: 787 of 787; debug with every: 787 of 787.
- `collect_stress`: release 36 passed, debug 36 passed.
- `--release --test refusal_sites --test method_bodies --test gate_table_c --test dispatch_seam`:
  exit 101. `dispatch_seam` 6, `gate_table_c` 22, `method_bodies` 23 passed; `refusal_sites`
  `the_table_holds_every_constructor_the_source_defines` failed because `corpus/refusal-sites.tsv`
  cites `lib.rs` line numbers and the new module lines moved them. Re-derived with
  `REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites` (5 passed); the diff strips to identical
  rows once `lib.rs:<n>` is masked, so only line numbers moved. No other file under `corpus/` or
  `crates/*/tests` cites `lib.rs` line numbers (`/bin/grep -arln 'src/lib\.rs:[0-9]'`).
  Committed with this report.
- `rexx-parse` `--test sourceline_oracle`: 1 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test concurrency_tests --
  --test-threads=1`: 32 passed, 838 s.
- Pinning: `RAYON_NUM_THREADS=4 CARGO_TARGET_DIR=<own> memcap 8G cargo test --release -p rexx-exec
  --features pinning --test concurrency_tests -- measured:: --nocapture --test-threads=1`: 18
  passed.
- `loom` (the gate command above, own target dir): 5 passed, 33 s wall.
- New lib tests: `a_collection_without_the_baton_fails_its_assertion` (should-panic; removing the
  `stop_the_world` call makes it fail), `the_creating_thread_holds_the_baton_and_collects`,
  `the_inbox_answers_posts_in_order_then_nothing`, `a_post_from_another_thread_ends_an_idle`.

## Performance (P43)

`bench-programs/callgrind.sh -r 1 -j 6 -p "dispatch fibcall rexxcps emptyloop sendloop startup"
base=<92af930a0> head=<1a5e70c4a>`, each `rexx-run` built release from `git archive` in its own
target dir with files touched (`Compiling rexx-exec` seen for both). Ir less libc and ld-linux:

| program | base | head | delta |
|---|---|---|---|
| dispatch | 21264424294 | 21264385265 | -0.0002% |
| fibcall | 8494044049 | 8494020576 | -0.0003% |
| rexxcps | 17800655598 | 17800619490 | -0.0002% |
| emptyloop | 7785903869 | 7785806246 | -0.0013% |
| sendloop | 14343771082 | 14343751585 | -0.0001% |
| startup | 58096537 | 58096570 | +0.0001% |

Nothing is added per clause or per send: the baton is taken once in `Interp::new`, and
`stop_the_world` runs once per collection.

"It works": every `rust/bench-programs/*.rex` on both binaries (release, `LD_LIBRARY_PATH` at the
oracle's `build/lib`): stdout and rc identical except `heapshape.rex`'s `build_seconds=` and
`gc_pause_seconds=` timing lines.

## Plan departures

- Files: `baton.rs` is new (not in `scheduler.rs`) so `tests/loom.rs` can compile it.
- "Takes and releases it around the scheduler loop": the creating thread takes the baton in
  `Interp::new` and nothing releases it; spec P6-3 releases only at a driver exit, `Interp::new`
  already touches state, and the crate's unit tests drive an `Interp` directly.
- `post`, `drain` and `release` are `#[cfg(test)]` until their first callers (Tasks 17 and 19).
- The timer thread ends when no interpreter is registered and is spawned again by the next arm or
  idle (ruling P49; spec section 4 amended in fix round 1). Spec section 4 had it sleep with no
  deadline for the process's life.

## Concerns

- `Interp::idle_until`'s deadline branch marks the run expired once `idle_until` returns. Today
  only the timer ends an idle (`Posted` is uninhabited); once posts exist, an idle ended by a post
  before the run's deadline must loop rather than expire.
- No request-bit ordering is load-bearing (M9 green), so `loom` cannot distinguish the chosen
  orderings from `Relaxed`.

## Fix round 1

Commit `d502c140c` (review `task-15-review.md`).

- **I1.** New model `a_sleeper_registers_as_the_timer_exits` (the reviewer's scratch model): one
  registration leaves, so the timer may decide to end, while another thread registers and idles.
  `join_timer` now joins every timer thread started (`State::timers` under `loom`), since a model
  can start a second timer while the first is ending. Mutation, the timer's exit clearing
  `timer_running` after dropping the registry lock (`drop(live); lock(&registry.state).timer_running
  = false; return;`): `cargo test --test loom -- --exact a_sleeper_registers_as_the_timer_exits`
  fails with loom's `deadlock; threads = [(Id(0), Blocked), (Id(1), Terminated), (Id(2),
  Blocked)]`; `an_idle_deadline_is_never_lost` stays green under it (1 passed). Restored from a
  copy, byte-compared.
- **M1.** `start_timer` sets `timer_running` only after `spawn` returns.
- **M2.** Ruling P49: listed under Plan departures; spec section 4's timer sentence amended, citing
  P49.
- **M3.** Gated: `lib.rs` carries `#![cfg(not(all(loom, test)))]`, so under `--cfg loom` the
  library's unit-test crate compiles to nothing and never meets `loom`'s primitives outside a model;
  `sync.rs`'s doc says so. The normal library build, which the integration tests link, is
  unaffected. `RUSTFLAGS="--cfg loom" cargo test -p rexx-exec --lib`: 0 tests, exit 0.
- **M4.** `Inbox::wake`: "Called with the registry locked: never take the registry under `queue`."
- `corpus/refusal-sites.tsv` re-derived (`REXX_REFUSAL_SITES_REFRESH=1`) for the 4 lines `lib.rs`
  gained; with `lib.rs:<n>` masked the removed and added rows are identical (46 each).

Checks at `d502c140c`, from `rust/`:

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; `-p rexx-exec --all-targets
  --features pinning`: exit 0; `RUSTFLAGS="--cfg loom" cargo clippy -p rexx-exec --all-targets -- -D
  warnings`: exit 0.
- `memcap 8G cargo test -p rexx-exec --lib`: 945 passed.
- `loom`, the gate command from an empty target dir: 6 passed, models 9.06 s, 45.5 s wall
  (`/usr/bin/time`).
- `RUSTFLAGS="--cfg loom" cargo test -p rexx-exec --lib`: 0 passed, exit 0.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: `787 of 787
  matching`, 29 passed.
- `--release --test refusal_sites`: 5 passed.
