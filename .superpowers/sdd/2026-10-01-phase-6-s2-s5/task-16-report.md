# Task 16 report: the native call split

Base `1e323ead0`. Commits `68917f399` (the split), `5eeb97fa1` (prepare fills in place; the halves
inline), and this report.

## Design

`invoke::run` is gone. `invoke::method` and `invoke::routine` now run, back to back on the baton:

- `NativeCall::empty()`, then `prepare(&mut native, signature, cx, arguments)`. This is
  `processArguments`, and it fills the call in place.
- `Host::between_halves()`.
- `call_method(&mut native, entry, context)` or `call_routine(...)`, which returns a `Completion`.
- `Host::between_halves()`.
- `finish(cx, completion)`, which is `valueToObject`.

`routine_signature` holds the classic-style refusal and the bounded read for routines, as
`signature` does for methods. `hook` and `command` are unchanged.

- **`NativeCall`** holds the declared result word (`u16`) and the
  `[ValueDescriptor; MAX_NATIVE_ARGUMENTS]` array. It is `Send` because its fields are.
- **`Completion`** holds the declared result word, `Option<Written>` (the element-zero payload) and
  the refused slot (`Option<&'static str>`, captured by `recording_refusals` on the calling
  thread). It is `Send` for the same reason.
- **What makes those fields `Send`:** `rexx-api/src/ffi.rs` declares `unsafe impl Send for
  ValueDescriptor` and `unsafe impl Send for Value`, each with a SAFETY note. Both types are plain
  words whose pointer members are addresses. Reading through such an address is `unsafe` at its own
  site, and a handle names an object only through the minting call's locals. Neither record gets an
  `unsafe impl` of its own, so `Send` stays derived from the fields, and an `ObjRef` field
  (`!Send`) would remove it.
- **Enforcement:**
  - a `const` block in `invoke.rs` asserts `NativeCall: Send` and `Completion: Send`;
  - a `compile_fail,E0277` doctest on `Completion` shows that a record carrying a `Completion` beside
    an `ObjRef` is not `Send`.
- **The native frame** stays pushed by `run_library_method` and `run_library_routine` from before
  `prepare` until after `finish`, as before. A returned handle resolves in that frame's `locals`
  inside `finish`.
- **`Host::between_halves`** has a default no-op. rexx-exec's `Interp` runs `collect_now()` there
  when `stress_collect` is set, so the collect-stress mode collects between prepare and finish (after
  the arguments are converted, and after the stub returns).

### Departure: pending conditions are not in `Completion`

The brief lists "pending conditions" among `Completion`'s fields. A condition the stub raises is
recorded by `RaiseException*` into the call's `Activation`, which is a callback into island state,
and `finish` reads it there. Copying it into the `Completion` would mean reading the `Activation`
from the thread that ran the call. This is stated on `Completion`'s doc.

### Performance remedy

The first shape (`68917f399`) answered `Result<NativeCall, Failure>` and took the call by value. It
cost extcall +3.65% Ir excluding libc, plus +555M Ir of libc `memcpy`: two copies of the descriptor
array per call, and `prepare` and `finish` out of line.

`5eeb97fa1` fixes it three ways:
- `prepare` fills a caller's `NativeCall::empty()`;
- `call_*` take it by `&mut`;
- the halves are `#[inline(always)]`.

Plain `#[inline]` measured +3.03% (scratch build), because it left them out of line.

The remaining +72M Ir (24 Ir per call) is `between_halves`:
- two dynamic calls through `&mut dyn Host`, each behind a `RefCell` borrow;
- 18M of that is the function's own test of `stress_collect`.

## Callgrind (Ir, libc and ld-linux excluded)

Command: `bench-programs/callgrind.sh -r 1 -j 3 -p "extcall dispatch fibcall" base=<base> head=<head>`.
- Each binary is `rexx-run`, built `--release` from `git archive` (`rust interpreter api`) into its
  own `CARGO_TARGET_DIR`.
- Every file was touched first, and both builds printed `Compiling rexx-api` and
  `Compiling rexx-exec`.
- base is `1e323ead0`; head is `5eeb97fa1`.

| program | base | head | delta |
|---|---|---|---|
| extcall | 8128604564 | 8200604627 | +0.8858% |
| dispatch | 21264385155 | 21264384971 | -0.0000% |
| fibcall | 8494020441 | 8494020504 | +0.0000% |

`cgdiff.py` on extcall, head against base:
- self: `run_library_routine` +54000000, `between_halves` +18000000;
- calls: `between_halves` +6000000;
- no `memcpy` delta.

For the intermediate `68917f399` against base: extcall +3.6538%, dispatch +0.0000%, fibcall
+0.0000%.

## Mutation evidence

Each mutant was applied by `scratchpad/t16/mut/mutate.py`, which copies the file, mutates it, runs
the command and restores from the copy. The rexx-exec mutants ran in a `git archive` of `5eeb97fa1`
under `--profile mutation`.

| mutant | result |
|---|---|
| test host's `between_halves` roots no locals (control) | `a_handle_built_during_the_call_resolves_after_collections_between_the_halves` fails |
| `routine` drops the second `between_halves` | same test fails (collections 1, not 2) |
| `method` drops the first `between_halves` | `a_native_call_that_raises_keeps_its_condition_across_the_halves` fails |
| `finish` keeps the pending condition on a refusal | lib test `a_refused_member_is_the_calls_answer_and_a_nested_call_keeps_its_own` fails |
| `unsafe impl Send for Value` removed | `rexx-api` does not compile: the `const` assertion, E0277 on `*const i8`, `*mut c_void`, `*mut RexxObjectPtr_` |
| `unsafe impl Send for ValueDescriptor` removed | same, E0277 on every pointer member |
| doctest's `ObjRef` replaced by `u64` | the `compile_fail` doctest fails (it compiles) |
| rexx-exec: a native frame's `locals` not rooted (`activity.rs:572`) | `collect_stress` exit 101, `the_l0_subset_passes_again_under_collect_on_every_allocation` (panic `dispatch.rs:1660`, "a live value") |
| the same, with `Interp::between_halves` a no-op | `collect_stress` exit 0, 36 passed |

The last two rows show that the new collection is the only thing in `collect_stress` that sees a
native call's locals going unrooted.

## Checks

All runs are from `rust/` at `5eeb97fa1` (the tree as committed), with statuses written unpiped by
`scratchpad/t16/checks{A,B}.sh`.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings`: exit 0.
- `memcap 8G cargo test -p rexx-api` (all targets and doctests), exit 0:
  - lib 64;
  - context 14, handles 5, invoke 16, layout 23, load 11, values 83;
  - doctests: 1 and 5, including `invoke::Completion (line 182) - compile fail ... ok`.
- `memcap 8G cargo test -p rexx-exec --lib`: 945 passed.
- `cargo test -p rexx-core --test unsafe_sites`: 2 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: `787 of 787
  matching`, exit 0.
- The same with `REXX_CORPUS_SWITCH=every`: 787 of 787, exit 0.
- Debug with `every`: 787 of 787, exit 0.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test [--release] -p rexx-exec --test collect_stress`:
  - release: 36 passed;
  - debug: 36 passed.
- `memcap 8G cargo test --release -p rexx-exec --test refusal_sites --test method_bodies --test
  gate_table_c --test dispatch_seam --no-fail-fast`: exit 0.
  - refusal_sites 5, method_bodies 23, gate_table_c 22, dispatch_seam 6.
  - `corpus/refusal-sites.tsv` needed no re-derivation.
- `memcap 8G cargo test --release -p rexx-parse --test sourceline_oracle`: 1 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test api_group_tests`: 24 passed, exit 0.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test concurrency_tests --
  --test-threads=1`: 32 passed, 839 s, exit 0.
- Pinning: `RAYON_NUM_THREADS=4 CARGO_TARGET_DIR=<own> memcap 8G cargo test --release -p rexx-exec
  --features pinning --test concurrency_tests -- measured:: --nocapture --test-threads=1`: 18
  passed.
- Loom: `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=<own> memcap 8G cargo test -p rexx-exec --test
  loom`: 6 passed.
- "It works": every `rust/bench-programs/*.rex` and `bench-rexxcps/rexxcps.rex` was run on both
  release binaries, from `rust/` with `LD_LIBRARY_PATH` at the oracle's `build/lib`.
  - Exit status is 0 and identical on both.
  - Stdout is identical except the timing lines of `heapshape` (`build_seconds=`,
    `gc_pause_seconds=`) and `rexxcps` (`Performance:`).
  - `extcall` prints `3000000` on both.

## Concerns

- `Completion` carries no pending condition (see the departure above).
- `between_halves` costs extcall about +0.9% Ir on every native call. This is within the brief's
  ~+1%. Making the hook free outside the stress mode would need a flag the `Activation` reads
  without a dynamic call.
