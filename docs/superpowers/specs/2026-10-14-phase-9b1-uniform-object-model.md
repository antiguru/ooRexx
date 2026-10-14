# Phase 9b1: a uniform object model

Status: approved by Moritz, 2026-10-14 ("Lgtm"), and revised for the spec review
(`.superpowers/sdd/2026-10-14-phase-9b1/spec-review.md`). It sits between 9b and 9c in the Phase 9
spec (`2026-10-10-phase-9-core-conformance.md`, row 9b1).

Principle (rust/CLAUDE.md, "Specialization is internal"): the crate's representation of a value is
never observable from a Rexx program. 9b1 removes the representation leaks the 9b1 scouts found
(`.superpowers/sdd/2026-10-14-phase-9b1/scout-leaks.md`, `scout-deviations.md`), and it states the one
language rule that makes the rest specified.

## 1. String and the numbers are value-based

Plain String values are value-based, in the sense of Java's value-based classes. That includes the
numbers, `.true` and `.false`: every value whose class is `.String`, with no enhancement and no
subclass.

* Whether two equal values are one object is unspecified, in both directions. A program can see this
  through `identityHash`, IdentityTable and IdentitySet keys, and the identity of `~copy` or of an
  operation that answers its receiver unchanged. None of those answers is specified for a value-based
  value. The oracle's per-program literal pool and its -10..100 integer cache are implementation
  details of the oracle, not semantics.
* A value-based value carries no per-object state. `objectName=` on one refuses loudly, with a message
  that names the rule ("a String is value-based"), not "this crate does not keep X". Moritz: "I don't
  think object name is an important API."
* Whether a WeakReference to a value-based value clears after the program drops its last reference is
  unspecified. It never clears while the value is reachable.
* What stays specified: the `identityHash` of a held value is stable through reads, aliases,
  arguments, returns, and storage in and retrieval from a collection.
* Subclass instances (`.MyString~new`) and enhanced strings are ordinary objects with full identity.
  They accept `objectName=` and every other per-object operation.

This replaces Deviation 31 with a new Deviation that cites this section, and rewrites Deviation 4 to
match the first bullet. Deviation 33 (hash order) is the model for the wording.

## 2. Defects the rule does not license

Each one is a wrong answer under any identity rule. Each is fixed, and each gets a corpus witness
checked against the oracle.

* L1: a WeakReference to a held inline value (a short string or a small integer) answers `.nil`
  after a collection, while the value is still held. The weak pass in `rexx-core/src/heap.rs` treats
  a non-heap handle as dead.
* L2: user arithmetic and prefix operators on a String subclass instance are not sent, and neither is
  the DO setup `+`. `eval.rs` and `loops.rs` convert to a number before the operator-receiver test.
* L3: a class-side `STRING` method is not sent for `say .k` or `length(.k)` (`reqstr.rs`).
* L10: `~addRoutine` of a copy of an internal-row Routine object answers 88.914, where the oracle adds
  it.
* L11: a native routine's frame (e.g. SysStemSort) is missing from a live `.context~stackFrames`.

L12 is not fixed. It is SysStemSort writing results back by tail after a STRING method has emptied
the stem mid-sort, which is mutation of a collection during an operation over it. It is licensed as
unspecified (a new Deviation, approved with this spec). Matching the oracle would need
compound-variable identity, which is representation.

## 3. Package is a heap object

A Package object today is an identity (`object_packages`) whose state lives in per-ProgramId side
maps on `Interp` (scout-leaks, "Side maps", Package group). 9b1 makes it a heap object.

* The object holds the per-package mutable state as fields: options, security manager, routines,
  public and merged routines, classes, public and merged classes, namespaces, locals, imports,
  parents, required packages, the routine-object table, `package_tables` and `constant_values`. The
  last two hold ObjRefs, so as fields they die with the package and stop being global roots.
* Where there is a compiled program, the object holds a shared handle to it. The REXX package has
  none.
* `~copy` is a field copy that shares the program. The REXX package copies like any other (L6), and
  so does a Package subclass instance (L5).
* The collector traces the object and frees it. A package that nothing references is collected, and
  a WeakReference to it clears (L7).
* The requires cache holds packages weakly, as the oracle's `PackageManager::addToRequiresCache`
  does.

Program lifetime: a program lives while a Package, Method or Routine object that the collector
reaches holds it. At sweep, a program that no live object holds is freed, and the rows keyed by its
ProgramId (plans, chunks and every other id-keyed table) are pruned. A ProgramId is never reused, so
a stale id can only miss, never name another program. `Interp::programs` stops being a strong
append-only holder.

The compiled program type lives in `rexx-parse`, and the heap's body types live in `rexx-core`,
which does not depend on it. The plan chooses how the Package body refers to its program across
that boundary.

New internal classes are allowed where they help, as long as they are namespaced (Moritz,
2026-10-12).

## 4. Executables and messages are collected

* L8: every `Method~new`, `Routine~new`, source-string `setMethod` and `Package~new` adds a program
  that is never freed. Each costs about 20 kB, and 80000 `Method~new` peak at 1.70 GB, against the
  oracle's 20 MB. Section 3's program lifetime frees them.
* L9: the Method, Routine and Message side maps keyed by ObjRef become fields of their object, or are
  pruned at sweep when the object dies. These are `executable_sources`, `method_flag_writes`,
  `message_outcomes`, `started_messages` and the rest of their group in scout-leaks.
* The collector's trigger charges, at allocation, the weight of `::CLASS` classes, packages and
  programs, as it already charges a `~subclass` class (`heap.rs` allocation charge). Then a loop that
  only creates executables still collects
  (`.superpowers/sdd/queued/2026-10-12-gc-trigger-blind-to-class-weight.md`).

## 5. The side-map sweep

Every per-object map the scout lists (Package, Class, Method/Routine, Message, Stem, any object) ends
in exactly one of three states, recorded in a table committed with the gate record:

1. a field of the object it describes;
2. keyed by an ObjRef, and pruned at sweep when that object dies;
3. compiled data keyed by a ProgramId, and pruned when section 3 frees the program.

A map keyed by a numeric id with none of these properties is a defect, and the sweep fixes it. The
Class maps are already in state 2, the registry's `by_name` included (`ClassRegistry::expunge`).

## 6. Criteria

* Every leak in scout-leaks' ranked table is either fixed (section 2, 3 or 4), or licensed (section 1
  or L12's Deviation). The gate record says which, per leak.
* A committed test greps `rust/crates` for refusal text that says a value or package is one "this
  crate did not build" or "does not keep", and it fails on any site not on its named allow-list. The
  allow-list holds Deviation 32's site and section 1's `objectName=` refusal, and the plan derives
  the rest of it from the grep.
* A corpus program pins section 1's specified guarantees on both sides: held-value `identityHash`
  stability through reads, aliases, arguments, returns and collections, and full identity plus
  `objectName=` on subclass and enhanced strings.
* `base/class/Package` `TESTRECLAIM01` passes within its own wait, with no timeout. The ooTest suite
  has no test regression against the 9b close
  (`docs/superpowers/records/2026-10-13-phase-9b/close2/suite/tests.tsv`), and the corpus
  differential stays green.
* Memory: nothing grows with the iteration count. For `Method~new`, `Routine~new`, source-string
  `setMethod`, `Package~new` and the message loop, the 80000-iteration peak RSS is within 10% of the
  20000-iteration peak. The oracle's figures are recorded beside ours, with the command. The absolute
  gap to the oracle is collector tuning (the 32 MB collection floor), which the GC-timing licence
  covers (controller ruling 2026-10-15, after Task 6 measured it; flagged to Moritz).
* Performance: every bench program stays within +0.5% of the Phase 9 pin
  (`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`, sha256 `c4b4e8c4…`), measured as callgrind
  whole-process instructions with libc included (`rust/bench-programs/callgrind.sh -B 0.5`), at
  close. The 9b close leaves the least headroom on hashnew (+0.31%), startup (+0.29%), pingguard
  (+0.27%) and alloc (+0.23%). Those are the programs section 3 and the trigger change touch. A task
  that changes the send, call or package-entry paths measures fibcall, fibfunc, extcall, dispatch,
  dispatchclass, sendloop, varlookup, pkglocal and startup at the task.
* The workspace close command and collect_stress pass.

## 7. Out of scope

* Interning or canonical tables for value-based values. Section 1 makes them unnecessary.
* Activation-scoped state (clock, RANDOM seed, SETLOCAL). Phase 6.1 moved it to the activation, and
  the scout's probes agree with the oracle. The queued items `2026-10-02` (clock) and `2026-10-07`
  (setlocal) are checked against that, including the REPLY seams, and closed if stale.
* The deferred SysFork and named-semaphore refusals (Moritz, 2026-10-14: owner undecided).
