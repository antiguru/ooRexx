# Phase 5h Task 5 — `Stem`

BASE Task 4. `Stem` shares no entry point with the rest of the phase, which
is spec section 7's reason for giving it a task of its own rather than
widening the protocol: its store is the language's tails, and `Body::Stem`
already tells a dropped tail from an absent one.

## What landed

`at`, `[]`, `put`, `[]=`, `items`, `isEmpty`, `hasIndex`, `hasItem`, `index`,
`remove`, `removeItem`, `empty`.

**A tail read has three answers, not two.** A tail that holds something
answers it; a DROPPED tail answers its own derived name, because the
tombstone is still there; a tail never assigned answers the stem's default,
or its derived name when there is none. Measured on `s. = 'dflt'` with `s.b`
dropped: `at('B')` is `S.B` and `at('ZZ')` is `dflt`. `items`, `hasIndex` and
`index` count and find only the first kind.

**`empty` deletes rather than drops**, which is the one place the tombstone
must not be written: after it a tail reads as the default, not as its derived
name. Measured, `u. = 5` with one tail set answers `items` 0 after `empty`
and `u~at('K')` still answers `5`; writing tombstones there answered `U.K`.
`empty` answers the receiver.

**`Stem`'s argument rules are the loosest in the phase.** A missing subscript
names the stem itself rather than being an error: `hasIndex()` is `1`,
`at()` is `S.`, `remove()` answers `S.` and takes nothing out. Only
`removeItem()` insists. The first version raised where the oracle answered
and the method-body table caught five rows -- the third time in this phase
that the argument layer was wrong while the semantics were right.

## What did not land, and why

**`allIndexes`, `allItems`, `makeArray` and `supplier` are still loud.** They
answer tails in the stem's own table order, and that order is a different
geometry from the one the mapped classes use. Measured, five string tails
inserted `zebra apple mango q longkeyname` come back `Q,MANGO,LONGKEYNAME,
ZEBRA,APPLE`, and twelve numeric tails come back `1,3,2,5,7,6,4,9,12,11,10,8`.
A search over the Task 1 geometry -- every bucket count from 2 to 200, with
and without front insertion, and again with the growth model -- reproduces
the first line at bucket size 39 and **the second at none of them**. So it is
not that table with a different size; it is another structure.

Landing those four with the wrong order would ship a divergence on the most
commonly used rows of the class. They stay loud until the stem's own
contents are ported, and that is a task rather than a detail.

`request`, `toDirectory` and `unknown` are also still loud: spec section 7
says none of them is a collection primitive and none should be forced through
the protocol, so they are left where the spec puts them.

## Two limbs nothing can witness

`hasItem()` and `index()` with no argument. Both are a SIGSEGV on the oracle
whenever the stem holds a tail -- they are already in
`corpus/oracle-crashes.txt` -- so no differential can be run for them at all.

**The probe that measured them got its answer by accident.** It ran `empty`
as its first case, so by the time it reached `hasItem()` the stem was empty,
which is the one shape the oracle survives. The answers written here (`0` and
`.nil`) come from that shape, the code comment says so, and
`stem_collection.rex` leaves both out rather than asserting an answer it
cannot check.

## What moved

`corpus/collection-arity.tsv`: `Stem` 13 `agree`, and -- with this task --
**`setup-differs` is 0 across the whole table**. Every one of the 433 rows now
has a receiver the instrument can build. The table's `agree` is 401 and the
31 `send-differs` rows are all named: the four order-dependent `Stem` rows and
what is built on them, `request`, `toDirectory`, `unknown`, `subset`'s `DO ...
OVER`, `setMethod`/`unsetMethod`'s `Method~new`, `Properties~save`'s LIBRARY
entry point, `setLogical`'s `ARG` option, and `Stem~union`/`~xor`, which need
`Array~copy`.

`a_stem_receiver_answers_stem_and_renders_its_own_value` pinned three of the
refusals this task removed and now asserts the oracle's answers instead.

## Gates

Witness: `corpus/lang/stem_collection.rex`, filed in all three places,
byte-identical to the oracle on both engines at rc 0. Strict corpus 436 of
436.

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `REXX_CORPUS_GATE=1 cargo test --release
--workspace --no-fail-fast` exit 0, 2124 passed. `refusal-sites.tsv`
re-derived with no drift; `method-bodies.txt` and `collection-arity.tsv`
refreshed.
