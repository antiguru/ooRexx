# Phase 6: concurrency, as isolated interpreters with continuation activities

**Status:** design, agreed with Moritz in conversation on 2026-09-29, section by section, then
revised after review round 1 (`docs/superpowers/records/2026-09-29-phase-6-design/spec-review-1.md`)
with his rulings R1 to R4 (section 1.1). It replaces the delivery order of the roadmap's D3
option (a), a coarse process-wide kernel lock. Roadmap row 6
(`docs/superpowers/plans/2026-07-27-rust-rewrite.md`) remains the exit contract, read through this
document.

**Inputs.** `docs/superpowers/specs/2026-09-08-concurrency-research-direction.md` (the research
direction); the records in `docs/superpowers/records/2026-09-29-phase-6-design/`:
`yield-spike-findings.md` (when to switch, and what it costs), `stackless-spike-findings.md` with
`stackless-spike.patch` and `stackless-reentry-sites.txt` (whether Rexx-to-Rexx calls can run
without native recursion), and `spec-review-1.md`. Oracle source paths below are relative to
`/home/moritz/dev/repos/ooRexx/interpreter`.

---

## 1. Decisions

| # | Decision | Why |
|---|---|---|
| P6-1 | **Isolation.** One interpreter owns its heap, collector, classes, `.environment`, `.local` and scheduler. Interpreters share nothing. | No process-wide lock, by construction. The field's shared-heap runtimes paid with a per-operation shared structure (research direction, section 5). |
| P6-2 | **Activities are continuations.** An activity's state is arena data, run by a driver loop; Rexx-to-Rexx calls swap frames inside that loop. | Switching on a slice or a Rexx-level wait is a pointer swap; REPLY is a continuation split. Precedents: CPython 3.11 (Python-to-Python calls off the C stack), Lua. |
| P6-3 | **The baton, released wherever the oracle releases its kernel lock.** An interpreter has one baton; holding it is the right to touch interpreter state. It is released around every call into extension C code and every blocking operation, and taken back on each API callback and on return. A spare driver thread takes a free baton when runnable activities exist. | The oracle runs every native call and every blocking wait off its kernel lock (`execution/NativeActivation.cpp:1303-1306`, `:1420-1423`, `:1541-1544`, `:1690-1692`; `api/ContextApi.hpp:64-77`). This also makes a separate offload layer for blocking calls unnecessary: a blocking call releases the baton and blocks on its own thread. The Go runtime's processor handoff around a system call is the same shape. |
| P6-4 | **Pinning.** An activity with native frames on its stack (a native call in progress, or native code re-entering Rexx) is pinned to the OS thread those frames live on. It cannot be switched as a continuation; it gives up the baton instead, as P6-3 describes. | The Loom shape: virtual threads, pinned to a carrier when native frames are present. Every native re-entry need not be resumable before activities work; each one made resumable is a measured win. |
| P6-5 | **A scheduling seam.** Nothing outside the scheduler knows how an activity is carried. | Keeps the carrier mechanism replaceable on measurement ("don't lock us into a specific design"). |
| P6-6 | **An RBED-shaped scheduler shipping round-robin.** Dispatch is separate from allocation (what position an activity gets); Phase 6 ships the uniform policy only. | Uniform allocation with arrival-order tie-breaks is round-robin, which is what parity needs; rates, priorities or budgets become a policy swap. |
| P6-7 | **The time slice comes from the existing clause countdown; the slice is a request bit set by a timer thread.** | No new hot-path branch. One mechanism for slice, HALT, signals and cross-thread requests; tests set the bit themselves. |
| P6-8 | **Out of scope:** the Rexx-level API for user-managed interpreters, and exchanging data between interpreters over queues. | A later phase, designed with the rxapi integration (Moritz, 2026-09-29). Section 10 records its starting precedents. |

### 1.1 Rulings recorded with this design (Moritz, 2026-09-29)

* **R1. D3 is narrowed to process-wide.** D3's "no invariant may rest on only one activity runs at a
  time" is read as process-wide: per-interpreter serialization is allowed, as with PEP 684's
  per-interpreter lock, and parallelism comes from separate interpreters. The roadmap's D3 gains a
  dated note saying so. D3's other rule stands unchanged: cross-activity requests go through a
  channel or an atomic flag the target polls, never a foreign frame pointer, and that holds inside
  one interpreter too (section 4).
* **R2. `.environment` per interpreter is a licensed divergence for Phase 9.** The oracle's
  `.environment` is process-wide (`RexxCore.h:267`, `Setup.cpp:300`) and only `.local` is per
  instance (`runtime/InterpreterInstance.cpp:208`). A single-interpreter program cannot observe the
  difference; multi-instance embedding (Phase 9) records it as licensed.
* **R3. Signal handling is a new `unsafe` site, not a new dependency.** One module installs the
  handlers through the libc already linked; the handler only sets atomics in an async-signal-safe
  registry. The grant is recorded with the others.
* **R4. Race checking** is a ThreadSanitizer run with the installed nightly toolchain (gate-only,
  never for shipped builds) plus a `loom` model of the baton, inbox and timer protocol. Moritz
  expressed no preference among nightly, `loom` and `shuttle`; the choice is on merit: TSan sees
  data races in the real binary, `loom` exhaustively finds the lost-wakeup and handoff bugs a baton
  protocol invites, which TSan cannot, and the deterministic switch mode (section 4) already covers
  interpreter-level interleavings, which is shuttle's niche. `loom` is a new dev-dependency, admitted
  by this ruling. The roadmap's D3 evidence clause gains a dated note.

---

## 2. Architecture

### 2.1 Units

* **Interpreter.** The unit of isolation. It owns the heap, collector, classes,
  `.environment`/`.local`, the guard table, the scheduler, the baton and an inbox (an atomic request
  word plus a mutex-guarded payload queue). The baton and the inbox are the only parts another OS
  thread touches without holding the baton.
* **Activity.** A continuation: an arena-resident chain of activations with their IR register
  frames, the parked driver state (resume point and pending result register), its request word,
  its API thread context (2.4), and its per-activity state (section 5). It is named by an offset
  handle, never a Rust reference.
* **Driver.** A resumable call op pushes the callee's frame and continues in the same loop
  iteration; RETURN pops it and delivers the result into the caller's register. Resumption lands
  mid-region (the call's result feeds later ops of the same clause, as in `x = f(1) + 2`), through
  a separate resume entry, so no per-clause resume test exists (the stackless spike measured that
  test at +3.5% on emptyloop). Resumable call ops: internal CALL and function calls, `::ROUTINE`
  calls, `Op::Send`, `Op::Message` (instruction-form sends), `~new` into INIT, and
  `Message~send`/`~sendWith` and `Object~send`/`~sendWith` (every ooTest body runs through
  `.message~new(self, name)~send`, `OOREXXUNIT.CLS` near line 1583, today a recursive re-entry at
  `dispatch/object_protocol.rs:893`). The depth cap that raises Error 11 stays, for parity.
* **Driver threads.** Any OS thread holding the baton runs the driver. The interpreter keeps a pool;
  when the baton is released with runnable activities waiting, a pooled thread takes it.
* **Scheduler seam.** An internal trait: `spawn`, `run_until_park`, `park(reason)`,
  `unpark(activity)`, `yield_at_slice`, `release_baton`/`acquire_baton` (the native-call and
  blocking boundary) and `stop_the_world`. It also admits a foreign thread joining as a new activity
  (`AttachThread`), which Phase 9 implements.
* **Pinned re-entry sites that remain.** Sort comparators, conversions, UNKNOWN, FORWARD, trap
  handlers, the native API's callbacks, library-program entry, and the spike's residual paths (calls
  in loop headers, non-flattened loops, INTERPRET, calls inside call arguments). They are counted
  per kind; converting one is a later, measured step.

### 2.2 Data flow

The scheduler takes the next ready activity; the driver runs it until it parks (a Rexx-level wait:
guard lock, GUARD WHEN, semaphore, Message wait), its slice ends, it releases the baton (a native
call or blocking operation), or it completes. A parked or preempted continuation costs a pointer
swap; a baton release hands the driver to a pooled thread. Inbox events (cross-thread requests,
signals, returns from native calls wanting the baton) are drained whenever control reaches the
scheduler.

### 2.3 The invariant

**Exactly one activity touches interpreter state at a time, per interpreter**: the holder of that
interpreter's baton. Other activities may be running native C code or blocked in a system call
concurrently, off the baton, exactly as in the oracle. This is per-interpreter serialization under
R1, never process-wide.

### 2.4 The native-API context is per activity

Today there is one `rexx_api::ffi::ThreadContext` per interpreter (`rexx-exec/src/lib.rs:1314`,
built at `:2056`, kept by libraries at `install.rs:1988`), with a `home` thread asserted only by
`AttachThread` and `AddCommandEnvironment` (`rexx-api/src/ffi.rs:3490`, `:3532`) and an `innermost`
activation cell saved and restored in strict nesting (`ffi.rs:508-525`). Interleaved native calls of
different activities would break that nesting. So:

* each activity gets its own thread context, and each activation its own call or method context,
  as in the oracle (`ActivationApiContexts.hpp`);
* **every** callback checks that the calling OS thread holds the baton on behalf of that context's
  activity, taking the baton back if it was released for this native call; any other thread is
  refused, as the oracle's `Activity::validateThread` does (`ContextApi.hpp:76`,
  `concurrency/Activity.cpp:3620-3626`).

### 2.5 The `Send` grant, as a lending protocol

Moving the driver between OS threads needs `unsafe impl Send` for the interpreter island (its `Rc`
and `RefCell` interior). Its SAFETY argument is a lending protocol, not "the whole island moves":
frames of a thread that released the baton for a native call still hold `&mut Interp` borrows, so
the baton carries the right to derive the next borrow from the innermost live reborrow, and a
thread returns only once it holds the baton again and its frames are again the innermost. The grant
lands with its first user (S4, the stage that first moves the baton), in a file the plan names and
records; `unsafe` is otherwise confined to `rexx-api/src/ffi.rs`, `rexx-api/src/load.rs`,
`rexx-core/src/bytes.rs` and `rexx-core/src/frame.rs`. Two `thread_local!`s belong to one native
call's own stack and stay per OS thread: `REFUSED` (`rexx-api/src/layout.rs:423`) and `HOOK_THREW`
(`rexx-api/src/load.rs:678`). Every other `thread_local!` holding interpreter state becomes
per-interpreter or per-activity.

---

## 3. Scheduling

* **A ready queue and a sleeper queue.** Ready activities are served in arrival order (the
  uniform-allocation policy); timed sleepers (SysSleep, Alarm, Ticker, timed waits) sit in a
  deadline min-heap and move to the ready queue when due. Allocation (what position a readied
  activity gets) is the component a later policy replaces.
* **Wait objects keep the oracle's wake orders.** Guard-scope reservations are FIFO
  (`execution/VariableDictionary.cpp:547-553`, `:585-590`); GUARD WHEN watchers wake in the order the
  oracle's identity table yields them; a Message's waiters all wake at once
  (`classes/MessageClass.cpp:660-672`).

---

## 4. Time slice, requests and signals

* **The countdown.** `count_clause_against_deadline` (`rexx-exec/src/clause.rs:272`) decrements a
  counter on every clause, including an empty-body loop's step; its `#[cold]` `countdown_reached`
  (`:283`) already runs every 1024 clauses whenever a deadline is set (`clause.rs:31`, `:296`;
  `lib.rs:3263`). The request word shares that path. Because cross-thread requests and signals can
  arrive for any interpreter, the reload is bounded (order 1024) whenever the interpreter has an
  inbox, not only when several activities are runnable. The spike measured the cold path at N = 50
  at +0.27% on emptyloop and +0.06% on rexxcps; N = 1024 is inferred to cost about a twentieth and is
  measured in S0.
* **Timer thread.** One per process. An interpreter arms it when it has a second ready activity or a
  due sleeper; with nothing armed it sleeps with no deadline. It sets the SLICE bit after the
  oracle's 24 ms and wakes idle interpreters for due sleepers. The cold path loads the request word;
  SLICE switches at that clause boundary, never mid-clause, as the oracle does. Precedent: CPython's
  `eval_breaker`, Ruby's timer thread.
* **Requests to an activity** (HALT, TRACE toggle, a raise), from inside the interpreter or from
  another thread, set bits in the target activity's request word and wake it if parked; the target
  applies them through `pending_traps` at its next clause boundary. The requester never touches the
  target's frames (D3). A request from another OS thread also sets the interpreter's request word so
  an idle interpreter wakes.
* **Signals.** The CLI (`rexx-run`) installs handlers for SIGINT, SIGTERM and SIGHUP at interpreter
  start, as the oracle does (`platform/unix/SystemInterpreter.cpp:95-110`, `:130-145`). A handler
  sets the halt bit in every live interpreter through an async-signal-safe registry (atomics only,
  no allocation or locking). Only activities that are running take HALT, at their next cold visit;
  parked activities are not woken, matching `InterpreterInstance.cpp:686-703`. The registry and the
  timer thread are the one exception to "no mutable process-global state", recorded with R3.
* **Long native builtins** such as SysStemSort run no clauses, so they are not halted mid-work,
  on the oracle either; like any native call they run off the baton, so other activities proceed.
* **True thread interruption** is not pursued: signal-based preemption (Go 1.14) needs
  compiler-emitted safe-point maps for the interrupted code, which Rust code does not have, and a
  handler could only set the flag the timer sets anyway.
* **Deterministic switch mode.** Tests may set SLICE themselves (switch at clause k, or at every
  opportunity), giving reproducible interleavings and a stress mode analogous to
  collect-at-every-allocation. It does not make time virtual: interleavings that depend on a
  sleeper's wake time stay nondeterministic.

---

## 5. Per-activity state, collector and roots

* **Per-activity state.** Everything that belongs to one execution moves off `Interp` into the
  activity: the `RootSet`'s per-activity part (`slots`, `aliases`, `frame_starts`, `temps`, `parked`,
  register frames), and `value_buffer` (cleared at every clause start, `clause.rs:242`),
  `running`/`suspended`, `clause_state`, `frames`, `flat_top`, `pending_traps`, `active_condition`,
  `fragments`, `depth`, `call_context`, `indents`, and `random_seed` (per activity in the oracle,
  `concurrency/Activity.hpp:426`). The exhaustive destructure in `Interp::object_roots` (`lib.rs`
  near 2682) is the checklist: every field is classified interpreter or activity, and a new field is
  a compile error until classified.
* **The interpreter part** of the `RootSet` keeps `globals`, `cells`, classes, the environment and
  the guard table. The running activity's part stays one indexed load away.
* **Frame arenas per activity.** `FrameArena` is strict LIFO with blocks of about 1 MiB
  (`rexx-core/src/frame.rs:43`, `:53`, `:70`, `:118`); one per activity at that size would allow
  roughly 900 activities under the oracle wrapper's 1 GiB limit. Each activity gets its own arena
  starting at a small block and growing on demand, so LIFO holds within an activity. `frame.rs`'s
  invariants change, which needs its grant restated.
* **Slots per activation as arena segments**, not one flat stack, so a frame below the top can grow.
  This removes the Phase 8 residual owned by Phase 6: setting, through a kept outer call context, a
  name the outer activation never bound.
* **Collection points.** Every allocation remains one, as a stated constraint (it is the only
  instrument that finds a missed root). A continuation switch and a baton release are added: anything
  live across either must be rooted, the rule that already holds across an allocation. A thread that
  released the baton for a native call therefore sits at a safepoint; its Rust frames root what they
  hold because the nested Rexx code they may call allocates.
* **Parallel-ready hooks.** `collect` calls the scheduler's `stop_the_world`, which today asserts
  that the caller holds the baton (so no other thread can touch interpreter state). A future
  per-activity or parallel collector replaces that one function. A stress mode collects at every
  countdown visit, switch and baton handoff.
* **Isolation at the type level.** `ObjRef` becomes `!Send` and `!Sync` (a phantom raw-pointer
  marker; a scratch build confirmed nothing sends one today), so the compiler refuses an object
  handle in the inbox, a helper message or another interpreter.
* **Sharing-fraction instrument.** A feature-gated build tags each object with the last activity
  that resolved it and counts objects touched by more than one activity: the number the research
  direction says decides isolation against shared-heap-plus-locking for the no-lock future.

---

## 6. Rexx semantics

Oracle behaviour is **Read** from source or **Measured** by review round 1 unless marked
*(verify)*, which the plan settles in the first task that needs it, with the command recorded.

* **Guard locks.** Per (object, scope): the owning activity, a nesting count and a FIFO waiter
  queue, in a per-interpreter table keyed by object identity. Guarded methods reserve on entry;
  unguarded ones skip. The owner re-enters by bumping the count (the oracle's owner is the same
  activity stack, nested attaches included, `VariableDictionary.cpp:527-533`). A contended reserve
  parks. **Deadlock detection:** a contested reserve, and `Message~wait`/`~result`, follow the chain
  of what each waiting activity waits on and raise 98.905 on a cycle (`Activity::checkDeadLock`,
  `concurrency/Activity.cpp:2000-2027`); GUARD WHEN waits and waits on an unsent message are not
  checked and block forever (`rust/corpus/oracle-crashes.txt` entry 7).
* **GUARD ON/OFF and WHEN.** GUARD ON reserves, OFF releases **one** nesting level. WHEN registers
  the activity as a watcher of each **variable named in its expression**
  (`instructions/GuardInstruction.cpp:141-144`), releases one level, and parks; the 99.913 error for a
  WHEN naming no exposed variable is kept. A **set or DROP** of a watched variable notifies
  (`RexxVariable.hpp:71-77`, `RexxVariable.cpp:158-169`), including a compound element of an exposed
  stem (`expression/ExpressionCompoundVariable.cpp:348`), and the notifying activity then yields
  (`RexxVariable.cpp:192-193`). The barrier therefore sits on every path that stores or drops an
  object variable: the pool path (`set_exposed_variable`, `rexx-exec/src/variables.rs:103-125`),
  stem element stores, attribute setters and `SetObjectVariable` (`dispatch/library.rs:718`). It is
  one "watched" test on those paths only, never on locals, and is measured. Each re-evaluation is
  traced as the oracle traces it (`GuardInstruction.cpp:176`, `:183`).
* **REPLY** splits the continuation: the caller resumes with the reply value; the rest of the
  activation becomes a new activity. The guard lock **moves** to it only when its nesting count is 1
  (`VariableDictionary::transfer`, `VariableDictionary.cpp:600-617`); otherwise the continuation
  reserves again (`execution/RexxActivation.cpp:766-773`, `:561-568`). REPLY then yields
  (`:776`), so the continuation usually runs first. GUARD.testGroup's cases at `:258-279` and
  `:290-330` depend on these rules. Today's deferral of replies to program end, and the refusal of
  REPLY inside DO/SELECT/IF, go. An untrapped error in a REPLY continuation is shown through
  `.traceOutput` (RAISE.testGroup:470-487).
* **Message objects.** `~start`, `~startWith` and `Object~start`/`~startWith` spawn an activity that
  makes the send; it starts without the caller's NUMERIC settings (`RexxCode.cpp:207`,
  `RexxActivation.cpp:152-158`), and a method's ADDRESS comes from the instance default (`:171`).
  `~result`, `~wait` and `~completed` read or park on the completion; `~reply` and `~replyWith`
  are implemented (`classes/MessageClass.cpp:562-640`); `~notify` is synchronous (`:209-226`,
  `:660-672`). An untrapped error in a started send prints its traceback on the started activity at
  once, `~hasError` answers 1, and `~result` re-raises the condition (`MessageDispatcher.cpp:64-72`;
  measured). `~start` stops running synchronously.
* **MutexSemaphore and EventSemaphore** (the classes) are per-interpreter objects. A MutexSemaphore's
  owner is the activity; it is re-entrant for its owner, released when the owning activity ends
  (`Activity.cpp:303`; `concurrency/MutexSemaphore.cpp:230-262`), and after REPLY stays with the
  caller.
* **RexxUtil `Sys*Sem`** are not the classes. On Unix, `SysCreateMutexSem` and friends are counting
  POSIX semaphores with no owner and no re-entry, a timeout of 0 meaning forever, and return codes
  121 and 6 (`platform/unix/SysRexxUtil.cpp:868-903`, `:945-1025`). Unnamed ones are built here,
  matching those semantics; **named** ones go through `sem_open` (`:668`), shared between processes,
  and belong to the RexxUtil remainder of Phase 10 (roadmap D11: "the remainder in Phase 10").
* **Alarm and Ticker** are Rexx code in `CoreClasses.orx` (near 1560-1575 and 1660-1672): REPLY,
  then the native `!startTimer` or `!waitTimer`, today deferred at `dispatch/native.rs:116-120`. The
  native timers are a timed park with early wake on cancel; firing runs on the replied activity, and
  program end waits for it.
* **Thread identity and guard state are observable.** `.context~thread` answers the activity's own
  identity, not today's constant 1 (`dispatch/context.rs:251-259`), and TraceObjects carry THREAD,
  CALLERSTACKFRAME, ISGUARDED, SCOPELOCKCOUNT, HASSCOPELOCK and ISWAITING as the oracle sets them
  (`RexxActivation.cpp:5160-5230`), not today's fixed values (`route.rs:216-221`).
* **UNINIT.** Uninit methods run on the activity that ends, when it ends, and at interpreter
  termination (`Activity.cpp:303`, `InterpreterInstance.cpp:571`) *(verify the exact ordering)*.
* **Program end** waits for every activity that is not pooled, including ones that never finish
  (a blocked WHEN, an uncancelled Ticker), then collects and runs uninits; the exit status is the main
  program's (`InterpreterInstance::terminate`, `:521-573`; measured).

---

## 7. Performance rule

Callgrind instruction counts, glibc and ld-linux excluded, each build in its own target directory
with a `Compiling` line, a layout control per comparison, **cumulative against the Phase 6 base
commit** (never only per step: per-step thresholds hid +32% of drift in Phase 5a). Programs:
`rexxcps`, `emptyloop`, `varlookup`, a recursive fib(22) by CALL and by function, and a send loop,
the last two committed as benchmark programs in S0. Tolerance: **+1.0%** on each program against
the base, beyond the layout control's own spread. S1 must end at or under that on every program; if
it does not after three recorded optimisation rounds, S1 stops and the carrier choice is ruled on
with the figures (the fallback is baton threads without continuations, behind the same seam).

---

## 8. Staging

Each stage ends with every gate green and within the performance rule.

| Stage | Delivers |
|---|---|
| S0 Foundations | The scheduler seam; per-activity state split out of `Interp` (section 5), with the exhaustive classification; per-activity frame arenas; `ObjRef` `!Send`; interpreter-state `thread_local!`s made per-interpreter; the committed call-heavy and send-heavy benchmarks. No behaviour change and no second activity yet. |
| S1 Stackless calls | The resumable call ops of 2.1, `Message~send`/`Object~send` included; mid-region resume through a separate entry; parked state in the arena with offset handles; slots per activation segment. |
| S2 Activities | Scheduler (section 3), request words and inbox, timer thread, deterministic switch mode; `Message~start`/`~result`/`~wait`/`~reply`/`~replyWith`/`~notify`; REPLY as a continuation split; `.context~thread` and TraceObject fields; program-end and UNINIT rules. |
| S3 Synchronisation | Guard locks with deadlock detection; GUARD ON/OFF/WHEN and its barrier; the classes MutexSemaphore and EventSemaphore; unnamed `Sys*Sem`; Alarm and Ticker. |
| S4 The baton | Per-activity API contexts with the per-callback check (2.4); the baton released around native calls and blocking operations and driver threads taking it (P6-3); the `Send` grant with its lending-protocol SAFETY (2.5); signals (R3); pinning counters. |
| S5 Close | Section 9. |

S2 and S3 run with a single driver thread: native calls and blocking operations still hold the
baton until S4, so their tests are those that need no concurrency with native code. ooTest's bodies
are resumable from S1 (2.1), so they are not pinned.

---

## 9. Exit criteria

1. **The concurrency groups** pass on the crate and agree with the oracle through a Phase 8-style
   instrument, excluding any test that changes rxapi's persistent state:
   `base/keyword/GUARD`, `base/keyword/REPLY`, `base/class/Message`, `base/class/EventSemaphore`,
   `base/class/MutexSemaphore`, `base/class/Alarm`, `base/class/Ticker`,
   `regressions/bug2003_guard_when`, `base/rexxutil/SysSleep`; and the concurrency tests of
   `base/class/TRACE_TraceObject` (near :306-324), `base/class/RexxContext` (near :200-284),
   `base/keyword/TRACE` (near :750-764), `base/keyword/RAISE` (near :470-487) and
   `base/class/Object` (near :271-284).
2. **The ooTest framework's ticker** (`ooTest.frm`, a REPLY thread and GUARD ON WHEN) runs without
   `-U`; L2 then remains blocked only by Phase 10's RXFUNCQUERY.
3. **Race checking (R4):** a ThreadSanitizer build of the test suite with the installed nightly is
   clean over every cross-thread path (baton, inbox, timer, driver pool, signals), and the `loom`
   model of the baton, inbox and timer protocol passes.
4. **The single-owner audit**, replacing roadmap row 6's lock-dependence wording under R1: an
   inventory of every `unsafe impl Send` and every cross-thread type, each with a test or a
   type-level check that no interpreter state is reachable from a thread not holding that
   interpreter's baton.
5. **The sharing fraction** is measured over the corpus and ooTest and recorded.
6. **A ping-pong benchmark** (message round trip, semaphore post/wait, GUARD WHEN handoff) is
   recorded against the oracle, not gated: the baseline for any later carrier change.
7. **No refusal names Phase 6.** Every `Loud` text, layout owner, native entry owner and exclusions
   row naming Phase 6 is gone or re-homed with a true reason. This covers at least the GUARD WHEN
   wait, REPLY inside a construct, `Message~result` of an unsent message, the Alarm and Ticker timer
   entries, the `Sys*Sem` routines, `SetGuardOnWhenUpdated`/`SetGuardOffWhenUpdated`, the
   kept-outer-context residual, the L2 ticker rows, and the Message and semaphore methods that fall
   through to the generic native-method refusal today. The enumeration is derived by a committed
   command, shown empty at the close, and enforced: `closed_phases` gains Phase 6, so a reintroduced
   owner or text naming it turns the gate red.
8. Every *(verify)* item of section 6 is settled against the oracle.

**Testing rule.** Corpus programs stay deterministic (`corpus/README.md`), so they witness only
orderings the oracle itself fixes. Interleaving-dependent behaviour is tested on the crate alone,
under the deterministic switch mode. The process-global `static FLAT` read from an environment
variable (`run/loops.rs`, marked SPIKE, already queued) is removed before S0 closes, since the rule
this design relies on is "no mutable process-global state" beyond R3's registry and the timer.

---

## 10. For the later phase: moving values between interpreters

Recorded here because P6-8 defers it. ooRexx offers no user-facing object serialization: queues
carry strings, and rxapi queues hold byte strings. The interpreter has an internal graph flattener
(`memory/Envelope.cpp`, `SmartBuffer.cpp`) used for `rexxc` images and `rexx.img`; it preserves
identity and cycles but assumes the same interpreter build reads it back. Elsewhere, from cheapest
to most general: values only (Tcl threads, rxapi queues); structured clone of a fixed type set,
refusing behaviour (JavaScript workers); copy, move or share-if-deeply-frozen (Erlang, Ruby Ractors,
Python PEP 734 "shareable" types); and full pickling with per-class hooks. The natural fit for Rexx
is structured clone over its own types (strings, numbers, Array, Directory, StringTable, Stem, Bag)
plus an opt-in pair of methods for user classes.
