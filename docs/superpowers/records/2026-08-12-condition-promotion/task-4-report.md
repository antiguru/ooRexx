# Task 4 report: `RETURN`, `EXIT`, `PUSH` and `QUEUE` get an op (`0d058c086`)

## The shape decision: **two ops, each with a keyword tag** -- not four, and not one

The brief offered four ops, or one op with a tag "if the four tails differ only by which `Flow` they answer".
Read from the code, that premise is **false**, and so is the implied dichotomy.
The four `step` arms as they stood:

| keyword | what the tail does with the value | what it answers |
|---|---|---|
| `RETURN` | `push_temp`, then `result_text` -> `trace_result`, **only when there is a value** | `Flow::Return(value)` |
| `EXIT` | the same, byte for byte | `Flow::Exit(value)` |
| `PUSH` | `push_temp`, `to_text(..).to_vec()` (or `Vec::new()`), then `trace_result` **unconditionally** | `Flow::Next`, after `queue.push(line)` |
| `QUEUE` | the same | `Flow::Next`, after `queue.queue(line)` |

So the pairs split cleanly and the groups do not.
`RETURN` and `EXIT` differ in exactly one constructor -- Task 3's premise, holding again.
`PUSH` and `QUEUE` differ in exactly one method -- and the tree-walker had **already** written them as one arm choosing between `Queue::push` and `Queue::queue`, so a tag there is that existing choice named rather than a new idea.

Between the two groups, three things differ, not one:

* what is traced -- `result_text` (gated on `results`, absent for a bare form) against `to_text` (always rendered, because the queue needs the bytes) with a null string traced for the bare form;
* the side effect -- none against a queue write;
* the region end -- `RegionEnd::Flowed(flow)` against falling through, because a `PUSH` answers `Flow::Next` and a `SAY`'s region already ends that way.

Folding all four into one op would have made the driver arm answer `Option<Flow>` and the shared half branch internally into two functions wearing one name.
Splitting all four would have written each pair's arm twice.
So: `Op::Return { index, src: Option<u16>, keyword: ReturnKeyword }` and `Op::Queue { index, src: Option<u16>, keyword: QueueKeyword }`.

**A third option considered and rejected**: `PUSH`/`QUEUE` is `Op::Say` with a different sink -- `say_evaluated` and the old `Push`/`Queue` arm are the same five lines but for the last statement -- so `Op::Say { index, src, sink }` with `Stdout`/`Push`/`Queue` would have been one op fewer.
It renames an op that every golden stream in the tree already spells `Say`, for no behavioural gain, and `Op::Say`'s own doc is written about printing.
Not done; recorded here because it is the reading that would make "one op with a tag" true.

The two tag enums live in `run.rs` beside the shared halves that read them, which is `HeaderRole`'s arrangement rather than `ConditionKeyword`'s.
`ConditionKeyword` lives in `ir/mod.rs` and converts to a `fn` pointer at the boundary because `condition_value` takes a raiser; here the tag *is* the shared half's parameter, so there is nothing to convert.

## What changed

* `rust/crates/rexx-exec/src/run.rs`
  * `ReturnKeyword` / `QueueKeyword`, beside `ConditionTrace`.
  * `Interp::returned_value(value: Option<ObjRef>, keyword) -> Flow` -- the root, the `>>>` and the `Flow`. Both `step` arms and `Op::Return` enter it.
  * `Interp::queue_evaluated(value: Option<ObjRef>, keyword)` -- the render, the `>>>` and the queue write. `step`'s arm and `Op::Queue` enter it.
  * The three `step` arms reduce to `eval` plus a call. The comments that were on them and are still true moved with the code they describe; nothing was dropped.
  * `chunk_node_at` and `eval_chunk_expr` each gain the four instructions to the `SAY` arm's or-pattern. `chunk_node_at`'s stated rule -- "a slot is in both exactly when it is offered to `push_native` *and* its declining fallback is `Op::EvalExpr`" -- holds unchanged for all four, so that doc needed no correction.
* `rust/crates/rexx-exec/src/ir/mod.rs` -- the two ops and their docs. `const _: () = assert!(size_of::<Op>() == 16)` is untouched and still holds; see the width check below.
* `rust/crates/rexx-exec/src/ir/compile.rs` -- two arms, each the `SAY` arm with a different tail op, each reading its keyword off the kind with `matches!` (no catch-all `match`, which is the convention `task-3-review.md`'s finding 6 asked for). The two new ops join `assert_region_ops_name_their_clause`'s index-bearing list.
* `rust/crates/rexx-exec/src/ir/drive.rs` -- the two region arms and the two not-driven arms. `Op::Return` ends its region with `RegionEnd::Flowed(flow)`; `Op::Queue` falls through. Both carry an arity `debug_assert` in `Op::Say`'s shape.
* `rust/crates/rexx-exec/src/ir/golden.rs` -- `Return index=N src=R keyword=RETURN|EXIT`, `Queue index=N src=R keyword=PUSH|QUEUE`, and a renderer per tag.
* `rust/crates/rexx-exec/src/ir/golden_tests.rs` -- five new pins (below).
* `rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs` -- `Root::of` classifies both ops as not-a-root; `promoted_as` gains `RETURN`, `EXIT`, `PUSH`, `QUEUE`; the expected-root match folds them into the `SAY` row; the anti-vacuity list gains all four, so a corpus that stopped containing one of them is red rather than silently unchecked.
* `rust/crates/rexx-exec/src/ir/drive/tests.rs` -- `a_body_entered_under_trace_r_echoes_its_promoted_clause_from_the_chunk` counted 1 echo for a callee whose body is `if 1 = 1 then nop` / `return`. The `RETURN` is now promoted too, so the count is 2 and the message names both clauses. **This is the one pre-existing test whose expectation this task moved**; it went red on the first run and its premise, not its subject, was what changed.
* `rust/crates/rexx-exec/tests/ir_dual_cases/return-and-queue` -- new, four rows, every expected byte oracle-captured.
* `rust/corpus/lang/push_queue.rex` and `rust/crates/rexx-parse/tests/sourceline_oracle/push_queue.txt` -- a false comment corrected; see below.

## Test counts

| when | passed | failed | ignored |
|---|---:|---:|---:|
| baseline, before any edit | 1471 | 0 | 4 |
| final | 1476 | 0 | 4 |

`memcap 8G cargo test --workspace --no-fail-fast` from `rust/`, status read unpiped: 0 both times.
The `+5` is the five new golden pins; the four new case rows are stanzas inside `both_engines_agree_on_every_case_file` and add no test.

Gates at the final state, each read unpiped from `rust/`:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0, zero lines matching `^warning|^error`.

**The report-mode harnesses did not move.** Diffing the whole baseline run against the whole final run, ignoring test-name and timing lines, the only difference is doctest wall clock: `4224 of 4259` assertion rows, `4920 of 4999` value rows / `184 of 186` raise rows, `51 of 51` keyword rows, `888 of 896` bif bodies -- identical before and after. That is the promotion moving no bytes anywhere the suite looks.

## The `Op` width

`const _: () = assert!(size_of::<Op>() == 16)` holds, and the assertion is live rather than assumed:

* built unpiped with the two variants added -- `cargo build --workspace --all-targets`, exit 0, and the whole diagnostic list is two lines ("Compiling rexx-exec", "Finished");
* negative control, the same assertion changed to `== 12`: exit 101, `error[E0080]: evaluation panicked: assertion failed: size_of::<Op>() == 12` at `ir/mod.rs:83`. Restored from a `cp` backup and rebuilt.

`u32 + Option<u16> + a one-byte tag` is 12 bytes with padding, well inside the widest variant.

## Oracle captures

Wrapper, from a directory created empty under the session scratchpad, absolute paths, program files kept in a *different* directory so the run directory stays empty (verified empty afterwards):

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/PROG.rex </dev/null )
```

Each program was then run on this crate twice, `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`, and stdout, stderr and exit status compared as three separate files.

| program | oracle rc | tree-walker | ir |
|---|---:|---|---|
| `trace r` / `say 'a'` / `exit 3` | 3 | identical | identical |
| `trace r`, `exit 5` from a called label | 5 | identical | identical |
| `trace r`, `push`/`queue`/`push` read back by three `parse pull`, then a bare `queue` and a fourth `parse pull` | 0 | identical | identical |
| `trace i`, `return length('abcd') + 1` with the caller reading `RESULT` | 0 | identical | identical |
| `trace r`, `return ''` against a bare `return`, caller reading `RESULT` both times | 0 | identical | identical |

Four of those five are the rows of `tests/ir_dual_cases/return-and-queue`.
The fifth (`exit 3` at top level) was **not** written as a row: `tests/trace_oracle/exit_value.rex` already pins an `EXIT`'s own `>>>` under `trace r` against the oracle on the default engine, and the row that is committed reaches the same line at a deeper indent.

Seven further three-way probes (oracle vs `tree-walker` vs `ir`) on shapes no committed row holds:

* `interpret "exit 3"` under `trace r` -- rc 3, all three identical;
* `return 7` from inside a `do forever` inside a called label -- rc 0, identical;
* `exit zi + 40` from inside a `WHEN` branch inside a counted loop, `trace i` -- rc 42, identical;
* `push zst.zk` (a compound tail) read back, `trace i` -- rc 0, identical;
* `queue`/`push`/`parse pull` split across a caller and a callee, the callee returning both lines -- rc 0, identical;
* `signal on syntax` over `return 1/0` in a called label -- rc 9, stdout and status identical, **stderr diverges from the oracle on both engines**, see below;
* the same shape with `say 1/0` in place of `return 1/0` -- same divergence, which is what says it is not this task's.

## The divergence found, and why it is not this task's

`signal on syntax name zh` / `call zsub` where the raise happens **inside** the callee: the oracle prints the handler's clauses at the outer indent, and this crate prints them one level in.

```
oracle          8 *-* zh:                  this crate      8 *-*   zh:
                9 *-* say 'trapped' sigl                   9 *-*   say 'trapped' sigl
                  >>>   "trapped 7"                          >>>     "trapped 7"
```

Every line is present with the same content and the same order; only the indent differs, and stdout and exit status agree.
**Both engines produce identical bytes**, so it is not an engine difference, and the control probe -- the identical program with `say 1/0` in place of `return 1/0`, so that no `RETURN` or `EXIT` is anywhere on the trapped path -- diverges exactly the same way.
It is the callee's indent not being restored when a `SIGNAL ON` transfer unwinds an activation.
I did not file it, did not fix it, and did not write a case row for it: a row would pin bytes that are known to be wrong.

## Mutations

Every row below is a full-workspace `memcap 8G cargo test --workspace --no-fail-fast`, run counts read rather than exit status, the tree restored from a `cp` backup verified with `sha256sum -c` and `rexx-run` rebuilt after each.
**All of M1-M5 and the M4 held-out run were re-run at the final state of the commit**, after the last comment edit landed; the numbers below are the re-runs and they match the first pass exactly.

| # | mutation | tests reddened |
|---|---|---:|
| M1 | `compile` tags an `EXIT`'s op `ReturnKeyword::Return` | 5 |
| M2 | `compile` tags a `QUEUE`'s op `QueueKeyword::Push` | 5 |
| M3 | `Interp::returned_value` emits no `>>>` | 7 |
| M4 | `Interp::queue_evaluated` emits no `>>>` | **1** |
| M5 | `chunk_node_at` loses its `RETURN`/`EXIT`/`PUSH`/`QUEUE` alternatives | 4 |
| M6 | `Op::Return` does not end its region | 36 |
| M7 | `Op::Return` drops the value it was handed | 19 |
| M8 | a bare `RETURN`/`EXIT` is given a register holding the null string | 47 |

Named:

* **M1** -- `both_engines_agree_on_every_case_file`, `ir::golden_tests::a_return_and_an_exit_compile_to_one_op_tagged_with_their_keyword`, `ir::golden_tests::a_traced_exit_carries_its_clause_echo_op`, `run::tests::exit_inside_a_routine_ends_the_routine_where_a_labels_exit_ends_the_program`, `eval::tests::an_exit_inside_a_routine_reached_by_expression_call_ends_the_whole_program`.
* **M2** -- `both_engines_agree_across_every_population`, `both_engines_agree_on_every_case_file`, `ir::golden_tests::a_push_and_a_queue_compile_to_one_op_tagged_with_their_end`, `pull_queue_covers_the_line_reading_sources_in_both_modes`, `the_exempt_set_matches_the_current_failures`.
* **M3** -- `both_engines_agree_on_every_case_file`, `exit_value_covers_the_exit_instructions_own_result_line`, `function_call_covers_the_function_prefix_after_the_callees_own_lines`, `control_variable_novalue_covers_a_dropped_control_variable_under_a_trap`, `run::tests::a_returned_value_traces_in_the_callee_and_again_in_the_caller`, `run::tests::current_value_indent_is_restored_after_a_nested_expression_call`, `run::tests::task_9s_two_new_indents_are_the_oracles_own_and_normalisation_cannot_see_them`.
* **M4** -- `both_engines_agree_on_every_case_file`, and **nothing else in the workspace**.
* **M5** -- `both_engines_agree_on_every_case_file`, `both_engines_agree_across_every_population`, `a_clause_value_survives_the_handler_its_boundary_runs`, `the_exempt_set_matches_the_current_failures`.
* **M6** and **M7** are broad breakage; neither is a discrimination any new row owns, and both are listed only because they are the two ways the driver arm could be wrong without failing to compile.
* **M8** is caught first by the arity `debug_assert` in `Op::Return`'s own driver arm, which panics before any row's bytes are compared.

### Which row catches what, measured per row

`datadriven::walk` stops at the first disagreeing stanza in whichever file `read_dir` returned first, so a red `both_engines_agree_on_every_case_file` names no row on its own (`task-2-report.md`'s correction).
Every attribution below was taken with **the other case files moved out of the directory**, so the failure names a stanza of this file:

| mutation | stanza that failed | what it saw |
|---|---|---|
| M1 | `EXIT` with a value from a called label | stdout between the engines: `""` against `"not reached\n"` |
| M2 | `PUSH`/`QUEUE` read back | stdout between the engines: `a c b` against `a b c` |
| M3 | `EXIT` with a value from a called label | the expected block, `>>>     "5"` missing |
| M4 | `PUSH`/`QUEUE` read back | the expected block, `>>>   ""` and the other three missing |
| M5 | a call inside a `RETURN`'s expression | stdout between the engines: `got 5` against nothing |
| M7 | `RETURN ''` against a bare `RETURN`, measured with that stanza as the **only** one in the directory | stdout: `after empty: RESULT` where the oracle prints `after empty: ` |

### The one unique catch, both directions

M4 -- `Interp::queue_evaluated` emitting no trace line -- is taken by **both** engines, so the engine-vs-engine comparison structurally cannot see it, and what does see it is the oracle-measured expected block.

* With the file present: exactly one red test in the whole workspace.
* With `tests/ir_dual_cases/return-and-queue` moved out of the directory and the same mutation applied: **1476 passed, 0 failed** -- the whole workspace green.

`corpus/lang/push_queue.rex` traces the same lines under `trace r`, but the harness that compares it against the oracle (`corpus.rs`) defaults to report mode and gates only under `REXX_CORPUS_GATE`, so it does not redden a default run.
That is the whole of why this row earns its place.

Nothing else in the new file is a unique catch, and each row's own comment says what else reddens under its mutation.

## Rows the brief asked for that were already in the tree, and were not written again

Checked with `/bin/grep -a` over `tests/`, `corpus/` and `src/`, and each read to confirm it covers what is claimed:

* **a bare `RETURN` under `trace r`** -- `ir_dual_cases/trace-settings`, "one body entered untraced and then under trace r": its callee ends `7 *-*   return` with no value line after it.
* **a bare `RETURN` under `trace i`, and a `RETURN` whose expression is an operator, with the caller's `RESULT`** -- `ir_dual_cases/calls`, "a CALL inside a body entered while the setting was already in force": `10 *-*     return zn + 1` with `>V>`/`>L>`/`>O>`/`>>>`, then `>V>     RESULT => "8"`, then `7 *-*   return` bare.
* **`RETURN` with a value whose caller reads `RESULT`, under `trace r`** -- `ir_dual_cases/calls`, "the same call under trace r": the two `>>>` for one value at two indents.
* **`EXIT` ending the program from a called label and only the routine from a `::ROUTINE`** -- `ir_dual_cases/calls` has it untraced, at rc 3.
* **an `EXIT` with a value under `trace r`** -- `tests/trace_oracle/exit_value.rex`, oracle-pinned, on the default engine.
* **`PUSH`/`QUEUE` traced under `trace r`, bare forms included** -- `corpus/lang/push_queue.rex`, in `phase-4b.txt` and so in this harness's own corpus population.
* **`PUSH`/`QUEUE` read back by `PULL`/`PARSE PULL`** -- `corpus/lang/pull_queue.rex`.

The one shape neither corpus program has is a *traced* `PUSH`/`QUEUE` **and** its read-back in one program, which is what the committed row is, and it is the shape M4's uniqueness rests on.
That list is in the case file's own header, so the next reader of the file sees it.

## The false comment corrected

`corpus/lang/push_queue.rex`'s header said: "an EXIT with a value has a pre-existing, unrelated gap in this crate's own EXIT arm (its >>> line is not traced)".
That is false, measured: `trace r` / `say 'a'` / `exit 3` prints `>>>   "3"` on both engines and matches the oracle byte for byte.
It also misattributes -- `condition_traps.rex`'s header, which it cites, says the gap closed at 4b Task 9, and `tests/trace_oracle/exit_value.rex` exists to pin it.
Replaced with five lines saying what is true now, the same number of lines, because a corpus program's line numbers are in its own traced output.

`crates/rexx-parse/tests/sourceline_oracle/push_queue.txt` is that program's own source lines, so it had to be regenerated, and the Phase 3 gate caught the omission on the next run rather than my noticing it.
Regenerated with the driver the harness's own module doc specifies, run from a directory created empty, on a **copy** of the program in the scratchpad rather than on the repository file, since the driver constructs `.Package` and the standing rule forbids that on a file in the tree.
The regenerated file differs from the committed one in exactly the five edited lines and the count is unchanged at 39, which is what says the regeneration is my edit and nothing else.

## What I am unsure about

1. **`Op::Queue` earns less than `Op::Return` does.** `PUSH`/`QUEUE` appear in no benchmark and in three corpus programs; the plan's own measured table does not list them. The op is here because the brief asked for all four, and its justification is uniformity with `Op::Say` rather than a measurement. Nothing about it is wrong; it is simply not a promotion the profile called for.
2. **The `EXIT`-with-a-value row and `exit_value.rex` overlap.** I kept the committed row because it also carries the keyword tag's discrimination (M1) and reaches the `>>>` at a callee's indent, and because `exit_value.rex` is normalised where a case row is not. A reviewer who thinks that is one row too many would be reading the same facts I did.
3. **M8's real catcher is a `debug_assert`, so the `RETURN ''` row's strongest claim is about a mutation only a debug build sees.** In a release build that assertion is gone and the row's expected bytes would be what catches it. I did not measure a release-profile mutation run.
4. **The `SIGNAL ON` indent divergence is unfiled.** I established it is pre-existing and engine-independent and stopped there. I did not search the plans or the SF tracker for an existing record of it, so it may be known.
5. **`corpus_shape_tests`' anti-vacuity list now names four more constructs**, which means a corpus that stopped containing, say, a `QUEUE` would fail that sweep rather than quietly checking nothing. That is the intended direction, but it does couple the sweep to three corpus programs (`push_queue.rex`, `pull_queue.rex`, `state_builtins.rex`) for the `PUSH`/`QUEUE` rows specifically.

---

# Fix round, against `task-4-review.md` (`a2952fd62`, on top of `9956d6abd`)

Three findings addressed; the four deferred ones are untouched by request.
Nothing was rebased; the work sits on `9956d6abd`, the tip after the coordinator filed the `SIGNAL ON SYNTAX` divergence as its own task.

## 1 (Major). `Interp::returned_value`'s rooting paragraph -- rewritten, not deleted

The sentence "that is `EXIT`'s window rather than a general one" claimed a discrimination between the two keywords the function serves, and the code refutes it: `Interp::apply_flow` takes `root_exit_value` on the `Flow::Return` arm and the `Flow::Exit` arm alike.
Rewritten rather than deleted, because the load-bearing half is still true and still worth saying -- that the `push_temp` here is a one-clause root and is **not** what keeps the value alive as far as `exit_code_for`.
What went is the discrimination; what replaces it is the property both keywords satisfy, with the citation moved from `Flow::Exit`'s narrative to `apply_flow`'s two arms.

Measured rather than taken from the review: `return 5` as a whole program is **rc 5** on the oracle, from a directory created empty.
That is the fact that makes the window `RETURN`'s as much as `EXIT`'s, and it is now in the sentence.

`root_exit_value`'s own doc in `lib.rs` is phrased in `EXIT` terms too ("`EXIT`'s result is pushed as an ordinary one-clause temp").
Left alone: it is incomplete rather than false, it claims no discrimination, and it is not in this task's diff.

## 2 (Minor). The `src: Option` justification -- **narrowed, not given a row**

Adding a row is not possible, and that is measured rather than assumed.
Oracle, from a directory created empty, `trace r` over `queue` / `parse pull za` / `say '[' || za || ']'` against the same program with `queue ''`:

```
2 *-* queue                    2 *-* queue ''
  >>>   ""                       >>>   ""
```

Every following line is identical and both are rc 0 with `[]` on stdout.
The only difference in the whole transcript is the clause echo of the program's own source text.
So no row can distinguish `Op::Queue { src: None }` from one handed a null-string register, and the header now says that in its own voice: the `Option` is `RETURN`'s discrimination on one side and `Op::Say`'s existing shape on the other.
The property that *does* hold for both pairs -- a bare `RETURN`/`EXIT` traces no value line where a bare `PUSH`/`QUEUE` traces the null string it queues -- is stated separately, and rows below show it.

`Op::Return`'s doc in `ir/mod.rs` was checked for the same overreach and does not have it: its measurement is attributed to `return ''` explicitly, and its trace sentence covers `EXIT` -- verified on the oracle this round, `exit ''` under `trace r` traces `>>>   ""` and a bare `exit` traces nothing, both rc 0.

## 3 (Minor). The cardinalities -- gone, and two more with them

* "three `run::tests` indent witnesses" -> the two catchers named: `tests/trace_oracle/exit_value.rex` and `run::tests::a_returned_value_traces_in_the_callee_and_again_in_the_caller`. Fewer things named, no quantifier.
* "a dozen `run::tests` besides" -> deleted. It was measured nowhere and the sentence is complete without it: the extracted-program sweep and the branch-shape table are named and that is the claim.
* Swept the rest of the file for the same class and found two more: "All four used to be one `Op::Generic`" -> "Each used to be", and "the two op-stream pins for this op" -> "the op-stream pins for this op".

## Mutations, re-run at the final state of this commit

Every claim the case file makes was re-measured after the last edit landed, whole workspace, `--no-fail-fast`, run counts read rather than exit status, tree restored from a `cp` backup verified with `sha256sum -c` and `rexx-run` rebuilt after each.
**All eight reproduce the numbers and the test names in the table above, exactly.**

| # | reddened | matches the first pass |
|---|---:|---|
| M1 | 5 | yes, test for test |
| M2 | 5 | yes |
| M3 | 7 | yes -- and it names both tests row 1's comment now cites |
| M4 | **1** | yes |
| M5 | 4 | yes |
| M7 | 19 | yes |
| M8 | 47 | yes |

Attribution runs, also re-run at the final state:

* **M4 with `tests/ir_dual_cases/return-and-queue` held out of the directory: 1476 passed, 0 failed, 4 ignored** -- the whole workspace green. With the file present, exactly one red test. The unique catch holds in both directions.
* **M7 with the `RETURN ''` row as the only stanza in the directory**: red, `after empty: RESULT` where the oracle prints `after empty: `.
* **M8 with the same single stanza**: the arity `debug_assert` at `ir/drive.rs:1051` fires first, which is what that row's comment says.
* **Unmutated control with that single stanza**: green, so the row is not trivially red.

## Gates

`cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0, zero `^warning`/`^error` lines; `memcap 8G cargo test --workspace --no-fail-fast` exit 0, **1476 passed, 0 failed, 4 ignored** -- the same as the review's baseline, since this round changes only prose.

One observation, not acted on: `RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc -p rexx-exec --document-private-items` fails on **pre-existing** unresolved links (`Activation::clock_stale`, `Activation::cached_clock`, `crate::Resolved::Builtin`, `Delivery::positionless`, `time_h_s_m_diverge_from_the_oracle_for_a_date_shaped_output`, and a `compile` function/module ambiguity), none of them in this diff.
Run only to confirm the one link this round adds, `[`Interp::apply_flow`]`, resolves -- it does not appear in the diagnostics.
