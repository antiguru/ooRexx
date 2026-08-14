/* PROCEDURE is legal only as the first instruction executed after an
 * INTERNAL call -- a label in this program's own source, reached by CALL or
 * as a function -- and a ::ROUTINE is neither, however it was reached.
 *
 * WHY THE REFUSALS ARE TRAPPED RATHER THAN FATAL. 17.1 is a SYNTAX
 * condition, measured trappable, and trapping it is what makes the caller's
 * own state readable AFTER the refusal. That is the half a fatal program
 * cannot show: the C and D blocks each assign a fresh variable once the
 * handler returns control, and an engine that admitted the PROCEDURE
 * instead has by then pushed a second frame onto an activation that already
 * owned one, so it is the CALLER's next unbound name that lands on a frame
 * which is no longer the top one. The last block leaves the same refusal
 * untrapped, because the exit status and the two-line clause echo are only
 * observable when it is.
 *
 * WHAT EACH BLOCK WOULD PRINT IF THE ENGINE WERE WRONG.
 *
 * A -- an internal label reached by CALL, PROCEDURE first. Legal. An engine
 *      that refused it prints `A trapped` and never `A caller-A`, and one
 *      that admitted it without isolating the pool prints `A callee-A`.
 *
 * B -- the same label rule for a label reached as a FUNCTION. Legal. The
 *      two routes are separate admissions and an engine can get one right
 *      and the other wrong; this block prints `B caller-B func-ok`.
 *
 * C -- a ::ROUTINE reached by CALL, PROCEDURE first. Refused, 17. An engine
 *      that admits it prints `rtn_call ran` and then `C unreachable`.
 *
 * D -- a ::ROUTINE reached as a FUNCTION, PROCEDURE first. Refused, 17,
 *      and it arrives through the expression path rather than the
 *      instruction path.
 *
 * E -- C again with the trap off: rc 239, and on stderr the failing clause
 *      echoed FIRST and the calling clause second. An engine that echoed
 *      only the failing clause, or the two the other way round, differs
 *      there and nowhere else.
 *
 * Determinism: no clock, no PID, no filesystem, and nothing printed on
 * stdout that names a path. The absolute path in block E's own error banner
 * is the oracle's own doing, and is the path the harness gives both
 * interpreters in the same run.
 */

signal on syntax name shown

say '-- A'
aa = 'caller-A'
call lbl_call
say 'A' aa

say '-- B'
bb = 'caller-B'
qq = lbl_func()
say 'B' bb qq

say '-- C'
step = 'C'
call rtn_call
say 'C unreachable'
cdone:
say 'C trapped rc' rc
cc = 'C-after'
say 'C' cc

say '-- D'
signal on syntax name shown
step = 'D'
qq = rtn_func()
say 'D unreachable'
ddone:
say 'D trapped rc' rc
dd = 'D-after'
say 'D' dd

say '-- E'
signal off syntax
call rtn_call
say 'E unreachable'
exit 0

shown:
if step = 'C' then signal cdone
signal ddone

lbl_call: procedure
aa = 'callee-A'
return

lbl_func: procedure
bb = 'callee-B'
return 'func-ok'

::routine rtn_call
procedure
say 'rtn_call ran'

::routine rtn_func
procedure
return 'rtn_func ran'
