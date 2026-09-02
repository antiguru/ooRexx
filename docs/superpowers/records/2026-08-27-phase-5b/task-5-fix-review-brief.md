# Task 5 fix round -- review brief

Review the four fix commits `d708491a7..9c0007723` on `plan/rust-rewrite`. The task's delivery is
UNINIT (D59/D60/D69). Its first round was reviewed in `task-5-review.md`; this round answers CRIT-1,
CRIT-1b, CRIT-2, IMP-1, IMP-2 and IMP-3 from that review. The implementer's own account is
`task-5-fix-report.md`.

## Work in a copy, and actually run things

You are read-only with respect to the live tree. Do not edit it, do not run cargo in it, and do not
take its `target/` lock. Extract the revision you want and work there:

    git archive <rev> | tar -x -C <your-scratch-dir>
    ln -s /home/moritz/dev/repos/ooRexx-rust-rewrite/ootest <your-scratch-dir>/ootest
    ln -s /home/moritz/dev/repos/ooRexx-rust-rewrite/oodocs <your-scratch-dir>/oodocs
    cd <your-scratch-dir>/rust && CARGO_TARGET_DIR=<your-own-target-dir> cargo test ...

Always pass your own `CARGO_TARGET_DIR`. One known trap:
`crates/rexx-bench/src/bin/rexx-bench-suite.rs:742` resolves `rexx-run` through
`CARGO_MANIFEST_DIR/../../target`, which ignores `CARGO_TARGET_DIR`, so
`every_blocked_axis_still_fails_on_this_crate` fails in a sandbox with its own target dir. That
failure is an artifact of the sandbox, not a defect -- do not report it.

The point of the copy is that you can run mutations rather than judge transcripts. A claim in the
report that you did not re-run yourself is unverified, and should be labelled that way.

## What to check, in priority order

1. **The three fixed behaviours actually agree with the oracle.** CRIT-1 (an inherited class-side
   finalizer fires once per class that has it, so twice for `::class k inherit m`), CRIT-1b, CRIT-2
   (an allocating finalizer does not run the sweep unbounded), IMP-1 (nested collection). Construct
   your own probes; do not reuse the report's. Oracle from a **fresh empty directory** -- the
   scratchpad is on the external-routine search path and a stray `.rex` there gets called. Three
   descriptors read separately, never `2>&1`, both `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`.
   Never run anything in `rust/corpus/oracle-crashes.txt` or construct one of those shapes.
2. **IMP-2 was "a witness that cannot fail".** `uninit_class_mixin.rex` gave `K` its own
   `::method uninit class`, so deleting `inherit m` changed no output on either side -- the row was
   green over nothing. Check the *replacement* rows the same way: for each new corpus row, delete the
   behaviour it is supposed to witness and confirm the row reddens. A row that stays green under that
   deletion is the same defect again. This is the third instance of that shape in this phase, so
   check every new row, not a sample.
3. **IMP-3 was a stale-binary measurement.** The report's pre-fix quadratic figures at 16k and 64k
   were identical to its post-fix figures. Re-measure interleaved if any performance claim survives
   in this round's report, and check the binary's mtime against the source's.
4. **D61.** No check may depend on an order the oracle does not reproduce.
   `uninit_nested_collection.rex` deliberately prints only *whether* the inner finalizer ran inline,
   never *when*. Confirm that is true of every new row -- run each several times on the oracle and
   look for a row whose output is not stable.
5. **The two void runs.** The report rules a gate 5 and a release suite void because the implementer
   edited corpus files while they ran. That ruling is correct. Confirm no figure elsewhere in the
   report is silently taken from either.

## Gate status you do not need to re-establish

All five gates are green at `2d5614e63`, which contains this fix round plus two later commits. That
covers this round on a superset, so do not spend a gate-5 run on it. Report the phase-gate table
rows if you read them, but the controller is running the phase gate separately.

## Output

Write `task-5-fix-review.md` beside this brief. Severity-tag each finding. For every finding state
what you ran and what it printed. If you could not verify something, say so rather than inferring it
from the report. Message the controller when you finish -- four agents this phase have finished
silently with a written report and no message, so the message is part of the task.
