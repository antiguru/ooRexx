# Phase 5g Task 4 — `Queue`

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `24dbeeecd`.

Thirteen rows of its own, and the whole of the surface it shares with `Array`.

---

## The decision the plan asked this task to state

**A `Queue` cannot be a `Body::Array`.** A `Body::Array` resolves to
`Primitive::Array` wherever it is asked, so an object carrying one answers
`.Array` for `~class` — there is nowhere in that body to say which class it
belongs to. Upstream has the opposite arrangement: `InheritInstanceMethods`
copies `Array`'s whole native behaviour into `Queue`, so one C++ body serves
both receivers.

**A `Queue` is therefore an ordinary instance whose pool holds an `Array`**,
and every body written against `Array` reaches it through one function,
`store_of`. That is `Supplier`'s arrangement from Task 1 and it is chosen for
the same reason: `ScopePools` is already walked by the collector, so the store
cannot be a payload the collector fails to see.

The alternative — widening `Body::Array` with a class field — costs every
array in the heap eight bytes to serve two classes, against `body.rs`'s own
size assertion. This costs a pool lookup on one class's collection methods.

**The shared registration was already there and only the store was missing.**
Measured before any code: `.Queue~new~allItems` refused with `a message send
to a value that is not an array` where `.List~new~allItems` refused with the
unimplemented message. The first is Array's body reaching a Queue and finding
nothing; the second is no body at all. That measurement is what said `Queue`
needed a store and `List` needs a task.

## What `store_of` did not reach, and how that surfaced

`Queue`'s inherited `at`, `[]`, `items` and `size` live in `dispatch.rs` and
take the receiver's own slots, so they never saw the store. The symptom was a
send that reached the right body, resolved the store, confirmed it live and an
array — and still refused. Four rows of `Queue`'s own fixed it, and
`native_array_at` was split so its body could be pointed at a store.

## The overrides, each of which answers the same thing as `Array`'s

* `queue` adds at the **end**, `push` at the **front**. Measured: after
  `queue('a')`, `queue('b')`, `push('z')` the queue reads `z,a,b`.
* `delete` and `remove` are **one body** upstream (`QueueClass::deleteRexx`)
  where `Array` has two, so both close the gap. `Array~remove` leaves a hole
  and keeps the size; `Queue~remove` does not.
* `put` replaces and does **not** grow: out of range is 93.918 `Incorrect list
  index "99".`, a constructor this crate did not have.
* `peek` and `pull` are the front, and both answer `.nil` on an empty queue
  rather than raising.

**And a section of a `Queue` is a `Queue`.** `sectionRexx` ends
`allocateArrayOfClass` (`classes/ArrayClass.cpp:1524`). The proof is indirect
and is why it was found at all: the witness calls `~makeString` on the result,
which a `Queue` does **not** answer because `RemoveMethod` stripped it. An
`Array` result answers it, so the crate produced *more* output than the oracle
— the only difference in the whole probe.

## `CircularQueue` fell out

`.CircularQueue~new(3)` now works end to end, including the circular overflow:
queueing four items into a three-deep queue answers `b,c,d`, byte-identical.
Its bodies are Rexx in `CoreClasses.orx` and its `init` reaches `Queue`'s, so
giving `Queue` a store gave it one. **28 of its rows moved on this task**, and
that is spec D90's "one instance carrying a native store and an object
variable pool" arriving without a mechanism of its own — the instance is an
ordinary instance and the store is a pool entry, so both were always
compatible.

## Witness and red controls

`corpus/lang/queue_operations.rex`, filed in all three places. Strict corpus:
**415 of 415 matching**.

**M7 — `push` appends instead of prepending.** Predicted red on
`queue_operations.rex`. **Confirmed**: 414 of 415, stdout differing.

**M8 — `section` always answers a raw `Array`.** Predicted red on all three
descriptors, because the program's last line is the `makeString` a `Queue`
refuses. **Confirmed**: 414 of 415, `stdout, stderr, exit code differ`.

## What moved

`corpus/method-bodies.txt`: **59 rows**, all `loud` -> `answers` — `Queue` 29,
`CircularQueue` 28, `Monitor` 2.

`corpus/collection-arity.tsv`: `agree` 55 -> **145**, `setup-differs`
231 from 321, 90 rows moved. That is the largest single movement of the phase
and it is the store rather than the bodies: two classes went from holding
nothing to holding something, and 90 rows became probeable at once.

`corpus/refusal-sites.tsv` gained `incorrect_list_index`.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
