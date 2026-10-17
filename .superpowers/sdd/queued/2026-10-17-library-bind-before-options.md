# `EXTERNAL "LIBRARY"` binds before `::OPTIONS` is checked on the oracle

Found 2026-10-17 by the Phase 9c Task 8 review (`.superpowers/sdd/2026-10-15-phase-9c/task-8-review.md`,
Minor 1). Program:

    ::routine r external "LIBRARY x"
    ::options digits 0

Oracle `rexx` and `rexxc`: 98.903 on line 1, rc 158. Crate `rexx` and `rexxc`: 26.5 on line 2,
rc 230. The `::method m attribute external "LIBRARY x"` form diverges the same way. The oracle binds
the library during translation before it validates `::OPTIONS`, while the crate validates
`::OPTIONS` first. No `parse-errors.tsv` row covers the pair. The owner is install order
(`rexx-exec/src/install.rs`). Fix by matching the oracle's order and add the pair as a TSV row.
