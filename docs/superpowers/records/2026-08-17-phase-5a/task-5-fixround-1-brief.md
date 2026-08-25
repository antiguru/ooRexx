# Task 5, fix round 1 — rulings on the review

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-5-review.md`. **Spec compliance: APPROVE** —
everything the brief asks is built, the counts are right, M9 is disposed of, the three structural
controls fire, the four verdict mutations are named against their owners **in files those tasks'
authors will read**, and all five gates exit zero at 106 of 106.

**Task quality: REWORK.** Seven findings, two major. **Both majors were demonstrated by a control the
reviewer ran, not argued from the code** — treat them as measurements.

---

## 1 — MAJOR. A row reads `agree` when both sides fail identically. **Ruled: fix, and generalise
what you already built.**

`run_probe` hands three descriptors to `verdict()` and **nothing asks whether the oracle produced the
output the probe was derived to produce.** You built exactly that check for the method family —
`expected_oracle_lines`, whose own doc says *"without it a probe that silently stopped asking half its
names would leave those rows comparing an absent line against an absent line, which reads `agree` for
a question nobody asked"*. **That reasoning applies unchanged to the class, edge and concept families,
where the derived text pins the expected line count exactly.**

Demonstrated: a synthetic `Zork` class row, edge row and three method rows, with correctly derived
probes. `.Zork` exists in neither interpreter, so `.Zork~id` raises 97.1 on both sides — identical
stderr, identical rc, empty stdout. **The table exited 0 and printed `agree` for all five**, and the
5a count went to `137 rows, 135 not yet agree`: two gated 5a rows green over a class that does not
exist.

**The consequence is not hypothetical, and the artifact's own history contains the instance.**
Task 9's "Done when" is *"the wiring rows for the classes this crate registers read `agree`"* and
Task 24's is *"every 5a row in both tables reads `agree`"*. A row naming a class this oracle build
does not ship satisfies both while neither interpreter can answer any of its questions — and
`class-set.txt`'s header records `RegularExpression` **removed by hand** for exactly that reason. Had
it been left in, **its wiring row would be green today.**

**Table D has the same property** — `gate_table_d.rs:404` builds its verdict from `compare_raw`
alone. **Ruled: fix table C's three families now**, where the derived text fixes the count and the
push is the generalisation the reviewer describes. **For table D, assess rather than assume**: if an
equivalent invariant exists over its probes, apply it; if it genuinely does not, **record it in that
table's header as a named gap, with the reason and the task that could close it.** Do not manufacture
a check that cannot discriminate — this plan calls that decoration.

## 2 — MAJOR. `check_probe_text` skips any probe whose bytes are not valid UTF-8. **Ruled: fix, and
the sibling table solved this exact problem one task ago.**

`let Ok(committed) = fs::read_to_string(&path) else { return; }` — the comment names *missing* as the
reason. `read_to_string` also fails on **invalid UTF-8** and on a **permission error**, and in both
the file exists, so the set check passes and the text check reports nothing.

Demonstrated: `classes/array.rex` replaced by a body reading `say 'id' .String~id` plus one `0xff`
byte. **The table exited 0.** That is precisely the mutation your own commit message names as the
reason the derivation exists — and one byte turns it off silently, while the probe still runs and
still produces a verdict.

**Task 4's N1 is the same defect and its fix is the pattern**: guard the structural push on the
directory listing (`on_disk.contains(...)`), so a genuinely missing file reports once through the set
check and anything else reports through this arm with its own error. Read that fix before writing
yours.

## 3 — MEDIUM. `expected_oracle_lines` reads `status` as a claim about the oracle. **Ruled: fix, both
halves.**

`class-set.txt`'s own header says **`not-covered` "CARRIES NO CLAIM ABOUT THE ORACLE"**, and the plan
says the same. And `covered` is a **disjunction** — *"a class a bare `~new` constructs, **or one
opted in with a committed construction program**"* — while `method_probe_text` derives a bare
`o = .X~new` for every `covered` class with no route for the second limb. Your doc restates one limb
as the whole definition.

**The failure this creates lands on the first task that does the thing `covered`'s definition exists
for:** committing an opt-in construction program flips a class to `covered`, leaves the derived probe
on a bare `~new`, the oracle prints zero lines where the table expects `rows`, and that is a
**structural** failure — red in `cargo test --release --workspace` and both corpus-gate commands the
global constraints require to exit zero. **The message points away from the cause.**

**Second half, disclosed but understated: 497 method rows across 23 groups sit on the `expected == 0`
path**, where `agree` means only that both sides raised alike at `~new`. Your report says these
"measure the constructor, not the method set", which is right; what it does not say is that **the
visible outcome is `agree`** — so when 5b lands `~new` parity, a third of the method surface turns
green at once with no `hasMethod` question ever asked. **State that where a reader of the verdict
summary will meet it**, not only in a report that is git-ignored until the plan closes.

## 4 — MEDIUM-LOW. **Ruled: fix.** `first_difference` compares `str::lines()`, which strips the line
terminator and a trailing `\r` and ignores a missing final newline — so a probe differing *only* in
line endings prints `line 13 derived: ""` / `line 13 committed: ""`. **That is the exact shape your
report says control 4 fixed**: a check that runs, exits non-zero, and tells the reader nothing. The
fix closed the case where a line's text differs and left the invisible case open.

**Blast radius, which is why this is not a nit:** there is no `.gitattributes` anywhere in this
repository, so a CRLF checkout fails **all 216 derived probes at once** with that non-message.

## 5 — LOW. **Ruled: correct it here, do not amend.** *"All 32 `agree` rows in this table exist
because of that split"* is false, and **one copy is in an uneditable commit message.** Amending
reviewed work to fix prose costs more than the false sentence does; correct it in the report and in
any in-tree copy, and let the ledger carry the note about the commit message.

## 6, 7 — LOW. **Ruled: fix both.** `obdes` falsifies "every one of the six because its section's
distinguishing claim needs an instance". And the `reason` column is interpolated into a Rexx block
comment **with no escaping** — a reason containing `*/` ends the comment early.

---

## Verification

The five gate commands, each with **its own** exit status; corpus **106 of 106**. Report both tables'
5a gated counts separately — `cargo test` without `--no-fail-fast` stops at the first failing target,
which you already found.

**Re-run the reviewer's two demonstrations after fixing**: the synthetic `Zork` rows must no longer
read `agree`, and the `0xff`-byte probe must no longer pass silently. Report what each now does.

`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` stays **expected non-zero**. Do not try to make it zero.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-5-report.md` under "Fix round 1".

**Return only:** status, commit SHAs, one line per finding, both gated counts with predicates,
whether the two demonstrations now fail, and anything you could not close.
