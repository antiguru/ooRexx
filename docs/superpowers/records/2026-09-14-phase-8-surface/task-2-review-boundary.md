# Task 2 review, boundary slice (`rust/crates/rexx-api/`)

Base `e7cb210d9`, head `c23c214f3`. Slice: `invoke.rs`, `load.rs`, `ffi.rs`, `values.rs`,
`layout.rs` and `tests/`. The package diff was read in passes: `invoke.rs` and `ffi.rs` as whole
files at head; `load.rs` and `values.rs` at head beside their `git diff`; `layout.rs` and the three
small test files from the diff alone; `tests/invoke.rs` and `tests/values.rs` from the diff; then the
four extensions' C++ for the slot list. `rexx-exec/src/dispatch/library.rs:60-260` and
`error.rs:336-355` were read only for the boundary risks named below.

Scratch: `scratchpad/t2-review-a/` (probe `f1`-`f5` each in its own directory with `run-o`/`run-c`
fresh run directories, three descriptors as `o.out o.err o.rc` / `c.out c.err c.rc`; `ab.sh` is the
runner; `slots.py` and its output `slots.txt` are the slot map; `arch/` is the `git archive
c23c214f3 rust` copy with `interpreter api build extensions ootest testbinaries` symlinked beside it;
`miri.txt` and `m1.txt` are the Miri run and the one mutant). Read `corpus/oracle-crashes.txt` first;
nothing below is one of its entries. Nothing registered with rxapi.

## Log

- Read the brief, the Q1/Q2 rulings, the report, `oracle-crashes.txt`; then the slice.
- `cargo test -p rexx-core --test unsafe_sites`: 2 passed (ran). `grep unsafe` over `invoke.rs`,
  `values.rs`, `layout.rs`, `tests/`: only the `unsafe extern "C" fn` slot types in `layout.rs`.
- `cargo build --release --bin rexx-run` at head: up to date, binary unchanged (the last commit
  touched only `refusal-sites.tsv`).
- Probes f1 (formatting, 31 rows), f2 (argument refusals, 30 rows), f4 (`NUMERIC` seen through an
  internal routine and a method): **identical on all three descriptors**. f3 (`RxCalcSin(30, 3,
  'X')`): oracle rc 168 / 88.916, crate rc 134 with `before` lost, as the report says. f5 (a
  `orxmethod` method reading the instance): oracle rc 0, crate **rc 139 with both descriptors
  empty** (new finding, I2).
- Miri, Stacked Borrows, in the archive copy: 20 passed, 0 failed, 3 ignored, exit 0; the routine's
  numeric test among the passes (`miri.txt:23`).
- Mutant M1 in the archive copy (`get_context_fuzz` answering digits), prediction written first:
  exactly the predicted test red, 22 passed, 1 failed; tree restored and `cmp`-checked.

### Spec Compliance (✅/❌ with file:line; ⚠️ unverifiable)

- ✅ **Step 1, style on the row and classic refused before the stub.** `load.rs:41-45` (the two
  header constants), `load.rs:212` (`NativeRoutineEntry::style`), `invoke.rs:85-87` (the check
  precedes the signature call); `invoke.rs:580-586` asserts no event reaches the stub and
  `error_number(false) == None`. The refusal names Phase 10 at
  `rexx-exec/src/dispatch/library.rs:226-231`.
- ✅ **Step 2, `CallContextInterface` populated.** `layout.rs:663` (`populated`), `ffi.rs:126-132`
  (`CALL_CONTEXT`), the three slots `ffi.rs:397-415`; `tests/layout.rs:228-231` pins the version;
  the old refusing `call_context_interface()` and its test row are gone (`layout.rs` diff,
  `tests/layout.rs:374-376` diff), consistent.
- ✅ **Ruling S2, one shared `run`.** `invoke.rs:98-142`; the too-many check is the one at
  `invoke.rs:131-133` (the oracle's `inputIndex < _argcount && !usedArglist`,
  `NativeActivation.cpp:680`); `invoke.rs:590-598` witnesses it on the routine path. `usedArglist`
  has one place to land: `run` sees the whole signature, so Task 3 can gate the check on an
  `ARGLIST` word there without touching either caller.
- ✅ **Ruling Q1, the four rows.** `values.rs:785-793` (`double`), `:961-969`
  (`positive_wholenumber_t`), `:760-768` (`RexxObjectPtr` from native), `ffi.rs:208` and `:385-392`
  with `values.rs:576-582` (`DoubleToObjectWithPrecision`). Per-row tests `tests/values.rs:981-1155`
  and, through the oracle's own `librxmath.so`, `tests/invoke.rs:498-568`. Several precisions
  witnessed: `corpus/lang/library_routine_requires.rex` rows c, d, e, n (3, 16, 20, 10^12, 3.0) and
  digits 20/3/9. Against the C++: `getDoubleValue` raises `Error_Invalid_argument_double` at
  `position + 1` (`NativeActivation.cpp:2072-2080`), `positiveWholeNumberValue` uses
  `objectToSignedInteger(o, temp, MAX_WHOLENUMBER, 1)` (`:1922-1931`), `valueToObject` for the
  object codes answers the handle (`:723`); the rows match. ⚠️ C1 (the row-deletion control) is
  the report's; I ran M1 instead (below).
- ✅ **Ruling Q2, the call context wired, three members filled, the rest refusing.**
  `ffi.rs:262-270` (`Contexts::call`, the pointer taken from the whole `Owned`), `ffi.rs:397-415`;
  the rest refuse through `layout.rs:381` ("is not implemented (Phase 8)"), which f3's backtrace
  shows for `NewStringFromAsciiz`.
- ✅ **`unsafe` in exactly `ffi.rs` and `load.rs`, each block with a `SAFETY:` note.**
  `unsafe_sites` passed (ran); every new block carries one (`ffi.rs:385-421`, `load.rs:228-334`).
  One note names an invariant that lives outside its module (I1).
- ✅ **Slot types stay `unsafe extern "C" fn`.** `layout.rs:392-397`; every `call(` row in
  `layout.rs` takes its `*mut Rexx…Context_` first (grep, no exception).
- ✅ **Miri under Stacked Borrows for the `ffi.rs`/`load.rs` change.** Ran in the archive copy:
  `scratchpad/t2-review-a/miri.txt`, `20 passed; 0 failed; 3 ignored`, `miri exit 0`; the ignored
  three are the child-process test and the two that open the running image.
- ✅ **No new dependency; headers and read-only trees untouched.** `git diff --stat e7cb210d9
  c23c214f3` over `api interpreter build extensions ootest testbinaries samples oodocs`, the crate's
  `Cargo.toml` and `Cargo.lock`: empty.
- ✅ **Comment rules in the slice.** No em-dash in the added lines (0); no set-size count found (the
  one "two calls" names the header's protocol, not a repo aggregate); dated measurements only where
  they state a property (`values.rs:163-167`, confirmed by f2).
- ✅ **Risk 4, the rows against the oracle.** f1/f2/f4 identical on all three descriptors (ran):
  NaN (`nan` for `sqrt(-1)`, `log(-1)`), `+infinity`/`-infinity` (overflow, `exp(710)`, `log(0)`),
  negative zero (`RxCalcTanH('-1e-400')` and `RxCalcSin('-1e-400')` both `0`), denormals
  (`exp(-745)` `4.94065646E-324`, `sqrt(1e-320)` `9.99994434E-161`), `exp(700)`
  `1.01423205E+304`, precision above the digits (`RxCalcSqrt(2, 17)` `1.414213562373095`,
  `(1e-320, 16)` `9.999944335758490E-161`), digits 1 (`1 1E+10 3`), digits 20, engineering form;
  refusals 88.921 / 88.905 / 88.901 / 88.922 with the same position and `found` text for `'NAN'`,
  `'Nan'`, `'+INFINITY'`, `'infinity'`, `'1.5.2'`, `''`, `' '`, `.nil`, `10^21`, `-1`, `0`,
  `2.5`, a NUL-bearing string, and `MAX_WHOLENUMBER` itself accepted. Precision 0 is unreachable
  from a program through `rxmath` (both sources are at least 1) and stays ⚠️ unprobed.

### Strengths

- `signature_of` and `call_stub` (`load.rs:273-334`) are generic over the context type, so the
  publish/unpublish of the descriptor array and the bounded signature walk exist once and the
  routine row gained no second copy of either `unsafe` body.
- `a_routine_runs_the_two_calls_a_method_runs` (`invoke.rs:561-575`) asserts the event order, the
  interned count and that the array is off the context afterwards; with `numeric_stub`
  (`ffi.rs:618-640`) the routine path is witnessed end to end through both tables and through Miri.
- The `rxmath` integration tests (`tests/invoke.rs:498-568`) assert what the extension asked
  `DoubleToObjectWithPrecision` for, `(4.0, 12)`, `(√2, 3)`, `(√2, 16)`: that is the
  `argumentExists(2)` read through the published array and the extension's 16 cap, observed after
  the call returns rather than inferred.
- `Failure::InvalidDouble`/`NotPositive` carry the argument handle, so the host renders `found
  "…"` from the object rather than from a copy made at conversion time.

### Issues

**Critical:** none found in the slice.

**Important**

- **I1 (`load.rs:228-233`, `:237-249`, `:253-263`; inferred by reading, no path reaches it
  today).** `NativeRoutineEntry::signature` and `::call` are safe `pub(crate)` functions whose
  soundness depends on a check made in another module: the SAFETY note at `load.rs:257-261` says
  "both readers above are reached only past the style check", and that check is
  `invoke::routine`'s (`invoke.rs:85`). A classic row's `entryPoint` is a function of the
  `RXSTRING` signature (`api/oorexxapi.h:190-208`), so any crate code calling
  `library.routine(name)?.signature(&ctx, n)` on a `ROUTINE_CLASSIC_STYLE` row transmutes it to the
  typed stub type and calls it with `(context, NULL)`. The crate's own bar for an `unsafe` site
  (`rust/CLAUDE.md`, "checkable by reading that module alone") is not met, and it is the same class
  the L2 review closed for contexts. Fix: `stub()` answers `None` unless `self.style ==
  ROUTINE_TYPED_STYLE` (so a classic row publishes no signature and calls nothing, whatever the
  caller checked), keep `invoke::routine`'s check for the named refusal, and add the negative
  test: a classic row's `signature` answers `None` and the stub records no entry.
- **I2 (`ffi.rs:217`; ran, f5).** `RexxThreadContext_.instance` is handed out null. The header
  routes `InterpreterVersion`, `LanguageLevel`, `AddCommandEnvironment` and `applicationData`
  through it (`api/oorexxapi.h:832-847`), and two shipped test libraries reach them from a
  program: `orxmethod`'s `TestInterpreterVersion`/`TestLanguageLevel`
  (`testbinaries/orxmethod.cpp:2221`, `:2233`) and `orxfunction`'s `TestAddCommandEnvironment`
  (`testbinaries/orxfunction.cpp:751`). Measured, `::method v external "LIBRARY orxmethod
  TestInterpreterVersion"`: oracle rc 0 printing `before` / `328448` / `after`; crate **rc 139,
  stdout and stderr both empty**. That is not a refusal: it is the one path in the boundary that
  answers by crashing, with no name and with the program's earlier output lost. Predates this task
  (methods ran since L2) and the slots are Task 5's, but the phase cannot close over it. Fix: make
  `RexxInstanceInterface` `populated` and hand out an `Owned<RexxInstance_, …>` whose `functions`
  is its `REFUSING` table, so the call refuses loudly today and falls under the I3 design tomorrow.
- **I3, the refusing-slot design (risk 1; list ran through `slots.py`, f3 ran).** The map: each
  `context->X` in the four sources through the header's inline wrappers (`api/oorexxapi.h:825-3347`)
  to its table slot, against the filled sets at `ffi.rs:115-132` and `:203-213`. Full output in
  `scratchpad/t2-review-a/slots.txt`; the script takes the first `String` overload, and rxmath's
  one-argument `String(u)` is `NewStringFromAsciiz` (`api/oorexxapi.h:2896-2898`), which f3's
  backtrace confirms.
  * `rxmath` (call context): thread `NewStringFromAsciiz`, `ArrayOfThree`, `RaiseException`, all on
    one path, `TrigFormatter`'s units default (`extensions/rxmath/rxmath.cpp:177-181`), reached by
    `RxCalcSin/Cos/Tan/Cotan/ArcSin/ArcCos/ArcTan` with a units argument whose first byte is not
    `D d R r G g`. The first slot the C++ evaluates aborts (f3: `NewStringFromAsciiz`, rc 134,
    `before` lost; oracle 88.916 rc 168). `MathLoadFuncs`/`MathDropFuncs` hit a row, not a slot
    (`CSTRING` result, `Unfilled`, loud rc 120 per the report).
  * `rxregexp` (method context): none; every member it calls is filled.
  * `orxfunction` (call context; `build/lib/liborxfunction.so` NEEDs only `libc.so.6`): call
    table `GetArguments`, `GetArgument`, `GetRoutineName`, `GetRoutine`, `SetContextVariable`,
    `GetContextVariable`, `DropContextVariable`, `GetAllContextVariables`, `ResolveStemVariable`,
    `FindContextClass`, `ThrowException0/1/2`, `ThrowException`, `ThrowCondition`; thread table
    `NewStringFromAsciiz`, `NewString`, `ObjectToStringValue`, `ObjectToValue`, `ValueToObject`,
    `StringSizeToObject`, `RaiseException`, `RaiseException1`, `RaiseException2`,
    `GetInterpreterInstance`; and `AddCommandEnvironment` through the null instance (I2). Its
    `ARGLIST`/`NAME` parameters are rows (`Unfilled`, loud), not slots.
  * `orxmethod` (method context; NEEDs only `libc.so.6`): method table `AllocateObjectMemory`,
    `FindContextClass`, `ForwardMessage`, `FreeObjectMemory`, `GetArgument`, `GetArguments`,
    `GetMessageName`, `GetMethod`, `GetObjectVariable`, `GetObjectVariableReference`, `GetScope`,
    `GetSelf`, `GetSuper`, `ReallocateObjectMemory`, `SetGuardOff`, `SetGuardOffWhenUpdated`,
    `SetGuardOn`, `SetGuardOnWhenUpdated`, `ThrowCondition`, `ThrowException0/1/2`,
    `ThrowException`; thread table every member it calls except `WholeNumberToObject`,
    `StringData`, `StringLength`, `DoubleToObjectWithPrecision`, `RaiseException0` and the data
    members `RexxNil`/`RexxTrue` (the list is in `slots.txt`); plus `InterpreterVersion` and
    `LanguageLevel` through the null instance (f5).

  **Assessment.** Record-and-return is feasible and, done one way, sound; it costs little and buys
  the thing the L2 slice said a control needs: a red with a name. Mechanism: `entry_stub!`
  (`layout.rs:401-415`) generates, per slot, an `extern "C" fn` that records the slot's name and
  returns the value the oracle's own stub returns from its `catch (NativeActivation *)` path
  (`NULLOBJECT`, `NULL`, `0`, `false`; `ThreadContextStubs.cpp:970-996`, `:1520-1532`), and
  `invoke::run` reads the record after `call` (`invoke.rs:135`) and answers a new
  `Failure::UnfilledSlot(&'static str)` before it touches element zero and before the host reads
  `pending`. The host's existing `Loud` arm renders it at rc 120 with the `SAY` output intact.
  Two ways to keep the record, and they differ in soundness:
  * *On the `Activation`*, through `activation_of(context)`: sound only under the invariant the
    filled slots already need (the context came from a live `Contexts`), **which the refusing
    tables do not have today**: `invoke.rs:477-483` and `:533-537` hand bare contexts with a
    sentinel `arguments` and no `Owned` wrapper to the refusing tables, and
    `ffi.rs:663` calls a refusing slot with a null context. A stub that reads `owner` from a bare
    struct reads past it; "no probe calls back into the refusing table" would turn from test
    hygiene into a soundness invariant of a test file. It needs a null check and the bare
    contexts rebuilt as `Owned` with a null owner, at least.
  * *In a per-thread cell* scoped to one call (cleared by `run` before the stub, read after):
    no context is trusted, no `unsafe` is added, every table and every bare test context is
    covered, and it is not process-global state. This is the sound-by-construction one; recommend
    it.
  Limits either way, to state in the design: (a) `ThrowException0/1/2`, `ThrowException`,
  `ThrowCondition` cannot be answered by returning; the oracle leaves the extension by a C++
  throw (`CallContextStubs.cpp:205-213`, `NativeActivation.cpp:2643`), and an extension calling
  one does not expect control back. Those stay aborting until an unwinding mechanism exists. (b)
  A pointer-returning data accessor (`BufferData`, `MutableBufferData`, `StringGet`) answered
  with `NULL` may be dereferenced by the extension before it returns, which turns an abort into a
  SEGV: the oracle returns `NULL` there too on its exception path, so the extension is outside
  the header's contract, but the loud message is still lost. Per-slot judgement, or keep those
  aborting. (c) Ordering in the host: a refusal must win over a `pending` condition the extension
  raised after a harmless value came back (`run_library_routine` reads `pending` first,
  `rexx-exec/src/dispatch/library.rs:177-179`). Nothing makes the design unsound; the
  activation-keyed variant is what would.

**Minor**

- **M1 (`values.rs:155-179`, `:168-170`; inferred).** `Failure::error_number(method)` is called
  nowhere in `rexx-exec` (grep), so the boundary's own answer for a routine's signature refusal
  (40918, `Error_Incorrect_call_signature`, `RexxErrorCodes.h:408`, chosen by
  `reportSignatureError` at `NativeActivation.cpp:192`) is dead: the host renders 93.968 for both
  paths (`rexx-exec/src/dispatch/library.rs:224-225`, `error.rs:336-347`). No shipped routine
  reaches `Failure::Signature` today (none declares `CSELF`, `orxfunction`'s `ARGLIST`/`NAME` are
  `Unfilled` rows), so latent; 40918 is read from the C++ and not measured. For the integration
  reviewer: either route the number through `error_number` or delete the dead half.
- **M2 (`invoke.rs:88-90`).** `routine` re-implements `signature`'s bound rather than sharing it;
  the documented bound check (`invoke.rs:156-159`) now lives in two places. Make `signature`
  generic over the stub's context type as `signature_of` already is.
- **M3 (`tests/invoke.rs:498-511`).** `assert_eq!(session.answered(outcome), b"4")` passes because
  the stand-in's `double_object` prints `format!("{value}")` and Rust renders `4.0` as `4`; the
  witness that matters in that test is `doubles == [(4.0, 12)]`. Say so, or assert on `doubles`
  alone, so a reader does not take the string for the oracle's rendering.

### Instruments (risk 5)

- The per-row tests assert values, not presence: `the_double_row_converts_what_the_host_reads`
  (`tests/values.rs:981-995`) pins `Value::Double(2.25)`, `ARGUMENT_EXISTS` and the stripped type,
  so a row writing another member or value reddens it; `a_double_the_host_cannot_read_is_88_921…`
  (`:998-1015`) pins `position: 2` and the argument handle, so an off-by-one position reddens it;
  the positive rows likewise (`:1056-1091`); `a_returned_object_the_table_dropped…` (`:1139-1155`)
  pins `StaleHandle`. The `rxmath` tests (`tests/invoke.rs:498-568`) pin what the extension asked
  for at each precision and the 88.901/88.922 pair without the extension running.
- **M1, ran** (archive copy, `m1.txt`): `get_context_fuzz` answering `.digits`; prediction written
  first (`m1-prediction.txt`): exactly
  `a_routine_reads_numeric_settings_and_builds_a_double_through_its_contexts` red, `(7, 7, 1)`
  against `(7, 2, 1)`; result `22 passed; 1 failed`, exit 101. Confirmed.
- **Miri reaches the new slots, ran**: `numeric_stub` (`ffi.rs:628-637`) calls
  `GetContextDigits/Fuzz/Form` through `CALL_CONTEXT` and `DoubleToObjectWithPrecision` through
  the linked thread table, and that test passed under Stacked Borrows (`miri.txt:23`).
- Not re-run: C1 (the report's row-deletion control) and the corpus witnesses; not loaded:
  `liborxfunction.so` (its members are Task 5's).

### Assessment

**Task quality (boundary slice): Needs fixes.** The routine half of the protocol is built the way
the brief and the rulings ask, the rows agree with the oracle on every formatting and refusal
probe I could reach through `rxmath`, the contexts keep the L2 provenance shape and pass Miri, and
the shared `run` is where S2 wanted it. Two things stop an approval: I1, a safe `pub(crate)`
interface in this task's own new code whose soundness the SAFETY note delegates to another module
(three lines and one test to close); and I2, a null instance pointer that two shipped test
libraries dereference from a program, rc 139 with both descriptors empty, which is not this task's
rows but is the boundary's one crashing path and must be routed before the phase closes. The I3
design is workable in its per-thread form; its activation-keyed form is the one that would be
unsound against the tree as it stands.
