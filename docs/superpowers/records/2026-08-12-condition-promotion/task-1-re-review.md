# Task 1 re-review: the fix round

Re-reviewed: `0459167cc` against `task-1-review.md`.
`cbdeaab71` is the controller's own plan edit and is out of scope.

* **Findings: all ten addressed.**
* **New statements: CHANGES REQUESTED.** One measured uniqueness claim was falsified by this same commit and left standing, and two mechanism sentences written this round misattribute where a line comes from.

---

## How this was verified

A detached `git worktree` at `0459167cc` under the session scratchpad, never in the repository tree, because a second implementer is editing it.
`ootest` is untracked, so it was symlinked into the worktree.

**Gates, run from `rust/` in that worktree, each status read unpiped:**

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, and no `warning:` line anywhere in the output |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1465 passed, 0 failed, 4 ignored** |

**Mutations, re-run rather than taken from the report.**
Every run was the whole workspace under `memcap 8G cargo test --workspace --no-fail-fast`.
Files were backed up with `cp`, restored from the backup, the restore checked with `sha256sum -c`, and `rexx-run` rebuilt after each restore.
No `git checkout --`.

| id | mutation | result |
|---|---|---|
| RRB | the fallback gains `Op::Condition` behind its `Op::EvalExpr` | exit 101, **two** tests red: `ir::golden_tests::a_condition_outside_the_native_set_stays_one_eval_expr` **and** `ir_dual::both_engines_agree_on_every_case_file`, the latter on the `if 1, 1` stanza, ir emitting a fourth `>>>   "1"` |
| RRC | the driver passes `checked: true` | `if 'x' then nop` exits **0** on the ir arm against **222** on the tree-walker; both 222 again after the restore and rebuild |
| RRA | `eval_condition`'s `let checked = matches!(...)` -> `let checked = false` | exit 0, **1465 passed, 0 failed** -- nothing red |

RRB and RRC are the two the dispatch asked for; RRA was re-run because the case file changed this round and the doc sentence it supports is a whole-workspace claim.
All three reproduce the implementer's numbers exactly.
One further probe, from the rebuilt clean binary: `trace r` over `if 0, 'x' then nop` exits **0** on both engines, printing `>>>   "0"` twice and never evaluating `'x'`.

---

## The ten findings

**F1 -- three discrimination claims in `tests/ir_dual_cases/conditions`. Addressed.**
All three passages are gone with the rows they described, and the new header says what the rows are rather than what they uniquely catch.
The one discrimination claim left in the file is on the `if 1, 1` row, and RRB confirms that row does catch the mutation it names.
See N1 for the uniqueness this leaves unresolved on the other side.

**F2 -- the false 34.6 re-check mechanism. Addressed in all three named places, and the replacement is true.**
`eval_logical_list`'s `Ok` path is `self.text(if holds { b"1" } else { b"0" })` unconditionally, so a list reaching the tail holds exactly `"0"` or `"1"`; RRA measures the consequence.
`eval_condition`'s doc, `condition_value`'s doc and the case file's 34.6 stanza no longer carry the mechanism, by reference or otherwise.
Two riders: N5 (a fourth copy of the same false mechanism survives in `run.rs`, unnamed by the review and untouched here) and N6 (`validates every element` is not true of a short-circuiting list).

**F3 -- "the oracle differential cannot see it". Addressed.**
Deleted with the header rewrite; the file now makes no claim about what any other harness can see.

**F4 -- `Loud::register_not_logical` and `Interp::register_holds`. Addressed, and the replacement holds.**
`compile` emits `Op::JumpUnless` at two sites, and the register each reads is written by `Op::Condition`, by the `Op::EvalExpr` of an `IF` whose condition declined, or by `Op::WhenTest`.
All three write `ObjRef::small_int(i64::from(holds))`, which is exactly what the new sentence says, and `register_holds` has one caller: the `Op::JumpUnless` arm.

**F5 -- `Op::Const`'s emission sites. Addressed, and the replacement holds.**
`Op::Const` is constructed at one site outside tests, `push_native`'s `ExprKind::Literal` arm.
The retained benchmark counts survive the widening for a reason worth recording: a bare number is `ExprKind::Constant` and compiles to `Op::LoadConstant`, so `x = x + 1` inside `varlookup`'s loop emits no `Op::Const`, and no bench program holds an `IF`.

**F6 -- `root_of`'s doc. Addressed.**
`root_of` has exactly one call site, `check_body`'s `value.map(root_of)`, which is what the new sentence names.

**F7 -- the two releases. Addressed, and the replacement holds.**
`push_native`'s `Binary` arm calls `registers.release(mark)` immediately after pushing `Op::TraceOperator`, and the `If` arm's own `registers.release(mark)` covers the register the condition landed in.

**F8 -- `eval_if_condition`'s doc. Addressed.**
Its two callers are `step`'s `If` arm and `eval_chunk_expr`'s `If` arm, which `Op::EvalExpr` enters -- and `compile` emits that `EvalExpr` for an `IF` only in the `else` of `native_shape`, which is what the new sentence says.

**F9 -- `push_native`'s `unreachable!` message. Addressed.**

**F10 -- the cardinality nit. Addressed.**
The header rewrite dropped it and introduced no new count.

## The controller's three rulings

* **c1/c2/c3/c5 dropped, c4/c6 kept.** Done -- and c7, the called-label live-indent row, went too.
  That follows the ruling's "keep c4 and c6" and the review's own measurement that c7 adds nothing, so the tree is right; the report's fix-round sentence enumerating the deletions is not (N7).
* **The plan's Step 10 corrected in the same commit.** Done: `0459167cc` touches the plan in exactly one hunk, Step 10, and the list it now gives is the four stanzas the tree holds. The paragraph under it overstates its method (N4).
* **A traced declining condition row added.** Done, and as two rows rather than one. The second is the one that carries the instrument: RRB reddens on `if 1, 1` and not on `if .nil`, exactly as the file says, because the refusing row raises before a second validating op could run.

---

## New or surviving false statements, most severe first

### N1 -- this commit falsified a measured uniqueness claim in a test it did not edit, and left it standing

`ir/golden_tests.rs`, `a_condition_outside_the_native_set_stays_one_eval_expr`:
*"Emitting a `Condition` behind the fallback's own `EvalExpr` -- the shape `push_value` would have produced -- reddens this test and nothing else. Measured 2026-08-12, the whole workspace under `--no-fail-fast`."*

Measured again at this commit, that mutation reddens **two** tests: this one and `ir_dual::both_engines_agree_on_every_case_file` (RRB).
The second catcher is the `if 1, 1` row **this commit added**, and the round's own table records both -- so the falsifying evidence was in hand when the sentence was left alone.
It is the same defect class as F1, in the one test the review had singled out as earning its place, and the two sentences now contradict each other across two files: the golden doc says nothing else catches it, and the case file's `if 1, 1` stanza says *"this row is what catches a second validating op emitted behind the fallback"*.

The fix is one clause in each: drop the uniqueness from the golden doc rather than restating it as a list of files, and let the case-file row say it catches the mutation rather than that it is what catches it.

### N2 -- "all from inside `eval`" misattributes the line the mutation above is about

`tests/ir_dual_cases/conditions`, the `if 1, 1` stanza: *"a comma list prints a `>>>` per element and one more for the list's own result, all from inside `eval`."*

`eval_logical_list` calls `self.trace_result(indent, &text)` once per element it evaluates and nothing else.
The list's own result line is `condition_value`'s `ConditionTrace::Result`, which is *why* a second `Op::Condition` prints a fourth one -- and `run.rs`'s own pre-existing test doc on `a_comma_list_conditions_own_elements_each_trace_their_result_under_trace_r` says so in as many words ("the third of which was already covered (`eval_condition`'s own `ConditionTrace::Result`/`Keyword` fires on whatever this function returns, list or not)").
Measured directly: `trace r` over `if 0, 'x' then nop` prints two `>>>` lines for a two-element list, because the short circuit evaluates one element and the second line is the tail's.

### N3 -- "the whole line sequence comes from the one `Op::EvalExpr`" names the wrong producer for most of the block

`tests/ir_dual_cases/conditions`, the `if .nil` stanza.

Of the five recorded `err>` lines, the `EvalExpr` produces the `>>>` and the raise.
The first `*-*` is the clause echo `compile`'s `push_echo` emits as `Op::TraceClause`, ahead of the `EvalExpr`; the second `*-*` and both `Error` lines are `Raised::report`'s, which prints a clause echo per `site.sites` entry with trace off as well as on (its own doc's first bullet).
The plan's wording for the same row -- "the lines the one `Op::EvalExpr` still owes when no `Op::Condition` follows it" -- is the accurate one and was available to copy.

### N4 -- "measured out rather than argued out" is true of half the rows it covers, and one copy is in the commit message

The plan's new Step 10 paragraph: *"the four rows that went were measured out rather than argued out. Each was written, captured from the oracle, and then held out of the directory while the mutation it was supposed to catch was re-run over the whole workspace"*.
The commit message carries the same sentence.

Two of the four kinds were measured out that way: the 34.1 row (the review's MXC) and the live-indent row (MXD).
The plain true/false rows and the `trace r` row had no mutation of their own on either side; they were dropped because they are the same *shape* as rows `BRANCH_CASES` and `trace-settings` already hold, which is an argument -- and the same paragraph's own last clause says so, two lines later.
Separately, "the four rows" counts kinds, not rows: five stanzas were deleted.

### N5 -- a fourth copy of the F2 mechanism survives, in a file this commit edited

`run.rs`, `if_condition_that_is_a_comma_list_raises_34_6_not_34_1`:
*"A comma list is 34.6 regardless of which element fails, never 34.1 -- re-checking `eval_logical_list`'s own result would misreport it."*

Same false mechanism, pre-existing, and not named by the review -- whose "the claim appears three times in the tree" was itself one short.
Not introduced here, but F2's premise is that the mechanism is false everywhere, and the round corrected the three sentences it was handed rather than the claim.

### N6 -- "validates every element" is not true of a short-circuiting list

`eval_condition`'s rewritten doc.
`eval_logical_list` breaks out of its loop on the first element that is `0`, so a later element is never evaluated and never validated: measured, `if 0, 'x' then nop` exits 0 on both engines and raises nothing.
The wording is carried over from the sentence this round replaced, and the conclusion the doc draws from it -- that the result arrives at the tail as exactly `b"0"` or `b"1"` -- is unaffected.

### N7 -- the report's deletion list is one row short

`task-1-report.md`'s fix round: *"c1, c2, c3 and c5 deleted, c4 and c6 kept"*.
c7 was deleted as well, which the same section's "now holds four stanzas" list and the plan both get right.

---

## The deliberate omission

Removing the header sentence that named what `BRANCH_CASES`, `trace-settings` and `trace_oracle` hold is the right call, and the rule cited is the right one.
`rust/CLAUDE.md`: *"A comment ... may not say how many call sites there are, what every other site does, or what a gate currently totals. Those change without the sentence being reread. If such a claim is load-bearing, assert it in a test; if it is not, delete it."*
A justification for this file's scope built from the contents of three other test files is exactly a mutable in-repo aggregate, and moving it to the plan and the report is what the neighbouring "history belongs in the commit message and the SDD ledger" rule asks for.

The file still explains its own scope: what the promotion changed, that its rows are byte-for-byte transcripts rather than a coverage set, which shapes they hold, and how to regenerate one from the oracle.
Worth noting that N1's honest fix would reintroduce a claim of exactly this kind if written as a cross-file list, which is the argument for deleting the uniqueness clause rather than updating it.

---

## Verdict

**Another round.** N1 is a false measured claim in a test doc, N2 and N3 are false mechanism sentences written this round, and N4 is a false method claim in the plan.
All four are comment or prose edits with no behaviour behind them; N5 and N6 are cheap while the file is open, and N7 is a ledger correction.
Nothing in the shipped behaviour is wrong, the gates are green at `0459167cc`, and every mutation the round claims reproduces exactly.
