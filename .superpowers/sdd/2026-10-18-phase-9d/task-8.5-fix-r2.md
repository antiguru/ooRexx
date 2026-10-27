# Task 8.5 fix round 2

Re-review: .superpowers/sdd/2026-10-18-phase-9d/task-8.5-rereview-r1.md (read fully). Its probes are under
.superpowers/sdd/2026-10-18-phase-9d/t85-rereview-scratch/ if still present; copy what you use to
docs/superpowers/records/2026-10-18-phase-9d/t8.5/r2/probes/ (records under t8.5/r2/). Append a "Fix round 2"
section to task-8.5-report.md.

Controller ruling (2026-10-27): every required-string position the re-review measured in Important 1 is
inside Moritz's "Rule everywhere" ruling (the referenced value's request('STRING') in every position). Keep
the behaviour; the work is witnessing and stating it.

1. Important 1: witness each position the re-review lists (comparison right operand, arithmetic right
   operand, String method arguments, DO count, INTERPRET, ADDRESS, PUSH, compound tails, SIGNAL VALUE,
   NUMERIC DIGITS, TRACE VALUE, the explicit concatenation message), red at 6e0bf5e5b where base differed,
   in a corpus program where the oracle agrees and unit tests where it does not. For each position, check
   the head answer against rexxref's rule for that position (provide.xml "Required String Values" and the
   instruction's own text; a compound tail and TRACE VALUE in particular): if rexxref gives a position a
   different rule than request('STRING') (for example the STRING method rather than request), follow
   rexxref and say so. List every position in the report and in block D's measurement.
2. Important 2: scope block D's oracle-bug sentence to the positions where the oracle takes stringValue;
   name the positions where the oracle already forwards (arithmetic, NUMERIC DIGITS, comparison). NOSTRING
   (controller ruling): the condition carries the referenced value and its STRING answer as the description,
   the oracle's answer in its forwarding positions, unless rexxref (condtra.xml NOSTRING, DESCRIPTION) says
   otherwise; change the unit test a_reference_raises_nostring_only_where_its_value_has_no_string_value to
   match and state it in D.
3. Minor 1: re-wrap corpus/lang/local_novalue_hook.rex:5 to the block's width; regenerate its sourceline
   file if lines move.

Out of scope (queued by the controller): String~equals / caselessEquals with a makeString argument; the
PROCEDURE EXPOSE alias oracle defect.

Perf: callgrind on every program against 6e0bf5e5b (rexx-base-6e0bf5e5b in
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/), +0.05% libc subtracted per program, if code changes. Tests
as before. Do not touch task-8-batches.md, progress.md, phase-4-exclusions.txt or oracle-defects/. Never go
idle on a background command; never pgrep -f. rm only literal paths. Commit, then send status.
