# Review: Phase 4e Task 7 (`caf16c90..98ac2042`)

**Spec compliance: PASS.**
**Task quality: PASS with reservations** -- no Critical, three Important, four Minor.

Reviewed at head `13a5bc91` with the tree clean before and after.
Baseline reproduced: **1385 passed, 0 failed, 4 ignored**, 79 `test result` lines.
`cargo fmt --all --check` 0, `cargo clippy --workspace --all-targets -- -D warnings` 0 (warm target).
The two adjudicated concerns (the refuted `varlookup` discharge, the tree-walker's 0.2-1.3%) are not re-litigated here.

## The `>L>` line, verified against the oracle rather than against the case file

Ten programs written for this review, each run three ways -- C++ oracle, `REXX_ENGINE=tree-walker`, `REXX_ENGINE=ir` -- with stdout, stderr and exit status compared as separate descriptors and byte for byte, from a fresh empty directory under the standard `ulimit -v` wrapper.
Under both `trace i` and `trace r`, for a literal (`n1 = 'abc'` / `say 'abc'`), a symbol read (`n1 = zv` / `say zv`), a computed compound tail (`aa.zi = 'v3'` / `say aa.zi`), a whole stem (`st. = 'dflt'` / `say st.`) and an expression with a sub-expression (`n1 = (zi + 3) * 4` / `say (zi + 3) * 4`), all three agree exactly.
The `>L>`, `>V>`, `>C>`, `>O>`, `>>>` and `>=>` lines and their indents are reproduced, in the oracle's order.

Six further programs push the indent, which is where a promoted clause echoing against a stale value indent would show: a `SAY` in an `OTHERWISE`, an assignment in a `SELECT CASE`'s `OTHERWISE` behind an absorbed `WHEN CASE` (the escape-elevation shift), a `SAY` two `IF`s deep, a promoted pair as a loop body, a promoted pair inside a `DO` block that is an `IF`'s branch, and a `SAY` reached through an escaped `OTHERWISE`.
All agree three ways.

Five trace-staleness programs -- `trace i` switched on mid-body, switched off mid-body, turned on inside a loop on pass 1, set in a called label, and set around an `INTERPRET` -- also agree three ways, so `Op::TraceLiteral`'s run-time gate and `Op::TraceClause`'s compiled one do not come apart per clause.

Eight numeric-literal shapes (`'5'`, `'05'`, `'5.0'`, `'1e5'`, `'0100'`, `'-7'`, `' 8 '`, and the same under `numeric digits 3`) agree three ways, which is what says `Op::Const`'s `Interp::literal` keeps a literal's source spelling.

**Independently of that, every one of the case file's 27 stanzas was re-run under the oracle and compared to its recorded expectation** (with the harness's `INLINE_PATH` substituted for the probe's own path): **0 mismatches**.
So "EVERY EXPECTED BYTE BELOW WAS MEASURED AGAINST THE C++ ORACLE" is true, including the two trailing-space rows the file's header calls out.

**And the file was green before the promotion.** A worktree at `f330a96a` runs `both_engines_agree_on_every_case_file` green (7 passed; the 2 failures are the two ootest-dependent tests, which fail because the worktree has no `ootest` checkout beside it).
So the rows can say whether they would ever have failed.

## Concern 2: the population sweep does catch a dropped `>L>`

**Verified, and the brief's premise is wrong.**
Mutating `Op::TraceLiteral` into a no-op gives **1382 passed / 3 failed**: `both_engines_agree_across_every_population`, `both_engines_agree_on_every_branch_shape`, `both_engines_agree_on_every_case_file`.
The sweep's own message names the programs:

```
2 of 10391 programs behave differently on the two engines:
[corpus] corpus lang/trace_output.rex: stderr: ... (452 bytes) vs ... (429 bytes)
[corpus] corpus lang/raise_array_substitution.rex: stderr: ... (582 bytes) vs ... (560 bytes)
```

`lang/trace_output.rex` is `trace i` with `if y > 5 then say "big"`, and `lang/raise_array_substitution.rex` is `trace i` with `say 'before'`; both are in `phase-4a.txt`, so both are in the sweep's corpus population.
With Task 7's own case file moved out of `tests/ir_dual_cases/`, the same mutation still gives **1382 / 3** -- the sweep and the older case files catch it without the new file.

`compare` in `ir_dual.rs` compares `tw.stderr != ir.stderr` on raw bytes with no normalisation, and its own doc says so ("Stdout, stderr and exit status are compared **unnormalised**: there is no oracle here, so `corpus.rs`'s DEVIATION 0 does not apply").
`tests/support/mod.rs`'s `PREFIX_OFFSET` normalisation is the *oracle* differential's and does not reach this file.

So the distinction the implementer drew is the right one and the correction they wrote into the plan is correct. **It is not complete** -- see findings 1 and 2.

## Findings

### Important

**I1. The refuted premise is still in the spec, which is the document the plan and the briefs derive from.**
`docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md:235` still reads "Every following line still matched, so only an exact stderr comparison sees it -- and no corpus instrument here does."
Line 229 carries the same claim in its "What it buys" list: "so no corpus instrument can see a trace indent".
Both are measured false above.
Task 7 corrected the plan (`2026-08-09-phase-4e-ir.md`) and the anchor and left the spec alone.
The spec is amendable by convention -- `2ff577bb` "Amend D23: the trace setting cannot be a compile-time-only input" and `d03cc3af` are Task 6's own amendments to this file, and D23 carries an inline "Amended 2026-08-09 after Task 6 measured the case the original wording gets wrong."
This crate's own rule is the one that makes it Important rather than cosmetic: "When a plan or brief is wrong, correct the plan -- not the message that carries the work", because "briefs regenerate from the plan, reviewers review against the brief". A brief regenerated from a spec that still says no corpus instrument can see it inherits the same wrong premise, which is exactly what happened to this brief.

**I2. `ir_dual.rs`'s module doc now states the opposite of what this task measured, and it is untouched.**
Lines 19-29 say: "**It cannot see a promotion that shares its semantics, and every promotion so far does.** ... So the two arms agree by construction, and this file's comparison is not evidence about expression evaluation, arithmetic, or any construct's semantics." and "a promotion that shares its semantics cannot diverge, and one that re-implements them can, which is what this comparison is here to catch **when it arrives**."
It has arrived. `Op::Const` does not enter `eval` at all, the ordering of load-then-echo is re-implemented as two ops, and the M1 mutation above proves the file's comparison *is* evidence about a literal's evaluation and its trace side effect.
The `StackSpan` divergence (finding m1) independently proves the arms no longer agree by construction.
`ir_dual.rs` was last changed at `b6d54856`, before this task.
This is the doc that produced the brief's wrong premise, and both the report and the plan now cite this file as a real guard while the file itself denies being one.

**I3. `Op::Const` and `Op::TraceLiteral` execute zero times on every registered benchmark axis, and nothing says so.**
`Op::Const` fires only for `ExprKind::Literal`, which is a *quoted* literal; an unquoted number is `ExprKind::Constant` and reaches `Op::EvalExpr` (the report is explicit about this, and `crates/rexx-parse/src/ast.rs:112` `Constant(SymbolId)` plus `eval.rs:316`/`:319` confirm it).
No loop body of `emptyloop`, `varlookup`, `arith`, `compound`, `strings` or `alloc4c` assigns or `SAY`s a bare quoted literal: every body clause's value is an operator, a call or a variable read (`alloc4c`'s `s = "item" || i` is a concatenation, `dispatch`'s `total = 0` is a `Constant`).
The report says this of `varlookup` only ("neither gets an `Op::Const` at all") and does not generalise it.
Two consequences the next task needs: the task's headline op pair has **no** performance evidence in either direction -- the saved `eval` entry on `Const` against the added op dispatch and run-time gate on `TraceLiteral` -- and **Task 7-M is now blocked on per-clause cost measured entirely on clauses that never run the new ops**, so a remedy tuned on those axes is untested against the shape Task 7 actually added.
Related: `docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md:229` claims "an untraced chunk then pays nothing at all, not even a flag test per clause". `Op::TraceLiteral` is emitted unconditionally and gates at run time (`echo_literal` tests `tracing_intermediates()`), so an untraced clause holding a literal now pays an op dispatch and a flag test. The decision itself is defensible and follows `Op::TraceKeyword`'s stated precedent -- though the report's reason (widening `ChunkTrace` "would put a second emission decision under a staleness rule of its own") understates what already exists, since the `Echo::Gated`/`Echo::Compiled` split is exactly that staleness rule and is already per clause. What is wrong is that the spec sentence the design now contradicts was left standing.

### Minor

**m1. The `StackSpan` divergence is real, is inert, and is recorded only in a file git ignores.**
Measured directly (temporary untracked test, removed; tree clean): for `say 'a'` and for `n1 = 'a'`, `Outcome.stack.max_depth` is **1** on the tree-walker and **0** on the compiled stream; `bytes` is 0 on both, so `bytes_per_frame()` answers `None` either way.
Task 11's flip does not redden the two in-repo readers, confirmed by reading them: `tests/spike.rs`'s `TERMS`-term `||''` chain and `lib.rs`'s 1000-term chain plus `INTERPRET` both reach `max_depth` through an operator chain, which is `Op::EvalExpr`, and `interpret`'s own fragment stays on the tree-walker under both engines.
So the implementer's judgement is right. What is left is that `Outcome.stack` is public, `ir_dual.rs` compares three channels only, `KNOWN_DIVERGENCES` does not cover a non-stdout difference, and the only record is `task-7-report.md`, which `.gitignore:19` excludes. Put it in the tree -- `Op::Const`'s doc or `StackSpan::max_depth`'s -- or assert it.

**m2. `Op::TraceLiteral`'s doc states an arrangement that nothing structural enforces.**
"**Only valid inside a [`Op::Clause`] region**, and immediately behind the `Const` whose register it reads."
`compile`'s `assert_trace_ops_open_a_clause_region` matches `Op::TraceClause` only (`compile.rs:869-884`), so neither half is asserted for `TraceLiteral`, and `run_ops` has only the `holds_register` debug assertion.
Mitigated: the report's M3 (the echo op in front of the load) goes 14 red on the golden streams, so the inversion is caught for today's shapes. The gap is the future one the sibling assertion exists to cover, and this crate's own rule is "if such a claim is load-bearing, assert it in a test".

**m3. The report misattributes where the "this task reads `plan`" promise lived.**
"`plan` is still not read, and **the plan's own text** said this task would read it."
The promise was `compile`'s own doc comment ("a task that promotes an assignment reads it to place the assignment's own `EvalExpr`"), which the diff correctly rewrites. Nothing in `docs/superpowers/plans/` or `docs/superpowers/specs/` says it -- grep for `reads it to place` and `name-to-slot` finds no such claim in the plan or the spec, and the plan's Task 7 section does not mention `plan` at all.
Small, but the sentence sends a reader to the wrong document.

**m4. The plan's Task 7 text is now one indirection out of date.**
"**`Store` goes through `assign_expr_target`**, which is what `step`'s own `Assignment` arm calls." `step`'s arm now calls `Interp::assign_evaluated`, which calls `assign_expr_target`. Still true transitively and the property it protects is intact; noted only so it is not read as a direct-call claim.

## Store, Say, and the shared implementation

Verified in the tree, not from the diff alone.
`run.rs:1327` (`step`'s `Assignment` arm) and `ir/drive.rs:908` (`Op::Store`) both call `Interp::assign_evaluated` (`run.rs:2781`), which is the sole caller of `assign_expr_target` on this path (`run.rs:2810`); the other `assign_expr_target` callers are `PARSE`'s (`run.rs:6494`, `:6502`, `parse_template.rs:679`), which pre-date this task.
`run.rs:1321` (`step`'s `Say` arm) and `ir/drive.rs:932` (`Op::Say`) both call `Interp::say_evaluated` (`run.rs:2743`).
The order inside each extracted function is the order the deleted arms had: `push_temp`, then `result_text`, then `trace_result`, then the write.
The `>=>`/`>C>`/`>>>` lines and the stem and compound-tail dispatch are therefore one implementation, which the oracle probes above confirm from the outside.
`echo_literal` (`trace.rs`) is likewise the one implementation `eval.rs`'s post-order hook and `Op::TraceLiteral` both enter.

## Boundaries: no divergence found

Twenty-two programs constructed for shapes the 27-row table does not hold, each run three ways. All agree except where noted.

* a handler that queues again at a promoted assignment that is the **last body clause of a loop**;
* the same at the **last clause of a called label**, before its `RETURN`;
* the same at a **`SAY`'s** boundary rather than an assignment's;
* the same at the **program's final clause** before `EXIT`;
* a **three-deep** requeue chain (`h` queues `zy`, `g` queues `zw`) across three consecutive promoted clauses;
* a requeue at a promoted `SAY` inside a loop, and at a promoted clause that is an `IF`'s **false-path landing instruction** (the one the `EndBranch`/`Jump` ops sit in front of) and one that **follows a taken branch**;
* a handler that **fails** at a promoted assignment's boundary inside a loop inside a called label, and at a promoted `SAY`'s boundary inside a matched `WHEN` inside a loop;
* `NOVALUE` raised by the assignment **target's own tail**, i.e. from inside `assign_expr_target` inside the region, plain and under `trace i`;
* a handler that `SIGNAL`s out of a promoted clause's boundary, from a loop body and from an `IF` branch;
* `PROCEDURE` behind a promoted **`SAY`** (17.1, three-way agreement), `USE LOCAL` behind a promoted assignment, `PROCEDURE` as the **callee's** first instruction where the caller is a promoted `SAY` that calls it, and `PROCEDURE EXPOSE` in that callee;
* a promoted pair as a `DO OVER` body, plain and traced.

Both `KNOWN_DIVERGENCES` rows still diverge exactly as recorded, and the compiled stream is still the arm that matches the oracle in both (oracle `after` / `G ran 4` and `G ran 7` / `after`).

**Out of scope, found while hunting, pre-existing and not this task's.** A `CALL ON` handler that requeues at the **last clause of a `DO` block that is an `IF`'s or a matched `WHEN`'s branch** reports `SIGL` one clause early on **both** engines: oracle `G ran 6`, both engines `G ran 5`; and oracle `G ran 7`, both engines `G ran 6`. Reproduced with an unpromoted `CALL raiser` in the same position, so it is independent of the promotion and was there before it. Nothing in the workspace covers it. Recorded here so it has somewhere to live.

## Mutation results

Six rows replayed, each `cp`-backed up, edited, `cargo test --workspace --no-fail-fast`, restored, `sha256sum -c` verified, and rebuilt. `git status` clean after every round.

| row | report | replayed | catchers seen |
|---|---|---|---|
| M1 `Op::TraceLiteral` a no-op | 3 red | **3 red** (1382/3) | population sweep, `BRANCH_CASES`, case files |
| M1, new case file held out | 3 red | **3 red** (1382/3) | same three |
| M7 no interning | 1 red | **1 red** (1384/1) | `one_literal_written_twice_is_one_interned_constant` alone |
| M9 assignment never releases | 1 red | **1 red** (1384/1) | `two_assignments_and_two_says_in_one_body_reuse_one_register` alone |
| M10 `Const` uses `text` | **0 red** | **0 red** (1385/0) | -- |
| M14 region does not spend the permission | 1 red | **1 red** (1384/1) | case files, `loop-header-boundaries`' `do i = 1 to 1; procedure; end` row |
| M15 region is not granted it | 1 red | **1 red** (1384/1) | case files, this task's "procedure after an assignment in a called label" row |
| M15, new case file held out | -- | **0 red** (1385/0) | -- |

Every replayed row reproduces its tabulated count exactly at head, which is the evidence that the table was taken after the last test landed: the last test landed in `3bcbcc53` and `f55ea409` adds only two `#[inline]` attributes (`git show --stat`), so head and the table's suite are the same suite.

**"Adds coverage for exactly one mutation" is confirmed.** M15 with the file held out is green at 1385/0 and the failing row under M15 is exactly `procedure after an assignment in a called label` (the assertion prints the 17.1 report against `""`), so the file is M15's only catcher. M1 with the file held out is still 3 red, so it adds nothing there.

**M10's green is legitimate, and I pushed it further than the suite does.** Under the mutation, 14 of the oracle probes above -- including the eight numeric-literal spellings and the `numeric digits 3` set -- still match the oracle byte for byte on stdout and stderr. `Interp::literal` inlines only a literal whose bytes are already the canonical rendering (`value.rs:62-73`, `canonical_small_int`), and `Interp::number` already puts a `SmallInt` in front of every consumer, so the wrong call there is a representation choice. Leaving it green with `Op::Const`'s doc saying why is the right call.

## Ordinary checks

* No em-dashes in any added line, in `rust/` or `docs/` (`git diff | grep '^+' | grep '—'` empty).
* No `unsafe` anywhere in the diff; the workspace still sets `unsafe_code = "forbid"`.
* No count of a mutable in-repo aggregate in tracked prose. The report's "nineteen-arm match" and "nine arguments" are such counts, but `task-7-report.md` is excluded by `.gitignore:19`, so they are ledger and not repo prose.
* Doc comments state contracts; reasoning sits at the decision point (`Op::Const`'s "why not `text`", `push_value`'s "two shapes", `Constants`' "why a lifetime and not an owning key", `assign_evaluated`'s `inline` measurement). No task numbers, no "used to", no history in comments.
* **The two corrected comments are now true, verified by running both halves.** `run_clause_region`'s take: M14 red, and the catcher is `loop-header-boundaries`' `procedure as a loop body's first instruction` row, exactly as the comment names. The grant in front of the region: M15 red, and the catcher is `assignment-and-say`'s `procedure after an assignment in a called label` row, exactly as the comment names. Each half is load-bearing and each names a real witness.
* `every_instruction_of_an_all_generic_body_compiles_to_one_generic_op` is now `nop` / `nop` / `drop n1` -- three unpromoted instructions -- and asserts the constant table is empty. The subject change is right and its doc says what should redden it.
* Three `op_of` expectations were removed and three replaced; no assertion was dropped. Spot-checked `an_if_with_an_else`: `op_of == [0, 3, 4, 8, 11, 15, 19]` matches the rendered stream instruction by instruction, and the last entry is one past the last op.
* `Engine::TreeWalker` is still the default (`invocation.rs:191`, `:311`).
* `ir_dual.rs` itself is unmodified, so `LOOP_CASES`, `BRANCH_CASES` and `KNOWN_DIVERGENCES` were not retrofitted and the `REWRITE` refusal covers the new file unchanged.
* The review package's diff holds all 12 files `git diff caf16c90..98ac2042` produces; it is the same diff at wider context.

## What I did not verify

* M2, M3, M5, M6, M8, M9b counts (six of the twelve rows). The six I did replay each reproduced exactly, at head.
* Any performance number. No benchmark was run, per the brief.
* Whether `arith`'s base instruction count is non-deterministic to 0.08% as the report says, or whether `emptyloop` is byte-identical on both arms -- both are benchmark claims.
* The suite at BASE `caf16c90` (the report's 1382). The arithmetic is consistent: three new golden tests against the 1385 measured here.
