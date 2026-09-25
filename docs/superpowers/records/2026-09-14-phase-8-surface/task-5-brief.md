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

