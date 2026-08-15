# Task 1 re-review 2: the second fix round

Re-reviewed: `d07a8988e` against `task-1-re-review.md`'s N1-N7.
Scope as dispatched: were the seven addressed, and did this round introduce anything false.
Design, and everything the first re-review closed, are out of scope.

* **N1-N7: all seven addressed**, five of them cleanly.
* **New statements: CHANGES REQUESTED.** Two of the replacements carry a claim their own evidence does not support -- a quotation attributed to a commit message that does not contain it, and a probe cited for non-evaluation that this tree already retired for exactly that use.

---

## How this was verified

A detached `git worktree` at `d07a8988e` under the session scratchpad, never in the repository tree.
`ootest` is gitignored, so it was symlinked in.
`CARGO_TARGET_DIR` was set under the scratchpad, and `rust/target` symlinked to it -- without that symlink `rexx-bench-suite`'s `every_blocked_axis_still_fails_on_this_crate` fails, because it looks for `target/debug/rexx-run` by relative path. That failure is an artifact of the out-of-tree target directory and nothing else; it is recorded here so the number below is not read as a flake.

**Gates, from `rust/` in that worktree, each status read unpiped:**

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no `warning:` line in the output |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1468 passed, 0 failed, 4 ignored** |

**The mutation, re-run rather than taken from the report.**
Whole workspace under `memcap 8G cargo test --workspace --no-fail-fast`, `cp` backup, restore verified with `sha256sum -c`, `rexx-run` rebuilt afterwards.

| id | mutation | result |
|---|---|---|
| R3B | the fallback gains `Op::Condition` behind its `Op::EvalExpr` | exit 101, **two** red: `ir::golden_tests::a_condition_outside_the_native_set_stays_one_eval_expr` and `ir_dual::both_engines_agree_on_every_case_file`, the latter on the `if 1, 1` stanza with the ir arm emitting a fourth `>>>   "1"` the oracle does not |

The implementer's R3MXB reproduces exactly, including which stanza and which line.
After the restore and rebuild, both engines print five `>>>` on that program and their stderr is byte-identical, so the run above is not a stale binary.

**Oracle probes, taken independently from a fresh `mkdir`ed directory under the standard wrapper:**

* `if 0, 'x' then nop` / `say 'fell through'` -- rc 0, stdout `fell through`, stderr empty. The implementer's capture reproduces.
* the same under `trace r` -- rc 0, and the trace is `>>>   "0"` **twice** and no `>>>   "x"`, so `'x'` is genuinely never evaluated. (This is the probe the doc *should* have cited; see NN2.)
* `if 'x', 1 then nop` -- rc 222, 34.6, which is the measurement the N5 sentence rests on for the failing element that is not the first.

---

## The seven items

**N1 -- the falsified uniqueness claim in `ir/golden_tests.rs`. Addressed, by deletion, and the deletion is clean.**
The whole `**What this catches that the rest of the suite does not**` paragraph is gone.
What remains is a complete thought: what the test pins, and why the fallback is not the same op with a piece missing (an `EvalExpr` that evaluates *and* validates *and* traces, so a `Condition` behind it would do the last two twice).
Nothing false is left, no dangling reference to the deleted paragraph, and no copy of the uniqueness claim survives anywhere in `rust/crates/rexx-exec/src/ir/` or `tests/`.
The deletion-not-rewrite ground holds: the honest restatement is "these two files catch it", which is a cross-file aggregate in a comment, and `rust/CLAUDE.md` puts that in the delete-or-assert class.
What is lost is the record that this test discriminates *anything* -- a true, single-test claim. That record now lives only in the plan and the reports. Acceptable, and arguably what the rule asks for, but it is a real subtraction rather than a pure correction, so it is named here.

**N2 -- "all from inside `eval`". Addressed, and the attribution is now right.**
The stanza says a `>>>` per element it evaluates, from `eval_logical_list`, and one more for the list's own result, from the tail.
Both halves check out in the code: `eval_logical_list` calls `self.trace_result(indent, &text)` once per element inside the loop and breaks on the first that does not hold, and the list's own result line is `condition_value`'s `ConditionTrace::Result(indent) => self.trace_result(...)`.
That is exactly why R3B produces a fourth line rather than a third.
One overreach, minor -- see NN3.

**N3 -- "the whole line sequence comes from the one `Op::EvalExpr`". Addressed, and the replacement claims only what it can.**
"With no `Op::Condition` behind it, the one `Op::EvalExpr` still owes the `>>>` and the refusal, and both are here" attributes two things and no more; the second `*-*` and the two `Error` lines are left unattributed rather than misattributed.
The raise does originate inside that op's execution (`eval_chunk_expr`'s `If` arm -> `eval_if_condition` -> `eval_condition` -> `condition_value` -> `raised_if_not_logical`), so "still owes ... the refusal" is true, whatever prints the report.

**N4 -- the plan's Step 10 method claim. Addressed in substance; the last sentence of the new paragraph is not (NN1).**
The row accounting is now correct and independently checkable:

* Five stanzas went: c1, c2 (plain true, plain false), c3 (34.1), c5 (`trace r`), c7 (live indent). The plan says five.
* Two were measured out. c3's mutation is the review's MXC (`checked: true`, conditions file held out) which reddened `both_engines_agree_on_every_branch_shape`; c7's is MXD (static indent, held out) which reddened `both_engines_agree_on_every_case_file` on a `trace-settings` stanza. The plan names `BRANCH_CASES`' own `if 'x' then say 'y'` -- that program is real, `tests/ir_dual.rs:752`.
* Three were argued out by shape with no mutation of their own. That matches both reports.
* "which before it reddened one op-stream test and nothing that runs a program" is MXB/M2, and R3B is why it needed the "before".

**N5 -- the fourth copy of the 34.6 re-check mechanism in `run.rs`. Addressed.**
The clause is gone and the surviving sentence is true and oracle-backed in both directions: `if 'x', 1` measured 34.6 here, and `if 1, 'x'` is the case file's own row.
A **fifth** copy survives thirty lines away -- see NN4.

**N6 -- "validates every element". Addressed as to the mechanism; the citation attached to it is not (NN2).**
The new sentence is true, clause by clause, against `eval.rs`'s `eval_logical_list`: it checks each element it evaluates (`logical_value(&text).ok_or_else(...)?` inside the loop), raises 34.6 on the first that is not, and `break`s on the first that does not hold, and its `Ok` path is `self.text(if holds { b"1" } else { b"0" })` unconditionally.
It does not overreach in the other direction either: it does not claim the list stops checking, only that it stops.

**N7 -- the report's deletion list. Addressed.**
`task-1-report.md`'s fix round now reads "c1, c2, c3, c5 **and c7** deleted", with the parenthetical explaining that the ruling named four and c7 went with them.

---

## New or surviving statements this round should not ship, most severe first

### NN1 -- the plan quotes `0459167cc`'s commit message saying something it does not say

The plan's new closing sentence:
*"`0459167cc`'s commit message says all four rows "were measured out rather than argued out ... each held out while the mutation it was supposed to catch was re-run""*.

That commit message contains neither "measured out" nor "argued out". Its actual sentence is:
*"drops the plain, 34.1, `trace r` and called-label rows: each was held out of the directory while its mutation was re-run over the whole workspace, and each was still caught, by `BRANCH_CASES` or by a `trace-settings` stanza."*

"were measured out rather than argued out" is the **plan's own superseded paragraph**, not the commit message -- the round attributed its own retired wording to the artifact it says it cannot edit.
The second fragment is a paraphrase inside quotation marks, and the commit message never says "four" (it lists four kinds).

The substance is right: that message does claim the hold-out method for all four kinds, it is true of two, and five stanzas went. Only the attribution is wrong. Fix: quote the message's actual clause, or drop the quotation marks and describe it.
This matters more than a citation nit because the sentence's entire job is to tell a later reader which of two records to believe, and it misdescribes the one it is correcting.

### NN2 -- the probe the new N6 sentence cites cannot support the half of the claim it is attached to, and this tree already retired it for that reason

`eval_condition`'s rewritten doc: *"stops at the first that is `0` -- measured, `if 0, 'x' then nop` exits 0 on the oracle and never evaluates `'x'`"*.

`eval.rs`'s `eval_logical_list` doc, unedited, thirty lines above the function:

> The probe is `(1/0)` rather than the `'x'` an earlier version of this comment cited, because `'x'` cannot tell the two candidate rules apart: a literal evaluates harmlessly, so skipping only the *check* would produce the same clean run. `(1/0)` raises 42.3 the moment it is evaluated, and `if 1, (1/0)` does exactly that (rc 214), so what is skipped is the evaluation.

`rc 0` on `if 0, 'x'` establishes that `'x'` was never **checked**. It cannot establish that it was never **evaluated**, which is what the sentence claims -- and that is the exact discrimination the neighbouring doc records as already having been got wrong once and corrected.
The conclusion is nonetheless true: I measured `trace r` over the same program on the oracle and it prints `>>>   "0"` twice with no `>>>   "x"`, so the element is not evaluated. So this is a correct statement riding an insufficient measurement, which is the shape that passes review on the statement's merits.
Fix is one clause: cite `(1/0)` (the tree already has the number, rc 214), or cite the `trace r` capture, or drop "and never evaluates `'x'`" and keep "never raises on `'x'`", which is what rc 0 does prove.

Worth recording that the round arrived here by doing the right thing -- taking its own capture rather than trusting a number it was handed -- and taking the retired one.

### NN3 -- "the tail every condition enters" is falsified by the first stanza of its own file

The `if 1, 1` stanza attributes the list's result line to *"the tail every condition enters"*.
`condition_value` is the shared tail for `IF`, `WHEN`, `WHILE`/`UNTIL` and the compiled `Op::Condition`, so the sentence is right about what it is for. But a condition that raises during evaluation never reaches it -- and the file's own first stanza, `if 1, 'x'`, is exactly that: `eval_logical_list` raises 34.6 inside `eval` and the tail is never entered. A universal quantifier that the same file refutes four stanzas up.
"the tail a condition's value enters, whichever keyword built it" says the same load-bearing thing without the quantifier.

### NN4 -- a fifth copy of "validated every element" survives thirty lines below the sentence N6 corrected, and its cross-reference is backwards

`run.rs`, `condition_value`'s `if checked` arm:
*"// `eval_logical_list` already validated every element and answers exactly `b"0"`/`b"1"` (its own doc comment), so this is a plain readback rather than a second check."*

Same wording, same falsity: a list that short-circuits validated only up to the first `0`.
And the parenthetical cites `eval_logical_list`'s own doc as the authority, where that doc says the opposite in as many words ("Every element up to and including the first false one ... nothing after it is touched").
Pre-existing (`addf89b10`), named by neither review, in a file this commit edited and thirty lines from the fix. This is N5's shape repeating for N6: the round corrected the sentence it was handed rather than the claim.
The conclusion the comment draws -- readback rather than re-check -- is unaffected, as with N6.

### NN5 -- two smaller ones

* **The plan names a program the tree does not hold.** Step 10's bullet and the new paragraph both call the instrumented row `if 1, 1 then nop`; the stanza's program is `if 1, 1 then say 'both'`, and the `say` is not decoration -- it produces two of the transcript's lines. The bullet is inherited from `0459167cc`; the paragraph repeats it in a sentence written this round, so a reader regenerating the row from the plan gets a different capture.
* **The plan's Step 1 still carries the mechanism F2 declared false.** Line 82: *"`eval_logical_list` has already validated every element, so its result is read back rather than re-checked, and re-checking it would report 34.6 as 34.1."* Both halves are the two claims this task has now corrected in `run.rs` twice over, in the document this round edited. Not named by any review, and it is a plan rather than a comment -- but it is the text the next implementer reads.

---

## Verdict

**Another round, narrowly.** N1-N7 are all addressed and nothing in the shipped behaviour is wrong; the gates are green at `d07a8988e` (1468/0/4) and the one discrimination sentence the round rewrote reproduces its measurement exactly, two red tests, on the stanza it names.
But the round traded one unsupported claim for another twice: NN1 attributes a quotation to an artifact that does not contain it, and NN2 cites a probe this tree had already ruled unable to make the distinction the sentence draws from it.
Both are one-clause edits. NN3, NN4 and NN5 are cheap while the same two files are open, and NN4 is the second time this task has corrected one copy of a claim and left another in the same function.
