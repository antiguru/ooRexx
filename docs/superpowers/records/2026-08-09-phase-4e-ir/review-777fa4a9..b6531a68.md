# Review: Task 4a, run a loop's body from the compiled stream

Range `777fa4a9..b6531a68`, commits `c6b1c32d` and `b6531a68`.
Scope as accepted in `27b522bf`: the loop header's flattening, `Op::Clause`, `Op::EvalExpr`,
`Op::Jump`, `Op::JumpUnless`, the register allocator and the no-`Clause`-before-`Generic` assertion
belong to Tasks 4b and 4c and are not judged here.

## Verdicts

* **Spec compliance: pass.**
  Every brief step inside the accepted scope is discharged, and the one that carries the phase's
  central rule -- Step 4, extract the loop semantics -- is discharged in the strongest form
  available.
* **Task quality: pass, with one Important finding.**
  One of the five `LoopKind` arms is promoted by code that no test pins, and the case named for it
  never reaches the promoted path.

## Question 1: extraction, or a second implementation?

**No loop semantics exists twice.**
`run_loop`, `run_repeating` and `loop_advance` are one implementation entered from both engines; the
only engine-dependent line in the whole promotion is `run_bounded`'s two-arm match on `BodyEngine`,
and its `Chunk` arm reaches the same `Interp::step_in_temps_frame` clause unit the `TreeWalker` arm
calls, one indirection later through `step_from_chunk` -> `run_clause_ops` -> `Op::Generic`.
Verified rather than read off the diff: `BodyEngine::Chunk` is constructed in exactly two places
(`run.rs:4926`, `ir/drive.rs:196`), both on that one path, and `step_in_temps_frame_with` has exactly
one non-`TreeWalker` caller.

A structural guard makes this hard to lose, and it was found by a mutation that failed to compile:
`Interp::step` is private to `run.rs`'s module, so the IR driver **cannot** call it.
`run_clause_ops`'s only reachable delegation is the clause wrapper.

## Question 2: is the loop body's clause unit still whole?

Yes, and it is whole by identity rather than by re-derivation.

`step_in_temps_frame` is now a thin wrapper over `step_in_temps_frame_with`, which is the old
function's body with one added parameter that it forwards to `step` and reads nowhere else.
So the `DATE`/`TIME` clock invalidation, the `>I>` trace-entry decay, `current_value_indent`, the
`SIGL` clause line, the `*-*` clause echo, the GC temps frame with its watermark tripwire, failure
site resolution and `CALL ON` delivery inside `in_clause` are the same code on both arms, taken once
per clause.

The obligations that live *outside* the clause unit stay symmetric too.
`grant_procedure_permission`, `offer_to_trap` and `apply_flow` are called only by the two outer
activation loops (`run_activation` and `run_chunk_clauses`), and `run_bounded` calls none of them on
either arm, so no body clause gets a second trap offer or a second permission grant.

Counted, not argued: a three-pass loop over a one-clause body enters the clause unit four times on
the IR arm, which `the_ir_engine_steps_a_loop_body_from_the_chunk` asserts, and dropping the wrapper
for body clauses under the chunk engine goes red in three places (mutation A2 below).

## Findings

### Important

**I1. No test pins `LoopKind::Simple`'s body to the compiled stream, and the case named for it never
reaches it.**

`LoopKind::Simple` is the one arm of `run_loop` with its own `run_bounded` call; the other four route
through `run_repeating`, which has a single one.
Changing that one call to pass `BodyEngine::TreeWalker` -- un-promoting a `DO ... END` block's body
while leaving every other loop promoted -- leaves the entire workspace green: 1353 passed, 0 failed.

Two things compound it.
`LOOP_CASES`'s "simple block" entry is written as `if 1 = 1 then do / say 'a' / say 'zz' / end`, and
`If`'s arm hard-codes `BodyEngine::TreeWalker`, so that program never reaches `Op::Loop` at all.
Measured with a temporary probe in the `Simple` arm: `chunk=false` for that program, `chunk=true` for
a bare top-level `do / say 'a' / end`.
And the only observable that can see promotion at all -- the clause count -- is asserted for one
program, a controlled loop.

So `LOOP_CASES`'s own doc, "the shapes the loop promotion has to keep: one per `LoopKind` this crate
runs", is true of engine *agreement* and not of the promotion: the `Simple` shape agrees on both
engines because on both engines it is the tree-walker.
The fix is one line in the test file (a top-level `do ... end` case) plus a clause-count assertion
for it, or the same shape added to the drive test.

### Minor

**M1. The report's mutation log understates its own new tests, and the understatement is a false
universal.**

The report says "M4 and M5 are where the new tests are the only thing standing".
Measured: dropping `UNTIL`'s second, unconditional `DO` re-echo under `Engine::Ir` is caught by
`both_engines_agree_on_every_loop_shape` **and by nothing else** -- the corpus sweep misses it,
because no program in the swept populations pairs a `TRACE` setting with an `UNTIL` loop.
The two traced `LOOP_CASES` entries are therefore uniquely load-bearing for trace-shaped,
engine-gated divergence, which is a third place the new tests stand alone.
The direction is undervaluing rather than overclaiming, so this is a correction to the ledger and not
a test defect -- but it is a claim about a measurement that the measurement does not support, which
is the shape this project treats as a defect regardless of direction.

**M2. `REXX_ENGINE`'s "rejected rather than defaulted" contract has a hole.**

`std::env::var("REXX_ENGINE").as_deref()` collapses `VarError::NotUnicode` into the same `Err(_)` arm
as `NotPresent`, so a non-UTF-8 value is treated as unset.
Measured against the built binary: `REXX_ENGINE=nonsense` and `REXX_ENGINE=` both exit 2 with the
diagnostic, and `REXX_ENGINE=$'\xff\xfe'` exits 0 and runs the tree-walker silently.
The doc comment says "Any spelling other than the two below is rejected rather than defaulted", which
is what the value of the surface rests on.
Matching on `env::var_os` closes it, or splitting the `Err` arm does.

Everything else about the surface checks out.
It selects: with a temporary `eprintln!` in `run_chunk`, unset gives 0 chunk entries, `tree-walker` 0
and `ir` 1, which reproduces the report's own probe exactly.
And nothing in the library reads an engine choice from the environment -- `env::var` appears in
`rexx-exec` only in `src/bin/rexx-run.rs`, and the carrier into the interpreter is still
`Invocation::with_engine`.

**M3. The plan still asserts twice that `rexx-exec/src` contains no `env::var` at all.**

Lines 357 and 444 of `docs/superpowers/plans/2026-08-09-phase-4e-ir.md` state it flatly, and it is
now false by path: `rexx-exec/src/bin/rexx-run.rs` is under `src/`.
Line 534 records the correction beside Task 4a's entry instead of correcting the two originals, so a
reader of the Decisions section or of Task 2's steps still reads a false sentence with nothing beside
it.
Outside this review's diff (it is `27b522bf`'s), but caused by this task's surface, and it is exactly
the "correct the plan, not the message that carries the work" rule applied halfway.

**M4. `Invocation`'s `Engine` doc argues against an environment variable with no cross-reference to
the one that now exists.**

It opens "**A field of [`Invocation`] rather than an environment variable**, and the difference is not
stylistic".
The argument it makes stays true -- a variable read once per process cannot give two arms inside one
`cargo test` process -- and `rexx-run` reading one and passing it through `with_engine` is consistent
with it.
But a reader of the type now concludes that no environment variable selects an engine anywhere, and
one does.

**M5. Two in-repo enumeration claims sit in prose where this tree's own rule says to assert or delete
them.**

`step_in_temps_frame_with`'s doc says "only the compiled stream's own driver ever passes anything but
`BodyEngine::TreeWalker`".
That is a claim about every call site in the crate.
It is true today -- verified, two `BodyEngine::Chunk` constructions, both on the promoted path -- and
nothing re-reads the sentence when a third appears.

`Op::Loop`'s doc says "Every promotion after this one is of an instruction that spends its life
inside a loop body".
That is a claim about tasks that do not exist, and it is the kind of narration of a future change the
comment rules exclude.
It is also doubtful on its own terms: Task 4b promotes `If`/`Select`, which occur at top level as
readily as inside a loop.

**M6. The instruction-to-op mapping and its `chunk_map_too_short` guard now exist twice.**

`run_chunk_clauses` does the `chunk.op_of.get(index)` lookup inline and `step_from_chunk` does it
again, four lines apart in the same file, and `step_from_chunk`'s doc says so ("it does the
instruction-to-op mapping the outer loop above does") rather than removing the duplication.
`run_chunk_clauses` can call `step_from_chunk`; the only ordering difference is
`grant_procedure_permission` moving ahead of a lookup that cannot fail for an index the loop guard
already admitted.

**M7. The anchor's new section is titled "Task 4", and Task 4 is now three tasks.**

`phase-4e-anchor.md` records the measurement under "## Task 4: `Do`/`Loop`, predicted versus
measured", while the plan now calls the landed commit Task 4a and reserves the header flattening for
4b and 4c.
A reader looking up "Task 4" in the anchor gets a partial promotion's numbers under the whole task's
name.
It was accurate when written -- the split was accepted afterwards -- so this is a consequence of the
split that `27b522bf` did not carry through.

## Question 5: the measurement's honesty

The caveat is in the tree, not only in the report.
`docs/superpowers/plans/phase-4e-anchor.md`'s new section states, in the tree, that the prediction was
wrong in sign on both axes, that the movement "arrived by no route anyone has named", that the arm
doing strictly more per-clause work is the faster one, and that the result "should not be quoted as
'promotion made loops faster' until something identifies one".
It also states that the percentages are IR-arm-against-tree-walker-arm and are not comparable with
the document's own tree-walker-against-oracle ratios, because the sitting ran no oracle.
`27b522bf` repeats the no-mechanism caveat in the plan and adds that no later task takes the numbers
as a baseline.

The first sitting's threefold overstatement and its cause -- the tree-walker arm, which compiles
nothing, reading 3.05 s in one build and 2.88 s in another -- are recorded in both places.
The negative control is real: the same within-binary comparison on a reverted build reads -0.2% on
both axes, and a reverted build is the correct control because there both arms delegate identically.

I did not re-run the benchmarks, per the review's own instruction.
What I can say is that no reader of the anchor could mistake this for a confirmed win, and that the
report's "what this measurement does not establish" section names the relink question honestly rather
than leaving it to be discovered.

## Mutation results

Every mutation was applied to a `cp` backup, the whole workspace was run with `--no-fail-fast`, and
the file was restored from the copy and verified with `sha256sum -c`.
No `git checkout --` anywhere.
Baseline: 1353 passed, 0 failed, 4 ignored -- which matches the report's `1349 + 4`.

| # | mutation | red tests | reading |
|---|---|---|---|
| M1 | `run_repeating`: drop the per-pass `END` echo | 9 -- `both_engines_agree_on_every_loop_shape` plus 8 pre-existing trace units | report confirmed exactly; the new case adds no coverage here |
| M2 | the same, gated on `Engine::Ir` | 2 -- `both_engines_agree_across_every_population`, `both_engines_agree_on_every_loop_shape` | report confirmed |
| M4 | `compile` emits `Generic` for `Do`/`Loop` again | 2 -- the golden test and the clause-count test | report confirmed; nothing else in the workspace |
| M5 | `run_bounded`'s `Chunk` arm falls back to `step_in_temps_frame` | 1 -- the clause-count test; the golden test stays green | report confirmed |
| A | `run_clause_ops`'s `Generic` arm calls `step` instead of `step_in_temps_frame` | does not compile: `step` is private to `run.rs` | the clause unit cannot be bypassed from the driver |
| A2 | loop-body clauses skip the clause unit under the chunk engine (`run_bounded`'s `Chunk` arm calls `step` directly) | 3 -- the population sweep, the loop-shape test, the clause-count test | the per-clause obligations are pinned |
| B | `DO OVER` runs zero passes under `Engine::Ir` | 2 -- the population sweep and the loop-shape test | the new case is a fast second catcher, not unique |
| C | drop `UNTIL`'s second unconditional `DO` re-echo under `Engine::Ir` | 1 -- `both_engines_agree_on_every_loop_shape` **only** | the traced cases are uniquely load-bearing; see M1 above |
| D | `LoopKind::Simple` passes `TreeWalker`, un-promoting a block's body | **0 -- the whole workspace stays green** | finding I1 |

The report's M3 was not re-run; mutation B is the same shape (an engine-gated iteration-count change
in a `LoopKind` arm) and reproduces its result.

## Ordinary checks

* No `unsafe` added, and the workspace still sets `unsafe_code = "forbid"`.
* No em-dash in any added comment or added Rust line.
* `cargo fmt --all --check` clean.
* `cargo clippy --workspace --all-targets -- -D warnings` clean, on a warm target directory, so
  provisional by this tree's own rule.
* `cargo test --workspace`: 1353 passed, 0 failed.
  `cargo test --workspace --release`: 1353 passed, 0 failed.
* Doc comments state contracts and the reasoning sits at the decision points.
  The one exception worth naming is that the large `step_in_temps_frame` contract stayed on the thin
  wrapper while the implementation moved to `step_in_temps_frame_with`; the `_with` doc points back at
  it, so this is a deliberate choice rather than a drift, and it reads correctly.
* Counts of mutable in-repo aggregates: none added.
  The two prose claims about in-repo state are M5 above, and neither is a count.
* The two traced `LOOP_CASES` expectations were checked against the oracle directly, and both match
  byte for byte on stderr.
  So the tests enshrine oracle-correct trace rather than this crate's own answer, even though the
  struct's doc only claims the tree-walker's answer.

## What the promotion leaves for 4b

Sound, on the evidence above.
The one engine-dependent line is a parameter, `Op::Loop` carries no payload it would have to unlearn,
`run_fragment` and the `If`/`Select`/`OTHERWISE` bodies pass `TreeWalker` explicitly with a reason at
the call site, and `run_clause_ops` still answers one clause's `Flow`, which is the shape 4b replaces
with an op program counter rather than the shape it has to undo.
Nothing here works by accident: the mutations that should be red are red, and the one that should be
red and is not is I1, which is a missing test rather than a broken mechanism.
