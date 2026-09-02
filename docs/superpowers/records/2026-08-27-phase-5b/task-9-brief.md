## Task 9: the instance-side reading of 5a's limits

**Goal.** Every limit 5a pinned with a class as the receiver is re-measured with an instance.

**BASE:** `b0faac922`, the commit named in your dispatch. Tree clean. Read
`.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at,
`rust/CLAUDE.md`, and the plan's Task 9 section
(`docs/superpowers/plans/2026-08-27-phase-5b.md:810`-`:993`), which is long and is the actual
specification of this task.

**This is an audit, not a build.** 5a could only send messages to class objects, so every refusal
text, error number and traceback line it pinned was measured on one receiver kind and *assumed* for
the other. A limit measured on one receiver and assumed for the other is a claim, not a measurement.

**5b's gate rows are all green as of `dcd8b468d`** -- table C 5b 6 rows 0 not-agree, table D 5b 2
rows 0, phase gate exit 0, verified by the controller re-running it rather than reading a report. So
nothing is blocking Task 10 but this task. That is not licence to make this one small.

---

## The enumeration is the acceptance, and picking it is where this task can go wrong

The plan says: derive the enumeration from the tree, do not choose it; pick one candidate, quote the
command that produced it, and commit the derived list. It also says why -- "a walk over three sites
would be complete, recorded, and would name its enumeration."

**So the failure mode is picking the enumeration that minimises the walk.** Sizes, from the
controller, with the command beside each so you can check them. These count *names appearing in*
`crates/rexx-exec/src`, which is **not** the same as the plan's "reachable from", and reachability is
the stronger notion the plan asks for -- so treat these as an order of magnitude, not as the answer:

```
/bin/grep -rhao "Loud::[a-z_][a-z_0-9]*" crates/rexx-exec/src | sort -u | wc -l      -> 42
/bin/grep -ac "^\s*pub(crate) fn " crates/rexx-exec/src/error.rs                     -> 122
/bin/grep -rhao "Raised::[a-z_][a-z_0-9]*" crates/rexx-exec/src | sort -u | wc -l    -> 120
```

Justify your pick against **the goal** -- limits 5a pinned with a class receiver -- and not against
its size. `Loud::` is the not-implemented refusal surface; `error.rs`'s constructors and `Raised::`
are the Rexx-level raise surface. Those are different populations and the goal names both kinds of
site ("refusal text, error number and traceback line"). If one enumeration does not cover the goal,
say so and take the union rather than the convenient half. If you take the smallest, the report must
say what the other two contain that it does not.

**Derive the list with a command and commit the derived artifact**, not a list you typed. Three
review rounds on this project once missed 104 bad rows that running the rule found in one pass.

---

## Done when

* The derived list is committed, with the command that produced it quoted in the report.
* The walk covers it, and each divergence found is **either closed or given a named owner**.
* **A test asserts the walk covers the derived list**, so a site added later is red rather than
  unwalked. Prose saying "all of them" is what this project has shipped wrong repeatedly; an
  assertion over the set is the fix that holds. Run a control against that test: add a site to the
  derived list without walking it and confirm the test reddens.
* The plan's existing divergence list is worked through. Several entries are **"recorded, not to be
  built"** -- the oracle stack overflows, the self-forward -- and those stay recorded; do not build
  toward matching an oracle crash.
* Five gates each 0, statuses read unpiped; phase gate reported with both tables' counts. At BASE
  that is table C 5b **0** not-agree and table D 5b **0**, so **any** red row is yours.

## Two things on that list that are the controller's, not yours

* **The self-forward with a preceding `REPLY` does not terminate on this crate**, and no bound has
  been invented. Whether to take a cap is Moritz's and the controller's; it is unruled. Do not
  invent one, and do not construct the shape against the oracle -- every member is in
  `corpus/oracle-crashes.txt`.
* **The stem tail order** under `FORWARD ARGUMENTS` is a representation change to `Body::Stem`, the
  hottest structure in the interpreter, and owes a performance sitting. If you judge it in scope, say
  what it would cost before doing it, not after.

## Rules

* Correct the plan or spec where you find it wrong. The plan's Task 8 discriminator was wrong and was
  corrected in place by that task; expect more of the same here.
* Never run a program in `rust/corpus/oracle-crashes.txt` and never construct one of those shapes.
  **Read that file first** -- this task's own list references several of its entries.
* No `unsafe`. Stop and say so rather than reach for it.
* Oracle probes run from a fresh empty directory. Three descriptors read separately, never `2>&1`.
  Both engines, always.
* **A gate run does not survive the turn that starts it.** Write each gate's status to a file as it
  finishes, arm a waiter that exits on the process vanishing as well as on completion, and read the
  statuses in the same turn you commit. Task 7's run sat green and uncommitted for five hours.
* Take a tree hash before the first gate and after the last, covering untracked files' **bytes** --
  `git status --porcelain` names an untracked path without reading it and `git diff` skips untracked
  files entirely. Task 8's instrument does this correctly; copy it rather than reinventing it.
* **Audit every `interpreter/` citation you add before committing.** Task 7 found seven wrong,
  Task 8 three plus a pre-existing one naming a corpus program that does not exist.
* Commit with `git commit -F <file>`, naming paths explicitly. Never `git add -A`, never amend, never
  a bare `git stash`, never `rm` with a star glob, never `git checkout --` on a file you have edited.
  Re-read `git diff --cached --stat` and confirm `Cargo.lock` is absent.
* Write your report to `.superpowers/sdd/2026-08-27-phase-5b/task-9-report.md` (git-ignored). Say
  plainly what you did not do -- on an audit that is the most load-bearing section of the report.
