# Phase 6: concurrency, as isolated interpreters with continuation activities

**Status:** design, agreed with Moritz in conversation on 2026-09-29, section by section. It
supersedes the delivery order in the roadmap's D3 option (a) (a coarse process-wide kernel lock)
and keeps D3's constraint: no global interpretation lock, in the end state or in any type,
invariant or API. Roadmap row 6 (`docs/superpowers/plans/2026-07-27-rust-rewrite.md`) remains the
exit contract, read through this document.

**Inputs.** `docs/superpowers/specs/2026-09-08-concurrency-research-direction.md` (the research
direction); the spikes of 2026-09-29, recorded under
`docs/superpowers/records/2026-09-29-phase-6-design/`: `yield-spike-findings.md` (when to switch,
and what it costs) and `stackless-spike-findings.md` with `stackless-spike.patch` and
`stackless-reentry-sites.txt` (whether Rexx-to-Rexx calls can run without native recursion).

---

## 1. Decisions

| # | Decision | Why |
|---|---|---|
| P6-1 | **Isolation.** One interpreter per OS thread, owning its heap, collector, classes, `.local` and scheduler. Interpreters share nothing. | The no-global-lock end state holds by construction. The field's shared-heap runtimes paid for it with a per-operation shared structure (research direction, section 5). |
| P6-2 | **Activities are continuations.** An activity's state is arena data run by one driver loop per interpreter; Rexx-to-Rexx calls swap frames inside that loop. | Switching is a pointer swap; recursion is bounded by memory, not the native stack; REPLY becomes a continuation split. Precedents: CPython 3.11 (Python-to-Python calls off the C stack), Lua. |
| P6-3 | **Pinning, with carriers as the fallback.** Native code that re-enters Rexx runs a nested driver and pins its activity; a pinned activity that must block or be preempted is promoted to its own OS thread holding the interpreter's baton. | The Loom shape. Every native re-entry does not have to be resumable before activities work; each one converted later is a measured win. |
| P6-4 | **A scheduling seam.** Nothing outside the scheduler knows how an activity is carried. | Keeps the carrier mechanism replaceable (baton threads, continuations, coroutines) on measurement, as Moritz asked: "don't lock us into a specific design". |
| P6-5 | **RBED-shaped scheduler, round-robin policy.** Dispatch by deadline from one min-heap; allocation (what deadline an activity gets) is a separate component shipping only the uniform-rate policy. | Uniform rates with deadline = now + slice and FIFO tie-breaks is round-robin, which is what parity needs; rates, priorities or soft real-time budgets become a policy swap. |
| P6-6 | **Time slice from the existing clause countdown; the slice is a request bit set by a timer thread.** | No new hot-path branch (spike: the countdown already runs on every clause). One mechanism for slice, HALT, SIGINT and inbox; deterministic tests can set the bit themselves. |
| P6-7 | **Out of scope:** the Rexx-level API for user-managed interpreters, and exchanging data between interpreters over queues. | A separate phase, designed with the rxapi integration (Moritz, 2026-09-29). Section 9 records the precedents it starts from. |

---

## 2. Architecture

### 2.1 Units

* **Interpreter.** The unit of isolation, on one OS thread at a time. It owns the heap, the
  collector, classes, `.environment`/`.local`, the guard side table, the scheduler and an inbox.
  The inbox (an atomic request word plus a mutex-guarded payload queue) is the only part other
  threads touch.
* **Activity.** A continuation: an arena-resident chain of activations with their IR register
  frames, the parked driver state (resume point and pending result register), pending traps, and a
  per-activity root region (section 5). It is named by an offset handle, never a Rust reference.
* **Driver.** One loop per interpreter. A resumable call op pushes the callee's frame and continues
  in the same loop iteration; RETURN pops it and delivers the result into the caller's register. A
  clause region is resumable through a separate entry, so no per-clause resume test exists (the
  stackless spike measured that test at +3.5% on emptyloop). Resumable call ops: internal CALL and
  function calls, `::ROUTINE` calls, `Op::Send`, `Op::Message` (instruction-form sends), and
  `~new` into INIT.
* **Scheduler seam.** An internal trait: `spawn`, `run_until_park`, `park(reason)`,
  `unpark(activity)`, `yield_at_slice`, `block_on(op)` and `stop_the_world`.
* **Pinning.** Native code that re-enters Rexx (sort comparators, conversions, UNKNOWN, FORWARD,
  `Object~send`, trap handlers, the native API, library-program entry) runs a nested driver on the
  native stack. While pinned, the activity is not switched cooperatively. If it must block or its
  slice ends, it is promoted to a **carrier**: its own OS thread holding the interpreter's baton.
  It returns the baton when the pinned section ends. Pinning is counted per re-entry kind.
* **Blocking operations** (sleep, commands, stream and console I/O, semaphore and Message waits)
  park the activity; the operation runs on a helper thread and completes through the inbox. With a
  single activity, the fast path calls it directly with no helper.

### 2.2 Data flow

The scheduler takes the earliest-deadline runnable activity; the driver runs it until it parks,
its slice ends, or it completes; control returns to the scheduler, which drains the inbox
(completions, cross-thread requests, SIGINT) and picks again.

### 2.3 The invariant

Exactly one activity runs per interpreter at any instant. This holds by structure (one driver loop,
or the baton held by one carrier), not by a lock anyone takes. The one sanctioned crossing of a
thread boundary by interpreter state is `unsafe impl Send` for the interpreter island, whose SAFETY
argument is that the whole island moves under the baton and no piece of it leaves. That grant is
new: `unsafe` is otherwise confined to `rexx-api/src/ffi.rs`, `rexx-api/src/load.rs`,
`rexx-core/src/bytes.rs` and `rexx-core/src/frame.rs`, so the plan must name the file it lands in
and record the grant.

---

## 3. Scheduling

* **Dispatch** is a min-heap keyed by (deadline, sequence number). A runnable activity's deadline
  is now + slice; the sequence number breaks ties in arrival order, giving FIFO round-robin. Timed
  sleepers (SysSleep, Alarm, Ticker, timed waits) sit in the same heap under their wake time.
* **Allocation** answers "what deadline does this activity get". Phase 6 ships the uniform-rate
  policy only.
* **Wait objects** (guard locks, semaphores, Message completions) keep their own FIFO waiter
  queues, independent of the scheduler, as the oracle's do.

---

## 4. Time slice, requests and SIGINT

* **Slice.** `count_clause_against_deadline` (`rexx-exec/src/clause.rs`) already decrements a
  counter on every clause, including an empty-body loop's step, and has a `#[cold]`
  `countdown_reached` that never runs today because the reload is `u32::MAX`. With one runnable
  activity and no SIGINT handler, the reload stays there. Otherwise it drops to N (order 1024; the
  spike measured N = 50 at +0.27% on emptyloop and +0.06% on rexxcps, and N only bounds latency
  because the timer supplies time). The cold path loads the request word and acts on its bits; a
  SLICE bit switches at that clause boundary, never mid-clause, as the oracle does.
* **Timer thread.** One process-wide thread. An interpreter arms it when it gains a second
  runnable activity or a due sleeper, and disarms it otherwise; with nothing armed it sleeps with
  no deadline. It sets SLICE after the oracle's 24 ms and wakes idle interpreters for due sleepers.
  Precedent: CPython's `eval_breaker`, Ruby's timer thread.
* **Requests inside an interpreter** (HALT, TRACE toggle, a raise aimed at another activity) write
  the target activity's state directly (same thread), are delivered through `pending_traps` at its
  next clause boundary, and wake it if parked.
* **Requests from another OS thread** set bits in the target interpreter's request word and push a
  payload to its inbox. They never touch frames (D3). Latency is at most N clauses; an idle
  interpreter is woken at once.
* **SIGINT.** A handler installed by the CLI (`rexx-run`) only, not by an embedding host, sets the
  halt bit in every live interpreter's request word and wakes idle ones. The running activity takes
  HALT at its next cold visit, parked ones when they wake: the oracle's Error 4.1 on `do forever`.
  The live-interpreter registry and the timer thread are the one exception to "no process-global
  state", granted by this design and to be recorded as such.
* **Long native builtins** (SysStemSort, a large `copies`) run no clauses and cannot be preempted
  or halted, on the oracle either. Accepted. `block_on` can later offload a builtin measured longer
  than a slice.
* **True thread interruption** is not pursued: Go 1.14's signal-based preemption needs
  compiler-emitted safe-point maps for the interrupted code, which Rust code does not have; a
  signal handler could only set the flag the timer already sets.
* **Deterministic switch mode.** Tests may set SLICE themselves (switch at clause k, or at every
  opportunity), giving reproducible interleavings and a stress mode analogous to
  collect-at-every-allocation.

---

## 5. Collector and roots

* **Split `RootSet`** (`rexx-core/src/roots.rs`) into an interpreter part (`globals`, `cells`,
  classes, environment, guard side table) and an activity part (`slots`, `aliases`,
  `frame_starts`, `temps`, `parked`, register frames). The running activity's part stays one
  indexed load away. This is the research direction's Stage 0 item 3, done before a second
  activity exists.
* **Slots per activation as arena segments**, not one flat stack, so a frame below the top can
  grow. This removes the Phase 8 residual owned by Phase 6: setting, through a kept outer call
  context, a name the outer activation never bound.
* **`Interp::object_roots` stays exhaustive** (destructured with no `..`) and walks every
  activity's region: running, parked, carriers and pending completions.
* **Collection points.** Every allocation remains one, as a stated constraint (it is the only
  instrument that finds a missed root). Two are added: an activity switch, and a baton handoff to
  or from a carrier. Anything live across either must be rooted, the same rule as across an
  allocation, so a carrier that gives up the baton is at a safepoint by definition.
* **Parallel-ready hooks.** `collect` calls the scheduler's `stop_the_world`, which today asserts
  every carrier is parked; a future per-activity or parallel collector replaces that one function.
  A stress mode collects at every countdown visit, switch and handoff.
* **Isolation at the type level.** `ObjRef` becomes `!Send` (a phantom marker), so the compiler
  refuses an object handle in the inbox, a helper message or another interpreter.
* **Sharing-fraction instrument.** A feature-gated build tags each object with the last activity
  that resolved it and counts objects touched by more than one activity: the number the research
  direction says decides isolation against shared-heap-plus-locking for the no-lock future.

---

## 6. Rexx semantics

Items marked *(verify)* are pinned against the oracle's source and runs in the plan's first task
that needs them, with the command recorded.

* **Guard locks.** A per-interpreter side table keyed by object identity holds, per (object,
  scope), the owning activity, a nesting count and a FIFO waiter queue. Guarded methods acquire on
  entry; unguarded ones skip. A contended acquire parks; release hands the lock to the first
  waiter; the owner re-enters by bumping the count. An unsatisfiable wait blocks forever as the
  oracle's does (`rust/corpus/oracle-crashes.txt` entry 7); whether the oracle detects any guard
  deadlock is *(verify)*.
* **GUARD ON/OFF [WHEN].** WHEN registers the activity on the scope's watch list, releases or keeps
  off the lock, and parks. A store to that scope's object variables marks it dirty and wakes its
  watchers FIFO; each re-evaluates. The 99.913 error for a WHEN naming no exposed variable is kept.
  The barrier is one "watched" test on object-variable stores only, never on locals, and is
  measured.
* **REPLY** splits the continuation: the caller resumes with the reply value; the rest of the
  activation becomes a new activity's root, and the guard lock's ownership moves with it. Today's
  deferral of replies to program end, and the refusal of REPLY inside DO/SELECT/IF, go.
* **Message objects.** `~start` and `Object~start`/`~startWith` spawn an activity making the send;
  `~result`, `~wait` and `~completed` read or park on a completion record; an error in the started
  send is stored and re-raised by `~result` (how the oracle reports an untrapped one is *(verify)*);
  `~notify` queues a notification. `~start` stops running synchronously.
* **Semaphores.** EventSemaphore and MutexSemaphore are per-interpreter objects; `wait` and
  `acquire` park FIFO; a MutexSemaphore is re-entrant for its owner. RexxUtil's `Sys*Sem` routines
  map onto them. Named semaphores across processes belong to the rxapi work.
* **Alarm and Ticker** are deadline-heap entries whose firing spawns an activity for the message;
  cancel removes the entry.
* **Program end.** Which started activities the main program waits for at exit follows the oracle
  *(verify)*.

---

## 7. Staging

Each stage ends with the six gates green. The performance gate is **cumulative against the Phase 6
base commit**, never only per step (per-step thresholds hid +32% of drift in Phase 5a). Callgrind,
glibc excluded, each build in its own target dir, with a layout control.

| Stage | Delivers | Gate beyond the six |
|---|---|---|
| S0 Foundations | The scheduler seam; the `RootSet` split; `ObjRef` `!Send`; every `thread_local!` in interpreter state removed or made per-interpreter; the interpreter island `Send` (the grant). No behaviour change. | Flat within noise on rexxcps, emptyloop, varlookup. |
| S1 Stackless calls | In-loop frame swaps for the resumable call ops of 2.1; resume at region entry; parked state in the arena with offset handles; slots per activation segment. | Parity or better on rexxcps, emptyloop, a recursive fib(22) by CALL and by function, and a send loop; recursion bounded by memory. **Decision point**: if parity is unreachable, fall back to baton threads behind the seam, ruled on with the figures. |
| S2 Activities | Scheduler (section 3), timer thread, request word, `Message~start`/`~result`/`~wait`/`~notify`, REPLY split, SIGINT, the deterministic switch mode. | Corpus witnesses; crate tests under forced switching. |
| S3 Synchronisation | Guard locks, GUARD ON/OFF/WHEN and its barrier, EventSemaphore, MutexSemaphore, `Sys*Sem`, Alarm, Ticker. | Barrier cost measured on object-variable stores. |
| S4 Pinning and blocking | Carrier promotion, blocking operations on helpers, pinning counters. | A pinning report over the corpus and ooTest. |
| S5 Close | Section 8. | Final review. |

---

## 8. Exit criteria

1. **The concurrency ooTest groups** pass on the crate and agree with the oracle through a
   Phase 8-style instrument: `base/keyword/GUARD`, `base/keyword/REPLY`, `base/class/Message`,
   `base/class/EventSemaphore`, `base/class/MutexSemaphore`, `base/class/Alarm`,
   `base/class/Ticker`, `regressions/bug2003_guard_when` and `base/rexxutil/SysSleep`, with any
   test that changes rxapi's persistent state excluded as in Phase 8.
2. **The ooTest framework's ticker** (`ooTest.frm`, a REPLY thread and GUARD ON WHEN) runs without
   `-U`; L2 then remains blocked only by Phase 10's RXFUNCQUERY.
3. **Nightly ThreadSanitizer**, gate-only, from a pinned nightly in a scratch `RUSTUP_HOME` (never
   for shipped builds), is clean over everything that crosses threads: timer, inbox, helpers,
   carriers, SIGINT.
4. **The single-owner audit**, replacing roadmap row 6's lock-dependence wording: an inventory of
   every `unsafe impl Send` and every cross-thread type, each with a test or type-level check that
   no state is reachable from two threads outside the baton or the inbox.
5. **The sharing fraction** is measured over the corpus and ooTest and recorded.
6. **A ping-pong benchmark** (message round trip, semaphore post/wait, GUARD WHEN handoff) is
   recorded against the oracle, not gated: the baseline for any later carrier change.
7. **No refusal names Phase 6.** Every `Loud` text, layout owner, native entry owner and
   exclusions row that names Phase 6 is gone or re-homed with a true reason. This covers at least
   the GUARD WHEN wait, REPLY inside a construct, `Message~result` of an unsent message, the Alarm
   and Ticker timer entries, the `Sys*Sem` routines, `SetGuardOnWhenUpdated`/`SetGuardOffWhenUpdated`,
   the kept-outer-context residual and the L2 ticker rows, and the Message and semaphore methods
   that today fall through to the generic native-method refusal. The enumeration is derived with a
   committed command, shown empty at the close, and enforced: `closed_phases` gains Phase 6, so a
   reintroduced owner or text naming it turns the gate red.
8. Every *(verify)* item of section 6 is settled against the oracle.

**Testing rule.** Corpus programs stay deterministic (`corpus/README.md`), so they witness only
orderings the oracle itself fixes. Interleaving-dependent behaviour is tested on the crate alone,
under the deterministic switch mode.

---

## 9. For the later phase: moving values between interpreters

Recorded here because P6-7 defers it. ooRexx offers no user-facing object serialization: queues
carry strings, and rxapi queues hold byte strings. The interpreter has an internal graph flattener
(`interpreter/memory/Envelope.cpp`, `SmartBuffer.cpp`) used for `rexxc` images and `rexx.img`; it
preserves identity and cycles but assumes the same interpreter build reads it back. Elsewhere,
from cheapest to most general: values only (Tcl threads, rxapi queues); structured clone of a
fixed type set, refusing behaviour (JavaScript workers); copy, move or share-if-deeply-frozen
(Erlang, Ruby Ractors, Python PEP 734 "shareable" types); and full pickling with per-class hooks.
The natural fit for Rexx is structured clone over its own types (strings, numbers, Array,
Directory, StringTable, Stem, Bag) plus an opt-in pair of methods for user classes.
