# Phase 6 spec review, round 2 (2026-09-29)

Reviewer: an Opus agent, read-only, against spec commit 559a3e0c5. Transcribed by the controller
from the agent's messages. Tags: [M] measured, [R] read, [I] inferred. Oracle paths under
/home/moritz/dev/repos/ooRexx/interpreter.

## Round-1 closure
C1 partial (direction right; 2.5's mechanism deadlocks, C-A; SysSleep is a Rust builtin calling
`std::thread::sleep` under `&mut Interp`, rexx-exec/src/builtin/rexxutil.rs:161-184). C2 partial
(per-activity contexts right; lending fails liveness; `native_handles` missed, I-F). I1, I2, I4, I5,
I8, I10, I11 resolved (I2 not in the roadmap, m9; I11 "inbox" means always, m5). I3 partial (C-B).
I6 resolved on semantics, staging broken (I-C). I7 resolved, needs a parking-native protocol (I-D).
I9 mostly (I-F). I12 partially wrong (I-E). I13 partial (I-G). I14 partial (I-H). I15 partial (m7).
I16 resolved by R4 (Global Constraints line not amended, m9).

## Critical
**C-A. The lending protocol makes native returns LIFO across threads and deadlocks programs the
oracle runs** (2.5; P6-3). [R/I] The `&mut Interp` lives in the native activation for the whole C
call (`Conversion { host: &'a mut dyn Host }`, rexx-api/src/values.rs:717, held in `Activation`
:749), and `&mut self` function arguments carry protectors under Stacked and Tree Borrows, so a
thread returning from C must wait until every later baton holder has fully unwound. Deadlock: main
`call SysSleep 1` releases; a started activity on T2 blocks in `SysWaitEventSem`; main wakes but is
not innermost and never reaches its `SysPostEventSem`. Fix: hold no borrow of interpreter state
across a baton release. A native call from a non-pinned point becomes a driver exit (like a park);
the C call runs holding only a raw root pointer; `Host` becomes a baton-guarded accessor whose
callbacks acquire the baton, derive a fresh `&mut` from the root pointer and drop it before returning
to C, so returns and callbacks may occur in any order. A native call from a pinned point keeps the
baton (a documented divergence, counted). Blocking Rust builtins split into prepare / block with no
borrow / finish. State the rule: nothing borrowed from `Interp`, the arenas or a `RefCell` is live
across a release.

**C-B. A pinned activity that parks or whose slice expires has no mechanism in S2/S3, and in S4
every such event deadlocks as in C-A** (P6-4; 2.1; section 8). [R/I] Pinned parks are common (a
guarded method via UNKNOWN or FORWARD, INTERPRET, a guard contended in a sort comparator, `m~result`
on a residual path); a busy-wait in a pinned loop livelocks. A started activity's first send
(dispatch/object_protocol.rs:824 `started_message` -> `send_message`) and the REPLY resume
(dispatch.rs:2528 `resume_reply` -> `run_activation`) are recursive and absent from 2.1, so started
activities are pinned from birth. Fix: add both to S1; specify S2/S3 behaviour for pinned parks and
slices (refuse loudly with a counted row, or LIFO nesting with a deadlock detector).

## Important
- **I-A.** Pool threads vs the stack reservation: `INTERPRETER_STACK_BYTES = 512 MiB`
  (lib.rs:183, :3180); rexx-run is tested under `ulimit -v 1048576`
  (tests/library_routine_memory.rs:66) and reservations count against it (rust/CLAUDE.md:92), so a
  second thread of that size fails (EAGAIN). No stack size, pool bound or spawn-failure fallback.
  The oracle's threads are 512 KiB (common/platform/unix/SysThread.hpp:67).
- **I-B.** Baton requests starve behind a compute-bound holder; the oracle prioritises callbacks
  (`requestApiAccess` -> `addWaitingApiActivity`, concurrency/Activity.cpp:2235-2249, :339;
  `relinquishIfNeeded` yields at once, ActivityManager.hpp:303-323). Fix: a baton request sets the
  request word and arms the slice; a callback forces a switch at the next cold visit.
- **I-C.** GUARD.testGroup:124-159, :256 and Message.testGroup:651-688 need `call SysSleep` to let
  another activity run; unnamed `Sys*Sem` waits holding the baton deadlock. Fix: SysSleep, timers and
  unnamed `Sys*Sem` waits are scheduler parks from S2/S3; list what still holds the baton.
- **I-D.** No protocol for a Rust-implemented native to park (`Message~result`/`~wait`,
  `MutexSemaphore~request`, `EventSemaphore~wait`, `!waitTimer`; every ooTest assertion runs
  `m~result`). Needs a `Park(reason)` outcome and a re-entry delivering the result register.
- **I-E.** The oracle halts every `isActive()` activity (nestedCount > 0,
  InterpreterInstance.cpp:686-703; Activity.hpp:249), waiting and preempted included; SIGINT
  interrupts `nanosleep` (no SA_RESTART, SystemInterpreter.cpp:127; SysThread.cpp:191-201).
  [M] `rc = SysSleep(5)` under `timeout -s INT 1` halts at about 1 s with Error 4.1. The oracle
  installs handlers from the library, only when none is set (SystemInterpreter.cpp:130-145).
- **I-F.** `native_handles` (lib.rs:1273-1280), pushed at dispatch/library.rs:475, is a
  per-interpreter LIFO; classify per activity.
- **I-G.** `.context~thread` numbers OS threads (`getIdntfr`, Activity.cpp:105-112) and pooled
  threads are reused: [M] "m1 2 m2 2 main 1", three runs of three.
- **I-H.** The performance rule is not a stopping rule: "round" undefined; one layout control has
  no spread (spike's was +0.77% on rexxcps); no budget or exit for S2-S4; callgrind counts a locked
  atomic as one Ir and no program exercises the per-call baton: add an extension-call loop and a
  wall-clock check.
- **I-I.** REPLY on a LIFO arena: the replying frame moves arenas; offset handles held by
  RexxContext/StackFrame objects and kept call contexts need a rewrite rule.
- **I-J.** The criterion-1 group list is not derived; REPLY/`~start` also appear in
  base/directives/METHOD (:708, :721), base/special.variables/RESULT_RC_SIGL (:188),
  doc/rexxref/chapter5/Section1 (:75-98), base/class/RexxQueue (:141, :261).

## Minor
- m1 citations: Activity.cpp:303 is `clearLocalReferences`'s doc; mutex release is
  `cleanupMutexes` at :258 and :292, uninit check :249 and :321-324; `classes/MutexSemaphore.cpp`;
  `base/keyword/TRACE_TraceObject`; `environment/route.rs`.
- m2 WHEN wake order is OS scheduling plus the kernel-lock queue (RexxVariable.cpp:176-193), not a
  reproducible table order: specify FIFO and license.
- m3 a once-watched variable keeps `dependents` (RexxVariable.cpp:137-150) and every later set
  notifies and yields: the watched flag must be sticky or the divergence recorded.
- m4 wrong-thread callbacks raise Error_Execution_invalid_thread after taking the lock
  (ContextApi.hpp:73-76).
- m5 "whenever the interpreter has an inbox" means always.
- m6 foreign threads must not write arena-resident request words; route through the inbox.
- m7 every FrameArena block carries GUARD = 65,536 cells (frame.rs:43, :69): at least ~512 KiB per
  activity whatever the starting block.
- m8 write the baton and inbox against a `cfg(loom)` shim so loom checks the shipped code.
- m9 roadmap note lacks R2 and R3; the Global Constraints require a Section 1 decision block for a
  new unsafe module before code; 2.5's unsafe list omits R3's module; row 6 still says "kernel
  lock"; roadmap :2646 "verify by construction ... rather than by test" vs criterion 4 "a test".
- m10 "Two `thread_local!`s" states a set size.
- m11 the loading activity's context and the per-interpreter `RexxInstance` outlive the activity
  (install.rs:1988, load.rs:809).
- m12 staging after an S1 fallback is unspecified.

Settled without change: NativeActivation and ContextApi citations; `home` asserted only at
ffi.rs:3490, :3532; countdown lines; Sys*Sem rc 121/6 and sem_open :668; MessageClass notify and
wake-all.
