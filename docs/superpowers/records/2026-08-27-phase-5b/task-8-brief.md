## Task 8: multidimensional `Array`, and `methodsbyclass` (D63)

**Goal.** `corpus/gate-tables/concepts/methodsbyclass.rex` agrees, with the equivalence arm and the
asymmetric second cell added.

**BASE:** `1f57c3d02`, the commit named in your dispatch. Tree clean. Read
`.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at,
`rust/CLAUDE.md`, the plan's Task 8 section
(`docs/superpowers/plans/2026-08-27-phase-5b.md:743`-`:790`), and D63 in the spec.

**This is the last red 5b row in either gate table.** Table C 5b is 6 rows, 1 not-agree, and that one
is yours. Table D 5b is 0. When this row agrees, Task 10's flip has nothing left blocking it but
Task 9.

---

## Verified by the controller against the tree at BASE. Re-measure all of it

* **The row is rc 120 today**, and the refusal is `Array~NEW`:
  `rexx-exec: method "NEW" of class "Array" is not implemented (Phase 5)`, stdout empty.
* **`Body::Array` is `Array(Vec<Option<ObjRef>>)` at `crates/rexx-core/src/body.rs:147`** -- flat,
  as the plan says. Note the crate: the representation change lands in **`rexx-core`**, not
  `rexx-exec`, so it crosses a crate boundary and `rexx-exec` is a consumer of it.
* **`gate_table_c.rs`'s `methodsbyclass` row declares `oracle_lines: 4`** (`:502`-`:510`), with the
  committed control "route `matrix[2, 3] = 0` to a single-index `[]=`, so the element read back is
  not the one written".
* **The plan's discriminator program answers exactly what the plan states.** Measured, oracle,
  standard wrapper, fresh empty directory, rc 0 and empty stderr:

```rexx
m = .array~new(2, 3)
m[1, 2] = 'a12'
m[2, 1] = 'a21'
m~"[]="('b23', 2, 3)
say 'x' m[1, 2] m[2, 1] m[2, 3]
say 'y' m~dimension m~dimension(1) m~dimension(2)
say 'z' m~size m~items
```
```
x a12 a21 b23
y 2 2 3
z 6 3
```

**Do not budget this task from a grep.** `Body` is a public enum in another crate; the number of
sites that name `Body::Array` is not the number that need changing, because `From`, `Deref` and
pattern ergonomics absorb most of them. Make the change and let the compiler enumerate the breakage.
A grep will over-count and a plan built on it will be wrong in the direction that makes the task look
larger.

---

## The acceptance problem is the task, and the plan is right about it

The committed probe writes one cell and reads the same cell, so **any injective index mapping passes**
-- row-major, column-major, transposed, off by a constant -- because the write and the read miss
together. Adding the message form `matrix~"[]="(0, 2, 3)` writes and reads that same cell again: that
is an arity check, not a mapping check. The committed control catches only the collapse to one
subscript. So D63 would close over an acceptance that cannot see the indexing at all.

The fix is the asymmetric second cell above: `m[1, 2]` and `m[2, 1]` are distinguishable only under
the right mapping, and `~dimension(1)`/`~dimension(2)` pin the shape where a bare `~dimension`
answering `2` does not.

**`oracle_lines` moves with the probe.** The row declares four lines, checked as
`OracleShape::Exactly`, and a structural failure is red in **every** mode -- so an added output line
that leaves that literal at 4 is red for a reason that has nothing to do with this task. Change both
in the same commit.

---

## Done when

* The row agrees on both engines, three descriptors, with the equivalence arm **and** the asymmetric
  second cell in the probe, and `oracle_lines` updated to match.
* **Both controls recorded as run, with transcripts.** The committed one (route `matrix[2, 3] = 0` to
  a single-index `[]=`) reddens the row; and **transposing the index mapping also reddens it** --
  which it cannot do against today's probe, and being able to is the point of this task's probe
  change. If a transposed mapping still passes, the probe is not finished.
* The probe's path is added to `corpus/phase-5b.txt` in the same commit if making the row agree puts
  it under the differential -- check the constraints file's rule on that rather than assuming either
  way.
* Five gates each 0, statuses read unpiped, phase gate reported with both tables' counts. At BASE
  that is table C 5b **1** not-agree (yours) and table D 5b **0**; after this task table C 5b should
  be **0**.

## Rules

* Correct the plan or spec where you find it wrong; do not correct your brief or your report around
  it.
* Never run a program in `rust/corpus/oracle-crashes.txt` and never construct one of those shapes.
  **Read that file first -- entry 6 is directly in your subject**: an array with exactly one item
  whose first slot is empty, as the sole subscript of `~at`, is a deterministic oracle SIGSEGV. You
  are building multidimensional subscripts and will be tempted by exactly that family. The clean
  neighbours are recorded there; use them.
* No `unsafe`. Stop and say so rather than reach for it.
* Oracle probes run from a fresh empty directory. Three descriptors read separately, never `2>&1`.
  Both engines, always.
* **A gate run does not survive the turn that starts it.** Write each gate's status to a file as it
  finishes, arm a waiter that exits on the process vanishing as well as on completion, and read the
  statuses in the same turn you commit. Task 7's run sat green and uncommitted for five hours because
  nothing woke it; Task 6 lost three runs outright.
* Take a tree hash before the first gate and after the last -- and **cover untracked files' bytes, or
  say that you did not**. `git status --porcelain` names an untracked path without reading it and
  `git diff` skips untracked files entirely; three instruments across Tasks 6 and 7 had descriptions
  wider than their coverage, and this is the same trap.
* **Audit your citations before committing.** Task 7 checked every `interpreter/` line number it had
  added and found seven wrong, one of them cited in three places. Nothing in the gates can see a
  wrong line number.
* Commit with `git commit -F <file>`, naming paths explicitly. Never `git add -A`, never amend, never
  a bare `git stash`, never `rm` with a star glob, never `git checkout --` on a file you have edited.
  Re-read `git diff --cached --stat` and confirm `Cargo.lock` is absent.
* Write your report to `.superpowers/sdd/2026-08-27-phase-5b/task-8-report.md` (git-ignored, like
  every sibling report). Say plainly what you did not do.
