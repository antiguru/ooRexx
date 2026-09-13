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
context functions. The rest is the other 211 function pointers, `testbinaries/`, and the eight
ooTest API groups.

**Out of scope, and owned elsewhere.** The queue entry points and RXAPI are Phase 10's, per D7 and
the Phase 7 close-out. `rexx`, `rexxc`, `rxqueue` and `rxsubcom` are Phase 9's, and with them the
embedding half of the API that `RexxStart.testGroup` and `ProcessRexxStart.testGroup` reach --
those two of the eight groups therefore may not close in this phase, and section 9 says so in the
gate rather than leaving it to be discovered.

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
IO-redirector interfaces respectively. Each is a `static` in Rust with `extern "C"` fields, built
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
hook runs (`:237-247`).

**Corrected 2026-09-14 by measurement**, because reading the error table produced the wrong
numbers for the path a program actually takes. A `::REQUIRES ... LIBRARY` naming a library that is
not there is **98.903** (`Error_Execution_library`, `PackageManager.cpp:214`), not 98.982, and it
fires before the program's first clause. A `::METHOD ... EXTERNAL` naming an entry the library does
not export is **90.998** and a `::ROUTINE` one is **90.999**, both at directive-install time.
`Error_Execution_library_method` = 98.978 belongs to `loadExternalMethod` and `Package~loadLibrary`
(`PackageManager.cpp:947`, `:968`). The transcripts are in this plan's SDD ledger.

**The test target is the oracle's own compiled extension, not a rebuild of it.** Measured
2026-09-14 and recorded as an amendment to D5: `build/lib/librxregexp.so` imports no symbol whose
name contains `rexx` and its `NEEDED` list is `libstdc++.so.6`, `libgcc_s.so.1`, `libc.so.6`. An
extension links nothing from the interpreter, so a prebuilt one runs against this crate as soon as
the `#[repr(C)]` tables match the frozen headers. Loading that exact file removes the question of
whether the two sides compiled against the same header, which a rebuild would leave open. The
embedding half of the API stays source-compatible and is Phase 9's, per the same amendment.

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
`OPTIONAL_` variants tolerating an omitted argument and the rest raising
`Error_Incorrect_method_signature` when they cannot. Afterwards `valueToObject(arguments)` converts
element 0 back.

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

`RaiseException`, `RaiseException0` and `RaiseException1` take an error number from
`api/oorexxerrors.h` and raise it as a condition in the caller's context. `rxregexp` uses
`Rexx_Error_Incorrect_method`. The requirement is the ordinary one for this project: the number,
the sub-number and the message text as the oracle produces them, on the descriptor the oracle uses.

The three loading failures in section 3 are the other error surface, and they are what a program
sees when a `::REQUIRES LIBRARY` names something that is not there.

## 8. The crate

`rexx-api/`, which the roadmap's Section 3 tree already reserves. Two modules carry
`#[allow(unsafe_code)]` and nothing else does (D-U1):

* `src/load.rs` -- `libloading`, the two-attempt name search, `RexxGetPackage`, and calling through
  a resolved function pointer. The outbound boundary.
* `src/ffi.rs` -- the exported `extern "C"` entry points and the recovery cast from a public
  context pointer to our private struct. The inbound boundary.

Everything else is safe Rust: the interface tables are `#[repr(C)]` data, the conversion table is a
match over a `u16`, and the handle registry is an ordinary map.

`crates/rexx-core/tests/unsafe_sites.rs` names the granted set and fails when it changes. This
phase's first commit updates it to name three files, and that update is the visible record of the
grant.

## 9. The gate

| criterion | instrument |
|---|---|
| a library loads and its package entry is read | a corpus witness using `::REQUIRES LIBRARY rxregexp`, against the oracle's own `build/lib/librxregexp.so` |
| the two-call protocol is honoured | a unit test asserting the signature call sees no arguments |
| the conversion table is right for the five L2 types | the 20 `rxregexp` L1 cases, differential |
| `CSELF` survives a collection | a witness that collects between two method calls on one object |
| a stale handle misses rather than lies | a unit test in `rexx-api` holding a handle past its activation |
| the load failures | corpus witnesses for 98.903, 90.998, 90.999 and 98.978 |
| **L2** | `ooTest.frm` loads and one test group executes |
| `testbinaries/` compile unchanged | a build of `testbinaries/` against the frozen headers |
| the six API groups that are not embedding | the ooTest run |
| no refusal names Phase 8 | `crates/rexx-exec/tests/closed_phases.rs` with `"Phase 8"` added |

**Negative controls**, each predicted in writing before it is run:

* the load test must fail when pointed at a library that does not exist, with the oracle's error
  and not a Rust panic
* the stale-handle test must be shown to pass for a *live* handle, or it is asserting that handles
  never work
* the `CSELF` collection witness must be shown to fail when collection is forced off, or it is not
  measuring survival
* `closed_phases.rs` already carries a negative control that finds an open phase by the same walk

**What the gate cannot see, stated here rather than discovered at the close.** `RexxStart` and
`ProcessRexxStart` reach the embedding API, which section 1 puts in Phase 9; if they do not pass
here that is a re-homing, and it must be recorded in `phase-4-exclusions.txt` as one rather than
carried as a silent gap. A `testbinaries/` build that succeeds says the headers are compatible, not
that the entry points behind them work.

## 10. What this spec does not settle

The threading model of `AttachThread` and `DetachThread`, which is Phase 6's ground and is why the
instance interface's seven pointers are not in the L2 slice. The exit interface, whose consumers are
`orxexits` in `testbinaries/`. Whether the classic-style routine entry (`ROUTINE_CLASSIC_STYLE`,
the `RXSTRING` calling convention) is needed before Phase 10, given that its users are `rxsubcom`
and the registered-function API. And the security manager's relationship to a native method:
`NativeActivation::run` reads a security manager off the `NativeMethod`
(`NativeActivation.cpp:1282-1286`), and this crate's manager is stored per package, so the two have
to be reconciled before a native method can be checked.
