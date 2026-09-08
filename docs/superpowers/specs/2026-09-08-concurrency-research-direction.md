# Concurrency and parallelism: a research direction

**Status: a research direction, not a plan and not a decision.** Goal one is parity with
ooRexx 5.3. Goal two, after it, is to make the interpreter better rather than to preserve
everything about it -- and concurrency is the area where the gap between what the language
*documents* and what the implementation *does* is widest.

Written 2026-09-08. Every table marked **Measured** was produced by a command given beside
it. Claims about the oracle's source are **Read**. Anything else is marked inferred, and
section 9 lists what is not established.

---

## 1. What ooRexx does today, measured

### 1.1 There is no parallelism

`::METHOD SPIN UNGUARDED` doing a CPU-bound integer loop, sent with `~start` to that many
distinct objects, against the same total work done serially on one activity. Machine has 32
cores (`nproc`).

| | wall | CPU |
|---|---|---|
| 1 activity | 0.36 s | 98% |
| 2 serial | 0.72 s | 99% |
| 2 via `~start` | **1.15 s** | **64%** |
| 4 serial | 1.43 s | 99% |
| 4 via `~start` | **1.52 s** | 96% |
| 8 serial | 2.85 s | 99% |
| 8 via `~start` | **3.00 s** | 98% |

Eight unguarded methods on eight distinct objects, 32 cores idle, **zero speedup and a small
loss**. CPU never exceeds one core's worth. At two activities it is 60% *slower* than serial
at 64% CPU; total CPU time is unchanged (0.74 against 0.72 core-seconds), so the missing
0.41 s is wall time in which nothing ran.

**Read**: `ActivityManager::kernelLock()` is a single `SysMutex`, commented "global kernel
semaphore lock"; every activity passes `Activity::requestAccess()` to run interpreter code.

### 1.2 CPU-bound activities do not interleave at all

Two activities each write an id into a shared object variable and immediately read it back,
counting every time it changed underneath them.

| loop body | switches observed, three runs |
|---|---|
| `c~mark = id` ; `if c~mark \= id then s = s + 1` | **0, 0, 0** |
| the same, plus `call charout '/dev/null', '.'` per iteration | **653, 708, 605** |

2,000,000 iterations per activity in the first row, 2,000 in the second. **The second row is
the positive control**: without it, a zero would be indistinguishable from a detector that
cannot fire.

**Read**, and it is the shape but not the explanation: `RexxActivation::run` checks for a
yield every `yieldInstructions = 50` instructions, and `ActivityManager::relinquishIfNeeded`
yields only when there are waiters *and* `timeSliceLength = 24` ms has elapsed. In a ~500 ms
CPU-bound loop with a second activity outstanding it produced no switch at all. **Why it
never fired is NOT established** -- an activity that fails `lockKernelImmediate` does queue
and does increment `waitingAccess`, so `hasWaiters()` ought to be true. Do not build on the
mechanism; the observable is what is measured.

### 1.3 So today's concurrency is cooperative scheduling at blocking points

Interleaving happens where a method blocks -- the manual's own worked example interleaves
because its loop body is a `SAY`. It does not happen inside computation.

**The consequence that matters for any future design is the opposite of the reassuring
one.** The guarantee a program can accidentally acquire today is far *stronger* than
anything the language promises: a CPU-bound method runs to completion with no other activity
observing intermediate state. Nothing in the reference says that. Every parallel design
breaks it.

Measured, and this is what "stronger than documented" looks like: two activities doing
`t = c~value` then `c~value = t + 1`, two million times each on a shared counter, lose
**zero** updates over three runs -- a read-modify-write split across two clauses, which the
language's own rules do not protect, and which the manual addresses by shipping a
hand-written **Rexx Semaphore Class** as an example.

---

## 2. The documented model is SCOOP-shaped, and the implementation collapsed it

**Two corrections before the comparison, both of which cut against the easy reading.**

*SCOOP is not a standardised Eiffel feature.* ECMA-367 2nd edition (June 2006, adopted as
ISO/IEC 25436) contains zero occurrences of "SCOOP" and zero of "concurren"; `separate`
appears in the reserved-word list and is given no semantics anywhere in the prose. (Measured
by the survey with `pdftotext` and `grep -c -a -i` over the standard; I did not re-run it.)
SCOOP is Meyer's research model plus a vendor implementation, and its three primary
sources -- OOSC2 ch. 30, Nienaltowski's thesis, the ETH papers -- disagree in load-bearing
ways.

*SCOOP is not the absence of a global lock; it is a partitioned one.* Its safety comes from
the reservation rule: a routine with N separate arguments atomically acquires N processor
locks and **holds them until the routine returns**, across user-code call boundaries. For a
project whose motivation is escaping a global lock, that is the thing to internalise -- SCOOP
replaces one coarse lock with many coarse locks whose granularity the programmer picks by
choosing argument lists.

The concurrency chapter of the reference (`oodocs/rexxref/en-US/xconcur.xml`) opens:

> "Conceptually, each Rexx object is like a small computer with its own processor to run its
> methods, its memory for object and method variables, and its communication links to other
> objects for sending and receiving messages. This is object-based concurrency."

That is a handler per object. And the lock it specifies is per *(object, scope)*, not global:

> "If you do not change the defaults, only one method of a given scope can run on a single
> object at a time. Once a method is running on an object, the language processor blocks
> other methods on other activities from running in the same object at the same scope"

> "Any number of objects can be active (running) at the same time."

**The language already carries most of a SCOOP surface:**

| concept | Rexx spelling |
|---|---|
| asynchronous call to another handler | `o~start('M', args)` |
| future, with wait by necessity | the Message object; `~result` blocks only when asked |
| handler continues past its own return | `REPLY` -- returns a value, keeps running, and **transfers the scope lock** to the new activity |
| per-handler mutual exclusion | `::METHOD ... GUARDED`, the default, per *(object, scope)* |
| opting out | `UNGUARDED` |
| precondition as wait condition | `GUARD ON WHEN expr`, re-tested when an exposed object variable changes |

**Where Rexx is weaker than SCOOP**, and this is the part a design must decide rather than
inherit. SCOOP reserves *every separate argument of a routine* for that routine's duration;
Rexx locks only the receiver's scope, for the duration of one message. Four exposures follow,
and the first is the one that matters:

1. **Multi-object atomicity violation.** A `GUARDED` method holds `self` for its whole body
   but takes and releases `target`'s lock only around each send to it:

       ::method transfer
         expose balance
         use arg target, amount
         balance = balance - amount
         target~deposit(amount)

   Between those two clauses another activity can observe `target`, or run the opposite
   transfer. `self~balance + target~balance = const` is observably broken. In SCOOP the
   program is not expressible: `target` must be a formal argument to be a legal separate
   call target, and being a formal argument is what reserves its handler for the whole
   routine -- the atomicity is forced by the same rule that makes the call legal.
   Nienaltowski's thesis names this class as the reason the reservation rule exists.
   **This example is derived from the documented locking rule and was not executed.**
2. **Intra-object concurrency across scopes.** Rexx's lock is per *(object, scope)*, so a
   subclass method and a superclass method on the **same object** run concurrently by design
   -- the manual demonstrates it with interleaved output. Any invariant spanning two scopes
   of one object is unprotected. SCOOP forbids intra-object concurrency outright.
3. **`UNGUARDED` takes no lock at all**, so it can race with a guarded method on the same
   scope at the level of individual variable reads and writes -- the low-level race class
   SCOOP eliminates.
4. **No documented ordering between two `~start` messages to one receiver.** SCOOP guarantees
   FIFO for consecutive calls on the same target. Whether ooRexx provides it is open and
   measurable; nobody has looked.

The manual's hand-written semaphore class is the evidence that (1) was understood and pushed
to the user. And Rexx has no `separate` type, so nothing in the language assigns objects to
handlers -- a SCOOP-faithful implementation would have to derive that assignment, and **the
derivation is the design**.

### 2.1 What SCOOP costs, measured by its own authors

West, Nanz and Meyer, *Efficient and Reasonable Object-Oriented Concurrency* (ESEC/FSE 2015):
their optimised runtime is **15× faster than a straightforward semantics-faithful
implementation** of the same model, with sync coalescing alone worth 12×-250× on parallel
benchmarks "due to the copying of large arrays between handlers", which it says "puts solving
data-heavy problems within reach of SCOOP, which it was not previously".

Read honestly, the cross-language table in that paper puts optimised SCOOP at a geomean of
1.32 s against Go 0.57 s, Haskell 0.89 s and C++/TBB 0.32 s -- so the paper's headline "2×
faster than other well-known safe concurrent languages" is carried by Erlang's 18.07 s
outlier, and against Go and Haskell SCOOP is 1.5-2× *slower*. **The 15× is the number for us:
the cost is in the model's synchronisation points, not incidental to one implementation.**

SCOOP also does not prevent deadlock.

### 2.2 The verdict on "Rexx is already SCOOP"

Half right, and the wrong half is the expensive one. Right: Rexx has per-object exclusive
state, futures with wait-by-necessity, an opt-out for read-only work, and condition
synchronisation folded into the method. Wrong: SCOOP's guarantees rest on two things Rexx has
neither of -- a **fixed object-to-handler map**, so exclusivity is ownership rather than a
lock, and the **reservation rule**, so a routine body is atomic with respect to everything it
will touch. Without those, Rexx gives object-level mutual exclusion and no composition rule.
Adding `separate`-style ownership is not a small change: it is the change that broke Ractor
adoption in Ruby.

The realistic target is therefore **passive-region SCOOP over the existing per-scope lock**,
not classic SCOOP: keep the shared heap, keep the scope as the region, and replace "lock the
receiver's scope" with "reserve, atomically and for the method's duration, the set of regions
this method will touch". In Eiffel that set is the argument list. **What plays that role in
Rexx is the open design question**, and this document does not answer it.

---

## 3. What parity actually pins, which is less than it looks

Concurrency is observable as timing and as interleaving order. Both are nondeterministic in
the oracle already, and `corpus/README.md`'s determinism rule excludes programs that expose
either, so the differential corpus cannot pin them. `corpus/phase-4-exclusions.txt` is where
a deliberate difference is licensed, and that mechanism has been used twice already
(DEVIATION 7 and DEVIATION 8).

**What parity does pin** is the sequential semantics: what `REPLY` returns and when the scope
lock moves, what a Message object's `~result` answers, when `GUARD ON WHEN` resumes and the
99.913 error when its expression names no exposed variable, and the fact that
`GUARD ON WHEN` with an unsatisfiable condition blocks forever (`corpus/oracle-crashes.txt`
entry 7).

**What parity does not pin** is whether two activities run at the same instant.

---

## 4. Our own design has a specific hazard, and it is measured

The universal mechanism for collecting without a global lock is the safepoint. Ours is
"every allocation": `Interp::alloc_with` calls `collect_if_due`, and `heap.rs` states the
consequence -- "every allocation site in that crate is a collection point, and a value held
only in a Rust local across one is a use-after-free rather than a cost."

Julia documents the failure mode this invites: a compute-bound, non-allocating loop never
reaches a safepoint and blocks collection for every other thread.

**Measured**: how much does ordinary Rexx allocate? Each program run twice, once normally and
once under `run_program_collect_every_alloc`, where every allocation collects and the
collection count is therefore an exact allocation count.

| program | allocations |
|---|---|
| `say 'x'` | 0 |
| 100,000 × `s = s + 1` | **0** |
| 100,000 × `v = 'abcdefg'` (7 bytes) | **0** |
| 100,000 × `v = 'abcdefgh'` (8 bytes) | **1** |

An ordinary Rexx loop allocates **nothing**. Tagged small integers and `INLINE_TEXT = 7`
between them make the allocation-free loop the common case rather than a corner. (That the
single allocation in the last row is an interned immortal literal is an inference from
`Chunk::interned`'s doc, not a measurement.)

**So "every allocation is a collection point" is exactly the right safepoint for one activity
and insufficient for two, and the reason is the same tagging that makes us fast.** A
multi-activity design needs a poll on loop back-edges, which is the hottest path in the
interpreter -- `bench-programs/emptyloop.rex` exists to measure precisely that path. This
raises the cost estimate for any parallel design and it is checkable now, before anything is
built.

---

## 5. What the field did

**Only CPython removed a global lock over a shared heap.** PEP 703 is Final (24-Oct-2023);
PEP 779 (Final 16-Jun-2025) makes the free-threaded build officially supported but still not
the default in 3.14. It took four mechanisms at once, none optional: biased reference
counting, immortal objects, deferred reference counting for code and functions, and mimalloc,
whose page-reuse rules are load-bearing for the lock-free read paths.

**Everyone else chose isolation**: Erlang (per-process heaps, copying sends, per-process
collection, no VM-wide pause), Ruby Ractors (deep-freeze isolation, deep copy or move on
send), Perl ithreads (interpreter clone, officially discouraged), Tcl apartments, PHP's TSRM,
JavaScript workers with structured clone. Julia is the one shared-heap counterexample and
reaches it by declining the problem: a data race is the programmer's error, and "Julia is not
memory safe" if you write one.

**Two numbers to design against, both from doing the obvious thing:**

* CPython's Gilectomy made reference counts atomic and ran **18.9× slower on 7 cores**
  (4.4 s → 83.0 s), from contention on `None` and the small-integer cache rather than from
  the atomics themselves. Immortalization exists because of this.
* Ruby's Ractors parse JSON in 16 Ractors at **10.17× the single-Ractor wall clock**, where
  16 forked processes take 3.3×. The residual shared structure is the collector: one
  objspace, every collection stops every Ractor. Feature #22227 (open) proposes per-Ractor
  local GC.

Both failures are one shape: **a shared structure touched once per operation, ending up worse
than the lock it replaced.**

**Ours is the arena.** Every allocation and every handle resolution reads the slot table.
Non-moving is a genuine simplification -- no forwarding, no read barriers -- but
generation-checked handles mean resolution is a shared read on the hot path. That is the
first thing to fix and the first thing to measure.

CPython's other transferable decision is procedural: it shipped a *second build* with a
runtime switch to put the lock back, rather than converting the runtime in one step.

### 5.1 Which camp is Rexx in, and the answer is uncomfortable

| runtime | model | cross-activity object references | collector scope |
|---|---|---|---|
| Erlang/BEAM | isolation, copying messages | none, except refcounted binaries | per-process, no VM pause |
| Perl ithreads | isolation, interpreter clone | only `:shared` variables | per-interpreter |
| Tcl | isolation, apartment | none; scripts posted to event queues | per-interpreter |
| PHP ZTS | isolation, per-thread globals | none | per-request |
| JS workers | isolation, structured clone | none; `SharedArrayBuffer` shares bytes, never objects | per-agent |
| Ruby Ractors | isolation, deep copy or move | only deeply frozen shareable objects | **shared objspace, global stop-the-world** |
| CPython 3.13+ | **shared heap + per-object locking** | unrestricted | global, with safepoints |
| Julia | **shared heap, no locking** | unrestricted | with safepoints |

The two runtimes that chose a shared heap are the two that could not choose isolation: Julia
because scientific code passes large arrays and copying them is the entire cost, and CPython
because thirty years of code assumes one shared object graph.

**Rexx is in that second camp by construction, and not by anyone's choice.** `.ENVIRONMENT`
is process-wide; objects are passed to `~start` by reference; and `REPLY` does not merely
share an object across activities, it *migrates an activation between them* and hands over
the scope lock. Isolation is therefore not available for the semantics as they stand -- which
is precisely why goal two matters. **Whether some part of the language could be given an
isolation story is itself a research question, and it is a language-design question rather
than an implementation one.** Ruby is the warning about doing this halfway: it chose
isolation and is still paying for the parts it could not isolate.

---

## 6. A staged direction

**Stage 0 -- now, during parity: stop baking the global lock in.** Each item is independently
useful, none commits us to a concurrency design.

1. An exhaustive `Interp::object_roots`, in the form `Activation::object_roots` already uses:
   destructure field by field with no `..`, so a new field is a compile error until someone
   decides whether it is a root. This is the fix that would have made the `DO OVER` defect of
   2026-09-08 unwritable.
2. A `HeapRef` newtype for the arena-handle case, so "does this field need rooting?" is a
   fact about a type. `Interp::text_numbers` and `Interp::flat_top` were the same shape --
   `ObjRef` in a field the collector does not walk -- and only a *type* fact (every key
   carries the inline tag) made one safe and the other a use-after-free.
3. Split `RootSet` into a global part (`globals`, `cells`) and a per-activity part (`temps`,
   `slots`, `aliases`, `frame_starts`), keeping the running activity's part one indexed load
   away. **Before a second activity exists, not during**: the slot arena fails loudly under
   interleaving (`pop_slots` asserts it closes the top frame) where `temps` fails silently
   (`pop_frame` is a watermark truncate, deliberately not balance-asserted).
4. Keep every allocation a collection point, stated as a constraint rather than as a
   description -- and record section 4's finding beside it, because the natural repair
   (safepoints at clause boundaries) is strictly weaker and would remove the only instrument
   that finds a missed root.

**Stage 1 -- implement the documented model honestly, still serialized.** Real activities,
real per-(object, scope) locks, real `GUARD ON WHEN`, `REPLY` transferring the lock. This is
parity work that is owed regardless, and it forces Stage 0's split to be correct.

**Stage 2 -- measure the number nobody has.** Once activities exist: *what fraction of objects
is ever touched by more than one activity, over real Rexx programs?* That single figure
decides between isolation and shared-heap-plus-locking, and it has never been measured for
Rexx workloads. It is cheap once Stage 1 lands and it should be measured before either design
is chosen.

**Stage 3 -- the experiment, with an escape hatch.** Two candidates against the same corpus,
built as an alternative configuration rather than a replacement:

* *passive-region SCOOP over the existing per-scope lock* (section 2.2), **not** classic
  SCOOP: shared heap kept, the scope as the region, and "lock the receiver's scope" replaced
  by "reserve the set of regions this method will touch, atomically, for its duration". The
  unanswered part is what determines that set, since Rexx has no argument list to read it
  from. Budget against the 15× that separates a semantics-faithful SCOOP from an optimised
  one.
* *shared heap with per-object locks*, CPython's route, with the arena's per-operation cost
  addressed first or not at all.

**Stage 4 -- decide from measurement**, against the two failure numbers in section 5 rather
than against a hoped-for speedup.

---

## 7. What to measure first, in order

1. The cost of a back-edge safepoint poll on `emptyloop.rex` and `varlookup.rex`, interleaved
   against a baseline binary in its own target directory. This is the cheapest experiment on
   the list and it prices every parallel design at once.
2. The cost of making handle resolution safe for concurrent readers, on the same two programs.
3. Stage 2's sharing fraction.

---

## 8. What this document does not claim

* **Not that removing the lock is worth it.** Nothing here measures a workload that would
  benefit; section 7 exists because that is unknown.
* **Not that parity is threatened.** Section 3 is the argument that it mostly is not, and the
  licensing mechanism for the rest already exists.
* **Not that ooRexx's implementation is bad.** A time-sliced global lock is a reasonable 1990s
  answer, and the measurements in section 1 are facts about what it does, not judgements.
* **Not that the mechanism behind section 1.2 is understood.** It is not; only the observable
  is established.

## 9. Not established

* Why `relinquishIfNeeded` never fired in a CPU-bound loop with a waiter outstanding (1.2).
* Whether the single allocation in section 4's last row is literal interning (inferred from a
  doc comment, not measured).
* Anything about how a handler assignment would be derived for objects that no `separate`
  type marks, or what plays the role of Eiffel's argument list in a reservation rule for
  Rexx -- section 2 names both as the design problem and solves neither.
* Whether ooRexx orders two `~start` messages to one receiver (section 2, exposure 4).
  Measurable, unmeasured.
* The ECMA-367 keyword counts in section 2 are the survey's measurement, not mine; I did not
  re-run them.
* The `transfer` example in section 2 is derived from the documented locking rule and was not
  executed against the oracle.

---

## Reproducing section 1 and section 4

Section 1's programs are `par*.rex`, `ser*.rex`, `race*.rex`, `interleave*.rex`; section 4's
are four-line loops. All were run against
`( ulimit -v 1048576; LD_LIBRARY_PATH=<oracle>/build/lib <oracle>/build/bin/rexx FILE )` from
a fresh directory, and the Rust figures through `target/release/rexx-run` and a temporary
test that calls `run_program_collect_every_alloc`. Timing used
`/usr/bin/time -f "%e wall %P cpu"`. The `GUARD ... WHEN` shapes in
`corpus/oracle-crashes.txt` entry 7 were avoided: that one blocks forever.
