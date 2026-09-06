# Phase 5g Task 6 — one instance carrying a store and a variable pool

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `34ea2e66e`.

---

## Most of this task was already done by Task 4

The plan named `CircularQueue` as this task's subject. It has not been this
task's subject since Task 4: giving `Queue` a store gave `CircularQueue` one,
its 28 rows moved there, and `.CircularQueue~new(3)` already matched the oracle
byte for byte including the circular overflow.

**That is the plan's framing being wrong in a useful direction.** Spec D90 said
the subject was "one instance carrying a native store *and* an object variable
pool at once", and the arrangement Task 4 chose for its own reasons — an
ordinary `Body::Instance` whose pool holds an `Array` — already has both
halves. There was never a mechanism to build, only a constructor to widen.

## What was left

`~new` and `~of` on a subclass of `Array`, which refused in the crate's own
words. Measured, oracle rc 0: `.array~subclass('K')~new(2,3)~size` is `6`,
`~of(1,2)~class~id` is `K`, and a section of a subclass is that subclass.

`native_array_new` and `native_array_of` now go through `array_of_class`: a
bare `Body::Array` for `.Array` itself, and an instance carrying it as a store
for anything else. `Loud::array_subclass_new` is gone, and the two unit tests
that asserted the refusal now assert the answer.

## The resolution point moved, and that is the real change

Task 4 put `store_of` in `dispatch/collection.rs`, which reached only the
bodies written there. An `Array` **subclass** inherits `Array`'s whole
registration, including the bodies in `dispatch.rs` — `at`, `put`, `items`,
`size`, `dimension` — so those refused on a subclass exactly as they had
refused on a `Queue`.

The fix is one function in `dispatch.rs`, `collection_store`, called by
`array_slots`, `array_dimensions`, `array_is_fixed_dimension`, `array_resize`
and `native_array_put`. **One resolution point for every Array-shaped
receiver**, which is what lets one body serve `.Array`, a `Queue`, a
`CircularQueue` and a user subclass of any of them. It answers the receiver
unchanged when there is no store, so every refusal a caller already raised
stays the one it raises.

The store's scope is now the `Array` class for every holder, where Task 4 used
`Queue`'s — a `Queue` and a user subclass of `Array` are the same shape and
should not be found by different lookups.

## Witness and red control

`corpus/lang/collection_subclasses.rex`, filed in all three places. It is
built around the property rather than the mechanism: a subclass instance must
answer its own `expose`d state **and** the inherited collection surface, and a
design giving it only one of the two passes half the program. `CircularQueue`
is in it as the library's own witness that both halves are needed on one
object — its `init` exposes `size` while its `queue` reaches `Queue`'s store.

Strict corpus: **417 of 417 matching**.

**M11 — `array_of_class` ignores the class and always answers the bare
store.** Predicted red on all three descriptors: the class line prints `Array`
instead of `MYARR`, and the `expose`d state is gone so `~tagged` raises.
**Confirmed**: 416 of 417, `stdout, stderr, exit code differ`.

## What moved, and what did not

**Neither derived table moved a single row**, and that is correct rather than
disappointing: `method-bodies.txt` and `collection-arity.tsv` both construct
`.Array~new`, never a subclass, so no row of either exercises the path this
task fixed. **The witness is this task's only evidence**, which is why the
mutation matters more here than elsewhere.

`corpus/refusal-sites.tsv` lost `array_subclass_new` and had 31 line numbers
re-derived. The re-derivation is now a script rather than arithmetic on a
diff: `cargo fmt` renumbered the file between the first attempt and the commit
and turned a correct shift into an off-by-one, which is
`generated-files-change-between-review-and-commit` in miniature.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
