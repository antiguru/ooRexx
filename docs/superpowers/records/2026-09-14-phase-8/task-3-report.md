# Task 3 report: The C structs, and the cast back to our own state

Base: `152740fbe`. Commit: `b975c534a`, seven files, `Cargo.lock` untouched: `rust/crates/rexx-api/src/layout.rs` and
`rust/crates/rexx-api/tests/layout.rs` created; `rust/crates/rexx-api/src/ffi.rs`,
`rust/crates/rexx-api/src/lib.rs`, `rust/crates/rexx-api/src/load.rs`,
`rust/crates/rexx-api/tests/load.rs` and `rust/crates/rexx-core/tests/unsafe_sites.rs` modified.

## The types defined

All in `rexx_api::layout` unless the line says otherwise. The module carries
`#![allow(non_camel_case_types, non_snake_case)]` so that every name is the frozen header's own
spelling and the file can be diffed against `api/oorexxapi.h` member for member.

**Scalars** (`api/rexx.h:78-79`, `:236-238`, `api/platform/unix/rexxapitypes.h:64`):
`wholenumber_t = isize`, `stringsize_t = usize`, `logical_t = usize`, `CSTRING = *const c_char`,
`POINTER = *mut c_void`, `REXXPFN = *mut c_void`.

**Object handles.** For each handle name in `api/rexx.h:117-159` an opaque `#[repr(C)]`
`<Name>_` with one private zero-length field, and `pub type <Name> = *mut <Name>_`. The pointee is
uninhabitable from outside the module, which is what keeps a handle opaque.

**Interface versions**: `INSTANCE_INTERFACE_VERSION`, `THREAD_INTERFACE_VERSION`,
`METHOD_INTERFACE_VERSION`, `CALL_INTERFACE_VERSION`, `EXIT_INTERFACE_VERSION`,
`REDIRECT_INTERFACE_VERSION`, each a `wholenumber_t` and each asserted against its `#define`.

**Structs**: `RexxCondition` (`:462-473`), `ValueUnion` + `ValueDescriptor` (`:281-394`),
`RexxRoutineEntry`, `RexxMethodEntry`, `RexxPackageEntry` and the `PackageHook` alias (moved out of
`load.rs`, not re-declared), `RexxInstance_` (`:788-791`), `RexxThreadContext_` (`:825-828`),
`RexxMethodContext_` (`:1666-1670`), `RexxCallContext_` (`:2518-2522`), `RexxExitContext_`
(`:3349-3353`), `RexxIORedirectorContext_` (`:4138-4140`).

**The private wrapper** is one generic type rather than one per context:

```rust
#[repr(C)]
pub struct Owned<C, T> {
    pub context: C,
    pub owner: *mut T,
}
```

That is the `ActivationApiContexts.hpp:58-95` shape -- public struct first by value, back-reference
second -- parameterised instead of repeated. It is generic in `T` because this crate has no owner
type yet: naming one now would be inventing the abstraction a later task owns. The exception is
`RedirectorContext` (`:88-94`), which carries three back-pointers rather than one; nothing hands a
redirector context out, so no wrapper for it is defined.

**`PackageHook`'s argument is now the real type**: `unsafe extern "C" fn(*mut RexxThreadContext_)`.
The alias lives in `layout.rs`, which is *not* granted `unsafe`. That is sound and was checked by
running rather than assumed: `rustc --edition 2024 --crate-type lib` on a file with
`#![deny(unsafe_code)]` and an `unsafe extern "C" fn` **type alias** exits 0 with no diagnostic --
the lint fires on `unsafe` definitions, blocks, impls and traits, not on function-pointer types.
`unsafe_sites.rs`'s scan likewise looks for `unsafe {` / `unsafe fn` / `unsafe impl` /
`unsafe trait`, and `unsafe extern "C" fn(` in a type position matches none of them.

## The tables

`RexxThreadInterface` and `MethodContextInterface` are populated, per the dispatch's scope ruling.
`RexxInstanceInterface`, `CallContextInterface`, `ExitContextInterface` and
`IORedirectorInterface` are defined and unconstructed; `layout::instance_interface()`,
`call_context_interface()`, `exit_context_interface()` and `io_redirector_interface()` are the
sites that would hand one out, and each panics with `<Name> is not implemented (Phase 8)`.
I found no path that contradicts the ruling: `RexxMethodContext_` holds a `*mut
RexxThreadContext_`, a `*mut MethodContextInterface` and a `*mut ValueDescriptor`, and
`RexxThreadContext_` holds a `*mut RexxInstance_` and a `*mut RexxThreadInterface`. The instance
pointer is a path to the instance table, but the L2 slice never builds an instance, and
`GetInterpreterInstance` is itself a stub that refuses.

A table is declared by one `interface!` invocation carrying its member list. From that one list the
macro builds the `#[repr(C)]` struct, a `FIELDS` const naming the members in order, and -- for a
`populated` table -- the `REFUSING` associated const in which every function member is a typed
`extern "C"` stub with the exact signature the header declares and a body that calls
`refuse(<table>.<member>)`.

**Why there is no runtime "no entry is null" assertion.** A function member's type is
`extern "C" fn(..)`, not `Option<extern "C" fn(..)>`, so a null is not a representable value, and
a struct literal that omits a field does not compile. The null failure mode is designed out at the
type level; what a test has to catch instead is a member *missing relative to the header*, which
is what the derived test below does. What a runtime witness can add is that a stub actually
refuses rather than silently answering, and that is `an_unbuilt_entry_refuses_loudly`.

**The stubs abort, they do not unwind.** `refuse` panics, the stub is an `extern "C"` frame, and
Rust 2024 turns a panic crossing that boundary into an abort with the message on stderr. That is
loud, and it is why the witness runs the call in a child process. **Whether the eventual real
bodies need `extern "C-unwind"` is left open** -- the C++ raises a Rexx condition by throwing
through the extension's frames (`RaiseException`), so a later task has to settle it; the header
declares plain C functions and this task matched that.

**Two entries are populated with a value that is not yet right, and a comment says so.**
`RexxThreadInterface`'s `RexxNil`, `RexxTrue`, `RexxFalse` and `RexxNullString` are object
pointers, not functions, so they cannot refuse; they are null until an interpreter instance fills
them. An extension reading `context->RexxNil` today gets null. This is not a null *function*
pointer and not the failure mode step 3 is about, but it is a hole, and it belongs to whichever
task wires the object system to a thread context.

## The counts the derived test asserts

Re-run here rather than taken from the dispatch:

```
$ grep -cE '\(RexxEntry \* ?[A-Za-z_][A-Za-z0-9_]*\)' api/oorexxapi.h
220
```

The generator that produced the member lists parses each struct body out of the header and reports
per-struct totals; run against `api/oorexxapi.h` on 2026-09-14 it gives

| struct | members | function pointers |
|---|---|---|
| `RexxInstanceInterface` | 8 | 7 |
| `RexxThreadInterface` | 147 | 142 |
| `MethodContextInterface` | 27 | 26 |
| `CallContextInterface` | 22 | 21 |
| `ExitContextInterface` | 12 | 11 |
| `IORedirectorInterface` | 12 | 11 |

218 function pointers in the six structs, which with the two `RexxPackageLoader` /
`RexxPackageUnloader` typedefs at `:259-260` is the header's 220. Every one of the six
per-struct figures matches the dispatch's.

The generator is a scratchpad tool and is not what stands. The standing instrument is
`tests/layout.rs::every_interface_declares_the_members_the_header_does`, which reads
`api/oorexxapi.h` at test time, extracts each struct's member names **in declaration order**, and
compares them against that struct's `FIELDS`. It asserts order and spelling, not just a count, so
a member inserted, dropped, renamed or moved reddens it. `the_scan_reads_one_struct_at_a_time` is
its control: it asserts that `RexxThreadInterface`'s members do not include a
`MethodContextInterface` member and the reverse, so an empty or over-wide match cannot pass as
agreement.

`every_interface_is_one_word_per_member` asserts `size_of::<T>() == FIELDS.len() * size_of::<usize>()`
for each table, which is the ABI statement the member list implies on this platform.

## The offsets, and where their numbers come from

The expected sizes and offsets in `tests/layout.rs` are what the system C++ compiler reports for
the frozen header, not numbers I worked out. Derived once with a scratch program:

```
$ g++ -std=c++17 -I<repo>/api -I<repo>/api/platform/unix -o off off.cpp && ./off
```

which printed, among others, `RexxInstance_` 16/(0,8), `RexxThreadContext_` 16/(0,8),
`RexxMethodContext_` `RexxCallContext_` `RexxExitContext_` 24/(0,8,16),
`RexxIORedirectorContext_` 8/(0), `ValueDescriptor` 16/(0,8,10), `RexxPackageEntry`
64/(0,4,8,16,24,32,40,48,56), `RexxMethodEntry` and `RexxRoutineEntry` 32/(0,4,8,16,24,28),
`RexxCondition` 72, and the six table sizes 64, 1176, 216, 176, 96, 96 -- each exactly eight times
its member count. The versions it printed are 101, 103, 102, 101, 101, 100.

The g++ run is evidence, not a gate: making `cargo test` depend on a C++ compiler would be a new
hard dependency for every platform in `ci/platforms`, so the numbers are asserted in Rust and the
derivation is recorded here.

## Every `unsafe` block, and its SAFETY invariant

One new site, in `ffi.rs`, plus the `unsafe fn` that contains it.

`pub unsafe fn owner_of<C, T>(context: *mut C) -> *mut T` -- its `# Safety` section states that
`context` came from `Owned::<C, T>::context` on a wrapper that is still alive, with the same `C`
and `T`. **Who establishes it:** not the pointer, which an extension holds as an opaque address and
can hand back with any value at all; the interpreter, which mints every context it hands out and
never hands out one it did not build. That is the same trust `Activity.hpp:503`'s cast rests on.

The single block inside it, `unsafe { (*owned).owner }`, cites that the caller's guarantee makes
`owned` a live `Owned<C, T>`, whose `context` is first by value, so the cast is the identity on the
address and `owner` is in bounds.

`ffi.rs`'s two `#[cfg(test)]` unit tests each carry a `SAFETY:` note of their own for the calls
they make. The tests live in `ffi.rs` rather than in `tests/` deliberately: an integration test
calling an `unsafe fn` would need its own `#[allow(unsafe_code)]` and would change both lists in
`unsafe_sites.rs`, which is a grant nobody made.

`crates/rexx-api/src/ffi.rs` was added to `unsafe_sites.rs`'s `uses` list in sort order, in the
same commit. The `granted` list needed no change -- `ffi.rs` already carried the opt-in from
Task 1.

**No `pub(crate)` accessor was needed.** Nothing in this task calls a native entry point, so
`NativeMethodEntry::entry_point` stays private with `has_entry_point()` as its only reader, and
the two `compile_fail` doctests are untouched and still pass.

## The negative control

The prediction was written to `<scratchpad>/prediction.md` **before** the mutation was applied.
Verbatim, the mutation was: in `layout.rs`, swap the second and third fields of
`RexxMethodContext_` so the order becomes `threadContext`, `arguments`, `functions`. Predicted:

1. the workspace still compiles, every construction being a named-field literal;
2. `a_method_context_has_a_thread_a_table_and_arguments` fails at
   `offset_of!(RexxMethodContext_, functions) == 8`, reporting `left: 16, right: 8`;
3. the `size_of == 24` assertion in that same test still holds, so the failure names `functions`;
4. every other test in `tests/layout.rs` passes -- 16 passed, 1 failed -- including
   `a_wrapper_puts_the_public_context_first`, the size being unchanged;
5. `every_interface_declares_the_members_the_header_does` passes, measuring tables not contexts;
6. the lib target's two `ffi::tests` pass;
7. `tests/load.rs` passes unchanged, 9 passed.

Outcome, part by part:

1. **confirmed** -- it compiled.
2. **confirmed** -- `panicked at crates/rexx-api/tests/layout.rs:251:5: assertion left == right
   failed, left: 16, right: 8`, and `:251` is the `functions` line.
3. **confirmed** -- the panic is at `:251`, past the size assertion at `:248`.
4. **confirmed** -- `test result: FAILED. 16 passed; 1 failed`.
5. **confirmed** -- it is in the 16.
6. **confirmed** -- `test result: ok. 2 passed`.
7. **confirmed, but only after a second command**: `cargo test -p rexx-api` stops at the first
   failing binary and never reached `tests/load.rs`, so in the first run this part was
   *unobservable*. `cargo test -p rexx-api --test load` was run separately and gave
   `test result: ok. 9 passed`.

Reverted by re-editing the file, never by `git checkout`. `git status --short` afterwards listed
exactly the five modified and two new paths of this task and nothing else, and the reverted
declaration was read back on screen.

## Commands and exit statuses

Run from `rust/`, each status read unpiped, on the tree as committed.

| command | exit | note |
|---|---|---|
| `cargo fmt --all` | 0 | |
| `cargo fmt --all --check` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | warm target directory |
| `cargo test -p rexx-api` | 0 | 32 `ok` lines: 2 lib, 18 `tests/layout.rs`, 9 `tests/load.rs`, 3 doctests |
| `cargo test -p rexx-core --test unsafe_sites` | 0 | 2 `ok` lines |

`rust/CLAUDE.md` calls a same-session clippy green provisional and asks for a clean-target run at a
phase boundary. This was a warm run; the phase's four-gate close is not this task's.

An earlier clippy run exited 101: `#[must_use]` on the four refusing accessors made every call in
`a_table_outside_the_slice_refuses_where_it_would_be_handed_out` an
`unused_must_use` error. The attribute is meaningless on a function that never returns and was
removed rather than suppressed.

## What the brief or the dispatch got wrong, or left open

1. **The header's `#else` branch is unusable, and the C++ reading is the one implemented.**
   `api/oorexxapi.h:135-174` typedefs each context to a *pointer* when `__cplusplus` is not
   defined, so in C `RexxThreadContext *` would be a pointer to a pointer and `context->functions`
   would not compile at all. Everything here follows the `__cplusplus` branch, which is what the
   oracle's own extensions compile against and what D5 freezes. Nothing in the spec or the brief
   says which branch binds; it should.
2. **`ffi.rs` had to become a `pub mod`.** The brief lists `lib.rs` as modified but not why: an
   `unsafe fn` that nothing in the crate calls yet is dead code unless it is `pub`, and Task 2 had
   already had to make `load` public for the same reason. `lib.rs` now reads
   `pub mod ffi; pub mod layout; pub mod load;`.
3. **`tests/load.rs` moved three imports.** `RexxPackageEntry`, `RexxMethodEntry` and
   `RexxRoutineEntry` now come from `rexx_api::layout`; the brief's file list does not mention it.
   No re-export was added, because two paths to one type is the thing that goes stale.
4. **Step 4 asks for `size_of` and `offset_of` "for every struct in step 1", and step 1 lists four
   context structs.** `RexxExitContext_` and `RexxIORedirectorContext_` are defined and asserted
   too, because the exit and redirector interface signatures name them and the structs are
   therefore needed to write the tables at all.
5. **The brief's step 3 says "only the entries later tasks fill need a body now".** The dispatch's
   ruling supersedes that and is what was implemented: every entry of the two reachable tables has
   its own typed stub, and no entry has a real body.
6. **`ValueDescriptor`'s `type` is spelled `r#type`** in Rust. It is the header's name and the only
   available spelling; anything else would be the "something equivalent" step 1 rules out.
7. **`RexxNil` and its three neighbours are null**, as described above. The brief does not say what
   a data member of an unbuilt table should hold and there is no loud option for one.
8. **Unwinding across the FFI boundary is unsettled.** Recorded above; it is a decision a later
   task cannot avoid, and it is not in the spec's section 10 list of open questions.

---

# Fix round 1

Review came back SPEC PASS, QUALITY CHANGES-REQUESTED with three findings. All three addressed in
one commit on top of `b975c534a`. Finding 1 is only half a rename: one of the two tables cannot be
a `static` in Rust, for a reason the C++ shares and this phase's own constraint sharpens.

## Finding 1: `const` where the brief said `static`

**The reviewer is right that the deviation was undeclared, and that is the part I got wrong.** The
report's list of what the brief got wrong should have carried it and did not.

`MethodContextInterface` is now a `static`:

```rust
pub static METHOD_CONTEXT_INTERFACE: MethodContextInterface = MethodContextInterface::REFUSING;
```

That is one object at one address, which is what a table handed to C should be. It compiles because
the type is `Sync`: every member is either a function pointer or a `wholenumber_t`.

**`RexxThreadInterface` cannot be a `static`, and this was measured rather than argued.** Compiling
`assert_sync::<T>()` against the crate answers exit 0 for `MethodContextInterface` and two E0277s
for `RexxThreadInterface`: `*mut RexxObjectPtr_` and `*mut RexxStringObject_` are not `Sync`, which
`RexxNil`, `RexxTrue`, `RexxFalse` and `RexxNullString` are. A `static` of that type does not
compile, and the only ways to make it compile are an `unsafe impl Sync` in a module that is not
granted `unsafe`, or changing those members to an atomic type and losing the header's spelling.

**And a `static` would be the wrong object even if it compiled.** The C++'s equivalent is mutable
process-global state: `Activity::threadContextFunctions` is patched at startup by
`Activity::initializeThreadContext` (`interpreter/concurrency/Activity.cpp:1844-1849`), which fills
exactly those four members from `TheNilObject` and friends. This phase's Global Constraints forbid
mutating process-global state, so the table has to belong to whatever hands out a thread context --
one object, one address, owned by interpreter state rather than by a global. `REFUSING` stays as its
initial value, and a comment at the site states why there is no `static` beside it.

So: half of the finding is applied as asked, and half is a declared deviation with a reason. If you
want the thread table to be a `static` anyway, the cost is a second `unsafe impl Sync` site.

## Finding 2: the comment and the commit message overclaim

Correct, and the falsifying names are the ones the reviewer lists. The commit message cannot be
changed and the record of the correction is here. The comment now reads:

```rust
// The `allow` is for the frozen header's own spellings (D5): the interface
// tables, the context structs, `RexxCondition` and `ValueDescriptor` carry the
// member names `api/oorexxapi.h` gives them. The entry structs moved here from
// `load.rs` and keep the Rust spellings they arrived with.
```

which is narrow enough to be true. The object handles are a third case and neither sentence covers
them: the header's incomplete type is `_RexxObjectPtr` and this module's is `RexxObjectPtr_`, and
the per-handle doc comment names both spellings rather than claiming they match.

## Finding 3: the negative control's pass count

Correct. **The "16 passed; 1 failed" figure was taken before the last test in the file existed.**
The control was run, and its prediction written, at a point where `tests/layout.rs` held 17 tests;
`a_table_outside_the_slice_refuses_where_it_would_be_handed_out` was appended afterwards, in
response to nothing but my own second thought that the four refusing accessors had no witness. The
figure was then carried into the report without being re-taken. Part 4 of that control should read
**17 passed, 1 failed**.

Re-run here on the fix-round tree, with the prediction written to
`<scratchpad>/prediction-fix1.md` first:

*Predicted:* `test result: FAILED. 17 passed; 1 failed`, the failure being
`a_method_context_has_a_thread_a_table_and_arguments` at `crates/rexx-api/tests/layout.rs:251`
with `left: 16, right: 8`, and the lib target still `2 passed`.

*Outcome:* **confirmed in every part.** `test result: FAILED. 17 passed; 1 failed`, the panic at
`crates/rexx-api/tests/layout.rs:251:5` reading `left: 16, right: 8`, and
`test result: ok. 2 passed` for the lib target. Reverted by re-editing the file, and the
declaration was read back on screen afterwards.

## Finding 4

Recorded, no action. `instance_interface()` keeps `(Phase 8)`.

## Commands and exit statuses

Run from `rust/`, each status read unpiped, on the tree as committed.

| command | exit | note |
|---|---|---|
| `cargo fmt --all` | 0 | |
| `cargo fmt --all --check` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | warm target directory |
| `cargo test -p rexx-api` | 0 | 32 `ok` lines, unchanged |
| `cargo test -p rexx-core --test unsafe_sites` | 0 | 2 `ok` lines |
