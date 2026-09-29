# Phase 6 spec review, round 3 (2026-09-29)

Reviewer: an Opus agent, read-only, against spec commit b8a03b317 (oracle not run; source settled
everything). Transcribed by the controller from the agent's messages. Tags: [M] measured, [R] read,
[I] inferred. Oracle paths under /home/moritz/dev/repos/ooRexx/interpreter.

## Round-2 closure
C-A partial: direction right, but owned `Rc` clones of island data are held across the C call
(`Rc::clone(&binding.library)`, `self.thread.clone()`, dispatch/library.rs:86, :94, :155, :165), so
2.5's SAFETY must cover every `Rc`/`Cell` value, not only borrows; and only natives reached from a
resumable op can exit (I-1). C-B partial (new defects C-1, I-2). I-A, I-B, I-C, I-F, m1, m2, m4-m8,
m10-m12 resolved (no ooTest file uses `Sys*Sem`, grep `Sys\w*Sem` over ootest/ empty [M]). I-D
partial (I-3). I-E mostly (I-6; the oracle reads only SIGHUP's old action before installing all
three, SystemInterpreter.cpp:136-139). I-G resolved wrongly (C-2). I-H mostly (m-5). I-I partial
(I-4). I-J partial (I-5). m3: DROP is not sticky (`drop()` notifies only when `dependents` is
non-empty, RexxVariable.cpp:161-164; `set()` whenever the table exists, RexxVariable.hpp:72-78). m9
partial (I-7). m10: the only non-test `thread_local!`s are REFUSED and HOOK_THREW; every other is
`#[cfg(test)]`, so S0's thread_local item converts nothing. Loose citations: `set_exposed_variable`
is at variables.rs:94; the destructure at lib.rs:2657; the clear at clause.rs:245.

## Critical
**C-1. A pinned park silently deadlocks against an activity returning from C.** A exits for a
native call; P takes the baton, is pinned, parks on A (`mA~result`, or a guard A holds across its
call); P's nested loop has nothing to run; A's thread requests the baton, but P's frames hold
protected `&mut`; the detector does not fire. Fix: a return from C or a blocking operation posts
its completion (raw handle and pending-condition number, no ObjRef) to the inbox and the baton
holder runs the finish half; only C callbacks need the baton; a pending callback request on the
wait chain counts as an inversion, refused loudly.

**C-2. `.context~thread` by current OS thread fails criterion-1 tests.** A park is a pointer swap on
the same thread, so REPLY continuations and started sends answer the parker's number;
RexxContext.testGroup:223-227 asserts `01 <> 13`, `01 <> 21`; TRACE_TraceObject.testGroup:324
asserts `setThread~items > 1`. Fix: number activities, assigned at creation from a free list released
at activity end (reproduces "m1 2 m2 2 main 1"; Activity.cpp:105-112).

## Important
- **I-1.** Most native and blocking calls cannot be driver exits today: `native_shape` rejects every
  message send and every call in argument position (ir/compile.rs:1212-1235), which become
  `Op::EvalExpr`; CALL, commands, ADDRESS, INTERPRET, GUARD, REPLY run through `Op::Exec`
  (compile.rs:1052-1078); blocking I/O sits in Stream natives via `send_message` (stream BIFs,
  builtin/stream.rs:48; PULL, input.rs:231; SAY to a routed `.output`, run.rs:1830; redirect,
  redirect.rs:376, :686); package loaders (install.rs:1993) and `Routine~call` of a library routine
  (dispatch/executable.rs:563) are always nested; the residual list omits tree-evaluated expressions
  and sends, trace/output delivery (route.rs:205), condition.rs, redirect. Fix: pinned = any Rust
  frame between scheduler and driver; S1 compiles sends and call/send arguments to ops; list the
  wrappers that keep the baton.
- **I-2.** Pinned slice expiry nests ready activities over a guard holder (TRACE.testGroup:752-766)
  and livelocks two pinned busy-waiters. Fix: defer SLICE at a pinned boundary to the next unpinned
  one (Lua, Loom); nest only for pinned parks; count deferrals.
- **I-3.** The continuation half of a parkable native is undefined when its caller is not the driver
  (`Message~result` from eval.rs:436, `dynamic_send` object_protocol.rs:807, `Message~send`;
  `linein` via BIF -> `send_message`). Fix: `Park` propagates through resumable entries; at the first
  non-resumable caller `send_message` converts it to a nested wait plus the continuation half; the
  park reason carries the wake outcome.
- **I-4.** Raw offsets exist today: `RootSet::aliases` holds absolute slot positions (roots.rs:67-71,
  :300-306, descending assert :334-337), `SlotFrame.start`, `frame_starts`, the `native_handles`
  index handed to rexx-api (library.rs:96). REPLY inside a non-flattened loop, INTERPRET or a CALL
  ON handler has Rust frames in the replier and cannot move: refuse loudly, counted.
- **I-5.** Per-group derivation sweeps in STREAM, TIME, Class, DateTime, MethodArgs, CALL,
  environmentEntries, extensions/rxsock/socketClass, samples; RexxQueue is Phase 10 (native.rs:
  260-269). Fix: derive per test method with committed exclusion reasons.
- **I-6.** A handler that only sets atomics cannot wake a condvar sleep, and the signal lands on an
  arbitrary thread. Fix: mask the signals except on one thread, or write to an eventfd/self-pipe.
  `libc` is only transitive: using it is a new direct dependency.
- **I-7.** D3 frame ownership (roadmap :2646): RexxContext and kept call contexts are GC-visible
  objects resolving into another activity's frames; no criterion verifies D3's frame ownership.

## Minor
- m-1. Guarded natives (Stream methods are guarded EXTERNALs, StreamClasses.orx:127-146) reserve on
  entry and keep it across a driver exit.
- m-2. The notifier yields mid-clause in the oracle (RexxVariable.cpp:192-193; REPLY :776); the spec
  switches at clause boundaries: say so and license it.
- m-3. HALT under nesting: an activity trapping HALT and parking again keeps enclosing pinned
  activities stuck; a targeted halt of an enclosing activity waits for the inner region.
- m-4. ~7 KB per pinned level (512 MiB / 72,821 sends): 9,999 levels needs ~70 MB per pool thread;
  accept a tiny bound or raise Error 11 on remaining stack as the oracle's `stackLimit` does.
- m-5. Budgets: increments vs running totals ambiguous; one noise band for the phase total; "the
  last three"/"the last two" are ambiguous set sizes; record the extension-call loop against the
  oracle too; S1 0.0% is plausibly unmet first time (spike +4.5% rexxcps, +10% fibcall).
- m-6. A same-thread request should lower the countdown to 1.
- m-7. Set sizes at :79 and :342.

## Criterion 9 estimate
Pinned parks in the criterion-1 groups are common unless I-1 lands: `self~assertEquals(x, m~result)`
(Message.testGroup :63-:841), `obj~start(...)~result` (Object :314), `r~call` (RexxContext :206),
`assertSame(0, sysSLEEP('1.1'))`, trace delivery to ArrayStream (route.rs:205), WHILE `t~items`.
With I-1 and I-2 fixed, what remains is mostly UNKNOWN, FORWARD, operators and trap handlers.

## Verdict
Not ready to plan from. Smallest set: C-1, C-2, I-1 to I-7 as above; Minor items may go to the
plan's first task.
