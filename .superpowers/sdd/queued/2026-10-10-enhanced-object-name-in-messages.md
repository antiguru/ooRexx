# An enhanced object is named `an Object` in error messages

Found by Phase 6.1 Task 4 (report concern 3). Queued by Phase 6.1 Task 12 (2026-10-10). Same rc and code; the 97.1 message differs.

Probe `t4b.rex`, run from a fresh empty directory:

    d = .stringtable~new; d['A'] = 'return 1'
    x = .object~enhanced(d)
    say x~a
    x~nosuch

Oracle, rc 159:

    [stdout]
    1
    [stderr]
         4 *-* x~nosuch
    Error 97 running t4b/t4b.rex line 4:  Object method not found.
    Error 97.1:  Object "enhanced Object" does not understand message "NOSUCH".

This crate (`rexx-run` at `6a87cd616`), rc 159:

    [stdout]
    1
    [stderr]
         4 *-* x~nosuch
    Error 97 running t4b/t4b.rex line 4:  Object method not found.
    Error 97.1:  Object "an Object" does not understand message "NOSUCH".

Suspected site: the default object name of a `Class~enhanced` instance (`dispatch/class_protocol.rs` `native_enhanced` and the object-name rendering): the oracle prefixes `enhanced`.
