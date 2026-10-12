# USE ARG into a message term refuses (owner: not assigned)

Found by the Task 8 review (m5, `review-s8/p/pm8.rex`); not fixed by fix round A, by ruling.
Task 8 gave `PARSE` a message-term target (`assign_expr_target`'s `ExprKind::Message` arm,
`run.rs`); `USE ARG` is a different instruction and still refuses the same target.

Probe, from a fresh empty directory, three descriptors separately:

    o = .t~new
    call sub 'x'
    exit
    sub: use arg o~a; say o~a
    ::class t
    ::attribute a

oracle: stdout `x`, stderr empty, rc 0. Here at `7f220b983`: stdout empty, stderr
`rexx-exec: a message send is not implemented`, rc 120.
