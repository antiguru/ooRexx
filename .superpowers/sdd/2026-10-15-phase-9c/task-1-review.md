# Phase 9c Task 1 review

Range fe4867787..1db6c4c75. Read-only: all runs in scratch (`9c-r1/`), tree status clean.

## Verdicts

* Spec compliance: PASS.
* Task quality: PASS, with Minor findings only.

## Checks run

1. TSV diff. Dropping the message column from the new file (`cut -f1-3,5-`, comments removed) and diffing against the old body: identical, 1021 rows each. The comment diff is the five count edits plus the message paragraph and the derive command; no other header change.
2. derive.sh from a fresh scratch root over the committed TSV: rc 0, 15.6 s, output byte-identical (`cmp`) to the committed file.
3. Mutations in a `git archive` copy (shared nothing with the repo): (a) one recorded message for 10.1 altered: `every_secondary_message_matches_the_oracle` red; (b) `(6, 1)` removed from PENDING: same test red; (c) `(6, 1)` swapped for `(6, 2)`: both the main test and `every_pending_message_pair_still_differs_from_the_oracle` red. Baseline 30 passed before the mutations. Both directions are live.
4. Deletions. `no_message_the_corpus_raises_holds_an_unfilled_substitution` is the brief's deletion. `holds_placeholder` and its three tests were its only consumer (grep finds no other use; clippy would flag it dead), so removing them is right. What the deleted test covered that the new one does not: an `&N` in a rendered message of a PENDING row is no longer asserted absent. That is the brief's stated reversal (R6), and Task 2 targets exactly those rows, so not a finding.
5. derive.sh by reading and running.
   * Anchoring: `^Error (\d+)\.(\d+):  ` and `^Error \d+ running .* line (\d+):` under `/m`; across all 1020 stderr files no unindented line other than `Error` lines exists, so no clause echo is mis-read.
   * Empty message maps to `-`, the same as no message; the test then panics on a translation row with `-`, so it is loud, not silent. Zero such rows.
   * Multi-line: only the first physical line is kept. No row has a continuation line, but a future one would be truncated silently (Minor 1).
   * `?` path, run on a two-row scratch corpus: `say (` filled as `translation 35.1 1` with its message; `say 1` filled as `- ok - -`; a `::routine ... LIBRARY x` row filled as `install 98.903`. Correct. A cheap witness would be a two-line scratch corpus in the report, not a committed test; not required.
   * Tab-IFS `read` collapses empty fields; no field other than `program` can be empty, and `program` is last, so safe.
6. No em-dashes in the diff. No set sizes in new prose (the report's 92 is not committed prose). Comments state current behaviour.

## Findings

Critical: none.
Important: none.

Minor:
1. `rust/corpus/errors/derive.sh:12-26` keeps only the first `Error N.M:` line; a multi-line secondary message would be recorded truncated with no failure. Cheap fix: die if stderr holds a non-`Error`, non-indented line after the message.
2. `rust/corpus/errors/derive.sh:44-50` with a corpus lacking a `class` header line leaves `layout` unset; `set -u` aborts with an unhelpful message. Cosmetic.
3. `rust/crates/rexx-parse/tests/errors.rs` `PENDING` has no check that every pair is a pair some translation row raises; the converse test covers it (an absent pair is "stale" and red), so this is already caught.
4. Header in `parse-errors.tsv` still carries older counts of other things ("eight error programs", "seven `error(...)`", "three"); not Task 1's, noted only.
