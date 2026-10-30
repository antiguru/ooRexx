**CLOSED 2026-10-30 by Phase 9d Task 8.8e2 (ae6a6bd04).**

# Two pre-existing findings from 9b1 Task 8b

1. `.context~executable` inside a running method whose class redefined it (define) answers the
   template's flags; measure the oracle and match.
2. A finished program's RESULT stays reachable between programs on one Interp (memory/GC timing only).
Source: .superpowers/sdd/2026-10-14-phase-9b1/task-8b-report.md.

## Disposition

Executing registered Method identity and flags now survive replacement, removal, hiding, aliases, and collection through the existing activation root. The original and extended witnesses agree with the oracle on stdout, stderr, and status. The separate RESULT reachability observation retains its existing licensed collection-timing disposition. Evidence and performance handoff are recorded in `../2026-10-18-phase-9d/task-8.8e2-report.md`.
