# Task 2 re-review, boundary slice (`rust/crates/rexx-api/`, `dispatch/library.rs` host side)

Fix base `c23c214f3`, head `bd64f3197`. Findings under verification: boundary I1, I2, I3, M1, M2,
M3 against rulings R1 and R2 (`progress.md`, "Task 2 fix round 1: sent"). Scratch:
`scratchpad/t2-rereview-a/` (probes each in their own directory with fresh `run-o`/`run-c` run
directories, three descriptors as `o.out o.err o.rc` / `c.out c.err c.rc`; `arch/` is the `git
archive bd64f3197 rust` copy for Miri and mutants). Read `corpus/oracle-crashes.txt` first; nothing
below is one of its entries. Nothing registered with rxapi.

## Log

- Read the brief, R1/R2, the report's "Fix round 1", `oracle-crashes.txt`; the slice diff per file
  (`diff-*.txt`), then `layout.rs:370-560`, `ffi.rs:100-335`, `invoke.rs:40-175`, `values.rs:515-570`,
  `load.rs:268-340`, `library.rs:60-260` at head.
- Worktree at `bd64f3197`, clean; `cargo build --release --bin rexx-run` "Finished" in 0.04s with no
  compile line, binary sha256 `b4b2dfb9…`, mtime 12:07:25 (79f08bed6 is 12:07:50 and `bd64f3197`
  touches one `.env`), so the binary is the head tree's.
- `git archive bd64f3197 rust` into `arch/`, read-only trees linked beside it; `diff -rq` of
  `arch/rust/crates/rexx-api` against the worktree: identical.
- `readelf -d`: `liborxmethod.so` and `liborxfunction.so` NEED `libc.so.6` alone; `librxmath.so`
  `libm`+`libc`; the forged `libforgesig.so` (scratch `surface-t2/fix1/ext/`) NEEDs nothing.
- **The oracle's failure-path values, derived** (`failpath.py` over the four stub files ->
  `failpath.tsv`, 220 functions): every literal after `catch (NativeActivation *)` is `0`, `false`,
  `NULL`, `NULLOBJECT`, `OREF_NULL` or a cast null, except one: `DisplayCondition` returns
  `Error_Interpretation/1000` (`ThreadContextStubs.cpp:1948`), and `RexxErrorCodes.h:456` in both
  trees reads `Error_Interpretation = 49000`. **That is 49; the tree says 48** (`layout.rs:695`
  `failing 48`, the comment above it, `ffi.rs:824` asserting `48`, the report's "DisplayCondition
  answers 48"). Recorded as I-A below.
- `InterpreterInstanceStubs.cpp:105` is blank; `interfaceVector` is `:106` (`ffi.rs:137` cites 105).
  `:79` and `:84` are `InterpreterVersion`/`LanguageLevel` as cited. `NativeActivation.cpp:190-193`
  is `reportSignatureError` with `isMethod() ? Error_Incorrect_method_signature :
  Error_Incorrect_call_signature`; `RexxErrorCodes.h:408` is `40918`, `:582` is `93968`.
  `oorexxapi.h:205` is `REXX_TYPED_ROUTINE`, `:209` the classic prototype, `:249`
  `REXX_CURRENT_LANGUAGE_LEVEL`, `:482-493` the instance interface (no member returns a pointer;
  `AttachThread` writes an out-parameter and answers `logical_t`), `:832-847` the thread context's
  inline `InterpreterVersion`/`LanguageLevel`/`AddCommandEnvironment` through `instance`.
  `CallContextStubs.cpp:207-213` is the Throw comment and `CallThrowException0`.
  `Activity.cpp:1080` opens `generateProgramInformation`, so `:1093-1113` is inside it.
- The crate never calls `RexxPackageEntry::loader` (grep over `rexx-api/src` and `library.rs`:
  only the field), and `signature_of`'s null-argument call runs none of the extension's code, so
  today no refusing stub runs outside `recording_refusals`. Every `REFUSING` use outside the table
  builders is inside the wrapper (`ffi.rs:814`) or an `aborts` member in a child (`ffi.rs:778`).
- Host order: `run` clears `pending` before answering `UnfilledSlot` (`invoke.rs:137-141`), and
  both `run_library_method` (`library.rs:111`) and `run_library_routine` (`:177`) read
  `activation.pending()` after `invoke::*` returned, so the refusal wins by construction; `settle_native_call`'s
  `Err(refused)` arm never reads `frame.raised`.
- Probes, predictions in `predictions.txt` written first, all ran as predicted (`p/*/run.*/`):
  * `i3a` (`RxCalcSin(30, 3, 'X')`): stdout `before` SAME; oracle 88.916 rc 168, crate one stderr
    line `rexx-exec: RexxThreadInterface.NewStringFromAsciiz is not implemented (Phase 8)` rc 120.
  * `i3c` (the same under `SIGNAL ON SYNTAX`): oracle `trapped SYNTAX 88` / `after` rc 0; crate
    `before` then the same loud line, rc 120, so the refusal is not a trappable condition.
  * `i2a` (`orxmethod` `TestInterpreterVersion`/`TestLanguageLevel`): oracle `version 328448` /
    `level 1542` / `as hex 50300 606` rc 0; crate `before` then rc 120 on the `size_t` FromNative row.
  * `i2b` (`orxfunction` `TestAddCommandEnvironment` through the instance table, `call` form): oracle
    `before` / `after` rc 0; crate `before` then `rexx-exec: RexxInstanceInterface.AddCommandEnvironment
    is not implemented (Phase 8)` rc 120. The instance's remaining members refuse by name.
  * `m1a` (forged `ForgeSelf()` merged from a required package): both `start`, `Error 40 ... line 2:
    Incorrect call to routine.` / `Error 40.918:  Invalid native function signature specification.`,
    rc 216; stderr differs only in my runner's `run-o`/`run-c` directory inside the path.
- `cargo test -p rexx-api` in `arch/` (`CARGO_TARGET_DIR=arch/target`): lib 27 passed; `context`
  14, `handles` 5, `invoke` 13, `layout` 18, `load` 11, `values` 48, doctests 1 and 4; exit 0.
- **Mutant M-B, prediction written before running.** In a second archive copy `arch-mut/`,
  `layout.rs:405` `let outer = REFUSED.replace(None);` -> `let outer = REFUSED.get();` (the cell is
  no longer cleared before the call, so an outer record is what a nested call sees and answers).
  Predicted under `cargo test -p rexx-api --lib`: exactly one red,
  `invoke::tests::a_refused_member_is_the_calls_answer_and_a_nested_call_keeps_its_own`, at its
  second assertion with `interpreter.nested == Some(Err(UnfilledSlot { entry:
  "RexxThreadInterface.NewStringFromAsciiz" }))` in place of `HaltThread` (the inner `refuse` finds
  the outer's record and keeps it; the outer's own assertions and the `pending` one still hold);
  `a_refusing_entry_records_itself_and_returns` green (its outer record is `None` on a fresh test
  thread); 26 passed, 1 failed.
- **Mutant M-B, ran** (`mutant-mb.txt`): exactly the predicted test red at `invoke.rs:885`, left
  `Some(Err(UnfilledSlot { entry: "RexxThreadInterface.NewStringFromAsciiz" }))` against right
  `HaltThread`; `26 passed; 1 failed`, exit 101. `arch-mut/.../layout.rs` restored from the copy and
  `cmp`-equal to the worktree's. Confirmed in full.
- **Miri, Stacked Borrows** (`miri.txt`; `RUSTUP_HOME=final-fix/rustup-home cargo +nightly miri test
  -p rexx-api --lib --offline`, `MIRIFLAGS` empty, `CARGO_TARGET_DIR=arch/miri-target`): `24 passed;
  0 failed; 3 ignored` (the child-process test and the two that open the running image), `miri exit
  0`; the record, nesting, instance and classic-row tests are among the 24.
- `cargo test -p rexx-core --test unsafe_sites` in `arch/`: 2 passed. `grep unsafe` over
  `layout.rs`, `invoke.rs`, `values.rs`: nothing but the `unsafe extern "C" fn` slot types.
- **The abort set, derived from the header** (`aborts.py`: a member whose declared return is
  `POINTER`, `CSTRING` or contains `*`, or whose name starts with `Throw`, over the four structs):
  `AllocateObjectMemory BufferData BufferStringData GetCSelf GetInterpreterInstance GetMessageName
  GetRoutineName MutableBufferData ObjectToCSelf ObjectToCSelfScoped ObjectToStringValue PointerValue
  ReallocateObjectMemory SetMutableBufferCapacity StringData` and the Throw members of both context
  tables; the difference against the report's list is empty in both directions, and `layout.rs`
  carries an `aborts` marker for each (the Throw ones twice). My first pass missed four, because the
  header writes `POINTER(RexxEntry * ObjectToCSelf)` with a space after the star; the tree's own
  `declarations_of` handles that form, which the passing `layout` test shows.
- A nested native call inside a recorded call cannot be built from shipped extensions: every filled
  member (`ffi.rs:217-222`, `:118-119`, `:129-131`, `:141-142`) answers from the host without running
  Rexx code, so no callback re-enters the interpreter. The stand-in host in
  `a_refused_member_is_the_calls_answer_and_a_nested_call_keeps_its_own` is the only witness, and
  M-B shows it is load-bearing.
- `LibraryPackage.cpp:279-286`: the oracle builds a `RegisteredRoutine` for `ROUTINE_CLASSIC_STYLE`
  and a `NativeRoutine` for every other style value.
- m1b (a `::ROUTINE ... EXTERNAL "LIBRARY forgesig ForgeSelf"` in `pk.cls`) and m1c
  (`ForgeOptional()`): both rc 216, stdout SAME, stderr identical modulo run directory; m1b's
  `Error 40 running .../pk.cls:` carries no line, m1c's names `main.rex line 2`.
- `numeric digits 3; say 0.00123 + 0` is `0.00123` on both sides (`p/fmt3`), for the out-of-scope
  note on `double_of`'s doc.

### Finding Verdicts (✅/❌/⚠️, file:line, check run)

- ✅ **I1, `stub()` typed-only.** `load.rs:257-266`: `stub` answers `None` unless
  `self.style == ROUTINE_TYPED_STYLE`, and the SAFETY note at `:261-265` now rests on that line in
  the same module; `invoke.rs:87-89` keeps the named `ClassicStyle` refusal ahead of it. Negative
  test `invoke.rs:620` (`a_classic_row_publishes_no_signature_and_calls_nothing`: a classic row
  with a real address publishes no signature and `call` records no event), green in the archive
  suite. Check: read, suite ran.
- ✅ **I2, the instance context.** `layout.rs:548` `populated`; `ffi.rs:139-144` `INSTANCE`
  filling `InterpreterVersion`/`LanguageLevel` over `REFUSING`; `ffi.rs:200-209` and `:228-234`
  an `Owned<RexxInstance_, Activation>` with the same `owner` as the thread context;
  `ffi.rs:294-298` links it wherever the thread table is linked, so its provenance and lifetime are
  the thread context's (a pointer into `self`, taken on `&mut self`, reachable only through the
  `MethodContext<'_>`/`CallContext<'_>` that borrows `self`). `applicationData` is a null field the
  header's `GetApplicationData()` reads without dereferencing (`oorexxapi.h:832`); the other five
  members are `REFUSING` stubs that record and return (`i2b` ran: `AddCommandEnvironment` refuses
  by name, rc 120, `before` kept). Measured (`i2a`): oracle `328448` and `1542`, matching
  `invoke.rs:834`'s pins; the crate reaches the instance and then refuses on the `size_t` row, as
  the report says. Miri covers the linked instance (24 passed). Check: read, ran, Miri.
- ⚠️ **I3, record-and-return.** The design as ruled: `layout.rs:385-409` (one thread-local cell,
  first record wins, cleared before and restored after the one call), `invoke.rs:135-141` (`run`
  wraps the stub call alone, answers `UnfilledSlot` ahead of the result, clears `pending`),
  `library.rs:236-244` (loud, Phase 8). Scoping and nesting: the nested test and my M-B mutant
  (exactly one red, at the nested assertion). Refusal over a pending condition: `invoke.rs:140` and
  both host readers of `pending` come after `run` (`library.rs:111`, `:177`); `i3c` shows the loud
  refusal is not trappable. Aborting slots: the set derived from the header equals the report's
  list and the `aborts` markers. Pointer decisions: no recording slot returns `POINTER`, `CSTRING`
  or a raw pointer (the test at `tests/layout.rs:143` derives that, and `CSTRING` has no
  `RefusedValue` impl so a mistake there fails to compile). Safe-code soundness: `recording_refusals`
  is `pub(crate)`, the stubs private, every slot still `unsafe extern "C" fn`, `unsafe_sites`
  green. The rule: the cell's doc (`layout.rs:386-391`) is true as the tree stands, since no stub
  runs outside `run` (no `loader` call, `signature_of` runs no extension code). Probe `i3a` as
  predicted. **The one failure-path value that is not zero is wrong** (I-A below): the ⚠️ is
  for that, not the design.
- ✅ **M1, the routine's 40.918.** `NativeActivation.cpp:190-193` is `reportSignatureError`,
  `reportException(isMethod() ? Error_Incorrect_method_signature : Error_Incorrect_call_signature)`,
  with `40918` at `RexxErrorCodes.h:408`; the host routes both refusals through
  `error_number(frame.method)` (`library.rs:224-229`, `error.rs:340-349`), so the dead half is
  live. Re-measured with the forged library: m1a, m1b, m1c identical on stdout and rc 216, stderr
  identical modulo my run directory, the directive-bound one lineless. The test
  (`library.rs:907`) goes through `settle_native_call` with a frame per flag. Check: read, ran.
- ✅ **M2.** `invoke.rs:167-173` `bounded`, the one bound both `routine` (`:90`) and `signature`
  (`:163`) share.
- ✅ **M3.** `tests/invoke.rs:494-497` asserts `doubles == [(4.0, 12)]` alone; the stand-in's
  rendering is not compared.

### New Issues In The Fix — Critical / Important / Minor (ran or inferred)

**Critical:** none found.

**Important**

- **I-A (`layout.rs:691-692`, `ffi.rs:828`, the report's "DisplayCondition answers 48"; ran).**
  `DisplayCondition`'s failure value is `Error_Interpretation/1000`, and `Error_Interpretation` is
  `49000` (`RexxErrorCodes.h:456`, the same in the oracle checkout), so the oracle returns **49**;
  the tree says `failing 48`, the comment beside it cites the right line and implies the wrong
  number, and `a_refusing_entry_records_itself_and_returns` pins 48. This is the only member in
  all four stub files whose failure literal is not a zero or a null (`failpath.tsv`), and it is
  the one that is wrong. No program observes it today (no shipped extension calls
  `DisplayCondition`; `provoke_locks.cpp` does, but `testbinaries/CMakeLists.txt:107` builds it as an
  executable; and the call refuses loudly afterwards regardless), so this is a false measured value
  pinned by a test rather than a wrong answer. Fix: `failing 49`, the assertion, the report line.

**Minor**

- **M-A (`ffi.rs:137`; ran).** Cites `InterpreterInstanceStubs.cpp:105`, a blank line;
  `InterpreterInstance::interfaceVector` is `:106`.
- **M-B (`load.rs:54-55`; ran).** "which `InterpreterInstance::LanguageLevel` answers": no such
  name in `interpreter/` or `api/`. The answerer is the free `RexxEntry LanguageLevel(RexxInstance
  *)` at `InterpreterInstanceStubs.cpp:84`, returning `Interpreter::getLanguageLevel()`
  (`Interpreter.hpp:101`); `ffi.rs:419` cites it correctly.
- **M-C (`invoke.rs:138-139`; inferred).** The comment names only "any condition the extension
  raised after the refused member answered it", but `clear_pending` also forgets one raised
  before it (`RaiseException0` is filled, so an extension can raise, run on, and then reach a
  refusing member). The `method` doc at `:52-54` states the whole effect; make the comment match
  it or drop the clause.
- **M-D (`load.rs:258`; inferred, read at `LibraryPackage.cpp:279-286`).** A row whose `style` is
  neither 1 nor 2 now answers `Failure::Signature`, which the host renders as 40.918, where the
  oracle builds a `NativeRoutine` for every non-classic value. No shipped table has such a row;
  refusing is the safe choice, but the number it refuses with names a different defect. Worth a
  sentence at `stub`, not a change.

### Out-of-Scope Observations

- `Refused::Unfilled` renders twice-suffixed: `i2a`'s stderr is `rexx-exec: Phase 8 owes the
  FromNative conversion for REXX_VALUE_size_t (33) is not implemented (Phase 8)`
  (`library.rs:236-244` appends the owner to a `Display` that already names it). Pre-existing;
  Task 3's rows retire the case, but the text is wrong today.
- `library.rs:445-447` (`double_of`, the integration slice): "takes the exponential form wherever
  the plain one would pad with zeros" is not what Rexx does; measured, `numeric digits 3; say
  0.00123 + 0` is `0.00123` on both sides. The load-bearing clause holds (the plain form is bounded
  by twice the digits after the point), so this is prose only.
- `GetInterpreterInstance` stays `aborts`, and every thread context now links an instance, so it
  could be answered as `(*context).instance`; Task 5's members, noted for whoever fills them.
- `layout.rs:524-525` ("Every function member refusing loudly") describes the call's outcome,
  not the stub's, since the stub now records quietly and `run` is what is loud; true end to end.
- The cell's doc holds only while no stub runs outside `run`. `RexxPackageEntry::loader`
  (`layout.rs:314`) is unread today; the task that first calls it with a thread context adds a
  path where a refusal is recorded, then cleared unread by the next `run`. Write that into that
  task's text when it is planned.

### Assessment — **Fix round (boundary): Needs another round** (small)

The round built what R1 and R2 asked and it holds under everything I could run against it: the
cell is per-call and per-thread, nesting is witnessed by a test my one-line mutant proves is
load-bearing, the refusal beats a pending condition by construction and is not trappable, the
abort set derived from the header is exactly the report's, no recording slot hands back a pointer,
Miri under Stacked Borrows is green, the instance context has the thread context's provenance, the
two version members measure as the oracle answers, and the routine's 40.918 reproduces on three
forged probes. What stops an approval is I-A: the ruling says a refusing slot "returns the
oracle's failure-path value", the one member with a non-zero value returns the wrong one, and a
test pins the wrong one. One character in three places, plus the three one-line prose fixes M-A to
M-C; nothing here needs a design change.
