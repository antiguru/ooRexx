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
