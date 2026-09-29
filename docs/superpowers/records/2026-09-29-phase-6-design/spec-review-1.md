# Phase 6 spec review, round 1 (2026-09-29)

Reviewer: an Opus agent, read-only, against spec commit 9385003fa. Transcribed by the controller
from the agent's three messages (its harness forbids writing report files). Tags: [M] measured,
[R] read in source, [I] inferred. Oracle paths are under /home/moritz/dev/repos/ooRexx/interpreter.

## Critical

**C1. The oracle runs every native call off its kernel lock; the spec keeps native code on the
baton** (P6-3; 2.1; 2.3; section 4). [R] NativeActivation.cpp:1303-1306 (methods), :1420-1423
(routines), :1541-1544 (classic functions), :1690-1692 (exits) call releaseAccess before the C
code and requestAccess after; every API callback reacquires through ApiContext
(api/ContextApi.hpp:64-77). SysSleep is a native RexxRoutine (runtime/RexxUtilCommon.cpp:1891),
which is how it lets other activities run; SysStemSort likewise runs off the lock, so section 4's
"cannot be halted on the oracle either" is wrong. Under the spec an extension blocking in its own C
code (rxsock accept/recv, a GUI loop) stalls every activity. Fix: release the baton on entry to
extension C code (Go's syscall handoff), take it back on each callback and on return, let the timer
or inbox hand a free baton to a driver thread; restate the invariant as "one activity touches
interpreter state".

**C2. The native-API thread context is per interpreter and handed to C, so the Send grant's SAFETY
argument ("no piece leaves") is false.** [R] One ThreadContext per Interp (rexx-exec/src/lib.rs:1314,
built :2056, kept by libraries install.rs:1988); it records `home` (rexx-api/src/ffi.rs:514, :545),
asserted by AttachThread and AddCommandEnvironment (:3490, :3532); it holds an `innermost`
activation cell saved/restored in strict nesting (:508-525). [I] Carriers interleave native calls of
different activities, breaking the nesting (A's callbacks reach B's activation); migrating the
driver trips the `home` assert. [R] Only those two callbacks check the thread; any other callback
from an extension thread is a data race on the Cells. The oracle validates the thread on every
callback (ContextApi.hpp:76 -> Activity::validateThread, Activity.cpp:3620-3626) and its contexts
are per activity (ActivationApiContexts.hpp). Foreign threads join via AttachThread, making a new
activity (Phase 9 today, but the seam must allow it). [I] Frames on a blocked carrier hold live
`&mut Interp` borrows; the next holder must derive from the innermost reborrow, so the SAFETY
argument is a lending protocol. Fix: a thread context per activity, a per-callback check that the
caller holds the baton for that activity, SAFETY rewritten as the lending protocol.

Cleared: [M] nothing sends an ObjRef (a scratch `ObjRef(u64, PhantomData<*const ()>)` passes
`cargo check --workspace --all-targets --offline`); [R/I] pinned stacks are root-safe at a handoff
(nested Rexx allocates, so live Rust locals are already rooted), except the per-execution state of
I9; [I] "exactly one runs" holds by baton exclusivity once C1 and C2 are fixed.

## Important

- **I1.** The invariant "exactly one activity runs per interpreter" is what roadmap D3 (:183)
  forbids; ObjRef !Send makes it permanent. Record a ruling narrowing D3 to process-wide (PEP 684).
  Section 4's "write the target activity's state directly" contradicts D3 :189 (channel or atomic).
- **I2.** `.environment` is process-global in the oracle (RexxCore.h:267, Setup.cpp:300); only
  `.local` is per instance (InterpreterInstance.cpp:208). License the Phase 9 divergence or
  constrain Phase 9.
- **I3.** Every ooTest body runs pinned: OOREXXUNIT.CLS (~1583) runs a test by
  `.message~new(self,name)~send` -> dispatch_held_message (object_protocol.rs:893), not resumable in
  2.1. S2/S3 depend on S4. Also unaddressed: loop headers, non-flattened loops, INTERPRET,
  nested-call arguments. Fix: make Message~send/sendWith and Object~send/sendWith resumable in S1,
  or move carriers earlier.
- **I4.** REPLY and GUARD WHEN release one guard nesting level: VariableDictionary::transfer
  (:600-617) moves the lock only at reserveCount 1, else the continuation reserves again
  (RexxActivation.cpp:766-773, :561-568); guardWait releases one level (:3350-3369);
  GUARD.testGroup:258-279, :290-330 depend on it. REPLY relinquishes (RexxActivation.cpp:776), so
  the continuation usually runs first.
- **I5.** WHEN watches named variables (GuardInstruction.cpp:141-144); set and DROP notify
  (RexxVariable.hpp:71-77, RexxVariable.cpp:158-169); exposed-stem compound elements are watchable
  (ExpressionCompoundVariable.cpp:348) and are stored via the stem, not the pool
  (this crate: set_exposed_variable, variables.rs:103-125); attribute setters and SetObjectVariable
  (dispatch/library.rs:718) need the barrier; each re-evaluation is traced
  (GuardInstruction.cpp:176,183); notify yields (RexxVariable.cpp:192-193).
- **I6.** Unix Sys*Sem are not the classes: Sys*MutexSem is a counting POSIX semaphore, no owner,
  no re-entry, timeout 0 = forever, rc 121/6 (platform/unix/SysRexxUtil.cpp:868-903, :945-1025);
  named ones use sem_open (:668), not rxapi. MutexSemaphore's owner is the activity, released at
  activity end (Activity.cpp:303; MutexSemaphore.cpp:230-262); after REPLY it stays with the caller.
- **I7.** Alarm and Ticker are CoreClasses.orx code (~1560-1575, ~1660-1672): REPLY then native
  !startTimer/!waitTimer (deferred at dispatch/native.rs:116-120). The job is timed parks plus
  early wake, not heap entries; firing runs on the replied activity; program end waits for it.
- **I8.** REFUSED (layout.rs:423) and HOOK_THREW (load.rs:678) are per native call stack and must
  stay per OS thread; land the Send grant with its first user (S4).
- **I9.** Per-execution Interp fields must be per activity: value_buffer (cleared per clause,
  clause.rs:242), running/suspended, clause_state, frames, flat_top, pending_traps,
  active_condition, fragments, depth, call_context, indents, random_seed (per activity in the
  oracle, Activity.hpp:426). Classify by the exhaustive destructure (lib.rs ~2682).
- **I10.** One heap mixing runnables and not-yet-due sleepers makes dispatch undefined: use a ready
  queue plus a sleeper queue.
- **I11.** Latency claim fails with one activity and no handler (any embedding): reload u32::MAX
  (clause.rs:283-286); clause_countdown is a plain field other threads cannot lower.
- **I12.** SIGINT: rustix's kernel_sigaction is experimental (rustix-1.1.4/src/runtime.rs:1-30,
  491-503): new dependency or unsafe site needed; the registry must be async-signal-safe. The oracle
  installs at interpreter start, halts on SIGTERM and SIGHUP too, halts only active activities
  without waking waiters (SystemInterpreter.cpp:95-110, 130-145; InterpreterInstance.cpp:686-703).
  Spec section 4 contradicts itself on waking parked activities.
- **I13.** Thread identity and guard state are observable: .context~thread answers constant 1
  (dispatch/context.rs:251-259); TraceObjects are fixed (route.rs:216-221); the oracle sets THREAD,
  CALLERSTACKFRAME, ISGUARDED, SCOPELOCKCOUNT, HASSCOPELOCK, ISWAITING (RexxActivation.cpp:5160-5230).
  Exit criterion 1 misses TRACE_TraceObject.testGroup:306-324, RexxContext.testGroup:200-284,
  TRACE.testGroup:750-764, RAISE.testGroup:470-487, Object.testGroup:271-284. Missing:
  Message~reply/replyWith (MessageClass.cpp:562-640); ~notify is synchronous (:209-226, :660-672).
- **I14.** Perf gates have no numbers: no reference, tolerance or stopping rule for "parity"; S0
  has no call-heavy program.
- **I15.** FrameArena is strict LIFO with ~1 MiB blocks (frame.rs:43,53,70,118): one per activity
  allows ~900 activities under the 1 GiB ulimit; a shared arena breaks LIFO; frame.rs invariants
  need a re-grant.
- **I16.** Nightly TSan contradicts roadmap :25 and D3 :191 (loom fallback) without a recorded
  ruling; needs a network toolchain install and build-std.

## Minor

- **M1.** countdown_reached does run today, every 1024 clauses when a deadline is set (clause.rs:31,
  :296; lib.rs:3263); the request word shares that path.
- **M2.** FIFO holds only for scope reservations (VariableDictionary.cpp:547-553, :585-590); WHEN
  watchers wake in identity-table order; Message waiters all wake at once (MessageClass.cpp:660-672).
- **M3.** Set sizes stated ("six gates", "Two are added").
- **M4.** The Error 11 depth cap stays for parity.
- **M5.** The deterministic switch mode does not make time virtual.
- **M6.** The rule is "no mutable process-global state"; run/loops.rs:1303 has a SPIKE-marked
  `static FLAT: OnceLock` read from an environment variable.
- **M7.** Resume lands mid-region, not at a clause boundary.
- **M8.** NUMERIC INHERIT does not cross ~start (RexxCode.cpp:207, RexxActivation.cpp:152-158); a
  method's ADDRESS comes from the instance default (:171).
- **M9.** Which activity runs UNINIT: the oracle checks at activity end (Activity.cpp:303) and at
  terminate (InterpreterInstance.cpp:571).

## Verify items settled

- **V1** [R] Guard deadlock detection exists: Activity::checkDeadLock raises 98.905 (Activity.cpp:
  2000-2027; RexxErrorCodes.h:606), following the wait chain, on a contested scope reserve
  (VariableDictionary.cpp:545) and Message~wait/~result (MessageClass.cpp:249-253); not on GUARD
  WHEN waits (Activity.cpp:2050-2057) or unsent messages.
- **V2** [M] An untrapped error in a started message prints its traceback at once on the started
  activity; hasError is 1; ~result re-raises (MessageDispatcher.cpp:64-72). A REPLY continuation's
  untrapped error shows through .traceOutput (RAISE.testGroup:470-487).
- **V3** [M] Program end waits for every non-pooled activity, even never-ending ones
  (InterpreterInstance::terminate :521-573), then collectAndUninit; pooling :395-416; rc is main's.
- **V4** [R] Guard re-entry: same activity stack, including nested attaches on one thread
  (VariableDictionary.cpp:527-533); REPLY and WHEN release one level.
- **V5** [R] WHEN re-evaluation: set or DROP of a named variable, compound elements included;
  notify yields (RexxVariable.cpp:120-194; GuardInstruction.cpp:141-190).
- **V6** [R] MutexSemaphore owner is the activity; released at activity end.
- **V7** [R] Alarm and Ticker: CoreClasses.orx REPLY plus native timers.

Confirmed: 99.913 (InstructionParser.cpp:2640-2646); the unsafe grant list matches the
`#[allow(unsafe_code)]` sites [M]; the yield-spike figures are quoted correctly.
