# A source that does not parse, compiled by Method~new, Routine~new, NewMethod, NewRoutine: the compiled source's frame (owner: not assigned)

Found by the Phase 8 surface plan's Task 8. Task 8's fix round A made the compile path raise the
parse error's SYNTAX condition with POSITION the line within the source
(`dispatch/class_protocol.rs`, `source_syntax`), so METHOD's TESTNEWMETHOD02 and
TESTNEWROUTINE02 pass and are no longer recorded. Witness: `corpus/lang/compiled_source_syntax.rex`
(trapped: code, RC, POSITION, condition('E')).

What is left is recorded in phase-4-exclusions.txt ("A SOURCE THAT DOES NOT PARSE, COMPILED BY
..."): the compiled source's own frame. Probe, from a fresh directory:

    signal on syntax
    r = .Routine~new('Testing', 'return arg(1')
    exit
    syntax: do l over condition("O")~traceback; say l; end

oracle prints `     1 *-* return arg(1`, the `Compiled method "NEW"` line and the caller's
clause; here the caller's clause alone. Untrapped, the oracle's report is `Error 36 running
Testing line 1` under that first line; here the caller's path and line. `ParseError` carries the
failing clause's start byte and no end, so there is no clause text to echo. The message's `&1`
placeholders are roadmap row 3's licence and not part of this item.
