# Task 2 report: Open a library and read its package entry

Base: `36e1f5b99`. Commit: `85fd2f136`, four files, `Cargo.lock` untouched. Files touched: `rust/crates/rexx-api/src/load.rs` (written),
`rust/crates/rexx-api/tests/load.rs` (created), `rust/crates/rexx-api/src/lib.rs` (one word),
`rust/crates/rexx-core/tests/unsafe_sites.rs` (the `uses` list and its message).

## The API produced

All in `rexx_api::load`.

```rust
pub const CURRENT_INTERPRETER_VERSION: c_int = 0x0005_0300;

pub type PackageHook = unsafe extern "C" fn(*mut c_void);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    LibraryVersion(String),
}
impl Failure {
    pub fn code(&self) -> (u16, u16);
}

#[repr(C)] pub struct RexxRoutineEntry {
    pub style: c_int, pub reserved1: c_int,
    pub name: *const c_char, pub entry_point: *mut c_void,
    pub reserved2: c_int, pub reserved3: c_int,
}
#[repr(C)] pub struct RexxMethodEntry { /* same fields, same order */ }
#[repr(C)] pub struct RexxPackageEntry {
    pub size: c_int, pub api_version: c_int, pub required_version: c_int,
    pub package_name: *const c_char, pub package_version: *const c_char,
    pub loader: Option<PackageHook>, pub unloader: Option<PackageHook>,
    pub routines: *mut RexxRoutineEntry, pub methods: *mut RexxMethodEntry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeMethodEntry { pub style: c_int, pub name: Vec<u8>, pub entry_point: *mut c_void }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRoutineEntry { pub style: c_int, pub name: Vec<u8>, pub entry_point: *mut c_void }

#[derive(Debug)]
pub struct Library { /* private */ }
impl Library {
    pub fn name(&self) -> Option<&[u8]>;
    pub fn version(&self) -> Option<&[u8]>;
    pub fn methods(&self) -> &[NativeMethodEntry];
    pub fn routines(&self) -> &[NativeRoutineEntry];
    pub fn method(&self, name: &[u8]) -> Option<NativeMethodEntry>;
}

pub fn open(name: &str) -> Result<Option<Library>, Failure>;
pub fn open_path(path: &Path) -> Result<Option<Library>, Failure>;
pub fn loads(path: &Path) -> bool;
pub fn check_version(entry: &RexxPackageEntry, name: &str) -> Result<(), Failure>;
```

Private: `dlopen`, `package_of`, and the three `unsafe fn` readers `c_bytes`, `method_table`,
`routine_table`.

### Where this departs from the brief's interface line, and why

* **`Failure` is a new type in this crate, not `rexx-exec`'s.** `rexx_exec::error::Failure` is
  `pub(crate)` and its payload `Raised` carries `ObjRef`, `Delivery` and the catalogue
  substitutions. `rexx-api` cannot see it, and depending on `rexx-exec` would invert the direction
  Task 8 needs. `load::Failure` carries the error's identity (98.982) and the one substitution the
  catalogue entry interpolates, and Task 8 converts. `Failure::code()` answers `(98, 982)`, so the
  number lives in one place.
* **`method` is keyed by `&[u8]`, not `&str`.** Method names arrive from a Rexx string, which is
  bytes here, and `CStr::to_bytes()` needs no validation and no lossy fallback.
* **`open` takes `&str`.** A `::REQUIRES ... LIBRARY` name is bytes; Task 8 converts, or this
  signature widens then.
* **`open_path` and `loads` are additions.** `open_path` is what lets a test load
  `build/lib/librxregexp.so` by absolute path with no `LD_LIBRARY_PATH` and no
  `std::env::set_var`. `loads` exists because `open_path` answers `Ok(None)` for two different
  situations and a control that cannot separate them proves nothing (see C2 below).
* **`check_version` is public** so a test can hand it a synthetic entry, which is what step 6's
  third control asks for. It takes `&RexxPackageEntry` and reads only an integer field, so it is
  safe to call; `package_of` calls the same function on the real entry, which is what mutation
  control A confirms.
* **The three `#[repr(C)]` structs live in `load.rs`.** **Task 3 must move them into `layout.rs`,
  not declare a second copy.** They are `RexxPackageEntry`, `RexxMethodEntry`, `RexxRoutineEntry`
  and the `PackageHook` alias, whose argument is `*mut c_void` today and becomes
  `*mut RexxThreadContext_` when Task 3 defines it.
* **`lib.rs` changed from `mod load;` to `pub mod load;`.** An integration test cannot reach a
  private module. The brief did not list `lib.rs`; the change is one word and Task 3 lists that
  file anyway.

### Fidelity notes

* The name search is `lib<name>` + `std::env::consts::DLL_SUFFIX`, then
  `/usr/lib/lib<name><suffix>` (`SysLibrary.cpp:88-95`). `DLL_SUFFIX` is what CMake passes the C++
  build as `ORX_SHARED_LIBRARY_EXT` (`CMakeLists.txt:333`).
* `MAX_LIBRARY_NAME_LENGTH` is 250 and a longer name answers `Ok(None)` without a `dlopen`
  (`SysLibrary.cpp:56`, `:83-86`). This is observable and reproduced; it is not covered by a test.
* `libloading::Library::new` is `dlopen(path, RTLD_LAZY | RTLD_LOCAL)` (verified in
  `libloading-0.8.9/src/os/unix/mod.rs:134-135`), and glibc resolves the oracle's bare `RTLD_LAZY`
  to the same pair.
* Both tables end on `style == 0`, which is what `loadRoutines` (`:270`) and `locateMethodEntry`
  (`:320`) test, not on a null name.
* `method` matches without regard to case, as `locateMethodEntry`'s `strCaselessCompare` does.
* A null `RexxGetPackage` return answers `Ok(None)`, matching `LibraryPackage::load`'s
  `package == NULL` arm (`:148-155`) rather than the deref `loadPackage` would do.
* `Library`'s `handle` field is declared last so it drops last: every `entry_point` in the copied
  tables points into that mapping. It carries `#[expect(dead_code)]` because being held is not a
  read.

## Every `unsafe` block, and its SAFETY invariant

Six sites, all in `load.rs`.

1. **`dlopen`, `libloading::Library::new(path)`.** *Invariant:* no property of `path` makes this
   sound, because `dlopen` runs the library's initialisers. What establishes it is the
   interpreter's own contract, that a Rexx program naming a library is authorised to load it,
   which is the trust the oracle extends at `SysLibrary.cpp:90`. D-U1 puts the `unsafe` here so
   `open`, `open_path` and `loads` stay safe for the rest of the tree.
2. **`package_of`, `handle.get(GET_PACKAGE_SYMBOL)`.** *Invariant:* the symbol is read at the
   signature `OOREXX_GET_PACKAGE` expands to, a nullary function answering `RexxPackageEntry *`
   (`api/oorexxapi.h:255`). Established by D5, which freezes that header as the declaration both
   sides compile against.
3. **`package_of`, `get_package()`.** *Invariant:* the address came from `dlsym` on a library
   `handle` still holds, so the code stays mapped across the call; the function answers the
   address of a static and does nothing else. Established by `handle` being live and by
   `oorexxapi.h:255`.
4. **`package_of`, `&*entry`.** *Invariant:* non-null (checked on the line above) and the address
   of the `RexxPackageEntry` static the library defines, which lives as long as the mapping;
   layout is the frozen header's. Established by the `OOREXX_GET_PACKAGE` contract and D5.
5. **`package_of`, the four-way tuple calling `c_bytes` twice, `method_table` and
   `routine_table`.** *Invariant:* `entry` is the library's own package entry, so each table
   pointer is null or the array the extension declared, terminated by a zero-`style` row, and
   every string in it is a literal in the same mapping; all of it outlives `handle`. Established
   by the extension having been compiled against the frozen header.
6. **Inside `c_bytes`, `method_table` and `routine_table`**, each of which is an `unsafe fn` with
   a `# Safety` section stating what its caller must guarantee, and each inner block cites which
   part of that it relies on. `method_table`'s row-name deref cites `REXX_METHOD_ENTRY`
   (`api/oorexxapi.h:223`) stringizing the name, so a row the terminator check admits has a
   non-null one; `routine_table`'s cites `REXX_ROUTINE` (`:203`) for the same reason. The
   `at.add(1)` blocks cite that the current row was not the terminator, so a further row follows.

**Known limit, stated rather than fixed.** `NativeMethodEntry` is returned by value and is
`Clone`, so a caller can outlive its `Library` while holding an `entry_point`. Encoding that
would need a lifetime on the return type, which the brief's `Option<NativeMethodEntry>` does not
have. It is documented on the field and on `Library::method`.

## Negative controls

Predictions were written to
`<scratchpad>/predictions.md` before any of the three ran; the file is quoted verbatim below the
outcomes. Every temporary edit was undone by re-editing the file, never by `git checkout`.

### C1 (behavioural, shipped as a test): a library that exists nowhere

*Prediction:* `load::open("zorkolib")` answers `Ok(None)`, not `Err`.
*Outcome:* **confirmed.** `a_library_that_is_nowhere_is_not_an_error` passes.

### C2 (behavioural, shipped as a test): a real library with no `RexxGetPackage`

*Prediction:* `libc.so.6` loads, and `load::open_path` on it answers `Ok(None)`.
*Outcome:* **confirmed**, both parts. The first part is asserted separately
(`load::loads(libc)`), because without it `Ok(None)` cannot be told from "did not load" and the
control would be vacuous. `a_library_without_the_exporter_is_not_an_error` passes.

### C3 (behavioural, shipped as a test): a synthetic entry above our version

*Prediction:* `check_version` on an entry whose `requiredVersion` is
`CURRENT_INTERPRETER_VERSION + 1` answers `Err(Failure::LibraryVersion("zorkolib"))`, and that
failure's `code()` is `(98, 982)`.
*Outcome:* **confirmed.** `a_package_asking_for_a_newer_interpreter_is_refused` passes. Its
neighbour `a_package_asking_for_this_interpreter_or_less_is_accepted` pins the other side, for
`0`, `0x0004_0000` and the current value.

C3 exercises the predicate but not its wiring into `package_of`, which is what A covers.

### A (mutation): `CURRENT_INTERPRETER_VERSION` temporarily `0x0003_0000`

*Prediction, written first:* 6 failed, 3 passed, specifically —
`the_interpreter_version_is_the_frozen_headers` FAIL;
`a_package_asking_for_this_interpreter_or_less_is_accepted` FAIL at `required = 0x0004_0000`;
`a_package_asking_for_a_newer_interpreter_is_refused` PASS;
the four `open_rxregexp` tests (`the_oracles_extension_loads_and_names_itself`,
`the_method_table_is_what_the_extension_declares`, `the_extension_declares_no_routines`,
`a_method_is_found_without_regard_to_case`) FAIL because `open_path` now answers
`Err(LibraryVersion)`; `a_library_that_is_nowhere_is_not_an_error` PASS;
`a_library_without_the_exporter_is_not_an_error` PASS.

*Outcome:* **confirmed in every part.** `test result: FAILED. 3 passed; 6 failed`, and the six
named failures are exactly the six predicted. The panic text on the four load tests was
`LibraryVersion("/home/moritz/dev/repos/ooRexx-rust-rewrite/build/lib/librxregexp.so")`, which
also witnesses that `open_path` puts the path into the substitution. Reverted by `sed`-ing the
constant back and re-reading the line.

### B (mutation): `Library::method` compares case-sensitively

*Prediction, written first:* `a_method_is_found_without_regard_to_case` FAIL, the other eight
PASS.
*Outcome:* **confirmed.** `test result: FAILED. 8 passed; 1 failed`, the one failure being that
test. Reverted by re-editing.

### C (mutation): the `uses` list in `unsafe_sites.rs` reverted to `bytes.rs` alone

*Prediction, written first:* `only_the_granted_module_may_say_unsafe` FAIL naming
`crates/rexx-api/src/load.rs`; `the_scan_reaches_the_whole_workspace` PASS.
*Outcome:* **confirmed.** `left: ["crates/rexx-api/src/load.rs", "crates/rexx-core/src/bytes.rs"]`
against `right: ["crates/rexx-core/src/bytes.rs"]`, 1 passed 1 failed. This is what shows the
record tracks the new `unsafe` rather than having been edited decoratively. Reverted by re-editing.

The `concat!` spelling of the needles is untouched, so the file remains subject to its own rule.
Its message read "outside the one module granted permission for it" and now reads "outside the
modules granted permission for it", since the list is no longer a singleton.

## Commands and exit statuses

Run from `rust/`, each read unpiped.

| command | exit | note |
|---|---|---|
| `cargo fmt --all` | 0 | rewrapped one `use` in `tests/load.rs` |
| `cargo fmt --all --check` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | warm target directory |
| `cargo test -p rexx-api` | 0 | 9 `ok` lines, all in `tests/load.rs`; lib and doc targets have no tests |
| `cargo test -p rexx-core --test unsafe_sites` | 0 | 2 `ok` lines |

`rust/CLAUDE.md` calls a same-session clippy green provisional and asks for a clean-target run at
a phase boundary. This was a warm run; the phase's four-gate close is not this task's.

## What the brief got wrong, or left open

1. **`Result<Option<Library>, Failure>` names a type that does not exist across the crate
   boundary.** `rexx-exec`'s `Failure` is `pub(crate)`. Handled as described above; the plan's
   Task 8 text should say which side owns the conversion.
2. **Task 3's "Produces" list already claims `RexxPackageEntry`/`RexxMethodEntry`**, which this
   task had to define. They are in `load.rs` and are Task 3's to move.
3. **The brief lists only `load.rs` and `tests/load.rs`.** `lib.rs` had to change one word for the
   integration test to see the module.
4. **`open` alone cannot load the test target.** Step 5 asks for `build/lib/librxregexp.so` by
   absolute path, but `open`'s whole job is to decorate a bare name; `open_path` was added rather
   than bending `open`.
5. **Step 6's second control is vacuous as written.** "A real library with no `RexxGetPackage`
   also answers `Ok(None)`" is satisfied by a library that failed to load. `loads` was added so
   the test can rule that out.
6. **The version constant.** `REXX_CURRENT_INTERPRETER_VERSION` is `REXX_INTERPRETER_5_3_0` =
   `0x00050300` (`api/oorexxapi.h:242`, `:241`). "Derive rather than write a number" is
   implemented as a written constant plus `the_interpreter_version_is_the_frozen_headers`, which
   re-reads `api/oorexxapi.h`, follows the alias and asserts the value. The header's own comment
   at `:229` says the encoding is two decimal digits per component, so a component of ten or more
   would be BCD rather than plain shifting; no such constant exists in the header and the test
   compares against the literal, so nothing here depends on the encoding rule.
7. **`98.982`'s message takes one substitution**, the library name:
   `Library <q><Sub position="1" name="name"/></q> is not compatible with current interpreter
   version.` (`interpreter/messages/rexxmsg.xml:5676-5684`). `Failure::LibraryVersion` carries it.
   Nothing in this task renders the message.

## What is not covered

* The `loader`/`unloader` hooks are read into the struct's fields but never called; `loadPackage`
  runs the loader through a dispatcher (`LibraryPackage.cpp:237-247`) and that needs the thread
  context, which is Task 3's.
* Unload ordering is satisfied structurally (the handle drops last) and is not tested, because
  nothing yet holds an `entry_point` past a `Library`.
* `MAX_LIBRARY_NAME_LENGTH` is reproduced and untested.
* `apiVersion` and `size` are read into the struct and not checked against
  `REXX_PACKAGE_API_NO` / `sizeof`. The C++ does not check either.

---

# Fix round 1

Review came back SPEC PASS, QUALITY CHANGES-REQUESTED with three findings. All three addressed;
finding 1 needed one step beyond the ruling, for a reason established by running rather than by
argument.

## Finding 1: the reachable use-after-unload

**The reviewer's diagnosis is right. Their prescribed fix does not close what they demonstrated,
and this was measured rather than reasoned about.**

Applying exactly the ruling (drop `Clone`, return `Option<&NativeMethodEntry>`) and then compiling
the reviewer's own six lines unchanged:

```rust
let entry_point = load::open_path(&rxregexp()).unwrap().unwrap()
    .method(b"RegExp_Parse").unwrap().entry_point;
```

`cargo build -p rexx-api --all-targets` exits **0**. The snippet still compiles, because
`entry_point` is a `*mut c_void`: raw pointers are `Copy` and carry no lifetime, so reading the
field out of a borrowed row copies the address past the temporary's drop. A temporary in a `let`
lives to the end of its statement, so the field read is in scope and the borrow ends before the
drop. What the borrowed return *does* close is holding the **row**: the same probe with
`let row = <temporary chain>.method(..).unwrap(); println!("{row:?}");` fails with **E0716**,
temporary value dropped while borrowed.

Both halves of that were run as one two-test file, then the failing test was deleted to confirm
the first compiled on its own; the probe file was removed afterwards.

So the fix is the ruling **plus** making the address unreachable from safe code outside the
module:

* `Clone` dropped from `NativeMethodEntry` and `NativeRoutineEntry`.
* `Library::method` returns `Option<&NativeMethodEntry>`.
* `entry_point` is now a **private** field on both row types. The public reader is
  `has_entry_point(&self) -> bool`, which is all the tests needed it for.
* There is no `routine(name)` accessor to change; `routines()` already borrowed.

Whichever task first calls an entry point adds a `pub(crate)` accessor beside `has_entry_point`,
in this module or `ffi.rs`. Handing the bare address to safe code outside the crate would re-open
this, and that is now a deliberate act rather than the default.

### The tests that pin it

`compile_fail` doctests, which need no new dependency (`trybuild` would be a second one, which
the phase forbids). Three doctests on the two items:

* `Library::method`, `no_run`: the library bound to a name, the row taken from it, the borrow
  used. Compiles.
* `Library::method`, `compile_fail`: the same but with the library left a temporary, and the
  borrow used after the statement. Must not compile.
* `NativeMethodEntry`, `compile_fail`: reading `.entry_point` off a row. Must not compile.

**The first draft of the second one was a test that could not fail**, and the instrument caught
it: written without a use after the binding, the borrow ended immediately and rustdoc reported
"Test compiled successfully, but it's marked `compile_fail`". Adding
`assert!(row.has_entry_point());` after the binding is what makes the borrow outlive the
temporary. This is exactly the shape the reviewer warned against, arrived at by a different route.

Error codes are deliberately not pinned on the `compile_fail` blocks; rustdoc only verifies an
`Exxxx` annotation on nightly, so it would read as a check and not be one. The two controls below
do that job instead.

### Controls, predicted before running

Written to `<scratchpad>/predictions.md` before either ran.

**Control D: `Library::method` reverted to owned + `Clone`.**
*Predicted:* `Library::method` compile_fail FAIL ("compiled successfully"); its `no_run` companion
PASS; `NativeMethodEntry` compile_fail PASS, the field still being private.
*Outcome:* **confirmed in every part.** `1 passed; 1 failed`, the failure being the
`Library::method` compile_fail at line 210, reason "Test compiled successfully, but it's marked
`compile_fail`". Reverted by re-editing.

**Control E: `entry_point` made `pub` again, `method` left borrowing.**
*Predicted:* `NativeMethodEntry` compile_fail FAIL; `Library::method` compile_fail PASS, E0716
being untouched by field visibility; `no_run` PASS.
*Outcome:* **confirmed in every part.** `1 passed; 1 failed`, the failure being the
`NativeMethodEntry` compile_fail at line 110. Reverted by re-editing.

Each control reddens exactly one of the two, and a different one, so neither is passing on the
other's mechanism.

## Finding 2: the false claim about drop order

Correct, and the reasoning holds: dropping a `Vec<NativeMethodEntry>` never dereferences an
address, so field order enforces nothing. The comment now states the mechanism that does:

```rust
    /// The mapping every `entry_point` above addresses. No safe code outside
    /// this module can copy an address out of a row, which is what keeps one
    /// from outliving this field.
```

The field is still last and still `#[expect(dead_code)]`; only the claim changed.

## Finding 3: `open_path`'s payload

Taken as "carry the name alongside the path", which is the type-level option and costs one
parameter:

```rust
pub fn open_path(path: &Path, name: &str) -> Result<Option<Library>, Failure>
```

`name` is what a raised `Failure` interpolates. A caller cannot now reach 98.982 with a path in
the message without having written the path as the name, which a doc note would not have
prevented. Call sites updated: `open_rxregexp` passes `"rxregexp"`, the libc control passes `"c"`.

## Commands and exit statuses

Run from `rust/`, each read unpiped, on the tree as committed.

| command | exit | note |
|---|---|---|
| `cargo fmt --all` | 0 | |
| `cargo fmt --all --check` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | warm target directory |
| `cargo test -p rexx-api` | 0 | 12 `ok` lines: 9 in `tests/load.rs`, 1 `no_run` doctest, 2 `compile_fail` doctests |
| `cargo test -p rexx-core --test unsafe_sites` | 0 | 2 `ok` lines |

## API changes since `85fd2f136`

```rust
pub struct NativeMethodEntry { pub style: c_int, pub name: Vec<u8> /* address private */ }
impl NativeMethodEntry { pub fn has_entry_point(&self) -> bool; }
pub struct NativeRoutineEntry { pub style: c_int, pub name: Vec<u8> /* address private */ }
impl NativeRoutineEntry { pub fn has_entry_point(&self) -> bool; }

impl Library { pub fn method(&self, name: &[u8]) -> Option<&NativeMethodEntry>; }

pub fn open_path(path: &Path, name: &str) -> Result<Option<Library>, Failure>;
```

Neither row type is `Clone` any more. `open`, `loads`, `check_version`, the three `#[repr(C)]`
structs and every SAFETY note are unchanged.
