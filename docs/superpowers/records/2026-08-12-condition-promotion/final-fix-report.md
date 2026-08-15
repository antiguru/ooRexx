# Final fix round: the whole-branch review's prose findings

One commit, on `plan/rust-rewrite`, over `a2952fd62`.
Eight items assigned; **eight fixed, plus one fourth instance of item 3's defect found in the same file and fixed with it**.
No behavioural change was made or intended: the only non-comment edit is three `debug_assert!`s and the two `use` lines they need.

## Gates, from `rust/`, at the final state of the commit

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0, no output |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `memcap 8G cargo test --workspace --no-fail-fast` | **1476 passed, 0 failed, 4 ignored**, exit 0 |

Baseline exactly. `both_engines_agree_across_every_population`, `both_engines_agree_on_every_case_file` and
`every_population_the_tree_calls_for_is_present_and_non_empty` all ran and passed -- worked in the repository itself,
not a detached worktree, so the 27 path-resolution failures the review documents did not arise.

## What each item became

**1. `ir/drive/tests.rs`, the `IF` count test's doc.** "the condition is evaluated by the same `eval_if_condition`
either way" is replaced by the `SELECT` sibling's own formulation: an `IF`'s own condition reaches the same
`Interp::condition_value` on either engine, whether it gets there through `eval_if_condition` or through the ops
`Op::Condition` ends. Both named routes were checked to terminate there -- tree-walker `step` -> `eval_if_condition` ->
`eval_condition` -> `condition_value`; compiled native condition -> `Op::Condition` -> `condition_value`; compiled
declining condition -> `Op::EvalExpr` -> `eval_chunk_expr`'s `If` arm -> `eval_if_condition`. The conclusion is kept.

**2. `ir/golden_tests.rs`.** The reason is replaced, not the conclusion: `native_shape` has no `ExprKind::Logical` arm
(`compile.rs`, the `_ => false` tail), so a comma list declines wherever it sits and a row for it would pass under
every implementation. Nothing about `Op::EvalExpr` is claimed any more.

**3. `queue.rs`, four passages rather than three.** The review named the module doc's opening line, the module doc's
I3 argument, and the same argument above `push_and_queue_actually_write_into_the_running_interpreters_queue`. A
**fourth** carried the identical mislocation -- `interleaved_push_and_queue_match_the_oracle_order`'s doc, "It says
nothing about whether `run.rs`'s `step` arms actually call these methods". All four now name
`Interp::queue_evaluated`. Verified by grep that outside this module's own tests the only `Queue::push`/`Queue::queue`
call sites in the crate are `run.rs:2901-2902`; the prose names the function, not a count.

The module doc's duplicate of the I3 argument is reduced to a pointer at the test's own doc, which is the one copy
that keeps the measurement. That measurement is kept and re-dated -- "Measured at review round 1, when the write sat
in `step`'s arm alone" -- because its numbers (978 passed, STRICT 39 of 39) are a record of a code shape that no
longer exists, and reading them as today's blast radius is exactly the defect the review found. The sentence beside
it now states the present mechanism instead: the calls are `queue_evaluated`'s, `step`'s arm and `Op::Queue` both
enter it, so a deletion there takes the write away from both engines at once. **No new mutation result is claimed**
for that passage, and none was measured for it.

Two stale asides in the rewritten passage went with it, since they were inside the sentences being replaced: the
"4c-shaped probe" framing and "the three `PULL`s 4c has not implemented yet" -- `Interp::pull_line` exists. The
sentence now just says the probe is the three `PUSH`/`QUEUE` clauses without the `PULL`s.

**4. `run.rs`, both `raise` glosses.** Deleted rather than extended to four: `eval_condition`'s now ends at "the
keyword-specific raiser for that case", and `condition_value`'s at "for the unchecked case, chosen by whichever
caller had the condition in hand". Naming all four numbers would be the same enumeration shape one task further
along, and `WHILE`/`UNTIL` are already discussed by name three paragraphs above the second one. The measured 34.6
-vs- 34.1 paragraph below the first is untouched -- it is evidence, not an enumeration.

**5. `2026-08-12-remaining-promotion-survey.md`.** Task B keeps its section and gains a leading record of what landed:
two ops with a keyword tag, why four were rejected (inside each pair the keyword is the only difference) and why one
was rejected (between the pairs the trace rule, the side effect and the region end all differ). The sketch below it
stays, marked as what the members still in Part 2's "one expression then a `Flow`" row take, and its one refuted
sentence -- "`Op::Return { index, src: Option<u16> }` (and its three siblings)" -- is neutralised to "an op carrying
the instruction index and `src: Option<u16>`". Part 2's row loses the four; "Already promoted" gains them.

Two neighbourhood consequences: the pointer at the decision table now says the plan *points at* the table (it lives
in `task-4-report.md`, outside the tracked tree) rather than carrying it; and "Expressions: ... -- once Task 4 lands
-- a call at any depth the path carries" lost its conditional, because it sat two paragraphs under a new
"Landed" heading and read as a contradiction. Checked before removing: `native_shape`'s `ExprKind::Call { .. } =>
path.is_some()` with `descend` is the promotion, and it is in the tree.

**The phantom was left alone.** The survey carries no claim that the comma list needs a jump inside a clause region;
`/bin/grep` over the file confirms it, and `progress.md:83` already records the ledger correction, so nothing was
owed there either.

**6. `tests/ir_dual.rs`'s module doc.** The list is deleted rather than extended, per "prefer a rule that stays true
over a list that rots". What replaces it is the property with no enumeration in it: a construct that both arms
resolve by calling the same `Interp` function answers identically on both by construction, and every promotion that
shares its tail instead of re-implementing it widens that blind spot rather than narrowing it. That last clause also
sets up the paragraph two below, which is about what the comparison *does* see -- an op that re-implements.

A rule keyed on the marker phrase "The one implementation both engines enter" was considered and rejected: it is
carried by some of these functions and not by others (`run_loop`, `invoke_call` and `condition_value` all say it in
different words or not at all), so it would have been a fresh false universal.

**7. `ir/drive.rs`, one `matches!` per arm, all three ops.** `Op::Return`, `Op::Queue` and `Op::Condition` each gained
a `debug_assert!` pinning the tag against the clause kind as a pair. `Op::Condition` had neither kind nor tag pinned;
it now has both, since the pair assertion subsumes the kind. Existing arity assertions are untouched.

**Each of the three was run red, one mutation per assertion, in a debug build**, with a non-zero test-run count read
off each result line rather than an exit status:

| mutation in `compile.rs` | test run | outcome |
|---|---|---|
| `Op::Condition`'s `If` tag emitted as `When` | `ir::drive::tests::the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk` | 0 passed / 1 failed, panic at `drive.rs`, "a Condition op's keyword does not name the clause whose condition it is validating" |
| `Op::Condition`'s `When` tag emitted as `If` | `ir::drive::tests::the_ir_engine_steps_a_selects_chosen_branch_from_the_chunk` | 0 passed / 1 failed, same assertion |
| `ReturnKeyword` selection swapped | `ir_dual::both_engines_agree_on_every_case_file` | 0 passed / 1 failed, "a Return op's keyword names the other half of the pair from the clause it ends" |
| `QueueKeyword` selection swapped | `ir_dual::both_engines_agree_on_every_case_file` | 0 passed / 1 failed, "a Queue op's keyword names the other half of the pair from the clause it writes for" |

**What this does and does not establish.** It establishes that none of the three assertions is vacuous -- each is
reached and each can fail. It does **not** establish that any of them adds coverage: the review already records a
behavioural witness for both halves of `ConditionKeyword` and for `Op::Return`'s pair, so a mis-tag was already a red
test. What the assertion buys is the failure arriving at the driver with the tag named, rather than as a byte diff.
No coverage claim is made anywhere in the code comments.

`compile.rs` was restored from a scratchpad copy (never `git checkout --`), verified with `sha256sum -c` (OK), and
the workspace was rebuilt before the final gate run, so no probe or gate here read a mutated binary.

**8. `Seen::native_conditions`.** "How many `Op::Condition` ops the sweep saw, keyed by the keyword each is tagged
with." The anti-vacuity paragraph after it is unchanged.

## Not fixed, and why

* **F2, the plan's closing `rexxcps` measurement and `phase-4f-record.md` Entry 27.** Not in this round's brief, and
  it is a measurement rather than a fix -- it needs the host-idle gate, two do-nothing controls and interleaved arms.
  It remains the one outstanding plan step, tracked in `progress.md`'s REMAINING line.
* **F8's second half, a sentence in `ConditionKeyword`'s doc** explaining why its half resolves the tag at the driver
  where `Op::Return`/`Op::Queue` pass the tag through. The brief asked only for the `matches!`. Adding a paragraph of
  fresh design prose is precisely the move that has been seeding new false statements on this branch, and the
  asymmetry it would explain is already visible in three adjacent driver arms. Left for whoever wants it.
* Everything on the ruled "leave alone" list: `push_queue.rex`'s two-of-three read-back list, the twice-written
  `If`/`When` mapping, `root_exit_value`'s `EXIT`-phrased doc, the triplicated `compile.rs`/`drive.rs` arms,
  `compile.rs`'s `_ =>` in the `When` match, `Op::EvalExpr`'s slot examples, the plan's unticked checkboxes.
* One pre-existing false line the review flagged as **not this plan's** and not on the fix list, still live and worth
  someone's next pass: `ir/drive/tests.rs`, twenty lines above the `IF` test, "`If` steps its own branch through the
  tree-walker (that promotion is not this task's)" -- contradicted by the very next test in the file.
