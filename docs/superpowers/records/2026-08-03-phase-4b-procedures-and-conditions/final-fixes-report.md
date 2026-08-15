# Phase 4b — final fix round

**Status: DONE.** One commit, `a7f1a020`, tree clean. Every fix dispatched as
"fix before merge" is in; every item dispatched as "record for 4c" is recorded
in a tracked plan document.

Baseline held exactly: `cargo test --workspace` **1020 passed / 0 failed / 4
ignored**, `cargo fmt --all --check` and `cargo clippy --workspace
--all-targets -- -D warnings` clean, `REXX_CORPUS_GATE=1` 9/9,
`REXX_ASSERTIONS_GATE=1` 5/5, `REXX_KEYWORD_GATE=1` 7/7,
`./scripts/mutate-4b.sh` **12 of 12 as declared** with a 42-of-42 baseline
after the last restore. Every exit status read unpiped.

---

## What was fixed

| # | file | what changed |
|---|---|---|
| I1 | `docs/superpowers/plans/l1-coverage.md` | the compound-`DO` paragraph now names 4c as owner and cites the `EXCLUSIONS` row; the miss is recorded rather than silently repaired |
| I2 | `rust/crates/rexx-exec/tests/support/mod.rs` | ten/nine prefix split replaced by the property, **in both places it appeared** |
| I3 | `rust/crates/rexx-exec/tests/coverage.rs` | the false justification deleted, **and the same false present tense above `read_subset` itself** |
| M-a | `phase-4b-gate.md` | ratio row re-anchored to `a1484211` with the property and a re-measurement command; the dependent trend and threshold sentences re-derived |
| M-b | `phase-4b-gate.md` | narrowed to the tagging matches, with the gap the two dispatch matches leave stated |
| M-d | `src/lib.rs`, `tests/owners.rs` | "unavoidably" → separate by construction, costed at Step 3b, in both copies |
| M-e | `tests/keyword_assertions.rs` | unasserted gate total replaced by the property and a pointer |
| M-f | `phase-4b-gate.md` | soft-wrapped identifier joined |
| I5 | `deferred-minors.txt` | five items appended, header explaining the generator and its two blind spots |

**Three fixes went past the dispatched line**, each because the same false
statement had a second live copy in the same file, and fixing one of two is the
defect this round exists to close:

* **I2** — `TRACE_PREFIXES`' own doc comment repeated "not only the ten this
  crate can emit yet" and "the extra nine". Same falsification, same file.
* **I3** — `read_subset`'s own doc said "Today's callers all pass a one-element
  slice containing only `phase-4a.txt`". `coverage.rs`'s copy only; the other
  two copies' doc comments were already correct, so this did not touch I4's
  deferred consolidation.
* **M-a** — the paragraph under the table attributed the whole 1,928→1,976 move
  to one new test, and the "Across 4b the harness grew 4%" sentence 50 lines
  down used the same superseded endpoint.

---

## Every replacement sentence with a count or a universal, and the command that established it

This is the part the round is for. Nothing below was reread into existence; each
was re-run at `a1484211` before the sentence was written.

### I1 — "the fix is 4c's", the `EXCLUSIONS` row title, "merged the two rows"

```bash
grep -n '^EXCLUSIONS\|^KNOWN GAPS' docs/superpowers/plans/phase-4-exclusions.txt
sed -n '140,155p;1110,1140p'      docs/superpowers/plans/phase-4-exclusions.txt
sed -n '568,580p'                 docs/superpowers/plans/phase-4-exclusions.txt
grep -n '^## ' docs/superpowers/plans/phase-4b-gate.md
sed -n '5275,5292p' rust/crates/rexx-exec/src/run.rs
```

Row title at `:148` verbatim; the pointer block at `:1117-1136` states "THERE
WERE TWO ROWS AND THEY WERE ONE DEFECT ... They are merged at the destination"
and "It left this section for the one reason this section's own rules allow: an
owner was assigned", which is the phrase I quote rather than paraphrase. Section
title "Step 3c: the ruling on the compound-`DO` gap" confirmed at gate `:694`.

**One thing I did not copy from the dispatch.** The old paragraph called "a real
divergence with no owner assigned" the `KNOWN GAPS` section's *definition*; the
section actually says "**Most** rows are a real divergence with no owner
assigned" and names an exception. The replacement attributes the phrase as a
description, not a definition.

The claim that the other two copies were corrected by the ruling is established
by the `run.rs` read above -- `bind_control`'s doc now says "owned by 4c since
4b's Task 12 ruled on it".

### I2 — "the nineteen markers `trace_prefix_table` lists" (kept), "the oracle's whole table, not the subset this crate has an emitter for"

```bash
grep -n 'trace_prefix_table' /home/moritz/dev/repos/ooRexx/interpreter/execution/RexxActivation.cpp
sed -n '3567,3590p'          /home/moritz/dev/repos/ooRexx/interpreter/execution/RexxActivation.cpp
grep -c '^    \*b"' rust/crates/rexx-exec/tests/support/mod.rs
sed -n '580,615p'   rust/crates/rexx-exec/tests/trace_oracle.rs
```

C++ table: **19 entries**, same order as `TRACE_PREFIXES`. Rust table: **19**.
`is_trace_line` iterates all of `TRACE_PREFIXES`, so the property I wrote --
normalisation qualifies on the whole table -- is what the code does, not what
the paragraph wished.

Nineteen stays as a number on purpose: it counts an enumeration **outside this
repository**, which `rust/CLAUDE.md` explicitly permits, and it is additionally
asserted in-repo. The ten/nine split was the opposite -- an in-repo mutable
aggregate -- so it is gone rather than corrected to thirteen/six, which would
have gone stale at 4c's first new emitter.

**I first cited a test name I had invented**
(`the_prefix_coverage_table_is_the_oracles_own_nineteen`), then read
`trace_oracle.rs` and replaced it with the real one,
`the_trace_surfaces_coverage_is_thirteen_of_nineteen_with_owners_for_the_rest`.
Worth recording because it is M-f's defect with a different cause: a reference
that greps to nothing.

### I3 — "do **not** overlap -- 30 entries and 12, union 42", "never takes the `seen.insert` false branch"

```bash
cd rust/corpus
for f in phase-4a.txt phase-4b.txt; do
  echo "$f entries=$(grep -v '^\s*#' $f | grep -v '^\s*$' | wc -l)" \
       "distinct=$(grep -v '^\s*#' $f | grep -v '^\s*$' | sort -u | wc -l)"
done
cat phase-4a.txt phase-4b.txt | grep -v '^\s*#' | grep -v '^\s*$' | sort -u | wc -l
```

30/30, 12/12, union **42**. No repeat within either file and none across them,
so `seen.insert` returns true on every line of every committed run and the
de-duplication branch is unreachable from the real inputs -- which is the
justification the test needed and did not have.

```bash
git log --oneline --reverse -S 'phase-4b.txt' -- rust/crates/rexx-exec/tests/coverage.rs
```

`a462e3e9` (4b Task 1), which is the commit I cite for "where a second file was
first passed".

**A claim I wrote and then killed.** The first version of this comment said a
`read_subset` that sorted its output "would leave every one of them green". It
would not: `phase_4a_subset_matches_the_committed_list` and its 4b twin compare
against ordered literals and would go red. I had not run it; I reasoned it. The
sentence never reached the commit, and it is exactly the shape the dispatch
warned about -- plausible, adjacent to true, and about code two functions away.

### M-a — 1,983 / 15,841 / 12.5%, "grew 55 ... 48 ... 7", "7% against 76%", "~2,600 (a 31% jump)"

```bash
for c in 4c8c1f68 0e14fac4 a1484211; do
  h=0; for f in tests/owners.rs tests/loud.rs tests/coverage.rs; do
    h=$((h+$(git show $c:rust/crates/rexx-exec/$f | wc -l))); done
  i=0; for f in src/run.rs src/eval.rs src/error.rs; do
    i=$((i+$(git show $c:rust/crates/rexx-exec/$f | wc -l))); done
  echo "$c harness=$h interpreter=$i"
done
git diff --stat 4c8c1f68 a1484211 -- rust/crates/rexx-exec/tests/{owners,loud,coverage}.rs
python3 -c "print(round(1983/15841*100,2), round(1983/1852*100-100,1), round(15841/9025*100-100,1), round(2600/1983*100-100,0))"
```

`4c8c1f68` 1928/15838 · `0e14fac4` 1976/15841 · `a1484211` **1983/15841**.
The diffstat shows the whole 55-line move is in `coverage.rs`, and the three-way
split gives 48 at `0e14fac4` and 7 more at `a1484211`. Ratios: 12.52%, +7.1%,
+75.5%, +31%.

**The row is anchored to a commit rather than to "now", which is the fix.**
`a1484211` is immutable, so the row stays true forever; the property and the
re-measurement loop are in the document beside it. My own commit moved the
harness total again -- which is the argument, and is why I did not re-label the
row "after the final fix round".

**Two dependent sentences were false for the same reason and are corrected.**
"the fix round's new `phase_4b_subset_matches_the_committed_list` adds 48
harness lines" accounted for 48 of 55; "Across 4b the harness grew 4% (1,852 →
1,928)" used the assessed tree as if it were 4b's end. Both re-derived from the
measurements above.

**And one guard added.** The 4c trigger "or if the fraction *rises* at all"
would fire on 12.2 → 12.5, which is nothing but this gate's own documentation
commits. It now says to measure from 12.5% and to look for a sustained rise.

### M-b — "all seven are generated by `owners.rs`'s `tags!` macro and that file contains no `_ =>` at all"

```bash
grep -n '^tags!(' rust/crates/rexx-exec/tests/owners.rs   # 7 invocations
grep -n '_ =>'    rust/crates/rexx-exec/tests/owners.rs   # no matches
grep -n '_ =>'    rust/crates/rexx-exec/tests/coverage.rs # 2 matches
```

Seven `tags!` invocations, one per enum the criterion names. Zero `_ =>` in
`owners.rs`, which is the property that makes "no wildcard arm" true of all
seven at once without counting arms -- the macro alone would not, since a `pat`
can be `_`.

**I cite the two dispatch matches by enclosing test and visitor, not by line
number.** My own commit shifts `coverage.rs:654`/`:666` by six lines, so writing
those numbers would have been a claim falsified by committing it.

### M-d — "both places", "half a day"

```bash
grep -rn 'unavoidab' --include=*.rs --include=*.md --include=*.txt \
  rust/ docs/ .superpowers/sdd/2026-08-03-phase-4b-procedures-and-conditions/
```

Before: two live copies (`src/lib.rs`, `tests/owners.rs`) plus dated records in
task reports and reviews, which are left alone. After: no live copy asserts
unavoidability. The half-day figure and the "assert `lib.rs`'s match equals
`owners.rs` expanded through `expand_for_witnesses`" framing are quoted from
gate Step 3b `:841-854`, read directly.

Incidentally, `task-0-review.md:309` already said "**both** asserting the third
copy is unavoidable". The roll-up's one-line summary of it lost the plural. The
undercount was in the summary, not in the finding.

### M-e — the totals' new home

```bash
grep -n '713\|730' docs/superpowers/plans/l1-coverage.md docs/superpowers/plans/phase-4b-gate.md
REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions
```

713 appears at `l1-coverage.md:598` and in gate criterion 10; the gate run
prints "100 of 896 bodies passing, carrying 713 of 1773 assertSame calls". 730
appeared **only** in the comment being fixed -- nothing reproduced it, which is
why it is deleted rather than relocated. The comment now states the property in
both directions and points at the two documents.

### Gate's new "What 4c inherits" section — "18 of the 50", "12 of them 4b's", "21 rows: 11 / 8 / 2"

```bash
added4b=$(git diff --name-only --diff-filter=A dc87708d a1484211 -- rust/corpus/lang/ \
          | xargs -n1 basename)
n_missing=0; n_missing_4b=0
for f in rust/corpus/lang/*.rex; do
  b=$(basename $f)
  if ! grep -q "${b%.rex}" rust/corpus/README.md; then
    n_missing=$((n_missing+1))
    if echo "$added4b" | grep -qx "$b"; then n_missing_4b=$((n_missing_4b+1)); fi
  fi
done
echo "absent=$n_missing of $(ls rust/corpus/lang/*.rex | wc -l), of those 4b's=$n_missing_4b"
```

50 files, **18** absent from the README, **12** of those added in
`dc87708d..a1484211` -- and all twelve of 4b's additions are absent. (The
review's twelve names and `phase-4b.txt`'s twelve entries are different sets:
`interpret_dynamic.rex` is in the subset but predates 4b and *is* in the table.)

```bash
R=.superpowers/sdd/2026-08-03-phase-4b-procedures-and-conditions/phase-4b-final-review.md
awk -F'|' '/^\| \*\*T/ && $5 ~ /\*\*4c\*\*/          {n++} END {print n}' $R   # 11
awk -F'|' '/^\| \*\*T/ && $5 ~ /\*\*drop/            {n++} END {print n}' $R   #  8
awk -F'|' '/^\| \*\*T/ && $5 ~ /\*\*fix before merge\*\*/ {n++} END {print n}' $R  # 2
awk -F'|' '/^\| \*\*T/ {n++} END {print n}' $R                                 # 21
```

**The dispatch and the review both say twelve 4c items. There are eleven.**
11 + 8 + 2 = 21, which is the row count. The cause is visible: **T1-1** is ruled
`**drop**` and its reason ends "Re-derive in 4c if it recurs", so a search for
the token `4c` picks it up -- `grep -c '4c'` over the ruling column returns 12
and the drops come out one short at 7, which is exactly the review's summary
line. The rulings are unaffected; only the totals were. The gate section records
eleven, names T1-1 as the cause, and gives the anchored commands.

### `deferred-minors.txt` header — "the grep finds 16, `grep -in` finds 18", "byte-identical"

```bash
cd .superpowers/sdd/2026-08-03-phase-4b-procedures-and-conditions
grep -c  'minor (deferred)' progress.md    # 16
grep -ic 'minor (deferred)' progress.md    # 18
grep -n  'minor (deferred)' progress.md > /tmp/.../regen.txt
diff /tmp/.../regen.txt deferred-minors.txt && echo IDENTICAL
```

Run **before** the file was edited, the diff was empty: the generating command
reproduced the roll-up **byte for byte**, which is what lets the new header
state the generator as fact rather than as a guess. It deliberately does not
reproduce it any more -- the header and the five hand-added rows are the point,
and the header says so, so a later regeneration cannot quietly overwrite them
and look correct.

The two misses are independent and the header says both, because fixing only
the first would leave the file just as wrong:

1. **Case.** Task 7's blocks head with "**Minor (deferred)**".
2. **Shape.** Even case-insensitively, those two hits are block *headers*.
   The five items live inside running prose across several lines, so no
   line-grep recovers them at all. The tail of the file was read out by hand
   and is marked as not regenerable.

---

## Recorded for 4c, not fixed

Put in **`docs/superpowers/plans/phase-4b-gate.md`**, in a new section "What 4c
inherits, explicitly rather than implicitly", rather than in
`phase-4-exclusions.txt` or a handoff note. Three reasons:

* `phase-4-exclusions.txt`'s sections are language-behaviour rows with per-section
  pinning rules and gate criteria reading them. Harness debt (`String` vs
  `&'static str`, a test's placement, README drift) fits none of them and would
  dilute a file whose rows are load-bearing.
* **`.superpowers/` is gitignored.** `deferred-minors.txt`,
  `phase-4b-final-review.md` and this report are all untracked. A handoff note
  there would not survive a fresh clone, which makes it the exact failure
  `rust/CLAUDE.md` describes -- "a correction written into a review summary is
  read once and then lost".
* The gate document already carries "Recommendation for the 4c plan" and is what
  a 4c plan author reads.

Contents: **I4** (three `read_subset` copies, one tested) with a note that the
three *doc comments* differ even though the bodies are byte-identical, so a
mechanical move must not drop what is not shared; **M-c** (the README table);
and the **eleven** deferred minors ruled 4c, with each ruling verbatim. The
section also warns that `deferred-minors.txt` is not a safe input and why.

Two further notes went into the gate's "What went wrong, so the next gate
expects it" list: the five drift findings this review made, and the one-sentence
statement of the geometry -- the task that falsifies a sentence is never the
task that owns the file containing it.

---

## What this round did not close

* **The review's own "could not verify" stands.** Whether a *further* stale
  prose copy exists in a file no task touched is still unknown. The method that
  found these five only finds drift whose changed fact is already known. I did
  run one exhaustive sweep of my own -- `grep -rn 'unavoidab'` over `rust/`,
  `docs/` and the SDD directory -- which is why M-d can be stated as closed
  rather than as "the two the review found". The same sweep is not available for
  the general case.
* **T0-M7** ("two comments describe a stderr shape that no longer exists") is
  still not reproducible as filed; the review dropped it on that basis and this
  round found nothing new to attach it to.
* **M-e was solved by deletion, not by assertion.** `rust/CLAUDE.md` offers both
  and prefers the assertion. Asserting 730 would mean adding a gate assertion to
  a report-mode harness on the strength of a comment, which is more change than
  the finding is worth; the property plus a pointer is the cheaper true thing.
  If 4c wants the figure pinned, that is a new test, not a comment edit.
