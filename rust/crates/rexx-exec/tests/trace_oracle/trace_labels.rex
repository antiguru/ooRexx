/* TRACE L (4b Task 9, review round 1 F8): every EXECUTED label clause is
 * echoed with the ordinary *-* prefix, and nothing else is.
 *
 * THE RULE IS ABOUT EXECUTION, NOT ABOUT ROUTES, and stating it the other
 * way round is what this header got wrong. The oracle's
 * RexxInstructionLabel::execute traces through traceLabel and nothing else,
 * and traceLabel's gate is tracingLabels() -- so a label echoes whenever
 * control reaches it, however it got there. That one C++ site is the whole
 * condition; nothing anywhere enumerates the ways control can arrive.
 *
 * An earlier version of this header said "the three ways a label is reached
 * all echo" and listed them. A reviewer found a fourth (an internal
 * FUNCTION call, ZF below) by attacking exactly that sentence. The routes
 * below are therefore examples that have been probed, never a closed set:
 *
 *   FELLTHROUGH  fallen into from the clause above
 *   SUB          a CALL target, which also shows the callee's own +2 indent
 *   ZF           an internal function call in an expression -- the route
 *                that was missing, and the reason this file was reopened
 *   THERE        a SIGNAL target
 *
 * The silent half is the point and is not decoration. The DO, the SELECT
 * and the IF between the labels produce no line at all: an implementation
 * that treated L as "echo everything" would add a dozen, and one that
 * treated it as TraceMode::OFF -- which this crate did until Task 9's
 * review -- produces none of the label lines.
 *
 * NOT here: TRACE ?L. The oracle emits two further +++ banner lines for the
 * interactive prefix that this crate does not (a KNOWN GAP in
 * phase-4-exclusions.txt), so a program with one in it would diverge for a
 * reason that has nothing to do with labels.
 *
 * Determinism: no clock, no PID, no filesystem state. */
trace l
say 'before any label'
fellthrough:
do ii = 1 to 2
  nop
end
select
  when 1 = 1 then nop
  otherwise nop
end
if 1 = 1 then say 'if body ran'
call sub
say 'function call returns' zf()
signal there
say 'never'
there:
say 'after the signal'
exit
sub:
procedure
say 'in the callee'
return
zf:
procedure
return 'ZF-VALUE'
