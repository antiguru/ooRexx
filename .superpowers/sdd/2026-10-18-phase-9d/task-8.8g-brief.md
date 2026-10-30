# Task 8.8g: Judge the remaining M6 mutant

## Scope

Implement only the preferred seeded REPLY attribution experiment in [the source proof](task-8.8-m6-timer-proof.md). The proof supplies the exact Rexx program, FIFO simulation options, pin key and expected assertions. Its results are predictions, so establish the baseline before judging the mutant. A reachable attribution catch judges M6 without asserting a forbidden language scheduling cadence or implementing the optional controlled-timer experiment.

## Implementation and evidence

* Add two feature-gated tests in the existing scheduler mutant test module, using the existing runner, deadline, simulation report and pinning counters.
* Use `sim:1,fifo,order=fifo,floor=1000000`, require actual contention below that floor, and verify the exact successful output and empty stderr.
* The REPLY case requires zero sorting-key deferred slices and zero pinned yields at baseline. Also require zero total deferrals and balanced pinning, so an unexpected key cannot hide the request.
* The negative twin changes only terminal `reply 1` to `return 1`. Run it as a separate named test, requiring zero deferrals at baseline and under the mutant.
* Commit the tests before baseline verification. If either baseline disagrees, inspect the discrepancy before changing an assertion or expanding the implementation.
* Commit the exact deletion of only `switch_to`'s SLICE clear before the mutant check. Require the primary attribution assertion to fail and the RETURN twin to pass, with nonzero selected-test counts.
* Restore by inverse edit, commit restoration, confirm the production file equals the pre-mutation version, and rerun the two named tests. Retain exact diffs, commands, statuses and counter outputs.

## Constraints and disposition

Read the common dispatch, global constraints and batch header before execution. Add no production hooks, timer framework, unsafe, dependency or fabricated scheduler state. Preserve the production clear and all surrounding comments, and use the existing bounded foreground and commit-before-verification workflow. Root retains the final broad gates and performance obligation, while a test-only slice adds no new release runtime path.

Correct the historical seeded-unreachability claim in the queued scheduler record and controller ledger using the source proof's REPLY producer. Close the six-mutant queue only if the compiled M6 catch succeeds, linking the existing five catches separately. Preserve earlier reports as historical records and append a correction rather than rewriting their measured evidence. Report any failed premise concretely without claiming a timer equivalence proof or a completed catch.

## Handoff

Write `task-8.8g-report.md` beside this brief and durable evidence under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m6/`. Run the named pinning-feature tests, formatting, affected all-target clippy and any source-table check justified by moved lines. Stage only assigned files, preserve other workers' outputs and the user's archive, and return source/evidence commits with the exact M6 disposition. Do not spawn subagents.
