# Task 8 review: the simulation mode (0765d19ef..1cc240446)

Probes ran on `git archive` copies of `1cc240446` and `0765d19ef` (with `interpreter/`, `docs/`) in
`/tmp/claude-1000/p61/t8rev/{head,base}`, each built in its own target directory (one `Compiling
rexx-exec` line each); rexx-run probes under `memcap 2G` + `timeout`, test binaries under `memcap 8G`
+ `timeout`. Probe texts in `/tmp/claude-1000/p61/t8rev/p1/` (determinism, timing) and `p3/` (deadlocks).

### Spec Compliance

- ❌ Issues found: spec section 4's row "a wait for a post only a native's own thread or a command's
  peer can deliver: refused loudly in sim" is not met. Both shapes hang the process at OS level with no
  refusal, and the run's deadline does not fire either (I1). The report discloses this as Concern 1;
  disclosure does not satisfy the row.
- ✅ Everything else in the brief verified: `SwitchMode::Sim`, `SimConfig` parse/Display round trip,
  splitmix64 + xoshiro256\*\* streams, virtual clock (seeded origin, `clock=midnight`, `clock=real`,
  per-run quantum 29..=116 ns, jump to the next deadline), the clock seam, pool bound 0, no timer slice,
  `RANDOM` and child seeds from streams, `gc=q`, `halt@K`, `fail=wait:K`, the `rexx-sim:` line
  (`trace=` omitted until Task 9 fills `trace_hash`, which the Task 9 plan step 4 assigns there), two
  LIMIT rows.
- ⚠️ Cannot verify from diff: whether the gate harness of Task 10 bounds a sim run with a process-level
  timeout. I1 means the in-process run deadline cannot end these hangs. The controller should carry
  that into the Task 10 brief.

### Checks run (by focus)

1. **Determinism.** `p1/conc.rex` (4 started workers with MutexSemaphore `acquire(1)`, random
   `SysSleep`s, `EventSemaphore~wait(2)` posted by an `.Alarm`, `GUARD ON WHEN` on a counter, a `REPLY`
   then sleep, `time('E')`, `time('L')`, `date('S')`, seeded and unseeded `random`): `sim:5`, 10
   processes, 10/10 identical stdout and 10/10 identical stderr including `rexx-sim: seed=5 policy=fifo
   steps=370 switches=39`. Same program `sim:21,gc=0.2` 10/10 identical. `sim:5,halt@150,gc=0.05` 5/5
   identical (rc 252). `sim:5,fail=wait:3` 5/5 identical. `p1/c2.rex` (Set and IdentityTable iteration
   over fresh objects, identityHash, 6 activities ordered by `GUARD ON WHEN turn = i` + `REPLY`, a
   MutexSemaphore `acquire(0.1)` timeout, a `do until time('E') > 0.0005` spin) `sim:11` 10/10 identical.
   `p1/u.rex` (UNINITs under `gc=0.2`, started activities) 10/10 identical. A command child running
   `rexx-run` under `sim:8`, 3/3 identical; each child gets its own drawn seed. `sim` alone printed
   `seed=4012482063170174797`; rerunning `sim:4012482063170174797` reproduced its output exactly.
   Seeds differ where the program depends on them: the worker queue order in `conc.rex` differs across
   seeds 1, 2, 5, 6, 7, 99 (six distinct orders); unseeded `random()` in main and in a started activity
   differs across seeds 1, 2, 3 and is unchanged by `gc=0.3` and `clock=real` (stream independence);
   `c2.rex`'s spin count differs by seed (2033, 1852, 2197: the quantum). Two default-mode runs of the
   same program give different `random()`. No nondeterminism found.
2. **Clock seam.** The brief's grep at HEAD prints exactly the sites in the report's allow-list table
   (`sync.rs:197,220,239-261`, `sim.rs:208,323,332,379,390`, `signal.rs:238,270`,
   `scheduler/pool.rs:43`, `clause.rs:37,309,336,371`, plus the four test modules). A wider search
   (`sync::now()`, `clock_gettime`, `Local::now`, `Utc::now`, `UNIX_EPOCH`, `RandomState`,
   `process::id`) found only file-timestamp conversions and the timer thread's own `now()`
   (`timer.rs:349,451`), which sim does not arm. `.DateTime~new`, `.DateTime~today`, `time('T')`,
   `date('E')`, `.DateTime~new~ticks` all read virtual time and repeat per seed. Mutation: removing the
   `#[allow]` on `Deadline::starting_now` makes `cargo clippy -p rexx-exec --lib -- -D warnings` fail
   with `disallowed method std::time::Instant::now` at `clause.rs:36`, so the lint is live.
   Timing, `p1/t5.rex` under `sim:1`: `SysSleep 5`, an `.Alarm` of 5 s posting an EventSemaphore
   awaited with `wait(10)`, `MutexSemaphore~acquire(5)` timing out, `EventSemaphore~wait(5)` timing
   out, `SysWaitEventSem(h, 5000)` timing out, `SysSleep 3600`: wall 0.07 s; `time('E')` reads
   3625.010012 of virtual time. `SysWaitEventSem` with a timeout and a post from a started activity gives
   `121 1 1 0 1` in default mode and in sim.
3. **Concern 1, OS-level deadlock.** Reproduced, results in I1 and P1.
4. **Default mode.** Callgrind spot check, my own builds, `callgrind.sh -r 1 -j 2 -p "startup
   emptyloop" base=... head=...`: startup 58249906 to 58249981 (+0.0001%), emptyloop 7761204250 to
   7761252748 (+0.0006%, the same delta the gate record quotes). Corpus not re-run (the report's gated
   run is the evidence).
5. **Refusals.** `sim_endless_wait` reached from a plain program (`p1/w3.rex`: a started method in
   `GUARD ON WHEN x = 1` that nothing sets, rc 120, the message, the `rexx-sim:` line). A main-only
   `EventSemaphore~wait`, a started activity's endless wait, and an untimed `SysWaitEventSem` answer the
   existing `unsatisfiable_wait` in sim. `sim_foreign_post` is reached only by the inbox-injection test;
   no native in `rexx_api::load` calls back from a detached thread after returning (both thread-spawning
   natives, `load.rs:770` and `:819`, join first), so no program reaches it today. The LIMIT row says
   what the design refuses and stays true. `cargo test -p rexx-exec --test refusal_sites --test
   refusal_dispositions`: 5 passed and 3 passed. The 15 sim and sim-native lib tests pass.
6. **clippy.toml.** Each `#[allow(clippy::disallowed_methods)]` outside `src` sits on a file with at
   least one `Instant::now`/`SystemTime::now` (21 test harness files, `rexx-api/src/load.rs`
   `await_buffer_changed`, `rexx-bench/src/timing.rs`); each in `src` matches the report's table. None
   allows a read the seam should own.

### Strengths

- The default path really is unchanged: every sim hook sits inside an existing taken branch
  (`collect_if_due`'s due arm, the switch-mode arm, `idle_until`/`idle_for_posts`/`idle_for_good`
  behind `self.sim.is_some()`), and callgrind agrees to four decimals.
- The stream split is sound: consecutive splitmix64 outputs per stream, and `Clock::new` draws the
  quantum and every origin variant in a fixed order whatever `clock=` is, so knobs do not re-roll
  other draws. I confirmed this by running, not only by reading.
- Determinism held on every program I wrote, including GC-driven UNINIT order under `gc=q`, guard waits,
  REPLY, and child processes.
- The endless-wait fix (`909d87b09`, `run_started_to_end` stopping on `sim_is_stuck`) came from the
  implementer's own test hitting an OOM, and the test pins it.
- `rexx-run` with bare `sim` prints the seed at start, so a run that never ends can still be replayed.
  That matters given I1.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

**I1. A sim wait that only a native's own thread or a command's peer can end hangs the process and is
not refused (spec section 4 row; brief Step 4).** Both shapes reproduced:

- Command peer, `p3/fifo.rex` (a fifo made with `mkfifo`):
  ```
  w = .w~new~start('WRITE')
  address system 'cat /tmp/claude-1000/p61/t8rev/p3/ff'
  say 'read done' w~result
  ::class w
  ::method write
    call SysSleep 0.1
    address system 'echo hi > /tmp/claude-1000/p61/t8rev/p3/ff'
    return 'wrote'
  ```
  Default mode: `hi`, `read done wrote`, rc 0, 0.17 s. `REXX_SWITCH_MODE=sim:1`: no output, killed by
  `timeout 10` (rc 124). With bound 0, `exit_for_block` (`scheduler.rs:659-664`) answers `Err(block)`
  and `command.rs:719` `block.wait()`s on the baton thread, so the writer activity never runs.
- Native's own thread: a scratch lib test (in my copy only, not the tree) over `run_shaped` with
  `Shape.switch = Some(Sim(sim:1))` running `SENDFROMANOTHERTHREAD(.loud~new, 'SPEAK')` under a 3 s
  run deadline hit the outer `timeout 20` (rc 124). The same program in default mode with a free pool
  answers `trapped 98.983`, `napped`, rc 0, 1.5 s.

In both cases the baton thread is blocked outside the clause loop, so the run's deadline
(`countdown_reached`) and `sim_foreign_post` (`sim.rs:513`, only screened when the inbox is drained)
never get a chance to fire. The spec puts these waits under loud refusal. The Task 10 gate fails on
"a hang past the run deadline", which these runs can only show through an external kill. Possible
fixes, for the controller or Moritz to choose:
(a) the foreign-thread callback path refuses loudly at once in sim when the baton is held by a thread
inside an inline native call, rather than queueing a `Recall` and waiting;
(b) a real-time watchdog for an inline command or native in sim that ends the run with a loud refusal;
(c) a ruling that narrows the spec row to a LIMIT the gate bounds externally. Then the row and the
report should say "hangs; bounded only by a process timeout" rather than "refused".

#### Pre-existing defect (default mode; not this task's regression)

**P1. Default mode hangs when the pool has no free thread and an inline command or native waits on
another activity.** `p3/full.rex`:
```
do i = 1 to 64
  s.i = .s~new~start('NAP')
end
call SysSleep 0.5
w = .w~new~start('WRITE')
address system 'cat <fifo>'
say 'read done' w~result
::class s
::method nap
  address system 'sleep 3'
::class w
::method write
  call SysSleep 0.1
  address system 'echo hi > <fifo>'
  return 'wrote'
```
The 64 `sleep 3` commands fill the pool (`POOL_BOUND` 64), so main's `cat` runs inline on the baton
thread and the writer never runs. Head: killed by `timeout 15` (rc 124; SIGTERM then halted 65 commands).
Base `0765d19ef`: identical (rc 124, 65 halts), so this predates Task 8. The native form reproduces too:
a scratch lib test with `Shape { bound: Some(1) }`, one activity holding the pool thread in `NAPLONG`
and main calling `SENDFROMANOTHERTHREAD`, hit `timeout 20` on both head and base. The oracle could not
be run at this size under `ulimit -v` 1 GB or 4 GB: `48.1 ERROR CREATING THREAD` at the 64th `start`.
The oracle has no pool (one thread per activity), so the hang is this crate's. Not a Task 8 finding,
but the brief asks that it be named: it belongs in the ledger as a defect with this program. A fix for
I1(a) may also cover the native half.

Probe hygiene for whoever reruns P1 or I1: `timeout ... memcap ... rexx-run` kills `memcap` and leaves
`rexx-run` and the blocked `cat`/`sh` alive. Put `timeout` inside `memcap`, and open the fifo from the
other side to release the leftovers.

#### Minor (Nice to Have)

- `sim.rs:265-268`: the `schedule` ("Preemption decisions") and `order` ("The pick among the
  activities one event readied") field docs describe mechanisms that do not exist until Task 9. That
  breaks the prose rule (decisions, not futures). The `#[expect(dead_code)]` reason already says
  what is true. Drop the two field docs or say "split, not drawn".
- `rust/clippy.toml:1-3`: the comment scopes the rule to "rexx-exec", but the file is workspace-wide
  and `rexx-api`/`rexx-bench` carry allows under it. Also `Instant::elapsed` and `std::thread::sleep`,
  both in the spec's grep, are not disallowed, so a future `.elapsed()` on the baton thread would
  bypass the seam silently. Consider adding `std::time::Instant::elapsed` and `std::thread::sleep` to
  `disallowed-methods`, with allows at the listed sites.
- The `#[allow(clippy::disallowed_methods)]` sites could be `#[expect]`, so that a site that stops
  reading the clock fails clippy rather than leaving a stale allow-list entry.
- `scheduler.rs:1287`: after a `Recall`, `file_completions` re-drains the inbox without
  `sim_screen_posts`. Harmless today because the `Recall` itself already set `breach`, but the screen
  is not where every drain passes.
- `one_seed_gives_one_output` (`sim/tests.rs`) compares two runs in one process. The cross-process
  property the report quotes from `rexx-run` is not pinned by a test. Task 9 Step 1 plans a two-process
  hash test, so this is a note, not a gap.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The mode is deterministic on every concurrent program I ran, the clock seam is complete
and lint-enforced, and default-mode cost is nil. However, the spec's loud-refusal row for waits that only
a native's own thread or a command's peer can end is not implemented: both shapes hang the process
past the run deadline (I1), which needs a fix or an explicit ruling before the Task 10 gate depends on
it. P1 is a pre-existing default-mode hang to record in the ledger.

## Fix round 1

Verified at `3ff074ab9` (code `95df78c16`): a `git archive` copy in `/tmp/claude-1000/p61/t8rev/fr1`,
its own target directory, `timeout` inside `memcap`.

**Verdict: I1(a) and I1(b) fixed; the four Minors fixed. One Important stays open: the stated gap is
reachable with a shipped library (I2). Two new Minors. Task quality: Needs fixes, or a ruling on I2.**

### Checks

- **I1(a).** My scratch probe (one activity in `NAPLONG`, then main calls `SENDFROMANOTHERTHREAD`)
  under `sim:1`, re-applied to the fix tree: rc 120, stdout `1`, stderr `rexx-exec: a callback from
  another thread in the simulation mode is not implemented`, in 1.52 s on each of 3 runs. The 1.52 s
  is the inline `NAPLONG` (1.5 s of real sleep), not a wait. In default mode with a free pool it still
  answers `trapped 98.983`, `napped`, rc 0. The committed witness passes too.
- **I1(b).** `p3/fifo.rex`:
  - default mode: `hi`, `read done wrote`, rc 0, 0.15 s (unchanged);
  - `sim:1`: rc 120 with `a command in the simulation mode that waits longer than its bound`, 2.06 s;
  - `sim:1,block=0.3`: rc 120, 0.36 s.
  `pgrep` found no `cat`, `sh` or `rexx-run` left after any run.
- **Knob parsing.** `block=-1`, `block=x` and `block=inf` exit 2 with `block is a number of seconds
  above 0`, and `block=0.5` round-trips (crate test). Exception: `block=1e30` (N2).
- **LIMIT rows.** `sim_blocked_command` is reachable as stated. `sim_foreign_post`'s widened text
  ("and a callback from a thread running none of the interpreter's native calls") is now reachable
  from a program through a test native. `refusal_sites` 5 passed, `refusal_dispositions` 3 passed,
  `sim_processes` 1 passed, and the `sim::`, `simulation_mode` and `callbacks::` lib tests 32 passed.
- **Determinism set**, each run 10 times with stdout, stderr and rc folded into one file. Every case
  gave 10/10 identical files:
  - `conc.rex sim:5`
  - `conc.rex sim:21,gc=0.2`
  - `c2.rex sim:11`
  - `u.rex sim:21,gc=0.2`
  - `conc.rex sim:5,halt@150,gc=0.05` (rc 252)
  - `ch.rex sim:8` (child `rexx-run` processes)
  - `cmd.rex sim:1` (commands with and without `WITH` redirection, and stdin; these now wait on the
    helper thread)

  `conc.rex sim:5` is byte-identical to the round-0 output, so the watchdog never fired on a program
  that finishes.
- **Default mode.** `collect_on_baton` falls through to `collect` when `sim_block_bound` is `None`.
  `refuses_foreign` answers `false` unless `SIM` is set, which only `start_sim` does. The gate record
  quotes callgrind against `0765d19ef` with the command (rexxcps +0.0001%, emptyloop +0.0006%,
  startup +0.0000%). I did not re-measure.
- **Minors from round 0.**
  - The `schedule` and `order` stream docs are gone.
  - `clippy.toml` says workspace-wide and adds `Instant::elapsed`, `SystemTime::elapsed` and
    `thread::sleep`.
  - Every exemption is `#[expect]`.
  - The recall re-drain is screened (`scheduler.rs:1288`).
  - `tests/sim_processes.rs` compares two processes. All fixed.

### Issues

#### Important

**I2. A native blocked inline on another activity, other than through a callback, still hangs sim with
no refusal, and a shipped library reaches it.** The ruling asked for a watchdog for blocking inline
commands and natives; the fix covers commands only, and the report names the native half as a gap.
`p4/sock2.rex` uses `rxsock` from `build/lib`, declared as `::routine ... external "LIBRARY rxsock
..."`. Main binds and listens, starts a server activity calling `SockAccept`, sleeps 0.1 s, then
connects:

- default mode: `bind 0 listen 0`, `connect 0`, `got 1`, rc 0, 0.19 s;
- `sim:1`: `SockAccept` runs inline on the baton, and main never runs to connect. Killed by
  `timeout 10`, rc 124.

The run's deadline cannot fire here, for the same reason as in I1. The way out is in that run's own
output: the SIGTERM made `SockAccept` return (EINTR), and the halt was then served normally. So a
sim-only watchdog that interrupts the baton thread after `block=` (a `pthread_kill` with the
handler's signal), plus a `sim_blocked_native` refusal, would end any native blocked in a system call.
It would not end a native that retries on EINTR or spins; the LIMIT row can say that. Alternatively,
rule the gap a LIMIT. Then the row should name the shape (a native waiting on another activity, e.g.
an `rxsock` server and client in one program), and Task 10 must bound such runs externally.

#### Minor

- **N1.** `block=` refuses any command slower than the bound, not only one waiting on a peer.
  `address system 'sleep 2.5'` is rc 0 in default mode and rc 120 under `sim:1` (rc 0 under
  `block=5`). The LIMIT row says "timing-dependent", but it describes the refusal as one for "a wait
  only a command's peer can end". It should also say that a command running longer than `block=`
  is refused. Task 10's calibration should set `block=` from the slowest command its programs run.
- **N2.** `block=1e30` parses and then panics at the first command (`cannot convert float seconds to
  Duration`, rc 101, from `Duration::from_secs_f64` in `sim_block_bound`). Validate it in
  `SimConfig::parse` with `Duration::try_from_secs_f64`.

## Fix round 2

Verified at `c0e1865ee` (code `27cbe2510`, `da89562a3`): a `git archive` copy in
`/tmp/claude-1000/p61/t8rev/fr2`, its own target directory, `timeout` inside `memcap`.

**Verdict: I2, N1 and N2 are fixed for every `block=` value above 0. Fix round 2 adds one new
Important (I3: `block=0`, which this round started accepting, hangs the run) and two Minors (N3, N4).
The rxsock test is not flaky. Task quality: Needs fixes (I3 is a one-line parse change).**

### Checks

- **I2, `p4/sock2.rex`** (a fresh port for each run):
  - default mode: `bind 0 listen 0`, `connect 0`, `got 1`, rc 0, 0.16 s;
  - `sim:1`: rc 120, `a native call in the simulation mode that runs longer than its bound`, 2.06 s;
  - `sim:1,block=0.5`: the same refusal in 0.56 s.
  No `rexx-run` was left afterwards, and `ss` showed only a TIME-WAIT from the default run's connect.
- **N1.** `address system 'sleep 2.5'` gives rc 120 under `sim:1` and rc 0 under `block=5`. Both
  LIMIT rows now say that any slower command or call is refused. True.
- **N2.** `block=-1`, `block=x`, `block=inf`, `block=NaN` and `block=1e30` each exit 2 with `block is
  a number of seconds from 0 to 86400`. `block=86400` runs (rc 0).
- **Fifo.** `p3/fifo.rex` in default mode: 0.15 s, rc 0. Under `sim:1`: rc 120 in 2.06 s, no
  leftovers.
- **SENDFROMANOTHERTHREAD.** My scratch probe under `sim:1` gives rc 120 with the callback refusal
  on 3 of 3 runs. It still answers `trapped 98.983`, `napped` in default mode with a free pool.
- **Determinism set**, each run 10 times: 10/10 identical every time (`conc.rex sim:5`, `sim:21,gc=0.2`,
  `sim:5,halt@150,gc=0.05`; `c2.rex sim:11`; `u.rex sim:21,gc=0.2`; `ch.rex sim:8`; `cmd.rex sim:1`).
  The hashes are equal to fix round 1's, so the watch fires on none of these programs.
  `rxmath` called 50 times in a loop: 100 runs at default `block`, and 150 at `block=0.001`, all rc 0.
- **The rxsock crate test** (`a_native_call_only_another_activity_can_end_is_refused_after_its_bound`):
  10 runs in a row, 10/10 passed; 10 rounds of two processes at once, 20/20 passed; 3 runs of the
  `sim::`, `simulation_mode`, `scheduler::tests::native::` and `callbacks::` tests together (42
  tests) on the default thread count, all passed. No `bind -1` and no flake.
- **The signal handler.** `interrupted` is an empty `extern "C" fn`, so it is async-signal-safe. It is
  installed with `sa_flags = 0` (no `SA_RESTART`, so a blocked system call answers EINTR) and a full
  mask. `INTERRUPT_INSTALL` runs only from `Interrupter::here`, which only `sim_watch_native` calls,
  and only where the action is still `SIG_DFL`, so it is installed in sim only. The watcher blocks
  signals itself and is joined before the call returns, so the `pthread_t` it signals is live, as
  the `Send` comment says.
- **Default mode.** `sim_watch_native` is `self.sim_block_bound()?`, which returns `None` at
  once, on `exit_for_native`'s inline branch. Default mode takes that branch only when the pool has
  no thread to give. The gate record quotes callgrind against `0765d19ef` with sha256s: rexxcps
  +0.0001%, emptyloop +0.0006%, startup +0.0009%. Cheap.
- `refusal_sites` 5 passed, `refusal_dispositions` 3 passed.

### Issues

#### Important

**I3. `block=0` livelocks the interpreter.** This round widened the accepted range from "above 0" to
"from 0", and `a_config_reads_back_what_it_prints` now asserts that `sim:7,block=0` round-trips. With
a zero bound, `recv_timeout(Duration::ZERO)` in `sim_watch_native` (`sim.rs`) times out at once on
every pass, so the watcher sends `pthread_kill` in a tight loop. The signal is a real-time one, so
each send is queued, not merged.

Probe `p4/m50.rex` calls `RxCalcSqrt` 50 times in a loop. Under `sim:1,block=0`, through a Python
runner (`/tmp/claude-1000/p61/t8rev/runmany.py`):
- one batch: 8 rc 0, 6 rc 120 (refused after 32 to 100 steps, varying), 1 hang at run 14;
- another batch: 16 rc 0, 1 hang at run 16.

Under `memcap ... timeout -k 2 20`, 3 of 60 runs ended rc 137: the hang, with SIGTERM caught as a
halt that never runs, then KILL. A `gdb` backtrace of a hang (saved to
`/tmp/claude-1000/p61/t8rev/hang-bt.txt`) shows:
- the interpreter thread inside `pthread_create`, called from `sim_watch_native` (`sim.rs:593`,
  from `scheduler.rs:649`);
- the new watcher thread, already running, inside `Interrupter::interrupt` at `signal.rs:235`;
- main joining.

The watcher starts flooding before its creator's `pthread_create` has returned, and the creator
makes no further progress. Even when it does not hang, `block=0` refuses or not depending on whether
the watcher is scheduled before a microsecond call returns.

Fix: reject `block=0` again (`(0.0..=BLOCK_LIMIT)` becomes a range that excludes 0) and move
`sim:7,block=0` back to the wrong list. A floor such as 1 ms also works; 150 runs at
`block=0.001` were clean.

#### Minor

- **N3.** The `sim_blocked_native` LIMIT row and the `Knobs::block` doc both say the bound covers "a
  native call made while another activity lives". That is false. `leaves_driver`
  (`dispatch/library.rs:318-322`) is true whenever `switch` is set, and sim sets it, so a lone
  activity's call is watched too. `p4/sel.rex` (a `SockSelect` with a 3 s timeout on a listening
  socket, no other activity) gives `switches=0` and is refused at 2.05 s under `sim:1`; default mode
  answers `select 0` in 3.07 s. Delete the clause. A false sentence is deleted, not reworded.
- **N4.** A native call that does not leave its driver is not watched, and the row does not say so.
  Those are calls under a pin (`pin_depth > 0`) or while resuming. `p4/pin.rex`: the server
  activity's `SockAccept` runs inside a `sortWith` comparator. It hangs in sim until `timeout 10`
  (rc 124), with no refusal. It also hangs in default mode, on HEAD and on base `0765d19ef` (a pinned
  call runs on the interpreter's thread in both modes). The oracle hangs too: 5 of 5 runs printed
  `connect 0` and never `got`, until `timeout 20`. So this is no divergence, only an unbounded
  case. The row's list of what "stays unbounded" should add "a native call under a pin".

## Fix round 3

Verified at `007adb014` (code `1f7b73ac0`): a `git archive` copy in `/tmp/claude-1000/p61/t8rev/fr3`,
its own target directory, `timeout` inside `memcap`.

**Verdict: I3, N3 and N4 are fixed. No new findings. Task quality: Approved.**

- **Parse floor.** `block=0`, `block=0.0009`, `block=-1`, `block=x`, `block=NaN` and `block=1e30` each
  exit 2 with `block is a number of seconds from 0.001 to 86400`. `block=0.001` runs.
- **The livelock probe** at `block=0.001`, 60 runs each through the Python runner, all rc 0, with no
  hang and no refusal:
  - `p4/m50.rex` (`RxCalcSqrt` 50 times, one activity);
  - `p4/m50b.rex` (the same with a started activity alive).
- **At most one signal per watched call.** The watcher now waits on `recv_timeout(bound)` once, calls
  `interrupt` once on timeout, then blocks on `stopped.recv()` until the call ends (`sim.rs`,
  `sim_watch_native`). `strace -f -e trace=tgkill,tkill,rt_tgsigqueueinfo` on `p4/sel2.rex` (a 3 s
  `SockSelect` on a listening socket) under `sim:1,block=0.2` and under `block=0.05` shows exactly
  one `tgkill(..., SIGRT_5)` each. `SIGRT_5` is strace's name for libc's `SIGRTMIN()+3`, signal 37.
  The run is refused with rc 120 after the call returns `-1` (EINTR).
- **N3.** The row now reads "a native call that leaves its driver", and the `Knobs::block` doc is
  "a command or a native call". Both true: a lone activity's call leaves its driver in sim
  (`leaves_driver`, `switch` set), and `sel2.rex` (no other activity) is refused.
- **N4.** The row names "a native call under a pin, which does not leave its driver (it hangs on the
  oracle too)", which matches my `p4/pin.rex` runs in fix round 2.
- **Unchanged behaviour.**
  - `p4/sock2.rex`: default `bind 0 listen 0`, `connect 0`, `got 1`, rc 0, 0.16 s; under `sim:1`
    rc 120 with the native refusal in 2.05 s.
  - `p3/fifo.rex`: default `hi`, `read done wrote`, rc 0, 0.15 s; under `sim:1` rc 120 with the
    command refusal in 2.06 s.
  - No `rexx-run` or fifo child was left afterwards.
- **Tests.** The `sim::`, `simulation_mode`, `scheduler::tests::native::` and `callbacks::` lib
  tests (with the new `quick_native_calls_at_the_smallest_bound_run_alike`) 115 passed under the
  filter, `refusal_sites` 5 passed, `refusal_dispositions` 3 passed.
