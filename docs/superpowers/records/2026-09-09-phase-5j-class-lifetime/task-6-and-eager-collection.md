# Phase 5j — Task 6, and two new observables of an already-licensed cause

## 1. Task 6: no Rexx program can send to a collected class

The plan asked for a witness in which a handle to a collected class reaches a send, and said that
if no such program can be written, saying so is the stronger result. It cannot be written, and this
is what was run rather than reasoned:

| program | what it tries | crate |
|---|---|---|
| `dangle1.rex` | hold the class only through a `WeakReference`, collect, then send | `weakref cleared, nothing to send to` |
| `dangle2.rex` | hold it in an array, drop the variable, collect, then send | `through the array: T2` — the array is a root, so the class lives |
| `dangle3.rex` | ask the superclass for it after the collection | `nothing dead is listed` — the sweep scrubbed the entry |

Every place a program can put a class handle is a root: a variable, an array, a stem, a class
variable. The one place that is not a root is a `WeakReference`, and that clears in the same
collection that frees the class. So a dangling class handle is not a program-reachable state.

**What remains is internal, and has a different instrument.** An `Interp` table holding a class
handle the collector does not root would dangle, and no Rexx program can be written to prove it
does not. `run_program_collect_every_alloc` is the instrument for that, and `Interp::object_roots`
is the list of places to look — which is why Task 2 rooted the three package class tables.

## 2. The crate collects a class the oracle does not, in one shape

Found while trying to construct Task 6's witness. Both sides stable over ten runs each.

```rexx
before = .Object~subclasses~items
c = .Object~subclass('E3')
drop c
call gc 'force'
call gc 'force'
say 'delta after two collections:' (.Object~subclasses~items - before)
```

oracle `1`, crate `0`. **Two forced collections, so it is not a one-cycle lag.**

The discriminating experiment is one clause:

```rexx
before = .Object~subclasses~items
c = .Object~subclass('E1')
say 'a clause between create and drop'      /* <-- the only difference */
drop c
call gc 'force'
say 'delta:' (.Object~subclasses~items - before)
```

oracle `0`, crate `0` — **they agree.** The same shape reaches a `WeakReference`: with no
intervening clause, the oracle's weak reference to a dropped class still answers after two forced
collections where the crate's has cleared.

**This is the cause `phase-4-exclusions.txt`'s "A DRIVEN COLLECTION HERE REACHES A JUST-ALLOCATED
OBJECT THE ORACLE'S CANNOT" already licenses**, seen through two observables that row does not
mention: a `~subclasses` count, and a `WeakReference`. That row's own witness pads with eight
clauses precisely to stay clear of the threshold, which is evidence the project already knew the
shape. Collection timing is not a specified observable under the standing GC-ordering licence, so
this is licensed; what is new is where it can be seen.

**It also means `class_collected.rex`'s `say` between the creation and the `drop` is load-bearing**,
not incidental drafting. A version of that witness without the intervening clause would record a
divergence rather than agreement. Said here so nobody "tidies" the program later.

**Not chased further.** Whether the threshold is clauses, allocations, or an evaluation-stack slot
is a question about the oracle's internals that nothing in this phase turns on, and the observable
is what the licence is written over.
