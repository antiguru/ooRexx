**OPEN, queued 2026-10-22 by the controller; proposed 9d.**

# A trapped traceback indents the INTERPRET clause by the fragment's nesting

Found by Phase 9d Task 8.2 (task-8.2-report.md, "Found, not fixed"; identical at base). When the
failing fragment clause is nested, the TRACEBACK line of the clause running the INTERPRET takes the
fragment's indent: oracle `15 *-*   interpret arg(1)`, here `15 *-*     interpret arg(1)`
(probe docs/superpowers/records/2026-10-18-phase-9d/t8.2/probes/while-retest/i1.rex). `clause_of`
reads the live `current_value_indent` for the running activation where `FragmentLevel::indent` holds
the INTERPRET clause's.
