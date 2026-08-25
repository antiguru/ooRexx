# Task 2, fix round 3 — rulings on re-review 2

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-2-rereview-2.md`. **All seven findings CLOSED**,
and the ruling held where it was aimed: every command in the ledger was extracted from raw bytes and
re-run, and all but one reproduce. Four new defects. The trend is 9 → 7 → 4 with the severities
falling, so this is converging; do not treat it as a document in trouble.

---

## The structural ruling, which is the point of this round

**Replace every completeness assertion with the pasted output of the check that supports it.**

The ledger says at `:120`-`:121`: *"With those placed, every C++ `file:line` the spec cites appears in
(a), (b), (c) or the found-wrong table, and the check that says so is run over the spec."* **The check
was run and it still misses one** — `ClassClass.cpp:988`-`:990`, cited by the spec at `:946`, in no
list. That is the second consecutive round in which a conservation sentence was wrong, and the
re-review names the exact consequence: **the completeness sentence is what stops the next reader
looking.**

An assertion cannot be checked at a glance and rots silently. **A pasted derivation can:** print the
spec's citations beside the list each was placed in, commit that output, and the claim *is* the
artifact. A reader spots a missing row without re-running anything, and a later task diffs it.

Your header already does the honest version of this for the document's own denominator — *"an
authority nobody thought to list is invisible to this document"* — and it is well written. **The C++
row's completeness claim is the sentence that does not live up to it.** Bring it into line: show the
set, do not assert over it.

Same rule anywhere else the ledger says "every", "all", or "none remain".

---

## Finding by finding

**D1 — fix, three parts.**
1. Place `ClassClass.cpp:988`-`:990`. The re-review resolved it: `:984` is
   `MethodClass *RexxClass::method(RexxString *method_name)` and `:988`-`:990` is the three-line
   comment beginning `// we keep the instance methods defined at this level in a separate`. **The
   citation is correct** — the defect is that it was never placed.
2. **Widen the extractor to the third citation form.** Your stated extraction is *"every
   `File.(cpp|hpp):N`, plus every bare `name :N`"*. This is neither: a bare `:N` continuing an
   *earlier citation in the same parenthesis*. That is the search-narrower-than-its-sentence defect on
   a **new axis**, inside the fix for the previous instance of it.
3. **The C++ authority row is marked `done` against a stopping point a complete run of its own check
   does not reach.** Say what the check reaches, and mark the row against that.

**D2 — fix, two parts.** The extension paragraph **drops files its own command printed**, and the
glob is **case-sensitive**: `--include='*.cls'` does not match `ootest/framework/OOREXXUNIT.CLS`,
which is the ooTest framework itself. Correct the enumeration to match the command's output, make the
globs case-insensitive wherever they are used, and **re-run every surviving negative under the
corrected set**. A negative that survives a wider search is worth something; one that was never
widened is worth nothing.

**D3 — fix.** Row `:137`'s command contains a literal `…` where every other row writes
`<extensions>`. Pasted verbatim it is `grep: …: No such file or directory`, **exit 2** — the pattern
read as a path. The report says every command was run as written before being written down; for this
one it was run with the includes and written with an ellipsis. **No command in this document contains
an ellipsis.** Every one pastes from raw bytes and runs.

**D4 — LOW, fix.** `:137`'s `INIT`-before-`INHERIT` negative narrows to exactly one file and then
concludes nothing asserts the discriminator, without saying why the hit does not count: that file's
three `::method init class` bodies are each a bare `nop` (`:14`, `:35`, `:51`). **A search that
returns a hit is being used to support a negative** — state the disqualifying fact, as every other row
does.

---

## Verification

Extract every command from the **raw bytes** of the committed file, run each, and paste what it
returns. Report the count run and the count reproducing — that number is this round's real output.

Five gate commands, each with its own exit status; corpus **106 of 106**. `REXX_PHASE_GATE=5a …`
stays unrun.

Append to `task-2-report.md` under "Fix round 3".

**Return only:** status, commit SHA, one line per finding, how many commands you re-ran and how many
reproduced, and anything you could not close.
