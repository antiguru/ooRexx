# Task 1 review brief

Review commit range `6fc65c487..c245dc418` on branch `plan/rust-rewrite` in the worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`. That is Task 1's commit `f7c38531c`
("Build ~new, INIT and the instance seam") plus Moritz's comment-only follow-up `c245dc418`.

Diff package (already generated): `.superpowers/sdd/2026-08-27-phase-5b/review-6fc65c487..c245dc418.diff`

## Read first

* `docs/superpowers/specs/2026-08-27-phase-5b-instances.md` -- binding spec, decisions D57-D69 + D59a
* `docs/superpowers/plans/2026-08-27-phase-5b.md` -- Task 1's section
* `.superpowers/sdd/2026-08-27-phase-5b/task-1-brief.md` -- the brief the implementer was given
* `.superpowers/sdd/2026-08-27-phase-5b/task-1-report.md` -- the implementer's report. **It is
  partial**: the agent was stopped mid-task. Treat every claim in it as unverified.
* `.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at
* `rust/CLAUDE.md`

## Two hard constraints on how you work

1. **The worktree is read-only to you.** A full debug corpus gate is running in it right now; a
   write of yours would void it. Do not edit, create, or delete any file under
   `/home/moritz/dev/repos/ooRexx-rust-rewrite/` except the one report file named below. Do not
   `git add`, `git commit`, `git checkout`, or `git stash`.
2. **Use your own cargo target directory** so you do not block on the running gate's lock:
   `export CARGO_TARGET_DIR=<your scratch dir>/review-target`. Every cargo command you run must
   have it set.

## What to check

**Spec compliance.** Does the commit do what Task 1's plan section and brief say, and does it obey
the spec's decisions? Name any decision it contradicts by number.

**Every acceptance criterion in the brief, re-measured by you.** The brief's "Done when" list has
five items. For each, run the thing yourself and quote the command beside the figure. In particular:

* the two gate-table rows (`creo.rex`, `abscla.rex`) agreeing on **both** engines under
  `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast`
* the `SELF`-clobbering program: three descriptors byte for byte against the oracle on
  `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, present in `corpus/phase-5b.txt`, and covered by
  a targeted `run_program_collect_every_alloc` case in `collect_stress.rs`
* the instance-naming witnesses in `corpus/phase-5b.txt`
* **the two controls.** The brief requires them "recorded as run" with transcripts: dropping
  `checkAbstract` makes `abscla` construct, and not sending `INIT` reddens `creo`. Since the tree
  is read-only to you, you cannot apply the mutations -- so instead report whether the report file
  actually carries each control's transcript, and judge whether the transcript shown could only
  have come from a real run. A control whose transcript is missing or is prospective (written
  before the run, never filled in) is a finding.

**The rooting hazard.** This is the task's stated use-after-free risk. Is the fix real, or does the
targeted case pass for a reason other than the hazard being closed? Look at `Interp::pool_owner`
(`run.rs`) and what roots a running send's receiver now.

**Silent wrong answers.** Every 5b task turns a loud refusal into an answer, which is when a silent
wrong answer gets introduced. Probe past the rows: send the six naming/identity methods to
instances in shapes the committed witnesses do not cover, and diff against the oracle. The oracle
binary is the system `rexx`; the crate is run via the workspace binary. Probe from a **fresh empty
directory** with absolute paths, `timeout -s KILL 20`, three descriptors read separately, never
`2>&1`. Never run anything listed in `rust/corpus/oracle-crashes.txt`.

**Quality.** `rust/CLAUDE.md`'s comment rule of 2026-08-27 governs: one sentence of overview, the
parameters, returns and panics, and properties not clear from the implementation. Also: no comment
states the size of a set; ASCII only; no em-dashes; no historical framing ("before this it was...").
Check rustdoc intra-doc links resolve to items that exist (`/bin/grep -n` the item name).

**Prose.** The commit message and the plan-file edits in this diff are part of the change. Check
their factual claims against the tree, especially universal quantifiers ("appears nowhere",
"the only", "every"). Record the exact pattern beside any negative claim you verify.

## Output

Write your findings to `.superpowers/sdd/2026-08-27-phase-5b/task-1-review.md` (this is the one
file you may create). Structure: spec compliance verdict; quality verdict (Approved / Approved with
comments / Changes requested); then findings as Critical / Important / Minor, each with file:line,
what is wrong, and the fix. Quote the command beside every figure you report. Say explicitly which
acceptance criteria you re-measured and which you could not.

Write the file first and append as you go. Message the controller when you finish.
