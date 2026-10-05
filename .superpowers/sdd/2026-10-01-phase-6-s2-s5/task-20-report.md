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

## Fix round 1

Base `55dd10af8`. Commits: `a48312f8e` (code and tests), `805b21795` (deviation row, spec
sentence, forge); this section and the review are committed after them.

### F1: the grown-buffer test raced its native

- **Where the out-of-bounds read was: the test's native, not the interpreter.** `hold_buffer`
  took the address and the length in two callbacks, `MutableBufferData` then
  `MutableBufferLength`, and read `length` bytes from the address. A growth between the two
  leaves the native with the old storage and the new length, and it reads past the old storage.
  The interpreter kept the old storage valid for its own length (retired, not freed). The oracle
  has the same window: each of the two stubs takes its own `ApiContext`
  (`interpreter/api/ThreadContextStubs.cpp:1989-2013`), and growth moves the data there too.
  Scenario 1 (growth before the first callback) is a legal schedule, not a defect either.
- **Fix:** `HOLDBUFFER` writes `PATH.held` once it holds the address and the length, and the
  `grow` and `overlay` methods poll `SysFileExists('PATH.held')` before changing the buffer. The
  SAFETY comment now states that ordering.
- **Proof, by running** the `scheduler::tests::lent::` filter of the `rexx-exec` lib test binary
  on default parallel test threads: 60 runs, 0 failed (release, at `a48312f8e`'s tree). With
  `--test-threads=1`: 40 runs, 0 failed (`--profile mutation`).
- **Mutant** (the `grow` method's wait removed, rebuilt with `--profile mutation`): 11 of 60
  runs red, all in `a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds`.

### F2: the report's oracle claim

The Checks line "the oracle's natives run holding the kernel lock, so it cannot schedule another
activity while a call holds the address" is false. `NativeActivation::run` and
`callNativeRoutine` release the kernel around the native (`NativeActivation.cpp:1304`, `:1420`),
so the oracle runs this interleaving. The forge now carries it: `lent.cpp` defines `HOLDBUFFER`
and `FINISHEDINPLACE` as rexx-api's test natives do, with the `.held` rendezvous, built by
`build.sh`. From a fresh directory per run, 30 runs each, oracle under `ulimit -v 1048576` and
`timeout -k 5 20`, and `rexx-run` built from `a48312f8e`:

| Program | Oracle | rexx-run |
|---|---|---|
| `grow.rex` | `abcdefghijklmnopqrstuvwxyz` / `abcd 5026` / `grown`, rc 0, 30 of 30 | same, 30 of 30 |
| `overlay.rex` | `abXYefghijklmnopqrstuvwxyz` / `ZZXY 26` / `overlaid`, rc 0, 30 of 30 | same, 30 of 30 |
| `growgc.rex` | `00000000` / `abcd 5026` / `grown`, 30 of 30 | `61626364` / `abcd 5026` / `grown`, 30 of 30 |
| `finishshort.rex` | `hello` / `helloxxxxx`, 30 of 30 | same, 30 of 30 |

`growgc.rex` collects between the growth and the call's read; its first line is the hex of the
first four bytes the call read. That row is the licensed divergence.

### F3: the divergence row

The row is gone from the Phase 8 close section and is DEVIATIONS entry 9, "A NATIVE CALL'S OLD
MUTABLEBUFFER STORAGE OUTLIVES A COLLECTION", owner none. It says what verdict 1 says: the two
agree while no collection runs during the call, and differ only after one, where the oracle's
call reads reclaimed memory and this crate's reads the original bytes. It cites the forge and
the runs above. Spec 2.5's sentence is replaced with the review's text; the false clause is
deleted, not reworded. The Divergence row section above, which says the row was in DEVIATIONS
and that the oracle cannot schedule the interleaving, was false on both counts.

### F4: the FinishBufferString terminator

Matched to the oracle. `CStringPool::written` now answers every byte made along with the
capped finished length, `Surface::finish_string` takes both, and a kept copy taken before the
finish gets every made byte with its terminator left at the made length, as `RexxString::finish`
sets only the length (`StringClass.hpp:541`). The string's value is still the finished bytes.

- **Test:** `finishing_a_buffer_string_keeps_its_kept_string_in_place` now calls
  `FINISHEDINPLACE('hello', 5)` and `FINISHEDINPLACE('hello', 10)` and expects `hello` and
  `helloxxxxx`. `finished_in_place` takes the made length, fills with `x`, and reads the early
  address up to its NUL.
- **Mutants** (whole `rexx-exec` lib, `--no-fail-fast`): F4a writes a NUL at the finished length
  after the copy, F4b copies only the finished bytes. Both red, in that test only (994 passed,
  1 failed each).
- A later `CSTRING` or `StringData` of the same string, while that kept copy lives, also reads the
  made bytes, as the oracle's one storage does. Without an early `StringData`, a later copy is
  made from the finished value and stops at the finished length where the oracle's does not;
  that is not introduced here.

### F5: abandon released what the native still held

Fixed by keeping the frame until the completion. `pop_native_frame` is split into
`take_native_frame` (pop, take the held condition) and `end_native_frame` (clear, release kept
holders and lent buffers). `abandon_native_call` takes the frame and parks it, with its frame
token, in `Activities::abandoned`, whose frames are roots (`NativeFrame::object_roots`, now
shared with `Activity::object_roots`). `file_completions` hands a completion that matches no
current call to `Interp::end_abandoned_call`, which ends the frame. A run that ends with the
call still in flight leaves it held, as P53 leaves the `Interp`.

- **Tests:**
  - `dispatch::library::tests::an_abandoned_calls_frame_holds_until_its_completion`: a frame
    lends a buffer and keeps a handle-carried string; the buffer grows; after the abandon the
    old storage is still retired, the buffer is still a root, and the holder count is 1; after
    `end_abandoned_call` all three are gone.
  - `scheduler::tests::lent::an_abandoned_calls_frame_ends_with_its_completion`: under
    `fail_native_wait`, `HOLDCSTRING` is abandoned with 11.1 and then completes; the run ends
    holding no abandoned frame (`Ran::abandoned`, read from a test-only thread-local set at the
    run's end). It waits 0.3 s after the native's file for the completion to drain.
- **Mutants** (whole `rexx-exec` lib, `--no-fail-fast`):

| Mutant | Result |
|---|---|
| F5a abandon ends the frame at once (the old behaviour) | red: the unit test |
| F5b `file_completions` never calls `end_abandoned_call` | red: the scheduler test |
| F5c abandoned frames not rooted | red: the unit test |
| F5d `end_abandoned_call` does nothing | red: both tests |

- Not tested: a native reading lent storage after an abandon. `fail_native_wait` fails the
  first wait, before any callback is served, so a callback-taking native like `HOLDBUFFER` gets
  null from its abandoned call and lends nothing. The unit test covers the lend.

### Checks

From `rust/`, `CARGO_TARGET_DIR=.../p6-scratch/t20f1/target`:
- `cargo fmt --all --check`: exit 0 (at `a48312f8e`'s tree, before commit).
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0 (same tree).
- `cargo clippy --workspace --all-targets --features pinning -- -D warnings`: exit 0 (same tree).
- `cargo test --release -p rexx-api -p rexx-core --no-fail-fast`: all green (same tree).
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release --no-fail-fast` at `805b21795`: exit 0, 142 `test result` lines, 2987 passed, 0 failed (summed over those lines).
- Mutation baseline M0 (`--profile mutation -p rexx-exec --lib --no-fail-fast`): 995 passed, 0
  failed. Each mutant was applied by exact-string replacement (one occurrence asserted) and
  restored from a saved copy (equality asserted); `git status` showed only the lead's
  `progress.md` after the round.

### Concerns

- F5's scheduler test waits 0.3 s for the abandoned call's completion to drain; the completion is
  posted within one 10 ms poll of the file appearing, but nothing orders it.
- `Activities::abandoned` is searched linearly per unmatched completion; it holds only calls
  abandoned while running.
