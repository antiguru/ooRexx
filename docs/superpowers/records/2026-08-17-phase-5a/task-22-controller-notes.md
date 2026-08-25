# Task 22 controller notes

The brief says "Re-measure on a current build -- `3ee6e4bf7` changed when an `EXTERNAL` directive is
refused." Done, by the controller, against the shipped oracle, 2026-08-24. **The old plan's numbers
still hold.**

Missing entry point, with a prologue that would print if binding were lazy:

    say "prolog ran"
    ::method m external "LIBRARY REXX no_such_entry_point_xyz"

Oracle: **rc 166, stdout EMPTY**, stderr

           2 *-* ::method m external "LIBRARY REXX no_such_entry_point_xyz"
    Error 90 running <path> line 2:  External name not found.
    Error 90.998:  Unable to find external method "no_such_entry_point_xyz".

Note the frame names **line 2**, the directive's own line, not line 1.

Real entry point, same shape:

    say "prolog ran"
    ::method m external "LIBRARY REXX file_separator"

Oracle: **rc 0**, stdout `prolog ran`.

The pair is the eager-bind differential and the brief's control at once: the second program is what
the first becomes if binding is made lazy. `file_separator` is one of the two entries the brief pulls
forward from Phase 7, so this pair is only writable after this task registers it.

## Crate state at dispatch, measured

Both engines, both probes above:

    rexx-exec: ::METHOD EXTERNAL is not implemented (Phase 7)    rc 120

**One message covers all three `EXTERNAL` forms and names Phase 7 for each.** The brief requires this
task to say which form it moves and which it does not, so that message has to split: the
`LIBRARY REXX` method form becomes implemented here, while `::ROUTINE EXTERNAL` naming a real shared
library stays Phase 7's and keeps its refusal. A single message that still says Phase 7 after this
task, or one that stops saying it for a form this task does not build, are both wrong -- and the
brief warns that the old plan's Task 8 fix rounds produced a wrong answer in exactly this
neighbourhood.

Check the refusal's own wording against what the task actually builds before closing, and make the
form that stays refused a corpus row of its own if the oracle's answer permits one.
