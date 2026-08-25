# Task 3, fix round 2 — rulings on the re-review

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-3-rereview.md`. **APPROVE — every finding
closed.** The `xrefstyle` guard was seen firing on the **production path**, over a real book with a
genuine sixth `template:` member introduced, at both call sites and on both arms; the both-directions
test fails in both directions; `TEMPLATE_MEMBERS`'s correction is right at the source.

Three residual items — one low, two nits, **none of which can make a gate pass while meaning
nothing**. This round is small and closes the task.

---

## N2 — NIT by label, and the one I actually want fixed.

`class-methods.txt`'s header says *"`section` is the `mth*` section **the name came from** and
`origin` is that section's `file:line`, so every row cites the book."* **For the two new rows the
name came from the class-table member's `xrefstyle`**, at `utilityclasses.xml:1281` and `:10492`,
which no row cites — and following `DateTime new class`'s own citation lands on
`<section id="mthDateTimeInit"><title>init</title>`, which contains no `new` anywhere.
`/bin/grep -a 'xrefstyle\|template\|displays'` over the committed file matches nothing, so **the
artifact carries no rule that explains its own two strangest rows.** `TEMPLATE_MEMBERS`'s doc
explains them, and it is in the crate rather than in the file.

**This is F3's ruling again, one round later: the explanation belongs in the committed artifact,
because `.superpowers/` is git-ignored and a reader with the data and not the crate cannot tell a
derived row from a wrong one.** Make the header state the override, and cite where the displayed
name came from — the mechanism is already right, only its record is missing.

## N1 — LOW. Fix the comment, and bring the report along.

`unconstructible_index`'s doc says a sentence *"may wrap across a line break or carry an `<xref>` in
the middle"*, and that *"a row's span being a range is what says it wraps"*. The second clause is
true; the first names a form no cited span has — **the one live mid-sentence tag is `<methodname>`,
not `<xref>`.** The two clauses also use "sentence" in two senses, the book's full sentence and the
cited span, and the criterion is true under one and false under the other.

**And the report still carries the count the round deleted from the code** — *"three of the six
sentences wrap"*, which measures as one under the cited-span reading and four under the
full-sentence one, and is not three under either. **A wrong count removed from the source and left
in the record is worse than either alone**, because the two now state different tests. Fix both, and
say which sense of "wraps" decides the span a later row should carry.

## N3 — NIT. `ArgUtil`'s derived-list entry cites `provide.xml` where both sibling lists in the same
header cite `file:line`, and `classes_without_method_rows` has the line in hand in the same
`ClassRow`. Give it the line.

---

## Verification

Re-derive every row set and confirm each committed file reproduces; run the both-directions test
**in both directions**. The five gate commands, each with **its own** exit status; corpus **106 of
106**. `REXX_PHASE_GATE=5a …` stays unrun.

State the predicate beside any count you report — "rows" and "non-comment lines" differ by the blank
line under the header, and I made exactly that mistake checking your last figure.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-3-report.md` under "Fix round 2".

**Return only:** status, commit SHA, one line per item, and anything you could not close.
