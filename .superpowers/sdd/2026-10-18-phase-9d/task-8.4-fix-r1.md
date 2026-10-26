# Task 8.4 fix round 1

Review: task-8.4-review.md (spec COMPLIANT, quality CHANGES REQUESTED, 0C/1I/4M). Moritz's rulings
since the report: candidate A extends Deviation 67 to every condition re-raised through
Message~result, HALT included ("Extend 67", 2026-10-26); the ::CONSTANT ~source departure keeps the
crate's answer, recorded as a Deviation and an oracle bug ("Own clause, oracle bug", 2026-10-26).

1. **I1:** the ::CONSTANT level and the level below it report the wrong `traceline` (the review's
   k4.rex, tl1.rex, tb1.rex): live frames and a trapped condition's `~stackframes` must show the
   directive's clause and the installing `::CLASS`, as the oracle does. Witness it (corpus program
   printing `traceline`, red at BASE f0164456d or at 9194fef58).
2. **M1:** correct or delete the false sentences in rust/corpus/errors/parse-errors.tsv (:12, :39).
3. **M2 (ruled):** give the controller a Deviation candidate block for the ~source answer (title,
   rule, measurement, rexxref lines, "Witnessed by" naming a crate witness you add: a unit test or
   crate-only expectation). Fix the `directive_clause_lines` comment to state the ruled answer.
4. **M3:** re-measure candidate A on the oracle (20 runs) and correct "8 of 8" to the measured race;
   record the debug-build witness run or delete the sentence.
5. **M4 and the review's queue candidate 2:** derive from rexxref what a ::CONSTANT expression
   method object does when run (`define`d and sent, `~run`, `enhanced`) and whether its level takes
   the class's guard; the oracle answers 91.999 "did not return a result" (rc 165) and 98.905 for the
   guard deadlock probe g2.rex. Fix if it is in reach (an installed MethodId, a guarded level), else
   report with a proposed owner; never a false refusal message.
6. **Deviation 67 extension:** a crate witness for the HALT shape (t8.4/naming/probes/r2.rex): the
   started report names the HALT's line, the waiter's its own (a unit test like
   a_started_messages_report_names_the_line_that_raised).

Review queue candidates 1 and 3 are queued by the controller; not this round. Same rules as the
first dispatch (perf: callgrind.sh on every program against f0164456d). Records under t8.4/r1/.
Append "Fix round 1" to task-8.4-report.md. Commit; send status, commits, one-line test summary,
concerns.
