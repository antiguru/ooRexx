# Task 21 review: signals

Reviewer: s4-t21-rev (opus). Range `12f5d843a..e86c8fa29`, HEAD `e86c8fa29`. The tree was clean
apart from the lead's `progress.md`. Every mutation was applied by script and reverted by copying
the saved file back. `git diff --stat -- rust/` is empty at the end.

## Verdicts

- **Spec compliance: FAIL.** Spec section 4 says "every activity with frames takes HALT", and "a
  parked or sleeping one when woken, and it is woken". Three cases break this. A halt is dropped
  when the program ends before the next cold visit (F1). A blocking wait is never interrupted
  (F2). A semaphore-parked activity with a halt queued becomes an internal refusal (F3). P58's
  preemption bound is also missing (F6).
- **Task quality: CHANGES REQUIRED.**

Counts: High 2, Medium 4, Low 3, Info 2.

How the probes were run. Each oracle and crate run starts from a fresh empty directory. The
signal goes by `kill -s` to the PID I started, or to the process group I created with `setsid`.
Probes run under `env --default-signal=INT` unless a probe says otherwise (see F4 for why that
matters). Driver: `$CLAUDE_JOB_DIR/tmp/sig.sh`. Counts are runs with identical rc and stdout and
stderr.

## Findings

### F1. High: a signal's halt is dropped when the program ends before the next cold visit

`timer.rs:421-424` posts `Posted::Halt`. In a running activity, only these serve it:
- the countdown's cold path (`clause.rs:301-313`), every `CLAUSES_PER_CHECK = 1024` clauses
  (`clause.rs:31`);
- the scheduler's switch points (`scheduler.rs:1535`, `:1924`).

Nothing drains the inbox at program end, or after a blocking command or builtin returns. A
signal that lands during a long builtin or an `ADDRESS` command, in a program with fewer than
1024 clauses left, never halts anything. The process exits 0.

Failure scenarios, all run:
- **Ctrl-C during a script's command.** `say 'a'; address system 'sleep 2'; say 'b' rc; address
  system 'sleep 2'; say 'c' rc`, started under `setsid`, gets SIGINT to its process group at 1 s,
  which is what a terminal's Ctrl-C sends.
  - Oracle, 10/10: rc 252, `a`, Error 4.1 at line 2, ends at 1.0 s.
  - This crate, 10/10: rc 0, `a` / `b -2` / `c 0`. It runs the next command and ends at 3.0 s.
    The user's Ctrl-C killed the child and the script went on.
- **`kill -INT` during `address system 'sleep 5'` in a single-activity program.**
  - Oracle, 30/30: 4.1 at line 2, at 1.5 s.
  - This crate, 30/30: rc 0, `rc 0`, at 5 s.
- **A long builtin.** `do i = 1 to 30; x = copies('ab', 30000000); end; say 'done'`, SIGINT at 1 s.
  - Oracle, 30/30: 4.1 at line 3.
  - This crate, 10/10: rc 0, `done`.
- **Control showing the post does arrive.** The same command followed by a 5000-iteration loop
  halts at the loop's `end`, 2 s late. The post arrives and is served only at the next cold
  visit.

Fix direction:
- Serve a pending `INBOX` at program end, in `run_started_activities` or the root driver's exit,
  before the run returns.
- Serve it after every blocking operation returns, at least for the command wait and native
  builtins.

### F2. High: a blocking wait on the holder's thread is never interrupted; `SA_RESTART`'s absence has no effect

The report keeps `SA_RESTART` off "for the oracle's reason (blocking reads elsewhere return
EINTR)". In this crate nothing ever sees `EINTR`:
- `std` retries it in `read_line`, `read_to_end` and `Child::wait`.
- `grep -rn Interrupted rexx-exec/src` finds nothing.

So a signal cannot end a blocking wait, and M9's survival is the absence of that effect, not
just a missing witness.

Failure scenarios, all run:
- **`say 'ready'; parse pull x` with stdin a FIFO held open and never written,** SIGINT at 1.5 s.
  - Oracle, 5/5: Error 4.1 at 1.5 s.
  - This crate, 5/5: hangs until killed (rc 137).
- **`x = linein()`:** the same result, oracle 4.1 and this crate hanging.
- **The `ADDRESS` wait in F1:** the oracle's `waitpid` returns `EINTR` and it halts at once. This
  crate waits for the child.

Interactively, Ctrl-C at a `PULL` prompt does nothing until Enter is pressed.

Report concern 3 lists blocking waits as "not woken", and P59 rules only on sleepers. This needs
the lead's ruling, or an implementation. Possible routes:
- run stdin reads and the command wait off the baton where a halt can abandon them;
- a read on a pollable fd together with the wake socket.

### F3. Medium: a signal turns a semaphore-parked activity into an internal refusal

`wake_for_halt` (`scheduler.rs:2086`) wakes only `asleep` and `when_parked`. An activity parked
in `SysWaitEventSem` gets the halt request queued and stays parked.

The probe: `o~start('w')`, where `w` does `call SysSleep 10`, then main does
`rc = SysWaitEventSem(SysCreateEventSem())`, SIGINT at 1 s. The signal wakes and halts `w`, which
ends. That leaves nothing that can end main's wait.
- This crate, 30/30: rc 120 with `w`'s Error 4.1, then
  `rexx-exec: a wait that nothing left to run can end is not implemented`.
- Oracle, 30/30: main's Error 4.1 at line 5 (`SysWaitEventSem`), then `w`'s 4.1 at line 10. It
  then waits on and is killed (rc 124).

The halt the spec promises becomes a Loud refusal. Without a second activity, the crate refuses
the same wait before any signal can arrive. That is pre-existing and not counted here.

Fix: withdraw a semaphore wait (and a message wait) that has a halt queued. Or rule it, with a
witness.

### F4. Medium: an undocumented divergence when SIGINT is ignored at start

`signal.rs:61-65` installs only over `SIG_DFL`. The oracle reads only SIGHUP's previous action
(`SystemInterpreter.cpp:136-145`), so it installs SIGINT over `SIG_IGN`.

A non-interactive shell starts background jobs (`rexx prog &`, `sh -c '... &'`) with SIGINT
ignored. There:
- the oracle halts (5/5 plus 1, rc 252 at 1.0 s);
- this crate ignores SIGINT and runs to the end (5/5 plus 1, rc 0, `after 0`, at 5.1 s).

This was found because my own background-job driver hit it, before I added
`env --default-signal=INT`.

The spec's "this design checks each" covers the behaviour. DEVIATIONS entry 10 describes only
the opposite direction, "halts where the oracle dies", and neither entry 10 nor `signals.rs`
witnesses this one. Add this direction to entry 10, with a witness run under
`sh -c "trap '' INT; exec ..."`.

### F5. Low: the witnesses depend on the test process's inherited SIGINT disposition

`signals.rs:64-77` spawns `rexx-run` with the test binary's dispositions. Running the signals
binary as `sh -c "trap '' INT; exec <signals-bin>"` fails 7 of 10 tests: every SIGINT one. This
is how `cargo test &` in a script, or some CI wrappers, start it.

Fix: reset the dispositions in the child. Spawning through `env --default-signal=INT,TERM` is
one way, and needs no `unsafe` in tests.

### F6. Medium: P58's preemption bound is not implemented (ruled; reported for the fix round)

`tests/loom.rs:142` (`an_idle_deadline_is_never_lost`) and `:162`
(`a_sleeper_registers_as_the_timer_exits`) call plain `loom::model`. The gate command still does
not finish.

With `LOOM_MAX_PREEMPTIONS=5`, `--release`, all 14 models pass in 32.05 s (run). The bound
belongs in code: a `loom::model::Builder` with `preemption_bound = Some(5)` on those two models
only.

### F7. Medium: "halts every live interpreter" is unwitnessed

The mutant I applied changes `for entry in &live.live` to `live.live.iter().take(1)` at
`timer.rs:422`. It survives:
- `cargo test --profile mutation -p rexx-exec --test signals --no-fail-fast`, 10/10 passed;
- the loom signal models, 2/2 passed.

Every witness has one interpreter. A model with two registrations and one signal, asserting a
halt in both inboxes, would catch it. So would a unit test with two `Interp`s.

### F8. Low: the loom anomaly is real but mislocated, and the shipped flag is unmodelled

I reproduced it outside this crate, in a 30-line standalone loom 0.7.2 model
(`$CLAUDE_JOB_DIR/tmp/loomrepro`). Thread A does `flag.store(true)`, then sets `woken` under a
mutex and calls `notify_one`. Thread B loops: `flag.swap(false)` and break on true, else wait on
the condvar for `woken`.

| B's flag read | Release / AcqRel | SeqCst |
| --- | --- | --- |
| `swap` | deadlock | deadlock |
| `fetch_and(false)` | deadlock | — |
| `load(Acquire)` | passes | — |
| `compare_exchange` | passes | — |

loom's RMW reads a stale value despite the mutex's happens-before. So the fault is in loom's
`swap`/`fetch_and`. It is not the crate's model, and not the Release store the report names.

It hides no ordering bug in the shipped code. The handler's store comes before `write(2)`. The
timer's `read(2)`, then the registry lock, come before the `swap`, and an RMW reads the latest
value whatever its ordering.

But the model keeps the flag under the wake lock (`sync.rs:137-144`), so the shipped lock-free
flag is not modelled. I replaced it with a loom `AtomicBool`: a Release store, and a take by
`compare_exchange(true, false, AcqRel, Acquire)`. Both signal models then pass (run, reverted).

Fix:
- Model the flag that way.
- Correct the report's sentence ("lost a Release store").

### F9. Low: `__errno_location` is Linux-only

`signal.rs:84` uses it. `cargo check --target aarch64-apple-darwin -p rexx-exec` gains E0425 at
`signal.rs:84` (run). macOS already failed to build, with E0308 at `rexxutil.rs:154`, so no
working platform broke.

Fix: `libc::__error` under `cfg(target_vendor = "apple")`, or `std::io::Error::last_os_error` to
read and a cfg'd setter to restore.

### F10. Info: `ADDRESS` children see SIGPIPE at its default, where the oracle's inherit `SIG_IGN`

An `ADDRESS` child's `/proc/self/status` shows:

| | SigIgn | ignored |
| --- | --- | --- |
| Oracle | `0x1004` | SIGQUIT, SIGPIPE |
| This crate | `0x4` | SIGQUIT |

So `address system 'yes | head -1'` writes `yes: standard output: Broken pipe` to stderr on the
oracle only. This is pre-existing: `std::process::Command` resets SIGPIPE. It qualifies the
report's "SIGPIPE ignored as the oracle does", which is true for the process but not for its
children.

The handlers are not inherited across exec. The self-pipe is CLOEXEC: the socket count in an
`ADDRESS` child is 0 here, and the oracle's is 1, its rxapi socket.

### F11. Info: report concern 5, confirmed

I sent SIGINT to the `rexx_exec` unit-test binary 6 s into a `--test-threads 1` run. It ran on
to the end (status 0 at 61 s), as concern 5 predicts and as the oracle's library behaves.
Ctrl-C on `cargo test` therefore leaves an orphaned test binary running. I found no existing
test that relies on default SIGINT:
- `program_end.rs`, `prompt_before_read.rs` and the oracle harness kill with SIGKILL;
- a handler is not inherited across exec.

## Checked and sound

**Handler** (`signal.rs:74-89`). By inspection:
- it does a SeqCst store, an Acquire load, one `write(2)`, and an errno save and restore, and
  nothing more;
- nothing allocates or locks;
- `sa_mask` is full and `sa_flags` is 0, so there is no `SA_RESTART`;
- install happens only where the action is `SIG_DFL`.

The SAFETY notes match D-U3's invariant. The errno-clobber mutant (`*errno = 99`) survives
`signals.rs`, which cannot observe it, so inspection is the only check.

Under `nohup`, SIGHUP stays ignored and SIGINT halts. The witnesses pass this, and my
background-job run confirms `SIG_IGN` is respected (F4).

**Self-pipe lifetime.**
- The registry static owns it and never closes it, so a write never reaches a reused fd.
- Discard at first registration and timer exit both happen under the registry lock. A signal
  between interpreters halts nothing, and the loom model covers this.
- When the socket is full, the write returns EAGAIN, which is ignored, and `PENDING` stays set.
  By inspection.
- I sent a burst of 3000 SIGINTs to a `CALL ON HALT` loop of short sleeps. rc 0, it ran to its
  end, and no crash (run).

**Halt reach.** I checked these by running, once each for the last two:
- SysSleep;
- GUARD WHEN, both parked and at program end;
- a busy loop;
- a message wait on an activity that is sleeping. This crate halts both activities at 1.0 s.
  The oracle prints both and hangs, rc 124.
- an `ADDRESS` command on a pool thread while another activity sleeps. This crate halts both,
  main's when the command ends at 5.2 s. The oracle halts main at 1.5 s and hangs, rc 124.

The waits that fail are F1-F3.

**Witnesses.** They are wall-clock bound and rerun once on a mismatch, per P48, inside the file.
`signals.rs` passed 20/20 runs while the workspace suite ran alongside, at load average 9-17. The
rerun is silent, so a first-run miss leaves no trace. Printing it would make flakiness visible.

## Checks (P51), at e86c8fa29

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. The status was captured
  directly, with output to a file.
- `cargo test --workspace --release --no-run`: exit 0. Then
  `memcap 8G cargo test --workspace --release`: exit 0, 143 result lines, 2997 passed, 0 failed,
  4 ignored. My first `memcap` run OOM-killed at 8G while recompiling, because the mutation
  restore had touched `sync.rs`. I rebuilt outside `memcap` and reran.
- loom with `LOOM_MAX_PREEMPTIONS=5 --release`: 14/14 in 32.05 s. I did not run it unbounded.
  The report's measurement stands.
