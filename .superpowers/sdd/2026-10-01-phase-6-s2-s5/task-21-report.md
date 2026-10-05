# Task 21 report: signals

Base `12f5d843a`. Commits: `532bf29fa` (signals), `ebba80f32` (refusal-sites.tsv re-derived, the
nohup SIGHUP witness); this report and the spec 4 P49 sentence are committed after them.

## Step 1: the self-pipe, settled

- **Where the pipe lives.** `sync::Wake` is a `UnixStream::pair` owned by the registry, which is a
  never-dropped static (`timer.rs`). It is created with the registry and never closed, so the write
  end the handlers hold stays valid for the process's life, across every timer-thread end and
  respawn (P49). The handlers reach it through `signal.rs`'s `WAKE` (the raw fd) and set `PENDING`;
  those two atomics and the install `Once` are the signal half of the timer's wake source and the
  only statics added. No other mutable global.
- **Read timeout.** The timer thread no longer waits on a condvar under the registry lock. Each
  round it locks the registry, serves a pending signal, ticks, unlocks, then reads the socket with
  `set_read_timeout(next deadline)` (no timeout where nothing is due; a zero wait skips the read).
  One read takes up to 64 bytes, so a burst of wakes is one wake.
- **Arming through the same socket.** Registration, `arm`, `idle_until` and a registration's drop
  write one byte where they notified the condvar. A byte persists, so a wake written between the
  timer's unlock and its read is not lost.
- **Nonblocking write end.** `set_nonblocking(true)` on the writer: a handler never blocks, and a
  full socket already holds a wake (the write's `EAGAIN` is ignored).
- **errno saved in the handler.** `handler` saves `*__errno_location()` before its `write` and
  restores it after; it does an atomic store, an atomic load and one `write`, and nothing else.
- **Installed where none is set.** At the first `Interp::new` in the process (through
  `Registration::new`): for each of SIGINT, SIGTERM, SIGHUP, `sigaction` reads the previous action
  and installs only where it is `SIG_DFL`; `sa_mask` full, `sa_flags` 0 (no `SA_RESTART`). SIGPIPE
  is set to `SIG_IGN` unconditionally, as the oracle does.
- **Who reads when no timer thread exists.** The timer thread now starts at every registration
  (P49 as the lead summarised it: ends when none is registered, respawns on the next registration),
  so whenever an interpreter is live a thread reads the socket. Arm and idle no longer start it. With
  none registered nobody reads: the byte stays in the socket (or is dropped with `EAGAIN` when full)
  and `PENDING` stays set. The first registration after that discards `PENDING`: a signal with no
  live interpreter halts nothing, as `haltAllActivities` over no instance halts nothing. Neither lost
  in a way later wakes notice (the stale byte is one spurious wake) nor fatal (the reader never
  closes, so no `EPIPE`/SIGPIPE, and the default action is no longer installed).
- **Orderings.** The pending flag is stored and swapped `SeqCst` on both sides. With `Release` and
  `AcqRel`, loom 0.7.2 lost the model's store (a `Release` store followed by a `SeqCst` load in the
  same thread read `false`), and the model then also lost it with `SeqCst` on a loom atomic; the
  model keeps the flag under its wake lock instead (`sync.rs`). The shipped flag is a std atomic
  followed by a `write`, which orders it for the reader either way.
- **How the halt reaches an interpreter.** The timer posts `Posted::Halt` to every live inbox
  rather than setting a request bit: a post already sets `INBOX` (a request bit the countdown's cold
  visit serves), wakes an idle holder, and is drained by every idle path the scheduler has. The
  holder's `halt_all` then queues a `HALT` request (`PendingTrap { request: true }`, the
  `Message~halt` mechanism) on every activity with a running activation, and wakes one asleep in
  `SysSleep` (which then answers EINTR, 4, as `nanosleep` does) or parked in `GUARD WHEN` (which
  raises it at its re-test). Others take it when they next run a clause.
- **Safe alternative tried: `signal-hook`.** A `#![forbid(unsafe_code)]` probe
  (`signal-hook` 0.3.18, `signal-hook-registry` 1.4.8) registering a socket for SIGHUP with
  `low_level::pipe::register` and raising it prints "the registered handler ran" both plainly and
  under `nohup`: it installs over an ignored disposition (and chains to a previous handler), and it
  has no safe way to read the previous action. So it cannot install only where none is set, which
  the oracle's check and the nohup row rest on; reading the action needs `sigaction` anyway.

## Witnesses

Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=... timeout -k 3 --preserve-status -s SIG 1 [nohup]
rexx FILE )`, each run from a fresh empty directory; this crate: the same without the `ulimit`, on
`rexx-run`. Counts are runs with identical rc, stdout and stderr (path masked).

| program | signal | oracle (30 runs) | this crate (30 runs) |
| --- | --- | --- | --- |
| `say 'before'; rc = SysSleep(5)` | INT, TERM, HUP | rc 252, 4.1 at line 2, ~1 s, 30/30 each | identical, 30/30 each |
| same, `signal on halt` | INT | rc 0, `halted  3 4` (EINTR assigned) | identical |
| same, `call on halt` | INT | rc 0, `after 4` then `halted  4` | identical |
| `say 'ready'; do forever; end` | INT | rc 252, 4.1 at `end` line 3 | identical |
| main sleeps, a started activity parked in `GUARD WHEN` | INT | main's 4.1, then hangs: killed, rc 137 | rc 252, main's 4.1 then the activity's 4.1 at line 13 |
| main ends, the activity parked in `GUARD WHEN` | INT | hangs: killed, rc 137 | rc 0, the activity's 4.1 at line 10 |
| sleep under `nohup` | INT / TERM | killed by the signal: 130 / 143 | rc 252, 4.1 (licensed row) |
| sleep under `nohup` | HUP | ignored: killed by `-k`, 137; with `SysSleep(2)`, rc 0 `after 0` | identical |
| 200000 `say` lines piped to `head -1` | SIGPIPE | rc 3, empty stderr (3 runs) | identical (3 runs) |

The SIGPIPE row cannot witness this crate's `SIG_IGN`: Rust's runtime ignores SIGPIPE before
`main` in `rexx-run` and in the test binaries alike.

## Tests and mutants

`tests/signals.rs`: each witness above but the SIGPIPE one, as a subprocess of the built
`rexx-run`, signalled by PID with `kill -s` once its readiness line is out (the busy loop, whose
output this crate hands over only when it ends, after one second). Expected bytes are the
oracle's. Each test is wall-clock bound (P48): a mismatch or an end later than 3 s after the
signal is run again once, and a second one fails. They live here, not in `concurrency_tests.rs`,
whose `WALL_CLOCK` list names ootest rows; the same rule is applied in the file. `tests/loom.rs`:
`a_signal_halts_an_idle_holder`, `a_signal_between_interpreters_halts_nothing`; the existing
timer models now run over the modelled wake source, and every model that registers joins the
timer thread.

Mutants, applied and reverted by hand-written script with a copy restore, `cargo test --profile
mutation -p rexx-exec --test signals --no-fail-fast` (unmutated control: 10 passed):

| mutant | red |
| --- | --- |
| M1 handler writes no byte | every signal test but `under_nohup_sighup_stays_ignored` |
| M2 halt wakes no sleeper | the sleep, trapped, nohup and parked-guard tests |
| M3 interrupted `SysSleep` answers 0 | `signal_on_halt_takes_a_signal`, `call_on_halt_takes_a_signal` |
| M4 `GUARD WHEN` takes no halt | both guard tests |
| M5 install over `SIG_IGN` | `under_nohup_sighup_stays_ignored` |
| M6 timer not started at registration | all |
| M7 program-end idle ignores a halt | `sigint_ends_a_program_waiting_on_a_guard_when` |
| M8 timer posts no halt | every signal test but `under_nohup_sighup_stays_ignored`; loom `a_signal_halts_an_idle_holder` (deadlock) |
| M9 `SA_RESTART` set | **survives** |
| M10 no discard at first registration (loom) | `a_signal_between_interpreters_halts_nothing` |
| M11 model signal skips its wake (loom) | the run aborts at `a_signal_halts_an_idle_holder` (exit 101; the deadlock text was read for M8 only) |

M9 survives because nothing here halts through an interrupted system call: a sleep is a park,
and the timer's read wakes on the byte either way. `SA_RESTART`'s absence is kept for the oracle's
reason (blocking reads elsewhere return `EINTR`), unwitnessed.

## Divergence row

`phase-4-exclusions.txt` DEVIATIONS entry 10, OWNER: none: under nohup this crate halts where the
oracle dies, and a parked `GUARD WHEN` is woken and halted where the oracle hangs. Witnessed by
`under_nohup_sigint_still_halts`, `sigint_reaches_a_parked_guard_when` and
`sigint_ends_a_program_waiting_on_a_guard_when`. No `LICENSED DIVERGENCE WITNESS:` marker: the
witness needs a signal and a subprocess, which `licensed_divergences.rs` cannot run.

## Checks

Target dirs under `p6-scratch/t21/`. At `ebba80f32`:

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; with `--features pinning`:
  exit 0; with `RUSTFLAGS="--cfg loom"`: exit 0.
- `cargo test --workspace --release --no-run`: exit 0; then `memcap 8G cargo test --workspace
  --release`: exit 0, 2997 passed, 0 failed, 4 ignored over 143 result lines. (The same run at
  `532bf29fa`, with `--no-fail-fast`, failed only `refusal_sites`: `lib.rs`'s new `mod signal`
  moved the Loud constructors' lines; re-derived with `REXX_REFUSAL_SITES_REFRESH=1` in
  `ebba80f32`.)
- loom, `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=.../t21/target-loom memcap 8G cargo test -p
  rexx-exec --test loom`: still running when this report was committed; see the addendum.

## Departures and concerns

1. **The halt is a post, not a request bit.** `Posted::Halt` through the inbox (Step 1). `INBOX` is
   the request bit that carries it, so the countdown's cold visit serves it; no new bit.
2. **The timer starts at every registration**, not at arm or idle, so a busy single-activity
   interpreter (which never arms) has a reader for its signals. Spec section 4's P49 sentence said
   "started again by the next arm or idle"; it now says "by the next registration", which is
   P49's own wording. Cost: one thread spawn at the first registration after the registry empties,
   unmeasured (no perf run under P51).
3. **Which parks a signal wakes.** A `SysSleep` and a `GUARD WHEN` (also at program end, through
   `idle_for_good`, which now returns when a halt readies an activity). A guard-lock, message,
   semaphore, timer, native or blocking wait is not woken; it takes the halt at its next clause.
   Spec 4 says a parked one "is woken"; the brief names the `GUARD WHEN` witness. Waking the rest
   needs per-kind withdrawal and a resume-with-halt path; not done. All sleepers are woken, where
   the oracle's signal interrupts only the `nanosleep` of the thread it lands on.
4. **The `GUARD WHEN` wake diverges from the oracle**, which hangs; it is recorded in DEVIATIONS
   entry 10 beside the brief's nohup row, OWNER none. Needs the lead's ruling: the brief licensed
   only the nohup half. A `CALL ON HALT` trap on a woken `GUARD WHEN` queues and the wait parks
   again, so its handler runs when the wait ends (not witnessed).
5. **Handlers are installed in every process that starts an interpreter, including the test
   binaries** (`Interp::new` registers). Ctrl-C on a test binary now halts its live interpreters
   instead of killing it; with none live it does nothing. Same for `rexx-run` after its
   interpreter has ended: a signal then neither halts nor kills. As the oracle's library does.
6. **Witnesses live in `tests/signals.rs`, not `concurrency_tests.rs`**; P48's rerun-once rule is
   applied in that file, since `WALL_CLOCK` names ootest rows.
7. **`loom` run time.** Every model that registers now has a timer thread from the start; see Checks
   for the cost. Four existing models gained `drop(registration); timer::join_timer()` (loom
   otherwise ended an execution under a live timer thread and panicked inside loom's object store).
8. **loom 0.7.2 lost a `Release` store** on the model's atomic flag (Step 1, Orderings); the model
   keeps the flag under its wake lock. Not investigated further.
9. **M9 (`SA_RESTART`) survives**, and the SIGPIPE `SIG_IGN` cannot be witnessed from a Rust host.
10. **`rexx-run` hands a busy loop's output over only when it ends** (the busy witness signals after
    a second rather than on its `ready` line). Pre-existing; the bytes agree with the oracle.
