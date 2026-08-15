# Final whole-branch review: condition promotion, `02c6b9b67..a2952fd62`

Reviewed 2026-08-13, 18 commits, four tasks.
Scope is what a per-task review structurally cannot see: statements one task falsified in code another task owns, coherence of the four ops as one family, the plan document against the tree it produced, and the deferred-minor ledger.
Per-task verdicts are not re-litigated.

**Verdict: the code is sound and the branch is mergeable on behaviour. Nothing found changes what any program prints. But six prose defects of the declared cross-task class are live, one of them written past by the very commit that fixed its sibling forty lines away, and the plan's own closing measurement was never taken.**

Everything checked at `a2952fd62` in a detached worktree under its own `CARGO_TARGET_DIR`; no repository file was edited.

---

## 1. What was run, and what it said

### Gates

From `rust/`, in the worktree:

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0, no output |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `memcap 8G cargo test --workspace --no-fail-fast` | **1476 passed, 0 failed, 4 ignored**, exit 0 |

That is the stated baseline exactly.

**The naive worktree run shows 27 failures and every one of them is path resolution, not the diff.**
Twenty-six panic at `crates/rexx-extract/src/lib.rs:570`, reading `crates/rexx-exec/../../../ootest/ooRexx/base/bif`, which does not exist beside a worktree -- and two of those twenty-six are `ir_dual.rs`'s own `both_engines_agree_across_every_population` and `every_population_the_tree_calls_for_is_present_and_non_empty`, so **the dual-engine population sweep is among the tests a naive worktree run silently loses**.
The twenty-seventh is the already-recorded `every_blocked_axis_still_fails_on_this_crate`.
Symlinking `ootest` into the worktree root and `target/debug/rexx-run` under `rust/target/debug/` clears all 27.
Worth writing into the next plan's constraints: the recorded number (27) is right, and the recorded *cause* covers one of them.

### Commit `cfbbdedd7`'s measured claims, verified by running them

`cfbbdedd7` is Task 2's second correction round, and it wrote a new measured claim into `tests/ir_dual_cases/loop-header-values`'s header:

> **Every row here reddens `both_engines_agree_on_every_case_file` on its own** under two mutations of the header's compilation: the keyword echo emitted in front of its slot's ops, and every slot resolving to slot 0.

Both mutations were applied and both were run per row, each row extracted into a file of its own with the rest of the case directory held out.

| row | echo in front of the slot's ops | every slot resolving to slot 0 | unmutated control |
|---|---|---|---|
| `trace i` over `to`/`by`/`for` | FAILED | FAILED | ok |
| `trace r` over the same header | FAILED | FAILED | ok |
| `trace i`, bound is a call | FAILED | FAILED | ok |
| `trace i` over `DO OVER ... FOR` | FAILED | FAILED | ok |

The echo mutation needed `assert_keyword_echoes_precede_their_value` neutralised, as the assertion's own doc says. The failure in every case was the engine-vs-engine `assert_eq!` on stderr, not a panic from elsewhere.
Row 0's own narrower claim reproduced verbatim: under the echo mutation the ir arm printed `>K>   "TO" => "The NIL object"`, and `"BY"` and `"FOR"` likewise.

**The claim holds as written. No correction is owed to `cfbbdedd7`.**

One detail the header does not claim and a reader might over-read: the `DO OVER ... FOR` row does *not* print `The NIL object` under the echo mutation -- it prints the right value in the wrong place, because the `OVER` slot's register still holds the previous assignment's value. Row 0's comment is scoped to row 0 and is correct; nothing generalises it.

### Sixteen cross-task probes, three ways

Twelve programs written for this review plus four follow-ups, each run under the oracle (wrapped, from a fresh empty directory, absolute paths), then `REXX_ENGINE=tree-walker`, then `REXX_ENGINE=ir`, comparing stdout, stderr and exit status.
The shapes deliberately cross task boundaries, which is where no task's own case file reaches:

* a native `IF`/`ELSE` inside the body of a loop whose `to`, `by` and `for` all compiled natively and whose `to` is a call (`trace r`);
* a `SELECT` whose `WHEN` conditions hold calls, inside such a loop (`trace i`);
* `EXIT` with a value from inside a `WHEN` branch inside a loop; `EXIT` from inside an `IF` branch inside a loop;
* `RETURN` with a concatenation and a call, reached from inside a loop inside a routine, plus a bare `RETURN` in the same program read back through `RESULT`;
* a comma-list condition (`if zi = 1, zn = 2 then`) -- the declining shape -- inside a natively compiled loop;
* a declining `.nil` condition inside a natively compiled loop;
* `PUSH` and `QUEUE` in opposite arms of an `IF` inside a loop, read back by `PARSE PULL` in a second loop;
* `PUSH`/`QUEUE` chosen by a `SELECT` inside doubly nested loops whose inner header reads the outer control variable;
* nested loops where the inner header's bound is the outer control variable, with an `IF` in the inner body;
* `DO OVER` a string with a `RETURN` inside the body and a `RETURN` after the `END`;
* a nested `DO` block inside an `IF` branch containing a `PUSH` and a further `IF` with a `QUEUE`, with an `EXIT` in the `ELSE`.

**All sixteen agreed byte for byte on all three, on stdout, stderr and exit status, with two exceptions that are neither this plan's nor engine divergences** (tree-walker and ir identical in both):

* `DO OVER` a **stem** is a `Loud` gap in this crate -- rc 120 against the oracle's rc 0. Pre-existing implementation gap, unrelated to the promotion.
* **The clause echo of any clause after a completed loop sits two columns deeper than the oracle's.** Found first on a `RETURN` after a `DO OVER`'s `END`, which would have implicated Task 4; isolated by substituting a `say` for the `return` and a counted `DO` for the `DO OVER`, and it reproduces in every combination. So it is the loop-exit indent restoration, not `RETURN`, and it is the same family as the already-filed handler-indent divergences (`e379b68c0`, `9956d6abd`). Not new, not this plan's, and the two engines agree.

### A measurement trap this review walked into, recorded so the next one does not

After the mutation runs, `compile.rs` was restored from backup and verified with `sha256sum -c`, and the probes were then run against `target/debug/rexx-run` **without rebuilding**.
That produced four convincing, reproducible "engine divergences" -- every `>K>   "TO"` reading `"1"`, wrong `RESULT`s, a wrong exit status -- which were the last mutation (`slot -> 0`) still living in the binary.
A clean source tree says nothing about `target/`. Rebuild after reverting.

---

## 2. Findings, most severe first

### F1 -- a doc the fixing commit itself walked past, in a file it edited (must fix)

`rust/crates/rexx-exec/src/ir/drive/tests.rs`, above `the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk`:

> **The only observable that separates a promoted `IF` from an unpromoted one**, and that is why it is a count. Both engines print the same bytes for every program -- **the condition is evaluated by the same `eval_if_condition` either way** -- so what changes is whether the branch's clauses reach the compiled stream at all.

False since Task 1. On the compiled engine an `IF` condition `native_shape` accepts never enters `Interp::eval_if_condition`: it is the expression's own native ops plus `Op::Condition`, which enters `Interp::condition_value`.
The test's own two programs are `if 1 = 1 then ...` and `if 1 = 0 then ...` -- a comparison over two literals, which `native_shape` accepts -- so **the doc describes a route its own test provably no longer takes**.

What makes this the sharpest instance in the branch rather than one more stale sentence:

* `git show d3372e429 -- rust/crates/rexx-exec/src/ir/drive/tests.rs` shows Task 3 **rewriting the sibling doc forty lines below**, on the `SELECT` count test, from "resolved through the same `scan_when`, ..." to "a `WHEN`'s own condition reaches the same `Interp::condition_value` on either engine -- whether it gets there through `scan_when` or through the ops `crate::ir::Op::Condition` ends."
* So the commit that fixed the `WHEN` sentence was editing this exact file, was reasoning about exactly this property, and left the `IF` sentence -- which a *different* task had falsified -- untouched forty lines up.

This is the declared cross-task failure mode with a witness: no per-task diff review could see it, because the falsifying task (1) never opened the file and the task that did open it (3) was looking at its own construct.

The doc's *conclusion* -- that the count is the only observable separating a promoted `IF` from an unpromoted one -- survives, because both engines still print the same bytes; they now do it by sharing `condition_value` rather than by sharing `eval_if_condition`. Fix the reason, keep the conclusion, and say it the way the `SELECT` sibling already says it.

**Adjacent, and pre-existing rather than this plan's** (flagged so it is not re-found as new): the same file, twenty lines above the test, says "**At the top of the body, not inside an `IF`.** `If` steps its own branch through the tree-walker (that promotion is not this task's)" -- contradicted by the very next test in the file, which asserts that it does. An earlier phase falsified that one.

### F2 -- the plan's closing measurement was never taken (must do before merge)

`docs/superpowers/plans/2026-08-12-condition-promotion.md`, section "Measurement, after all four land":

> The accept rule (plan section 164) is unchanged and applies once, to the whole plan [...] Record the result in `docs/superpowers/plans/phase-4f-record.md` as the next entry, including the axes that did not move.

The newest entry in `phase-4f-record.md` is **Entry 26**, "what the intermediate echo ops cost when nothing is traced", written by `fdd992cdb` -- a side spike, not this plan's accept measurement.
No commit in `02c6b9b67..a2952fd62` adds an entry for the plan.

This matters more than an unticked box. The plan's entire ordering argument is a measured `rexxcps` clause profile, every task was chosen because it removes an `Op::EvalExpr` from a hot path, and **not one number says the removal bought anything**. The branch is currently four correct promotions of unknown value.

It is tracked -- `progress.md`'s last line reads `REMAINING: [...] then the closing measurement -- rexxcps via rexx-bench-suite, TWO do-nothing controls not one, because identical builds span 9.04% on arith` -- so this is confirmation rather than discovery. But it is the one section of the plan with no artifact, and Entry 26's own closing note is that these axes now need two do-nothing controls rather than one.

**Action:** take the measurement and write Entry 27, or record an explicit waiver of the plan's own accept rule. Do not merge on the assumption it was done.

### F3 -- `golden_tests.rs` states a premise this plan falsified, in a comment no task touched (must fix)

`rust/crates/rexx-exec/src/ir/golden_tests.rs`, in the doc above `the_value_shapes_outside_the_native_set_stay_general`:

> `ExprKind::Logical`, the comma list, is out of the native set too and is not a row here: it appears only in a condition, and **a condition is compiled to `Op::EvalExpr` whatever its shape**, so a row for it would pass under every implementation of this function's subject.

Written at `5a801e1e9`; `git log -S` over the range confirms **no commit in `02c6b9b67..a2952fd62` touched it**.

Two clauses are now false:

* an `IF`'s condition `native_shape` accepts compiles to the expression's own native ops plus `Op::Condition` (Task 1), not to `Op::EvalExpr`;
* a plain `WHEN`'s condition that *declines* compiles to `Op::WhenTest` (Task 3), which is not `Op::EvalExpr` either -- so there is no shape at all for which "a condition is compiled to `Op::EvalExpr`" is unconditionally true.

The conclusion survives, because `native_shape` has no `ExprKind::Logical` arm (verified, `compile.rs:1019-1042`), so a comma list still declines and a row for it still could not fail. That is what the sentence should say.

Same shape as F1: a still-correct conclusion resting on a reason that is gone. This plan has already corrected `Op::Arith`'s doc, `Op::Const`'s emission-site enumeration, `Loud::register_not_logical` and its own profile table for this reason, each found by a different reviewer looking at a different diff.

### F4 -- `queue.rs` locates the queue write where it no longer is, in three places (must fix)

`rust/crates/rexx-exec/src/queue.rs` is **untouched by the range**, and Task 4 moved the two writes out from under it.

Verified: `git show 0d058c086^:.../run.rs` has `self.queue.push(line)` / `self.queue.queue(line)` inside `step`'s `Push | Queue` arm at lines 2288/2290. In the tree today the only two interpreter-side write sites are `run.rs:2901-2902`, inside `Interp::queue_evaluated`, which `step`'s arm **and** `crate::ir::Op::Queue` both enter.

Three passages are now wrong about that:

* module doc, opening line: "The in-process external data queue (I15) that `PUSH` and `QUEUE` write to (**`run.rs`'s own arms for both**)." On the default engine -- `Engine::DEFAULT` is `Engine::Ir` -- a `PUSH`/`QUEUE` clause is `Op::Queue` inside an `Op::Clause` region and never reaches `step`'s arms at all.
* module doc: "the first two cannot see whether **`run.rs`'s `step` arms ever call `Queue::push`/`Queue::queue`** at all -- review round 1's I3 found that **deleting just those two call sites** (keeping the evaluation and the trace) left every other gate green".
* the same argument again above `push_and_queue_actually_write_into_the_running_interpreters_queue`.

The recorded measurement itself is correctly dated to review round 1 and is not the problem. What is stale is the mechanism: "those two call sites" are no longer `step`'s, and the named mutation would now silence the compiled engine too -- so the sentence understates the mutation's blast radius and mislocates the code.

One thing the doc still gets right and should keep: that unit test builds `Interp::new()` directly, and `Interp::new` sets `engine: Engine::TreeWalker` explicitly (`lib.rs`, with its own doc saying so), so the test really does exercise `step`'s arm. The test is fine; its explanation is not.

### F5 -- a raiser enumeration this plan wrote names two of four (must fix)

`rust/crates/rexx-exec/src/run.rs`, twice, introduced in-range by `7ea946434` and refined by `0459167cc`:

> `raise` is the keyword-specific raiser for that case (34.1 `IF`, 34.2 `WHEN`).

> `checked` is whether whatever produced `value` has already validated it [...] `raise` is the keyword-specific raiser for the unchecked case (34.1 `IF`, 34.2 `WHEN`).

`Interp::eval_condition` has five callers, and two of them are `WHILE` (`run.rs:6153`, `raised_while_not_logical`, 34.3) and `UNTIL` (`run.rs:6267`, `raised_until_not_logical`, 34.4). Both reach the same `raise` parameter of the same `condition_value`.
The second doc contradicts itself inside one comment: three paragraphs above it discusses "a per-pass frame for the two callers that are loop headers".

New text written by Task 1's split, not inherited rot -- and exactly the enumeration shape `Loud::expression`'s own doc argues against ("Neither message lists what *is* implemented, and both used to").
Say "the keyword-specific raiser for the unchecked case" and stop, or name all four.

### F6 -- the promotion survey is stale about this plan's own subject, and the ledger's description of it is half wrong (must fix the survey)

`docs/superpowers/plans/2026-08-12-remaining-promotion-survey.md` is untouched by the whole range.

* Its **Part 1, Task B: the value-returning instructions** proposes `RETURN`, `EXIT`, `PUSH` and `QUEUE` as the next increment, and names the shape: "Each becomes `<ops for the expression -> dst>` followed by `Op::Return { index, src: Option<u16> }` (and its **three siblings**)". Task 4 did the work and **explicitly rejected** four ops -- and rejected one tagged op too -- for two tagged ops. A reader arriving at the survey is told to do work that is done, in a shape the tree refused.
* Its **Part 2**, under "Promotable, and the sketch is the same each time", still lists all four in the "One expression then a `Flow`" row.
* Its **"Already promoted"** list is about *instructions* rather than conditions and is still accurate as written; it should gain the four rather than be rewritten.

**The ledger's other half of this item does not check out.** The deferred-minor entry says the survey "still describes the comma list as needing a jump inside a clause region, which a later commit withdrew". It does not. The survey's Task A sketch shows `Op::JumpUnless` per element with a forward `end:` target and says nothing about regions; the withdrawn blocker lived in this plan's own document and its commit message, and `cbdeaab71` touched only `2026-08-12-condition-promotion.md`. Correct the ledger entry rather than hunting for text that is not there.

### F7 -- `ir_dual.rs`'s inventory of its own blind spot is short by exactly the three functions this plan created (should fix)

`rust/crates/rexx-exec/tests/ir_dual.rs`, module doc, also untouched by the range:

> **The one thing it cannot see is work the two engines share.** [...] a construct resolved by one function entered from both arms -- `Interp::run_loop`, `Interp::assign_evaluated`, `Interp::say_evaluated`, `Interp::invoke_call` -- answers identically on both by construction.

This plan created three more functions of precisely that shape, each documented in `run.rs` as "The one implementation both engines enter": `Interp::condition_value`, `Interp::returned_value`, `Interp::queue_evaluated`.

The list is not marked exhaustive, so this is an incomplete enumeration rather than a false proposition -- but it is *the* doc that tells a reader what the dual sweep structurally cannot police, and this plan's own Task 4 leans on that property in writing: `tests/ir_dual_cases/return-and-queue`'s header says "Both engines take that mutation, so the engine comparison cannot see it." The inventory that explains why should name the function.

### F8 -- the family is coherent, with one asymmetry worth closing cheaply (should fix)

Asked directly: do `Op::Condition`, `Op::Return` and `Op::Queue` read as one family?

**The tags mean the same kind of thing.** All three name the source keyword that introduced the clause, and each selects exactly one downstream difference: `ConditionKeyword` the raiser (34.1 / 34.2), `ReturnKeyword` the `Flow` constructor, `QueueKeyword` the end of the queue. None carries semantics beyond that one choice. One concept, three instances.

**The register discipline is the same, and the one difference is forced.** `Op::Return` and `Op::Queue` carry `src: Option<u16>`, read-only, allocated only when the instruction has an expression -- `Op::Say`'s shape exactly, and the `Option` is load-bearing for `RETURN` (a bare `RETURN` leaves `RESULT` unset) and not for `PUSH`/`QUEUE`, which the case file says in its own header. `Op::Condition` carries one unconditional `reg`, read *and written in place*, because a condition always has an expression and `Op::JumpUnless` must read the validated answer back out of the same place. All three take `registers.mark()`, allocate inside it, and release to it after `close_region`. The only cosmetic drift is that the `IF`/`WHEN` arms allocate `dst` before pushing `Op::Clause` where `Say`/`Return`/`Queue` allocate after; the mark is taken first in both, so nothing depends on it.

**The shared halves follow two conventions, not one.** `Op::Return` and `Op::Queue` pass the *tag itself* into the shared half (`self.returned_value(value, *keyword)`, `self.queue_evaluated(value, *keyword)`), which matches on it internally. `Op::Condition` resolves the tag at the driver (`keyword.raiser()`) and hands `condition_value` a function pointer, so the shared half never sees the keyword. That is defensible -- `condition_value` was split out of a pre-existing `eval_condition` that already took `raise: fn(&[u8]) -> Raised`, and `WHILE`/`UNTIL` need raisers `ConditionKeyword` cannot name -- but it deserves a sentence in `ConditionKeyword`'s doc, because the asymmetry currently reads as an oversight and is not one. Note also that `ConditionKeyword` lives in `ir/mod.rs` while `ReturnKeyword` and `QueueKeyword` live in `run.rs` beside their shared halves.

**The one real asymmetry is the debug assertions, and it runs the wrong way.** In `ir/drive.rs`:

* `Op::Say` asserts the clause is a `Say` **and** that its arity matches `src`;
* `Op::Return` asserts `Return | Exit` **and** arity -- but never that `keyword` names *which* of the two;
* `Op::Queue` asserts `Push | Queue` and arity, likewise never the keyword;
* `Op::Condition` asserts **neither the clause kind nor the keyword** -- only `chunk.holds_register` and `debug_assert_names_the_clause`.

So the plan's three ops check three different amounts, and the one thing all three could check -- that the tag agrees with the clause -- is checked by none. The ledger raised this for `Op::Return`; it is not `Op::Return`'s alone.

**Severity is low, because the behaviour is covered.** A mis-tagged `EXIT` reddens `tests/ir_dual_cases/return-and-queue` on stdout between the engines. A mis-tagged `WHEN` reddens `tests/ir_dual_cases/when-conditions` on 34.2 against 34.1. A mis-tagged `IF` reddens `ir_dual.rs`'s inline case "if condition is not a logical value" (`if 'x' then say 'y'` -- a literal, so the condition compiles and the op carries `keyword=IF`) -- worth recording, because the `conditions` case file's own 34.1 row uses `if .nil then nop`, a *declining* condition whose 34.1 comes from `eval_if_condition` and not from the tag. Both halves of `ConditionKeyword` have a behavioural witness; they are in two different files.

One `matches!` per arm is three lines and closes the class.

### F9 -- `Seen::native_conditions`' doc counts something the code does not (leave or one-word fix)

`rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs`:

> How many promoted clauses carry an `Op::Condition`, keyed by the keyword the op is tagged with.

The loop below increments once per `Op::Condition` **op** in the region, not once per region that has one. Equivalent today, since `compile` emits at most one per region; false as written, and the field's whole purpose is anti-vacuity, which is a job for a count that means what it says.

### F10 -- `push_queue.rex`'s header enumerates two read-back places where there are three (leave)

`rust/corpus/lang/push_queue.rex` names `lang/pull_queue.rex` and `input_oracle.rs`'s `queue-round-trip` row. `tests/ir_dual_cases/return-and-queue`'s `PARSE PULL` row is a third. Harmless: the header's actual subject -- what *this program* can and cannot pin -- is unchanged, and the list is an aside.

### F11 -- `render_condition_keyword`'s mapping is written twice (leave)

`ir/golden.rs`'s `render_condition_keyword` and `ir/corpus_shape_tests.rs`'s inline `match keyword { ConditionKeyword::If => "IF", ConditionKeyword::When => "WHEN" }`. Both `#[cfg(test)]`, both two arms over a two-variant enum, both a compile error if a variant is added. The duplication cannot rot silently.

### F12 -- `root_exit_value`'s doc is `EXIT`-phrased on a path `RETURN` shares (leave, or fix while nearby)

`rust/crates/rexx-exec/src/lib.rs`: "**The one value in this crate whose lifetime the temps stack cannot express.** `EXIT`'s result is pushed as an ordinary one-clause temp [...]".

`apply_flow`'s `Flow::Return` arm has called `root_exit_value` for every returned value since `967adba3e`, which predates this plan -- inherited rot, not Task 4's. What earns it a line is that the range's **last commit**, `a2952fd62` "Say the rooting window in terms both keywords satisfy", fixed the *sibling* statement in `returned_value`'s doc and walked past this one. The correction was aimed at the right class and stopped one function short.

### F13 -- the triplicated arms (leave)

`ir/compile.rs`'s `Say`, `Return` and `Queue` arms are three copies of mark / `Op::Clause` / `push_echo` / optional `push_value` / the op / `close_region` / `release`, and `ir/drive.rs`'s three arms repeat the same `debug_assert_names_the_clause` + arity assert + `src.map(|register| { holds_register; temp_at })` preamble.

**Leave them.** Factoring the compile side means threading `ops`, `consts`, `registers`, `hints`, `calls` and `plan` through a helper that then takes a closure or an enum for the tail, which is more machinery than the copies cost; the three tails genuinely differ (a print; a `Flow` that ends the region; a queue write that does not). The driver side is the same story with a borrow problem on top, and the arity asserts differ per arm anyway.

### F14 -- `compile.rs`'s `_ =>` in the `When` match (leave, and correct the ledger)

The ledger calls it "against the file's stated exhaustiveness convention". `compile.rs` states no such convention; the rule is stated in `lib.rs`'s `Loud::instruction` and `corpus_shape_tests.rs`'s `Root::of`, both about matching an enum where a new variant must be a compile error. The arm in question is a two-way choice inside an already-guarded `When | WhenCase` arm whose fallback is a real answer (`Op::WhenTest` does the whole job), not a silent hole.

### F15 -- `Op::EvalExpr`'s slot examples, and the plan checkboxes (leave)

`ir/mod.rs`'s `Op::EvalExpr` doc illustrates `slot` with `If` and `SELECT CASE` and now also omits `RETURN`/`EXIT`/`PUSH`/`QUEUE` -- but it already omitted `Assignment`, `SAY` and the `DO`/`LOOP` header slots before this plan, so its character has not changed and `eval_chunk_expr`'s own doc carries the full list.
Separately, every `- [ ]` in the plan is still unticked although all four tasks landed; the plan now reads as a record, its Task sections carrying bolded "what actually happened" paragraphs.

---

## 3. The plan document against the tree

Checked, and holding:

* **"Deliberately out of scope"** is honoured. The comma list still declines (`native_shape` has no `ExprKind::Logical` arm); `WHILE`/`UNTIL` still reach `eval_condition` with `ConditionTrace::Keyword` and their own 34.3/34.4 raisers; `PARSE` and `THEN` are untouched; a call's arguments still go back through `eval.rs`.
* **"`Op::Jump`'s in-region arm is unreachable today, because `compile` emits `Op::Jump` only outside a region"** holds. The two emission sites are `compile.rs:615` (the `SELECT` arm, after `close_region`) and `compile.rs:1364` (`emit_before`, which runs before the instruction's own region opens). The in-region arm at `drive.rs:1315` is dead as claimed.
* **The profile table's dating** (`54afba8a1`) is the right fix for the rot class: the table is explicitly a measurement at `0459167cc` that the tasks falsify on purpose, rather than a description maintained per task.
* **Task 3's `chunk_node_at` doc resolution** holds in the tree: containment replaced by "a slot is in both exactly when it is offered to `push_native` *and* its declining fallback is `Op::EvalExpr`", with both directions of the counterexample real.
* **Task 4's shape argument** holds leg by leg in the code: `returned_value` differs from `queue_evaluated` in the trace rule (`result_text` only when there is a value, against `to_text` always), in the side effect, and in the region end.

Not checked, and flagged rather than asserted: the plan's "a depth-tracking scan of the corpus found [the comma list] in one program". Reproducing that needs the same scan, not a grep, and nothing here depends on the number.

---

## 4. Triage of the deferred minors

**Must fix before merge**

1. F2 -- take the plan's closing measurement and write `phase-4f-record.md` Entry 27, or record an explicit waiver. (Not a "minor"; listed here because it is the one outstanding plan step.)
2. F1 -- `ir/drive/tests.rs`'s "the condition is evaluated by the same `eval_if_condition` either way". Say it the way the `SELECT` sibling forty lines below already says it.
3. F3 -- `golden_tests.rs`'s "a condition is compiled to `Op::EvalExpr` whatever its shape". Replace the reason with `native_shape`'s missing `ExprKind::Logical` arm; keep the conclusion.
4. F4 -- `queue.rs`'s three passages locating the queue write in `step`'s arms.
5. F5 -- the "(34.1 `IF`, 34.2 `WHEN`)" gloss on `condition_value`'s and `eval_condition`'s `raise` parameter, twice in `run.rs`. This plan wrote it.
6. F6 -- `2026-08-12-remaining-promotion-survey.md`'s Part 1 Task B and its Part 2 "One expression then a `Flow`" row. Also correct the ledger entry, which credits the survey with a comma-list claim it does not make.

**Should fix, cheap, not blocking**

7. F7 -- `ir_dual.rs`'s module-doc list of functions both engines share, plus the three this plan added.
8. F8 -- one `matches!` per arm in `drive.rs` pinning each tag against its clause kind, for all three ops rather than `Op::Return` alone; and one sentence in `ConditionKeyword`'s doc saying why its half resolves the tag at the driver where the other two pass it through.
9. F9 -- `Seen::native_conditions`' doc: "how many `Op::Condition` ops", not "how many promoted clauses".

**Leave**

10. F10 -- `push_queue.rex`'s two-of-three read-back list. The header's subject is unaffected.
11. F11 -- the twice-written `If`/`When` string mapping. Both copies are test-only and neither can rot silently.
12. F12 -- `root_exit_value`'s `EXIT`-phrased doc. Pre-existing; fix only if someone is already in `lib.rs`.
13. F13 -- the triplicated `compile.rs` and `drive.rs` arms. The abstraction costs more than the copies.
14. F14 -- `compile.rs`'s `_ =>` in the `When` match. No convention is violated.
15. F15 -- `Op::EvalExpr`'s slot examples and the unticked plan checkboxes.

**Note on the pattern.** Five of the six must-fixes are prose, and four of those five sit in files the tasks did not touch or touched somewhere else. That is the same distribution the per-task rounds produced -- behaviour right on the first pass, prose wrong until someone reads the neighbourhood rather than the diff. The cheapest structural defence available here is not another review round: it is that `condition_value`, `returned_value` and `queue_evaluated` are now the named entry points, so a future grep for `eval_if_condition` or `self.queue.push` finds every doc that still routes around them.

---

## 5. Checked and clean -- do not re-check these

* Gates at `a2952fd62`: fmt exit 0, clippy exit 0, tests **1476 / 0 / 4**, exit 0, in a detached worktree with its own `CARGO_TARGET_DIR`.
* The 27 spurious worktree failures: 26 are `rexx-extract` resolving `../../../ootest`, one is `every_blocked_axis_still_fails_on_this_crate`. Two of the 26 are `ir_dual.rs`'s population sweep, so a naive run loses them silently. Symlinks fix all 27.
* `cfbbdedd7`'s headline measured claim, **run**: all four `loop-header-values` rows redden `both_engines_agree_on_every_case_file` on their own under both stated mutations, with a green unmutated control per row. Row 0's `The NIL object` claim reproduced verbatim.
* Sixteen cross-task probes agree three ways (oracle / tree-walker / ir) on stdout, stderr and exit status.
* The two divergences those probes found are pre-existing and engine-agnostic: `DO OVER` a stem is a `Loud` gap; the clause echo after a completed loop is two columns deep against the oracle, reproduced with `say` and with a counted `DO`, so it is not `RETURN` and not Task 4.
* `assert_region_ops_name_their_clause`'s exhaustive `Op` match classifies all three new ops.
* `RegionEnd::Flowed`'s rooting chain forwards to `ClauseValue for Flow`, the same rule the tree-walker's `in_clause` reaches -- so a `RETURN`/`EXIT` value crossing a promoted clause boundary is rooted by one implementation, not two.
* Both halves of `ConditionKeyword` have a behavioural witness, not only an op-stream pin: 34.2 in `when-conditions`, 34.1 in `ir_dual.rs`'s inline "if condition is not a logical value".
* `Op::WhenTest`, `Op::EvalExpr`, `Op::Say`, `Op::Store`, `Op::Call`, `Loud::register_not_logical`, `Loud::instruction`, `Loud::call_op_off_its_node`, `scan_when`, `eval_if_condition`, `eval_chunk_expr`, `chunk_node_at`, `corpus_shape_tests`'s module doc and its `promoted_as`/`root_of` expectations, and `eval.rs`'s module doc were all read against the tree as it now stands and are correct.
* Swept and clean: `src/trace.rs`, `src/clause.rs`, `src/input.rs`, `src/error.rs`, `src/plan.rs`, `src/invocation.rs`, all of `rust/corpus/` except `push_queue.rex`, all of `rust/crates/rexx-bench/`, and the untouched case files `trace-settings`, `calls`, `assignment-and-say`, `operators`, `arithmetic`, `variable-reads`, `loop-refusals`, `loop-header-boundaries`. One pre-existing stale line noted in passing and **not** this plan's: `tests/ir_dual_cases/trace-settings` says "an assignment is Generic today", contradicted by `Op::Store` since an earlier phase.
