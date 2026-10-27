# Task 8.5 fix round 3

Re-review: .superpowers/sdd/2026-10-18-phase-9d/task-8.5-rereview-r2.md (APPROVED with minors; read fully).
Its probes are under .superpowers/sdd/2026-10-18-phase-9d/t85-rereview2-scratch/ if present; copy what you use
to docs/superpowers/records/2026-10-18-phase-9d/t8.5/r3/probes/. Append a "Fix round 3" section to
task-8.5-report.md.

1. Queue candidate 2, taken here because 074ada1da made it (base overflowed the stack, head loops without a
   HALT check and ignores SIGTERM): a cyclic variable reference (`r = >r; say length(r)`, and the
   two-variable cycle) in any string conversion. Controller ruling: the crate must terminate, loudly, and stay
   interruptible. The oracle hangs (an oracle defect, listed by the controller), so there is no oracle answer
   to match and no Deviation: pick the error from rexxref (a resource or recursion error, or the shape the
   crate already gives for other unbounded recursion; say which and why), witness both cycle shapes in a unit
   test under a timeout, and add both programs to rust/corpus/oracle-crashes.txt in its format (a hang). Keep
   the non-cyclic path at its current cost.
2. Minor 4: witness conversion_subject's loop: a reference to a reference (`rr = >r`) in
   a_reference_raises_nostring_only_where_its_value_has_no_string_value and in the corpus witness in a
   position the oracle forwards (`1 + rr`). Show the while->if mutant red.
3. Minor 3: correct the "unchanged" sentence in the report; witness the ::OPTIONS NOSTRING SYNTAX message for a
   reference to a value with only a STRING method (head `Object "nstr"`), and add that position to block D.
4. Minors 1 and 2: correct the line-count sentence; move String method arguments out of block D's NOSTRING
   clause (the oracle raises 88.909 there, no NOSTRING).

Out of scope (queued by the controller): a plain stem whose default has no string value in an arithmetic
operand (queue candidate 1).

Perf: callgrind on every program against 6e0bf5e5b (rexx-base-6e0bf5e5b in
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/), +0.05% libc subtracted per program; record the run of the
final code commit. Tests as before. Do not touch task-8-batches.md, progress.md, phase-4-exclusions.txt or
oracle-defects/. Never go idle on a background command; never pgrep -f. rm only literal paths. Commit, then
send status.
