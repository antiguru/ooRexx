/* PUSH and QUEUE (4b Task 8, I15), under TRACE R throughout.                */
/*                                                                           */
/* WHAT THIS PROGRAM CAN AND CANNOT PIN. Nothing that reads the queue back   */
/* -- PULL, PARSE PULL, QUEUED() -- is implemented before 4c, so no program  */
/* in this subset can differentially witness which end of the queue a line  */
/* landed on or in what order. What it CAN pin, and what TRACE R is here    */
/* for: PUSH/QUEUE evaluate their own expression, render it to string form  */
/* and trace the result exactly like SAY does (the oracle's own             */
/* RexxInstructionQueue::execute shares SAY's evaluateStringExpression), and */
/* a bare PUSH/QUEUE with no expression traces a null string rather than     */
/* being skipped or erroring. Without TRACE R this program would produce    */
/* empty stdout, empty stderr, rc 0 whether PUSH/QUEUE store anything at     */
/* all -- a test that cannot fail -- so the >>> lines below are the whole    */
/* point, not decoration. The interleaved storage order itself is pinned by */
/* queue.rs's own unit tests (rust/crates/rexx-exec/src/queue.rs), the only */
/* place it can be, per that task's own report.                             */
/*                                                                           */
/* Ends with a bare EXIT, not `exit 0`: condition_traps.rex's own header     */
/* already recorded why -- an EXIT with a value has a pre-existing,          */
/* unrelated gap in this crate's own EXIT arm (its >>> line is not traced),  */
/* and a program exercising a different construct should not diverge for    */
/* that reason.                                                             */

trace r
a = 'hello'
push a
queue 'world'
push a 'there'
queue
push
exit
