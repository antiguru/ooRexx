# Phase 8, the surface half — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to
> implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the ooTest framework load, fill the native API surface the extension-only ooTest
groups reach, and pass `METHOD`, `CONVERSION` and `FUNCTION` against the oracle's own prebuilt
test extensions.

**Architecture:** No new crate and no new grant. `rexx-api` already holds the boundary, with
`unsafe` confined to `ffi.rs` and `load.rs` by D-U1, a conversion table keyed by the extension's
declared types, and a handle registry in which a stale `RexxObjectPtr` misses. This half widens
each of those along seams the L2 slice built, and first closes the one gap outside the boundary
that stops every ooTest group from loading at all.

**Tech Stack:** Rust; `rexx-api`, `rexx-exec`; `libloading`; no new dependency.

**Spec:** `docs/superpowers/specs/2026-09-14-phase-8-native-api.md`, whose evidence base is
`docs/superpowers/specs/2026-09-14-phase-8-scoping.md`. The L2 slice's close is
`docs/superpowers/plans/phase-8-gate.md`, its measurement `docs/superpowers/plans/phase-8-l2.md`,
and its whole-branch review the three `final-review-*.md` reports under
`docs/superpowers/records/2026-09-14-phase-8/`.

**Amended 2026-09-15, before execution**, from the L2 slice's final review and one ruling by
Moritz of 2026-09-14. What changed and why:

* **Task 1 is new.** Every ooTest API group requires `ooTest.frm`, which stops at `:49` on
  `.local~hasEntry`; nine `Directory` methods refuse on `.local` and `.environment` (D-L2). Moritz
  ruled that this phase implements them: "It's an oversight we didn't do it yet." He recalls a
  blocker for `.environment` and not what it was, so the task finds it by running before it
  designs.
* **The group partition was wrong, and it is now derived.** The spec and this plan had two
  embedding groups and six that are not. The groups' own directives and the oracle build's
  `readelf -d`/`nm -D` say otherwise (the scoping survey's section 3 carries the commands): `METHOD` and `CONVERSION` load
  `orxmethod`, `FUNCTION` loads `orxfunction`, and neither library needs an interpreter library or
  imports a `Rexx*` symbol. `RexxStart`, `ProcessRexxStart`, `INVOCATION` and `ProcessInvocation`
  all load `INVOCATIONTester.cls`, which binds `orxinvocation`, which NEEDs `liborxexits.so`, which
  NEEDs `librexx.so.4` and imports `RexxCreateInterpreter` and `RexxStart`: embedding, Phase 9's.
  `CLASSIC` loads `orxclassic` and registers `orxclassic1` through `rxfuncadd`; between them they
  import the function, subcom, queue and macro-space registries: Phase 10's. Roadmap rows 9 and 10 and
  `phase-4-exclusions.txt` carry that re-homing with its commands since `313807bc5` and `3bb1d34d2`.
* **"`testbinaries/` compile unchanged" is already witnessed**, by the oracle's own build of
  sources byte-identical to this tree's against headers byte-identical to this tree's. Building them
  against this crate would need a `librexx.so`, which is Phase 9's SONAME question. Task 7 records
  the measurement instead of attempting that build.
* **The routine half widens** (Task 2): a library's routines are registered only by `::REQUIRES
  ... LIBRARY`, so `Package~loadLibrary`, `::ROUTINE ... EXTERNAL` and `.Routine~loadExternalRoutine`
  load the library and then answer 43.1 where the oracle calls the routine.
* **The thread context gets an interpreter lifetime before the thread table is filled** (Task 4):
  a forged extension that keeps `context->threadContext` reads a freed stack frame, confirmed under
  AddressSanitizer. The package `loader` and `unloader` hooks, which receive a thread context with
  no call in flight, join that task: the crate runs neither today, and the oracle runs a held
  package's unloader at termination.
* **The exit and IO-redirector interfaces are reached from `FUNCTION`**, through
  `AddCommandEnvironment` (`testbinaries/orxfunction.cpp:750-768`), not through `orxexits`, whose
  exit registration is embedding.

## What the L2 slice established, which this half inherits

* **Nothing unwinds across the boundary.** `RaiseException` records a pending condition and returns
  to the extension; entry points stay `extern "C"` and a Rust panic aborts. A native method that
  raises still runs to completion and still returns a value.
* **A panic inside a callback aborts the whole binary** rather than failing one test, so a control
  that can only redden by abort gives a red with no name. Observe what a callback did after the
  call returns.
* **The conversion table is what drives conversion**, and deleting a row must redden exactly that
  row's tests. A second code-keyed match written outside the table is invisible to that control.
* **`values::repr` names the union member for a code.** Use it rather than a new match.
* **`NativeMethodEntry::entry_point` is private**, with `has_entry_point()` the only public reader,
  because an owned copy of that address outlives its `Library`. Add `pub(crate)` accessors, never
  `pub`.
* **`Interp::libraries` exposes only `get` and a non-replacing `hold`**, in its own module so the
  field is private to it.
* **A native argument's string value is `requiredString`'s**, `REQUEST('STRING')` and nothing
  else, not the operator protocol that falls back to the default name. The final review found the
  host using the wrong one, silently, on every string parameter.
* **Blame a directive through the helper that names its package.** Two helpers sit side by side
  in `lib.rs`; the one that names the running program was used where the declaring package was
  owed, and only a two-file witness sees the difference. Every witness for an error a directive or
  a native boundary raises declares the directive in a required package, not in the program that
  runs.
* **Every interface table slot is an `unsafe extern "C" fn`**, and `ffi::value_of` is `unsafe`: a
  safe path from a table to a callback let safe code hand it a forged context. A slot this half
  fills keeps that type, and a Rust test that calls one lives in `ffi.rs` or `load.rs`.
* **A library's shared code follows the oracle's objects.** A method's code is one object per
  spelling asked for; a routine's is one object per routine table entry, found by exact spelling
  and then without regard to case; the first directive that binds either names its package, as that
  directive resolves, and a `loadExternal*` object reads it retroactively. A package whose
  translation raised is not kept and has no routine, method or resource table and no prolog.
  `Libraries` holds a library that loaded, including one refused for its version, which answers
  loaded later with no routines. The final review and its rounds found a silent wrong answer in
  every one of these, so a task touching them probes past its witnesses.
* **A number is measured or it is not written.** Five error numbers taken from
  `interpreter/messages/RexxErrorCodes.h` named paths nothing runs, against none taken from a run.
* **Check every citation by printing that line**, including citations into files this plan itself
  edits: the roadmap line a decision cited moved under the phase's own insertions.

## Global Constraints

* **`unsafe` in `rexx-api` in exactly two files**, `rexx-api/src/ffi.rs` and `src/load.rs`, each
  block with a `SAFETY:` note naming the invariant and who establishes it (D-U1, Moritz 2026-09-14).
  `crates/rexx-core/tests/unsafe_sites.rs` is the record, is file-granular, and scans `tests/`.
* **No new dependency** beyond `libloading`.
* **The headers are frozen.** No edit under `api/`; one that seems necessary reopens D5.
* **The C++ tree, `build/`, `samples/`, `ootest/`, `oodocs/` and `testbinaries/` are read-only.**
  No extension is ever rebuilt: loading the oracle's own compiled extensions is the instrument, per
  the D5 amendment. The corpus differential loads them from the oracle checkout's `build/lib`.
* **Never load a prebuilt extension that NEEDs `librexx.so` or `librexxapi.so` into this crate,
  directly or through another library it NEEDs.** `liborxexits.so`, `liborxclassic.so` and
  `liborxclassic1.so` do directly, and `liborxinvocation.so` does through `liborxexits.so`; `dlopen`
  would map the oracle's own interpreter into this process, and a green result would be the
  oracle's. Before the first load of any library not named in this plan, follow its `readelf -d`
  `NEEDED` entries transitively, or run `ldd` on it.
* **No process-global state is mutated.**
* **Correctness is byte-identical stdout, stderr and exit status against the C++ oracle**, on three
  separate descriptors, never `2>&1`, through the standard wrapper from a fresh empty directory.
* **Probe hazards.** Read `rust/corpus/oracle-crashes.txt` first and never run its entries.
  `testOORexx.rex` with no arguments writes fixture files into `ootest/`; use the single-group form.
* **Comments minimal** (rust/CLAUDE.md, Moritz 2026-08-27), **never stating a set's size**, no
  em-dashes, every citation landing on its subject.
* **Every extent claim is derived and its command committed.**
* **Write each negative control's prediction before running it.**
* **A change to `rexx-api/src/ffi.rs` or `load.rs` runs `rexx-api`'s lib tests under Miri's Stacked
  Borrows** before it commits. Miri is not installable through `rustup component add` here; the
  L2 slice's fix round installed it into a scratch `RUSTUP_HOME` (its report says how). Tree
  Borrows passed the tree the F8 finding was about, so it is not a substitute.
* **The phase gates green at the closing commit**, and commit before a long gate run.

---

### Task 1: `.environment` and `.local` answer the whole `Directory` surface

`owns` (`rust/crates/rexx-exec/src/dispatch/hash.rs`) answers false for any `Body::Native`
receiver, and `.environment` and `.local` are `Body::Native` directories on `NativeObject`'s map,
so every store-family method but `at` and `put` reaches `not_this_task`: `method "HASENTRY" of
class "Directory" is not implemented (Phase 5)`. On a plain `.Directory~new` all nine D-L2 names
answer byte-identically. Phase 5h Task 4 kept these two `Body::Native` deliberately
(`docs/superpowers/records/2026-09-07-phase-5h-mapped-collections/task-4-report.md`).

**Files:** `rust/crates/rexx-exec/src/environment.rs`, `src/dispatch/hash.rs`, and whatever the
design step names; witnesses under `rust/corpus/lang/`.

- [ ] **Step 1: Find the blocker by running, before designing.** Moritz remembers one and not
      what it was. Each candidate gets a probe on both sides and a verdict in the report:
      (a) iteration order -- the oracle's `.environment~allIndexes` is bucket order, identical
      across runs, `LOCAL` last, and `.local`'s is `SYSCARGS INPUT TRACEOUTPUT DEBUGINPUT STDOUT
      OUTPUT STDERR STDIN STDQUE ERROR`; a `NativeObject` map cannot reproduce that; (b) `.local`'s
      streams and monitors are minted on first demand, so a whole-collection read must see them
      as present, and `STDQUE` is Phase 10's external queue; (c) `EnvironmentModel::unbuilt` is
      computed once and never shrinks as the library bootstrap installs names; (d) the D45 security
      chokepoint every `.local`/`.environment` read passes; (e) the cost of `.NAME` resolution,
      which reads `.environment` on every class reference: measure `rexxcps` and a `.NAME`-heavy
      loop before and after, interleaved, a few runs each; (f) anything else the Phase 5 records
      say (`docs/superpowers/records/2026-08-*-phase-5a*/`, `2026-09-07-phase-5h-*`).
- [ ] **Step 2: Choose the representation and write the choice into the report before code.** The
      likely shape is the store `dispatch/hash.rs` gives a `.Directory~new` instance, with its
      string-value key protocol, filled in the oracle's
      insertion order at the oracle's bucket size, with the security chokepoint and `.local`'s
      minting kept on the lookup path. If Step 1 shows a cheaper shape reproduces the order, take
      it. Whatever answers must answer for every `Directory` method, not only the nine: enumerate
      them from `Directory`'s method table at test time.
- [ ] **Step 3: Witnesses.** Every `Directory` method on `.environment` and on `.local` against
      the oracle: `allIndexes`, `allItems`, `supplier`, `makeArray` and `items` compared as
      ordered output; the entry family's upper-casing; `setEntry` with no value removing; a
      program-added entry; `remove` of a class name and a later `.NAME` for it; `.local`'s minted
      names before and after first use. Where `STDQUE` makes an answer Phase 10's, the refusal
      names Phase 10 and a witness pins that.
- [ ] **Step 4: Controls.** Restore `owns`'s `Body::Native` early return and predict which
      witnesses redden; reverse the insertion order and predict which ordered witnesses redden.
- [ ] **Step 5: Walk the framework.** Run `ooTest.frm`'s loading against this crate with the
      single-group form of `METHOD` and record, in the report and in `phase-8-l2.md`'s successor
      section, the next place it stops and who owns it. **Do not fix what that finds in this task.**
- [ ] **Step 6:** `closed_phases.rs` gains `"Phase 5"` if no refusal names it any more; otherwise
      list what still does, each with its owner, in the report. Commit.

---

### Task 2: The routine half of the calling protocol, at every load site

`::ROUTINE EXTERNAL` and a `::REQUIRES ... LIBRARY`'s routines install and then refuse loudly at the
call. **And a library's routines become callable only through `::REQUIRES ... LIBRARY`**: measured
by the final review with `rxmath`, `Package~loadLibrary('rxmath')`, `::routine sq external "LIBRARY
rxmath RxCalcSqrt"` and `.Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt')` each load
the library and then `RxCalcSqrt(16)` answers 43.1 at rc 213 where the oracle prints `4`. The
`Method` and `Routine` objects `loadExternalMethod`/`loadExternalRoutine` answer are shells whose
use refuses naming Phase 5.

**Files:** `rust/crates/rexx-api/src/invoke.rs`, `src/layout.rs`, `rust/crates/rexx-exec/src/lib.rs`,
`src/dispatch.rs`, `src/dispatch/package.rs`

- [ ] **Step 1:** `RexxRoutineEntry` carries a `style`: `ROUTINE_TYPED_STYLE` uses the same two-call
      protocol as a method, `ROUTINE_CLASSIC_STYLE` is the `RXSTRING` convention
      (`api/oorexxapi.h:190-208`). Build the typed style; leave classic refusing and name Phase 10,
      since its consumers are `rxsubcom` and the registered-function API.
- [ ] **Step 2:** Populate `CallContextInterface`, which is what a routine receives.
- [ ] **Step 3:** Register a library's routines at the one resolution path every load site shares,
      not at one caller of it, so every load site (the fix re-review enumerated nine for a first
      ask) makes the routines callable by name from any
      package, including one loaded before the library was. Measure on the oracle which packages
      see a routine a library loaded inside a required package's routine made available.
- [ ] **Step 4:** A `Routine` from `loadExternalRoutine` answers `~call`, and a `Method` from
      `loadExternalMethod` runs when a class defines it.
- [ ] **Step 5:** Delete the refusals, and update the expectations in `run/tests.rs` that assert
      their text.
- [ ] **Step 6:** Witnesses, using `rxmath` (routines) and `rxregexp` (methods): a typed routine
      answering through each load site; the classic style refusing; the two loaded objects used;
      each with `sourceline_oracle` companions and `corpus/phase-8.txt` rows. Commit.

---

### Task 3: The conversion table's remaining rows

**Files:** `rust/crates/rexx-api/src/values.rs`, `src/invoke.rs`, `tests/values.rs`

- [ ] **Step 1:** Fill every `REXX_VALUE_*` row `NativeActivation::processArguments`
      (`interpreter/execution/NativeActivation.cpp:219`) handles, one row at a time, each with the
      `Repr` its code uses.
- [ ] **Step 2:** `ARGLIST` carries `usedArglist` (`NativeActivation.cpp:680`), so the too-many
      check in `invoke::method` is skipped for a signature that takes one.
- [ ] **Step 3:** The special codes as *return* types answer `Failure::ResultSignature` (93.968,
      reported with its line against the sender, as F9 measured for the result side), which is
      `valueToObject`'s `default:`, rather than `Unfilled`. Not `Failure::Signature`, which is the
      argument side's lineless delivery against the declaring package.
- [ ] **Step 4:** Per-row tests both directions, and the row-deletion control re-run on a row this
      task adds rather than one it inherited. Commit.

---

### Task 4: A thread context that lives as long as its interpreter

`Contexts` (`rust/crates/rexx-api/src/ffi.rs`) owns the `RexxThreadContext_` and its table for one
`invoke::method`. The oracle's thread context is the activity's (`ActivityContext`, `interpreter/concurrency/ActivationApiContexts.hpp:64-68`),
valid for the attached thread, and `RexxPackageLoader` is handed one with no call in flight
(`oorexxapi.h:259`). The final review's forged extension, keeping `context->threadContext` across
two calls, aborts rc 134 where the oracle prints `x 42`; AddressSanitizer names
`stack-use-after-return` in `ffi::whole_number_to_object`.

**Files:** `rust/crates/rexx-api/src/ffi.rs`, `src/invoke.rs`, `rust/crates/rexx-exec/src/dispatch/library.rs`

- [ ] **Step 1:** Give the thread context and its table the interpreter's lifetime, and have each
      callback resolve the *current* native frame from the owner, which is what
      `contextToActivation` does for a thread context (`interpreter/concurrency/Activity.hpp:458`). A callback through a thread context with no
      native frame in flight gets the oracle's answer for that, measured.
- [ ] **Step 2:** A unit test that keeps the thread context across two calls, in the crate's own
      tests (not the corpus, which loads only the oracle's extensions). Under the old design it
      must fail; say how, given that the failure is a use-after-return and not an assertion. Run
      it under Miri as well.
- [ ] **Step 3:** The package `loader` hook, run with that thread context after the library's
      routines register (`interpreter/package/LibraryPackage.cpp`, `loadPackage`; print the lines),
      never for a library refused for its version; and the `unloader` at termination for every held
      library, measured on the oracle for order and for what a raise inside one does. Say which
      shipped extensions declare either hook, from their package entries, since a hook no shipped
      extension declares can be witnessed only by a unit test. **A hook runs extension code outside
      `invoke::run`**, which is today the only place a refusing slot's record is read (Task 2's fix
      round 1): a refusal recorded inside a hook must be read and refused loudly there too, not
      cleared unread, and the record cell's doc must say so. Its re-review named this path.
- [ ] **Step 4:** Commit.

---

### Task 5: The thread, method-context and call-context tables

**Amended after Task 2**, by the ruling recorded in the ledger (Task 2 pre-flight, Q2): Task 2
populated `CallContextInterface` with `GetContextDigits`, `GetContextFuzz` and `GetContextForm`, the
thread table with `DoubleToObjectWithPrecision`, and the instance table with `InterpreterVersion`
and `LanguageLevel`; this task takes the rest of the call-context members `orxfunction.cpp` calls,
beside the thread and method tables. `AddCommandEnvironment` stays Task 6's.

**Files:** `rust/crates/rexx-api/src/ffi.rs`, `src/layout.rs`, `tests/layout.rs`

- [ ] **Step 1:** Enumerate, at test time and from the header, the pointers `orxmethod.cpp` and
      `orxfunction.cpp` call, across the thread, method-context and call-context tables, and fill
      those first. The rest of the 211 get bodies grouped by what
      they touch: object construction, string and buffer access, collections, variables,
      conditions, and the interpreter's own state. Each group is its own commit.
- [ ] **Step 2:** `RequestGlobalReference` and its table. The handle is a function of the object, so
      a global and a local reference to the same object are the **same pointer**: a boundary
      `resolve` must consult both tables rather than choose one.
- [ ] **Step 3:** A derived test asserting no stub remains that a populated table should have
      replaced, read from the header at test time rather than from a list.
- [ ] **Step 4:** A test comparing each slot's *signature*, not only its name, against the header:
      the final review's `sigcheck.py`, copied into `docs/superpowers/records/2026-09-14-phase-8/`,
      found none wrong, and nothing in the tree would see one.
- [ ] **Step 5:** Commit per group.

---

### Task 6: The exit and IO-redirector interfaces, as a command handler reaches them

`FUNCTION`'s `TestAddCommandEnvironment` (`testbinaries/orxfunction.cpp:750-768`) registers a
direct and a redirecting command handler through `AddCommandEnvironment`; the handlers receive a
`RexxExitContext` and a `RexxIORedirectorContext`. That is this phase's consumer. Exit
*registration* (`REGISTERED_EXITS`, `DIRECT_EXITS`, `RexxRegisterExitExe`) is embedding or RXAPI and
is not.

**Files:** `rust/crates/rexx-api/src/ffi.rs`, `src/layout.rs`, `rust/crates/rexx-exec/src/`

- [ ] **Step 1:** `RexxInstanceInterface::AddCommandEnvironment` and the call and method contexts'
      forwarders to it; the handler's registration meets Phase 7's `ADDRESS` handler table.
- [ ] **Step 2:** `ExitContextInterface` as a command handler receives it, and
      `IORedirectorInterface`, which meets Phase 7's `ADDRESS ... WITH` work.
- [ ] **Step 3:** Witnesses against the oracle through `orxfunction`'s handlers, including a
      redirecting handler reading and writing each stream. Commit.

---

### Task 7: The prebuilt test extensions, and the group partition

**Files:** `docs/superpowers/plans/phase-8-gate.md`, a derived test beside `rexx-api/tests/load.rs`

- [ ] **Step 1:** Record, with the commands, that `api/` and `testbinaries/` are byte-identical
      between the oracle checkout and this tree, and that the oracle's `build/lib` holds the six
      library products and its `build/bin` the `rexxinstance` and `provoke_locks` executables. **This is what "compile unchanged against frozen headers" is
      witnessed by**, and the gate document says so, and says that a build succeeding says the
      headers are compatible, not that the entry points behind them work.
- [ ] **Step 2:** A test that reads each API group's `EXTERNAL "LIBRARY <name>"` and `rxfuncadd`
      directives, and each named library's `NEEDED` entries followed transitively and its
      undefined `Rexx*` symbols, and asserts the partition against a list the test itself holds:
      the groups whose libraries need no interpreter library are exactly `METHOD`, `CONVERSION`
      and `FUNCTION`, and every other group maps to the phase its imports belong to (embedding to
      Phase 9, the RXAPI registries to Phase 10). Derived, so a group that changes its library
      fails the test. Task 8 runs the groups that list names.
- [ ] **Step 3:** The re-homing is already recorded (`313807bc5`, `3bb1d34d2`); make the test's
      partition and those records agree, and where they do not, correct the records. Commit.

---

### Task 8: `METHOD`, `CONVERSION` and `FUNCTION`

**Files:** `rust/corpus/phase-8.txt` and witnesses

- [ ] **Step 1:** Run each with the single-group form, on both sides.
- [ ] **Step 2:** Enumerate the failing set rather than counting it, and fix or record each member
      with an owner phase.
- [ ] **Step 3:** Commit.

---

### Task 9: Close the phase

- [ ] **Step 1:** The phase gates from the run, unpiped, both failing sets enumerated and every
      member attributed, plus `rexx-api`'s lib tests under Miri's Stacked Borrows. Say in the gate
      document whether Miri is a gate or a recorded run, since the gate hosts cannot install it
      through `rustup`.
- [ ] **Step 2:** `closed_phases.rs` gains `"Phase 8"`, and `dispatch::native`'s `OPEN` list loses
      it. Every refusal naming Phase 8 must be gone or re-homed with a reason.
- [ ] **Step 3:** Write `phase-8-gate.md`'s surface section from the run, update the roadmap's row 8
      to the partition Task 7 derived, and move the `Rung` column: **if L2 is still blocked, say
      where and by whose work rather than closing over it.**
- [ ] **Step 4:** Commit.
