## Review the Phase 5c spec and plan

**BASE:** `414b71b22`. The worktree is `/home/moritz/dev/repos/ooRexx-rust-rewrite`. **Read-only to
you**: report findings, fix nothing, commit nothing.

## What to read, in this order

1. `docs/superpowers/specs/2026-09-02-phase-5c-class-methods.md` -- the spec. D70-D74, M1-M9.
2. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- the plan under review. Seven tasks.
3. `rust/CLAUDE.md` -- the project's standing rules.
4. `docs/superpowers/specs/2026-08-27-phase-5b-instances.md` and
   `docs/superpowers/plans/2026-08-27-phase-5b.md` -- 5b, whose handovers 5c inherits. D57-D69.
5. `rust/corpus/docs/class-set.txt` and `class-methods.txt` -- the generated authority, headers
   included; they explain their own statuses.
6. `rust/crates/rexx-exec/tests/gate_table_c.rs` -- what consumes those files.
7. `rust/crates/rexx-exec/src/dispatch.rs` around `NATIVE_METHODS` -- the table D71 and D72 are about.
8. `docs/superpowers/records/2026-08-27-phase-5b/` -- how 5b's tasks actually went, including the
   final review and its fix round. The defect class named there is the one to expect again.

## The standard this project holds a plan to

**A claim is not verified until something has been run.** This plan's author has been wrong four
times in recent memory by checking that a symbol exists and asserting what it does. Findings that
consist of reading are worth much less than findings that consist of running.

**Hunt these specifically:**

1. **Vacuous criteria.** A criterion that is achievable without doing the work, or a witness that
   cannot fail. D70 exists because 546 rows could go green over a refusal; check whether the plan
   has re-introduced the same shape somewhere else, and whether D70's own replacement criterion can
   itself be gamed.
2. **Citations that do not say what is claimed.** Every `file:line` in both documents. There are
   many. Several were added late and some were copied between documents. Open each.
3. **Numbers.** The partition (49/34/15/12/22 = 132), the group sums (135+177+70+70 = 452),
   `not-covered` = 546, the 20 classes, the 59 native entries, the seven `unreachable` rows.
   Re-derive from `rust/corpus/docs/`, do not trust the prose. **The spec already had one defect of
   exactly this kind** -- two different 23-class sets treated as one -- so assume there is another.
4. **Task independence.** The plan claims Tasks 3, 4 and 5 share no code and may run in parallel.
   Check that against what they would actually touch, especially `NATIVE_METHODS` and the per
   mechanism slices D72 asks for. If they would collide, the plan's parallelism claim is false.
5. **Task 2's size.** M2-M6 merged into one task was Moritz's ruling and is not up for reversal, but
   whether the plan gives it enough structure to be reviewable **is** in scope. 132 names in one
   task is the largest unit this project has attempted.
6. **The done-when clauses.** For each task, could it be satisfied by work that left the subject
   broken? Name the case.
7. **What the plan does not say that an implementer would need.** 5b's tasks repeatedly discovered
   that installing a directive runs Rexx code, and similar premises that failed. Look for premises
   here that nobody has tested.

## Specific things worth running

* Construct the three comparators, `VariableReference` and `Singleton` on the oracle. The plan says
  nobody has. If any is native-only, Task 1's shape is wrong.
* Check whether `class-set.txt`'s `covered` "opted in with a committed construction program" arm is
  actually implemented in `rexx-extract-docs`, or only documented in the header. The plan tells Task
  1 to find out; if the arm does not exist in code, that is a finding now.
* Confirm the receiver-position table in both documents by running the eight pairs.

Oracle runs: from a **fresh empty directory**, three descriptors read separately, never `2>&1`,
wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
Never run anything in `rust/corpus/oracle-crashes.txt`.

## Report

To `.superpowers/sdd/2026-09-02-phase-5c/plan-review-report.md`. Severity-ordered. Each finding gets
the transcript or the command that produced it. **Say plainly what you did not check**, and if you
believe the plan is sound in some area, say that too rather than padding with speculation.
