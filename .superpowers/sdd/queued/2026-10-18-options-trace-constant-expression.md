**CLOSED 2026-10-26 by Phase 9d Task 8.4 (7fe50bedd).**

**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** gap.

**OPEN: found by Phase 9d Task 6.6, not yet ruled.** `::OPTIONS TRACE` does not trace a `::CONSTANT` expression's method here. The oracle does.

# `::OPTIONS TRACE` does not reach a `::CONSTANT` expression

Found 2026-10-18 by Phase 9d Task 6.6 (`task-6.6-report.md`, Concerns). The oracle traces the
`::CONSTANT` method's entry, clause, results and exit under the package's `::OPTIONS TRACE`
(`docs/superpowers/records/2026-10-18-phase-9d/t6.6/probes/constant/constant_c`). The crate traces
nothing there, before and after Task 6.6. Task 6.6 applies the package's numeric and condition
settings to the expression, but not the trace mode, because the mode alone would print the
expression's results without the method lines.
