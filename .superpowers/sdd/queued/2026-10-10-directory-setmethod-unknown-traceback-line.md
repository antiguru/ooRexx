**CLOSED 2026-10-22 by Phase 9d Task 8.2 (a52fa561b).**
**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** both probes still print an extra Compiled method "UNKNOWN" traceback line here.

# A Directory~setMethod entry adds a `Compiled method "UNKNOWN"` traceback line

Found by Phase 6.1 Task 4 (report concern 3); Task 4 fix round 1 fixed the name (`running GO`), not the extra line. Queued by Phase 6.1 Task 12 (2026-10-10). Same rc and message; stderr has one extra line here.

Probe `t4e.rex`, run from a fresh empty directory:

    d = .directory~new
    d~setMethod('go', 'return 1/0')
    say d~go

Oracle, rc 214:

    [stdout]
    (empty)
    [stderr]
         1 *-* return 1/0
         3 *-* say d~go
    Error 42 running GO line 1:  Arithmetic overflow/underflow.
    Error 42.3:  Arithmetic overflow; divisor must not be zero.

This crate (`rexx-run` at `6a87cd616`), rc 214:

    [stdout]
    (empty)
    [stderr]
         1 *-* return 1/0
           *-* Compiled method "UNKNOWN" with scope "Directory".
         3 *-* say d~go
    Error 42 running GO line 1:  Arithmetic overflow/underflow.
    Error 42.3:  Arithmetic overflow; divisor must not be zero.

Suspected site: `dispatch/hash.rs` `run_stored_method`, which runs the entry method under the Directory's UNKNOWN method; the oracle's traceback has no frame for it.

2026-10-12 (Phase 9a Task 8 review): the Stem sibling. `x = .stem~new('S.'); say x~nosuch` prints an extra `Compiled method "UNKNOWN" with scope "Stem".` traceback line the oracle does not (rc 159 both), for a base or subclass Stem receiver.
