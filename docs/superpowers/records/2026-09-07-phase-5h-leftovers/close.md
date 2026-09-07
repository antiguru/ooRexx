# Phase 5h's leftovers

The list Phase 5h's close left "still open, with an owner". BASE `405a033fe`.

## Done

**`Array~copy` and `Stem~copy`** (`d8dd6f9e5`). `native_copy` answered only
for a plain instance. The close said this was what two `Stem` rows waited on;
re-derived, it moved six `Array` rows instead -- `difference`, `disjoint`,
`intersection`, `subset`, `union`, `xor`, all of which begin `self~copy` --
and the `Stem` pair then stopped at `Stem~supplier`, so `copy` was their
prerequisite rather than their blocker.

**The stem's tail tree** (`8c054ea0f`, lint fixed in `f4582418f`). Not a hash
table: a balanced binary tree keyed on (length, bytes), rebalanced with a
single rotation and a depth counter per side, walked in post-order. Ported and
validated by simulation against four oracle cases before any Rust was written.
Eleven `Stem` rows moved to `agree`; four method-body rows moved `loud` to
`answers`.

## Not done, and why

**`Directory~setMethod` / `~unsetMethod`.** It is a task, not a leftover --
which is what the 5h plan said would happen if it did not fit. Its
specification is now complete and measured, so the next session starts from a
design rather than a question:

* A method entry behaves as an ordinary entry whose value is the method's
  RESULT, computed on every read. Measured: after `d['plain'] = 1` and
  `d~setMethod('GREET', 'return "hi"')`, `d~items` is 2, `allItems` is
  `1,hi`, `d['GREET']` is `hi`, and `d~greet` is `hi`.
* The name is upper-cased: `setMethod('lower', ...)` shows as `LOWER`.
* It lives in a SECOND table, not the contents (`DirectoryClass.cpp:496`), and
  the two are merged on read: contents first in their own order, then the
  method table in its own. Measured, inserting `AAA`(method), `zzz`, `MMM`
  (method), `bbb` answers `zzz,bbb,MMM,AAA`.
* A name in both is not doubled and the method wins: `g['AAA'] = 'value'` then
  `setMethod('AAA', ...)` answers `allIndexes` `AAA`, `g['AAA']` `1`, `items`
  `1`.
* `unsetMethod` removes it; the name then reads as `.nil`.

The second table can be the Phase 5h Task 1 store -- the merge order above is
that store's own geometry for each half -- so the work is the merge at each
read point and running a method-valued entry, not a new structure. The
`Method`-object form of `setMethod` additionally needs `Method~new`; the
source-text form does not.

**`RexxQueue`** is untouched: spec D93 assigns it to Phase 7, and pulling it
forward is a decision rather than a leftover.

## What went wrong here, and it was mine

The clippy check before the stem commit ran as `cargo clippy ... | tail -3`
inside an `&&` chain. The chain's status was `tail`'s and the three lines
shown came from a crate that had finished, so a real
`unnecessary_to_owned` error read as a pass. G2 of the gate run caught it at
101 with every other gate zero.

Every check in this session's last stretch now reads its own exit status into
a variable instead of being piped, which is the only form that cannot lie.

## Gates

Over `8c054ea0f`: G1 0, **G2 101**, G3 0, G4 0, G5 0, G6 0, G7 0.
Over `f4582418f`, after the lint fix: all seven zero, `failed-suites=0` on
each of the five suite-running gates (`scratchpad/gates-lo2.status`).

`corpus/collection-arity.tsv` closes at `agree` 418, `send-differs` 14,
`setup-differs` 0. Strict corpus 437 of 437.
