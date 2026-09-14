# Task 6 report: the two-call protocol

Files added: `rust/crates/rexx-api/src/invoke.rs`, `rust/crates/rexx-api/tests/invoke.rs`.
Files modified: `rust/crates/rexx-api/src/load.rs`, `rust/crates/rexx-api/src/ffi.rs`,
`rust/crates/rexx-api/src/values.rs`, `rust/crates/rexx-api/src/lib.rs`.
`rexx-exec` was not touched.

## The API

```rust
// invoke.rs
pub const MAX_NATIVE_ARGUMENTS: usize = 16;

pub fn signature(
    entry: &NativeMethodEntry,
    context: &mut RexxMethodContext_,
) -> Result<Vec<u16>, Failure>;

pub fn method(
    entry: &NativeMethodEntry,
    context: &mut RexxMethodContext_,
    cx: &mut Conversion<'_>,
    arguments: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure>;

// ffi.rs
pub fn value_of(descriptor: &ValueDescriptor, repr: Repr) -> Value;

// load.rs
pub(crate) type NativeMethod =
    unsafe extern "C" fn(*mut RexxMethodContext_, *mut ValueDescriptor) -> *mut u16;

impl NativeMethodEntry {
    pub(crate) fn signature(&self, context: &mut RexxMethodContext_, limit: usize)
        -> Option<Vec<u16>>;
    pub(crate) fn call(&self, context: &mut RexxMethodContext_,
                       arguments: &mut [ValueDescriptor]);
    fn stub(&self) -> Option<NativeMethod>;
}

#[cfg(test)]
pub(crate) fn stub_entry(name: &[u8], stub: NativeMethod) -> NativeMethodEntry;
```

### Four things the interfaces say that the brief did not

1. **The context arrives as `&mut RexxMethodContext_`, not as a raw pointer.** `unsafe` is
   permitted only in `ffi.rs` and `load.rs`, and the workspace's `unsafe_code = "deny"` covers an
   `unsafe fn` *declaration* as well as a block, so `invoke::method` cannot be `unsafe` and cannot
   take a pointer whose validity is the caller's promise. A `&mut` carries that promise in the type,
   and writing `context.arguments` then needs no `unsafe` at all. Task 7 holds an
   `Owned<RexxMethodContext_, Activation>` and passes `&mut owned.context`.
2. **`Result<Option<ObjRef>, Failure>`, not `Result<ObjRef, Failure>`.** `valueToObject` answers
   `OREF_NULL` for a null object handle and for a `type` of zero (`NativeActivation.cpp:848-853`),
   and Task 5's `from_native` already answers `Option`.
3. **`arguments: &[Option<ObjRef>]`.** `processArguments` tests `_arglist[inputIndex] != OREF_NULL`
   (`NativeActivation.cpp:327`), so an omitted middle argument is a position in the list, not a
   short list.
4. **No accessor handing out the entry-point address was added.** The dispatch asked for a
   `pub(crate)` accessor; what is there instead is `pub(crate)` *methods that call it* --
   `NativeMethodEntry::signature` and `::call` -- with a private `stub()` doing the transmute. The
   address never leaves `load.rs`, so the two `compile_fail` doctests keep their whole meaning
   rather than most of it. This is stricter than what was asked for; say so if it is the wrong
   trade.

## The `unsafe` blocks and what establishes each

`crates/rexx-core/tests/unsafe_sites.rs` passes unchanged: the granted set is still
`ffi.rs`, `load.rs`, `rexx-core/src/lib.rs`, and the using set is still `ffi.rs`, `load.rs`,
`rexx-core/src/bytes.rs`. **No third file was needed**, and neither `invoke.rs` nor either test
file says `unsafe`.

| site | invariant | who establishes it |
|---|---|---|
| `load::NativeMethodEntry::stub`, the `transmute` | the address is the stub the `RexxMethodN` macro generated, whose C signature is `NativeMethod`'s | `REXX_METHOD_ENTRY` fills `entryPoint` from the macro (`api/oorexxapi.h:223`, `:4286`) and D5 freezes the header both sides compile against; the null case is checked first |
| `load::signature`, the call | the code stays mapped for the call | the row is borrowed from the `Library` that owns the mapping; a null `arguments` is the signature request and runs none of the extension's own code (`api/oorexxapi.h:4342-4350`) |
| `load::signature`, `*types.add(offset)` | the array ends in `REXX_ARGUMENT_TERMINATOR` | the macro emits it (`api/oorexxapi.h:4338`). `limit` bounds how far a malformed array is followed; it is a cap on the damage, not a substitute for the invariant, and the note says so |
| `load::call`, the call | as above, plus: `array` is the caller's live slice and nothing else touches it for the duration | `call` takes `&mut [ValueDescriptor]` and derives the pointer from it immediately before the call, so no reborrow invalidates it |
| `ffi::value_of`, the union read | the member read is the member written | the stub writes through the declared type (`api/oorexxapi.h:4282`) and `repr` is derived from that same type through the table |

**One aliasing detail that shaped the code.** `context.arguments` is set inside `load::call`, from
the same pointer the call is given, rather than in `invoke::method` before calling. Setting it in
`method` would mean creating the pointer, then reborrowing `descriptors` mutably to pass it, which
invalidates the pointer the context holds while the extension is reading through it.

## How the signature call was made observably argument-free

Three properties, each with its own test, all in `src/invoke.rs`'s unit tests against a Rust probe
stub that records every entry into it and a `Host` that records every conversion it is asked for.

* **`the_signature_call_runs_before_any_argument_is_touched`** asserts the *whole event list*, in
  order: the stub is entered with no array, then the table asks the interpreter to convert, then
  the stub is entered with an array. Converting first flips the first two, and passing an array to
  the signature call changes the first. The same test asserts the pool holds one interned `CSTRING`
  afterwards, which is the contrast the property needs: the argument the signature call could not
  see is there for the call that follows.
* **`the_signature_call_publishes_no_array`** calls `invoke::signature` on a context built holding
  a sentinel address and asserts the field still holds it. This is the half that matters for
  `argumentExists` (`api/oorexxapi.h:4276`), which reads the array through the context rather than
  through the parameter, so the parameter being null is not on its own enough.
* **`the_context_holds_no_array_once_the_call_is_over`** asserts the field is null after a
  successful `method`, from the same sentinel start, so the null is this call's doing.

**What no test here covers, stated plainly.** `load::call` *publishes* the array on the context
before the second call, mirroring `NativeActivation.cpp:1291`. Deleting that line reddens nothing:
the probes cannot read the field without `unsafe`, and `rxregexp` never uses `argumentExists`.
The clear afterwards is covered (control D); the publish is not.

**The divergence from the oracle here.** The oracle publishes the array before the *signature*
call, at `:1291`, pointing at an uninitialised stack array. This crate publishes it before the
second call only. Nothing observes the difference: the generated stub returns the type array
without running any extension code when `arguments` is null, so no extension code exists that could
read the field during a signature request.

## The bound, and what the oracle does

`MAX_NATIVE_ARGUMENTS = 16` (`interpreter/execution/NativeActivation.hpp:209`), asserted against
that header at test time by `the_array_is_as_long_as_the_oracles`, which parses the declaration out
of it rather than restating the number.

The oracle checks `outputIndex >= maximumArgumentCount` at the top of each loop iteration
(`NativeActivation.cpp:238-241`) and calls `reportSignatureError`, which is
`Error_Incorrect_method_signature` **93.968** in a method and `Error_Incorrect_call_signature`
**40.918** in a call (`NativeActivation.cpp:190-193`). `outputIndex` starts at 1, so the widest
signature it accepts has fifteen parameters and a sixteenth is refused. Neither number is witnessed
by a run: producing one needs an extension with a malformed signature, which means compiling a new
`.so`, and `librxregexp.so` is not to be rebuilt.

This crate applies the same bound one step earlier, where the unsafe walk over the published array
has to be bounded anyway: `invoke::signature` passes `limit = MAX_NATIVE_ARGUMENTS + 1`, so a
terminator must appear within the return type, fifteen parameters and itself. Sixteen parameters
answer `None`, which `invoke::signature` turns into `Failure::Signature`, whose `error_number` is
the same pair. `a_signature_that_fills_the_array_is_run` and
`a_signature_longer_than_the_array_is_refused_and_not_called` are the adjacent pair.

**The bound lives in exactly one place, and control A shows what that costs.** With the limit
raised by one, the loop writes descriptor 16 of 16 and panics rather than refusing. I kept the
single mechanism rather than adding the oracle's in-loop check as well, because two mechanisms each
sufficient on their own leave a test unable to redden either. The coupling is stated in the comment
at the call site.

## What the oracle does about a signature that is not merely long

* **No terminator at all**: the oracle's `for (; *currentType != REXX_ARGUMENT_TERMINATOR; ...)`
  (`NativeActivation.cpp:235`) walks off the end of the array. This crate stops at `limit` and
  refuses.
* **A null type array**: the oracle dereferences it at `descriptors[0].type = *argumentTypes`
  (`:228`) and crashes. This crate refuses, which
  `a_stub_that_publishes_no_signature_is_refused` pins.

Both are malformed extensions and neither is reachable from `librxregexp.so`.

## Arguments the signature does not consume

`processArguments` ends with `if (inputIndex < _argcount && !usedArglist)` and raises
`Error_Invalid_argument_maxarg` (`NativeActivation.cpp:680-683`), which is **88.922**. Measured
2026-09-14, `.RegularExpression~new('a*','MAXIMAL','extra')` from a fresh directory under the
standard wrapper, three descriptors kept apart:

```
Error 88 running /home/moritz/dev/repos/ooRexx/build/bin/rxregexp.cls:  Invalid argument.
Error 88.922:  Too many arguments in invocation; 2 expected.
```

rc 168, stdout empty. So the insert is `inputIndex`, the number of arguments the signature did
consume, which is what `Failure::TooManyArguments { expected }` carries.

**`usedArglist` is not carried.** It is set only by the `REXX_VALUE_ARGLIST` case
(`NativeActivation.cpp:306-313`), and that row's `to_native` is unfilled, so a signature reaching
the extra-argument check with `usedArglist` true cannot occur here. Writing the flag now would be a
line no test can reach. **Whoever fills the `ARGLIST` row owes this**: an extension that asks for
the argument list must not then be refused for the arguments it did not name individually.

## Ruling 1, witnessed against the oracle's own binary

`an_extension_that_raises_still_finishes_and_still_returns` calls `RegExp_Init` with `[`, which
`automaton::parse` rejects, so the extension calls `RaiseException0(Rexx_Error_Invalid_template)`
(`extensions/rxregexp/rxregexp.cpp:83`, the number 38000 at `api/oorexxerrors.h:362`). The test
asserts all three parts: the raise reached our thread-interface stub, `SetObjectVariable` ran
afterwards so the body did not stop, and `invoke::method` still answered `Ok(Some(0))` from
element zero. There is no unwinding path and no early exit in `invoke::method` between the call and
the conversion of element zero.

## The end-to-end run

`tests/invoke.rs` loads `build/lib/librxregexp.so` by absolute path from the repository root and
never rebuilds it. The context handed to the extension is a `RexxThreadInterface::REFUSING` and a
`MethodContextInterface::REFUSING` with exactly the members `rxregexp` calls replaced by recording
stubs: `NewPointer`, `WholeNumberToObject`, `RaiseException0`, `SetObjectVariable`,
`DropObjectVariable`. Any other member the extension might reach panics, and an `extern "C"` frame
turns that into an abort rather than an unwind into extension code.

| test | what runs |
|---|---|
| `the_extension_publishes_the_signatures_its_source_declares` | `RegExp_Init` answers `[int, OPTIONAL_CSTRING, OPTIONAL_CSTRING]` and `RegExp_Parse` answers `[int, CSELF, CSTRING, OPTIONAL_CSTRING]`, read out of the compiled binary and compared with `rxregexp.cpp:55` and `:105-109` |
| `regexp_init_with_no_arguments_answers_zero_and_allocates_an_automaton` | both optionals omitted; the extension allocates, wraps the pointer through `NewPointer`, stores `CSELF`, answers 0; `RegExp_Uninit` then frees it |
| `regexp_init_with_one_argument_parses_it` | `a*b` parses, nothing is raised |
| `an_extension_that_raises_still_finishes_and_still_returns` | above |
| `a_second_method_reads_the_cself_the_first_produced` | `Init`, then `Parse('a*b')` answering 0, then `Parse('[', 'MINIMAL')` answering **3** -- a non-zero result the extension wrote into element zero -- then `Uninit`. This is the one case exercising `CSELF`, a required `CSTRING`, an optional `CSTRING` supplied, and an optional one omitted, in one chain |
| `a_missing_required_argument_stops_the_call` | `Parse()` refuses with 88.901 at position 1 and the extension never runs, which is visible because `RegExp_Parse` stores `!POS` on every path |
| `an_argument_the_signature_does_not_consume_is_refused` | three arguments to `RegExp_Init`, 88.922, expected 2 |

Each automaton the extension allocates is freed by a `RegExp_Uninit` in the same test, except in
`an_argument_the_signature_does_not_consume_is_refused`, where none is allocated.

## `Value` and `Repr` were extended together

`ffi::value_of` matches on `Repr`, so it has to be total on it, and ruling 4 licenses extending
`Value` to match. Added: `Int8`, `Int16`, `Int32`, `Int64`, `Uint8`, `Uint16`, `Uint32`, `Uint64`,
`Isize`, `Usize`, `Double`, `Float`, each with its `as_union` arm. `Repr::Isize` writes and reads
`value_wholenumber_t` and `Repr::Usize` `value_stringsize_t`, which are the isize and usize members
the header's other spellings alias.

`every_repr_reads_back_the_member_the_table_wrote` runs the round trip over every row: it builds a
value of that row's `Repr`, writes it through `values::descriptor`, reads it back through
`ffi::value_of`, and compares. A mismatched arm changes the variant, so the enum tag alone catches
it, which control E confirms.

**No row's conversion was filled.** `to_native` and `from_native` still refuse for every row Task 5
left unfilled; a `Value` variant is a union member, not a conversion.

## Negative controls

Predictions were written to `scratchpad/predictions.md` at 02:42:09, before the first control ran.
Baseline: `--lib` 9 passed (7 in `invoke`, 2 in `ffi`), `--test invoke` 9 passed. Every control was
an edit to a source file undone by re-editing, and `sha256sum` against a copy taken beforehand
confirmed each restore.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| A | `invoke::signature`'s limit raised by one | `a_signature_longer_than_the_array_is_refused_and_not_called` alone fails, and with an index panic rather than a wrong `Err`; integration all green | exactly that: `panicked at src/invoke.rs:76: index out of bounds: the len is 16 but the index is 16`, lib 8 passed 1 failed, integration 9 passed | **confirmed**, both parts |
| B | the extra-argument check deleted | the same-named test fails in both binaries, nothing else | exactly that, 8 passed 1 failed in each | **confirmed** |
| C (first run) | `load::signature` publishes `dangling_mut()` on the context | `the_signature_call_publishes_no_array` fails | **nothing failed** | **falsified** |
| C (after the fix) | the same edit | as above | `the_signature_call_publishes_no_array` alone fails; integration 9 passed | **confirmed** |
| D | `load::call` no longer clears the context field | `the_context_holds_no_array_once_the_call_is_over` alone fails | exactly that; integration 9 passed | **confirmed** |
| E | `ffi::value_of`'s `Repr::Int` arm reads `value_wholenumber_t` into `Value::Isize` | 3 named lib tests and 6 named integration tests fail, 4 and 3 pass | exactly those sets, lib 6/3, integration 3/6 | **confirmed** |
| F | `load::signature` hands the stub a real zeroed array instead of null | the integration binary crashes on `RegExp_Parse` with a null `CSELF`; **failing that**, 7 named tests fail leaving 2 | no crash; exactly the 7 named failures and the 2 named passes, lib 2 passed 7 failed | first part **falsified**, second part **confirmed** |

**Control C is the one that earned its keep, and it found a test that could not fail.**
`the_signature_call_publishes_no_array` built its context with `std::ptr::dangling_mut()` as the
sentinel, and the control wrote `std::ptr::dangling_mut()` -- the same address, because
`dangling_mut` is the type's alignment. The before-and-after compare was satisfied by a context
field that had been overwritten. The fix is `sentinel()`, an address nothing in this crate
produces, and the comment at it says why `dangling_mut` will not do. The control was re-run after
the fix and reddened.

**Why control F did not crash.** `the_extension_publishes_the_signatures_its_source_declares`
asserts `RegExp_Init`'s signature before asking for `RegExp_Parse`'s, and
`a_second_method_reads_the_cself_the_first_produced` calls `RegExp_Init` first; under F both fail
at that first assertion, so no `RegExp_Parse` implementation ever ran with a null `self`. The
prediction's crash branch was reasoning about a path the tests do not reach in that order.

## Commands and exit statuses

Every status read unpiped, from `rust/`.

| command | exit |
|---|---|
| `cargo fmt --all` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |

`cargo test -p rexx-api` reports 89 `ok` result lines across six test binaries and the doctests: lib 9, `handles` 5,
`invoke` 9, `layout` 18, `load` 9, `values` 36, and 3 doctests over two result lines.
`unsafe_sites` reports 2 passed, so the granted set and the using set are both unchanged.

`cargo test -p rexx-exec --lib` was not run: this task did not touch that crate.

The oracle run for 88.922 used the standard wrapper from a fresh empty directory, three descriptors
kept apart:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/extra.rex" ) >"$D/extra.out" 2>"$D/extra.err"
```

with `::requires '/home/moritz/dev/repos/ooRexx/build/bin/rxregexp.cls'` as the program's last
line. A first attempt put the directive first and answered 99.916, `Unrecognized directive
instruction`.

## What the brief and the dispatch got wrong

1. **`invoke::method(entry, receiver, args)`.** There is no `receiver` parameter. `CSELF` reaches
   the table through `Host::cself` and `OSELF` will reach it the same way, both Task 7's; what
   `method` needs beyond the arguments is the context to hand out and the `Conversion` Task 5
   defined.
2. **`-> Result<ObjRef, Failure>`** should be `Result<Option<ObjRef>, Failure>`, for the reason
   above.
3. **"add a `pub(crate)` accessor" for the entry point.** Not needed, and not done; see the API
   section. The address stays inside `load.rs`.
4. **Step 4's "the C++ passes `MaxNativeArguments`" is right but the reading "sixteen parameters"
   is not.** The array's first element is the return value and `outputIndex` starts at 1, so the
   accepted width is fifteen parameters.
5. **The brief says `load.rs` is the only file modified.** `ffi.rs` and `values.rs` were modified
   too: the union read is `ffi.rs`'s by ruling 3, and it cannot be total on `Repr` until `Value`
   has a variant per member, which is ruling 4.

Committed at `9f3227058f1ea05d601b5d61941a426e1d59a7a2`. The report itself is not committed:
`.gitignore:30` ignores `.superpowers/`, which is what Tasks 1 to 5 did too.

## Open for later tasks

* **`usedArglist`**, above: whoever fills the `ARGLIST` row owes it.
* **The publish of the array on the context is untested**, above.
* `invoke::method` has no channel for a condition an extension raised. `RaiseException*` stashes
  one and `NativeActivation::run` raises it after the call returns
  (`interpreter/api/ThreadContextStubs.cpp:1863-1885`); the stash lives behind the context's
  function table, which is Task 7's, and the raise after the call is the caller's.
* Nothing here calls `Table::clear` or `CStringPool::clear`. A `CSTRING`'s lifetime is the call and
  a local reference's is the activation, and both outlive `invoke::method`, so the end of the
  activation is where they are dropped. That is Task 7's.

---

# Fix round 1

Three findings from the Task 6 review, committed at
`39aefd6021b7532a9c589292b7336ee7d5aead6c` on top of
`9f3227058f1ea05d601b5d61941a426e1d59a7a2`. Files touched:
`rust/crates/rexx-api/src/ffi.rs`, `rust/crates/rexx-api/src/invoke.rs`,
`rust/crates/rexx-api/tests/invoke.rs`. `load.rs` and `values.rs` are unchanged.

## Finding 1: the raising test could not fail

`RegExp_Init` ends `return 0` on every path (`extensions/rxregexp/rxregexp.cpp:88`) and
`invoke::empty()` starts element zero at zero, so `returned(0)` held whether or not the extension
wrote anything. The report's sentence claiming the answer came "from element zero" was unwitnessed,
and it is corrected below.

`an_extension_that_raises_still_finishes` keeps the two halves that were real, and its name and
doc no longer claim the third. `a_call_that_raised_still_returns_what_it_wrote` is the witness:
`RegExp_Parse('[', 'BOGUS')` raises `Rexx_Error_Incorrect_method`
(`extensions/rxregexp/rxregexp.cpp:130`, the number 93000 at `api/oorexxerrors.h:518`) and then
returns what the parse answered (`:135`), which for that template is 3. So the call raised, ran on,
wrote a value that is not the one the descriptor started at, and `invoke::method` read it back.

## Finding 2: the publish gap is closed

`ffi.rs` gains a `#[cfg(test)] pub(crate)` stub, `reading_stub`, which reads its argument the way an
extension using `argumentExists` does (`api/oorexxapi.h:4276`): through `context.arguments` rather
than through the parameter. It compares the published array against the one it was handed and, only
where the two agree, reads the `flags` word of the argument, which is what the reviewer asked for.
A Rust reader re-states the macro rather than using it, so asserting the pointer alone would say
that something was published and not that the right thing was.
`the_call_publishes_its_array_on_the_context` asserts `Seen { same_array: true, flags: 1 }`.

Where the two pointers differ the stub reads nothing, so the mutation that removes the publish
leaves a sentinel in the field and is observed rather than dereferenced.

**What the unsafe record can and cannot say.** The stub's two `unsafe` blocks are inside `ffi.rs`,
which is already granted, so `crates/rexx-core/tests/unsafe_sites.rs` is unchanged and still
passes. That test is file-granular: it holds a list of files carrying an opt-in and a list of files
containing an `unsafe` block, and neither can show that a new, test-only site appeared inside a
file that was already on both lists. **Nobody should read the unchanged record as saying no unsafe
was added.**

## Finding 3: the claim is narrowed to what is measured

`MethodContextInterface` carries `GetArguments` and `GetArgument`
(`rust/crates/rexx-api/src/layout.rs:629-630`), which reach the activation's argument list and not
`context.arguments`. Under Task 7's real table that list exists before the signature call, so an
extension calling either during a signature request would see an argument. **The oracle has the
same hole**: `createMethodContext` runs at `NativeActivation.cpp:1289`, before the signature call at
`:1297`, so this is not a divergence and no behaviour changed.

`the_signature_call_runs_before_any_argument_is_touched` keeps its name, which was always the
narrow one, and its doc no longer opens with "observes nothing". It now says what holds: no
argument has been produced when the signature is asked for, the array channel is untouched, and
`GetArgument` is a channel of its own that this says nothing about.
`the_signature_call_publishes_no_array`'s doc is narrowed to the array channel the same way.

## Corrections to the first report

* "**Ruling 1, witnessed against the oracle's own binary**" claimed the test asserted
  "`invoke::method` still answered `Ok(Some(0))` from element zero". It asserted the value, not that
  it came from element zero, and could not tell the two apart. The witness is now
  `a_call_that_raised_still_returns_what_it_wrote`.
* "**What no test here covers, stated plainly**" is superseded: the publish is covered by
  `the_call_publishes_its_array_on_the_context`, and control G below is what says so.
* "**How the signature call was made observably argument-free**" overstates its heading. The array
  channel is what was measured; `GetArgument` is not, in either implementation.

## Controls

Predictions were written to `scratchpad/predictions-fix1.md` at 02:58:42, before either control
ran. Baseline: `--lib` 10 passed, `--test invoke` 10 passed. Each control was undone by re-editing
and `sha256sum` against a copy taken beforehand confirmed the restore of all five files.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| G | `load::call` no longer publishes the array | `the_call_publishes_its_array_on_the_context` alone fails, reading `Seen { same_array: false, flags: 0 }` against `{ same_array: true, flags: 1 }`, no crash; integration all green | exactly that, lib 9 passed 1 failed, integration 10 passed | **confirmed** |
| H | `invoke::method` answers from a fresh descriptor instead of element zero | `a_call_that_raised_still_returns_what_it_wrote` and `a_second_method_reads_the_cself_the_first_produced` fail; **`an_extension_that_raises_still_finishes` passes**; lib all green | exactly that, integration 8 passed 2 failed, lib 10 passed, and that test is in the `ok` list | **confirmed** |

G is the control that was green before this round, which is what made finding 2 a gap. H reproduces
finding 1 directly: the test the review called out stays green while the read of element zero is
gone, and the new test is what reddens.

## Commands and exit statuses

Every status read unpiped, from `rust/`.

| command | exit |
|---|---|
| `cargo fmt --all` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |

`cargo test -p rexx-api` reports 91 `ok` result lines: lib 10, `handles` 5, `invoke` 10, `layout`
18, `load` 9, `values` 36, and 3 doctests over two result lines. `unsafe_sites` reports 2 passed.
`rexx-exec` was not touched.
