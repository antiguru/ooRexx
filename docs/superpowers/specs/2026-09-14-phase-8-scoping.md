# Phase 8 scoping survey — the native API, and the smallest thing that unlocks L2

**Surveyed 2026-09-14**, before any Phase 8 code was written. Every figure here comes from a
command run against the tree on this date; where a number is a count of something the tree can
enumerate, the command that produced it is quoted beside it.

The phase's exit gate is the plan's row 8: *"`testbinaries/` compile unchanged against frozen
headers; native-API ooTest groups pass. **L2 arrives here**, not in Phase 7."* This survey exists
to separate those two clauses, because they are very different sizes and only one of them is on
the critical path for every phase after this one.

## 1. The full surface is 218 function pointers plus 37 flat entry points

`api/oorexxapi.h` declares six interface structs, each a table of `RexxEntry` function pointers
that Rust has to populate. Counted with `grep -cE '\(RexxEntry \*'` over each struct's line range:

| struct | header line | pointers |
|---|---|---|
| `RexxInstanceInterface` | `oorexxapi.h:482` | 7 |
| `RexxThreadInterface` | `:503` | 142 |
| `MethodContextInterface` | `:682` | 26 |
| `CallContextInterface` | `:718` | 21 |
| `ExitContextInterface` | `:749` | 11 |
| `IORedirectorInterface` | `:769` | 11 |

The table is closed and was checked for that: `grep -cE '\(RexxEntry \* ?[A-Za-z_][A-Za-z0-9_]*\)'`
finds 220 declarations in the whole header, and the two that are not members of those six structs
are the `RexxPackageLoader` and `RexxPackageUnloader` typedefs at `:259-260`. **The count is of
header declarations and of nothing else** -- in particular it is not a count of exported symbols,
which is a different and much larger set that no extension imports (see the D5 amendment). An
earlier pass of this survey used `[A-Za-z_]+` for the pointer's name, which excludes digits and
silently dropped `SendMessage0`, `Int64ToObject`, `RaiseException0` and their neighbours; the
pattern above is the corrected one.

Beside them, `api/rexx.h` declares 37 `RexxReturnCode REXXENTRY` functions -- the classic flat API,
counted with `grep -c 'RexxReturnCode REXXENTRY' api/rexx.h`. Many of those are the queue, macro
space and subcom registries, which the plan already assigns to Phase 10 and Phase 9 respectively.

D5 froze `api/oorexxapi.h` byte-identical, so none of this is negotiable in shape: a header edit
reopens D5 as a Section 1 decision rather than being a task.

## 2. `rxregexp` needs seven of those pointers, and that is the whole L2 blocker

Phase 7's close measured that `ooTest.frm` stops at `::METHOD INIT EXTERNAL "LIBRARY rxregexp
RegExp_Init"`. The natural reading is that L2 waits on the native API; the measurement says it
waits on a specific and small part of it.

`extensions/rxregexp/rxregexp.cpp` makes seven distinct context calls, counted with
`grep -oE 'context->[A-Za-z_0-9]+' | sort | uniq -c`. **The name an extension writes is not always
the name of a function pointer**, because the headers put C++ inline conveniences on the context
structs that forward to a differently named pointer, so the table below carries both. Reading only
the left column names two entries that do not exist.

| written in the extension | uses | the function pointer it reaches | table |
|---|---|---|---|
| `SetObjectVariable` | 4 | `SetObjectVariable` (`:693`) | method context |
| `WholeNumber` | 3 | **`WholeNumberToObject`** (`:548`), via the inline at `:1041` | thread |
| `StringData` | 3 | `StringData` (`:577`) | thread |
| `RaiseException0` | 3 | `RaiseException0` (`:635`) | thread |
| `StringLength` | 2 | `StringLength` (`:576`) | thread |
| `NewPointer` | 1 | `NewPointer` (`:615`) | thread |
| `DropObjectVariable` | 1 | `DropObjectVariable` (`:695`) | method context |

It is `RaiseException0` and not the general `RaiseException`, which takes an argument array.

Its five methods declare five value types between them -- `int` as every return type, then
`CSTRING`, `OPTIONAL_CSTRING`, `CSELF` and `RexxStringObject`.

**The extension does not call the API to fetch its arguments.** The `RexxMethodN` macros
(`oorexxapi.h:4333` onward) generate a stub with the signature `uint16_t *RexxEntry name
(RexxMethodContext *, ValueDescriptor *)`, and that stub is called twice for two different
purposes. Called with a null `arguments` it returns a static `uint16_t[]` of `REXX_VALUE_*` codes:
element 0 is the return type, the rest are the parameters, terminated by
`REXX_ARGUMENT_TERMINATOR`. Called with an array it reads `arguments[1..n].value.value_<type>` and
writes its result into `arguments[0]`. So the conversion between Rexx objects and C types is the
interpreter's work, driven by a signature the extension publishes -- which makes the
`ValueDescriptor` conversion table a first-class piece of this phase rather than an incidental one.

Loading is the other half: `REXX_GET_PACKAGE` exports a `RexxPackageEntry *` under a known symbol,
and `rxregexp`'s entry names a version floor (`REXX_INTERPRETER_4_0_0`), a method table, and null
load/unload hooks.

**The L2 slice is therefore: `dlopen` and symbol lookup, the package entry, the method-entry table,
the two-call stub protocol, the `ValueDescriptor` conversions for five types, and seven function
pointers.** Not 218. The seven are `WholeNumberToObject`, `StringData`, `StringLength`,
`NewPointer` and `RaiseException0` in the thread table, and `SetObjectVariable` and
`DropObjectVariable` in the method-context table.

Note what the subtraction does and does not say. 218 minus those seven leaves 211 function pointers
the slice does not fill, but three of the six things in the sentence above are not function
pointers at all, so "211 remain" is not the same claim as "the slice is nearly done".

## 3. The gate's other clause is `testbinaries/` and eight ooTest groups

`testbinaries/` holds `orxclassic.cpp`, `orxclassicexits.cpp`, `orxfunction.cpp`,
`orxinstance.cpp`, `orxinvocation.cpp`, `orxmethod.cpp`, `provoke_locks.cpp`, `rexxinstance.cpp`
and `orxclassic1.c`, with three `.def` files. Those exercise the API deliberately and broadly,
which is the point of them.

The ooTest groups that consume them are eight files under `ootest/ooRexx/API/`, found with `find
ootest/ooRexx/API -name '*.testGroup'`: `classic/CLASSIC.testGroup` and, under `oo/`,
`CONVERSION`, `FUNCTION`, `INVOCATION`, `METHOD`, `ProcessInvocation`, `ProcessRexxStart` and
`RexxStart`. The suite as a whole has 409 `.testGroup` files, so the native-API groups are a small
named subset.

**Which surface each group reaches was measured on 2026-09-15, and the reading this survey first
gave -- two embedding groups, six extension-only -- was wrong.** Each group loads its package file
at its own line 52 with `.context~package~loadPackage(...)`
(`/bin/grep -n -E "Package\.cls|Tester\.cls" ootest/ooRexx/API/*/*.testGroup`), and each package
file names one library (`/bin/grep -oE 'LIBRARY [A-Za-z0-9]+' ootest/ooRexx/API/*/*.cls | sort |
uniq -c`): `METHOD` and `CONVERSION` bind `orxmethod`, `FUNCTION` binds `orxfunction`,
`INVOCATION`, `ProcessInvocation`, `RexxStart` and `ProcessRexxStart` all load
`INVOCATIONTester.cls`, which binds `orxinvocation`, and `CLASSIC` binds `orxclassic` and registers
`orxclassic1` through `rxfuncadd` (`CLASSIC.testGroup:822,861,875`). On the oracle checkout's
`build/lib`, which holds one product per `testbinaries/` target compiled from sources
byte-identical to this tree's (`diff -rq` of `api/` and of `testbinaries/` against
`/home/moritz/dev/repos/ooRexx`, both empty): `readelf -d` gives `liborxmethod.so` and
`liborxfunction.so` a `NEEDED` list of `libc.so.6` alone, and `nm -D --undefined-only` shows
neither importing a `Rexx*` symbol; `liborxinvocation.so` NEEDs `liborxexits.so`, which NEEDs
`librexx.so.4` and `librexxapi.so.4` and imports `RexxCreateInterpreter`, `RexxStart` and the exit
and subcom registries; `liborxclassic.so` and `liborxclassic1.so` import the function, subcom,
queue and macro-space registries. So three groups are extension-only and this phase's: `METHOD`,
`CONVERSION` and `FUNCTION`. Four reach the embedding API, which is what `rexx-cli` will need in
Phase 9, and are Phase 9's: `RexxStart`, `ProcessRexxStart`, `INVOCATION` and `ProcessInvocation`.
`CLASSIC` reaches the registries the plan assigns to Phase 10 and is Phase 10's. The roadmap's rows
9 and 10 and `phase-4-exclusions.txt`'s Phase 8 section record the re-homing.

## 4. What the crate tree already reserves

The plan's Section 3 tree names `rexx-api/` for this phase, with `src/ffi.rs` annotated as *"the
only module carrying `#[allow(unsafe_code)]`, under a crate root of deny (not forbid), and only
after a Section 1 decision block says so"*. D-U1 (section 7) then granted `src/load.rs` as well,
and the annotation was corrected on 2026-09-15 to name both modules. The crate does not exist yet:
`ls rust/crates/` returns
`rexx-bench`, `rexx-classes`, `rexx-core`, `rexx-exec`, `rexx-extract`, `rexx-inventory`,
`rexx-lib`, `rexx-num`, `rexx-oracle`, `rexx-parse`. Note that `rexx-sys`, which the tree reserves
for Phase 7's platform layer, was never created either -- Phase 7 put its platform code in
`rexx-exec` -- so the tree is a proposal, not a record.

## 5. The refusals this phase owes

Phase 7's close re-homed every refusal that opens a shared library to Phase 8, and
`crates/rexx-exec/tests/closed_phases.rs` asserts that none of them names a closed phase. Found
with `grep -rn 'Phase 8' crates/*/src`:

* `::ROUTINE EXTERNAL` (`lib.rs:785`)
* `::REQUIRES LIBRARY` (`lib.rs:820`)
* `::METHOD EXTERNAL` and `::ATTRIBUTE EXTERNAL` naming a library other than `REXX` (`lib.rs:801`)
* `loadExternalMethod` / `loadExternalRoutine` and `Package~loadLibrary`
* `handle_set` (`dispatch/native.rs:217`), the one `Stream` entry point held back here --
  re-homed to Phase 10 at `08d232ecc`, whose message says it needs `from_raw_fd` and never needed
  the loader

`dispatch::native`'s `every_deferred_entry_point_names_an_open_phase` carries `"Phase 8"` in its
`OPEN` list, so those refusals stay legal until this phase closes and then must all be gone.

## 6. The 20 L1 cases that are waiting

`ls rust/corpus-l1/ | grep -c rxregexp` gives 20 extracted micro-programs named for `rxregexp`, out
of 12,059 L1 cases. They are the immediate observable payoff of section 2's slice, and they are
extracted programs rather than the framework, so they can be run before L2 is reached.

## 7. The three decisions, and how they were settled

**D-U1: the unsafe grant, and `dlopen`'s provider. Closed by Moritz 2026-09-14**, before any
`extern "C"` entry point was written, which the roadmap's section 6 Phase 8 bullet requires
(`2026-07-27-rust-rewrite.md:2638`). The block is in Section 1 of `2026-07-27-rust-rewrite.md`.
Two modules are granted `#[allow(unsafe_code)]` -- `rexx-api/src/ffi.rs` for the inbound boundary
and `rexx-api/src/load.rs` for the outbound one -- and `libloading` 0.8.9 joins the dependency set
for the loader. Phase 7's "no new dependency beyond `rustix`" constraint was that phase's own and
does not carry forward.

**D-U3: ordering. Ruled here: L2 first.** Section 2 measures the L2 slice at seven context
functions, five value types and the load path; section 3's clause is the whole 218-pointer surface
and eight test groups. Every phase after this one is gated on an instrument that L2 unlocks --
Phase 6's own exit criterion reads "ooTest concurrency groups pass" and cannot be demonstrated
until the framework loads -- so the slice that reaches L2 is worth more per hour than any other
part of this phase, and it is also the part that tells us earliest whether the design of the
boundary is right. **The cost if this ruling is wrong** is that the L2 slice's shape has to be
generalised when the remaining 211 pointers arrive, which is rework confined to one crate. The
alternative failure -- building the full surface first and discovering at the end that the
framework needs something the design cannot express -- is not confined to anything.

## What this survey did not do

It did not read `ThreadContextStubs.cpp`, the 2,265-line C++ file that populates the vtables; the
spec will. It did not measure what the eight API test groups actually assert, only that they exist
and which binaries they load. It did not check whether `.Pointer` can hold a value -- the class is
in `native_classes.rs` and its `NEW` is deliberately unsupported to match the oracle's 93.967, but
whether an instance can be minted internally for `NewPointer` is unmeasured. And it took no
position on the embedding API beyond noting that two of the eight groups reach it.
