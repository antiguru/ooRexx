# Phase 5j — design notes for Tasks 2 and 3, and two plan amendments

Read-only work done while Task 1's gates ran.

## What a class actually owns in the arena

`run.rs:3185-3210` is the only site that builds a class's variable-pool object. It is **lazy** —
built on first need, not at class definition — and it is a `Body::Instance` whose `class` field is
the *metaclass*, so tracing the pool keeps the metaclass alive. It is stored in
`Interp::class_variables[class]` and rooted by `add_global(&format!(".class-variables {class}"))`.

That is not the only per-class arena object held by a global root. Two more are keyed by class:

* `Interp::method_objects: HashMap<(ObjRef, Box<[u8]>), ObjRef>` — the `Method` object per class
  method, rooted under `method_object_root_key(class, name)`;
* `Interp::annotations: HashMap<Annotated, ObjRef>` — rooted under `annotation_root_key(site)`,
  where `Annotated::Class(ObjRef)` and `Annotated::Member(ObjRef, ..)` are class-keyed.

**Plan amendment 1.** Task 3 as written covers only the variable pool. If the other two stay
globally rooted, a collected class leaks its method objects and its annotations — which is exactly
D59a's growth consequence, so it is in scope, not a follow-on. Task 3 either makes all three class
payload, or removes the latter two at expunge. Decide there; do not let it default.

## Seeding the built-ins

D70 requires a built-in to be marked and traced every cycle, and the reason is concrete here rather
than theoretical: once Task 3 removes the per-class global root, a kernel class's variable pool is
reachable only through the class. `ObjectModel` holds 16 class handles, and the kernel has far more
classes than that, so a kernel class neither in the object model nor named by the running program
would have its pool swept while the class stayed registered and usable. That is a wrong answer, not
a leak.

**Plan amendment 2.** Do not enumerate the built-ins out of the registry on every collection.
`ClassGraph::classes` is a `HashMap`, and `native_classes.rs:398-406` already records why a loop
whose order is a map's is a loop that can come to matter. Capture the set once, when
`library_bootstrap` closes, into a new `Interp::static_classes: Vec<ObjRef>`, and seed the mark
phase from that vector.

The new field is its own small proof that yesterday's work does something: adding it to `Interp`
will not compile until `Interp::object_roots` names it.

## The `ClassEdges` shape

`Interp::classes()` is `&mut self.object_model().classes`, so the registry lives inside the
`object_model` field. `collect_now` can therefore hand the heap a borrowed view over
`class_variables`, `static_classes` and `object_model` while `heap` is borrowed mutably, using the
same disjoint-field destructure `Interp::object_roots` already uses. No `Rc`, no clone, no
restructuring.

## Blast radius of the `Heap::collect` signature change

One production call — `lib.rs:7952`, which `dispatch_seam.rs`'s
`heap_collect_is_called_from_collect_now_alone` already pins as the only door. The other callers are
`rexx-core`'s own tests (`tests/uninit.rs`) and `benches/heap.rs`, which have no interpreter and no
classes.

So the parameter gets a blanket no-op implementation — `impl ClassEdges for ()` — and those callers
pass `&()`. That keeps one entry point rather than growing a second `collect` variant the seam
assertion would have to learn about, and it makes "this heap has no classes" a statable position
rather than an omission.

## `RootSet` cannot remove a global, which settles amendment 1

`roots.rs` exposes `add_global` and nothing else — there is no `remove_global`, `drop_global` or
equivalent. So "unroot the method objects and annotations at expunge" is not an option the API
offers; the choice is to add removal to `RootSet`, or to make those objects class payload and
reached by tracing.

**Class payload, and the prior art already said so.** The JVM keeps statics inside the mirror
object; every Smalltalk keeps the class pool as a plain field of the class; Ruby keeps per-class
rows in the class object rather than in side tables, which is what removes the weak-key problem
entirely. All three surveys independently reached "this should not be a side table keyed by class",
and this tree cannot even unroot one. Task 3 makes all three class payload.

## And the store itself is worth seeing

```rust
globals: Vec<(String, ObjRef)>,
```

with `add_global` doing `self.globals.iter_mut().find(|(n, _)| n == name)` — a linear scan of
string comparisons per call, over a vector that `RootSet::iter` then walks on every collection.
One entry per class, per class-method object, and per annotation, none of them ever removed.

That is a second, independent argument for D68 alongside the lifetime one: the per-class root is not
only permanent, it is an entry in a linearly-scanned vector that a class-creating program grows
without bound. No figure is claimed here — nothing has been measured — but the shape is in the type.
