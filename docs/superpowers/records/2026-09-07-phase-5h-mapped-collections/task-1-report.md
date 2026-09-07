# Phase 5h Task 1 — a finding that changes what the store has to be

The plan calls the store "the phase's only new data structure" and says what it
must hold (`ObjRef` in both positions) and that the collector must see it. It
does not say the store's GEOMETRY is observable. It is.

## Measured

Iteration order is bucket order, not insertion order, and it reproduces across
runs. `.Table~new` with the keys `'zebra' 'apple' 'Mango' 'q'
'longer-key-name' '' '  ' 'Z'` inserted in that order answers `allIndexes`:

    |  |longer-key-name|Z|zebra|Mango|apple|q

and inserting the same keys in reverse gives a different order again:

    |  |Z|longer-key-name|zebra|Mango|apple|q

so it is neither insertion order nor its reverse nor sorted. Forty integer keys
`1..40` come back `40,20,21,...,36,10,1,37,11,2,38,3,12,39,4,...`. `Directory`
and `StringTable` answer the same order as `Table` for the string keys; two
runs of each, identical.

A store that keeps insertion order therefore diverges on `allIndexes`,
`allItems`, `makeArray`, `supplier` and `do over` for every class in this
phase, on any receiver holding more than a couple of entries -- which is most
of them, and which no small witness would flatter.

## What matching it costs, and why it is the cheaper end

The geometry is small and completely specified in the C++:

* the string hash is Java's, `h = 31 * h + byte` over the bytes, cached, with
  `HashCode` a `size_t` (`classes/StringClass.hpp:328`,
  `classes/ObjectClass.hpp:71`);
* `hashIndex` is `getHashValue() % bucketSize` for the identity contents and
  `hash() % bucketSize` for the equality one
  (`classes/support/HashContents.hpp:204`, `:443`);
* `calculateBucketSize` is `max(capacity, 17)` forced odd, capped at `1 << 30`
  (`classes/support/HashCollection.cpp:150`, `HashCollection.hpp:126`);
* the table is `bucketSize` primary slots plus an equal overflow area --
  `allocateContents(bucketSize, bucketSize * 2)` (`HashCollection.cpp:84`) --
  with a free chain over the overflow;
* growth doubles the TOTAL size and recalculates:
  `expandContents(contents->capacity() * 2)` where `capacity()` is `totalSize`
  (`HashCollection.cpp:96`, `HashContents.hpp:316`), then `reMerge` refills;
* `TableIterator` walks the primary buckets in order, following each `next`
  chain into the overflow (`HashContents.hpp:104`-`:124`).

So the port is the hash, four sizing rules, the free chain, `reMerge`, and the
iterator. Against it: a store that does not reproduce them makes every
iteration row in the phase a divergence to be argued about one at a time, on
nine classes.

The standing rule is byte-for-byte agreement with the oracle everywhere except
`MAX_EVAL_DEPTH`, and iteration order here is reproducible rather than
timing-dependent -- unlike GC ordering, which is licensed precisely because it
is not a specified observable. **So Task 1 ports the geometry.** That is inside
the plan's boundary -- Task 1 already owns the store -- and larger than the
plan's sentence about it implies. Recorded here rather than discovered halfway
through Task 4.

## The one part that is not reproducible, and does not need to be

Identity-keyed iteration over objects whose hash is their address does not
reproduce across runs even on the oracle (recorded in memory as
`oorexx-hash-iteration-order`). A witness must therefore key an `IdentityTable`
by strings, whose `getHashValue()` is the string hash -- which is also what
`hashIndex`'s own comment says the reason for is.

## The model is validated before any Rust is written

A Python simulation of the geometry above -- Java string hash, `bucketSize`
primary slots, an overflow free chain running upward from `bucketSize`,
`append` linking at the END of a bucket's chain, expansion when the free chain
empties, `reMerge` re-adding in old bucket order, iteration walking buckets
0..bucketSize-1 and following each chain -- predicts the oracle's answer
exactly on three independent cases:

| case | keys | result |
|---|---|---|
| eight irregular strings | `zebra apple Mango q longer-key-name '' '  ' Z` | predicted order identical |
| the same eight, inserted in reverse | -- | predicted order identical, and it differs from the first |
| forty integer keys, which grows the table twice | `1..40` | all forty in order, identical |

The third is the one that matters, because it exercises the parts a
static-table simulation cannot reach: `isFull` is `freeChain == NoMore` and not
an item count (`HashContents.hpp:297`), which is why a table can expand while
`itemCount < totalSize`; the first attempt used the item count and ran the free
chain dry. Integers hash by their string value
(`classes/IntegerClass.cpp:83`), which is what lets the same simulation cover
them.

So Task 1 is a port of a specification that is already checked against the
thing it is meant to reproduce, rather than a design to be discovered by
divergence.

---

# What Task 1 landed

`crates/rexx-exec/src/dispatch/hash.rs`: the store, and `Table` and
`IdentityTable`'s sixteen instance rows plus their `NEW`.

## The store

Five pool entries under the `Table` class as scope -- three parallel `Array`
objects (`INDEXES`, `ITEMS`, `NEXT`) and two counted scalars (`BUCKETS`,
`FREE`). `Array` objects in a `ScopePools`, which the collector already walks,
so **spec D101's GC visibility is met by construction rather than by new
collector code**, the same trick Phase 5g used for `Queue`'s store.

The item count is derived from occupancy rather than maintained, so there is
no counter that can disagree with the slots.

`MapCollection` would have been the natural scope and the natural
"may have a store" test, and it is neither: the class registry does not answer
`lookup` for it -- it is donated by `CoreClasses.orx` -- so reaching for it
panicked on the first `t['k'] = 'v'`. The scope is only a namespace key, so a
native class serves.

## One method identity, seven classes

`Setup.cpp` donates `IdentityTable`'s `At`, `Put` and `[]` rows to
`StringTable` and that whole set on to `Directory` (`memory/Setup.cpp:881`,
`:933`), and `Set`, `Bag` and `Relation` take theirs the same way. This crate
models that faithfully: **`Table~AT` and `Directory~AT` are one method
identity and one native entry**, so registering a body for `Table` registers
it for all seven.

Found by running, not by reading: the first `t['k'] = 'v'` panicked inside a
`StringTable` during the bootstrap. Then, with the bodies registered but
unguarded, the method-body table reported **38 rows regressing from `answers`
to `diverge`** across `Bag`, `Directory`, `Relation` and `Set`.

So every body guards on the two classes this task owns and hands every other
receiver back the refusal it had before this file existed. `Set`'s index-only
rule, `Bag` and `Relation`'s multi-value `put` and the string classes' entry
map are their own tasks, and none of them is silently given a `Table`.

## The argument errors are two, and neither is the obvious one

Measured on `.Table~new`: `at()`, `put()`, `put('v')`, `hasIndex()` and
`remove()` are **88.901** -- `Missing argument; argument index is required.`,
the named form -- while `hasItem()`, `index()` and `removeItem()`, which take
an ITEM, are the positional **93.903**. The first version used 93.903
throughout and the method-body table caught it.

## What moved

`corpus/collection-arity.tsv`, re-derived: `Table` and `IdentityTable` each
go from 24 `setup-differs` to **23 `agree` and 1 `send-differs`**. The whole
table's `setup-differs` falls 193 to 145 and `agree` rises 183 to 229.

The one row left per class is `subset`, and it is not this task's: it is a
Rexx-level `Collection` method that runs `DO ... OVER` on the receiver, and
`DO OVER` over an instance of a user class is unimplemented. Naming it here
rather than counting it as a store gap.

`corpus/method-bodies.txt`, re-derived: 24 rows loud to `answers`, twelve per
class. The `Properties` rows change the class named in their refusal text
from `Directory` to `Properties` -- this crate's own loud text, never
compared against the oracle, and the verdict on every one of them is `loud`
before and after.

## What this task does not witness

Spec D96's `HASHCODE` branch. The witness it prescribes needs
`.MyStr~new('gamma')` on a `String` subclass, and `String` subclass `NEW` is
unimplemented here -- measured, `method "NEW" of class "MYSTR" is not
implemented (Phase 5)`. The branch is written the way D96 sets it out, base
class in and everything else through the send, and **nothing in this tree
exercises the second limb**. Said plainly rather than counted as covered.

Identity for two equal SHORT strings also cannot come apart here: a short
string is a value in the handle, so `it['alpha']` and `it['alph' || 'a']` are
the same object where the oracle has two. The witness uses a key long enough
to be heap-allocated, and with that key `IdentityTable` and `Table` answer
differently exactly as the oracle does. The short-string case is the value
model's own deviation, not the store's.

## Gates

Witness: `corpus/lang/hash_table_store.rex`, filed in all three places,
byte-identical to the oracle on both engines at rc 168. Strict corpus 432 of
432.

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `REXX_CORPUS_GATE=1 cargo test --release
--workspace --no-fail-fast` exit 0, 2124 passed. `refusal-sites.tsv`
re-derived with no drift.
