# Phase 6: concurrency, as isolated interpreters with continuation activities

**Status:** design, agreed with Moritz in conversation on 2026-09-29 section by section, then
revised after review rounds 1 to 4 (`spec-review-1.md` to `spec-review-4.md` in
`docs/superpowers/records/2026-09-29-phase-6-design/`) with his rulings R1 to R5 (1.1). It replaces
the delivery order of the roadmap's D3 option (a), a coarse process-wide kernel lock. Roadmap row 6
(`docs/superpowers/plans/2026-07-27-rust-rewrite.md`) remains the exit contract, read through this
document.

**Inputs.** `docs/superpowers/specs/2026-09-08-concurrency-research-direction.md`; the records in
`docs/superpowers/records/2026-09-29-phase-6-design/` (`yield-spike-findings.md`,
`stackless-spike-findings.md` with `stackless-spike.patch` and `stackless-reentry-sites.txt`, and
the reviews). Oracle source paths are relative to `/home/moritz/dev/repos/ooRexx/interpreter`.

---

## 1. Decisions

| # | Decision | Why |
|---|---|---|
| P6-1 | **Isolation.** One interpreter owns its heap, collector, classes, `.environment`, `.local` and scheduler. Interpreters share nothing. | No process-wide lock, by construction. Shared-heap runtimes paid with a per-operation shared structure (research direction, section 5). |
| P6-2 | **Activities are continuations.** An activity's state is arena data run by a driver loop; Rexx-to-Rexx calls and sends swap frames inside that loop. | A switch on a slice or a Rexx-level wait is a pointer swap; REPLY is a continuation split; and it is what makes handing an interpreter between OS threads sound in Rust (P6-3). Precedents: CPython 3.11, Lua. |
| P6-3 | **A per-interpreter baton, released only at a driver exit.** Holding the baton is the right to touch interpreter state. It is released where the oracle releases its kernel lock, around calls into extension C code and around blocking operations (`execution/NativeActivation.cpp:1303-1306`, `:1420-1423`, `:1541-1544`, `:1690-1692`; `api/ContextApi.hpp:64-77`), and only at a **driver exit**: a point where the releasing thread holds no value of the interpreter island at all (no borrow, no `Rc` clone, no `Cell` or `RefCell` access), only raw handles. The native call or blocking operation then runs off the baton; its **completion** (a typed record, 2.1, never an `ObjRef`) is posted to the interpreter's inbox; the baton holder moves it into the activity's parked record and readies the activity, and the finish half runs **as that activity**, as its first step when it resumes. Only a C **callback** into the API needs interpreter state while off the driver, and it is served under the baton (2.4); the return of a callback is itself a baton release. | `&mut` function arguments are protected under Stacked and Tree Borrows, so a thread still holding one cannot let another touch the same state; handing the baton from inside nested Rust frames would force returns into LIFO order across threads and deadlock programs the oracle runs (review 2, C-A). Posting completions means a return never waits for the baton, so a pinned waiter can consume it (review 3, C-1). The Go runtime's processor handoff around a system call is the same shape. |
| P6-4 | **Pinning.** An activity is **pinned** while any Rust frame lies between the scheduler and its running driver (a native re-entry, a tree-evaluated expression, a Rust wrapper that sends a message). A pinned activity never releases the baton. A pinned activity that must wait runs a nested scheduler on its own stack; a SLICE seen while pinned is deferred to the next unpinned clause boundary. | Loom's pinning; Lua's "cannot yield across a C-call boundary". Nesting is LIFO, so a wait inverted against the nesting cannot complete: detected and refused loudly, counted (2.6). Deferring slices, as Lua and Loom do, avoids nesting ready activities above a guard holder and livelocking two pinned busy-waiters (review 3, I-2). Correctness rests on pinned waits being rare, which exit criterion 9 measures. |
| P6-5 | **A scheduling seam.** Nothing outside the scheduler knows how an activity is carried. | Keeps the carrier replaceable on measurement. |
| P6-6 | **An RBED-shaped scheduler shipping round-robin.** Dispatch is separate from allocation; Phase 6 ships the uniform policy only. | Uniform allocation with arrival-order tie-breaks is round-robin, which parity needs. |
| P6-7 | **The slice comes from the existing clause countdown; the slice is a request bit set by a timer thread.** | No new hot-path branch; one mechanism for slice, HALT, signals and cross-thread requests; tests set the bit themselves. |
| P6-8 | **Out of scope:** the Rexx-level API for user-managed interpreters, and exchanging data between interpreters over queues. | A later phase, with the rxapi integration (Moritz, 2026-09-29). Section 10 records its starting precedents. |

### 1.1 Rulings recorded with this design (Moritz, 2026-09-29)

* **R1. D3 is narrowed to process-wide.** Per-interpreter serialization (the baton, as PEP 684's
  per-interpreter lock) is allowed; no invariant may rest on one activity running at a time across
  the process. D3's rule that cross-activity requests go through a channel or an atomic flag, never a
  foreign frame pointer, is unchanged and holds inside one interpreter too.
* **R2. `.environment` per interpreter is a licensed divergence for Phase 9.** The oracle's is
  process-wide (`RexxCore.h:267`, `Setup.cpp:300`); only `.local` is per instance
  (`runtime/InterpreterInstance.cpp:208`).
* **R3. Signal handling is a new `unsafe` site.** One module installs the handlers; a handler only
  sets atomics and writes a byte to a self-pipe (section 4).
* **R4. Race checking** is a ThreadSanitizer run with the installed nightly toolchain (gate-only,
  never for shipped builds) plus `loom`, admitted as a dev-dependency, over the baton, inbox and
  timer, written against a `cfg(loom)` synchronisation shim so `loom` checks the shipped code. Moritz
  had no preference among nightly, `loom` and `shuttle`: TSan sees data races in the real binary;
  `loom` exhaustively finds lost wakeups and handoff bugs; the deterministic switch mode covers
  interpreter-level interleavings, which is shuttle's niche.
* **R5. `libc` becomes a direct dependency** of the signal module (it is already in the dependency
  graph transitively, so no new code enters the build; it gives correct `sigaction` layouts on every
  platform Phase 11 targets).

The roadmap records these rulings: R1 and R4 in D3's dated note already; R2, R3 and R5 are added
there by the plan's first task, together with the Section 1 decision blocks the Global Constraints
require before code for the new `unsafe` sites (2.5, R3) and for `frame.rs`'s changed invariants,
row 6's "kernel lock" wording read through R1, and the Global Constraints' nightly line amended for
R4.

---

## 2. Architecture

### 2.1 Units

* **Interpreter.** The unit of isolation. It owns the heap, collector, classes,
  `.environment`/`.local`, the guard table, the scheduler, the baton, a pool of driver threads, and an
  inbox (an atomic request word plus a mutex-guarded queue of requests and completions). The baton
  and the inbox are the only parts a thread touches without holding the baton.
* **Activity.** A continuation: an arena-resident chain of activations with their IR register frames,
  the parked driver state (resume point and pending result register), its request bits, its API
  thread context (2.4), its activity number (section 6) and its per-activity state (section 5). It is
  named by an offset handle.
* **Driver.** A resumable call op pushes the callee's frame and continues in the same loop iteration;
  RETURN pops it and delivers the result into the caller's register. Resumption lands mid-region (the
  call's result feeds later ops of the same clause, `x = f(1) + 2`) through a separate resume entry,
  so no per-clause resume test exists (the stackless spike measured that test at +3.5% on emptyloop).
  The depth cap raising Error 11 stays, for parity.
* **Resumable entries (S1).** Internal CALL and function calls; `::ROUTINE` calls; **message sends in
  expression position and the arguments of calls and sends**, compiled to ops rather than rejected by
  `native_shape` into `Op::EvalExpr` (`rexx-exec/src/ir/compile.rs:1212-1235`); a new `Op::Send`
  (today only in `stackless-spike.patch`);
  `Op::Message` (instruction-form sends); `~new` into INIT; `Message~send`/`~sendWith` and
  `Object~send`/`~sendWith` (every ooTest body runs through `.message~new(self, name)~send`,
  `OOREXXUNIT.CLS` near line 1583, today recursive at `dispatch/object_protocol.rs:893`);
  `Routine~call` of a Rexx routine; a started activity's first send (today `started_message` ->
  `send_message`, `object_protocol.rs:824`); and the REPLY continuation's resume (today `resume_reply`
  -> `run_activation`, `dispatch.rs:2528`). Without these, the concurrency tests would park pinned
  (review 3, criterion 9 estimate).
* **Park points for instructions (S1 provides the channel; S2 and S3 use it).** GUARD, REPLY,
  FORWARD and some CALLs run inside `Op::Exec` (`ir/compile.rs:1052-1078`; `exec_instruction` ->
  `exec_guard`, `run.rs:1114`). Their arms return an outcome to the driver instead of waiting inside:
  a contended GUARD ON and a GUARD WHEN return **Park**, REPLY returns **Split**, and the
  guarded-method reservation parks in a send's begin half. Without these, every GUARD wait would be a
  pinned park and every REPLY immovable; GUARD.testGroup `test_wait_multiple` (`:287-330`) would
  invert deterministically (review 4, C-1).
* **Plain DO blocks are flattened (S1).** `DO; ... END` (`LoopKind::Simple`) today falls back from
  the flat-loop path (`run/loops.rs:1356`) to a nested `run_loop_with_header` with `BodyEngine::Chunk`
  (`ir/drive.rs:1900-1915`), so a wait in `if ... then do ... end` would pin. S1 runs its body inline
  (it has no pass boundary; END is a no-op). **The flat-loop path is adopted**: it is committed and
  runs by default, and its "SPIKE, not for commit" markers (`run/loops.rs:1284`, `ir/compile.rs:1038`)
  and the environment-variable toggle behind `static FLAT` are stale; S0 removes both (the queued
  item `2026-09-26-flatloop-spike-comment` covers the comment).
* **Parkable natives.** A native implemented in Rust may answer `Park(reason)` instead of a value;
  the reason carries the wake outcome (a value, a timeout, a lock acquired). **Composition:** `Park`
  propagates through resumable entries to the driver, which parks the activity and on wake re-enters
  the native's continuation half, delivering its result into the caller's register. At the first
  non-resumable caller (a pinned path), `send_message` converts it into a pinned wait (2.6) followed
  by the same continuation half. `Message~result`/`~wait`, `MutexSemaphore~request`,
  `EventSemaphore~wait`, the native timers `!startTimer`/`!waitTimer`, SysSleep and the unnamed
  `Sys*Sem` waits are parkable natives.
* **Native calls and blocking Rust operations** split into prepare, call (or block) and finish.
  Today `invoke::run` holds `Conversion { host: &mut dyn Host }` across the whole C call
  (`rexx-api/src/values.rs:716-719`, `dispatch/library.rs:96-106`) and then runs
  `values::from_native` (`rexx-api/src/invoke.rs:218-224`); it splits into those halves. Prepare runs
  under the baton; the driver exits; the call runs holding no island value; its completion record
  carries the declared result type, the `Written` payload and the refused slot (captured from the
  thread-local `REFUSED` on the calling thread).
  The call's native frame stays pushed and rooted as that activity's until the finish half pops it,
  since a returned handle resolves only in that frame's `locals` (`rexx-api/src/handles.rs:43-52`).
  `Completion` carries no pending condition: it stays in island state (the native frame), read by
  finish, since `RaiseException` callbacks hold the baton (2.4).
  Builtins that work on island data (SysStemSort sorts a stem) copy it out in prepare and back in
  finish, or stay on the baton. This applies where the operation is reached from a resumable entry.
  **Wrappers that keep the baton** (reached only through nested Rust frames today, so pinned): the
  stream BIFs (`builtin/stream.rs:48`), PULL (`input.rs:231`), SAY to a routed `.output`
  (`run.rs:1830`), trace output delivery (`environment/route.rs`, TraceObject construction at `:205`), ADDRESS WITH redirection
  (`redirect.rs:376`, `:686`), condition handling (`run/condition.rs`), package loaders
  (`install.rs:1993`) and `Routine~call` of a library routine (`dispatch/executable.rs:563`). While
  one of these blocks (a console read, a pipe), the interpreter's other activities wait. This is
  a recorded divergence, counted by the pinning report; converting a wrapper is a later measured step.
* **Scheduler seam.** An internal trait: `spawn`, `run_until_park`, `park(reason)`,
  `unpark(activity)`, `yield_at_slice`, `exit_for_native`/`exit_for_block` (the baton-releasing driver
  exits), `post_completion`, `request_baton` (callbacks only) and `stop_the_world`. It also admits a
  foreign thread joining as a new activity (`AttachThread`), which Phase 9 implements.
* **Pinned re-entry kinds**, counted per kind: sort comparators, conversions, UNKNOWN, FORWARD,
  operators dispatching to Rexx methods, trap handlers, native-API callbacks, library-program entry,
  the wrappers above, calls in loop headers, non-flattened loops and INTERPRET.

### 2.2 Data flow

The scheduler takes the next ready activity; the driver runs it until it parks, its slice ends, it
exits for a native call or a blocking operation (releasing the baton), or it completes. Parks and
preemptions cost a pointer swap. On a release with ready activities, a pooled driver thread takes the
baton. When an off-baton operation finishes, its thread posts the completion to the inbox and becomes
idle or returns to the pool; the baton holder drains the inbox at its next scheduler entry or cold
visit, runs the finish half, and readies the activity. A C callback requests the baton on its own
thread (section 4).

### 2.3 The invariant

**Exactly one thread touches interpreter state at a time, per interpreter**: the holder of that
interpreter's baton. Other activities may be running extension C code or blocked in a system call
concurrently, off the baton, as in the oracle. Per-interpreter serialization under R1.

### 2.4 The native-API context is per activity

Today one `rexx_api::ffi::ThreadContext` serves the interpreter (`rexx-exec/src/lib.rs:1314`, built
at `:2056`), with a `home` thread asserted only by `AttachThread` and `AddCommandEnvironment`
(`rexx-api/src/ffi.rs:3490`, `:3532`), an `innermost` activation cell saved and restored in strict
nesting (`ffi.rs:508-525`), and `native_handles` a per-interpreter stack, "innermost last"
(`lib.rs:1273-1280`, pushed at `dispatch/library.rs:475`, its index handed to rexx-api at `:96`). So:

* each activity gets its own thread context and native-handle stack, and each activation its own
  call or method context, as in the oracle (`ActivationApiContexts.hpp`);
* the interpreter's `RexxInstance`, and the context of the activity that loaded a library, outlive
  that activity, since libraries keep them (`install.rs:1988`, `rexx-api/src/load.rs:809`) and
  extensions cache the instance for `AttachThread`;
* `Host` becomes a baton-guarded accessor, served in one of two ways (review 4, I-3). If the baton's
  holder is unpinned, the callback takes the baton at the holder's next cold visit, derives a fresh
  `&mut` from the interpreter's root pointer, and drops every island value before returning to C.
  If the holder is pinned (its frames hold protected borrows), the callback is **delegated**: it posts
  the API member to the inbox and blocks; the holder's nested loop runs it on behalf of the callback's
  activity, reborrowing its own `&mut`, and posts the answer (and any refused slot) back. The oracle
  takes its kernel lock per API call (`ContextApi.hpp:64-77`), so the cost shape matches; a callback
  never inverts a pinned wait;
* "the thread of an activity" means the thread currently running that activity's native call (an
  activity may run on different OS threads over its life); a callback from any other thread raises
  Error_Execution_invalid_thread, after taking the baton, as the oracle's `Activity::validateThread`
  does (`ContextApi.hpp:73-76`, `concurrency/Activity.cpp:3620-3626`).

### 2.5 The `Send` grant

Moving the driver between OS threads needs `unsafe impl Send` for the interpreter island (its `Rc`,
`Cell` and `RefCell` interior). SAFETY: the island is reachable only through one root pointer and the
baton; a thread touches any island value (derives a borrow, clones or drops an `Rc`, reads or writes a
`Cell`) only while holding the baton; the baton is released only at a driver exit, where the
releasing thread holds no island value, only raw handles. Today's native call holds `Rc` clones across
the C call (`Rc::clone(&binding.library)`, `self.thread.clone()`, `dispatch/library.rs:86`, `:94`,
`:155`, `:165`); those become raw handles or move into the activity record before the exit. The grant
lands with its first user (S4) in a module the plan names, recorded as a Section 1 decision block with
R3's signal module; `unsafe` is otherwise confined to `rexx-api/src/ffi.rs`, `rexx-api/src/load.rs`,
`rexx-core/src/bytes.rs` and `rexx-core/src/frame.rs`. The `thread_local!`s `REFUSED`
(`rexx-api/src/layout.rs:423`) and `HOOK_THREW` (`rexx-api/src/load.rs:678`) belong to one native
call's own stack and stay per OS thread; they are the only non-test `thread_local!`s today.

**Island memory lent to C.** C reads and writes some island memory through raw pointers while off
the baton: kept C strings (`kept_strings`, `dispatch/library.rs:984-1010`, pruned by
`drop_loose_kept_strings`) and the storage behind `BufferData` and `MutableBufferData`
(`dispatch/library/surface.rs:212-237`). The rule, per kind: storage lent to a native call is not
freed or reallocated in place while that call is in flight. A kept string is not pruned while any
call that received it is in flight; a MutableBuffer mutation that would reallocate lent storage
allocates new storage and keeps the old alive until the lending call completes. C reads and writes
through the old address after such a reallocation do not reach the buffer, as in the oracle, where
growth gives the buffer a new data object (`MutableBuffer::ensureCapacity`,
`interpreter/classes/MutableBufferClass.cpp:246`) and leaves the old one as garbage. The licensed
divergence is that storage's lifetime: the oracle's next collection reclaims it under the call, after
which the call's accesses touch freed memory; here it stays valid until the lending call completes.

### 2.6 Pinned waits

A pinned activity that parks runs a nested scheduler loop on its own stack until its wait is
satisfied; other activities run inside it as continuations, and inbox completions are drained there
too, so an activity returning from C can complete even while a pinned waiter holds the baton. A
**wait is inverted** when what the innermost pinned waiter needs can only come from an activity whose
continuation lies below it in the nesting; then nothing the nested loop can run will satisfy it.
While an off-baton operation is still in flight the nested loop blocks on the inbox (its completion,
or a delegated callback, may satisfy the wait); only when nothing is ready, nothing is in flight and
the wait is inverted does it raise a loud refusal naming the pinned re-entry kind. A single pinned
busy-waiter on a ready activity never yields, since its slices are deferred; the oracle preempts it.
An activity that traps HALT and parks again while nested keeps the enclosing pinned activities
waiting, and a targeted HALT of an enclosing activity waits until the inner region ends. These are
known divergences from the oracle (whose every activity has its own OS thread), recorded with an
owner of none and bounded by exit criterion 9.

### 2.7 Driver threads

The interpreter's first thread keeps today's large reservation (`INTERPRETER_STACK_BYTES`,
`lib.rs:183`, `:3180`). Pooled driver threads get a smaller stack (the oracle's activity threads are
512 KiB, `common/platform/unix/SysThread.hpp:67`; reservations count against the `ulimit -v` the
oracle wrapper applies). On a pool thread, Error 11 is raised when the remaining stack falls below a
margin, as the oracle's `stackLimit` does, besides the depth cap; pinned recursion is the only
native-stack recursion left after S1. The pool has a bound; when a spawn fails or the bound is
reached, a release with ready activities leaves them for the next thread that takes the baton.

---

## 3. Scheduling

* **A ready queue and a sleeper queue.** Ready activities are served in arrival order (the uniform
  allocation policy); timed sleepers (SysSleep, timers, timed waits) sit in a deadline min-heap and
  move to the ready queue when due.
* **Wait queues.** Guard-scope reservations are FIFO, as the oracle's are
  (`execution/VariableDictionary.cpp:547-553`, `:585-590`). GUARD WHEN watchers wake FIFO; the
  oracle's order is OS scheduling plus its kernel-lock queue (`RexxVariable.cpp:176-193`), not
  reproducible, so the difference is licensed. A Message's waiters all wake at once
  (`classes/MessageClass.cpp:660-672`).

---

## 4. Time slice, requests and signals

* **The countdown.** `count_clause_against_deadline` (`rexx-exec/src/clause.rs:272`) decrements a
  counter on every clause, including an empty-body loop's step; its `#[cold]` `countdown_reached`
  (`:283`) already runs every 1024 clauses whenever a deadline is set (`clause.rs:31`, `:296`;
  `lib.rs:3263`). The request bits share that path. The reload is always bounded (order 1024), since
  any interpreter can receive a request or a completion; the spike measured the cold path at N = 50
  at +0.27% on emptyloop and +0.06% on rexxcps, and N = 1024 is measured in S0.
* **Timer thread.** One per process. An interpreter arms it when it has a second ready activity, a
  due sleeper, or a pending completion or baton request; with nothing armed it sleeps with no
  deadline, and it ends when no interpreter is registered, started again by the next arm or idle
  (ruling P49). It sets the SLICE bit after the oracle's 24 ms and wakes idle interpreters for due
  sleepers. SLICE switches at that clause boundary, never mid-clause, and while pinned is deferred to
  the next unpinned boundary (P6-4). Precedent: CPython's `eval_breaker`, Ruby's timer thread.
* **Baton requests** come only from C callbacks (2.4). Against an unpinned holder a callback sets
  the request word, which forces a switch at the holder's next cold visit without waiting out the
  24 ms; against a pinned holder it is delegated instead, as the oracle prioritises API
  callers (`requestApiAccess` -> `addWaitingApiActivity`, `concurrency/Activity.cpp:2235-2249`,
  `:339`; `relinquishIfNeeded`, `concurrency/ActivityManager.hpp:303-323`). Completions of returning
  calls never request the baton (P6-3).
* **Requests to an activity** (HALT, TRACE toggle, a raise) from inside the interpreter set the
  target's request bits, lower the countdown to 1 so the running activity sees them at its next clause,
  and wake the target if parked; the target applies them through `pending_traps` at its next clause
  boundary. A request from another OS thread goes through the interpreter's inbox (it never writes
  arena data). Nobody touches another activity's frames (D3).
* **Signals.** Handlers for SIGINT, SIGTERM and SIGHUP are installed at interpreter start, only where
  no handler is already set, as the oracle's library does
  (`platform/unix/SystemInterpreter.cpp:95-110`, `:130-145`; the oracle in fact reads only SIGHUP's
  previous action before installing any, `:136-139`, and this design checks each; under `nohup`,
  with SIGHUP ignored, the oracle installs none and SIGINT kills the process, where this design halts,
  a licensed divergence), without
  SA_RESTART (`SystemInterpreter.cpp:127`). A handler, on whatever thread the signal lands, sets a
  pending bit and writes one byte to a self-pipe (both async-signal-safe); the timer thread waits on
  that pipe alongside its deadline and, on wake, sets the halt bit in every live interpreter and
  wakes the idle ones. Every activity with frames (the oracle's `isActive()`, nestedCount > 0,
  `Activity.hpp:249`; `InterpreterInstance.cpp:686-703`) takes HALT: a running one at its next cold
  visit, a parked or sleeping one when woken, and it is woken, as SIGINT interrupts the oracle's
  `nanosleep` (measured: `rc = SysSleep(5)` under `timeout -s INT 1` halts at about 1 s with
  Error 4.1). The live-interpreter registry and the timer thread are the one exception to "no mutable
  process-global state" (R3).
* **Long native builtins** such as SysStemSort run no clauses and are not halted mid-work, on the
  oracle either; reached from a resumable entry they run as blocking operations, off the baton.
* **True thread interruption** is not pursued: signal-based preemption (Go 1.14) needs
  compiler-emitted safe-point maps, which Rust code does not have.
* **Deterministic switch mode.** Tests may set SLICE themselves (switch at clause k, or at every
  opportunity), giving reproducible interleavings and a stress mode analogous to
  collect-at-every-allocation. It does not make time virtual: interleavings depending on a sleeper's
  wake time stay nondeterministic.

---

## 5. Per-activity state, collector and roots

* **Per-activity state.** Off `Interp` and into the activity: the `RootSet`'s per-activity part
  (`slots`, `aliases`, `frame_starts`, `temps`, `parked`, register frames), `native_handles` and
  `native_spares`, `value_buffer` (cleared at every clause start, `clause.rs:245`),
  `running`/`suspended`, `clause_state`, `frames`, `flat_top`, `pending_traps`, `active_condition`,
  `fragments`, `depth`, `call_context`, `indents`, and `random_seed` (per activity in the oracle,
  `concurrency/Activity.hpp:426`). The exhaustive destructure in `Interp::object_roots`
  (`lib.rs:2657`) is the checklist: every field is classified interpreter or activity, and a new field
  is a compile error until classified.
* **The interpreter part** of the `RootSet` keeps `globals`, `cells`, classes, the environment and the
  guard table. The running activity's part stays one indexed load away.
* **Frame arenas per activity.** `FrameArena` is strict LIFO, and every block carries a guard of
  65,536 cells beyond its payload (`rexx-core/src/frame.rs:43`, `:69`), so an activity costs at least
  about 512 KiB of arena, as an oracle activity costs a 512 KiB thread stack. Each activity has its own
  arena, so LIFO holds within it; `frame.rs`'s changed invariants are re-granted (1.1).
* **Slots per activation as arena segments**, not one flat stack, so a frame below the top can grow.
  This removes the Phase 8 residual owned by Phase 6: setting, through a kept outer call context, a
  name the outer activation never bound.
* **Offsets become activation-relative.** Today several structures hold raw positions:
  `RootSet::aliases` holds absolute slot positions and asserts that targets descend
  (`rexx-core/src/roots.rs:67-71`, `:300-306`, `:334-337`); `SlotFrame.start`; `frame_starts`; and
  the `native_handles` index handed to rexx-api (`dispatch/library.rs:96`). S1 rewrites each to an
  activation identity plus an offset, resolved through the activation's own record, so REPLY can move
  frames by rewriting that record alone.
* **REPLY moves frames** from the replier's arena to the new activity's. A REPLY with Rust frames
  inside the replier (inside a non-flattened loop, INTERPRET or a CALL ON handler) cannot move and
  refuses loudly, counted, with an owner of none (recorded divergence).
* **D3 frame ownership.** No GC-visible object can hold a frame: `RegFrame<'a>` borrows its arena,
  while heap `Body` values are `'static`, so the type system refuses a frame reference in any heap
  object (review 4, I-5). RexxContext and StackFrame objects and kept call contexts hold an activation
  identity and resolve it under the baton by looking the activation up among **all** the
  interpreter's activities, not by scanning the running activity's frames as today
  (`dispatch/context.rs:136-148`): a live activation in another activity answers as the oracle's does
  (its `checkValid` tests only for null, `ContextClass.cpp:145-151`); a finished one answers the
  oracle's error. The evidence is that type-level fact plus a test of a context object read from
  another live activity, a finished one and a moved (REPLY) one.
* **Collection points.** Every allocation remains one, as a stated constraint (the only instrument
  that finds a missed root). A continuation switch and a driver exit are added; anything live across
  either must be rooted, and **no island value is held across a driver exit** (P6-3).
* **Parallel-ready hooks.** `collect` calls the scheduler's `stop_the_world`, which today asserts that
  the caller holds the baton. A future collector replaces that one function. A stress mode collects at
  every countdown visit, switch and driver exit.
* **Isolation at the type level.** `ObjRef` becomes `!Send` and `!Sync` (a phantom raw-pointer marker;
  a scratch build confirmed nothing sends one today), so the compiler refuses an object handle in the
  inbox, a completion or another interpreter.
* **Sharing-fraction instrument.** A feature-gated build tags each object with the last activity that
  resolved it and counts objects touched by more than one activity.

---

## 6. Rexx semantics

Oracle behaviour is Read or Measured in the reviews unless marked *(verify)*, which the plan settles
in the first task that needs it, with the command recorded.

* **Guard locks.** Per (object, scope): owning activity, nesting count, FIFO waiters, in a
  per-interpreter table keyed by object identity. Guarded methods reserve on entry; unguarded ones
  skip. Guarded natives (Rust or library: the oracle's Stream methods are guarded EXTERNALs,
  `StreamClasses.orx:127-146`) reserve on entry and keep the reservation across a driver exit, which is
  also what makes the blocking half's exclusive use of a stream's state safe. The owner re-enters by
  bumping the count (same activity stack, nested attaches included, `VariableDictionary.cpp:527-533`).
  A contended reserve parks. **Deadlock detection:** a contested reserve, and `Message~wait`/`~result`,
  follow the chain of what each waiting activity waits on and raise 98.905 on a cycle
  (`Activity::checkDeadLock`, `concurrency/Activity.cpp:2000-2027`); GUARD WHEN waits and waits on an
  unsent message block forever (`rust/corpus/oracle-crashes.txt` entry 7, and its
  `Message~result` entry after entry 8).
* **GUARD ON/OFF and WHEN.** ON reserves, OFF releases **one** nesting level. WHEN registers the
  activity as a watcher of each **variable named in its expression**
  (`instructions/GuardInstruction.cpp:141-144`), releases one level, and parks; 99.913 for a WHEN
  naming no exposed variable is kept. A **set or DROP** of a watched variable notifies
  (`RexxVariable.hpp:71-77`, `RexxVariable.cpp:158-169`). A compound element store does not notify:
  the oracle's parser watches only the stem and the simple tail variables a WHEN names (Phase 6 S3
  Task 12, oracle 30 of 30). The barrier sits on every path that stores or
  drops an object variable: the pool path (`set_exposed_variable`, `rexx-exec/src/variables.rs:94`),
  stem element stores, attribute setters and `SetObjectVariable` (`dispatch/library.rs:718`); it is
  one "watched" test on those paths only, and is measured. The watched mark is **sticky for stores**,
  as the oracle's `dependents` table stays once created (`RexxVariable.cpp:137-150`): every later
  **set** of a once-watched variable notifies, while a DROP notifies only while watchers remain
  (`RexxVariable.cpp:161-164`). The oracle's notifier then yields mid-clause (`RexxVariable.cpp:192-193`),
  but its `yieldControl` hands over only to an activity already queued for the kernel lock, which a
  just-woken waiter is not; here a notify readies the waiter and asks for no switch, and the waiter
  runs at the notifier's next wait, end or slice (ruling P44, a licensed cadence difference of the
  P32 class).
  Each re-evaluation is traced as the oracle traces it (`GuardInstruction.cpp:176`, `:183`).
* **REPLY** splits the continuation: the caller resumes with the reply value; the rest becomes a new
  activity (its frames move, section 5, keeping the activation's `invocation` number, which
  RexxContext.testGroup:236 asserts, `dispatch/context.rs:152-160`). The guard lock moves only at
  nesting count 1
  (`VariableDictionary::transfer`, `VariableDictionary.cpp:600-617`); otherwise the continuation
  reserves again (`execution/RexxActivation.cpp:766-773`, `:561-568`). REPLY then yields (`:776`), as a
  switch request at the next boundary. GUARD.testGroup `:258-279` and `:290-330` depend on these rules.
  The deferral of replies to program end, and the refusal of REPLY inside DO/SELECT/IF, go (a REPLY
  that cannot move refuses as section 5 says). An untrapped error in a REPLY continuation shows
  through `.traceOutput` (RAISE.testGroup:470-487).
* **Message objects.** `~start`, `~startWith` and `Object~start`/`~startWith` spawn an activity
  making the send, without the caller's NUMERIC settings (`RexxCode.cpp:207`,
  `RexxActivation.cpp:152-158`); a method's ADDRESS comes from the instance default (`:171`).
  `~result`, `~wait` and `~completed` read or park on the completion; `~reply` and `~replyWith` are
  implemented (`classes/MessageClass.cpp:562-640`); `~notify` is synchronous (`:209-226`, `:660-672`).
  An untrapped error in a started send prints its traceback on the started activity at once,
  `~hasError` answers 1, and `~result` re-raises (`MessageDispatcher.cpp:64-72`; measured).
* **MutexSemaphore and EventSemaphore** (the classes) are per-interpreter objects. A MutexSemaphore's
  owner is the activity; it is re-entrant for its owner and released when the owning activity ends or
  is discarded (`cleanupMutexes`, `concurrency/Activity.cpp:258`, `:292`;
  `classes/MutexSemaphore.cpp:230-262`); after REPLY it stays with the caller.
* **RexxUtil `Sys*Sem`** are not the classes. On Unix, `SysCreateMutexSem` and friends are counting
  POSIX semaphores: no owner, no re-entry, timeout 0 meaning forever, return codes 121 and 6
  (`platform/unix/SysRexxUtil.cpp:868-903`, `:945-1025`). Unnamed ones are built here as parkable
  natives with those semantics; **named** ones use `sem_open` (`:668`), shared between processes, and
  belong to the RexxUtil remainder of Phase 10 (roadmap D11: "the remainder in Phase 10"). No ooTest
  file uses `Sys*Sem`, so the corpus witnesses them.
* **Alarm and Ticker** are `CoreClasses.orx` code (near 1560-1575 and 1660-1672): REPLY, then the
  native `!startTimer` or `!waitTimer`, today deferred at `dispatch/native.rs:116-120`. The natives are
  parkable timed waits with early wake on cancel; firing runs on the replied activity, and program end
  waits for it.
* **Thread identity and guard state.** `.context~thread` answers the **activity's number**, assigned
  as the oracle assigns its thread numbers: main is 1; a new activity takes the oldest number from a
  bounded FIFO of numbers freed by ended activities (the oracle's thread pool is a FIFO,
  `availableActivities`, `concurrency/ActivityManager.cpp:556`, `:650-662`, bounded by
  MAX_THREAD_POOL_SIZE, `ActivityManager.hpp:358`), else the next unused number (`getIdntfr`,
  `concurrency/Activity.cpp:105-112`). Measured on the oracle: two sequential `~start`s both answer 2;
  with B and C running concurrently and D started after both end, `main 1 D 3 B 2 C 3` (5 of 5). Live
  activities stay distinct (RexxContext.testGroup:223-227, TRACE_TraceObject.testGroup:324). TraceObjects carry THREAD, CALLERSTACKFRAME, ISGUARDED,
  SCOPELOCKCOUNT, HASSCOPELOCK and ISWAITING as the oracle sets them (`RexxActivation.cpp:5160-5230`),
  not today's fixed values (`rexx-exec/src/environment/route.rs:216-221`).
* **UNINIT** runs on the ending activity when it ends, and at interpreter termination
  (`concurrency/Activity.cpp:249`, `:321-324`; `InterpreterInstance.cpp:571`) *(verify the exact
  ordering)*.
* **Program end** waits for every activity that is not pooled, including ones that never finish (a
  blocked WHEN, an uncancelled Ticker), then collects and runs uninits; the exit status is the main
  program's (`InterpreterInstance::terminate`, `:521-573`; measured).

---

## 7. Performance rule

* **Instrument.** Callgrind instruction counts, glibc and ld-linux excluded, each build in its own
  target directory with a `Compiling` line; and wall clock, five interleaved runs, median.
* **Programs.** `rexxcps`, `emptyloop` and `varlookup`, plus, committed as benchmark programs in S0:
  a recursive fib(22) by CALL, the same by function, a send loop, and an extension-call loop through
  `liborxfunction` (the cost of a driver exit per native call).
* **Noise.** Three layout controls (dead code of increasing size) are measured against the base once
  per stage; a program's noise band is the largest control delta.
* **Budget.** Every figure is a **running total against the Phase 6 base commit**, measured once per
  stage against the base with that stage's noise band: after S0 at most +0.3% beyond the band on
  every program, after S1 at most +0.3%, after S2 at most +0.6%, after S3 at most +0.8%, after S4 at
  most +1.0%. Wall clock may not regress beyond ±4% (the recorded layout noise) on any program, and the
  extension-call loop's wall clock is also recorded against the oracle.
* **Stopping rule.** A **round** is one committed candidate change measured on every program. A stage
  over its budget gets at most three rounds; if still over, it stops, and the figures go to Moritz for
  a ruling: accept the cost, or spend more rounds. S1 is the likeliest to need them (the spike was
  +4.5% on rexxcps and +10% on fib as built, with the in-loop design unbuilt). There is no cheaper
  carrier to fall back to: handing an interpreter between threads is sound only at a driver exit
  (P6-3), which is what S1 builds.

---

## 8. Staging

Each stage ends with every gate green and within its budget.

| Stage | Delivers |
|---|---|
| S0 Foundations | The scheduler seam; per-activity state split out of `Interp` with the exhaustive classification (section 5); per-activity frame arenas; `ObjRef` `!Send`/`!Sync`; the process-global `static FLAT` (`run/loops.rs`, marked SPIKE, already queued) removed; the countdown reload bounded; the benchmarks of section 7 committed. No behaviour change, no second activity. |
| S1 Stackless calls | The resumable entries of 2.1 (sends in expressions and call and send arguments compiled to ops included); the park-point channel for `Op::Exec` arms; plain DO flattened; mid-region resume through a separate entry; parked state in the arena; activation-relative offsets (section 5); slots per activation segment. |
| S2 Activities, single driver thread | The scheduler (section 3), request bits and inbox, timer thread, deterministic switch mode; parkable natives and their composition; `Message~start`/`~result`/`~wait`/`~reply`/`~replyWith`/`~notify`; REPLY as a split with frame moves; SysSleep and the native timers as parks; pinned waits and slice deferral (2.6, P6-4); REPLY's Split from its `Op::Exec` arm; activity numbers and TraceObject fields; program-end and UNINIT rules. Native calls and blocking Rust operations still run on the baton here. |
| S3 Synchronisation | Guard locks with deadlock detection, parking at the send's begin half; GUARD ON/OFF/WHEN parking from their `Op::Exec` arms, and the barrier; the classes MutexSemaphore and EventSemaphore; unnamed `Sys*Sem` as parkable natives; Alarm and Ticker. |
| S4 The baton | Driver exits for native calls and blocking Rust operations, with completions through the inbox; `Host` as a baton-guarded accessor; per-activity API contexts and native-handle stacks with the per-callback thread check (2.4); the driver pool with its stack, bound and fallback (2.7); callback priority and delegation (2.4); lent island memory (2.5); the `Send` grant (2.5); signals (R3, R5); pinning counters and report. |
| S5 Close | Section 9. |

---

## 9. Exit criteria

1. **The concurrency tests pass** on the crate and agree with the oracle through a Phase 8-style
   instrument. The list is **derived per test method** by a committed command: every ooTest test
   method whose body, or a helper it calls within its group, uses REPLY, `~start`, `~startWith`, GUARD,
   the semaphore classes, `Sys*Sem`, Alarm, Ticker, SysSleep, `.context~thread` or TraceObject thread
   and guard fields. Exclusions are committed with a reason each: tests changing rxapi's persistent
   state; RexxQueue tests (every RexxQueue method is Phase 10's, `dispatch/native.rs:260-269`); tests of
   extensions and samples that Phase 10 recompiles (`extensions/rxsock`, `samples`). The derived
   list's groups include at least `base/keyword/GUARD`, `base/keyword/REPLY`, `base/class/Message`,
   `base/class/EventSemaphore`, `base/class/MutexSemaphore`, `base/class/Alarm`, `base/class/Ticker`,
   `regressions/bug2003_guard_when`, `base/rexxutil/SysSleep`, `base/keyword/TRACE_TraceObject`,
   `base/class/RexxContext`, `base/keyword/TRACE`, `base/keyword/RAISE`, `base/class/Object`,
   `base/directives/METHOD`, `base/special.variables/RESULT_RC_SIGL` and
   `doc/rexxref/chapter5/Section1`.
2. **The ooTest framework's ticker** (`ooTest.frm`, a REPLY thread and GUARD ON WHEN) runs without
   `-U`; L2 then remains blocked only by Phase 10's RXFUNCQUERY.
3. **Race checking (R4):** a ThreadSanitizer build of the test suite with the installed nightly is
   clean over every cross-thread path (baton, inbox and completions, timer, driver pool, signals), and
   `loom` over the shim-built baton, inbox and timer passes.
4. **The single-owner audit**, replacing roadmap row 6's lock-dependence wording under R1: an inventory
   of every `unsafe impl Send` and every cross-thread type, each shown by construction (a type-level
   fact) where possible, as the roadmap asks, and by a test only where no type-level form exists, that
   no interpreter state is reachable from a thread not holding that interpreter's baton.
5. **D3 frame ownership** (roadmap :2646) is shown by the type-level facts and the test of section 5.
6. **The sharing fraction** is measured over the corpus and ooTest and recorded.
7. **A ping-pong benchmark** (message round trip, semaphore post/wait, GUARD WHEN handoff) is recorded
   against the oracle, not gated.
8. **No refusal names Phase 6.** Every `Loud` text, layout owner, native entry owner and exclusions row
   naming Phase 6 is gone or re-homed with a true reason. This covers at least the GUARD WHEN wait,
   REPLY inside a construct, `Message~result` of an unsent message, the Alarm and Ticker timer entries,
   the `Sys*Sem` routines, `SetGuardOnWhenUpdated`/`SetGuardOffWhenUpdated`, the kept-outer-context
   residual, the L2 ticker rows, and the Message and semaphore methods that fall through to the generic
   native-method refusal today. The enumeration is derived by a committed command, shown empty at the
   close, and enforced: `closed_phases` gains Phase 6. The divergences this design records (inverted
   pinned waits, an immovable REPLY, wrappers blocking on the baton, HALT under nesting, a pinned
   busy-waiter, stale C writes to reallocated lent storage, the `nohup` signal difference) name no phase:
   each is an exclusions row with an owner of none and its reason.
9. **Pinned waits are measured and bounded.** The pinning report over the corpus and the tests of
   criterion 1, run normally and under the deterministic switch-at-every-opportunity mode, lists every
   pinned park, deferred slice, immovable REPLY and inverted-wait refusal by kind; the tests of
   criterion 1 contain **no** inverted-wait refusal and no hang in either mode.
10. Every *(verify)* item of section 6 is settled against the oracle.

**Testing rule.** Corpus programs stay deterministic (`corpus/README.md`), so they witness only
orderings the oracle itself fixes. Interleaving-dependent behaviour is tested on the crate alone, under
the deterministic switch mode.

---

## 10. For the later phase: moving values between interpreters

Recorded here because P6-8 defers it. ooRexx offers no user-facing object serialization: queues carry
strings, and rxapi queues hold byte strings. The interpreter has an internal graph flattener
(`memory/Envelope.cpp`, `SmartBuffer.cpp`) used for `rexxc` images and `rexx.img`; it preserves
identity and cycles but assumes the same interpreter build reads it back. Elsewhere, from cheapest to
most general: values only (Tcl threads, rxapi queues); structured clone of a fixed type set, refusing
behaviour (JavaScript workers); copy, move or share-if-deeply-frozen (Erlang, Ruby Ractors, Python
PEP 734 "shareable" types); and full pickling with per-class hooks. The natural fit for Rexx is
structured clone over its own types (strings, numbers, Array, Directory, StringTable, Stem, Bag) plus
an opt-in pair of methods for user classes.

---

## 11. Settled in the plan's first tasks

Review round 4 found these small enough to settle while planning, with their evidence recorded there:

* S1's per-site node addressing for calls and sends in argument position (`NodePath` is binary,
  `ir.rs:309-319`; arguments compile with `path: None`, `ir/compile.rs:918`, `:1417`) and the leaf
  kinds `native_shape` must accept (DotVariable, List and the others), with S1's performance risk
  (the spike's +520 and +585 Ir per stackless send) measured on the first send-compilation step.
* **An early pinned-park instrument**, counting pinned parks per kind over the criterion-1 tests on
  today's code, run before S1 closes, so criterion 9's risk is measured rather than inferred.
* The self-pipe's details (`UnixStream::pair`, a read timeout, arming through the same socket, a
  nonblocking write end, errno saved in the handler), the record the Global Constraints require of a
  safe alternative tried (`signal-hook` chains to prior handlers where this design, like the oracle,
  installs only where none is set), and ignoring SIGPIPE as the oracle does
  (`platform/unix/SystemInterpreter.cpp:146-148`) for an embedded interpreter.
* Activities migrating between OS threads: successive native calls of one activity may land on
  different threads (a thread-affine extension behaves differently from the oracle), and the Error 11
  depth on a pool thread depends on scheduling. Both are recorded divergences.
