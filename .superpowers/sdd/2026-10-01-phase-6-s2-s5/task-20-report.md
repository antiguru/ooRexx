# Task 20 report: lent island memory

Base `2fa650384`. Commits: `6ceab29b0` (code, tests, divergence row), `67a5159fb` (shrink test
witnessed by capacity, after M9 survived); this report is committed after them.

## What was built, per kind

- **MutableBuffer (`MutableBufferData`, `SetMutableBufferCapacity`).** `BufferState::bytes` is now a
  `BufferBytes` (`rexx-core/src/body.rs`): a `Vec<u8>` with a lend count and a list of retired
  storages. It dereferences to `[u8]`, so in-place reads and writes cannot reallocate; the growth
  methods (`try_reserve_exact`, `extend_from_slice`, `resize`) move the bytes to new storage when
  they are lent and the growth does not fit, and keep the old storage in the list; `shrink_to` does
  nothing while lent; `edit` hands out the `Vec` for the shortening helpers (`delete_range`,
  `delword_bytes`) and panics if a lent storage moved. The surface's `mutable_buffer` and
  `set_mutable_buffer_capacity` call `Interp::lent_buffer`, which lends the buffer once per native
  frame (`NativeFrame::lent`); `pop_native_frame` releases each, and the last release frees the
  retired storages. A frame's lent buffers are roots (`Activity::object_roots`). The compiler found
  the mutation sites: every `state.bytes` growth now goes through `BufferBytes`.
- **Buffer (`BufferData`).** Nothing to change: `NativeState::Data`'s `AlignedBytes` is never
  resized, and it is freed only with its object, which the call's handle roots (a released local
  handle stays rooted until the call ends, phase-4-exclusions.txt's ReleaseLocalReference row).
- **Kept strings, handle-carried values.** Already held: `kept_c_string` counts each call in
  `kept_holders`, and `drop_loose_kept_strings` keeps a held copy. The callback runs with the
  call's activity switched in (`Recalling`), so the count lands on the right frame. Witnessed now
  under the pool interleaving.
- **Kept strings, heap strings.** A copy is dropped only when a collection frees its string, which
  the call roots. Witnessed now under the pool interleaving.
- **`FinishBufferString`.** It dropped the string's kept copy, freeing an address `StringData`
  had answered to the same call. It now rewrites the copy in place where the finished bytes fit,
  which they do for any copy taken before a finish, and drops it otherwise. The oracle's
  `StringData` and `BufferStringData` answer the same storage and `RexxString::finish` sets only
  the length (`StringClass.hpp:541`): a forged extension
  (`docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-20-forge/`: `finish.cpp`, its
  `build.sh`, `finish.rex`) answers `hello` on the oracle,
  rc 0, empty stderr, 3 of 3 runs, and on this crate's `rexx-run` at `6ceab29b0`.

## Tests and mutants

New natives in `rexx-api/src/load.rs` (`doc(hidden)`, `unsafe` stays there): `hold_c_string`,
`hold_buffer`, `finished_in_place`. Each waiting native answers a failure value where its rendezvous
file never appeared, so a run in which the other activity did not run during the call is red.

| Test | Shows |
|---|---|
| `scheduler::tests::lent::a_held_handle_carried_string_survives_another_activitys_prune` | main holds `'abc'`'s copy on a pool thread while another activity takes 5000 kept strings (past the prune bound) |
| `..::a_held_heap_string_survives_another_activitys_collection` | main holds a 40-byte string's copy while another activity allocates until collections run (`collections > 0` asserted) |
| `..::a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds` | another activity appends 5000 bytes; the call reads its 26 original bytes, its `ZZ` is not seen (`abcd 5026`) |
| `..::a_buffer_changed_in_place_under_a_call_stays_shared` | another activity overlays in place; the call reads `abXY...`, Rexx reads the call's `ZZ` |
| `..::finishing_a_buffer_string_keeps_its_kept_string_in_place` | `hello` through the early `StringData` address |
| `dispatch::library::tests::a_lent_buffers_old_storage_lives_until_its_last_call_ends` | two frames lend one buffer (the inner twice); the inner pop keeps the storage, the outer frees it |
| `tests::a_native_activations_call_state_is_rooted_only_while_it_lives` | gains the lent buffer |
| `rexx-core/tests/buffer_bytes.rs` | each growth path moves lent bytes; unlent and fitting growth do not; shrink keeps lent storage; a copy is unlent; `edit` panics |

Mutation harness `p6-scratch/t20/mut.py`, run in a copy of the tree (`git archive 6ceab29b0`,
with the read-only source directories symlinked) on its own target dir: exact-string apply (one
occurrence asserted), restore from a saved copy (asserted equal), `memcap 8G cargo test --profile
mutation -p <crate> ... --no-fail-fast -- <tests> --exact`, counts read from the `test result`
lines; a run of nothing is reported as such (the first round ran nothing: the copy lacked
`interpreter/`, fixed). M0 is the unmutated copy: green, 7 and 6 passed.

| Mutant | Result |
|---|---|
| M1 `drop_loose_kept_strings` ignores holders | red: the carried interleaving test and the existing `a_handle_carried_values_copy_lasts_until_a_collection` |
| M2 a collection drops every heap copy | red: the heap interleaving test and the existing `a_kept_c_string_lives_and_dies_with_its_string` |
| M3 `lent_buffer` lends nothing | red: grown-buffer test, `a_lent_buffers_...` |
| M4 lent growth reallocates in place | red: grown-buffer test; two `buffer_bytes` tests |
| M5 lent bytes move on every reservation | red: in-place test; `unlent_or_fitting_growth_keeps_the_storage` |
| M6 finish drops the kept copy | red: finish test |
| M7 no release at frame pop | red: `a_lent_buffers_...` |
| M8 lent buffers not rooted | red: the rooting test |
| M9 shrink while lent | first green (glibc shrank in place, the address test could not see it); red after `67a5159fb` |
| M10 no per-frame dedupe | green: equivalent; each duplicate lend is pushed and released, so the count balances. The dedupe keeps `frame.lent` short. |
| M11 `edit`'s check removed | red: `an_edit_growing_lent_bytes_panics` |

M1 and M2 were already caught by existing unit tests; the new interleaving tests add the
pool-thread schedule (P52), not mutant coverage.

## Divergence row

`docs/superpowers/plans/phase-4-exclusions.txt`, DEVIATIONS section (the file's permanent,
owner-none rows, as the Phase 8 API divergences are recorded), last row: "C WRITES TO A
MUTABLEBUFFER'S STORAGE AFTER ANOTHER ACTIVITY GREW IT ARE NOT SEEN", OWNER: none. It has no
`LICENSED DIVERGENCE WITNESS` marker: `licensed_divergences.rs` runs pure Rexx programs against the
oracle, and this one needs a native extension and an interleaving the oracle cannot schedule.

## Checks

From `rust/`, `CARGO_TARGET_DIR` in the task's scratch, at `6ceab29b0`'s tree:
- `cargo fmt --all --check`: exit 0 (also after `67a5159fb`).
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo clippy --workspace --all-targets --features pinning -- -D warnings`: exit 0.
- `cargo test --workspace --release --no-run`: exit 0; then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, 142 `test result` lines, 2985 passed, 0 failed (summed over
  those lines).
- After `67a5159fb` (test file only): `cargo clippy -p rexx-core --all-targets -- -D warnings`
  exit 0; `cargo test --release -p rexx-core --test buffer_bytes`: 6 passed.
- Oracle: the finish forge above, 3 runs (not concurrent). The buffer interleaving has no oracle
  run: the oracle's natives run holding the kernel lock, so it cannot schedule another activity
  while a call holds the address.

## Departures and concerns

- **The spec's divergence sentence is not quite the oracle's behaviour.** Spec 2.5 says C writes
  after a reallocation "are writes into freed memory" in the oracle. Read: `ensureCapacity` gives
  the buffer a new data object (`MutableBufferClass.cpp:246`), so the oracle does not see those
  writes either, and the old object is garbage a later collection reclaims. The row states that;
  the spec sentence is unchanged.
- **Outside the brief:** the `FinishBufferString` fix (a kept string pruned under its own call)
  and rooting a frame's lent buffers.
- **`BufferBytes::edit` panics** if a shortening helper ever reallocates lent storage; today's
  helpers (`delete_range`, `delword_bytes`) only shorten. A loud failure rather than a free under
  the call.
- **A lent buffer whose object a collection freed** releases nothing at pop (`buffer_mut` answers
  `None`); with lent buffers rooted, this cannot happen while the frame lives.
- No perf measurement (P51): a `MutableBufferData` call now does a `Vec::contains` over the frame's
  lent buffers and a second heap lookup.
