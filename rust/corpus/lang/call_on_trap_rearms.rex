/* A trap that resumes and then raises again (4b Task 10), under trace r
 * throughout. `condition_traps.rex`'s own CALL ON block fires exactly once;
 * this program raises the *same* trapped condition twice and pins that
 * CALL ON, unlike SIGNAL ON (`condition_traps.rex`'s block 2, which
 * re-arms SYNTAX under a fresh label because the first trap is disabled
 * once it fires), does **not** disarm on firing -- the second raise below
 * reaches the identical handler with no re-arming instruction anywhere in
 * the program. A `SIGNAL` to a label (Task 6) sits between the two raises,
 * so what runs between them is genuinely a jump, not just the next
 * sequential clause -- crossing Task 6 and Task 7's own machinery rather
 * than exercising either alone.
 *
 * ZLOG accumulates one segment per event, so a missing or extra handler
 * firing changes a different, individually visible part of the string
 * rather than only a final count. Line numbers below are cited from a live
 * oracle run of this file exactly as committed, not from an earlier draft,
 * per this corpus's own README on why an edit invalidates a stale citation:
 *
 *   'S'          the starting segment.
 *   '[H54]'      the first handler firing. SIGL is the *calling* clause's
 *                own line (the `call raiser` at line 54) -- **not** `raiser`'s
 *                own RAISE line (66) -- because `RAISE ... RETURN` already
 *                unwound `raiser` like a `RETURN` before the handler is
 *                delivered, so by the time SIGL is read the interpreter has
 *                already returned to the calling activation. Measured
 *                directly against the oracle, not assumed: an earlier draft
 *                of this file stated the opposite claim.
 *   '1:V'        RESULT read back after `call raiser` resumes, proving the
 *                handler ran and returned *before* this clause, not after
 *                the whole program unwound.
 *   'K'          reached only by the `SIGNAL skip` two lines above jumping
 *                over the UNREACHABLE segment -- an implementation that let
 *                SIGNAL fall through would prepend that segment instead.
 *   '[H60]'      the *second* handler firing, SIGL again the calling
 *                clause's own line (the second `call raiser`, line 60) --
 *                from the *same* `call on` with no intervening re-arm, which
 *                is the segment that does not exist if CALL ON traps disarm
 *                like SIGNAL ON's.
 *   '2:V'        RESULT read back after the second `call raiser` resumes.
 *
 * Checked against a mutation: with `deliver_pending_trap`'s re-insertion of
 * the fired trap (run.rs, the `if let Some(trap) = removed` arm run after
 * the handler call) skipped, ZLOG reads `S[H54]1:VK2:V` -- the '[H60]'
 * segment is simply gone, because the second `raise user zx return 'V'`
 * finds no trap armed and returns 'V' to its caller unhandled, exactly the
 * "nothing traps it" arm `exec_raise` already has for an unarmed condition.
 * Every other segment is unchanged, which is what pins the missing segment
 * to re-arming specifically rather than to the raise or the SIGNAL jump.
 *
 * Determinism: no clock, no PID, no filesystem state. */
trace r
call on user zx name h
zlog = 'S'
call raiser
zlog = zlog || '1:' || result
signal skip
zlog = zlog || 'UNREACHABLE'
skip:
zlog = zlog || 'K'
call raiser
zlog = zlog || '2:' || result
say zlog
exit

raiser:
raise user zx return 'V'
return

h:
zlog = zlog || '[H' || sigl || ']'
return
