# Task 5 report: `UNINIT` (D60, D61, D69), and characterising the sweep order

BASE: c65b51641. Report opened before any building; appended as work proceeded.

## Status

Commits `3ff1055de` (the delivery) and `cf85bfd67` (the sweep made linear). Sections below are in
the order the work happened; the gate statuses are at the end.

## Log

## Reading, re-measured against the tree at c65b51641

Every "what is already there" sentence in the brief, re-read at the site:

* `rexx-core/src/heap.rs:48` `pub pending_uninit: Vec<ObjRef>` in `CollectStats`; filled at `:229`
  (`pending_uninit.push(r)` for a flagged object the mark missed), the resurrection loop follows,
  returned at `:280`. `:314` `pub fn set_uninit` is the only writer of `true` and is the only
  pusher onto `Heap::uninit`; `:339` `pub fn clear_uninit` clears the flag and `retain`s the
  registry entry away. CONFIRMED.
* `rexx-classes/src/class_graph.rs:376` `has_uninit`, `:395` `check_uninit`, `:435`
  `parent_has_uninit`, all about a class's *instances*. CONFIRMED.
* `class_graph.rs:386`-`:389`: "goes on to `if (hasUninitMethod()) requiresUninit();` (`:1222`),
  which asks the class *object's own* behaviour ... This crate has no such table." CONFIRMED, and
  `ClassClass.cpp:1224` is where that call actually sits in the oracle checkout today (the doc says
  `:1222`; the line moved or the citation was off by two -- the call is
  `requiresUninit();` at `classes/ClassClass.cpp:1224`).
* `rexx-exec/src/lib.rs:6446`-`:6449`, inside `Interp::collect_now`, carries the
  `debug_assert!(stats.pending_uninit.is_empty(), ...)` whose comment names `native_new` as the
  setter and itself as the site that owes the delivery. CONFIRMED.
* `rexx-exec/src/dispatch.rs:3961`-`:3962`, in `native_new`:
  `if interp.classes().has_uninit(class) { interp.heap.set_uninit(object); }`. CONFIRMED -- so an
  instance of a class defining `UNINIT` is flagged today and nothing ever delivers.
* `gate_table_c.rs:457` control text: still "stop running `UNINIT` before the object's storage is
  reclaimed, which is **silent**". CONFIRMED as the wording to correct.

The whole set of callers of the uninit API outside `heap.rs`/`class_graph.rs`/tests is
`dispatch.rs:3961`-`:3962` (the setter) and `lib.rs:6448` (the assertion). Nothing calls
`clear_uninit` outside `rexx-core/tests/uninit.rs`.

## The order question: what the oracle actually does

Read first, in `/home/moritz/dev/repos/ooRexx/interpreter/`, rather than inferred from transcripts.

The termination sweep is `MemoryObject::lastChanceUninit` (`memory/RexxMemory.cpp:324`), reached
from `runtime/Interpreter.cpp:279`; it calls `collectAndUninit(true)` -- `clearSaveStack()`,
`collect()`, `runUninits()` -- and then empties the table. `collect()` runs `checkUninit()`
(`:271`), which walks the table and sets `setReadyForUninit()` on every entry the mark missed;
`runUninits()` (`:337`) then walks the table again and runs each ready entry's finalizer.

**Both walks are `uninitTable->iterator()`, and `uninitTable` is an `IdentityTable`
(`RexxMemory.cpp:193`, `new_identity_table()`).** So the order is that hash table's iteration
order, which `HashContents::iterateNext` (`classes/support/HashContents.cpp:489`) defines as:
ascending bucket index `0..bucketSize`, and within a bucket the overflow chain, which
`HashContents::put` (`:226`) **appends** to -- so insertion order inside a bucket.

The bucket is `HashContents::hashIndex` (`classes/support/HashContents.hpp:197`-`:205`):

```
return (ItemLink)(index->getHashValue() % bucketSize);
```

and the comment there says why it is `getHashValue()` and not `identityHash()`. That is the whole
answer, because **`RexxClass::getHashValue()` (`classes/ClassClass.cpp:209`-`:215`) returns
`id->getHashValue()` -- the hash of the class's id STRING**, not of its address:

```
HashCode RexxClass::getHashValue()
{
    return id->getHashValue();
}
```

`RexxString::getStringHash()` (`classes/StringClass.hpp:328`-`:344`) is the Java-style fold
`h = 31 * h + stringData[i]` over the bytes, in `size_t` arithmetic (`HashCode` is `typedef size_t`,
`ObjectClass.hpp:71`), and `stringData` is `char`, signed on this platform.

`HashCollection::DefaultTableSize` is **17** (`classes/support/HashCollection.hpp:130`), and
`new_identity_table()` builds the uninit table at exactly that size
(`classes/IdentityTableClass.hpp:69`).

### The rule

**A class object's position in the termination sweep is `strhash(uppercased id) % 17` ascending,
ties broken by the order the classes entered the table.** Nothing about creation order enters it
except as the tie-break.

And the same reading explains why instances are NOT reproducible: an instance is a `RexxObject`,
whose `getHashValue()` is the inherited `identityHash()` = `((uintptr_t)this) ^ UINTPTR_MAX`
(`classes/ObjectClass.hpp:335`, `:340`), so its bucket is its ADDRESS mod 17 and moves with ASLR.
D61's "two instances, two orders over twenty runs" and D60's "reproducible for class objects" are
the same mechanism reading two different `getHashValue` overrides.

### The rule checked against every transcript already in the spec and brief, before running anything

`strhash(s) % 17`, ids uppercased by the directive:

| id | fold | % 17 |
|---|---|---|
| `M1` | 31*77 + 49 = 2436 | 5 |
| `M2` | 31*77 + 50 = 2437 | 6 |
| `D1` | 31*68 + 49 = 2157 | 15 |
| `K` | 75 | 7 |
| `P` | 80 | 12 |
| `M` | 77 | 9 |
| `K1` | 31*75 + 49 = 2374 | 11 |
| `K2` | 31*75 + 50 = 2375 | 12 |

* D60's falsifier (`Meta`/`M1`/`D1`/`M2`, creation order `Meta,M1,D1,M2`) fires
  `uninit-meta M1` / `uninit-meta M2` / `uninit D1`. Predicted 5, 6, 15 -- **matches**, and it is
  exactly the pair the spec calls out: `D1` is created before `M2` and fires after it.
* The brief's first inherited arm (`::CLASS P` then `::CLASS K SUBCLASS P`) fires `K` then `P`.
  Predicted 7 then 12 -- **matches**, and it explains the spec's puzzle that "P is declared first
  and fires second".
* The brief's second arm (`::CLASS M MIXINCLASS Object` then `::CLASS K INHERIT M`) fires `K` then
  `M`. Predicted 7 then 9 -- **matches**.
* The spec's "twenty runs of two directive classes gave declaration order `uninit k1` then
  `uninit k2`" -- predicted 11 then 12, declaration order **by coincidence of the two ids**.

Four transcripts, four agreements, none of them ordered by creation. To be confirmed by running a
program designed to make the two disagree.

### The rule, confirmed by running

Probes in a fresh empty directory, absolute paths, oracle wrapped in
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 20 .../build/bin/rexx FILE )`,
three descriptors separately. `parse version` at the time of these runs:
`REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`.

**e2 -- bucket order against declaration order, made to disagree everywhere.** Four directive
classes each with `::METHOD uninit class`, declared `C`, `B`, `A`, `D`; buckets 16, 15, 14, 0.
Predicted `D A B C`, the reverse of declaration. Three runs, rc 0, empty stderr:

```
start u D u A u B u C
start u D u A u B u C
start u D u A u B u C
```

**e4 -- twenty classes `A`..`T`, declared in alphabetical order.** Predicted from
`strhash % 17` ascending, ties by declaration: `D E F G H I J K L M N O P Q A R B S C T`. Three
runs, rc 0, empty stderr, all three:

```
start u D u E u F u G u H u I u J u K u L u M u N u O u P u Q u A u R u B u S u C u T
```

Twenty entries and the bucket count is still 17, so the table had not expanded at that size.

**e3 / e3b -- three ids that collide in bucket 7 (`AB`, `ZZ`, `K`), declared in two different
orders.** `HashContents::put` appends to a bucket chain, so the prediction is declaration order
within the bucket, and reversing the declarations reverses the output. Three runs each:

```
e3   (declared AB, ZZ, K)   start u AB u ZZ u K
e3b  (declared K, ZZ, AB)   start u K u ZZ u AB
```

**e5 -- the "runtime-built classes fire before directive-installed ones" witness, falsified as a
rule.** Directive classes `D` (bucket 0) and `E` (bucket 1) plus a runtime
`.Object~subclass('T', .Meta)` (bucket 16). Predicted `u D`, `u E`, then the runtime one. Three
runs, rc 0:

```
start built T u D u E u-runtime T
```

So a runtime-built class fires **last** here. What the spec observed was hash order, not a
runtime/directive split.

**e6b -- a class and an instance in the same sweep: NOT reproducible.** `::CLASS C` with a
class-side `UNINIT` (bucket 16) and one live instance of `::CLASS S`. Twenty runs:

```
19  start u instance u C
 1  start u C u instance
```

which is the address-hashed bucket of the instance moving under ASLR, exactly as the reading
predicts. (The same program with the class at bucket 0 instead gives `u D u instance` 20 times out
of 20, because bucket 0 is the lowest a bucket can be -- a shape that looks reproducible and is
not, which is why the discriminating probe puts the class at bucket 16.)

### The answer to the order question

**Characterised, cheaply, and confirmed by running.** For class objects the order is

> `strhash(id) % 17` ascending, ties broken by the order the classes entered the table,

where `strhash` is `h = 31*h + byte` over the id as stored, in `size_t` arithmetic.

**D61 can be lifted for class objects and must stay for anything involving an instance.** The
instance half of the same mechanism is a bucket over the object's address, so a sweep mixing an
instance `UNINIT` with a class `UNINIT` is not reproducible on the oracle (19/20 above) and no row
may depend on it.

**The bound on the rule, stated so it can be checked rather than assumed.** `bucketSize` is 17 only
while the table has not expanded; `HashCollection::expandContents` doubles and rehashes
(`classes/support/HashCollection.cpp:93`), which reorders everything. Twenty entries do not expand
it, measured above. A program with enough `UNINIT` objects to expand the table is outside what this
rule predicts, and no corpus program is anywhere near it.

## Where the oracle delivers, measured

Not only "at a collection" and "at termination" -- the oracle's delivery points are narrower than
that, and two of these results change what this task has to build.

`MemoryObject::collect` marks eligible objects with `setReadyForUninit()`; `runUninits()` is what
actually calls the finalizer, and it runs from `checkUninitQueue()` at method and routine return
(`execution/RexxActivation.cpp:705`, `execution/NativeActivation.cpp:1361`) and at activity
boundaries (`concurrency/Activity.cpp:249`, `:324`, `:3440`) -- **never from inside the collector**.
`GC('force')` is the exception: `expression/BuiltinFunctions.cpp:3033` calls
`memoryObject.collectAndUninit(false)`, which is collect-then-run inline.

| probe | program | oracle, three runs each |
|---|---|---|
| e9 | `o = .K~new` / `drop o` / `call gc 'force'` / `say 'b'` | `a b uninit` -- **the finalizer did NOT run at the forced collection** |
| e11 | `o = .K~new` / `say 'b'` / `drop o` / `call gc 'force'` / `say 'c'` | `a b uninit c` -- it did |
| e12 | same, but 200,000 `.Array~new` instead of the forced collection | `a b c uninit` -- deferred to termination |
| e13 | same, but 200,000 string concatenations | `a b c uninit` -- deferred to termination |

**e9 against e11 is the reproducible shape's real precondition**, and the spec's phrasing "`drop`
then `call gc 'force'`" leaves it out: an intervening clause between `~new` and `drop` is required,
because without one the object is still protected and the forced collection does not reach it.
`collectAndUninit(false)` keeps the save stack, which is what holds it. **Any witness of delivery 1
must have that clause**, and a witness written the spec's way is a witness that cannot fire.

**e12 and e13 say an ordinary allocation-driven collection does not deliver mid-program at all**
in either shape, so this crate does not need a mid-program safe point: queue at the collection,
drain at `GC('force')` and at termination, and every measured oracle transcript is matched. That
also keeps the change off the clause loop entirely.

### What a failing `UNINIT` does, measured

| probe | oracle |
|---|---|
| e14 `x = 1/0` in the main body, class-side `UNINIT` present | rc 214, stdout `main` / `uninit ran`, the 42.3 traceback on stderr -- **the sweep runs after an error** |
| e15 `exit 7`, class-side `UNINIT` present | rc 7, stdout `main` / `uninit ran` |
| e16 `y = 1/0` **inside** a class `UNINIT` at termination | rc 0, stdout `main` / `in uninit`, **empty stderr** |
| e17 the same inside an instance `UNINIT` under `GC('force')` | rc 0, stdout `a b in uninit c`, empty stderr, the rest of the program runs on |
| e18 `raise syntax 40.900` inside that `UNINIT` | identical to e17 |

So a condition raised inside `UNINIT` is **swallowed whole**: no stderr, no status change, no
interruption. That is `UninitDispatcher` under `activity->run` (`memory/RexxMemory.cpp:381`-`:384`).

## BASE state, re-measured at c65b51641

Built from `git archive c65b51641 | tar -x` into a scratch tree with its own `CARGO_TARGET_DIR`,
so the worktree was untouched.

```
obdes.rex
  oracle             rc=0   stdout  main / uninit ran     stderr empty
  BASE release ir    rc=0   stdout  main                  stderr empty
  BASE release tw    rc=0   stdout  main                  stderr empty

do i = 1 to 200000 ; o = .K~new ; end ; say 'main'   ::CLASS K / ::METHOD uninit / nop
  oracle             rc=0   stdout  main                  stderr empty
  BASE debug ir      rc=101 stdout empty
                     stderr  thread 'rexx-interp' panicked at crates/rexx-exec/src/lib.rs:6447:9:
                             an object was resurrected for UNINIT and nothing here runs a finalizer
  BASE debug tw      rc=101 same
  BASE release ir    rc=0   stdout  main                  stderr empty

arm1: ::class p / ::method uninit class / ::class k subclass p
  oracle three runs  rc=0   stdout  main / uninit on K / uninit on P
  BASE release ir    rc=0   stdout  main
  BASE release tw    rc=0   stdout  main

arm2: ::class m mixinclass Object / ::method uninit class / ::class k inherit m / ::method uninit class
  oracle three runs  rc=0   stdout  main / uninit on K / uninit on M
  BASE release ir    rc=0   stdout  main
  BASE release tw    rc=0   stdout  main
```

## The design this task builds

* `Heap` keeps the flagged objects alive already -- `collect` resurrects an unreachable flagged
  object and retains its registry entry -- so **the flag is the root** and a queued object needs no
  park. Nothing else has to hold it.
* `Interp` gains an ordered ready list. `collect_now` stops asserting and appends
  `stats.pending_uninit` to it, deduped, which is the oracle's `setReadyForUninit`.
* The drain removes the entry first (`Heap::clear_uninit`), then roots the object as a temporary,
  then sends `UNINIT` -- the order `runUninits` uses (`removeAndAdvance`, then `ProtectedObject`,
  then `activity->run`), and the order that makes a collection inside the finalizer safe.
* `GC('force')` drains right after `Interp::collect_now`, which is `collectAndUninit`.
* Termination runs a sweep over **every** live flagged object (D69), then over every class object
  with a class-side `UNINIT` (D60) in the order above.
* A `Failure::Raised` out of a finalizer is discarded, per e16/e17/e18. A `Failure::Loud` is not:
  this crate saying it cannot run a construct must stay visible, and swallowing it is exactly the
  silent gap the loud rule exists to exclude.

## What was built

* `rexx-core/src/heap.rs`: `Heap::uninit_flagged`, the flagged handles oldest first.
* `rexx-classes/src/class_graph.rs`: `ClassGraph::check_uninit` gains
  `ClassClass.cpp:1224`'s other half -- it now also asks the class-**side** behaviour for
  `UNINIT` and enters the class object in a new `uninit_classes` list. `ClassGraph::uninit_classes`
  reads it back.
* `rexx-classes/src/registry.rs`: `ClassRegistry::uninit_classes_in_sweep_order`, which stable-sorts
  that list by `uninit_bucket`, the oracle's `strhash(id) % DefaultTableSize`.
* `rexx-exec/src/lib.rs`: `Interp::uninit_ready`; `collect_now`'s `debug_assert!` replaced by the
  append to it; the termination sweep in `execute`, after `run_deferred_replies`.
* `rexx-exec/src/dispatch.rs`: `run_one_uninit`, `run_ready_uninits`, `run_termination_uninits`,
  and the `UNINIT` message name.
* `rexx-exec/src/builtin/state.rs`: `GC('force')` drains the ready list, which is what makes it
  `collectAndUninit` rather than `collect`.

## The three deliveries, measured against the oracle

Debug build of the worktree, both engines, three descriptors separately, fresh empty directory.

| program | oracle | crate ir | crate tree-walker |
|---|---|---|---|
| `obdes.rex` | rc 0, `main` / `uninit ran` | identical | identical |
| c1, forced collection, one instance | rc 0, `start` / `built K` / `uninit ran` / `after-gc` | identical | identical |
| c2, class-scope root, one instance | rc 0, `start` / `after-gc` / `instance uninit` | identical | identical |
| c3, `::CLASS K SUBCLASS P` | rc 0, `main` / `uninit on K` / `uninit on P` | identical | identical |
| c4, `::CLASS K INHERIT M` | rc 0, `main` / `uninit on K` / `uninit on M` | identical | identical |
| c5, four classes declared `C B A D` | rc 0, `main` / `uninit on D` / `uninit on A` / `uninit on B` / `uninit on C` | identical | identical |

Empty stderr on every cell. c2 is delivery 2's witness and its shape is what makes it one: the
instance is rooted by a class-scope variable, so the forced collection cannot reach it and the
finalizer lands **after** `after-gc`. c1 is delivery 1's and lands **before** it.

**The two inherited arms agree** and are corpus rows rather than open items, which is what the
order question being answered buys.

## A divergence this task did not close, with its transcripts

**When a driven collection reaches a just-created instance.** The oracle holds recently allocated
objects out of a collection -- `MemoryObject::holdObject` pushes every new object onto a
`PushThroughStack` of `Memory::SaveStackSize` (`memory/Memory.hpp:107`, value 10), and
`GC('force')` is `collectAndUninit(false)`, which keeps that stack. This crate has no such hold, so
a forced collection reaches an object the oracle's still protects.

Measured, `o = .K~new` then N padding clauses each allocating one string, then `drop o`,
`call gc 'force'`, `say 'c'`; three runs at each N, rc 0 and empty stderr throughout:

```
N       oracle                crate (ir and tree-walker)
0..8    a c uninit            a uninit c
9..     a uninit c            a uninit c
```

So the threshold is between 8 and 9 padding clauses, which is what a ten-entry hold stack with the
instance in it predicts. `nop` in place of a padding clause does not move it, and two `GC('force')`
calls in a row do not either.

**Not closed here, and stated rather than absorbed.** Modelling it is a ring of recently allocated
handles rooted from `Interp::alloc_with` -- the hottest path in the interpreter, a change to
collection reachability for every object rather than for `UNINIT`, and a `collect_stress`
interaction. That is a decision about this crate's collector, not about `UNINIT`, and D59a's
licence is explicitly about class objects and does not cover it. It is written into the plan's
Task 6 so the phase decides it rather than discovering it.

Before this task every one of these programs printed no `UNINIT` at all on this crate, so the
divergence replaced is wider than the one left.

## The case that would have caught the debug abort

`corpus/lang/uninit_instance_collected.rex` is that case: an instance whose class defines `UNINIT`
becomes garbage and a driven collection reaches it. Run from a fresh empty directory, three
descriptors separately.

```
BASE debug ir            rc=101  stdout empty
                         stderr  thread 'rexx-interp' panicked at crates/rexx-exec/src/lib.rs:6447:9:
                                 an object was resurrected for UNINIT and nothing here runs a finalizer
BASE debug tree-walker   rc=101  same
BASE release ir          rc=0    stdout  start / built K / after-gc          stderr empty
HEAD debug ir            rc=0    stdout  start / built K / uninit ran / after-gc   stderr empty
HEAD debug tree-walker   rc=0    same
oracle                   rc=0    stdout  start / built K / uninit ran / after-gc   stderr empty
```

So it fails on BASE both ways -- rc 101 in debug, and a **silent** missing `uninit ran` at rc 0 in
release, which is the shape no gate saw -- and passes after.

## An environmental failure that reads like a build failure

The first attempt at the release test run died with
`failed to write .../invoked.timestamp: No space left on device (os error 28)`, exit 101. The
filesystem under `/home/moritz/dev/repos` was momentarily at 100% of 1.7T; a `df` a minute later
read 555G used and 1.1T available, and the identical command then ran. Nothing in this task filled
it (this task's scratch tree is on `/tmp`, which had 59G free throughout). Recorded because the
message is indistinguishable from a compile failure at a glance.

## Commit

`3ff1055de` -- "Deliver UNINIT, and characterise the termination sweep's order", read back with
`git log --oneline -1`.

Files: `rexx-core/src/heap.rs`, `rexx-classes/src/class_graph.rs`, `rexx-classes/src/registry.rs`,
`rexx-exec/src/lib.rs`, `rexx-exec/src/dispatch.rs`, `rexx-exec/src/builtin/state.rs`;
`rexx-classes/tests/uninit_sweep_order.rs` (new), `rexx-exec/tests/coverage.rs`,
`rexx-exec/tests/gate_table_c.rs`; five `corpus/lang/uninit_*.rex` with their
`sourceline_oracle/*.txt`; `corpus/phase-5b.txt`; and the plan.

## The in-crate test for the order

`crates/rexx-classes/tests/uninit_sweep_order.rs` carries four cases, each an oracle transcript
recorded before the ordering was written: the four-class contradiction of declaration order, the
twenty-class run, the shared-bucket tie-break (which no corpus program can see, because no corpus
program's ids collide), and that an instance-side `UNINIT` does not put the class object in the
sweep -- measured, `say 'main'` with `::class k` / `::method uninit` prints `main` and nothing else
at rc 0 on both the oracle and this crate.

## Machine contention during this task, recorded because it changes what a reading means

Three separate readings on this machine were shaped by other work running beside this task, and
each of them looks like a property of the change if it is not framed:

1. **A build failed at `No space left on device`** and the same command succeeded minutes later
   (above).
2. **The performance sitting could not be taken on the first attempt**, with the same message Task 1
   recorded: `rexx-arms: perf stat produced no cycles:u and instructions:u pair for pinned tw
   small`. **Framed with the probe before and after, and the reading is not the same as Task 1's.**
   Minutes before that attempt, `perf stat -x, -e cycles:u,instructions:u` on this crate's binary
   read `100.00` for both events; minutes after it, three consecutive runs on the pinned binary read
   `49.00/50.00`, `50.00/50.00` and `49.00/50.00`. So on this machine the pair **is** schedulable and
   was schedulable during this session -- what stops it is another process holding counters, not a
   machine property. `/proc/loadavg` read `46.75 49.92 37.63` at the failing attempt, against a
   session doing nothing but wait.
3. Both a `cargo build` and a `cargo test` of this task's own sat for tens of minutes behind that
   load.

**The consequence for Task 1's finding**: its conclusion that the guard "could not be satisfied"
because "the machine exposes one usable hardware performance counter" is a machine-property claim,
and the 100.00/100.00 reading above falsifies it as stated. The sitting is retakeable; it needs a
quiet machine, not a different machine.

## The controls, recorded as run

Each mutation is applied to a scratch copy of `cf85bfd67` (`git archive | tar -x`, its own
`CARGO_TARGET_DIR`), so the shared worktree was never mutated. The command in every cell is

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast
```

read unpiped. **Unmutated control on that scratch tree: `277 of 277 matching`, status 0.**

Every mutation is run twice: once with `corpus/phase-5b.txt` and `EXPECTED_SUBSET_5B` as committed
(**full**), and once with this task's six entries removed from both (**reduced**) -- which is the
"can fail is not adds coverage" check, and the reduced arm is the one that says whether anything
already in the corpus catches the mutation.

### C1 -- `Interp::run_termination_uninits` returns immediately

This is `obdes`'s committed control, "skip the termination sweep", and it is recorded as run.
**`272 of 277 matching`, status 101**, five mismatches, every one of them **silent** -- rc 0 and
empty stderr on both sides, differing on stdout alone:

```
gate-tables/concepts/obdes.rex        rust "main\n"
                                      oracle "main\nuninit ran\n"
lang/uninit_instance_retained.rex     rust "start\nafter-gc\n"
                                      oracle "start\nafter-gc\ninstance uninit\n"
lang/uninit_class_inherited.rex       rust "main\n"
                                      oracle "main\nuninit on K\nuninit on P\n"
lang/uninit_class_mixin.rex           rust "main\n"
                                      oracle "main\nuninit on K\nuninit on M\n"
lang/uninit_class_sweep_order.rex     rust "main\n"
                                      oracle "main\nuninit on D\nuninit on A\nuninit on B\nuninit on C\n"
```

`lang/uninit_instance_collected.rex` is **not** among them, and that is the two deliveries being
separable rather than an omission: its finalizer comes from `GC('force')` draining the ready list,
which C1 does not touch.

**Nothing already in the corpus catches it**: the reduced arm is `271 of 271 matching`, status 0.
The five rows above are the only witnesses of the termination sweep in this tree.

### C2 -- `ClassGraph::check_uninit` never enters a class object in `uninit_classes`

`ClassClass.cpp:1224`'s half deleted. `273 of 277`, status 101, four mismatches, all silent: the
same four class rows as C1 (`obdes`, both arms, the order witness) and **not**
`uninit_instance_retained.rex`, which is the instance half of the sweep and is untouched by it.
Reduced arm `271 of 271`, status 0.

### C3 -- the sweep runs class objects in the order they were entered, not in bucket order

`uninit_classes_in_sweep_order`'s `sort_by_key` deleted, which is exactly the implementation an
earlier draft of D60 called for. `274 of 277`, status 101, three mismatches, all silent and all
**order-only** -- the finalizers all run and run in the wrong order:

```
lang/uninit_class_inherited.rex     rust "main\nuninit on P\nuninit on K\n"
                                    oracle "main\nuninit on K\nuninit on P\n"
lang/uninit_class_mixin.rex         rust "main\nuninit on M\nuninit on K\n"
                                    oracle "main\nuninit on K\nuninit on M\n"
lang/uninit_class_sweep_order.rex   rust "main\nuninit on C\nuninit on B\nuninit on A\nuninit on D\n"
                                    oracle "main\nuninit on D\nuninit on A\nuninit on B\nuninit on C\n"
```

**`obdes` stays green under it**, which is the point of the three added rows: the gate row this task
owns has one class with a `UNINIT` and cannot see the ordering rule at all. Reduced arm
`271 of 271`, status 0.

### C4 -- `GC('force')` collects without draining the ready list

`276 of 277`, status 101, **one** mismatch, and it is the only witness of delivery 1:

```
lang/uninit_instance_collected.rex   rust "start\nbuilt K\nafter-gc\nuninit ran\n"
                                     oracle "start\nbuilt K\nuninit ran\nafter-gc\n"
```

Silent, and it is a *late* delivery rather than a missing one -- the finalizer still runs, at
termination, which is exactly the shape a program that only checked "did `UNINIT` run" would call a
pass. Reduced arm `271 of 271`, status 0.

## A quadratic sweep, found by running the brief's own witness program

The debug-abort witness -- `do i = 1 to 200000 ; o = .K~new ; end` under `::METHOD uninit` -- is
also a scale test, and at `3ff1055de` it was **killed at the 20-second bound** where the oracle
answers `main` at rc 0. Measured on the debug build, `REXX_ENGINE=ir`, `/usr/bin/time`:

**CORRECTED at the fix round -- the pre-fix column below replaces one that did not reproduce.**
The original table read 0.30, 0.86, 9.67 and 49.55 for `3ff1055de`. Its first two cells are that
program's *post-fix* times, which is the stale-binary shape, and this task's own report already
records one mutation batch thrown away for exactly that reason. The re-take was interleaved -- both
revisions alternating inside one loop, three rounds -- with each tree checked against
`git show REV:path` by `sha1sum` before it was built and each binary's mtime recorded, so a build
that did not happen could not answer for one that did.

```
instances    3ff1055de              cf85bfd67          instrument
  16,000     2.57 / 2.57 / 2.57     0.30 x 3           mine, interleaved, debug, ir
  64,000     37.01 / 37.07 / 36.93  0.86 x 3           mine, interleaved, debug, ir
 128,000     155.51 / 155.66 / 181.75  1.63 / 1.66 / 1.66   the reviewer's, same method
 200,000     killed at the 300 s bound  2.60            the reviewer's
 200,000, class with no UNINIT
             1.34 / 1.35 / 1.33     1.29 / 1.30 / 1.29  mine, interleaved
 oracle at 200,000   0.39 s
```

**The fix is larger than the original table said, not smaller.** The quadratic term is already
visible at 16,000 -- 2.57 s against 0.30 -- where the old column said the two were equal, and at
64,000 it is 43x rather than the 1.0x claimed. The control is unchanged: the identical program with
a class that defines no `UNINIT` reads 1.33 s and 1.29 s, so the ordinary path is untouched and
linear.

The 128,000 and 200,000 rows are the reviewer's measurements, not re-taken here -- at 200,000 the
pre-fix arm exceeds a five-minute bound, and three rounds of it interleaved would cost an hour to
confirm a direction two cheaper sizes already establish.

**Two quadratic terms, both in the delivery.**

* `Heap::clear_uninit` is a `retain` over the registry, so clearing one flag per finalizer is one
  pass per object. `Heap::take_uninit_flagged` and `Heap::clear_uninit_all` do a batch in one pass.
  The batch is then parked for the length of the run, because the flag they just cleared was its
  only root -- which is exactly `runUninits` removing the table entry and then holding the object in
  a `ProtectedObject` (`memory/RexxMemory.cpp:363`-`:373`).
* `Interp::collect_now` scanned the ready list before appending, and the collector re-reports an
  unreachable flagged object on **every** collection, so that scan was one pass per object per
  collection. `Object::ready_for_uninit` is the oracle's `setReadyForUninit`: resurrect every time,
  report once.

**Pinned by a test rather than by this paragraph.**
`rexx-core/tests/uninit.rs`'s `a_still_unreachable_flagged_object_is_reported_once_and_resurrected_every_time`
asserts both halves, and each is red under its own mutation, run and restored from a copy:

```
delete the `ready_for_uninit` test in `Heap::collect`
    -> uninit.rs:141 assertion `left == right` failed: reported once, not once per collection
       left: [ObjRef(0)]  right: []
move `resurrect.push(r)` inside that test
    -> uninit.rs:146 "and still alive to be finalized"
restored from the copy, byte-identical, 7 passed
```

`taking_the_flagged_objects_clears_every_flag` is the second new case: a rooted flagged object is
never reported by a collection and is still answered by `take_uninit_flagged`, which is D69's shape
at the unit level.

Committed as `cf85bfd67`, "Make the UNINIT sweep linear in the objects it finalizes".

## A mutation batch that had to be thrown away, and why

The first mutation batch restored each source file with `cp -p` between mutations. **`-p`
preserves the mtime**, so cargo saw no change and kept the previous mutation's compiled crate: C4's
and C5's runs reddened three class-order rows that neither mutation can affect, which is C3's
mutation still in the binary. The tell was in the transcripts -- `uninit on P` before `uninit on K`
under a mutation that only touched `GC('force')`.

The whole batch is discarded rather than reasoned about. The replacement harness copies without
`-p`, `touch`es every restored file, prints which sources differ from pristine before each build,
and runs a second unmutated pass at the end.

## The phase gate, at cf85bfd67

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

status **101**, and 101 is the expected reading here: the three rows still red are Tasks 2, 3 and
4's.

```
agree   loud=no  5b   obdes Object Destruction and Uninitialization  depth 1  parent xcremet
                       provide.xml:709   obdes.rex

table C   5b: 6 rows, 3 not yet `agree`
table D   5b: 2 rows, 0 not yet `agree`

3 row(s) of gate table C owned by a closing or closed phase do not `agree` with the oracle;
the first of them are ["gate-tables/concepts/objcla.rex", "gate-tables/concepts/usesem.rex",
"gate-tables/concepts/methodsbyclass.rex"].
```

`obdes` is `agree`. It was `diverge-stdout` at BASE, and it is the only 5b concept row this task
owned. The control text the same run prints beside it is the corrected one.

## The same oracle mechanism was already analysed in this tree, for a different collection

`Interp::hash_collection_indexes` (`rexx-exec/src/run.rs`) carries `DO OVER`'s version of this
reading -- bucket-ascending `HashContents` iteration, `31*h + byte` for a string key, `identityHash`
for anything else -- and its conclusion is the opposite of this task's: it **declines** to model the
order and sorts the keys instead, because "reproducing it needs `HashContents` modelled *and* this
crate's insertion sequence matching the oracle's for that table, which holds for a package's public
classes and not for `.environment`".

Both conclusions are right, and the difference is exactly the two conditions that doc names. For the
uninit table both hold: **the key is a class object, whose `getHashValue` is its id string's**, and
**the insertion sequence is the order entries are made**, which this crate can match. Neither holds
for `.environment`.

**CORRECTED at the fix round.** This paragraph originally ended "which this crate already matches
because `check_uninit` is called from the same places the oracle calls it". That was false, and it
was the argument the ordering implementation rested on. The oracle also enters the table from
`RexxClass::updateSubClasses` (`classes/ClassClass.cpp:1052`), which both `inherit` (`:1360`) and
`uninherit` (`:1413`) end in and which recurses into every subclass; this crate called
`check_uninit` from three places, none of them an inherit. Four deliveries went missing silently as
a result -- the review's CRIT-1, and the fix round's first commit. What is true is the weaker claim
above: the sequence is *entry order*, and matching it is a thing this crate has to do at each of the
oracle's entry points rather than something that follows from class creation order.

That doc, `corpus/README.md`'s determinism rule and this task's `uninit_bucket` are three statements
of one mechanism; nothing in the tree implemented it before this task.

## What is left for the phase, written into the plan rather than only here

* **The instance-side collection-reachability divergence** (the ten-entry hold stack), written into
  the plan's Task 6 with its transcripts and the two ways out.
* **D61 lapses over class-against-class order and stays over anything involving an instance.** The
  spec's own wording makes this automatic -- D60 says "until it is characterised D61 covers class
  objects too" -- so no spec edit is needed, and the plan's Task 5 section now carries the
  characterisation.

### C5 -- the termination sweep runs class objects only, never a live instance

The instance loop deleted from `run_termination_uninits`.

**The first attempt at this one did not run**, and that is the harness working rather than a gap:
`mutate.py` asserts its pattern is present exactly once, and `cf85bfd67` had replaced the loop it
named. It aborted with `AssertionError: ('rust/crates/rexx-exec/src/dispatch.rs', 0)` and produced
no verdict, where a `sed` would have silently changed nothing and reported the unmutated suite as
green. The pattern was updated and the arm re-run.

`276 of 277`, status 101, **one** mismatch, and it is the only witness of delivery 2:

```
lang/uninit_instance_retained.rex   rust "start\nafter-gc\n"
                                   oracle "start\nafter-gc\ninstance uninit\n"
```

Silent, and an **absence** rather than a late delivery -- the finalizer never runs at all, which is
the shape D69 exists to close and the one a collection-driven delivery alone would produce. Reduced
arm `271 of 271 matching`, status 0.

### What the five controls say together

| mutation | full arm | rows it reddens | reduced arm |
|---|---|---|---|
| C1 skip the termination sweep | 272 of 277, 101 | `obdes`, `uninit_instance_retained`, both arms, the order witness | 271 of 271, 0 |
| C2 never enter a class object in the table | 273 of 277, 101 | `obdes`, both arms, the order witness | 271 of 271, 0 |
| C3 sweep in entry order, not bucket order | 274 of 277, 101 | both arms, the order witness | 271 of 271, 0 |
| C4 `GC('force')` does not drain | 276 of 277, 101 | `uninit_instance_collected` | 271 of 271, 0 |
| C5 the sweep skips live instances | 276 of 277, 101 | `uninit_instance_retained` | 271 of 271, 0 |

Every reduced arm is `271 of 271 matching` at status 0: **nothing already in the corpus catches any
of these**, so each of the six added rows earns its place rather than merely being able to fail.
And no two rows are redundant -- C3 is caught only by the three order-sensitive rows and not by
`obdes`, C4 only by `uninit_instance_collected`, C5 only by `uninit_instance_retained`.

**The batch closes with a second unmutated pass**, which is what says the restore worked rather
than leaving the last mutation compiled in: `277 of 277 matching`, status 0, same as the opening
one. The per-run self-check printed exactly one `MUTATED <file>` line before each build, and it was
the intended file every time.

## The five gates, at d708491a7

Run from `rust/`, each status read unpiped from its own status file.

```
cargo fmt --all --check                                     0
cargo clippy --workspace --all-targets -- -D warnings        0
cargo test --release --workspace                             0    no failing test binary
REXX_CORPUS_GATE=1 cargo test --release --workspace          0    corpus 277 of 277 matching
```

The phase-gate command at the same commit:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast          101

  agree  loud=no  5b  obdes Object Destruction and Uninitialization  ...  obdes.rex
  table C  5b: 6 rows, 3 not yet `agree`
  table D  5b: 2 rows, 0 not yet `agree`
  3 row(s) ... do not `agree` ... ["gate-tables/concepts/objcla.rex",
  "gate-tables/concepts/usesem.rex", "gate-tables/concepts/methodsbyclass.rex"]
```

101 is the expected reading: those three rows are Tasks 2, 3 and 4's.

### Gate 5, the debug run, and why it has no status yet

```
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast
```

`command -v memcap` answers present, so this is the real command and not the `ulimit` substitute.

**Two attempts were lost to the harness rather than to the tree.** The first got through 23 test
binaries with **zero** `test result: FAILED` and was still inside
`the_exempt_set_matches_the_current_failures` when the background shell that owned it was killed;
its log stopped at that point and its exit status was never written, so there is nothing to read as
a gate line. The partial log is kept at `scratchpad/task5/g5-orphaned.log`.

**No status is claimed for gate 5 in this report.** A third attempt is running, launched under
`setsid` so it outlives its shell, writing `scratchpad/task5/g5.log` and `g5.status`. The
controller can read the status from that file; until it exists, gate 5 is **not measured** and this
report says so rather than inferring it from the partial log.

The four other gates and the phase gate are measured above, all at `d708491a7`.
