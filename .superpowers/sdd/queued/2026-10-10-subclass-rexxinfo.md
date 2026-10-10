# `::class k subclass rexxinfo` reports 98.909 where the oracle reports 99.949

Found by Phase 6.1 scout A (2026-10-07, section 4, `b8_rexxinfo_*`). Queued by Phase 6.1 Task 12 (2026-10-10). Both refuse the program; the error number, message and rc differ.

Probe `b4_subclass_rexxinfo.rex`, run from a fresh empty directory:

    say .k~new~go
    ::class k subclass rexxinfo
    ::method go
      expose x; x = 3; return x

Oracle, rc 157:

    [stdout]
    (empty)
    [stderr]
         2 *-* ::class k subclass rexxinfo
    Error 99 running b4_subclass_rexxinfo.rex line 2:  Translation error.
    Error 99.949:  "REXXINFO" is not a valid class.

This crate (`rexx-run` at `6a87cd616`), rc 158:

    [stdout]
    (empty)
    [stderr]
         2 *-* ::class k subclass rexxinfo
    Error 98 running b4_subclass_rexxinfo.rex line 2:  Execution error.
    Error 98.909:  Class "REXXINFO" not found.

Suspected site: the `::CLASS` superclass resolution in `install.rs` (`class_not_found`): `.RexxInfo` is an instance, which the oracle rejects as "not a valid class" at translation time.
