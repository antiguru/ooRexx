# A full native pool runs a blocking command inline and can hang the process

Default mode, pre-existing (hangs the same at 0765d19ef). When every pool thread is busy, a command
or native call runs inline on the baton thread. If what it blocks on needs another activity to run
(a peer on the baton), the process hangs: the run deadline cannot fire because the baton thread is
outside the clause loop.

Probe (Phase 6.1 Task 8 review, scratchpad `p3/full.rex`): start 64 activities that each run
`address system 'sleep 3'` to fill the pool, then run `cat` on a fifo whose writer is a started
activity. Default mode hangs until killed. The native form (SENDFROMANOTHERTHREAD with pool bound 1)
hangs on base and head alike.

Owner: Phase 6's pool design (inline fallback). Either never run a blocking call inline while other
activities are runnable, or bound the inline wait and refuse loudly.
