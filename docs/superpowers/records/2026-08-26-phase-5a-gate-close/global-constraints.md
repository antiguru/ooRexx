## Global constraints

`docs/superpowers/records/2026-08-17-phase-5a/` has the constraints this project runs under and they
bind unchanged. The ones this plan leans on hardest:

* **Differential correctness.** stdout, stderr and exit status byte for byte against the oracle, on
  `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`. Three descriptors read separately, never
  `2>&1`. Probes from a fresh empty directory with absolute paths. The crate bootstraps on every run,
  so bound crate probes with `timeout -s KILL 20`.
* **A silent wrong answer is the worst defect here** and no gate sees it. Every task below turns a
  loud refusal into an answer, which is exactly when one is introduced.
* **The five gate commands** at the end of every task, `--no-fail-fast`, each status read unpiped,
  and **the command that printed a figure quoted beside the figure**.
* Comments: ASCII only, no em-dashes, no historical framing, and no comment states the size of a set.
* Never `git checkout -- <path>` on a file you have edited; `cp` to the scratchpad and restore from
  the copy. Never `rm` with a star glob. Commit with `-F`, name paths explicitly, never amend.
* **Prefer deleting to rewriting.** Every false sentence the previous plan shipped arrived as an
  added justification whose argument was right.

**Measured at `8b4a7459d`, by the controller, before this plan was written.** Each row's *first*
missing piece is what its refusal names; what follows behind it was probed separately and is listed
per task, so no task is sized from an assumption.

---


## Waiting costs tokens

When waiting on something long-running -- a gate run, a release build, a corpus sweep -- wait in
stretches of about **an hour**, not ten minutes. Every wake costs a full turn and re-reads context.
The debug corpus gate alone runs about 64 minutes, so a 10-minute poll buys six wasted wakes before
the first one that could find anything.

A backgrounded command re-invokes you when it exits, so a long wait is a **fallback against a hang**,
not the mechanism. And read each gate's status from its own `.rc` file, never from a completion
notification: a wrapper's exit status is its last command's, so a trailing `echo` reports success
over any failure inside it.

## The gates can run outside the main worktree

`scratchpad/controller-gates.sh <sha> <logdir>` gates a **committed SHA** in two dedicated worktrees,
two lanes in parallel. Validated green at `edc9e69c3` in 66 min against 84 serial. It writes
`g{1,2,4,5}.rc`, matching `.log` files and `elapsed.seconds` into the logdir, refuses to run if either
gate worktree is dirty, and cannot gate uncommitted work.
