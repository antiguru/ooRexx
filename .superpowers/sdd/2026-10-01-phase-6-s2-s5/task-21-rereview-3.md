# Task 21 re-review, fix round 3

Reviewer: s4-t21-rr3 (opus). Range `bf76afa1d..bfffdd33b`, HEAD `bfffdd33b`.

How the probes were run:
- Driver: `$CLAUDE_JOB_DIR/tmp/drv.py` (job `91ae65d5`). Each run gets a fresh empty directory
  and `env --default-signal=INT,TERM,HUP`. A PID signal goes to the interpreter process; a group
  signal to its process group; `--tg` is a `tgkill` to named threads of the interpreter. `--fifo`
  makes a named FIFO `f` in the run directory with a writer or reader thread in the driver.
  The pty probes use `ptydrv.py`: standard input is a pty slave, whose path is also the
  program's argument.
- Oracle: `bash -c 'ulimit -v 1048576; LD_LIBRARY_PATH=... exec rexx FILE'`, with the 20 s
  timeout (then SIGTERM, then SIGKILL to the group) in the driver rather than `timeout(1)`:
  `timeout` forwards a group signal to its child, which would double every group signal.
- "HEAD" is `rexx-run` built at `bfffdd33b`, copied aside before any mutation. "Base" is
  `rexx-run` at `bf76afa1d`, built from `git archive` (`rust` and `interpreter`) in a separate
  target dir, with one `Compiling rexx-exec` line.
- "Under load" means 48 busy `sh` loops on 32 cores (`load.sh 48`), load average quoted.
- "Identical" means rc, stdout and stderr, path masked. Run counts are stated per cell.

## Verdict

**CHANGES REQUIRED.**

Counts:
- Items: 5 resolved (R-N4 for one interpreter, R1, R3, R2, concern 2 as ruled), 0 partly,
  0 not resolved. Concern 2's fix carries the regression below.
- New findings: High 1 (F-1, regression from P66), Medium 1 (F-2, pre-existing, outside
  Task 21), Low 1 (F-3, a note for the API phase), Info 1 (F-4).
- Points: EINTR audit clean (no path gives a wrong result); readhalt rerun licensable (the oracle
  takes the same order 1/30 under load); children's masks as recorded; two interpreters reopen
  the race (F-3); no per-clause cost.

What blocks approval: **F-1.** With P66, a read of the default input stream runs the other
activities on the reader's stack while it holds `.stdin`'s guard, and any other activity that
then touches the default input stream (`lines()`, PULL, LINEIN) ends the program with the loud
inverted-wait refusal, rc 120. Base and the oracle both run these programs (`main [one]` /
`w lines 1`). No test has two activities on the default input stream.

## Items from re-review 2

| Item | Status | Evidence at HEAD |
| --- | --- | --- |
| R-N4 (N4 residual) | **Resolved for one interpreter per process**, as the P67 amendment scopes it. Open with two (point 5). | rr2's `group` program, SIGINT to the group at 0.5 s, under load: HEAD 120/120 4.1 at line 2 (load average 0.6-21 for the first 60, 30-37 for the second). Base, same harness, interleaved with the second HEAD set: 1/60 `a` / `b -2` then 4.1 at line 4 (load 37-42), so the probe still sees the race. Oracle 30/30 line 2 (load 22-28). The signals binary 20 times with `--nocapture` at load average 49-54: 37 passed each run, **no rerun line at all**. |
| Concern 2 (P66) | **Resolved as ruled, but the fix introduced a regression** (F-1, High). | rr2's stamped probes, 10 runs each side: busy form identical to the oracle 10/10; `SysSleep` form `w 1` ... `w 8` then `main hi 2.0` 10/10 on both, stamps drifting by up to 0.1 s by `w 8`. The drift is SysSleep overshoot, not the read: with `call SysSleep 2` in place of the read, HEAD and base drift the same way (10 runs each). `keptend` (SIGINT 1 s, input 2 s): identical 10/10, `w 3` / `wh 3` / `mh` / `main []` / `end`. |
| R1 | **Resolved.** | `n3any` (SIGINT at 0.5 and 1.2 s): identical 10/10, `h in` / `h out` / `main after`. |
| R3 | **Resolved.** | `anyerror`: identical 2/2, `h in ERROR` / `h out 2` / `main after`, rc 0, 0.03 s. |
| R2 | **Resolved.** | `a_halt_readies_a_set_aside_activity_once` exists and passes; M-b (the `set_aside` check deleted) turns it red (Mutations). |
| N6/N7 (rechecked, the read path moved) | Unchanged from rr2. | `n67` (SIGINT 0.5 s, four lines at 1.0 s): HEAD 30/30 `[]` `one` `two` `thr` `1 1` `ee` `four`; oracle 30/30 the same but `one` lost (DEVIATIONS 10). |

## Point 2: EINTR audit

**Verdict: no path turns an EINTR into a wrong result at HEAD.** Every blocking call the
interpreter thread (or a thread whose mask it lends) can be in either retries `EINTR` or cannot
block. The stream-class paths that the brief names as the risk cannot block at all, because of a
pre-existing defect (F-2): a FIFO or terminal opened through the stream class reads nothing and
writes nothing.

Which threads can take a halting signal now (`/proc/<pid>/task/*/status` during a command):
main and `rexx-timer` `SigBlk 0x4003`; `rexx-interp` `0`; the command's pool worker and its
scoped stderr reader `0`. Linux queues a process-directed signal for the first thread after the
group leader that does not block it (`complete_signal`'s round robin from `curr_target`), which
here is nearly always `rexx-interp`.

How the call sites were enumerated: every `.read(`, `.write(`, `read_to_end`, `read_line`,
`write_all`, `.wait()`, `thread::sleep` and `libc::` call under `rexx-exec/src`, `rexx-api/src`
and `rexx-core/src`, outside tests. The only `libc` calls are the signal ones and the handler's
`write`; no `poll`, `nanosleep`, `sem_wait` or socket call exists.

| Path | Blocking call, thread | Result of EINTR | How shown |
| --- | --- | --- | --- |
| Stream class CHARIN / LINEIN / LINES on a named FIFO | `read_from` seeks before reading (`dispatch/stream.rs:684-697`); `lseek` on a FIFO is `ESPIPE`, the `?` returns it, and `unwrap_or_default` answers nothing | **Unreachable**: the read never blocks. Pre-existing defect F-2. | `fcharin` / `flinein` (SIGINT 0.5 s, data at 1.0 s), 30 runs each side. HEAD 30/30: `x=[]` `ERROR:0` (CHARIN) and `l=[]` `NOTREADY:EOF` (LINEIN), process gone before the signal. Oracle 30/30: blocks; the EINTR becomes `ERROR:4 Interrupted system call`, then `halted`, then the next read gets the data (and loses a byte in the LINEIN case). |
| Stream class CHAROUT / LINEOUT to a FIFO with a slow reader | `write_at` seeks first (`:1314-1317`) | **Unreachable**: every write fails `ERROR:29 Illegal seek`. F-2. | `flineout` (300 lines of 1000 bytes, reader starts at 1 s, SIGINT 0.5 s), 5 runs each: HEAD `fail at 1 ERROR:29` / `wrote 300 bad 300`, 0 bytes reach the reader. Oracle: `halted 67`, 297999 bytes reach the reader. |
| Stream class on a terminal (a pty slave by path) | the same seek | **Unreachable.** F-2. | `ptystream`, 30 runs each: HEAD ends before the signal at 0.5 s (the `kill` finds no process) with `NOTREADY:EOF` twice and no `halted`; the oracle blocks, then `NOTREADY:EOF` / `halted`. |
| Stream class on regular files | `File::read` in `read_from` is not retried; `write_all` and `File::open` (std's `cvt_r`) are | Not reachable on a local filesystem, whose reads do not return EINTR. On NFS `intr` or FUSE it would answer a short read with `ERROR`, which is what the oracle's unretried `SysFile::read` (`SysFile.cpp:402`, `:441`) does too. | Code reading only. |
| Default input stream (PULL, PARSE PULL, LINEIN/CHARIN/LINES on stdin), pool thread | `read_stdin_chunk` inside `signal::unblocked` | Retried (`input.rs:80-84`); the halt wakes the reader through `wake_for_halt`, which answers nothing as before. | `n67` 30/30 as above. `ptypull` (stdin a pty, SIGINT 0.5 s, `one` at 1.0 s): HEAD 30/30 `halted` / `v=[]` / `v=[one]`; oracle 30/30 `halted` / `v=[]` / `v=[]` (DEVIATIONS 10). `tgkill` to the reading pool thread, 10/10, and to `rexx-interp`, 10/10: both `halted` / `v=[]` / `v=[one]`. K8 (Mutations). |
| Same, inline (no pool reservation under `ulimit -v 1048576`) | `read_stdin_chunk` on `rexx-interp` | Retried; the read completes and the halt follows (DEVIATIONS 12). | `ptypull` with a pipe under the `ulimit`: 10/10 `v=[one]` / `halted` / `v=[two]`. `ulpool` shows the fallback is taken: a started activity runs only after the command under the `ulimit` (`main` / `child` / `main rc 0` / `w ran`), during it without. |
| Unredirected ADDRESS command, pool thread | `block.stream` inside `unblocked`: `read` in `read_all`, `read_to_end`, `Child::wait` | `read_all` retries (`command.rs:467-472`); std retries the others. | `k7live` (a child echoing `1`..`4` 0.3 s apart, PID SIGINT at 0.45 s, program alive 1.5 s more): HEAD 20/20 `1` `2` `rc N` `halted` `3` `4` `end`; oracle 20/20 the same (rc varies, entry 10). `tgkill` to the worker alone: 10/10 the same. K7 (Mutations). |
| Same, inline under the `ulimit` (P64) | `Block::wait` → `collect`: `read_to_end`, `Child::wait`, on `rexx-interp` and a scoped thread | Retried; uninterruptible, halt after (DEVIATIONS 12). | `k7` under the `ulimit`, 10/10: `1` `2` `3` `4` `rc 0` `halted`. |
| Redirected command (`WITH OUTPUT STEM`, `WITH INPUT`) | `collect` on `rexx-interp` with scoped threads (which inherit its unblocked mask): `write_all`, `read_to_end`, `Child::wait` | Retried. The oracle's EINTR is 98.923 here (DEVIATIONS 12). | `k7r`, 10 runs each: HEAD `rc 0 4 1 4` / `halted`; oracle rc 158, 98.923 "Address command redirection failed (Interrupted system call)". |
| `fork`/`exec` of a child | std's spawn (`posix_spawn` or fork with the CLOEXEC error pipe, whose read std retries) | Retried. | Code reading; every command probe above. |
| The queue (PUSH, QUEUE, PULL from the queue, QUEUED) | none: the session queue is in-process | n/a | No socket, `rxapi` or IPC call in the crates outside `sync.rs`'s timer socket. |
| SysSleep, GUARD WHEN, timers, `Sys*Sem` and object semaphore waits, `m~result` | parked in the scheduler; `rexx-interp` idles in `Inbox::idle` (`Condvar`, futex) | std's futex wait treats EINTR as a spurious wake and the loop rechecks. `Sys*Sem` are modelled in `semaphores.rs`; there is no `sem_wait`. | Code reading; the signals binary's sleep and semaphore witnesses. |
| Implemented RexxUtil file functions (SysFileTree, SysFileDelete, SysFileExists, SysIsFile, SysMkDir, SysRmDir) | `stat`, `unlink`, `mkdir`, `rmdir`, `readdir` | Not retried, but these do not return EINTR on a local filesystem; the oracle does not retry them either. The blocking RexxUtil functions (SysGetKey, SysWait, SysFileSearch, SysFileCopy, SysCreatePipe, SysFork) are Phase 10 stubs (`internal_routines.rs:103-164`). | Code reading. |
| SAY and other output | appended to a buffer; `rexx-run`'s sinks `write_all` before a stdin read, and main (blocked) writes the rest at the end | Retried; main never takes the signal. | Code reading (`lib.rs:2501`, `input.rs:395-407`, `bin/rexx-run.rs:49-57`, `:95-98`). |
| A native routine's own syscalls | on a pool thread (signals blocked, never unblocked around it) when one can be reserved, else on `rexx-interp` | On the pool it sees no EINTR, where the oracle's (on its Rexx thread) would. The same was true before this round, since main took the signal then. Info, no action. | Code reading (`scheduler.rs:625-640`). |
| The timer's wake socket read | `rexx-timer`, blocked | Never interrupted. | Code reading; the mask measurement. |

## Point 3: the readhalt rerun

**Verdict: a schedule the oracle takes too. Licensable under P48; not a divergence.**

The witness (`a_halt_during_a_read_reaches_an_activity_that_ran_meanwhile`) signals at 1 s
(`UNREADY_WAIT`) and writes `hi` 1 s later. The same program and timing through the driver,
under load, 30 runs each:

| | Output | Load average |
| --- | --- | --- |
| HEAD | 30/30 `w 3` / `wh 3` / `mh` / `main []` / `end` | 48-49 |
| Oracle | 29/30 the same; **1/30 `w 3` / `mh` / `main []` / `wh 3` / `end`** | 49-50 |

The oracle's odd run is exactly the order the fixer's rerun printed (`mh` / `main []` before
`wh 3`). On the oracle, main's EINTR and the busy activity's next clause boundary race for the
kernel lock; here, the slice can end between the halt being served and the activity's next
clause. Both are timing, and the oracle's own order varies. The witness's P48 rerun is the right
treatment. A comment saying the oracle takes both orders would keep the next reviewer from
re-asking; not required.

## Point 4: ADDRESS children's masks

**Verdict: as recorded. `SigBlk` is empty on both; `SigIgn` differs only by SIGPIPE
(DEVIATIONS 11).**

`sigblk.rex` greps `SigBlk`/`SigIgn` from `/proc/self/status` in a child started five ways:
unredirected (pool wait), redirected (`with output stem`), by a started activity, from inside a
`CALL ON HALT` handler after a SIGINT, and the unredirected one again under `ulimit -v 1048576`
(the inline fallback).

| | SigBlk | SigIgn |
| --- | --- | --- |
| HEAD, all five | `0000000000000000` | `0000000000000000` |
| Oracle, all four it runs (no fallback there) | `0000000000000000` | `0000000000001000` (SIGPIPE) |

Every child is spawned from `rexx-interp`, which is unblocked, and std empties the mask before
`exec` anyway. Entry 11 quotes `0x1004` / `0x4`; the extra SIGQUIT bit there is the measuring
shell's, absent under this driver's `env --default-signal`.

## Point 5: two interpreters per process

**Verdict: the race reopens with two interpreters, measured. A note for the API phase, recorded
as Low finding F-3: it is outside the P67 amendment's stated scope and reachable today only by a
Rust-API embedder that calls `install_signal_handlers` itself (`RexxCreateInterpreter` is Phase
9's).**

Embedder (`$CLAUDE_JOB_DIR/tmp/twointerp`, a scratch crate with a path dependency on
`rexx-exec`): main calls `install_signal_handlers`, starts thread B running `call SysSleep 3`
through `run_program`, waits 50 ms, then starts thread A running rr2's group program. Both
embedder threads inherit main's blocked mask, so the unblocked threads are the two
`rexx-interp`s and A's pool worker and stderr reader. Group SIGINT at 0.5 s, under load, 60 runs
per row, interleaved:

| | A's result | Load average |
| --- | --- | --- |
| Two interpreters | 55/60 4.1 at line 2; **5/60 `a` / `b -2`, 4.1 at line 4** | 50 |
| One interpreter (same binary, `ONE=1`: B runs `nop` and ends) | 60/60 4.1 at line 2 | 50 |
| Two interpreters, again | 54/60 line 2; **6/60 line 4** | 49-50 |

Mechanism: the kernel queues the signal for B's `rexx-interp` (the first unblocked thread after
the group leader); A's child dies of the same group signal, A's pool thread posts `Unblocked`, and
A's interpreter drains it before B's thread has run the handler. That is R-N4 with B's
interpreter thread in the joiner's old role.

Within one interpreter, a third unblocked thread exists too (a pool thread reading stdin for
another activity while main waits on a command). `groupread` (a started activity in `parse pull`,
main running rr2's commands, group SIGINT at 0.5 s, load 48-49): HEAD 120/120 4.1 at line 4 (the
command's line in that program), oracle 30/30 the same. The round robin picks `rexx-interp`
before the reader, so this did not reproduce.

Fix direction, for the record: after `unblocked` restores the block, the pool thread can ask
`sigpending()` whether a halting signal is still queued for the process (it is, until some
thread dequeues it) and post the halt with `Unblocked`. That narrows the window to dequeue-to-
handler; it does not close it. Where the API phase records it, it should also note that
`install_signal_handlers` blocks the signals in the thread that calls it, which for a C embedder
is its own thread.

## Point 6: regressions

**One regression: F-1 (High).** Otherwise none found.

- **P43, per-clause cost: none.** `bench-programs/callgrind.sh -r 1`, base against HEAD, libc
  subtracted: emptyloop, assign, dispatch, arith, compound +0.0000%; parse +0.0001%; sayloop
  +0.0007%; startup +0.0012%. The absolute difference is 584-834 Ir in every program, the same
  whatever the clause count: a per-process cost (the extra `pthread_sigmask` calls at thread
  starts), not a per-clause one. Each ADDRESS command now makes two `pthread_sigmask` calls on
  the pool thread, beside a `fork`/`exec`.
- **Timer and pool masks, and loom.** loom does not model masks, and does not need to: the
  property P67 relies on is the kernel's (a handler runs on the thread the signal is queued for,
  before that thread returns from its syscall), not a memory-model one. What loom does model is
  unchanged: the handler's effect (`PENDING` and the wake byte, `timer::signal()` in the model)
  and the timer's take. The changes are thread-local masks: the timer's `block()` is compiled out
  under `cfg(loom)` (`timer.rs:440`), and the pool's only changes the real test thread's mask in
  a loom build, which no model reads. `unblocked` restores the mask it found in a `Drop` guard,
  so a panic in the wait (`posting_panics` catches it) still leaves the pool thread blocked.
  Measured masks during a command: main and timer `0x4003`, `rexx-interp` and the waiting pool
  threads `0`; after the command, the pool worker is blocked again (the fixer's witness; the K3
  record). The one gap is F-4.
- **Halts are not doubled by the mask change.** Two `tgkill`s, one to each of the command's two
  pool threads, gave a second `halted` in 4 of 10 runs, but that is two signals, not one. One
  `tgkill` to the worker: 10/10 single halt.
- **Behaviour already licensed, re-seen, unchanged:** the redirected command's 98.923 on the
  oracle (entry 12); `n67`'s lost `one` on the oracle (entry 10); the started reader in
  `groupread` halted here and left hanging on the oracle (entry 10, P59/P60).

## New findings

### F-1 (High, regression from P66). A second activity touching the default input stream during a read ends the program with a loud refusal

`fill_stdin` now parks the reader in `pinned_wait(ParkReason::Input)` (`input.rs:368`), so the other activities run **on the reader's Rust stack**.
The reader reached the read through the `.INPUT` monitor's forward to `.stdin`'s guarded
method, so it holds `.stdin`'s guard while parked. Another activity that then touches the
default input stream (PULL, LINEIN, LINES, ...) goes the same way, finds the guard held, and
parks in a pinned guard wait on top. When the chunk arrives, the reader is readied but buried
below that wait, and the loop refuses. (Mechanism inferred from the refusal naming a guard and from the
`Monitor` UNKNOWN frame in the reader's traceback in `groupread`; not instrumented. The
observable is measured.)

Failure scenario (`mainreads.rex`), `one\ntwo\n` written at 1.0 s, 3 runs each:

```
o = .w~new; m = o~start('go')
parse pull v
say 'main [' || v || ']'
say m~result
exit
::class w
::method go
  call SysSleep 0.2
  return 'w lines' lines()
```

| | Result |
| --- | --- |
| HEAD | rc 120, stderr `rexx-exec: a pinned wait for a guard that only an activity pinned below it can end is not implemented`, no stdout |
| Base `bf76afa1d` | `main [one]` / `w lines 1`, rc 0 |
| Oracle | `main [one]` / `w lines 1`, rc 0 |

The same refusal, 3-5 runs each, HEAD only (base and oracle succeed): a started activity reads
and main then calls `lines()` (`pullstdin`); two activities each `parse pull` (`tworeaders`,
`tworeaders2`, `tworeaders3`; oracle `r [one]` / `main [two]`). Shapes that do **not** refuse:
the other activity only SAYs (the stamped probes), waits for the reader's result
(`readerresult`, `readersleep`), or calls a guarded method of a user object the reader holds
(`guardread`; presumably because that guard wait has no Rust frames under it). No test covers two activities on the default input
stream, so the suite stays green.

Fix directions: do not hold `.stdin`'s guard across the `Input` park (the oracle holds it, but
its waiter blocks on its own thread rather than on the reader's stack); or keep the old idle
when the read is reached through a guarded stream method with other activities able to touch
the same stream; or make that guard wait a non-pinned park. Witness: `mainreads` and
`pullstdin` above, as `signals.rs`-style or lib tests against the oracle bytes.

### F-2 (Medium, pre-existing, outside Task 21). Named FIFOs and terminals do not work through the stream class

`read_from` and `write_at` (`dispatch/stream.rs:684-697`, `:1314-1317`) seek before every read
and write. A FIFO or a terminal answers `ESPIPE`, so a read answers nothing at once and a write
fails `ERROR:29 Illegal seek`. An explicit `stream('f', 'c', 'open read')` on a FIFO is
Error 91.999 ("Message "COMMAND" did not return a result") on HEAD, `ERROR:2` then a working read
on the oracle (`fopen`). The oracle reads and writes both (table in point 2). None of this is in
DEVIATIONS (`phase-4-exclusions.txt` has no FIFO or `ESPIPE` row), and the diff does not touch
these functions, so it predates this round.

Failure scenario: `x = charin('f',,3)` with `f` a FIFO whose writer writes `abcdef` at 0.3 s:
HEAD `x=[]` `ERROR:0` immediately; oracle `x=[abc]` `READY:`. Consequence for this task: once
F-2 is fixed, `read_from`'s unretried `File::read` will meet EINTR on these streams. Answering
`ERROR:4` and letting the halt run is what the oracle does, so the fix should keep that rather
than add a retry; and a halt will not end such a read (DEVIATIONS 12) unless P60 is extended.

### F-3 (Low, note for the API phase). With two interpreters in one process a group Ctrl-C can halt the next command

Point 5: 11/120 under load with two interpreters, 0/60 with one, same embedder binary. The P67
amendment scopes the guarantee to one interpreter, so this is a record, not a blocker. It belongs
where Phase 9 will read it (its plan, or a DEVIATIONS row naming the rate and the
`sigpending` direction), together with the note that `install_signal_handlers` blocks the
halting signals in its caller's thread.

### F-4 (Info). A new thread is unblocked until its first instruction

The timer and pool threads block the signals as their first act (`timer.rs:441`,
`pool.rs:178`), but they are spawned from `rexx-interp`, which is unblocked, and inherit its mask.
Between the spawn and that call such a thread can be chosen for a signal, and its handler then
runs on a thread that neither waits on a child nor drains the inbox: the R-N4 shape, for the
length of a thread start. Not measured. Blocking in the spawner around `spawn` (and restoring
after) closes it. No action needed for approval.

## Mutations

`$CLAUDE_JOB_DIR/tmp/mut.py`: per mutant, copy the file aside, assert the site occurs once,
write the mutant, build `rexx-run` (copied aside as `bin/rexx-run.<name>` for probes), run
`cargo test --release -p rexx-exec --lib --test signals --no-fail-fast`, copy the file back and
`cmp`. `git status --short` after the set: only the lead's `progress.md`.

| Mutant | Claim it tests | Result |
| --- | --- | --- |
| MB: `make_ready` without the `set_aside` check (rr2's M-b) | R2 is witnessed | **red**: `a_halt_readies_a_set_aside_activity_once` (lib); signals 37/37 |
| MX6: the trap's own entry never delayed (`true => ANY`) | R1/R3's key choice holds a `CALL ON HALT` | **red**: 2 lib (`a_call_handler_reports_call_and_delay_and_leaves_nothing_behind`, `a_message_halt_inside_its_handler_is_dropped`), 2 signals (`a_second_signal_in_a_call_on_halt_handler_is_dropped`, `a_second_signal_ends_a_handlers_sleep`) |
| MX8: `fill_stdin` ignores `woken_by_halt` | a halted read answers nothing | **red**: 7 signals (every read witness, the readhalt one included); lib green |
| MX4: `Posted::Input` readies readers without taking them out of flight | P66's in-flight count | **red**: `a_halted_read_loses_no_input`; three more signals tests (`spin`, `readruns`, `masks` programs) hung until this reviewer killed their children at 7 min; lib green |
| MX9: `cancel_wait` leaves a reader in flight | the reader withdrawal on a failed wait | **survives**, lib and signals. Reaching it needs a reader's pinned wait to fail; the one failure found (F-1) ends the program, so the leaked count is not observable after it. Not a finding on its own. |
| K7: `read_all` ends at EINTR | the command pipe read retries | **survives** the suite, as the fixer recorded. The `tgkill` probe catches it: `k7live` with the signal to the command's worker, 5/5 `1` `2` `rc N` `halted` `end` (`3` `4` lost) against HEAD's 10/10 with them. |
| K8: `read_stdin_chunk` ends at EINTR | the stdin read retries | **survives** the suite. `ptypull` (stdin a pipe) with the signal to the reading pool thread: 5/5 `v=[]` twice and standard input closed before the write, against HEAD's 10/10 `v=[]` / `v=[one]`. |

K7 and K8 stay unwitnessed in `cargo test` because the kernel picks `rexx-interp` for a
process-directed signal (point 2). A `signals.rs` case could `tgkill` the pool thread by name,
as these probes do; not required for approval, since the probes are recorded.

## Checks (P51)

At `bfffdd33b`, after every mutant was restored. Target dir
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/p6-scratch/t21rr3/target`; each
status captured unpiped into a status file.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; the log has one
  `Checking rexx-exec` line.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, 143 result lines, 3029 passed, 0 failed, 4 ignored.
- Not run: loom, pinning clippy, loom clippy (not in this review's bar).
- `git status --short` at the end: only the lead's `progress.md`. The scratch target dirs
  (`target`, `target-base`, `target-two`) and the base tree are deleted.
