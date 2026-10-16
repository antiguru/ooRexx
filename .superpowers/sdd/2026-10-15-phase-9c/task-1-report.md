# Phase 9c Task 1 report

## Commands (run from `rust/`, statuses unpiped)

* `corpus/errors/derive.sh $S/run1 > $S/new.tsv` from a fresh empty dir (`S` = the 9c-t1 scratch root): rc 0, wall 15.5 s (1020 oracle rexxc runs, each from its own directory). A second run over the new file: rc 0, 15.6 s, output byte-identical to the committed file (idempotent, accepts either column layout).
* `cargo fmt --all --check`: 0. `cargo clippy -j 4 -p rexx-parse --all-targets -- -D warnings`: 0.
* `cargo test -j 4 -p rexx-parse --test errors`: 0, 30 passed.

## TSV diff

`git diff --stat -- corpus/errors/parse-errors.tsv`: 1033 insertions, 1025 deletions (every row gains a field; `cut -f1-3` of old and new is identical, so class, expect and line are unchanged). Header edits: the "four tab-separated fields" and both "nine rows" phrasings dropped, the message-column paragraph and the derive command added, column header gains `message`.

## PENDING

92 (major, sub) pairs, derived from the test's own failure list with `PENDING = &[]`: run `every_secondary_message_matches_the_oracle`, extract `row N: A.B renders` pairs from the failure output, sort numerically, paste. No translation row was accepted by the parser (no `<accepted>` in the failures).

## Converse check, shown red

With `(10, 1)` (a pair that currently matches) added to PENDING, `every_pending_message_pair_still_differs_from_the_oracle` fails (rc 101): `PENDING entries that now match: [(10, 1)]`. Reverted by restoring a saved copy; the tree test is green again (rc 0).

## Other changes

Deleted `no_message_the_corpus_raises_holds_an_unfilled_substitution` and, with it, `holds_placeholder` and its three tests (their only consumer). Test names: `every_secondary_message_matches_the_oracle`, `every_pending_message_pair_still_differs_from_the_oracle`.

## Oracle disagreements

None: derive.sh checked `expect` and `line` for every row against the oracle and failed on none. No `?` rows existed, so the fill-from-oracle path is untested here. Messages carry non-ASCII only as `\xNN` (one row, 13.1, `\xc3\xa4`). No message was empty. Only the first physical line after `Error N.M:  ` is recorded; no row's message spans lines.

## Fix round 1

* `derive.sh` now dies, naming the row and the oracle's stderr file, when text follows the `Error N.M:` line. Checked on a two-line stderr (perl exits 255, "the message spans more than one line") and on a one-line one (accepted).
* `derive.sh` now fails with "row N: no class header line before the first row" when the corpus lacks the header (checked on a one-row file without it, rc 1).
* Header counts dropped from `parse-errors.tsv`: "four steps", "eight error programs", "seven `error(...)`", "three `logical_error(...)`", "Two rows", "Five more", "all five kinds". Other wording unchanged.
* Full `derive.sh` rerun after the header edit: rc 0, 15.2 s, `cmp` against the committed TSV identical.
* `cargo test -p rexx-parse --test errors`: rc 0, 30 passed.
