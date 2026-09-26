# Task 5 review: the boundary (soundness, unsafe, handles, Miri)

Reviewer: boundary half. Range `e67b2f703..fb52b794e`, read in the main checkout at `fb52b794e`
(clean) and built from `git archive fb52b794e rust api interpreter testbinaries` into
`$R=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s5b/tree`.
Probes: `$R/../bprobe.cpp` (built `g++ -shared -fPIC -O1 -pthread` against the tree's `api/`,
NEEDED `libc.so.6` only, no undefined `Rexx` symbol), programs `$R/../probes/*.rex`, outputs
`$R/../out/<probe>.<side>.{out,err}`; each side run from a fresh `mktemp -d`, oracle under
`ulimit -v 1048576`, crate = `$R/../target/release/rexx-run` built from that tree.

## Verdict

**Not ready: one Critical.** A conforming extension corrupts the heap through `MutableBufferData`
on a copied buffer. The rest of the boundary holds, apart from one Important lifetime gap and
some Minor findings.

## Findings

**Critical C1: `MutableBufferCapacity` reports more room than `MutableBufferData`'s allocation has.**
`surface.rs:226-233` hands out `state.bytes.as_mut_ptr()` with `state.capacity` as the extent. But
nothing keeps `bytes.capacity() >= capacity`:
* `Object~copy` clones the body (`object_protocol.rs:598`), and `Vec::clone` allocates only the
  length.
* `new_mutable_buffer` ignores a failed reservation (`surface.rs:212-216`).
* `BufferState::ensure_capacity` raises `capacity` before `try_reserve_exact` can fail
  (`body.rs:408-415`).

Measured: `probes/mb.rex` gives `copy len=3 cap=100 usable_ge_cap=0` on the crate. The original
buffer gives `usable_ge_cap=1`, which is the control showing the instrument can tell the two
apart. `probes/mbfill.rex` fills the reported capacity of `.MutableBuffer~new('abc',4000)~copy`:
the oracle prints `4000`/`4000` at rc 0, and the crate aborts with `realloc(): invalid next size`
at rc 134. The `Surface::mutable_buffer` contract (`callbacks.rs:97-99`) states no writable
extent. Miri cannot see this, because the FakeHost only makes fresh buffers.

**Important I1: a `CSTRING` answer dies when the call ends, even when the object outlives it.**
`StringData` (`values.rs:803-813`) and `ObjectToStringValue`/`GetMessageName`
(`callbacks.rs:571-586`, `:1410`) intern into the per-call `CStringPool` (`library.rs:86`, `:151`,
`:191`). The oracle hands out an interior pointer that lives as long as the string does
(`CStringPool::intern_for`'s own doc). Measured with `probes/kept.rex` (`RequestGlobalReference`,
keep `StringData`, 3000 allocations, read it in a later call):
* oracle: `same_address=1 kept=[hello world, kept]`
* crate: `same_address=0 kept=[ReadKept]`, which is a read of freed memory already reused.

The pool design predates Task 5 (object variables could reach it before). Global references
make it an ordinary pattern. Owner is the controller's ruling.

**Minor M1: `attach_thread` reads another thread's `Cell` before refusing it.** At
`ffi.rs:3155-3158` it reads `(*thread).innermost.0.get()`, a `!Sync` `Cell`, together with `home`,
and only then asserts `home == current`. Called from a foreign thread, that read races the home
thread's `enter`/`Entered::drop` writes. It is UB just before the abort. Check `home` first. The
refusal itself is sound and loud: `probes/attach.rex` (a pthread calling `AttachThread`) gives
rc 134, `RexxInstanceInterface.AttachThread is not implemented (Phase 9)`, where the oracle gives
`1`.

**Minor M2: the `SAFETY` precondition of `activation_of`/`innermost_activation` is no longer
true.** It says "no caller holds its conversion state for the duration" (`ffi.rs:619-622`,
`:643-646`). But `with_surface` holds the `RefMut` across `Surface::send` (`callbacks.rs:241-253`,
`:872-882`). An extension that uses its outer call context from a call nested inside a
`SendMessage` gets `RefCell already borrowed` (`values.rs:735`), and the process aborts at rc 134
(`probes/outer.rex`). The oracle answers `USEOUTER`. This is sound, because the `RefCell` catches
it, but the stated invariant is false and it is a divergence.

**Minor M3: zero-size object memory aliases.** `AllocateObjectMemory(0)` answers the same dangling
`Vec` address every time, so `FreeObjectMemory(p1)` drops every zero-size buffer
(`surface.rs:508-516`, `retain`). `probes/zero.rex`: the crate gives `distinct=0 realloc_null=1`,
the oracle `distinct=1 realloc_null=0`. This is not UB. Separately, `Vec<u8>` promises only
alignment 1. glibc gives 16 in practice, but nothing states it.

**Minor M4: Miri reaches little of the new surface.** `RUSTUP_HOME=<surface-4>/rustup-home
cargo +nightly miri test -p rexx-api --lib --offline` on the archived tree: exit 0, 34 passed,
8 ignored (`$R/../miri.txt`), which matches the report. Of the 168 member names the diff assigns
(`grep '^+ *table\.X =' ffi.diff`), Miri reaches 26: the member calls in `ffi.rs`'s test module
and `invoke/tests.rs` (`comm -12`, list in `$R/../miri-all.txt`). Unreached:
* `attach_thread`'s `offset_of!` walk back to `Thread` (its provenance is sound by reading,
  `ffi.rs:455`)
* object memory, CSELF, `MutableBufferData`
* every `variables::`/`messages::` slot
* `RegisterLibrary`

## Checked and holding

* **1. `unsafe`.** Every `unsafe {` in `ffi.rs`/`load.rs` has a `SAFETY` line within six lines
  (awk scan, no hits). `load::registered` (`load.rs:818`) is an `unsafe fn` over
  `Library::this()`. `StringGet` bounds its copy (`ffi.rs:1364-1373`). `BufferStringData`'s area
  is a `Vec` in the per-call pool, stable until the call ends (`values.rs:462-491`). `Buffer`'s
  `Data` vector is never resized (`NativeState::Data` is written only at `surface.rs:202`,
  `:531`).
* **2. Handles.** A handle is the `ObjRef`'s bits plus one, generation included
  (`handles.rs:81-83`). `Host::resolve` tries locals then globals (`library.rs:782-786`). Globals
  are rooted (`lib.rs:2721`). `ReleaseLocalReference` removes only the local entry.
  `ReleaseGlobalReference` keeps the handle: verified that the oracle's `removeGlobalReference`
  calls `addGlobalReference` (`interpreter/runtime/InterpreterInstance.cpp:674-679`). That leaks,
  but never aliases.
* **3. Object memory.** Double free and a foreign pointer are no-ops. Realloc copies the old bytes
  and then frees them. The buffers are rooted through the receiver's pool, and the new buffer is
  temp-rooted (`surface.rs:666`) before `set_object_memory` allocates.
* **4. Threads and instance.** `GetInterpreterInstance` only reads. `AttachThread`/`DetachThread`
  write no global state. Apart from M1, a foreign thread gets a loud abort.
* **5. `callbacks.rs`.** It contains no `unsafe` and dereferences no pointer:
  `free`/`realloc_object_memory` only compare addresses. A context is built only by
  `ThreadContext::enter`, and `Activation` is not a context, so no safe path forges one.
  `register_library` takes a `Library` that only the unsafe loaders make. The split is right.
  C1 is a gap in the `Surface` contract, not in this module.
* **7. Derived tests.** `header_types` reparses `api/oorexxapi.h` at test time. A slot filled with
  a wrong signature fails to compile, so comparing the field types is enough. Refusal is derived
  from fn addresses against the `REFUSING` stubs, which fails safe: a spurious difference reddens
  the test. The baseline `cargo test -p rexx-api --test layout` is 24 passed. Predictions were
  written before each run:
  * C1: `ReleaseLocalReference`'s `RexxObjectPtr` becomes `isize` in `layout.rs`, with the
    `ffi.rs` fn changed to match. Predicted: `every_member_has_the_type_the_header_declares` red
    alone. Result: as predicted, 23 passed and 1 failed (`assertion failed: RexxThreadInterface`).
  * C2: `("RexxThreadInterface.StringData","Phase 8")` is added to `REFUSING_MEMBERS`. Predicted:
    `a_populated_table_refuses_exactly_the_members_it_names` red alone. Result: as predicted,
    23 passed and 1 failed.

  Both mutations were restored from copies and checked with `cmp`.
* **8. Throw\*.** All ten are `aborts` (`layout.rs:801-805`, `:829-833`). In `probes/throw.rex`
  (`signal on syntax`, `ThrowException0(40001)`, then a `printf`), the crate gives rc 134,
  `CallContextInterface.ThrowException0 is not implemented (Phase 8)`. It prints no
  `AFTER-THROW-RAN` and the trap is not taken. The oracle gives `trapped 40`. The refusal is loud
  and cannot be trapped.
