**CLOSED 2026-10-30 by Task 8.8d (03d74f029).**

# collect_stress zero-collection list drifted

Found by Phase 9d Task 8.3 (report concern 1) and its review (Q2). collect_stress's
NO_ALLOCATION_PROGRAMS had drifted before Task 8.3 (base never reached the assertion); 8.3 pruned it
to what a run observes. Open: when each dropped program began allocating (the implementer traced
`do_to_class`'s first collection to `deliver_trace_line`'s `TraceObject`), and what the +4
collections in `call_trace_nested`, `trace_output`, `trace_results` and `mutation_controlled_order`
are (docs/superpowers/records/2026-10-18-phase-9d/t8.3/probes/collections-base-head.txt). Decide
whether each is a regression or an accepted allocation.

## Task 8.8d disposition

Focused stacks and bounded history attribute the first allocations to TraceObject delivery introduced by c239cd23f and the extra four allocations to main-executable annotation tables introduced by 68e7f15d5. Current counts reproduce the previously recorded head counts, and options remains a zero-allocation control. The controller accepts both as allocations of implemented semantics, with any laziness optimization belonging to the later performance round. The temporary backtrace probe was restored exactly, and no production allocation path or collect_stress assertion changed.

See [Task 8.8d report](../2026-10-18-phase-9d/task-8.8d-report.md) for per-subpart evidence and limits.
