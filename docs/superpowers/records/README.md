# SDD execution records

Every plan under `docs/superpowers/plans/` that was executed by the
subagent-driven-development skill produced a working directory of ledgers, briefs, reports and
reviews. That directory is git-ignored scratch. This tree is its committed record, one
subdirectory per plan, named after the plan file's basename.

**Why these are kept.** A commit says what changed. It does not say what a task was asked to do,
what a reviewer found, which findings were fixed and which were ruled out of scope, or what a
measurement read before a change was accepted. The ledger and the review reports are the only
place that reasoning exists, and the skill's own last step deletes them. The record is worth more
than the disk it costs: this project has already re-derived a conclusion it had reached and thrown
away.

**Append-only, like `plans/phase-4f-record.md`.** An entry that turned out wrong is corrected by a
later document saying so, not by editing the one that was wrong. Do not tidy these files. They are
dated records of what was known at the time, including the parts that were mistaken, and their
value is that they are not retouched.

## What is here

| kind | what it is |
|---|---|
| `progress.md` | the plan's ledger: one entry per task, the rulings made during execution, and the commits each task produced |
| `task-*-brief.md` | the requirements extracted from the plan and handed to an implementer |
| `task-*-report.md` | what the implementer did, what it measured, and what it flagged |
| `task-*-review.md`, `*-rereview*.md` | the task review and the scoped re-reviews of each fix round |
| `*-gate.md`, `final-review-*.md` | the whole-plan reviews and gate reports |
| everything else | measurements, spike patches, triage notes and control data a task produced |

## What is deliberately not here

Files named `review-<base>..<head>.diff` -- the review packages the skill generates for a
reviewer to read. They are excluded because **each one is exactly reconstructible from the
repository it was taken from**: the filename carries the commit range, both endpoints of every
excluded file were verified reachable at the time of the copy, and

```
git log --oneline <base>..<head>
git diff --stat <base>..<head>
git diff -U10 <base>..<head>
```

reproduces one. They are the bulk of the workspace by size and carry no information the branch
does not already hold.

Diffs whose names do **not** carry a commit range are kept, because nothing in the repository says
what they span.

## The live workspace

`.superpowers/sdd/` stays git-ignored. It is where a running plan writes, and where the skill's
resume logic looks for a ledger. Copy a plan's directory here when the plan closes; do not point
the skill at this tree.
