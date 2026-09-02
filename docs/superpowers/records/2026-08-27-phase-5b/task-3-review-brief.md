# Task 3 review brief

Review `f65694c8b..d4fac6707` on `plan/rust-rewrite` (three commits: `4e613c349`, `8bcf6375e`,
`d4fac6707`). Subject: per-object methods, D66 and D67. The implementer's account is
`task-3-report.md`; its brief is `task-3-brief.md`.

**Why this one gets a full review.** It is the highest-risk change of the phase so far. It altered
**the send path every program uses**, added a boxed per-object dictionary to `Body::Instance` behind a
monotone flag, implemented both the D67 scope-pool rule and the D66 restricted-private check, and
made a compiled method source into a program of its own. A defect here is reachable from every corpus
row rather than from a new one.

## Work in a copy, and run things

Read-only with respect to the live tree: do not edit it, do not run cargo in it, do not take its
`target/` lock. Extract and work in your own copy:

    git archive <rev> | tar -x -C <your-scratch-dir>
    ln -s /home/moritz/dev/repos/ooRexx-rust-rewrite/ootest <your-scratch-dir>/ootest
    ln -s /home/moritz/dev/repos/ooRexx-rust-rewrite/oodocs <your-scratch-dir>/oodocs
    cd <your-scratch-dir>/rust && CARGO_TARGET_DIR=<your-own-target-dir> cargo test ...

A `git archive` extract has **no `ootest/` or `oodocs/`**, so a baseline arm already fails around 31
tests. Compare the failing **set** between arms, never a count against zero.
`crates/rexx-bench/src/bin/rexx-bench-suite.rs` resolves `rexx-run` through
`CARGO_MANIFEST_DIR/../../target`, which ignores `CARGO_TARGET_DIR`, so
`every_blocked_axis_still_fails_on_this_crate` fails in a sandbox. That is an artifact, not a defect;
do not report it.

## What to check, in priority order

1. **The send path, for every receiver kind that is not an instance.** The per-object dictionary is
   searched ahead of the class behaviour and gated on a monotone flag. Check what a program that
   never sends `SETMETHOD` now pays and, more importantly, that no *other* receiver kind (string,
   array, stem, class object, `.nil`) reaches or skips the new branch wrongly. Probe past the corpus.
2. **D67, the scope pools.** `FLOAT` is one pool **per object**, shared across that object's `FLOAT`
   one-offs and separate from the class's; `OBJECT` shares the class's. One `FLOAT` method cannot
   tell "per object" from "per method" -- confirm the committed rows use two, and construct the third
   case nobody wrote: two objects of the same class each with `FLOAT` one-offs, checking the pools do
   not bleed between *objects*.
3. **D66, the two refusals.** `97.2 ... cannot accept private message` at rc 159 with **no** method
   frame, versus `98.991 Method SETMETHOD may only be invoked from a method of the same object or one
   of its classes.` at rc 158 **with** a `Compiled method "SETMETHOD" with scope "Object".` frame.
   **The frame is the discriminator, not the number.** Verify both, and verify `send`/`sendWith`
   answer from a program context.
4. **Every new corpus row reddens** when the behaviour it witnesses is deleted. Not a sample -- the
   report claims ten arms, one per row; re-run them. This shape has shipped four times in this phase.
5. **A row that passes for the wrong reason** is the harder failure, found in Task 5's second fix
   round. The report claims two rows were checked against the nearest wrong explanation. Check the
   other eight the same way: for each, what is the nearest wrong explanation for its output, and does
   any control separate them?
6. **The compiled-method-as-a-program change.** A method compiled from source now reports under its
   own name (`Error 42 running MM line 1:`, `parse source` giving `LINUX METHOD MM`), and
   `FailureSite::Sourceless` was renamed `Named` and widened. Check the widening did not change what
   an *existing* sourceless site reports -- that is a rendering path many 5a rows pin.
7. **The unreclaimed synthetic program.** One per compiled source, never reclaimed. Measure whether a
   loop of `setMethod` calls grows memory without bound, and say what the shape costs rather than
   whether it is licensed.

## What not to spend time on

The five gates and the phase gate: the controller re-ran the phase gate itself at `d4fac6707` and
reads `usesem` `agree`, table C 5b 1 not-agree, table D 5b 0. The sitting is recorded and inside
0.09% on every axis. The gate-5 wall-clock flake the report mentions is fixed in `76a669c24`.

## Output

`task-3-review.md` beside this brief -- that directory is the only place in the repo you may write.
Severity-tag each finding; for every one, state what you ran and what it printed. If you could not
verify something, say so rather than inferring it from the report. **Message the controller when you
finish** -- several agents this phase have written a report and stopped without messaging.
