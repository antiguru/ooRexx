**CLOSED 2026-10-29 by Phase 9d Task 8.7c (e620097f0).**
**OPEN, queued 2026-10-28 by the controller; proposed 9d (Task 8.7c).**

# Primitives that reach Rexx code with no live frame

Found by Phase 9d Task 8.7b (report Concerns 2, confirmed by task-8.7b-review.md). Task 8.7b gave a primitive frame to RUN, SORT, HASITEM, REQUEST, PUT, []=, UNKNOWN and NEW when they call Rexx code; others still push none: List and Table `hasItem`, `Array~index`, `removeItem`, and the like. A StackFrame list or traceback read from the Rexx code they call lacks the primitive's frame the oracle shows.
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7b/review-probes/primitives-without-frame-f3.rex. Find the set from the oracle's native methods that send messages, not from this list.
