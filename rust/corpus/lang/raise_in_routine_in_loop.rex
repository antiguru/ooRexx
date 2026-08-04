/* A raise inside a routine inside a loop (4b Task 10), under trace r
 * throughout. Combines Task 5 (PROCEDURE) and Task 7 (RAISE, CALL ON) inside
 * 4a's own loop machinery -- the shape the plan calls out by name, because
 * `Stem` and arithmetic each had a witness on their own and `Stem` as an
 * arithmetic operand had none (`a. = 5; say a. + 1` aborted the process).
 * Nothing below pins a single construct in isolation; each pins how two of
 * them interact.
 *
 * `looper` is called from every pass of a three-iteration loop and raises a
 * trapped USER condition on exactly the middle pass. What that pins, per
 * pass:
 *
 *   pass 1 (zi=1)  looper returns 10 normally -- the adjacent case a raise
 *                  never reaches, so a wrong RETURN path would already show
 *                  here.
 *   pass 2 (zi=2)  `raise user ping return 'TRAPPED'` ends looper exactly as
 *                  RETURN 'TRAPPED' would (looper's own `return zn * 10` on
 *                  the next line never runs), settles RESULT to 'TRAPPED' at
 *                  the *calling* clause (`call looper zi`), and only then
 *                  runs the CALL ON handler -- SIGL inside the handler reads
 *                  looper's own RAISE line, not the calling clause's.
 *   pass 3 (zi=3)  looper returns 30 normally again, which is what proves
 *                  the loop's own control state survived the trapped pass:
 *                  an implementation that left the loop or the calling
 *                  activation in the wrong frame after delivering the trap
 *                  would compute the wrong `zi`, call the wrong routine
 *                  clauses, or not reach a third pass at all.
 *
 * ZACCUM accumulates one segment per pass plus the handler's own segment, so
 * a wrong RESULT, a wrong SIGL, a skipped pass or an extra handler firing
 * each change a different, individually visible part of the string rather
 * than only the pass count.
 *
 * Checked against a mutation: with the CALL ON arm of `exec_raise` (run.rs)
 * changed to deliver `Flow::Return(None)` instead of `Flow::Return(result)`,
 * ZACCUM's middle segment reads `RESULT:2` (RESULT's own derived name, an
 * unset variable's reading) instead of `TRAPPED:2`, and every other segment
 * is unchanged -- the failure is exactly the value RAISE...RETURN carries
 * across the call boundary, not the surrounding loop or trap machinery.
 *
 * Determinism: no clock, no PID, no filesystem state. */
trace r
call on user ping name handler
zaccum = ''
do zi = 1 to 3
  call looper zi
  zaccum = zaccum || result || ':' || zi || ' '
end
say 'accum:' zaccum
exit

looper: procedure expose zaccum
use arg zn
if zn = 2 then
  raise user ping return 'TRAPPED'
return zn * 10

handler:
zaccum = zaccum || '[H@' || sigl || ']'
return
