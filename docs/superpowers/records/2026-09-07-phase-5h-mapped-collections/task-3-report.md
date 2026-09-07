# Phase 5h Task 3 — `Relation` and `Bag`

BASE `2b3f4b3ec` (Task 2). One task for the two, because their native
entry-point sets are the same set: `Setup.cpp` writes `Bag`'s `AllAt`,
`AllIndex`, `Items`, `RemoveAll`, `Supplier` and `UniqueIndexes` as
`RelationClass::` bodies, and only `HasItem` and `RemoveItem` are `BagClass::`
-- and those two answer the same thing here, because a `Bag`'s index is its
item.

## The store change is one function

`MultiValueContents`'s `put` is `addFront`
(`classes/support/HashContents.cpp:1650`): a new entry for an index the table
already holds goes to the FRONT of that index's chain. The bucket slot cannot
move, so the entry that was there is copied into a free slot and chained
behind the new one (`:313`).

That order is observable and the witness asserts it rather than sorting it
away: a relation given `k -> v1` then `k -> v2` answers `allAt('k')` as
`v2,v1` and `at('k')` as `v2`.

**`expand` had to stop using `insert`.** `insert` replaces an index the table
already holds, which is right for a `Table` and would lose one entry per
duplicated index every time a `Relation` grew. `reMerge` adds rather than puts
(`:1238`), so growth now uses a raw append that never looks for a duplicate,
in the old walk order -- which is also what keeps each index's chain in its
order across a growth.

## The rest is surface

`allAt`, `allIndex`, `uniqueIndexes`, `items([index])`, `removeAll`,
`supplier([index])`, `hasItem(item [, index])` and `removeItem(item [,
index])`. `items` and `supplier` take an optional index here where every other
class's take none, so they shadow the shared bodies rather than sharing them.

`removeItem` needed a remove that names an entry by its SLOT rather than by
its index, since two entries under one index differ only by their item; the
existing `take` was split so both use one unlinking body.

## The argument errors split the other way

`RelationClass`'s own methods take their index POSITIONALLY where
`HashCollection`'s take it as a named argument. Measured on a `.Relation~new`:
`allAt()` and `removeAll()` are 93.903, `Missing argument in method; argument
1 is required.`, where `at()` and `remove()` are 88.901, `Missing argument;
argument index is required.`

The first version used the named form for both and the method-body table
caught it -- four rows moving `loud -> diverge`. That is the second time in
this phase that the argument layer was the thing that was wrong while the
semantics were right, and both times the derived table found it rather than a
witness.

## The mixin rows, run as the plan asks

`CoreClasses.orx:80-87` puts `union xor intersection difference subSet putAll`
on `Bag` and `Relation` from `BagMixin` and `ManyItemMixin`. All of them agree
on the first run -- unlike `Set`, whose three failures in Task 2 turned out to
be `copy` sharing its store. That defect is fixed, so this task inherits a
working `copy`, and the witness still runs `union` before the operations that
read the receiver again.

## What moved

`corpus/collection-arity.tsv`: `Relation` and `Bag` each go from 28
`setup-differs` to **28 `agree`** -- every row, as `Set` did. The table's
`agree` rises 253 to 309 and `setup-differs` falls 121 to 65.

Every remaining `setup-differs` row is now `Directory`, `StringTable`,
`Properties` or `Stem`, which are Tasks 4 and 5.

## Gates

Witness: `corpus/lang/relation_multivalue.rex`, filed in all three places,
byte-identical to the oracle on both engines at rc 163. Strict corpus 434 of
434.

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `REXX_CORPUS_GATE=1 cargo test --release
--workspace --no-fail-fast` exit 0, 2124 passed. `refusal-sites.tsv`
re-derived with no drift; `method-bodies.txt` and `collection-arity.tsv`
refreshed.
