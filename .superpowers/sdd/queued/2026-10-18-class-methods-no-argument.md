**CLOSED 2026-10-30 by Phase 9d Task 8.8e1 (probe agrees).**

**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** gap.

**OPEN: found by Phase 9d Task 3, not yet ruled.** `.Array~methods` with no argument lists fewer methods here than on the oracle.

# `Class~methods` with no argument lists fewer methods than the oracle

Found 2026-10-18 by Phase 9d Task 3. The probe is
`docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/seeds/methods_count.rex`.
`.Array~methods` with no argument enumerates 76 names here and 107 on the oracle. With `.nil` as the
argument, both sides answer 36. The no-argument form should list the methods of the class and its
superclasses (rexxref, `Class~methods`). The crate's no-argument enumeration may omit inherited
methods. This is measured once, as counts only. The name sets have not been diffed.


## Task 8.8e1 evidence

Complete inherited method-name multisets agree for no argument and .nil. Duplicate entries are preserved. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.
