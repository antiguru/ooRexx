# Task 7 report: the seven context functions, `.Pointer`, and `CSELF`

Committed at `15e7c2797292e6338a34dcaed3d87199cd0aaaf2`,
`0d1671621a48208632ca62329d7369061c6f2cca` and
`c4128d31d06b226ecb05ad17b906feded9a91aff`. The report itself is not committed:
`.gitignore:30` ignores `.superpowers/`, as Tasks 1 to 6 found.

Files added: `rust/crates/rexx-api/tests/context.rs`.
Files modified: `rust/crates/rexx-api/src/ffi.rs`, `rust/crates/rexx-api/src/values.rs`,
`rust/crates/rexx-api/src/invoke.rs`, `rust/crates/rexx-api/tests/invoke.rs`,
`rust/crates/rexx-api/tests/values.rs`, `rust/crates/rexx-core/src/body.rs`,
`rust/crates/rexx-core/src/lib.rs`, `rust/crates/rexx-exec/src/dispatch.rs`,
`rust/crates/rexx-exec/src/value.rs`.

---

## 1. The seven, and what each one does

Each is an `extern "C"` function in `ffi.rs` that decodes the pointers it was handed and
forwards to a method on the `Activation` the context addresses. The bodies are in
`values.rs`, which says `unsafe` nowhere.

| member | Rust signature in `ffi.rs` | what it does |
|---|---|---|
| `SetObjectVariable` | `extern "C" fn(*mut RexxMethodContext_, CSTRING, RexxObjectPtr)` | resolves the handle through the activation's local-reference table and hands the name and the object to `Host::set_object_variable`. A handle the table does not hold, a null one included, is `None`, which is the `OREF_NULL` the oracle would store. A null name is a no-op. |
| `DropObjectVariable` | `extern "C" fn(*mut RexxMethodContext_, CSTRING)` | `Host::drop_object_variable`, same name rules. |
| `WholeNumberToObject` | `extern "C" fn(*mut RexxThreadContext_, wholenumber_t) -> RexxObjectPtr` | `Host::whole_number`, then registers the answer as a local reference, which is what `ApiContext::ret` does (`interpreter/api/ThreadContextStubs.cpp:651`). |
| `StringData` | `extern "C" fn(*mut RexxThreadContext_, RexxStringObject) -> CSTRING` | resolves the handle, reads the bytes through `Host::string_bytes`, and interns them **keyed on the object**. A handle the table does not hold, or an object with no bytes, answers null, which is the `NULL` the stub answers on an exception (`:1079`). |
| `StringLength` | `extern "C" fn(*mut RexxThreadContext_, RexxStringObject) -> usize` | the same resolve, answering the byte count, and zero where `StringData` answers null (`:1065`). |
| `NewPointer` | `extern "C" fn(*mut RexxThreadContext_, POINTER) -> RexxPointerObject` | `Host::new_pointer`, then registers the answer as a local reference. |
| `RaiseException0` | `extern "C" fn(*mut RexxThreadContext_, usize)` | records the number on the activation and returns. A second raise overwrites the first, as `setConditionInfo` (`NativeActivation.cpp:2678`) does. |

The name an extension writes is not always the pointer it reaches, and the dispatch's
table was right on every row: `WholeNumber(n)` is the inline at `api/oorexxapi.h:1041`
calling `WholeNumberToObject`, and the raise is `RaiseException0` and not `RaiseException`.

### The state behind them

```rust
// values.rs
pub struct Activation<'a> {
    conversion: RefCell<Conversion<'a>>,
    pending: Cell<Option<usize>>,
}

impl<'a> Activation<'a> {
    pub fn new(conversion: Conversion<'a>) -> Activation<'a>;
    pub fn conversion(&self) -> RefMut<'_, Conversion<'a>>;
    pub fn raise(&self, number: usize);
    pub fn pending(&self) -> Option<usize>;
    pub fn clear_pending(&self);
    pub fn set_object_variable(&self, name: &[u8], value: RexxObjectPtr);
    pub fn drop_object_variable(&self, name: &[u8]);
    pub fn whole_number(&self, value: isize) -> RexxObjectPtr;
    pub fn new_pointer(&self, value: POINTER) -> RexxObjectPtr;
    pub fn string_data(&self, handle: RexxObjectPtr) -> CSTRING;
    pub fn string_length(&self, handle: RexxObjectPtr) -> usize;
    pub fn constants(&self) -> Constants<RexxObjectPtr>;
}

// ffi.rs
pub static METHOD_CONTEXT: MethodContextInterface;      // the two method-context members
pub struct Contexts<'a, 'h> { /* the two Owned wrappers and the thread table */ }
impl<'a, 'h> Contexts<'a, 'h> {
    pub fn new(activation: &'a Activation<'h>) -> Contexts<'a, 'h>;
    pub fn method(&mut self) -> &mut RexxMethodContext_;
    pub fn constants(&self) -> Constants<RexxObjectPtr>;
}
```

**Why the activation is reached by *shared* reference, and why `invoke::method`'s signature
changed.** The context an extension holds and the protocol driving that call reach the same
state. If `invoke::method` kept taking `&mut Conversion` it would hold a unique borrow
across `entry.call`, while the callback derived a second `&mut` to the same object from the
raw `owner` pointer: two live unique references to one object, which is aliasing UB whether
or not any tool here would catch it. `Activation` is therefore handed round as `&`, the
mutation lives in a `RefCell` and a `Cell`, and `invoke::method` takes the cell for one
conversion at a time and gives it back before calling the extension. An extension that
re-entered the interpreter while a conversion was in flight would panic in an `extern "C"`
frame, which aborts, rather than aliasing silently. That is the only change to
`invoke::method`: `cx: &mut Conversion<'_>` became `cx: &Activation<'_>`, and its behaviour
is unchanged, which Task 6's ten witnesses in `tests/invoke.rs` still assert unaltered.

**`Contexts` carries two lifetimes** so that a context cannot outlive the activation behind
it: `PhantomData<&'a Activation<'h>>`. `Contexts::method` writes the links between its own
fields on every call rather than at construction, because each is the address of a field of
`self` and moving `self` changes it.

### The method-context table is a `static`, the thread table is not

`ffi::METHOD_CONTEXT` is a `static MethodContextInterface` built in a const block from
`MethodContextInterface::REFUSING` with `SetObjectVariable` and `DropObjectVariable`
replaced. It matches `Activity::methodContextFunctions`
(`interpreter/api/MethodContextStubs.cpp:374`), a plain initializer nothing patches. The
thread table is owned by each `Contexts`, because its four object members are handles that
activation minted.

`layout::METHOD_CONTEXT_INTERFACE`, the all-refusing static Task 1 added, is left in place;
`src/invoke.rs`'s unit tests build their contexts from it. The populated one lives in
`ffi.rs` and not in `layout.rs` because its members are function bodies, and `layout.rs` is
the `#[repr(C)]` mirror of the frozen header.

---

## 2. The object-variable scope, and how it was measured

**Measured against the oracle 2026-09-14**, from a fresh empty directory under the standard
wrapper, three descriptors kept apart. `scope.rex`:

```rexx
src = .array~of("expose CSELF", "return CSELF")
.RegularExpression~define('BASECSELF', .Method~new('b', src))
psrc = .array~of("expose !POS", "return !POS")
.RegularExpression~define('BASEPOS', .Method~new('p', psrc))
r = .Sub~new('a*')
say 'base cself class:' r~baseCself~class~id
say 'sub cself class:' r~subCself~class~id
say 'sub cself value:' r~subCself
x = r~parse("aaa")
say 'base pos:' r~basePos
say 'sub pos:' r~subPos
say 'position:' r~position
::requires '/home/moritz/dev/repos/ooRexx/build/bin/rxregexp.cls'
::class Sub subclass RegularExpression public
::method subCself
  expose CSELF
  return CSELF
::method subPos
  expose !POS
  return !POS
```

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/scope.rex" ) \
  >"$D/scope.out" 2>"$D/scope.err"
```

rc 0, stderr empty, stdout:

```
base cself class: Pointer
sub cself class: String
sub cself value: CSELF
base pos: 3
sub pos: !POS
position: 3
```

**The discriminating part is the subclass.** `BASECSELF` is defined on `.RegularExpression`
with `~define`, so its scope is the class whose native `INIT` did the write; `subCself` is
declared on `Sub`, so its scope is `Sub`. The base method reads a `.Pointer` and the
subclass method reads the uninitialised state, whose value is the derived name `CSELF`. The
same split holds for `!POS`, which `RegExp_Parse` writes. So a native method writes into the
pool of **the scope the method belongs to**, not the receiver's class and not `.Object`,
which is `receiver->getObjectVariables(getScope())` at `NativeActivation.cpp:1878`.

`the_write_reaches_the_methods_scope_and_no_other` pins the negative half in the crate, and
`whole_number_to_object_carries_the_position_the_extension_computed` pins the `3` the oracle
answered for `parse("aaa")`.

**The name rules are the interpreter's, not the boundary's.** `SetObjectVariable` forwards
the extension's own spelling. `getVariableRetriever`
(`interpreter/execution/VariableDictionary.cpp:738`) upper-cases it and answers nothing for
a name that is not a variable, and `NativeActivation.cpp:3010` further refuses a literal and
a compound name; all of that is `VariableDictionary`'s and reaches the API only as "the
write did nothing". The doc on `Host::set_object_variable` says so, and the stand-in
interpreters upper-case as that function does.

---

## 3. `.Pointer`: how an instance is minted and what it answers

### The representation

`NativeState::Pointer(*mut c_void)` in `rexx-core/src/body.rs`, beside `Buffer` and
`Stream`, with `NativeState::pointer()` beside `buffer()` and `stream()`.
`Body::pointer(class, behaviour, address)` builds the instance: no name, no variables, no
methods of its own. `rexx_core::pointer_to_string(address)` is `Numerics::pointerToString`
(`interpreter/runtime/Numerics.cpp:882`): `0x0` for a null address and Rust's `{:p}` (which
is glibc's `%p` here) otherwise.

The `Host` seam mints it: `Host::new_pointer(&mut self, POINTER) -> ObjRef`. In `rexx-exec`
a real `.Pointer` is `Body::pointer` with the class the registry answers for `"Pointer"`.

### What the instance answers, all measured against the oracle 2026-09-14

The instance was obtained with the technique the dispatch gave, a method defined on the
class that can `expose CSELF`. **No corpus witness prints the address**: it changes between
runs.

| sent | oracle | where it is asserted |
|---|---|---|
| `~class~id` | `Pointer` | `a_pointer_renders_as_its_address_and_names_its_class` |
| `~objectName` | `a Pointer` | same |
| `~string` | `0x55f062832410`, length 14, first two bytes `0x` | same |
| `~string` after `~objectName = 'zzz'` | still the address | same |
| `~isNull` | `0`; `1` for the null address | `a_null_pointer_renders_as_zero_and_answers_is_null` |
| `p == p`, `p = p` | `1` | `a_pointer_compares_on_its_address_and_nothing_else` |
| `p \== p`, `p \= p` | `0` | same |
| `p == other`, `p = 'abc'`, `p == 'abc'` | `0` | same |
| `p \== 'abc'` | `1` | same |
| `p~'=='()` | `Error 93.903: Missing argument in method; argument 1 is required.` rc 163 | `a_pointer_comparison_needs_its_argument` |
| `p~makeString` | `Error 97.1 ... does not understand message "MAKESTRING"` | not asserted; `.Pointer` registers no `MakeString` and inherits nothing that answers it |
| `say p`, `'x'||p` | the address | not asserted directly; `to_text` is |

The five rows `Setup.cpp:1645-1649` registers now have bodies: `=` and `==` share
`native_pointer_equal`, `\=` and `\==` share `native_pointer_not_equal`, and `IsNull` is
`native_pointer_is_null`. Anything that is not a `.Pointer` compares unequal, which is
`PointerClass::equal`'s `isOfClass` check.

**`~string` needed a change to `Object~string` rather than a row of its own.** `.Pointer`
registers no `String` method: the oracle's `RexxObject::stringRexx` calls the virtual
`stringValue()`, which `PointerClass` overrides. A `("Pointer", "STRING", ...)` row here
would resolve to `.Object`'s own `MethodId` and replace `Object~string` for every receiver,
so instead `native_string` checks for a pointer body before it sends `OBJECTNAME`. The
`~objectName = 'zzz'` measurement is what says that check belongs before the send and not
after.

The three readers in `value.rs` that derive a value's bytes take the arm a mutable buffer's
contents take: `to_text` answers the rendered address whether or not the instance is named,
`text_len` answers its length, and `redirect_of` keeps a pointer off the
`Redirect::InstanceDefault` path. `try_text` answers `None`, as it does for an array, because
the rendering is held nowhere. `text_len_agrees_with_to_text` now walks a null and a non-null
pointer in both the named and the unnamed shape.

### What was *not* built, deliberately

* **No `new_pointer` in `rexx-exec`.** Nothing in a Rexx program can build a `.Pointer`
  (`.NullPointer` is `addToSystem` and not reachable as an environment symbol -- measured,
  `say .NullPointer~class~id` answers `String` and `~isNull` is 97.1), and the production
  caller is `Host::new_pointer`, which is Task 8's. A `pub(crate) fn` with no caller is dead
  code the gate rejects and a shell the constraints warn about, so the four dispatch tests
  mint the instance themselves, in one helper that says why.
* **No `impl Host for Interp`.** See section 7.

---

## 4. The object-keyed string pool

`CStringPool::intern` is unchanged and still answers a fresh copy per call, which
`an_optional_cstring_that_is_supplied_converts_like_a_required_one` pins with `assert_ne!`.
The new entry point beside it is

```rust
pub fn intern_for(&mut self, object: ObjRef, bytes: &[u8]) -> CSTRING;
```

which answers the copy this pool already holds for `object`, and makes one keyed on it
otherwise. The entries became `Vec<(Option<ObjRef>, Box<[u8]>)>`; `intern`'s key is `None`,
so its copies are never shared. `bytes_at` and `clear` are otherwise unchanged, and each
copy is still its own allocation, so growing the pool moves no address already handed out.

---

## 5. Every control, its prediction and its outcome

Predictions were written to `scratchpad/predictions.md` at 03:38:08, before the first
control ran. Baselines: `--test context` 14 passed, `--test invoke` 10, `--lib` 10,
`--test values` 36; in `rexx-exec`, the four pointer tests and `text_len_agrees_with_to_text`.
Every control was one edit undone by re-editing, and `sha256sum -c` against copies taken
beforehand confirmed all five files restored at the end (`values.rs`, `ffi.rs`,
`tests/context.rs`, `dispatch.rs`, `value.rs`, all `OK`), with `git status --short` empty.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| A | `Activation::string_data` calls `intern` instead of `intern_for` | `string_data_answers_one_address_for_one_object` and `the_object_keyed_copy_outlives_the_object_it_came_from` fail, nothing else | exactly those, 12 passed 2 failed, `left: 0x7fc52c000d00 right: 0x7fc52c000c30` | **confirmed** |
| B | `Activation::raise` does nothing | `raise_exception0_records_the_condition_and_the_call_finishes` alone fails, `left: None right: Some(38000)` | exactly that, 13 passed 1 failed, that message | **confirmed** |
| C | `Contexts::new` does not write the four data members | `the_thread_table_carries_the_four_constant_objects` alone fails at "RexxNil is still null" | exactly that, 13 passed 1 failed | **confirmed** |
| D | `Activation::whole_number` does not register the object | `whole_number_to_object_carries_the_position_the_extension_computed` and `cself_survives_a_collection_between_two_calls` fail | exactly those, 12 passed 2 failed | **confirmed** |
| E | `Activation::drop_object_variable` does nothing | `drop_object_variable_unbinds_the_name` alone fails | exactly that, 13 passed 1 failed | **confirmed** |
| F | `Activation::string_length` answers 0 | `string_data_and_string_length_hand_over_the_subject` and `a_collection_inside_the_call_leaves_it_able_to_finish` fail; `string_data_answers_one_address_for_one_object` passes | those two failed **and a third**, `the_object_keyed_copy_outlives_the_object_it_came_from`, which also asserts a length; 11 passed 3 failed. The named pass held | **confirmed for the predicted set, one failure unpredicted** |
| G | the collection between two calls also roots the garbage | `cself_survives_a_collection_between_two_calls` alone fails at "the collection reclaimed nothing, so it witnesses nothing" | exactly that, 13 passed 1 failed | **confirmed** |
| H | `Activation::set_object_variable` always forwards `None` (filtered run) | `new_pointer_and_set_object_variable_store_the_cself_block` fails at "CSELF was not bound"; `the_write_reaches_the_methods_scope_and_no_other` **passes**, being an absence assertion; the unfiltered binary would crash in `RegExp_Parse` on a null automaton | exactly that, 1 passed 1 failed | **confirmed**, including the vacuity |
| I | `Activation::new_pointer` answers a null handle without registering (filtered) | fails at "CSELF was not bound" | exactly that | **confirmed** |
| J | `native_string`'s pointer branch removed | `a_pointer_renders_as_its_address_and_names_its_class` alone fails at the `~STRING` assertion, left `a Pointer`, right the address | exactly that, 4 passed 1 failed | **confirmed** |
| K | `native_pointer_not_equal` answers `equal` | `a_pointer_compares_on_its_address_and_nothing_else` alone fails on the first `\=` case | exactly that, 4 passed 1 failed | **confirmed** |
| L | `to_text`'s pointer arm removed | the two rendering tests and `text_len_agrees_with_to_text` fail | exactly those three, 2 passed 3 failed, **but by a panic at `unreachable!("Redirect::InstanceDefault answers an unnamed instance")` rather than by a value mismatch**: `redirect_of` keeps its own pointer guard, so the body match is reached with no arm for it | failing set **confirmed**, mechanism **falsified** |

**G is the control the dispatch asked for**, in the form that can fail. Running the survival
test with the collection simply switched off would have left it green and said nothing; what
it rests on is that a collection ran and reclaimed the object the test made garbage, so the
assertion names that object. Before that assertion existed the test asked only whether the
sweep count was above zero, which the harness's own spare scope object satisfies on its own;
`c4128d31d06b226ecb05ad17b906feded9a91aff` is the commit that closed it, and G is what shows
the closed version reddens.

**H found the second vacuity and it was predicted rather than discovered.**
`the_write_reaches_the_methods_scope_and_no_other` asserts that a scope the method does not
belong to holds nothing, and a write that never happens satisfies it. It is kept, because
the property it states is the one the oracle measurement is about, but it is only meaningful
beside `new_pointer_and_set_object_variable_store_the_cself_block`, which asserts the write
did land. The pair is the "refusal beside its adjacent success" shape.

---

## 6. Commands and exit statuses

Every status read unpiped, from `rust/`.

| command | exit |
|---|---|
| `cargo fmt --all` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |
| `cargo test -p rexx-exec --lib` | 101 |
| `cargo test -p rexx-exec --no-fail-fast` | 101 |

`cargo test -p rexx-api` reports **105** `ok` result lines across its binaries and doctests:
lib 10, `context` 14, `handles` 5, `invoke` 10, `layout` 18, `load` 9, `values` 36, and 3
doctests over two result lines.

`cargo test -p rexx-core --test unsafe_sites` reports 2 passed, so **the granted set and the
using set are both unchanged**: `ffi.rs`, `load.rs`, `rexx-core/src/lib.rs` granted;
`ffi.rs`, `load.rs`, `rexx-core/src/bytes.rs` using. All the new `unsafe` is inside `ffi.rs`,
which was already on both lists, so **that unchanged result does not say no `unsafe` was
added** -- the test is file-granular, as Task 6's fix round already recorded.

`cargo test -p rexx-exec --lib` reports **803 passed, 3 failed**, the failures the three
pre-existing `ir::drive::tests` cases. The baseline taken before any edit in this task, at
`39aefd602`, was 799 passed with the same three, so the set is unchanged and the four new
tests are the difference.

`cargo test -p rexx-exec --no-fail-fast` was run as well, because `native_string` is a path
every receiver takes. Three binaries fail: `--lib` (the three above), `collect_stress` (9
passed, 2 failed: `a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop` and
`the_l0_subset_passes_again_under_collect_on_every_allocation`), and `refusal_sites` (4
passed, 1 failed: `the_table_holds_every_constructor_the_source_defines`). **All three sets
are pre-existing**, checked by running the two extra binaries at `39aefd602` in a detached
worktree with its own `CARGO_TARGET_DIR`, which reproduced the same names and the same
counts; the worktree and its target directory were then removed and `git worktree list` no
longer shows it. `refusal_sites` disagrees on every row of `corpus/refusal-sites.tsv`, not on
a row this task moved.

The oracle runs used the standard wrapper from a fresh empty directory made with `mktemp -d`
under the session scratchpad, three descriptors kept apart, never `2>&1`.
`rust/corpus/oracle-crashes.txt`'s entry titles were read before running anything; none of
the probes here is one of them.

---

## 7. What the brief and this dispatch got wrong, and what is open

1. **The brief lists the files to modify as `ffi.rs`, `values.rs`, `body.rs` and
   `dispatch.rs`.** `invoke.rs`, `value.rs`, `rexx-core/src/lib.rs` and the two committed
   test files were modified too. `invoke.rs` for the aliasing reason in section 1;
   `value.rs` because a `.Pointer`'s string value is a rendering and the three readers that
   derive one all live there.

2. **Step 5 of the brief asks for `RaiseException`, `RaiseException0` and
   `RaiseException1`.** Only `RaiseException0` is filled. `rxregexp` calls no other, the
   other two take object arguments whose conversion is a different question, and a member
   with no caller is the shell the constraints warn about. The other two still refuse loudly.

3. **`Host` grew five methods, not one.** The dispatch names `Host::cself` as the `CSELF`
   seam and says not to redesign it; it is unchanged. `constants`, `set_object_variable`,
   `drop_object_variable`, `whole_number` and `new_pointer` are new, because each of the
   seven needs something only the interpreter can do.

4. **`invoke::method`'s third parameter changed type.** Section 1. Task 6's tests were
   re-wrapped, not rewritten: every assertion in `src/invoke.rs` and `tests/invoke.rs` is
   the same, and both binaries still report 10 passed.

5. **`exactly_the_rows_this_task_filled_convert` was renamed** to
   `exactly_the_filled_rows_convert`, and `POINTER` joined its outbound set. The old name
   said "this task", which was Task 5's, and a second task filling a row made it false.
   The row filled is `from_native` for `REXX_VALUE_POINTER`, which is `new_pointer` at
   `NativeActivation.cpp:840`.

6. **`valueToObject` has no arm for the six special codes** -- `ARGLIST`, `NAME`, `SCOPE`,
   `CSELF`, `OSELF`, `SUPER` -- so it reaches `default:` and raises the signature error
   (`NativeActivation.cpp:854`). Their `from_native` is therefore not "unfilled" but
   93.968/40.918. **Not changed here**: no signature in reach declares one as a return type,
   and filling six rows nobody asked for is wider than this task. Whoever needs one owes the
   change and the test.

7. **The `GetArgument` hole was left open**, as ruled. Nothing here narrows or widens it.

8. **Open, and a decision for you: a collection triggered from inside a `Host` callback
   cannot see the local-reference table.** `Conversion` holds `locals` beside `host`, and a
   callback runs with the cell taken, so the `Host` cannot read the table whose objects it
   must root. The same shape blocks Task 8 one level up: `Interp::native_handles`
   (`crates/rexx-exec/src/lib.rs:1751`) is already rooted by `Interp::object_roots`, so
   `Conversion { host: &mut interp, locals: &mut interp.native_handles.last_mut() }` is two
   mutable borrows of one `Interp` and does not compile. The two candidate fixes are to move
   the table behind the `Host` (`Host::register` / `Host::resolve`, dropping
   `Conversion::locals`), or to put it behind interior mutability inside `Interp`. I did not
   take either: both change Task 4's and Task 5's committed interfaces.
   `a_collection_inside_the_call_leaves_it_able_to_finish` names the roots from the test for
   exactly this reason, and its doc says so.

9. **`impl Host for Interp` is not written, so no extension method can yet run inside a Rexx
   program.** `Host::set_object_variable` and `Host::cself` need the running native method's
   receiver and scope, which is activation state Task 8 introduces along with the dispatch
   that reaches a library; writing it now would mean inventing that model twice. What Task 7
   delivers is the boundary and everything behind it, exercised end to end against
   `build/lib/librxregexp.so` in `tests/context.rs` with a stand-in interpreter that keeps
   its object variables in a real `ScopePools` and collects with a real `Heap`. **Task 8
   owes the `Interp` implementation**, and item 8 above is the design question it has to
   settle first.

10. **`invoke::method` does not raise the condition it recorded.** It answers the value the
    extension wrote and leaves `Activation::pending()` set, because the oracle raises it in
    the caller's frame after the call returns (`checkConditions`,
    `NativeActivation.cpp:1787`). `Activation::clear_pending` is there for the caller that
    raises it. Nothing in this crate consumes it yet; that is Task 8's, and leaving
    `invoke::method` to raise would have broken Task 6's
    `a_call_that_raised_still_returns_what_it_wrote`.

11. **`.NullPointer` is not reachable.** `Setup.cpp:1749` puts it in the system directory
    with `addToSystem`, and measured against the oracle, `.NullPointer` resolves to the
    literal string `.NULLPOINTER`. Nothing here changes that.

12. **The `owner_of` round trip takes a pointer whose provenance covers only the context
    field.** `Contexts::method` hands out `&mut self.method.context`, and `owner_of` casts it
    back to the whole `Owned` to read `owner`. That is the design Task 1 committed and the
    C++ shape it mirrors (`ActivationApiContexts.hpp:58-95`); it is noted rather than changed,
    because the alternative needs `load::call` to take a raw pointer, which would make it an
    `unsafe fn` and put `unsafe` at its call site in `invoke.rs`, where D-U1 forbids it.

---

# Fix round 1

Three findings from the Task 7 review, committed at
`0183acfdab213d03631e8e31d0be77e68e69b066` on top of
`c4128d31d06b226ecb05ad17b906feded9a91aff`. Files touched:
`rust/crates/rexx-api/src/ffi.rs`, `rust/crates/rexx-api/src/values.rs`,
`rust/crates/rexx-api/src/invoke.rs`, `rust/crates/rexx-api/tests/context.rs`,
`rust/crates/rexx-core/src/body.rs`, `rust/crates/rexx-exec/src/dispatch.rs`.
`rexx-exec/src/value.rs` and the two Task 5 and Task 6 test files are unchanged.

**A correction to that commit's own subject line.** It reads "Correct three citations"; the
body and this section say five, which is the number. The subject counted the three the
review called wrong and not the two it asked me to tighten while I was there. Amending is
forbidden here, so the correction lives at this line instead.

## Finding 1: the in-call collection test measured something else

The review is right, and its control is the proof: stopping `whole_number` from adding
`roots_during_call` left the binary at 14 passed, exit 0. **The machinery was inert**, and a
`CSTRING`'s copy is a `Box<[u8]>` outside the heap, so no collection can invalidate one and
no test can witness it doing so.

The field, its doc, and the loop that read it in `Interpreter::whole_number` are gone rather
than kept with a caveat: the doc called it "what a collection inside a call roots", and a
later reader would build on it. The collection inside a call now roots the receiver alone,
which is the interpreter state that really is reachable from a host callback.

`a_collection_inside_the_call_leaves_it_able_to_finish` is now
`a_collection_inside_the_call_leaves_the_cself_intact`, and its doc says what the assertions
hold and what they do not: the call finishes with the answer the extension computed and the
receiver's block is where it was, and the `StringData` pointer is not the subject, with
`the_object_keyed_copy_outlives_the_object_it_came_from` named as the test that is.

**The renamed test's central assertion is live, and control M shows it cleanly.** Prediction
written to `scratchpad/predictions-fix1.md` at 04:21:53, before the control ran; baseline
`--test context` 14 passed.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| M | `ScopePools::trace` emits no pool values | `a_collection_inside_the_call_leaves_the_cself_intact` and `cself_survives_a_collection_between_two_calls` fail at their `cself` comparisons, cleanly rather than by aborting; `the_cself_object_is_swept_when_the_receiver_is_not_a_root` passes; 12 passed 2 failed | exactly that, `left: None right: Some(0x7f7e20000de0)` | **confirmed** |

M matters because the review's own second control on this test could only redden it by
`SIGABRT`. M reddens the same assertion from the test body, after the call has returned, so
the assertion is shown to be able to fail without the abort.

**The abort is a fact about this whole crate's tests, and the next task should design around
it.** A panic raised inside a host callback crosses an `extern "C"` frame and aborts, which
is ruling 2 working: nothing unwinds into extension code. The consequence is that a test
whose only failure mode is a panic inside a callback takes the whole test binary with it
rather than failing one case, so it reports as a crashed binary and not as a named failure.
Two of this task's own controls (H and I) were run filtered for the same reason. **A test
that has to observe something a callback did should observe it after the call returns**, out
of the `extern "C"` frame, where an assertion failure is a normal red.

**The brief's step 4 is discharged by copying, not by rooting.** "The object must be rooted
for that long" describes the oracle, which hands out an interior pointer into a non-moving
string. This crate cannot: a Rexx string here carries no terminator and the arena
reallocates its slot vector, so `CStringPool` owns the copies instead and the lifetime is
the pool's rather than the object's. The divergence was Task 5's and is deliberate; it was
undocumented against the brief until now.

## Finding 2: five citations did not land on their subject

Each replacement was checked by printing the line it names, from the repository root.

| site | was | is | what is at the new line |
|---|---|---|---|
| `values.rs:386` | `Numerics.cpp:855` | `Numerics.cpp:184` | `RexxObject *Numerics::wholenumberToObject(wholenumber_t v)` |
| `values.rs:390` | `PointerClass.hpp:114` | `PointerClass.hpp:81` | `inline PointerClass *new_pointer(void *p)` |
| `rexx-core/src/body.rs:162` | `PointerClass.hpp:59` | `PointerClass.hpp:77` | `void *pointerData;` |
| `values.rs:1037` | `NativeActivation.cpp:840` | `:839` | `return new_pointer(value->value.value_POINTER);` |
| `rexx-core/src/body.rs:711` | `PointerClass.cpp:118` | `:122` | `void *PointerClass::operator new(size_t size)` |
| `dispatch.rs:8498` | `PointerClass.cpp:73`, `:94` | `:71`, `:91` | `RexxObject *PointerClass::equal(...)` and `::notEqual(...)` |

The last row was not in the review's list and is the same defect: both pointed at a body's
first statement rather than at its signature, and `:94` was a blank line.

The range naming where the thread table's data members are filled moved from
`Activity.cpp:1841-1849`, which starts on the comment above the function, to `:1846-1849`,
the four assignments themselves. Three sites carried it: `ffi.rs:99`, `values.rs:366` and
`tests/context.rs:453`.

## Finding 3: eight comments named a set's size

`ffi.rs:97` and `:157`, `values.rs:394` and `:515`, `invoke.rs:327`, `tests/context.rs:453`,
and `dispatch.rs:743` and `:9096`. Each names the set instead: "the thread table's data
members", "every comparison spelling", "`=` and `==` name one function and `\=` and `\==`
name the other".

`dispatch.rs:743` was attributed to the pre-existing text in the review; it is this task's
own, added with the `.Pointer` rows. The pre-existing counts elsewhere in that file
(`:106`, `:3954`, `:5744`, `:5931`) were left alone.

One name still carries a count: the test `the_thread_table_carries_the_four_constant_objects`.
It is a test name and not a comment, and its body enumerates the four rather than asserting
the number, which is the case the rule exempts.

## Commands and exit statuses

Every status read unpiped, from `rust/`.

| command | exit |
|---|---|
| `cargo fmt --all` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |
| `cargo test -p rexx-exec --lib` | 101 |

`cargo test -p rexx-api` reports 105 `ok` result lines, unchanged, with `context` still at
14 passed. `unsafe_sites` reports 2 passed, so the granted set and the using set are
unchanged. `cargo test -p rexx-exec --lib` reports 803 passed and the three pre-existing
`ir::drive::tests` failures, which is the expected state.

The control edited `crates/rexx-core/src/body.rs` and was undone by re-editing;
`run_control.py` compares the file's sha256 before and after and refuses to report otherwise,
and `git status --short` was empty before the commit.
