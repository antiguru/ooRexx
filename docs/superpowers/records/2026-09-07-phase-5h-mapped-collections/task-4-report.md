# Phase 5h Task 4 — the design question, answered before any code

The plan: "the first question is not what to add but whether the existing
store is the one these classes should use, or whether it is the environment's
and the collections get the Task 1 store with a string key protocol on top.
**Answer it in the report before writing the surface** -- a user `Directory`
that silently uppercases its keys is a divergence that most probes will not
show."

## Measured, and the answer is that it is the environment's

A `Directory` does **not** uppercase the keys `[]`, `at` and `put` use:

    d = .Directory~new;  d['lower'] = 1
    d~allIndexes   ->  lower
    d['LOWER']     ->  The NIL object
    d~hasIndex('LOWER') -> 0

`NativeObject`'s map holds its keys already uppercased, by its callers, for
the reason its own doc gives. So it is `.environment` and `.local`'s store and
not a user `Directory`'s, and Task 4 gives these three the Task 1 store.

## What uppercases is the ENTRY family, and only it

    d~setEntry('viaEntry', 9)
    d~allIndexes   ->  lower, VIAENTRY, ...
    d['viaEntry']  ->  The NIL object
    d['VIAENTRY']  ->  9
    d~entry('lower') -> The NIL object      -- entry() uppercases its lookup too

So `entry`, `hasEntry`, `setEntry` and `removeEntry` are the uppercasing
accessors over the same store the index family reads without uppercasing, and
`setEntry` with no value REMOVES the entry -- measured, `d~setEntry('beta')`
after `d~setEntry('beta', 2)` leaves `items` at 0.

## The key protocol is the string VALUE, and it does not refuse a non-string

    d[1] = 'one';  d['1']  ->  one,  allIndexes -> 1
    d[.Array~new] = 'x'    ->  accepted, items 2

`StringHashContents` compares with `memCompare` and hashes the string hash, so
a number and its digits are one key. An object index is not refused; it is
taken by its string value.

## Consequence for the shared bodies

`.environment` and `.local` stay `Body::Native`, so the split in
`native_hash_at` and `native_hash_put` has to be on the receiver's BODY and
not on its class -- a `Body::Native` receiver reads the entry map, everything
else reads the store. That is the shape Task 1 wrote and Task 1 narrowed to
`owns` when the donated method identities turned out to reach seven classes;
Task 4 widens it again, now that `Directory` is a class this file owns.

## `Properties`

`p['a'] = 'b'` and `getProperty`/`setProperty` all answer on the oracle, and
its Rexx-level methods are already installed here. The plan's note stands:
what blocked it was `native_directory_new` giving a subclass a plain instance
so `expose` works, and Task 1's store removed that conflict -- a plain
instance is now also a collection.

---

# What Task 4 landed

`Directory`, `StringTable` and `Properties` join the classes this file owns,
`Directory~new` and `StringTable~new` build a store-backed instance where they
built a `Body::Native`, and `StringHashContents`'s key protocol -- the index's
string value, hashed and compared as bytes -- joins the two Task 1 wrote.

`StringHashCollection`'s four accessors are new: `entry`, `hasEntry`,
`setEntry` and `removeEntry`, each upper-casing the name on the way in and
reading the same store the index family reads without upper-casing.
`setEntry` with no value removes.

`UNKNOWN` -- the entry-method mechanism, where an entry answers a message of
its own name -- reads the store for a collection and the map for the
environment, the same split the index family takes.

## Two things this broke, both found by running

**The internal method-table walkers read only the map.** `Class~enhanced`,
`~defineMethods` and `~setMethod` are handed a `.StringTable~new` a program
filled, walk it with `native_keys`/`native_entry`, and refuse with a
`SUPPLIER` failure when the walk comes back empty. Three corpus programs went
red -- `enhanced_scope.rex`, `enhanced_unset.rex` and `usesem.rex` -- because
the table they hand over stopped being a `Body::Native`. Both walkers now read
either store, and the guard in front of them asks whether the value is a
string-keyed table rather than whether its `receiver_kind` is one of two
native kinds.

**`a_directory_subclass_keeps_the_instance` recorded a split that is now
closed.** It asserted a `Directory` subclass's entry write REFUSING, and its
own doc said the oracle answers `1` and that the two rows could not both be
green: `native_directory_new` gave `.Directory` a `Body::Native`, which has no
variable pool, and a subclass a plain instance, which had no store. The store
lives in the pool, so an instance is now both. The test asserts the oracle's
answer on both halves.

## What moved

`corpus/collection-arity.tsv`: `Directory` 28 `agree`, `StringTable` 28,
`Properties` 33, and no `setup-differs` left in any of the three. The table's
`agree` rises 309 to 388 and `setup-differs` falls 65 to 27 -- and every one
of the 27 is now `Stem`, which is Task 5.

The nine `send-differs` rows left across the three are named and none is a
store gap:

| rows | what blocks them |
|---|---|
| `setMethod`, `unsetMethod` on `Directory` and `Properties` | `Method~new` is unimplemented, and the plan says `setMethod` is not a store operation |
| `subset` on all three | `DO ... OVER` over an instance of a user class, the same row `Table` and `IdentityTable` keep |
| `Properties~save` | a `LIBRARY REXX` entry point |
| `Properties~setLogical` | `ARG` option `A` |

**`Properties` needed no body of its own**, as the plan predicted: every one
of its rows resolves to `Directory`'s, and its Rexx-level methods were already
installed and answering. Run rather than assumed -- `p['a'] = 'b'`,
`getProperty`, `setProperty` and `allIndexes` all agree.

## Gates

Witness: `corpus/lang/directory_string_keys.rex`, filed in all three places,
byte-identical to the oracle on both engines at rc 0. Strict corpus 435 of
435.

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `REXX_CORPUS_GATE=1 cargo test --release
--workspace --no-fail-fast` exit 0, 2124 passed. `refusal-sites.tsv`
re-derived with no drift; `method-bodies.txt` and `collection-arity.tsv`
refreshed.
