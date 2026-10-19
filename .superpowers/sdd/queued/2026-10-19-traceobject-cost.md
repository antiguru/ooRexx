**CLOSED, ruled by Moritz 2026-10-19: the performance of traced programs is irrelevant. There are no targets for it and no need to match the oracle.** Found by the Phase 9d Task 6.10 review. Building a TraceObject in Rexx code makes tracing several times slower than the oracle.

# The cost of a Rexx-built TraceObject

Found 2026-10-19 by the Task 6.10 review (`task-6.10-review.md`, Minor). Since Task 6.10 each trace
line builds a TraceObject by running Rexx code, as the oracle does. `TRACE R` over 10000 passes
takes 1789 ms at the Task 6.10 head, 81 ms at its parent and 513 ms on the oracle: about 221
thousand instructions per line here against the oracle's 53 thousand. About a third of the crate's
cost is a recursive class check under the hash put.
