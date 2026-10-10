# A subclass of Package builds a plain Package

Found by Phase 6.1 scout A (2026-10-07, section 4, `b8_package_*`). Queued by Phase 6.1 Task 12 (2026-10-10). Loud on ours (97.1), the oracle answers.

Probe `b3_subclass_package.rex`, run from a fresh empty directory:

    say .k~new('p', 'say 1')~go
    ::class k subclass package
    ::method go
      expose x; x = 3; return x

Oracle, rc 0:

    [stdout]
    1
    3
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 159:

    [stdout]
    1
    [stderr]
         1 *-* say .k~new('p', 'say 1')~go
    Error 97 running b3_subclass_package.rex line 1:  Object method not found.
    Error 97.1:  Object "a Package" does not understand message "GO".

Suspected site: `Package~new` (`dispatch.rs` `native_package_new`, `dispatch/package.rs` `package_new`) answers an instance of Package itself, not of the receiver class.
