# The collection classes

Survey: `docs/superpowers/plans/2026-09-06-collections-survey.md`, committed at `2f041fd49`. Read it
first for the vacuity finding; this spec does not restate it, it acts on it.

The survey closed with three questions for Moritz. This spec answers all three from measurement
taken 2026-09-06 at `3c6e60f16`, and adds a fourth finding the survey did not have: **the shape of
the remaining work is not the one the survey assumed**, because the C++ interpreter's own
inheritance is visible in two files that can be scanned, and scanning them moves the estimate.

---

## 1. Where the work actually is, measured

The survey scoped from `corpus/method-bodies.txt`'s verdict column and warned that the column
undercounts. It does. But the same table, joined against the interpreter's own two definition
sites, also says something the verdict column cannot: **which loud rows are C++ bodies and which
are Rexx bodies this crate already runs.**

The two sites are `interpreter/memory/Setup.cpp` -- whose `StartClassDefinition(X)` blocks and
`AddMethod("Name", entry, args)` lines are the native instance behaviour, with `A_COUNT` or a
literal maximum in the third operand -- and `interpreter/RexxClasses/CoreClasses.orx`, whose
`::CLASS`/`::METHOD` directives this crate already parses and installs at start
(`Interp::bootstrap_library`).

**The mixin leverage has already been spent.** Joining the two against the loud rows of the surveyed
classes, in the scope order the oracle's own `~superClasses` reports, puts almost every loud row on
a native entry point. The Rexx-level bodies -- `Collection`'s `union`, `difference`, `subSet`,
`xor`, `disjoint`, `equivalent`, `intersection`, and `OrderedCollection`'s sort family -- are
already installed and already reading `answers`, which is why they are not in the loud set. Only
`List`'s two sort rows and `CircularQueue`'s own overrides land on Rexx bodies.

This is the finding that reshapes the phase, and it cuts the opposite way from the survey's
hopeful reading of shared names: **there is no free tier.** What there is instead is one
already-working example of the pattern to copy, which is worth more.

**But the C++ shares bodies where the names do not.** `Setup.cpp` has an
`InheritInstanceMethods(source)` macro that copies a whole native behaviour before the derived
class overrides part of it, and it is used across the mapped classes and from `Array` into `Queue`.
So the loud rows collapse onto far fewer distinct C++ entry points than they have names, and they
collapse along the *entry point*, not along the name -- `HashCollection::hasIndexRexx` is one body
answering `hasIndex` across the mapped classes, while `hasItem` reaches a body of its own on a
`Set`, on a `Bag` and on a `Relation`, and the shared one everywhere else.

**D88 -- the entry point is the unit of work, not the name.** The survey counted distinct method
names and said the ratio flatters, which was right for the wrong reason: the ratio flatters because
a name is not a body, and it also *understates*, because `InheritInstanceMethods` gives one body
several classes' rows. Neither number is the estimate. The estimate is the distinct
entry points the join produces, and a task is sized by those -- with one correction the join cannot
make for itself: the entry point is the token `Setup.cpp` writes, and two tokens sometimes name one
C++ function reached through inheritance, so the distinct-token count is an upper bound on distinct
bodies rather than a count of them.

**D89 -- the join is a committed artifact, not a paragraph.** Every number above is derived by
reading two read-only upstream files, and a number derived that way and then written into prose
rots the moment either file moves under it -- and neither is pinned by anything in this tree except
`rexx-lib`'s sha256 on the `.orx`. So the partition is generated into `corpus/collection-scopes.tsv`
and re-derived by its own test on every run, the way `corpus/refusal-sites.tsv` is: columns for the
class, the documented method, the arm, the scope that defines it, whether that scope is native or
Rexx, and for a native scope the entry point and its `Setup.cpp` arity operand. A row whose scope
cannot be resolved is a failure, not a blank -- that is what catches an upstream rename.

---

## 2. The three questions the survey left open

**D90 -- question 1, the third tier's storage. There is no third tier.** The survey grouped
`Table`, `IdentityTable`, `Relation`, `Properties` and `CircularQueue` as "no backing storage at
all". Two of those five do not belong in the group, and the reason changes what has to be built.

`CircularQueue` and `Properties` are not native classes. `CoreClasses.orx` declares them
`::CLASS 'CircularQueue' subclass queue` and `::CLASS 'Properties' subclass Directory`, they appear
in no `StartClassDefinition` block, and the oracle's `~superClasses` confirms `The Queue class` and
`The Directory class`. Their storage is their superclass's. What refuses in this crate is not their
storage but their construction, and the crate says so in its own words -- measured 2026-09-06 at
`3c6e60f16`:

```rexx
a = .MyArr~new(3)
a[1] = 'x'
say a[1] a~items
::CLASS MyArr SUBCLASS Array
```

Oracle rc 0, `x 1`. This crate rc 120, `rexx-exec: ~new on a subclass of Array is not implemented
(Phase 5)`. `.Properties~new` and `.CircularQueue~new(3)` fail the same way one message later, at
`a value that is not a hash collection` and `a value that is not an array` respectively, because
`~new` on a Rexx subclass of a native class produces an instance with no native body.

So those two classes cost one mechanism, not two class implementations, and that mechanism is owed
to user code anyway: `::CLASS MyArr SUBCLASS Array` is ordinary Rexx that any program may write.
**The remaining three -- `Table`, `IdentityTable`, `Relation` -- do need a store this crate does not
have**, and so do `Set` and `Bag`, which the survey put in the tier above on the strength of a
divergence rather than a probe.

**D91 -- question 2, the receiver sweep, folds in and is answered by this phase's instrument.** The
5c follow-up left 105 rows across 13 classes whose zero-argument probe cannot discriminate because
the documented receiver is empty, and Phase 5f's Task 0 measured that a populated receiver would
sharpen `Queue`, `Array` and `List` rows. Those classes are these classes. A `RECEIVER_OVERRIDES`
entry for a collection is worth nothing until the collection can hold something, and is nearly free
afterwards -- so the sweep is not a separate job, it is the last step of each store's task.

**D92 -- question 3, `Bag~put`/`Set~put` does not wait for the phase, and its rule is already
written down.** `HashCollection.cpp`'s `IndexOnlyHashCollection::validateValueIndex` is the whole
specification: the value argument is required; the index is optional but, if given, must satisfy
`isIndexEqual` against the value or the send raises `Error_Incorrect_method_nomatch`; and the index
is then made the value. `Bag` and `Set` are the two `IndexOnlyHashCollection` subclasses. This is a
small self-contained correction to an argument layer that already exists, and it is the phase's
first commit rather than one of its tasks.

---

## 3. What is in scope, and the split

**D93 -- `RexxQueue` is out of scope.** It is not a collection. `StreamClasses.orx` declares it, its
class methods are `EXTERNAL 'LIBRARY REXX rexx_create_queue'` and friends, and its instance methods
are the external data queue's -- `push`, `queue`, `pull`, `queued`, `lineIn`, `lineOut`, `say`,
sitting on the interpreter's native API and on `RexxQueueMethods.cpp`. It is I/O wearing a
collection's name, it shares no storage and no primitive with the classes here, and it belongs with
`Stream` in Phase 7. Its rows stay loud and the phase's gate excludes it by name, not by silence.

**D94 -- the phase splits in two, and the split is the backing store.** One phase over the
remaining classes is roughly twice Phase 5f's whole size in rows and more than that in entry
points, and the split that falls out of the join is not arbitrary: the ordered classes share
`Array`'s native behaviour by `InheritInstanceMethods`, the mapped classes share
`IdentityTable`'s, and no entry point crosses the line except `RexxObject::makeArrayRexx`, which
every class's `makeArray` row lands on.

* **Phase 5g -- the ordered collections.** `Array`, `Queue`, `List`, `CircularQueue`.
* **Phase 5h -- the mapped collections.** `IdentityTable`, `Table`, `Set`, `Bag`, `Relation`,
  `Directory`, `StringTable`, `Properties`, `Stem`.

Ordered first, for three reasons that are about evidence rather than taste. `Array`'s store already
exists in this crate (`Body::Array`, with `dimensions`), so the contents protocol D95 introduces can
be designed against a store that works instead of one being invented in the same task. `Queue`
inherits `Array`'s whole native behaviour upstream, so the second class is nearly the first one
again and any protocol that does not make it so is wrong early rather than late. And
`RexxObject::makeArrayRexx` is the one shared body, so it lands where its store is real.

**5h's task list is not written here.** Its sizing depends on Task 0's instrument -- the same
instrument whose absence is what the survey is a warning about -- and writing a task list against
the verdict column now would be the exact mistake the survey exists to prevent. Its shape is in §5.

---

## 4. The design

**D95 -- one contents protocol, mirroring the interpreter's own virtuals.** The C++ does not write
`allItems` per class. `HashCollection` and `ArrayClass` implement the shared surface once against a
small set of virtuals, and each concrete class supplies the virtuals. The shared surface is the
large half of the loud rows -- `allItems`, `allIndexes`, `supplier`, `index`, `makeArray`, `empty`,
`isEmpty`, `items`, `hasIndex`, `hasItem`, `remove`, `removeItem` -- and every one of them is
mechanical given iteration, lookup and deletion.

So this crate gets an internal protocol with the same shape, and the shared surface is written once
per *store*, not once per class. What a store supplies: iterate (index, item) pairs in the store's
own order, look up by index, put, remove by index, count. What the protocol must **not** hide is the
part that genuinely differs per class, which is the key semantics of D96 and the argument layer of
D97.

**D96 -- key semantics are four, and the interpreter names all four.**
`classes/support/HashContents.hpp` defines the variants and which class uses each: identity
comparison hashed by `getHashValue()`; `EqualityHashContents` comparing by `equalValue` and hashed
by `hash()`; `MultiValueContents`, which is the equality one with `put` remapped to `addFront`, for
`Relation` and `Bag`; and `StringHashContents`, comparing by `memCompare` and hashed by the string
hash, for the string-keyed classes. **What `hash()` is, exactly, because the loose reading is wrong in a way that matters.**
`ObjectClass.hpp` and `ObjectClass.cpp` between them define three things, not one.
`identityHash()` is the address complemented. `getHashValue()` defaults to `identityHash()` and is
overridden -- `RexxString` caches a string hash, `.NIL` holds a static one. And `RexxObject::hash()`
branches: for a **base-class** object it is `getHashValue()`, and for any **other** object it
*sends the `HASHCODE` message* and decodes the binary string that comes back, raising if the send
answers nothing.

So the `HASHCODE` body Phase 5f Task 4d landed is reached by this store only for objects of a user
class -- exactly the objects whose `::METHOD hashCode` an author writes to make their instances
usable as keys. For a string, a number or a collection, the store must use the internal hash and
never the message. An implementation that always sends `HASHCODE` is observably wrong: it makes a
user's `hashCode` override change where a *string* key lands.

Note the identity variant's own comment: it matches on reference identity but hashes with
`getHashValue()` rather than `identityHash`. That distinction is deviation 4's neighbourhood and is
where an `IdentityTable` implementation that "obviously" uses the handle will diverge.

**D97 -- the argument layer is per entry point and comes from `Setup.cpp`, not from the docs.** The
third operand of each `AddMethod` is the interpreter's own arity: a literal is a maximum, `A_COUNT`
is `Arity::Counted`. The generated table of D89 carries it, so a row's `Arity` is read from the
join rather than guessed from the documentation -- which is where Phase 5f's `Arity::Fixed(n)`
misreadings came from. **A shared body still needs its own argument checking per class** where the
upstream overrides it: `Set`'s `hasItem` is `IdentityTable::hasIndexRexx` at arity 1, `Bag`'s is
`BagClass::hasItemRexx` at arity 2, and a protocol that routes both to one Rust body with one arity
is wrong in a way no zero-argument probe can see.

**D98 -- the new store is GC-visible from its first commit.** `Body::Native`'s existing map is keyed
by bytes and holds `ObjRef` values; an object-keyed store holds `ObjRef` in *both* positions, and a
key the collector cannot see is a use-after-free that a small test will not produce. Whatever
`Body` variant the mapped store lands as is walked by the collector in the same commit that
introduces it, with `collect_stress` as the witness rather than a unit test.

---

## 5. The gate

Inherited from Phase 5f unchanged, because the reason it was written still holds: the verdict
column cannot see a stub, and for these classes it cannot see a wrong signature either.

* **The gate is the corpus programs**, under `REXX_CORPUS_GATE=1`. The plain `--test corpus` binary
  is report mode and exits 0 on a divergence.
* **`method-bodies.txt` is a drift check only.** No row of a class in scope may move to `diverge`,
  and no row that was answering may stop. `loud` -> `answers` is reported, never required.
* **Every witness sends a short argument list as well as a good one**, because the shared
  missing-argument raiser makes those rows agree in the table for free.
* Every new corpus program is filed in `corpus/phase-5c.txt`, in `EXPECTED_SUBSET_5C`, and as a
  `crates/rexx-parse/tests/sourceline_oracle/<name>.txt`.
* **And one addition this phase needs.** A collection row's `answers` verdict is not evidence, and
  the instrument that replaces it is Task 0's: a re-probe of every row in scope with a real argument
  list, committed as a table, so that a row this phase leaves alone is a row someone has looked at
  rather than one whose verdict happened to be green. The phase's own progress is read from that
  table and not from the verdict column.

---

## 6. What neither phase does

* `RexxQueue` (D93).
* The `Collection` mixin bodies, which already run.
* `Stream` and anything reached through the native API.
* Performance. `Body::Array`'s `Vec<Option<ObjRef>>` and whatever the mapped store becomes are
  correctness structures; a hash collection's bucket layout is not this phase's subject and the
  oracle's iteration order is licensed to differ anyway only where it is already licensed
  (`oorexx-hash-iteration-order`: unseeded and bucket-ordered, string keys reproduce, identity keys
  do not reproduce even across runs of the oracle itself). **Where the oracle's own order is not
  reproducible, a witness may not depend on it** -- that is a corpus program that cannot be written,
  not a divergence to chase.

## 7. Risks

* **The instrument is the phase's load-bearing measurement and it is a probe of my own.** Probe
  discipline applies at full strength: a re-probe harness that sends a wrong argument list produces
  a table that looks exactly like a right one. Task 0's own red control is a row it must classify
  wrongly if the harness is broken.
* **`Stem` is a `MapCollection` whose store already exists** (`Body::Stem`, with tombstones) and
  whose semantics are the language's, not the collection framework's. It is in 5h by inheritance
  and may not fit the protocol; if it does not, it gets its own task rather than a widened protocol.
* **The subclass-`~new` mechanism of D90 is not scoped by this spec.** It is named as the blocker
  for two classes and as ordinary user-facing Rexx, but what it costs -- which `Body` variant a
  subclass instance carries, and how `Primitive` resolution changes from identity to descent -- is
  a question for 5g's plan and may be large enough to be its own task.
