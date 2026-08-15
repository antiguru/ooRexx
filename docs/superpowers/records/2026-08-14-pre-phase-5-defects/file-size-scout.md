# `run.rs` file-size scout

Read-only. Every number below is a **measurement of the working tree at the moment I read
it** (`run.rs` = 17,381 lines), not a durable fact: HEAD `8d0acced1` carries 17,343 and
another session is editing the file live. Line ranges are therefore structural, and will
have drifted by a few dozen lines by the time anyone acts on them.

---

## 1. The note exists. There are two, and a triage entry.

### The decision, in full

`.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/progress.md:1158-1188`

> ## Deferred decision: splitting `run.rs`
>
> Raised 2026-08-07, **deferred by the user until Task 13 lands.** Do not open it before then.
>
> Measured at `bf272e63`: `run.rs` is 13,466 lines -- 6,395 of them the test module after
> `#[cfg(test)]` at `:7071`, and 4,436 of the remainder comment lines, leaving roughly
> **2,600 lines of production code** in a single `impl Interp` block (`:565` onward, 85
> methods). The next largest file is `lib.rs` at 2,592. So the headline number overstates
> it: the density is this project's evidence-comment style, not sprawl.
>
> If it goes ahead it belongs in a **post-4c consolidation plan, not a Task 16**: [...]
>
> * A pure-move refactor has **no differential signal** -- output is unchanged, so a green
>   corpus proves almost nothing. What can actually break is dropped comments (against the
>   standing rule) and intra-doc links. Those need a mechanical check of their own: comment
>   line count before against after, and `cargo test`'s doc-test pass for rustdoc links.
>
> Mechanically cheap when the time comes: inherent impls may span modules in one crate, so
> `run/instruction.rs`, `run/call.rs`, `run/condition.rs` can each carry their own
> `impl Interp`.

**Status of that deferral: unblocked and never picked up.** Task 13 landed. No "post-4c
consolidation plan" exists — the plans directory jumps from `2026-08-04-pre-4c-consolidation.md`
(which predates the deferral, 2026-08-04 vs raised 2026-08-07) to `2026-08-08-phase-4d-1-...`.
Nothing in `2026-08-13-phase-4g-structural.md` or `2026-08-14-pre-phase-5-defects.md` mentions it.

### The design's own anticipation

`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md:342`

> One file per concept, as elsewhere in the workspace, each readable in one sitting; `run.rs`
> is the one at risk, and the split when it comes is the loop from the per-instruction handlers.

### The triage that flagged it as at risk of being lost

`.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/deferred-minors-triage.md:66`

> Task 13 has since landed (the ledger records it complete), so this decision is technically
> unblocked, but it is an architectural scope decision reserved for the user, not a minor
> finding this triage should rule on. Flagged here only so it is not silently lost between
> "Task 13 lands" and whoever next opens the ledger.

### One correction to the deferral's own prescribed check

The note says the mechanical check is "comment line count before against after, and
`cargo test`'s doc-test pass for rustdoc links". **The second half does not do what it says.**
`cargo test --doc` compiles and runs doc *code blocks*; it does not resolve intra-doc links.
Broken `[`Interp::foo`]` links are `rustdoc::broken_intra_doc_links`, which is warn-by-default,
and nothing in this workspace denies it (no `deny(rustdoc` anywhere outside `target/`). The
check that works is `cargo doc --no-deps` with its warnings read, or adding
`#![deny(rustdoc::broken_intra_doc_links)]` to the crate root first so the split cannot land
a silent breakage.

### What I searched

`docs/superpowers/plans/*.md`, `docs/superpowers/specs/*.md`, all of `.superpowers/sdd/**`
(including the seventeen dated subdirectories and their `progress.md`, `*-report.md`,
`*review*.md`), and every `*.md` in the repo, with `/bin/grep -a` for: `run.rs` co-occurring
with split/large/carve/monolith/shrink; `splitting \`run.rs\``; `run/instruction.rs`;
`run/call.rs`; `run/condition.rs`; `run.rs is N lines`. The three citations above are the
complete set of hits.

---

## 2. What is actually in `run.rs`

### The gross shape

| region | lines | code | comment |
|---|---|---|---|
| header + `use` | 1–96 | ~37 | 55 |
| free types and loop-header helpers | 97–956 | 236 | 592 |
| **`impl Interp`** (one block, 95 methods) | 957–8664 | 2,765 | 4,807 |
| free functions and plain data | 8665–9667 | 436 | 521 |
| **`#[cfg(test)] mod tests`** (249 `#[test]`, 10 helpers) | 9668–17381 | 5,125 | 2,177 |
| **total** | **17,381** | **8,600** | **8,127** |

Two facts dominate everything below:

* **44.4% of the file is the test module** (7,714 lines).
* **The production half is 63% comment** — 3,474 code lines against 5,950 comment lines in
  1–9,667. The 2026-08-07 note's "the headline number overstates it" is still true: there
  are about **3,500 lines of production code** here, which is a normal-sized file wearing a
  17,000-line coat. Any split proposal that quotes 17,381 as the problem is misleading.

Of the 95 methods on `impl Interp`, **41 are `pub(crate)`** and **54 are private**. The
`pub(crate)` surface is not decoration: 32 of those methods are called from other modules,
23 of them by `ir/drive.rs` alone (the compiled engine drives the same clause and condition
machinery the tree-walker does). That existing cross-module surface is the reason a split
is mechanically cheaper here than the method count suggests.

### The groups, with entry points and what they reach

| # | group | lines | code | entry points | reaches |
|---|---|---|---|---|---|
| A | types & loop-header plan | 97–956 | 236 | `Flow`, `Ended`, `Echo`, `SteppedClause`, `Resolved`, `LeaveOrigin`, `HeaderRole`/`HeaderPlan`, `loop_header_plan`, `loop_header_slot`, `ReturnKeyword`, `QueueKeyword` | `rexx_parse` AST only; no `self` |
| B | the driver | 957–1457 | 98 | `run_activation` (pub), `grant_procedure_permission` (pub), `apply_flow` (pub) | C, I |
| C | **the `step` dispatcher** | 1458–2398 | 300 | `step` (private) | 29 `InstructionKind` arms; calls into D,F,G,J,L,M,N,O,P,Q,R |
| D | PROCEDURE / EXPOSE / USE | 2399–2887 | 216 | `exec_procedure`, `exec_use`, `exec_use_arg`, `bind_use_target`, `expose_names` | E, O |
| E | assignment, SAY, RETURN, QUEUE, SIGL | 2888–3300 | 134 | `say_evaluated`, `returned_value`, `queue_evaluated`, `assign_evaluated`, `assign_expr_target` (all pub), `assign_by_name`, `set_sigl` | `stem.rs`, `trace.rs` |
| F | conditions, traps, RAISE, SIGNAL | 3301–4170 | 316 | `exec_condition_trap`, `trap_for` (pub), `novalue_check` (pub), `offer_to_trap` (pub), `deliver_pending_traps` (pub), `exec_raise`, `resolve_signal_target` | E (`set_sigl`), U (`raised_*`) |
| G | calls and activations | 4171–4935 | 261 | `resolve_call` (pub), `invoke_call` (pub), `resolve_and_run_call` (pub), `exec_call`, `invoke_named_call` (pub) | E, P, Q, `clause.rs` save/restore |
| H | **the stepped-clause unit** | 4936–5308 | 124 | `step_in_temps_frame`, `in_stepped_clause`, `in_stepped_clause_with`, `enter_stepped_clause`, `leave_stepped_clause`, `echo_stepped_clause`, `echo_compiled_clause` (all pub) | `clause.rs`'s `ClauseEntry`/`ClauseOutcome` — see §3's flag |
| I | failure sites | 5309–5486 | 47 | `record_failure_site` (pub), `record_failure_at`, `leave_origin`, `record_leave_failure` | `error.rs` |
| J | SELECT / WHEN / OTHERWISE | 5487–5815 | 128 | `select_case`, `open_select_case`, `scan_when`, `leave_otherwise`, `leave_select` (all pub), `run_otherwise`, `pop_search_frame` | L, M, T |
| K | `printed_indent` | 5816–5884 | 7 | `printed_indent` (pub) | S |
| L | **loops** | 5885–7275 | 631 | `run_bounded`, `run_loop`, `run_loop_with_header` (pub), `run_repeating`, `loop_advance`, `bind_control`, `do_body_outcome`, `eval_loop_header`, `accept_header_value` (pub) | A, I, M, T, U |
| M | condition/expression glue | 7276–7744 | 170 | `chunk_node_at` (pub), `eval_chunk_expr` (pub), `condition_value` (pub), `eval_condition`, `condition_holds`, `test_case_when`, `eval_if_condition` | `eval.rs` |
| N | INTERPRET | 7745–7878 | 47 | `run_fragment` | B, L, O, P |
| O | `seal_site_level`, DROP | 7879–8026 | 49 | `seal_site_level`, `drop_variable`, `drop_by_name` | T |
| P | TRACE and invocation tracing | 8027–8251 | 76 | `exec_trace`, `trace_invocation_entry`/`_exit`, `enter_fragment`, `leave_fragment` | `trace.rs` |
| Q | ADDRESS and NUMERIC | 8252–8443 | 108 | `exec_address`, `exec_numeric`, `numeric_operand` | U |
| R | clause site / line | 8444–8664 | 53 | `clause_site`, `clause_line`, `clause_line_at` (all pub) | `clause.rs`, `plan.rs` |
| S | static indent tables | 8665–8999 | 202 | `all_indents` (pub), `static_indent` (pub), `fill_indents`, `indent_in_range` | AST only; **no `self`** |
| T | name shapes, indirect words | 9000–9142 | 40 | `shape_of` (pub), `NameShape`, `control_slot`, `split_indirect_words`, `validate_indirect_word` | **no `self`** |
| U | control-flow target arithmetic | 9143–9385 | 103 | `absorb`, `if_targets`, `when_targets`, `select_parts`, `select_escape`, `skip_else`, and the `*Targets`/`SelectResume`/`SelectEscape` types | **no `self`** |
| V | `Raised` constructors + num helpers | 9386–9667 | 91 | 16 `raised_*` fns, `raise_syntax_condition`, `condition_name`, `numeric_less`, `round_via_unary_plus` | `error.rs`, `rexx_num`; **no `self`** |
| W | **tests** | 9668–17381 | 5,125 | `mod tests` — 249 `#[test]`, 10 helpers | see §3 rank 1 |

---

## 3. Split proposal, ranked by value / risk

The workspace already uses the target layout in two places, in this crate and the sibling
parser: `ir/drive.rs:1908` is `#[cfg(test)] mod tests;` pointing at `ir/drive/tests.rs`
(top-level `#[test]` fns, no nested `mod`), and `rexx-parse/src/instruction.rs:2715` /
`directive.rs` do the same. So `run.rs` + `run/` needs no new convention and no change to
`lib.rs:104`'s `mod run;`.

### Rank 1 — move the test module to `run/tests.rs`. Do this one.

* **Moves:** 9,668–17,381 (7,714 lines, 5,125 code). `run.rs` drops to ~9,670.
* **Visibility widened: none.** Measured — the test module calls **zero** private
  `impl Interp` methods. Its only direct `impl Interp` call is `interp.run_activation`,
  already `pub(crate)`; everything else goes through ten module-local helpers and reads
  `interp.out` / `interp.trace` / `interp.failure_site`. Its imports are
  `use super::*; use crate::Activation; use crate::plan::{BodyKey, ProgramId};
  use rexx_parse::{Program, parse_program};` — all four resolve identically from a child
  file, and `super` is still `crate::run`.
* **Why this boundary:** it is the only boundary in the file that is already a boundary.
  Nothing production reaches across it, and it removes 44% of the file for one line of code.
* **Not byte-identical, but mechanically checkable:** the body must be dedented by four
  spaces. Verify with
  `diff <(git show HEAD:…/run.rs | sed -n '9670,17380p' | sed 's/^    //') run/tests.rs-body`
  plus `cargo fmt --check`.
* **Mutation-script cost: zero.** No mutation pattern in `scripts/mutate-4{a,b,c}.sh`
  resolves into the test region (measured against every pattern in all three scripts).

### Rank 2 — move the four `self`-free tails to `run/` siblings.

`run/indent.rs` (S, 335 lines), `run/name.rs` (T, 143), `run/targets.rs` (U, 243),
`run/raised.rs` (V, 282). Together 1,003 lines, 436 code.

* **Why this boundary:** these take no `self`, so the whole `impl Interp` visibility question
  never arises. They are the four coherent, testable, self-contained tables in the file —
  indent arithmetic, symbol-shape classification, branch-target arithmetic, error
  constructors — and each is exactly the kind of thing this project already puts in its own
  file elsewhere.
* **Visibility widened: 18 private free items would need `pub(super)`** (not `pub(crate)` —
  `pub(super)` from `crate::run::raised` is visible to `crate::run` and every sibling under
  it, which is strictly narrower than today's de-facto whole-`crate::run` reach). They are:
  `control_slot`, `split_indirect_words`, `validate_indirect_word`, `raise_syntax_condition`,
  `condition_name`, `raised_from_settings`, `numeric_less`, `round_via_unary_plus`, and ten
  `raised_*` constructors. `indent_in_range` is **not** among them — its only mention above
  8664 is in a doc comment.
* **Existing paths survive unchanged:** the 21 already-`pub(crate)` items here are named from
  outside as `crate::run::static_indent`, `crate::run::shape_of`, `crate::run::NameShape`,
  `crate::run::SelectEscape` and `crate::run::all_indents`
  (part of 16 `crate::run::` references crate-wide, including intra-doc links). A `pub(crate) use` re-export in
  `run.rs` keeps every one of them working with **no edit to any other file**, which is the
  single biggest risk reducer available and should be a condition of the change.
* **Mutation-script cost: zero** — no live pattern resolves above 8664.

### Rank 3 — carve `impl Interp` by responsibility. My recommendation is **don't**, or not yet.

This is what the 2026-08-07 note sketched (`run/instruction.rs`, `run/call.rs`,
`run/condition.rs`) and what the design anticipated ("the loop from the per-instruction
handlers"). The measured objection:

* **29 of the 54 private methods are called across the group boundaries** in §2's table, so
  more than half the type's private surface becomes `pub(super)`. `step` alone reaches 14 of
  them; `record_leave_failure` is reached from four different groups; `run_bounded` from
  three; `eval_condition` and `test_case_when` from two each. The call graph is not a tree —
  loops ↔ select ↔ condition-glue ↔ step form a cycle — so there is no boundary that cuts
  few edges.
* **The value is small.** After ranks 1 and 2 the file is ~8,660 lines of which **2,765 are
  code**. Carving that into three files yields ~900 code lines each, dressed in ~1,600 lines
  of comment each. The reading problem this is meant to solve — "where does `DO OVER` decide
  its iteration order" — is a navigation problem that `step`'s 29-arm match already solves
  better than a file boundary would.
* If it is done anyway, the least-bad cut is **C alone → `run/step.rs`** (the dispatcher, 941
  lines) leaving everything it calls behind. That widens `step`'s callees, which is the
  expensive direction; the cheap direction is the reverse — move the dispatcher's *callees*
  out and leave `step` where the driver can see it privately. `run/call.rs` (G, 765 lines) is
  the most nearly self-contained handler group and would be the honest first probe.
* **Mutation-script cost is real here:** of the 17 rows in `scripts/mutate-4{a,b,c}.sh` that
  name `${RUN_RS}`, **11 still apply** to the current tree and would each need their file
  argument repointed — L1744 and L1675 → `step`, L6137 and L6730 → `loops`, L2483 and L2673
  → procedure/use, L4466 and L4867 → calls, L3468 → conditions, L7978 → drop, L839 →
  preamble. The edit is one token per row, but it is an edit to a *gate* instrument.
  (Separately worth knowing: the other **6 rows are already stale** — 5 match zero times and
  1 matches twice — so `mutate-4a.sh` row 2, `mutate-4b.sh` rows 7/9/11 and `mutate-4c.sh`
  row 7 would abort their script with `UNAPPLIED PATTERN` today, split or no split.)

### The visibility flag you asked for

I read `clause.rs`'s module doc (`clause.rs:11-120`) in full. **The good news is that a
`run.rs` → `run/*.rs` split does not touch its guarantee at all.** The property is that
`ClauseState`'s line field is private to `crate::clause`, and `crate::run` is not an ancestor
of `crate::clause`. `crate::run::step` is not an ancestor either. Nothing in ranks 1–3 changes
what any module can reach.

**The move that would be a regression is the opposite one, and it is the tidy-looking one:**
relocating group H (4,936–5,308: `enter_stepped_clause`, `leave_stepped_clause`,
`in_stepped_clause`, `echo_stepped_clause`) *into* `clause.rs` on the grounds that clause
things belong with clause things. That would put `run.rs`'s clause-boundary callers inside
the module that owns the line field, and the whole four-round argument at `clause.rs:11-40`
— "that difference is what turns 'remember to do both things' into 'you cannot do one of
them'" — evaporates silently, with every test still green. Do not do it, and do not let a
reviewer suggest it as a tidy-up.

One second-order cost to state honestly: `clause.rs:105-120` already names the hazard its
types *cannot* close — `run.rs` restoring a stale `SavedClauseState` at a moment other than
the one it was taken from, "measured: builds, passes clippy, and passes all 296 lib tests,
undetected". Today every `save_clause_state`/`restore_clause_state` call site is in one file
a reviewer can read end to end. Spreading them across `run/call.rs` and `run/interpret.rs`
makes that audit a multi-file one. That is an argument against rank 3 specifically; ranks 1
and 2 move no such call site.

---

## 4. The other files over ~1,500 lines

Production/comment counts are for the region before the file's own `#[cfg(test)]`.

| file | lines | verdict |
|---|---|---|
| `rexx-exec/src/lib.rs` | 3,446 | **Not a problem.** 380 code / 945 comment before `mod tests` at 1,379; the remaining 2,067 lines are tests. The crate root is small. If anything itches, `mod tests;` here too. |
| `rexx-parse/src/instruction.rs` | 2,715 | **Doing one thing.** 1,929 code, tests already in `instruction/tests.rs` (`mod tests;` at 2,715). It is one `impl Inst` — a keyword-dispatch parser with ~60 small methods averaging ~30 lines. Coherent; leave it. |
| `rexx-exec/src/eval.rs` | 2,674 | **Not a problem.** 575 code / 849 comment, then 1,190 lines of tests. |
| `rexx-exec/src/ir/compile.rs` | 2,316 | 1,064 code / 724 comment, tests from 1,837. Largest genuinely-dense production file after `run.rs`, and it is one lowering pass. Leave it; revisit only if it doubles. |
| `rexx-exec/src/builtin/convert.rs` | 2,155 | **One builtin family** (669 code), 1,135 lines of tests. Leave. |
| `rexx-parse/src/instruction/tests.rs` | 2,150 | **Exhaustive test file, already split out.** Leave. |
| `rexx-exec/src/builtin/datetime.rs` | 2,125 | **One builtin family** (828 code) — the date/time format tables. Leave. |
| `rexx-exec/src/ir/golden_tests.rs` | 2,105 | **Golden test file.** Leave. |
| `rexx-exec/src/error.rs` | 2,075 | **A transcribed catalogue.** 495 code / **1,170 comment** — the highest comment ratio in the workspace — mostly one `impl Raised` (232–1,348) of message constructors. This is exactly the "one coherent table" case. Leave. |
| `rexx-exec/src/builtin/string.rs` | 2,009 | **One builtin family** (635 code). Leave. |
| `rexx-exec/src/plan.rs` | 1,956 | 464 code / 545 comment, tests from 1,043. Fine. |
| `rexx-exec/src/ir/drive.rs` | 1,909 | 975 code / 772 comment; **already uses `mod tests;`** → `ir/drive/tests.rs`. This is the model for rank 1. Fine. |
| `rexx-parse/src/ast.rs` | 1,728 | **Type definitions with doc comments** — the AST itself. Nothing to split. |
| `rexx-exec/tests/ir_dual.rs` | 1,598 | **Integration test** (997 code / 555 comment); the dual-engine equivalence harness. One thing. Leave. |
| `rexx-parse/src/directive/tests.rs` | 1,544 | **Exhaustive test file, already split out.** Leave. |
| `rexx-exec/src/ir/mod.rs` | 1,469 | Below the threshold but worth naming: the largest `mod.rs`. Fine today. |

**Nothing on this list needs splitting.** `run.rs` is not one of a family of oversized files;
it is an outlier at **5.0×** the next largest by total lines (17,381 against `lib.rs`'s 3,446).
But by *production code* it is only **1.8×** the next largest (3,474 against
`instruction.rs`'s 1,929) — which is the number that should govern rank 3, and it does not
justify carving `impl Interp` up.

---

## 5. What a split costs

Being adversarial, because the cheap parts of this are cheap and the expensive parts are
permanent.

**`git blame` and `git log -p` across the moved region.** `run.rs` took 138 commits in the
last 90 days — 16% of the branch's 865 commits — so this is the file people most often ask
"why is this line like this" about. After a move, plain `git blame run/tests.rs` attributes
every line to the move commit. `git blame -C -C -C` and `git log -p -M -C --follow` recover
it, but a reader has to know to reach for them, and `--follow` on a partial split is
heuristic rather than exact. **This tax is permanent and is the single largest real cost.**
It is smallest for rank 1 (a contiguous 7,714-line block, which rename/copy detection scores
very well) and largest for rank 3 (many small hunks from one file into several).

**Bisect.** Not actually harmed: a pure move changes no behaviour, so no bisect can land on
it as the culprit. The cost is that a bisect *stepping over* the move commit gets no
information from it, and that any bisect crossing it must rebuild the whole crate.

**Can the move be made verifiable? Yes, for ranks 1 and 2, and this is the strongest argument
for doing them separately from anything else.** The commit can be constrained to: new files,
`mod` declarations, `use` lines, `pub(super)` on the 18 named items, and `pub(crate) use`
re-exports. Then a reviewer can check it *mechanically* rather than by reading 8,000 lines:

1. `git show HEAD~1:…/run.rs` sliced to the moved ranges, dedented where required, `diff`ed
   against the new files' bodies — must be empty.
2. Comment-line count before against after — must be equal. (The 2026-08-07 note's own
   prescription, and the right one: the standing rule against dropping evidence comments is
   what a big move most plausibly violates.)
3. `cargo doc --no-deps` clean of `broken_intra_doc_links` — **not** `cargo test --doc`,
   which does not check links (see §1's correction).
4. `cargo fmt --check` and `cargo clippy` clean.
5. The suite, the corpus, and `ir_dual` green — necessary but, as the note says, close to
   worthless as evidence: a pure move has no differential signal.

What the reviewer of such a commit can actually check is therefore **(1) that nothing was
edited and (2) that nothing was dropped** — which is exactly the pair of things that can go
wrong. What they cannot check by reading is whether the new boundaries are good ones; that
judgement has to be made before the commit, not from its diff.

**Recommendation.** Do rank 1 alone, as one commit, verified by (1)–(5). It is 44% of the
file for one line of code, zero visibility change, zero gate-script change, an established
in-crate precedent, and the most rename-detectable shape a move can have. Then decide about
rank 2 with the file at ~9,670 lines and see whether it still itches. Leave rank 3 closed —
the file's production body is ~3,500 lines, and the 2026-08-07 note's own reasoning
("the density is this project's evidence-comment style, not sprawl") is still the correct
reading of it.
