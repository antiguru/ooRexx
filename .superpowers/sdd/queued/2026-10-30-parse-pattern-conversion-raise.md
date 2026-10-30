**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# PARSE pattern rendering drops a raising conversion

`parse var x (q) y` where q's DEFAULTNAME raises on its first send: the oracle traps 98.900, this crate runs on and prints `returned` (the pattern's rendering takes no conversion that can raise). I1's rule: a required string value's conversion that raises, raises in the clause. Probe docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/c2/onceparse.rex. Pre-existing at d6720f350 (8.7c re-review r2, task-8.7c-rereview-r2.md, R2-1).

## Task 8.8f evidence

String and caseless PARSE patterns use required-string conversion after operand evaluation. Raising and NOSTRING controls fail in the PARSE clause. Numeric positional triggers retain their conversion boundary. The ordinary and collection-stress witnesses pass. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
