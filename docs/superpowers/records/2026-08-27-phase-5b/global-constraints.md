# Phase 5b global constraints

`docs/superpowers/records/2026-08-17-phase-5a/global-constraints.md` binds unchanged. **Read it.**
This file records only what 5b adds or amends, and it is not a substitute for that one.

Where this file and the plan disagree, the plan wins; where the plan and
`docs/superpowers/specs/2026-08-27-phase-5b-instances.md` disagree, the spec wins.

## Amendments

* **The phase subset file is `rust/corpus/phase-5b.txt`**, not `phase-5a.txt`. A task that adds a
  corpus program adds it there, in the same commit.
* **A task that makes a gate-table row agree adds that row's probe path to `corpus/phase-5b.txt` in
  the same commit.** Gate-table probes are not corpus programs and live outside every subset file
  (`gate_table_d.rs:30`-`:34`); a phase file means "agrees with the oracle", so a probe moves there
  in the task that makes its row agree, which is also what puts it under the differential and the
  collect-on-every-allocation runs. `phase-5a.txt` carries five such entries with the reason written
  beside each.

## The invocation that reads "agrees" and "reddens"

The five gate commands do not provide one. A gate-table verdict mismatch is an exit status only
under `REXX_CORPUS_GATE` **and** only for a row whose owning phase is closing or already closed
(`gate_tables/mod.rs`'s `verdict_is_gated`). `CLOSED_PHASES` holds `"5a"` until Task 10, so **every
5b row can be red and all five gates exit zero.** Every "agrees" and "reddens" is read under

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

quoted beside the figure like any other command.

## The five gate commands

From `rust/`, at the end of every task, each status read unpiped:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --release --workspace
REXX_CORPUS_GATE=1 cargo test --release --workspace
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast
```

`cmd | tail` in an `&&` chain reports tail's status, not the command's. Read every status unpiped.
Use `--no-fail-fast` where the command takes it: `cargo test` stops at the first failure otherwise,
so "nothing else caught it" would be unmeasured.

## Three rules this phase adds

1. **Every "what is already there" paragraph in a brief is a claim to re-measure, not a premise.**
   Four of them were wrong on the previous plan, all in the direction of making the task look
   smaller. Correct the plan file where you find it wrong, not your brief and not your report.
2. **A witness must be run against a control that makes it fail.** Gate table D's two `DELEGATE`
   rows are green today over a mechanism that is not implemented, measured. Two more would be green
   over a *wrong* implementation rather than a missing one. Each task names its control; **record
   the control as run**, with its transcript, or the acceptance is not met.
3. **No check may depend on an ordering the oracle does not reproduce** (D61, D68). Two instance
   `UNINIT`s at termination, and `~completed` sampled before `~result`, are both racy on the oracle,
   measured. If a probe needs two finalizers, force each with `drop` then `call gc 'force'`.

## Probing

* Differential correctness is stdout, stderr and exit status byte for byte against the oracle, on
  `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`. Three descriptors read separately, never
  `2>&1`.
* Run probes **from a fresh empty directory** with absolute paths. A leftover `.rex` file on the
  search path gets called as an external routine.
* The crate bootstraps on every run, so bound crate probes with `timeout -s KILL 20`.
* **Never run a program listed in `rust/corpus/oracle-crashes.txt`**, and never construct one of
  those shapes.
* A silent wrong answer is the worst defect in this phase and no gate sees one. **Every 5b task
  turns a loud refusal into an answer**, which is exactly when one is introduced. Probe past the row.

## Comments

ASCII only, no em-dashes, no historical framing, and **no comment states the size of a set** (true
counts included; measurements keep their numbers). `rust/CLAUDE.md`'s comment rule of 2026-08-27
governs: one sentence of overview, the parameters, returns and panics, and properties not clear from
the implementation. Prefer no comment to a paragraph, and assert a load-bearing property in a test
instead of describing it.

"Record" means the report and the ledger, not a comment in the source. A measurement justifying the
design **as it now stands** earns a comment; an account of what the code did **before** is history
and goes elsewhere.

A rustdoc intra-doc link to an item that does not exist is not a compile error and no gate sees it.
`/bin/grep -n` the item name before writing the link.

## Git

Never `git checkout -- <path>` on a file you have edited; `cp` to the scratchpad and restore from
the copy. Never `rm` with a star glob. Never a bare `git stash`. Commit with `git commit -F <file>`,
name paths explicitly, never amend. Never `git add -A`.

## Preferences

* **Prefer deleting to rewriting.** Every false sentence the previous two plans shipped arrived as
  an added justification whose argument was right.
* Prefer correctness over performance.
* No `unsafe`. The workspace lint is `unsafe_code = "deny"`, not `forbid`, so the lint does not make
  it impossible; `rust/CLAUDE.md`'s exception process does, and it is Moritz's decision per site.
  Stop and say so rather than reach for it.

## Read-only trees

The C++ oracle source is `/home/moritz/dev/repos/ooRexx/interpreter/`. `ootest/` and `oodocs/` are
**inside this worktree**, at the repository root, and are git-ignored working copies:
`oodocs/rexxpg` and `oodocs/rexxref` at r13198, `ootest/` at r13178, verified 2026-08-21 with
`svn info`. Check rather than assume. Nothing writes to any of the three.

## Reporting and waiting

* **Write your report file first and append to it as you go.** An agent that is killed or goes idle
  with a partial file on disk has kept its findings; one holding a perfect unwritten report has not.
* **Do not poll a long-running command on a short interval.** Background it and let its exit
  re-invoke you, or wait in roughly hour-long stretches. The corpus gate on this project runs long,
  and a short poll costs a full turn plus a context re-read for nothing.
* **Message the controller when you finish.** Writing the report file alone does not reach it.
