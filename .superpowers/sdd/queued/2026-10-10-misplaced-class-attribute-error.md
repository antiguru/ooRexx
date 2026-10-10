# `::attribute foo class` with a body and no `::CLASS` reports 99.937 where the oracle reports 99.905

Found by Phase 6.1 Task 5 I2 (ATTRIBUTE TESTMISPLACEDCLASSMETHOD in `whole_groups`); older than Phase 6.1. Queued by Phase 6.1 Task 12 (2026-10-10). Same rc; error number, message and traceback line differ.

Probe `attrcls.rex`, run from a fresh empty directory:

    say 'x'
    ::attribute foo class
      return 1

Oracle, rc 157:

    [stdout]
    (empty)
    [stderr]
         2 *-* ::attribute foo class
    Error 99 running attrcls/attrcls.rex line 2:  Translation error.
    Error 99.905:  CLASS keyword on ::METHOD directive requires a matching ::CLASS directive.

This crate (`rexx-run` at `6a87cd616`), rc 157:

    [stdout]
    (empty)
    [stderr]
         3 *-* return 1
    Error 99 running attrcls/attrcls.rex line 3:  Translation error.
    Error 99.937:  Attribute methods without a SET or GET designation cannot have a method body.

Suspected site: the directive parser's order of checks for `::ATTRIBUTE` (`rexx-parse`): the oracle rejects CLASS without a matching `::CLASS` before it looks at the body.
