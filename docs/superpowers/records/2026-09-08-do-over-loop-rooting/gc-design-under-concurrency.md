# What the `DO OVER` rooting defect implies for the GC, especially under concurrency

Written 2026-09-08 alongside the fix in `diagnosis-and-fix.md`. The survey behind it read both
trees and ran nothing; every claim reproduced here was spot-checked against the source named,
and the ones that were not are marked. The full survey, with its own unverified list, is not
committed — what follows is what survived checking.

## 0. The one-sentence generalisation

The defect was **a reference living in a place the root enumerator does not visit**. Everything
below is about how much harder that gets when a second activity exists, and about which
mechanisms make it impossible rather than merely unlikely.

## 1. The asymmetry that let it happen

`Activation::object_roots` destructures the activation **exhaustively, with no `..`**, so a new
field is a compile error rather than a value that silently stops being rooted; its own doc says
so. `Interp` has no such function. `Activation` has the mechanism and did not have the bug;
`Interp` lacks it and did.

**This is the cheapest available fix for the class, and it is not the fix that landed.** An
exhaustive `Interp::object_roots` called from `collect_now` would have made the defect
unwritable, at the cost of one function plus a `_` binding per new field forever. It is
recommended below and is not yet done.

## 2. Root enumeration stops working at the second activity

`RootSet` is one flat set: `globals`, `temps`, `slots` + `aliases` + `frame_starts`, `cells`,
`parked`. Of those, `globals` and `cells` are genuinely global; `temps`, `slots`, `aliases` and
`frame_starts` are per-activity in nature and are shared today only because there is one
activity.

The hazard is asymmetric between the two stacks, and it decides the ordering of the work:
**the slot arena fails loudly under interleaving** — `pop_slots` asserts the frame it closes is
the top one — **while `temps` fails silently**, because `pop_frame` is a watermark truncate with
no balance assertion, deliberately (its doc explains that an assertion there would fire on the
ordinary error path of a correct program). Two activities interleaving pushes and pops on one
`temps` stack corrupt each other's registers with no diagnostic. So the split has to happen
*before* a second activity exists, not during.

The shape to aim for is the one `Interp::running`/`suspended` already uses: the running
activity's root set is a field reached by one indexed load, the others live in a vector.
The budget is set by the measurements already in `roots.rs` — removing a single indexed load
from the alias check was worth 3.83% on `varlookup.rex` — so a design that reaches the slot
arena through an activity index is out.

## 3. The GIL, and why keeping it is the cheap answer

The oracle is genuinely multi-threaded and holds a global kernel lock. Its collector is
reachable from any activity, and `ActivityManager::live` walks every activity in the process
under the resource lock, marking each; each `Activity` then marks its own activations, and each
activation its `doStack`. That is the multi-activity form of what `Interp::collect_now` does by
hand for `running` and `suspended` today.

Keeping a GIL buys the property our whole rooting discipline depends on: **when a collection
happens, exactly one activity is running Rexx code and every other one is at a point where its
roots are enumerable.** It costs Rexx-level parallelism, which the language's own `GUARD`
machinery suggests nobody is counting on. The design effort then goes into release points, not
into the collector: a cooperative yield check in the driver loop, a cross-thread request stored
as an atomic flag on the target activity rather than written into its frames, and
release/reacquire around every block.

Two smaller things break at two activities and are worth fixing before then: the exit value is
held under a single named global key, which is last-writer-wins the moment two activities can
exit, and `add_global` is a linear scan over a `Vec<(String, ObjRef)>` under what would become a
shared lock.

## 4. Safepoints: keep every allocation one

There are three candidate safepoints — the clause boundary, every allocation site, and the flat
loop's pass boundary. The current design is **every allocation**: `Interp::alloc_with` calls
`collect_if_due`, and `heap.rs` states the consequence, that "every allocation site in that crate
is a collection point, and a value held only in a Rust local across one is a use-after-free
rather than a cost".

**That property must be stated as a requirement rather than as a description, because retreating
to clause boundaries is the natural-looking move under concurrency and it silently removes the
only instrument that finds a missed root.** A clause allocates many times; a collector that can
only run at clause boundaries cannot see a root missed within one.

The invariant at a safepoint, stated for this codebase: *every `ObjRef` that names an arena slot
and that any thread will dereference afterwards is reachable from `RootSet::iter()` chained with
`Heap::immortal`.* Two refinements matter. It is about `Decoded::Heap` handles below
`CLASS_SLOT_BASE` only — small integers, inline text up to seven bytes, `.nil` and class
identities are not arena objects and cannot dangle. And the collector's own site is part of the
argument: `Activation::context_object` is rooted by `collect_now`'s sweep rather than by any
state, which a test pins by asserting `heap.collect(` appears at exactly one call site in the
crate. Any new direct `Heap::collect` call reopens that.

One observation from this defect that generalises: **the exposure window was not narrow.**
`flat_top` is `Some` for a flattened loop's entire life, so the items were unrooted across every
op of every pass. What made the bug rare was the collection watermark, not the window. Under
concurrency the watermark stops being the buggy thread's own — another activity's allocation
triggers the collection, at a point this one never chose — so the same defect becomes a
heisenbug without becoming any rarer in principle.

And the register file living inside `temps` cuts both ways: it is why the landed fix needed no
new mechanism, and it is why a second activity sharing one `temps` stack would corrupt registers
rather than merely lose roots.

## 5. Incremental and concurrent marking

Non-moving is worth keeping: no handle rewriting, no read barriers, and the generation-check-on-
free means a stale handle *misses* rather than aliasing.

If incremental marking is ever wanted, the shape that fits is a **deletion (SATB/Yuasa) barrier
at `Heap::get_mut`**, because that is the single choke point through which a `Body` is mutated —
an insertion barrier would need `get_mut` replaced by narrow setters. Root writes should *not* be
barriered; `set_frame_slot` and the `alias_count == 0` fast path are two of the measured hot-path
wins in `roots.rs`, and the root set is small enough to re-scan at cycle end. That yields an
incremental collector rather than a fully concurrent one, which under a GIL costs nothing extra.
The handle tag is the barrier's early-out: `SmallInt`, `Text`, `Nil` and class identities can
never create an obligation.

One thing not to do: separate the sweep from the generation bump. Freeing a slot without
advancing its generation widens a window in which a stale handle silently resurrects a live
object, which loses the one property that makes this defect class findable at all.

## 6. What would have caught this bug, honestly

| design | recurrence of the class | caught by |
|---|---|---|
| today: hand-maintained `RootSet` | high | the stress mode, on covered paths only |
| loop state in registers (what landed) | fixed for this instance, unchanged for the class | same |
| a shadow stack like `ProtectedObject` | moderate | same; converts to "forgot to declare a protector" |
| conservative stack scanning | low, but rejected | n/a |
| type-level (`gc-arena` branding, or a Servo-style must-root lint) | lowest | at compile time |

**Conservative scanning is rejected and should stay rejected, for two independent reasons.** An
`ObjRef` is not a pointer — it is a tagged `u64` — so there is no address for a scanner to
recognise, and any word-shaped heuristic would pin slots by accident. And the workspace denies
`unsafe`, with granted sites recorded and asserted; a conservative scanner is a large permanent
`unsafe` surface that cannot meet that bar.

Of the type-level options, `gc-arena`'s lifetime branding would have made the field unwritable,
but it requires the interpreter to live inside a mutation callback and moves collection points
from "any allocation" to "between callbacks" — a different collector, and the loss of the
property section 4 argues to keep. A Servo-style `unrooted_must_root` lint would have reported
exactly this shape at lower cost, but needs a custom lint driver. `shredder` and `rust-gc` would
have dissolved the bug by making holding a handle *be* rooting, at a per-copy cost this
interpreter's measurements rule out.

**The cheap intermediate is none of those**: an exhaustive `Interp::object_roots`, section 1.

## 7. Recommended order

1. **Now, cheap**: an exhaustive `Interp::object_roots` called from `collect_now` (§1). Write
   down that conservative scanning is rejected and why (§6). Restate "every allocation is a
   collection point" as a constraint rather than a description (§4).
2. **Now, needs a spike to size**: a `HeapRef` newtype for the arena-handle case, so that "does
   this field need rooting?" is a fact about a type rather than about a value. Size it from a
   build, not a grep.
3. **Before the second activity exists**: split `RootSet` into global and per-activity parts,
   holding the running activity's part one indexed load away (§2); extend the collector's walk
   to every activity; make the stress mode collect on any activity's allocation, since it is the
   only instrument that finds this defect class and its single-activity form stops proving
   anything the moment another activity can trigger a collection. The parked group can be deleted
   *with* that split and not before.
4. **Only if incremental collection is wanted**: the `get_mut` deletion barrier, no root
   barriers, sweep and generation together (§5).
