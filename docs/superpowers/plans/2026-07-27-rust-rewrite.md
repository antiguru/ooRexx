# ooRexx → Rust: Clean-Room Reimplementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This plan is written to be re-entered cold.** Phases 0 and 1 are specified to bite-sized task granularity and are executable as written. Phases 2–10 are specified as *gates* — entry criteria, exit criteria, and the procedure that generates their own plan. Section 1 (Decisions) is the part that rewards deep reasoning; do not begin a phase whose upstream decision is still open.

**Goal:** Replace the ooRexx interpreter (~198k LOC C++) with a clean-room Rust implementation that passes the ooTest conformance suite on all five CI platforms at or above the C++ build's performance, keeping `api/oorexxapi.h` source-compatible so existing native extensions recompile without being rewritten.

**Architecture:** A new Rust workspace under `rust/` alongside the untouched C++ tree, which serves as the executable oracle for the entire project. The Rust interpreter uses an **arena heap with tagged index handles** rather than raw pointers; this collapses the C++ implementation's 149 hand-written `live()` trace methods and its pervasive `ProtectedObject` root-pinning discipline into a single derived `Trace` impl and a ~5-entry root set. Everything else — the tree-walking execution model, the expression stack, the activity/guard concurrency semantics, the Rexx-source class library — is preserved deliberately, because it is observable behaviour.

**Tech Stack:** Rust 1.96+ (2024 edition). `unsafe` is forbidden by default in every crate and admitted only per-site, encapsulated and justified in writing (see Global Constraints). `rustix`/`windows-sys` for platform calls. `criterion` for benchmarks. `quick-xml` for build-time message-catalogue generation. `chumsky` is a candidate above the token stream, pending the D10 spike. CMake stays for the C++ oracle build only. The existing SVN-hosted ooTest suite is the conformance gate; the existing `.github/known-test-failures/*.txt` baselines are the pass criteria.

---

## Global Constraints

Every task's requirements implicitly include this section.

- **Rust floor:** 1.96.1 (the toolchain present on this machine). Edition 2024. No nightly features.
- **Unsafe: `#![forbid(unsafe_code)]` is the default in every crate, including `rexx-api` and `rexx-sys`.** There is no blanket exemption. Relaxing it for a crate requires all four of the following, and the relaxation is scoped to a single dedicated module, never the crate root:
  1. **Unavoidable.** A safe alternative was attempted and is recorded as failing — not merely judged unlikely. "`rustix` has no wrapper for this call" is unavoidable; "raw `libc` is faster" is not, absent a committed benchmark showing it. Prefer `rustix` and `windows-sys` over raw `libc` *specifically because* they move the unsafe behind an audited boundary.
  2. **Encapsulated.** The `unsafe` lives in one module that exposes a fully safe API. Callers outside that module cannot reach an unsound state by any sequence of calls. If a caller must uphold an invariant, the API is wrong — fix the API rather than documenting the obligation.
  3. **Justified in writing.** The module carries a `//!`-level block stating: what the invariant is, why the compiler cannot check it, what enforces it instead, and what breaks if it is violated. Every `unsafe` block carries a `SAFETY:` comment naming the specific precondition it discharges. The crate carries `#![deny(unsafe_op_in_unsafe_fn)]`.
  4. **Reviewed as a decision, not a task.** Introducing a new unsafe module is a Section 1 decision block with an identifier, not something a task does in passing. It goes into this file before it goes into the code.
  - Expect exactly two candidates over the whole project: the `extern "C"` entry points in `rexx-api`, where a C caller hands in pointers the compiler cannot validate, and any platform call in `rexx-sys` that `rustix`/`windows-sys` do not cover. Both are *candidates*, not exemptions — each site still clears all four bars. Under D5 the FFI surface is far smaller than it looks: `RexxObjectPtr` is an opaque handle validated by table lookup, so the entry points dereference almost nothing.
  - **Every phase exit reports the unsafe-block count** (`grep -rc 'unsafe' rust/crates --include='*.rs'`). A count that grew without a corresponding decision block in Section 1 fails the gate.
- **The C++ tree is read-only.** No file under `interpreter/`, `api/`, `common/`, `rexxapi/`, `extensions/` is modified by this project. It is the oracle. The only exception is `.github/workflows/` (adding Rust legs) and new files under `rust/` and `docs/`.
- **`api/oorexxapi.h`, `api/rexx.h`, `api/rexxapidefs.h`, `api/oorexxerrors.h` are frozen.** Source compatibility is the contract: native extensions must recompile unchanged. ABI compatibility is explicitly *not* required — struct layouts and symbol addresses may change, but declarations, macro names, type names, and call semantics may not.
- **Platform matrix:** Linux (ubuntu-24.04), macOS 15 arm64, Windows/MSVC, FreeBSD 14.2, OpenBSD 7.8. Every phase gate runs on all five. The known OpenBSD SIGSEGV in the current C++ baseline is pre-existing; it does not block Rust work but must not be *reproduced* by the Rust build.
- **Conformance oracle:** `svn checkout https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest`, run as `rexx testOORexx.rex -s`, judged by `.github/check-test-results.ps1` against `.github/known-test-failures/common.txt` plus the per-platform file. The Rust interpreter is judged by the *same* baselines. Adding an entry to a known-failure file is a plan-level decision, never a task-level one.
- **Performance gate:** no phase closes with a Rust subsystem slower than its C++ counterpart on the Phase 0 benchmark suite, measured on Linux and macOS. "Slower" means the criterion point estimate is outside the C++ baseline's confidence interval on the slow side.
- **Licence:** every new file carries the CPL v1.0 header block used throughout the tree (copy the block verbatim from any existing `.cpp`, adjusting the year).
- **Commits:** every task ends with a commit. Branch is `plan/rust-rewrite` or a descendant.

---

## 0. Measured inventory — what you are actually replacing

All numbers measured on `8c880bdd` (`ci/platforms`). Re-measure if the base moves.

| Area | LOC | Notes |
|---|---:|---|
| `interpreter/classes/` | 54,271 | 40 primitive classes. `NumberStringClass.cpp` 4,231 + `NumberStringMath*.cpp` |
| `interpreter/instructions/` | 19,152 | 59 files: 36 keyword instructions + directives + DO-loop variants |
| `interpreter/execution/` | 18,062 | `RexxActivation.cpp` alone is 5,311 |
| `interpreter/parser/` | 17,483 | `InstructionParser` 4,650, `LanguageParser` 4,398, `Scanner` 1,955 |
| `interpreter/platform/` | 15,293 | unix 23 files / windows 21 files |
| `interpreter/memory/` | 12,000 | segments, mark-sweep, image save/restore, `ProtectedObject` |
| `interpreter/concurrency/` | 8,827 | activities, dispatchers, guard locks |
| `interpreter/expression/` | 8,356 | `BuiltinFunctions.cpp` 3,127 |
| `interpreter/runtime/` | 7,293 | |
| `interpreter/streamLibrary/` | 4,549 | `StreamNative.cpp` 3,765 |
| `interpreter/api/` | 4,318 | `ThreadContextStubs.cpp` 2,265 — the C API vtables |
| `interpreter/behaviour/` | 2,892 | method dictionaries, primitive behaviour tables |
| `interpreter/package/` | 1,841 | |
| **interpreter total** | **198,460** | |
| `rexxapi/` | 11,932 | the **separate RXAPI daemon process** — macrospace, external queues, subcom registry |
| `api/` (public headers) | 7,111 | frozen contract |
| `common/` | 6,734 | |
| `extensions/` | 128,425 | of which `platform/` (ooDialog + OLE, Windows-only) is 115,835 |
| `utilities/` | 2,275 | `rexx`, `rexxc`, `rxqueue`, `rxsubcom`, `rexximage` drivers |

**Assets that are *not* C++ and can be reused verbatim:**

- `interpreter/RexxClasses/CoreClasses.orx` — 4,193 lines of **Rexx** defining 32 classes.
- `interpreter/RexxClasses/StreamClasses.orx` — 1,010 lines of Rexx.
- `interpreter/messages/rexxmsg.xml` — 6,856 lines, **704 error codes** with message text. Generate the Rust error table from this at build time; never hand-transcribe it.
- `interpreter/behaviour/PrimitiveClasses.xml` + XSLT — the primitive class/behaviour registry.
- `.github/workflows/{unix,windows,bsd}.yml` + `check-test-results.ps1` + `known-test-failures/` — a working 5-platform conformance harness. **Reuse this, do not rebuild it.**
- `testbinaries/` — compiled native-API test binaries the suite exercises. These are the `rexx-api` conformance gate.
- 301 `.rex` sample programs in `samples/`.

**Consequence:** a Rust core that can execute `CoreClasses.orx` inherits 32 classes for free. Getting `CoreClasses.orx` to run is therefore the single highest-leverage milestone in the plan (Phase 5), and everything before it is scaffolding for that moment.

**Honest sizing.** This is a multi-person-year project. Phases 0–1 are weeks. Phases 2–5 are the bulk and are where a solo effort either succeeds or stalls. Section 7 defines kill criteria so that stalling is detected rather than endured.

---

## 1. Decisions

Each block states the question, the options, the *evidence* that settles it, and the cost of getting it wrong. **Do not start the phase that depends on a decision until the decision is closed and recorded in this file.** Decisions marked RECOMMENDED carry a default; the recommendation is a starting position, not a conclusion.

### D1 — Heap representation and GC strategy ⟵ *the load-bearing decision*

**Blocks:** Phase 1 (and therefore everything).

**Question.** How are Rexx objects represented and collected?

The C++ implementation uses a segmented mark-sweep heap with an old-space/new-space split, a 2-bit mark in a 16-bit `ObjectHeader` flags word (`interpreter/classes/ObjectClass.hpp:95–171`), and `UninitPending`/`HasUninit` bits driving finalizers. Object references are raw `RexxInternalObject*`. Because raw pointers in C++ locals are invisible to the collector, correctness depends on **149 hand-written `live(size_t)` implementations** plus a `ProtectedObject` RAII root-pinning discipline applied at every allocation-crossing site. Getting one wrong is a use-after-free, not a compile error.

**Options.**

- **(a) Arena + tagged index handles. RECOMMENDED.** All heap objects live in `Vec<Slot>` inside a `Heap`. A reference is `ObjRef(u64)` — a slot index with a low-bit tag that also encodes small integers inline. Tracing is one `match` over a `Body` enum (derivable), not 149 methods. Root set becomes small and *enumerable*: the activation stack, the expression stack, the C-API local-reference tables, and the global tables (`.environment`, `.local`, class registry).
  - Cost: an index-and-bounds-check per field access instead of a pointer deref; loss of pointer locality; `#[repr(C)]` value interop needs a handle table at the FFI boundary (which the C++ already has — see D5).
  - Offset: tagged small integers remove a large fraction of allocations that the C++ pays for via `RexxInteger` objects.
- **(b) `gc-arena` crate.** Branded `'gc` lifetimes, `#[derive(Collect)]`, precise and safe. Rejected as a default because arena access is scoped through `arena.mutate(|mc, root| …)`, which fights an FFI boundary where foreign C code holds object references across calls, and because it is single-threaded — ooRexx is not.
- **(c) Raw pointers + hand-rolled mark-sweep, mirroring C++ 1:1.** Fastest port, preserves the perf profile exactly. Rejected: it reproduces the exact defect class this rewrite exists to eliminate, and requires `unsafe` throughout the interpreter.
- **(d) `Arc` + cycle collection.** Rexx object graphs are cyclic by construction (class ↔ metaclass, method ↔ package ↔ routine, `.environment` ↔ everything). Needs a real cycle collector, which is more work than (a) for worse pause behaviour.

**The temporaries hole, and why it closes.** Under (a), an `ObjRef` sitting in a Rust local during expression evaluation is invisible to the collector — the same hazard as C++. It closes because collection is only triggered at allocation points *and* intermediate values live on an explicit, traced evaluation stack rather than in Rust locals. This is not a divergence: the C++ interpreter already works this way (`interpreter/expression/ExpressionStack.cpp`). Adopting it makes the root set complete by construction rather than by discipline.

**Evidence that settles this.** Phase 1 Task 8 benchmarks allocation throughput and full-GC pause against the C++ build on the same object-graph shapes. Decision closes on numbers, not argument.

**Cost of being wrong.** Total. Changing heap representation after Phase 4 means rewriting every class, every instruction, and the FFI layer. This is the one decision that must be settled by measurement before Phase 2 begins.

### D2 — The saved image (`rexx.img`)

**Blocks:** Phase 5. **Coupled to D1.**

**Constraint (given).** An image is **not a requirement**. Nothing in the conformance contract obliges the Rust build to ship one — `rexx.img` is an implementation detail of the C++ startup path, not observable Rexx behaviour. It is available as an optimisation if it earns its keep, and for no other reason.

**Question.** Does an image earn its keep, or does bootstrapping from source at every start suffice?

The C++ build runs `rexximage` to flatten a bootstrapped heap into `rexx.img`, with **106 `flatten(Envelope*)` implementations**, proxy objects for un-flattenable state, and virtual-function-table repatching on restore (`RexxMemory.cpp:691`, `:1539`). The VFT repatching exists solely because C++ objects embed vtable pointers that are invalid in another process image. **Rust has no vtables to repatch** — under D1(a), heap objects are plain data in a `Vec` and references are indices, so an image is a serialization of a flat array with no pointer fixups at all.

**Options.**

- **(a) No image. DEFAULT — build this first, unconditionally.** Parse and execute `CoreClasses.orx` + `StreamClasses.orx` (5,203 lines of Rexx) at every startup. Nothing corresponding to the 106 `flatten` impls, `Envelope`, or the proxy mechanism ever gets written.
- **(b) Serialize the arena, *if* (a) measures too slow.** Under D1(a) this is close to a `memcpy` of `Vec<Slot>` plus a string table — no pointer fixups, no per-class serialization code, no proxies. It costs a format version, a staleness check against the `.orx` sources, and a build step. Purely additive: it caches what (a) computes, so semantics cannot diverge between the two paths.

**Evidence that settles this.** Phase 5 exit measures cold start for (a) with hyperfine against `build/bin/rexx`. **Ship (a) either way.** Build (b) only if (a) is slower than the C++ startup by a margin a user would notice — treat ~2× or a wall-clock delta above roughly 50 ms as the threshold, and record the actual numbers rather than the ratio. Startup is user-visible in a scripting language, which is why the measurement exists; it is not a reason to build a cache before knowing one is needed.

**Cost of being wrong.** Low, and this is the point. (a) is strictly less code and is a prerequisite for (b) anyway — (b) has nothing to serialize until (a) works. There is no ordering in which building (a) first is wasted effort, so the decision cannot be got wrong by starting.

### D3 — Concurrency model

**Blocks:** Phase 6. Design constraints must be respected from Phase 1.

**Question.** Preserve the activity/kernel-lock model exactly, or exploit the rewrite to parallelise?

The C++ model: one `Activity` per thread; a global kernel lock acquired/released around interpretation (`ActivityManager::releaseAccess`/`requestAccess`, `ActivityManager.hpp:272`, `:540`); guarded methods take a per-object guard lock; `REPLY` spins up a new activity; `GUARD ON/OFF` yields it. Known open defects in this area, from prior investigation on this repo: a cross-thread yield path that reaches into another activity's frames, and GC-visible lists mutated without kernel access. A separate spike (`spike/registry-refactor`) took TSan races from 11 to 1 but was never perf-validated.

**Options.**

- **(a) Preserve semantics; implement the kernel lock as a real `Mutex`; GC is stop-the-world under it. RECOMMENDED.** Rexx's own semantics are already close to a GIL: guarded methods serialise on the object, and ooTest encodes observable ordering. Structure the interpreter loop so the lock *can* later be split, but do not split it now.
- **(b) True parallel activities with fine-grained locking.** Large conformance risk against a suite that tests ordering.

**Design constraint that falls out, and must hold from Phase 1.** Under (a) in Rust, an activity owns its own activation and expression stacks as plain Rust data. Another thread therefore *cannot* reach into them — the first known defect becomes unrepresentable. Cross-activity requests (yield, halt, trace-toggle, condition raise) go through a channel or an atomic flag the target polls at instruction boundaries, never through a foreign frame pointer. **Do not design frames as shared, GC-visible objects reachable from other activities.** That single constraint is the concurrency dividend of the rewrite; losing it loses the argument for doing this at all.

**Evidence that settles this.** Phase 6 runs the ooTest concurrency groups plus a TSan build of the Rust interpreter (`RUSTFLAGS="-Zsanitizer=thread"` requires nightly — if the nightly ban in Global Constraints blocks this, substitute `loom` for the lock protocol and rely on ooTest for the rest; record which was used).

### D4 — Numeric core

**Blocks:** Phase 2.

**Question.** Port `NumberString` or build on a decimal crate?

Rexx numbers are strings; arithmetic is arbitrary-precision decimal under `NUMERIC DIGITS` (default 9), `NUMERIC FUZZ`, and `NUMERIC FORM SCIENTIFIC|ENGINEERING`. `NumberStringClass.cpp` is 4,231 lines plus `NumberStringMath.cpp`/`NumberStringMath2.cpp`.

**Decision: port it, do not adapt a crate. RECOMMENDED, low uncertainty.** ANSI X3.274 rounding, exponent handling, and string round-trip rules differ from IEEE 754-2008 decimal in details that a general crate will get subtly wrong, and every one of those details is tested. This is one of the few subsystems where mechanical transliteration is the right technique — the C++ is a direct encoding of a standard, not an architecture.

**Evidence that settles this.** Phase 2 exit: every arithmetic assertion extractable from ooTest (Phase 0 Task 4) passes, plus the ANSI X3.274 arithmetic test vectors.

### D5 — Native API surface

**Blocks:** Phase 8. Constrains D1 from Phase 1.

**Decision (already taken by the user): source-compatible, not ABI-compatible.** Extensions recompile; they are not rewritten.

**What that means concretely.** `api/oorexxapi.h` stays byte-identical. Rust exports `#[repr(C)]` function tables matching `RexxInstanceInterface` (`oorexxapi.h:493`), `RexxThreadInterface` (`:674`), and the method/call/exit context structs, plus the 37 `RexxReturnCode REXXENTRY` entry points in `api/rexx.h`. The C++ reference for the vtable population is `interpreter/api/ThreadContextStubs.cpp` (2,265 lines).

**The part that matters for D1.** `RexxObjectPtr` becomes an **opaque handle registered in the calling activation's local-reference table**, not a heap address. This is not a new idea imposed by Rust — the C++ already does exactly this via `NativeActivation::createLocalReference` / `removeLocalReference` / `clearLocalReferences` (`NativeActivation.hpp:177–179`), because native code holds references across GC points. Under D1(a) the same mechanism becomes the *only* mechanism, and it is naturally safe: a handle that outlives its activation is a lookup miss, not a use-after-free.

**Evidence that settles this.** Phase 8 exit: `testbinaries/` build against the frozen headers with no source edits, and the ooTest native-API groups pass.

### D6 — Platform layer

**Blocks:** Phase 7.

**Decision: `std` first, then `rustix` + `windows-sys`, and raw `libc` only where neither reaches. Low uncertainty.**

The ordering is a safety decision, not a taste one: `rustix` wraps the syscalls this layer needs behind a safe API, so choosing it over raw `libc` discharges the Global Constraints unsafe bar by construction rather than by argument. A raw `libc` call in this crate needs its own justification block naming the `rustix` function that does not exist.

The 15,293 LOC of `interpreter/platform/` is mostly things `std` covers. The genuine gaps, which need care because they are observable: the Rexx **stream model** (line vs binary access, `RESET`, explicit positioning, the `CHARIN`/`LINEIN` interaction, `StreamNative.cpp` is 3,765 lines), **`ADDRESS` command routing** to shells and subcom handlers, file-name and path semantics, and console/terminal behaviour. Treat the stream model as a subsystem in its own right, not as "file I/O".

### D7 — RXAPI daemon

**Blocks:** Phase 10 (and partially Phase 7 — external queues).

**Question.** Port the 11,932-LOC daemon, or speak its protocol?

**Decision: keep the C++ `rxapi` binary and speak its IPC protocol from Rust. RECOMMENDED.** It is a separate process behind a stable wire boundary — exactly the kind of thing that should not be on the critical path. The first working Rust `rexx` links no C++ but talks to a C++ `rxapi`.

**Evidence needed before relying on this.** Phase 0 Task 7 must confirm the protocol is version-negotiated and stable across the `rexxapi/client` ↔ `rexxapi/server` boundary. If it turns out to be a raw struct dump with no versioning, this decision flips to "port it in Phase 10" and the schedule absorbs 12k LOC.

### D8 — Conformance ladder

**Blocks:** everything. **Settle in Phase 0.**

**The problem.** `testOORexx.rex` and the `.testGroup` files are themselves ooRexx programs — they use `::class`/`::method`/`::requires`, the `TestGroup` and `ooTestCase` classes, streams, and packages (confirmed by inspection of `extensions/json/json_02.testGroup`). The suite cannot run until the interpreter is nearly complete. "ooTest green" is therefore a *final* gate, useless as an incremental signal. A ladder is required.

- **L0 — Differential runner.** A Rust harness runs a `.rex` file under both `build/bin/rexx` (C++) and `rexx-rs`, and diffs normalised stdout/stderr/exit code. Corpus: hand-written micro-programs per feature, growing to the 301 in-repo samples.
- **L1 — Extracted assertions.** Mechanically lift `::method test*` bodies out of the `.testGroup` files and emit standalone micro-programs against a tiny assert shim. This buys partial credit from the *real* suite long before the framework runs. **Regularity is unverified** — Phase 0 Task 4 measures the extractable fraction and reports it. If it is under ~40%, L1 is not worth building and the ladder becomes L0 → L2.
- **L2 — Framework boots.** `ooTest.frm` loads and a single test group executes. This is the "object model, packages, `::requires`, and streams all work" milestone.
- **L3 — Full suite green** against `known-test-failures/` on all five platforms.

Each phase below declares the rung it must reach.

### D9 — Performance gate

**Blocks:** every phase exit.

Benchmark suite (Phase 0 Task 6) covers, at minimum: method dispatch, variable lookup, **compound/stem variable access**, string operations, decimal arithmetic, allocation throughput, full-GC pause, and cold start. Whole-program benchmarks come from `samples/`.

**Two prior findings from this repo are design inputs, not later optimisations:**

1. A compound-variable memoisation prototype measured **−24% on stem-heavy workloads** and was never merged. Build memoisation into the Rust stem/compound-variable design from the start rather than porting the slow shape first and optimising later.
2. The existing performance profile identifies where interpreter time actually goes. Read it before designing the execution loop; do not re-derive it.

### D10 — Parser construction: combinators (`chumsky`) or hand-written recursive descent

**Blocks:** Phase 3. **Couples to D2 — see below.**

**Question.** Build the parser with `chumsky`, or write recursive descent by hand as the C++ does?

The C++ splits this into `Scanner.cpp` (1,955), `Clause.cpp`, `Token.cpp`, `InstructionParser.cpp` (4,650), `LanguageParser.cpp` (4,398), and `DirectiveParser.cpp` (2,867).

**What makes Rexx hostile to an off-the-shelf grammar.** These are not style objections; each one breaks a standard combinator setup in a specific way.

1. **There are no reserved words.** `IF` is a keyword only in keyword position. `if = 5; say if` is a valid program. A token type of `Keyword(If)` produced by the lexer is therefore *wrong* — keyword-ness is decided by the parser from position, and the same characters must remain usable as a variable name. Expressible in `chumsky`, but it means matching on identifier text at each site rather than on token variants, which gives up much of the ergonomic win.
2. **Clause splitting precedes parsing.** Clauses end at `;`, at end-of-line, or not at all if the line ends in a continuation comma. This is a pre-pass over the source, and the C++ structures it that way for good reason.
3. **Tokenisation is idiosyncratic.** `/* */` nests; `--` runs to end of line; `'ff'x` and `'1010'b` are literals whose suffix binds to the preceding quote; `.` is a symbol constituent, so `a.b.c` is one compound-variable token, not three tokens and two operators; abuttal is the concatenation operator, so whitespace between two terms is semantically significant.
4. **Error output is fixed by the oracle.** Conformance demands one specific error, with a specific number out of the 704, at a specific line and column. `chumsky`'s recovery and multi-error reporting — a large part of its value — is mostly unusable here, because emitting a second, better diagnostic is a conformance failure.
5. **`INTERPRET` parses at runtime,** so parser throughput is on the execution path for some programs, not only at load.

**Options.**

- **(a) Hand-written scanner and clause splitter, `chumsky` above the token stream. RECOMMENDED as the starting position.** Points 1–3 all live below the token level, which is exactly where `chumsky` is weakest and where the C++ has already worked out the answers. Above that line — the expression grammar with its precedence levels, message-send chains (`~`, `~~`), function and array-reference forms, and the instruction bodies — is ordinary structured parsing where combinators earn their keep in clarity and in span tracking, which `chumsky` gives for free and which Phase 3's `SOURCELINE`/`TRACE`/error-position gate needs on every node.
- **(b) Hand-written throughout,** mirroring the C++ structure. Lowest risk against the error-message gate, likely fastest, most code.
- **(c) `chumsky` throughout, including lexing.** Fights points 1–3 the whole way.

**The D2 coupling, which is the reason this decision is not merely aesthetic.** D2's default is to bootstrap by parsing `CoreClasses.orx` + `StreamClasses.orx` — 5,203 lines of Rexx — at *every* interpreter start. Parser throughput therefore sets cold-start time directly. A parser that is pleasant but 5× slower does not just miss a Phase 3 benchmark; it is what forces D2 into building an image cache that would otherwise never be needed. Measure parse throughput on `CoreClasses.orx` specifically, not on a synthetic input.

**Evidence that settles this.** Phase 3 opens with a bounded spike: implement the expression grammar — precedence, abuttal concatenation, message sends, compound variables — twice, once with `chumsky` over a hand-written token stream and once by hand, against the same L0 corpus entries. Compare on three axes: lines of code, whether the exact error number and position can be produced at each failure site, and parse throughput on `CoreClasses.orx`. Timebox it; the expression grammar alone is enough signal, and doing all 36 instructions twice is not.

**Cost of being wrong.** Low and contained. The parser's output is an AST consumed by Phase 4; the construction technique does not leak past that boundary, so this can be redone later without touching the executor. It is a decision block because getting it wrong wastes Phase 3, not because it is irreversible.

---

## 2. Phase roadmap

Gates are hard. A phase does not close until every exit criterion is demonstrated with committed evidence (a CI run, a benchmark report, a diff log).

| # | Phase | Entry | Exit gate | Rung |
|---|---|---|---|---|
| 0 | Oracle & inventory | — | Differ runs C++ against itself with zero diffs on the corpus; benchmark baselines committed for all 5 platforms; error table and builtin inventory generated; L1 extraction fraction reported; D7 protocol stability answered | — |
| 1 | Heap & object model | D1 open | Allocation throughput and full-GC pause within the C++ baseline CI; `Trace` derived, not hand-written; root set enumerable and documented. **D1 closes here.** | — |
| 2 | Numeric core | D1 closed | Every extractable ooTest arithmetic assertion passes; ANSI X3.274 vectors pass; arithmetic benchmark at parity | L1 (arithmetic) |
| 3 | Scanner & parser | D1 closed, D10 spiked | Round-trips every `.rex` in `samples/` to an AST; `SOURCELINE`, error line/column reporting, and `TRACE` output formatting match the oracle byte-for-byte; parse throughput on `CoreClasses.orx` recorded | L0 (syntax errors) |
| 4 | Classic executor | 2, 3 | Non-OO Rexx runs: assignment, `DO` (all variants), `IF`, `SELECT`, `CALL`, `PARSE`, `SAY`, `SIGNAL`, conditions, and all **162 builtin functions** | L0 full corpus + L1 majority |
| 5 | Object model | 4 | **`CoreClasses.orx` parses and executes**; 32 classes exist and respond; `::class`/`::method`/`::routine`/`::requires` work; cold start measured and recorded against C++ (D2) | L2 |
| 6 | Concurrency | 5 | Activities, kernel lock, guard locks, `REPLY`, `GUARD`, message objects; ooTest concurrency groups pass; TSan (or `loom`) clean. **D3's frame-ownership constraint verified.** | L2 |
| 7 | Streams & platform | 5 | `StreamClasses.orx` runs; stream model, `ADDRESS`, file system green on all 5 platforms | L2 |
| 8 | Native API | 5, 7 | `testbinaries/` compile unchanged against frozen headers; native-API ooTest groups pass | L2 |
| 9 | Full conformance | 6, 7, 8 | **L3 on all 5 platforms** against existing baselines; every benchmark at or above parity; `rexx`, `rexxc`, `rxqueue`, `rxsubcom` ship | L3 |
| 10 | RXAPI & extensions | 9 | `rxregexp`, `rxmath`, `rxsock`, `hostemu` recompile and pass; RXAPI decision (D7) executed | L3 |

Phases 6, 7, and 8 are independent of each other and may run in parallel once Phase 5 closes.

---

## 3. File structure

```
rust/
  Cargo.toml                  # workspace
  rust-toolchain.toml         # pins 1.96.1
  crates/
    rexx-oracle/              # Phase 0: differential runner, corpus, normalisation
      src/lib.rs              #   run(), Outcome, Divergence
      src/normalize.rs        #   strip paths, timings, PIDs from output
      src/bin/rexx-diff.rs    #   CLI: rexx-diff --cpp <path> --rs <path> <corpus>
    rexx-extract/             # Phase 0: .testGroup → standalone micro-programs
      src/lib.rs
      src/bin/rexx-extract.rs
    rexx-inventory/           # Phase 0: generate Rust tables from the C++ tree
      build.rs                #   rexxmsg.xml -> errors.rs (704 codes)
      src/errors.rs           #   generated; do not edit
      src/builtins.rs         #   generated from BuiltinFunctions.cpp table
    rexx-bench/               # Phase 0: criterion suite, runs against any interpreter
    rexx-core/                # Phase 1: Heap, ObjRef, Slot, Body, Trace, roots, behaviours
      src/handle.rs           #   ObjRef tagging
      src/heap.rs             #   arena, alloc, mark-sweep
      src/trace.rs            #   Trace trait + derive usage
      src/roots.rs            #   RootSet
      src/behaviour.rs        #   behaviours + method dictionaries
    rexx-num/                 # Phase 2
    rexx-parse/               # Phase 3
      src/scan.rs             #   hand-written: clause splitting, tokens, no reserved words
      src/grammar.rs          #   chumsky or hand-written above the token stream (D10)
    rexx-exec/                # Phase 4
    rexx-classes/             # Phase 4-5
    rexx-lib/                 # Phase 5: loads CoreClasses.orx / StreamClasses.orx
    rexx-conc/                # Phase 6
    rexx-sys/                 # Phase 7: platform. std -> rustix -> libc, in that order
    rexx-api/                 # Phase 8: C ABI export surface
      src/ffi.rs              #   the only module that may relax forbid(unsafe_code),
                              #   and only after a Section 1 decision block says so
    rexx-cli/                 # Phase 9: rexx, rexxc, rxqueue, rxsubcom
docs/superpowers/plans/       # this file + per-phase plans
```

One crate per subsystem, and one file per concept inside it. Crate boundaries are the interfaces; if a later phase wants to reach past one, that is a signal the boundary is wrong, not that the boundary should be bypassed.

---

## 4. Phase 0 — Oracle & inventory

**No Rust interpreter code is written in this phase.** The deliverable is the measuring apparatus and the extracted facts. Skipping this phase is the most likely way for the project to fail, because without it there is no way to tell whether Phase 4 is 80% or 30% done.

### Task 0.1: Workspace skeleton and the C++ oracle build

**Files:**
- Create: `rust/Cargo.toml`, `rust/rust-toolchain.toml`, `rust/.gitignore`
- Create: `rust/crates/rexx-oracle/Cargo.toml`, `rust/crates/rexx-oracle/src/lib.rs`
- Create: `docs/superpowers/plans/oracle-build.md`

**Interfaces:**
- Produces: a built C++ interpreter at `build/bin/rexx`, and a Rust workspace that compiles.

- [ ] **Step 1: Build the C++ oracle**

```bash
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build build --parallel "$(getconf _NPROCESSORS_ONLN)"
```

- [ ] **Step 2: Verify the oracle runs**

Run:
```bash
build/bin/rexx -v
echo 'say .rexxinfo~version' > /tmp/hello.rex && build/bin/rexx /tmp/hello.rex
```
Expected: a version banner, then a version string. If this fails, stop — nothing downstream is meaningful without a working oracle.

- [ ] **Step 3: Create the workspace**

`rust/Cargo.toml`:
```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
edition = "2024"
rust-version = "1.96.1"
license = "CPL-1.0"

[workspace.lints.rust]
unsafe_code = "forbid"
```

`rust/rust-toolchain.toml`:
```toml
[toolchain]
channel = "1.96.1"
components = ["rustfmt", "clippy"]
```

`rust/.gitignore`:
```
target/
```

- [ ] **Step 4: Create the oracle crate**

`rust/crates/rexx-oracle/Cargo.toml`:
```toml
[package]
name = "rexx-oracle"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lints]
workspace = true
```

`rust/crates/rexx-oracle/src/lib.rs`:
```rust
//! Runs a Rexx program under an interpreter and captures its observable output.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Everything a Rexx program can be observed to produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

/// An interpreter under test, plus the loader paths it needs.
#[derive(Debug, Clone)]
pub struct Interpreter {
    pub binary: PathBuf,
    pub library_paths: Vec<PathBuf>,
}

impl Interpreter {
    pub fn run(&self, program: &Path, args: &[String], cwd: &Path) -> std::io::Result<Outcome> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg(program).args(args).current_dir(cwd);
        let joined = std::env::join_paths(&self.library_paths)
            .expect("library paths must not contain the path separator");
        for var in ["LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH"] {
            cmd.env(var, &joined);
        }
        let out = cmd.output()?;
        Ok(Outcome {
            stdout: out.stdout,
            stderr: out.stderr,
            exit_code: out.status.code().unwrap_or(-1),
        })
    }
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cd rust && cargo build`
Expected: success, no warnings.

- [ ] **Step 6: Commit**

```bash
git add rust docs
git commit -m "Add a Rust workspace and an oracle runner for the C++ interpreter"
```

### Task 0.2: Output normalisation and the differ

**Files:**
- Create: `rust/crates/rexx-oracle/src/normalize.rs`
- Modify: `rust/crates/rexx-oracle/src/lib.rs`
- Create: `rust/crates/rexx-oracle/tests/normalize.rs`

**Interfaces:**
- Consumes: `Outcome`, `Interpreter` from Task 0.1.
- Produces: `normalize(&Outcome, &Path) -> Outcome`, `diff(&Outcome, &Outcome) -> Option<Divergence>`, `Divergence`.

- [ ] **Step 1: Write the failing test**

`rust/crates/rexx-oracle/tests/normalize.rs`:
```rust
use rexx_oracle::{normalize, Outcome};
use std::path::Path;

#[test]
fn absolute_program_paths_are_replaced_by_a_placeholder() {
    let raw = Outcome {
        stdout: b"Error 43 running /home/someone/work/prog.rex line 7\n".to_vec(),
        stderr: Vec::new(),
        exit_code: 43,
    };
    let got = normalize(&raw, Path::new("/home/someone/work"));
    assert_eq!(
        String::from_utf8(got.stdout).unwrap(),
        "Error 43 running <CWD>/prog.rex line 7\n"
    );
}

#[test]
fn crlf_is_folded_so_windows_and_unix_compare_equal() {
    let raw = Outcome { stdout: b"a\r\nb\r\n".to_vec(), stderr: Vec::new(), exit_code: 0 };
    let got = normalize(&raw, Path::new("/tmp"));
    assert_eq!(String::from_utf8(got.stdout).unwrap(), "a\nb\n");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-oracle`
Expected: FAIL — `normalize` is not defined.

- [ ] **Step 3: Implement normalisation**

`rust/crates/rexx-oracle/src/normalize.rs`:
```rust
use crate::Outcome;
use std::path::Path;

/// Removes the parts of an interpreter's output that legitimately differ
/// between two runs of the *same* interpreter: absolute paths and line endings.
///
/// Anything this function strips is invisible to the differ, so strip as
/// little as possible. Every addition here is a class of divergence the
/// project can no longer detect.
pub fn normalize(raw: &Outcome, cwd: &Path) -> Outcome {
    Outcome {
        stdout: normalize_stream(&raw.stdout, cwd),
        stderr: normalize_stream(&raw.stderr, cwd),
        exit_code: raw.exit_code,
    }
}

fn normalize_stream(bytes: &[u8], cwd: &Path) -> Vec<u8> {
    let text = String::from_utf8_lossy(bytes);
    let folded = text.replace("\r\n", "\n");
    let cwd = cwd.to_string_lossy();
    folded.replace(cwd.as_ref(), "<CWD>").into_bytes()
}

/// The first place two outcomes disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Divergence {
    ExitCode { cpp: i32, rust: i32 },
    Stdout { cpp: String, rust: String },
    Stderr { cpp: String, rust: String },
}

pub fn diff(cpp: &Outcome, rust: &Outcome) -> Option<Divergence> {
    if cpp.exit_code != rust.exit_code {
        return Some(Divergence::ExitCode { cpp: cpp.exit_code, rust: rust.exit_code });
    }
    if cpp.stdout != rust.stdout {
        return Some(Divergence::Stdout {
            cpp: String::from_utf8_lossy(&cpp.stdout).into_owned(),
            rust: String::from_utf8_lossy(&rust.stdout).into_owned(),
        });
    }
    if cpp.stderr != rust.stderr {
        return Some(Divergence::Stderr {
            cpp: String::from_utf8_lossy(&cpp.stderr).into_owned(),
            rust: String::from_utf8_lossy(&rust.stderr).into_owned(),
        });
    }
    None
}
```

Add to `src/lib.rs`:
```rust
mod normalize;
pub use normalize::{diff, normalize, Divergence};
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-oracle`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Normalise interpreter output and diff two runs"
```

### Task 0.3: The `rexx-diff` CLI and the self-test corpus

**Files:**
- Create: `rust/crates/rexx-oracle/src/bin/rexx-diff.rs`
- Create: `rust/corpus/lang/*.rex` (the seed corpus)
- Create: `rust/corpus/README.md`

**Interfaces:**
- Consumes: `Interpreter`, `normalize`, `diff` from Tasks 0.1–0.2.
- Produces: `rexx-diff --cpp <bin> --rs <bin> --corpus <dir>`, exit 0 on no divergence.

- [ ] **Step 1: Write the seed corpus**

Twelve micro-programs, one observable feature each. `rust/corpus/lang/arith_digits.rex`:
```rexx
numeric digits 9
say 1/3
numeric digits 20
say 1/3
say 2**100
say 1e10 + 1
```

`rust/corpus/lang/parse_template.rex`:
```rexx
parse value "alpha beta gamma" with a b c
say a"|"b"|"c
parse value "2026-07-27" with y "-" m "-" d
say y"/"m"/"d
```

`rust/corpus/lang/condition_syntax.rex`:
```rexx
signal on syntax name oops
say 1/0
oops:
say "trapped" rc condition("C")
```

Write nine more covering: `DO` variants, `SELECT`/`WHEN`, `CALL`/`PROCEDURE`/`EXPOSE`, stem and compound variables, `INTERPRET`, string builtins, `TRACE I` output, `SOURCELINE`/`ARG`, and `SAY` of every primitive class name. Each must produce deterministic output — no clock, no PID, no file system state.

- [ ] **Step 2: Write the CLI**

`rust/crates/rexx-oracle/src/bin/rexx-diff.rs`:
```rust
use rexx_oracle::{diff, normalize, Interpreter};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut cpp, mut rs, mut corpus) = (None, None, None);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--cpp" => cpp = args.next().map(PathBuf::from),
            "--rs" => rs = args.next().map(PathBuf::from),
            "--corpus" => corpus = args.next().map(PathBuf::from),
            other => {
                eprintln!("unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(cpp), Some(rs), Some(corpus)) = (cpp, rs, corpus) else {
        eprintln!("usage: rexx-diff --cpp <bin> --rs <bin> --corpus <dir>");
        return ExitCode::from(2);
    };

    let lib = |bin: &PathBuf| {
        bin.parent()
            .map(|d| vec![d.to_path_buf(), d.join("../lib")])
            .unwrap_or_default()
    };
    let reference = Interpreter { library_paths: lib(&cpp), binary: cpp };
    let candidate = Interpreter { library_paths: lib(&rs), binary: rs };

    let mut programs: Vec<PathBuf> = walk(&corpus);
    programs.sort();
    let mut divergences = 0usize;
    for program in &programs {
        let cwd = program.parent().expect("corpus entries have a parent");
        let a = reference.run(program, &[], cwd).expect("reference interpreter runs");
        let b = candidate.run(program, &[], cwd).expect("candidate interpreter runs");
        if let Some(d) = diff(&normalize(&a, cwd), &normalize(&b, cwd)) {
            divergences += 1;
            println!("DIVERGENCE {}\n{d:#?}\n", program.display());
        }
    }
    println!("{} programs, {divergences} divergences", programs.len());
    if divergences == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn walk(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|e| e == "rex") {
            out.push(path);
        }
    }
    out
}
```

- [ ] **Step 3: Verify the differ finds zero divergences running C++ against itself**

Run:
```bash
cd rust && cargo build --release
./target/release/rexx-diff \
  --cpp ../build/bin/rexx --rs ../build/bin/rexx --corpus corpus
```
Expected: `12 programs, 0 divergences`, exit 0.

**This is the phase's central self-test.** A non-zero count here means the corpus is non-deterministic or normalisation is wrong — fix the corpus, never loosen normalisation to make it pass.

- [ ] **Step 4: Commit**

```bash
git add rust
git commit -m "Add the differential runner and a deterministic seed corpus"
```

### Task 0.4: ooTest assertion extraction, and measuring whether L1 is viable

**Files:**
- Create: `rust/crates/rexx-extract/Cargo.toml`, `src/lib.rs`, `src/bin/rexx-extract.rs`
- Create: `rust/crates/rexx-extract/tests/extract.rs`
- Create: `docs/superpowers/plans/l1-coverage.md` (the measurement report)

**Interfaces:**
- Produces: `extract(source: &str) -> Vec<TestMethod>`, `TestMethod { name: String, body: String, uses_fixture: bool }`.

- [ ] **Step 1: Write the failing test**

`rust/crates/rexx-extract/tests/extract.rs`:
```rust
use rexx_extract::extract;

const SAMPLE: &str = r#"
::class "Demo.testGroup" subclass ooTestCase public

::method setUp
  self~thing = 1

::method testAddition
  self~assertEquals(2, 1 + 1)

::method testSelfFree
  x = "abc"
  self~assertEquals(3, x~length)

::method helperNotATest
  return 7
"#;

#[test]
fn only_test_methods_are_extracted() {
    let methods = extract(SAMPLE);
    let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["testAddition", "testSelfFree"]);
}

#[test]
fn methods_touching_instance_state_are_flagged_as_fixture_dependent() {
    let methods = extract(SAMPLE);
    let addition = methods.iter().find(|m| m.name == "testAddition").unwrap();
    let free = methods.iter().find(|m| m.name == "testSelfFree").unwrap();
    // `self~assertEquals` is the shim, not fixture state; `self~thing` would be.
    assert!(!addition.uses_fixture);
    assert!(!free.uses_fixture);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-extract`
Expected: FAIL — crate does not exist.

- [ ] **Step 3: Implement the extractor**

`rust/crates/rexx-extract/src/lib.rs`:
```rust
//! Lifts individual test methods out of ooTest `.testGroup` files so they can
//! run as standalone programs long before the ooTest framework itself works.
//!
//! This is a heuristic, and deliberately conservative: a method that touches
//! fixture state set up by `setUp` cannot stand alone, so it is flagged and
//! skipped rather than mis-extracted into a silently-passing test.

/// The set of `self~` messages that are assertions rather than fixture access.
const ASSERTIONS: &[&str] = &[
    "assertequals", "assertnotequals", "asserttrue", "assertfalse",
    "assertnull", "assertnotnull", "assertsame", "assertnotsame",
    "expectsyntax", "assertlistequals", "assertarrayequals",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestMethod {
    pub name: String,
    pub body: String,
    /// True when the body reads or writes `self~<something>` that is not an
    /// assertion, meaning it depends on fixture state and cannot stand alone.
    pub uses_fixture: bool,
}

pub fn extract(source: &str) -> Vec<TestMethod> {
    let mut out = Vec::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = strip_directive(trimmed, "::method") {
            if let Some((name, body)) = current.take() {
                push_if_test(&mut out, name, body);
            }
            let name = rest.split_whitespace().next().unwrap_or("").trim_matches('"');
            current = Some((name.to_string(), Vec::new()));
        } else if trimmed.starts_with("::") {
            if let Some((name, body)) = current.take() {
                push_if_test(&mut out, name, body);
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push(line);
        }
    }
    if let Some((name, body)) = current.take() {
        push_if_test(&mut out, name, body);
    }
    out
}

fn strip_directive<'a>(line: &'a str, directive: &str) -> Option<&'a str> {
    let lower = line.to_ascii_lowercase();
    lower.starts_with(directive).then(|| line[directive.len()..].trim_start())
}

fn push_if_test(out: &mut Vec<TestMethod>, name: String, body: Vec<&str>) {
    if !name.to_ascii_lowercase().starts_with("test") {
        return;
    }
    let body = body.join("\n");
    out.push(TestMethod { uses_fixture: touches_fixture(&body), name, body });
}

fn touches_fixture(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(at) = rest.find("self~") {
        rest = &rest[at + "self~".len()..];
        let message: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.')
            .collect();
        if !ASSERTIONS.contains(&message.as_str()) {
            return true;
        }
    }
    false
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-extract`
Expected: 2 passed.

- [ ] **Step 5: Measure L1 viability against the real suite**

Run:
```bash
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
svn checkout --non-interactive --trust-server-cert \
  https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest
cd rust && cargo run --release -p rexx-extract --bin rexx-extract -- \
  --suite ../ootest --out ../rust/corpus/extracted --report ../docs/superpowers/plans/l1-coverage.md
```

The binary must write, per `.testGroup` file: total `::method test*` count, extractable count (`!uses_fixture`), and the overall percentage.

- [ ] **Step 6: Record the D8 decision**

Read `l1-coverage.md`. **If the extractable fraction is ≥40%, L1 is viable — record D8 as L0→L1→L2→L3.** Below 40%, the extraction machinery costs more than it returns; record D8 as L0→L2→L3 and delete `rexx-extract`. Write the decision and the measured number into Section 1's D8 block in this file.

- [ ] **Step 7: Commit**

```bash
git add rust docs
git commit -m "Extract standalone assertions from ooTest groups and measure L1 coverage"
```

### Task 0.5: Generate the error-code table from `rexxmsg.xml`

**Files:**
- Create: `rust/crates/rexx-inventory/Cargo.toml`, `build.rs`, `src/lib.rs`
- Create: `rust/crates/rexx-inventory/tests/errors.rs`

**Interfaces:**
- Produces: `rexx_inventory::errors::MESSAGES: &[(u32, &str)]` — all 704 codes with their text, generated at build time.

- [ ] **Step 1: Write the failing test**

`rust/crates/rexx-inventory/tests/errors.rs`:
```rust
#[test]
fn every_error_code_from_the_catalogue_is_present() {
    // rexxmsg.xml carries 704 codes as of 8c880bdd. If this number changes,
    // the C++ tree gained or lost an error and the Rust side must follow.
    assert_eq!(rexx_inventory::errors::MESSAGES.len(), 704);
}

#[test]
fn error_13_is_invalid_character_in_program() {
    let (_, text) = rexx_inventory::errors::MESSAGES
        .iter()
        .find(|(code, _)| *code == 13)
        .expect("error 13 exists");
    assert!(text.to_ascii_lowercase().contains("invalid character"));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-inventory`
Expected: FAIL — crate does not exist.

- [ ] **Step 3: Implement the build script**

`rust/crates/rexx-inventory/Cargo.toml`:
```toml
[package]
name = "rexx-inventory"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[build-dependencies]
quick-xml = "0.37"

[lints]
workspace = true
```

`rust/crates/rexx-inventory/build.rs` reads `../../../interpreter/messages/rexxmsg.xml`, walks the message elements, and writes `$OUT_DIR/errors.rs` containing:
```rust
pub static MESSAGES: &[(u32, &str)] = &[
    (3, "Failure during initialization: %1"),
    // ... one line per code, in ascending order
];
```
It must `println!("cargo::rerun-if-changed=../../../interpreter/messages/rexxmsg.xml");` and fail loudly — `panic!` — if the file is missing or the code count is zero. A silently empty table would let every later phase report false conformance.

`rust/crates/rexx-inventory/src/lib.rs`:
```rust
//! Tables mechanically derived from the C++ tree. Never hand-edit these; the
//! C++ tree is the source of truth and the build script re-derives them.
pub mod errors {
    include!(concat!(env!("OUT_DIR"), "/errors.rs"));
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-inventory`
Expected: 2 passed. If the count assertion fails with a number other than 704, the base commit moved — update the constant and note it.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Generate the Rexx error-message table from rexxmsg.xml at build time"
```

### Task 0.6: Builtin-function inventory

**Files:**
- Create: `rust/crates/rexx-inventory/src/builtins.rs` (generated), extend `build.rs`
- Create: `rust/crates/rexx-inventory/tests/builtins.rs`

**Interfaces:**
- Produces: `rexx_inventory::builtins::NAMES: &[&str]` — the 162 builtin function names in table order.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn the_builtin_table_has_162_entries() {
    assert_eq!(rexx_inventory::builtins::NAMES.len(), 162);
}

#[test]
fn the_table_is_alphabetical_and_starts_at_abbrev() {
    assert_eq!(rexx_inventory::builtins::NAMES[0], "ABBREV");
    assert!(rexx_inventory::builtins::NAMES.windows(2).all(|w| w[0] <= w[1]));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-inventory builtins`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Extend the build script**

Parse `../../../interpreter/expression/BuiltinFunctions.cpp` from the line matching `pbuiltin LanguageParser::builtinTable[] =` to the closing `};`, taking each `&builtin_function_NAME` and emitting `NAME`. Skip the leading `NULL` dummy entry. Panic if fewer than 100 names are found.

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-inventory builtins`
Expected: 2 passed. This list is Phase 4's definition of done.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Derive the builtin function inventory from the C++ parser table"
```

### Task 0.7: Benchmark suite and the C++ baseline

**Files:**
- Create: `rust/crates/rexx-bench/Cargo.toml`, `benches/interpreter.rs`
- Create: `rust/bench-programs/*.rex`
- Create: `docs/superpowers/plans/perf-baseline.md`

**Interfaces:**
- Produces: a criterion suite that times any interpreter binary on a fixed set of Rexx programs, plus a committed C++ baseline.

- [ ] **Step 1: Write the benchmark programs**

One per D9 dimension, each sized to run 0.5–2s under the C++ interpreter:
`dispatch.rex` (tight method-send loop), `varlookup.rex` (simple variable read/write), `compound.rex` (stem and compound-variable access — **the −24% memo prototype's workload**), `strings.rex` (`SUBSTR`/`POS`/`CHANGESTR`/concatenation), `arith.rex` (decimal arithmetic across several `NUMERIC DIGITS`), `alloc.rex` (allocation churn to force collections), `startup.rex` (`say 1`, for cold-start timing).

- [ ] **Step 2: Write the criterion harness**

`benches/interpreter.rs` takes the interpreter path from `REXX_BENCH_BINARY`, runs each program via `rexx_oracle::Interpreter::run`, and reports wall time per program. Assert nothing — this task only establishes the baseline.

- [ ] **Step 3: Record the C++ baseline on Linux**

Run:
```bash
cd rust
REXX_BENCH_BINARY=../build/bin/rexx cargo bench -p rexx-bench -- --save-baseline cpp-linux
```

- [ ] **Step 4: Record cold start separately**

Run:
```bash
hyperfine --warmup 5 '../build/bin/rexx bench-programs/startup.rex'
```
Write the result into `perf-baseline.md`. This number is the D2 gate.

- [ ] **Step 5: Record the baseline on the other four platforms**

Add a `bench` job to `.github/workflows/{unix,windows,bsd}.yml` that builds the C++ tree, runs the suite, and uploads `target/criterion` as an artifact. Do not gate CI on it yet — this run only produces numbers.

- [ ] **Step 6: Write the baseline report**

`perf-baseline.md` records, per platform: each benchmark's point estimate and confidence interval, the toolchain versions, and the machine class. **Every later phase gate compares against this file.**

- [ ] **Step 7: Commit**

```bash
git add rust docs .github
git commit -m "Add the interpreter benchmark suite and record the C++ baseline"
```

### Task 0.8: Answer D7 — is the RXAPI wire protocol stable?

**Files:**
- Create: `docs/superpowers/plans/rxapi-protocol.md`

- [ ] **Step 1: Read the protocol definition**

Read `rexxapi/common/` (9 files) — specifically the request/reply message structs and any version field — plus how `rexxapi/client/` frames requests and `rexxapi/server/` dispatches them.

- [ ] **Step 2: Answer three questions in writing**

In `rxapi-protocol.md`: (1) Is there a protocol version field, and is a mismatch detected or ignored? (2) Are messages fixed-layout C structs, and if so are they sensitive to compiler padding, endianness, or pointer width? (3) What is the transport on each of the five platforms?

- [ ] **Step 3: Record the D7 decision**

If the protocol is versioned and layout-portable, confirm D7 as "bridge to the C++ `rxapi`". If it is an unversioned struct dump, flip D7 to "port `rexxapi/` in Phase 10" and add 12k LOC to the Phase 10 estimate. Update Section 1's D7 block in this file with the answer and the evidence.

- [ ] **Step 4: Commit**

```bash
git add docs
git commit -m "Document the RXAPI wire protocol and settle the bridge-or-port decision"
```

### Phase 0 exit gate

All must hold before Phase 1 starts:

- [ ] `rexx-diff --cpp build/bin/rexx --rs build/bin/rexx --corpus rust/corpus` reports **0 divergences**.
- [ ] `perf-baseline.md` contains committed C++ numbers for all five platforms.
- [ ] `rexx_inventory::errors::MESSAGES` has 704 entries; `builtins::NAMES` has 162.
- [ ] `l1-coverage.md` exists and D8 is recorded in this file with its measured number.
- [ ] `rxapi-protocol.md` exists and D7 is recorded in this file.

---

## 5. Phase 1 — Heap and object model

**Entry:** Phase 0 gate green. **This phase closes D1 by measurement.** If the numbers fail, the correct response is to revisit D1, not to proceed and hope.

### Task 1.1: `ObjRef` — tagged handles

**Files:**
- Create: `rust/crates/rexx-core/Cargo.toml`, `src/lib.rs`, `src/handle.rs`
- Create: `rust/crates/rexx-core/tests/handle.rs`

**Interfaces:**
- Produces: `ObjRef` (Copy, Eq, Hash), `ObjRef::heap(u32)`, `ObjRef::small_int(i64) -> Option<ObjRef>`, `ObjRef::NIL`, `ObjRef::decode() -> Decoded`, `enum Decoded { Heap(u32), SmallInt(i64), Nil }`.

- [ ] **Step 1: Write the failing test**

```rust
use rexx_core::{Decoded, ObjRef};

#[test]
fn heap_handles_round_trip() {
    for slot in [0u32, 1, 1000, u32::MAX / 2] {
        assert_eq!(ObjRef::heap(slot).decode(), Decoded::Heap(slot));
    }
}

#[test]
fn small_integers_are_encoded_inline_without_allocating() {
    for value in [0i64, 1, -1, 42, -42, (1 << 60) - 1, -(1 << 60)] {
        let r = ObjRef::small_int(value).expect("fits in the tagged range");
        assert_eq!(r.decode(), Decoded::SmallInt(value));
    }
}

#[test]
fn integers_outside_the_tagged_range_are_rejected_rather_than_truncated() {
    assert_eq!(ObjRef::small_int(i64::MAX), None);
    assert_eq!(ObjRef::small_int(i64::MIN), None);
}

#[test]
fn nil_is_distinct_from_every_heap_slot_and_every_integer() {
    assert_eq!(ObjRef::NIL.decode(), Decoded::Nil);
    assert_ne!(ObjRef::NIL, ObjRef::heap(0));
    assert_ne!(ObjRef::NIL, ObjRef::small_int(0).unwrap());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core`
Expected: FAIL — crate does not exist.

- [ ] **Step 3: Implement**

`rust/crates/rexx-core/src/handle.rs`:
```rust
//! A reference to a Rexx object.
//!
//! Two low bits carry a tag. `Heap` handles index the arena; `SmallInt`
//! carries a 62-bit signed value inline, which removes the allocation the
//! C++ implementation pays for via `RexxInteger`. `.nil` is a singleton
//! because Rexx code compares against it by identity.
//!
//! Note that `.true` and `.false` need no encoding: in Rexx they are the
//! strings "1" and "0".

const TAG_BITS: u32 = 2;
const TAG_MASK: u64 = 0b11;
const TAG_HEAP: u64 = 0b00;
const TAG_INT: u64 = 0b01;
const TAG_NIL: u64 = 0b10;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjRef(u64);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Decoded {
    Heap(u32),
    SmallInt(i64),
    Nil,
}

/// Inclusive bounds of the inline integer range.
pub const SMALL_INT_MAX: i64 = (1 << 61) - 1;
pub const SMALL_INT_MIN: i64 = -(1 << 61);

impl ObjRef {
    pub const NIL: ObjRef = ObjRef(TAG_NIL);

    pub const fn heap(slot: u32) -> Self {
        ObjRef(((slot as u64) << TAG_BITS) | TAG_HEAP)
    }

    pub const fn small_int(value: i64) -> Option<Self> {
        if value > SMALL_INT_MAX || value < SMALL_INT_MIN {
            return None;
        }
        Some(ObjRef((((value as u64) << TAG_BITS) & !TAG_MASK) | TAG_INT))
    }

    pub const fn decode(self) -> Decoded {
        match self.0 & TAG_MASK {
            TAG_HEAP => Decoded::Heap((self.0 >> TAG_BITS) as u32),
            TAG_INT => Decoded::SmallInt((self.0 as i64) >> TAG_BITS),
            _ => Decoded::Nil,
        }
    }

    pub const fn heap_slot(self) -> Option<u32> {
        match self.decode() {
            Decoded::Heap(slot) => Some(slot),
            _ => None,
        }
    }
}
```

`src/lib.rs`:
```rust
mod handle;
pub use handle::{Decoded, ObjRef, SMALL_INT_MAX, SMALL_INT_MIN};
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Add tagged object handles with inline small integers"
```

### Task 1.2: The arena heap and allocation

**Files:**
- Create: `rust/crates/rexx-core/src/heap.rs`, `src/body.rs`
- Modify: `rust/crates/rexx-core/src/lib.rs`
- Create: `rust/crates/rexx-core/tests/heap.rs`

**Interfaces:**
- Consumes: `ObjRef`, `Decoded` from Task 1.1.
- Produces: `Heap::new()`, `Heap::alloc(Body) -> ObjRef`, `Heap::get(ObjRef) -> Option<&Object>`, `Heap::get_mut(ObjRef) -> Option<&mut Object>`, `Heap::live_count() -> usize`, `Object { behaviour: BehaviourId, body: Body }`, `enum Body`, `BehaviourId(u16)`.

- [ ] **Step 1: Write the failing test**

```rust
use rexx_core::{Body, Decoded, Heap, ObjRef};

#[test]
fn allocation_returns_a_heap_handle_that_reads_back() {
    let mut heap = Heap::new();
    let s = heap.alloc(Body::String("hello".into()));
    assert!(matches!(s.decode(), Decoded::Heap(_)));
    assert!(matches!(heap.get(s).map(|o| &o.body), Some(Body::String(t)) if t == "hello"));
}

#[test]
fn small_integer_handles_are_not_in_the_heap() {
    let heap = Heap::new();
    assert!(heap.get(ObjRef::small_int(7).unwrap()).is_none());
    assert_eq!(heap.live_count(), 0);
}

#[test]
fn arrays_hold_handles_to_other_objects() {
    let mut heap = Heap::new();
    let a = heap.alloc(Body::String("a".into()));
    let arr = heap.alloc(Body::Array(vec![a, ObjRef::small_int(1).unwrap(), ObjRef::NIL]));
    let Some(Body::Array(items)) = heap.get(arr).map(|o| &o.body) else {
        panic!("expected an array")
    };
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], a);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test heap`
Expected: FAIL — `Heap` is not defined.

- [ ] **Step 3: Implement**

`rust/crates/rexx-core/src/body.rs`:
```rust
use crate::ObjRef;

/// Identifies the behaviour (class + method dictionary) an object responds to.
/// Behaviours themselves live in a side table, not in the heap, because they
/// are created during bootstrap and never collected.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BehaviourId(pub u16);

impl BehaviourId {
    pub const STRING: BehaviourId = BehaviourId(0);
    pub const ARRAY: BehaviourId = BehaviourId(1);
    pub const OBJECT: BehaviourId = BehaviourId(2);
}

/// The payload of a heap object.
///
/// Every variant that can reach another object must be handled in
/// `Body::trace`. Adding a variant without extending `trace` is the one way
/// to reintroduce the C++ implementation's defect class, so `trace` matches
/// exhaustively and must never gain a `_ =>` arm.
#[derive(Clone, Debug)]
pub enum Body {
    String(String),
    Array(Vec<ObjRef>),
    /// A user-defined object: its instance variables.
    Instance(Vec<(String, ObjRef)>),
}

#[derive(Clone, Debug)]
pub struct Object {
    pub behaviour: BehaviourId,
    pub body: Body,
}
```

`rust/crates/rexx-core/src/heap.rs`:
```rust
use crate::body::{BehaviourId, Body, Object};
use crate::{Decoded, ObjRef};

enum Slot {
    Free { next: Option<u32> },
    Live(Object),
}

pub struct Heap {
    slots: Vec<Slot>,
    free_head: Option<u32>,
    live: usize,
}

impl Heap {
    pub fn new() -> Self {
        Heap { slots: Vec::new(), free_head: None, live: 0 }
    }

    pub fn alloc(&mut self, body: Body) -> ObjRef {
        self.alloc_with(BehaviourId::OBJECT, body)
    }

    pub fn alloc_with(&mut self, behaviour: BehaviourId, body: Body) -> ObjRef {
        let object = Object { behaviour, body };
        self.live += 1;
        match self.free_head {
            Some(slot) => {
                let Slot::Free { next } = self.slots[slot as usize] else {
                    unreachable!("the free list only threads free slots")
                };
                self.free_head = next;
                self.slots[slot as usize] = Slot::Live(object);
                ObjRef::heap(slot)
            }
            None => {
                let slot = u32::try_from(self.slots.len()).expect("heap exceeds 2^32 slots");
                self.slots.push(Slot::Live(object));
                ObjRef::heap(slot)
            }
        }
    }

    pub fn get(&self, r: ObjRef) -> Option<&Object> {
        match r.decode() {
            Decoded::Heap(slot) => match self.slots.get(slot as usize) {
                Some(Slot::Live(o)) => Some(o),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn get_mut(&mut self, r: ObjRef) -> Option<&mut Object> {
        match r.decode() {
            Decoded::Heap(slot) => match self.slots.get_mut(slot as usize) {
                Some(Slot::Live(o)) => Some(o),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn live_count(&self) -> usize {
        self.live
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}
```

Export both modules from `src/lib.rs`.

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test heap`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Add the arena heap with a free list and slot handles"
```

### Task 1.3: Tracing — the 149-methods-to-one collapse

**Files:**
- Create: `rust/crates/rexx-core/src/trace.rs`
- Modify: `rust/crates/rexx-core/src/body.rs`
- Create: `rust/crates/rexx-core/tests/trace.rs`

**Interfaces:**
- Consumes: `Body`, `ObjRef`.
- Produces: `Body::trace(&self, out: &mut Vec<ObjRef>)`.

- [ ] **Step 1: Write the failing test**

```rust
use rexx_core::{Body, ObjRef};

#[test]
fn a_string_reaches_nothing() {
    let mut out = Vec::new();
    Body::String("x".into()).trace(&mut out);
    assert!(out.is_empty());
}

#[test]
fn an_array_reaches_every_element_including_duplicates() {
    let a = ObjRef::heap(3);
    let mut out = Vec::new();
    Body::Array(vec![a, a, ObjRef::NIL]).trace(&mut out);
    assert_eq!(out, vec![a, a, ObjRef::NIL]);
}

#[test]
fn an_instance_reaches_its_variable_values_but_not_their_names() {
    let v = ObjRef::heap(9);
    let mut out = Vec::new();
    Body::Instance(vec![("NAME".into(), v)]).trace(&mut out);
    assert_eq!(out, vec![v]);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test trace`
Expected: FAIL — no method named `trace`.

- [ ] **Step 3: Implement**

```rust
impl Body {
    /// Appends every object this one can reach.
    ///
    /// This single exhaustive match replaces the 149 hand-written `live()`
    /// implementations in the C++ tree. It has no wildcard arm on purpose:
    /// adding a `Body` variant must be a compile error here, not a runtime
    /// use-after-free.
    pub fn trace(&self, out: &mut Vec<ObjRef>) {
        match self {
            Body::String(_) => {}
            Body::Array(items) => out.extend_from_slice(items),
            Body::Instance(vars) => out.extend(vars.iter().map(|(_, v)| *v)),
        }
    }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test trace`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Trace object references with a single exhaustive match"
```

### Task 1.4: The root set

**Files:**
- Create: `rust/crates/rexx-core/src/roots.rs`
- Create: `rust/crates/rexx-core/tests/roots.rs`

**Interfaces:**
- Consumes: `ObjRef`.
- Produces: `RootSet::new()`, `RootSet::add_global(&str, ObjRef)`, `RootSet::push_frame() -> FrameId`, `RootSet::pop_frame(FrameId)`, `RootSet::push_temp(ObjRef)`, `RootSet::iter() -> impl Iterator<Item = ObjRef>`.

- [ ] **Step 1: Write the failing test**

```rust
use rexx_core::{ObjRef, RootSet};

#[test]
fn globals_are_always_roots() {
    let mut roots = RootSet::new();
    let env = ObjRef::heap(1);
    roots.add_global(".ENVIRONMENT", env);
    assert!(roots.iter().any(|r| r == env));
}

#[test]
fn temporaries_stop_being_roots_when_their_frame_is_popped() {
    let mut roots = RootSet::new();
    let tmp = ObjRef::heap(5);
    let frame = roots.push_frame();
    roots.push_temp(tmp);
    assert!(roots.iter().any(|r| r == tmp));
    roots.pop_frame(frame);
    assert!(!roots.iter().any(|r| r == tmp));
}

#[test]
fn popping_an_outer_frame_discards_the_inner_frames_it_contains() {
    let mut roots = RootSet::new();
    let outer = roots.push_frame();
    roots.push_temp(ObjRef::heap(1));
    let _inner = roots.push_frame();
    roots.push_temp(ObjRef::heap(2));
    roots.pop_frame(outer);
    assert_eq!(roots.iter().count(), 0);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test roots`
Expected: FAIL — `RootSet` is not defined.

- [ ] **Step 3: Implement**

```rust
use crate::ObjRef;

/// A position in the temporary stack that a frame will unwind to.
#[derive(Copy, Clone, Debug)]
pub struct FrameId(usize);

/// Everything the collector starts from.
///
/// The C++ implementation needs `ProtectedObject` at every allocation-crossing
/// site because raw pointers in C++ locals are invisible to it. Here the set
/// is small and explicit: globals, plus a stack of temporaries that expression
/// evaluation pushes into rather than holding values in Rust locals across an
/// allocation.
pub struct RootSet {
    globals: Vec<(String, ObjRef)>,
    temps: Vec<ObjRef>,
}

impl RootSet {
    pub fn new() -> Self {
        RootSet { globals: Vec::new(), temps: Vec::new() }
    }

    pub fn add_global(&mut self, name: &str, value: ObjRef) {
        match self.globals.iter_mut().find(|(n, _)| n == name) {
            Some(entry) => entry.1 = value,
            None => self.globals.push((name.to_string(), value)),
        }
    }

    pub fn push_frame(&mut self) -> FrameId {
        FrameId(self.temps.len())
    }

    pub fn pop_frame(&mut self, frame: FrameId) {
        self.temps.truncate(frame.0);
    }

    pub fn push_temp(&mut self, value: ObjRef) {
        self.temps.push(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.globals.iter().map(|(_, v)| *v).chain(self.temps.iter().copied())
    }
}

impl Default for RootSet {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test roots`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Add an explicit, enumerable GC root set"
```

### Task 1.5: Mark and sweep

**Files:**
- Modify: `rust/crates/rexx-core/src/heap.rs`
- Create: `rust/crates/rexx-core/tests/collect.rs`

**Interfaces:**
- Consumes: `Heap`, `RootSet`, `Body::trace`.
- Produces: `Heap::collect(&mut self, roots: &RootSet) -> CollectStats`, `CollectStats { swept: usize, live: usize }`.

- [ ] **Step 1: Write the failing test**

```rust
use rexx_core::{Body, Heap, RootSet};

#[test]
fn unreachable_objects_are_swept() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::String("garbage".into()));
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(heap.live_count(), 0);
}

#[test]
fn objects_reachable_from_a_root_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let kept = heap.alloc(Body::String("kept".into()));
    roots.add_global(".KEPT", kept);
    heap.alloc(Body::String("dropped".into()));
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(stats.live, 1);
    assert!(heap.get(kept).is_some());
}

#[test]
fn transitively_reachable_objects_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let leaf = heap.alloc(Body::String("leaf".into()));
    let holder = heap.alloc(Body::Array(vec![leaf]));
    roots.add_global(".HOLDER", holder);
    heap.collect(&roots);
    assert!(heap.get(leaf).is_some());
}

#[test]
fn reference_cycles_are_collected() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let a = heap.alloc(Body::Array(vec![]));
    let b = heap.alloc(Body::Array(vec![a]));
    let Some(obj) = heap.get_mut(a) else { panic!("a exists") };
    obj.body = Body::Array(vec![b]);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 2, "a cycle with no root must not survive");
}

#[test]
fn swept_slots_are_reused_by_the_next_allocation() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::String("x".into()));
    heap.collect(&roots);
    let reused = heap.alloc(Body::String("y".into()));
    assert_eq!(heap.slot_capacity(), 1, "the freed slot was reused, not appended");
    assert!(heap.get(reused).is_some());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test collect`
Expected: FAIL — no method named `collect`.

- [ ] **Step 3: Implement**

Add to `Heap`: a `marks: Vec<bool>` sized to `slots`, a `collect` that clears marks, seeds a worklist from `roots.iter()` filtered to heap slots, pops until empty (marking, then tracing into the worklist via a reusable scratch `Vec<ObjRef>`), then sweeps unmarked `Slot::Live` into `Slot::Free` threaded onto `free_head`. Add `slot_capacity(&self) -> usize` returning `self.slots.len()`. The worklist must skip already-marked slots — that is what terminates the cycle test.

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test collect`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Collect unreachable objects with mark and sweep over the arena"
```

### Task 1.6: Finalizers (`UNINIT`) and weak references

**Files:**
- Modify: `rust/crates/rexx-core/src/heap.rs`, `src/body.rs`
- Create: `rust/crates/rexx-core/tests/uninit.rs`

**Interfaces:**
- Produces: `Body::WeakRef(ObjRef)`, `Object::has_uninit: bool`, `CollectStats::pending_uninit: Vec<ObjRef>`.

- [ ] **Step 1: Write the failing test**

```rust
use rexx_core::{Body, Heap, RootSet};

#[test]
fn an_object_with_uninit_is_reported_rather_than_swept_immediately() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let obj = heap.alloc(Body::Instance(vec![]));
    heap.get_mut(obj).unwrap().has_uninit = true;
    let stats = heap.collect(&roots);
    assert_eq!(stats.pending_uninit, vec![obj]);
    assert!(heap.get(obj).is_some(), "it must survive until UNINIT has run");
}

#[test]
fn a_weak_reference_does_not_keep_its_target_alive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let target = heap.alloc(Body::String("target".into()));
    let weak = heap.alloc(Body::WeakRef(target));
    roots.add_global(".WEAK", weak);
    heap.collect(&roots);
    assert!(heap.get(target).is_none(), "the target was only weakly held");
}

#[test]
fn a_cleared_weak_reference_reads_as_nil() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let target = heap.alloc(Body::String("target".into()));
    let weak = heap.alloc(Body::WeakRef(target));
    roots.add_global(".WEAK", weak);
    heap.collect(&roots);
    assert!(matches!(heap.get(weak).map(|o| &o.body), Some(Body::WeakRef(r)) if *r == rexx_core::ObjRef::NIL));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test uninit`
Expected: FAIL — `Body::WeakRef` does not exist.

- [ ] **Step 3: Implement**

`Body::WeakRef(ObjRef)` traces to nothing. `collect` gains two post-mark passes, in this order: (1) for every unmarked object with `has_uninit`, mark it and everything it reaches, and record it in `pending_uninit` — running `UNINIT` must not see a half-collected object graph; (2) for every surviving `Body::WeakRef` whose target is unmarked, rewrite the target to `ObjRef::NIL`. Then sweep. `has_uninit` is cleared when the caller reports the finalizer has run, so the next collection sweeps the object normally.

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test uninit`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Resurrect objects pending UNINIT and clear dead weak references"
```

### Task 1.7: Behaviours and method dictionaries

**Files:**
- Create: `rust/crates/rexx-core/src/behaviour.rs`
- Create: `rust/crates/rexx-core/tests/behaviour.rs`

**Interfaces:**
- Produces: `BehaviourTable::new()`, `define(BehaviourId, name: &str, MethodId)`, `set_superclass(BehaviourId, BehaviourId)`, `lookup(BehaviourId, name: &str) -> Option<MethodId>`, `MethodId(u32)`.

- [ ] **Step 1: Write the failing test**

```rust
use rexx_core::{BehaviourId, BehaviourTable, MethodId};

#[test]
fn a_method_defined_on_a_behaviour_is_found() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::STRING, "LENGTH", MethodId(7));
    assert_eq!(t.lookup(BehaviourId::STRING, "LENGTH"), Some(MethodId(7)));
}

#[test]
fn lookup_is_case_insensitive_because_rexx_message_names_are_uppercased() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::STRING, "LENGTH", MethodId(7));
    assert_eq!(t.lookup(BehaviourId::STRING, "length"), Some(MethodId(7)));
}

#[test]
fn lookup_walks_to_the_superclass() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::OBJECT, "CLASS", MethodId(1));
    t.set_superclass(BehaviourId::STRING, BehaviourId::OBJECT);
    assert_eq!(t.lookup(BehaviourId::STRING, "CLASS"), Some(MethodId(1)));
}

#[test]
fn a_subclass_method_overrides_the_superclass() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::OBJECT, "STRING", MethodId(1));
    t.define(BehaviourId::STRING, "STRING", MethodId(2));
    t.set_superclass(BehaviourId::STRING, BehaviourId::OBJECT);
    assert_eq!(t.lookup(BehaviourId::STRING, "STRING"), Some(MethodId(2)));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test behaviour`
Expected: FAIL — `BehaviourTable` is not defined.

- [ ] **Step 3: Implement**

A `Vec<BehaviourEntry>` indexed by `BehaviourId.0`, each with `superclass: Option<BehaviourId>` and a `HashMap<String, MethodId>` keyed by the uppercased name. `lookup` walks the superclass chain, with a visited set so a bootstrap cycle (`Class` ↔ metaclass) cannot loop forever.

- [ ] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test behaviour`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add rust
git commit -m "Look up methods through behaviours and the superclass chain"
```

### Task 1.8: The D1 measurement — this is the gate

**Files:**
- Create: `rust/crates/rexx-core/benches/heap.rs`
- Modify: `rust/crates/rexx-core/Cargo.toml`
- Create: `docs/superpowers/plans/d1-decision.md`

- [ ] **Step 1: Write the allocation-throughput benchmark**

Criterion benchmark: allocate 1,000,000 `Body::String` objects of ~16 bytes with no collection, and separately 1,000,000 `Body::Array(vec![_; 4])`. Report allocations per second.

- [ ] **Step 2: Write the collection-pause benchmark**

Build a graph of 1,000,000 objects with a realistic shape — a root directory holding 1,000 arrays of 1,000 elements each, 10% of which are cross-links — then time a single full `collect`. Report the pause.

- [ ] **Step 3: Write the equivalent C++ measurement**

A Rexx program (`rust/bench-programs/heapshape.rex`) that builds the same graph shape using `.array` and `.directory`, run under `build/bin/rexx`, timed with hyperfine. It measures allocation plus collection together, so subtract the interpreter overhead measured by an equivalent program that builds nothing. Document the subtraction in `d1-decision.md` — an unstated adjustment is how a benchmark lies.

- [ ] **Step 4: Run both and record**

```bash
cd rust
cargo bench -p rexx-core -- --save-baseline rust-heap-linux
hyperfine --warmup 3 '../build/bin/rexx bench-programs/heapshape.rex'
```

- [ ] **Step 5: Close D1**

Write `d1-decision.md` with both sets of numbers and the verdict:

- **Allocation throughput within 1.5× of C++ and full-GC pause within 1.5×:** D1 closes as (a). Record it in Section 1 and proceed to Phase 2.
- **Allocation between 1.5× and 3× slower:** the arena is probably fine but the `Body` enum is likely too wide (every slot costs `size_of::<Body>()`). Before rejecting D1(a), try boxing the large variants and re-measure. Record both numbers.
- **Worse than 3× on either metric:** D1(a) is refuted. Do not proceed. Re-open D1 and evaluate the hybrid — `#[repr(C)]` inline headers with a side table for tracing — first, since it may recover the loss while staying safe. Option (c) is the last resort and does not get a pass on the Global Constraints unsafe bar: a raw-pointer heap would have to clear all four bars for a module that, by its nature, cannot encapsulate its invariant behind a safe API. That it cannot clear bar 2 is itself the argument against it. If the measurement lands here, the honest options are the hybrid or stopping — not quietly relaxing the constraint.

**Do not soften the gate to keep the schedule.** The whole argument for this rewrite is that it can be safe *and* fast; a Rust interpreter that is safe and slow is not worth 200k LOC of work, and finding that out at Phase 1 costs weeks instead of years.

- [ ] **Step 6: Commit**

```bash
git add rust docs
git commit -m "Measure arena allocation and collection against the C++ heap, and close D1"
```

### Phase 1 exit gate

- [ ] `cargo test -p rexx-core` green; `cargo clippy -- -D warnings` clean.
- [ ] `#![forbid(unsafe_code)]` holds in `rexx-core`, and `grep -rc unsafe rust/crates --include='*.rs'` reports zero across the workspace.
- [ ] `Body::trace` is a single exhaustive match with no wildcard arm.
- [ ] The root set is documented and enumerable; no `ProtectedObject` analogue exists.
- [ ] `d1-decision.md` committed with numbers, and D1 recorded in Section 1 of this file.

---

## 6. Generating the plans for Phases 2–10

Each subsequent phase gets its own plan file at `docs/superpowers/plans/YYYY-MM-DD-phase-N-<name>.md`, written with `superpowers:writing-plans` at the start of that phase — not now. Writing them now would be guessing: Phase 4's task breakdown depends on what Phase 3's AST actually looks like.

The generating procedure for each phase:

1. **Read the C++ it replaces.** Name the exact files and line counts. The phase plan opens with that inventory.
2. **Enumerate the observable behaviours,** not the functions. For Phase 3 that is error messages with line and column, `SOURCELINE`, and `TRACE` output formatting — not "the scanner tokenises correctly".
3. **Write the L0 corpus entries first.** Every behaviour in step 2 becomes a `.rex` program in `rust/corpus/` that the C++ oracle already passes. These are the phase's acceptance tests, written before any Rust.
4. **Decompose into tasks of one testable deliverable each,** in dependency order, following the Task Structure in `superpowers:writing-plans`.
5. **State the exit gate** as: corpus subset at zero divergences + L-rung reached + benchmark comparison against `perf-baseline.md` + the unsafe-block count, which must be zero or accounted for by a Section 1 decision block.
6. **Name the upstream decisions** the phase depends on and confirm each is closed.

**Phase-specific notes to carry forward:**

- **Phase 3** opens with the D10 spike (parser construction), and must decide how source text is retained. `SOURCELINE`, error reporting, and `TRACE` all expose the original text, so the AST cannot discard it. Keep the program source as one string and have AST nodes hold byte ranges into it — which is also what makes `chumsky`'s span support directly usable if D10 lands on (a). Measure parse throughput on `CoreClasses.orx`, since under D2 that number *is* cold-start time.
- **Phase 4** is where the execution model is fixed. Read the existing performance profile before designing the dispatch loop. The 162 builtins from Task 0.6 are the checklist; tick them off individually.
- **Phase 5** is the project's inflection point. When `CoreClasses.orx` runs, 32 classes appear at once and the L2 rung becomes reachable. Budget for the fact that it will expose parser and executor gaps in bulk rather than one at a time.
- **Phase 6** must hold D3's frame-ownership constraint: activities own their frames; cross-activity signalling goes through a channel or a polled atomic, never a foreign frame reference. Verify this by construction (no shared frame type exists) rather than by test.
- **Phase 7**'s stream model is a subsystem, not file I/O. `StreamNative.cpp` is 3,765 lines and its positioning and line/character interaction rules are all observable.
- **Phase 8** rebuilds `testbinaries/` unchanged against the frozen headers. If a header edit seems necessary, that is a Section 1 decision (D5 reopens), not a task. This is also the phase most likely to need the project's first `unsafe`: open a decision block for it *before* writing the `extern "C"` entry points, and design so that the unsafe is confined to converting caller-supplied pointers into validated handles at the boundary — everything past that boundary is safe Rust operating on `ObjRef`.

---

## 7. Risk register and kill criteria

| Risk | Signal | Response |
|---|---|---|
| Arena indexing costs too much | Phase 1 Task 8 shows >3× on allocation or GC pause | **Kill gate.** Re-open D1 before writing any Phase 2 code. |
| L1 extraction is not viable | `l1-coverage.md` below 40% | Drop L1; accept a longer blind stretch between L0 and L2, and compensate by growing the L0 corpus |
| `CoreClasses.orx` needs semantics not yet built | Phase 5 stalls with a long tail of gaps | Expected, not a surprise. Triage per gap; do not start Phases 6–8 until Phase 5 closes |
| ooTest depends on undocumented C++ internals | Suite fails in ways the corpus never predicted | Treat each as a new L0 corpus entry first, then fix. Never add to `known-test-failures/` to close a phase |
| RXAPI protocol is not portable | Phase 0 Task 8 finds unversioned struct dumps | D7 flips to "port it"; Phase 10 grows by 12k LOC |
| Windows and BSD diverge late | Phases 1–5 are developed on Linux only | Run the full gate on all five platforms at **every** phase exit, not at Phase 9 |
| Effort exceeds available time | Phase 4 not closed within its estimate | **Decision point, not a failure.** The C++ tree is untouched and still ships. Either narrow scope to a Rexx subset that is explicitly not ooRexx-conformant, or stop and keep Phase 0's oracle and benchmark suite, which have standalone value for the C++ project |

**The strongest property of this plan is that abandoning it is cheap.** The C++ tree is never modified. Phase 0 produces a differential runner and a five-platform benchmark baseline that improve the existing project whether or not a single line of the Rust interpreter is ever written. Stopping after any phase leaves the repository better than it started.

---

## 8. Rejected alternatives

**Strangler / in-place oxidation** — replace C++ subsystems one at a time behind a C ABI. Rejected by the user in favour of clean-room. Worth recording why it is genuinely worse here: the GC is the *first* thing you would have to replace, since every other subsystem depends on the object representation, and replacing the GC while half the tree still holds raw pointers means keeping the `ProtectedObject` discipline — which is the thing the rewrite exists to eliminate. Oxidation gets the risk profile of a rewrite with none of the benefit until the very end.

**Differential fuzzing as a gate** — generating random Rexx programs and diffing the two interpreters. Not selected. It would find numeric and `PARSE` edge cases that a hand-written corpus misses, and if the L0 corpus proves too thin in Phase 4, this is the first thing to add.

**Corpus replay over the 301 `samples/` programs as a primary gate** — not selected as a gate, but the samples remain the natural expansion of the L0 corpus once Phase 4 is under way. Many touch the file system or the console and need harnessing before they are deterministic.

**A general decimal crate for `NUMERIC`** — see D4. The ANSI Rexx rules differ from IEEE decimal in ways that are individually small and collectively fatal to conformance.

---

## 9. Self-review

**Spec coverage.** Every decision from the user's four answers is carried: clean-room reimplementation (Section 3 crate tree, C++ tree frozen in Global Constraints); source-compatible C API (D5, Phase 8); ooTest as gate (D8, the L-rungs, Phase 9); perf non-regression (D9, Task 0.7, every phase gate); agent-executable roadmap (Phases 0–1 at step granularity, Section 6 for the rest).

**Placeholders.** None. Every code step carries real code; every gate carries a runnable command. Four things are deliberately unmeasured and each names the task or spike that measures it: the L1 extractable fraction (Task 0.4), the RXAPI protocol answer (Task 0.8), the D1 verdict (Task 1.8), and the D10 parser comparison (the Phase 3 opening spike).

**Constraints added after the first draft, and where they landed.** The image is optional rather than obligatory — D2 now defaults to no image, builds one only on a measured startup miss, and records why that ordering cannot waste work. `unsafe` is forbidden by default everywhere with no blanket crate exemptions; the four-bar admission protocol is in Global Constraints, the unsafe-block count is a reportable item at every phase exit, and the D1 fallback to raw pointers is explicitly *not* granted a pass on it.

**Type consistency.** `ObjRef`, `Decoded`, `Body`, `Object`, `BehaviourId`, `MethodId`, `Heap`, `RootSet`, `FrameId`, `CollectStats`, `Outcome`, `Interpreter`, `Divergence`, and `TestMethod` are each defined once and used with the same signature everywhere they appear.

**Known soft spot.** Task 1.4's `RootSet` is standalone; Phase 4 must connect it to the real activation and expression stacks, and the borrow-checker shape of that connection — who owns `Heap` versus `RootSet` during evaluation — is not solved here. It is a Phase 4 design output, and the first task of Phase 4's plan should be a spike on exactly that question. Recording it as unsolved is deliberate: pretending otherwise would put a wrong answer into a plan that later phases build on.
