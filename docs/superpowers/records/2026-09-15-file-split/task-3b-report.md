# Task 3b report: `rexx-exec/src/run.rs`

BASE `36bbb3684`. Eight code commits, then this report's commit. Every artifact
cited is under `docs/superpowers/records/2026-09-15-file-split/task-3b-files/`
(`files/` below). Tooling is Task 2's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | child `wc -l` | `run.rs` after |
| --- | --- | --- | --- | --- |
| c1 | `65d8bfd52` | clause indent computation and `Interp::printed_indent`, to `run/indent.rs` | 356 | 8928 |
| c2 | `dcce5b15d` | the `raised_*` constructors, `raise_syntax_condition`, `raised_naming_the_operand`, `raised_from_settings`, to `run/raised.rs` | 205 | 8748 |
| c3 | `9aa22f02d` | SELECT's clauses and the IF/WHEN/SELECT branch-target arithmetic, to `run/select.rs` | 351 | 8424 |
| c4 | `91ffafaaf` | INTERPRET fragments, DROP, the debug pause, to `run/interpret.rs` | 307 | 8145 |
| c5 | `863e9b813` | ADDRESS, NUMERIC, TRACE and the `>I>`/`<I<` tracers, to `run/settings.rs` | 399 | 7769 |
| c6 | `2d9aadebd` | condition traps and delivery, RAISE, SIGNAL, to `run/condition.rs` | 962 | 6830 |
| c7 | `d37641150` | CALL and function-call resolution, with the call types, to `run/call.rs` | 1145 | 5715 |
| c8 | `5c5380b79` | DO/LOOP, with the loop types, to `run/loops.rs` | 2322 | 3430 |

`run.rs` went from 9264 lines to 3430. Each commit message says
`git blame -w -C -C -C` recovers the moved lines. `impl Interp` members moved
one by one: each child holds its members in one `impl Interp { ... }` block of
its own, every member still indented as a member, so the moved text is
byte-identical to its BASE text except where listed below.

## What stays in `run.rs`

`Flow`, `Ended`, `Echo`, `SteppedClause`, `Resolved`, `LeaveOrigin`,
`ConditionTrace`, `run_activation`, `grant_procedure_permission`,
`apply_flow`, `exec_instruction` (whole) and `run_bounded`; and the clusters
the survey left without a home, each kept for the reason given:

* **message, PROCEDURE, EXPOSE, USE, SAY** (`exec_message` to
  `say_evaluated`): each is one arm's body that `exec_instruction` calls; no
  ruled child is about them, and a module for them would be a child the
  proposal did not rule.
* **GUARD, REPLY, FORWARD, QUEUE, RETURN, assignment**, with `set_sigl` and
  `reserved_result_slot`: the same reason. `set_sigl` is also called from
  `condition.rs` and `call.rs`, and `top_level_clause` stays beside its one
  caller, `exec_reply`.
* **Stepped-clause bookkeeping, trace echo, failure sites**
  (`enter_stepped_clause` to `record_failure_site_at`, and
  `end_promoted_branch`): the clause boundary every clause and the IR driver
  cross; it reads `SteppedClause`'s private fields.
* **Condition and chunk evaluation** (`eval_if_condition` to
  `test_case_when`): shared by IF, WHEN, WHILE/UNTIL and the IR driver;
  `test_case_when` is called both by `exec_instruction` and by `select.rs`.
* **Name shape and indirect words**: `NameShape`/`shape_of` are named across
  the crate; `split_indirect_words`, `is_symbol_byte` and
  `validate_indirect_word` serve PROCEDURE EXPOSE and EXPOSE (which stay) as
  well as DROP. `control_slot` went to `loops.rs`, its only user.
* **Stream resolution** (`resolve_stream`, `check_stream_access`,
  `forget_stream`, `standard_stream_name`), after `mod tests;`: not in the
  survey at all; no ruled child is about it (concern 1).
* `clause_site`, `clause_line`, `clause_line_at`, `Conversion`,
  `makearray_lines`, `ReturnKeyword` and `QueueKeyword` stay as well.
  `MAX_ADDRESS_NAME_LENGTH` went to `settings.rs` with `exec_address`.

## Departures from the ruled proposal

1. **Boundaries re-derived at BASE**, from `tools/plan_run.py`: `impl Interp`
   members by BASE line range, other items by key. The survey's ranges are from
   `5d84dd8cb` and drift by up to about 35 lines here.
2. **Types.** The survey places no type. Each moved with the one child that is
   its only reader inside `run`, found by a word scan of every unit at BASE
   (`tools/plan_run.py` records the result): the loop types `DoOutcome` through `LoopHeaderValues` with
   their impls to `loops.rs`; `CallResolution`, `Entered`, `CallEntry`,
   `entered_receiver` and `MAX_ACTIVATION_DEPTH` to `call.rs`. `Resolved`
   stays, as the brief says. Paths other modules name keep working through
   `pub(crate) use` in `run.rs`, each listed in its commit message.
3. `Interp::printed_indent`, in the survey's SELECT range, went to `indent.rs`:
   it is `static_indent` plus the activation's offsets.
4. `set_debug_skip`, in the survey's INTERPRET/debug-pause range, went to
   `settings.rs`: its only caller is `exec_trace`, and it is `TRACE n`.
5. `enter_fragment`/`leave_fragment`, in the survey's TRACE range, went to
   `interpret.rs`: their callers are the INTERPRET arm and
   `run_debug_fragment`.
6. `end_promoted_branch` (start of the SELECT range), `run_bounded`,
   `top_level_clause` (branch-target range) and `test_case_when` stay, for the
   reasons above. `seal_site_level` went to `interpret.rs` with its range,
   though `call.rs`, `dispatch.rs` and `lib.rs` call it too.
7. `numeric_less`, `round_via_unary_plus` (outside the survey's raised range)
   and `control_slot` (in its name-shape range) went to `loops.rs`;
   `condition_name` to `condition.rs`; `raise_syntax_condition`,
   `raised_naming_the_operand` and `raised_from_settings` to `raised.rs`.
8. **Two imports under `#[cfg(test)]`**: `run/tests/message.rs` names
   `Entered` and `entered_receiver`, and `run/tests/loops.rs` names
   `numeric_less`, through `run/tests.rs`'s `use super::*;`. Nothing else in
   `run.rs` uses them, so a plain import is an unused-import warning outside
   tests; `#[cfg(test)] use call::{Entered, entered_receiver};` and
   `#[cfg(test)] use loops::numeric_less;` keep every test's path and name.
9. No test moved, so no test's fully qualified name changed.

Visibility: items another part of `run` reaches became `pub(super)` in their
child (listed per commit in `files/c<N>/c<N>-instrument2.txt`, and in total in
`files/final/cumulative.txt`); items that were `pub(crate)` kept it. No field
visibility changed.

## Pinned items

* `/bin/grep -rn 'src/run' crates/*/tests crates/*/src`: only
  `rexx-core/tests/unsafe_sites.rs`, which names `crates/rexx-exec/src/run.rs`
  as a file its scan must reach. `run.rs` still exists.
* `refusal-sites.tsv`: re-derived at every commit
  (`files/c<N>/c<N>-refusal-sites.txt`, each showing the mtime moved). c1
  shifted the 19 `run.rs` rows' line numbers; c2 moved them to
  `run/raised.rs`. At every commit, every column other than column 4 is
  identical row for row to the parent's, all 247 rows, and the header lines are
  identical. Column 3 of the moved rows stays `body` / `body+ir`, since
  `run/` is `body` like `run.rs`.
* Prose naming `run.rs` for a thing that moved was corrected in the commit that
  moved it: `error.rs` (c1, c2, c7), `run/tests/signal.rs` (c3), `trace.rs`
  (c5, c8), `eval.rs`, `clause.rs`, `lib.rs`, `tests/owners.rs` (c7),
  `corpus/phase-4b.txt` (c8); each diff is `files/c<N>/c<N>-other-edits.diff`.
  Left as they are: the read-only corpus programs and their
  `sourceline_oracle/` copies (`signal_forms`, `mutation_digits_at_render`,
  `call_on_trap_rearms`, ...); `run/tests/indent.rs:283`'s "`run.rs`'s panic
  site named in the fix's own commit" and `condition.rs`'s "`run.rs:2153-2154`
  in the tree this task started from", both records of a past state; mentions
  of things that stayed.

## Comment and doc edits

All are in the commit that made the text false, each a declared edit the
instruments show as a diff:

* c1 `indent_in_range`: "this module's own test" to "`run`'s own test";
  "`clause_site`'s own fallback, this / file" to "..., in / `run.rs`".
* c2 `raised_when_not_logical`: [`Interp::eval_condition`] to
  [`crate::Interp::eval_condition`] (`raised.rs` does not import `Interp`;
  `cargo doc --document-private-items` showed the break).
* c4 `exec_instruction`'s EXIT arm: "`run_fragment`'s own propagating / arm,
  below)" to "arm)".
* c6 `signal_to_label`: "identical fix for `CALL`, immediately below" to
  "identical fix for `CALL`".

`files/final/comments.txt`: of BASE `run.rs`'s 3339 comment lines, exactly the
six lines of those edits are gone; everything added is a license header, a
module doc, a `mod` comment, or a replacement line above.

## The instruments

Per-commit outputs in `files/c<N>/`, made by `tools/checks.sh` over the tree
as committed (the pipeline, `tools/pipeline.sh`, restored each snapshot over
the parent commit, re-ran every check, confirmed the tree byte-identical to the
snapshot, ran instrument 4, and committed).

* **Instrument 1.** (a) the parent's `run.rs` minus the removed lines against
  the new `run.rs`, every equal block also `cmp`'d; (b) each moved unit's text,
  comments above it included, `cmp`'d against its new text after undoing only
  a visibility change.
* **Instrument 2.** Token streams per unit, visibility stripped.
* **Instrument 3.** Decoded literals per unit from raw token trees, walking
  every group (macro arguments included), cross-checked per unit on both sides
  against `tools/rustlex.py`'s independent count; plus the whole-file multiset.
* **Instrument 4.** `memcap 8G cargo test -j 4 --release -p rexx-exec
  --no-fail-fast`, per result block and per test, compared whole with BASE
  (`files/instrument4/`). It ran in a scratch worktree holding the parent
  commit plus the snapshot; `commit_run.sh` refused to commit unless the
  staged `rust/` tree hash equalled the tested one, and every one did (the
  hashes are in `i4-c<N>.meta` and `instrument4/pipeline.log`).
* **Units.** 278 at BASE (`tools/item-tool`).

| # | 1(a) unmoved | 1(b) moved units | 2 tokens | 3 literals | 4 tests | load before / after (1, 5, 15 min) |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 8924 equal, 4 inserted | 4 of 5, 1 declared | 278 of 278 | 278 of 278; 1230 literals; 0 control mismatches | identical | 2.41 6.68 9.02 / 2.24 4.86 6.90 |
| c2 | 8737 equal, 11 inserted | 18 of 19, 1 declared | 274 of 275, 1 declared | 274 of 275, 1 declared; 1216; 0 | identical | 1.70 4.00 6.43 / 2.35 4.53 5.57 |
| c3 | 8416 equal, 8 inserted | 22 of 22 | 259 of 259 | 259 of 259; 1143; 0 | identical | 2.33 4.28 5.45 / 2.21 4.87 5.30 |
| c4 | 8141 equal, 3 inserted, 1 declared | 8 of 9, 1 reflowed | 240 of 240 | 240 of 240; 1066; 0 | identical | 2.20 4.56 5.18 / 1.65 3.96 4.69 |
| c5 | 7766 equal, 3 inserted | 11 of 11 | 232 of 232 | 232 of 232; 1023; 0 | identical | 2.30 3.88 4.64 / 2.20 4.62 4.79 |
| c6 | 6827 equal, 3 inserted | 18 of 19, 1 declared | 221 of 222, 1 declared | 221 of 222, 1 declared; 982; 0 | identical | 2.31 4.34 4.68 / 23.24 35.91 23.01 |
| c7 | 5709 equal, 6 inserted | 34 of 35, 1 reflowed | 204 of 204 | 204 of 204; 879; 0 | identical | 22.91 33.84 23.12 / 2.66 9.41 15.09 |
| c8 | 3421 equal, 9 inserted | 58 of 59, 1 reflowed | 172 of 172 | 172 of 172; 759; 0 | identical | 3.38 8.76 14.63 / 5.43 6.82 10.47 |

"Reflowed" is rustfmt re-wrapping a signature after `pub(super)` was added
(`drop_variable`, `entered_receiver`, `numeric_less`); each is token-identical
once the comma rustfmt adds before the `)` closing the parameter list is
dropped. Instrument 4: 1582 test lines in 50 result blocks, 1581 passed,
0 failed, 1 ignored, at BASE and at every commit.

**Two tooling changes, both re-controlled.**

* `entered_receiver`'s re-wrap exposed a gap in Task 2's relaxation: with the
  comma present, `Option<ObjRef>,` tokenizes its `>` as Joint, so dropping the
  comma still left `>+` against `>`. `splitlib.drop_trailing_commas` now also
  clears the Joint mark on the one token directly before the comma it drops,
  and nothing else.
* Doc-test names end in their source line (`run::Interp::run_activation (line
  790)`), which a move above them shifts. `tools/test_results.py` replaces
  `(line N)` by `(line _)` in doc-test blocks only; the item path, the count
  and the verdict are still compared.

**Controls.** Task 2's five, on Task 2's c1 pair, with this task's tooling:
5 of 5 caught before the first move (`files/final/controls-task2-before-first-move.txt`)
and at the end (`files/final/controls-task2.txt`). This task's own four, on
c8's pair (`tools/controls_run.py`, `files/final/controls-run.txt`): an unmoved
`run.rs` line gaining a space (I1a), a space inside a moved `debug_assert!`
message (I3, token-blind), a moved token change (I1b, I2), and a token change
beside the comma the relaxation drops (`numeric_less`'s `fuzz: u64,` to
`u32,`; I1b, I2): 4 of 4 caught.

**Cumulative** (`files/final/cumulative.txt`): every one of BASE's 278 units is
present exactly once after c8, identical modulo visibility, or a declared edit.

`cargo doc --no-deps -p rexx-exec`: 2 warning lines, none in `run.rs` or
`run/`, at BASE and every commit. With `--document-private-items`: 53, one in
`run.rs` (BASE's `[`Interp::step`]` in `exec_instruction`'s doc), at BASE and
at every commit as committed; c2 showed its broken link before it was fixed.
With `--cfg test` as well: 52, the same one. `tools/tests_links.sh` (the
`#[test]` lines stripped from `run/tests.rs` and `run/tests/*.rs`): one
warning, BASE's own in `run/tests/message.rs:66`, at BASE and every commit.

## Performance

callgrind `summary:` minus `libc.so.6` and `ld-linux`, two interleaved rounds,
BASE built in its own worktree and target directory, c8 in the main
checkout's; `.text` sha256 `f6840a66...` (BASE, the same as Task 2's c7) and
`da915d7c...` (c8). `files/perf/results.txt` has the commands and raw figures.

| program | BASE round 1 | BASE round 2 | c8 round 1 | c8 round 2 | change |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17897281850 | 17897269939 | 17897276202 | 17897270309 | -0.00001% |
| nop | 9340095825 | 9340101869 | 9340098936 | 9340095640 | -0.00002% |
| assign | 19540359902 | 19540363573 | 19540360482 | 19540357306 | -0.00001% |
| emptyloop | 9285882102 | 9285882851 | 9285881270 | 9285883334 | -0.00000% |
| varlookup | 14842896448 | 14842899362 | 14842902737 | 14842896505 | +0.00001% |
| arith | 11519344673 | 11519344093 | 11519339397 | 11519339631 | -0.00004% |
| compound | 9234392334 | 9234395965 | 9234392866 | 9234390632 | -0.00003% |
| dispatch | 20471075413 | 20471073085 | 20471078671 | 20471076023 | +0.00002% |
| strings | 17788063283 | 17788065447 | 17788058872 | 17788054477 | -0.00004% |

No axis moves more than 0.00004%, so nothing was bisected. The spread between
two runs of one binary is the same size.

## Gates

Run over this report's commit by `files/tools/gates.sh` after committing; the
results are recorded in the commit that follows this one.

## Concerns

1. **`run.rs` is 3430 lines**, most of it clusters the survey gave no home:
   the instruction bodies (message through assignment, about 1250 lines), the
   stepped-clause boundary (about 400), condition and chunk evaluation (about
   300), stream resolution (about 120). The instruction bodies and the stream
   resolution are the parts that are not the loop.
2. `loops.rs` (2322) and `call.rs` (1145) are over the trigger; each is one
   construct. `loops.rs` holds the nested and the flat-loop paths side by
   side, which could be a later cut.
3. **The arity suites do not follow `CARGO_TARGET_DIR`.**
   `tests/support/arity.rs` runs `rust/target/release/rexx-run` by path. My
   first BASE instrument-4 run, in a worktree with no `rust/target`, failed
   those rows; with a target directory elsewhere and an old binary at
   `rust/target`, they pass against that old binary. Task 2's instrument 4 and
   gates set `CARGO_TARGET_DIR` elsewhere while the main checkout's
   `rust/target/release/rexx-run` was dated 2026-09-23, so their
   `collection_arity` and `introspection_arity` verdicts compared a binary that
   was not the commit under test. Here instrument 4 linked `rust/target` to the
   test target directory, and the gates use the default one.
4. `exec_trace` carries `exec_numeric`'s doc comment ("`NUMERIC
   DIGITS`/`FUZZ`/`FORM`...") at BASE, and `exec_numeric` has none. Moved
   as it was.
5. `lib.rs`'s comment on the call-context push cites "the
   `push_temp(argument.value())` beside the argument list" in `Interp::invoke_call`;
   that text is in no file at BASE. c7 changed only its file name.
6. Instrument 4 for c2 records `head 36bbb3684`: the worktree's move to c1
   failed on c1's files, so it tested BASE plus the cumulative c2 snapshot. That
   tree is c2's tree, and the commit check confirmed its hash. `sync_wt.sh` was
   fixed before c3.
