# Task 5 report: the conversion table

Files added: `rust/crates/rexx-api/src/values.rs`, `rust/crates/rexx-api/tests/values.rs`.
One line added to `rust/crates/rexx-api/src/lib.rs`. Nothing else was touched; `rexx-exec` was not
modified, so its pre-existing `ir::drive::tests` failures were not re-measured.

## The table's shape

`static TABLE: &[Row]` holds one row per `REXX_VALUE_*` code the frozen header defines, and
`fn row(code) -> Option<&'static Row>` is the only lookup. A row is

```rust
struct Row {
    code: u16,
    name: &'static str,
    source: Source,   // Argument | Special
    absent: Absent,   // Zero | Signature
    to_native: Option<ToNative>,
    from_native: Option<FromNative>,
}

type ToNative = fn(&mut Conversion<'_>, ObjRef, usize) -> Result<Value, Failure>;
type FromNative = fn(&mut Conversion<'_>, Value) -> Result<Option<ObjRef>, Failure>;
```

`source` records whether the code consumes an argument, which is the split
`processArguments` makes between its outer switch and its `default:` arm; `absent` records what an
omitted optional writes, which is the third switch at `NativeActivation.cpp:615`. Both are data on
the row rather than control flow, so a later task fills a row rather than editing a match.

The public entry points are

```rust
pub fn to_native(cx: &mut Conversion<'_>, declared: u16, argument: Option<ObjRef>, position: usize)
    -> Result<Converted, Failure>;
pub fn from_native(cx: &mut Conversion<'_>, declared: u16, value: Value)
    -> Result<Option<ObjRef>, Failure>;
pub fn descriptor(code: u16, converted: Converted) -> ValueDescriptor;
pub fn consumes_argument(declared: u16) -> Option<bool>;
pub fn rows() -> impl Iterator<Item = (u16, &'static str)>;
```

with

```rust
pub struct Conversion<'a> {
    pub host: &'a mut dyn Host,
    pub locals: &'a mut Table,       // handles.rs, Task 4
    pub strings: &'a mut CStringPool,
}

pub enum Value { Omitted, Int(c_int), CString(CSTRING), Pointer(POINTER), Object(RexxObjectPtr) }
pub struct Converted { pub value: Value, pub flags: u16 }
```

`declared` is the signature word the extension published, optional bit included;
`argument_type` and `is_optional` mirror the header's two macros at `api/oorexxapi.h:4272-4273`.
`Converted::flags` is `ARGUMENT_EXISTS` for a supplied argument, `ARGUMENT_EXISTS |
SPECIAL_ARGUMENT` for a special, and zero for an omitted optional, which is what the three branches
of `processArguments` write.

`to_native` settles the absent cases before it asks the row for a conversion function, because the
C++ does: a missing required argument and an omitted optional are both answered without entering
the type switch. The first draft had this the other way round and made an omitted `OPTIONAL_int`
refuse; `an_omitted_optional_argument_does_not_need_its_row_filled` pins the order.

### The rows this task filled

| code | to_native | from_native |
|---|---|---|
| `CSELF` (5) | `cself_to_native` | unfilled |
| `int` (12) | unfilled | `int_from_native` |
| `CSTRING` (15) | `cstring_to_native` | unfilled |
| `RexxStringObject` (17) | `string_object_to_native` | `object_from_native` |

`OPTIONAL_CSTRING` is not a row. `REXX_VALUE_OPTIONAL_CSTRING` is
`(REXX_OPTIONAL_ARGUMENT | REXX_VALUE_CSTRING)` and `ARGUMENT_TYPE` strips the bit before the
switch, so it is the `CSTRING` row reached with the bit set.
`every_optional_name_is_a_base_code_with_the_optional_bit` derives this over every
`#define REXX_VALUE_OPTIONAL_` line in the header rather than asserting it for the one row this
slice needs.

Every other row carries `None` in both directions and refuses with
`Failure::Unfilled { code, name, direction }`, whose `Display` names Phase 8.
`Failure::error_number` answers `None` for it, so a refusal cannot be mistaken for a condition.

## The `CSELF` seam for Task 7

```rust
pub trait Host {
    fn is_method(&self) -> bool;
    fn string_value(&mut self, object: ObjRef) -> Option<ObjRef>;
    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>>;
    fn cself(&mut self) -> Option<POINTER>;
}
```

`cself_to_native` is the whole row: a signature error when `is_method()` is false, and otherwise
`Value::Pointer(host.cself().unwrap_or(null_mut()))`. Everything Task 7 owns is behind
`Host::cself`: taking the guard lock through `methodVariables`, reading the `CSELF` object variable
at the method's scope, and unwrapping the `.Pointer` it holds
(`NativeActivation::cself`, `interpreter/execution/NativeActivation.cpp:2091`). `None` is the
oracle's NULL return for a receiver that never set the variable, and it is not an error, which
`a_receiver_with_no_cself_variable_converts_to_null` asserts.

The direction Task 7 also owes is `from_native` for `CSELF` and `POINTER`, which needs a `.Pointer`
instance minted internally; the row is present and refusing.

`Host::string_value` is the `requiredString` seam
(`interpreter/classes/ObjectClass.cpp:1341`): a base class answers `makeString()`, anything else is
sent `REQUEST("STRING")` and `.nil` back is the `None` here.

## How a `CSTRING`'s bytes stay reachable for the call

The oracle roots the string object and hands out `getStringData()`, a pointer into it. That is not
available here for two independent reasons, both stated in `CStringPool`'s doc comment: a Rexx
string in this crate carries no terminator, and the arena reallocates its slot vector, so no
address inside the heap survives an allocation even though the collector moves nothing.

So `CStringPool` owns the copies. `intern` copies the bytes, appends a zero, and stores the result
as a `Box<[u8]>`; the pointer is that box's `as_ptr()`. Two properties follow. A collection cannot
free the bytes, because they are not in the heap. And growing the pool cannot move a pointer already
handed out, because each entry is its own allocation and only the `Vec` of boxes reallocates.
Dropping or clearing the pool is what ends the lifetime, and that is the end of the call.

`a_cstring_outlives_a_collection_and_the_object_it_came_from` converts an unrooted string, collects
(asserting `swept == 1` and that `heap.get(subject)` misses, so the collection really did reclaim
the source), interns sixty-four more strings, and reads the original bytes back through
`CStringPool::bytes_at`.

**What that test does not do is dereference the pointer**, because a test may not say `unsafe`
either: `crates/rexx-core/tests/unsafe_sites.rs` scans every `.rs` under `crates/`, `tests/`
included. `bytes_at` finds the entry whose `as_ptr()` equals the pointer and returns that entry's
slice, so the test proves the pool still owns those bytes at that address; the step from there to a
valid load is the ownership argument above, not a measurement. Task 6 is where a real load happens,
in `ffi.rs`.

Task 7 hits the same constraint for `StringData`, and with one extra requirement this task did not
have: the oracle's `StringData(obj)` answers the same address every time for the same object, so a
pool keyed only by insertion, as this one is, would answer a fresh address per call. Keying on the
object is the change that costs nothing now and is awkward later.

## Errors: three numbers, two of them measured, and the dispatch was wrong

The dispatch and the brief both say a required argument that is *absent or unconvertible* is
93.968 for a method and 40.918 for a call. That is not what either the source or the oracle says.

* **Absent required argument: 88.901**, `Error_Invalid_argument_noarg`
  (`interpreter/messages/RexxErrorCodes.h:462`). `processArguments`' `if (!isOptional)` arm raises
  it directly (`NativeActivation.cpp:611`), before any type-specific code runs.
  Measured against the oracle, `.RegularExpression~new~parse()`:
  `Error 88.901:  Missing argument; argument 1 is required.`, rc 168.
* **Unconvertible argument: 88.909**, `Error_Invalid_argument_string`
  (`RexxErrorCodes.h:470`), from `RexxInternalObject::requiredString(size_t)`
  (`ObjectClass.cpp:1373`) under `stringArgument`.
  Measured, `.RegularExpression~new~parse(.foo~new)` for a bare `::class foo`:
  `Error 88.909:  Argument 1 must have a string value.`, rc 168.
* **93.968 / 40.918** are `reportSignatureError` (`NativeActivation.cpp:190`), raised for a bad
  *signature*: a type code the switch does not know, a special argument asked for outside a method,
  or more declared arguments than the descriptor array holds. The two numbers are at
  `RexxErrorCodes.h:582` and `:408`, which the dispatch cited correctly; what it attached them to is
  wrong. **Neither is witnessed by a run.** Producing one needs an extension whose signature is
  malformed, and building one means compiling a new `.so`, which is outside this task and is not
  `librxregexp.so`.

`Failure::error_number(method: bool)` is the single place these live, and
`the_signature_error_is_93_968_in_a_method_and_40_918_in_a_call` pins the pair.

Oracle runs used the standard wrapper from a fresh empty directory, three descriptors kept apart:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/$f.rex" ) >"$D/$f.out" 2>"$D/$f.err"
```

with `::requires '/home/moritz/dev/repos/ooRexx/build/bin/rxregexp.cls'`. A bare
`::requires 'rxregexp.cls'` fails 43.901 from a scratch directory, which is what the first three
runs answered.

The happy path was measured too, and is what the filled rows have to reproduce:
`parse rc: 0`, `match rc: 1`, `pos rc: 1`, `bad rc: 3` for `~parse('[')`, and
`.RegularExpression~new('b*','MINIMAL')~match('bb')` answering 1.

## Negative controls

Predictions were written to the scratchpad before the first control ran
(`scratchpad/predictions.md`, 02:04:20). Each control edited `src/values.rs` and was restored by
re-editing; `sha256sum` against a pristine copy confirmed the restore each time, and
`git status --short` at the end shows only the two new files and the one-line `lib.rs` change.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| A | the `CSTRING` row deleted from `TABLE` | 10 named tests fail | exactly those 10, 25 passed | **confirmed** |
| B | `CSTRING` row kept, `to_native: None` | 6 named tests fail | exactly those 6, 29 passed | **confirmed** |
| C | `int` row `from_native: None` | 3 named tests fail | exactly those 3, 32 passed | **confirmed** |
| D | `CSELF` row `to_native: None` | 4 named tests fail | exactly those 4, 31 passed | **confirmed** |
| E | `RexxStringObject` row `to_native: None` | 5 named tests fail | exactly those 5, 30 passed | **confirmed** |

The named sets, in full:

* **A** `the_cstring_row_points_at_a_copy_of_the_argument`,
  `a_cstring_outlives_a_collection_and_the_object_it_came_from`,
  `a_required_cstring_that_is_absent_is_88_901`,
  `a_required_cstring_with_no_string_value_is_88_909`,
  `a_cstring_returned_by_an_extension_is_not_converted_yet`,
  `an_omitted_optional_cstring_is_a_zero_with_no_flags`,
  `an_optional_cstring_that_is_supplied_converts_like_a_required_one`,
  `a_descriptor_carries_the_stripped_code_and_the_flags`,
  `the_table_has_a_row_for_every_code_the_header_defines`,
  `exactly_the_rows_this_task_filled_convert`.
* **B** A's list minus `a_required_cstring_that_is_absent_is_88_901`,
  `an_omitted_optional_cstring_is_a_zero_with_no_flags`,
  `a_cstring_returned_by_an_extension_is_not_converted_yet` and
  `the_table_has_a_row_for_every_code_the_header_defines`.
* **C** `the_int_row_converts_a_returned_value`, `the_int_row_refuses_a_value_of_another_type`,
  `exactly_the_rows_this_task_filled_convert`.
* **D** `the_cself_row_reads_the_seam_and_consumes_no_argument`,
  `a_receiver_with_no_cself_variable_converts_to_null`,
  `cself_outside_a_method_is_a_signature_error`, `exactly_the_rows_this_task_filled_convert`.
* **E** `the_string_object_row_round_trips_through_a_handle`,
  `a_string_the_conversion_built_is_rooted_for_the_call`,
  `the_same_string_is_swept_when_the_table_is_not_a_root`,
  `a_string_object_with_no_string_value_is_88_909`,
  `exactly_the_rows_this_task_filled_convert`.

**A and B are the pair that says what the table is.** Deleting the row reddens the coverage test as
well, because the coverage test is about the table as a whole and is supposed to notice; the
predictions said so before the run. Neutralising the row's function leaves the coverage test green
and reddens only the conversions, which is the sharper reading of "the table is what drives the
conversion". B, C, D and E each redden their own row's tests and
`exactly_the_rows_this_task_filled_convert`, which is a set assertion over the whole table and is
also supposed to notice; no test for another row moved in any of the five.

The tests in this file that no control reddened are the ones that are about the pool or the header
rather than a row: `clearing_the_pool_drops_the_copies`,
`an_embedded_zero_is_copied_rather_than_refused`,
`every_platform_alias_names_a_code_the_table_has`,
`every_optional_name_is_a_base_code_with_the_optional_bit`.

`a_string_the_conversion_built_is_rooted_for_the_call` carries its own control in the file:
`the_same_string_is_swept_when_the_table_is_not_a_root` runs the same conversion, collects with an
empty `RootSet`, and asserts the built string is gone. Without it the rooting test is satisfied by a
collector that sweeps nothing.

Committed at `06fd862d8f0d5a401a3e24800f3f39d39d9c5514`. The report itself is not committed: `.gitignore:30`
ignores `.superpowers/`, and no SDD file for this phase is tracked in this worktree, which is what
Tasks 1 to 4 did too.

## Commands and exit statuses

Every status read unpiped.

| command | exit |
|---|---|
| `cargo fmt --all` (from `rust/`) | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |

`cargo test -p rexx-api` reports 72 `ok` result lines across seven binaries: lib 2, `handles` 5,
`layout` 18, `load` 9, `values` 35, and 3 doctests. `unsafe_sites` reports 2 passed, so the granted
set is unchanged: `values.rs` says `unsafe` nowhere and asked for no grant.

`cargo test -p rexx-exec --lib` was not run: this task did not touch that crate.

## What the brief and the dispatch got wrong

1. **The error numbers.** Section above. 93.968 and 40.918 are the signature errors, not the
   absent-or-unconvertible errors, and the two cases the dispatch describes are 88.901 and 88.909,
   both measured against the oracle.
2. **`values::from_native(&ValueDescriptor)` cannot be written in `values.rs`.** Reading a union
   field is `unsafe` in Rust, and D-U1 grants `values.rs` nothing. The signature is
   `from_native(&mut Conversion<'_>, u16, Value)`, taking the payload already extracted.
   Constructing a union is safe, so the outbound half does produce a real `ValueDescriptor` through
   `descriptor(code, converted)`. Task 6 owns the one unsafe read of `arguments[0]`, in `ffi.rs`;
   the cleanest shape for it is a small match on a representation the row names, so the code-keyed
   table stays the only place that knows what a code means. I did not add that to `ffi.rs`, because
   nothing in this task needs it and a seam with no caller is the shell the constraints warn about.
3. **`to_native(ObjRef, code)` needs more than an `ObjRef` and a code.** It needs the local
   reference table (to mint a handle), the string pool (to own a `CSTRING`'s bytes), the host (to
   force a string value and to read `CSELF`), whether an argument was supplied at all, and the
   argument's position for the error insert. `Conversion<'a>` bundles the first three.
4. **`OPTIONAL_CSTRING` is not a fifth row**, so "five rows" is four rows plus the shared
   omitted-argument path. The table has a row per code the header defines, which is more than the
   "about thirty" the spec says; the coverage test derives the exact set rather than stating it.
5. **A raise does not end the call, and nothing here assumes it does.** Every function in this
   module is total on its inputs and returns a `Result`; there is no unwinding path and no early
   exit that could skip `arguments[0]`. Whether `from_native` still runs after a raise is Task 6's
   control flow, not this table's.

## Open for later tasks

* `Failure::StaleHandle` is a divergence the oracle cannot have: it casts whatever pointer the
  extension returned. D5 makes a stale handle a lookup miss, so `object_from_native` refuses rather
  than answering the slot's next occupant. Where that refusal surfaces to a program is unsettled.
* `Value` covers only the union members this slice needs. `double`, `float` and the wide integers
  each need a variant, and `descriptor` needs the matching `ValueUnion` arm.
* `Value::Omitted` writes a zero word for every type. The C++ writes `value_int64_t = 0`,
  `0.0` and `0.0f` in three arms, which are the same bytes; if a future row wants a non-zero
  default, `Absent` is where it goes.

---

# Fix round 1

Three findings from the Task 5 review, committed at `64d1bfeef0928a272824f6ab2d3fd3236b61de2f`
on top of `06fd862d8f0d5a401a3e24800f3f39d39d9c5514`. Files touched:
`rust/crates/rexx-api/src/values.rs`, `rust/crates/rexx-api/tests/values.rs`.

## Finding 1: the descriptor's type field

`descriptor` took a parameter named `code` and stored it unchanged, so a caller that passed the
signature word wrote `0x800F` where the oracle writes `15` (`NativeActivation.cpp:243`, `:246`).
The only reason no test saw it is that the one call site spelled `argument_type(declared)` itself.

The signature is now `descriptor(declared: u16, converted: Converted) -> ValueDescriptor`, and it
strips the bit. `a_descriptor_carries_the_stripped_code_and_the_flags` passes `declared` straight
in, which is the shape Task 6 would naturally write, and also checks a plain code so the stripping
is not the only thing asserted.

## Finding 2: the union member is a row field

`Repr` is a new public enum naming which member of `ValueDescriptor`'s union a code's value
occupies, and `Row` carries one. The accessor is

```rust
pub fn repr(declared: u16) -> Option<Repr>;
```

`Repr` distinguishes members and not codes, so it is coarser than the table: `Object` covers every
object-typed member, `Pointer` covers `value_POINTER` and `value_POINTERSTRING`, `Isize` covers
`wholenumber_t`, `ssize_t` and `intptr_t`, and `Usize` covers `stringsize_t`, `size_t`, `uintptr_t`
and `logical_t`. That is what `ffi.rs` needs to know to perform the one unsafe read: a match on
`Repr` rather than a second match on the code, so a row the surface half adds is readable through
it the moment its row exists.

I accepted the ruling rather than arguing it. The reason it is right is the one the review gives:
the second match would be outside the table and invisible to the row-deletion control. The reason
it is cheap is that the header states the mapping itself, so the row is not a new claim but a
transcription of one that can be checked.

`every_row_names_the_union_member_the_header_gives_its_code` does that check. For each row it reads
`ARGUMENT_TYPE_<name>` out of `api/oorexxapi.h` (`:4196-4239`), which is the header's own statement
of the C type that code carries, maps the C type to a `Repr` through a table keyed on type names
rather than codes, and compares. It also asserts the optional bit does not change the answer, that
an unknown code has no `Repr`, and that the scan found the defines at all.

**One thing that test cannot see**: it iterates `rows()`, so a row deleted from the table is simply
not checked rather than reported. `the_table_has_a_row_for_every_code_the_header_defines` is what
catches a deletion, and control A' below confirms the pair still behaves that way.

## Finding 3: the comment naming a set's size

`tests/values.rs`, "The two numbers a signature error carries, which differ by context" is now
"A signature error's number differs by context."

## Preserved

`CStringPool::intern` is unchanged and still answers a fresh entry per call, which is what
`an_optional_cstring_that_is_supplied_converts_like_a_required_one` asserts with `assert_ne!(a, b)`.
Object-keyed interning for Task 7's `StringData` is therefore a second entry point beside `intern`,
not a change to it; nothing added here forecloses that.

## Controls

Predictions written to `scratchpad/predictions-fix1.md` at 02:17:55, before any control ran. Each
control edited `src/values.rs` and was undone by re-editing, with `sha256sum` against a copy taken
before the round confirming the restore.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| F | `descriptor` stores `declared` unstripped | `a_descriptor_carries_the_stripped_code_and_the_flags` alone fails, left 32783 right 15 | exactly that, 35 passed, `left: 32783` `right: 15` | **confirmed** |
| G | the `CSELF` row's `repr` set to `Repr::Object` | `every_row_names_the_union_member_the_header_gives_its_code` alone fails, naming CSELF | exactly that, 35 passed, "REXX_VALUE_CSELF is declared POINTER" | **confirmed** |
| A' | control A re-run: the `CSTRING` row deleted | the same ten names as control A, with the new repr test passing because a deleted row is not iterated | exactly those ten, 26 passed | **confirmed** |

## Commands and exit statuses

| command | exit |
|---|---|
| `cargo fmt --all` (from `rust/`) | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |

`cargo test -p rexx-api` reports 73 `ok` result lines, `tests/values.rs` at 36. `unsafe_sites`
passes both its tests, so `values.rs` still says `unsafe` nowhere.
