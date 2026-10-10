# RAISE NOVALUE RETURN into a caller reaches its CALL ON ANY

Found by Phase 6.1 scout A (2026-10-07, section 4, `c_condition_d_raise7`); recorded as a known gap in `docs/superpowers/plans/phase-4-exclusions.txt` ("A CONDITION CALL ON CANNOT TRAP, RAISED WITH RETURN INTO A CALLER") at Task 3. Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both. `raise lostdigits return` is the same divergence (the exclusions row).

Probe `b2_raise_novalue_call_on_any.rex`, run from a fresh empty directory:

    call on any name h
    call sub
    say 'back'
    exit
    sub: raise novalue return
    h: say 'D=['condition('D')']' condition('C'); return

Oracle, rc 0:

    [stdout]
    back
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    D=[] NOVALUE
    back
    [stderr]
    (empty)

Suspected site: `run/condition.rs`, the ANY fallback in the caller-side trap lookup (`trap_at_depth`), which does not exclude the conditions CALL ON cannot trap when the condition arrives from a RAISE ... RETURN.
