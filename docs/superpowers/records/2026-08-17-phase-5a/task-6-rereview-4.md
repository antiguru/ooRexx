# Task 6, fix round 4 -- re-review

Fix base `6572d67cc`, head `c27b46ea4` (confirmed HEAD, full hash
`c27b46ea475d54b8280c408801eee6c9d4c09c77`). Working tree confirmed clean throughout;
nothing in this checkout was mutated.

## Verdicts

**NEW-1** ADDRESSED. `rust/corpus/phase-5a.txt:213-219`. "the other four adapters
already have. One program stands for the six non-strict comparison operators..."
replaced with "...the other adapters have. One program stands for the operators that
reach `compare_values`'s numeric path -- `>` here, and an operator is on that path when
`is_comparison` admits it and `is_strict_compare` rejects it, both in `eval.rs`...".
Verified: `is_comparison` at `eval.rs:1902`, `is_strict_compare` at `eval.rs:2018`, exact
line numbers as cited. Verified the property is the deciding one: `compare_values_body`
(`eval.rs:1353`) sets `let strict = is_strict_compare(op)` and calls `self.to_number`
on both operands only when `!strict`; `compare_values_body` is reached only for `op` where
`is_comparison(op)` (dispatch arm `eval.rs:1585`). So "on the numeric path" =
`is_comparison(op) && !is_strict_compare(op)`, which enumerates to exactly the ten
operators the finding measured (`>` `<` `>=` `<=` `=` `\=` `\>` `\<` `<>` `><`). No count,
no list, no historical framing in the replacement.

**NEW-2** ADDRESSED. Same file/hunk: "the other four adapters" -> "the other adapters".
Set-size phrase gone, "already" (decoration) struck too.

**NEW-3** ADDRESSED. `rust/corpus/lang/operator_frame_stem_compare_overflow.rex:6-9` and
twin `rust/crates/rexx-parse/tests/sourceline_oracle/operator_frame_stem_compare_overflow.txt:7-10`.
"`<`, `>=`, `<=`, `=` and `\=` all reach..." (list, omitting 4 of 10 members) and
"Strict `==`/`>>`" (an incomplete 2-of-8 example pair) both replaced: "an operator joins
it by being a comparison `is_strict_compare` (`eval.rs`) rejects" and "The strict family".
Both enumerations gone (the second one wasn't even flagged by NEW-3, but the round fixed
it too). Reflow verified: `.rex` is still 17 lines (`wc -l`), line 17 is still
`say s. > 1`. Twin's `count 17` is correct and its body (line 2 onward) is byte-identical
to the `.rex` (`diff` confirms). `cargo test --release -p rexx-parse --test
sourceline_oracle` passes.

**NEW-4** ADDRESSED. `eval.rs:1099-1105` (`blame_stem_forwarded_operator`'s doc). The
roll-call of named callers is gone, replaced by a membership rule -- "A caller is an
adapter that evaluates one operator whose receiver is `value`... `Interp::header_number`
(`run.rs`) among them" -- with "among them" marking it as a partial example, not an
exhaustive list. Verified against the tree: exactly 5 call sites exist
(`eval.rs:822,977,1322,1476`, `run.rs:7298`), each `if result.is_err() {
self.blame_stem_forwarded_operator(...) }` immediately after a `_body` call, matching the
report's own search. The fifth call site this round added (`compare_values`, `eval.rs:1322`)
is covered by the rule without needing to be named. No count, no historical framing.

**NEW-5** ADDRESSED. `eval.rs:973-977`. "unchanged... exactly as before... the removed
inline call inside `arith_left_operand`" (naming code not in the tree) replaced with "The
receiver test is `blame_stem_forwarded_operator`'s own, so this site gates on failure
alone and hands it every failing path here, leaving a non-stem receiver for that predicate
to discard." Deciding test applied and passes: no historical framing to strike, and the
sentence states a present fact. Verified against code: the site's only gate is
`result.is_err()` (`eval.rs:976`), and `blame_stem_forwarded_operator` opens with
`if !self.is_stem_receiver(value) { return; }` (`eval.rs:1115`) -- exactly the "leaving a
non-stem receiver ... to discard" claim.

**NEW-6** ADDRESSED. Report `:436` and `:727` (report's own line numbers, both confirmed by
grep) corrected in place, not merely contradicted later, each tagged "(Sentence corrected
in fix round 4; the finding is fix round 3's finding 2.)". `:436`'s replacement ("A path
that succeeds pays the wrapper's own `is_err()` test... what success skips is the shape
check and the frame render, not the branch") and `:727`'s ("What this round's diff added
*does* run on the path `arith`'s benchmark exercises... `bench-programs/arith.rex` is all
`arith_general` work") both verified against code: `arith_general`'s `if result.is_err()`
runs unconditionally after every `arith_general_body` call, and `bench-programs/arith.rex`
contains only `+ - * / ** //` (no comparison, no non-arithmetic operator). Round 3's own
citation of the second line updated from `:724` to `:727`, confirmed correct for the
lengthened file.

**NEW-7/NEW-8** ADDRESSED. All three false summarizing sentences ("again min equal to max
on all four rows", "the same four values to six decimal places", "Only `alloc4c/ir/small`
differs from an exact `1.000000`") no longer appear as live claims -- each survives only
as a quoted "Was:" excerpt inside the fix-round-4 section. The sitting table itself
verified against `rust/bench-baselines/phase-5a-arms.tsv`: all 24
`pinned>head across_builds ... instructions:u` rows for commit `56c842cb0` match the
report's table exactly (median, min, max) for every axis/engine/size cell. The surviving
conclusion ("arith did not move ... reaches `compare_values` on none") verified true:
`rust/bench-programs/arith.rex` has no comparison operator (`+ - * / ** //` and a
`do i = 1 to n` loop only), and `run.rs:8916-8925` is confirmed to be the match arm
answering the loop bound via the small-integer fast path or `controlled_within_wide`,
neither of which calls `compare_values`.

## Comment-prose pass (collapsed-comment diff, house method)

Collapsed each contiguous comment block to one line for base and head across all three
prose-bearing files (`eval.rs`, `phase-5a.txt`, `operator_frame_stem_compare_overflow.rex`)
and diffed. Exactly 4 comment blocks changed (2 in `eval.rs`, 1 in `phase-5a.txt`, 1 in the
`.rex`), matching the 4 findings above one-to-one. Read every new line in full (not
keyword-filtered): no new set-size phrase, no new member list, and no new historical
framing ("used to"/"no longer"/"previously"/"formerly") found anywhere in the diff. A
targeted scan of every added (`+`) line for number-words and historical-framing phrases
turned up only pre-existing/non-offending text ("One program stands for the family" is
unchanged base text; "one operator"/"any one step" describe a per-call cardinality, not a
stale enumerable count).

ASCII check: every byte in the diff file is <= 0x7F (`grep -nP '[^\x00-\x7F]'` and a
byte-level Python scan both found nothing).

## Code-unchanged claim

Verified against the diff, not just the report's word: 4 files changed, 26
insertions/26 deletions total. Every changed line across all four files is either a Rust
`//` comment (`eval.rs`), a corpus manifest `#` comment (`phase-5a.txt`), a `.rex` `/* ...
*/` comment (`operator_frame_stem_compare_overflow.rex`), or the fixture copy of that same
comment (`sourceline_oracle` twin). No non-comment line changed in any file. `eval.rs`'s
hunk is symmetric (19/19); `.rex`/twin symmetric (10/10, both stay 17 lines);
`phase-5a.txt` is 6 removed / 7 added (net +1 comment line, a `#`-prefixed manifest
comment with no line-sensitive consumer -- confirmed by the passing corpus/coverage
tests that read that file). Working tree at HEAD is clean (`git status --porcelain`
empty) confirming no local drift from the committed diff.

## Reflow claim

Confirmed correct in full: `.rex` file line count unchanged at 17, `say s. > 1` still on
line 17, twin's `count 17` line correct, twin body byte-identical to the `.rex` via
`diff`/`cmp`-equivalent comparison, and `cargo test --release -p rexx-parse --test
sourceline_oracle` passes (`1 passed; 0 failed`).

## Gate verification (run independently, unpiped, from a scratch copy with `interpreter/`,
`oodocs/`, `ootest/`, `docs/`, `samples/`, and other sibling top-level directories
symlinked in -- the repo checkout itself was never touched)

- `cargo fmt --all --check` -- exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0.
- `cargo test --release --workspace` -- exit 0, no `test result: FAILED` lines.
- `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit 0, `118 of 118 matching`,
  no `test result: FAILED` lines.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` -- exit 0,
  `118 of 118 matching`, zero `test result: FAILED` lines.

All five match the report's claims. (Two environment-only failures were hit and fixed
before these numbers: `rexx-bench`'s baseline-caveat test and `rexx-parse`'s
`every_sample_program_parses` both need sibling directories -- `docs/`, `samples/`, etc. --
symlinked next to the copied `rust/` directory, which the task brief's own copy recipe
under-specifies; not a defect in the round, an artifact of the isolated build setup.)

## Residual claim

Confirmed: `git show -s 56c842cb0` has "the sixth operator family that needs a number" in
its subject and "the six non-strict operators that reach it" in its body. Unreachable
without a history rewrite (round 4 did not and could not touch it). The tree (post round
4) is correct per NEW-1's verification above -- the residual is real and the report's
"tree is right" claim holds.

## New breakage

None found. No new false statement, no new banned set-size or member-list phrase, no new
historical framing, no code change, no reflow drift, no ASCII violation, no gate failure
attributable to the round's own diff.

## Round verdict

PASS. All 8 findings addressed correctly; the fix round introduced no new defects of the
kind the last three rounds each introduced. The mechanism remains unchanged and verified
(gates green, corpus 118/118, sourceline_oracle green); this was in fact prose-only, and
the prose is now materially more accurate than at any prior round for this file set.
