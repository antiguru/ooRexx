/* TRACE L (4b Task 9, review round 1 F8): every executed LABEL clause is
 * echoed with the ordinary *-* prefix, and nothing else is.
 *
 * The oracle's RexxInstructionLabel::execute traces through traceLabel and
 * nothing else, and traceLabel's gate is tracingLabels() -- so the three
 * ways a label is reached all echo, and no other clause does. All three are
 * here on purpose: fallen through (FELLTHROUGH), a CALL target (SUB, which
 * also shows the callee's own +2 indent), and a SIGNAL target (THERE).
 *
 * The silent half is the point and is not decoration. The DO, the SELECT
 * and the IF between the labels produce no line at all: an implementation
 * that treated L as "echo everything" would add a dozen, and one that
 * treated it as TraceMode::OFF -- which this crate did until Task 9's
 * review -- produces none of the three label lines.
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
signal there
say 'never'
there:
say 'after the signal'
exit
sub:
procedure
say 'in the callee'
return
