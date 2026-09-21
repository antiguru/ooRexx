# Defect: `every_unstable_row_is_really_unstable` can redden by coincidence

Found 2026-09-21 when a release gate went rc 101 on a commit that could not
have caused it, then rc 0 on a re-run of the same binary, with the same test
green in the debug sweep of the same commit and 20 isolated re-runs clean.

**A flaky gate is worse than a missing one.** Every gate reading in this project
is evidence, and a test that reddens by chance teaches the reader to re-run
rather than to look.

## The mechanism, read from the code rather than inferred from the flake

`crates/rexx-exec/tests/support/arity.rs:569`, `stable_rows_marked_unstable`:

    let first = run(&mut oracle_command(&oracle, &path, &dir));
    let second = run(&mut oracle_command(&oracle, &path, &dir));
    if first == second {
        stable.push(...)
    }

The row's marker says the oracle does not reproduce the value. The check is two
runs and an equality. **Two samples cannot establish that a value varies**, and
for a value drawn from a finite space they can coincide.

The row that fired is `Object~hashCode`, which is identity-derived. Project
memory already records that identity keys in this interpreter do not reproduce
across runs and do not even match themselves. That is what makes the row
`UNSTABLE`, and it is also what makes two runs agreeing a matter of chance
rather than a contradiction.

The failure is one-sided: the test can only fail **falsely**, by concluding a row
is stable when it is not. It cannot pass falsely, so nothing has been let through.
That is why this is a defect in an instrument rather than in the tree.

## What the work is

Not "re-run it" and not "delete the row". Options, in the order they look
promising, none chosen:

* **More samples, and the right assertion.** A row is wrongly marked only if
  *every* run agrees; one disagreement anywhere is proof of instability. Cheap,
  since these are oracle runs and the loop already has the machinery.
* **Assert the property instead of the sample.** An identity-derived value is
  unstable because of what it is, not because two runs happened to differ.
* **A per-row sample count**, for rows whose value space is small enough that
  coincidence is likely.

Whoever takes it should say what the residual false-failure rate is under the
chosen fix, because "we made it rarer" and "we made it correct" are different
outcomes and only one of them restores the gate.

## Do not lose the good property while fixing it

The test exists because a marker that is wrong in the other direction hides a
real comparison. Keep the check that a row marked `UNSTABLE` is not silently
comparable; only the evidence standard is wrong.
