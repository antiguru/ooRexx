# Phase 5j — what the oracle does about class lifetime, measured 2026-09-09

Every probe run from a fresh empty directory, oracle under the standard wrapper, crate
via `target/debug/rexx-run`, stdout/stderr/rc read on three separate descriptors. All
runs below are rc 0 with empty stderr on both sides unless stated.

## 1. The oracle collects a runtime-created class

`subclasses.rex`

```rexx
before = .Object~subclasses~items
c = .Object~subclass('TEMPC')
say 'created:' (.Object~subclasses~items - before)
drop c
call gc 'force'
say 'after drop+gc:' (.Object~subclasses~items - before)
```

| | created | after drop+gc |
|---|---|---|
| oracle | 1 | **0** |
| crate  | 1 | **1** |

This is D59a's `~subclasses` divergence, reproduced. Silent, rc 0 both sides.

## 2. A live instance pins its class, and only the instance

`instpins.rex` — create `TEMPC`, keep an instance, drop the class variable.

| | instance alive | after dropping the instance |
|---|---|---|
| oracle | delta 1 | **delta 0** |
| crate  | delta 1 | delta 1 |

So the oracle's rule is ordinary reachability: instance → class is a strong reference,
and nothing else pins a runtime class. This is the target semantics for 5j.

## 3. A `::CLASS`-declared class is not collected

`declared.rex` — a `::CLASS DECL` directive, then `gc('force')`.

Oracle: `declared: still DECL`. Its package holds it, and there is no way to drop the
binding. **So 5j's practical surface is runtime-created classes** — `~subclass`,
`.Class~new`, mixins — not declared ones.

## 4. NEW DEFECT, and it is not one of D59a's four

`weaklive.rex`

```rexx
c = .Object~subclass('TEMPC')
w = .WeakReference~new(c)
call gc 'force'
v = w~value                      /* c is still live here */
```

| | live class | live ordinary object (control) |
|---|---|---|
| oracle | `weakref reads TEMPC` | `weakref reads an Object` |
| crate  | **`weakref reads NIL`** | `weakref reads an Object` |

**A `WeakReference` to a class reads NIL in the crate even while the class is live.**
The control on an ordinary object is correct on both sides, so the failure is specific to
class identities — consistent with `Heap::resolve` answering `None` for a class slot and
the weak-clearing pass at `heap.rs:191-202` reading that as "target gone".

Reproduced three times: `weaklive.rex`, `weakref.rex`, `declared.rex`.

**This inverts D59a**, which lists *"a `WeakReference` to a dropped class still answering
it"* as a time-boxed consequence. The measured behaviour is the opposite: a weak
reference to a **live** class already answers NIL. It is a wrong answer today,
independent of whether classes are ever collected.

**And it made an earlier probe look green over it.** `weakref.rex` — take a weak
reference, drop the class, force a collection — prints `after: cleared` on *both*
engines. That agreement is a coincidence: the crate prints `cleared` because it always
does for a class, not because it collected anything. Only the live-class control
separates the two.

## 5. The weak-reference defect's mechanism, located

`heap.rs:196` in the weak-clearing pass:

```rust
let target_alive = self.resolve(target).is_some_and(|t| self.marks[t]);
```

with the comment above it stating the rule deliberately: *"'Dead' includes unresolvable:
a target whose slot was already freed, or whose generation has moved on, died in an
earlier cycle and its reference must still clear."*

That rule is right for an arena handle and wrong for a class identity. `resolve` does
`self.slots.get(slot as usize)?`, and a class slot is at or above `CLASS_SLOT_BASE`,
which is past the end of `slots` — so every class target is "unresolvable", hence "dead",
hence cleared. The class being immortal is exactly why it reads as dead.

**Under today's semantics the correct predicate is one term wider**: a class identity is
always live, because nothing collects one. That is a fix available now, independent of
5j's lifetime change, and it makes the crate match the oracle on all three probes above.

**And it is an argument about 5j's central fork.** If class identities stay outside the
arena, this predicate has to ask "is this class still registered", which `rexx-core`
cannot answer — it knows nothing of `rexx-classes` — so the liveness would have to be
passed into `collect`. If class identities become ordinary arena objects, `resolve`
answers correctly and the predicate needs no special case at all. The weak pass is the
first place where "keep the reserved range" costs a layering violation.
