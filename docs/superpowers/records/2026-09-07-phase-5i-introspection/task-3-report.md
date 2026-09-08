# Task 3 — `RexxInfo`, twenty-eight rows

BASE on arrival: `3d2c7dd75`, `git status --porcelain` empty. The controller landed `d290958a3`
(the plan correction below) and `774cf86de` (Task 2's gate cells) while this task was running;
neither touches a file this task edits, confirmed by `git show --stat`.

**Result: 26 rows implemented, 2 declined with the measurement.**

## Pre-flight, and what it changed

Eight findings went to the controller before any code. Three were blocking; the first killed a
standing ruling.

**1. `~executable` and `~libraryPath` answer `.File` objects, not path strings.** The brief, the
plan and the 2026-09-08 ruling all reasoned about a string. `RexxInfoClass.cpp`'s
`getRexxExecutable` and `getRexxLibrary` both end with
`fileClass->sendMessage(GlobalNames::NEW, pathString, result)`. Measured, oracle rc 0, fresh dir:

```
exec class File     exec string /home/moritz/dev/repos/ooRexx/build/bin/rexx
lib class  File     lib string  /home/moritz/dev/repos/ooRexx/build/lib
```

`corpus/introspection-arity.tsv` showed them as bare paths because its value comparison prints
`vv~string`, which cannot see an object's class. **That blindness is the general finding**, and the
controller wrote it into the plan's Global constraints beside the second-send rule.

Ruled `d290958a3`: decline both, revisit in Phase 7. See *Declined rows*.

**2. The ruling's harness normalisation needed `crates/rexx-exec/tests/support/arity.rs`**, which is
Task 0's instrument and not in this task's file list. Moot once the rows were declined; that file is
untouched.

**3. `Numerics::MAX_WHOLENUMBER` is private**, at `crates/rexx-num/src/settings.rs:107`, so
`internalMaxNumber`/`internalMinNumber` looked to need a `pub const` move in `rexx-num` — outside
this task's file list. **Withdrawn: no `rexx-num` change was needed and none was made.** The two are
derived from `rexx_num::ARGUMENT_DIGITS`, which is already public, because
`runtime/Numerics.hpp:85`-`:98` sets the pair together per pointer width and its own comment says
`ARGUMENT_DIGITS` is the digits setting chosen "to allow for the full range": eighteen digits beside
eighteen nines on 64-bit, nine beside nine on 32-bit. `10^ARGUMENT_DIGITS - 1` therefore holds in
*both* branches, which is strictly better than reading the 64-bit constant — it is right on a 32-bit
build, where a copy of `settings.rs`'s value would not be.

**4. `debug` must not read `cfg!(debug_assertions)`**, for two independent reasons. It would redden
a gate — `method_bodies.rs:601` runs `env!("CARGO_BIN_EXE_rexx-run")`, which is the *debug* binary
under G3 while the table is refreshed `--release`, so the row would flip between runs. And it is the
wrong mirror: `_DEBUG` is an MSVC construct, no `-D_DEBUG` appears in ooRexx's `CMakeLists.txt`, and
the oracle's `CMakeCache.txt` has `CMAKE_CXX_FLAGS_DEBUG:STRING=-g` with nothing defining it. So
`getDebug` answers `.false` for every Linux build of the interpreter, Debug included.

**5. `revision` is not a version field.** `modification` is `ORX_MOD`, the third component of
`5.3.0`; `revision` is `ORX_BLD`, which `CMakeLists.txt:87` defaults to `0` and `:143` overrides
from SVN. The brief asked which *field of the version* each one was, which presumes an answer that
is false for `revision`. Corrected in the plan by `d290958a3`.

**6. Version fields: parsed, not restructured.** Reasoning in its own section below.

**7. `caseSensitiveFiles` mirrors a fallback, not a probe.** Stated at the site and below.

**8. `maxPathLength` needs a `PATH_MAX` this crate does not have.** Named const, cited.

## What each reader reads, and where that state comes from

The phase's first NEW constraint, one sentence per row. Everything lives in
`crates/rexx-exec/src/dispatch/rexx_info.rs` unless said otherwise.

### The defaults, which are not the settings in force

| row | reads |
| --- | --- |
| `digits` | `rexx_num::Settings::default().digits()` — the settings object a program *starts* with in this crate, mirroring `Numerics::DEFAULT_DIGITS`. Not the activation's live `NUMERIC`. |
| `fuzz` | `rexx_num::Settings::default().fuzz()`, mirroring `Numerics::DEFAULT_FUZZ`. |
| `form` | The constant `SCIENTIFIC`. **A constant is genuinely right**: `RexxInfo::getForm` returns `GlobalNames::SCIENTIFIC` unconditionally and its own comment is "always scientific". Reading `Settings::default().form()` instead would agree on this interpreter and stop mirroring the C++ on one where the default form and this answer parted. |

Reading `Settings::default()` rather than a literal `9`/`0` is the point of rows one and two: the
answer moves if this crate's own starting `NUMERIC` state ever moves, which is the fact
`RexxInfo` reports.

### The version fields, all from one constant

All six are cut out of `crate::parse_template::VERSION` at compile time; see the next section.

| row | reads |
| --- | --- |
| `name` | `parse_template::VERSION` itself — the same object `PARSE VERSION` answers. `RexxInfo::initialize` assigns `Interpreter::getVersionString()`, which is that string. |
| `version` | `VERSION_NUMBER`, the `ORX_VER.ORX_REL.ORX_MOD` field. |
| `majorVersion` | `MAJOR_VERSION`, that field's first component. |
| `release` | `RELEASE`, its second. |
| `modification` | `MODIFICATION`, its third. |
| `languageLevel` | `LANGUAGE_LEVEL`, the word after the `-bit` one. |
| `date` | `BUILD_DATE`, the string's tail. |

### The constants that are genuinely constants

| row | reads, and the C++ it mirrors |
| --- | --- |
| `revision` | `BUILD_LEVEL`, mirroring `ORX_BLD`. `CMakeLists.txt:87` sets `ORX_BLD_LVL` to `0`; `:143` overrides it with `ORX_WC_LAST_CHANGED_REV` when the source is an SVN working copy. Nothing behind this crate is one, so the configured default is the answer — the same reason the oracle's own build answers it. **Independent of `modification`**: on a build where they differ, `modification` follows the version string and `revision` does not. |
| `debug` | `0`, mirroring `#ifdef _DEBUG`. `_DEBUG` is never defined for a Linux build of the interpreter — see pre-flight 4 — so this is a constant on this platform rather than a copied value, and deliberately *not* `cfg!(debug_assertions)`. |
| `platform` | `parse_template::PLATFORM`. Not a coincidence and not a second copy: `SystemInterpreter::getPlatformName()` returns `ORX_SYS_STR`, which is the same constant `PARSE SOURCE`'s first word reads. |
| `directorySeparator` | `/`, mirroring `SysFileSystem::getSeparator` (`platform/unix/SysFileSystem.cpp:1358`, which is a literal `return "/"`). |
| `pathSeparator` | `:`, mirroring `SysFileSystem::getPathSeparator` (`:1369`). |
| `endofline` | `\n`, mirroring `SysFileSystem::getLineEnd` (`:1380`). |
| `maxPathLength` | `PATH_MAX`, a named const mirroring `linux/limits.h`. `RexxInfo::getMaxPathLength` is `SysFileSystem::MaximumPathLength - 1` and `SysFileSystem.hpp:65` makes `MAXIMUM_PATH_LENGTH` be `PATH_MAX + 1`, so the answer is `PATH_MAX` exactly. Rust's standard library exposes no such limit and this workspace has no `libc` dependency. |
| `caseSensitiveFiles` | `1`. **This mirrors the C++ fallback, not its probe, and that is a real limitation.** `SysFileSystem::isCaseSensitive("/")` does `ioctl(FS_IOC_GETFLAGS)` on Linux and falls through to its own documented "non-determined, just return true" (`SysFileSystem.cpp:1335`) where that is unavailable. No `libc`, `unsafe_code = "deny"`, so the ioctl is unreachable. **A casefolded ext4 directory would make the oracle answer `0` where this answers `1`.** Recorded here rather than hidden. |

### Computed, not constant

| row | reads |
| --- | --- |
| `architecture` | `size_of::<*const ()>() * 8`, mirroring `sizeof(void *) * 8`. Correct on a 32-bit build of this crate rather than only on this one. A unit test pins it against `PARSE VERSION`'s own `-bit` field, so `~architecture` and `~name` cannot contradict each other. |

### The numeric-core limits, read from what this crate enforces

| row | reads |
| --- | --- |
| `internalDigits` | `rexx_num::ARGUMENT_DIGITS` — the precision every argument-to-machine-integer conversion in this crate runs at. |
| `maxExponent` | `rexx_num::MAX_EXPONENT` — the bound this crate's own arithmetic reports error 42 past. |
| `minExponent` | `rexx_num::MIN_EXPONENT`. |
| `maxArraySize` | `dispatch::MAX_FIXED_ARRAY_SIZE` — the same constant `.Array~new` raises 93.959 past. |
| `internalMaxNumber` | `10^rexx_num::ARGUMENT_DIGITS - 1`, mirroring `Numerics::MAX_WHOLENUMBER` — derived, not copied; see pre-flight 3. |
| `internalMinNumber` | that negated, mirroring `Numerics.hpp:87`/`:95`, where `MIN_WHOLENUMBER` is `MAX_WHOLENUMBER` negated. |

The witness asserts each of these against what the interpreter actually does rather than against
its own answer — see *Witnesses*.

### The object-valued row

| row | reads |
| --- | --- |
| `package` | `interp.package_object(Package::Rexx)`, the memoised REXX package object `.Array~package` already answers. |

## The version constant: parsed, not restructured

**Decision: parse the fields out of `parse_template.rs`'s `VERSION`.** Not restructure it into
components a renderer reassembles.

`VERSION`'s own doc comment argues that the constant is a verbatim claim about the oracle *binary*
and that `tests/parse_version_oracle.rs` is the only thing that can notice it going stale.
Restructuring would move `Version.cpp:73`'s assembly rule — the `(MT)`, the `-bit`, the field
spacing — into this crate as a second thing that can be wrong, checkable by nothing but that same
harness. Parsing makes every field structurally downstream of the one policed source: a rebuilt
oracle that moves to `5.3.1` moves `version`, `modification` and `name` together, and there is no
way for them to drift apart.

**The cuts are compile-time.** `seek()` is a `const fn` whose miss is `panic!`, so a `VERSION`
without one of its delimiters is a build failure (E0080), not a test failure. `part()` is
`split_at` in const context.

**The parse is asserted, not assumed.** `parse_template::tests::the_version_fields_reassemble_into_the_constant`
rebuilds the whole string from the fields and compares it to `VERSION`, and checks that
`major.release.modification` is `VERSION_NUMBER`. It is written as the *assembly* rather than as
six expected strings, so it holds for whatever `VERSION` the next oracle rebuild puts here — six
expected strings would be the second copy the constant's own doc rules out.

The same test pins `PARSE VERSION`'s `-bit` field against `size_of::<*const ()>() * 8`, which is
what `~architecture` reads. `__REXX64__` and `sizeof(void *)` are the same fact in the C++, and this
is what stops a `VERSION` carrying a width that disagrees with the build.

`.RexxInfo~name = v` after `parse version v` is asserted in the corpus witness, so the two readers
of the one constant cannot drift.

## Declined rows

Two, both by the controller's ruling in `d290958a3`, both revisited in Phase 7.

**`~executable`.** The path is knowable — the C++ Linux path is `readlink("/proc/self/exe")` then
`realpath()` (`common/platform/unix/SysProcess.cpp:265`-`:296`), which is exactly
`std::env::current_exe()`. It is declined for its **class**: the oracle answers a `.File`, and
`.File` is not constructible in this crate. Measured, both engines, rc 120:
`.File~new('/tmp/x')` → `rexx-exec: the LIBRARY REXX entry point "file_qualify" is not implemented (Phase 7)`.
Answering a String would agree on `corpus/introspection-arity.tsv`'s `~string` comparison and
diverge on `~class~id`, `~exists`, `~parent` and every other `File` send — a wrong answer the
instrument cannot see, which is this phase's first NEW constraint exactly.

**`~libraryPath`.** Declined for a second, independent reason: it has **no source at all** here.
`SysProcess::getLibraryLocation` (`:309`) is `dladdr` on the address of `RexxCreateQueue`, i.e. the
directory holding `librexx.so`. No crate in this workspace declares a `crate-type`, so there is no
such directory, and answering the executable's directory instead would be inventing a value.

Both rows stay `loud` and stay `send-differs`. Neither is closed by a green cell anywhere.

## Witnesses

`rust/corpus/lang/rexx_info_readers.rex`, with
`rust/crates/rexx-parse/tests/sourceline_oracle/rexx_info_readers.txt` (89 lines), filed in
`corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`.

Measured, all three descriptors, both engines, oracle from a fresh `mkdir`ed directory: **byte
identical, rc 0 on all three sides.**

What it pins that a value table cannot:

* **The defaults, and this is the task's central claim.** The program sets
  `numeric digits 5; numeric form engineering; numeric fuzz 2` **first**, then prints
  `defaults 9 SCIENTIFIC 0` on one line and `in force 5 ENGINEERING 2` on the next. A reader wired
  to the activation's live state passes every probe that leaves `NUMERIC` alone and fails here.
* **One source for the version.** `name is parse version 1` is `.RexxInfo~name = v` after
  `parse version v`.
* **The version fields as relations**, never as printed strings: `date` is `name`'s tail,
  `languageLevel` and `version` occur in `name`, and `version` is `majorVersion.release.modification`.
  Printing them would commit a differential over the oracle's build date that the next rebuild
  breaks — which is why no corpus program prints `parse version` either. The relations survive a
  rebuild.
* **The numeric-core limits against what the interpreter enforces**, each with its adjacent success:
  `datatype('1E+' || maxExponent, 'N')` is `1` and one past it is `0`, both directions; a subscript
  of `internalDigits` nines answers `The NIL object` and one digit wider is 93; `.Array~new` of
  `maxArraySize + 1` is 93; and `internalMaxNumber` is `internalDigits` nines with
  `internalMinNumber` its negation, which is the relation the two are derived through. A body answering a plausible number instead of the enforced one fails
  here while agreeing with any table that only compares the answer.
* **`package` through a second send**: the object renders `The REXX Package`, then `~name` is
  `REXX`. A program's own package renders `a Package` and answers its file path, so the second send
  discriminates.
* **The refusals**: an argument to a method declared with a count of zero is 93, and
  `~fileSeparator` — a name `RexxInfo`'s dictionary does not hold — is 97.

## Red controls

Each predicted before running, then marked.

| control | prediction | outcome |
| --- | --- | --- |
| **The 5f/5g registration control, part 1.** Add a row `("RexxInfo", "FILESEPARATOR", …)` — a name `RexxInfo`'s dictionary does not hold | panic at `ObjectModel::build` | **confirmed** — rc 101, `panicked at crates/rexx-exec/src/dispatch.rs:1265:21: NATIVE_METHODS names RexxInfo~FILESEPARATOR, which that class's behaviour does not answer`. This is what makes the `system_lookup` fallback safe: a name in neither table still panics. |
| **Part 2.** Bind `PLATFORM` to the `date` body | `.RexxInfo~platform` answers the build date, so the send really reaches the bound body | **confirmed** — `platform 30 Jul 2026` on both engines |
| **Part 3.** Remove the `REVISION` row | the name goes loud again | **confirmed** — both engines, rc 120, `rexx-exec: method "REVISION" of class "RexxInfo" is not implemented (Phase 5)` |
| **The compile-time cut.** Remove the `(` from `VERSION` | `cargo build` fails at const evaluation, not at a test | **confirmed** — rc 101, `error[E0080]: evaluation panicked: VERSION does not carry the delimiter its fields are cut at`, at `parse_template.rs:150` inside `seek` |
| Plant `fn clippy_probe(v: &Vec<u8>) -> bool { v.len() == 0 }` in the new module | clippy exits non-zero naming `rexx_info.rs` | **confirmed** — rc 101, `ptr_arg` and `len_zero` at `rexx_info.rs:109`/`:110`; restored, re-ran green. Run because a 1.48s clippy after a `touch` reads exactly like a lint that did not run. |

### The `digits`/`form`/`fuzz` control, and what it proves beyond "can fail"

Mutation: all three readers changed to `interp.activation().settings`, the live state
`builtin/state.rs`'s `DIGITS()`, `FORM()` and `FUZZ()` read.

Predicted before running: the witness goes red on its first line; the arity table's three rows stay
`agree`, because that probe never changes `NUMERIC`; nothing else in the corpus moves.

**All three confirmed.**

* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` → rc 101,
  `1 of 446 corpus programs disagree`, and the one is `lang/rexx_info_readers.rex`:
  `rust: "defaults 5 ENGINEERING 2\n…"` against `oracle: "defaults 9 SCIENTIFIC 0\n…"`.
* `cargo test --release -p rexx-exec --test introspection_arity` → **rc 0, all 25 passing.**

The second line is the one worth having. **The value-comparing arity table cannot see this defect at
all** — it sends to a fresh receiver from a program that leaves `NUMERIC` alone, so a reader wired to
the live state agrees with the oracle there. Under the mutation the whole rest of the suite plus that
table stay green and exactly one program fails. That is the difference between "the witness can fail"
and "the witness adds coverage", and it is why the witness changes `NUMERIC` before its first `say`.

### The `name` control, in two variants — and the brief's prediction for it was wrong

The brief said: break the shared source and both the `PARSE VERSION` consumers and this witness go
red. Run, that is not what happens, because the two failure modes are separate and no single
instrument catches both.

**Variant A — break the constant's VALUE.** `VERSION` changed `5.3.0` → `5.4.0`.

Predicted: `parse_version_oracle` red; the arity table red on the version rows; the corpus witness
**green**, since every assertion in it is a relation among this crate's own answers and a moved
constant moves them together; both `parse_template` unit tests green, being self-comparisons by
design. **All four confirmed:**

* `REXX_CORPUS_GATE=1 … --test parse_version_oracle` → rc 101,
  `rust: "REXX-ooRexx_5.4.0…"` / `oracle: "REXX-ooRexx_5.3.0…"`.
* `--test introspection_arity` → rc 101, `the_table_matches_the_three_sides` failing, first row
  `Row { class: "RexxInfo", method: "name", … }`.
* `REXX_CORPUS_GATE=1 … --test corpus` → **446 of 446 matching**.
* `--lib parse_template` → 12 passed, 0 failed.

**Variant B — break the LINK, leaving the constant alone.** `name`'s body changed to answer
`VERSION_NUMBER` instead of `VERSION`.

Predicted: the corpus witness red on `name is parse version`; `parse_version_oracle` green, since
`PARSE VERSION` still reads the constant. **Both confirmed:** corpus rc 101, `445 of 446 matching`,
the one being `lang/rexx_info_readers.rex`; `parse_version_oracle` 19 passed, 0 failed.

So the one-source property is pinned by two instruments doing different jobs: the corpus witness
asserts that `~name` and `PARSE VERSION` *agree*, and `parse_version_oracle` plus the arity table
assert what they agree *on*. Neither alone is enough, and the brief's sentence describes a single
control that does not exist.

After every control the file was restored from a `cp` copy and the binary rebuilt. No
`git checkout --` was used at any point. The restored tree re-measured green: corpus
**446 of 446 matching**, `parse_version_oracle` 19/0, `introspection_arity` 25/0.

## The pre-existing defect this task ran into

Not this task's, not fixed here, and confirmed by running rather than by reasoning.

**A builtin call nested in another builtin's argument list loses its own argument when that argument
is a message send.** Minimal repro, fresh dir:

```rexx
v = 'hello world'
say 'd' right(v, length(v))          /* oracle: hello world.  here: hello world */
say 'e' right(v, length(.Array~id))  /* oracle: world.        here: rc 216 */
```

Both engines, `Error 40 ... Error 40.3: Not enough arguments in invocation of LENGTH; minimum
expected is 1.` `right(v, 5 + length(.Array~id))` fails the same way; `length(length(.Array~id))`
and `right(v, length(v))` are fine, so it is not simply nesting. No `RexxInfo` is involved.

**Confirmed pre-existing at BASE `3d2c7dd75`**: the three edited files were copied to scratch,
restored from `git show HEAD:`, `cargo build --release` re-run, the repro run (rc 216, both
engines), then restored from the copies and rebuilt. No `git checkout --` at any point.

It cost this task a witness line — the first draft asserted the build date with
`right(v, length(.RexxInfo~date))`, which this defect refuses.

## What could not be asserted, and is flagged rather than hidden

`(.RexxInfo~package == .Array~package)` would pin the package's **identity**, and `==` on one of the
interpreter's own objects is loud here (Task 5's territory):
`rexx-exec: the operator == applied to one of the interpreter's own objects is not implemented (Phase 5)`.
The witness uses `~name` instead, which discriminates the REXX package from any program's package.
Identity holds by construction through `Interp::package_object`'s memo, but no corpus program can
say so today.

## Shared artifacts moved

Refreshed with the commands quoted, before/after recorded by diffing against a `cp` copy taken
before the refresh. `corpus/refusal-sites.tsv` and `corpus/unfiled.txt` are **untouched** — this
task adds no `Loud`/`Raised` constructor.

`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`

`corpus/method-bodies.txt`: 26 `RexxInfo` instance rows moved
`loud [method "<NAME>" of class "RexxInfo"]` → `answers [rc 0]`. The whole-file diff is 52 lines,
i.e. exactly those 26 rows; the count of changed lines that do not mention `RexxInfo` is **0**. The
rows that did **not** move are `executable` and `libraryPath`, which are still `loud`.

`REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity`

`corpus/introspection-arity.tsv`: the same 26 rows moved `send-differs` → `agree`; whole-file diff
52 lines, and the count of changed lines that do not mention `RexxInfo` is **0**. `executable` and
`libraryPath` remain `send-differs`. Since this table compares the answered **value**, an `agree`
row here says the two interpreters gave the same answer, not only the same status — with the
caveat pre-flight 1 established, that it cannot see the answer's class.

`corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C` in `crates/rexx-exec/tests/coverage.rs`: one line
each, `lang/rexx_info_readers.rex`.

`crates/rexx-exec/tests/dispatch_seam.rs`: `src/dispatch/rexx_info.rs` added to
`CLEARANCE_CONSUMERS`, as the global constraints require of a new file naming the seam token.

## Gates

Run in the gate worktree `/home/moritz/dev/repos/ooRexx-5i-gates`, detached at `acf617abd`
(`worktree HEAD: acf617abdc8ab6cc598b244aecb01cf1458f2fc9`, recorded by the script before G1). Each
status written unpiped to `gate-status.txt` as it went. **All seven zero.**

| gate | command | result |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0, no output on either descriptor |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | exit 0, 116 `test result: ok`, no `test result: FAILED`, 2280 passed |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0, 116 `test result: ok`, no `test result: FAILED`, 2281 passed |
| G5 | `cargo test --release -p rexx-exec --test collection_arity` | exit 0, 23 passed |
| G6 | `cargo test --release -p rexx-exec --test introspection_arity` | exit 0, 25 passed |
| G7 | `cargo test --release -p rexx-exec --test introspection_scopes` | exit 0, 22 passed |

G4's own stderr carries `mode: STRICT (the gate) -- REXX_CORPUS_GATE is set` and
`446 of 446 matching`, which is the run that includes `lang/rexx_info_readers.rex`. The one test in G4's list and not G3's, found by
`comm` over the two runs' test names, is
`bytes::tests::the_bytes_past_len_are_never_part_of_the_value`, which carries
`#[cfg(debug_assertions)]` — the reason `rust/CLAUDE.md`'s Gates section keeps the debug run beside
the release ones.

## Concerns

1. **An unowned pre-existing defect** — the nested-builtin argument loss above. It is not
   `RexxInfo`'s and nothing in this phase owns it.
2. **`caseSensitiveFiles` is the one row of the 26 whose agreement is conditional on this
   filesystem.** It mirrors a fallback because the C++ probe needs an ioctl this workspace cannot
   make. On a casefolded ext4 directory the oracle would answer `0` and this crate `1`.
3. **`==` on a native object is loud**, so the identity of `.RexxInfo~package` and `.Array~package`
   is unasserted from Rexx. It holds by construction through `Interp::package_object`'s memo. Worth
   revisiting when Task 5 lands `Class`'s operators.
4. **`corpus/introspection-arity.tsv` cannot see an answer's class.** Established here by
   `~executable`, now in the plan's Global constraints. Every remaining task in this phase with an
   object-valued row inherits the problem, and `Method`, `Routine`, `Package` and `StackFrame` all
   have them.
5. **The brief's `name` red control describes something that does not happen.** Corrected above and
   worth carrying into the phase close, since the same sentence would mislead a later task told to
   "break the shared source".
