# Task 4 report: the handle registry

**Status: complete.** `rexx-api`'s `handles::Table` turns an `ObjRef` into the opaque
`RexxObjectPtr` an extension holds and back, `Interp` owns a stack of them and roots it through
`Interp::object_roots`, and the stale-handle property is a test with a confirmed negative control
rather than a claim.

## The API, with exact signatures

`rust/crates/rexx-api/src/handles.rs`:

```rust
pub struct Table { /* HashMap<usize, ObjRef> */ }

impl Default for Table;                                                  // derived
impl Table {
    pub fn new() -> Table;
    pub fn register(&mut self, object: ObjRef) -> RexxObjectPtr;
    pub fn resolve(&self, handle: RexxObjectPtr) -> Option<ObjRef>;
    pub fn remove(&mut self, handle: RexxObjectPtr);
    pub fn clear(&mut self);
    pub fn roots(&self) -> impl Iterator<Item = ObjRef> + '_;
}

fn address_of(object: ObjRef) -> usize;                                  // private
```

`RexxObjectPtr` is `*mut RexxObjectPtr_` from `layout.rs`, which is what the frozen header
declares (`api/rexx.h:134`, `_RexxObjectPtr` at `:117`). No `unsafe` is needed anywhere in the
file: the pointer is minted with `std::ptr::without_provenance_mut` and read back with
`<*mut T>::addr`, both safe, and nothing ever dereferences it. The `unsafe_code` grant set is
unchanged, and `cargo test -p rexx-core --test unsafe_sites` is green below.

### How a handle is encoded, and why

A handle's address is the `ObjRef`'s own bits plus one, and the table maps that address back to
the `ObjRef` it was minted from. Three properties fall out and each is asserted:

* **The generation rides in the handle.** `ObjRef`'s layout is generation in bits 34 and up, slot
  in bits 2 to 33, tag in the low two (`rexx-core/src/handle.rs`), so a handle minted for slot *n*
  at generation *g* is a different value from one minted for slot *n* at generation *g+1*. This is
  the whole mechanism: `resolve` is a lookup keyed on the full bits, so a handle whose slot has
  been recycled is a *miss*, not a hit on the slot's next occupant.
* **Pointer identity is object identity.** The encoding is a function of the object, so the same
  object registered twice answers the same handle and `==` between two handles means what it means
  in the C++, where a handle is a real object pointer. `registering_the_same_object_twice_answers_
  the_same_handle` pins it.
* **No live object encodes to `NULLOBJECT`.** `NULLOBJECT` is the null pointer
  (`api/rexx.h:157`), and `ObjRef::heap(0, 0)` -- the first object a fresh heap allocates -- has
  bits zero, so the raw bits could not be used directly. The plus-one is a bijection on `u64` whose
  only preimage of zero is all-bits-set, which no constructible `ObjRef` has: the tag reserves the
  low two bits, and `TAG_TEXT`, the only tag that sets both, leaves the three bits above the length
  clear. `address_of` asserts it rather than relying on that argument, and
  `the_extremes_of_the_handle_space_are_not_null` exercises the corners.

A compile-time `assert!(usize::BITS == u64::BITS)` guards the width, so a 32-bit target is a build
error rather than a runtime panic at the FFI boundary.

## Forcing the slot reuse deterministically

**Deterministic, not hoped for.** `Heap::collect` threads each swept slot onto the free list head
and `Heap::alloc_with_uncollected` takes the head first (`rexx-core/src/heap.rs`), so when a
collection frees exactly one slot the very next allocation is that slot at generation `g+1`. The
test allocates one object into a fresh `Heap`, clears the table, collects with an empty `RootSet`,
and asserts `stats.swept == 1` before allocating again.

The test then asserts **both halves** of the reuse, because a test that cannot tell "the slot was
reused by a different object" from "the slot was never reused" is not measuring what it claims:

```rust
assert_eq!(heap_parts(second).0, heap_parts(first).0);   // same slot
assert_ne!(heap_parts(second).1, heap_parts(first).1);   // different generation
assert_eq!(array_len(&heap, second), Some(4));           // and a different object
```

The two objects are arrays of different lengths, so "answered the wrong object" is visible in the
heap and not only in the handle.

## The two controls

### Positive control (step 3), run first

`a_live_handle_resolves_to_the_object_it_was_made_from`. A registered handle is non-null, resolves
to the object it was made from, and that object is the three-slot array the test allocated. It then
collects with the table's own `roots()` as the root set and asserts `swept == 0` and that the
handle still resolves; then collects with an empty root set and asserts `swept == 1`. The second
collection is what stops the first from being a claim about an object nothing could have collected.

Without this test, the staleness test below is satisfied by a `Table` that resolves nothing at all.

### Negative control (step 4)

**Prediction, written before any test binary was executed** (the file is
`<scratchpad>/prediction.md`; only `cargo build -p rexx-api -p rexx-exec`, exit 0, had been run at
that point). The mutation: in `address_of`, replace

```rust
let biased = object.bits().wrapping_add(1);
```

with

```rust
let biased = (object.bits() & ((1u64 << 34) - 1)).wrapping_add(1);
```

`GEN_SHIFT` is 34, so this keeps the tag and the slot and drops the generation, which is exactly
the roadmap's "a bare slot index held across a collection".

| # | Test | Predicted | Outcome |
|---|------|-----------|---------|
| 1 | `a_live_handle_resolves_to_the_object_it_was_made_from` | passes | **confirmed** |
| 2 | `a_handle_that_outlived_its_activation_misses_the_slots_next_occupant` | **fails**, at `assert_eq!(later.resolve(stale), None)`, reporting `left: Some(ObjRef(..))` naming `second`; every assertion before it passes; `assert_ne!(stale, live)` never reached | **confirmed** |
| 3 | `remove_drops_one_registration_and_clear_drops_them_all` | passes | **confirmed** |
| 4 | `registering_the_same_object_twice_answers_the_same_handle` | passes | **confirmed** |
| 5 | `the_extremes_of_the_handle_space_are_not_null` | passes -- masking makes `heap(0,0)` and `small_int(SMALL_INT_MIN)` collide at address 1, but the loop resolves each object before the next overwrites it, and a masked address can never be zero | **confirmed** |
| 6 | `rexx-exec` `a_native_activations_local_references_are_roots_only_while_it_lives` | passes -- rooting goes through `Table::roots`, which never touches the encoding | **confirmed** |

Measured, mutation applied:

```
test a_handle_that_outlived_its_activation_misses_the_slots_next_occupant ... FAILED
panicked at crates/rexx-api/tests/handles.rs:112:5:
assertion `left == right` failed
  left: Some(ObjRef(17179869184))
 right: None
test result: FAILED. 4 passed; 1 failed
```

`17179869184` is `0x4_0000_0000`, which decodes as slot 0 at generation 1 -- `second`, the slot's
next occupant. So the mutation does not merely lose an answer, it produces the wrong one, which is
the defect class the whole decision exists to remove. `cargo test -p rexx-exec --lib
a_native_activations_local_references` stayed at exit 0 under the same mutation, confirming row 6.

**Restore.** `handles.rs` was restored by re-editing the one line, never by
`git checkout -- <path>`. Verified against a copy taken before the mutation:
`sha256sum` agrees at `6e428a45a54a0741bfcdfdc8390cb19de8479668a895396aa1b3709a45efd36b` and
`diff` is empty. `git status --short` after the restore lists only the files this task changed.

## Roots (step 5), and the `rexx-exec` edge

Ruling 3's edge is added: `rexx-exec` now depends on `rexx-api`. `cargo tree -p rexx-exec
--depth 1` resolves and `cargo metadata` exits 0, so there is **no cycle** --
`rexx-exec -> rexx-api -> rexx-core`, and `rexx-core` depends on neither.

`Interp` gains

```rust
native_handles: Vec<rexx_api::handles::Table>,
```

initialised empty in `Interp::new`, named in `Interp::object_roots`'s exhaustive destructure, and
handed to the collector with `out.extend(native_handles.iter().flat_map(|table| table.roots()))`.
It is a `Vec` because native activations nest, innermost last; Task 8 owns the push and pop.

`a_native_activations_local_references_are_roots_only_while_it_lives` (in `rexx-exec`'s in-crate
`mod tests`, which is what reaches the private field) asserts both directions: an object held only
by a table on that stack survives `collect_now`, and the same object is swept once the table is
popped. The second half is the anti-vacuity half -- without it the test passes for an object
something else was rooting.

## Where this differs from the C++, and why

`NativeActivation::createLocalReference` / `removeLocalReference` / `clearLocalReferences`
(`interpreter/execution/NativeActivation.cpp:1185`, `:1219`, `:1248`) hold a `firstSavedObject`
fast-path field plus a lazily created identity table, keyed and valued by the object pointer.

* **No fast-path field.** The C++ keeps the first object out of the table so a native method that
  returns one freshly allocated string never allocates a save list. A `HashMap` here does not
  allocate until its first insert, so the same saving is already there and the second code path
  would only be somewhere for the two halves to disagree.
* **A map, not an identity set.** The C++ uses `new_identity_table` explicitly to avoid sending
  `==` to the key. `ObjRef` is a `Copy` bit pattern whose `Eq` *is* identity for a heap object, so
  the identity table's whole reason for existing does not arise. The map is keyed by the handle
  address rather than by the `ObjRef` because `ObjRef` has no public constructor from bits, and
  adding one would widen `rexx-core`'s API to let any caller mint an arbitrary reference.
* **The handle is not the object's address.** That is D5, and it is the point of the task.
* **`clear` does what `clearLocalReferences` does** -- drops both halves at once -- and `remove`
  ignores a handle the table never held, as the C++ does.

## Does the design admit a global reference table?

**Yes, and it needs no new type.** `Table` carries no lifetime and no activation identity; a
global table is a second `Table` owned by `Interp` rather than by an activation, rooted by the same
`object_roots` line, with `RequestGlobalReference` calling `register` and
`ReleaseGlobalReference` calling `remove`. Two consequences worth writing down before Task 8 or
whoever builds it:

* Because a handle is a function of the object, a global reference to an object that also has a
  local one is **the same pointer value**. That matches the C++, where both are the object's
  address, so an extension that requests a global reference for something it already holds gets a
  value it can compare equal -- but it also means `resolve` at the boundary has to consult the
  activation's table *and* the global table, not just one. That is a lookup-order decision, not a
  shape problem.
* A global reference outlives the activation by construction, so the object stays rooted and its
  slot is never recycled while the registration stands. The staleness property is therefore
  unchanged: a *released* global reference goes stale exactly the way a local one does.

I did **not** build it, per dispatch item 5.

## Commands and exit statuses

Each was run unpiped for its status. From `rust/`.

| Command | Exit | Result lines |
|---|---|---|
| `cargo fmt --all` | 0 | -- |
| `cargo fmt --all --check` | 0 | -- |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | no warnings |
| `cargo test -p rexx-api` | 0 | 37 `ok` lines over six binaries (2, 5, 18, 9, 1, 2) |
| `cargo test -p rexx-core --test unsafe_sites` | 0 | 2 `ok` |
| `cargo test -p rexx-exec --lib` | **101** | 799 passed, 3 failed |

**The three `rexx-exec` failures are pre-existing and are not this task's.** They are
`ir::drive::tests::a_call_site_resolves_once_and_answers_from_what_it_kept`,
`ir::drive::tests::a_long_constant_is_built_once_however_many_passes_read_it` and
`ir::drive::tests::the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk`. Measured rather than
assumed: `git worktree add --detach <scratch> HEAD` at `1ca4e3084`, then
`CARGO_TARGET_DIR=<separate> cargo test --manifest-path <scratch>/rust/Cargo.toml -p rexx-exec
--lib ir::drive::tests` fails with exit 101 and the same three names, on a tree that does not
contain this change and in a target directory that has never held a binary of it. The failing set
is identical; the only difference between the two runs is the filtered count, 786 at HEAD against
787 here, which is the one test this task adds. They also fail under `--test-threads=1`, so they
are not a parallelism artifact.

**One caveat on the clippy line.** The run re-checked `rexx-api` and `rexx-exec` only; the rest of
the workspace came from a warm target directory. `rust/CLAUDE.md` records that a warm green is
provisional. The two crates this task edits were both re-linted.

## What the brief and the dispatch got wrong

* **The brief's Interfaces block asks for "a global-reference table with the same shape and a
  different lifetime"; dispatch item 5 says not to build it.** I followed the dispatch. The brief's
  step list never mentions it again, so the Interfaces line is the only place it appears and it
  reads as a leftover.
* **The brief's step 4 as written is vacuous, and the dispatch's warning is what saves it.** "Register
  a handle, end the activation, force a collection, allocate enough to reuse the slot, and assert
  `resolve` answers `None`" passes with the generation removed, because ending the activation
  clears the table and an empty table misses everything. The generation only becomes load-bearing
  when the recycled slot's *new* occupant is registered in a later activation's table, which is the
  step the brief does not name. The test does that, and the negative control above is what proves
  the extra step is what makes it fail.
* **The brief's step 1 says "one per activation" while step 5 says to add the table to
  `Interp::object_roots`.** Those pull in different directions: `Interp::object_roots` destructures
  `Interp`, not `Activation`. Ruling 3 settles it in favour of `Interp`, and the field is a stack so
  that "one per activation" still holds. Worth stating because a reader of the brief alone would
  put the field on `Activation` and reach `Activation::object_roots` instead.
* **`rexx-api` had no `rexx-core` dependency.** The brief's Consumes line says it consumes
  `ObjRef`, and the dispatch names only the `rexx-exec -> rexx-api` edge, so this task adds two
  workspace edges rather than one. Neither is external, so the "one new dependency, `libloading`"
  constraint is untouched.
* **`ObjRef` has no constructor from bits**, only `bits()`. Any design that wants to rebuild an
  `ObjRef` from a handle has to either widen `rexx-core` or keep the mapping, as this one does.
* **`rexx-exec`'s `Cargo.toml` calls `chrono` "the only external crate `src/` depends on"**, with
  `rustix` listed below it under its own reasons. That sentence was already inaccurate before this
  task and is untouched by it -- `rexx-api` is a workspace crate, not an external one -- but it is
  worth someone's correction pass.

## The commit

`6dc03a009fdeeea738b647ce64cfab61e12149ad`, "Turn a stale native-API handle into a lookup miss",
read back with `git log -1`. Seven paths, staged explicitly, no `git add -A`:

```
M  rust/Cargo.lock
M  rust/crates/rexx-api/Cargo.toml
A  rust/crates/rexx-api/src/handles.rs
M  rust/crates/rexx-api/src/lib.rs
A  rust/crates/rexx-api/tests/handles.rs
M  rust/crates/rexx-exec/Cargo.toml
M  rust/crates/rexx-exec/src/lib.rs
```

`rust/Cargo.lock` was reviewed before staging rather than waved through: the whole diff is the two
workspace edges, `rexx-core` under `rexx-api` and `rexx-api` under `rexx-exec`, and no dependency
version moved. `git status --short` is empty afterwards.

**This report is not in the commit.** `.gitignore:30` ignores `.superpowers/` in this repository --
"Subagent-driven-development scratch: ledgers, briefs, review packages" -- and `git ls-files
.superpowers/sdd/2026-09-14-phase-8/` is empty, so tasks 1 to 3's reports are untracked too. The
tracked SDD material lives under `docs/superpowers/`. Adding this file would have needed `-f`, so
it was left where the other reports are.
