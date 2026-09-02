# Task 4 review brief

Review `ccc6dbd13..8e891cd5a` (four commits: `d8e48d353`, `164d5c106`, `acb015bd4`, `8e891cd5a`).
Subject: `FORWARD` as an instruction and `DELEGATE` on `::METHOD` and `::ATTRIBUTE`, D62. The
implementer's account is `task-4-report.md`; its brief is `task-4-brief.md`.

**Why it gets a review.** It adds a new instruction to the language, a new `GeneratedKind`, a new
send path, and a rendering change (`98.937`), and it moved `FORWARD` in scope across
`instruction_owner`, `owners.rs` and `loud.rs`. Every task this phase that has had a dedicated review
has had a Critical or an Important found in it, twice a silent wrong answer that the task's own
controls did not reach.

## Work in a copy, and run things

Read-only with respect to the live tree: no edits, no cargo there, do not take its `target/` lock.

    git archive <rev> | tar -x -C <your-scratch-dir>
    ln -s /home/moritz/dev/repos/ooRexx-rust-rewrite/ootest <your-scratch-dir>/ootest
    ln -s /home/moritz/dev/repos/ooRexx-rust-rewrite/oodocs <your-scratch-dir>/oodocs
    cd <your-scratch-dir>/rust && CARGO_TARGET_DIR=<your-own-target-dir> cargo test ...

An extract has no `ootest/` or `oodocs/`, so a baseline arm already fails around 31 tests -- compare
failing **sets**, never a count against zero. After restoring an extract with `cp -a`, **`touch` the
sources before rebuilding**: restored mtimes predate the mutated build's artifacts, cargo rebuilds
nothing, and you measure the previous arm. A previous round recorded exactly that as a void reading.
`rexx-bench-suite.rs` resolves `rexx-run` through `CARGO_MANIFEST_DIR/../../target`, ignoring
`CARGO_TARGET_DIR`, so `every_blocked_axis_still_fails_on_this_crate` fails in a sandbox -- an
artifact, not a defect.

## THE SHAPE YOU MUST NOT RUN

**A `FORWARD` without `CONTINUE` that resolves back to the method it is in SIGSEGVs the oracle** --
rc 139, deterministic; the minimal form is a bare `forward` in a method. It is in
`rust/corpus/oracle-crashes.txt` with its cause. Never run it, never construct it, including while
exploring. The crate answers it rc 245 with a clean 11.1 by decision, and that divergence is licensed
on Task 9's list. **Check that licence is honoured rather than testing it against the oracle**: the
implementer measured the crate alone and never handed the oracle one, which is correct.

## What to check, in priority order

1. **The new send path, for the receivers and shapes the corpus does not cover.** `FORWARD` and
   `DELEGATE` both change how a message reaches a method. Probe past the eleven new rows: a delegate
   whose target is `.nil`, a string, a class object; a delegate naming an attribute that was never
   set; `FORWARD` out of an `::ATTRIBUTE` body; `FORWARD` inside a `DO` inside a `SELECT`; a delegate
   chain two deep; `DELEGATE` to a name the target does not answer.
2. **The `98.937` change in `acb015bd4`.** A non-continuing `FORWARD` after a valued `REPLY` now
   raises where it was silent. Check the **adjacent successes**: a bare `reply` with no value, a
   `REPLY` with no `FORWARD` after it, a continuing `FORWARD` after a valued `REPLY`. The rule is
   supposed to be pinned to the replied *value*, not to the keyword.
3. **The traceback, since `DELEGATE` was built as a `GeneratedKind` precisely because the written-out
   equivalence renders differently.** Verify the frame count claim yourself over a failing inner
   method, both directions, and check a `FORWARD` frame inside a delegated call.
4. **Every new row reddens** when the behaviour it witnesses is deleted -- the report claims ten arms
   and six option-specific witnesses. Re-run them; do not read the table.
5. **A row that passes for the wrong reason.** The spec's own D62 probe was green over a build
   reading the directive's name instead of the delegate symbol, because `'length'` and `'abcdef'` are
   both six characters. That was found in this task and the controller has corrected the spec. Check
   the **eleven new rows** for the same class of coincidence: a value whose length, name or rendering
   matches something else in the program.
6. **The scope move.** `FORWARD` moved across `instruction_owner`, `owners.rs` and `loud.rs`. Confirm
   no other instruction's ownership or refusal text moved with it.

## Not your job

The five gates and the phase gate: the controller ran the phase gate at `1aa54204b` and reads table D
5b **0 not-agree** over probes that send, table C 5b 1 (`methodsbyclass`). The two replaced probes
were verified against the oracle here. **The second sitting is owed and blocked** -- this machine's
PMU will not schedule `cycles:u` and `instructions:u` together, confirmed independently -- so do not
attempt one and do not treat its absence as a finding.

## Output

`task-4-review.md` beside this brief; that directory is the only place in the repo you may write.
Severity-tag each finding, and for every one say what you ran and what it printed. Label anything you
could not verify. **Message the controller when you finish.**
