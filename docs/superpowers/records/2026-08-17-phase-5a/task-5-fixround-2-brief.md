# Task 5, fix round 2 — rulings on the re-review

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-5-rereview.md`. **All seven findings CLOSED and
both demonstrations now fail** — the synthetic `Zork` rows exit 101 and read `unanswered`, the `0xff`
probe exits 101 with two instruments naming it, and the case finding 3 was about no longer produces a
misdirected structural failure. `OracleShape`'s design, the `unanswered` verdict that keeps a row in
the table, and the cross-family referential check are all right and are not reopened.

One medium, five low. **This round closes the task.**

---

## N1 — MEDIUM. **The guard you built for one zero-count arm is missing from the two you then
created.** Ruled: fix both.

You added `asked` — withholding a verdict from a row whose own line is absent from **both** sides —
*precisely because* `AllOrNothing(n)` admits `0` and the shape check alone left three synthetic method
rows green. **Then the same round introduced two more arms whose admitted count is zero and gave
neither of them that guard.** Where the expectation is "the oracle printed nothing", *nothing there
at all* satisfies it.

**Table C, the `entry` column.** `gate_table_c.rs:1444` is
`Exactly(if row.entry == "class" { asked } else { 0 })`. Demonstrated: the same synthetic `Zork` row,
filed with `entry` reading `instance`, exits **0** and reads **`agree`** — *"a gated 5a row green over
a class that exists in neither interpreter"*, which is the original demonstration reproduced after
the fix.

The live row is `RexxInfo`. It reads `diverge-both` today, and its false green needs only the oracle
to stop shipping `.RexxInfo` — **which is the `RegularExpression` scenario `OracleShape`'s own doc
cites as the reason the type exists.**

**Ruled, two parts.** Give an `entry != "class"` row a **non-zero** bound by asking one question the
documented entry does answer — the re-review names `.RexxInfo~class~id`, one line an entry answers
and an absent name does not. **And validate `row.entry`**: it is read at exactly two decision sites
and validated nowhere, so a fourth entry kind or a typo **silently exempts its row** instead of
reddening. An unrecognised value must be structural. (`check_edge_endpoints` excludes `instance` rows,
so the referential check does not reach this row either — that is correct for edges and is why the
bound has to carry it.)

**Table D, `ORACLE_REFUSES`.** Seven rows have `expected == 0`, so a verdict is computed from
`compare_raw` exactly as before this round. Demonstrated: replacing a **gated 5a row's** probe with a
program reading `nop` moves it from `diverge-both` to **`agree`**, takes the 5a open count from 10 to
9, and exits **0** with no structural failure.

**Ruled: first ask whether a non-zero bound exists for those seven** — the oracle refuses at install
time because a named shared library is absent, and a probe that prints one line *before* the refusing
directive would have one. If it genuinely does not exist, **then** apply my earlier fallback properly:
`# What this table cannot see` is where that table names its gaps, the round did not touch it, and
the fallback asks for **the reason and the task that could close it**. `ORACLE_REFUSES`'s doc gives
the reason each row is refused; it does not say those rows' verdicts still carry finding 1's
property.

**Why this is worth the round.** Task 24's "Done when" is *every 5a row in both tables reads `agree`*.
Three rows can satisfy it with neither interpreter having answered a question: `RexxInfo`'s wiring
row, and table D's `::ATTRIBUTE EXTERNAL` and `::METHOD EXTERNAL`.

## N2 — LOW-MEDIUM. **Ruled: fix.** `corpus/gate-tables/README.md` says *"Every row's oracle side is
checked for having answered at all, before any verdict exists"* and *"`directives/` has the same
check, against the one line each of its probes prints."* Measured: **72 print one line and 7 print
none**, and those 7 are exempted rather than checked. **The README is tracked and is what a reader
consults to learn what protects a row's verdict** — unlike the report, which is git-ignored until the
plan closes. Fix the smaller inaccuracy in the same sentence too: for `classes/` the bound is the
derived text **gated on the `entry` column**.

## N3 — LOW. **Ruled: fix, one line.** The 497 sentence computes `constructor_raised` from an
**oracle-only** predicate while `unanswered` uses a **both-sides** one. Both read 497 today, so
nothing is wrong now — but a group whose oracle raised at `~new` while this crate answered would be
counted by the sentence and would not be `unanswered`. **The reader would meet a false claim in the
line I asked for specifically because a reader meets it.** Sum the method rows whose verdict is
`None`.

## N4 — LOW. **Ruled: hoist.** `verdict_label`, `UNANSWERED` and `stdout_line_count` are duplicated
verbatim across the two test binaries, and **both already `mod gate_tables`** where `Verdict`,
`verdict()`, `Report` and `Structural` live. This is **not** the `corpus.rs` situation, where three
integration binaries genuinely cannot `mod` one another — the shared module exists and is already
imported. Nothing makes the two labels stay the same string, and a consumer reading both tables would
not see them drift.

## N5 — LOW. **Ruled: strike both.** `check_probe_text`'s *"Measured: one `0xff` byte appended…"* and
`first_difference`'s *"so a file differing… produced two identical empty strings"* are history, and
the surrounding sentences already carry the contract in full. **`Measured::verdict`'s "Dropping it
would…" stays** — a rejected route stated as the reason the current type is what it is, which the
same rule keeps.

## N6 — LOW. **Ruled: qualify the heading.** *"No oracle answer is recorded anywhere in this file or
beside it"* now has an exception: `Concept::oracle_lines`. The field's own doc meets the objection
squarely and the re-review confirmed it is **not** a rubber stamp — for six concept probes the
committed count differs from the probe's top-level `say` count, so the number carries information the
text does not. The section heading was simply not qualified when the field landed.

---

## Verification

Five gate commands, each with **its own** exit status; corpus **106 of 106**. Both tables' 5a gated
counts separately, with predicates.

**Re-run the re-review's three controls after fixing**: the `Zork` row filed as `entry = instance`,
an unrecognised `entry` value, and table D's `nop` probe on a gated 5a row. Report what each now does.

`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` stays **expected non-zero**.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-5-report.md` under "Fix round 2".

**Return only:** status, commit SHA, one line per finding, both gated counts, whether the three
controls now fail, and anything you could not close.
