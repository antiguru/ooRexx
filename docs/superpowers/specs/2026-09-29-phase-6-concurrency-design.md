# Phase 6: concurrency, as isolated interpreters with continuation activities

**Status:** design, agreed with Moritz in conversation on 2026-09-29 section by section, then
revised after review rounds 1 and 2 (`spec-review-1.md`, `spec-review-2.md` in
`docs/superpowers/records/2026-09-29-phase-6-design/`) with his rulings R1 to R4 (1.1). It replaces
the delivery order of the roadmap's D3 option (a), a coarse process-wide kernel lock. Roadmap row 6
(`docs/superpowers/plans/2026-07-27-rust-rewrite.md`) remains the exit contract, read through this
document.

**Inputs.** `docs/superpowers/specs/2026-09-08-concurrency-research-direction.md`; the records in
`docs/superpowers/records/2026-09-29-phase-6-design/` (`yield-spike-findings.md`,
`stackless-spike-findings.md` with `stackless-spike.patch` and `stackless-reentry-sites.txt`, and
both reviews). Oracle source paths are relative to `/home/moritz/dev/repos/ooRexx/interpreter`.

---

## 1. Decisions

| # | Decision | Why |
|---|---|---|
| P6-1 | **Isolation.** One interpreter owns its heap, collector, classes, `.environment`, `.local` and scheduler. Interpreters share nothing. | No process-wide lock, by construction. Shared-heap runtimes paid with a per-operation shared structure (research direction, section 5). |
| P6-2 | **Activities are continuations.** An activity's state is arena data run by a driver loop; Rexx-to-Rexx calls swap frames inside that loop. | A switch on a slice or a Rexx-level wait is a pointer swap; REPLY is a continuation split. It is also what makes handing an interpreter between OS threads sound in Rust (2.5). Precedents: CPython 3.11, Lua. |
| P6-3 | **A per-interpreter baton, and no borrow of interpreter state across its release.** Holding the baton is the right to touch interpreter state. It is released where the oracle releases its kernel lock: around calls into extension C code and around blocking operations (`execution/NativeActivation.cpp:1303-1306`, `:1420-1423`, `:1541-1544`, `:1690-1692`; `api/ContextApi.hpp:64-77`). A release happens only at a **driver exit**, where no Rust frame holds a borrow of the interpreter, its arenas or a `RefCell`. | Under Stacked and Tree Borrows a `&mut` function argument is protected for the call, so a thread still holding one cannot let another thread touch the same state. Handing the baton from inside nested Rust frames would force returns into LIFO order across threads and deadlock programs the oracle runs (review 2, C-A). The Go runtime's processor handoff around a system call has the same shape. |
| P6-4 | **Pinning.** An activity with native frames on its stack (native code that re-entered Rexx) is pinned to its OS thread and **never releases the baton**. A pinned activity that must wait runs a nested scheduler on its own stack (other activities run as continuations until its wait is satisfied); a native call made while pinned keeps the baton. | Loom's pinning, and Lua's "cannot yield across a C-call boundary". Nesting is LIFO, so a wait inverted against the nesting cannot complete: it is detected and refused loudly, and counted (2.6). Correctness therefore rests on pinned waits being rare, which exit criterion 9 measures. |
| P6-5 | **A scheduling seam.** Nothing outside the scheduler knows how an activity is carried. | Keeps the carrier replaceable on measurement. |
| P6-6 | **An RBED-shaped scheduler shipping round-robin.** Dispatch is separate from allocation; Phase 6 ships the uniform policy only. | Uniform allocation with arrival-order tie-breaks is round-robin, which parity needs; rates or budgets become a policy swap. |
| P6-7 | **The slice comes from the existing clause countdown; the slice is a request bit set by a timer thread.** | No new hot-path branch; one mechanism for slice, HALT, signals and cross-thread requests; tests set the bit themselves. |
| P6-8 | **Out of scope:** the Rexx-level API for user-managed interpreters, and exchanging data between interpreters over queues. | A later phase, with the rxapi integration (Moritz, 2026-09-29). Section 10 records its starting precedents. |

### 1.1 Rulings recorded with this design (Moritz, 2026-09-29)

* **R1. D3 is narrowed to process-wide.** Per-interpreter serialization (the baton, as PEP 684's
  per-interpreter lock) is allowed; no invariant may rest on one activity running at a time across
  the process. D3's rule that cross-activity requests go through a channel or an atomic flag, never a
  foreign frame pointer, is unchanged and holds inside one interpreter too.
* **R2. `.environment` per interpreter is a licensed divergence for Phase 9.** The oracle's is
  process-wide (`RexxCore.h:267`, `Setup.cpp:300`); only `.local` is per instance
  (`runtime/InterpreterInstance.cpp:208`). A single-interpreter program cannot observe the difference.
* **R3. Signal handling is a new `unsafe` site, not a new dependency.** One module installs the
  handlers through the libc already linked; a handler only sets atomics in an async-signal-safe
  registry.
* **R4. Race checking** is a ThreadSanitizer run with the installed nightly toolchain (gate-only,
  never for shipped builds) plus `loom` over the baton, inbox and timer protocol, admitted as a
  dev-dependency. Moritz had no preference among nightly, `loom` and `shuttle`: TSan sees data races
  in the real binary; `loom` exhaustively finds the lost-wakeup and handoff bugs a baton protocol
  invites; the deterministic switch mode (section 4) covers interpreter-level interleavings, which is
  shuttle's niche. The baton and inbox are written against a `cfg(loom)` synchronisation shim, so
  `loom` checks the shipped code, not a transcription of it.

The roadmap records these rulings (R1 and R4 in D3's dated note already; R2 and R3 added there by
the plan's first task, with the Section 1 decision blocks the Global Constraints require before code
for the new `unsafe` sites of 2.5 and R3 and for `frame.rs`'s changed invariants, row 6's "kernel
lock" wording read through R1, and the Global Constraints' nightly line amended for R4).

---

## 2. Architecture

### 2.1 Units

* **Interpreter.** The unit of isolation. It owns the heap, collector, classes,
  `.environment`/`.local`, the guard table, the scheduler, the baton, a pool of driver threads, and an
  inbox (an atomic request word plus a mutex-guarded payload queue). The baton and the inbox are the
  only parts a thread touches without holding the baton.
* **Activity.** A continuation: an arena-resident chain of activations with their IR register frames,
  the parked driver state (resume point and pending result register), its request bits, its API
  thread context (2.4), and its per-activity state (section 5). It is named by an offset handle.
* **Driver.** A resumable call op pushes the callee's frame and continues in the same loop
  iteration; RETURN pops it and delivers the result into the caller's register. Resumption lands
  mid-region (the call's result feeds later ops of the same clause, `x = f(1) + 2`), through a
  separate resume entry, so no per-clause resume test exists (the stackless spike measured that test
  at +3.5% on emptyloop). The depth cap raising Error 11 stays, for parity.
* **Resumable entries.** Internal CALL and function calls; `::ROUTINE` calls; `Op::Send`;
  `Op::Message` (instruction-form sends); `~new` into INIT; `Message~send`/`~sendWith` and
  `Object~send`/`~sendWith` (every ooTest body runs through `.message~new(self, name)~send`,
  `OOREXXUNIT.CLS` near line 1583, today recursive at `dispatch/object_protocol.rs:893`); a started
  activity's first send (today `started_message` -> `send_message`, `object_protocol.rs:824`); and the
  REPLY continuation's resume (today `resume_reply` -> `run_activation`, `dispatch.rs:2528`). Without
  the last two, every started activity would be pinned from birth.
* **Parkable natives.** A native implemented in Rust may answer `Park(reason)` instead of a value;
  the driver parks the activity, and on wake re-enters the native's continuation half, which delivers
  the result into the caller's register. `Message~result`/`~wait`, `MutexSemaphore~request`,
  `EventSemaphore~wait`, the native timers `!startTimer`/`!waitTimer`, SysSleep and the unnamed
  `Sys*Sem` waits are parkable natives, so none of them pins and none holds the baton while waiting.
* **Blocking Rust builtins** (stream and console I/O, commands) split into prepare, block and finish:
  prepare runs under the baton, the driver exits, block runs on the activity's thread holding no
  borrow and no baton, finish runs under the baton again (P6-3).
* **Scheduler seam.** An internal trait: `spawn`, `run_until_park`, `park(reason)`,
  `unpark(activity)`, `yield_at_slice`, `exit_for_native`/`exit_for_block` (the baton-releasing driver
  exits), `request_baton` and `stop_the_world`. It also admits a foreign thread joining as a new
  activity (`AttachThread`), which Phase 9 implements.
* **Remaining pinned re-entry sites.** Sort comparators, conversions, UNKNOWN, FORWARD, trap
  handlers, native-API callbacks, library-program entry, calls in loop headers, non-flattened loops,
  INTERPRET, and calls inside call arguments. Counted per kind; converting one is a later measured
  step.

### 2.2 Data flow

The scheduler takes the next ready activity; the driver runs it until it parks, its slice ends, it
exits for a native call or a blocking operation (releasing the baton), or it completes. Parks and
preemptions cost a pointer swap; a release lets a pooled driver thread take the baton if activities
are ready. A thread whose native call or blocking operation returns requests the baton through the
inbox (I-B below), then continues its activity's continuation from the driver loop. Inbox events
(baton requests, cross-thread requests, signals) are drained whenever control reaches the scheduler.

### 2.3 The invariant

**Exactly one activity touches interpreter state at a time, per interpreter**: the holder of that
interpreter's baton. Other activities may be running extension C code or blocked in a system call
concurrently, off the baton, as in the oracle. Per-interpreter serialization under R1.

### 2.4 The native-API context is per activity

Today one `rexx_api::ffi::ThreadContext` serves the interpreter (`rexx-exec/src/lib.rs:1314`, built
at `:2056`), with a `home` thread asserted only by `AttachThread` and `AddCommandEnvironment`
(`rexx-api/src/ffi.rs:3490`, `:3532`), an `innermost` activation cell saved and restored in strict
nesting (`ffi.rs:508-525`), and `native_handles` a per-interpreter stack, "innermost last"
(`lib.rs:1273-1280`, pushed at `dispatch/library.rs:475`). So:

* each activity gets its own thread context and native-handle stack, and each activation its own
  call or method context, as in the oracle (`ActivationApiContexts.hpp`);
* the interpreter's `RexxInstance`, and the context of the activity that loaded a library, outlive
  that activity, since libraries keep them (`install.rs:1988`, `rexx-api/src/load.rs:809`) and
  extensions cache the instance for `AttachThread`;
* `Host` becomes a baton-guarded accessor: each callback acquires the baton on behalf of its
  context's activity, derives a fresh `&mut` from the interpreter's root pointer, and drops it before
  returning to C; returns and callbacks may therefore occur in any order;
* a callback from a thread that is not its activity's raises Error_Execution_invalid_thread, after
  taking the baton, as the oracle's `Activity::validateThread` does (`ContextApi.hpp:73-76`,
  `concurrency/Activity.cpp:3620-3626`).

### 2.5 The `Send` grant

Moving the driver between OS threads needs `unsafe impl Send` for the interpreter island (its `Rc`
and `RefCell` interior). Its SAFETY argument: the island is reachable only through one root pointer
and the baton; a thread derives a `&mut` from the root only while holding the baton, and the baton is
released only at a driver exit, where the releasing thread holds no borrow (P6-3). So no two threads
ever hold live borrows of the island, and every borrow a thread derives is created after the previous
holder's last one died. The grant lands with its first user (S4) in a module the plan names, recorded
as a Section 1 decision block with R3's signal module; `unsafe` is otherwise confined to
`rexx-api/src/ffi.rs`, `rexx-api/src/load.rs`, `rexx-core/src/bytes.rs` and
`rexx-core/src/frame.rs`. The `thread_local!`s `REFUSED` (`rexx-api/src/layout.rs:423`) and
`HOOK_THREW` (`rexx-api/src/load.rs:678`) belong to one native call's own stack and stay per OS
thread; every other `thread_local!` holding interpreter state becomes per-interpreter or
per-activity.

### 2.6 Pinned waits

A pinned activity that parks runs a nested scheduler loop on its own stack until its wait is
satisfied; other activities run inside it as continuations. A pinned activity whose slice expires
likewise runs the nested loop for one slice. If a nested activity waits on something only an
enclosing pinned activity can provide (an inverted wait), neither can proceed: the nested scheduler
detects that nothing it can run will satisfy the innermost wait and the chain includes an enclosing
pinned activity, and raises a loud refusal naming the pinned re-entry kind. This is a known
divergence from the oracle, whose every activity has its own OS thread; it is counted and reported,
and exit criterion 9 bounds it.

### 2.7 Driver threads

The interpreter's first thread keeps today's large reservation (`INTERPRETER_STACK_BYTES`,
`lib.rs:183`, `:3180`). Pooled driver threads get a smaller stack, sized in S4 by measurement so that
pinned recursion on one still reaches Error 11 at the depth cap; the oracle's activity threads are
512 KiB (`common/platform/unix/SysThread.hpp:67`), and stack reservations count against the
`ulimit -v` the oracle wrapper applies (`rust/CLAUDE.md`, the stack-reservation note). The pool has a
bound. When a spawn fails or the bound is reached, a release with ready activities falls back to
running them on the next thread that takes the baton; an activity blocked in C then simply holds its
own thread, as in the oracle.

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
  any interpreter can receive a request; the spike measured the cold path at N = 50 at +0.27% on
  emptyloop and +0.06% on rexxcps, and N = 1024 is measured in S0.
* **Timer thread.** One per process. An interpreter arms it when it has a second ready activity, a
  due sleeper, or a pending baton request; with nothing armed it sleeps with no deadline. It sets the
  SLICE bit after the oracle's 24 ms and wakes idle interpreters for due sleepers. SLICE switches at
  that clause boundary, never mid-clause. Precedent: CPython's `eval_breaker`, Ruby's timer thread.
* **Baton requests.** A thread returning from C or from a blocking operation, or making a callback,
  requests the baton through the inbox, which sets the request word and arms the slice. A callback's
  request forces a switch at the holder's next cold visit, without waiting out the 24 ms, as the
  oracle prioritises API callers (`requestApiAccess` -> `addWaitingApiActivity`,
  `concurrency/Activity.cpp:2235-2249`, `:339`; `relinquishIfNeeded`,
  `concurrency/ActivityManager.hpp:303-323`); a return queues behind the slice.
* **Requests to an activity** (HALT, TRACE toggle, a raise) from inside the interpreter set the
  target's request bits and wake it if parked; the target applies them through `pending_traps` at its
  next clause boundary. A request from another OS thread goes through the interpreter's inbox (it
  never writes arena data), and the holder applies it the same way. Nobody touches another
  activity's frames (D3).
* **Signals.** Handlers for SIGINT, SIGTERM and SIGHUP are installed at interpreter start, and only
  where no handler is already set, as the oracle's library does
  (`platform/unix/SystemInterpreter.cpp:95-110`, `:130-145`), without SA_RESTART
  (`SystemInterpreter.cpp:127`). A handler sets the halt bit in every live interpreter through an
  async-signal-safe registry (atomics only). Every activity with frames (the oracle's
  `isActive()`, nestedCount > 0, `Activity.hpp:249`; `InterpreterInstance.cpp:686-703`) takes HALT:
  a running one at its next cold visit, a parked or sleeping one when woken, and it is woken, as
  SIGINT interrupts the oracle's `nanosleep` (measured: `rc = SysSleep(5)` under `timeout -s INT 1`
  halts at about 1 s with Error 4.1). The registry and the timer thread are the one exception to "no
  mutable process-global state" (R3).
* **Long native builtins** such as SysStemSort run no clauses and are not halted mid-work, on the
  oracle either; run as blocking Rust builtins (2.1), they do not hold the baton.
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
  `native_spares`, `value_buffer` (cleared at every clause start, `clause.rs:242`),
  `running`/`suspended`, `clause_state`, `frames`, `flat_top`, `pending_traps`, `active_condition`,
  `fragments`, `depth`, `call_context`, `indents`, and `random_seed` (per activity in the oracle,
  `concurrency/Activity.hpp:426`). The exhaustive destructure in `Interp::object_roots` (`lib.rs`
  near 2682) is the checklist: every field is classified interpreter or activity, and a new field is a
  compile error until classified.
* **The interpreter part** of the `RootSet` keeps `globals`, `cells`, classes, the environment and
  the guard table. The running activity's part stays one indexed load away.
* **Frame arenas per activity.** `FrameArena` is strict LIFO, and every block carries a guard of
  65,536 cells beyond its payload (`rexx-core/src/frame.rs:43`, `:69`), so an activity costs at least
  about 512 KiB of arena, as an oracle activity costs a 512 KiB thread stack. Each activity has its own
  arena, so LIFO holds within it; `frame.rs`'s changed invariants are re-granted (1.1).
* **Slots per activation as arena segments**, not one flat stack, so a frame below the top can grow.
  This removes the Phase 8 residual owned by Phase 6: setting, through a kept outer call context, a
  name the outer activation never bound.
* **REPLY moves frames.** The replying activation's frames move from the replier's arena to the new
  activity's. Anything holding an offset handle to them (RexxContext and StackFrame objects, a kept
  call context, a parked resume point) holds it through an activation identity plus offset that the
  move rewrites in one place: the activation's own record. No other structure stores a raw offset
  into an arena.
* **Collection points.** Every allocation remains one, as a stated constraint (the only instrument
  that finds a missed root). A continuation switch and a driver exit are added; anything live across
  either must be rooted. The Rust-level rule beside it: **nothing borrowed from `Interp`, an arena or
  a `RefCell` is live across a driver exit** (P6-3).
* **Parallel-ready hooks.** `collect` calls the scheduler's `stop_the_world`, which today asserts
  that the caller holds the baton. A future collector replaces that one function. A stress mode
  collects at every countdown visit, switch and driver exit.
* **Isolation at the type level.** `ObjRef` becomes `!Send` and `!Sync` (a phantom raw-pointer
  marker; a scratch build confirmed nothing sends one today), so the compiler refuses an object
  handle in the inbox, a helper message or another interpreter.
* **Sharing-fraction instrument.** A feature-gated build tags each object with the last activity
  that resolved it and counts objects touched by more than one activity.

---

## 6. Rexx semantics

Oracle behaviour is Read or Measured in the reviews unless marked *(verify)*, which the plan
settles in the first task that needs it, with the command recorded.

* **Guard locks.** Per (object, scope): owning activity, nesting count, FIFO waiters, in a
  per-interpreter table keyed by object identity. Guarded methods reserve on entry; unguarded ones
  skip. The owner re-enters by bumping the count (same activity stack, nested attaches included,
  `VariableDictionary.cpp:527-533`). A contended reserve parks. **Deadlock detection:** a contested
  reserve, and `Message~wait`/`~result`, follow the chain of what each waiting activity waits on and
  raise 98.905 on a cycle (`Activity::checkDeadLock`, `concurrency/Activity.cpp:2000-2027`); GUARD
  WHEN waits and waits on an unsent message block forever (`rust/corpus/oracle-crashes.txt` entry 7).
* **GUARD ON/OFF and WHEN.** ON reserves, OFF releases **one** nesting level. WHEN registers the
  activity as a watcher of each **variable named in its expression**
  (`instructions/GuardInstruction.cpp:141-144`), releases one level, and parks; 99.913 for a WHEN
  naming no exposed variable is kept. A **set or DROP** of a watched variable notifies
  (`RexxVariable.hpp:71-77`, `RexxVariable.cpp:158-169`), including a compound element of an exposed
  stem (`expression/ExpressionCompoundVariable.cpp:348`), and the notifier then yields
  (`RexxVariable.cpp:192-193`). The barrier sits on every path that stores or drops an object variable:
  the pool path (`set_exposed_variable`, `rexx-exec/src/variables.rs:103-125`), stem element stores,
  attribute setters and `SetObjectVariable` (`dispatch/library.rs:718`); it is one "watched" test on
  those paths only, and is measured. The watched mark is **sticky**, as the oracle's `dependents`
  table stays once created (`RexxVariable.cpp:137-150`), so every later store to a once-watched
  variable notifies and yields. Each re-evaluation is traced as the oracle traces it
  (`GuardInstruction.cpp:176`, `:183`).
* **REPLY** splits the continuation: the caller resumes with the reply value; the rest becomes a new
  activity (its frames move, section 5). The guard lock moves only at nesting count 1
  (`VariableDictionary::transfer`, `VariableDictionary.cpp:600-617`); otherwise the continuation
  reserves again (`execution/RexxActivation.cpp:766-773`, `:561-568`). REPLY then yields (`:776`).
  GUARD.testGroup `:258-279` and `:290-330` depend on these rules. The deferral of replies to program
  end, and the refusal of REPLY inside DO/SELECT/IF, go. An untrapped error in a REPLY continuation
  shows through `.traceOutput` (RAISE.testGroup:470-487).
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
  belong to the RexxUtil remainder of Phase 10 (roadmap D11: "the remainder in Phase 10").
* **Alarm and Ticker** are `CoreClasses.orx` code (near 1560-1575 and 1660-1672): REPLY, then the
  native `!startTimer` or `!waitTimer`, today deferred at `dispatch/native.rs:116-120`. The natives are
  parkable timed waits with early wake on cancel; firing runs on the replied activity, and program end
  waits for it.
* **Thread identity and guard state.** `.context~thread` numbers OS threads, and the oracle reuses
  pooled threads (`getIdntfr`, `concurrency/Activity.cpp:105-112`; measured: two sequential
  `~start`s both answer 2, main answers 1). It answers the number of the driver or blocking thread
  the activity currently runs on, with threads numbered in creation order and pool threads reused;
  exact agreement with the oracle's numbers is expected only for single-thread-at-a-time histories
  and is otherwise licensed. TraceObjects carry THREAD, CALLERSTACKFRAME, ISGUARDED, SCOPELOCKCOUNT,
  HASSCOPELOCK and ISWAITING as the oracle sets them (`RexxActivation.cpp:5160-5230`), not today's
  fixed values (`rexx-exec/src/environment/route.rs:216-221`).
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
* **Programs.** `rexxcps`, `emptyloop`, `varlookup`, a recursive fib(22) by CALL and by function, a
  send loop, and an extension-call loop through `liborxfunction` (the per-call baton cost of S4),
  the last three committed as benchmark programs in S0.
* **Noise.** Three layout controls per comparison (dead code of increasing size); a program's noise
  band is the largest control delta.
* **Budget.** Cumulative against the Phase 6 base commit, each program's callgrind delta beyond its
  noise band stays within: S0 +0.3%, S1 0.0% (parity or better), S2 +0.3%, S3 +0.2%, S4 +0.2%, the
  phase +1.0% in total. Wall clock may not regress beyond ±4% (the recorded layout noise) on any
  program. Atomic and lock costs that callgrind under-counts are covered by the wall-clock check and
  the extension-call loop.
* **Stopping rule.** A **round** is one committed candidate change measured on every program. A stage
  over its budget gets at most three rounds; if still over, it stops, and the figures go to Moritz
  for a ruling: accept the cost, or spend more rounds. There is no cheaper carrier to fall back to:
  handing an interpreter between threads is sound only at a driver exit (P6-3), which is what S1
  builds.

---

## 8. Staging

Each stage ends with every gate green and within its budget.

| Stage | Delivers |
|---|---|
| S0 Foundations | The scheduler seam; per-activity state split out of `Interp` with the exhaustive classification (section 5); per-activity frame arenas; `ObjRef` `!Send`/`!Sync`; interpreter-state `thread_local!`s made per-interpreter; the process-global `static FLAT` (`run/loops.rs`, marked SPIKE, already queued) removed; the benchmarks of section 7 committed. No behaviour change, no second activity. |
| S1 Stackless calls | The resumable entries of 2.1; mid-region resume through a separate entry; parked state in the arena with activation-relative handles; slots per activation segment. |
| S2 Activities, single driver thread | The scheduler (section 3), request bits and inbox, timer thread, deterministic switch mode; parkable natives; `Message~start`/`~result`/`~wait`/`~reply`/`~replyWith`/`~notify`; REPLY as a split with frame moves; SysSleep and the native timers as parks; pinned waits by nested scheduling with inverted-wait detection (2.6); `.context~thread` and TraceObject fields; program-end and UNINIT rules. Native calls and blocking Rust builtins still hold the baton here; only they do. |
| S3 Synchronisation | Guard locks with deadlock detection; GUARD ON/OFF/WHEN and its sticky barrier; the classes MutexSemaphore and EventSemaphore; unnamed `Sys*Sem` as parkable natives; Alarm and Ticker. |
| S4 The baton | Driver exits for native calls and blocking Rust builtins; `Host` as a baton-guarded accessor; per-activity API contexts and native-handle stacks with the per-callback thread check (2.4); the driver pool with its stack size, bound and fallback (2.7); baton requests and callback priority; the `Send` grant (2.5); signals (R3); pinning counters and report. |
| S5 Close | Section 9. |

---

## 9. Exit criteria

1. **The concurrency tests pass** on the crate and agree with the oracle through a Phase 8-style
   instrument, excluding any test that changes rxapi's persistent state. The test list is **derived**
   by a committed command (every ooTest group whose source uses REPLY, `~start`, `~startWith`, GUARD,
   the semaphore classes, `Sys*Sem`, Alarm, Ticker, SysSleep, `.context~thread` or TraceObject thread
   fields) and committed with its output; it includes at least `base/keyword/GUARD`,
   `base/keyword/REPLY`, `base/class/Message`, `base/class/EventSemaphore`,
   `base/class/MutexSemaphore`, `base/class/Alarm`, `base/class/Ticker`,
   `regressions/bug2003_guard_when`, `base/rexxutil/SysSleep`, `base/keyword/TRACE_TraceObject`,
   `base/class/RexxContext`, `base/keyword/TRACE`, `base/keyword/RAISE`, `base/class/Object`,
   `base/directives/METHOD`, `base/special.variables/RESULT_RC_SIGL` and
   `doc/rexxref/chapter5/Section1`, and the non-rxapi tests of `base/class/RexxQueue`.
2. **The ooTest framework's ticker** (`ooTest.frm`, a REPLY thread and GUARD ON WHEN) runs without
   `-U`; L2 then remains blocked only by Phase 10's RXFUNCQUERY.
3. **Race checking (R4):** a ThreadSanitizer build of the test suite with the installed nightly is
   clean over every cross-thread path (baton, inbox, timer, driver pool, signals), and `loom` over the
   shim-built baton, inbox and timer passes.
4. **The single-owner audit**, replacing roadmap row 6's lock-dependence wording under R1: an
   inventory of every `unsafe impl Send` and every cross-thread type, each shown by construction
   (a type-level fact) where possible, as the roadmap asks, and by a test only where no type-level
   form exists, that no interpreter state is reachable from a thread not holding that interpreter's
   baton.
5. **The sharing fraction** is measured over the corpus and ooTest and recorded.
6. **A ping-pong benchmark** (message round trip, semaphore post/wait, GUARD WHEN handoff) is
   recorded against the oracle, not gated.
7. **No refusal names Phase 6.** Every `Loud` text, layout owner, native entry owner and exclusions
   row naming Phase 6 is gone or re-homed with a true reason. This covers at least the GUARD WHEN
   wait, REPLY inside a construct, `Message~result` of an unsent message, the Alarm and Ticker timer
   entries, the `Sys*Sem` routines, `SetGuardOnWhenUpdated`/`SetGuardOffWhenUpdated`, the
   kept-outer-context residual, the L2 ticker rows, and the Message and semaphore methods that fall
   through to the generic native-method refusal today. The enumeration is derived by a committed
   command, shown empty at the close, and enforced: `closed_phases` gains Phase 6. The inverted-wait
   refusal of 2.6 names no phase: it is a recorded divergence with an owner of none, explained in the
   exclusions.
8. Every *(verify)* item of section 6 is settled against the oracle.
9. **Pinned waits are measured and bounded.** The pinning report over the corpus and the tests of
   criterion 1 lists every pinned park, pinned slice and inverted-wait refusal by re-entry kind; the
   tests of criterion 1 contain **no** inverted-wait refusal, and any pinned park in them is listed
   with its kind.

**Testing rule.** Corpus programs stay deterministic (`corpus/README.md`), so they witness only
orderings the oracle itself fixes. Interleaving-dependent behaviour is tested on the crate alone,
under the deterministic switch mode.

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
