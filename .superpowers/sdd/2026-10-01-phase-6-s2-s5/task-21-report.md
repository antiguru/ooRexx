# Task 21 report: signals

Base `12f5d843a`. Commits: `532bf29fa` (signals), `ebba80f32` (refusal-sites.tsv re-derived, the
nohup SIGHUP witness); this report and the spec 4 P49 sentence (`8bb1304f1`), and the loom addendum after them.

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
  rexx-exec --test loom`: **did not finish.** 12 of 14 models passed (the two signal models and
  every baton, inbox and timer model but two); `a_sleeper_registers_as_the_timer_exits` and
  `an_idle_deadline_is_never_lost` were still exploring when I stopped the run by PID after 1 h 39 min
  (a `--release` run alongside: same two, stopped after 37 min). Both register more than once, so
  each now explores a timer thread from its first registration and a respawn. With
  `LOOM_MAX_PREEMPTIONS` on the release binary, both pass at every bound tried:

  | bound | `an_idle_deadline_is_never_lost` | `a_sleeper_registers_as_the_timer_exits` |
  | --- | --- | --- |
  | 3 | 0.09 s | 0.43 s |
  | 4 | 0.65 s | 4.08 s |
  | 5 | 3.14 s | 27.90 s |
  | 6 | 12.51 s | 159.06 s |
  | 7 | 43.96 s | 734.76 s |

  Growth is about 5x per bound level, so the unbounded gate command will not finish in a gate's
  time. Needs a ruling: a preemption bound for the loom gate (or those two models), or the timer
  not started at registration.

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
7. **`loom` does not finish unbounded** (Checks). Every model that registers now has a timer thread
   from its first registration; two Task 15 models no longer finish. Four existing models gained `drop(registration); timer::join_timer()` (loom
   otherwise ended an execution under a live timer thread and panicked inside loom's object store).
8. **loom 0.7.2 lost a `Release` store** on the model's atomic flag (Step 1, Orderings); the model
   keeps the flag under its wake lock. Not investigated further.
9. **M9 (`SA_RESTART`) survives**, and the SIGPIPE `SIG_IGN` cannot be witnessed from a Rust host.
10. **`rexx-run` hands a busy loop's output over only when it ends** (the busy witness signals after
    a second rather than on its `ready` line). Pre-existing; the bytes agree with the oracle.

## Fix round 1

Base `e86c8fa29`. Commits: `9b47a4df5` (F1-F10), `8063a6939` (a `HALT` request bit for F1's
check), `492f0bab8` (a stale command end witnessed; an unreachable token filter dropped), and
this section. Oracle runs: the CLAUDE.md wrapper from fresh empty directories, each program
signalled by PID (or by its process group where a row says so), with SIGINT, SIGTERM and SIGHUP
reset by `env --default-signal`. This crate's runs are the same without the `ulimit` (concern 2).
"Identical" means rc, stdout and stderr, with the directory masked.

### F1: a halt posted during a blocking operation is served when it returns

`Interp::serve_posted_halt` (`scheduler.rs`) runs after every builtin (`builtin::run`) and after
every command clause that waited on the baton (`exec_command`). It takes a pending `Posted::Halt`
out of the inbox, puts every other post back for the next drain, and calls `halt_all`. The
request is then queued for the clause still running, which raises it at its end: where the
oracle's `processClauseBoundary` raises it, after every instruction including the last. Only the
halt is filed there: a `Recall` filed mid-expression would lend the baton out with a builtin's
answer still unrooted.

The first version tested `INBOX`: 23 Ir per builtin call, and in a multi-activity program it
drained and requeued unrelated posts on every call. The timer now sets a `HALT` request bit with
each halt it posts (`timer.rs`), and the check is an inline test of that bit with a cold body.
Callgrind (`bench-programs/callgrind.sh -r 1`), `e86c8fa29` against `8063a6939`'s tree: rexxcps
+0.10%, strings +0.41%, textnum +0.28%, parse +0.29%, emptyloop +0.0000%: about 6 Ir per builtin
call, nothing per clause (P43). Concern 1.

Program end: a halt queued before the last clause ends is raised by that clause; after main's
last clause, a halt reaches started activities through `next_runnable`'s drain; with no activity
left there is nothing to halt, as on the oracle.

Witnesses (`tests/signals.rs`), each run 30 times on the oracle and on this crate, identical in
every run: `sigint_halts_after_a_long_builtin` (the review's `copies` loop, 4.1 at line 2),
`ctrl_c_halts_a_script_running_commands` (the review's group SIGINT: line 2 at 1.0 s, where it was
`b -2`/`c 0` at 3 s), `sigint_halts_a_last_clause_command`. `ctrl_c_halts_a_redirected_command`
(a redirected command still waits on the baton) is a divergence: the oracle gives 98.923, 30 of
30 (F2). Mutants: M1 no serve after a builtin, red on the builtin witness; M2 no serve after a
command clause, red on the redirected witness; M15 the timer sets no `HALT` bit, red on both.

### F2 (P60): a halt ends the command wait and reads of standard input

**Design: off-baton abandonment, not poll.** For the command wait the machinery existed: a command
off the baton parks its activity on `ParkReason::Block` while a pool thread waits for the child
and posts `Unblocked`. Two changes. An unredirected command now takes that path everywhere outside
a park's continuation, where it needed a second live activity or a test mode (`blocks_off_baton`
is gone). And `wake_for_halt` abandons a `Block` park: the clause gets `Blocked::abandon`'s ending
(RC -4), its token joins `abandoned_blocks` and leaves `in_flight`, so the program's end does not
wait for the child, which runs on as the oracle's does; the late `Unblocked` for that token is
dropped. Poll would have needed a descriptor per interpreter that the timer writes on a halt (the
inbox is a condvar), and the child's pipes polled on the holder's thread, where `collect` drains
them concurrently on purpose.

Standard input the same way (`input.rs`): `Source::Stdin` keeps its own buffer; `fill_stdin` has
a pool thread read one 8 KiB chunk and post it as `Posted::Input`, while the holder idles on the
inbox, keeping other posts aside and requeueing them after. On `Posted::Halt` it marks the input
interrupted, calls `halt_all` and answers nothing: as the oracle's interrupted read, the null
string, then NOTREADY, and CHARS and LINES 0. The read still in flight is filed when it arrives,
so the next read loses nothing. Where no pool thread can be reserved the chunk is read inline.
LINES never blocks, here or on the oracle (it answered 0 at once on an open FIFO), so it has
nothing to interrupt.

The halt from an interrupted read lands in the `.INPUT` monitor's forwarding `UNKNOWN` activation.
Untrapped, that gives the oracle's traceback, `Monitor` line included, unchanged. A `CALL ON HALT`
trap found through that forwarding activation is now queued on its owner, the caller, and
delivered at the end of the caller's clause, as on the oracle (`halted 3` before `v=[]`); it was
queued on the monitor's activation and never delivered.

The child under a PID-only signal keeps running on both; a terminal Ctrl-C signals the group, the
child dies of it, and both halt at the command's line (the group witness above).

Witnesses, each identical to the oracle in 30 of 30 runs except where noted:
`sigint_ends_a_command_wait`; `call_on_halt_sees_an_abandoned_command` (RC differs: -4 here 30 of
30, the oracle's unset `waitpid` status -100, -64, -44, -38 and -36 across 30 runs);
`an_abandoned_command_ends_no_later_one` (oracle 3 runs, the same but RC);
`sigint_ends_a_parse_pull`, `sigint_ends_a_linein`, `sigint_ends_a_charin` (Monitor frame);
`sigint_ends_a_stdin_linein` (none); `call_on_halt_sees_an_interrupted_read`. Mutants: M3 the
command waits inline, red on five command witnesses; M4 no abandonment, red on four; M5 the read
ignores the halt, red on all five read witnesses; M6 no interrupted mark and M7 the CALL ON trap
queued on the running activation, each red on the CALL ON read witness; M16 the late end not
dropped, red on `an_abandoned_command_ends_no_later_one`.

DEVIATIONS entry 10 takes the RC, the abandoned child's later output and the read that loses
nothing; new entry 12 (OWNER: none) the waits a halt does not end: other streams, a redirected
command (the oracle's 98.923), and either wait under `ulimit -v`.

### F3: a halt withdraws a semaphore wait; a message wait is not withdrawn (departure)

`park` records a semaphore wait on the activity, with a timed wait's sleeper order;
`wake_for_halt` removes the sleeper, withdraws it from the semaphore's queue
(`Semaphores::withdraw` now says whether it was queued), marks it and readies it.
`retest_semaphore` answers a marked wait without re-testing: an untimed `Sys*Sem` wait `0`, as the
oracle's `sem_wait` interrupted by EINTR answers (`SysRexxUtil.cpp:836-850`), every other wait
that it took nothing. `halt_all` now goes in handle order, so main's report comes first, as on the
oracle. Witnesses, each identical to the oracle in 30 of 30 runs: `sigint_ends_a_semaphore_wait`
(the review's program: main's 4.1 at line 5, then the activity's at line 10; this crate ends at
1 s, the oracle at 10 s because it does not wake the sleeper) and
`call_on_halt_sees_a_withdrawn_semaphore_wait` (`after 0`, `halted 7`). Mutants: M8 no
withdrawal, red on both; M9 a withdrawn untimed wait answers 121, red on the second.

**Message waits are not withdrawn.** I implemented it and measured it moving this crate away from
the oracle. On the oracle a halt leaves `m~result` waiting until the runner halts, and main then
re-raises the runner's condition (`11 *-* call SysSleep 3` / `Error 4 running ... line 5` under
`CALL ON HALT`); withdrawn, main answered `.nil`, printed `after The NIL object` and ran its
handler. Without withdrawal the runner, always woken now (P59), completes the message with its
halt and main re-raises it as the oracle does: that program's output is identical to the oracle's,
and the untrapped one differs only by the oracle's interleaving of its two threads' lines. Every
runner a halt reaches ends, so a message wait no longer meets the refusal except behind a runner a
halt does not wake (concern 3).

### F4 (P61): install rules

`signal.rs`'s `HALTING` pairs each signal with whether it installs over `SIG_IGN`: SIGINT and
SIGTERM yes, SIGHUP no; anything else, a handler, is left alone. Witnesses:
`sigint_ignored_at_start_still_halts` (`sh -c "trap '' INT; exec ..."`, identical to the oracle,
30 of 30) and `under_nohup_sighup_stays_ignored`. Mutants: M10 SIGINT not over `SIG_IGN`, red on
the first; M11 SIGHUP over `SIG_IGN`, red on the second. DEVIATIONS entry 10 now states both
installs, the oracle's (SIGHUP's action decides all three) and this crate's.

### F5 and F11 (P62): handlers at the entry point

`Registration::new` no longer installs; `rexx_exec::install_signal_handlers()` (`lib.rs`) does,
and `rexx-run`'s `main` calls it first. The C API's interpreter creation does not exist yet
(`RexxCreateInterpreter` is Phase 9's), so there is no second call site. The witnesses spawn
`env --default-signal=INT,TERM,HUP rexx-run`, with `nohup` or `sh -c` after `env` where a test
needs them. `signal::tests::running_a_program_installs_no_handler` runs a program and asserts the
install never ran; M13 (install at registration again) turns it red; M12 (`rexx-run` without the
call) turns every signal witness but the nohup SIGHUP one red. The signals binary run as
`sh -c "trap '' INT TERM HUP; exec <bin>"`: 25 of 25 pass; with the `env` reset removed from the
harness the same run fails `sighup_halts_a_sleep` alone (under P61 an ignored SIGINT or SIGTERM is
installed over anyway). The witnesses now print a line when P48 reruns them.

### F6 (P58): preemption bound

`registration_model` (`tests/loom.rs`), a `loom::model::Builder` with `preemption_bound =
Some(5)`, runs `an_idle_deadline_is_never_lost` and `a_sleeper_registers_as_the_timer_exits`, and
F7's new model (concern 4); the others stay unbounded. The loom gate now finishes (Checks).

### F7: every live interpreter

`a_signal_halts_every_live_interpreter`: two registrations, one modelled signal, each inbox must
get exactly one halt. M14 (the halt posted to the first registration only) turns it red: loom
deadlock, the second idles for ever.

### F8: the modelled flag is the shipped one

The model's `Wake` holds the signal flag as a loom `AtomicBool`: `signal` does
`store(true, Release)` then a notify, `take_signal` `compare_exchange(true, false, AcqRel,
Acquire)`. The shipped code now does the same: `signal::handler` stores with `Release` and
`take_pending` uses that `compare_exchange` in place of `swap(SeqCst)`, so model and code run the
same operations, and the operation is one the reviewer measured loom reading correctly. Either is
correct on hardware: an RMW reads the latest value whatever its ordering. All three signal models
pass.

**Correction to Step 1's "Orderings" bullet and concern 8 above:** loom 0.7.2 did not lose a
`Release` store. Its RMW `swap` and `fetch_and` read a stale value despite the mutex's
happens-before (the review's standalone repro); `compare_exchange` and `load(Acquire)` read the
store.

### F9: errno per platform

`errno_location()` (`signal.rs`): `__errno_location` on Linux and Android, `__error` on Apple
targets and FreeBSD, `__errno` on OpenBSD and NetBSD. `cargo check --target aarch64-apple-darwin
-p rexx-exec` now reports only the existing E0308 at `rexxutil.rs:154`; the mutant using
`__errno_location` on Apple brings E0425 back.

### F10 (P63): DEVIATIONS entry 11

New, OWNER: none: an ADDRESS child's SigIgn is `0x4` here and `0x1004` on the oracle, so
`yes | head -1` prints "Broken pipe" only on the oracle. No code change; measured by the review;
no suite witness.

### Mutants

Applied by a script that saves the file, writes the mutant, runs the command, and copies the saved
file back, asserting the bytes match; `git status` clean afterwards apart from the lead's
`progress.md`. M1-M12, M15 and M16 with `cargo test --release -p rexx-exec --test signals
--no-fail-fast`; M13 with `cargo test --release -p rexx-exec --lib --no-fail-fast signal::`; M14
with `RUSTFLAGS="--cfg loom" cargo test --release -p rexx-exec --test loom --no-fail-fast
every_live`. Every one went red as listed above; none survived.

### Checks (P51), at `492f0bab8`

Target dirs `p6-scratch/t21f1/target` and `.../target-loom`; each status captured unpiped to a
file.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; `--features pinning`: exit 0;
  `RUSTFLAGS="--cfg loom"`: exit 0.
- `cargo test --workspace --release --no-run`: exit 0; then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, 143 result lines, 3013 passed, 0 failed, 4 ignored.
- Loom: `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=.../target-loom memcap 8G cargo test -p
  rexx-exec --test loom`: exit 0, 15 passed, 363 s. It finishes.
- The same gates at `9b47a4df5` were also all exit 0 (3012 passed; loom 362 s).

### Concerns

1. **F1 costs about 6 Ir per builtin call** (rexxcps +0.10%, strings +0.41%). Nothing per clause,
   but every builtin call pays. The alternative is to check only after builtins that can run
   long, which needs a list; a builtin missing from it would not be served until the next
   countdown check.
2. **Commands now leave the baton in single-activity programs too**: one pool-thread hand-off per
   unredirected command, against a `fork`/`exec`, unmeasured; output streams in as it arrives
   rather than after the child's exit. The release suite is green. Under `ulimit -v 1G` a pool
   thread's 512 MiB stack cannot be reserved, so the command wait and the stdin read fall back
   inline and are not interrupted (DEVIATIONS 12). That is why this crate's probes ran without the
   `ulimit`, as Task 21's did. A command wait needs no big stack; a pool sized for it would remove
   this, not done.
3. **F3 departure: message waits are not withdrawn** (argued above). A message wait whose runner
   no halt wakes (a guard-lock, `Timer` or pinned-native park) still meets the "nothing left to
   run" refusal after a signal. The narrow fix would wake a halted parked activity at that refusal
   instead of refusing; not done, needs the lead's ruling.
4. **F7's model runs under the preemption bound too**, beyond P58's two: it registers twice, and
   unbounded took 378 s in release; bounded at 5, 3.14 s.
5. **Divergences added to entry 10**: the abandoned command's RC (-4 against an unset status), its
   child's later output reaching this interpreter only while it runs, and the next read after a
   halted read keeping every byte where the oracle lost a line (one run). Entry 12 has the
   redirected command's 98.923.
6. **The F5 witness is a command run, not a suite test**: the suite cannot change its own
   dispositions without `unsafe`.

## Fix round 2

Base `159231bb8`. Commits: `f9fb61990` (N1-N5 fixes, F1's method case, witnesses),
`2e50fc410` (N7's reset moved to one site; the redirected case of the N4 witness), and this
section with `task-21-rereview-1.md`. Oracle runs: the CLAUDE.md wrapper from fresh empty
directories, signalled by PID; this crate the same without the `ulimit` (fix round 1, concern 2).
"Identical" means rc, stdout and stderr, with the path masked.

### F1 residual: a long native method

`begin_invoke` serves a posted halt after a `NativeBody::Run` method returns, as `builtin::run`
does. Witness `sigint_halts_after_a_long_method` (the re-review's `'ab'~copies` loop): oracle
10/10 and this crate identical. Mutant: the serve deleted, red on that test. Cost, callgrind
`-r 1` base `159231bb8` against `2e50fc410`: rexxcps -0.0000%, strings +0.0000%, emptyloop
+0.0000%, sendloop -0.035%, dispatch -0.024%, dispatchclass -0.025%. The send programs got
cheaper, which is layout, not this change; no measurable cost.

### N1: readying is idempotent

`Activities::make_ready` pushes an activity only where it is in neither the ready queue nor
`set_aside`. `unpark`, `wake_due_sleepers` and `wake_for_halt`'s sleep branch use it. Every path
the re-review asked about readies through it: notify plus halt, timer plus notify, deadline plus
halt. Witnesses, each the re-review's shape with a 2 s busy activity, signalled at 1 s:
`a_halt_leaves_a_deadline_woken_wait_ready_once_for_a_sleep` (`SysSleep 1` after the halt;
`slept 1`) and `..._for_a_wait` (an untimed `t~wait` that only the busy activity's post ends;
`waited 1 1`). Oracle 10/10 each, this crate identical (5/5 by the driver, and in the suite).
Mutant: the check removed, red on both.

The busy loops are `do forever; if time('E') >= 2 then leave; end`, not `do while time('E') < 2;
end`: on this crate the second never ends, because `TIME('E')` in a `DO WHILE` with an empty
body does not advance (oracle: ends at 1 s). Pre-existing: the same at `159231bb8`. Concern 3.

### N2: the stdin read blocks

`fill_stdin` keeps the posts it sets aside in one deque for the whole read and requeues them when
it returns, so each idle waits for a new post. Witness `a_stdin_read_waits_without_spinning`:
a started activity's `address system 'sleep 1'` ends while main waits in `PARSE PULL`; the test
reads `/proc/<pid>/stat` utime+stime over one second of that wait and asserts under 10 ticks.
Mutant: requeue on every iteration (the old behaviour), red on that test.

### N3: a halt inside its own CALL ON handler is dropped

`raise_requested_halt` returns without raising where the `HALT` trap (or `ANY`) it finds is
delayed. The oracle's `trap` queues it on the handler's activation, whose `processTraps` keeps
re-queueing a delayed handler until that activation ends. Witnesses, each oracle 30/30 and this
crate identical: `a_second_signal_in_a_call_on_halt_handler_is_dropped` (busy handler, SIGINT at
0.3 s and 0.7 s later; `h in 4` / `h out`), `a_second_signal_ends_a_handlers_sleep` (`h out 4`,
the handler's `SysSleep` answering EINTR), and the scheduler test
`a_message_halt_inside_its_handler_is_dropped` (two `Message~halt`s, phases kept in a
`.directory`, no timing). Mutant: the early return disabled, red on all three.

### N4: a signal pending at a command's end is served first

`timer::serve_signal` does what the timer thread does for a signal (one `take_signal`, a halt
posted to every live interpreter), on the holder's thread. `Interp::serve_signal_now` calls it and
then serves the posted halt. It runs at the `Unblocked` arm before the end is filed, and after an
on-baton command in place of `serve_posted_halt`. Witness
`signal::tests::a_signal_pending_at_a_commands_end_halts_it`: in a process of its own (the test
re-executes its binary, since the halt reaches every live interpreter), a thread sets `PENDING`
with no wake byte at 0.2 s while `address system 'sleep 0.5'` runs, and again for the same
command redirected; each must halt at line 2. Mutants: the `Unblocked` call removed, red; the
command-clause call back to `serve_posted_halt`, red. Both stay green on `signals.rs`, which
cannot order the race.

Flake check without P48's rerun: the signals binary at `2e50fc410`, 20 runs with
`--nocapture`, 33 passed each, 0 "run again" lines in all 20 (load average 1.6-3.3). The gated
workspace run had 0 too.

### N5, N6, N7

- **N5.** `an_abandoned_command_ends_no_shorter_later_one`, the re-review's program (`sleep 3;
  exit 7` abandoned, then `sleep 0.3`): `second 0 1`. Oracle 3/3 identical. Mutant: `next_block
  += 1` deleted, red.
- **N6.** `a_halted_read_loses_no_input`: the re-review's stdin program, `one\ntwo\nthree\nfour\n`
  written 0.5 s after the signal. This crate: `[]`, `one`, `two`, `thr`, `1 1`, `ee`, `four`.
  The oracle, 5/5, differs only by losing `one` (DEVIATIONS entry 10). Mutant: `receive` drops a
  chunk that arrives while the input is marked interrupted, red.
- **N7.** `Input::reading_stdin`, called once `fill_stdin` returns ready, clears the zeroed counts
  before the read: after the next read `LINES() CHARS()` answer `1 1`, the oracle's 5/5, where
  they answered `0 0`. Same witness. Mutant: the call deleted, red.

### Mutants

`p6-scratch/t21f2/mut.py`: per mutant, copy the file, assert the site occurs once, write the
mutant, run, copy back and `cmp`. Commands: `cargo test --release -p rexx-exec --test signals
--no-fail-fast`, and for N3 and N4 also `cargo test --release -p rexx-exec --lib --no-fail-fast
-- signal::tests a_message_halt_inside`. Every run counted 33 signals tests (3 or 4 lib tests).

| mutant | red |
| --- | --- |
| N1 no dedupe in `make_ready` | both `a_halt_leaves_a_deadline_woken_wait_ready_once_*` |
| F1 no serve after a native method | `sigint_halts_after_a_long_method` |
| N2 requeue each iteration | `a_stdin_read_waits_without_spinning` |
| N3 delayed trap not checked | both `a_second_signal_*`; `a_message_halt_inside_its_handler_is_dropped` |
| N4a no serve at `Unblocked` | `a_signal_pending_at_a_commands_end_halts_it` (signals.rs green) |
| N4b `serve_posted_halt` after an on-baton command | `a_signal_pending_at_a_commands_end_halts_it` (signals.rs green) |
| N5 constant block token | `an_abandoned_command_ends_no_shorter_later_one` |
| N6 chunk dropped after a halted read | `a_halted_read_loses_no_input` |
| N7 counts not reset | `a_halted_read_loses_no_input` |

`git status` afterwards showed only the lead's `progress.md`.

### Checks (P51), at `2e50fc410`

Target dirs `p6-scratch/t21f2/target` and `.../target-loom`; each status captured unpiped.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; `--features pinning`: exit 0;
  `RUSTFLAGS="--cfg loom"`: exit 0. Each log has one `Checking rexx-exec` line. The loom clippy
  at `f9fb61990`'s first draft failed on `serve_signal` unused under loom; it is gated like
  `install_signals`.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, 143 result lines, 3023 passed, 0 failed, 4 ignored.
- Loom: `RUSTFLAGS="--cfg loom" memcap 8G cargo test -p rexx-exec --test loom`: exit 0, 15
  passed, 359.67 s test time, 392 s wall including the build.

### Concerns

1. **The N4 witness is in-process, not a signal.** It sets `PENDING` directly so the timer cannot
   win; the real race is only covered statistically (20/20 above). It re-executes the lib test
   binary with `--exact`; a renamed test breaks the inner run loudly (`1 passed` asserted).
2. **Posts set aside during a stdin read wait until it ends** (N2 keeps that). A started
   activity's `say` after its command runs after main's read on this crate; on the oracle it
   prints first (`w 0` before `v=[hi]`, 3/3). Pre-existing: the reading holder keeps the baton.
3. **`TIME('E')` does not advance in `do while time('E') < N; end`** on this crate, so the loop
   never ends; the oracle's ends. Pre-existing at `159231bb8`; not filed, not in Task 21.
4. **N7 leaves the pre-existing `LINES()` divergence**: once `LINES()` has been asked, a later ask
   answers 0 here (`lines_asked`) where the oracle answers 1 after a read. The witness asks only
   after the reads.
