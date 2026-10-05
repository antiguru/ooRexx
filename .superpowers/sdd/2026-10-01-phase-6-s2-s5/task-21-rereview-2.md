# Task 21 re-review, fix round 2

Reviewer: s4-t21-rr2 (opus). Range `159231bb8..bf76afa1d`, HEAD `bf76afa1d`.

How the probes were run:
- Driver: `$CLAUDE_JOB_DIR/tmp/drv.py`. Each run gets a fresh empty directory and
  `env --default-signal=INT,TERM,HUP`. A PID signal goes to the interpreter process itself; a
  group signal goes to the interpreter's process group (`killpg(getpgid(interpreter))`).
- Oracle: the CLAUDE.md wrapper (`ulimit -v 1048576`, `timeout -k 5 20`). This crate:
  `timeout -k 5 20` with no `ulimit` (fix round 1, concern 2).
- "HEAD" is `rexx-run` built at `bf76afa1d`, copied aside before any mutation. "Base" is
  `rexx-run` at `159231bb8`, built from `git archive` in a separate target dir (with a
  `Compiling rexx-exec` line).
- "Identical" means rc, stdout and stderr, with the path masked.
- "Under load" means 48 busy `sh` loops on 32 cores (`$CLAUDE_JOB_DIR/tmp/load.sh 48 200`), or
  the memcapped workspace suite running alongside. The load average is quoted each time.

## Verdict

**CHANGES REQUIRED.**

Counts:
- Items: 6 resolved (F1 residual, N1, N2, N5, N6, N7), 2 partly resolved (N3, N4).
- New findings: High 1 (pre-existing, outside Task 21), Medium 1 (concern 2, needs a ruling),
  Low 2.

What blocks approval:
- **N4 is not closed.** A terminal Ctrl-C during a command still halts a clause late:
  - 2 of 60 driver runs under load. The oracle gave 0 of 30 at load average 50.
  - 2 of the 20 signals-binary runs needed P48's rerun.

  So point 5 is not met. The fix's premise, that the handler has run before any thread
  returns from the kernel, is false for this process (R-N4 below).
- **N3 still diverges when the trap is `CALL ON ANY`**, 10 of 10 runs (R1). The cause is a
  pre-existing defect in which trap gets delayed. Its general form is R3: a `CALL ON ANY` handler
  that raises `ERROR` recurses until it is killed.

## Items F1 residual, N1-N7

Each line below is HEAD against the oracle.

| | Status | Evidence at HEAD |
| --- | --- | --- |
| F1 (method) | **Resolved** | rr1's `'ab'~copies(30000000)` loop with SIGINT at 1 s: identical 10/10, rc 252 and 4.1 at line 2, 1.02-1.07 s (oracle 1.03 s). |
| N1 | **Resolved** | See below. |
| N2 | **Resolved** | rr1's program. A started activity runs `address system 'sleep 1'`, main does `parse pull`, and stdin is written at 4 s. CPU from `wait4` rusage on `rexx-run` itself: HEAD user 0.02 s, sys 0.01 s; base user 1.97 s, sys 1.04 s. Both elapsed 4.01 s. |
| N3 | **Partly resolved** | See below. |
| N4 | **Partly resolved** | The in-process witness is sound (point 4), but the signal race it targets is still open (R-N4, point 5). |
| N5 | **Resolved** | rr1's program (`sleep 3; exit 7` abandoned, then `sleep 0.3`): stdout identical 5/5, `a` / `first` / `second 0 1`. Elapsed is 0.81 s here and 3.01 s on the oracle. The oracle's abandoned child holds the inherited stdout open, which DEVIATIONS entry 10 records. |
| N6, N7 | **Resolved** | rr1's stdin program, with SIGINT at 0.5 s and `one\ntwo\nthree\nfour\n` written at 1.0 s. This crate, 5/5: `[]` `one` `two` `thr` `1 1` `ee` `four`. Oracle, 5/5: `[]` `` `two` `thr` `1 1` `ee` `four`. The only difference is the oracle's lost `one`, which entry 10 records. The counts now agree (`1 1`). |

N1 in detail:
- rr1's program with `call SysSleep 2` after the halt, and a 3 s busy activity under `CALL ON
  HALT`: identical 10/10, `slept 1`.
- The `r2 = t~wait` form no longer answers `r2 1`. At 3 s it ends in the loud refusal
  `a wait that nothing left to run can end is not implemented` (rc 120), 3/3. The oracle hangs
  until it is killed, 3/3. Nothing can ever post `t`, so this is the existing deadlock refusal,
  not a halt effect.
- Point 2 covers lost wakeups.

N3 in detail. Each probe is rr1's shape, SIGINT by PID twice (0.5 s, then 1.2 s), 10/10
identical:
- a busy `CALL ON HALT` handler: `h in` / `h out` / `main after`;
- a handler doing `r = SysSleep(2)`: `h out 4`;
- a third signal after the handler has returned (0.5, 1.0, 2.2 s), where the handler runs again:
  `h in 1` / `h out 1` / `h in 2` / `h out 2` / `main after 2`;
- a handler that re-arms `call on halt name h2` inside itself: `h in` / `h2` / `h out`.

**`call on any name h`** in place of `call on halt` diverges, 10/10:
- this crate: `h in` / `h in` / `h out` / `h out` / `main after`;
- oracle: `h in` / `h out` / `main after`.

Base `159231bb8` gives the same as this crate (3/3). See R1.

## Point 2: N1's fix and lost wakeups

**Verdict: no lost wakeup found.**

By inspection, every wake that `make_ready` can merge is retested when the activity resumes:
- semaphore waits through `retest_semaphore`;
- `GUARD WHEN` through its condition;
- a command through `blocked.end`, set before `unpark`;
- a sleep through its `asleep` record.

So a second wake that arrives while an entry is still queued is seen by the run that entry
starts. `yield_at_slice` (`scheduler.rs:588`) still pushes without the check. That is safe,
because the running activity is never in `ready`.

Probes, 30 runs each side, all identical:

| Probe | Shape | Both sides, 30/30 |
| --- | --- | --- |
| `pingpong` | 40 rounds of `a~wait; a~reset` / `b~post` against a worker re-parking in `b~wait(0.001)` and an occasional `SysSleep 0.002`: a notify after the activity has run and parked again, 40 times | `main 40` / `w done` |
| `timernotify` | main `s~wait(0.05)` times out while two activities run (one busy 2 s, one posting and resetting `s` for 0.2 s), then untimed `t~wait` posted at 1.5 s: a timer and a notify on different parks | `r2 1 1 1` |
| `sleepnotify` | `SysSleep 0.1`, timed `MutexSemaphore~request(0.05)` held elsewhere, `SysSleep 0.2`, untimed `s~wait` posted at 1 s, then the mutex | `r1 The NIL object r2 1 r3 The NIL object 1 1` |
| `guardwhen` | a `GUARD ON WHEN n >= k` loop over five `bump`s 0.05 s apart, beside a 1.5 s busy activity, then `m~result` | `result 5 5` / `end 1 1` |
| `haltthenwait` | SIGINT at 1 s ends a `s~wait(0.001)` loop under `CALL ON HALT`; then untimed `t~wait` posted at 2 s; then `SysSleep 0.3` | `h` / `r2 1 1 1` / `slept 1` |

Mutations of the fix. Each was applied by a Python replace that asserts one occurrence, and
reverted by copying the saved file back and running `cmp`. Tests:
`cargo test --release -p rexx-exec --no-fail-fast --lib --test signals`.

| Mutant | Tests | Probes |
| --- | --- | --- |
| M-a: lose wakes (`make_ready` pushes only into an empty queue) | **red**: 19 lib, 3 signals | `timernotify`, `sleepnotify` and `guardwhen` end in the loud refusal, 3/3. `pingpong` and `haltthenwait` stay correct. So the probes do see a lost wakeup. |
| M-f: no check at all (the fixer's mutant) | **red**: both `a_halt_leaves_a_deadline_woken_wait_ready_once_*` | rr1's `n1sleep`: `slept 0` in 4 of 5 |
| M-c: `unpark` pushes without the check | green | `n1sleep` `slept 1` 10/10; `n1wait` refusal 3/3, no early `r2 1` |
| M-e: `wake_due_sleepers` pushes without the check | green | `n1sleep` `slept 1` 5/5 |
| M-g: `wake_for_halt`'s sleep branch pushes without the check | green | `n1sleep` `slept 1` 5/5 |
| M-b: the `set_aside` half of the check deleted | **green** | R2 |

The witnesses need the check gone from more than one site before they go red. No single site
is pinned. That is not a defect. It does mean the witnesses' names ("deadline-woken") describe
the program shape, not a site that has been shown to be load-bearing.

## Point 3: concern 2, posts during a stdin read

**Verdict: confirmed. The read holds the baton and starves every other activity. It breaks no
Task 18 rule. It is spec 2.1's recorded divergence. Severity Medium as a product divergence, for
a ruling. Not a Task 21 regression: base `159231bb8` is identical.**

Probe, as the brief asked. A started activity prints 8 lines, 0.25 s apart, stamped with
wall-clock seconds since main started (`.DateTime~fullDate` under `NUMERIC DIGITS 20`, not
`TIME('E')`, which is per activity). Main does `parse pull v` on a FIFO that is opened at once
and written `hi` at 2 s.

| | `SysSleep 0.25` between prints | busy wait between prints |
| --- | --- | --- |
| Oracle | `w 1 0.0` ... `w 8 1.8`, then `main hi 2.0` | the same |
| HEAD | `main hi 2.0`, then `w 1 2.0` ... `w 8 3.8` | `main hi 2.0`, then `w 1 2.0` ... `w 8 2.0` |
| Base `159231bb8` | `main hi 2.0`, then `w 1 2.0` ... `w 8 3.8` (2 runs) | |

The started activity does not run at all until the read ends. Its output is not merely held:
its first clause runs at 2.0 s.

Why it is not a Task 18 violation:
- Spec 2.1 lists PULL (`input.rs`) among the "wrappers that keep the baton" and says: "While one
  of these blocks (a console read, a pipe), the interpreter's other activities wait. This is a
  recorded divergence, counted by the pinning report."
- Task 18's classification in `phase-6-gate.md` (S4, Blocking operations) classes `input.rs:75`
  and `:128` as "spec 2.1 wrapper".
- The read is `pinned!` as `PinKind::PullWrapper` (`input.rs:311`).
- `fill_stdin_keeping` (`input.rs:359`) idles on the inbox (`:379`) and never runs a ready
  activity.

Why Medium rather than Low. P60 made the console read the main target of Ctrl-C, and that
exposes a consequence nothing records: **a halt during the read is lost for a started activity
the read starved before its first clause.**

The `keptend` program:

```
call on halt name mh
o = .w~new; m = o~start('go')
parse pull v
say 'main [' || v || ']'
m~result
say 'end'
exit
mh: say 'mh'; return
::class w
::method go
  call on halt name wh
  address system 'exit 3'
  say 'w' rc
  call time 'R'
  do forever; if time('E') >= 2 then leave; end
  return
wh: say 'wh' rc; return
```

SIGINT by PID at 1 s, `hi` written at 2 s, 5 runs each:

| | Output | Elapsed |
| --- | --- | --- |
| Oracle | `w 3` / `wh 3` / `mh` / `main []` / `end` | 2.02 s |
| HEAD | `mh` / `main []` / `w 3` / `end`. `wh` is never printed: the activity had no Rexx frame when the halt came, so `request_halt` dropped it, and its loop then ran its full 2 s. | 3.01 s |
| Base | identical to HEAD | |

With `call SysSleep 0.3` before the read, so that `w` gets its first slice, the halt does reach
it: HEAD `w 3` / `mh` / `main []` / `wh 3` / `end`. Only the order differs from the oracle there.

The DEVIATIONS file (`phase-4-exclusions.txt`) has no row for any of this. Entries 10 and 12
cover how a read is interrupted, not that it starves other activities.

Ruling options for the controller:
1. Add a DEVIATIONS row (or extend entry 12) naming both observables: no other activity runs
   during a console read, and a halt during the read misses a started activity that has not
   run yet.
2. Since P60 already runs the read on a pool thread and posts `Input`, let the read's wait run
   ready activities, as `next_runnable` does, instead of idling. Spec 2.1 calls this "converting
   a wrapper", which it says is "a later measured step".

## Point 4: the N4 witness's in-process design

**Verdict: sound for what it claims, and safe under parallel `cargo test`. What it claims is
narrower than the race (R-N4).**

- **Re-execution.** The outer test (`signal.rs:152-164`) only spawns
  `current_exe() --exact signal::tests::a_signal_pending_at_a_commands_end_halts_it
  --test-threads=1` with `REXX_SIGNAL_CASE_ALONE=1`, asserts the child's success and `1 passed`,
  and returns. It touches no static in the parent, so the halt that `serve_signal` posts to
  every live interpreter cannot reach a sibling test.
  - The parent's own arguments (filters, `--nocapture`) are not passed on. The child's output
    format is therefore the default, and the `1 passed` check is stable.
  - A renamed test fails loudly (`0 passed`).
  - The full lib binary at HEAD, run 3 times with default parallelism: 998 passed each time,
    the witness `ok` each time. It was also green in the workspace gate run.
- **Setting `PENDING` without the wake byte.** The timer thread takes the flag only when it
  wakes. In this single-activity program nothing wakes it between 0.2 s and 0.5 s. The fixer's
  mutants depend on that, and this review's mutant N4a confirms it:
  - Mutant N4a: `serve_signal_now()` deleted at the `Unblocked` arm (`scheduler.rs:1274`).
    Result: `a_signal_pending_at_a_commands_end_halts_it` red (inner failure at
    `signal.rs:182`). Reverted with `cmp`.
  - If the timer thread did wake, it would post the halt itself and the test would pass under
    the mutant. The design can therefore only err toward a false green, never a false red.
- **What it models.** It models the state "handler has run, flag set, nobody has posted".
  `serve_signal_now` does handle that state. The failing schedule seen in the field is a
  different state: the handler has **not run yet** when `Unblocked` is drained (R-N4). No
  in-process witness can model that without controlling which thread the kernel delivers to.

## Point 5: signals binary 20/20 under load

**Not met.** The binary passed 20 of 20 runs, but 2 of the 20 passed only through P48's rerun.

The runs used `--nocapture`, alongside the memcapped workspace suite, at load average
10.6-23.0. Runs 6 and 7 each printed one rerun line, both for the same test:

```
group under SIGINT: a mismatch in 21.43119ms, run again: Ended { code: Some(252), stdout: "a\nb -2\n", stderr: "     4 *-* address system 'sleep 2'\n..." }
group under SIGINT: a mismatch in 33.213984ms, run again: ...same...
```

That is N4's race, still open (R-N4).

## New findings

### R-N4 (Medium, the N4 residual). A terminal Ctrl-C can still halt the next command instead of the running one

The race is still open. The fix serves `PENDING` at the `Unblocked` arm (`scheduler.rs:1274`)
and after an on-baton command (`command.rs:925`). That covers only a handler that has already
run.

The failure scenario is rr1's group program, `say 'a'` / `address system 'sleep 2'` /
`say 'b' rc` / `address system 'sleep 2'` / `say 'c' rc`, with SIGINT to the process group at
0.5 s:

| | Under load (48 busy loops) | Without load |
| --- | --- | --- |
| HEAD | 58/60 4.1 at line 2. **2/60 `a` / `b -2`, then 4.1 at line 4.** | 60/60 line 2 |
| Oracle | 30/30 4.1 at line 2, at load average 46-50 | |

The signals binary showed the same thing, 2 in 20 (point 5).

Likely mechanism (inferred, not instrumented). `run_program` runs the interpreter on a spawned
thread (`lib.rs:2943` → `on_thread_of`, `lib.rs:3121`). The process's main thread only joins
it. A process-directed SIGINT is queued for one thread, usually the group leader, which is that
idle joiner. The handler runs only when that thread is next scheduled. The child dies of the
same signal, so the pool thread's `waitpid` returns and posts `Unblocked`. The holder can drain
that post before the joiner's handler has set `PENDING`.

The oracle runs Rexx on the group leader, which is itself in `waitpid`. Its handler therefore
runs before `waitpid` returns.

The report's sentence "The handler has run before any thread of this process returns from the
kernel after the group's `kill`" (rr1's fix direction, which the fix relied on) is false for
this process layout.

Fix directions:
- Make the thread that reaps the child the thread that takes the signal. Block SIGINT, SIGTERM
  and SIGHUP in every thread this crate creates (and in `rexx-run`'s main thread, which installs
  the handlers under P62), and unblock them only around the pool thread's child wait. The
  kernel then interrupts that `waitpid` and runs the handler before it can return.
- Or rule the residual a licensed divergence, with its measured rate. Even then, `signals.rs`
  `ctrl_c_halts_a_script_running_commands` and `..._redirected_command` stay on P48's rerun,
  which is what point 5 asked to be rid of.

### R1 (Low). N3's drop misses a halt that a `CALL ON ANY` trap caught

- `raise_requested_halt`'s check (`scheduler.rs:2062-2068`) looks up `HALT`, then `ANY`, and
  asks whether that trap is delayed.
- Delivery delays `traps[pending.condition]`, the **raised** condition's key
  (`run/condition.rs:520-521`). Undelaying uses the same key (`:583-584`).
- For `call on any`, there is no `HALT` entry, so nothing is delayed while the handler runs, and
  the second halt runs the handler again.
- The oracle delays the trap instruction's own `conditionName` (`CallInstruction.cpp:603`,
  `:632`). For `CALL ON ANY` that is `ANY`.

Failure scenario: `n3any`, 10/10. This crate gives `h in` / `h in` / `h out` / `h out`; the
oracle gives `h in` / `h out` (Items). Base is the same as HEAD.

Fix: delay and undelay the key the trap was found under. That is the condition where the table
has it, else `ANY`. The same line fixes R3. Witness: `n3any`, or R3's program, which needs no
timing.

### R2 (Low). The `set_aside` half of `make_ready` is unwitnessed

Mutant M-b deletes `&& !self.set_aside.iter().any(...)` (`scheduler.rs:357`). It stays green on
the lib and `signals.rs` tests.

The arm matters for an activity that is buried below a nested loop:
- a loop above it pops the activity and sets it aside;
- a halt's `unpark` would then add a second entry, which N1's stale entry would follow.

Four programs tried to reach it, 5 runs each on M-b, HEAD and the oracle. None separated M-b from
HEAD:
- a pinned `interpret 's~wait(0.001)'`;
- a wait in a loop header;
- a loop-header function, beside an activity in a pinned `interpret 'u~wait(0.05)'`.

So reachability is neither shown nor ruled out. Either add a witness that reaches the arm, or
record why it is unreachable, with a run.

Seen on the way, not a finding: in that shape, main stays set aside until the other activity's
pinned loop ends. `sa_e` printed `slept 1` at 5.05 s, against the oracle's 3.02 s. That is P30's
burial rule, outside Task 21.

### R3 (High, pre-existing, outside Task 21). A `CALL ON ANY` handler that raises its condition again recurses until it is killed

```
call on any name h
address system 'exit 1'
say 'main after'
exit
h:
  say 'h in' condition('C')
  address system 'exit 2'
  say 'h out' rc
  return
```

| | Result |
| --- | --- |
| Oracle, 2/2 | `h in ERROR` / `h out 2` / `main after`, rc 0, 0.02 s |
| HEAD and base, 2/2 each | `h in ERROR` without end. SIGTERM at 20 s does not stop it, because the halt is trapped by the same undelayed `ANY` handler. It ends by SIGKILL at 25 s. |

The cause is R1's: the wrong key is delayed. The bug predates Phase 6. File it or queue it
separately. The one-line fix belongs with R1.

## Mutations

Each mutation was applied by a Python replace that asserts one occurrence, and reverted by
`cp` from `p6-scratch/t21rr2/scheduler.rs.orig`, then `cmp` and `git status --short`. Tests were
`cargo test --release -p rexx-exec --no-fail-fast --lib --test signals`, in the scratch target
dir.

| Mutant | Result |
| --- | --- |
| M-a: `make_ready` pushes only into an empty queue | red: 19 lib, 3 signals; 3 of 5 point-2 probes refuse |
| M-b: no `set_aside` check | **survives** (R2) |
| M-c: `unpark` bypasses the check | survives; the probes are clean |
| M-e: `wake_due_sleepers` bypasses the check | survives; the probes are clean |
| M-f: no check at any site | red: both `ready_once` witnesses; `n1sleep` `slept 0` 4/5 |
| M-g: `wake_for_halt` sleep branch bypasses the check | survives; the probes are clean |
| N4a: no `serve_signal_now` at `Unblocked` | red: `a_signal_pending_at_a_commands_end_halts_it` |

## Checks (P51), at bf76afa1d

Target dir `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/p6-scratch/t21rr2/target`.
Each status was captured unpiped.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. The log has one
  `Checking rexx-exec` line.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, 143 result lines, 3023 passed, 0 failed, 4 ignored.
- Not run: loom, pinning clippy, loom clippy. They are not in this review's bar.
- `git status --short` at the end shows only the lead's `progress.md`. The target dirs are
  deleted.
