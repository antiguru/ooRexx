# Immortal types vs collectable types in dynamic-language runtimes

Prior art for: *built-in classes may keep a static lifetime; every user-defined class
must be collected like an ordinary object; no permgen* — and for the harder half,
keeping the VM's own registries from pinning the collectable ones.

## How to read this

Every substantive claim carries a citation: a PEP number, a file and function in a
named revision of CPython / V8 / CRuby, or a docs URL. Sentences that are my
reasoning rather than a source's statement are prefixed **Inference:**. Sentences I
could not source are prefixed `UNSOURCED:`.

Revisions read (downloaded and grepped locally, not quoted from memory):

- CPython `3.12` branch — `Objects/typeobject.c`, `Modules/gcmodule.c`, `Include/object.h`
- V8 `12.4.254.20` — `src/objects/transitions.h`, `src/objects/map.h`, `src/objects/map.tq`,
  `src/objects/map.cc`, `src/objects/objects-body-descriptors-inl.h`,
  `src/heap/mark-compact.cc`, `src/flags/flag-definitions.h`
- CRuby `v3_3_0` — `gc.c`, `class.c`, `variable.c`, `vm_callinfo.h`

---

## 0. The fact that decides most of this

Three of the four runtimes below are **reference counted** (CPython) or have a
**concurrent/incremental marker** (V8, and CRuby's incremental marking). A large part
of what they do about type liveness exists only to service those two properties. The
Rexx collector has neither.

The single most important consequence, stated up front because the framing of the
brief invites the opposite conclusion:

> **A class's back-references to itself — its method bodies' owning-scope pointers, its
> class-variable pool, its metaclass link — are not a problem for a tracing collector.**
> A cycle that no root reaches is unmarked in its entirety and swept in its entirety.
> Self-reference is fatal only to refcounting.

CPython's entire heap-type GC apparatus (`Py_TPFLAGS_HAVE_GC` on heap types, a
`tp_traverse` that visits `Py_TYPE(self)`, `type_clear` breaking `tp_mro`) exists to
let its *cycle collector* see cycles that its *refcount* cannot resolve. From
`Objects/typeobject.c:5264` (`type_clear`):

```
   Otherwise, the we need to clear tp_mro, which is
   part of a hard cycle (its first element is the class itself) that
   won't be broken otherwise (it's a tuple and tuples don't have a
   tp_clear handler).
```

A mark-sweep collector needs none of that. **Inference:** roughly half of the CPython
material below is therefore instructive about *what the edges are* and worthless as an
*implementation model*; I flag which is which per mechanism.

The problem that does survive translation is the one the brief names correctly: the
VM's own registries are *roots*, or are traced from roots, and so they pin. That is a
root-set-design problem, not a cycle problem, and §4 (V8) and §5 (Ruby) are the real
prior art for it.

---

## 1. CPython: static types vs heap types

### 1.1 What makes a static type uncollectable

Two independent mechanisms, exactly parallel to the two in the Rexx tree.

**(a) The collector is told the object is not a GC object.** `Objects/typeobject.c:5311`:

```c
static int
type_is_gc(PyTypeObject *type)
{
    return type->tp_flags & Py_TPFLAGS_HEAPTYPE;
}
```

`type_is_gc` is `PyType_Type`'s `tp_is_gc` slot. `PyObject_IS_GC(obj)` consults it, so
a *static* type object is not a GC-tracked object at all: the cycle collector never
puts it in a generation list, never traverses it, never frees it. `type_traverse`
asserts this rather than handling it (`Objects/typeobject.c:5235`):

```c
    /* Because of type_is_gc(), the collector only calls this
       for heaptypes. */
    if (!(type->tp_flags & Py_TPFLAGS_HEAPTYPE)) {
        ... _PyObject_ASSERT_FAILED_MSG(...)
```

This is the same shape as the Rexx heap's `resolve` returning `None` for a handle at or
above `1 << 31`: a type-level fact makes the class invisible to the mark loop. The
difference worth noting is that CPython's version is *asserted* — reaching the traverse
function with a non-heap type is a hard failure, not a silent skip.

**(b) Static type objects are statically allocated C globals.** They are not in any
allocator's arena, so there is nothing to sweep. PEP 384 makes this explicit as an ABI
property: "The structure of type objects is not available to applications; declaration
of 'static' type objects is not possible anymore (for applications using this ABI)"
([PEP 384](https://peps.python.org/pep-0384/)).

**(c) Since 3.12, they are additionally immortal by refcount.** `Include/object.h:131`
redefines `PyObject_HEAD_INIT` under `Py_BUILD_CORE` to initialise `ob_refcnt` to
`_Py_IMMORTAL_REFCNT`, so every statically declared object — every static type included
— starts immortal. See §2.

Teardown of static builtin types is *not* collection; it is an explicit walk at
interpreter finalisation, `_PyStaticType_Dealloc` (`Objects/typeobject.c:5026`), which
asserts `_Py_IsImmortal((PyObject *)type)` and clears the type's subclass table, dict,
bases, mro and weakrefs by hand. **This is the shape to copy for "built-in classes keep
a static lifetime":** immortal during the run, torn down at shutdown by walking a known
list, never by the collector.

### 1.2 What pushed types onto the heap

Two pressures, both about *isolation*, not about memory:

- **PEP 384 (stable ABI).** Extensions cannot see `PyTypeObject`'s layout, so they
  cannot declare a static one; `PyType_FromSpec()` builds a heap type from a
  slot table instead ([PEP 384](https://peps.python.org/pep-0384/)).
- **PEP 573 / PEP 630 (module isolation).** A static type is a C-level global and "has
  no information about which module object it belongs to"
  ([PEP 573](https://peps.python.org/pep-0573/)). PEP 630 states the rule directly: "if
  any method of such a type requires access to module state, the type must be converted
  to a *heap-allocated type*" ([PEP 630](https://peps.python.org/pep-0630/)).

**Inference:** neither pressure is a memory-reclamation pressure. CPython moved types
to the heap to get *per-interpreter* type identity, and collectability came along as a
consequence. For Rexx the motivation is the reverse (reclamation first), which means
the CPython trade-off table does not transfer — but the *edge set* it forced people to
enumerate does.

### 1.3 What a heap type must do that a static type need not

The C API documentation states the obligations
([docs.python.org/3.12/c-api/typeobj](https://docs.python.org/3.12/c-api/typeobj.html)):

- **Instances hold a reference to their type.** "This bit is set when the type object
  itself is allocated on the heap… In this case, the `ob_type` field of its instances
  is considered a reference to the type, and the type object is INCREF'ed when a new
  instance is created, and DECREF'ed when an instance is destroyed."
- **The traverse function must visit the type.** "Instances of heap-allocated types
  hold a reference to their type. Their traversal function must therefore either visit
  `Py_TYPE(self)`, or delegate this responsibility by calling `tp_traverse` of another
  heap-allocated type… **If they do not, the type object may not be garbage-collected.**"
- **The deallocator must release it.** "if the type is heap allocated
  (`Py_TPFLAGS_HEAPTYPE`), the deallocator should release the owned reference to its
  type object (via `Py_DECREF()`) **after** calling the type deallocator" — the doc gives
  the `tp = Py_TYPE(self); tp->tp_free(self); Py_DECREF(tp);` ordering explicitly,
  because decref-then-free would dangle.
- **The GC flag.** "Heap types should also support garbage collection as they can form
  a reference cycle with their own module object."

The instance→type reference was retrofitted in
[bpo-35810 / gh-79991](https://github.com/python/cpython/issues/79991), commit
[`364f0b0`](https://github.com/python/cpython/commit/364f0b0f19cc3f0d5e63f571ec9163cf41c62958),
which made `PyObject_Init` incref the type when `Py_TPFLAGS_HEAPTYPE` is set. Before it,
"heap types could be destroyed before the last instance is destroyed" — i.e. a live
instance with a dead class. **That is the exact hazard for Rexx**, and it is a
root-set-completeness hazard, not a refcount one.

CPython's own generic implementation, `subtype_traverse`
(`Objects/typeobject.c:1776`), shows the delegation rule and why it is fiddly:

```c
    if (type->tp_flags & Py_TPFLAGS_HEAPTYPE
        && (!basetraverse || !(base->tp_flags & Py_TPFLAGS_HEAPTYPE))) {
        /* For a heaptype, the instances count as references
           to the type. ... Skip this visit if basetraverse belongs to a heap type: in
           that case, basetraverse will visit the type when we call it later. */
        Py_VISIT(type);
    }
```

The double-visit guard exists only because visiting twice would corrupt CPython's
refcount-subtraction cycle algorithm. **A mark-sweep collector is idempotent under
double visits**, so this whole branch has no Rexx analogue — the Rexx tracer just marks
the class from the instance, once or twice, harmlessly.

### 1.4 What the type object itself traverses

`type_traverse` (`Objects/typeobject.c:5235`) is the authoritative enumeration of a
class's outgoing strong edges in CPython:

```c
    Py_VISIT(type->tp_dict);      /* method / class-variable dictionary */
    Py_VISIT(type->tp_cache);
    Py_VISIT(type->tp_mro);
    Py_VISIT(type->tp_bases);
    Py_VISIT(type->tp_base);
    Py_VISIT(((PyHeapTypeObject *)type)->ht_module);
```

and the comment immediately after names what is deliberately *not* traversed:

```
       type->tp_subclasses is a list of weak references,
       ((PyHeapTypeObject *)type)->ht_slots is a tuple of strings,
       ((PyHeapTypeObject *)type)->ht_*name are strings.
```

**The subclass list is the registry CPython chose to make weak.** `add_subclass`
(`Objects/typeobject.c:7607`) stores `PyWeakref_NewRef((PyObject *)type, NULL)` under a
key that is `PyLong_FromVoidPtr((void *) type)` — a weak *value* under a key that is the
subclass's address as a plain integer, so neither half of the entry is a strong edge to
the subclass. Entries are removed in `remove_subclass`, called from `type_dealloc`.

Note the ordering hazard flagged in that function's own comment:

```c
    // Only get tp_subclasses after creating the key and value.
    // PyWeakref_NewRef() can trigger a garbage collection which can execute
    // arbitrary Python code and so modify base->tp_subclasses.
```

### 1.5 The type↔instance and type↔module cycles

Three edges close two cycles:

| edge | held how |
|---|---|
| instance → type | strong, `ob_type`, increfed since bpo-35810 |
| type → module | strong, `ht_module`, set by `PyType_FromModuleAndSpec` (PEP 573) |
| module → type | strong, the module dict entry |
| type → method → module | strong, methods reach module state via `PyType_GetModule` (PEP 573) |

PEP 573 acknowledges the cycle it creates and declines to avoid it: "creating a class
with `ht_module` set will create a reference cycle involving the class and the module",
justified because "module teardown is not performance-sensitive" and the existing
cycle collector plus the "set all module globals to `None`" teardown already break the
equivalent cycle through function globals ([PEP 573](https://peps.python.org/pep-0573/)).

**What happens when an extension author forgets `Py_VISIT(Py_TYPE(self))`:** the
documented consequence is a leak — "the type object may not be garbage-collected"
(c-api/typeobj). It is not a crash, and no build configuration detects it directly;
it shows up as growing memory or as a leak found only when subinterpreters are used
(this was the motivation for the sweep documented at
[vstinner.github.io/subinterpreter-leaks.html](https://vstinner.github.io/subinterpreter-leaks.html)).
**Inference, and the transferable lesson:** the failure mode of an incomplete traverse
under refcounting is a *silent leak*; under precise mark-sweep the same omission is a
*use-after-free*. The Rexx version of this bug is strictly more dangerous than
CPython's, so it wants a type-level construction rather than a convention — the
per-object trace must be derived from the object's shape, not hand-written per class.

### 1.6 A cache that does not pin, and one that does

`_PyType_Lookup` (`Objects/typeobject.c:4725`) is CPython's method cache. Its entries
store the *value* as an explicitly borrowed reference:

```c
        entry->version = type->tp_version_tag;
        entry->value = res;  /* borrowed */
        ...
        Py_SETREF(entry->name, Py_NewRef(name));   /* name is strong */
```

The name key is strong, the looked-up method is borrowed. Validity is by
`tp_version_tag`, invalidated by `PyType_Modified`; `type_clear` calls
`PyType_Modified(type)` *before* clearing the dict, with the comment "We need to
invalidate the method cache carefully before clearing the dict, so that other objects
caught in a reference cycle don't start calling destroyed methods."

**Transferable:** a version-tagged cache with non-owning payloads is the standard way to
keep a method cache from pinning classes, and the invalidation must be ordered before
the teardown that would make the cached pointer stale.

---

## 2. CPython PEP 683 — immortal objects (3.12)

The most direct analogue of "built-in classes may keep a static lifetime".

**Representation.** A magic refcount value. `Include/object.h:110` (64-bit):
`#define _Py_IMMORTAL_REFCNT UINT_MAX`, and the test at `Include/object.h:239` is a
sign check on the low half-word:

```c
static inline Py_ALWAYS_INLINE int _Py_IsImmortal(PyObject *op)
{
#if SIZEOF_VOID_P > 4
    return _Py_CAST(PY_INT32_T, op->ob_refcnt) < 0;
#else
    return op->ob_refcnt == _Py_IMMORTAL_REFCNT;
#endif
}
```

The header's own comment explains the choice of *all low 32 bits set*: it is
**backward-compatible with un-recompiled extensions** — "C-Extensions without the
updated checks in `Py_INCREF` and `Py_DECREF` [can] safely increase and decrease the
objects reference count. The object would lose its immortality, but the execution would
still be correct." Increments use saturating arithmetic on the low word so a stray
incref cannot roll the value over.

**Cost.** Every `Py_INCREF`, `Py_DECREF` and `Py_SET_REFCNT` gains a branch
(`Include/object.h:649`, `:684`, `:700`). PEP 683 reports "a 2% slowdown (3% with MSVC)"
for the naive implementation and claims approximate neutrality after mitigations, with a
TODO admitting the final number was not re-verified in the PEP text
([PEP 683](https://peps.python.org/pep-0683/)).

**What is immortal.** Per PEP 683: the singletons (`None`, `True`, `False`, `Ellipsis`,
`NotImplemented`), "all static types (e.g. `PyLong_Type`, `PyExc_Exception`)", and all
static objects in `_PyRuntimeState.global_objects`. The set is closed and known at build
time.

**What it bought.** Not memory. Per PEP 683: avoiding cache-line invalidation from
refcount writes on shared objects, removing the data races that blocked a
per-interpreter GIL, and stopping copy-on-write page faults in pre-fork servers
(Instagram, YouTube are named).

**Interaction with the cycle collector.** Immortal objects "will not participate in GC";
`update_refs` (`Modules/gcmodule.c:419`) moves any object it finds immortal, at `:430`, into the
permanent generation:

```c
        /* Move any object that might have become immortal to the
         * permanent generation as the reference count is not accurately
         * reflecting the actual number of live references to this object
         */
        if (_Py_IsImmortal(FROM_GC(gc))) {
           gc_list_move(gc, &get_gc_state()->permanent_generation.head);
```

**Does it survive translation?** *The mechanism, no. The design rule, yes.*

- The magic-refcount representation and the incref/decref branch are pure refcounting
  artefacts. There is nothing to port.
- The `permanent_generation` list is a *generational* artefact — a list the collector
  skips. A non-generational mark-sweep has no such list; the equivalent is simply "the
  handle is not an arena slot", which the Rexx tree already has.
- What does transfer: **immortality is a closed, build-time-decided set, and it is
  represented in the object, not in a side table.** Membership is answerable in O(1) from
  the handle with no lookup. The Rexx design already satisfies this by construction
  (a tagged handle range), and that is a *better* representation than CPython's, because
  it cannot be lost by a stray operation the way `UINT_MAX` can be decremented away.
- Also transferable, and cheap: PEP 683's backward-compatibility reasoning is really the
  principle *make the immortal encoding degrade to "correct but slower", never to
  "incorrect"*. The Rexx analogue is the guard that a class handle must never be
  mistakable for an arena slot — which the tree's `CLASS_SLOT_BASE` doc comment already
  identifies as the exact failure it was introduced to prevent
  (`rust/crates/rexx-core/src/handle.rs:88`).

---

## 3. V8: Maps, transition trees, and pruning at end-of-marking

This is the closest structural match to the Rexx problem: a shape/type object that is
*collectable*, held in a tree of internal bookkeeping that would otherwise pin every
intermediate node, in a heap that has no refcounting.

### 3.1 The asymmetry: transitions weak, back pointers strong

`src/objects/map.tq:37` gives the field layout. The strong fields are `prototype`,
`constructor_or_back_pointer_or_native_context`, `instance_descriptors`,
`dependent_code`, `prototype_validity_cell`; the last field's declared type is

```
  transitions_or_prototype_info: Map|Weak<Map>|TransitionArray|PrototypeInfo|Smi;
```

and the body descriptor (`src/objects/objects-body-descriptors-inl.h:1013`) confirms the
split at the tracer:

```c
    IteratePointers(obj, Map::kStartOfStrongFieldsOffset,
                    Map::kEndOfStrongFieldsOffset, v);
    IterateMaybeWeakPointer(obj, kTransitionsOrPrototypeInfoOffset, v);
```

So: **parent → child (transition) is weak; child → parent (back pointer) is strong.**
`src/objects/transitions.h:44` states the contract in prose:

> Stored transitions are weak in the GC sense: both single transitions stored inline
> and `TransitionArray` fields are cleared when the map they refer to is not otherwise
> reachable.

**Inference (structural, and the key one for Rexx):** this asymmetry is what makes a
tree of derived shapes collectable *leaf-first* while keeping every live shape's
ancestry alive. It is exactly the right shape for a class hierarchy: **superclass links
strong, subclass lists weak.** A live subclass drags its superclasses in (correct — you
need them for method resolution); a dead subclass drags nothing, and the superclass's
subclass list gets pruned. Ruby (§4) independently arrived at the same split.

### 3.2 The pruning pass

`MarkCompactCollector::ClearNonLiveReferences` (`src/heap/mark-compact.cc:2764`) runs
after marking, before sweeping. Inside it, in the `MC_CLEAR_MAPS` scope
(`:2864`):

```c
    // ClearFullMapTransitions must be called before weak references are
    // cleared.
    ClearFullMapTransitions();
    WeakenStrongDescriptorArrays();
```
then, in a separate scope, `ClearWeakReferences(); ClearWeakCollections(); ClearJSWeakRefs();`

`ClearFullMapTransitions` (`:3213`) drains a worklist of transition arrays recorded
during marking, and for each calls `CompactTransitionArray` (`:3280`), which:

1. checks `TransitionArrayNeedsCompaction` — is any target unmarked;
2. slides every live entry left, re-recording the moved slots (`RecordSlot`) so the
   compactor's remembered set stays correct;
3. right-trims the array in place (`heap_->RightTrimArray`);
4. reports whether the descriptor array whose owner died must also be trimmed.

For the degenerate case where a map stores a *single* inline weak transition rather
than an array, `ClearPotentialSimpleMapTransition` (`:2950`) handles it from the
weak-reference pass.

Two details worth carrying:

- **Ordering is load-bearing and is stated as a requirement in the source, not left to
  chance:** map transitions are compacted *before* general weak references are cleared,
  because the compaction pass reads the (not-yet-nulled) weak targets to decide which
  entries died — note the `DCHECK(!map.is_null());  // Weak pointers aren't cleared yet.`
  at `:3223`.
- **Array size is capped so that the trim stays cheap:**
  `kMaxNumberOfTransitions = 1024 + 512`, with the comment "The size of transition
  arrays are limited so they do not end up in large object space. Otherwise
  `ClearNonLiveReferences` would leak memory while applying in-place right trimming"
  (`src/objects/transitions.h:93`).

### 3.3 V8 deliberately *un*-collects maps for a while

`MarkCompactCollector::RetainMaps` (`src/heap/mark-compact.cc:2340`) walks a per-context
`WeakArrayList` of maps with an age counter, and **marks unmarked maps anyway** if they
are young enough:

```c
  // Retaining maps increases the chances of reusing map transitions at some
  // memory cost, hence disable it when trying to reduce memory footprint more
  // aggressively.
```

The age budget is `--retain_maps_for_n_gc`, default `2`
(`src/flags/flag-definitions.h:1728`). The ageing rule is subtle: a map whose prototype
is unmarked ages down; a map whose prototype and constructor are marked does *not* age,
because then "this map keeps only transition tree alive, not JSObjects".

**Transferable, and it is a policy the Rexx design should decide explicitly:** a shape
or class that has become unreachable is not always *worth* collecting, because
recreating it costs re-learning. V8 pays memory for a bounded number of collections to
avoid the churn. **Inference:** for Rexx this matters much less — classes are created by
directives and by explicit `~subclass`, not implicitly on every property store — so the
V8 retention hack is prior art for *why you might not bother*, rather than something to
copy.

### 3.4 Deprecation is a separate mechanism, and is not GC

`Map::DeprecateTransitionTree` (`src/objects/map.cc:593`) marks a map and its whole
transition subtree `is_deprecated`, deoptimises dependent code, and leaves the objects
in place. Recovery is by `Map::TryUpdate` / `Map::Update` (`src/objects/map.h:722`),
documented as: "Returns a non-deprecated version of the input… the non-deprecated
version is found by re-transitioning from the root of the transition tree using the
descriptor array of the map."

**Inference:** deprecation is a *versioning* mechanism (invalidate the cached shape,
force a re-resolve), not a reclamation mechanism; it makes maps *become* garbage that
the weak-transition machinery then collects. Its Rexx analogue is not class collection
at all — it is the method-cache/version-tag invalidation problem, and it is worth
keeping the two separate in the design so that "this class's layout changed" and "this
class is dead" are different events.

### 3.5 Does V8's mechanism survive translation?

**Yes, more completely than any other item in this report.**

- Weak transitions + a post-mark, pre-sweep clearing pass is *exactly* the shape a
  precise, non-moving, stop-the-world mark-sweep supports. No write barrier is involved
  in the *clearing*: the barrier work in `CompactTransitionArray` (`RecordSlot`) exists
  only because V8 also *moves* objects and marks incrementally. A non-moving collector
  drops all of it and keeps the loop.
- The strong-back-pointer / weak-forward-pointer asymmetry needs no runtime support at
  all; it is a decision about which fields the tracer follows.
- The one V8 thing that does **not** transfer is the interaction with concurrent
  marking: `TransitionsAccessor`'s locking (`full_transition_array_access`) and the
  "must be discarded and recreated after Insert" caching contract
  (`src/objects/transitions.h:33`) exist because background threads read transition
  arrays while the mutator mutates them. Stop-the-world removes that entire class of
  hazard.

---

## 4. Ruby: classes are ordinary objects, and the split is emergent

### 4.1 Classes are collected, with no static/heap distinction

`T_CLASS` is an ordinary object type. It is marked in `gc_mark_children`
(`gc.c:7365`) with no immortality test:

```c
    gc_mark(objspace, any->as.basic.klass);      /* every object marks its class */

    switch (BUILTIN_TYPE(obj)) {
      case T_CLASS:
        if (FL_TEST(obj, FL_SINGLETON)) gc_mark(objspace, RCLASS_ATTACHED_OBJECT(obj));
      case T_MODULE:
        if (RCLASS_SUPER(obj)) gc_mark(objspace, RCLASS_SUPER(obj));
        mark_m_tbl(objspace, RCLASS_M_TBL(obj));     /* method table */
        mark_cvc_tbl(objspace, obj);                 /* class-variable cache */
        cc_table_mark(objspace, obj);                /* call caches */
        ... RCLASS_IVPTR ...                         /* class instance variables */
        mark_const_tbl(objspace, RCLASS_CONST_TBL(obj));   /* constants */
        gc_mark(objspace, RCLASS_EXT(obj)->classpath);
```

Note `gc_mark(objspace, any->as.basic.klass)` at the top: **every object marks its
class**, unconditionally, for all object types. This is Ruby's version of
`Py_VISIT(Py_TYPE(self))`, and it costs nothing to state because it is in the generic
marker rather than in per-type code. **Inference: this is the shape to prefer** — the
instance→class edge should come from the tracer's generic object handling, not from a
per-class trace function that an author can forget (see §1.5).

### 4.2 What roots a class: the constant table, strongly

`mark_const_tbl` → `mark_const_entry_i` (`gc.c:6793`):

```c
    const rb_const_entry_t *ce = (const rb_const_entry_t *)value;
    gc_mark(objspace, ce->value);
    gc_mark(objspace, ce->file);
```

Constants are **strong**. A class assigned to a constant of `Object` is therefore
strongly reachable from `rb_cObject`, which is a GC root. There is no rule about
classes here at all: it is the ordinary reachability rule applied to a table that
happens to hang off a rooted object.

### 4.3 What does *not* root a class: subclass lists, unlinked at free

Ruby's subclass lists are intrusive doubly-linked `rb_subclass_entry_t` chains. Search
of `gc.c` for `subclass` finds **only** the free-time unlink sites and the compaction
fixup — there is no `mark` site:

```
$ grep -n "subclass" gc.c
3592:        rb_class_remove_subclass_head(obj);
3593:        rb_class_remove_from_module_subclasses(obj);
3594:        rb_class_remove_from_super_subclasses(obj);
3706:        (same three, for T_ICLASS)
10692:update_subclass_entries(...)   /* compaction pointer fixup */
10706:        update_subclass_entries(objspace, ext->subclasses);
```

Lines 3592–3594 are inside `obj_free`'s `case T_CLASS:`. `rb_class_remove_from_super_subclasses`
(`class.c:129`) splices the dying class out of its superclass's list and frees the entry.

**This is a different mechanism from V8's, and cheaper:** rather than a weak reference
that the collector clears in a scan, the *sweeper* unlinks the entry as part of freeing
the object. Cost is O(1) per dead class instead of O(registry) per collection.

Preconditions for it to be sound (**inference**, but forced by the code shape): the
registry entry must be reachable *from the dying object* (here, the class holds its own
`rb_subclass_entry_t`), the owner of the list must outlive the entry, and the sweeper
must be allowed to run a destructor hook per object. All three hold for a mark-sweep
collector with a per-object finalisation step.

### 4.4 A cache that is explicitly excluded from marking

`struct rb_callcache` (`vm_callinfo.h:278`) carries the single clearest comment in this
whole survey:

```c
    /* inline cache: key */
    const VALUE klass; // should not mark it because klass can not be free'd
                       // because of this marking. When klass is collected,
                       // cc will be cleared (cc->klass = 0) at vm_ccs_free().
```

The cache's *key* is a class, deliberately unmarked, and cleared when the class dies.
The cache's *value* (`cme_`, the method entry) is marked, and the marking loop
`cc_table_mark_i` (`gc.c:3356`) also *deletes* invalidated entries during marking
(`if (METHOD_ENTRY_INVALIDATED(ccs->cme)) { rb_vm_ccs_free(ccs); return ID_TABLE_DELETE; }`).

### 4.5 A cache that *does* pin

For contrast, the inline constant cache does hold its result strongly
(`gc.c:7306`, `gc_mark_imemo`):

```c
      case imemo_constcache:
        {
            const struct iseq_inline_constant_cache_entry *ice = ...;
            gc_mark(objspace, ice->value);
        }
```

So a compiled instruction sequence that has resolved `Foo::Bar` keeps that class alive
for as long as the instruction sequence lives. **Inference:** Ruby accepts this because
instruction sequences are themselves collectable and typically shorter-lived than the
constants they name, so the pin is bounded. The Rexx analogue is any per-call-site
resolution cache in the IR: if the IR body outlives the class, a strong cache entry
turns "no user program can reach this class" into a permanent pin — and the IR bodies
in this tree are held in a never-shrinking `programs: Vec<Rc<Program>>`
(`rust/crates/rexx-exec/src/lib.rs:3126`), so the pin would be for the process lifetime.

### 4.6 Named vs anonymous in Ruby

`Class.new` produces a class with no classpath; `Module#name` returns `nil`. A class
acquires a permanent name when it is assigned to a constant: `set_namespace_path`
(`variable.c:3504`) sets `RCLASS_SET_CLASSPATH(named_namespace, namespace_path, true)`
— the `true` being `permanent_classpath` — and recursively names nested namespaces.

There is **no collector rule** distinguishing the two. The difference is entirely §4.2:
a named class is a strong entry in a constant table hanging off a rooted object; an
anonymous one is not. Assigning an anonymous class to a constant does not change its
GC treatment; it adds a strong reference from something rooted.

---

## 5. .NET collectible assemblies: liveness delegated to a container

### 5.1 The mechanism

Two flavours, same idea:

- **`AssemblyBuilderAccess.RunAndCollect`** for emitted assemblies: "The lifetime of a
  collectible assembly is controlled by the existence of references to the types it
  contains and the objects that are created from those types"
  ([collectible-assemblies](https://learn.microsoft.com/en-us/dotnet/fundamentals/reflection/collectible-assemblies)).
- **A collectible `AssemblyLoadContext`** (`base(isCollectible: true)`) for loaded
  assemblies. `AssemblyLoadContext.Unload()` only *initiates*; unloading "is
  'cooperative'" and finishes when no thread has a frame from those assemblies on its
  stack and nothing outside holds a strong reference or a strong/pinned `GCHandle` to
  the assemblies, their types, or instances of those types
  ([unloadability](https://learn.microsoft.com/en-us/dotnet/standard/assembly/unloadability)).

The runtime's internal per-container object is the **`LoaderAllocator`**; the debugging
recipe in the unloadability doc is literally "find what's keeping a `LoaderAllocator`
that belongs to the specific `AssemblyLoadContext` alive", via `!dumpheap -type
LoaderAllocator` then `!gcroot <address>`. All the runtime's native bookkeeping for the
assembly is grouped under that allocator so it can be freed as a unit.

### 5.2 The documented list of things that pin — read this as a checklist

From [collectible-assemblies](https://learn.microsoft.com/en-us/dotnet/fundamentals/reflection/collectible-assemblies),
the runtime will not unload while any of these exists for any type `T` in the assembly:
an instance of `T`; an array of `T`; a generic instantiated over `T` *even if empty*; a
`Type` or `TypeBuilder` representing `T`; a static reference from another dynamic type; a
`ByRef` to a static field of `T`; a `RuntimeTypeHandle` / `RuntimeFieldHandle` /
`RuntimeMethodHandle` naming `T` or a part of it; any reflection object from which `T`'s
`Type` is reachable; a method of `T` on any thread's call stack; a delegate to a static
method in the assembly. And "the runtime does not actually unload the assembly until
finalizers have run for all items in the list."

The unloadability doc adds the non-obvious ones: JIT-introduced stack temporaries and
register-held references; `RegisteredWaitHandle` callbacks; and fields on the custom
`AssemblyLoadContext` subclass itself — because "while unloading is in progress, the
runtime holds a strong GC handle to the `AssemblyLoadContext` to coordinate the unload".

Restrictions accepted to make it work: "Types in an ordinary dynamic assembly cannot have
static references to types that are defined in a collectible assembly… a
`NotSupportedException` exception is thrown"; no `DllImport`, no `calli`, no marshalling
of types defined in a collectible assembly, no COM interfaces, no thread-static
variables, reflection-emit as the only supported load mechanism (that last set is
scoped to .NET Framework in the current doc). For collectible ALCs specifically: no
C++/CLI, and ReadyToRun code is ignored.

### 5.3 Does it transfer?

**The container idea, yes. The rest, mostly not.**

- **Container-as-unit-of-liveness is the strongest idea here for Rexx**, and it maps
  cleanly onto packages: make the *package* the collectable unit; a class is live iff
  its package is live or something outside the package references it. The
  registry-pinning problem then reduces from "every per-class table entry must be weak"
  to "each package's tables are owned by the package and die with it" — which is exactly
  `LoaderAllocator`. In a Rust implementation this is very natural: per-package `HashMap`s
  owned by a package struct, dropped as a unit. **Inference.**
- **The published restriction list is the honest cost of the container approach.** Note
  what its members have in common: every one is a way for a reference to *leave* the
  container without the container knowing (a handle, a native pointer, a stack frame, a
  reflection object, a generic instantiation owned elsewhere). The Rexx equivalents are
  the ones to enumerate before committing: `.Package` / `.Method` / `.Routine` reflection
  objects, `Class~subclass` producing a class whose superclass lives in another package,
  saved `.Message` objects, external native handles, and a class on an activation's
  call stack.
- **`Type`-object identity is the trap.** In .NET, merely holding a `Type` for `T`
  prevents unload, so the container's boundary must be enforced at every place a type
  identity can escape. In Rexx a class identity is a first-class value a program can
  store anywhere, so the container cannot be an *enforcement* boundary — only a
  liveness *default*. **Inference: a Rexx design must be "package roots its classes, and
  any outside reference also roots them", i.e. plain reachability with the package as
  one more root — not .NET's "outside references are forbidden".**
- The cooperative-unload dance (`WeakReference`, `GC.Collect()` in a loop,
  `WaitForPendingFinalizers`) is an artefact of a concurrent, finalising, generational
  collector; a stop-the-world mark-sweep has none of it.

---

## 6. Question 1 — is a weak registry enough?

> If the name→class tables held weak references and the collector cleared unmarked
> entries before sweeping, would that be sound, given that a class's own method bodies
> and class-variable pool reference the class back?

**The back-reference is not a problem, and it is worth separating that from the rest of
the question.** Under mark-sweep, a class whose only inbound edges come from its own
method bodies and its own class-variable pool forms an unreachable cycle: nothing marks
it, so nothing marks them, so the whole cluster sweeps together. This is precisely the
case that defeats refcounting (§0, §1.5) and precisely the case tracing exists to
handle. Weakening the name table is *sufficient to defeat the self-reference*.

It is **not sufficient on its own**. Here are the failure modes, in the order I would
expect them to bite:

**F1 — an unrooted holder of a class handle (silent use-after-free).** Any place that
stores a class identity outside the traced root set — a cached `BehaviourHandle`, a
resolved dispatch-cache entry, a `MethodId → owning class` row, an activation's saved
`SUPER` scope, a `flat_loops`-style register array the tracer does not visit — becomes a
dangling handle the moment the registry entry is cleared. Under precise non-moving
mark-sweep the slot is *reused*, so the symptom is a wrong answer, not a crash. This is
strictly worse than CPython's version of the same omission, which is only a leak (§1.5).
It is also a defect this tree has already produced once in a different guise
(`flat_top`/`flat_loops` as the only unrooted `ObjRef` holders).

**F2 — an incomplete enumeration of registries.** The brief lists per-package name→class
tables, export tables, method dictionaries, metaclass links, subclass lists,
class-variable pools, annotation tables. Weakening *some* of them buys nothing: a class
strongly reachable from the export table is live regardless of the name table. In this
tree, `ClassRegistry` (`rust/crates/rexx-classes/src/registry.rs:40`) alone holds
`graph`, `names`, `default_names`, `object_names`, `by_name`, `by_system_name`, and
`Interp` adds `class_packages` and `package_classes`. Every one of them is native Rust
storage keyed by `ObjRef`, and every one must be given a strength decision. The
soundness condition is an *extent* claim, and extent claims have to be derived by
running an enumeration, not asserted.

**F3 — weak values are the easy half; weak keys are the hard half.** A name→class table
wants weak *values*. A table keyed *by* class — `names`, `default_names`,
`object_names`, `class_packages`, a class-variable pool keyed by class, an annotation
table — wants weak *keys with strong values*, and if any value transitively mentions the
key you have an ephemeron problem: a naive weak-key table with strong values keeps the
class alive through its own row. CPython sidesteps this in `tp_subclasses` by making the
key a plain integer (the address) and the value a weakref
(`Objects/typeobject.c:7607`). Ruby sidesteps it entirely by not having a side table —
the rows live *in* the class object (§4.1), so they die with it.

**Inference, and the recommendation:** the cheapest correct answer for the by-class
tables is Ruby's, not CPython's — **move per-class rows into the class object itself**,
so there is no side table to weaken and no clearing pass to run. Weakness is then needed
only for the genuinely inverted tables (name→class, subclass lists).

**F4 — clearing order.** V8 states the constraint as a requirement in the source: map
transitions are compacted *before* general weak-reference clearing, because the
compaction pass needs to read the not-yet-nulled weak targets
(`mark-compact.cc:2864`). CPython's `type_clear` invalidates the method cache *before*
clearing the dict (`typeobject.c:5264`). For Rexx the equivalent constraints are: run
`UNINIT` before or after clearing? — and if a class's `UNINIT` can run Rexx code, that
code can look the class up by name, and it must either find it or get a defined answer.
This tree already has a `take_uninit_classes_in_sweep_order`
(`rust/crates/rexx-classes/src/registry.rs:352`), so the ordering question is live and
concrete.

**F5 — resurrection / identity split.** If a name is cleared from a package table but
the package can re-resolve it (re-running a directive, a lazy install, a `::REQUIRES`
re-entry), a program can end up with two distinct class identities for one name. The
observable is `a~class == b~class` answering `0`, and an oracle divergence with no
memory symptom at all.

**F6 — cost.** A weak-clearing pass is a new collector phase, O(entries) per collection,
that must run between marking and sweeping with no allocation and no user code. V8 caps
its transition arrays at `1024 + 512` entries partly to keep the trim cheap
(`transitions.h:93`); a package's class table in Rexx is small, so **inference:** the
per-collection cost is negligible here and the *complexity* cost — one more ordered
phase with its own invariants — is the real price.

**Summary answer.** Sound *for the stated worry* (the back-references), yes — that is
what tracing buys you. Sufficient, no. The sufficient condition is: *every path by which
a running program can still name or reach a class is either a strong edge the tracer
follows, or is cleared in the same pause, and every place holding a raw class handle is
in the root set.* F1 and F2 are where the bugs will be, and both are enumeration
problems rather than algorithm problems.

---

## 7. Question 2 — named versus anonymous: principled, or accident?

### 7.1 It is not a rule anywhere; it is reachability with a rooted namespace

None of the four runtimes has a collector rule for "named". In Ruby the constant table is
marked strongly (`gc.c:6793`) and hangs off `rb_cObject`, a root; that is the whole
mechanism, and `Class.new` differs only in that nothing points at it. In CPython a
module-level class is a strong entry in a module dict, and modules are in `sys.modules`;
`type('C',(),{})` differs only in that nothing points at it. In V8 a Map is retained by
whatever holds it, plus a deliberate age-based pin (§3.3), with no naming concept.

So: **the distinction is principled in the sense that it is not a special case at all —
it is the ordinary rule applied to a namespace that happens to be permanently rooted.**
It is an *accident* in the sense that its practical effect ("named classes are
immortal") is a property of *how long namespaces live*, not of the classes. When
namespaces themselves become collectable — `Module#remove_const`, deleting from
`sys.modules`, unloading an `AssemblyLoadContext` — the named/anonymous asymmetry
evaporates, and .NET is the proof: it made the *namespace container* the unit of
liveness precisely so that named types could die (§5).

**This is the design lesson for Rexx:** do not build a named/anonymous rule. Build
reachability, and decide the lifetime of the *package table*. The named/anonymous split
will then fall out on its own, exactly as it does in Ruby, and it will fall out
*correctly* if packages ever become collectable.

### 7.2 Does a Rexx class ever become unreachable in practice?

I checked the tree rather than guessing. Two derivations, with the commands:

```
$ grep -rn "programs\.\(remove\|pop\|clear\|truncate\|retain\|drain\)\|
           package_classes\.\(remove\|clear\|retain\)\|
           class_packages\.\(remove\|clear\|retain\)" --include=*.rs crates/
   (no matches; exit 1)
$ grep -n "self\.programs\." crates/rexx-exec/src/lib.rs
   (every hit is .len() or .push(); none removes)
```

What that establishes:

- `Interp::programs` is a `Vec<Rc<Program>>` (`rust/crates/rexx-exec/src/lib.rs:3126`)
  that is only ever pushed to. A `ProgramId` is an index into it. **Packages are
  permanent for the process lifetime.**
- `package_classes: ProgramId → name → class` and `class_packages: class → ClassPackage`
  are only ever inserted into (`Interp::record_package_class`,
  `rust/crates/rexx-exec/src/environment.rs:1308`; `:1346` for the null case).
- `class_packages`'s own doc comment states the split already exists in the tree: "A
  class built by `~subclass` or `~mixinClass` is in this table alone, under
  `ClassPackage::Null`, because no directive installed it into any package"
  (`rust/crates/rexx-exec/src/lib.rs:3350-3352`).

**So the honest answer is: a `::CLASS` class never becomes unreachable today, and would
not under any weak-registry scheme either, because the package that names it never
dies.** The named/anonymous split is not merely likely to dominate — it is total, and it
is already encoded in `ClassPackage::Null`.

That changes what the work is worth, and I think it changes what the work *is*:

1. **The classes that can actually be collected are the run-time-created ones** —
   `~subclass`, `~mixinClass`, `.Class~new`, and whatever else mints an identity without
   filing it against a package. Those are exactly the `ClassPackage::Null` rows. A
   program that creates classes in a loop is the only workload that leaks today.
   **Inference: this is a real workload** (mixin/metaclass idioms, and any program that
   builds classes from data), but it is a small and well-delimited one.
2. **Weakening the `::CLASS` name tables buys nothing while packages are permanent.**
   Effort spent making `package_classes` weak is effort spent on a table whose owner is
   immortal. It becomes valuable only as part of making packages collectable.
3. **The larger prize, if it is wanted, is .NET's:** make the *package* the unit, drop
   its class tables and its classes together, and take the restriction list in §5.2 as
   the enumeration of what must be checked before a package can go. That is a much
   bigger piece of work and should be a separate decision, not smuggled in under "collect
   user classes".
4. **The cost side is favourable regardless**, because there is a second reason to do
   the small version: it forces the class identity into the root-set discipline. Today a
   class is invisible to the mark loop (`resolve` returns `None`), which means *nothing
   checks* that class handles are rooted — the tree cannot currently distinguish "rooted"
   from "not rooted" for a class. Making run-time classes collectable turns every
   unrooted class handle into a bug the collector can find. **Inference**, but it is the
   same argument as F1 in §6, from the other direction.

My recommendation, stated as one sentence: **collect `ClassPackage::Null` classes,
leave `::CLASS` classes rooted by their package, and say in the design that
package-level collection is the follow-on that would make the rest of it pay** — this is
Ruby's arrangement exactly (named classes rooted by a rooted namespace, anonymous ones
ordinary garbage), reached by the same route, with the .NET container as the documented
next step if package unloading is ever wanted.

---

## 8. Mechanism table

| mechanism | source | invariant that makes it sound | cost | survives translation to precise, non-moving, non-generational, STW mark-sweep, no barriers, no refcounting? |
|---|---|---|---|---|
| Static type is not a GC object (`type_is_gc` → `HEAPTYPE`) | CPython `typeobject.c:5311` | The predicate is total and checked, so an immortal object is never handed to a path that assumes collectability | one flag test | **Yes** — this is already the tree's `CLASS_SLOT_BASE` design, and CPython's asserting version is the improvement to copy |
| Explicit shutdown teardown of immortal types | CPython `_PyStaticType_Dealloc`, `typeobject.c:5026` | Immortal set is enumerable | a walk at exit | **Yes** |
| Immortal-by-refcount (PEP 683) | PEP 683; `object.h:110,239` | Encoding degrades to "slower but correct" if a stale writer touches it | branch on every incref/decref; 2–3% naive | **No** — refcount-only. The *rule* (closed build-time set, answerable from the object) transfers |
| Permanent generation | `gcmodule.c:430`, `gc.freeze` | Generational skip list | list membership | **No** — generational artefact |
| Instance → class as a strong edge in the *generic* tracer | CRuby `gc.c:7362` | Every object marks `basic.klass` with no per-type code to forget | one visit per object | **Yes, and preferable to CPython's per-type convention** |
| Instance → class as a per-type `tp_traverse` obligation | CPython c-api/typeobj; bpo-35810 | Author remembers; omission leaks (CPython) / dangles (Rexx) | none, until forgotten | **Yes, but do not** — the failure is silent and worse here |
| Weak forward transitions + strong back pointer | V8 `transitions.h:44`; `map.tq:37`; body descriptor `:1013` | Live descendants keep ancestry; dead descendants keep nothing | one maybe-weak field | **Yes — best structural fit for a class hierarchy** |
| Post-mark, pre-sweep clearing/compaction pass | V8 `ClearNonLiveReferences`, `mark-compact.cc:2764`, `ClearFullMapTransitions:3213` | Runs in the pause, after marking is final, in a stated order | O(registry) per GC; a new phase with its own invariants | **Yes**; the `RecordSlot` half drops out with moving/incremental |
| Unregister-at-free from back-pointer registries | CRuby `gc.c:3592-3594`; `class.c:129` | Entry is reachable from the dying object; list owner outlives it | O(1) per dead object; needs a sweeper hook | **Yes — cheaper than weak refs for subclass lists** |
| Rows live in the class object, not a side table | CRuby `RCLASS_*` tables, `gc.c:7365` | No side table to weaken | none | **Yes — removes the weak-key/ephemeron problem entirely** |
| Cache keyed by class, key unmarked, cleared on class death | CRuby `vm_callinfo.h:282` | The cache is a cache: a miss is safe, a stale hit is not | clearing hook at class death | **Yes** |
| Version-tagged cache with borrowed payload | CPython `_PyType_Lookup`, `typeobject.c:4725`; `PyType_Modified` | Invalidation strictly precedes teardown of what is cached | a version compare per lookup | **Yes** |
| Deliberate retention of unreachable shapes for N GCs | V8 `RetainMaps`, `mark-compact.cc:2340`; `--retain_maps_for_n_gc=2` | Bounded: an age counter guarantees eventual release | memory, bounded | Yes, but **not worth it here** (inference: Rexx classes are not created implicitly) |
| Container as the unit of liveness (`LoaderAllocator` / ALC) | .NET unloadability & collectible-assemblies docs | Every escape route out of the container is enumerated and either forbidden or counted | a documented restriction list; hard to debug | **Yes, and it is the right shape for packages** — but as a liveness default, not as .NET's enforcement boundary |
| Cooperative unload + `GC.Collect()` loop + finalizer wait | .NET unloadability doc | — | — | **No** — concurrent/finalising-collector artefact |
| Double-visit guard in `subtype_traverse` | CPython `typeobject.c:1812` | Refcount subtraction must count each edge once | a branch | **No** — mark-sweep is idempotent under repeated visits |
| Transition-array locking / accessor-invalidation contract | V8 `transitions.h:33,49` | Background readers vs mutator | locks | **No** — concurrency-only |

---

## 9. What I could not establish

- **Whether V8's `RetainMaps` age heuristic has published data behind the default of 2.**
  I have the flag and its comment (`flag-definitions.h:1728`) but no design doc
  justifying the number.
- **The current post-mitigation figure for PEP 683's overhead.** The PEP itself carries a
  TODO admitting the "performance-neutral" claim was not re-verified against the final
  branch; I did not find a superseding measurement.
- **Whether CPython has any build-time or runtime detector for a heap type that omits
  `Py_VISIT(Py_TYPE(self))`.** The documented consequence is a leak; I found no
  assertion, warning, or debug-build check that fires on the omission itself. Treat "no
  detector exists" as unverified rather than established.
- `UNSOURCED:` I did not verify how CRuby's global constant cache (`rb_vm_t`'s
  ID-keyed constant-cache table, as distinct from the per-instruction-sequence
  `imemo_constcache` I did read at `gc.c:7306`) is kept from pinning classes.
- **ooRexx specifics I deliberately did not assert:** which operations besides
  `~subclass` and `~mixinClass` mint a class identity that is never filed against a
  package (`~enhanced`, singleton/metaclass paths, `.Package~new` on a source string),
  and whether a `::CLASS` class can outlive its package in the oracle. §7.2's
  enumerations cover only the Rust tree's `programs` / `package_classes` /
  `class_packages` tables, which is what I ran the greps against; the wider question of
  what mints classes was out of scope for this survey and should be enumerated before
  the design is fixed.
