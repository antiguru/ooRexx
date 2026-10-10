# Translation errors print their message inserts unsubstituted

Found by the Phase 6.1 Task 6 review (M5, `l1`, `cs`). Queued by Phase 6.1 Task 12 fix round 1 (2026-10-10). Same rc and code; the message line differs. Ruling R3 keeps parse-error inserts out of 6.1 (roadmap row 3, the 2026-07-28 scope decision), so this is the work R3 defers: "a later reversal goes through a raising API that cannot be called without the inserts its message needs". A run-time error with inserts (`trace value x`, `x = 's'`) substitutes them on both engines (Task 12 review).

Probe `cs.rex`, run from a fresh empty directory:

    trace ?s
    'true'
    'false'
    'nonexistentcmd_zz 2>/dev/null'
    address command 'false'
    say 'x'
    lbl:
    call on error
    'false'
    signal on failure name ff
    'nonexistentcmd_zz 2>/dev/null'
    exit
    ff: say 'ff'
    exit

Standard input `cs.in` (its first lines; the rest are empty or more `.stderr~lineout` markers):

    .stderr~lineout('p1')

    .stderr~lineout('p2')

    .stderr~lineout('p3')

    .stderr~lineout('p4')

    .stderr~lineout('p5')

    .stderr~lineout('p6')

Oracle, rc 232:

    [stdout]
    (empty)
    [stderr]
         1 *-* trace ?s
    Error 24 running cs/cs.rex line 1:  Invalid TRACE request.
    Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "S".

This crate (`rexx-run` at `6a87cd616`), rc 232:

    [stdout]
    (empty)
    [stderr]
         1 *-* trace ?s
    Error 24 running cs/cs.rex line 1:  Invalid TRACE request.
    Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "&1".

Probe `e103.rex`, run from a fresh empty directory:

    do i = 1 to 2
    end j

Oracle, rc 246:

    [stdout]
    (empty)
    [stderr]
         2 *-* end j
    Error 10 running e103/e103.rex line 2:  Unexpected or unmatched END.
    Error 10.2:  Symbol following END ("J") must match block specification name ("I") on line 1 or be omitted.

This crate (`rexx-run` at `6a87cd616`), rc 246:

    [stdout]
    (empty)
    [stderr]
         2 *-* end j
    Error 10 running e103/e103.rex line 2:  Unexpected or unmatched END.
    Error 10.2:  Symbol following END ("&1") must match block specification name ("&2") on line &3 or be omitted.

Suspected site: the `ParseError`-to-condition path (Task 5, `59eb57f37`) and `rexx-parse`'s error constructors, which carry the code without its insert values.
