# Phase 8 — the native API

**Written 2026-09-14**, after `docs/superpowers/specs/2026-09-14-phase-8-scoping.md` measured the
surface and D-U1 settled the `unsafe` grant. That survey is this spec's evidence base; every count
it carries is quoted here rather than re-derived.

The phase's exit gate is the roadmap's row 8: *"`testbinaries/` compile unchanged against frozen
headers; native-API ooTest groups pass. **L2 arrives here**."*

## 1. Scope, and the order

The scoping survey ruled the order: **the L2 slice first**, then the rest of the surface. The slice
is what `extensions/rxregexp` needs -- library loading, the package entry, the method-entry table,
the two-call stub protocol, the `ValueDescriptor` conversions for five value types, and seven
context functions. The rest is the 211 function pointers the slice does not fill, `testbinaries/`,
and the eight ooTest API groups.

**Out of scope, and owned elsewhere.** The queue entry points and RXAPI are Phase 10's, per D7 and
the Phase 7 close-out. `rexx`, `rexxc`, `rxqueue` and `rxsubcom` are Phase 9's, and with them the
embedding half of the API. **Which of the eight groups reach it was measured on 2026-09-15** (the
scoping survey's section 3, corrected): `RexxStart`, `ProcessRexxStart`, `INVOCATION` and
`ProcessInvocation` all load `INVOCATIONTester.cls`, which binds `orxinvocation`, whose library
NEEDs `liborxexits.so` and through it `librexx.so.4`, importing `RexxCreateInterpreter` and
`RexxStart`; those four may not close in this phase and are Phase 9's. `CLASSIC` binds `orxclassic`
and `orxclassic1`, which import the function, subcom, queue and macro-space registries, and is
Phase 10's. `METHOD`, `CONVERSION` and `FUNCTION` bind `orxmethod` and `orxfunction`, which NEED
`libc.so.6` alone and import no `Rexx*` symbol, and are this phase's. This spec first read the
partition as two embedding groups and six extension-only, from the group names alone; section 9
records the re-homing rather than leaving it to be discovered.

**Not in scope and not anywhere: ooDialog.** The roadmap's risk register accepts that it may never
recompile, and D5 must not be read as promising it.

## 2. What an extension sees, measured

The headers are frozen (D5), so the layouts below are requirements, not choices. Each was read out
of `api/oorexxapi.h` on 2026-09-14.

```
struct RexxInstance_        { RexxInstanceInterface *functions; void *applicationData; }
struct RexxThreadContext_   { RexxInstance *instance; RexxThreadInterface *functions; }
struct RexxMethodContext_   { RexxThreadContext *threadContext; MethodContextInterface *functions;
                              ValueDescriptor *arguments; }
struct RexxCallContext_     { RexxThreadContext *threadContext; CallContextInterface *functions;
                              ValueDescriptor *arguments; }
```

The C++ wraps each of those in a private struct whose **first member is the public one by value**,
followed by a back-pointer to the owning activation
(`interpreter/concurrency/ActivationApiContexts.hpp:58-95`, `MethodContext` at `:71-75`):

```
typedef struct { RexxMethodContext threadContext; NativeActivation *context; } MethodContext;
```

so a cast from the public pointer to the private struct recovers the owner
(`Activity.hpp:503`). **Rust reproduces this exactly**: a `#[repr(C)]` struct whose first field is
the public `#[repr(C)]` struct and whose second is our own back-reference. The extension only ever
sees the first two or three words, and the recovery cast is one of the two things `ffi.rs` is
granted `unsafe` for.

The interface structs themselves are tables of `RexxEntry` function pointers, populated once and
shared: 7, 142, 26, 21, 11 and 11 pointers for the instance, thread, method, call, exit and
IO-redirector interfaces respectively. Each is a `static` in Rust whose fields are
`unsafe extern "C" fn` pointers -- every slot is unsafe to call, since `69a579370`, because a bare
or forged context reaching a slot from safe code was the boundary re-review's finding -- built
at first use rather than at load, because a `const` table of function pointers is the shape the C++
uses (`Activity::methodContextFunctions`) and it costs nothing.

## 3. Loading, measured

`common/platform/unix/SysLibrary.cpp` is the whole of the C++ load path, and it is short:

* the file name is `lib<name>` plus `ORX_SHARED_LIBRARY_EXT`, formatted at `:88`
* `dlopen(name, RTLD_LAZY)` at `:90`
* on failure, a second attempt at `/usr/lib/lib<name><ext>` at `:94-95`
* `dlclose` on unload at `:115`
* symbol lookup is a bare `dlsym` at `:71`

The symbol an extension publishes is `RexxGetPackage`, returning `RexxPackageEntry *`
(`oorexxapi.h:253-255`), and the loader looks it up by that literal name
(`interpreter/package/LibraryPackage.cpp:211`). If the entry's `requiredVersion` is non-zero and
greater than the interpreter's, the load raises `Error_Execution_library_version` = **98.982**
(`LibraryPackage.cpp:232-235`). Routines are registered from the table, then the optional `loader`
hook runs (`:237-247`). This crate reads the entry's name, version and two tables and runs neither
the `loader` nor the `unloader` hook; the Phase 8 section of `phase-4-exclusions.txt` records that
gap.

**Corrected 2026-09-14 by measurement**, because reading the error table produced the wrong
numbers for the path a program actually takes. A `::REQUIRES ... LIBRARY` naming a library that is
not there is **98.903** (`Error_Execution_library`, `PackageManager.cpp:214`), not 98.982, and it
fires before the program's first clause. A `::METHOD ... EXTERNAL` naming an entry the library does
not export is **90.998** and a `::ROUTINE` one is **90.999**, both at directive-install time.
`Error_Execution_library_method` = 98.978 is reachable from no surface at all: measured
2026-09-14, `loadExternalMethod` and `loadExternalRoutine` answer `.nil`, `Package~loadLibrary`
answers `1` or `0`, and `.Object~package~loadLibrary('nosuchlib_zz')` is **98.984** at rc 158 --
with a name argument; with none it is 88.901 first, rc 168 (re-run 2026-09-15).
`PackageManager.cpp:947` and `:968` are restore and reflatten paths a program cannot reach. **This
was the fifth error number on this phase taken from `RexxErrorCodes.h` that named a path nothing
runs**, which is why the rule is now that a number is measured or it is not written. The
transcripts are in this plan's SDD ledger.

**The test target is a compiled extension, not a rebuild of it.** Measured 2026-09-14 and recorded
as an amendment to D5: `librxregexp.so` imports no symbol whose name contains `rexx` and its
`NEEDED` list is `libstdc++.so.6`, `libgcc_s.so.1`, `libc.so.6`. An extension links nothing from
the interpreter, so a prebuilt one runs against this crate as soon as the `#[repr(C)]` tables match
the frozen headers. **Two builds of it exist, and the instruments split between them** (measured
2026-09-15, the amendment's second paragraph): the corpus differential hands both interpreters the
oracle checkout's `/home/moritz/dev/repos/ooRexx/build/lib/librxregexp.so` through the
`{oraclelib}` a `.env` sidecar expands, so there loading the same bytes on both sides removes the
question of whether the two sides compiled against the same header; the in-crate tests
(`rexx-api/tests/{load,invoke,context}.rs` and `dispatch/library.rs`'s unit tests) open this
worktree's own `build/lib/librxregexp.so`, a second build from identical sources that the oracle
never runs. The measurements above hold on both. The embedding half of the API stays
source-compatible and is Phase 9's, per the same amendment.

**`libloading` owns the handle**, per D-U1. The two-attempt search and the `lib`/`.so` decoration
are ours to reproduce, because they are observable: a program that names a library which exists in
neither place has to produce the oracle's failure, not a different one.

## 4. The two-call protocol, and the conversion table

This is the measurement that shapes the phase, and it is easy to get backwards. **An extension does
not call the API to fetch its arguments.** The `RexxMethodN` macros (`oorexxapi.h:4333` onward)
generate a stub

```
uint16_t *RexxEntry name(RexxMethodContext *context, ValueDescriptor *arguments)
```

which the interpreter calls **twice** (`NativeActivation.cpp:1296-1306`):

1. with `arguments == NULL`, and the stub returns a static `uint16_t[]`: element 0 is the return
   type, elements 1..n the parameter types, terminated by `REXX_ARGUMENT_TERMINATOR`
2. with an array the interpreter filled from that signature, and the stub reads
   `arguments[i].value.value_<type>`, calls the real implementation, and writes the result into
   `arguments[0]`

Between the two calls sits `NativeActivation::processArguments` (`:219`), which is the conversion
table: about thirty `REXX_VALUE_*` cases converting a Rexx object into the declared C type, with
`OPTIONAL_` variants tolerating an omitted argument and the rest refusing when they cannot.

**What the refusal is, measured 2026-09-14 against the oracle through `rxregexp`**: an absent
required argument is **88.901** (`Missing argument; argument 1 is required.`) and an unconvertible
one is **88.909** (`Argument 1 must have a string value.`), rc 168 each. This corroborates Phase
7's close, which found `stream_position` answering 88.901 for the same reason.
`Error_Incorrect_method_signature` = 93.968 and `Error_Incorrect_call_signature` = 40.918 are
`reportSignatureError` and belong to a malformed *signature*, not to an argument; neither is
reachable through an oracle-built extension, so neither is a corpus witness. 93.968 was then
measured through a forged extension in scratch (the ledger's `final-fix-report.md`, F2 and F9):
raised for a parameter code the table does not know, it is lineless and names the declaring
package; raised for a result word carrying the optional bit, it keeps its line and names the
sender. `rexx-api`'s `invoke` and `values` tests pin the two failures, `dispatch/library.rs`'s
`a_refusal_before_the_call_is_lineless_and_one_after_it_is_not` pins the two deliveries, and
`corpus/refusal-sites.tsv` carries both rows as measured. 40.918 is the routine form and waits on the routine half.
Afterwards `valueToObject(arguments)` converts element 0 back.

**Two properties this crate must hold and the C++ gets for free.** The signature call happens
before any argument is touched, so a signature request must not observe or consume arguments. And
the conversion is driven entirely by the extension's declared types -- an extension that declares
`CSTRING` gets a pointer whose lifetime is the call, which means the string it points at has to be
reachable from a root for the duration and not merely borrowed from a temporary.

**The L2 slice needs five of the thirty cases**: `int` as the return type, `CSTRING`,
`OPTIONAL_CSTRING`, `CSELF` and `RexxStringObject`. The table is written as a table from the start
so that the remaining cases are rows rather than a redesign.

## 5. `RexxObjectPtr` is a handle, and a stale one misses

D5 already settled this and the roadmap's risk register carries the row: `RexxObjectPtr` is an
opaque handle registered in the calling activation's local-reference table, not a heap address.
The C++ does the same thing through `NativeActivation::createLocalReference` /
`removeLocalReference` / `clearLocalReferences`, because native code holds references across GC
points.

**The generation field in `ObjRef` is what makes this safe**, and the risk register says why: slots
are recycled through a free list, so a bare slot index held across a collection would name whatever
was allocated into it next. That is memory-safe and answers the wrong object, which is the same
defect wearing different clothes and landing exactly at the boundary this decision advertises as
the win. A handle that outlives its activation must be a **lookup miss**, and section 9 makes that
a test rather than a claim.

`RequestGlobalReference` is the escape hatch the header offers for a reference that outlives the
activation, and it is a separate table with a separate lifetime. The L2 slice does not need it;
the design must not make it impossible.

## 6. `CSELF`, `NewPointer`, and uninit

`rxregexp` stores a C++ `automaton *` in the object and gets it back as `CSELF`. The mechanism is
an object variable: `RegExp_Init` calls `context->NewPointer(pAutomaton)` and
`SetObjectVariable("CSELF", ...)`, and the `REXX_VALUE_CSELF` conversion reads that variable back
out (`NativeActivation.cpp:294`). So three things have to exist together: a `.Pointer` instance
that can be minted internally, an object-variable pool reachable from the native context, and the
`CSELF` case in the conversion table.

`.Pointer` is already an `.environment` class in `native_classes.rs` with `NEW` deliberately
unsupported, matching the oracle's 93.967. Whether an instance can be minted internally is
unmeasured, and is the first thing the plan's Task 1 settles.

`RegExp_Uninit` is a native `UNINIT`, so the collector has to call into a loaded library at
finalisation. **That is a hard ordering constraint**, not a detail: an `UNINIT` that runs during
collection must not allocate, and must not run after the library it lives in has been unloaded.
The plan sizes this as its own task.

## 7. Errors, and what the extension can raise

`RaiseException0`, `RaiseException1`, `RaiseException2` and the array-taking `RaiseException`
(`oorexxapi.h:635-638`) take an error number from `api/oorexxerrors.h` and raise it as a condition
in the caller's context. `rxregexp` calls `RaiseException0` alone, with
`Rexx_Error_Incorrect_method` (`rxregexp.cpp:73`, `:130`) and `Rexx_Error_Invalid_template`
(`:83`); `ffi.rs` fills that one slot and leaves the other three at the refusing stub. The
requirement is the ordinary one for this project: the number,
the sub-number and the message text as the oracle produces them, on the descriptor the oracle uses.

The three loading failures in section 3 are the other error surface, and they are what a program
sees when a `::REQUIRES LIBRARY` names something that is not there.

**Nothing unwinds across the boundary, measured 2026-09-14.** Each of `RaiseException0`, `1` and
`2` wraps `reportException` in a `try` and catches `NativeActivation *` **inside the stub**
(`interpreter/api/ThreadContextStubs.cpp:1863-1897`: `RaiseException0` at `:1863`, `1` at `:1875`,
`2` at `:1887`), returning normally to the extension. The
condition is stashed on the activation and raised after the native call returns, by
`NativeActivation::checkConditions` (`NativeActivation.cpp:1787-1807`). The extension's own code
agrees: `RegExp_Parse` raises for an unrecognised match type and then falls straight through to
`pAutomaton->parse(...)`.

Two consequences bind the rest of this phase. The exported entry points are `extern "C"` and stay
that way -- `extern "C-unwind"` would make a Rust panic crossing into extension frames
well-defined, where an abort is the behaviour a bug on our side should have. And a native method
that raises still runs to completion and still returns a value, so the two-call protocol in
section 4 must deliver `arguments[0]` even for a call that raised.

> **Note, appended 2026-09-26 (Phase 8 surface Task 5, fix round 1; controller ruling,
> provisional).** The paragraph above holds for the `Raise*` members and does not hold for the
> `Throw*` members, which it did not consider. `ThrowException0`, `1`, `2`, `ThrowException` and
> `ThrowCondition` of the method and call contexts call `reportException` or `raiseCondition`
> with no `try` (`interpreter/api/CallContextStubs.cpp:207-240`, "don't use try/catch to allow a
> return to the caller. This will unwinded back to the invoking NativeActivation"), so the C++
> exception unwinds through the extension's frames, running their destructors, and the code after
> the call never runs (`testbinaries/orxmethod.cpp:2122-2162` sets `CONTINUE` after each call, and
> the METHOD and FUNCTION groups assert it stays unset). An aborting slot ended the whole ooTest
> process. So these slots, the exit context's `Throw*` members, and the call into an extension's
> entry point are `extern "C-unwind"`: a `Throw*` slot records its condition as the matching
> `Raise*` member does and unwinds with a private marker payload, and the one call boundary
> (`load::call_stub`) catches exactly that marker and lets the held condition be raised as after
> any return; any other payload is resumed. Every other slot stays `extern "C"`, so a panic there
> still aborts. Measured: a forged extension's local with a destructor runs on both sides, and
> `rust/corpus/lang/library_callback_throw.rex` agrees with the oracle byte for byte. An unwinding
> extension needs the process's one unwinder, `libgcc_s`: a forge linking `libgcc` statically
> aborted at its first `Throw`.
>
> **Note, appended 2026-09-26 (Task 5, fix round 2).** The last sentence above is a divergence
> from the oracle, not a requirement this phase may place on extensions, and it is one of two: an
> extension linked with a static `libgcc` aborts at a `Throw` where the oracle's runs on, and an
> extension that swallows a `Throw` in its own `catch (...)` aborts ("Rust panics must be
> rethrown") where the oracle's continues. Both are recorded in `phase-4-exclusions.txt` with no
> owner, because Rust cannot raise a C++ exception. A `Throw` in a library loader that
> `LoadLibrary` or `RegisterLibrary` ran leaves the extension that called them too, as the
> oracle's does, so those two members and the package hooks are `extern "C-unwind"` as well.
>
> **Note, appended 2026-09-26 (Task 5, fix round 3).** The previous note's reason for leaving
> the two divergences without an owner is corrected: matching them needs a real C++ exception,
> which a C++ shim could raise, and a shim would add a C++ compiler to this crate's build, which
> the no-new-dependency rule keeps out. `LoadLibrary` and `RegisterLibrary` unwind only with the
> `Throw` marker; any other panic in them aborts, as every other slot's does.

**The `__cplusplus` branch of the header binds.** Under `#ifndef __cplusplus` a context is
typedefed to a pointer (`oorexxapi.h:135-174`), which would make `RexxThreadContext *` a pointer to
a pointer. Nothing in this tree builds that way: the one `.c` file that includes `oorexxapi.h` is
`ootest/misc/dlOpenTest.c` (`:49`; `find . -name '*.c' -not -path './build/*' -not -path
'*/target/*' | xargs /bin/grep -l oorexxapi.h`), which no `CMakeLists.txt` outside `build/` names,
and `testbinaries/orxclassic1.c`, the only C translation unit among the test binaries, includes
`rexx.h`, which does not include it either.

## 8. The crate

`rexx-api/`, which the roadmap's Section 3 tree already reserves. Two modules carry
`#[allow(unsafe_code)]` and nothing else does (D-U1):

* `src/load.rs` -- `libloading`, the two-attempt name search, `RexxGetPackage`, and calling through
  a resolved function pointer. The outbound boundary.
* `src/ffi.rs` -- the exported `extern "C"` entry points and the recovery cast from a public
  context pointer to our private struct. The inbound boundary.

Everything else is safe Rust: the interface tables are `#[repr(C)]` data whose every slot is typed
`unsafe extern "C" fn`, so a call through one needs an `unsafe` block that only those two modules
may write; the conversion table is a match over a `u16`; and the handle registry is an ordinary
map. `ffi::value_of`, which reads a union member, is `pub unsafe fn` since `13268f0e1`, with a
`compile_fail` doctest as its witness.

`crates/rexx-core/tests/unsafe_sites.rs` names the granted set and fails when it changes. This
phase's first commit updates it to name three files, and that update is the visible record of the
grant.

## 9. The gate

| criterion | instrument |
|---|---|
| a library loads and its package entry is read | a corpus witness using `::REQUIRES LIBRARY rxregexp`, against the oracle checkout's `build/lib/librxregexp.so` through the `{oraclelib}` sidecar (section 3) |
| the two-call protocol is honoured | a unit test asserting the signature call sees no arguments |
| the conversion table is right for the five L2 types | the 20 `rxregexp` L1 cases, differential |
| `CSELF` survives a collection | a witness that collects between two method calls on one object |
| a stale handle misses rather than lies | a unit test in `rexx-api` holding a handle past its activation |
| the load failures | corpus witnesses for 98.903, 90.998, 90.999 and 98.984; 98.982 cannot be one, since no oracle-built extension asks for a newer interpreter, and is witnessed by `dispatch/library.rs`'s unit tests and by a forged extension in scratch (the ledger's `final-fix-report.md`, F5) |
| **L2** | `ooTest.frm` loads and one test group executes |
| `testbinaries/` compile unchanged | a build of `testbinaries/` against the frozen headers |
| the three extension-only API groups, `METHOD`, `CONVERSION` and `FUNCTION` (section 1) | the ooTest run |
| no refusal names Phase 8 | `crates/rexx-exec/tests/closed_phases.rs` with `"Phase 8"` added |

**Negative controls**, each predicted in writing before it is run:

* the load test must fail when pointed at a library that does not exist, with the oracle's error
  and not a Rust panic
* the stale-handle test must be shown to pass for a *live* handle, or it is asserting that handles
  never work
* the `CSELF` collection witness must be shown to fail when collection is forced off, or it is not
  measuring survival
* `closed_phases.rs` already carries a negative control that finds an open phase by the same walk

**What the gate cannot see, stated here rather than discovered at the close.** Five of the eight
groups reach surfaces section 1 puts elsewhere -- `RexxStart`, `ProcessRexxStart`, `INVOCATION`
and `ProcessInvocation` the embedding API (Phase 9), `CLASSIC` the registries (Phase 10) --
measured 2026-09-15 and recorded as a re-homing in `phase-4-exclusions.txt`'s Phase 8 section and
in the roadmap's rows 9 and 10, rather than carried as a silent gap. A `testbinaries/` build that
succeeds says the headers are compatible, not that the entry points behind them work.

## 10. What this spec does not settle

Two questions that were on this list have been answered by measurement and moved into section 7:
whether anything unwinds across the boundary, and which branch of the header binds. Both were
raised by an implementer rather than found here, which is the argument for tasks reporting what
their brief did not answer.

The threading model of `AttachThread` and `DetachThread`, which is Phase 6's ground and is why the
instance interface's seven pointers are not in the L2 slice. The exit interface, whose consumers are
`orxexits` in `testbinaries/`. Whether the classic-style routine entry (`ROUTINE_CLASSIC_STYLE`,
the `RXSTRING` calling convention) is needed before Phase 10, given that its users are `rxsubcom`
and the registered-function API. And the security manager's relationship to a native method:
`NativeActivation::run` reads a security manager off the `NativeMethod`
(`NativeActivation.cpp:1282-1286`), and this crate's manager is stored per package, so the two have
to be reconciled before a native method can be checked.
