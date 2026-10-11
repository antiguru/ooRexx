# Scout D: the refusals and queued items Phase 9 inherits

Read `rules.md` in this directory first. Scratch directory: `/tmp/claude-1000/p9scope/d/`.
Report: `.superpowers/sdd/2026-10-10-phase-9-scoping/scout-d-report.md`.

Row 9 of `docs/superpowers/plans/2026-07-27-rust-rewrite.md` names inherited work. Enumerate it
from the tree, not from the row's prose.

1. **Refusals naming Phase 9.** Run `grep -rn '"Phase 9"' rust/crates/*/src` and
   `grep -n 'OWNER: Phase 9' docs/superpowers/plans/phase-4-exclusions.txt`, and also check
   `rust/corpus/refusal-sites.tsv`, `rexx-inventory`'s phase table, `rexx-api`'s
   `layout::REFUSING_MEMBERS`, and any other table that maps a refusal to a phase. For each site:
   what it refuses, a probe that reaches it (run it on both sides), what the oracle answers, and a
   size estimate (small / medium / large) with one sentence why. In particular: `Loud::native_method`
   (enumerate which primitive methods it currently covers, as a list a command produces), a second
   interpreter instance, conditions and stack frames under INTERPRET, `ReleaseLocalReference`.
2. **Refusals naming Phase 10 or 11 or none** that a non-Phase-10 ooTest group would hit: list them
   by the same commands, so the spec can check none of them is really Phase 9's.
3. **`NEW` on a subclass of String, Stem, Method, Routine and Message** (spec
   `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` D4 and ruling R2). Check whether
   `.superpowers/sdd/2026-10-07-phase-6-1/scout-d-prototype.patch` still applies at HEAD
   (`git apply --check` only, never apply), and what the 6.1 records say it lacks.
4. **Queued items.** Read every file in `.superpowers/sdd/queued/` that names Phase 9, or that is
   open and not obviously Phase 10/11's. For each: one line, the owner the file names, and whether it
   blocks an ooTest group (if you can tell cheaply). Also read the "For Moritz" section of
   `.superpowers/sdd/2026-10-07-phase-6-1/task-12-report.md` and the Phase 6.1 gate record for items
   handed forward.
5. **Licensed divergences and deviations** that affect the suite gate: where the records say this
   crate deliberately differs from the oracle (`licensed_divergences.rs`, the deviation list in the
   6.1 spec and earlier specs). List those that would make an ooTest test differ, so the spec can
   decide how the gate treats them.
