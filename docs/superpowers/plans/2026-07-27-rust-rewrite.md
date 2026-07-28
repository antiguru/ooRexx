# ooRexx → Rust: Clean-Room Reimplementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This plan is written to be re-entered cold.** Phases 0 and 1 are specified to bite-sized task granularity and are executable as written. Phases 2–10 are specified as *gates* — entry criteria, exit criteria, and the procedure that generates their own plan. Section 1 (Decisions) is the part that rewards deep reasoning; do not begin a phase whose upstream decision is still open.

**Goal:** Replace the ooRexx interpreter (~198k LOC C++) with a clean-room Rust implementation that passes the ooTest conformance suite on all five CI platforms at or above the C++ build's performance, keeping `api/oorexxapi.h` source-compatible so existing native extensions recompile without being rewritten.

**Explicitly out of scope:** `extensions/platform/` — ooDialog and the OLE/ActiveX support, 115,835 LOC of Windows-only GUI code, 90% of everything under `extensions/`. It is a consumer of the native API, not part of the interpreter, and rewriting it is a separate project with a separate justification. The source-compatibility contract (D5) is what should let it recompile against the Rust interpreter unchanged; **whether it actually does is untested by this plan and must not be claimed.** The in-scope extensions are `rxregexp`, `rxmath`, `rxsock`, `hostemu`, `orxncurses`, and the Rexx-source packages (`json`, `yaml`, `csvStream`, `dateparser`, `rxftp`), all in Phase 10.

**Architecture:** A new Rust workspace under `rust/` alongside the untouched C++ tree, which serves as the executable oracle for the entire project. The Rust interpreter uses an **arena heap with tagged index handles** rather than raw pointers; this collapses the C++ implementation's 148 hand-written `live()` trace methods and its pervasive `ProtectedObject` root-pinning discipline into a single derived `Trace` impl and a ~5-entry root set. Everything else — the tree-walking execution model, the expression stack, the activity/guard concurrency semantics, the Rexx-source class library — is preserved deliberately, because it is observable behaviour.

**Tech Stack:** Rust 1.96+ (2024 edition). `unsafe` is forbidden by default in every crate and admitted only per-site, encapsulated and justified in writing (see Global Constraints). `rustix`/`windows-sys` for platform calls. `criterion` for benchmarks. `quick-xml` for build-time message-catalogue generation. `chumsky` is a candidate above the token stream, pending the D10 spike. CMake stays for the C++ oracle build only. The existing SVN-hosted ooTest suite is the conformance gate; the existing `.github/known-test-failures/*.txt` baselines are the pass criteria.

---

## Global Constraints

Every task's requirements implicitly include this section.

- **Rust floor:** 1.96.1 (the toolchain present on this machine). Edition 2024. No nightly features.
- **Unsafe: forbidden by default in every crate, including `rexx-api` and `rexx-sys`.** There is no blanket exemption.
  - **The mechanics, which are not interchangeable.** A crate with no unsafe at all carries `#![forbid(unsafe_code)]` at the root. A crate that has been granted an unsafe module carries `#![deny(unsafe_code)]` at the root and `#[allow(unsafe_code)]` on that one module. **`forbid` cannot be relaxed by an inner `allow`** — it is a hard error, `E0453: allow(unsafe_code) incompatible with previous forbid`, verified by compiling it. So the choice of `forbid` versus `deny` at the root *is* the record of whether a crate has been granted an exception, and downgrading a root from `forbid` to `deny` is exactly the visible, reviewable event that bar 4 below is about.
  - Granting a module an exception requires all four of:
  1. **Unavoidable.** A safe alternative was attempted and is recorded as failing — not merely judged unlikely. "`rustix` has no wrapper for this call" is unavoidable; "raw `libc` is faster" is not, absent a committed benchmark showing it. Prefer `rustix` and `windows-sys` over raw `libc` *specifically because* they move the unsafe behind an audited boundary.
  2. **Encapsulated as far as the boundary allows** — and the two kinds of boundary differ, so state which applies:
     - **Rust-facing unsafe** (anything reachable only from Rust, i.e. all of `rexx-sys`): full encapsulation, no exceptions. Callers outside the module cannot reach an unsound state by any sequence of calls. If a Rust caller must uphold an invariant, the API is wrong — fix the API rather than documenting the obligation.
     - **FFI-facing unsafe** (the `extern "C"` entry points in `rexx-api`): full encapsulation is *impossible* and demanding it would be incoherent — a C caller passing a valid pointer is a precondition no Rust API can enforce. The requirement here is instead: validate everything that *can* be validated (handles go through the local-reference table, never a raw dereference), keep the unvalidatable surface as small as possible, enumerate it explicitly in the module doc, and make the first statement of every entry point the validation, so the unsafe region is one line long rather than the whole function.
  3. **Justified in writing.** The module carries a `//!`-level block stating: what the invariant is, why the compiler cannot check it, what enforces it instead, and what breaks if it is violated. Every `unsafe` block carries a `SAFETY:` comment naming the specific precondition it discharges. The crate carries `#![deny(unsafe_op_in_unsafe_fn)]`.
  4. **Reviewed as a decision, not a task.** Introducing a new unsafe module is a Section 1 decision block with an identifier, not something a task does in passing. It goes into this file before it goes into the code.
  - Expect exactly two candidates over the whole project: the `extern "C"` entry points in `rexx-api`, and any platform call in `rexx-sys` that `rustix`/`windows-sys` do not cover. Both are *candidates*, not exemptions. Under D5 the FFI surface is far smaller than it looks: `RexxObjectPtr` is an opaque handle validated by table lookup, so the entry points dereference almost nothing.
  - **Every phase exit reports the unsafe-block count** (`grep -rc 'unsafe' rust/crates --include='*.rs'`) **and the list of crate roots carrying `deny` rather than `forbid`**. Either growing without a corresponding decision block in Section 1 fails the gate.
- **The C++ tree is read-only.** No file under `interpreter/`, `api/`, `common/`, `rexxapi/`, `extensions/` is modified by this project. It is the oracle. The only exception is `.github/workflows/` (adding Rust legs) and new files under `rust/` and `docs/`.
- **`api/oorexxapi.h`, `api/rexx.h`, `api/rexxapidefs.h`, `api/oorexxerrors.h` are frozen.** Source compatibility is the contract: native extensions must recompile unchanged. ABI compatibility is explicitly *not* required — struct layouts and symbol addresses may change, but declarations, macro names, type names, and call semantics may not.
- **Platform matrix:** Linux (ubuntu-24.04), macOS 15 arm64, Windows/MSVC, FreeBSD 14.2, OpenBSD 7.8. Every phase gate runs on all five. The known OpenBSD SIGSEGV in the current C++ baseline is pre-existing; it does not block Rust work but must not be *reproduced* by the Rust build.
- **Conformance oracle:** `svn checkout https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest` (409 `.testGroup` files, 14,122 `::method test*`, 20 MB), run as **`rexx testOORexx.rex -s < /dev/null`**, judged by `.github/check-test-results.ps1` against `.github/known-test-failures/common.txt` plus the per-platform file.
  - **The `< /dev/null` is not optional.** Several groups — `ADDRESS.testGroup`, and the `CHARIN`/`CHAROUT` BIF groups — start child processes that read stdin. Without an stdin of its own the suite silently hangs there rather than failing, and in an ssh-driven VM it takes the session down with it. This repository already fixed it once for the BSD legs (commit `5ea8bc6c`); it applies equally to any local run, and this project hung on it before noticing. The Rust interpreter is judged by the *same* baselines. Adding an entry to a known-failure file is a plan-level decision, never a task-level one.
- **Performance gate.** Two thresholds, deliberately different, and it matters which applies where:
  - **Shipping gate (parity).** No phase from 2 onward closes with a Rust subsystem slower than its C++ counterpart on the Phase 0 benchmark suite, measured on Linux and macOS. "Slower" means the criterion point estimate falls outside the C++ baseline's confidence interval on the slow side. This is the rule everywhere unless a phase says otherwise.
  - **Phase 1 viability threshold (1.5×).** Task 1.8 judges D1 against a *looser* bar, because at that point there is no interpreter — only a heap benchmarked through a Rust API against a C++ heap benchmarked through an interpreted Rexx program. That comparison is directional, not like-for-like, and a tight bar on a rough measurement would reject a sound design on noise. Phase 1 exits at 1.5×; the parity gate then applies from Phase 2 on, once there is a real interpreter to measure. A Phase 1 result between parity and 1.5× is a recorded debt, not a pass — name it in `d1-decision.md` and re-measure at Phase 4 when dispatch exists.
- **Licence:** every new file carries the CPL v1.0 header block used throughout the tree (copy the block verbatim from any existing `.cpp`, adjusting the year).
- **Commits:** every task ends with a commit. Branch is `plan/rust-rewrite` or a descendant.

---

## 0. Measured inventory — what you are actually replacing

All numbers measured on `8c880bdd` (`ci/platforms`). Re-measure if the base moves.

| Area | LOC | Notes |
|---|---:|---|
| `interpreter/classes/` | 54,271 | 40 primitive classes. `NumberStringClass.cpp` 4,231 + `NumberStringMath*.cpp` |
| `interpreter/instructions/` | 19,152 | 59 `.cpp` (112 files with headers): 35 keyword instructions + directives + DO-loop variants |
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

Blocks are numbered in the order they were raised and ordered below by topic, so the numbering is not sequential. Index, in document order:

| | Decision | Blocks | State |
|---|---|---|---|
| **D1** | Heap representation and GC strategy | everything | **closed** — arena + handles; pause 1.45×, a recorded debt (2026-07-27) |
| **D2** | The saved image | Phase 5 | default set (no image); threshold measured at Phase 5 |
| **D3** | Concurrency model | Phase 6 | recommendation set; constrains Phase 1 |
| **D4** | Numeric core | Phase 2 | settled — port `NumberString` |
| **D5** | Native API surface | Phase 8 | settled by the user — source-compatible |
| **D6** | Platform layer | Phase 7 | settled — `std` → `rustix` → `libc` |
| **D13** | AST ownership | Phase 3 | **closed** — plain owned Rust data (2026-07-27) |
| **D14** | String representation | Phase 4, constrains Phase 3 | **closed** — byte strings, UTF-8 arrives as operations (2026-07-28) |
| **D11** | RexxUtil / `Sys*` | Phase 7, and L2 | settled — subset in Phase 7, rest in Phase 10 |
| **D12** | Security manager | Phases 5 and 7 | settled — split across both |
| **D7** | RXAPI daemon | Phase 10 | **closed** — bridge to the C++ rxapi (2026-07-27) |
| **D8** | Conformance ladder | everything | **closed** — L1 viable at 86.2% (2026-07-27) |
| **D9** | Performance gate | every phase exit | settled — two thresholds, see Global Constraints |
| **D10** | Parser construction | Phase 3 | open — spike at the head of Phase 3 |

### D1 — Heap representation and GC strategy ⟵ *the load-bearing decision*

**Blocks:** Phase 1 (and therefore everything).

**Question.** How are Rexx objects represented and collected?

The C++ implementation uses a segmented mark-sweep heap with an old-space/new-space split, a 2-bit mark in a 16-bit `ObjectHeader` flags word (`interpreter/classes/ObjectClass.hpp:95–171`), and `UninitPending`/`HasUninit` bits driving finalizers. Object references are raw `RexxInternalObject*`. Because raw pointers in C++ locals are invisible to the collector, correctness depends on **148 hand-written `live(size_t)` implementations** plus a `ProtectedObject` RAII root-pinning discipline applied at every allocation-crossing site. Getting one wrong is a use-after-free, not a compile error.

**Options.**

- **(a) Arena + tagged index handles. RECOMMENDED.** All heap objects live in `Vec<Slot>` inside a `Heap`. A reference is `ObjRef(u64)` — a slot index with a low-bit tag that also encodes small integers inline. Tracing is one `match` over a `Body` enum (derivable), not 148 methods. Root set becomes small and *enumerable*: the activation stack, the expression stack, the C-API local-reference tables, and the global tables (`.environment`, `.local`, class registry).
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

The C++ build runs `rexximage` to flatten a bootstrapped heap into `rexx.img`, with **105 `flatten(Envelope*)` implementations**, proxy objects for un-flattenable state, and virtual-function-table repatching on restore (`RexxMemory.cpp:691`, `:1539`). The VFT repatching exists solely because C++ objects embed vtable pointers that are invalid in another process image. **Rust has no vtables to repatch** — under D1(a), heap objects are plain data in a `Vec` and references are indices, so an image is a serialization of a flat array with no pointer fixups at all.

**Options.**

**What dropping the image does *not* buy.** Object flattening is not only the image's mechanism — it is also how compiled programs are serialized. `RoutineClass::save`/`restore` (`interpreter/classes/RoutineClass.cpp:143`, `:291–311`, `:389–426`) runs through `Envelope` plus `ProgramMetaData`, and that is exactly what `rexxc` does. Phase 9 ships `rexxc`, so **program flattening survives regardless of D2**. The saving from choosing (a) is the image build step, the proxy mechanism, and the VFT repatching — not the flattening machinery wholesale. Treat the `.rxo`/`ProgramMetaData` format as its own Phase 9 sub-problem with its own compatibility question: whether Rust-produced compiled files must be readable by the C++ interpreter (probably not, since both ship together) and whether C++-produced ones must be readable by Rust (also probably not, but say so deliberately rather than discovering it).

- **(a) No image. DEFAULT — build this first, unconditionally.** Parse and execute `CoreClasses.orx` + `StreamClasses.orx` (5,203 lines of Rexx) at every startup. Nothing corresponding to the image build, the proxy mechanism, or the VFT repatching ever gets written; program serialization is built separately for `rexxc`.
- **(b) Serialize the arena, *if* (a) measures too slow.** Under D1(a) this is close to a `memcpy` of `Vec<Slot>` plus a string table — no pointer fixups, no per-class serialization code, no proxies. It costs a format version, a staleness check against the `.orx` sources, and a build step. Purely additive: it caches what (a) computes, so semantics cannot diverge between the two paths.

**Evidence that settles this.** Phase 5 exit measures cold start for (a) with hyperfine against `build/bin/rexx`. **Ship (a) either way.**

**The threshold is an absolute delta, not a ratio: build (b) only if (a) costs more than ~50 ms of wall clock over the C++ startup.** Ratio is a diagnostic, not the gate. An earlier draft said "2× or 50 ms", which fires on a 5 ms → 10 ms result that no user could perceive — and the "or" made the weaker condition win. Perception is absolute; report the ratio alongside, and treat a large ratio at a small delta as interesting rather than actionable.

**The C++ number is now measured, and it makes this concrete.** `rexx bench-programs/startup.rex` — a one-line `say` — costs **median 5.1 ms** (min 3.3, mean 5.1, max 7.7; 50 runs after 10 warmups). That is what memory-mapping a prebuilt image buys.

So the target for (a) is: **parse and execute 5,203 lines of `CoreClasses.orx` + `StreamClasses.orx` in under ~55 ms**, since 5.1 + 50 is the gate. That is roughly 100k lines/second of parse-and-bootstrap, sustained, on every start. Comfortably achievable for a competent parser and not remotely automatic — which is exactly why D10 measures parser throughput on `CoreClasses.orx` specifically rather than on synthetic input, and why a pleasant-but-slow parser is the thing most likely to force the image cache into existence.

Note also that the ratio here will look terrible whatever happens — 5 ms against 50 ms is 10× — and that is precisely why the ratio is not the gate.

**Note what the comparison is, so nobody games it.** The C++ side memory-maps a prebuilt image; the Rust side parses and executes 5,203 lines of Rexx. The comparison is *deliberately unfavourable to Rust*, and that is the point: the question is not "can Rust-without-image beat C++-with-image" — it usually cannot and need not — but "is Rust-without-image fast enough in absolute terms that a user does not notice". Framing the gate as a ratio invites building (b) to win a benchmark rather than to serve anyone.

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

**The part that matters for D1.** `RexxObjectPtr` becomes an **opaque handle registered in the calling activation's local-reference table**, not a heap address. This is not a new idea imposed by Rust — the C++ already does exactly this via `NativeActivation::createLocalReference` / `removeLocalReference` / `clearLocalReferences` (`NativeActivation.hpp:177–179`), because native code holds references across GC points. Under D1(a) the same mechanism becomes the *only* mechanism, and a handle that outlives its activation is a lookup miss rather than a use-after-free.

**That claim depends on the generation field in `ObjRef` (Task 1.1), and is false without it.** Slots are recycled through a free list, so a bare slot index held across a collection would silently name whatever is allocated into that slot next — memory-safe, but returning the wrong object, which is the same defect class in different clothing and landing at precisely the boundary this decision advertises as the win. The generation is what converts "stale" into "miss". Do not treat it as an optimisation to add later.

**Evidence that settles this.** Phase 8 exit: `testbinaries/` build against the frozen headers with no source edits, and the ooTest native-API groups pass.

### D6 — Platform layer

**Blocks:** Phase 7.

**Decision: `std` first, then `rustix` + `windows-sys`, and raw `libc` only where neither reaches. Low uncertainty.**

The ordering is a safety decision, not a taste one: `rustix` wraps the syscalls this layer needs behind a safe API, so choosing it over raw `libc` discharges the Global Constraints unsafe bar by construction rather than by argument. A raw `libc` call in this crate needs its own justification block naming the `rustix` function that does not exist.

The 15,293 LOC of `interpreter/platform/` is mostly things `std` covers. The genuine gaps, which need care because they are observable: the Rexx **stream model** (line vs binary access, `RESET`, explicit positioning, the `CHARIN`/`LINEIN` interaction, `StreamNative.cpp` is 3,765 lines), **`ADDRESS` command routing** to shells and subcom handlers, file-name and path semantics, and console/terminal behaviour. Treat the stream model as a subsystem in its own right, not as "file I/O".

### D13 — AST ownership: heap objects or plain Rust data

**Blocks:** Phase 3, and constrains D10. **Coupled to D1 and to `rexxc` (see D2).**

**The fact the rest of the plan missed.** `RexxInstruction` is not a plain node — it is `class RexxInstruction : public RexxInternalObject` (`interpreter/instructions/RexxInstruction.hpp:63`) with `live(size_t)`, `liveGeneral(MarkReason)`, and `flatten(Envelope*)` at `:74–76`. **In the C++ implementation the AST is garbage-collected and serializable**, and a large share of the 148 `live()` and 105 `flatten()` implementations are instruction and expression nodes rather than data classes. Every earlier section of this plan silently assumed the AST was ordinary data. It is not, and until this is settled Phase 3 cannot start.

**Question.** In Rust, are AST nodes arena objects, or plain owned data?

**Options.**

- **(a) Plain owned Rust data, held inside one arena object per code body. RECOMMENDED.** `Body::Code { instructions: Vec<Instruction>, … }`, where `Instruction` and `Expr` are ordinary Rust enums with `Box`/`Vec` children. The parser allocates nothing in the heap and needs no root discipline at all; the executor walks contiguous owned data.
- **(b) Every node is its own arena object,** mirroring C++. Uniform, and matches the oracle's structure exactly, but puts the parser inside the GC's root discipline and scatters the hot execution path across the arena.

**Why (a), and the one thing that makes it non-trivial.** Instructions embed *literal Rexx objects* — string and numeric constants, and resolved variable references. Those are real heap objects, so under (a) an `Instruction` still holds `ObjRef` fields and **the code body still has to be traced**. What (a) buys is not "no tracing" but "one `trace` impl that walks a `Vec<Instruction>`" instead of a per-node method — the same collapse D1 buys everywhere else, applied here too. It also keeps D10's argument alive: a combinator parser can produce ordinary Rust enums, which it cannot comfortably do if every node must be allocated through a heap handle.

For `rexxc` (which needs program flattening regardless of D2), owned Rust data with a derived serializer is easier than 105 hand-written `flatten` methods, not harder.

**CLOSED — take (a).** Settled 2026-07-27 by interrogating the running interpreter rather than by sampling the test suite, which is both faster and a stronger argument. An earlier draft proposed grepping `ootest/` for messages returning instruction-level objects. That is the wrong instrument: a test can only observe what the language exposes, so enumerating the *exposed surface* settles it for every possible test, present and future, while a grep settles it only for the tests that exist today.

The complete instance-method surface of the code-bearing classes, dumped from ooRexx 5.3.0 via `~methods`:

- **`Package`** (38 methods) — `SOURCE` returns an `Array` of `String`; `SOURCELINE` returns a `String`; `SOURCESIZE` an integer; `CLASSES`/`ROUTINES`/`DEFINEDMETHODS`/`PUBLICROUTINES`/… return collections of `Class`, `Routine`, and `Method` objects. `PROLOG` returns a **`Routine`**, which was the one plausible leak and is not one.
- **`Method`** (17) — `SOURCE` returns source *text*. Nothing else reaches code structure.
- **`Routine`** (8) — `SOURCE`, `CALL`, `CALLWITH`, `[]`. Same.
- **`StackFrame`** (11) — `LINE` is an integer, `TRACELINE` a formatted `String`, `EXECUTABLE` a `Method`/`Routine`, `CONTEXT` a `RexxContext`.
- **`RexxContext`** (16) — `EXECUTABLE`, `PACKAGE`, `LINE`, `VARIABLES`, `STACKFRAMES`, … all coarser than an instruction.

And the native API is no different: `api/oorexxapi.h` declares `RexxMethodObject`, `RexxRoutineObject`, and `RexxPackageObject`, and **no type naming an instruction, clause, expression, or code node at all** — so D5's source-compatibility contract does not constrain this either.

**Nothing in the language or the C API exposes an object below `Method`/`Routine`/`Package` granularity, and source is exposed as text, never as structure.** The AST is therefore a private implementation detail, and Rust is free to represent it as plain owned data. As a bonus, `Package~source` returning an `Array` of `String` confirms the source-retention approach Phase 3 already planned: keep the program text and hand out slices of it.

**Cost of being wrong.** Would have been high — it is the representation the parser produces and the executor consumes, so Phases 3 and 4 both rest on it. Settled for the cost of two probe programs.

### D14 — String representation, and keeping the UTF-8 door open

**Blocks:** nothing yet. **Constrains** the object model Phase 4 builds, and Phase 3's `ProgramSource`.

**Why this exists.** Moritz intends to move the language to proper UTF-8 handling soon after the rewrite works: character length and byte length as distinct functions, with checked conversion. This decision does **not** do that. It records what the implementation must avoid so that doing it later is a change of operations rather than a rewrite of the object model.

**The constraint that shapes every option.** A Rexx string is an arbitrary byte sequence and must stay one. `'FFFE'x` is legal today, and so are `c2x`, `x2c`, `bitand` and binary stream I/O. Measured against the oracle: a source file containing a raw `FF FE` inside a literal runs, `c2x` returns `FFFE`, `length` returns 2. So a single UTF-8-*validated* string type cannot represent legal Rexx values. That is exactly why Python 3 needed a separate `bytes` type, not a preference for having two.

**Consequence, and it is the useful part.** The single-type design Moritz wants is achievable, but it runs through the *operations* rather than the type: the value stays a byte string, and character semantics arrive as new operations plus explicit checked decode. Both length functions can then coexist without `'FF'x` becoming unrepresentable, and no second type is ever introduced.

**There is no conversion to design, and one probe settles why.** `x2c('C3A4') == 'ä'` is **1**. A source literal is not "text" that gets converted to bytes; it already *is* the bytes, identical to what `x2c` returns. So a scheme where `x2c` yields a byte string and `bitand` converts a generic string to one presupposes a text/bytes distinction the language does not have. Stream I/O is byte-transparent for the same reason: `FF FE C3A4 00 41` round-trips through `charout`/`charin` unchanged, embedded NUL included, at `length` 6.

**So the real question is not when to convert but what character operations do on invalid UTF-8**, and it is a total-function question: raise, replace with U+FFFD, or fall back to one-character-per-byte. Left open; it is the substance of the later change and does not constrain anything today.

**Superseded on one point, by a deeper analysis done separately.** `docs/superpowers/specs/2026-07-28-unicode-design-space.md` (in the C++ tree) surveys the design space properly, and it corrects the framing above: **"character" is already four distinct concepts inside the existing BIF set** — graphemes (`SUBSTR`, `POS`), display columns (`CENTRE`, `LEFT`), codepoints plus folding (`UPPER`, `COMPARE`), and bytes (`C2X`, `BITAND`, `CHARIN`). So "add a character-length operation" is under-specified before it is even implemented, and any design that adds one new character *type* gets three of the four wrong. The conclusion it reaches is the same shape as this decision but sharper: in Rexx **the distinction belongs on the verb, not the noun**, because Rexx has an option-letter tradition (`DATATYPE(s,'A')`) that Python lacked and was therefore forced to encode in the type.

Two further findings from it that bear on the rules below. utf8proc is already compiled unconditionally into `librexx.so` and is **10.5 % of the shipped library** while being used in exactly one place, so "Unicode tables would bloat the interpreter" is not an available objection. And `FlagSet<StringFlag,32>` at `StringClass.hpp:816` uses 5 of its 32 bits, so the encoding/validity tag this decision asks room for costs zero object growth in the C++ and should cost none here either.

Read that note before reopening this decision. Nothing in it changes what Phase 3 does.

**The rule must be fixed, never ambient.** Python 2's failure was not that conversion was implicit but that the encoding was *ambient* — locale-dependent, so identical programs failed on one machine and not another. Python 3 did not remove that, it relocated it to the I/O boundary, where `open()` still consults the locale. Byte strings plus byte-transparent I/O remove the failure mode outright, because reading a file involves no decode decision. Whatever the character operations later do, the encoding they assume is a constant of the language and validity is a determinable fact about a value, never a property of the environment.

**Where the later fork sits, left open deliberately.** Whether `LENGTH` keeps byte semantics and a new BIF returns characters, or `LENGTH` becomes characters and a new BIF returns bytes, is a compatibility judgement rather than a representation one. The first breaks nothing and reads oddly; the second is the "proper" answer and changes the result of existing programs. Nothing below forecloses either.

**Rules the implementation must follow, all cheap today:**

- **A Rexx string value is a byte string** (`Box<[u8]>` or equivalent), never a Rust `String`. Rust `String` enforces UTF-8, which would reject legal values outright.
- **Every index and length in the interpreter is a byte offset.** Byte offsets stay correct under both later models; character offsets are derived on demand. Phase 3 already does this for spans.
- **Leave room for a lazily computed encoding tag on the string object** — at minimum "known valid UTF-8 / known invalid / not yet checked". This is the single thing that makes the later switch cheap, because checked conversion becomes O(1) after the first check instead of a rescan per operation. Do not compute it eagerly; most strings never need it.
- **Do not put `&str` in any value-carrying signature.** `rexx-num`'s `compare` currently takes `&str` and its `string_order` helper is already byte-based underneath, so this is a signature change and not a rewrite. Recorded as Phase 2 debt (M5).
- **Never name an internal accessor `length` ambiguously.** Byte length and character length must be distinguishable at every call site from the start, so that changing which one `LENGTH` maps to is a one-line change.

**Audited 2026-07-28, and the tree is clean on all of this.** `rexx-core` has no Rexx string type yet, so the decisive representation choice is still unmade. In `rexx-num` the only `&str` on a value path is `compare`/`parse`, and `Number::parse` is sound as-is because a byte sequence that is not valid UTF-8 can never be a valid Rexx number and therefore maps to error 41 anyway. The 13 character-oriented call sites in the tree are all in message rendering over generated ASCII, not on value paths.

**Not adopting `utf8proc`.** The interpreter vendors it for exactly one purpose: decoding the offending byte sequence so error 13.1 can print a whole character rather than one byte (`Scanner.cpp:49`, used once). Phase 3 does not reproduce parse-error text, so no equivalent is needed. Symbols cannot contain non-ASCII at all — `LanguageParser::characterTable` is zero for every byte `0x80`–`0xFF`, and `bäc = 2` is error 13.1 — so nothing in the scanner needs Unicode awareness either.

**Cost of being wrong.** Low today, high if deferred: it is the representation every string BIF consumes. Settled by an audit rather than by a spike, because no code commits to the alternative yet.

### D11 — RexxUtil / `Sys*` functions

**Blocks:** Phase 7, and through it **L2** — which makes this more urgent than its size suggests.

`interpreter/runtime/RexxUtilCommon.cpp` (2,207) plus `interpreter/platform/unix/SysRexxUtil.cpp` (1,631) and `interpreter/platform/windows/SysRexxUtil.cpp` (3,325) implement the `Sys*` library: `SysFileTree`, `SysTempFileName`, `SysSleep`, `SysFileDelete`, `SysDumpVariables`, and the rest.

**Why it is on the critical path — verified against the suite, not assumed.** Checked out from SVN and grepped:

| Call site | Needs |
|---|---|
| `worker.rex:318`, `:364` | `SysFileExists` — **this is the CI path**: `testOORexx.rex` is a 99-line kicker that sets `PATH` and calls `worker.rex`, which is where the real work and the `::requires "ooTest.frm"` live |
| `worker.rex:861` | `.File` (`~new`, `~absolutePath`) |
| `framework/runTestUnits.rex:45`, `:129–133` | `SysFileTree`, used to *discover test files* — the framework's other runner |
| `framework/WinUtils.cls:123` | `SysSleep` |

So the suite cannot start — not "runs with some failures", cannot start — without `SysFileExists` and `.File`, and the framework's own runner additionally needs `SysFileTree`. **`Sys*` blocks L2.** A plan that schedules it as a Phase 10 nicety cannot reach its own Phase 5 gate.

`SysFileExists` and `SysFileTree` are implemented in `interpreter/runtime/RexxUtilCommon.cpp` with platform halves in `interpreter/platform/{unix,windows}/SysRexxUtil.cpp`.

**The full suite is now checked out, and the surface is far wider than the framework's three functions.** 409 `.testGroup` files, 14,122 `::method test*` methods, 20 MB. Grepping all of it for `Sys*` yields **99 distinct identifiers**, of which roughly half are real routines — the remainder are documentation placeholders (`SysFileXXX`, `SysXxx`), deliberately-absent names (`SysDoesNotExist`), and false positives from the pattern (`System`, `SystemRoot`). Call-site counts for the busiest:

| | | | |
|---|---|---|---|
| `SysFileTree` 100 | `SysFromUnicode` 57 | `SysFileExists` 57 | `SysFileDelete` 45 |
| `SysIni` 43 | `SysToUnicode` 41 | `SysSleep` 41 | `SysSearchPath` 41 |
| `SysTextScreenSize` 40 | `SysStemSort` 38 | `SysFileSearch` 35 | `SysIsFileDirectory` 32 |

So this is a real subsystem's worth of work, not a handful of shims. It also spans several distinct capability groups that will not all land together: file system, Unicode conversion, terminal (`SysTextScreen*`), POSIX identity (`SysGetpwnam`, `SysGetgrgid`, `SysGeteuid`, …), extended attributes (`SysGetXattr` and friends), Windows INI and printers, and stem utilities.

**One group crosses into D7.** The macrospace functions — `SysAddRexxMacro`, `SysDropRexxMacro`, `SysQueryRexxMacro`, `SysReorderRexxMacro`, `SysClearRexxMacroSpace`, `SysLoadRexxMacroSpace`, `SysSaveRexxMacroSpace` — are served by the **RXAPI daemon**, not by the interpreter. They therefore cannot work until D7 is executed, which puts their test groups squarely on the Phase 9 exclusion list. That the exclusion list was needed at all (see the Phase 9 gate) is confirmed here rather than assumed.

**Decision: build the `Sys*` subset ooTest depends on as part of Phase 7, and the remainder in Phase 10.** Phase 7's plan opens by turning the grep above into a prioritised worklist ordered by call-site count, so the functions blocking the most test groups land first. `.File` is resolved: it **is** an environment class (`.file~id` returns `File` under ooRexx 5.3.0), so Phase 7 owns it alongside the file-system `Sys*` calls.

**Note the platform asymmetry:** the Windows `SysRexxUtil.cpp` is twice the size of the unix one, so this is also where the Windows leg is most likely to fall behind.

### D12 — Security manager

**Blocks:** Phases 5 and 7, in that order.

`interpreter/execution/SecurityManager.{cpp,hpp}` intercepts command issuance, stream access, external function calls, and `.local`/`.environment` lookups when a security manager object is installed. It is observable and reachable from the public API, and **ooTest covers it directly: `base/security.manager/SecurityManager.testGroup`** (confirmed present in the SVN tree).

**Decision: split it across the two phases that own its interception points, rather than pretending it is one unit.**

- **Phase 5** builds the security manager object, its installation path, and the hooks that live in dispatch and name resolution — `.local`/`.environment` lookup and external function resolution.
- **Phase 7** adds the hooks in command issuance (`ADDRESS`) and stream access, because those call sites do not exist until Phase 7 builds them.

Assigning the whole thing to Phase 5, as an earlier draft of this plan did, is wrong on its face: Phase 5 cannot intercept a command handler that Phase 7 has not written. What matters is that the *interception design* is fixed in Phase 5 so Phase 7 adds call sites to an existing mechanism rather than inventing a second one. Retrofitting the mechanism after Phases 6–8 means touching every one of those paths twice.

### D7 — RXAPI daemon

**Blocks:** Phase 10 (and partially Phase 7 — external queues).

**Question.** Port the 11,932-LOC daemon, or speak its protocol?

**Decision: keep the C++ `rxapi` binary and speak its IPC protocol from Rust. RECOMMENDED.** It is a separate process behind a stable wire boundary — exactly the kind of thing that should not be on the critical path. The first working Rust `rexx` links no C++ but talks to a C++ `rxapi`.

**CLOSED — bridge confirmed.** Settled 2026-07-27; full analysis in `docs/superpowers/plans/rxapi-protocol.md`.

The feared answer was half-right and turned out not to matter. It **is** a raw struct dump — `ServiceMessage::writeMessage` does `pipe.write((void *)this, sizeof(ServiceMessage), messageData, messageDataLength, …)` (`rexxapi/common/ServiceMessage.cpp:141–152`), with the pointer field on the wire being garbage that every receiver ignores. What makes it safe anyway:

- **Layout is stable.** `sizeof(ServiceMessage)` is **600** and `sizeof(ServiceRegistrationData)` is **544**, independently confirmed here by compiling a probe against the real headers. All scalars sit at natural alignment with no interior padding, and there is no `long`, so LP64 and LLP64 agree.
- **Skew fails cleanly rather than corrupting.** The rendezvous name embeds the version triple, the pointer width, and the username — `snprintf(path, len, "%s/.ooRexx-%d.%d.%d-%s-%s", …, ORX_VER, ORX_REL, ORX_MOD, "64"|"32", name)` (`SysCSStream.cpp:522–528`) — so a 32-bit build, a different release, or another user never finds the same socket. Transport is strictly host-local, so endianness cannot differ.
- **There is a version handshake.** The server answers `CONNECTION_ACTIVE` with `parameter1 = REXXAPI_VERSION` (=100, `ServiceMessage.hpp:199`; set at `APIServer.cpp:243`) and the client throws a version-conflict `API_FAILURE` on mismatch (`LocalAPIManager.cpp:227`, `:258`). Client-side only — the server validates nothing — but that is the direction that matters here.
- **Transport.** Unix: `AF_UNIX SOCK_STREAM` at `$XDG_RUNTIME_DIR/.ooRexx-5.3.0-64-<user>.service`. Windows: a local named pipe with `PIPE_REJECT_REMOTE_CLIENTS`. A TCP path exists in the source but nothing instantiates it.

**What the Rust client must get right:** replicate both struct layouts with static assertions and exact enum discriminants; generate the rendezvous name byte-for-byte against the *target* rxapi's version — **the worst failure mode is a wrong name silently spawning a second, empty daemon rather than erroring**; perform the `CONNECTION_ACTIVE` handshake; and send `CLOSE_CONNECTION`/`PROCESS_CLEANUP` as the C++ client does. Accept that the bridge pins one rxapi release series, and re-validate the size probe on every version bump.

Roughly 1–2k lines of Rust against 12k LOC of porting. The bridge wins clearly.

### D8 — Conformance ladder

**Blocks:** everything. **Settle in Phase 0.**

**The problem.** `testOORexx.rex` and the `.testGroup` files are themselves ooRexx programs — they use `::class`/`::method`/`::requires`, the `TestGroup` and `ooTestCase` classes, streams, and packages (confirmed by inspection of `extensions/json/json_02.testGroup`). The suite cannot run until the interpreter is nearly complete. "ooTest green" is therefore a *final* gate, useless as an incremental signal. A ladder is required.

- **L0 — Differential runner.** A Rust harness runs a `.rex` file under both `build/bin/rexx` (C++) and `rexx-rs`, and diffs normalised stdout/stderr/exit code. Corpus: hand-written micro-programs per feature, growing to the 301 in-repo samples.
- **L1 — Extracted assertions. VIABLE, measured 2026-07-27.** Mechanically lift `::method test*` bodies out of the `.testGroup` files and emit standalone micro-programs against a tiny assert shim. **409 groups, 14,122 test methods, 12,176 extractable = 86.2%**, or **82.7%** after correcting for the `expose` blind spot below. Either figure is far clear of the 40% threshold, so **the ladder is L0 → L1 → L2 → L3** and `rexx-extract` stays. Full per-file table in `docs/superpowers/plans/l1-coverage.md`.

  Three limits of the extractor, known and quantified rather than discovered later:

  1. **The `expose` blind spot.** `touches_fixture` catches `self~<message>` but not the other fixture idiom — `setUp` stores state in an exposed instance variable and the test body says `expose <var>` then uses `<var>` directly, with no `self~` anywhere. Those are wrongly marked extractable and would fail to parse, since `expose` is not legal inside `::routine`. Measured: 491 of 12,176 extracted files (4.0%) contain `expose`. That is the 86.2% → 82.7% correction.
  2. **No block-comment awareness.** `extract()` does not track `/* … */`, so a commented-out `::method test…` is counted as live. Observed in the sibling `Assert.testUnit`, where 20 of 48 methods were dead code inside a comment. Inflates the numerator on files that carry commented-out tests.
  3. **`ASSERTIONS` omits `expectCondition`,** which is the same kind of assertion as `expectSyntax` and appears in real test code. Add it.

  Two real defects the full run hit, both fixed in the binary: `C2X.testGroup` is ISO-8859 with a literal `0xAA` byte, so `read_to_string` aborted the whole run — read bytes and use `from_utf8_lossy`, which is safe because `extract()` only matches ASCII markers. And `Assignments.testGroup` defines `::method "test_/="` and `"test_//="` to test the `/=` and `//=` operators, so building `<group>_<method>.rex` from the raw name tried to write through a `/` — sanitise the method-name component with `[^A-Za-z0-9_-]` → `_`.

  Note for anyone re-measuring: **do not count test methods with a naive `grep '::method test'`.** ooTest quotes method names heavily, and the unquoted-only count is 6,380 — less than half the real 14,122. This plan carried that undercount for several commits.
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

1. **There are no reserved words.** `IF` is a keyword only in keyword position; `if = 5; say if` is a valid program. The rule is visible in `InstructionParser.cpp:179–196`: if the first token of a clause `isSymbol()` and the second `isSubtype(OPERATOR_EQUAL)`, it is an assignment — no keyword check happens first. A lexer emitting `Keyword(If)` is therefore *wrong*; keyword-ness is decided by the parser from position, and the same characters must stay usable as a variable name. Expressible in `chumsky`, but it means matching on identifier text at each site rather than on token variants, which gives up much of the ergonomic win.
2. **Clause splitting precedes parsing.** Clauses end at `;`, at end-of-line, or not at all if the line ends in a continuation comma. This is a pre-pass over the source, and the C++ structures it that way for good reason.
3. **Whitespace is semantic, and the worst case is `f(x)` versus `f (x)`.** Abuttal is the concatenation operator, so a blank between two terms is an operator. **Verified against ooRexx 5.3.0** — with a routine `F` in scope and a variable `f = "VAR"`:

   ```rexx
   say f(1)    /* ROUTINE-CALLED-WITH-1  -- a function call        */
   say f (1)   /* VAR 1                  -- variable, then abuttal */
   ```

   Same tokens, one space, entirely different program. The C++ threads this through the scanner as an explicit parameter: `locateToken(character, blanksSignificant)` returning `SIGNIFICANT_BLANK` (`Scanner.cpp:271`, `:296–299`). **This is the specific hazard of a combinator library**, most of which skip whitespace by default and would parse both spellings identically — producing a working parser that is silently wrong on a construct real Rexx code uses. `rust/corpus/lang/whitespace_significant.rex` pins the behaviour.

4. **The literal-suffix rule bites harder than it looks, because it usually does not fail loudly.** `'ff'x` and `'1010'b` are literals whose suffix binds to the *preceding* quote. The visible consequence is that `say a"|"b` dies with error 15.4 — `"|"b` is an invalid binary string. The dangerous consequence is that `say a''b`, which reads as the classic Rexx idiom for blank-free concatenation, **prints `x` rather than `xy`**: `''b` is an *empty binary literal*, so the line concatenates `a` with `""` and never reads `b` at all. No error, just a different program. Both were hit for real while building this project's seed corpus, the second one inside a comment that confidently described the wrong behaviour.
5. **The rest of tokenisation is idiosyncratic too.** `/* */` comments nest (`Scanner.cpp:200–250`, tracked with an explicit nesting level); `--` runs to end of line; `.` is a symbol constituent, so `a.b.c` is one compound-variable token rather than three tokens and two operators, while a *leading* `.` means environment lookup (`.array`, `.nil`).
4. **Error output is fixed by the oracle.** Conformance demands one specific error, with a specific number out of the 704, a specific sub-number, a specific **line**, and the specific substitution values the message quotes. There is no column: `POSITION` on the condition object is the line, and stderr carries no offset, because ooRexx locates an error by quoting the offending token instead. `chumsky`'s recovery and multi-error reporting — a large part of its value — is mostly unusable here, because emitting a second, better diagnostic is a conformance failure.
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
| 3 | Scanner & parser | D1 closed, D13 closed ✓, D10 spiked | Round-trips every `.rex` under `samples/` to an AST (301 files); `SOURCELINE` and `TRACE`'s `*-*` source lines match the oracle byte-for-byte; parse errors give the oracle's **number and sub-number on a plausible line**, with message text and substitutions deliberately not reproduced (2026-07-28 scope decision); parse throughput on `CoreClasses.orx` recorded | L0 (syntax errors) |
| 4 | Classic executor | 2, 3 | Non-OO Rexx runs: assignment, `DO` (all variants), `IF`, `SELECT`, `CALL`, `PARSE`, `SAY`, `SIGNAL`, conditions, and all **81 builtin functions** | L0 full corpus + L1 majority |
| 5 | Object model | 4 | **`CoreClasses.orx` parses and executes**; 32 classes exist and respond; `::class`/`::method`/`::routine`/`::requires` work; security manager interception points in place (D12); cold start measured and recorded against C++ (D2) | L2 |
| 6 | Concurrency | 5 | Activities, kernel lock, guard locks, `REPLY`, `GUARD`, message objects; ooTest concurrency groups pass; TSan (or `loom`) clean. **D3's frame-ownership constraint verified.** | L2 |
| 7 | Streams & platform | 5 | `StreamClasses.orx` runs; stream model, `ADDRESS`, file system green on all 5 platforms; the `Sys*` subset ooTest needs (D11) works | L2 |
| 8 | Native API | 5, 7 | `testbinaries/` compile unchanged against frozen headers; native-API ooTest groups pass | L2 |
| 9 | Core conformance | 6, 7, 8 | **L3-core on all 5 platforms**: the full suite green against existing baselines *excluding* the groups enumerated below; every benchmark at parity; `rexx`, `rexxc`, `rxqueue`, `rxsubcom` ship | L3-core |
| 10 | RXAPI & extensions | 9 | `rxregexp`, `rxmath`, `rxsock`, `hostemu`, `orxncurses` recompile and pass; RXAPI decision (D7) executed; **L3-full** with no exclusions | L3-full |

Phases 6, 7, and 8 are independent of each other and may run in parallel once Phase 5 closes.

**Why Phase 9's gate is L3-*core*, not L3.** The suite exercises things Phase 10 delivers — the extension test groups (`json`, `yaml`, `rxregexp`, and the rest), and the RXAPI-dependent features: external data queues, macrospace, and `rxsubcom` registration. This is confirmed rather than assumed: the checked-out suite calls the seven `Sys*RexxMacro*`/`Sys*RexxMacroSpace` routines, which the RXAPI daemon serves (see D11). Gating Phase 9 on the unqualified full suite would make it unevaluable until Phase 10 was already done. Phase 9's plan must therefore **enumerate the excluded groups explicitly, by name, in a committed file** (`docs/superpowers/plans/phase-9-exclusions.txt`), and Phase 10 deletes that file. An exclusion list that is not written down is indistinguishable from a suite that quietly does not run.

**D8 kept L1** — measured at 86.2%, so the Phase 2 and Phase 4 rows stand as written.

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
      build.rs                #   rexxmsg.xml -> $OUT_DIR/errors.rs (704 messages)
      src/lib.rs              #   include!()s the generated files; no generated
                              #   file is ever written into src/
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
      src/ffi.rs              #   the only module carrying #[allow(unsafe_code)],
                              #   under a crate root of deny (not forbid), and only
                              #   after a Section 1 decision block says so
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

- [x] **Step 1: Build the C++ oracle**

```bash
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build build --parallel "$(getconf _NPROCESSORS_ONLN)"
```

- [x] **Step 2: Verify the oracle runs**

Run:
```bash
build/bin/rexx -v
echo 'say .rexxinfo~version' > /tmp/hello.rex && build/bin/rexx /tmp/hello.rex
```
Expected: a version banner, then a version string. If this fails, stop — nothing downstream is meaningful without a working oracle.

- [x] **Step 3: Create the workspace**

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
channel = "stable"
components = ["rustfmt", "clippy"]
```

**Pin the channel, not the version number.** `channel = "1.96.1"` makes rustup insist on a toolchain installed under that exact name and try to install it if absent, which fails outright wherever `~/.rustup` is not writable — including sandboxed and CI environments that pre-provision a toolchain. The version floor belongs in `rust-version = "1.96.1"` in the workspace manifest, which cargo checks against whatever toolchain is actually in use and refuses to build under an older one. That is the constraint Global Constraints asks for, and it is enforced where it works.

`rust/.gitignore`:
```
target/
```

- [x] **Step 4: Create the oracle crate**

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

- [x] **Step 5: Verify it compiles**

Run: `cd rust && cargo build`
Expected: success, no warnings.

- [x] **Step 6: Commit**

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

- [x] **Step 1: Write the failing test**

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

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-oracle`
Expected: FAIL — `normalize` is not defined.

- [x] **Step 3: Implement normalisation**

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

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-oracle`
Expected: 2 passed.

- [x] **Step 5: Commit**

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

- [x] **Step 1: Write the seed corpus**

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

- [x] **Step 2: Write the CLI**

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

    // Canonicalise before use. Each program runs with its own directory as
    // the child's cwd, and a relative binary path would then be resolved
    // against *that* directory rather than ours -- so `--cpp ../build/bin/rexx`
    // fails with a bare NotFound once the child cwd moves.
    let absolute = |p: PathBuf, what: &str| match std::fs::canonicalize(&p) {
        Ok(abs) => abs,
        Err(e) => {
            eprintln!("cannot resolve {what} {}: {e}", p.display());
            std::process::exit(2);
        }
    };
    let cpp = absolute(cpp, "--cpp");
    let rs = absolute(rs, "--rs");
    let corpus = absolute(corpus, "--corpus");

    let lib = |bin: &PathBuf| {
        bin.parent()
            .map(|d| vec![d.to_path_buf(), d.join("../lib")])
            .unwrap_or_default()
    };
    let reference = Interpreter { library_paths: lib(&cpp), binary: cpp };
    let candidate = Interpreter { library_paths: lib(&rs), binary: rs };

    let mut programs: Vec<PathBuf> = walk(&corpus);
    programs.sort();
    // A mistyped or empty corpus directory would otherwise report
    // "0 programs, 0 divergences" and exit 0 -- the phase's central
    // self-test passing by finding nothing.
    if programs.is_empty() {
        eprintln!("no .rex programs under {}", corpus.display());
        return ExitCode::from(2);
    }
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
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // Do not swallow this: an unreadable directory read as "empty" is how
        // a self-test reports success for work it never did.
        Err(e) => panic!("cannot read {}: {e}", dir.display()),
    };
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

- [x] **Step 3: Verify the differ finds zero divergences running C++ against itself**

Run:
```bash
cd rust && cargo build --release
./target/release/rexx-diff \
  --cpp ../build/bin/rexx --rs ../build/bin/rexx --corpus corpus
```
Expected: `12 programs, 0 divergences`, exit 0.

**This is the phase's central self-test.** A non-zero count here means the corpus is non-deterministic or normalisation is wrong — fix the corpus, never loosen normalisation to make it pass.

- [x] **Step 4: Commit**

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

- [x] **Step 1: Write the failing test**

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

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-extract`
Expected: FAIL — crate does not exist.

- [x] **Step 3: Implement the extractor**

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
    "expectsyntax", "expectcondition", "assertlistequals", "assertarrayequals",
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

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-extract`
Expected: 2 passed.

- [x] **Step 5: Write the `rexx-extract` binary**

`rust/crates/rexx-extract/src/bin/rexx-extract.rs` takes three flags, parsed the same way as `rexx-diff` in Task 0.3:

- `--suite <dir>` — the checked-out `ootest/` tree; walked recursively for `*.testGroup`.
- `--out <dir>` — where standalone micro-programs are written; use `rust/corpus-l1`, a **sibling** of `rust/corpus/` and never a child, because `rexx-diff` walks its corpus argument recursively and would otherwise sweep all 12,059 L1 programs into the 13-program L0 self-test, one `.rex` per extractable method, named `<group>_<method>.rex`.
- `--report <file>` — a Markdown table written with one row per `.testGroup`: file, total `::method test*` count, extractable count, percentage; then a total line.

Each emitted program wraps the method body with a minimal assert shim so it stands alone:

```rexx
/* extracted from <group>::<method> */
::routine main public
  <body>
::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
```

The shim must define exactly the assertion messages listed in `ASSERTIONS`; a method using one that the shim lacks is not extractable, so extend `touches_fixture` to treat an unknown `self~` message as fixture-dependent — which it already does, since anything not in `ASSERTIONS` returns true.

Exit non-zero if the suite directory holds no `.testGroup` files, for the same reason `rexx-diff` refuses an empty corpus.

- [x] **Step 6: Measure L1 viability against the real suite**

Run:
```bash
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
svn checkout --non-interactive --trust-server-cert \
  https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest
cd rust && cargo run --release -p rexx-extract --bin rexx-extract -- \
  --suite ../ootest --out ../rust/corpus-l1 --report ../docs/superpowers/plans/l1-coverage.md
```

- [x] **Step 7: Record the D8 decision**

Read `l1-coverage.md`. **Measured 2026-07-27: 86.2% (82.7% corrected). L1 is viable; D8 is closed as L0→L1→L2→L3.** The rule was: if the fraction is ≥40%, keep L1. Below 40%, the extraction machinery costs more than it returns; record D8 as L0→L2→L3, delete `rexx-extract`, and change the Phase 2 and Phase 4 roadmap rows from L1 to L0. Write the decision and the measured number into Section 1's D8 block in this file.

- [x] **Step 8: Commit**

```bash
git add rust docs
git commit -m "Extract standalone assertions from ooTest groups and measure L1 coverage"
```

### Task 0.5: Generate the error-code table from `rexxmsg.xml`

**Files:**
- Create: `rust/crates/rexx-inventory/Cargo.toml`, `build.rs`, `src/lib.rs`
- Create: `rust/crates/rexx-inventory/tests/errors.rs`

**Interfaces:**
- Produces: `rexx_inventory::errors::MESSAGES: &[Message]`, `Message { major: u16, sub: u16, number: u16, symbol: &'static str, text: &'static str }`, `errors::lookup(major: u16, sub: u16) -> Option<&'static Message>`.

**The catalogue's shape, which decides the key.** `rexxmsg.xml` holds **56 `<Message>` majors and 648 `<SubMessage>` children**, 704 total. Identity is the pair `(Code, Subcode)`; `Code` alone repeats across every submessage of a major. There is also a separate `<MessageNumber>` that is neither the code nor contiguous — error 3.001 carries `MessageNumber` 200. A flat `&[(u32, &str)]` has no unique key and must not be used. Key on `(major, sub)`.

**Message text is markup, and the rendering rules are fixed by the oracle's own generator** — `interpreter/messages/RexxErrorMessages.xsl`, whose output is the checked-in `RexxErrorMessages.h`. Do not invent a rendering; copy this one:

| Markup | Renders as | XSL |
|---|---|---|
| `<Sub position="N"/>` | `&N` | `:98–100` |
| `<q>X</q>` | `"X"` — **literal double quotes, kept** | `:86–88` |
| `<sq/>` | `'` | `:90–92` |
| `<dq/>` | `"` | `:94–96` |

`<q>` is emphatically **not** a documentation-only wrapper to be dropped. The generated header proves it (`RexxErrorMessages.h:62`):

```
MESSAGE(Error_Program_unreadable_name, "Failure during initialization: File \"&1\" is unreadable.")
```

There are **363 `<q>` occurrences** — nearly every message that names an operand — and 36 of them wrap literal text with no substitution at all (`Unmatched <q>/*</q> or quote.` → `Unmatched "/*" or quote.`). Dropping the wrapper would diverge from the oracle on those even where no substitution exists, and L0 would catch it as 363 separate failures.

Keep the substitution marker as `&N` rather than translating to `%N`. The table is private to the Rust side and either would work, but matching the oracle byte-for-byte removes a transformation that could silently disagree, and makes the generated table directly diffable against `RexxErrorMessages.h`.

- [x] **Step 1: Write the failing test**

`rust/crates/rexx-inventory/tests/errors.rs`:
```rust
use rexx_inventory::errors;

#[test]
fn every_message_from_the_catalogue_is_present() {
    // 56 <Message> + 648 <SubMessage> = 704, as of 8c880bdd. If this changes,
    // the C++ tree gained or lost an error and the Rust side must follow.
    assert_eq!(errors::MESSAGES.len(), 704);
    assert_eq!(errors::MESSAGES.iter().filter(|m| m.sub == 0).count(), 56);
}

#[test]
fn a_major_carries_its_own_text_with_no_substitutions() {
    let m = errors::lookup(3, 0).expect("error 3 exists");
    assert_eq!(m.text, "Failure during initialization.");
    assert_eq!(m.symbol, "Error_Program_unreadable");
}

#[test]
fn a_submessage_is_keyed_by_the_pair_and_renders_markup_like_the_oracle() {
    let m = errors::lookup(3, 1).expect("error 3.001 exists");
    assert_eq!(m.number, 200, "MessageNumber is independent of the code");
    // <q> keeps its quotes; <Sub position="1"/> becomes &1.
    // Compare against RexxErrorMessages.h:62.
    assert_eq!(m.text, "Failure during initialization: File \"&1\" is unreadable.");
}

#[test]
fn q_markup_around_literal_text_still_renders_its_quotes() {
    // 36 messages wrap literal text in <q> with no substitution at all.
    // Dropping the wrapper would diverge from the oracle on every one.
    let m = errors::lookup(6, 0).expect("the unmatched-quote error exists");
    assert_eq!(m.text, "Unmatched \"/*\" or quote.");
}

#[test]
fn error_13_is_invalid_character_in_program() {
    let m = errors::lookup(13, 0).expect("error 13 exists");
    assert!(m.text.to_ascii_lowercase().contains("invalid character"));
}
```

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-inventory`
Expected: FAIL — crate does not exist.

- [x] **Step 3: Implement the build script**

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

`rust/crates/rexx-inventory/build.rs` reads `../../../interpreter/messages/rexxmsg.xml`, walks each `<Message>` and its nested `<Subcodes>/<SubMessage>` children, and writes `$OUT_DIR/errors.rs` containing:
```rust
pub struct Message {
    pub major: u16,
    pub sub: u16,
    pub number: u16,
    pub symbol: &'static str,
    pub text: &'static str,
}

pub static MESSAGES: &[Message] = &[
    Message { major: 3, sub: 0, number: 3, symbol: "Error_Program_unreadable",
              text: "Failure during initialization." },
    Message { major: 3, sub: 1, number: 200, symbol: "Error_Program_unreadable_name",
              text: "Failure during initialization: File \"&1\" is unreadable." },
    // ... one entry per message, majors and their submessages in document order
];

pub fn lookup(major: u16, sub: u16) -> Option<&'static Message> {
    MESSAGES.iter().find(|m| m.major == major && m.sub == sub)
}
```

Text rendering, applied in this order: replace `<q>X</q>` with `"X"`, `<sq/>` with `'`, `<dq/>` with `"`; replace `<Sub position="N" …/>` with `&N`; unescape XML entities last. The ordering is safe — only four texts contain entities (`&gt;`, `&lt;`, `&apos;`), none of which can form markup when unescaped, and there are no nested `<q>`. A major with no `<Text>` of its own is an error in the catalogue, not something to paper over with an empty string — panic.

Cross-check the output against the checked-in `interpreter/messages/RexxErrorMessages.h`, which the oracle generates from the same XML through `RexxErrorMessages.xsl`. If the Rust table and that header disagree on any text, the Rust renderer is wrong. **Encode this as a test** (`tests/oracle_agreement.rs`), not a one-off script: it validates every markup rule across all 704 messages at once, which is the only check that would catch a `<q>` regression. Note that the header carries 705 `MESSAGE(...)` lines — the extra one is a `Table_end` sentinel with empty text, and must be skipped. The only escape the header uses is `\"`.

It must `println!("cargo::rerun-if-changed=../../../interpreter/messages/rexxmsg.xml");` and `panic!` if the file is missing, if the total is zero, or if any `(major, sub)` pair repeats. A silently empty or colliding table would let every later phase report false conformance.

`rust/crates/rexx-inventory/src/lib.rs`:
```rust
//! Tables mechanically derived from the C++ tree. Never hand-edit these; the
//! C++ tree is the source of truth and the build script re-derives them.
pub mod errors {
    include!(concat!(env!("OUT_DIR"), "/errors.rs"));
}
```

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-inventory`
Expected: 5 passed. If the count assertion fails with a number other than 704, the base commit moved — update the constant and note it.

- [x] **Step 5: Commit**

```bash
git add rust
git commit -m "Generate the Rexx error-message table from rexxmsg.xml at build time"
```

### Task 0.6: Builtin-function inventory

**Files:**
- Modify: `rust/crates/rexx-inventory/build.rs` (emit `$OUT_DIR/builtins.rs`), `src/lib.rs` (add `pub mod builtins { include!(...) }`)
- Create: `rust/crates/rexx-inventory/tests/builtins.rs`

**Interfaces:**
- Produces: `rexx_inventory::builtins::NAMES: &[&str]` — the 81 builtin function names **in table order**, which is the index order the parser uses.

- [x] **Step 1: Write the failing test**

```rust
#[test]
fn the_builtin_table_has_81_entries() {
    // The table at BuiltinFunctions.cpp:3042 holds 81 entries plus a leading
    // NULL dummy, which the extractor skips.
    assert_eq!(rexx_inventory::builtins::NAMES.len(), 81);
}

#[test]
fn table_order_is_preserved_because_the_parser_indexes_by_position() {
    // NOT alphabetical: the table is mostly sorted but has an appended tail
    // (…X2D, XRANGE, USERID, LOWER, UPPER, RXFUNCADD, RXFUNCDROP,
    // RXFUNCQUERY, ENDLOCAL, SETLOCAL, QUALIFY, GC). Sorting it would break
    // the index the parser resolves builtins through, so the test pins the
    // ends rather than asserting an ordering.
    assert_eq!(rexx_inventory::builtins::NAMES[0], "ABBREV");
    assert_eq!(rexx_inventory::builtins::NAMES[80], "GC");
}
```

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-inventory builtins`
Expected: FAIL — module does not exist.

- [x] **Step 3: Extend the build script**

Parse `../../../interpreter/expression/BuiltinFunctions.cpp` from the line matching `pbuiltin LanguageParser::builtinTable[] =` to the closing `};`, taking each `&builtin_function_NAME` and emitting `NAME` **in source order**. Skip the leading `NULL` dummy entry. Panic if fewer than 50 names are found — a threshold that catches a broken parse without tripping on the real count of 81.

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-inventory builtins`
Expected: 2 passed. This list is Phase 4's definition of done.

- [x] **Step 5: Commit**

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

- [x] **Step 1: Write the benchmark programs**

One per D9 dimension, each sized to run 0.5–2s under the C++ interpreter:
`dispatch.rex` (tight method-send loop), `varlookup.rex` (simple variable read/write), `compound.rex` (stem and compound-variable access — **the −24% memo prototype's workload**), `strings.rex` (`SUBSTR`/`POS`/`CHANGESTR`/concatenation), `arith.rex` (decimal arithmetic across several `NUMERIC DIGITS`), `alloc.rex` (allocation churn to force collections), `startup.rex` (`say 1`, for cold-start timing).

- [x] **Step 2: Write the criterion harness**

`benches/interpreter.rs` takes the interpreter path from `REXX_BENCH_BINARY`, runs each program via `rexx_oracle::Interpreter::run`, and reports wall time per program. Assert nothing — this task only establishes the baseline.

- [x] **Step 3: Record the C++ baseline on Linux**

Run:
```bash
cd rust
REXX_BENCH_BINARY="$PWD/../build/bin/rexx" \
    cargo bench --offline -p rexx-bench --bench interpreter -- --save-baseline cpp-linux
```

**Two corrections to the obvious form of this command, both found by running it.** The binary path must be **absolute**: cargo runs a bench binary with the *crate's* manifest directory as cwd, not the invocation directory, so a relative `../build/bin/rexx` resolves against `rust/crates/rexx-bench/../build/bin/` and does not exist. This fails identically on every platform, so it is a plan bug rather than a local quirk. And `--bench interpreter` is required: without it, `--save-baseline` is forwarded to every test and bench binary in the package, including plain `#[test]` harnesses that reject it with "Unrecognized option".

Criterion's defaults are also wrong for programs this size — `sample_size = 100` and a 5 s measurement time would cost minutes per benchmark. Use `sample_size(10)` (criterion's floor), 500 ms warmup, and a 30 s measurement ceiling per group. Later phases adding Rust numbers to the same file must keep these settings or the comparison is meaningless.

- [x] **Step 4: Record cold start separately**

hyperfine is an external binary and is not available on every platform this gate must run on, so the workspace carries its own: `rexx-bench`'s `src/bin/rexx-time.rs`, which runs a command N times and reports min/median/mean.

```bash
cargo run --offline -p rexx-bench --bin rexx-time -- \
    --warmup 10 --runs 50 "$PWD/../build/bin/rexx" bench-programs/startup.rex
```

This number is the D2 gate. Use it rather than criterion's `startup` row — criterion goes through `Command::output()` pipe capture on every iteration, which is a real code path but a different one.

- [x] **Step 5: Record the baseline on the other four platforms**

Add a `bench` job to `.github/workflows/{unix,windows,bsd}.yml` that builds the C++ tree, runs the suite, and uploads `target/criterion` as an artifact. Do not gate CI on it yet — this run only produces numbers.

**If OpenBSD cannot produce a baseline,** because of the open SIGSEGV in the current C++ build, record that in `perf-baseline.md` as a missing row with the failure output attached, and proceed. Benchmarks may well run on a build whose test suite crashes, so try first. What is not acceptable is a silently absent row: the Phase 0 gate below asks for five platforms, and "four plus a documented reason" passes while "four" does not.

- [x] **Step 6: Write the baseline report**

`perf-baseline.md` records, per platform: each benchmark's point estimate and confidence interval, the toolchain versions, and the machine class. **Every later phase gate compares against this file.**

- [x] **Step 7: Commit**

```bash
git add rust docs .github
git commit -m "Add the interpreter benchmark suite and record the C++ baseline"
```

### Task 0.8: Answer D7 — is the RXAPI wire protocol stable?

**Files:**
- Create: `docs/superpowers/plans/rxapi-protocol.md`

- [x] **Step 1: Read the protocol definition**

Read `rexxapi/common/` (9 files) — specifically the request/reply message structs and any version field — plus how `rexxapi/client/` frames requests and `rexxapi/server/` dispatches them.

- [x] **Step 2: Answer three questions in writing**

In `rxapi-protocol.md`: (1) Is there a protocol version field, and is a mismatch detected or ignored? (2) Are messages fixed-layout C structs, and if so are they sensitive to compiler padding, endianness, or pointer width? (3) What is the transport on each of the five platforms?

- [x] **Step 3: Record the D7 decision**

If the protocol is versioned and layout-portable, confirm D7 as "bridge to the C++ `rxapi`". If it is an unversioned struct dump, flip D7 to "port `rexxapi/` in Phase 10" and add 12k LOC to the Phase 10 estimate. Update Section 1's D7 block in this file with the answer and the evidence.

- [x] **Step 4: Commit**

```bash
git add docs
git commit -m "Document the RXAPI wire protocol and settle the bridge-or-port decision"
```

### Phase 0 exit gate

**Assessed 2026-07-27: four of five met. Phase 1 may start.**

- [x] `rexx-diff --cpp build/bin/rexx --rs build/bin/rexx --corpus rust/corpus` reports **13 programs, 0 divergences**. Negative control checked: substituting another binary reports 13 divergences and exit 1; an empty or unreadable corpus exits 2.
- [ ] **NOT MET — `perf-baseline.md` has the Linux row only.** macOS, Windows, FreeBSD and OpenBSD need CI runs, which need a push. Nothing else in Phase 0 or Phase 1 depends on those four rows, so this does not block Phase 1; it blocks *closing* Phase 0, and it must be met before any phase gate claims a cross-platform performance result.
- [x] `rexx_inventory::errors::MESSAGES` has 704 entries (56 majors + 648 submessages, keyed by `(major, sub)`); `builtins::NAMES` has 81 in table order. Eight tests, including `oracle_agreement.rs`, which checks all 704 renderings against `RexxErrorMessages.h` and finds zero mismatches.
- [x] `l1-coverage.md` committed; **D8 closed at 86.2%** (82.7% corrected), so L1 stays.
- [x] `rxapi-protocol.md` committed; **D7 closed** as bridge-to-C++-rxapi, with `sizeof(ServiceMessage) == 600` verified by compiling a probe.

**Beyond the gate**, Phase 0 also produced two things the plan did not ask for and later phases need:

- `conformance-baseline.md` — the C++ oracle's own ooTest result on Linux: 24,372 tests, 391,542 assertions, 2 failures, 3 errors. The L3 gate is now *proven runnable* rather than merely specified, and it establishes that the target is "match the oracle", not "zero failures".
- **D13 closed** — the AST is a private implementation detail, so the Rust AST is plain owned data. That unblocks Phase 3 and preserves D10's combinator option.

Three decisions closed in Phase 0 (D7, D8, D13), all by measurement rather than argument.

---

## 5. Phase 1 — Heap and object model

**Entry:** Phase 0 gate green. **This phase closes D1 by measurement.** If the numbers fail, the correct response is to revisit D1, not to proceed and hope.

### Task 1.1: `ObjRef` — tagged handles

**Files:**
- Create: `rust/crates/rexx-core/Cargo.toml`, `src/lib.rs`, `src/handle.rs`
- Create: `rust/crates/rexx-core/tests/handle.rs`

**Interfaces:**
- Produces: `ObjRef` (Copy, Eq, Hash), `ObjRef::heap(slot: u32, generation: u32)`, `ObjRef::small_int(i64) -> Option<ObjRef>`, `ObjRef::NIL`, `ObjRef::decode() -> Decoded`, `enum Decoded { Heap { slot: u32, generation: u32 }, SmallInt(i64), Nil }`, `GENERATION_MAX`.

**Why the generation field exists.** Slots are recycled through a free list (Task 1.2) and swept (Task 1.5). Without a generation, a handle held across a collection silently aliases whatever is allocated into that slot next — `Heap::get` returns `Some(wrong object)`. That is memory-safe and semantically exactly the wrong-object defect class this rewrite exists to eliminate, and it would land at the FFI boundary, which is the one place D5 advertises as the win. The generation makes a stale handle a lookup miss, which is what D5 actually claims.

- [x] **Step 1: Write the failing test**

```rust
use rexx_core::{Decoded, ObjRef};

#[test]
fn heap_handles_round_trip() {
    for slot in [0u32, 1, 1000, u32::MAX] {
        for generation in [0u32, 1, 7, rexx_core::GENERATION_MAX] {
            assert_eq!(
                ObjRef::heap(slot, generation).decode(),
                Decoded::Heap { slot, generation }
            );
        }
    }
}

#[test]
fn the_same_slot_at_different_generations_is_a_different_handle() {
    assert_ne!(ObjRef::heap(4, 0), ObjRef::heap(4, 1));
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
    assert_ne!(ObjRef::NIL, ObjRef::heap(0, 0));
    assert_ne!(ObjRef::NIL, ObjRef::small_int(0).unwrap());
}
```

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core`
Expected: FAIL — crate does not exist.

- [x] **Step 3: Implement**

`rust/crates/rexx-core/src/handle.rs`:
```rust
//! A reference to a Rexx object.
//!
//! Two low bits carry a tag. A `Heap` handle carries a 32-bit slot index and
//! a 30-bit generation; `SmallInt` carries a 62-bit signed value inline,
//! which removes the allocation the C++ implementation pays for via
//! `RexxInteger`. `.nil` is a singleton because Rexx code compares against it
//! by identity.
//!
//! Note that `.true` and `.false` need no encoding: in Rexx they are the
//! strings "1" and "0".
//!
//! Layout, low to high: [tag: 2][slot: 32][generation: 30].

const TAG_BITS: u32 = 2;
const TAG_MASK: u64 = 0b11;
const TAG_HEAP: u64 = 0b00;
const TAG_INT: u64 = 0b01;
const TAG_NIL: u64 = 0b10;

const SLOT_SHIFT: u32 = TAG_BITS;
const SLOT_BITS: u32 = 32;
const SLOT_MASK: u64 = (1 << SLOT_BITS) - 1;
const GEN_SHIFT: u32 = SLOT_SHIFT + SLOT_BITS;
const GEN_BITS: u32 = 30;

/// The highest generation a slot can reach. A slot that would exceed this is
/// retired rather than reused, so a stale handle can never alias a live one.
pub const GENERATION_MAX: u32 = (1 << GEN_BITS) - 1;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjRef(u64);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Decoded {
    Heap { slot: u32, generation: u32 },
    SmallInt(i64),
    Nil,
}

/// Inclusive bounds of the inline integer range.
pub const SMALL_INT_MAX: i64 = (1 << 61) - 1;
pub const SMALL_INT_MIN: i64 = -(1 << 61);

impl ObjRef {
    pub const NIL: ObjRef = ObjRef(TAG_NIL);

    pub const fn heap(slot: u32, generation: u32) -> Self {
        debug_assert!(generation <= GENERATION_MAX);
        ObjRef(
            ((generation as u64) << GEN_SHIFT)
                | ((slot as u64) << SLOT_SHIFT)
                | TAG_HEAP,
        )
    }

    pub const fn small_int(value: i64) -> Option<Self> {
        if value > SMALL_INT_MAX || value < SMALL_INT_MIN {
            return None;
        }
        Some(ObjRef((((value as u64) << TAG_BITS) & !TAG_MASK) | TAG_INT))
    }

    pub const fn decode(self) -> Decoded {
        match self.0 & TAG_MASK {
            TAG_HEAP => Decoded::Heap {
                slot: ((self.0 >> SLOT_SHIFT) & SLOT_MASK) as u32,
                generation: (self.0 >> GEN_SHIFT) as u32,
            },
            TAG_INT => Decoded::SmallInt((self.0 as i64) >> TAG_BITS),
            _ => Decoded::Nil,
        }
    }
}
```

Note the cost this buys back: a `Heap` handle no longer has spare bits, so the arena is capped at 2^32 slots. That is 4 billion live objects, far past any Rexx program, and `Heap::alloc` already fails loudly at that bound.

`src/lib.rs`:
```rust
mod handle;
pub use handle::{Decoded, ObjRef, GENERATION_MAX, SMALL_INT_MAX, SMALL_INT_MIN};
```

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core`
Expected: 5 passed.

- [x] **Step 5: Commit**

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

- [x] **Step 1: Write the failing test**

```rust
use rexx_core::{Body, Decoded, Heap, ObjRef};

#[test]
fn allocation_returns_a_heap_handle_that_reads_back() {
    let mut heap = Heap::new();
    let s = heap.alloc(Body::String("hello".into()));
    assert!(matches!(s.decode(), Decoded::Heap { .. }));
    assert!(matches!(heap.get(s).map(|o| &o.body), Some(Body::String(t)) if t == "hello"));
}

#[test]
fn a_handle_from_a_stale_generation_does_not_read_the_slots_new_occupant() {
    let mut heap = Heap::new();
    let stale = heap.alloc(Body::String("gone".into()));
    let Decoded::Heap { slot, generation } = stale.decode() else { panic!("heap handle") };
    let forged = ObjRef::heap(slot, generation + 1);
    assert!(heap.get(forged).is_none(), "a generation mismatch is a miss, not an alias");
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

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test heap`
Expected: FAIL — `Heap` is not defined.

- [x] **Step 3: Implement**

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

/// A slot carries its generation whether occupied or not, so that a handle
/// minted before a sweep cannot read the slot's next occupant.
enum Slot {
    Free { next: Option<u32>, generation: u32 },
    Live { object: Object, generation: u32 },
}

impl Slot {
    fn generation(&self) -> u32 {
        match self {
            Slot::Free { generation, .. } | Slot::Live { generation, .. } => *generation,
        }
    }
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
                let Slot::Free { next, generation } = self.slots[slot as usize] else {
                    unreachable!("the free list only threads free slots")
                };
                self.free_head = next;
                self.slots[slot as usize] = Slot::Live { object, generation };
                ObjRef::heap(slot, generation)
            }
            None => {
                let slot = u32::try_from(self.slots.len()).expect("heap exceeds 2^32 slots");
                self.slots.push(Slot::Live { object, generation: 0 });
                ObjRef::heap(slot, 0)
            }
        }
    }

    /// Resolves a handle, or `None` if it names no slot, a free slot, or a
    /// slot whose generation has moved on.
    fn resolve(&self, r: ObjRef) -> Option<usize> {
        let Decoded::Heap { slot, generation } = r.decode() else { return None };
        let entry = self.slots.get(slot as usize)?;
        (entry.generation() == generation && matches!(entry, Slot::Live { .. }))
            .then_some(slot as usize)
    }

    pub fn get(&self, r: ObjRef) -> Option<&Object> {
        let slot = self.resolve(r)?;
        match &self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
        }
    }

    pub fn get_mut(&mut self, r: ObjRef) -> Option<&mut Object> {
        let slot = self.resolve(r)?;
        match &mut self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
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

Export both modules from `src/lib.rs`. Note that `Slot::Free` is not constructed until Task 1.5, so this task's commit would trip `-D warnings` on dead code; add `#[allow(dead_code)] // constructed by the sweep in Task 1.5` to the variant and delete the attribute in Task 1.5. Suppressing a warning you are about to make true is fine; leaving it suppressed is not.

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test heap`
Expected: 4 passed.

- [x] **Step 5: Commit**

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

- [x] **Step 1: Write the failing test**

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
    let a = ObjRef::heap(3, 0);
    let mut out = Vec::new();
    Body::Array(vec![a, a, ObjRef::NIL]).trace(&mut out);
    assert_eq!(out, vec![a, a, ObjRef::NIL]);
}

#[test]
fn an_instance_reaches_its_variable_values_but_not_their_names() {
    let v = ObjRef::heap(9, 0);
    let mut out = Vec::new();
    Body::Instance(vec![("NAME".into(), v)]).trace(&mut out);
    assert_eq!(out, vec![v]);
}
```

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test trace`
Expected: FAIL — no method named `trace`.

- [x] **Step 3: Implement**

```rust
impl Body {
    /// Appends every object this one can reach.
    ///
    /// This single exhaustive match replaces the 148 hand-written `live()`
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

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test trace`
Expected: 3 passed.

- [x] **Step 5: Commit**

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
    let env = ObjRef::heap(1, 0);
    roots.add_global(".ENVIRONMENT", env);
    assert!(roots.iter().any(|r| r == env));
}

#[test]
fn temporaries_stop_being_roots_when_their_frame_is_popped() {
    let mut roots = RootSet::new();
    let tmp = ObjRef::heap(5, 0);
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
    roots.push_temp(ObjRef::heap(1, 0));
    let _inner = roots.push_frame();
    roots.push_temp(ObjRef::heap(2, 0));
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

- [x] **Step 1: Write the failing test**

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

#[test]
fn a_handle_to_a_swept_object_does_not_alias_the_slots_next_occupant() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let stale = heap.alloc(Body::String("x".into()));
    heap.collect(&roots);
    let reused = heap.alloc(Body::String("y".into()));
    assert_ne!(stale, reused, "reuse must bump the generation");
    assert!(heap.get(stale).is_none(), "the stale handle reads as a miss");
    assert!(matches!(heap.get(reused).map(|o| &o.body), Some(Body::String(t)) if t == "y"));
}
```

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test collect`
Expected: FAIL — no method named `collect`.

- [x] **Step 3: Implement**

Add a `marks: Vec<bool>` field to `Heap` **and initialise it in `Heap::new()`** — the struct and the constructor both, or this does not compile. Then:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectStats {
    pub swept: usize,
    pub live: usize,
}

impl Heap {
    pub fn slot_capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn collect(&mut self, roots: &RootSet) -> CollectStats {
        self.marks.clear();
        self.marks.resize(self.slots.len(), false);

        let mut work: Vec<ObjRef> = roots.iter().collect();
        let mut reached = Vec::new();
        while let Some(r) = work.pop() {
            let Some(slot) = self.resolve(r) else { continue };
            if std::mem::replace(&mut self.marks[slot], true) {
                continue; // already marked: this is what terminates cycles
            }
            let Slot::Live { object, .. } = &self.slots[slot] else {
                unreachable!("resolve rejects free slots")
            };
            reached.clear();
            object.body.trace(&mut reached);
            work.extend(reached.iter().copied());
        }

        let mut swept = 0;
        for slot in 0..self.slots.len() {
            if self.marks[slot] || matches!(self.slots[slot], Slot::Free { .. }) {
                continue;
            }
            let generation = self.slots[slot].generation();
            swept += 1;
            self.live -= 1;
            // A slot whose generation would overflow is retired, not reused:
            // wrapping would let a stale handle alias a live object again,
            // which is the whole reason the generation exists.
            self.slots[slot] = match generation.checked_add(1) {
                Some(next) if next <= crate::GENERATION_MAX => {
                    let free = Slot::Free { next: self.free_head, generation: next };
                    self.free_head = Some(slot as u32);
                    free
                }
                _ => Slot::Free { next: None, generation },
            };
        }
        CollectStats { swept, live: self.live }
    }
}
```

Note the two invariants the tests depend on and that are easy to drop: the sweep decrements `live`, and `marks` is resized on every collection because the heap grows between them.

The retirement branch is not reachable from the public API — no test can allocate 2^30 times — so cover it with a unit test inside `src/heap.rs`, where the private fields are visible:

```rust
#[cfg(test)]
mod retire_tests {
    use super::*;
    use crate::{Decoded, GENERATION_MAX};

    #[test]
    fn a_slot_at_generation_max_is_retired_not_reused() {
        let mut heap = Heap::new();
        let roots = RootSet::new();
        let r = heap.alloc(Body::String("old".into()));
        let Decoded::Heap { slot, .. } = r.decode() else { panic!("heap handle") };
        if let Slot::Live { generation, .. } = &mut heap.slots[slot as usize] {
            *generation = GENERATION_MAX;
        }
        let stale = ObjRef::heap(slot, GENERATION_MAX);
        heap.collect(&roots);
        let next = heap.alloc(Body::String("new".into()));
        assert_eq!(heap.slot_capacity(), 2, "the retired slot must not be reused");
        assert!(heap.get(stale).is_none(), "the stale handle still misses");
        assert!(heap.get(next).is_some());
    }
}
```

One residual, worth knowing rather than fixing: `ObjRef::heap`'s `debug_assert!` on the generation compiles out in release, so a release-mode call with `generation == 2^30` would wrap into the slot bits silently. That is safe only as long as `collect`'s retirement branch remains the sole producer of generations, which it is. If a second producer ever appears, promote the `debug_assert!` to a real check.

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core`
Expected: 6 integration tests in `collect` plus the `retire_tests` unit test, all passing.

- [x] **Step 5: Commit**

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

**Pass order is fixed by the oracle, and it is the opposite of the intuitive one.** `MemoryObject::markObjects` (`interpreter/memory/RexxMemory.cpp:415–433`) runs `markObjectsMain` → **`checkWeakReferences`** → `checkUninit` → `markObjectsMain(uninitTable)`, and the comment at `:422–426` gives the reason verbatim: weak references are processed before the uninit list *"so that the uninit list doesn't mark any of the weakly referenced items. We don't want an object placed on the uninit queue to end up strongly referenced later."*

Getting this backwards is observable: take a `WeakReference` whose target is unreachable but pending `UNINIT`. Clearing weak refs first — the oracle's order — reads `.nil`. Resurrecting first reads the live object. Both are defensible designs; only one is ooRexx.

- [x] **Step 1: Write the failing test**

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

#[test]
fn a_weak_reference_to_an_uninit_pending_object_is_still_cleared() {
    // The oracle clears weak references BEFORE the uninit list is marked, so
    // resurrection for UNINIT must not retroactively rescue a weak reference.
    // See RexxMemory.cpp:422-426.
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let target = heap.alloc(Body::Instance(vec![]));
    heap.get_mut(target).unwrap().has_uninit = true;
    let weak = heap.alloc(Body::WeakRef(target));
    roots.add_global(".WEAK", weak);
    let stats = heap.collect(&roots);
    assert_eq!(stats.pending_uninit, vec![target], "it is still queued for UNINIT");
    assert!(heap.get(target).is_some(), "and still alive until UNINIT has run");
    assert!(
        matches!(heap.get(weak).map(|o| &o.body), Some(Body::WeakRef(r)) if *r == rexx_core::ObjRef::NIL),
        "but the weak reference was cleared before resurrection, as in the oracle"
    );
}
```

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test uninit`
Expected: FAIL — `Body::WeakRef` does not exist.

- [x] **Step 3: Implement**

`Body::WeakRef(ObjRef)` traces to nothing. `collect` gains two post-mark passes **in the oracle's order**:

1. **Clear weak references.** For every surviving `Body::WeakRef` whose target is unmarked, rewrite the target to `ObjRef::NIL`. "Unmarked" must include *unresolvable* — a target whose slot is already free, or whose generation has moved on, is dead, and a weak reference whose target died in an earlier cycle must still clear. Route the check through `resolve`, which answers all three cases at once.
2. **Resurrect for `UNINIT`.** For every unmarked object with `has_uninit`, mark it and everything it reaches, and record it in `pending_uninit` — running `UNINIT` must not see a half-collected object graph.

Then sweep. `has_uninit` is cleared when the caller reports the finalizer has run, so the next collection sweeps the object normally.

Do not swap these for tidiness. Pass 2 marks objects; if it ran first, pass 1 would see those marks and leave the weak references pointing at objects the oracle would have cleared.

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test uninit`
Expected: 4 passed.

- [x] **Step 5: Commit**

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

- [x] **Step 1: Write the failing test**

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

- [x] **Step 2: Run to verify it fails**

Run: `cd rust && cargo test -p rexx-core --test behaviour`
Expected: FAIL — `BehaviourTable` is not defined.

- [x] **Step 3: Implement**

A `Vec<BehaviourEntry>` indexed by `BehaviourId.0`, each with `superclass: Option<BehaviourId>` and a `HashMap<String, MethodId>` keyed by the uppercased name. `lookup` walks the superclass chain, with a visited set so a bootstrap cycle (`Class` ↔ metaclass) cannot loop forever.

- [x] **Step 4: Run to verify it passes**

Run: `cd rust && cargo test -p rexx-core --test behaviour`
Expected: 4 passed.

- [x] **Step 5: Commit**

```bash
git add rust
git commit -m "Look up methods through behaviours and the superclass chain"
```

### Task 1.8: The D1 measurement — this is the gate

**Files:**
- Create: `rust/crates/rexx-core/benches/heap.rs`
- Modify: `rust/crates/rexx-core/Cargo.toml`
- Create: `docs/superpowers/plans/d1-decision.md`

- [x] **Step 1: Write the allocation-throughput benchmark**

Criterion benchmark: allocate 1,000,000 `Body::String` objects of ~16 bytes with no collection, and separately 1,000,000 `Body::Array(vec![_; 4])`. Report allocations per second.

- [x] **Step 2: Write the collection-pause benchmark**

Build a graph of 1,000,000 objects with a realistic shape — a root directory holding 1,000 arrays of 1,000 elements each, 10% of which are cross-links — then time a single full `collect`. Report the pause.

- [x] **Step 3: Write the equivalent C++ measurement**

A Rexx program (`rust/bench-programs/heapshape.rex`) that builds the same graph shape using `.array` and `.directory`, run under `build/bin/rexx`, timed with hyperfine. It measures allocation plus collection together, so subtract the interpreter overhead measured by an equivalent program that builds nothing.

**State the comparison's weakness in `d1-decision.md`, do not bury it.** The Rust side is a direct API microbenchmark; the C++ side is an interpreted program minus an estimated overhead. These are not like-for-like, the subtraction is an estimate, and the interpreted side pays for parsing, dispatch, and variable lookup that the Rust side never touches. The number is directional — good enough to detect a 3× disaster, not good enough to adjudicate 15%. That is exactly why the Phase 1 threshold is 1.5× rather than parity, and why Phase 4 re-measures on equal footing. An unstated adjustment is how a benchmark lies.

- [x] **Step 4: Run both and record**

```bash
cd rust
cargo bench -p rexx-core -- --save-baseline rust-heap-linux
hyperfine --warmup 3 '../build/bin/rexx bench-programs/heapshape.rex'
```

- [x] **Step 5: Close D1**

Write `d1-decision.md` with both sets of numbers and the verdict:

- **Allocation throughput within 1.5× of C++ and full-GC pause within 1.5×:** D1 closes as (a). Record it in Section 1 and proceed to Phase 2.
- **Allocation between 1.5× and 3× slower:** the arena is probably fine but the representation needs work. Two candidate causes, and they want opposite fixes — diagnose before acting:
  - *The `Body` enum is too wide.* Every slot costs `size_of::<Body>()`, set by the largest variant. Box the large, rare variants and re-measure.
  - *Strings are paying two allocations where C++ pays one.* **This is the more likely cause, and boxing makes it worse.** See below.

  **Pre-registered: the string-representation problem.** C++ stores string bytes *inline with the object header* — `RexxString` ends in `char stringData[4]`, the flexible-array-member idiom, so a string is one variable-sized allocation with its header and bytes contiguous (`StringClass.hpp:98`, `:538–543`). The arena as specified in Task 1.2 uses `Body::String(String)`, which is a fixed-size slot *plus* a separate heap buffer: **two allocations and a pointer chase for the single most common object in the language.** Rexx is string-dominated in a way few languages are — every number is a string, every symbol is a string, every concatenation makes one — so this is the likeliest thing to show up as a bad number in Task 1.8.

  The fix, if the measurement demands it, is a **side byte-arena**: `Body::String { offset: u32, len: u32 }` indexing a `Vec<u8>` string heap held alongside the slot vector. That restores C++'s one-allocation, contiguous-bytes property while staying entirely safe, and compaction of the byte heap folds naturally into the sweep. It works because `RexxString` is **immutable** in ooRexx — strings are never modified in place, so no slot ever needs to grow. `MutableBuffer`, which *is* mutable, keeps its own `Vec<u8>` and is rare enough not to matter.

  Do not build the byte-arena speculatively. Build `Body::String(String)` first because it is simpler, measure, and reach for this only if the number says to. It is recorded here so that the response to a bad measurement is a considered design rather than an improvised one.
- **Worse than 3× on either metric:** D1(a) is refuted. Do not proceed. Re-open D1 and evaluate the hybrid — `#[repr(C)]` inline headers with a side table for tracing — first, since it may recover the loss while staying safe. Option (c) is the last resort and does not get a pass on the Global Constraints unsafe bar: a raw-pointer heap would have to clear all four bars for a module that, by its nature, cannot encapsulate its invariant behind a safe API. That it cannot clear bar 2 is itself the argument against it. If the measurement lands here, the honest options are the hybrid or stopping — not quietly relaxing the constraint.

**Do not soften the gate to keep the schedule.** The whole argument for this rewrite is that it can be safe *and* fast; a Rust interpreter that is safe and slow is not worth 200k LOC of work, and finding that out at Phase 1 costs weeks instead of years.

- [x] **Step 6: Commit**

```bash
git add rust docs
git commit -m "Measure arena allocation and collection against the C++ heap, and close D1"
```

### Phase 1 exit gate

**Assessed 2026-07-27: all five met. Phase 2 may start.**

- [x] `cargo test -p rexx-core` green — **33 tests**. `cargo clippy --all-targets -- -D warnings` clean.
- [x] `#![forbid(unsafe_code)]` holds; `grep -rc unsafe rust/crates` reports **zero** across the workspace, and every crate root is `forbid`, not `deny` — so no crate has been granted an exception.
- [x] `Body::trace` is a single exhaustive match with **no wildcard arm**, verified by grep. Adding a `Body` variant is a compile error rather than a silent leak.
- [x] The root set is documented and enumerable, and **no `ProtectedObject` analogue exists** — the only occurrence of the name in the crate is the doc comment in `roots.rs` explaining why there isn't one.
- [x] `d1-decision.md` committed; **D1 closed** as arena + generation-checked handles.

**The one thing carried forward as debt.** The GC pause is **1.45×** the C++ figure (26.5 ms against 18.2 ms) — inside Phase 1's 1.5× viability threshold but outside parity, which is the gate from Phase 2 onward. Re-measure at Phase 4 on equal footing. The pre-registered string-representation fix (side byte-arena) is the first thing to reach for if it still misses, and remains deliberately unbuilt.

---

## 6. Generating the plans for Phases 2–10

Each subsequent phase gets its own plan file at `docs/superpowers/plans/YYYY-MM-DD-phase-N-<name>.md`, written with `superpowers:writing-plans` at the start of that phase — not now. Writing them now would be guessing: Phase 4's task breakdown depends on what Phase 3's AST actually looks like.

The generating procedure for each phase:

1. **Read the C++ it replaces.** Name the exact files and line counts. The phase plan opens with that inventory.
2. **Enumerate the observable behaviours,** not the functions. For Phase 3 that is error messages with number, sub-number, **line** and substitution values, `SOURCELINE`, and `TRACE`'s `*-*` source lines — not "the scanner tokenises correctly". There is **no column** anywhere in the oracle: the condition object exposes `POSITION`, which is the line, and stderr carries no offset either, because ooRexx locates an error by quoting the offending token. Do not gate any phase on a column.
3. **Write the L0 corpus entries first.** Every behaviour in step 2 becomes a `.rex` program in `rust/corpus/` that the C++ oracle already passes. These are the phase's acceptance tests, written before any Rust.
4. **Decompose into tasks of one testable deliverable each,** in dependency order, following the Task Structure in `superpowers:writing-plans`.
5. **State the exit gate** as: corpus subset at zero divergences + L-rung reached + benchmark comparison against `perf-baseline.md` + the unsafe-block count, which must be zero or accounted for by a Section 1 decision block.
6. **Name the upstream decisions** the phase depends on and confirm each is closed.

**Phase-specific notes to carry forward:**

- **Phase 3** opens with the D10 spike (parser construction), then decides how source text is retained. The spike has a hard number to beat: C++ starts in 5.1 ms from a memory-mapped image, so under D2 the Rust parser must get through `CoreClasses.orx` fast enough to keep total cold start inside ~55 ms. D13 is already closed: the AST is plain owned Rust data inside one arena object per code body. `SOURCELINE`, error reporting, and `TRACE` all expose the original text, so the AST cannot discard it. Keep the program source as one byte buffer -- not a Rust `String`, because a Rexx literal may hold bytes that are not valid UTF-8 (D14) -- and have AST nodes hold byte ranges into it — which is also what makes `chumsky`'s span support directly usable if D10 lands on (a). Measure parse throughput on `CoreClasses.orx`, since under D2 that number *is* cold-start time.
- **Phase 4** is where the execution model is fixed. Read the existing performance profile before designing the dispatch loop. The 81 builtins from Task 0.6 are the checklist; tick them off individually.
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
| `Sys*` blocks L2 later than expected | ooTest cannot enumerate groups without `SysFileTree` (D11) | Phase 7's plan opens by grepping `ootest/` for `Sys` call sites, so the required subset is measured rather than guessed |
| ooDialog does not recompile against the Rust API | Only discovered after Phase 10, if ever | Accepted. It is out of scope and the goal statement says so. If recompiling it matters, add a Phase 11 with its own gate rather than letting D5 imply a guarantee nothing tests |
| A stale native-API handle reads the wrong object | Silent wrong answers at the FFI boundary, not a crash | Designed out by the generation field in `ObjRef` (Task 1.1), tested in Tasks 1.2 and 1.5. If a future change drops generations for space, this row is why it must not |
| Effort exceeds available time | Phase 4 not closed within its estimate | **Decision point, not a failure.** The C++ tree is untouched and still ships. Either narrow scope to a Rexx subset that is explicitly not ooRexx-conformant, or stop and keep Phase 0's oracle and benchmark suite, which have standalone value for the C++ project |

**The strongest property of this plan is that abandoning it is cheap.** The C++ tree is never modified. Phase 0 produces a differential runner and a five-platform benchmark baseline that improve the existing project whether or not a single line of the Rust interpreter is ever written. Stopping after any phase leaves the repository better than it started.

---

## 8. Rejected alternatives

**Strangler / in-place oxidation** — replace C++ subsystems one at a time behind a C ABI. Rejected by the user in favour of clean-room. Worth recording why it is genuinely worse here: the GC is the *first* thing you would have to replace, since every other subsystem depends on the object representation, and replacing the GC while half the tree still holds raw pointers means keeping the `ProtectedObject` discipline — which is the thing the rewrite exists to eliminate. Oxidation gets the risk profile of a rewrite with none of the benefit until the very end.

**Differential fuzzing as a gate** — generating random Rexx programs and diffing the two interpreters. Not selected. It would find numeric and `PARSE` edge cases that a hand-written corpus misses, and if the L0 corpus proves too thin in Phase 4, this is the first thing to add.

**Corpus replay — *executing* the 301 `samples/` programs — as a primary gate** — not selected as a gate, but the samples remain the natural expansion of the L0 corpus once Phase 4 is under way. Many touch the file system or the console and need harnessing before they are deterministic. This does **not** apply to *parsing* them, which needs no harnessing at all and *is* a Phase 3 gate criterion: all 301 pass `build/bin/rexxc` today, so the expected answer for every one is "parses" and the oracle half is one shell loop.

**A general decimal crate for `NUMERIC`** — see D4. The ANSI Rexx rules differ from IEEE decimal in ways that are individually small and collectively fatal to conformance.

---

## 9. Self-review

**Spec coverage.** Every decision from the user's four answers is carried: clean-room reimplementation (Section 3 crate tree, C++ tree frozen in Global Constraints); source-compatible C API (D5, Phase 8); ooTest as gate (D8, the L-rungs, Phase 9); perf non-regression (D9, Task 0.7, every phase gate); agent-executable roadmap (Phases 0–1 at step granularity, Section 6 for the rest).

**Placeholders.** No "TBD"s, and every gate carries a runnable command. But the earlier claim that *every* code step carries real code was false, and is withdrawn: several steps specify behaviour in prose rather than code — `build.rs` in Tasks 0.5 and 0.6, the benchmark harness in Task 0.7, the `rexx-extract` binary in Task 0.4, and `BehaviourTable` in Task 1.7. Each of those states its inputs, outputs, failure modes, and the tests it must satisfy, which is enough to implement against; but an implementer will be writing code the plan describes rather than transcribing code the plan supplies, and should expect that. The steps that *do* supply code supply all of it.

Five things are deliberately unmeasured and each names the task or spike that measures it: the L1 extractable fraction (Task 0.4), the RXAPI protocol answer (Task 0.8), the D1 verdict (Task 1.8), and the D13 AST-ownership grep and D10 parser comparison (both at the head of Phase 3).

**Constraints added after the first draft, and where they landed.** The image is optional rather than obligatory — D2 now defaults to no image, builds one only on a measured startup miss, and records why that ordering cannot waste work. `unsafe` is forbidden by default everywhere with no blanket crate exemptions; the four-bar admission protocol is in Global Constraints, the unsafe-block count is a reportable item at every phase exit, and the D1 fallback to raw pointers is explicitly *not* granted a pass on it.

**Type consistency.** `ObjRef`, `Decoded`, `Body`, `Object`, `BehaviourId`, `MethodId`, `Heap`, `Slot`, `RootSet`, `FrameId`, `CollectStats`, `Message`, `Outcome`, `Interpreter`, `Divergence`, and `TestMethod` are each defined once and used with the same signature everywhere they appear.

**Corrections applied after external review, recorded so the same errors are not reintroduced.** Message rendering had `<q>` markup dropped as documentation-only; the oracle's own generator renders `<q>X</q>` as `"X"` with the quotes kept (`RexxErrorMessages.xsl:86–88`, proven by `RexxErrorMessages.h:62`), across 363 occurrences, 36 of which wrap literal text and would have diverged even without substitutions. The substitution marker is `&N`, not `%N`. The builtin count was 162 and is 81 — the original figure double-counted declarations against table entries, and the table is *not* alphabetical, so the sortedness assertion went too. `live()` is 148 and `flatten()` is 105, both counted as definitions in `.cpp`; the 106th `flatten` match is a commented-out line in `RexxCore.h`. Keyword instructions are 35. The error catalogue is 56 majors plus 648 submessages keyed by `(major, sub)`, not a flat code table. Task 1.6's weak-reference and `UNINIT` passes were in the wrong order relative to `RexxMemory.cpp:415–433`. `ObjRef` gained a generation field because the original design let a stale handle alias a recycled slot, which would have falsified D5's central safety claim. D2 overstated its savings by ignoring that `rexxc` needs program flattening regardless.

**Corrections from the third pass, over the decision blocks the first two never covered.** `RexxInstruction` derives from `RexxInternalObject` with `live`/`liveGeneral`/`flatten` (`RexxInstruction.hpp:63`, `:74–76`) — **the C++ AST is garbage-collected and serializable**, which every earlier section had silently assumed away; D13 now settles what the Rust AST is before Phase 3 can start. The unsafe protocol demanded `#![forbid(unsafe_code)]` at the root *and* per-module relaxation, which cannot compile (`E0453`, verified) — the mechanics are now `forbid` for clean crates, `deny` plus a module `allow` for granted ones. Its encapsulation bar also contradicted itself, requiring full encapsulation while acknowledging FFI entry points that cannot achieve it; Rust-facing and FFI-facing unsafe now carry different, separately-stated requirements. D2's "2× or 50 ms" threshold fired on deltas no user could perceive and is now absolute delta only. D10 claimed parser throughput "sets cold-start time directly" when parsing is one component of it alongside bootstrap execution. D12 assigned the whole security manager to Phase 5 despite half its interception points living in code Phase 7 has not written yet.

**Claims upgraded from assertion to evidence in the same pass.** D11's "`Sys*` blocks L2" and D12's "ooTest exercises the security manager" were both guesses when written. Both are now verified against the SVN suite with citations, and both turned out to be true — which is luck, not method, and is why they were checked.

**Known soft spot.** Task 1.4's `RootSet` is standalone; Phase 4 must connect it to the real activation and expression stacks, and the borrow-checker shape of that connection — who owns `Heap` versus `RootSet` during evaluation — is not solved here. It is a Phase 4 design output, and the first task of Phase 4's plan should be a spike on exactly that question. Recording it as unsolved is deliberate: pretending otherwise would put a wrong answer into a plan that later phases build on.
