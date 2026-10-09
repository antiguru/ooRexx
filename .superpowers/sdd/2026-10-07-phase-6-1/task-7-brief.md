### Task 7: Dispositions and `closed_phases`

**Files:** new `rust/corpus/refusal-dispositions.tsv`, new test `rust/crates/rexx-exec/tests/refusal_dispositions.rs`,
`tests/closed_phases.rs` (`CLOSED`, debt paragraph `:31-38`, scan roots), `lib.rs` (owners of the
remaining Phase 5 constructors, `receiver_class`), `environment.rs:363` (b18), `redirect.rs:652` and
`run.rs:3838` (`.STREAM`), `tests/owners.rs` (`:154`, `:213`, `:324-331`, count `:468-473`),
`tests/assertions.rs` (`Literals` EXEMPT rows `:185-281`), `tests/bif_assertions.rs:781`,
`tests/keyword_assertions.rs:668`, `corpus/oracle-crashes.txt`, `docs/superpowers/plans/phase-4-exclusions.txt`,
`corpus/refusal-sites.tsv` (regenerated), the queued `2026-09-29-closed-phases-owner-gaps` (NC-h, NC-i),
`bench-programs/alloc4c.rex:6`, `bench-control/alloc4c-101.rex:6` (stale quoted text).

**Interfaces:** `refusal-dispositions.tsv` columns: constructor, kind (GUARD, DEVIATION, LIMIT,
REHOME), phase (REHOME only), record (probe names; an `oracle-crashes.txt` entry or exclusions row; the
spec section or DEVIATIONS entry; the quoted roadmap text). Task 8 adds LIMIT rows for its sim refusals.

- [ ] **Step 1: The test first.** It enumerates every `Loud` constructor whose owner is `None` at its
      definition (`lib.rs:366-890`) or at a call site passing `None`, class (d) included, and requires a
      row; it fails on an unlisted constructor, on a row with no constructor, and on a REHOME row whose
      constructor does not carry that phase. Run it: it fails, listing today's ownerless constructors.
- [ ] **Step 2: Rows.** GUARD: c7 `Refused::Raised`, b9, b11, b12, b15, b16, b17, b18 (relabel
      `owed[0]` as an internal guard with no phase), the `receiver_kind` `Err` and "this crate did not
      build" sites of b13, and the existing d-guard constructors, each with scout A's probe names or the census's reason, and a doc comment at each GUARD constructor citing its probes. DEVIATION: b13's guards a wrong-type native row reaches; this step writes a new `oracle-crashes.txt` entry from
      Task 4 Step 3's probe for the crashing rows (List ITEMS, Supplier ITEM, Package NAME, VariableReference NAME, StackFrame
      NAME, Routine CALL, a Supplier subclass) and the others (Array and Queue ITEMS, Table ITEMS,
      Method SCOPE, Class ID) as one exclusions row; the d-licensed constructors with their records.
      LIMIT: the Phase 6 design limits with their DEVIATIONS entries. REHOME: none from (b)/(c) unless
      a task above records one.
- [ ] **Step 3: `closed_phases`.** `CLOSED` gains `"Phase 5"`; the debt paragraph goes; the scan covers
      `src/` and the `tests/` owner tables; prune `"Phase 5"` from the `PHASES` and
      `SPLIT_TABLE_PHASES` vocabularies (`owners.rs`'s count is Task 3's). The `Literals` rows: run each
      `runDynamicSource` case outside the harness; if the interpreter runs it, fix the harness gap,
      else re-home with the reason. Close NC-h and NC-i per the queued item. Regenerate
      `refusal-sites.tsv`; fix the stale quoted text in the two bench comments.
- [ ] **Step 4:** `git grep -n '"Phase 5"' -- rust/crates` shows only `closed_phases.rs`'s `CLOSED`; the
      test and `closed_phases` pass; the per-task check. Commit.

