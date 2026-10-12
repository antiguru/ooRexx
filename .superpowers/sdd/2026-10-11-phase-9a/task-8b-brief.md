### Task 8b: The items Tasks 5 and 6 moved out of their batches

Added 2026-10-12 by controller ruling (ledger, Tasks 5 and 6): each needs a new store or a new
identity rule, so the mechanical-batch rule moved it here. Each is a section 3 refusal.

**Files:** found from each refusal's constructor (`grep -rn` the refusal text in `rust/crates/rexx-exec/src`).

- [ ] **Step 1:** Corpus programs, each run against the oracle first (stdout read):
      `OBJECTNAME=` on a String, a small integer, an Array, a Stem and a VariableReference, read back
      through `~objectName` and `~string`; a metaclass with its own `NEW` (scout D section 1a's
      `id k` probe), including the second `INIT`; `Class~enhanced` running `NEW` (moved from Task 8:
      an enhancing `INIT` reached by `NEW`, `.array~enhanced(d, 3)` answering `0 Array`, the existing
      enhanced-object witnesses unchanged, memory per call no worse than the oracle's with
      Method-object sources; the rules, probes and recommended mechanism are in the workspace's
      `enhanced-semantics.md`), with `SUPER` in a `.nil`-scoped method answering the owning class
      (today `.NIL`, which also breaks `setMethod` methods); `Routine~copy`, then `.context~executable` inside both
      the original and the copy; `Package~copy`, then the copy's tables against the original's;
      `~executable` on a native-level `StackFrame`; `VariableReference~copy`; `Package~local`,
      `~annotations` tables and the security manager's info directory as store-backed tables
      (Task 6's shape); a write to `.ROUTINES` followed by a `CALL` of the written name (the oracle
      resolves through the table: `put` then `call` answers the new routine, `remove` then `call`
      raises 43.1).
- [ ] **Step 2:** Implement each as the oracle answers, one commit per item group (object names;
      metaclass `NEW`; `Routine`/`Package`/`VariableReference` copies and native `~executable`; the
      three tables; `.ROUTINES` resolution). Delete each refusal and its pins as it lands. An item that
      still cannot be done without a change to a shared hot path is reported with its measured cost.
      Per-task check per commit, with `Object`, `RexxContext`, `StackFrame`, `VarRef`, `Package`,
      `Class`, `ROUTINE` and `doc/rexxref/chapter5/Section1` through the harness.

