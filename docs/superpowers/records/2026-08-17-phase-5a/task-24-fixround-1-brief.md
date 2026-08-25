# Task 24, fix round 1 -- the last round of the plan

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-24-review.md`. **Your central conclusion is
correct and the reviewer could not find a sixth row.** It re-derived the set of five three
independent ways without taking a number from your report, cross-checked every cell of your section
3 tables, and confirmed the gating arm is the switch the brief believes it is. The decision not to
commit the flip is sound and the plan records enough to perform it later.

Spec compliance is CHANGES REQUIRED only because the brief's first clause cannot be met by editing
these commits -- 5a is open, and that is the finding, not a defect in your work.

## M1. Must fix: the control build you say is still needed was already run, at this source

Section 12 says separating the fixed cost from the per-pass one "needs the control build", and
section 14 says this task "did not run" it. **It exists, committed, in the file section 12 is
reading.** I verified: the last 156 rows of `bench-baselines/phase-5a-arms.tsv` are task
`23-attempt-2-control-no-bootstrap` at commit `cfffc7899`, and
`git diff --stat cfffc7899..HEAD -- 'rust/crates/*/src' 'rust/Cargo.toml' 'rust/Cargo.lock'` is
empty, so it is a control build of this tree's source.

It already separates them. `strings`, `per_pass`, `instructions:u`, ir arm:

    pin                          5,369.53
    head, bootstrap suppressed   5,406.48
    head as shipped              7,136.86

So the movement is **per-pass, arriving with the resident library** -- not the bootstrap's fixed
148M. That is also arithmetically forced: 34% of 16.1e9 instructions cannot be a 148M fixed cost.

Three things to fix, and the third matters most:

1. **The negative claim is false** and one `awk` over the cited file would have falsified it. This is
   the shape the plan has already paid for repeatedly: a negative is exactly as wide as the pattern
   that looked for it, and here nothing looked at all.
2. "The last sitting recorded in the file is task `23-attempt-2`" is wrong about the file. The last
   rows are the control's.
3. **Task 23 handed a decision forward and your boundary drops it.** Its Concern 1 says the per-pass
   regression needs a decision that was not its to take -- a question about the collector's cost
   model with a large resident set. That belongs in section 10's boundary and section 16's concerns,
   as a decided, measured, open cost handed to 5b and 5c. Carrying it instead as an unresolved
   *measurement* question is a weaker and wrong version of something already answered.

## m1, m2, m3

* **m1** -- section 8's EXEMPT counts are each one too many. `/bin/grep -ac 'ExemptRow {'` also
  matches `struct ExemptRow {` at `assertions.rs:313`. It is 13 now and 35 at the pin
  (`/bin/grep -ac '^    ExemptRow {'`). The derived "22 retired" survives because the off-by-one
  cancels; the two absolute figures do not. **This is the most frequent wrong number in this
  project** -- a `grep -c` over code counting a definition alongside its uses.
* **m2** -- "under an eighth of the oracle's median" is wrong: an eighth of 7.437 is 0.9296 and the
  pin is 0.970, so it is under a *seventh*. The parenthetical already gives the right ratio. Note
  that your own last commit exists to fix this exact shape one paragraph away.
* **m3** -- your catch-all audit runs one direction only. You checked the row filed *into* 5a by
  omission; the row filed *out* of it is `("::ROUTINE", "EXTERNAL") => Some("7")`, which covers both
  spellings because the probe picks the shared-library form. Measured:
  `::routine r external 'LIBRARY REXX Filespec'` is oracle rc 168 running the routine against crate
  rc 120 -- the same `LIBRARY REXX` spelling D37 moved into 5a for `::METHOD`, and the exact
  argument you use to keep `::ATTRIBUTE EXTERNAL` in 5a. Applied symmetrically it is at least
  arguable, and **unlike `::ATTRIBUTE EXTERNAL`, no row of either table can see it**. Phase 7 does
  own it and `gate_table_d.rs`'s doc records it, so nothing is concealed -- but the brief asked you
  to hand 5b and 5c a stated boundary, and this is a divergence no phase gate row currently sees.
  Name it there.

The three observations need no change. Leave the abridged terminal blocks and the historical commit
message alone.

## How to close

This should be documentation only. Prove it the way the last two rounds did -- `.text` section hash
across a forced rebuild -- rather than taking a sitting. All five gates at the commit you report,
`--no-fail-fast`, and quote the command that printed each figure beside the figure.

Prefer deleting to rewriting: where a sentence closes by striking a clause, strike it.
