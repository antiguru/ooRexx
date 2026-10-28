**CLOSED 2026-10-28 by Phase 9d Task 8.7b (5053fbd9b).**
**OPEN, queued 2026-10-22 by the controller; proposed 9d (Task 8.7).**

# executable_at is quadratic in activation depth

Found by the Phase 9d Task 8.3 review (task-8.3-review.md, Q3); predates the task.
`dispatch::context::executable_at` recurses one depth at a time and each level calls
`Interp::frame_at`, which walks the frame chain with `nth`. At the activation depth cap,
`corpus/lang/call_depth_cap_call.rex` does not finish in 400 s in a debug build, at efe8d3f40 and at
99fd7ae2b, so a debug corpus gate (rust/CLAUDE.md) times out on it. Fix: one walk.
