**CLOSED 2026-10-22 by Phase 9d Task 8.2 (4acd606e9).**
**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** probe still differs: 28.3 untrapped rc 228 here, trapped 4 on the oracle.

# Error 28 from LEAVE or ITERATE is not trapped by SIGNAL ON SYNTAX

Found by the Phase 6.1 Task 6 review (M5, `lv4o`, no TRACE); pre-existing at the 6.1 base. Queued by Phase 6.1 Task 12 (2026-10-10). Loud here (rc 228), the oracle traps it.

Probe `lv4o.rex`, run from a fresh empty directory:

    nop
    signal on syntax
    do i = 1 to 2
      leave zz
    end
    exit
    syntax: say "trapped" sigl

Oracle, rc 0:

    [stdout]
    trapped 4
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 228:

    [stdout]
    (empty)
    [stderr]
         4 *-* leave zz
    Error 28 running lv4o/lv4o.rex line 4:  Invalid LEAVE or ITERATE.
    Error 28.3:  Symbol following LEAVE ("ZZ") must either match the label of a current loop or block instruction.

Suspected site: the raise of 28.x for a LEAVE or ITERATE naming no active loop (`ir/drive.rs`, `run/loops.rs`), which reports the error without routing it through the SYNTAX trap.
