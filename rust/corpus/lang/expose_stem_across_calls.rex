/* An exposed stem mutated by a callee, across three separate calls (4b Task
 * 10), under trace r throughout. `call_procedure_expose.rex`'s own `stem`
 * block already pins that a single PROCEDURE EXPOSE aliases the caller's
 * stem rather than copying it; what that program cannot show is whether the
 * alias is re-established correctly on *every* call or whether it happens
 * to work once and then drifts -- `bump` below is called three times, once
 * per loop pass, and each call both reads the accumulator a previous call
 * left behind and writes a new tail.
 *
 * ST. defaults to 0 (`st. = 0`), so ST.SUM starts at a known value rather
 * than an unset compound's own derived name. Each call sets ST.<n> to n*n
 * and adds it into ST.SUM; the third call's addition pushes ST.SUM past 10,
 * which raises a trapped USER condition whose handler -- running in the
 * *caller's* activation, not `bump`'s -- writes ST.FLAG through the same
 * exposed table. What the final SAY line pins:
 *
 *   ST.1, ST.2, ST.3  three tails, each written by a *different* call to the
 *                     same routine, all visible from the caller: an
 *                     implementation that exposes a fresh copy per call
 *                     (rather than the one alias) would still get any one
 *                     of these right in isolation but not the running total
 *                     below, which depends on every earlier call's write
 *                     still being there.
 *   ST.SUM            the running total, which is wrong if any single call's
 *                     exposure did not persist into the next call.
 *   ST.FLAG           set by the handler, itself running through the same
 *                     alias `bump` used -- proof the exposed table is one
 *                     table shared by caller, callee and handler alike, not
 *                     three separate views of it.
 *
 * Checked against a mutation: with the `alias_slot` call inside
 * `exec_procedure` (run.rs) skipped -- the exposed table left un-aliased --
 * the *first* call already fails, before either the loop or the trap gets a
 * chance to matter: `st. = 0`'s default lives only in the caller's own
 * table, so the unaliased callee's ST.SUM reads its own derived name
 * ("ST.SUM") on `st.sum = st.sum + st.zk2` and Error 41.1 ("Nonnumeric
 * value") aborts the program at rc 215 with no stdout at all, rather than
 * the accumulated line below.
 *
 * Determinism: no clock, no PID, no filesystem state. */
trace r
st. = 0
call on user overflow name handler
do zk = 1 to 3
  call bump zk
end
say 'after loop:' st.1 st.2 st.3 st.sum st.flag
exit

bump: procedure expose st.
use arg zk2
st.zk2 = zk2 * zk2
st.sum = st.sum + st.zk2
if st.sum > 10 then
  raise user overflow return 'OVER'
return

handler:
st.flag = 'H@'sigl
return
