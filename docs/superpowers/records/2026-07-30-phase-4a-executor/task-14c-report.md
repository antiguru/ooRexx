STATUS: DONE (committed ba126cef)

# Task 14c report -- fold the driver fix in, fix README drift

Scope, per the team lead's dispatch:

1. Fold the `SIGNAL ON SYNTAX` + `LINEIN` fallback (found and used locally
   in Task 14b, for `trace_numeric_request.rex`, whose whole point is a
   prolog that raises 24.901 before the stock `.Package~new`-based driver
   can print anything) into `rust/crates/rexx-parse/tests/sourceline_oracle.rs`'s
   module documentation, as the documented driver rather than a footnote.
   Carry forward: that it's verified not to change behaviour for
   non-crashing files, and *why* the wall exists (any witness program
   whose point is crashing its own prolog hits it, and this phase has
   started writing such programs on purpose). Note the adjacent hazard in
   the same breath: `.Package~new` executing a repository file's prolog is
   the project's standing "never do this" rule, and this driver is the one
   place that already does it -- safe so far only because corpus prologs
   have been trivial, and this is the first file where the prolog does
   something, with no guarantee the next one won't do something worse than
   raise.
2. Fix the README drift I flagged but didn't touch last round: the "Phase
   4a additions" and `num/` tables in `rust/corpus/README.md` are behind
   `phase-4a.txt`'s actual membership after Task 14b removed three programs
   from the subset and added three new ones.

File scope this time: `rust/crates/rexx-parse/tests/` and `rust/corpus/`
only. Not `rexx-exec` -- two live lanes there per the dispatch.

**On `STATUS`:** the team lead's correction is noted and taken seriously --
last round's report read `STATUS: DONE` while its body was still the
unfilled plan skeleton and nothing was committed, which could have caused
a bad rescue commit. This file states `IN PROGRESS` now and stays that way
until the work below is actually committed; only then does the line
change.

## Plan

1. Read `sourceline_oracle.rs`'s current module doc in full, and design the
   fold-in so the documented driver *is* the crash-safe one, not a primary
   path with a caveat bolted on -- since the team lead's framing is
   explicit: "A driver that works only on programs that succeed is not a
   driver for a corpus that deliberately contains failures."
2. Re-verify the fallback driver once more from this file's actual current
   state (not from memory of Task 14b) against `trace_numeric_request.rex`
   and at least one ordinary file, before writing it into the doc comment
   as fact.
3. Write the doc comment: the driver script itself, the reasoning for the
   `SIGNAL ON SYNTAX` wrap, the `.Package~new`-executes-the-prolog hazard
   stated explicitly (not just implied by the trap), and why the fallback
   is safe for the files it needs to be safe for (no CRLF/CTRL-Z/missing-
   trailing-newline edge cases, the one thing that would make a `LINEIN`
   substitute unfaithful to `~source`).
3. Diff `rust/corpus/phase-4a.txt` against `README.md`'s tables; update
   both the "Phase 4a additions" table and the `num/` table to match
   current membership.
4. Run `cargo test -p rexx-parse` to confirm nothing broke.
5. Stage only my paths, commit once, flip this file's STATUS to DONE only
   after the commit exists.

## What was done

### 1. Folded the fallback driver into `sourceline_oracle.rs`

Re-verified the driver once more from a fresh scratch copy (not trusted
from memory of Task 14b) before writing anything into the doc comment:
ran it against `trace_numeric_request.rex` (fallback path, `%SRCG%COUNT
10`, matching `wc -l`), against `prefix_dotvar_logical_over_label.rex`
(primary path, `20`, matching `wc -l`), and against
`no_trailing_newline.rex` (primary path, `7` -- **not** `wc -l`'s `6`,
because that file's entire purpose is a final line with no terminator,
which `wc -l` cannot count and `~source` correctly does). That third case
is the one that would have broken silently if the `SIGNAL ON SYNTAX` wrap
somehow changed which path a normal file takes; it didn't.

Replaced the driver script and its surrounding prose in the module doc
comment. What it now says, in order: the driver already violates the
project's "never instantiate `.Package~new` on a repository file" rule
(stated explicitly, not left implicit in the trap), and has been safe only
because corpus prologs used to be trivial; `trace_numeric_request.rex` is
the first with a prolog that does something (raises `24.901`), and this
phase now writes such witness programs on purpose, so a driver that only
works on succeeding programs isn't a driver for this corpus anymore; the
fallback is `LINEIN()` in a loop rather than a second `.Package` call,
specifically so a crashing prolog can't run twice; and the fallback is
faithful to `~source` only for files without CRLF terminators, an embedded
`CTRL-Z`, or a missing final newline -- naming `no_trailing_newline.rex`
by name as the file that must never be the one whose prolog is made to
crash, since that would be exactly the case where `LINEIN` and `~source`
diverge.

### 2. Fixed the README drift

Diffed `phase-4a.txt`'s actual membership against `README.md`'s tables.
Two gaps, both now closed:

* A new "Phase 4a additions -- closing criterion 1's variant-coverage gap"
  subsection and table for Task 14b's three programs, in the same place
  and format as Task 14a's subsection.
* A note above the `num/` table stating that `digits_rounding.rex`,
  `exponential.rex` and `operators.rex` are listed there as corpus
  programs but are not in the Phase 4a subset, and why (`ExprKind::List`
  from a comma in `SAY`) -- the table itself is unchanged, since all three
  are still real corpus files that belong in it; only their subset
  membership changed.

Dropped an initial draft's reference to `criterion-1-coverage-gap.md` by
name: that file lives under `.superpowers/sdd/`, which is gitignored, so a
committed `README.md` pointing at it by path would be pointing at nothing
for anyone who doesn't have this session's scratch tree. Described the
same fact (a coverage-gap analysis found nineteen unconstructed variants)
without depending on the ephemeral file surviving.

## Verification run

`cd rust && cargo test -p rexx-parse --no-fail-fast`: every test binary
passes, including `sourceline_matches_the_interpreter_for_every_corpus_program`
(confirms the doc-comment-only change to `sourceline_oracle.rs` didn't
touch anything the test actually checks) and
`every_variant_is_constructed_by_the_corpus_and_samples`. `cargo clippy -p
rexx-parse --all-targets -- -D warnings` and `cargo fmt --check -p
rexx-parse`: both clean.

## Files changed

* `rust/crates/rexx-parse/tests/sourceline_oracle.rs`: module doc comment
  only -- the driver script, the reasoning for the `SIGNAL ON SYNTAX`
  wrap, the `.Package~new` hazard stated explicitly, and the fallback's
  safety conditions. No code logic touched.
* `rust/corpus/README.md`: one new subsection/table (Task 14b's three
  programs) and one new note (the three `num/` files excluded from the
  subset).

Not touched: `rexx-exec` (two live lanes, per the dispatch) and
`rust/corpus/phase-4a.txt`/`rust/corpus/lang/*.rex` (Task 14b's own
commit, `eeca9951`, already landed and is not this task's to redo).
