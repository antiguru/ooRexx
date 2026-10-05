# Task 21 re-review, fix round 1

Reviewer: s4-t21-rr1 (opus). Range `e86c8fa29..159231bb8`, HEAD `159231bb8`. Every mutation was
applied by a script or `sed`, built in a separate target dir (`p6-scratch/t21rr1/probe-target`),
and reverted by copying the saved file back. `git status --short` at the end shows only the
lead's `progress.md`.

How the probes were run:
- Driver: `$CLAUDE_JOB_DIR/tmp/drv.py`. Each run gets a fresh empty directory and
  `env --default-signal=INT,TERM,HUP`.
- The oracle runs under the CLAUDE.md wrapper (`ulimit -v 1048576`, `timeout -k 5 20`). This
  crate runs under `timeout -k 5 20` with no `ulimit`, for P64's reason (concern 2).
- A PID signal goes to the interpreter itself, the child of `timeout`, found through
  `/proc/<pid>/task/<pid>/children`. A group signal goes to a session made with `setsid`.
- "Identical" means rc, stdout and stderr, with the path masked.
- The base binary is `e86c8fa29`, built from `git archive` into a separate target dir.

## Verdict

**CHANGES REQUIRED.**

Counts: findings F1-F11: 10 resolved, 1 partly resolved (F1). New findings: High 1, Medium 3,
Low 3.

- **N1 (High)** is a regression from F3's fix. Under a halt, an activity that is already ready
  gets readied a second time. The stale ready entry then ends a later untimed
  `EventSemaphore~wait` as posted, or cuts a later `SysSleep 2` to 10 µs.
- **N2 (Medium)** is a regression from F2's fix. A stdin read spins on the CPU while any other
  post is pending.
- **N3** and **N4** are halt-delivery divergences the new mechanisms expose or leave open.

## Findings F1-F11

| | Status | Evidence at HEAD |
| --- | --- | --- |
| F1 | **Partly resolved** | Builtin, command and group Ctrl-C scenarios are identical to the oracle, 5/5 each. A long native **method** at program end still drops the halt, 10/10 (below). The race N4 is open. |
| F2 | Resolved (P60) | The `PARSE PULL` and `LINEIN()` probes on a held-open pipe are identical to the oracle, 5/5 each, ending at 0.02 s. The command wait is identical too (F1 row). Regression N2 is new. |
| F3 | Resolved (P65) | The review's probe ran 30/30 on each side with identical descriptors. This crate ends at 0.01-0.02 s; the oracle ends at 9.97-10.00 s because it leaves the sleeper parked (P59). Regression N1 is new. |
| F4 | Resolved (P61) | `sh -c "trap '' INT; exec …"` with SIGINT at 1 s: both give rc 252 and 4.1, 3/3 each. |
| F5 | Resolved (P62) | The signals binary under `sh -c "trap '' INT TERM HUP; exec <bin>"` passed 25/25, three times. |
| F6 | Resolved (P58) | `registration_model` is a `Builder` with `preemption_bound = Some(5)` (`tests/loom.rs:141-143`). The loom gate finishes (Checks). |
| F7 | Resolved | M14 (`live.live.iter().take(1)` at `timer.rs:429`) goes red, run here: `a_signal_halts_every_live_interpreter` reaches a loom deadlock. |
| F8 | Resolved | By inspection, the model's `Wake::take_signal` (`sync.rs`) and `signal::take_pending` use the same `compare_exchange(true, false, AcqRel, Acquire)`, and the handler's store is `Release` in both. |
| F9 | Resolved | `cargo check --target aarch64-apple-darwin -p rexx-exec` gives only E0308 at `rexxutil.rs:154`. |
| F10 | Resolved (P63) | DEVIATIONS entry 11. |
| F11 | Resolved (P62) | `signal::tests::running_a_program_installs_no_handler` passes in the gate. |

### F1's remainder: a long native method at program end

The program is the review's builtin loop, with the builtin replaced by the equivalent string
method:

```
do i = 1 to 30
  v = 'ab'~copies(30000000)
end
say 'done'
```

SIGINT goes by PID at 1 s.
- Oracle, 10/10: rc 252, 4.1 at line 2, 0.03 s after the signal.
- This crate, 10/10: rc 0, `done`, 1.0-1.2 s after the signal. The loop runs to its end.

The halt bit is tested only after `builtin::run` (`builtin.rs:743`) and after a command clause
(`command.rs:925`). A native method's return is not covered, and the program has about 60 clauses,
so the 1024-clause countdown never fires. The review's first fix direction was to serve a pending
halt at program end, before the run returns. That was not done. The report's "Program end"
paragraph covers only a halt already queued before the last clause.

Fix: serve `HALT` once at main's end, before the run returns, and raise it on the last clause as
the oracle's `processClauseBoundary` does. Or test the bit after native method calls too, at
the same cost the report measured for builtins.

## New mechanisms

### The HALT request bit and the check after each builtin

- **Where it is served.** After every builtin and every on-baton command clause. This is
  verified by the identical builtin, command and last-clause-command runs. It is not served at
  program end or after a native method (F1 above).
- **Can a halt arrive twice?** No, not from one signal. `request_halt` refuses a second queued
  request (`scheduler.rs:2225-2229`). `serve_halt_now` clears the bit, then drains. The timer
  posts first and sets the bit second. So a drain that takes the post before the bit is set
  leaves one stale bit. The next builtin then pays one cold drain that finds nothing.
- **A halt inside a CALL ON HALT handler.** It does arrive where it should not: N3.
- **SIGNAL ON HALT with a second SIGINT in the handler.** Identical, 5/5 on each side: rc 252,
  `h in 3`, 4.1 at line 7.
- **Mutants.**
  - `halt_all` in reverse handle order: red on `sigint_ends_a_semaphore_wait` and
    `sigint_reaches_a_parked_guard_when`.
  - `queued_during_delivery: true` in place of `owner == running` (`scheduler.rs:2069`): green
    on `signals.rs`. It is equivalent in release: the flag is read only by a `debug_assert!`
    (`clause.rs:270`). Not a finding.

### Off-baton command waits and stdin reads (P64)

- **The abandoned command.** The witness `an_abandoned_command_ends_no_later_one` stays green
  when the token is made constant. See N5, which has a program that shows the defect the token
  exists to prevent.
- **The child's fate.**
  - A terminal Ctrl-C (a group SIGINT) on `say 'a'; address system 'sleep 2'; say 'b' rc; …`
    is identical, 5/5 on each side: 4.1 at line 2, at 0.00-0.02 s.
  - With a PID-only SIGINT, both abandon the wait and leave the child running.
  - The race in N4 makes the group case nondeterministic here. It was seen twice in the
    witnesses.
- **Stdin bytes.** Verified: none are lost or duplicated. The program is a `CALL ON HALT`
  program that does a halted `PARSE PULL`, then `PARSE PULL`, `LINEIN`, `CHARIN(,,3)`, `LINEIN`
  and `LINEIN`, printing `LINES() CHARS()`. The driver writes `one\ntwo\nthree\nfour\n` 0.5 s
  after the signal.
  - This crate, 5/5: `[]`, `one`, `two`, `thr`, `ee`, `four`, with no loss and no duplicate.
  - Oracle, 5/5: `[]`, `[]`, `two`, `thr`, `ee`, `four`. The oracle loses `one`, which
    DEVIATIONS entry 10 records.
  - Nothing in the suite witnesses it (N6). The counts after a halt diverge (N7).
- **The read's idle loop spins** whenever a post other than input or halt is pending (N2).

### Semaphore wait withdrawal (F3, P65)

- The review's probe ran 30/30 identical on both sides (F3 row).
- `call_on_halt_sees_a_withdrawn_semaphore_wait` passes.
- The withdrawal unparks an activity that a timed wait's deadline had already readied: N1.

### P51 bar: is `signals.rs` flaky?

- 20/20 passes of the signals binary while the memcapped workspace suite ran alongside. Load
  average was 13-23. Output was captured, so a P48 rerun would not have shown.
- 20/20 more with `--nocapture`, at load average 1.6-3.7. Two first-run mismatches were rerun
  and passed:
  - `redirected` and `group`, each `stdout "a\nb -2\n"`, halted at line 4 instead of line 2;
  - this is N4.
- 30/30 more with `--nocapture`: no mismatch.
- So the suite does not fail, but P48's rerun is masking a real race. It showed in 2 of 50
  binary runs.

## New findings

### N1. High: a halt readies a deadline-woken activity twice; the stale entry ends a later wait early

`wake_for_halt`'s semaphore branch (`scheduler.rs:2161-2172`) unparks when the sleeper was
removed **or** `Semaphores::withdraw` answers true.

A timed `EventSemaphore~wait` or `MutexSemaphore~request` is both a sleeper and enqueued.
- When its deadline passes, `wake_due_sleepers` (`scheduler.rs:1370-1378`) pops the sleeper and
  pushes the activity onto `ready`.
- The activity stays in `semaphores.parked` until its resume runs `retest_semaphore`.
- If a halt is served in that window, `withdraw` answers true, and `unpark` pushes the same
  activity onto `ready` a second time.

A busy second activity holds the baton for up to a slice, so the window is wide.

Instrumented: a temporary `eprintln!` when `table.ready.contains(&activity)` in that branch,
reverted. It printed `PROBE double ready ActivityId(0)` in 4 of 5 runs.

The failure scenario. Main loops on `r = s~wait(0.001)` (an `EventSemaphore`) under
`CALL ON HALT`, and leaves the loop once halted. A started activity busy-loops for 3 s. SIGINT
goes by PID at 1 s.
- **Then `r2 = t~wait`,** on an `EventSemaphore` nobody posts.
  - This crate, 9/10: `r2 1`. The untimed wait answers "posted": `retest_semaphore` answers
    `true` to any wake of an untimed event wait (`semaphores.rs:225-227`).
  - Oracle, 3/3: the wait never ends, and the process is killed at the timeout.
- **Then `call SysSleep 2`.**
  - This crate, 10/10: it returns after 0.000006-0.000013 s.
  - Oracle, 2/2: it returns after 2.0003 s.
  - Base `e86c8fa29`, 5/5: 2.004 s. So this regression is new in this round.

Fix: for a timed wait, unpark only where the sleeper was still in the heap. Where the sleeper is
gone, the activity is already ready: mark `woken_by_halt` and withdraw it, but do not push it.
`wake_semaphore_waiter` already has this rule. A witness is the `SysSleep` program above.

### N2. Medium: a stdin read busy-spins while any other post is pending

`fill_stdin` (`input.rs:341-369`) loops on `self.timer.idle()`. It keeps every post that is not
`Input` or `Halt`, and `requeue`s them (`input.rs:365`). `requeue` sets `INBOX` and the queue is
non-empty again, so the next `idle()` returns at once with the same posts. The loop spins until
input arrives.

The failure scenario. A started activity runs `address system 'sleep 1'`, which waits off the
baton. Main sleeps 0.2 s, then does `parse pull v`. stdin gets `hi` at 4 s.
- This crate: elapsed 4.00 s, user 1.93 s, sys 1.07 s. That is one core for the 3 s after the
  command's `Unblocked` is posted.
- Control without the started activity: user 0.01 s.
- Base `e86c8fa29`: user 0.02 s.

Any `Output`, `Unblocked`, `Completed` or `Recall` posted during a prompt does this. Examples are
an abandoned command still writing, or another activity's command ending.

Fix: block on the inbox for a **new** post while keeping the set-aside ones local. For example,
hold `kept` across iterations and `requeue` only on exit. Or idle on a condition that ignores the
kept posts.

### N3. Medium: a second halt inside a CALL ON HALT handler ends the program; the oracle ignores it

`raise_requested_halt` (`scheduler.rs:2045-2077`) finds no enabled trap while the `CALL ON HALT`
handler runs. It takes the `None` arm and raises an untrapped 4.1.

Failure scenarios, SIGINT by PID twice, 0.5 s and 0.7 s apart:
- **A `CALL ON HALT` handler that busy-waits 2 s** (`callbusy5.rex`).
  - Oracle, 5/5: rc 0, `h in 4` / `h out` / `main after`.
  - This crate, 5/5: rc 252, 4.1 at the handler's line 9.
- **A handler that does `call SysSleep 2`** (`callh2.rex`).
  - Oracle, 5/5: rc 0 with `h out 4`.
  - This crate, 5/5: rc 252.
- **The same through `Message~halt` twice** (`mhalt.rex`).
  - Oracle: `h in` / `h out` / `w end`.
  - This crate: 4.1 inside the handler.

The behaviour predates this round: base `e86c8fa29` gives the same rc 252. The `Message~halt`
case shows the defect is older than Task 21's signals. Signals make it reachable by pressing
Ctrl-C twice during cleanup. The oracle leaves the condition delayed while its handler runs.

Fix: while the HALT trap's handler runs (the trap delayed), drop a requested halt as the oracle
does, rather than raising it untrapped. Or rule it a divergence, with a witness.

### N4. Medium: a terminal Ctrl-C can lose to the child's end and halt a clause later

The group SIGINT reaches the child and this process at once. Two posts then race:
- the pool thread's `Unblocked` for the dead child;
- the timer thread's `Halt`, sent after it reads the self-pipe.

When `Unblocked` arrives first, the drain resumes the activity, and the command clause ends with
no halt queued. `say 'b' rc` then runs, and the halt lands in the next command. The oracle's
handler sets its halt on the signalled thread before `waitpid` returns `EINTR`, so the oracle
always halts at the command's own line (the review's group runs, 10/10).

Seen, not induced: in the `--nocapture` batch, `ctrl_c_halts_a_script_running_commands` and
`ctrl_c_halts_a_redirected_command` each mismatched once on the first run. Both gave
`stdout "a\nb -2\n"` and `4 *-* address system 'sleep 2'`, and P48's rerun passed them. That is
2 in 20 binary runs. It was 0 in 60 runs of those two tests alone and 0 in 200 driver runs, so
it is load- and timing-dependent.

Fix direction: before an `Unblocked` resumes its activity (`scheduler.rs:1259`), take
`signal::PENDING` on the holder's thread and serve the halt first. The handler has run before
any thread of this process returns from the kernel after the group's `kill`, so the flag is
already set when the pool thread posts.

### N5. Low: the abandoned-command token's identity is unwitnessed

Mutant: delete `self.activities.next_block += 1;` (`scheduler.rs:643`), which makes every token
0. Result: `signals.rs` stays 25/25 green.

The witness's abandoned child (`sleep 1`) always ends before the next command (`sleep 2`), so
dropping the first end with token 0 happens to be right.

With the order reversed, the mutant misroutes:

```
call on halt name h; say 'a'; address system 'sleep 3; exit 7'; say 'first' rc
call time 'R'; address system 'sleep 0.3'; say 'second' rc (time('E') < 1)
```

- Mutant, 3/3: `second 7 0` at 3 s.
- Shipped, 3/3: `second 0 1` at 0.3 s.
- Oracle, 3/3: `second 0 1`. Its `first` RC is unset, -64 or -38.

Fix: add this program as a witness.

### N6. Low: "the next read loses nothing" is unwitnessed

Mutant: `Input::receive` drops the chunk when `exhausted` is set, which drops the chunk the halted
read was waiting for (`input.rs`, reverted). Result: `signals.rs` stays 25/25 green.

The stdin program above, under the mutant, 3/3: `[]` for every read after the halt, against
`one two thr ee four`. DEVIATIONS entry 10 states the property, and it needs a witness. The
program above, with `CALL ON HALT`, a halted `PARSE PULL` and input written after the signal,
would do.

### N7. Low: the counts stay 0 for the rest of the run after a halted read; the oracle's recover

`Input::interrupted` sets `exhausted` (`input.rs:118-119`), and nothing clears it.

In the stdin program, after `four\n` is buffered:
- this crate answers `LINES() CHARS()` `0 0`;
- the oracle answers `1 1`, 5/5.

The comment's "from now on" and entry 10's "leaves the counts at 0 on both" are true only
before the next read on the oracle.

Without a halt, this crate's stdin counts already differ from the oracle's (`0 1` against
`1 1`, also at `e86c8fa29`). That pre-existing divergence does not belong to this round.

## Mutations

Each mutation was applied by a script or `sed` and reverted by copying the saved file back.
`signals.rs` ran as `cargo test --release -p rexx-exec --test signals --no-fail-fast`, and the
loom model as `RUSTFLAGS="--cfg loom" cargo test -p rexx-exec --test loom --no-fail-fast
every_live`.

| Mutant | Result |
| --- | --- |
| Constant block token (`scheduler.rs:643`) | survives, N5 |
| Chunk dropped after a halted read (`input.rs` `receive`) | survives, N6 |
| `queued_during_delivery: true` (`scheduler.rs:2069`) | survives; equivalent in release |
| `halt_all` in reverse order (`scheduler.rs:2114`) | red, 2 tests |
| M14 again, `take(1)` (`timer.rs:429`) | red, loom deadlock |

## Checks (P51), at 159231bb8

Target dir `p6-scratch/t21rr1/target`, with loom in `.../target-loom`. Each status was captured
unpiped.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. Captured directly; it checked
  `rexx-exec`.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, with 143 result lines, 3013 passed, 0 failed and 4 ignored.
- Loom, `RUSTFLAGS="--cfg loom" memcap 8G cargo test -p rexx-exec --test loom`: exit 0, 15
  passed, 361.78 s test time and 396 s wall including the build.
- One check strayed from the scratch dir. `cargo check --target aarch64-apple-darwin` for F9 ran
  without `CARGO_TARGET_DIR` and created `rust/target/aarch64-apple-darwin`. Its mtimes were all
  from that run, so I removed it.
