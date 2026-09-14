# Phase 8, the surface half — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to
> implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fill the native API surface the L2 slice did not, build `testbinaries/` unchanged against
the frozen headers, and pass the ooTest API groups that are not embedding.

**Architecture:** No new crate and no new grant. `rexx-api` already holds the boundary, with
`unsafe` confined to `ffi.rs` and `load.rs` by D-U1, a conversion table keyed by the extension's
declared types, and a handle registry in which a stale `RexxObjectPtr` misses. This half widens
each of those along seams the L2 slice built for the purpose: a row per `REXX_VALUE_*` code, a
typed stub per function pointer, and one library resolution path.

**Tech Stack:** Rust; `rexx-api`, `rexx-exec`; `libloading`; no new dependency.

**Spec:** `docs/superpowers/specs/2026-09-14-phase-8-native-api.md`, whose evidence base is
`docs/superpowers/specs/2026-09-14-phase-8-scoping.md`. The L2 slice's own close is
`docs/superpowers/plans/phase-8-gate.md` and its measurement is
`docs/superpowers/plans/phase-8-l2.md`.

**This plan was written after a real extension ran against the design**, which is why it exists
separately. The L2 slice deliberately refused to specify the generalisation first.

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
  field is private to it. `mem::replace` of the whole value still compiles and is accepted, named in
  the ledger rather than closed.
* **A number is measured or it is not written.** Five error numbers taken from
  `interpreter/messages/RexxErrorCodes.h` named paths nothing runs, against none taken from a run.
* **Check every citation by printing that line.** Six landed off their subject.

## Global Constraints

* **`unsafe` in exactly two files**, `rexx-api/src/ffi.rs` and `src/load.rs`, each block with a
  `SAFETY:` note naming the invariant and who establishes it (D-U1, Moritz 2026-09-14).
  `crates/rexx-core/tests/unsafe_sites.rs` is the record, is file-granular, and scans `tests/`.
* **No new dependency** beyond `libloading`.
* **The headers are frozen.** No edit under `api/`; one that seems necessary reopens D5.
* **The C++ tree, `build/`, `samples/`, `ootest/` and `oodocs/` are read-only**, and
  `build/lib/librxregexp.so` must never be rebuilt: loading the oracle's own compiled extension is
  the instrument, per the D5 amendment.
* **No process-global state is mutated.**
* **Correctness is byte-identical stdout, stderr and exit status against the C++ oracle**, on three
  separate descriptors, never `2>&1`, through the standard wrapper from a fresh empty directory.
* **Probe hazards.** Read `rust/corpus/oracle-crashes.txt` first and never run its entries.
  `testOORexx.rex` with no arguments writes fixture files into `ootest/`; use the single-group form.
* **Comments minimal** (rust/CLAUDE.md, Moritz 2026-08-27), **never stating a set's size**, no
  em-dashes, every citation landing on its subject.
* **Every extent claim is derived and its command committed.**
* **Write each negative control's prediction before running it.**
* **Four gates green at the closing commit**, and commit before a long gate run.

---

### Task 1: The routine half of the calling protocol

`::ROUTINE EXTERNAL` and a `::REQUIRES ... LIBRARY`'s routines install and then refuse loudly at the
call. `rxregexp` exports none, which is why the L2 slice could stop here.

**Files:** `rust/crates/rexx-api/src/invoke.rs`, `src/layout.rs`, `rust/crates/rexx-exec/src/lib.rs`

- [ ] **Step 1:** `RexxRoutineEntry` carries a `style`: `ROUTINE_TYPED_STYLE` uses the same two-call
      protocol as a method, `ROUTINE_CLASSIC_STYLE` is the `RXSTRING` convention
      (`api/oorexxapi.h:190-208`). Build the typed style; leave classic refusing and say which
      phase owes it, since its consumers are `rxsubcom` and the registered-function API.
- [ ] **Step 2:** Populate `CallContextInterface`, which is what a routine receives.
- [ ] **Step 3:** Delete the refusals, and update the expectations in `run/tests.rs` that assert
      their text.
- [ ] **Step 4:** Witnesses for a typed routine answering and for the classic style refusing, with
      `sourceline_oracle` companions and `corpus/phase-8.txt` rows.
- [ ] **Step 5:** Commit.

---

### Task 2: The conversion table's remaining rows

**Files:** `rust/crates/rexx-api/src/values.rs`, `tests/values.rs`

- [ ] **Step 1:** Fill every `REXX_VALUE_*` row `NativeActivation::processArguments`
      (`interpreter/execution/NativeActivation.cpp:219`) handles, one row at a time, each with the
      `Repr` its code uses.
- [ ] **Step 2:** `ARGLIST` carries `usedArglist`, which the L2 slice left because its only path
      was this unfilled row.
- [ ] **Step 3:** Per-row tests both directions, and the row-deletion control re-run on a row this
      task adds rather than one it inherited.
- [ ] **Step 4:** Commit.

---

### Task 3: The thread and method-context tables

**Files:** `rust/crates/rexx-api/src/ffi.rs`, `src/layout.rs`

- [ ] **Step 1:** Replace refusing stubs with bodies, grouped by what they touch: object
      construction, string and buffer access, collections, variables, conditions, and the
      interpreter's own state. Each group is its own commit.
- [ ] **Step 2:** `RequestGlobalReference` and its table. The handle is a function of the object, so
      a global and a local reference to the same object are the **same pointer**: a boundary
      `resolve` must consult both tables rather than choose one.
- [ ] **Step 3:** A derived test asserting no stub remains that a populated table should have
      replaced, read from the header at test time rather than from a list.
- [ ] **Step 4:** Commit per group.

---

### Task 4: The exit and IO-redirector interfaces

**Files:** `rust/crates/rexx-api/src/ffi.rs`, `src/layout.rs`, `rust/crates/rexx-exec/src/`

- [ ] **Step 1:** `ExitContextInterface`, whose consumer is `testbinaries/orxexits`.
- [ ] **Step 2:** `IORedirectorInterface`, which meets Phase 7's `ADDRESS ... WITH` work.
- [ ] **Step 3:** Witnesses against the oracle for each exit code an exit handler can be registered
      for.
- [ ] **Step 4:** Commit.

---

### Task 5: `testbinaries/` build unchanged

**Files:** none in the repository; a build directory outside it.

- [ ] **Step 1:** Build `testbinaries/` against the frozen headers **with no source edit**. A change
      that seems to need one reopens D5 as a Section 1 decision and stops the task.
- [ ] **Step 2:** Record the command and its output in the gate document. A build that succeeds says
      the headers are compatible, not that the entry points behind them work, and the gate document
      must say so.
- [ ] **Step 3:** Commit.

---

### Task 6: The six ooTest API groups that are not embedding

**Files:** `rust/corpus/phase-8.txt` and witnesses

- [ ] **Step 1:** Run `CLASSIC`, `CONVERSION`, `FUNCTION`, `INVOCATION`, `METHOD` and
      `ProcessInvocation` with the single-group form. **`RexxStart` and `ProcessRexxStart` reach the
      embedding API and are Phase 9's**; re-home them there explicitly rather than leaving them to
      be discovered.
- [ ] **Step 2:** Enumerate the failing set rather than counting it, and fix or record each member
      with an owner phase.
- [ ] **Step 3:** Commit.

---

### Task 7: Close the phase

- [ ] **Step 1:** The four gates from the run, unpiped, both failing sets enumerated and every
      member attributed.
- [ ] **Step 2:** `closed_phases.rs` gains `"Phase 8"`, and `dispatch::native`'s `OPEN` list loses
      it. Every refusal naming Phase 8 must be gone or re-homed with a reason.
- [ ] **Step 3:** Write `phase-8-gate.md`'s surface section from the run, and update the roadmap's
      row 8. **If L2 is still blocked by D-L2, say so rather than closing over it.**
- [ ] **Step 4:** Commit.
