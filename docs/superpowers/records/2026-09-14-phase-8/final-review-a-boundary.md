# Final review, slice A: the `rexx-api` crate as a boundary

Range: `659312de0..e64202ae7`. Reviewer is read-only; scratch under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-a/`.
Every finding says whether it was run or inferred.

Sections appear in the order they were established, not the brief's order; the findings table at the end is by severity.

(appended 14:20) Scratch tree: `final-a/tree/rust` is a `tar` copy of `rust/` (no `target/`) with
`api`, `build`, `interpreter`, `extensions` symlinked beside it so the tests' `../../../api` paths
resolve; `CARGO_TARGET_DIR=final-a/target`, `-j 4`, `--locked`. Baseline there, run:
`cargo test -p rexx-api --locked -j 4 --no-fail-fast` -> exit 0; per binary: lib 10, context 14,
handles 5, invoke 10, layout 18, load 10, values 36 passed, 0 failed (plus doctests 2 passed).

## 2. Layout vs header (measured)

**2.1 Offsets, sizes, alignments: identical to g++'s reading of the frozen header.**
`final-a/offsets.cpp` includes `api/oorexxapi.h` as C++ (`g++ 16.2.0 -std=gnu++11
-Wno-invalid-offsetof -I api -I api/platform/unix`) and prints `sizeof`/`alignof`/`offsetof` for
every `#[repr(C)]` struct the crate declares, the two `Owned` wrapper shapes, the union members
the crate reads, and the version defines. `tree/rust/crates/rexx-api/tests/zz_offsets_probe.rs`
prints the same 111 lines from `size_of`/`align_of`/`offset_of!`.
`diff offsets.cpp.txt offsets.rs.txt` -> no output, `OFFSETS-IDENTICAL`. Run.
Covers: `RexxInstance_`, `RexxThreadContext_`, `RexxMethodContext_`, `RexxCallContext_`,
`RexxExitContext_`, `RexxIORedirectorContext_`, `ValueDescriptor` (value 0, type 8, flags 10,
size 16), `RexxCondition` (size 72, nine members), `RexxPackageEntry` (size 64), `RexxMethodEntry`
and `RexxRoutineEntry` (size 32), the six interface tables (size = members x 8; first, last, and
the L2 slice's populated slots and the four object members at their C offsets), `Owned<RexxMethodContext_,_>.owner`
= 24 = `offsetof(MethodContext, context)`, `Owned<RexxThreadContext_,_>.owner` = 16, and
`REXX_CURRENT_INTERPRETER_VERSION` = 328448 (0x00050300) on both sides.

**2.2 Every function-pointer slot's signature, not only its name, matches the header.**
`tests/layout.rs::every_interface_declares_the_members_the_header_does` compares member *names*
in order, so it catches a swapped pair of adjacent slots (the names swap with them) but is blind
to a slot whose argument list or return type is wrong: every slot is one word wide, so the size
test cannot see it either. `final-a/sigcheck.py` parses each `(RexxEntry *Name)(args)` line of the
six header structs and each `call(...) -> ret` of `layout.rs`'s `interface!` bodies, maps C types
to the crate's spellings, and compares (name, kind, args, return) per slot:
```
RexxInstanceInterface: 8 members compared
RexxThreadInterface: 147 members compared
MethodContextInterface: 27 members compared
CallContextInterface: 22 members compared
ExitContextInterface: 12 members compared
IORedirectorInterface: 12 members compared
mismatches: 0
```
Run. (A first run reported 7 mismatches, all the parser's: those header lines carry parameter
names, `CSTRING d`, `size_t count`; stripping a trailing identifier removed them.)

**Answer to the brief's question:** yes, `tests/layout.rs` would redden on a swapped pair of
same-typed adjacent slots (M-none needed: the names vector is order-sensitive). What it would not
catch is a slot with the right name and the wrong signature; nothing in the crate's tests would.
Minor, see findings.

**2.3 Version fields.** `every_interface_version_is_the_frozen_headers` reads the six `#define`s
out of the header; `the_populated_tables_carry_the_interface_version` pins `REFUSING` and the
static. `Contexts::new` starts from `REFUSING`, so the handed-out thread table carries 103 and the
method table 102, the header's values. Read, and the C++ print of `versions 101 103 102 101 101
100` matches `layout.rs:165-175`.

## 5. Instruments that cannot fail: predictions (written before the runs)

Harness: `final-a/mutate.sh`, one mutation at a time in the scratch copy, full
`cargo test -p rexx-api --no-fail-fast`, then the file restored from the worktree.

| id | mutation | predicted red |
|---|---|---|
| M1 | `Table::clear` body removed | `handles::remove_drops_one_registration_and_clear_drops_them_all`; `values::a_handle_the_table_dropped_does_not_convert_back`. `handles::a_handle_that_outlived_its_activation_misses...` stays green (it resolves through a *second* table). Note: `rexx-exec` never calls `clear`; it drops the frame. |
| M2 | `descriptor()` keeps the optional bit in `type` | `values::a_descriptor_carries_the_stripped_code_and_the_flags` only; `rxregexp`'s stubs never read `type`. |
| M3 | `to_native` flags for an argument row = 0 | `values::the_cstring_row_points_at_a_copy_of_the_argument`, `values::the_string_object_row_round_trips_through_a_handle`, `values::a_descriptor_carries_the_stripped_code_and_the_flags`, `invoke(unit)::the_call_publishes_its_array_on_the_context`. `an_optional_cstring_that_is_supplied_converts_like_a_required_one` stays green (compares two zeros). `rxregexp` does not use `argumentExists` (grep of `extensions/rxregexp/rxregexp.cpp`: no hit), so every integration test stays green. |
| M4 | `Contexts::new` leaves the four object members null | `context::the_thread_table_carries_the_four_constant_objects` only. |
| M5 | `signature()` ignores `limit` | `invoke(unit)::a_signature_longer_than_the_array_is_refused_and_not_called`, as a panic (index 16 of a 16-element array) rather than an assertion. |
| M6 | `ffi::raise_exception0` drops the number | `context::raise_exception0_records_the_condition_and_the_call_finishes` only; `tests/invoke.rs` wires its own callbacks. |
| M7 | `address_of` masks the generation out (keeps tag+slot) | `handles::a_handle_that_outlived_its_activation_misses_the_slots_next_occupant` only; the six extremes stay distinct after the mask (worked by hand). |

## 1. Soundness of `unsafe` blocks (detail)

Every `unsafe` block in `ffi.rs` and `load.rs` was read against its `SAFETY:` note and against the
safe callers that reach it (`invoke.rs`, `values.rs`, `tests/*.rs`, and
`rexx-exec/src/dispatch/library.rs:64-108`, the one interpreter site). Miri could not be installed
(`rustup component add miri` fails: the download cache is on a read-only filesystem), so the
aliasing-model item below is argued, not run; everything else in this section was run.

**1.1 `ffi::value_of` is a safe `pub fn` whose precondition nothing enforces (Important, run).**
`ffi.rs:52-79` reads the union member `repr` names. The note says the member read "is the member
that was written" because "the caller derives `repr` from it through the table". The function's
type does not say so: `ValueDescriptor` and `ValueUnion` are `pub` with `pub` fields, and
`values::descriptor(declared, converted)` (`values.rs:247-253`) writes whichever member the
`Value` variant names without checking it against `declared`. So safe code can write a one-byte
member and read the eight-byte one, and the seven bytes it reads were never initialised:
```
// final-a/tree/rust/crates/rexx-api/tests/zz_uninit_probe.rs, under #![forbid(unsafe_code)]
let d = descriptor(code::INT64_T, Converted { value: Value::Int8(1), flags: 0 });
let v = value_of(&d, repr(code::INT64_T).expect("a row"));   // reads value_int64_t
let d = ValueDescriptor { value: ValueUnion { value_float: 1.5 }, r#type: 0, flags: 0 };
let v = value_of(&d, Repr::Double);                            // reads value_double
```
Both compile under `#![forbid(unsafe_code)]` and run (output in `final-a/uninit-probe.txt`). A
union literal initialises only the named member's bytes; reading an integer or float from bytes
that are not initialised is undefined behaviour (Rust Reference, "Behavior considered undefined":
"an integer, floating point value ... obtained from uninitialized memory"). Nothing in the tree
reaches it today: `invoke::method` writes every element with `Value::Omitted` (eight zero bytes)
and the three filled `to_native` rows all write eight-byte members, and element zero is read
through the same `Repr` the stub wrote through. The hole is the API, not the current path.
Fix shape: make `Value::as_union` start from `ValueUnion { value_int64_t: 0 }` and assign the
member through it (which is what `processArguments` does at `NativeActivation.cpp:229`: it zeroes
the word before the narrow write), so every union it produces is fully initialised whatever
member is read; and either make `value_of` `unsafe fn` with the precondition in its signature, or
have it take the `Repr` from the descriptor's own `type` rather than from the caller.

**1.2 `owner_of` reads outside the range of the `&mut` it is derived from (Important, inferred).**
`Contexts::method` (`ffi.rs:150-155`) returns `&mut self.method.context`, a reference to the
24-byte `RexxMethodContext_` field. `invoke::method` and `NativeMethodEntry::call` carry it as
`context: &mut RexxMethodContext_` and hand the stub `&raw mut *context` (`load.rs:118`, `:157`).
The stub passes that pointer back to `ffi::set_object_variable`, which casts it to
`*mut Owned<RexxMethodContext_, Activation>` and reads `.owner` at offset 24 (`ffi.rs:39-45`), i.e.
past the end of the place the reference was created for. The C++ does the same cast
(`Activity.hpp:503`) but has no aliasing model. Under Stacked Borrows (Miri's default) a reborrow
`&mut self.method.context` pushes its tag on exactly those 24 bytes, and a read at offset 24 with a
pointer derived from it is a use of a tag the location's stack does not hold: UB as Miri reports
it. Under Tree Borrows the same access is allowed for an unprotected reference but the reference
here is a function argument (protected for the call). The Reference's own "dangling" definition
does not reach it (the bytes are inside the same live `Contexts` allocation), so this is the
aliasing model, which is not yet normative; the unit test `a_context_we_handed_out_recovers_its_owner`
(`ffi.rs:331-349`) has the same shape with `&raw mut wrapper.context` on a local, and could not
tell either way without Miri. The `SAFETY:` note names the wrong invariant: "the cast is the
identity on the address and `owner` is in bounds" is about the address, and the concern is
provenance. Fix shape: `Contexts::method` returns `*mut RexxMethodContext_` derived from
`&raw mut self.method` (the whole `Owned`) and `invoke::method`/`call` take a raw pointer; the
layout test already pins `offset_of!(Owned<...>, context) == 0`. Two callers change
(`tests/context.rs:245-246`, `dispatch/library.rs:92-93`); the tests that build a bare
`RexxMethodContext_` (`src/invoke.rs::method_context`, `tests/invoke.rs::with_context`) are
unaffected because their callbacks never call `owner_of`.

**1.3 A thread context does not outlive the call, and a conforming extension can hold one
(Important, run).** `ffi.rs:94-107`: `Contexts` owns the `RexxThreadContext_` and its table, and
lives for one `invoke::method`. In the oracle the thread context is the `Activity`'s
(`ActivationApiContexts.hpp:71-75`, `Activity::getThreadContext`), valid for the attached thread,
and `RexxPackageLoader` is even handed one with no method call in flight (`oorexxapi.h:259`).
`final-a/ext/forge.cpp` stores `context->threadContext` in a static on one call and calls
`WholeNumberToObject` through it on a later call:
```
b2_stash_deep.rex   (stash at Rexx depth 40, use at depth 0): oracle rc 0 "x 42"; rust rc 0 "x 42"
b3_stash_shallow_use_deep.rex (stash at 0, use at 40):        oracle rc 0 "x 42"; rust rc 134:
   thread 'rexx-interp' panicked at crates/rexx-api/src/values.rs:449:25: RefCell already borrowed
   panicked at library/core/src/panicking.rs:225:5: panic in a function that cannot unwind
```
The `b2` agreement is the accident of the second `Contexts` landing at the first one's stack
address; `b3` is the same read landing on something else. Both are use-after-free of a
`run_library_method` local; the abort is one of its outcomes, not a refusal. `rxregexp` never
does this, so the L2 gate cannot see it. An ASan build of `rexx-run` was queued to confirm the
read is stack-use-after-return (result below if it finished). Fix shape is a design change, not a
patch: the thread context and its table have to belong to something with the interpreter's
lifetime (the four object members are already stable across calls, since a handle is a pure
function of the `ObjRef` and the constants are tag-encoded or immortal), and the callbacks then
resolve the *current* native frame from the owner, which is what `contextToActivation` does for a
thread context in the C++.

**1.4 The rest, read and found sound under their notes (run where a test reaches them).**
`load::dlopen` (contract-level, D-U1); `package_of`'s `dlsym` at the header's signature, the
`&*entry` deref while `handle` lives, and the table walks to a zero-`style` row (`load.rs:380-437`,
`:459-517`; `tests/load.rs` runs all of it against `librxregexp.so`); `signature`'s bounded read
(`load.rs:106-137`, M5 below reddens when the bound goes); `call`'s publish/unpublish of the
array (`load.rs:144-159`); `stub`'s transmute (same width, header-declared type); `name_of` and
`c_bytes` (NUL-terminated by the macros that stringise the names). Field drop order in
`Library` puts `handle` last, so no row outlives the mapping. `activation_of`'s unbounded `'a` is
consumed within the callback and never escapes. `RefCell` reentry panics rather than aliasing
(`values.rs:443-450`), which is the designed refusal; no L2 entry re-enters.

## 3. Two-call protocol and the conversion table vs `NativeActivation` (run)

`final-a/ext/libforge.so` (built with g++ against the frozen header) plus `final-a/probes/*.rex`,
run on both interpreters from fresh empty directories, three descriptors. Oracle wrapper as the
constraints give it; runner `final-a/target/release/rexx-run` with `LD_LIBRARY_PATH=final-a/ext`.

| probe | oracle | rust | verdict |
|---|---|---|---|
| unknown code 9, non-optional, no argument | `88.901` | `93.968` | **diverges** (Minor: forged signature only) |
| unknown code 9, argument supplied | `93.968` | `93.968` | agrees |
| unknown code 9 optional, with/without argument | `93.968` | `93.968` | agrees |
| return type `OPTIONAL|int` | `93.968` | `7` | **diverges** (Minor: forged signature only) |
| `argumentExists(1)` on omitted / `""` / `"x"` | `0 1 1` | `0 1 1` | agrees (flags and array publish) |
| `StringLength` of `"hello"`, of `12345` | `5 5` | `5 5` | agrees |
| `RexxNullString` returned as `RexxStringObject` | `[]` | `[]` | agrees (constants + `object_from_native`) |
| `StringData` twice on one object, same address? | `1` | `1` | agrees (`intern_for`) |
| `SetObjectVariable("Y", NULLOBJECT)` after `y = 5` | `Y` | `Y` | agrees: storing `OREF_NULL` unsets, which `values.rs:471-473` claims |
| required `RexxStringObject` missing / with no string value | `88.901` / `88.909` | `88.901` / `3` | **diverges**, see 3.3 |
| `int8_t` return; `ARGLIST` parameter | `-5`; `3` | rc 120, loud `Phase 8 owes ...` | unfilled rows refuse loudly, as designed |

**3.1 Unknown code with a missing argument.** `processArguments`'s outer `default:` checks
argument presence before the type (`NativeActivation.cpp:327`, `:607-612`): absent and not optional
is `Error_Invalid_argument_noarg` before the absent-switch's `default: reportSignatureError()`.
`invoke::method` answers `Failure::Signature` from `consumes_argument` (`invoke.rs:75`) before
`to_native` is reached, and `to_native` itself puts `row(code)` before the presence check
(`values.rs:923-926`). `tests/values.rs:399-415` pins the crate's order with the message "an
unknown code must not be read as a missing argument", which is the opposite of the oracle's order
for the non-optional case (measured above: 88.901). Reachable only through a hand-written type
array, so Minor; but the test asserts the divergence as the rule.

**3.2 A return type carrying the optional bit.** `processArguments` stores the raw first word
(`descriptors[0].type = *argumentTypes`, `:228`) and `valueToObject` switches on it unstripped
(`:718-861`), so `0x800C` is its `default:` (93.968). The crate strips it three times
(`values::descriptor` at `invoke.rs:62`, `values::repr` at `:99`, `from_native` at `:970`) and
converts. Same reachability as 3.1, Minor.

**3.3 `Host::string_value` accepts what `requiredString` refuses (Important, run; the code is
slice B's).** `values.rs:347-352` names the reference correctly: `RexxInternalObject::requiredString`
(`ObjectClass.cpp:1341-1361`) sends `REQUEST('STRING')` to a non-base-class object and treats
`.nil` as no string value; `stringArgument`/`cstring` then raise 88.909. The impl at
`rexx-exec/src/dispatch/library.rs:142-148` calls `required_string_value(object).ok()`
(`dispatch.rs:3027`), which is the operator/BIF coercion that falls back to the object's default
name. Measured, `e_stringvalue.rex`, both sides rc 0:
```
                    oracle            rust
len(.f~new)         step 3 syntax 88.909   3      ("a F")
len(.object~new)    step 4 syntax 88.909   9      ("an Object")
len(.nil)           step 6 syntax 88.909   14     ("The NIL object")
exists(.f~new)      step 9 syntax 88.909   1      (OPTIONAL_CSTRING, same path)
len(.g~new)         4                      4      (a class with MAKESTRING: both convert)
length(.f~new)      3                      3      (the BIF path, where the default name is right)
```
So every `CSTRING`, `OPTIONAL_CSTRING` and `RexxStringObject` parameter of every native method
silently converts an arbitrary object to its default name here where the oracle raises. Not
`rxregexp`-visible, since its callers pass strings. The crate-side test
`a_required_cstring_with_no_string_value_is_88_909` passes because its stand-in host answers
`None`; the contract is tested, the implementation is not.

**3.4 Everything else compared, agrees by reading and by the probes above:** the signature call
first with a null array; the array published on the context for the call and cleared after
(`argumentExists` reads it, probe `exists`); `outputIndex >= MaxNativeArguments` as a bound of
15 parameters (`signature(entry, 17)` reads at most 16 words + terminator; `LONGEST`/`TOO_LONG`
tests); `inputIndex` counting only consumed arguments and the one-based position in errors;
`Error_Invalid_argument_maxarg` with the consumed count (88.922 probe in `tests/invoke.rs`);
optional-absent zeroing via `value_int64_t = 0` (`Value::Omitted`); `RexxVariableReferenceObject`
absent as a signature error (not in the absent switch, `:617-648`); specials with `!isMethod()`
refused (CSELF filled, the others stubbed); `valueToObject`'s `case 0` as no result; `CSTRING`
result `NULL` as no result (row unfilled here). `usedArglist` (`:680`) is not modelled: once the
`ARGLIST` row is filled, the too-many check at `invoke.rs:90` has to be skipped for it; today the
row refuses loudly before that check is reached (probe `d`). The special codes as *return* types
are the oracle's `default:` (93.968) and here `Unfilled` (loud), so the second half should answer
`Signature` for them rather than fill them.

**3.5 No second code-keyed `match` exists.** `grep -rn 'code::' rust/crates --include='*.rs'`
outside `values.rs` and `tests/` hits only test-only type arrays (`ffi.rs:288-289`,
`src/invoke.rs:188-206`); `ffi::value_of` switches on `Repr`, which the table supplies. The
row-deletion control (`the_table_has_a_row_for_every_code_the_header_defines`,
`exactly_the_filled_rows_convert`) therefore sees every switch there is.

## 4. Handle model (read, with the tests run)

A handle is `ObjRef::bits() + 1` (`handles.rs:77-84`), 64 bits of tag, slot, and a 30-bit
generation (`rexx-core/src/handle.rs:14-40`), resolved through the activation's own `HashMap`.
* **Wrong object from a stale handle:** needs two `ObjRef`s with equal bits naming different
  objects. The heap increments the generation on every free and *retires* a slot at
  `GENERATION_MAX` instead of wrapping (`heap.rs:233-247`), so equal bits are the same object.
  Within one activation the table's entries are roots (`rexx-exec/src/lib.rs:5220`), so nothing
  it holds can be freed under it. No path found; `tests/handles.rs:82-118` runs the slot-reuse
  case and M7 below reddens it when the generation is dropped.
* **Table cleared while a nested activation holds handles:** `locals()` is the top
  `NativeFrame`, pushed and popped around one call (`library.rs:81-96`); no L2 entry re-enters,
  so there is no nested native frame. When the second half adds `SendMessage`, a handle minted in
  frame N and used in frame N+1 is a miss here and a valid pointer in the oracle (rooted by frame
  N's `saveList`); worth a line in that spec.
* **`NULLOBJECT`:** `resolve(null)` is `entries.get(&0)`, which the `+1` bias keeps empty
  (`the_extremes_of_the_handle_space_are_not_null` runs the six edge encodings). Paths:
  `SetObjectVariable(null)` unsets (measured equal to the oracle, table above);
  `from_native(Object(null))` is `Ok(None)` = `OREF_NULL`; `StringData(null)` answers `NULL` and
  `StringLength(null)` 0 where the oracle would dereference null. Every header-permitted null
  path in the L2 slice is handled.
* **Handles are identity across activations:** the same object gives the same bits in every
  table, so pointer equality in an extension means object identity, as in the oracle; and the
  four constants' handles are stable across calls (relevant to 1.3).

**1.3, continued: the ASan reading (run).** `rexx-run` rebuilt with
`RUSTFLAGS=-Zsanitizer=address cargo +nightly build --release --target x86_64-unknown-linux-gnu`
in `final-a/target-asan`, run with `ASAN_OPTIONS=detect_stack_use_after_return=1` and no
`ulimit -v` (ASan cannot reserve its shadow range under one):
```
b3_stash_shallow_use_deep: ERROR: AddressSanitizer: stack-use-after-return ... READ of size 8
    #0 rexx_api::ffi::whole_number_to_object      (rexx-run+0x2d073a)
    #1 #2 libforge.so                              (Forge_UseStash and its stub)
    #3 rexx_api::invoke::method                   (rexx-run+0x2d4121)
    #4 <rexx_exec::Interp>::invoke
  Address ... is located in stack of thread T1 at offset 4080 in frame <rexx_exec::Interp>::invoke, into which run_library_method and its Contexts local are inlined (final-a/asan-frames.txt)
b2_stash_deep:            the same report (the "x 42" agreement in the plain build was luck)
a_table:                  rc 0, no ASan report (control: the ordinary path is clean)
```
Full reports in `final-a/run-rust/asan-*.err`. The read of size 8 is `owner_of`'s `(*owned).owner`
through the stale thread-context pointer. So 1.3 is run in both directions, and the plain
build's `b2` agreement is retired as evidence of anything.

## 5. Instruments that cannot fail: results (run)

`final-a/mutations.txt`; each line is the full `cargo test -p rexx-api --no-fail-fast` under one
mutation, file restored from the worktree afterwards.

| id | predicted | observed | match |
|---|---|---|---|
| M1 clear no-op | 2: `handles::remove_drops_one...`, `values::a_handle_the_table_dropped...` | those 2 (`handles.rs:137`, `values.rs:848`) | yes |
| M2 optional bit kept | 1: `values::a_descriptor_carries_the_stripped_code...` | that 1 (`values.rs:947`) | yes |
| M3 argument flags 0 | 4 | those 4: `values.rs:456`, `:735`, `:952`, `src/invoke.rs:473` | yes |
| M4 constants unset | 1: `context::the_thread_table_carries_the_four...` | that 1 (`context.rs:478`) | yes |
| M5 signature unbounded | 1, as a panic | `invoke::tests::a_signature_longer...` panicked at `src/invoke.rs:85` (index out of bounds) | yes |
| M6 raise dropped | 1: `context::raise_exception0_records...` | that 1 (`context.rs:440`) | yes |
| M7 generation masked | 1: `handles::a_handle_that_outlived...` | that 1 (`handles.rs:116`) | yes |

Every mechanism defeated reddened at least one test, and none reddened a test outside its
prediction, so the predictions were not vacuous either. What the table says about the
instruments: the `type` word (M2), the four constants (M4), the raise wiring (M6) and the
generation (M7) are each pinned by exactly one test, and the integration tests against
`librxregexp.so` see none of M2, M3, M4 or M6 because `rxregexp` reads neither `type` nor
`flags` nor the constants and `tests/invoke.rs` supplies its own callbacks. That is not a
defect (the unit tests are where those belong) but it is the shape the brief asked about: the
"real extension" tests would stay green through a table that publishes no flags.

**Tests that cannot fail, looked for and not found.** Each of the crate's 103 tests was read
for a degenerate implementation that satisfies it: the handles tests carry their positive
control (`a_live_handle_resolves...`) and the slot-reuse test asserts both that the slot *was*
reused and that the generation differs; the two collection tests in `context.rs` assert that
the collection reclaimed something before asserting survival, and each has its "not a root"
control; `the_scan_reads_one_struct_at_a_time` is the negative control for the header scan;
`a_library_without_the_exporter_is_not_an_error` asserts `loads()` first so its `Ok(None)` is
the second kind; `every_platform_alias_names_a_code_the_table_has` and
`every_optional_name_is_a_base_code_with_the_optional_bit` assert their scan found something.
One instrument is weaker than its name: `tests/invoke.rs::every_repr_reads_back_the_member_the_table_wrote`
writes and reads through the same `Repr`, so it cannot see 1.1 (a mismatch between the two).

## Findings by severity

**Critical:** none.

**Important**
1. `rust/crates/rexx-api/src/ffi.rs:94-107`, `:100-166` (`Contexts`): the thread context and its
   table live for one call; the oracle's live for the thread. An extension that keeps
   `context->threadContext` (a conforming pattern; the package loader is handed one with no
   call in flight) reads a freed stack frame on its next use. Run: `b3_stash_shallow_use_deep`
   aborts rc 134 (`RefCell already borrowed` inside an `extern "C"` frame) where the oracle prints
   `x 42`; ASan names it `stack-use-after-return` in `ffi::whole_number_to_object` for both call
   orderings. Section 1.3.
2. `rust/crates/rexx-exec/src/dispatch/library.rs:142-148` (`Host::string_value`, slice B's file,
   found through the boundary's contract at `values.rs:347-352`): converts a non-string object to
   its default name where `requiredString` raises 88.909. Run: `len(.f~new)` answers `3` here,
   `88.909` on the oracle; same for `.object~new` (`9`), `.nil` (`14`), and an `OPTIONAL_CSTRING`
   parameter. Silent, rc 0, on every string-typed parameter of every native method. Section 3.3.
3. `rust/crates/rexx-api/src/ffi.rs:52-79` (`value_of`): a safe `pub fn` that reads a union
   member chosen by its caller, with `descriptor()`/`ValueUnion` letting safe code write a
   narrower one. Run: under `#![forbid(unsafe_code)]`, `value_of(&descriptor(INT64_T, Int8(1)),
   Repr::Int64)` compiles and reads seven uninitialised bytes; the `value_float`/`Repr::Double`
   variant printed `6.9483158606053e-310`. Unreached by the tree's callers today. Section 1.1.
4. `rust/crates/rexx-api/src/ffi.rs:39-45` with `:150-155` (`owner_of` through
   `Contexts::method`'s `&mut self.method.context`): the owner field is read at offset 24
   through a pointer whose reborrow covered 24 bytes; Stacked Borrows rejects it, Miri could not
   be installed to confirm, the Reference's definitions do not reach it. Inferred. The
   `SAFETY:` note argues address, not provenance. Section 1.2.

**Minor**
5. `rust/crates/rexx-api/src/invoke.rs:75` and `values.rs:923-926`, pinned by
   `tests/values.rs:399-415`: an unknown non-optional code with no argument is `Signature`
   (93.968) here and `Error_Invalid_argument_noarg` (88.901) in the oracle, which checks presence
   first (`NativeActivation.cpp:607-612`). Run. The test's message asserts the crate's order as
   the rule. Forged signatures only.
6. `rust/crates/rexx-api/src/invoke.rs:62`, `:99`, `values.rs:970`: the return type is
   stripped of the optional bit; `valueToObject` switches on it raw and answers 93.968. Run:
   `retopt()` prints `7` here. Forged signatures only.
7. `rust/crates/rexx-api/tests/layout.rs`: no test compares a slot's *signature* against the
   header, only its name and one-word size; `final-a/sigcheck.py` does and finds 0 mismatches
   across the six tables, and could be ported as a test. Run.
8. `rust/crates/rexx-api/src/values.rs:620-886`: the six special codes as return types answer
   `Unfilled` (loud) where the oracle's answer is already fixed at `Signature` (`valueToObject`
   has no case for them); and `usedArglist` (`NativeActivation.cpp:680`) has no counterpart at
   `invoke.rs:90`, which the `ARGLIST` row's filling will have to add. Read. For the second
   half's spec.
9. `rust/crates/rexx-api/src/invoke.rs:419-424` (docstring): "the oracle builds its context
   before the signature call too (`NativeActivation.cpp:1289-1291`)" is true and elides that
   the oracle also *publishes* the (uninitialised) array on the context before that call
   (`:1291`), which is what the test beside it asserts the crate does not do. Not observable by
   a conforming stub. Read.

## Not reached

* `tests/load.rs`'s name search against a directory list with more than one entry, and
  `MAX_LIBRARY_NAME_LENGTH`'s 250 against `SysLibrary.cpp` (cited, not re-read).
* The `compile_fail` doctests in `load.rs` were run by the baseline (2 passed) but not read for
  whether they fail for the intended reason.
* `Failure::error_number`'s 88.901/88.909/88.922 were measured by the tests' own probes and by
  `a_table.rex` here; 40.918 (the call-context signature error) was not exercised, since no call
  context exists in the slice.
* Miri under either aliasing model (not installable here); MSan for 1.1.
* `rexx-exec`'s rooting of `native_handles`, `Interp::object_roots`, the `Rc<Library>` drop
  order against the finaliser sweep, and `condition_of`'s rendering of a raised number: slice B.
* The prose documents (spec, plan, ledger, gate document): slice C, except the two citations
  above (`values.rs:347-352`, `invoke.rs:419-424`).
* The union's member *list* against the header's (all members alias one word; the crate reads
  only through `Repr`, whose members were sized against g++ in 2.1), and the four handle
  typedef families beyond `RexxObjectPtr`/`RexxStringObject`/`RexxPointerObject`.
* Whether a `RexxPackageLoader` hook, if the second half calls it, can be handed a thread
  context at all under the current `Contexts` design (follows from finding 1; not tried).
