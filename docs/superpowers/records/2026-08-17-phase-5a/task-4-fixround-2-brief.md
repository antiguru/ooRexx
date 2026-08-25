# Task 4, fix round 2 — rulings on the re-review

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-4-rereview.md`. **APPROVE — nine of nine
closed, every control fires, all five gate commands exit zero at 106 of 106.** It confirmed each fix
by reverting it in a scratch tree and watching the old behaviour return, which is the strongest form
available.

One low, one report-only low, three nits. **This round closes the task.**

---

## N1 — **Ruled: fix, two lines, and fix it because Task 5 copies this pattern.**

The `Err` arm's message says *"the probe is listed in the directory but cannot be resolved"*. The
missing-probe check above it pushes a `Structural` but **does not remove the row from `rows`**, so
the loop still reaches `fs::canonicalize` for that row and fails with the same `os error 2` a
dangling symlink gives. **A genuinely deleted probe now reports twice, and the second message asserts
the file is listed in the directory when it is not.**

That matters more than its severity: **deleting or renaming a probe is the commonest structural
failure this table has** — it is the brief's own mutation 3 — so this is the message a reader meets
most often, and it sends them after a symlink or permission problem that does not exist. **The one
fact distinguishing the new arm's case from the old check's is the one fact the message states
without checking.**

`on_disk` is still in scope: guard the push with `on_disk.contains(&probe)`, or drop the clause and
say *"the probe path cannot be resolved: {error}"*. Then **run both cases** — a deleted probe and a
dangling symlink — and confirm each reports once, with the right message.

## N2 — **Ruled: fix.** The report's L6 sentence names three probes that carry a readback and there
are four: `options__engineering__value_of_form.rex` is byte-identical to the `FORM` one after L2's
change, and round 1's own L2 paragraph named it as discriminating. The sentence reads as exhaustive
and **contradicts the L5 paragraph two sections above it**, which correctly says a `<keyword>
subkeyword` row and its `value-of(<keyword>)` row are exercised by the same clause. A reader counts
the rows that would notice an accept-and-ignore implementation, gets one fewer than there are, and
concludes `value-of` rows never discriminate.

## The three nits — **Ruled: fix all three.**

* `gate_table_d.rs`'s new comment mis-describes its own example: a probe *"whose bytes cannot be
  reached"* does **not** reach the `Err` arm — measured, `chmod 000` canonicalises fine and panics
  later at `gate_tables/mod.rs:192`. Still red, so nothing is at risk; the example is simply not this
  arm's. Give the arm an example that is.
* `run_on_both_engines`'s doc says *"The assertion is unconditional and names the program"* with
  **two** assertions now under it. The `chunks_refused` half is the one a reader of Task 5's table C
  would need, and the stated contract does not mention it.
* `corpus.rs:509` is a 93-character line in a doc block otherwise wrapped at 71–78 — new text joined
  to the old tail without rewrapping. **`cargo fmt` does not rewrap doc comments, so no gate can see
  it**; this is a case where the only instrument is a person looking.

## Not yours — already fixed

`global-constraints.md` still carried the stale `forbid` sentence. **That file is a copy I extract
from the plan for each dispatch, and I had extracted it before correcting the plan** — so the
attention lens handed to this task was stale while the plan was right. Re-extracted; it now reads
`deny`. Nothing for you to do, and worth knowing that the copy, not the source, was the last carrier.

---

## Verification

Five gate commands, each with **its own** exit status; corpus **106 of 106**. Report the 5a
non-`agree` count with its predicate. `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` is **expected to exit
non-zero** — that is the design until Task 24, not a regression.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-4-report.md` under "Fix round 2".

**Return only:** status, commit SHA, one line per item, whether both N1 cases report once with the
right message, and anything you could not close.
