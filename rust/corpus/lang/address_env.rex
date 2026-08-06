/* ADDRESS naming an environment: the constant form, the two computed forms,
   the bare toggle, and the 250-byte name limit.

   WHAT THIS PROGRAM CANNOT SEE, STATED FIRST BECAUSE IT SHAPES EVERY BLOCK.
   The environment an ADDRESS instruction sets has exactly five readers, and
   the enumeration is the C++'s rather than a guess: `settings.currentAddress`
   is read by `RexxActivation::toggleAddress`, by `RexxActivation::setAddress`,
   by `CommandInstruction`'s dispatch, by the ADDRESS() builtin, and by the
   default environment handed to an external .rex called as a routine. Of
   those, the two that a Rexx program can observe are ADDRESS() and issuing a
   command. This corpus cannot call either. So no program here can print the
   environment, and none of the blocks below asserts the SWAP -- what a bare
   ADDRESS actually does is pinned by crates/rexx-exec/src/run.rs's own unit
   tests, which read the state directly, and it is owed a corpus witness by
   whichever task makes ADDRESS() answer.

   WHAT IS LEFT IS STILL DIFFERENTIAL, AND IT IS THE TRACE AND THE ERROR.

   WHICH WRONG ANSWER EACH BLOCK PRINTS.

   A: under TRACE R, the clause echo for every form, and the value line for
   exactly the two computed ones. ADDRESS VALUE nm and ADDRESS (nm) each trace
   `>>>` for the string they computed; the constant form, the literal form and
   the bare toggle trace no value line at all, because they evaluate nothing.
   An engine that traced a value for a form that evaluates nothing adds a line
   after each of this block's constant, literal and bare clauses; one that
   evaluated a computed form without tracing it drops that form's own `>>>`.

   B: the same forms under TRACE I. The computed form's own value line stays
   `>>>` here -- it is NOT the `>=>`/`>>>` choice a PARSE target's value line
   is (lang/parse_sources.rex has that one). What TRACE I adds is the `>V>`
   for reading NM, ahead of the `>>>`. An engine that made the value line
   mode-dependent prints `>=>` and fails here while passing A.

   C: the name limit. 250 bytes is accepted and 251 raises Error 29.1, which
   SIGNAL ON SYNTAX traps, so the program continues and prints RC. An engine
   with no limit prints `C2 not reached`; one that raised at 250 too never
   prints `C1 accepted 250`; one that raised the wrong error prints a
   different RC. The trapped clause is the computed form because the constant
   form would need a 251-byte literal in the source.

   Determinism: no clock, no PID, no filesystem, no path. The traced clauses
   are this file's own text and the error is raised and trapped in-program. */

nm = 'mIxEd'

/* A */
trace r
address envA
address
address value nm
address (nm)
address 'LiTeRaL'
address
trace off
say 'A done'

/* B */
trace i
address envB
address value nm
address
trace off
say 'B done'

/* C */
signal on syntax name toolong
ok = copies('z', 250)
address value ok
say 'C1 accepted' length(ok)
bad = copies('z', 251)
address value bad
say 'C2 not reached'
exit 0

toolong:
say 'C2 trapped rc' rc
exit 0
