# Phase 5j — class lifetime

**Written 2026-09-09 at `d74d8bbb1`.** This phase executes one instruction, recorded as the D59
correction in `2026-08-27-phase-5b-instances.md:995` and quoted here so the spec argues from it
rather than from a paraphrase:

> *"it was licensed as a temporary state to unblock the object model work without having weak
> references. it is fine to declare built-in classes with a static lifetime, but all others should
> be collected like regular objects. (no permgen). we can move the fix to after phase 5i, but
> before we start new work."*

D59 and D60 are reopened by that correction. D59a's four consequences are time-boxed defects, not
licensed divergences. The **When** was *"after Phase 5i closes and before any new work starts"*;
Phase 5i closed 2026-09-08.

---

## 1. What the oracle does, measured

Every probe run from a fresh empty directory, oracle under the standard wrapper, crate via
`target/debug/rexx-run`, stdout/stderr/status read on three separate descriptors. All rc 0, empty
stderr on both sides.

**A runtime-created class is collected.** `c = .Object~subclass('TEMPC')`, then `drop c` and
`call gc 'force'`:

| | created | after drop + gc |
|---|---|---|
| oracle | delta 1 | **0** |
| crate | delta 1 | 1 |

**A live instance pins its class, and nothing else does.** Keeping an instance holds the class in
`~subclasses`; dropping the instance releases it (oracle delta 1 → 0; crate stays 1). So the
oracle's rule is plain reachability with instance→class strong.

**A `::CLASS`-declared class is never collected.** Its package holds it and no program can drop the
binding — oracle answers `still DECL` after a forced collection.

**The built-in/user boundary already exists in the tree and already agrees with the oracle.**
`ClassGraph::is_rexx_defined` is set on every kernel class at image build
(`native_classes.rs:362, 363, 406`) and on a program's class only while `library_bootstrap` holds
(`lib.rs:6278`). Observable through the `~define` refusal at `dispatch.rs:5092`:

| | `.Object~subclass('UC')~define(...)` | `.Array~define(...)` |
|---|---|---|
| oracle | allowed | refused rc 98 |
| crate | allowed | refused rc 98 |

**The oracle's subclass list is weak, from its own source.** `ClassClass.hpp:208` declares
`ListClass *subClasses; // our list of weak referenced subclasses`; `RexxClass::addSubClass`
(`ClassClass.cpp:483-488`) does `WeakReference *ref = new WeakReference(subClass)` before adding;
`getSubClasses` (`:473`) returns `subClasses->weakReferenceArray()`, commented *"remove any gc
classes from the list now"*. This is the edge that decides whether the phase does anything at all —
see D72.

**The runtime mint paths are exactly two.** `class_factory` (`dispatch.rs:5575`) has exactly two
callers, `native_subclass` (`:5464`) and `native_mixin_class_factory` (`:5484`). `.Class~new` is
refused rc 93 on both engines, so it is not a third. The other mint sites are the kernel bootstrap
(`native_classes.rs:321-323, 394, 396`) and `Interp::install_class` (`lib.rs:6244`, via
`define_unregistered_class`) for `::CLASS` directives.

### 1.1 A defect found while measuring, which inverts D59a

D59a lists *"a `WeakReference` to a dropped class still answering it"* as a consequence of the
licence. The measured behaviour is the opposite:

| | weak ref to a **live** class | weak ref to a live ordinary object (control) |
|---|---|---|
| oracle | `TEMPC` | `an Object` |
| crate | **`.NIL`** | `an Object` |

Mechanism, `heap.rs:196`:

```rust
let target_alive = self.resolve(target).is_some_and(|t| self.marks[t]);
```

under a comment stating the rule deliberately — *"'Dead' includes unresolvable"*. That is right for
an arena handle and wrong for a class identity, which is unresolvable **because** it is immortal:
`resolve` does `self.slots.get(slot as usize)?` and a class slot is at or above `CLASS_SLOT_BASE`,
past the end of `slots`.

**How it hid.** The first probe dropped the class before reading the weak reference, and both
engines printed `cleared`. Only the live-class control separates a correct clear from a constant
one.

---

## 2. What the tree says

**A class identity has no generation.** `ObjRef::class(id)` is `ObjRef::heap(CLASS_SLOT_BASE | id, 0)`
(`handle.rs:211-216`), generation fixed at zero so `class_id` can read the range back. The arena's
stale-handle safety is the generation bump on free (`heap.rs:120-130`): a dead handle *misses*.
Classes have no such bump, so a recycled class id would make a stale handle **alias** the next class
defined — a wrong answer at rc 0, not a crash. **D67 dissolves this** rather than settling it: an
arena slot carries a generation, so the question stops existing. The first draft's answer was D69's
never-recycle rule, withdrawn in §4.0.

**Two independent reasons a class never dies.** The identity is outside the arena, so the sweep
never sees it; and `run.rs:3207` roots each class's variable pool as a named global keyed by
`format!(".class-variables {class}")`.

**`ClassRegistry` holds every class strongly**, in `graph`, `names`, `default_names`,
`object_names`, `by_name`, `by_system_name` (`registry.rs:39-65`) — under a type doc asserting they
are *"written once per class while the library is being built"*, a premise this phase makes both
load-bearing and false.

**`ClassDef::subclasses: Vec<ObjRef>`** (`class_graph.rs:128`) is strong, appended at `:366`, and
never pruned. It also drives the behaviour cascade at `:582` and `:596`.

**The tracer is already prepared.** `Body::Instance` and `Body::Native` both push their class handle
in `Body::trace` (`body.rs:775-790`), with the comment: *"it names no arena slot today… and the arm
stays correct on the day a class object is allocated like anything else."*

**`Interp::object_roots` is this phase's checklist.** Landed `d74d8bbb1`. Every field bound `_` whose
stated reason is that class identities are not arena objects becomes a live root question here. The
compiler cannot catch that flip; the comments are the only record.

---

## 3. Prior art

Three surveys, written to `docs/superpowers/records/2026-09-09-phase-5j-class-lifetime/` from the
research run of 2026-09-09: `jvm-j9.md`, `smalltalk.md`, `dynamic-languages.md`. Each cites its
sources and labels what it could not establish. The findings converge, which is why the design
below is short.

**No production runtime lets the collector decide that a *named* class is garbage.** Pharo, Squeak,
GNU Smalltalk and Strongtalk all root the global name table strongly and remove a class by explicit
teardown. The JVM forbids it by rule rather than by reachability: JLS 12.7 permits unloading only
what cannot be asked for by name again, because re-deriving it would re-run initialisers and change
identity. Both arguments transfer to Rexx verbatim — installing a `::CLASS` directive runs Rexx code
— and both predict the measurement in §1 rather than merely agreeing with it.

**The one mechanism that generalises is "marked but not traced".** Spur marks its class table
without tracing it, so a class is marked only from true roots (chiefly instance→class), and
`expungeDuplicateAndUnmarkedClasses:` clears unmarked entries between marking and sweeping;
Strongtalk's symbol table does the same with `follow_used_symbols`; V8 prunes its transition arrays
in a post-mark, pre-sweep pass. Three production mark-sweep precedents for exactly the scheme this
phase needs, none of them requiring a write barrier, a generation, or concurrency.

**The class→pool→class back-reference is a cycle, not an ephemeron problem.** Mark-sweep collects
cycles natively; such a class pins only if something outside roots a member of it. For a
`name → class` table whose value *is* its key, an untraced registry plus a sweep is equivalent to
ephemerons rather than merely close.

**Built-ins as a closed, build-time set is the standard answer.** Spur reserves class-table page 0
for VM-known classes and never expunges it; the JVM's boot loader is born with
`_keep_alive_ref_count = 1`. `CLASS_SLOT_BASE` plus `is_rexx_defined` already gives this tree the
same predicate. Static built-ins and "no permgen" are not in tension — the JVM is the existence
proof.

**Never-collected is not never-traced.** JDK-8253081 is the case where an "immortal" archived array
was updated to point at a moving object and crashed G1. A built-in Rexx class can hold a user method,
a class variable and a subclass entry, so it must be a root on every cycle even though it is never
swept.

**Per-container liveness buys nothing here.** The JVM has no per-class liveness at all; it has
per-container liveness and obtains per-class granularity by making a container of one. Against the
measurement in §1 that collapses: a Rexx package pins its declared classes for the life of the
process, so "package as holder" means "permanent", which is a flag and not a container. The general
case never arises, and the cheap rule and the expensive rule agree on the entire observable surface.

**The per-class named global root has no counterpart anywhere.** The JVM keeps statics inside the
mirror object; every Smalltalk keeps the class pool in a plain field of the class. `run.rs:3207` is
not a mechanism to be weakened; it is a root that should not exist.

---

## 4. Decisions

### 4.0 These were rewritten on the day they were written, and why

The first version of §4 kept class identities outside the arena and built the collection machinery
around that: a mark vector of their own, an edge supplier so the heap could follow what a class
owns, a captured set of built-ins to seed as roots, a never-recycle rule for ids, and a special case
in the weak-reference predicate. That was implemented as Task 2, gated green at `8080cb981`, and
reverted at `bf91af935` after Moritz asked why classes are not simply stored in the heap like any
other object.

**The recorded reason did not survive the question.** D59's own *"Why the alternative is not taken"*
argues that collecting classes means identities *"stop being a monotone registry outside the arena,
which is the shape 5a's whole class model is built on"*. That was written under the licence Moritz
corrected on 2026-09-08, and the first draft of this spec carried its conclusion forward as a
non-goal without re-arguing it.

**The performance half of the argument is backwards, checked by reading the sites.** The claim was
that `ObjRef::class_id` is asked before the arena on every send, so a class in the arena would cost
a memory access. But `eval.rs:1367` and `eval.rs:1760` both do the class test and then immediately
`self.heap.get(value)` on the fall-through; `run.rs:4051` has the same shape; `dispatch.rs:1986`
classifies a receiver inside a match whose other arms read the body. Every site already fetches the
object. Folding the class test into a `Body` match arm removes a branch rather than adding a fetch.

**And no dependency inversion is needed.** `rexx-classes` keys classes by `ObjRef` and its own
manifest says it *"does not touch the heap, `Object` or dispatch, only the identity type a class is
keyed by"* — which stays true when the identity is an arena handle. The one thing it cannot do is
*mint* one, because minting becomes allocating. D76 settles that.

The decisions below are what replaced them. Each names what it deletes, because the point of the
change is that most of this phase is now a deletion.

### 4.1 The decisions

**D66. Class collection is by reachability, per class.** Unchanged from the first draft. No holder,
container or package-liveness machinery: §3's survey and §1's measurement agree that the container
rule and the per-class rule cover the same observable surface, so the cheap one wins.

**D67. A class is an ordinary arena object.** `CLASS_SLOT_BASE`, `ObjRef::class`, `ObjRef::class_id`
and `is_class_slot` are deleted. A class identity becomes a handle with a slot and a generation like
any other value, and the class/non-class question becomes an arm of the `Body` match the callers
already perform.

**D68. What a class owns lives in the class object.** The variable pool, and the per-class rows
`Interp` holds today, are reached by tracing the class body — not by a named global root, and not by
an edge supplier the heap has to ask. This is what all three surveys independently recommended: JVM
statics live inside the mirror object, every Smalltalk keeps the class pool as a field of the class,
and Ruby keeps per-class rows in the class object rather than in side tables. It deletes
`run.rs:3207`'s `add_global(&format!(".class-variables {class}"))`, and it deletes the `ClassEdges`
trait the reverted design needed.

**D69. Built-in classes are allocated immortal.** `Heap::immortal` is already a root that is
*traced* rather than merely skipped — `collect` chains it into the initial work list, and the
field's own doc says seeding the mark phase from there marks the objects and everything they reach.
So "a built-in is a root on every cycle, and never swept" is a mechanism this heap already has.
Deletes `Interp::static_classes`, the per-collection built-in seed, and the reverted design's
`ClassEdges::built_ins`.

**D70. Class ids are recycled like any other slot, and the never-recycle rule is withdrawn.** The
first draft forbade reuse because a class identity had generation zero by construction, so a
recycled id would make a stale handle *alias* a new class. An arena slot has a real generation, so a
stale class handle *misses*, which is the property every GC defect in this project has been caught
by.

**D71. The sweep is the expunge.** When the sweeper frees a class slot it unlinks that class's
registry rows, which is Ruby's shape — O(1) per dead class instead of a scan of the registry per
collection. Deletes the separate expunge pass, and with it the question of where it sits relative to
the `UNINIT` resurrection: a class with a pending finalizer is resurrected exactly like any other
object, by the machinery that already does that.

**D72. Subclass lists hold weak entries and are pruned.** Unchanged, and still the decision that
decides whether the phase does anything: `.Object`'s strong `subclasses` vector would pin every
runtime class forever while every gate stayed green. The oracle's own source is the specification —
`ClassClass.hpp:208` declares `subClasses` a list of `WeakReference`, and `getSubClasses` prunes as
it reads.

**D73. The weak-reference predicate returns to one term.** Task 1's `target.class_id().is_some() ||`
was correct for a world where a class is unresolvable because it is immortal. Once a class is an
arena object, `resolve` answers for it and the original predicate is right again. Task 1's witness
stays and must keep passing across the change — it is what says the revert did not reintroduce the
defect.

**D74. A send to a collected class is loud.** Unchanged in intent, cheaper in mechanism: a stale
class handle now misses, so it lands on the existing "a live value" path rather than needing one of
its own.

**D75. Divergence licence.** Unchanged. Collection *timing* stays unspecified under the existing
GC-ordering licence; D59a's four observables are no longer licensed and each owes a witness.

**D76. `rexx-classes` does not allocate.** It cannot mint an identity once minting means allocating,
and inverting the dependency to let it would put the heap inside the class model. Instead the
identity source is passed in — `native_classes::build` takes a minting callback — so the crate keeps
holding only the identity type, exactly as its manifest says. A generic heap parameter was
considered and is not needed for this.

## 5. Non-goals

* **Package collection.** `Interp::programs` (`lib.rs:3126`) is push-only and `package_classes` /
  `class_packages` have no removal site, so a `::CLASS` class cannot become unreachable and
  weakening the package name tables buys nothing while packages are immortal. That is a separate
  decision, and the oracle pins declared classes too, so it is not a parity gap.
* **Ephemerons.** §3: the hazard is a cycle, and mark-sweep collects a cycle natively.
* **Replacing the slot table with real pointers.** A separate question, queued behind this phase as
  its own measured spike. D67 puts classes into the slot table that exists; it does not touch what
  that table is made of.

This list previously carried *"making class identities arena objects"* as a non-goal. That is now
D67 — see §4.0 for what changed and why.
* **`become:` / class redefinition.** Not a Rexx operation.

---

## 6. The predicted failure mode

**Any global strong memo makes this phase a silent no-op while every gate passes.** A class registry
row, a method cache keyed by class, an IR inline cache, a `::REQUIRES` package cache — one of these
holding a class strongly, and nothing collects, and every test still goes green because nothing
asserts that a class *died*.

**And D67 adds a second shape the first design could not have had.** While a class was immortal, a
class handle held anywhere was always valid; a table the tracer never visited was a leak at worst.
Once a class is an ordinary object, an unrooted holder of a class handle is a **use-after-free** —
which the surveys named as the worst failure mode of the whole change, and which is the shape this
tree has already produced once, in `flat_loops`. The generation makes it a loud miss rather than an
alias, so it fails as `a live value` rather than as a wrong answer; it is still the thing to hunt.
The instrument is `run_program_collect_every_alloc`, which reaches a missed root as a failure rather
than as luck, and `Interp::object_roots`' class-identity comments are the list of places to look
first.

So the gate asserts the observable, not memory: `~subclasses` returns to its baseline, a
`WeakReference` clears, and a program that mints classes in a loop does not grow without bound. And
the enumeration of what holds a class strongly is a derived list committed as an artifact, not a
prose claim — the same discipline `corpus/refusal-sites.tsv` is under. `Interp::object_roots`'
class-identity comments are the starting set, and they are a starting set, not the answer.

---

## 7. Exit gate

5j closes when, at one commit, with `REXX_CORPUS_GATE=1` and on both engines:

1. Every existing gate is green — `cargo fmt --all --check`, `cargo clippy --workspace
   --all-targets -- -D warnings`, `cargo test --release --workspace --no-fail-fast`, and
   `REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast`.
2. The four probes of §1 and §1.1 agree with the oracle byte for byte on all three descriptors, and
   are committed corpus programs rather than scratch files.
3. A negative control is recorded for each new mechanism: with the class-mark propagation removed,
   with the expunge pass removed, and with the subclass-list scrub removed, a named test reddens in
   each case — and the prediction for each control is written before it is run.
4. A committed, derived enumeration of every site that holds a class identity strongly, with the
   command that produced it.
5. `run_program_collect_every_alloc` passes the L0 subset, as it does today.
6. A class-creation loop's resident set is bounded — the D59a consequence that is about growth
   rather than about an answer.
