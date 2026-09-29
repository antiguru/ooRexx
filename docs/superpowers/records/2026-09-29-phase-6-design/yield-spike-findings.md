# Spike: when should a running activity yield? (Phase 6)

Tree `plan/rust-rewrite` at `0b698dff2`. Oracle 5.3 at `/home/moritz/dev/repos/ooRexx/build`, source in
`interpreter/` (this fork, #2074 `yieldRequested` fix applied). Probes ran from fresh `mktemp -d` dirs via
`probes/run.sh`, which wraps the standard `ulimit -v 1048576` oracle command in `timeout 20`.
Tags: Measured / Read / Inferred.

## 1. Oracle

### 1.1 Lock release sites (Read)
- `RexxActivation.cpp:616-635`: the check `++instructionCount > yieldInstructions(50)` runs every 51
  instructions. A pending `isYieldRequested()` leads to `relinquish`. Otherwise `relinquishIfNeeded`
  yields if `hasWaiters()` and either an API waiter exists or more than 24 ms have passed since
  `lastLockTime` (`ActivityManager.hpp:303-323, 359`).
- `RexxActivation.cpp:777`: REPLY, via `oldActivity->relinquish()`.
- `Activity.cpp:269/273/328`: activity end, pooling and exitCurrentThread. `Activity.cpp:1908/1962/2045/2058`:
  blocking waits (SafeLock/UnsafeBlock).
- `NativeActivation.cpp:1304/1420/1541/1691`: native code in "safe" mode.
- `Interpreter.cpp:336` and `InterpreterInstance.cpp:249/336/640`: instance lifecycle.
- `ActivityManager.cpp:220`: `addWaitingActivity(release=true)`, the handoff.
- Cross-activity requests: yield is an atomic flag (`Activity.cpp:2132`). HALT and external TRACE write
  into the target's frame (`Activity::halt` -> `RexxActivation::halt` sets `clauseBoundary`,
  `RexxActivation.cpp:4151`), and the target polls that flag every clause (`:651`). This is the frame
  poke that D3 forbids.

### 1.2 Section 9's question: settled (Measured)
The check period is 51 instructions. `interleave.rex` and `race3.rex` both have 3-instruction loops, and
51 = 17x3, so the yield always lands on the same clause boundary. For their start offsets, that boundary
is outside the detector's window. I added NOPs before the loop only, at 500k iterations for interleave and
the original 2M for race3:

| pre-loop NOPs | interleave switches (3 runs) | race3 final (expect 4000000) |
|---|---|---|
| 0 | 0,0,0 | 4000000 x3 |
| 1 | 6,4,4 | 4000000 x3 |
| 2 | 0,0,0 | **2000000, 2194206, 2184329** |

- A 4-instruction body (one NOP inside the loop) observed 1, 1 and 3 switches. `off1.rex`: 0.22 s wall at
  53% CPU, which is about 5 slices per activity.
- `ts.rex` (time('F')): the run intervals overlap. A runs 0-199 ms, B runs 50-198 ms.
- **Direction doc sections 1.2 and 1.3 record a phase-lock artifact.** The oracle preempts on a slice of
  about 24 ms, and it does not guarantee that a method runs to completion.

### 1.3 Behaviour that needs preemption or polling (Measured / Read)
- `probes/busywait.rex`: main does `~start`s a setter, then spins `do while \o~flag`. It terminates in 3/3
  runs after about 300k spins. Pure run-until-block would hang it.
- SIGINT (`timeout -s INT 1`) on `do forever; n=n+1; end` gives Error 4.1 HALT at `4 *-* end`. The Rust
  tree has no SIGINT handling (grep for SIGINT/sigaction/signal_hook/ctrlc under crates/*/src is empty).
- ooTest (Read): I listed the conditional loops in the 21 files that start activities. Every
  cross-activity wait sleeps or blocks (METHOD:342,710; ooTest.frm:2452; GUARD:240; socketClass:309), so
  none busy-waits. `Message.testGroup`'s `assertFalse(m~completed)` relies on a 2 s syssleep.
- Docs: `grep -i 'time.?slic|preempt|yield|relinquish' oodocs` finds nothing about scheduling.
  `xconcur.xml` says "Any number of objects can be active (running) at the same time."

## 2. Rust hot-path cost (Measured)

The per-activity mailbox is an `Arc<AtomicU32>` field on Interp. The poll is two loads, a test and a
branch to a cold function. Each variant was built into its own target dir, the binaries' md5s all
differ, and the patches are in `patches/`.

| Variant | What it adds |
|---|---|
| F | the field only (null control) |
| Ctl | a poll in the never-executed `Op::Queue` arm (layout control) |
| B50 | the existing clause countdown reloaded to 50 instead of `u32::MAX`, with an oracle-shaped check on the cold side |
| Bflag | a poll inside `count_clause_against_deadline`, i.e. on every clause |
| C | polls on Jump (both levels), LoopNext, the `run_loop_with_header` loop head, and `push_activation` |

Tool: `valgrind --tool=callgrind --cache-sim=no --branch-sim=no` (`cg/run.sh`). Programs were reduced:
emptyloop 2.5M, varlookup 1.9M, rexxcps averaging=10, dispatchclass 400k. Two rounds each, six for
dispatchclass. I excluded glibc (`cg/own.sh`), because `memcmp` alone spread by up to 9.6M between runs.
With it excluded, the per-variant spread is under 11k.

| | F | Ctl | B50 | Bflag | C |
|---|---|---|---|---|---|
| emptyloop | 0.000% | 0.000% | +0.27% (1.12 Ir/iter) | +1.43% (6.0) | +0.95% (4.0) |
| varlookup | 0.000% | 0.000% | +0.20% (1.68) | +1.42% (12.0) | +0.47% (4.0) |
| rexxcps | 0.000% | 0.000% | +0.06% | +0.55% | +0.02% |
| dispatchclass | 0.000% | 0.000% | +0.04% (1.68) | +0.19% (9.0) | +0.17% (8.0) |

- F and Ctl move each program by at most about 5k Ir, so every non-zero cell is real executed work.
- A B50 build whose countdown never reached zero was a third null control. Its results fall inside the
  libc noise.
- Bflag reproduces the memory note's cost for a per-clause branch.
- One poll costs about 4 Ir, but that is site-dependent. On an empty-body loop, Bflag's poll executed
  every pass and netted about 0.
- B50 costs about 28 Ir per cold visit and scales as 1/N. N=1024 would be about 20x cheaper (Inferred,
  not built).
- The counter covers every loop pass. `do i=1 to 1000000; end` costs B50 +572k Ir, which is 1M/50 visits
  x 28 Ir. The deadline tests already rely on the counter bounding `do forever`.
- Wall clock: 7 interleaved rounds on the full-size benches (`wall.txt`). Every variant, controls
  included, is within ±4% of base, so no wall difference can be attributed to any variant.

## 3. Policy for the isolation model

The model: one interpreter per OS thread, green activities, no lock, blocking calls offloaded to helper
threads.

- **Time slice.** `count_clause_against_deadline` already decrements `clause_countdown` on every clause,
  empty loop bodies included, and calls a `#[cold]` `countdown_reached` at zero. That adds no new hot-path
  branch.
  - While the interpreter has only one runnable green activity, keep the reload at `u32::MAX` (today's cost).
  - While more than one is runnable, reload to N. On the cold side, read a monotonic clock, and once the
    slice has expired (24 ms, as the oracle does) switch to the next runnable activity at this clause
    boundary. B50 measured +0.27% emptyloop, +0.20% varlookup, +0.06% rexxcps; N=1024 should be about
    20x cheaper (Inferred).
  - A switch happens only at a clause boundary, and never mid-clause. This matches the oracle, whose
    yield is also between instructions.
- **Blocking points.** A blocking call (SysSleep, stream or command I/O, `~result`/`~wait`, GUARD WHEN,
  semaphores) parks the green activity and hands the syscall to a helper thread. Its completion is posted
  to the interpreter thread's inbox, and the scheduler runs something else meanwhile.
  - GUARD WHEN needs no poll. The writer of a watched variable marks the waiters runnable, and they are in
    the same interpreter, so that is a plain call.
- **Cross-activity requests** (HALT, TRACE toggle, raise):
  - Within one interpreter, the request is a direct write to the target's own green-activity state
    (same thread). It is serviced at the target's next clause boundary through the existing
    `pending_traps` path. If the target is parked, the write also wakes it.
  - From another OS thread, the sender writes only to the target interpreter's atomic request word plus a
    locked queue for the payload. Sets are forced visible by a countdown reload to 1, done by the owner
    when it sees the word on its next cold visit. Latency is at most N clauses, or immediate if the
    interpreter is idle, since the inbox wakes it. The sender never touches the target's frames.
- **SIGINT.** The handler only sets the same atomic word, as a halt for every interpreter, and pokes the
  inboxes of idle ones. Each interpreter delivers HALT to its running green activity at the next cold
  visit, and to parked ones on wake. For prompt delivery, keep N bounded (e.g. 1024) whenever a handler
  is installed, even with one activity. At N=1024 that costs about 0.01%.
- **Long native builtins** (e.g. SysStemSort, a large `copies`): no clause runs inside them, so neither a
  counter nor back-edge polls can preempt them or halt them. There are three options:
  1. Accept it, as the oracle does: its time slice also stops at instruction boundaries.
  2. Chunk the native at internal loop checkpoints that consult the same atomic word.
  3. Offload the call to a helper thread like I/O. That only works if its arguments are immutable or
     copied, and isolation makes that cheap.

  I recommend (1) as the default, and (3) only for builtins measured above one slice.
- **Future GC.** A per-interpreter collector needs no cross-thread safepoint. The cold visit is still the
  place to run an incremental step. That is root-safe by Inference only; test it with a "collect at every
  countdown visit" stress mode.

## 4. Not established
- The N=1024 cost is extrapolated, not built.
- That collecting at `countdown_reached` is root-safe.
- The cause of the oracle's idle time at handoff (53% CPU).
- The ooTest scan is pattern-based.
