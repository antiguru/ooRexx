# Task 3 review: a plain `WHEN`'s condition compiles (`d3372e429`)

**Spec compliance: PASS.** Every requirement of the brief is met in the tree, one
of them by pre-existing rows the report names, and one (Step 4's "`trace r`/`trace
i` over each") met in spirit rather than to the letter without the deviation being
declared.

**Code quality: PASS.** The `WHEN` arm is the `IF` arm with the keyword swapped and
the fallback op changed, which is what the design called for. Findings below are
prose and duplication nits; none is a behaviour defect, and I found no divergence.

## How this was checked

Detached worktree at `d3372e429`, own `CARGO_TARGET_DIR`; the repository working
tree was not touched. `ootest/` and `rust/corpus-l1/` are gitignored and had to be
symlinked in, and `every_blocked_axis_still_fails_on_this_crate` resolves
`target/debug/rexx-run` relative to the crate rather than through
`CARGO_TARGET_DIR`, so that binary had to be linked in too. With all three in
place:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0, zero lines
  matching `^warning|^error`.
* `memcap 8G cargo test --workspace --no-fail-fast` -- **1471 passed, 0 failed, 4
  ignored**, exactly the report's number.
* `const _: () = assert!(size_of::<Op>() == 16)` is unchanged at
  `ir/mod.rs:80` and the workspace compiles, so the width holds at 16 after the
  tag. `Op::Condition` is `u32 + u16 + ConditionKeyword` = 7 bytes plus the
  discriminant; nothing was widened and no variant was reshaped to make room.

## The three specific checks

**Only a listed `WHEN` with a native condition promotes.** `compile.rs:648-679`.
The outer arm is unchanged (`When | WhenCase if when_info[index].is_some()`); the
inner `match` promotes only `InstructionKind::When { condition, .. } if
native_shape(condition, Some(NodePath::ROOT))`. All three excluded shapes are
right:

* a `WhenCase` cannot reach the promoting arm at all -- it is a distinct
  `InstructionKind`, and `rexx-parse`'s `if_instruction` maps
  `EnclosingSelect::Plain -> When` and `EnclosingSelect::Case -> WhenCase`, so
  `info.case` is `None` for every `When` that can promote. Even if the parser
  invariant broke, `scan_when`'s `When` arm ignores `case_text`, so behaviour
  would not change -- the report's uncertainty #2 is accurate and correctly left
  unasserted;
* a declining `When` falls to the `_` arm and keeps `Op::WhenTest` with
  `case: info.case`, pinned by the new
  `a_when_condition_outside_the_native_set_stays_one_when_test` (`.nil` is
  `ExprKind::DotVariable`, which `native_shape`'s `_ => false` declines -- checked
  in `rexx-parse/src/ast.rs:152` and `compile.rs:924-947`);
* an absorbed `WHEN` has no `when_info` entry, never reaches the arm, and stays
  `Op::Generic`, pinned by the new `an_absorbed_when_compiles_to_generic` (op 13
  is `Generic index=3` and there is no `Clause index=3`, exactly as its doc says).

The mark/alloc/`push_native`/`close_region`/`release` sequence is byte-for-byte
the `IF` arm's, with `Op::EnterWhen` still emitted past the region.

**The raiser is right on both sides.** Tree-walker: `scan_when`'s `When` arm is
`eval_condition(.., ConditionTrace::Result(indent), raised_when_not_logical)`
(`run.rs:5287`), unchanged. Compiled: `drive.rs`'s one `Op::Condition` arm passes
`keyword.raiser()`, and `ConditionKeyword::raiser` maps `If ->
raised_if_not_logical` / `When -> raised_when_not_logical`. `eval_if_condition`
differs from the `When` arm in exactly the raiser -- same trace variant, same
`checked` computation -- so "one arm for both keywords" is true rather than
convenient. The doc's oracle citation is a measurement and I re-took it from an
empty cwd: `if 'x' then nop` -> rc 222, `Error 34.1: ... following IF keyword ...`;
`select; when 'x' then nop; end` -> rc 222, `Error 34.2: ... following WHEN keyword
...`.

**Prose falsified in a file this task did not otherwise touch.** I searched the
whole tree (`/bin/grep -a`) for `WhenTest`, `scan_when`, `Op::Condition`,
`native_shape` and for the docs that enumerate slots. Nothing untouched is
falsified:

* `eval_chunk_expr`'s own doc (`run.rs:6892-6933`) is untouched and stays true --
  no `When` arm was added to it, so its enumeration is unchanged;
* `drive.rs`'s `Op::WhenTest` arm comment ("exactly as an `IF`'s condition is")
  still holds;
* `tests/ir_dual_cases/conditions`' header is scoped to `IF` throughout;
* `docs/superpowers/plans/{phase-4f-record,phase-4e-handoff,phase-4f-expression-spike,2026-08-12-expression-promotion}.md`
  mention `native_shape`/`Op::EvalExpr` only in ways this task does not reach.

The one stale statement is *inside* a file the commit edited -- see Finding 1.

## Independent measurement

**Oracle re-capture of all four new case rows**, from a directory created empty and
verified empty afterwards, absolute paths, the wrapper the plan specifies. All four
are byte-identical to what is committed on stdout, stderr and exit status,
including the trailing blank on every `when ...` echo, the doubled `3 *-*   when
'x' ` line before the 34.2 report, the `>K>   "CASE" => "2"` line, and the
`SELECT CASE`'s two `>>>` per value. The absorbed-`WHEN` row's surprising middle
-- inner condition traced `>>>  "1"` and its consequence never run, control leaving
the `SELECT` without reaching `OTHERWISE` -- reproduces on the oracle exactly as
recorded.

**Six further three-way probes** (oracle vs `REXX_ENGINE=tree-walker` vs
`REXX_ENGINE=ir`), on shapes no committed row holds. **No divergence on any
channel on any of them:**

* `when 1, 0 then` -- a comma list that declines and matches nothing (rc 0);
* `when 'x', 1 then` -- 34.6 from inside the list, not 34.2 (rc 222), which is the
  live check that the promotion did not capture a list;
* `when st.zi == 'v' then` under `select label s`, `trace i` -- a compound tail in
  a promoted condition, so the `plan` slot path is exercised (rc 0);
* `signal on syntax` over `when 1/0 = 1 then` -- a condition that raises *during*
  evaluation rather than at validation (rc 0, trapped);
* `signal on novalue` over `when zq = 1 then` under `trace i` (rc 0);
* `select case 'x'` followed by `when \0 then` -- a `WhenCase` and a prefix
  condition in one program (rc 0);
* plus `call on user zx name h` with the handler raised from a routine called
  inside the `WHEN`'s condition and delivered at that clause's boundary (rc 0), and
  a `SELECT` nested in a `DO` inside an `IF` branch with a 34.2 at the deeper
  indent (rc 222) -- both agree three ways.

**Two mutations reproduced from scratch**, each a full workspace run under
`--no-fail-fast` with `rexx-run` rebuilt and linked, reading run counts rather than
status:

| mutation | reddened | matches report |
|---|---|---|
| MT1: a `WHEN`'s `Op::Condition` emitted `ConditionKeyword::If` | `both_engines_agree_on_every_case_file`, `ir::corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops`, `a_select_with_an_otherwise_compiles_to_a_scan_chain_and_two_frames`, `a_select_with_no_otherwise_scans_out_onto_its_own_end`, `a_call_in_a_whens_condition_is_addressed_at_the_conditions_slot`, `an_absorbed_when_compiles_to_generic` -- **6** | exact |
| MT2: `chunk_node_at` loses its `When` arm | `both_engines_agree_on_every_case_file`, `both_engines_agree_across_every_population`, `both_engines_agree_on_every_branch_shape`, `the_exempt_set_matches_the_current_failures` -- **4** | exact |

Two things worth recording from those runs. `both_engines_agree_across_every_population`
is green under MT1 and red under MT2, which is exactly the report's asymmetry: the
extracted corpus holds a call inside a `WHEN`'s condition and holds no program that
raises 34.2 from a condition that compiled. And `run::tests::when_condition_that_is_
not_0_or_1_raises_34_2` stays green under MT1 -- `Interp::new()` starts on
`Engine::TreeWalker` (`lib.rs:2084`), so that unit test cannot see a compiled-path
keyword bug at all. That is what makes the new 34.2 row load-bearing rather than a
duplicate of it, and it is the fact the row's rewritten comment now states
correctly.

## The six concerns, adjudicated on evidence

**1. `chunk_node_at`'s doc corrected rather than `eval_chunk_expr` mirrored --
accepted, both halves verified.** `Op::EvalExpr` is emitted at four sites in
`compile.rs` (the loop-header slot, the `IF` decline, the `SELECT CASE`
expression, and `push_value`'s decline for an assignment/`SAY`); none names a
`WHEN`, so a `When` arm in `eval_chunk_expr` would be dead. `eval_chunk_expr` does
carry `(Select { case: Some(..) }, 0)`, which `chunk_node_at` does not and must
not. The replacement rule is exact, not approximate: `chunk_node_at`'s arms are
`{Assignment 0, Say 0, If 0, When 0, Do/Loop slot}`, which is precisely the set of
`push_native` offer sites, and "in both exactly when offered to `push_native` *and*
its declining fallback is `Op::EvalExpr`" yields `{Assignment, Say, If, Do/Loop}`,
which is exactly the intersection. One correction to the framing: the *stated*
containment (`chunk_node_at` subset of `eval_chunk_expr`) was **true** before this
commit and is false only after it; what was already wrong was the choice of
containment as the invariant, which is what the report and the new doc actually
say. The doc also drops the four-slot enumeration, which is right on the plan's own
no-cardinality rule and would have rotted at Task 4.

**2. The self-caught MT1 comment -- accepted, replacement verified true.** The old
sentence ("the only thing in the workspace that reddens") is refuted by my own MT1
run, which reddens six tests. The replacement -- "the only row anywhere that sees
the wrong sub-number in a program's own stderr" -- is what MT1 measures: the four
golden pins see the tag in the rendered stream, the corpus sweep sees it through
`Seen::native_conditions`, and no other test in the workspace produces a program
stderr carrying 34.1 where 34.2 is owed. The two other 34.2 sites
(`run.rs:10120`, `error.rs:1877`) are tree-walker-only and table-only respectively
and are green under MT1, which is consistent rather than a counterexample.

**3. No new row is a unique catch, each says so, and the absorbed row claims
nothing -- accepted; MT1 and MT2 reproduced exactly.** Both mutation sets match the
report test-name for test-name. Every new row's comment names what else reddens
under its mutation, and the absorbed-`WHEN` row's opening words are "A transcript,
claiming nothing about catching", which is the honest label for a row that reddened
under none.

**4. Two rows already in the tree -- verified, and they cover what is claimed.**
`assignment-and-say` line 246, "a SAY inside a matched WHEN's branch under trace i",
runs `select / when 1 = 1 then say 'w'` -- a matching `WHEN` with a native
condition, whose expected block holds the `>L> >L> >O> >>>` sequence a promoted
condition now produces. `trace-settings` line 149 runs `when 1 = 0 then say 'a'`
under `trace r` -- non-matching, `>>>  "0"`. Both are green at `d3372e429` and
their expected bytes are unchanged by the commit, which is the substance of "the
promotion moved no bytes" for those two programs. Not re-writing them was the right
call.

**5. `an_absorbed_when_compiles_to_generic` -- it earns its place; keep it.** I
confirmed it is the only stream-level pin of the absorbed-`WHEN` shape anywhere:
`golden_tests.rs` has no other, and `run::tests`' nine absorbed-`WHEN`/`WhenCase`
tests pin behaviour on the tree-walker only (`Interp::new()` is
`Engine::TreeWalker`), so none of them constrains the compiled stream. Its doc
claims only what the stream shows, and it does **not** claim to be a unique catch,
so the honesty test is passed. It is true that it reddened under MT1 for a reason
unrelated to its subject (the outer `WHEN`'s tag), but that makes it a fragile
witness, not an inflated one.

**6. The corrected comments -- all verified; I found no missed one.** Each of the
eight is genuinely falsified-or-already-false as described: `Op::Condition`'s doc,
`Op::WhenTest`'s doc, `scan_when`'s doc, `raised_when_not_logical`'s doc (the new
comma-list sentence is *not* a reintroduction of the claim `367cc8d61` deleted --
that commit removed the "already raised on any element" mechanism, and the new
wording asserts only that `eval_condition` hands a list over `checked` and that
`native_shape` declines `ExprKind::Logical`, both of which are true from the code),
`drive/tests.rs`' `scan_when`-either-way sentence, `corpus_shape_tests`' already
false "`WHEN CASE`'s evaluate through `Op::EvalExpr`", and the two `SELECT` golden
docs whose op numbers moved (`op_of[7]` is 32 with `EnterOtherwise` at 32 and
`Generic index=7` at 33, and the `END`'s own op is 17 -- both check out against the
committed streams). The dropped "eleven instructions" is correct under the
no-cardinality rule. The neighbourhood sweep for missed ones turned up nothing in
`drive.rs`'s `WhenTest` arm, `run.rs`'s `Select` arm, `compile.rs`'s assertion
helpers, `eval_chunk_expr`, `Loud::call_op_off_its_node` or the `conditions` case
file.

## Findings

**1. (Low) The plan's own survey table is now stale in the file this commit
edited.** `docs/superpowers/plans/2026-08-12-condition-promotion.md:22` still reads
`| WHEN | 165,924 | promoted, and its condition runs eval inside scan_when |` under
a present-tense column header "how it runs today". That is now false for every
`WHEN` whose condition is native, which after this commit is the ordinary case. The
`IF` row above it ("**every one** runs an `Op::EvalExpr` for its condition") was
falsified by Task 1 in the same way and never corrected, so this is a rot line the
phase is accumulating rather than a defect this task invented. The table is
explicitly a dated measurement, which is a defence; the column header is not. The
report does not name it. Either date the column ("how it ran when this plan was
written") or correct the three rows -- this is the plan's own declared cross-task
failure mode, and it is now on its third instance.

**2. (Low) Step 4's "`trace r`/`trace i` over each" was met with one setting per
shape, and the deviation is not declared.** The new file is `trace r` for 34.2,
`trace i` for the call row, `trace i` for the `SELECT CASE`, `trace r` for the
absorbed `WHEN`. Across the whole tree the pairing is nearly complete once the two
pre-existing rows (`trace i` matching, `trace r` non-matching) and `ir_dual.rs`'s
`BRANCH_CASES` row "select case under trace r" are counted, so the coverage gap is
small and doubling the rows would have been inflation. But the report's "The rows
the brief asked for that were already in the tree" section accounts only for the
matching/non-matching pair and is silent about the trace-setting half of the same
sentence. A deviation taken deliberately should be written down.

**3. (Low) One sentence over-attributes what two green rows prove.** Plan document,
Step 4 addition: "both stayed green through this task, which is what says the
promotion moved no bytes" (the same sentence is in the report). Two green rows say
those two programs' bytes did not move. What says the promotion moved no bytes is
`both_engines_agree_on_every_case_file` over the whole case directory,
`both_engines_agree_on_every_branch_shape`, and
`both_engines_agree_across_every_population` over the extracted corpus -- and MT3's
measured result (the case file stays green when the promotion is switched off) is
the sharper form of the same claim. This is the recurring defect class in its mild
form: a claim slightly wider than its stated instrument.

**4. (Nit) The keyword rendering is written twice, both in test-only code.**
`golden.rs`'s `render_condition_keyword` and `corpus_shape_tests.rs:412-419`'s
counting loop each carry `If => "IF", When => "WHEN"`. `corpus_shape_tests` already
does `use super::golden::render`, so it could have used the same helper (made
`pub(super)`), and the two would not have to be kept in step by hand. Both are
exhaustive matches, so a third keyword is a compile error in both places -- this is
duplication, not a hazard.

**5. (Nit) `Seen::native_conditions`'s doc names the wrong unit.** It says "How
many promoted clauses carry an `Op::Condition`", but the code counts `Op::Condition`
*ops* inside each region (`for op in region.iter()`), where the predecessor counted
clauses (`region.iter().any(..)`). The two agree today because a region holds at
most one `Op::Condition`, and the guard below only tests `contains_key`, so nothing
depends on it. Either say "how many `Op::Condition` ops" or keep the `any`.

**6. (Nit) The inner `match` uses a catch-all where the file's convention is
exhaustiveness.** `compile.rs:672`'s `_ => ops.push(Op::WhenTest { .. })` is
reachable only for a `WhenCase` or a declining `When`, since the outer arm's
pattern already restricts the kinds -- so it is correct. But `Root::of`,
`golden::render` and `assert_region_ops_name_their_clause` all argue in their own
docs for no catch-all so that a new variant is a compile error, and this one would
absorb a new `InstructionKind` silently if a future `SELECT` ever collected one.

**7. (Not a defect, for the next reviewer.)**
`rexx-bench-suite`'s `every_blocked_axis_still_fails_on_this_crate` resolves
`target/{release,debug}/rexx-run` relative to the crate rather than through
`CARGO_TARGET_DIR`, so it fails in any detached-worktree build until the binary is
linked into `rust/target/debug/`. `ootest/` and `rust/corpus-l1/` are gitignored
and must be symlinked in too; without them 27 tests fail for reasons that have
nothing to do with the commit under review.
