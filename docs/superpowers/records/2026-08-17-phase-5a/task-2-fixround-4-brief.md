# Task 2, fix round 4 — rulings on re-review 3

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-2-rereview-3.md`. **Every prior finding is
CLOSED.** The round's central deliverable works: the committed extractor, pulled from the file's raw
bytes, reproduces its committed output **byte for byte** with no `UNPLACED`, and **both negative
controls were run and both fired**. Seven new defects — three medium, four low, **no highs**.

You are a fresh implementer on this task. That is protocol for round four, not a judgement on the
work: the same author has now had seven instances of one correlated error, and fresh eyes break the
chain. **The findings below are precisely localized — most are one line.** Do not redesign anything.

---

## The document, in one paragraph

`docs/superpowers/plans/phase-5-reading-ledger.md` records which authorities behind the Phase 5 spec
have been read end to end, to what stopping point, at which revision. It exists because the spec's
enumeration of mechanisms has no denominator. Three fix rounds have driven one principle into it:
**a claim carries the command that produced it, and a completeness assertion is replaced by the
committed output of the check that supports it.** Everything below serves that principle.

## The recurring defect, so you do not become its eighth instance

**A search is exactly as wide as its pattern, and its result is a claim about the pattern, not about
the world.** Seven instances in this task, each on a new axis: a keyword list, a quoted phrase, a
construction shape, a heading level, a file-extension glob, a citation form, and now a line wrap.
**Before you write any negative, put the pattern beside it.** And **run every command verbatim, from
the file's raw bytes, before you write it down** — three procedures have shipped that were never
executed as written.

---

## Finding by finding

**N1 — MEDIUM. Fix the extractor, not the sentence.** The spec's prose is hard-wrapped, so form 3 —
a bare `:N` continuing an earlier citation in the same parenthesis — sometimes has its filename on the
**previous line**. The per-line filename reset drops every such token, **and drops it invisibly**: no
row is printed, so the "no `UNPLACED`" verdict cannot see it. Three are in the committed spec
(`:1349`-`:1350`, `:1359`-`:1360`, `:1360`-`:1361`).

**It costs no conclusion today** — all three are cited elsewhere in a form the extractor reaches. Fix
it anyway, and fix the mechanism rather than documenting the hole: **carry the filename across a line
break, resetting on a blank line or on any intervening filename.** The re-review's argument is
decisive — *a wrap is not a choice the author makes about the citation*, so this is the likeliest
shape of the next citation added, and when it arrives the check prints a clean derivation with no
`UNPLACED` and the reader who runs nothing sees nothing wrong.

Keep the `07:06` → `Setup.cpp:06` false positive suppressed; that is what the per-line reset was for.

**N2 — MEDIUM. Qualify the match by file.** `where()` searches the sections in a fixed order and
returns the first carrying a matching `:N` **with no file association**, so two rows are mislabelled
`found-wrong` — `Setup.cpp:1285` and `ClassClass.cpp:1148`, both of which the ledger's own list (a)
records as **correct**. Both flip to `(a)` when the search order is permuted and nothing else changes.
**An artifact whose labels depend on the order its sections happen to be listed in is not
reproducible.** Fix it properly; do not close it by disclosing the limitation.

**N3 — MEDIUM. Delete five words.** `:484` says the nine block patterns are run *"with the extension
set above"*. **This round deleted the extension filter** — `:400` says so, and `:407`-`:416` exists to
condemn it. A reader following `:484` re-runs three negatives under the narrowing whose removal was
the previous round's fix, and on row `:137` under a filter that cannot see the file the row is about.

**N4 — LOW.** The rule *"resets on any filename, not only a C++ one"* is stated as load-bearing and is
**inert against the committed script** — narrowing it changes nothing. It becomes genuinely
load-bearing once N1's fix relaxes the per-line reset, so state it as the guard it is, and **verify
after N1 that it now does something**.

**N5 — LOW, report side.** The D3 measurement — *"no line containing an ellipsis also contains `grep`,
`awk`, `find`, `python3` or `perl`"* — returns a hit as written (`:33` contains `<section id="cls…">`
and the word find**ings**), and its five-word list omits `sed`, `svn`, `cargo` and `python`, all of
which appear in commands in this document. The conclusion holds; the stated check does not support it.

**N6 — LOW.** The D4 insertion replaced a semicolon with a full stop and left the following clause
beginning with a lower-case `and`. **This is a recorded hazard on this project — an insertion
silently orphaning the text after it.** Repair the join.

**N7 — LOW. Finish applying the structural ruling.** It was applied to the C++ row only. The
four-books authority row still reads `done` for *"every `<section id="cls…">`'s prose … extracted
mechanically"* — a completeness claim naming a mechanical extraction with neither the extraction nor
its output committed, which is the shape the C++ row just shed. The re-review measured that it costs
nothing today (the denominator is stable across three widths of the pattern), so this is cheap:
commit the extractor and its output, as the C++ row now does.

---

## Verification

Extract every command from the **raw bytes** of the committed file, run each, paste what it returns,
and report how many ran and how many reproduced. After N1 and N2, **re-run the extractor and diff its
output against the committed block** — the diff is the deliverable, and both negative controls must
still fire.

Five gate commands, each with its own exit status; corpus **106 of 106**. `REXX_PHASE_GATE=5a …` is
**not** run — the constant does not exist in the tree, so it would exit 0 for an unrelated reason.

No `unsafe`, no subagents, no `git add -A`, no bare `git stash`, `/bin/grep -a` for counts. Commit
with `git commit -F`, ASCII only, ending:
`Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`
`Claude-Session: https://claude.ai/code/session_01JbTwLiDsg7WUyhMqWthgZZ`

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-2-report.md` under "Fix round 4".

**Return only:** status, commit SHA, one line per finding, the commands-run/reproduced counts, and
anything you could not close.
