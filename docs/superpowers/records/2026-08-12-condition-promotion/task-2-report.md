# Task 2 report: a `DO`/`LOOP` header's values compile

Status: DONE_WITH_CONCERNS.
One commit on `plan/rust-rewrite`: `b8e9db0e5` -- "Compile a DO/LOOP header's values instead of evaluating each whole", read back with `git log` after committing.

**One blemish in that message, recorded because it cannot be edited:** the identifier `a_header_operands_register_goes_back_to_the_body` is wrapped across two lines mid-word.

## What changed

### `ir/compile.rs`: the header emission (Step 1)

The per-slot `Op::EvalExpr` becomes a decision:

```rust
match loop_header_slot(body_node, slot) {
    Some(expr) if native_shape(expr, Some(NodePath::ROOT)) => push_native(..., slot, Some(NodePath::ROOT), dst)?,
    _ => ops.push(Op::EvalExpr { index, slot, dst }),
}
```

`Op::TraceKeyword` and `Op::LoopHeaderValue` follow exactly as they did, so no op was added and none changed.
Each slot decides for itself: a header with one declining expression keeps native ops for its others.

**A slot `loop_header_slot` has no expression for takes the same arm as one `native_shape` declines**, because it is the same answer -- that slot stays a whole `Op::EvalExpr`, which is what every slot was before.
No shape reaches that today (`loop_header_plan` pushes a role only where `header_expr_for` has an expression for it), and it is not asserted; the arm is there because the type is an `Option` and the driver's own `eval_chunk_expr` is loud for it.

**`native_shape` + `push_native` directly, not `push_value`**, as the dispatch resolved.
I did not copy Task 1's reason for that choice, because it is false here: Task 1 avoided `push_value` because an `IF`'s fallback `Op::EvalExpr` does the whole job including validation, so `Op::Condition` behind it would double up.
A header slot's fallback owes exactly what a native slot owes -- `Op::LoopHeaderValue` -- so `push_value` would emit an identical stream.
What the explicit form buys is one `Op::EvalExpr` push site instead of two, because the declining slot and the expressionless slot fall into one arm; that is the reason the code carries.

**The `header_plan` rename.** `loop_header_plan`'s result shadowed the `plan: &Plan` this arm now reads for a promoted slot's variable slots. Renamed rather than re-bound.

**The register hazard, which the brief does not name.**
The header's destination registers are allocated in the enclosing scope and released past the `END`, and `push_native` allocates its operand temporaries from the same allocator -- so an intermediate that outlived its slot would be handed to a body clause that `Op::LoopRun` steps while the loop is still running.
It does not happen: `push_native`'s binary arm is the only one that allocates, and it releases to its own mark as soon as the operation has run.
That is now pinned twice -- a `debug_assert_eq!` in the arm that the register top after the header loop is exactly the header's own registers, and `golden_tests`'s `a_header_operands_register_goes_back_to_the_body`, which shows the body's assignment taking the operand's register back.
Measured: MU5 below.

### `ir/compile.rs`: `assert_keyword_echoes_precede_their_value` (beyond the brief)

The dispatch asked me to verify that `Op::TraceKeyword` has no adjacency assertion and to consider whether that is now a gap.
**Verified**: it is the one echo op with none, and it is in the `None` group of `assert_region_ops_name_their_clause` because it carries no index. Inserting native ops in front of it trips nothing.

The sibling assertions all look one place *back*, at the op that computed the echoed register. That shape is no longer available here: what sits in front of a `>K>` is now the last of the slot's own ops, which varies.
The available invariant is the pair on the other side, and it is what the assertion states: **every `Op::TraceKeyword` is immediately followed by the `Op::LoopHeaderValue` that files the value it echoes, on the same register and under the same role.**

**It is not a coverage gap, and that is measured** (MU6): emitting the echo in front of the slot's ops reddens thirteen tests without the assertion.
What the assertion adds is the shape of the failure -- a compile-time refusal naming the op rather than thirteen output divergences -- which is the same thing `assert_region_ops_name_their_clause`'s own doc claims for itself. The doc says so in those words.

### `run.rs` (Step 2)

* `chunk_node_at` gains `(InstructionKind::Do(body) | InstructionKind::Loop(body), slot) => loop_header_slot(body, u32::from(slot))?`. The arm binds `slot` rather than matching a number, because every slot of a header is addressable.
* `loop_header_slot` becomes `pub(crate)` and its doc says it is the one resolution the compiler and both driver entries read.
* `chunk_node_at`'s doc named "a `DO` header's value" as a slot that holds no node any op names. Corrected. The subset claim it makes against `eval_chunk_expr` still holds: `chunk_node_at`'s four slot arms are a strict subset of `eval_chunk_expr`'s five.
* `run_loop`'s doc enumerated the header's ops as `EvalExpr`, `TraceKeyword`, `LoopHeaderValue`. Replaced by the mechanism: one group per `HeaderPlan` entry, each ending in the op that validates and files that value.

### `ir/mod.rs`

`Op::TraceKeyword`'s doc said it is "a separate op from the [`Op::EvalExpr`] that produced the value". Now "from whatever produced the value".

### `ir/corpus_shape_tests.rs` (not in the brief's file list, required by it)

`check_body`'s comment said the constructs other than an assignment, a `SAY` and an `IF` "evaluate through `Op::EvalExpr` unconditionally", which this change falsifies.
Extended rather than narrowed, which is the direction Task 1's review endorsed: a `DO`/`LOOP`'s expectation is per slot, each `Op::LoopHeaderValue` ending one group, and the last `Root` in a group is what that slot's expression must end in.
`Seen::native_header_values` is asserted non-zero, for the reason `native_conditions` is: without it the header rows are satisfied by a corpus whose every header expression declines.

**One dependency this file otherwise avoids**: it asks `loop_header_slot` which expression a slot holds, and that is the resolution `compile` emits from, so a slot resolving to the wrong expression would agree with itself here. What stays independent is `root_of`, which is the axis the file is about. The comment says this.

### `ir/golden_tests.rs` (Step 3)

Five pinned streams moved, because `1`, `3`, `2` and `4.5` are constant symbols and now compile to `Op::LoadConstant` + `Op::TraceLiteral`:
`a_counted_loop_compiles_its_header_to_a_clause_region_and_its_body_to_generic`,
`a_traced_counted_loop_echoes_its_do_clause_from_the_stream`,
`a_block_has_an_empty_header_region_and_a_do_over_echoes_only_its_target`,
`a_nested_loops_registers_sit_above_the_enclosing_loops_and_a_later_loops_reuse_them`,
`two_constructs_ending_at_one_instruction_release_to_the_lower_mark`.
Their doc comments moved with them; the first's "six ops of the seven" and the second's "the same three or two ops" were both falsified and both are gone rather than recounted.

Three new tests:

* `a_header_bound_that_is_a_symbol_and_one_that_is_a_call_take_their_own_ops` -- the brief's Step 3, both halves.
* `a_header_slot_outside_the_native_set_leaves_the_other_slots_native` -- `do i = .nil to 3`, slot 0 an `Op::EvalExpr` beside slot 1's load. This is the per-slot decision.
* `a_header_operands_register_goes_back_to_the_body` -- the register hazard.

### `tests/ir_dual_cases/loop-header-values` (Step 4)

Three stanzas. See "Oracle captures" and "Concerns".

## Test counts

| point | passed | failed | ignored | cargo exit |
|---|---:|---:|---:|---:|
| before any edit | 1465 | 0 | 4 | 0 |
| final | 1468 | 0 | 4 | 0 |

The three new runs are the three new golden tests.
The three new `ir_dual` stanzas run inside `both_engines_agree_on_every_case_file`, which is one test either way.

Gates, from `rust/`, each status read unpiped and never through a pipeline:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0.
* The same clippy from a **clean** `CARGO_TARGET_DIR` under the scratchpad, per `rust/CLAUDE.md`'s rule that a same-session green is provisional -- exit 0, with `rexx-num`, `rexx-parse`, `rexx-core`, `rexx-exec`, `rexx-extract`, `rexx-oracle` and `rexx-bench` all in its `Checking` list and no `warning:` or `error` line anywhere in its output.
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0.
* `const _: () = assert!(size_of::<Op>() == 16)` is untouched and still holds: no variant was added or widened, and `cargo build --workspace --all-targets` was run **unpiped** with its whole diagnostic list read -- no `E0080`, and the build finished.

## Oracle captures

Three programs, written into a fresh directory I `mkdir`ed under the session scratchpad, absolute paths for every redirect, stdout/stderr/exit status as three separate descriptors. None of the three crashing programs was run.

```
( cd $SP && ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx $SP/vN.rex \
  </dev/null >$SP/vN.oracle.out 2>$SP/vN.oracle.err )
```

then, from `rust/`, each program on each engine:

```
REXX_ENGINE=tree-walker ./target/debug/rexx-run $SP/vN.rex </dev/null >... 2>...
REXX_ENGINE=ir          ./target/debug/rexx-run $SP/vN.rex </dev/null >... 2>...
```

and four diffs per program. **Taken twice**: once when the rows were written, and once at the end against the final `rexx-run`, because the binary had been rebuilt through six mutations in between.

| # | program | rc | result |
|---|---|---:|---|
| v1 | `trace i` over `do zi = 1 to zn by 2 for 2` | 0 | all four diffs empty |
| v2 | `trace i` over `do zi = 1 to length(zs)` | 0 | all four diffs empty |
| v3 | `trace i` over `do qq over zs for 1` | 0 | tw-vs-ir empty on both channels; oracle-vs-tw stderr differs by exactly one line |

**v3 is a pre-existing divergence this task's captures found, and it is the finding of the task.**
The oracle prints `       >K>   "FOR" => "1"` for a `DO OVER ... FOR`'s count; this crate prints nothing.
Measured on both engines, under `trace i` and under `trace r`, on `do qq over zs for 1` and on `do qq over 4.5 for 2` -- the exact program `golden_tests` compiles.
`HeaderRole::OverFor::keyword()` answers `None`, and both engines read that one table, so it is not the compiled form's.

Two doc comments asserted the opposite as a measurement and are corrected:

* `run.rs`, `HeaderRole::OverFor`: "which the oracle echoes nothing for -- unlike every other count here."
* `golden_tests.rs`, `a_block_has_an_empty_header_region_and_a_do_over_echoes_only_its_target`: "measured, the oracle traces `>K>  "OVER"` for the target and nothing at all for the `FOR` count that follows it, unlike a controlled loop's `FOR`."

**I did not change the behaviour.** It is one line (`HeaderRole::OverFor => Some("FOR")`), it is not this task's, and it would move the `>K>` output of every `DO OVER ... FOR` on both engines -- which deserves its own oracle-gated task rather than a rider on this one. The v3 row pins our answer so the gap cannot move unnoticed while it is open.

## Mutations

Every run was the **whole workspace** under `memcap 8G cargo test --workspace --no-fail-fast`, so "nothing else caught it" is measured rather than inferred from a run that stopped at the first catcher.
`compile.rs` and `run.rs` were backed up with `cp`, restored from the backup, the restore verified with `sha256sum -c`, and `rexx-run` rebuilt after each restore.
No `git checkout --` was used.

| id | mutation | reddened |
|---|---|---|
| MU1 | the header arm never takes the native path (`Some(expr) if false && native_shape(...)`) | 9: `corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops` and the eight pinned `DO` streams. **No `ir_dual` test at all**, which is the promotion's own contract: promoted or not, the program prints the same bytes. |
| MU2 | `chunk_node_at` loses its `Do`/`Loop` arm | 3: `both_engines_agree_across_every_population`, `both_engines_agree_on_every_case_file`, `the_exempt_set_matches_the_current_failures` |
| MU3 | every slot resolves to slot 0 (`loop_header_slot(body_node, 0)`) | `both_engines_agree_on_every_case_file` and `both_engines_agree_on_every_branch_shape`; **`both_engines_agree_across_every_population` did not terminate** in eleven minutes and the run was killed. A wrong `TO` bound can make a corpus loop never end, which is what `rust/CLAUDE.md` warns a mutation run is for. |
| MU4 | the header's call op addressed at slot `0` instead of `slot` | 4: `both_engines_agree_across_every_population`, `both_engines_agree_on_every_case_file`, `golden_tests::a_header_bound_that_is_a_symbol_and_one_that_is_a_call_take_their_own_ops`, `the_exempt_set_matches_the_current_failures` |
| MU5 | `push_native`'s binary arm never releases the operand register | 8, including `golden_tests::a_chain_of_operators_reuses_the_destination_register`, `golden_tests::a_header_operands_register_goes_back_to_the_body`, `corpus_differential`, `keyword_assertions_differential` and the corpus shape sweep. **The new `debug_assert_eq!` fired seven times and is what reddened the shape sweep.** |
| MU6 | `Op::TraceKeyword` emitted in front of the slot's ops, **without** the new assertion | 13: the population, loop-shape and case-file harnesses, `trace_oracle`'s three control-variable transcripts, and six pinned streams. The echo reads a register nothing has written and prints `>K>   "TO" => "The NIL object"`. |
| MU6b | the identical mutation **with** the new assertion in the tree | 44, and `assert_keyword_echoes_precede_their_value`'s message appears 44 times: the failure is now a compile-time refusal naming the op. |

**WITHDRAWN in fix round 1.** This paragraph concluded from which file the case-file harness *reported* that no row of `loop-header-values` catches anything. The inference is invalid and the conclusion is false -- see "Fix round 1".

What survives, and the corrected runs confirm it: **none of the six mutations is caught *uniquely* by the new file.** Each reddens pre-existing tests as well.

## What went into the plan

`docs/superpowers/plans/2026-08-12-condition-promotion.md` Steps 3 and 4 were rewritten in the same commit, because the next fix round regenerates a brief from the plan:

* Step 3 now names the two golden tests the dispatched list did not ask for and the corpus-shape extension, with the comment that made it necessary.
* Step 4 now names the three rows that landed, says the `DO FOREVER` row was captured, matched the oracle on both engines, and was then **deleted** -- a `FOREVER` header holds no expression, so nothing this task changed can reach it -- and says a plain counted loop and the `to`/`by`/`for` combination were written as one row.
* Step 4 records the `DO OVER ... FOR` divergence and that fixing it is not this task's.
* Step 4 records that no mutation is uniquely caught by the file.

## Concerns

1. **WITHDRAWN in fix round 1.** This concern said the new case file has no measured catching power and offered the `to`/`by`/`for` row for deletion. Both the conclusion and the mechanism behind it are false; the corrected measurement is in "Fix round 1" below. The rows are kept, a fourth was added, and each row's own comment now names what it caught.
2. **The `DO OVER ... FOR` divergence is unfixed and now has a committed expectation that encodes it.** That is the shape `loop-header-boundaries` already uses for its two divergences, but it does mean a future fix has to edit this row as well as the two doc comments. It is one line of output and the row's own comment says which line.
3. **CORRECTED in fix round 1: MU3 was completable and I stopped one flag short.** The population sweep does not terminate under it, but that is one test and a run that skips it is bounded. Re-run at the final state: **nine lib tests** redden -- the corpus shape sweep, four `ir::drive` tests, both new header golden tests and the declining-slot one -- plus `both_engines_agree_on_every_case_file`, `..._loop_shape` and `..._branch_shape`. The expression half is well covered. "Could not be completed" was an overstatement.
4. **`corpus_shape_tests.rs` now reads `loop_header_slot`**, which is a step away from that file's stated independence. `root_of` -- the axis it exists for -- stays independent, and the comment says which half is which, but a reviewer who values that independence absolutely would rather I had restated the header enumeration in the test file. Restating `LoopKind::Controlled`'s `order` walk there is a second copy of a table that changes with the parser, which is why I did not.
5. **The `_ =>` arm covering "the header has no expression for this slot" is unreachable today and untested.** `loop_header_plan` pushes a role only where `header_expr_for` answers `Some`. I did not write a test for it because I could not construct a program that reaches it, and `rust/CLAUDE.md`'s rule about unreachability claims is why I am flagging it rather than asserting it cannot happen.

---

# Fix round 1

Against `.superpowers/sdd/2026-08-12-condition-promotion/task-2-review.md`, on top of `fdd992cdb`.
One commit: `355017720` -- "Withdraw the case-file measurement that read a directory's inode order", read back with `git log` after committing.
Spec passed. Nothing below is behaviour: the only non-comment change is one new case-file stanza.
Every mutation quoted here was re-run **at the final state of this commit**, after the new stanza landed, because a later hunk can falsify a measurement taken earlier.

## F1 -- the case file's measurement, which was false in every clause

**What the first round claimed:** that `loop-header-boundaries` sorts first, that `datadriven` stops at the first mismatch, and therefore that no row of `loop-header-values` is ever reached, catches nothing, and could be trimmed.

**What is actually there**, read out of `datadriven-0.9.0/src/lib.rs`:

* `test_files` builds its list from `fs::read_dir` and never sorts. `walk_exclusive` runs `test_files(dir)` in that order.
* `walk_exclusive` visits **every** file, accumulating one failure per file and panicking at the end with all of them. Only `TestFile::run_normal` breaks, and only within one file.
* Neither is what stops this harness. `render_both_engines` asserts the two engines against each other **inside** the callback (`ir_dual.rs:946`), so the first disagreeing stanza panics out of `walk` before datadriven compares anything to an expected block.

Filesystem order, measured in both places: `os.listdir` on the repository tree returns `loop-header-boundaries` before `loop-header-values`; the reviewer's worktree of the same commit returned them the other way round. **Which file reports a shared failure is inode order and is not a property of the tree at all** -- which is why the first round's observation generalised to nothing.

**Re-measured, with every other case file moved out of the directory so the harness can see only this file** (`both_engines_agree_on_every_case_file`, run alone):

| mutation | file alone | which row, and how it shows |
|---|---|---|
| MU2 -- `chunk_node_at` loses its `Do`/`Loop` arm | **FAILED** | the call-bound row: tree-walker prints `1\n2\n`, ir prints nothing |
| MU4 -- the header's call op addressed at slot `0` | **FAILED** | the same row, the same way |
| MU6 -- the `>K>` echo emitted in front of the slot's ops | **FAILED** | the `to`/`by`/`for` row, on stderr: every `>K>` prints `"The NIL object"` |
| MU1 -- the header arm never takes the native path | ok | correct, and it is the promotion's contract: the bytes do not move |

And with the **new `trace r` stanza as the only file in the directory**, MU6 **FAILED** on it too -- so that row catches the echo-position mutation on its own, with no intermediate line in its block to move.

So the file catches this task's own Step 2 being removed, the address half of Step 1, and the echo order the brief names as most at risk. **None of those is a unique catch** -- each mutation reddens pre-existing tests as well -- and that is the honest statement the first round should have made.

Fixed: the false paragraph is withdrawn in this report, plan Step 4 is rewritten with the mechanism as it actually is, the case file's header states how the measurement has to be taken, and each row's comment names what it caught.

## F2 -- the `debug_assert_eq!` named a consequence its condition cannot produce

The message said a leaked register means "the loop's body would be handed it while the loop is still running". A register left allocated **above** `header_top` is precisely one the body is not handed.

Measured, at the final state: leaking one register per header slot with the assertion removed leaves the whole `ir_dual` suite green, population sweep included, and moves six golden streams, every one of them a pinned `DO`. Waste, not corruption.

The equality is kept -- it is an equality, so it guards the harmful direction too -- and the comment and the message now say what each side means: above `header_top` is a register nothing reuses until the `END`; below it is a header value handed to the body while `LoopState` still reads it.

## F3 -- the sentence between the two that were corrected

`HeaderRole::keyword()`'s own doc still read "or `None` for the roles the oracle echoes nothing for", two lines under the `OverFor` variant this task corrected -- the exact function the report names as the divergence's cause. Now: `None` is the tag this table withholds, and a `None` here is not one answer -- `Initial` is measured to match the oracle and `OverFor` is measured not to.

## F4 -- what the new golden test is actually for

**CORRECTED in fix round 2 -- the leak bullet below was measured with the `debug_assert_eq!` removed and written as a claim about the tree as it stands.** The corrected pair, re-measured over the whole workspace at round 2's final state:

* **the harmful direction** -- the header's own registers released at the region's end instead of past the `END`. The body's assignment lands at register 0, a value `LoopState` still reads. `ir_dual` stays green, population sweep included; two tests move, this one and `a_nested_loops_registers_sit_above_the_enclosing_loops_and_a_later_loops_reuse_them`, and nothing else in the workspace.
* **the leak direction** -- `push_native` never releasing the operand's register. **As the tree stands, this is caught by the `debug_assert_eq!` first**: eight tests move, `both_engines_agree_across_every_population` among them, failing on the assertion's own message, and this golden test panics on the assertion before `render` is reached. With the assertion removed as well, the assignment lands at register 3 and exactly two tests move: this one and `a_chain_of_operators_reuses_the_destination_register`.

So the new test catches both directions, and no differential harness in the tree sees either. Its doc now says that, with both register numbers, and says why the differential cannot reach the harmful one: a header value's `ObjRef` is rooted by its register only on the compiled engine, and nothing in the corpus collects while a loop is running. That contradicts the first round's own verdict on its new tests, and the contradiction is the point.

**The first draft of this fix said the leak "moves the assignment up to register 3" and cited a mutation that allocated a spare per slot, which moves it to 4.** The number was written from the concept and the measurement was of something else; it was caught by reading the mutation's own output rather than the sentence, and the single-operand-leak mutation was then run to get the number the sentence is about.

## F5 -- a gate total in a comment

"reddens thirteen tests" is gone from `assert_keyword_echoes_precede_their_value`'s doc, replaced by what moves: the population, loop-shape and case-file sweeps, `trace_oracle`'s control-variable transcripts and every pinned `DO` stream. Re-run at the final state, the count is still thirteen and the categories are unchanged -- which is exactly why the number was not worth writing down.

**One copy is in `b8e9db0e5`'s commit message and cannot be edited.** So is that commit's "1468 passed", which is a gate total of the same kind, and its wrapped `a_header_operands_register_goes_back_to_the_body`.

## F6 -- the self-contradicting mechanism

"a header temporary that outlived its slot would push that assignment to register 3 and be overwritten by it" ran the two opposite failures together: pushed to 3, the temporary at 2 is exactly what is not overwritten. Rewritten to name them as opposites -- up to 3 is waste, down to 0 is the overwrite -- with F4's measurement under it.

## F7 -- the `trace r` requirement

Restored rather than argued away. `trace r` over `do zi = 1 to zn by 2 for 2`, oracle-captured from a fresh `mkdir`ed directory under the standard wrapper, run on both engines: **all four diffs empty** (tw vs ir on stdout and stderr, oracle vs tw on stdout and stderr). It prints the three `>K>` lines adjacent and in keyword order with no intermediate line at all, which is the instrument `trace i` cannot be: it separates the echo order from the values' own ops. Measured to catch MU6 on its own.

## F9, F10

"Three instructions -- `DO`, `nop`, `END`" is now "The body is `DO`, `nop`, `END`". `Op::TraceKeyword`'s doc is reflowed.

## F8

Left to the coordinator, who is amending the plan. `phase-4-exclusions.txt` untouched.

## Gates, at the final state

From `rust/`, each status read unpiped:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
* the same clippy from a **clean** `CARGO_TARGET_DIR` -- exit 0, all seven crates in the `Checking` list, and zero lines matching `^warning` or `^error` in its whole output
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, **1468 passed, 0 failed, 4 ignored**

Unchanged from `b8e9db0e5`: the new stanza runs inside `both_engines_agree_on_every_case_file`, which is one test either way.

## Not fixed

Nothing from the review. F8 is the coordinator's by his own ruling.

---

# Fix round 2

Against `.superpowers/sdd/2026-08-12-condition-promotion/task-2-re-review.md`, on top of `d3372e429`.
One commit: `cfbbdedd7` -- "Say which configuration each register measurement was taken under", read back with `git log` after committing.
Six items, all prose, plus one restoration. No behaviour changed: every edit in this round is a comment or a document.

Every mutation below was re-run **at this commit's final state**, over the whole workspace under `memcap 8G cargo test --workspace --no-fail-fast`, with `compile.rs` and `run.rs` backed up by `cp` into a **private** scratchpad directory (`r3/backup`, not the shared `backup/` the re-review's process note warns about), restored from that copy, each restore verified with `sha256sum -c`, and `rexx-run` rebuilt after each.

## The measurements this round rests on

| id | mutation | result |
|---|---|---|
| M-A | `push_native`'s binary arm never releases the operand register, **assertion left in place** | 1463 passed / **8 failed**, `both_engines_agree_across_every_population` among them, failing on the new assertion's own message; the assertion fired seven times |
| M-B | the same, **assertion removed as well** | 1469 / **2**: `a_chain_of_operators_reuses_the_destination_register` and `a_header_operands_register_goes_back_to_the_body`; the stream shows `14: LoadConstant dst=3` |
| M-C | the header's registers released at the region's end instead of past the `END` | 1469 / **2**: this test and `a_nested_loops_registers_sit_above_the_enclosing_loops_and_a_later_loops_reuse_them`; the stream shows `14: LoadConstant dst=0`; `ir_dual` green |
| M-D | the `>K>` echo emitted in front of the slot's ops, the check removed | 1458 / **13**; `two_constructs_ending_at_one_instruction_release_to_the_lower_mark` is in the **ok** list |
| M-E | one leaked register per header slot, assertion removed | 1465 / **6**, every one a golden stream; `ir_dual` green |

Per-stanza, each row extracted into a file of its own with the rest of the directory held out:

| row | echo in front | every slot resolves to 0 | `chunk_node_at` arm gone | call op at slot 0 |
|---|---|---|---|---|
| `trace i` to/by/for | FAILED | FAILED | ok | -- |
| `trace r` to/by/for | FAILED | FAILED | ok | -- |
| `trace i` call bound | FAILED | FAILED | **FAILED** | **FAILED** |
| `trace i` `DO OVER ... FOR` | FAILED | FAILED | ok | -- |

and the whole file under "the header never promotes": **ok**, which is the promotion's contract.

## N1 -- the leak bullet's unstated condition

`a_header_operands_register_goes_back_to_the_body`'s doc said that in **both** directions the `ir_dual` suite stays green and nothing else in the crate moves. M-A says otherwise: as the tree stands, the leak mutation is caught by the `debug_assert_eq!` before this test can render anything, and it reaches the population sweep.

The doc now leads with the harmful direction, where the claim is true (M-C), and states the leak direction's condition in bold: the register-3 reading needs `compile`'s assertion removed as well, and with the assertion in place the mutation is *louder*, not quieter. The report's own F4 section carried the same false claim and is corrected in place.

This was the round-1 defect repeating one level down: a measurement of one configuration written as a claim about another. Round 1 caught it in the *number* (register 3 versus 4) and not in the *condition*.

## N2 -- a false universal where a rotting count had been

"every pinned `DO` stream" is gone from `assert_keyword_echoes_precede_their_value`'s doc. M-D is why: `two_constructs_ending_at_one_instruction_release_to_the_lower_mark` pins a `DO` stream and stays green, because it asserts a substring and the mutation reorders ops inside a slot group without moving any op index.

The replacement names no set at all -- the sweeps and `trace_oracle`'s transcripts, which are the categories that make the point, and then the mechanism. Deleted rather than requantified, per the round's rule.

**The lesson is worth its own line, because it is the converse of the rule that produced it**: removing a cardinality is not free. A stale count reads as suspicious; a false universal reads as authoritative. Where the property that picks out the moved set is not obvious, the honest move is to name fewer things, not to quantify over more.

## N3 -- "measured" for a row that had no measurement

Plan Step 4 claimed each row was measured to catch something; the `DO OVER ... FOR` row had no run behind it, because a whole-file run stops at the first disagreeing stanza. Now measured, per row, with each stanza extracted into a file of its own -- the table above. The claim came out stronger than it was written: **all four rows catch two mutations each on their own**, and the call row catches two more that the other three are measured green under.

The case file's header and every row comment are rewritten against that table. The header no longer promises that each comment "names the one it caught" -- three rows share the same two catches, and saying so is shorter than pretending they differ.

## N4 -- a count that rotted inside its own round

"under three mutations" is gone from the case file header; MU3 makes four, and the next mutation would make five. The header names the mutations instead.

## N5 -- the assertion comment's own loose clause

"moves the pinned streams that hold a `DO`, and nothing else" now reads "moves golden streams and nothing else", which is what M-E shows and carries no universal. The two `nop`-bodied header pins do not move, because no op's register index changes in them.

## N6

"The body is `DO`, `nop`, `END`" is now "The program is ...", so "body" means one thing in a doc whose test name uses it for the loop's.

## The restoration

The `DO OVER ... FOR` row's comment now records that it is the only trace-level record of that shape in the crate, and points at `docs/superpowers/plans/2026-08-13-over-for-keyword.md` as the task whose Step 4 has to rewrite the block. Plan Step 4 records the same.

**I verified the uniqueness myself rather than taking it**: across `tests/`, `src/` and both corpora, the other `DO OVER ... FOR` occurrences are `run.rs`'s `say_output` unit test (stdout only, no trace), `golden_tests.rs`'s compiled stream (never run), and two `parse-errors.tsv` rows. `ir_dual.rs`'s `do qq over 4.5` has no `FOR`.

## Gates, at the final state

From `rust/`, each status read unpiped:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
* the same clippy from a **clean** `CARGO_TARGET_DIR` -- exit 0, every crate in the `Checking` list, no line matching `^warning` or `^error`
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, **1471 passed, 0 failed, 4 ignored**, the baseline for this round

## Not fixed

Nothing from the re-review.
