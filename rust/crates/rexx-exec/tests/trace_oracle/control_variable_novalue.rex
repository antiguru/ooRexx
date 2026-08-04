/* A Controlled loop's re-test is an EVALUATION of the control variable, so
 * it raises NOVALUE when the body has dropped it (4b Task 9, review round 1
 * re-review, NEW-1).
 *
 * The second loop drops JJ, so the next re-test raises NOVALUE, the trap
 * fires, and the program exits 0 through the handler. Reading the variable
 * with a NOVALUE-blind reader instead -- which this crate did for one
 * commit -- gives the derived name "JJ", fails 41.1 on it, and exits 215.
 *
 * Under trace r the failing re-test shows the DO clause re-echoed and NO
 * value line at all: the raise happens inside the evaluation, before the
 * >>> would be traced. A version that traced first and raised second would
 * differ here even though its stdout and rc agreed.
 *
 * The first loop is the adjacent passing case and is not decoration: its
 * body leaves II alone, so the trap must NOT fire and the loop must run to
 * completion. A witness with only the second loop would pass against an
 * implementation that raised NOVALUE on every re-test.
 *
 * SIGL is printed rather than a flag: it pins that the transfer is
 * attributed to the DO clause the re-test belongs to. */
trace r
signal on novalue name nv
do ii = 1 to 3
  nop
end
say 'ordinary loop ended at' ii
do jj = 1 to 3
  drop jj
end
say 'never'
exit
nv:
say 'handler at' sigl
exit 0
