# The collection classes

Survey: `docs/superpowers/plans/2026-09-06-collections-survey.md`, committed at `2f041fd49`, for the
vacuity finding it exists to record: a documented method sent no arguments agrees about an *arity
error*, so an `answers` verdict on a collection row is not evidence that anything works.

**Read the survey for that finding and for nothing else. Its counts do not reproduce.** It states
479 instance rows, 289 `loud` and 188 `answers` across the fourteen classes it names. Filtering
`corpus/method-bodies.txt` to those classes' instance arm gives 439, 267 and 172 -- the same at
`2f041fd49`, at `3c6e60f16` and at HEAD, so the table did not move under it. Nothing below is
derived from the survey's arithmetic.

This spec was reviewed on 2026-09-06 before any code was written against it, and the review found
the first version wrong in six places. **Every correction is marked in the decision it belongs to,
because the wrong version is the one a reader is likely to reconstruct.** Probes and outputs:
`.../scratchpad/review-collections-spec-fable-9a7c/`.

---

## 1. Where the work is, and how to ask

Almost every `loud` row of these classes is a C++ body. The Rexx bodies of `Collection` and its
mixins -- `union`, `difference`, `subSet`, `xor`, `disjoint`, `equivalent`, `intersection` -- are
installed and running at start (`Interp::bootstrap_library`), which is *why* they read `answers`
rather than `loud`. **There is no free tier.** The leverage the survey hoped for from shared names
was spent before the phase began.

**The Rexx-bodied `loud` rows are these, measured, and the first version of this spec named only the
third line.** `sort` and `stableSort` on `Queue`, `List` and `CircularQueue`, all at scope
`OrderedCollection`; `CircularQueue`'s own `insert makeArray makeString string supplier` and its
`of`; and the `of` class-arm row on each mapped class, at scope `MapCollection`. Note what this
corrects: the sort *family* does not already answer. Only `sortWith` and `stableSortWith` do, and
only because `use strict arg comparator` raises before the body reaches `makeArray`.

**`Setup.cpp` shares native bodies where the names do not**, through `InheritInstanceMethods(source)`
-- `Queue` from `Array`, the mapped classes from `IdentityTable`, `Directory` from `StringTable`,
`Bag` from `Relation`. So rows collapse along the *entry point* rather than the name.

**D88 -- the entry point is the unit of work, and it bounds the work from both sides.** The token
`Setup.cpp` writes is a citation, not a body identity, and it is wrong in both directions:

* **an upper bound on distinct bodies.** `Set`'s `HasItem` is written `IdentityTable::hasIndexRexx`,
  and `IdentityTableClass.hpp` declares no such member -- it is `HashCollection::hasIndexRexx`
  through C++ inheritance. Counting tokens overcounts bodies.
* **a lower bound on distinct behaviours**, which the first version of this decision missed
  entirely. One token reaches three behaviours selected by the contents class the receiver
  allocated: `HashCollection::putRexx` on a `Relation` or `Bag` reaches `MultiValueContents::put`
  and its `addFront`, and on a `Set` or `Bag` passes `IndexOnlyHashCollection::validateValueIndex`
  first. A task sized by tokens alone undercounts exactly where the semantics are hardest.

**D89 -- the scope column comes from the oracle, not from a file scan.** The first version built the
whole join by scanning `Setup.cpp` and `CoreClasses.orx`, and that recipe is defective: it misses
`RemoveMethod` (`Setup.cpp:792-804` strips `sort sortWith stableSort stableSortWith makeString
toString` and three more from `Queue` *after* `InheritInstanceMethods(Array)` copied them in, moving
nine documented rows to Rexx bodies), it misses `HideMethod`, and it misses the prolog's phony
inherits at `CoreClasses.orx:80-87`, which put `Set~union`, `Bag~union`, `Relation~union` and their
neighbours at the class's own scope on a *mixin* body -- `SetMixin~union` copies then adds, where
`Collection~union` is single-valued. Three separate ways to attribute a row to the wrong body.

Ask the oracle instead. **`instance~instanceMethod(name)~scope~id` resolves every inherited scope**,
over all 471 documented rows with no misses. The call that returns `.nil` -- and that sent the first
version down the file-scanning path -- is the one sent to the *class object*, whose behaviour holds
class methods only:

```
.Array~instanceMethod('UNION')                  ->  The NIL object
.Array~new~instanceMethod('UNION')~scope~id     ->  OrderedCollection
.Queue~new~instanceMethod('SORT')~scope~id      ->  OrderedCollection
.Table~new~instanceMethod('HASINDEX')~scope~id  ->  Table
```

So `corpus/collection-scopes.tsv` is derived as: **scope from the oracle** by that call, then
`Setup.cpp` keyed by (scope, name) for native-or-Rexx and the token -- present in that scope's
post-`InheritInstanceMethods` table means native with that token, absent means Rexx. That leaves
`RemoveMethod`, `HideMethod`, inherit order and the phony inherits unable to corrupt the scope
column at all. The file is committed and re-derived by its own test, the way
`corpus/refusal-sites.tsv` is; a row whose scope resolves nowhere is a failure, not a blank.

**The Method object cannot tell native from Rexx** -- `~source~items` is 0 for `CircularQueue`'s
Rexx `queue` and `Array`'s native `[]` alike, and `~package~name` is `REXX` for both -- which is why
`Setup.cpp` is still read for that column and only that column.

---

## 2. The three questions the survey left open

**D90 -- question 1, the third tier. There is no third tier, and the first version of this decision
got the mechanism wrong.** It claimed `CircularQueue` and `Properties` fail because `~new` on a
Rexx subclass of a native class produces an instance with no native body, one mechanism owed to user
code anyway. Measured, there are three different things happening and one of them is not a subclass
effect at all:

| probe | oracle | this crate, both engines |
|---|---|---|
| `.Queue~new~items` (no subclass) | `0` | rc 120 `a value that is not an array` |
| `.CircularQueue~new(3)~items` | `0` | rc 120, **the same message** |
| `.MyST~new; s['k']='v'; say s['k']` (subclass `StringTable`) | `v` | **`v`, rc 0** |
| `.MyDir~new; d['k']='v'` (subclass `Directory`) | `v` | rc 120 `not a hash collection` |
| `.MyArr~new` (subclass `Array`), no message sent | `0` | rc 120 `~new on a subclass of Array` |

* **`CircularQueue` says nothing about subclassing.** Plain `.Queue~new` has no store either and
  fails identically, so every observation the first version drew from `CircularQueue` was an
  observation about `Queue`. Its Rexx `init` and `size` run *today*, because the instance is an
  ordinary instance with a variable pool.
* **`Properties` is blocked by a deliberate design, not a missing mechanism.**
  `native_directory_new` (`dispatch.rs:7839`) branches: the exact `.Directory` class gets the hash
  body, a subclass gets a plain instance so `expose` works, and the pair is pinned by the test
  `a_directory_subclass_keeps_the_instance` (`dispatch.rs:11786`).
* **A `StringTable` subclass already carries a working store** and round-trips at rc 0, taking the
  other branch (`Primitive::StringTable(ObjRef)`, `dispatch.rs:1366`). `TraceObject subclass
  StringTable` (`CoreClasses.orx:3993`) has shipped on it. The mechanism is not missing; the two
  conventions coexist on purpose.
* **`Array` refuses at `~new`** (`dispatch.rs:12068`), before any message.

**So the real subject is not "subclass `~new` is unimplemented". It is one instance carrying a
native store *and* an object variable pool at once**, and the witness that both are needed on one
object is `CircularQueue~init`'s `expose size` (`CoreClasses.orx:1726-1728`). That is a
better-defined task than the first version described, and a larger one.

**D91 -- question 2, the receiver sweep, folds in.** The 5c follow-up's 105 rows across 13 classes
(`records/2026-09-04-phase-5c-followup/task-0-report.md:59`) are these classes. A
`RECEIVER_OVERRIDES` entry is worth nothing until the collection can hold something and nearly free
afterwards, so the sweep is each store's last step, not a separate job.

**D92 -- question 3, `Bag~put`/`Set~put`, and it is smaller than the first version claimed.** The
rule is `IndexOnlyHashCollection::validateValueIndex` (`HashCollection.cpp:1129-1142`): the value is
required; the index is optional but, if given, must satisfy `isIndexEqual` against the value or the
send raises 93.949; the index then becomes the value. `Set` and `Bag` are its only subclasses.
Re-measured: `b~put('x')` is oracle rc 0 `1` against this crate's rc 168 `88.901`, and
`b~put('x','y')` is oracle rc 163 `93.949`.

**But `.Bag~new` has no store**, so the argument-layer fix moves rc 168 to rc 120 -- loud instead of
wrong. That is worth doing and it is not the oracle's answer. The first version called this "a small
self-contained correction" landing as the phase's first commit while D90 two paragraphs above said
`Set` and `Bag` need a store; both cannot be true. **It lands first, it is worth landing, and it
closes nothing** -- 5h Task 3's store closes it.

---

## 3. What is in scope, and the split

**D93 -- `RexxQueue` is out of scope.** `StreamClasses.orx:439` declares it; its class methods are
`EXTERNAL 'LIBRARY REXX rexx_create_queue'` and its `push queue pull lineIn queued empty` are
`EXTERNAL` too. `~superClasses` is `Object` alone -- no native store, no `HashContents`, no
`ArrayClass` entry point. Phase 7's, with `Stream`.

**D94 -- the phase splits in two, and the split is the backing store.**

* **Phase 5g -- the ordered collections.** `Array`, `Queue`, `List`, `CircularQueue`.
* **Phase 5h -- the mapped collections.** `IdentityTable`, `Table`, `Set`, `Bag`, `Relation`,
  `Directory`, `StringTable`, `Properties`, `Stem`.

The seven hash classes share `IdentityTable`'s native behaviour. **`Stem` shares nothing** -- its
table is `StemClass::*` throughout with no `InheritInstanceMethods` -- and the first version of this
sentence said the mapped classes share `IdentityTable`'s behaviour without that exception. `Stem` is
in 5h by inheritance from `MapCollection` and by nothing else.

Ordered first: `Body::Array` exists, so the protocol of D95 is designed against a store that works;
`Queue` inherits `Array`'s behaviour upstream, so the second class tests the protocol immediately.

**D99 -- `Supplier` is in scope and belongs to 5g.** Every `supplier` row in *both* phases lands on
it -- `Collection~supplier` is `.supplier~new(self~allItems, self~allIndexes)`
(`CoreClasses.orx:753`) -- and it is unimplemented here:

```
.Supplier~new(.Array~of('x1','x2'), .Array~of(1,2))~available
    oracle rc 0    crate rc 120  method "AVAILABLE" of class "Supplier"
```

Its `Available Index Next Item Init` are `SupplierClass::*` (`Setup.cpp:1624-1628`). Thirteen
`supplier` rows cannot be witnessed until those exist. The first version mentioned it only as
something a task might report.

**D100 -- the `of` class-arm rows are blocked outside both phases.** The mapped classes' `of` runs
`MapCollection~OF` (`CoreClasses.orx:1238`), whose `args = arg(1, 'a')` is unimplemented at
`crates/rexx-exec/src/builtin/state.rs:1135`. It is a BIF option, it is not collection work, and it
belongs to whoever owns `ARG`. Named here so it is not silently inherited by a collection task.

---

## 4. The design

**D95 -- one contents protocol, mirroring the interpreter's own virtuals.** The C++ writes the
shared surface once against a small set of virtuals. So does this crate: iterate (index, item) pairs
in the store's order, look up by index, put, remove by index, count. What the protocol must not hide
is the key semantics of D96 and the argument layer of D97.

**D98 -- and `makeArray` is not part of the shared surface, which the first version got exactly
backwards.** It said `RexxObject::makeArrayRexx` is "the one body every class in both phases lands
on -- write it once". The token is shared because `ObjectClass.cpp` defines it as
`return makeArray();`, a **virtual**, and the answers differ per store:

```
Directory  makeArray: k1 k2    allItems: v1 v2      (HashCollection::makeArray -> allIndexes)
Array      makeArray: p r      allIndexes: 1 3      (ArrayClass::makeArray    -> allItems)
Stem       makeArray: b a                           (StemClass::makeArray     -> tailArray)
```

Hash collections answer **indexes**; `Array` and `List` answer **items**; `Stem` answers its tail
array. Writing it once produces items where every 5h class needs keys.

**D96 -- the hash, keyed on `isBaseClass()` and not on being a string.** `ObjectClass.hpp` defines
three things: `identityHash()` is the address complemented (`:340`); `getHashValue()` defaults to it
and is overridden by `RexxString` and `.NIL` (`:335`, `:622`); and `RexxObject::hash()` branches --
`if (isBaseClass()) return getHashValue();` else it **sends `HASHCODE`** and decodes the string,
raising if the send answers nothing.

**The consequence the first version drew from this is false.** It said an implementation that always
sends `HASHCODE` would let a user's override change where a *string* key lands. There is no such
override:

```
.String~define('HASHCODE', ...)   ->  rc 158, Error 98.985
                                      User additions are not allowed to the REXX language classes
```

and for a base string the two paths are value-identical anyway. **The branch is observable one step
over, on a subclass**, and that is the witness to write:

```rexx
t = .Table~new;  t['gamma'] = 5;  k = .MyStr~new('gamma')
say t[k] t~hasIndex(k)
::CLASS MyStr SUBCLASS String
::METHOD hashCode        -- with this method:    The NIL object 0
  return '4141414141414141'x   -- without it:    5 1
```

So the rule is the oracle's: a **base-class** object hashes internally, and anything else -- a user
class *and* a `String` subclass -- gets the message. **The trap for this crate is that "has a string
value" and "is a base-class object" are different questions**, and the first version of this
decision conflated them.

`IdentityTable` compares by reference identity and still hashes with `getHashValue()`; measured, an
`IdentityTable` with string keys iterates reproducibly across oracle runs and `it['alpha'~copy]` is
`.NIL`. An implementation hashing the handle would not reproduce. That is the observable, and it is
writable as a witness.

**D97 -- the argument layer is per entry point and comes from `Setup.cpp`.** A literal third operand
is a maximum, `A_COUNT` is `Arity::Counted`, and the scopes table carries it verbatim so a row's
`Arity` is read rather than guessed. A shared body still needs its own checking per class where
upstream overrides it: `Set`'s `hasItem` is the shared `hasIndex` body at arity 1, `Bag`'s is
`BagClass::hasItemRexx` at arity 2. (The first version justified this decision by attributing
`Arity::Fixed(n)` misreadings to Phase 5f; that phase's records do not support it, and the decision
stands on the upstream operand alone.)

**D101 -- the new store is GC-visible from its first commit.** An object-keyed store holds `ObjRef`
in both positions, and a key the collector cannot see is a use-after-free no small test produces.
`collect_stress` is the witness, in the commit that introduces the store.

---

## 5. The gate

Inherited from Phase 5f, plus what the review found missing.

* **The gate is the corpus programs**, under `REXX_CORPUS_GATE=1`. The plain `--test corpus` binary
  is report mode and exits 0 on a divergence.
* **The per-row control is 5f's and it is the only one there is: replace a landed body with a
  93.903 stub and the witness must go red.** The first version said "inherited unchanged" without
  restating it. Restate it: nothing else in this gate checks a row individually.
* **`method-bodies.txt` is a drift check only.** No row may move to `diverge`, none that answered may
  stop. `loud` -> `answers` is reported, never required.
* Every witness sends a short argument list as well as a good one; every new corpus program is filed
  in `corpus/phase-5c.txt`, in `EXPECTED_SUBSET_5C`, and as a `sourceline_oracle/<name>.txt`. **The
  filing rule is load-bearing, not a chore**: `corpus.rs:692` documents that a subset losing a whole
  file exits 0 in both modes, so `EXPECTED_SUBSET_5C` is the only thing that notices.
* **The instrument that replaces the verdict column, and the rule that makes it mean anything.** A
  re-probe of every row in scope with a real argument list, committed as a table. **A row's argument
  list is real only if the oracle answers rc 0 to it.** An empty list, or one the oracle rejects, is
  a harness failure for that row and not a data point -- without that rule a harness can send real
  arguments to the two control rows and nothing else and read green, because every other row agrees
  at zero arguments exactly as it does today.

---

## 6. What neither phase does

`RexxQueue` (D93). The `Collection` mixin bodies, which already run. `ARG(1,'A')` (D100). `Stream`
and the native API. Performance: where the oracle's own iteration order does not reproduce, a
witness may not depend on it -- that is a witness that cannot be written, not a divergence to chase.

## 7. Risks

* **The instrument is load-bearing and it is a probe of my own.** This spec's first version was
  wrong in six places, and the two that survived review longest were both mechanism claims built on
  correct probes. Outcome rulings have held here; mechanism rulings have not.
* **The red control's own tree matters.** D92's commit makes `Bag~put` *loud*, and in a harness
  shaped like `method_bodies.rs` a loud row never reaches the oracle -- so "these two rows must
  disagree" becomes unfalsifiable rather than red. Decide whether D92 or the instrument lands first,
  and write the prediction for the tree the control will actually run on.
* **`Stem`** may not fit the protocol. If it does not, it gets its own task rather than a widened
  protocol.
* **D90's cost is not scoped here.** Which `Body` a subclass instance carries, and whether
  `Primitive` resolution moves from identity to descent, is 5g's to answer; both conventions already
  coexist deliberately and `dispatch.rs` says why.
