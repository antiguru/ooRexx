# Phase 5h Task 2 — `Set`, and a `copy` that shared its store

BASE `ce0f4c32e`.

## What `Set` needed

Three things, and the first two are small because Task 1's store already
answers everything else:

**The index-only `put`.** `IndexOnlyHashCollection::validateValueIndex`
(`classes/support/HashCollection.cpp:1129`): the value is required, the index
is optional, an index that is given must equal the value, and it then becomes
the value. Measured: `s~put('a')` twice leaves `items` at 1, `s~put('b','b')`
is accepted, `s~put('c','d')` is 93.949. Before this, `s~put('a')` was
88.901 -- spec D92's live divergence, and it is closed here rather than by
the argument-layer-only change D92 describes, because the store it needed
landed in Task 1.

**Two overrides that are method identities of their own.** `Setup.cpp` writes
`Set`'s `HasItem` as `IdentityTable::hasIndexRexx` and its `RemoveItem` as
`IdentityTable::removeRexx`, so they route to the INDEX bodies. Adding `Set`
to the owned set alone left both refusing, because `Set~hasItem` is not
`Table~hasItem`; they are rows of their own.

**`Set~of`.** `SetClass::ofRexx` is native, unlike the mapped classes' `of`,
which is `MapCollection~OF` in Rexx and blocked by spec D100. Measured,
`.Set~of('x','x')~items` is 1, so the arguments go through the index-only
rule rather than being appended.

## What running the mixin rows found

The plan says the `SetMixin` rows "are unmeasurable until `put` works, so the
first thing Tasks 2 and 3 do after their store lands is run them". Run, three
of six disagreed: `intersection` 3 against 2, `xor` 4 against 2, `disjoint` 1
against 0.

The cause is not in the mixin and not in `Set`. `SetMixin~union` is
`new = self~copy` followed by a loop of `put`
(`RexxClasses/CoreClasses.orx:480`), and **`copy` handed back an instance
whose pool entries named the receiver's own store arrays**. A
`Body::Instance` clone copies the pool map, which is right for an ordinary
instance variable and wrong for a collection's contents; upstream draws the
same line by overriding the virtual `copy()`, and `HashCollection::copy`
copies the base object and then its contents
(`classes/support/HashCollection.cpp:237`).

So `union` was mutating its receiver, and every later operation on that
receiver read the mutated set. Only the ORDER of the probe made it visible:
`union` ran before `intersection` in the six-operation probe and after it in
the one-operation probe, and the second answered correctly. That is why the
witness runs `union` first and prints the receiver's contents on both sides
of it.

**The same defect was already shipped for the ordered collections.** Measured:
`q = .Queue~of('a','b')`, `c = q~copy`, `c~queue('z')` left both at 3 items
where the oracle answers 2 and 3, and `List` the same. `native_copy` now
gives the copy an array of its own for each of the nine store entries this
crate uses -- `Array`'s ITEMS, `List`'s three, `Supplier`'s two and the hash
store's three -- with the ITEMS themselves still shared, which is the shallow
element copy upstream makes.

**`Array~copy` is a different gap and is not fixed here.** It refuses loudly:
`native_copy` answers only for `Primitive::Instance` and an `Array` is a
`Body::Array`. That is a missing body rather than an aliasing bug, and giving
`Array` one is not this task's.

## What moved

`corpus/collection-arity.tsv`: `Set` goes from 24 `setup-differs` to **24
`agree`** -- every row. The table's `agree` rises 229 to 253 and
`setup-differs` falls 145 to 121.

`Set` sweeps where `Table` and `IdentityTable` each keep one `send-differs`,
and the difference is real rather than luck: their remaining row is `subset`,
whose `Collection` body runs `DO ... OVER` on the receiver, and `SetMixin`
gives `Set` a `subset` that does not.

One new refusal-site row, `index_does_not_match`, `agrees`, witnessed here.

## A flake, framed

The full suite failed once on
`command_line_arguments_and_the_console_agree_with_the_oracle`, whose
`input-uppercase-split` case had the ORACLE reading a single byte where it
reads a whole line. It did not reproduce: the test passes alone, and the full
suite run immediately after was exit 0 with 2124 passed. Recorded rather than
retried silently, since a console test that loses stdin looks exactly like a
real divergence.

## Gates

Witness: `corpus/lang/set_operations.rex`, filed in all three places,
byte-identical to the oracle on both engines at rc 163. Strict corpus 433 of
433.

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `REXX_CORPUS_GATE=1 cargo test --release
--workspace --no-fail-fast` exit 0, 2124 passed. `refusal-sites.tsv`
re-derived after `cargo fmt`, `method-bodies.txt` and
`collection-arity.tsv` refreshed.
