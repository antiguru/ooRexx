**OPEN: found by Phase 9d Task 3, not yet ruled.** `.Array~methods` with no argument lists fewer methods here than on the oracle.

# `Class~methods` with no argument lists fewer methods than the oracle

Found 2026-10-18 by Phase 9d Task 3. The probe is
`docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/seeds/methods_count.rex`.
`.Array~methods` with no argument enumerates 76 names here and 107 on the oracle. With `.nil` as the
argument, both sides answer 36. The no-argument form should list the methods of the class and its
superclasses (rexxref, `Class~methods`). The crate's no-argument enumeration may omit inherited
methods. This is measured once, as counts only. The name sets have not been diffed.
