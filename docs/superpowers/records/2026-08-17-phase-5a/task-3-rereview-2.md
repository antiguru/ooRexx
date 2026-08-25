# Task 3, fix round 2 -- re-review

Scope `c0fda0a44..094fbbdd8`, two commits, re-reviewed at `094fbbdd8` with a clean working tree.
`oodocs/rexxref` and `oodocs/rexxpg` at r13198, `ootest/` at r13178, checked with `svn info` today
(2026-08-21) -- unchanged from fix round 1.

**Verdict: APPROVE.** N1 and N3 close cleanly. N2's substance closes: the header now states the
override rule and a reader following `DateTime new class`'s citation lands on the member that
displays `new`. The new test genuinely catches the offset bug it was written against -- but it does
**not**, and structurally cannot, catch the N2 defect itself if that defect recurred; the
both-directions check remains the actual backstop for that, and I verified it still is one. One new
finding, NIT, bounded by an existing safety net the module already discloses.

## Per-item tally

* **N1 -- CLOSED.** `unconstructible_index`'s doc no longer names `<xref>` as the mid-sentence tag
  (it names `<methodname>`, the one that is actually there) and states the span/sentence distinction
  as one criterion rather than two conflicting ones. The report's two copies of the deleted count
  (the "what the extraction found" entry and fix round 1's F6 entry) are both corrected in place, to
  the same measurement, in the same words. Re-measured independently below.
* **N2 -- CLOSED** (header/data substance). `origin` now cites the class-table member's own line for
  the two rows whose name came from an `xrefstyle` override, and the header states the override rule
  by name, with `clsDateTime` worked through. Following `DateTime new class`'s citation now lands on
  `utilityclasses.xml:1281`, which displays `new (Inherited Class Method)`. See "New defects" for a
  caveat on the new test's power, which does not reopen this item.
* **N3 -- CLOSED.** `ArgUtil`'s derived-list entry now cites `provide.xml:838`, matching the
  `class-set.txt` row's own citation and its two sibling lists in the same header.

## New defects

### D1 -- NIT. `every_method_rows_origin_line_names_its_section` cannot distinguish an override citation from the section it overrides, because both lines carry the same id by construction

The assertion is `found.contains(section)`, where `section` is the row's `mth*` id. For any override
row, the class-table member's line contains that id because its `<xref linkend="mth...">` names it,
and the target section's own line contains it because that is its `<section id="mth...">`. **Both
candidate citations satisfy the assertion identically**, so the test cannot tell which one a row
actually holds -- which is exactly the choice N2 was about.

**Demonstrated three ways, on a relocated copy outside the repository** (same sources, `oodocs/` and
`interpreter/` beside it; the tracked trees are read-only for this review):

1. In the committed `class-methods.txt` alone, with the crate untouched, `DateTime new class`'s
   origin edited back to `utilityclasses.xml:1775` -- the exact pre-fix citation N2 named, which
   lands on `<section id="mthDateTimeInit"><title>init</title>`, carrying no `new`.
   `every_method_rows_origin_line_names_its_section` run alone: **passed**.
2. The same single-row edit, full suite: `every_row_set_is_exactly_what_its_extractor_derives_today`
   catches it (`rows derived and not committed` naming `:1281`, `rows committed and no longer
   derived` naming `:1775`); the new test still passes. So an ordinary single-sided drift is still
   caught by the pre-existing both-directions check, not by the new one.
3. **The clean case**: `classes.rs`'s origin logic itself reverted to always cite `{file}:{line}` (the
   section), the extractor rebuilt, and `class-methods.txt` **regenerated** from that reverted
   extractor so both sides agree -- the exact pre-fix N2 state, reproduced honestly rather than
   hand-edited. All 8 integration tests **passed**, `every_method_rows_origin_line_names_its_section`
   included.

**What the test does catch, confirmed as claimed.** With the fix restored and only `head_base`
forced to `0` (the offset bug the round's own commit message names), then the corpus file
regenerated to match: `every_method_rows_origin_line_names_its_section` **failed**, verbatim --
`the row citing utilityclasses.xml:48 names mthDateTimeInit, and that line reads "  <xi:include
href=\"utilityclassesintro.xml\" xmlns:xi=\"http://www.w3.org/2001/XInclude\" />"`. Matches the
report's "seen firing" quote exactly.

**Consequence, and it is bounded.** The report's own framing is honest about what the assertion checks
(`origin line must contain the row's section id`) and does not claim it guards the override-vs-section
choice; my finding is that a reader could still take "the check that now catches it" (said of a
defect adjacent to N2) as broader assurance than the assertion supports. The residual risk is a
**regeneration** -- code regresses to citing the section for an override row *and* the corpus file is
regenerated to match in the same commit -- which is already this module's own disclosed, structural
blind spot (`extract_docs.rs`'s doc: "it fires when one side moves and not when both move together in
one commit -- a regeneration... reviewed as artifacts rather than waved through"), not something this
round introduced. A single-sided drift, the far more likely accident, is still caught. NIT rather than
LOW because the actual backstop exists and is itself disclosed.

## Verification

**N1, re-measured independently, over the six `UNCONSTRUCTIBLE` rows at the lines and text
`classes.rs` carries:**

| row | cited span | span wraps? | span carries |
|---|---|---|---|
| Buffer `:429` | one line | no | (plain) |
| Pointer `:6910` | one line | no | (plain) |
| RexxContext `:7545` | one line | no | (plain) |
| RexxInfo `:7942` | one line | no | (plain) |
| StackFrame `:9407` | one line | no | (plain) |
| VariableReference `:12556`-`:12557` | range | **yes** | `<methodname>` |

So under the cited-span reading, **one** row wraps and **one** carries inline markup (the same row),
and no cited span carries an `<xref>` -- matching both the doc comment and the report exactly.

Read at the book (`oodocs/rexxref/en-US/utilityclasses.xml`) for the whole-sentence reading: Buffer's
sentence is `:428`-`:429` (wraps, one-line citation); Pointer's is `:6909`-`:6910` (wraps, one-line
citation); RexxContext's sentence sits wholly on `:7545` (no wrap); RexxInfo's sentence is
`:7940`-`:7942`, with an `<xref>` on `:7941` -- inside the *sentence* but outside the *cited span*
(wraps); StackFrame's sits wholly on `:9407` (no wrap); VariableReference's sentence and cited span
coincide at `:12556`-`:12557` (wraps). So under the whole-sentence reading, **four** wrap: Buffer,
Pointer, RexxInfo, VariableReference -- matching the doc comment and both corrected report entries,
which now state the identical measurement.

**Re-derivation**, from a fresh scratch output directory:

```
rexx-extract-docs --oodocs oodocs --interpreter <repo>/interpreter --out <scratch>
```

All five files byte-identical to the committed copies (`cmp`, no output). `--check` in place against
`corpus/docs`, all five `unchanged`, exit 0, working tree clean afterward.

**Row count, predicate stated.** `class-methods.txt`: `wc -l` 1432; `/bin/grep -ac '^#'` 84;
`awk '!/^#/ && !NF'` (blank lines) 1; `awk '!/^#/ && NF'` (data rows) 1347 -- 1432 = 84 + 1 + 1347,
matching the report's stated breakdown exactly. The other four files by the same data-row predicate:
21, 57, 63, 79 -- matching.

**The both-directions test, in both directions, on the relocated copy with the fixed crate:**

| direction | what I did | result |
|---|---|---|
| row removed | deleted `DateTime new class` from `class-methods.txt` | FAILED, `rows derived and not committed (1)` naming it |
| row added | appended an invented `DateTime zznosuch class` row | FAILED, `rows committed and no longer derived (1)` naming it |

**N3.** `class-methods.txt:82`: `ArgUtil  --  the books document it nowhere; its only citation is
provide.xml:838`, matching `class-set.txt:93`'s own `provide.xml:838` for the same class. The other
two derived lists in the same header (`mth*` sections no class table names; `*classmethods.xml` files
no class table includes) already carried `file:line` or a bare filename respectively, unchanged by
this round.

**ASCII and `unsafe`.** `git show 658514a42 094fbbdd8 | grep -aP '[^\x00-\x7F]'` after the second
commit: nothing outside the diff's own `+`/`-` markers for the two lines the second commit corrects;
the tree at `094fbbdd8` (`classes.rs`, `extract_docs.rs`, `class-methods.txt`) and both commit
messages are clean. `git show ... | grep -an '^+.*unsafe'` exits 1.

**Crate scope.** `git show --stat` for both commits together: `corpus/docs/class-methods.txt` and two
files under `crates/rexx-extract/{src/docs,tests}/`. None of `rexx-exec`, `rexx-core`, `rexx-classes`,
`rexx-lib` -- no performance sitting owed, matching the report.

**Gates**, run by me from `rust/` at `094fbbdd8`, each status read on its own, none piped into
another command. Clippy re-run after touching the two changed files to rule out a stale per-crate
result:

```
cargo fmt --all --check                                             exit 0
cargo clippy --workspace --all-targets -- -D warnings                exit 0  (re-run after touch, still 0)
cargo test --release --workspace                                    exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 exit 0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  exit 0   106 of 106 matching
```

Zero `test result: FAILED` lines in any of the three test runs; `rexx-extract`'s lib suite is 32 unit
tests and `extract_docs.rs` is 8 integration tests, `every_method_rows_origin_line_names_its_section`
among them, both read in the log rather than assumed. `REXX_PHASE_GATE=5a` correctly unrun:
`/bin/grep -arn 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/crates/` exits 1.

`svn info` re-checked today for `oodocs/rexxref`, `oodocs/rexxpg` (both r13198) and `ootest`
(r13178) -- unchanged from fix round 1.

## What I could not check

* **Whether these row sets are the right shape for tables C and D.** Unchanged from both prior
  reviews; still Tasks 4 and 5's to answer.
* **A coincident regeneration reverting N2 itself**, both sides moving together in one commit --
  this is D1's residual risk and, as the module's own doc says, is a diff for a human by design.
* **My scan shares the prior review's text-level assumption** (comments blanked, tag-stripped
  matching rather than a real XML parser); nothing in this round's diff touches that boundary, so I
  did not re-probe it.
* **Anything already closed by fix round 1's re-review** (F1-F6, the `xrefstyle` guard's assertion
  firing at both call sites and on both arms, the `hashCode` measurement, CI running no `cargo`) --
  out of scope for this round and not reopened.
