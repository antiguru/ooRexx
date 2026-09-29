# Phase 6 spec review, round 4 (2026-09-29)

Reviewer: an Opus agent, read-only, against spec commit 80983923e. Transcribed by the controller
from the agent's messages. Tags: [M] measured, [R] read, [I] inferred. Oracle paths under
/home/moritz/dev/repos/ooRexx/interpreter.

## Round-3 closure
C-1 partial: the drainer runs another activity, so a finish half run there reads the wrong
activity's state (`trap_for`, `pending_traps`, `activation().id`, the innermost native frame;
dispatch/library.rs:365-437); callbacks before return are the ordinary case (I-3). C-2 resolved for
the tests (every ootest thread assertion is relational [M]) but "reproduces the oracle's numbering" is
false (m-1). I-1 partial (plain DO, GUARD/REPLY in `Op::Exec`, S1's leaf kinds). I-2 resolved (new
residual m-7). I-3 resolved for natives only (C-1). I-4, I-5 resolved. I-6 resolved in direction
(m-2). I-7 partial (I-5). m-1 to m-6 resolved; m-4 residual m-8; m-7 partial (m-9). Round-2 C-A:
2.5 omits raw pointers into island memory (I-2). Citations spot-checked and landing [R]; the only
non-test `thread_local!`s are REFUSED and HOOK_THREW [M]; route.rs:205 is TraceObject construction.

## Critical
**C-1. Only natives can park; instructions cannot.** GUARD, REPLY, FORWARD and some CALLs compile
to `Op::Exec` (compile.rs:1052-1078) and run inside `exec_instruction` -> `exec_guard` (run.rs:1114),
so every GUARD WHEN wait and contended GUARD ON is a pinned park and every REPLY is immovable.
GUARD.testGroup `test_wait_multiple` (:287-330) inverts deterministically: main pinned-parks on
WHEN; W runs nested, sets var1, then pinned-parks on its own WHEN (:326) above main. Fix: park points
for the `Op::Exec` arms (GUARD contended ON and WHEN return Park; REPLY returns Split) and for the
guarded-method reservation in the send's begin half.

**C-2. Plain `DO; ... END` runs nested.** `flat_loop_start` answers Fallback for `LoopKind::Simple`
(run/loops.rs:1356) and the driver calls `run_loop_with_header` with `BodyEngine::Chunk`
(ir/drive.rs:1900-1915) [R]. Waits inside `if ... then do ... end` pin; REPLY there is immovable. Fix:
S1 flattens Simple DO, and the spec states the status of the flat-loop path marked "SPIKE, not for
commit" (loops.rs:1284, compile.rs:1038).

## Important
- **I-1.** A raw handle is not enough: the finish half runs `values::from_native`
  (invoke.rs:218-224) and needs the declared result type, `Written`, the refused slot (thread-local
  `REFUSED`, captured on the calling thread) and `pending` in the rexx-api `Activation`
  (values.rs:749-802); a returned handle resolves only in that call's native-frame `locals`
  (handles.rs:43-52), so the frame stays pushed and rooted until finish pops it. `Activation` holds
  `Conversion { host: &mut dyn Host }` across the C call (values.rs:716-719, library.rs:96-106):
  `invoke::run` must split into prepare, call, finish. Fix: the drain moves the completion into the
  activity's parked record; finish runs as that activity's first step on resume.
- **I-2.** Island memory reached by C through raw pointers while off the baton: kept C strings
  (`kept_strings`, library.rs:984-1010, pruned by `drop_loose_kept_strings`); `BufferData` and
  `MutableBufferData` pointing into a `Vec` in `BufferState` (library/surface.rs:212-237). Another
  activity may reallocate it while C writes. Fix: enumerate exported memory, rule per kind.
- **I-3.** A nested loop can serve a callback by delegation: the callback posts the API member to the
  inbox and blocks; the holder runs it as the callback's activity and posts the answer back (sound: it
  reborrows its own `&mut`); oracle API entries take the kernel lock per call (ContextApi.hpp:64-77).
  Hybrid: take the baton when the holder is unpinned, delegate when pinned.
- **I-4.** S1 send compilation: `native_shape` must accept DotVariable, List and other leaves;
  `NodePath` is binary (ir.rs:309-319) and arguments compile with `path: None` (compile.rs:918,
  :1417), so per-site addressing is needed. Risk: spike +520/+585 Ir per send stackless, +7.8%
  sendloop, `Op::Send` alone -157; `Op::Send` exists only in stackless-spike.patch:551.
- **I-5.** No baton-token type exists; contexts resolve by scanning `self.frames()`
  (dispatch/context.rs:136-148), which per activity would answer 98.981 for a live foreign activation
  the oracle reads (`checkValid` tests only null, ContextClass.cpp:145-151). `frame_at` is
  `pub(crate)` returning `&Activation`. Real fact: `RegFrame<'a>` borrows the arena and heap `Body` is
  `'static`, so no GC object can hold a frame.

## Minor
- m-1 [M] lowest-free numbering diverges: the oracle's pool is FIFO (`availableActivities`,
  ActivityManager.cpp:556, :650-662) bounded by MAX_THREAD_POOL_SIZE (ActivityManager.hpp:358);
  B (0.3 s) and C (0.1 s) concurrent, then D: oracle `main 1 D 3 B 2 C 3` (5/5); main is 1 even when
  a started thread asks first.
- m-2 self-pipe via `UnixStream::pair()` (CLOEXEC) with `set_read_timeout`; arm through the same
  socket; nonblocking write end; save and restore errno. Bar 1 needs the tried-and-failed safe
  alternative recorded (signal-hook chains to prior handlers rather than skipping). The oracle
  ignores SIGPIPE (SystemInterpreter.cpp:146-148), not covered for an embedded library.
- m-3 under `nohup` the oracle installs none of the three handlers (it reads only SIGHUP's action);
  this design halts on SIGINT there: license.
- m-4 a baton request arriving while pinned is deferred; P6-3 should count a callback's return.
- m-5 SysStemSort off the baton sorts island data: copy out and back, or stay on the baton.
- m-6 define "the thread of an activity" as the thread running its native call.
- m-7 a single pinned busy-waiter never yields (no criterion-1 group has one [M]): record.
- m-8 activities migrate threads: thread-affine extensions differ; main may run on a small-stack pool
  thread, so Error 11 depth depends on scheduling: record.
- m-9 set sizes at :13, :228; `Op::Send` cited as existing; route.rs:205 mislabelled.
- m-10 RexxContext.testGroup:236 asserts the invocation number survives REPLY: the moved activation
  keeps `invocation` (context.rs:152-160).

## Criterion 9 and verdict
Achievable only after C-1 and C-2; then remaining pinned kinds rarely park in the criterion-1 groups
(the ooTest ticker parks unpinned once SysSleep in a flat loop is a parkable native). It fails on two
activities pinned-parking with cross dependencies, or a pinned wait on an activity whose native calls
back (I-3). Measure pinned parks early.

Not ready. Must change in the spec: C-1, C-2, I-1, I-2, I-3, I-5. Can go to the plan's first tasks:
I-4's addressing and leaf ops, m-1 to m-10, an early pinned-park instrument run before S1 closes.
