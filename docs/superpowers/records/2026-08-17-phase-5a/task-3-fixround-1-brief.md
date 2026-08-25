# Task 3, fix round 1 — rulings on the review

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-3-review.md`. **Spec compliance: APPROVE** — every
hard case the brief enumerates is a named branch with a unit test, and each one the reviewer
re-derived independently from `oodocs` agrees. **Task quality: REWORK**, on one finding.

The row sets held everywhere the reviewer could push them: every derived count reproduces from an
independent pass, both oracle sweeps reproduce row for row, and about sixty sampled rows agree with
the book at the line they cite. One medium, four low, one nit.

**F5's controller half is already done** — the plan now points at `directive-options.txt`'s header as
the authority on the parser-side rule, committed at `c36f52881`. Your half was already right: four
places, one of them the artifact. Nothing to do there.

---

## F1 — MEDIUM. The one unguarded rule, and it is missing rows from a denominator.

`method_rows` reads a class table's `<member><xref linkend="mth…">` and takes the name from that
section's `<title>` — but the book does not always display the section's title. Five members carry
`xrefstyle="template:<text>"`, which replaces the rendered name outright. Three come out right by
accident. **Two do not**: `DateTime`'s class table names its constructor `new (Inherited Class
Method)` pointing at `mthDateTimeInit`, and `TimeSpan` is the same shape — so the row set carries
`DateTime init instance` and **no `DateTime new class`**.

Measured: `.DateTime~hasMethod("NEW")` is `1`, `.TimeSpan~hasMethod("NEW")` is `1`. **The emitted
rows are true; the missing ones are missing.**

**Why this one matters more than its severity suggests.** `DateTime` is `covered`, so its rows are
the ones Task 5's table C actually probes, and the entry the book's own class table displays as
DateTime's constructor is not among them. **The both-directions check is structurally blind to it**:
it compares the extractor with itself, so a row that was never derived has no arm to disagree about
and the file and the derivation agree perfectly on its absence.

**This is the same shape as hierarchy rule 1, which you did guard** — an extractor that misreads the
book in a direction nothing downstream contradicts. Rule 1 got the `ArgUtil` assertion; this rule got
none.

**Ruled: emit the missing rows, and guard the rule in the shape you already use.** Assert in
`method_rows` that every class-table member's `xrefstyle` begins `select:title`, with a named
exception list for the `template:` members whose displayed name you have decided about. **Make the
assertion fire before you trust it** — a sixth `template:` member added upstream must redden, not
drift. Put the pattern that finds the five beside the claim, as the reviewer did:
`<member>\s*<xref linkend="(mth[^"]*)"\s*xrefstyle="([^"]*)"` over the four books plus every
`*classmethods.xml`, comments blanked.

## F2 — LOW. **Ruled: record the decision, do not add a row.**

Task 2's ledger finding 11 — `fundclasses.xml:102` comments out `mthClassHashCode` with the reason
*"won't document hashCode"*, there is no such section anywhere, and `.Class~hasMethod("HASHCODE")` is
`1` — is addressed in neither the report, the code, nor the header. `Class` is `not-covered`, so a
row would be gated on nothing and **the gap costs no coverage**. What it costs is a *decision* the
next reader has to re-derive from scratch. Record it where a reader looks: your report's "what the
extraction found that the plan and the spec do not carry" lists three near-identical cases and not
this one.

## F3 — LOW. **Ruled: the explanation moves into the committed artifact.**

`ArgUtil` is the one class in `class-set.txt` with no rows in `class-methods.txt`, and **nothing
went missing** — it is `covered`, the books document it nowhere, so it has no `mth*` sections. That
is correct and the reviewer confirmed it with `comm`. But the explanation lives only in a report
under `.superpowers/`, which is git-ignored until the plan closes, while `class-methods.txt`'s header
explains `RegularExpression`'s absence and says nothing about `ArgUtil`. **A reader with the data and
not the report cannot tell a deliberate absence from a lost row.** One line in the header.

## F4 — LOW. **Ruled: fix both.** Both conclusions are right and both stated evidences are not.

* *"finds it only in the file itself, exit 1 for everything else"* — re-run, the command prints
  nothing and exits 1; the string occurs in no file at all, and one `grep` invocation has one exit
  status. The reviewer's confirmation is better and is available to you: 16 `*classmethods.xml` files
  exist, `/bin/grep -arho 'href="[a-z]*classmethods.xml"'` finds 15 distinct hrefs, and
  `objectclassmethods.xml` is the one absent.
* *"The four that read only the committed files run everywhere"* — **CI runs no `cargo` at all**;
  all three workflows check out `ootest` and nothing else, so none of the tests in `extract_docs.rs`
  runs on any CI platform. Your test's own module doc says the true thing; the report does not.

## F6 — NIT. **Ruled: fix these two, do not sweep the crate.**

`classes.rs:188-189` and `:393-394` count **subsets by status** of a table whose whole point is that
a later task may add a row — adding a seventh falsifies both sentences while the code stays right.
Your other counts sit immediately above the set they name, which is what the constraint asks for.
The reviewer notes the rest of the crate does the same thing; that is pre-existing and **not yours to
sweep in this round**.

---

## Verification

The five gate commands, each with **its own** exit status; corpus **106 of 106**.
`REXX_PHASE_GATE=5a …` stays unrun.

After F1: re-run every extractor, confirm each committed file reproduces, and **run the
both-directions test in both directions** — a row removed and a row added must each redden. Report
the new `class-methods.txt` row count and the two rows added.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-3-report.md` under "Fix round 1".

**Return only:** status, commit SHAs, one line per finding, the new row count, whether the
`xrefstyle` assertion was seen firing, and anything you could not close.
