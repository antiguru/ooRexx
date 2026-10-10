# `trace c` omits the clause echo of an ADDRESS instruction's command

Found by the Phase 6.1 Task 6 review (M5, `cc2`); pre-existing. Queued by Phase 6.1 Task 12 (2026-10-10). Same stdout and rc; one trace line is missing here.

Probe `cc2.rex`, run from a fresh empty directory:

    trace c
    'true'
    address command 'false'
    say 1

Oracle, rc 0:

    [stdout]
    1
    [stderr]
         2 *-* 'true'
           >>>   "true"
         3 *-* address command 'false'
           >>>   "false"
           +++   "RC(1)"

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    1
    [stderr]
         2 *-* 'true'
           >>>   "true"
           >>>   "false"
           +++   "RC(1)"

Suspected site: the command trace for `ADDRESS env command` (`command.rs` / the ADDRESS instruction in `run.rs`), which traces the `>>>` value without the clause.
