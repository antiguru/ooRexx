# Phase 5j — what the tree says, surveyed 2026-09-09 at d74d8bbb1

Read-only survey. Every claim names the file and line it came from.

## 1. A class identity has no generation, and that inverts the arena's safety property

`ObjRef::class(id)` is `ObjRef::heap(CLASS_SLOT_BASE | id, 0)` — `handle.rs:211-216` —
and the doc says why: *"Generation zero, always: a class identity names no slot, so it
has no generation to move on, and fixing it at zero is what lets `ObjRef::class_id` read
the range back exactly."*

The arena's stale-handle safety comes from the generation bump on free (`heap.rs:120-130`):
a dead handle *misses* rather than aliases. Class identities have no generation, so if
5j recycles a collected class's id, a stale handle **aliases the next class defined**.
That is a wrong-answer defect, not a crash.

Two ways out, and the spec has to pick one:

* never recycle ids — monotonic, 2^31 of them, and the collection frees only the class's
  payload (its variable pool, its method dictionary, its registry rows), leaving the id
  itself burned;
* give class slots a real generation, which changes `ObjRef::class`, `ObjRef::class_id`
  and `is_class_slot` (`handle.rs:88, 200-235`) and every decode that matches on them.

## 2. `ClassRegistry` holds every class strongly, in six places

`registry.rs:39-65`: `graph: ClassGraph`, `names`, `default_names`, `object_names`,
`by_name`, `by_system_name`. Every one is a `NameMap`, and the type's own doc says
*"they are all written once per class while the library is being built, before a
program's first clause runs"* (`registry.rs:36-38`). **That premise is false for a
user class** and is already false today; 5j makes it load-bearing, because a weak
registry has to support removal at collection time.

## 3. One named global root per class, keyed by a formatted string

`run.rs:3207`: `.add_global(&format!(".class-variables {class}"), owner)`. This is the
second, independent reason a class never dies — even once the identity itself is
collectable, the arena object holding its variable pools is globally rooted. The
`format!` per class is also an allocation the design should not need.

## 4. A flag that already draws exactly the boundary — established by running

`ClassKind` is `Regular | Mixin` (`class_graph.rs`), which is not the axis. But
`ClassGraph::is_rexx_defined` (the oracle's `setRexxDefined`) **does** partition kernel
from user, and the call sites say so:

* `native_classes.rs:362, 363, 406` set it on every kernel class the image build creates;
* `lib.rs:6278` sets it for a program's class **only** `if self.library_bootstrap`, so a
  user program's `::CLASS` and every runtime-created class carry it false.

Confirmed by running, not only by reading, through the refusal at `dispatch.rs:5092`:

```rexx
c = .Object~subclass('UC');  c~define('M', 'return 1')   /* user   */
.Array~define('ZZQQ', 'return 1')                        /* kernel */
```

| | user class | kernel class |
|---|---|---|
| oracle | define allowed | refused rc 98 |
| crate  | define allowed | refused rc 98 |

So the boundary Moritz asked for — *"built-in classes with a static lifetime, all others
collected"* — is already a flag the tree carries, already agrees with the oracle, and is
already observable from a Rexx program. 5j does not have to invent it.

(An earlier draft of this survey said neither existing flag was designed as a lifetime
boundary and that the question needed running. The running has been done and the answer
is the opposite of what the names suggested.)

## 5. The machinery the licence was waiting for exists

`Body::WeakRef(ObjRef)` (`body.rs:235`), traced as a non-reference (`body.rs:803`), and
cleared to `ObjRef::NIL` in a post-mark pass before the `UNINIT` pass
(`heap.rs:166-202`). D59's licence was taken *"without having weak references"*; they
are here.

## 6. `Interp::object_roots` is 5j's checklist

Landed d74d8bbb1. Every field bound `_` whose stated reason is that class identities
are not arena objects flips to a live root question the day classes are collected:
`merged_public_classes`, `package_classes`, `package_public_classes`, `class_packages`,
`object_model`, and the key halves of `class_variables`, `method_objects` and
`annotations`. The compiler cannot catch that flip — the field is still there and still
binds `_`. The comments are the only record.
