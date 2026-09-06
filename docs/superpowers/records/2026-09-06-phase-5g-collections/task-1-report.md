# Phase 5g Task 1 — the contents protocol, `Supplier`, and `Array`'s shared surface

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `aa74e48f2`.

Sixteen rows bound: `Array`'s `allIndexes allItems empty hasIndex hasItem index
isEmpty makeArray remove removeItem supplier`, and `Supplier`'s
`available index item next init`.

---

## The protocol

`ordered_pairs` is the whole of it: every (index, item) pair a store holds, in
its own order, holes skipped. Everything else on the shared surface is one line
over it.

**The index is an `ObjRef` and not a `usize`, and that is measured rather than
tidy.** A multi-dimensional array's index is an `Array` of coordinates:
`.Array~new(2,3)` with `[1,1]` and `[2,3]` set answers `allIndexes` as two
`Array`s reading `1,1` and `2,3`, and `index('b')` as `2,3`. Nothing about a
one-dimensional array shows it, and a protocol carrying a number would have
been wrong in a way every single-dimension test passes.

**`makeArray` is written here as `allItems` and is not shared with Phase 5h.**
`RexxObject::makeArrayRexx` is `return makeArray();`, a virtual:
`ArrayClass::makeArray` answers items, `HashCollection::makeArray` answers
*indexes*, `StemClass::makeArray` answers the tail array. Measured, a
`Directory` holding `k1`/`k2` answers `makeArray` as `k1,k2`.

## What the measurements changed

**Item equality is `==` on string values, and none of the three obvious
readings is right.** Measured on the oracle: `'1'` matches `1`; `' 2'` does
**not** match `2`; `1` does not match `1.0`; and a class defining
`::METHOD "==" return 1` makes `hasItem` find an object that was never put in.
So it is neither byte equality of the spelling nor numeric equality, and it
**sends** rather than comparing handles.

**`Array~empty` answers the receiver.** `ArrayClass::empty` ends `return this;`.
A body answering nothing passes any witness that calls `~empty` as a statement
-- mine did -- and reddens the moment something reads the value. Caught by
`method_bodies.rs` refusing to write the refresh: `Array empty (instance arm):
loud -> diverge`. `say a~empty` is a blank line at rc 0 on the oracle against
`91.999 Message "EMPTY" did not return a result.` at rc 165 without the fix.

**`empty` keeps the array's shape.** `.Array~of('x','y')~empty` leaves `size 2`,
`items 0`, `isEmpty 1`; a multi-dimensional array keeps its dimensions. So
`isEmpty` is not `size == 0`.

**`Supplier~new` takes the items first.** Measured:
`.Supplier~new(.Array~of('i1'), .Array~of('x1'))` answers `~item` as `i1` and
`~index` as `x1`. Past the end `~item`, `~index` **and** `~next` all raise
93.937, so stepping past the end is an error rather than a no-op. Mismatched
array lengths are accepted at construction and the shorter one bounds the walk.

**The state lives in the receiver's own pool**, under the `Supplier` class as
scope, rather than in a new `Body` variant: `ScopePools` is already walked by
the collector (`Body::trace`'s `Instance` arm), and a supplier holds two
arrays, so a payload the collector could not see is the use-after-free spec
D101 exists to prevent.

**The old `Supplier~init` was a shell** -- it validated both arrays and dropped
them, which `rust/CLAUDE.md` names as the thing not to build. Removed, not
extended.

## The refusals, which are three different errors

Measured, and no zero-argument probe can tell them apart because both families
are loud until the bodies exist:

| call | error |
|---|---|
| `hasItem`, `index`, `removeItem` with none | 93.903 `Missing argument in method; argument 1 is required.` |
| `remove`, `hasIndex`, `at` with none | 93.901 `Not enough arguments for method; 1 expected.` |
| `allItems`, `allIndexes`, `isEmpty`, `supplier` with one | 93.902 `Too many arguments in invocation of method; 0 expected.` |

The witnesses trap each send to show the family, then let the last one escape,
because only stderr carries the decimal: `condition('O')` answers a `Directory`
this crate does not build yet.

## Witnesses

`corpus/lang/array_enumeration.rex`, `array_item_argument.rex`,
`array_index_argument.rex`, `array_extra_argument.rex`,
`supplier_iteration.rex`. All five filed in `corpus/phase-5c.txt`, in
`EXPECTED_SUBSET_5C`, and as `sourceline_oracle/<name>.txt`. Strict corpus:
**410 of 410 matching**, up from 405.

### Red controls, one of them falsified

**M1 -- `native_array_index` always answers `.nil`.** Predicted red on
`array_enumeration.rex`. **Confirmed**: `409 of 410`, that file, stdout stderr
and exit code all differing.

**M2 -- `same_item` compares handles instead of sending `==`.** Predicted red.
**Falsified on the first run**: `410 of 410`, green. The witness could not see
it, because a literal used twice is one object, so handle identity satisfied
every case it contained. That is
`test-can-fail-is-not-test-adds-coverage` exactly: the mutation proved the
witness did not cover the decision the code had just made.

Fixed by adding the cases that discriminate -- `'1'` against `1`, `' 2'`
against `2`, `1` against `1.0`, and the `==` override -- and **M2 re-run with a
rebuild is red**, `409 of 410`, stdout differing.

**A third error, mine, worth recording separately.** Between the two M2 runs I
restored the source and compared without rebuilding, and read the stale
binary's answer as a defect in the override handling. `stale-binary-outlives-its-revert`,
and the tell was that the same program answered correctly when run again after
a build.

## A use-after-free, caught by the stress harness and nothing else

`collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation`
failed on `array_enumeration.rex`: plain exit 0, stress exit 120 with
`a message send to a value whose object is no longer live`, stopping exactly at
the multi-dimensional half.

`ordered_pairs` allocated each index object into a plain `Vec` and then
allocated the next one. For a one-dimensional array the index is a small
integer and nothing is allocated, so every test passed; for a multi-dimensional
array it is a fresh `Array`, and the second allocation collected the first.
Fixed by rooting each index as it is built, and `array_of` roots its result for
the same reason.

**This is spec D101's whole point and it was still nearly shipped.** The
ordinary release run, the corpus differential and both engines all agreed
before the fix. Only collect-on-every-allocation disagreed.

## Four bookkeeping failures, all of them the plan's own list

The full `cargo test --release --workspace --no-fail-fast` found four, none
visible to the fast subset a task is tempted to run instead:

* `collect_stress` -- the use-after-free above.
* `dispatch_seam`'s `the_seam_token_is_named_only_by_the_dispatch_module` --
  `dispatch/collection.rs` is a new clearance consumer and had to be added to
  `CLEARANCE_CONSUMERS`.
* `refusal_sites`'s `the_table_holds_every_constructor_the_source_defines` --
  the new `no_more_supplier_items` constructor shifted every `error.rs` line
  below it by eleven and needed a row of its own.
* `dispatch::tests::a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state`
  -- `Supplier` moved out of the refusal group, which is that test working:
  its refusal rows exist to catch a constructor that fabricates a body, and
  `Supplier` now keeps what it is given.

## What moved

`corpus/method-bodies.txt`, refreshed: **44 rows**, of which 23 `loud` ->
`answers` -- `Array` 11, `Supplier` 6, `Queue` 3, `CircularQueue` 3 -- and 21
that stay `loud` with different evidence, `Queue` 9 and `CircularQueue` 12.
Those twenty-one now reach a body and refuse with `a message send to a value
that is not an array` where they used to say the method was not implemented:
Task 4's remaining work, made visible.

`corpus/collection-arity.tsv`, refreshed: `agree` 21 -> 38, `send-differs`
90 -> 73. **Seventeen rows now agree under a real argument list**, against
`method-bodies.txt`'s 23. The gap between those two numbers is the whole reason
Task 0 built the instrument: a zero-argument send agrees about an arity error.

`corpus/collection-scopes.tsv` is unchanged, as it must be -- nothing upstream
moved.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing.

Seven gates over the committed tree at `a478d1627`, from
`scratchpad/gates-5g1.status`: G1 0, G2 0, G3 0, G4 0, G5 0, G6 0, G7 0, with
`failed-suites=0` on each of G3, G4, G5, G6 and G7.
