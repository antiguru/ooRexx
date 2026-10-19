**OPEN: found by the Phase 9d Task 6.10 review, not yet ruled.** A crate process looping inside a trace delivery ignores SIGTERM.

# SIGTERM is not acted on inside a trace delivery

Found 2026-10-19 by the Task 6.10 review (`task-6.10-review.md`, Important 2's note). A trace
destination whose LINEOUT is `do forever; end` keeps the crate running after `timeout` sends
SIGTERM, at the Task 6.10 head and at its parent. The halt that SIGTERM requests is likely
delivered only at a clause boundary the delivery never reaches. One consequence: `memcap 2G
timeout 20` does not bound such a probe, so its runs need `timeout -s KILL`.
