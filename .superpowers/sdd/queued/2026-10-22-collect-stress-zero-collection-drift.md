**OPEN, queued 2026-10-22 by the controller; proposed 9d (Task 8.8).**

# collect_stress zero-collection list drifted

Found by Phase 9d Task 8.3 (report concern 1) and its review (Q2). collect_stress's
NO_ALLOCATION_PROGRAMS had drifted before Task 8.3 (base never reached the assertion); 8.3 pruned it
to what a run observes. Open: when each dropped program began allocating (the implementer traced
`do_to_class`'s first collection to `deliver_trace_line`'s `TraceObject`), and what the +4
collections in `call_trace_nested`, `trace_output`, `trace_results` and `mutation_controlled_order`
are (docs/superpowers/records/2026-10-18-phase-9d/t8.3/probes/collections-base-head.txt). Decide
whether each is a regression or an accepted allocation.
