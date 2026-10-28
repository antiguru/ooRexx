**OPEN, queued 2026-10-28 by the controller; proposed 9d (Task 8.7, first).**

# TRACE I over a reference assignment panics: the traced value is not rooted

Found by the Phase 9d Task 8.5 narrow re-review (task-8.5-rereview-r6.md, Out of scope); predates the task
(rexx-base-6e0bf5e5b panics the same). `trace i`, then `q = >y` 300000 times
(docs/superpowers/records/2026-10-18-phase-9d/queued-probes/trace-ref-rooting/tr4.rex) panics
`dispatch.rs:1831:9: a live value` (rc 101) in the normal release binary; one such assignment (tr2.rex)
panics under the collector stress mode. Backtrace: assign_evaluated -> result_text -> string_value_text ->
to_text -> not_in_arena: the fresh reference is not rooted while its trace text is rendered. `trace r`
does not panic. Root it; witness under collect_stress.
