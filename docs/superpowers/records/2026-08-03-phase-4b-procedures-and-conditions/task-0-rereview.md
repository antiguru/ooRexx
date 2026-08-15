# Task 0 re-review -- fix round 1 (BASE `f1527d44`, HEAD `1b7c4b1f`)

Scoped re-review of the six Important findings from `task-0-review.md`. Every
claim below was re-derived by reading the fix diff in full and by running
mutations against the fixed tree myself (not taken from `task-0-report.md`'s
"Round 1" account). Tree confirmed clean (`git diff HEAD --stat` empty) before
starting and after every mutation.

## Verdicts

**I1 -- ADDRESSED.** `run.rs`'s four "takes the loud path" tests
(`do_with_takes_the_loud_path:5069`,
`do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on:5078`,
`do_over_a_stem_target_takes_the_loud_path:5218`,
`do_over_a_parenthesised_stem_target_is_also_caught:5245`) each now assert
`loud.message == "DO is not implemented"` beside the existing
`Failure::Loud(_)` match. Mutation F (delete the `None`/`"4a"` carve-out in
`owned_message`, `lib.rs:595`) re-run by me: **caught**, all four tests fail
with `left: "DO is not implemented (4a)"` / `right: "DO is not implemented"`,
exit 101.

**I2 -- ADDRESSED.** `loud.rs:606-607`'s `every_out_of_scope_variant_fails_loudly`
now builds `format!(" is not implemented ({})", witness.owner)` and asserts
`stderr.trim_end().ends_with(&want_suffix)`, replacing the old
`stderr.contains(witness.owner)`. Mutation G (rewrite the shape to
`"[{owner}] {name}: unimplemented"`) re-run by me: **caught**, all 27
witnesses fail with e.g. `Call::Named (4b): stderr does not end with " is not
implemented (4b)": "rexx-exec: [4b] CALL: unimplemented\n"`, exit 101.

**I3 -- NOT ADDRESSED.** The fix adds documentation in three places
(`lib.rs:686-692`'s doc on `expr_owner`, `loud.rs:381-393`'s comment beside the
`VariableReference` witness, `owners.rs:518-523`'s pinned-items note) all
repeating the claim that no witness can reach `expr_owner`'s
`VariableReference` arm without `CALL` firing first, because "its only legal
position is a call argument list." **That claim is false, and a witness that
reaches the arm exists today.** `say >x` (or the bare `r = >x`) parses and
evaluates with no `CALL` instruction anywhere in the program, and hits
`expr_owner(ExprKind::VariableReference)` directly through `SAY`/assignment
evaluation, which is 4a's own and already implemented. I verified this two
ways:
- Ran `cargo run -p rexx-exec --bin rexx-run -- say-gt-x.rex` (`say >x` alone):
  stderr `rexx-exec: a variable reference is not implemented (4b)`, exit 120
  (`NOT_IMPLEMENTED_EXIT`) -- no `CALL` in the source at all.
- Mutated `expr_owner`'s `VariableReference` arm to `Some("Phase 7")` and
  re-ran the same program: stderr changed to
  `rexx-exec: a variable reference is not implemented (Phase 7)`, proving the
  output is actually sourced from `expr_owner`, not from `instruction_owner`
  or anywhere else.

  This directly contradicts `ast.rs`'s cited "error 20.930" reasoning: that
  error is about what token follows the `>`/`<` prefix (it must be a
  `Variable` or `Stem`, or parsing fails), not about which instruction or
  expression context `>x` as a whole term may appear in. The parser accepts
  `>x` in any expression position (`expr.rs:804-807`'s `message_subterm`
  handles it before falling through to the general operator/term loop), and
  `eval.rs:22-25`'s own module doc already says `VariableReference` "still
  fails loudly through the existing, exhaustive `form_name`" for *any*
  evaluation, not only one reached via `CALL`.

  So the argument in the fix is not merely undocumented risk -- it is
  incorrect, and the review's own escape hatch applies: "If a witness could
  reach the arm today, the argument is wrong and I3 is not closed." The
  correct fix was to replace (or add to) the `VariableReference` witness in
  `loud.rs`'s `EXPR_WITNESSES` with `"say >x\n"` (matching the existing
  convention, "Wrapped in `SAY`... except `VariableReference`", which itself
  no longer needs the exception once this is fixed), not to document the arm
  as unreachable. As it stands, mutation C (`expr_owner`'s `VariableReference`
  arm `Some("4b")` -> `Some("Phase 7")`) is still **not caught** -- I re-ran it
  myself, full suite green, exit 0 -- and it did not need to stay that way.

**I5 -- ADDRESSED.** `coverage.rs:428-451` adds
`read_subset_unions_two_files_first_seen_order_deduplicated`: writes two temp
files with an overlapping entry (`two.rex` in both) and asserts the exact
union `["one.rex", "two.rex", "three.rex"]`, first-seen order, the repeat
dropped. Read and confirmed it exercises the multi-file path no other test
touches; ran it as part of the full suite (passes).

**I6 -- ADDRESSED.** `instruction_owner`/`expr_owner` (`lib.rs:631`, `:694`)
now return `Option<&'static str>` with `None` meaning "implemented here";
`owned_message` (`lib.rs:595-598`) matches on the `Option` instead of
comparing against the literal `"4a"`. No remaining reference to `"4a"` as a
sentinel anywhere in `instruction_owner`/`expr_owner`/`owned_message`
(checked by grep). This is the exact `Option<&'static str>` shape the
controller's adjudication specified, not the rejected empty-string-constant
alternative.

**I4 -- confirmed out of scope, correctly untouched.** `git diff f1527d44..1b7c4b1f --stat -- docs/` is empty; the fix commit touches only
`rust/crates/rexx-exec/{src/lib.rs,src/run.rs,tests/coverage.rs,tests/loud.rs,tests/owners.rs}`. Not otherwise assessed, per instruction.

## Mutation re-verification (measured by me, independently of the report)

| # | Mutation | Expected | My result |
|---|---|---|---|
| F | `owned_message` always appends the owner (deleted the `None` carve-out) | fail | **caught** -- 4 tests in `run.rs` fail, exit 101 |
| G | Shape rewritten to `"[{owner}] {name}: unimplemented"` | fail | **caught** -- `every_out_of_scope_variant_fails_loudly` fails on all 27 witnesses, exit 101 |
| C | `expr_owner`'s `VariableReference` arm `Some("4b")` -> `Some("Phase 7")` | fail, or explain | **not caught, full suite green, exit 0** -- and the explanation offered for why is wrong (see I3 above); a witness (`say >x`) exists today that would have caught it |

Tree restored and confirmed clean (`git diff HEAD --stat -- rust/crates/rexx-exec/` empty) after each of the three.

## `run.rs` scope question

**Not scope creep.** The original review's own proposed fix for I1
(`task-0-review.md:109-118`) explicitly named `run.rs`'s test module as the
fix location ("add one assertion beside the existing four, e.g. in `run.rs`'s
test module... for at least `do_counter_...` and
`do_over_a_stem_target_...`"). `run.rs` not being in Task 0's original file
list is expected -- Task 0's brief predates this review round and could not
have anticipated a fix to a gap the review itself found. Touching `run.rs`
here is the review's prescribed remedy, not an expansion beyond it. The fix
applied the assertion to all four existing tests (a superset of the review's
"at least two"), which is a reasonable, low-risk completion of the same fix,
not new scope.

## Full-suite re-verification (clean tree, HEAD `1b7c4b1f`)

All exit statuses read unpiped.

| Check | Command | Result |
|---|---|---|
| Full suite | `cargo test -p rexx-exec` | exit **0**; grep for `FAILED` in the log: none |
| Corpus | inside the above | `29 of 29 matching`, unchanged |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0** |
| Format | `cargo fmt --all --check` | exit **0**, no diff |

No occurrence of `cargo fmt --edition 2024 --check` anywhere in the fix diff
or the Round 1 report section (M6 was correctly left un-repeated).

## New findings introduced by this fix

**Important.** I3's "fix" plants an incorrect factual claim in three places
instead of one (`lib.rs`, `loud.rs`, `owners.rs` all now assert the arm is
unreachable "until Task 3"). Because the claim is wrong, this is worse than
the pre-fix state in one respect: before this round, the arm's untested
status was visibly a gap (I3 said so); now it reads as an intentionally
deferred, well-understood limitation, which will make a future reader less
likely to question it. Concretely, this is the same finding as I3 above, not
a distinct defect -- it is not double-counted in the six verdicts.

No other new findings. The `Option` refactor (I6) touched every call site of
`instruction_owner`/`expr_owner` and I confirmed by grep there are exactly two
(`lib.rs:444`, `:468`), both updated; nothing outside `lib.rs` calls either
function. The new `coverage.rs` test (I5) uses fixed temp-file names under
`std::env::temp_dir()`, matching the existing convention `loud.rs` already
uses for its own scratch file -- a pre-existing pattern, not a new risk this
fix introduced.

## Requirements-table delta

Only the six Important rows change status from the original review; nothing
else in `task-0-review.md`'s requirements table is affected by this round.
