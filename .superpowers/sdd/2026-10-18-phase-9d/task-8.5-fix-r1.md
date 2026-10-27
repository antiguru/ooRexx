# Task 8.5 fix round 1

Review: .superpowers/sdd/2026-10-18-phase-9d/task-8.5-review.md (read fully). Its probes, if still present, are under
.superpowers/sdd/2026-10-18-phase-9d/t85-review-scratch/; copy what you use to
docs/superpowers/records/2026-10-18-phase-9d/t8.5/r1/probes/ (records under t8.5/r1/). Append a "Fix round 1"
section to task-8.5-report.md.

1. Important 1: witness the two unwitnessed NOVALUE hook paths (`a.1 = 1; say a.2`, and a reachable
   `procedure expose c. (c.1)` selector) in corpus/lang/local_novalue_hook.rex; regenerate its sourceline
   file; show each red against a mutant with the hook disabled on that path.
2. Minor 1 (taken here, not queued): EXPOSE and PROCEDURE EXPOSE of a stem create the stem variable
   (rexxref intro.xml:2562; probes p05, p08). Fix it, witness it red at 6e0bf5e5b. Keep the
   create_stem_variable doc true.
3. Minor 2: remove the set sizes at report lines 100 and 41.
4. Minor 3: candidate A cites instrc.xml:1228, :2483 and :2515 for EXPOSE and PROCEDURE EXPOSE.
5. The stopped item 2026-10-10-variable-reference-concatenation.md, now RULED by Moritz 2026-10-27 ("Rule
   everywhere, Deviation"; its first line): a variable reference's string value is its referenced value's,
   by request('STRING'), in every position: left and right operand of concatenation and abuttal, SAY,
   builtin arguments, and NOSTRING (none raised where request('STRING') answers a string). Implement it,
   witness each position (red at 6e0bf5e5b where the crate followed the oracle), close the item with the
   usual first line and inventory columns, and write Deviation candidate block D (title, rule with
   citations, ORACLE BUG cause VariableReference.cpp:258-261 against :364-370, measurement with probe paths,
   witness names). If a position cannot follow the rule without a cost outside the reference path, report it.

Queue candidate 2 (operator on a reference leaves out the Compiled method traceback line) is queued by the
controller; do not fix it.

Perf: callgrind on every program against 6e0bf5e5b (rexx-base-6e0bf5e5b in
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/), +0.05% libc subtracted per program; the concatenation
path is hot, so measure before committing to a shape. Tests as before (fmt, clippy, touched binaries, lib,
collect_stress, corpus gate, coverage and rexx-parse errors/sourceline if corpus programs change, whole
suite compared with t8.5/final/suite/). Do not touch task-8-batches.md, progress.md, phase-4-exclusions.txt
or oracle-defects/. Never go idle on a background command: poll in the foreground, and never use
`pgrep -f` (it matches its own loop). rm only literal paths. Commit, then send status.
