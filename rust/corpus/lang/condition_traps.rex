/* Condition traps, RAISE and NOVALUE (4b Task 7), under trace r throughout.
 *
 * Every block asserts a value a handler SET, never that the program reached
 * the end: a trap test that checks only the exit code is satisfied by a
 * program that never raised at all. ZWITNESS accumulates one segment per
 * block and the whole string is printed at the end, so a block that silently
 * did not run removes its segment rather than leaving the output unchanged.
 * No segment is a variable's derived name or any prefix of one -- an unset
 * Rexx variable reads as its own uppercased spelling, so a flag left unset
 * renders as plausible-looking data, and the values here are chosen so that
 * a failure prints ZWITNESS or ZUNSET_PROBE instead of something readable.
 *
 * What each block pins, and the wrong answer it would print:
 *
 *   1. SIGNAL ON SYNTAX. The trap fires on a raise from an ordinary
 *      expression and SIGL is the RAISING clause's line, not the SIGNAL ON
 *      clause's and not the handler's. An implementation that never traps
 *      gets the fatal 42.3 report instead of any output at all.
 *
 *   2. The trap is DISABLED once it fires. Block 2 re-arms SYNTAX under a
 *      different label and reaches that second label; an implementation that
 *      left the first trap armed would re-enter TRAP_SYNTAX and loop.
 *
 *   3. SIGNAL ON NOVALUE, on a simple variable and on a compound. A bare
 *      stem is deliberately NOT here: measured, `say zstem.` does not raise
 *      NOVALUE where `say zstem.1` does.
 *
 *   4. CALL ON USER, whose handler runs at the CLAUSE BOUNDARY rather than
 *      at the raise. The `call raiser` clause settles RESULT from the
 *      routine's own RAISE ... RETURN value first, and only then does the
 *      handler append its segment -- so the ordering of `/RAISER-RETURNED`
 *      and `/USER-CALLED` in the output is what distinguishes "resumed
 *      after the clause" from "transferred at the raise".
 *
 *   5. SIGNAL OFF. The second `say 1/0` is NOT trapped, which is what ends
 *      the program: the file's last three lines of stderr are the ordinary
 *      fatal report, and an implementation that ignored SIGNAL OFF would
 *      print the handler's output and exit 0 instead of 214.
 *
 * NOT here: `exit <value>` anywhere under `trace r`. That was originally a
 * refusal -- the clause's own `>>>` value line was missing from this crate
 * (a gap in the EXIT arm, unrelated to conditions) and a program containing
 * one would have diverged for a reason that had nothing to do with what it
 * is witnessing. **That gap is closed as of 4b Task 9**, and this program
 * simply has no use for one: it ends on an untrapped raise, which is what
 * its last block is for. The witness for the EXIT value line is
 * tests/trace_oracle/exit_value.rex.
 */
trace r
signal on syntax name trap_syntax
zwitness = 'START'
say 1/0
say 'unreachable-after-block-1'

trap_syntax:
zwitness = zwitness'/SYNTAX-AT-'sigl
signal on syntax name trap_raise
raise syntax 40.4 array ('ZORKROUTINE', 7)
say 'unreachable-after-block-2'

trap_raise:
zwitness = zwitness'/RAISED-AT-'sigl
signal on novalue name trap_novalue
say zunset_probe
say 'unreachable-after-block-3'

trap_novalue:
zwitness = zwitness'/NOVALUE-AT-'sigl
signal on novalue name trap_tail
say zunset_stem.1
say 'unreachable-after-block-4'

trap_tail:
zwitness = zwitness'/TAIL-AT-'sigl
call on user marker name trap_user
call raiser
zwitness = zwitness'/'result
say 'accumulated:' zwitness
signal block_five

raiser:
raise user marker return 'RAISER-RETURNED'

trap_user:
zwitness = zwitness'/USER-CALLED-AT-'sigl
return

block_five:
signal on syntax name trap_never
signal off syntax
say 'final:' zwitness
say 2/0
say 'unreachable-after-block-5'

trap_never:
say 'THIS-HANDLER-MUST-NOT-RUN'
