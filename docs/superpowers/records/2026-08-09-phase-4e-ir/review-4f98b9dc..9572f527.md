# Review: Phase 4e Task 6, `4f98b9dc..9572f527`

Reviewed at `d03cc3af` with the three task commits in place.
Every number below was re-measured rather than read out of the report.

## Verdicts

**Spec compliance: meets the spec.**
Every step of the brief is discharged and I verified each one independently.
The two adjudicated departures are out of scope for this review, but the one controller instruction that was deviated from -- `datadriven` for a new case table -- rests on a stated reason that this repository's own files contradict (F5).

**Task quality: accept with required fixes.**
The nine-row mutation log reproduces exactly, row for row, including both adds-coverage checks.
The performance table reproduces to five significant figures on an interleaved A/B against a build of `4f98b9dc`.
Against that: one decision-point measurement is wrong (F1), the measurement the report says is recorded at its decision point is not recorded anywhere (F2), and the task's central mechanism can be switched off in production with the whole workspace green (F3).

## Method

* Suite baseline confirmed at **1373 passed, 0 failed, 4 ignored**, before and after every mutation.
* Mutations applied to a `cp` backup, restored from it, verified with `sha256sum -c` against the repository, `git status --porcelain` empty, and rebuilt.
* `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` clean on a warm target, so provisional.
* Oracle probes run from fresh empty directories, absolute paths, `ulimit -v 1048576`, stdout/stderr/status read as three descriptors.
* Perf A/B against a throwaway git worktree at `4f98b9dc`, `perf stat -e instructions:u`, interleaved base/here within one sitting, two sittings. The worktree is removed.

## The staleness check

**I could not construct a case it misses, and I tried 540 programs.**

The structural argument first, because it is what the probes are evidence for.
`Op::TraceClause` is the only site in the tree that emits a clause echo from a compiled chunk; every other echo goes through `in_stepped_clause`'s `Echo::Gated` arm, which the tree-walker and the driver's `Generic`/`Loop` arms share unchanged.
`stale` is read in the `Op::Clause` arm from `self.chunk_trace()`, which is `ChunkTrace::of(self.activation().trace_mode)` -- the activation that is executing now, not the one the chunk was fetched for -- and the only code between that read and the `TraceClause` op is `in_stepped_clause_with`'s prologue and `in_clause`, neither of which touches `trace_mode` or the activation stack.
So there is no window, and the question reduces to whether `ChunkTrace` is the whole of what decides an echo.
It is: `compile` asks `ChunkTrace::echoes` and `Interp::tracing_clause` now delegates to the same function, so the two cannot disagree by construction rather than by convention.

What I ran against that argument:

* **`INTERPRET`.** `interpret "trace r"` at top level, inside a branch, and inside a loop body before a later promoted `IF`; `interpret "trace n"` in a body entered under `trace r`. The fragment runs in the enclosing activation and does change the setting, and all three engines agree on every one.
* **`TRACE VALUE expr`.** `trace value cv` before a promoted `IF`, under all six settings, with and without a preceding `trace r`/`trace i`. The setting change propagates correctly and the staleness check catches it. See F8 for a separate, pre-existing gap this turned up.
* **The propagation rules.** A `CALL` to an internal label inheriting `trace_mode` and then turning it off (the caller's next clause still echoes); a `::ROUTINE` resetting to `NORMAL` under a caller running `trace r`; a `PROCEDURE` body. `run_activation` asks `chunk_for` after the callee's activation is pushed, so a `::ROUTINE`'s reset is visible at the lookup rather than only at the first clause.
* **The mode collapse.** `ChunkTrace` maps `A`/`R`/`I` to one key and `O`/`N`/`C`/`E`/`F` to another, so `trace r` -> `trace i` is *not* stale and the compiled echo keeps firing. That is correct -- the `*-*` bytes do not depend on `results` or `intermediates` -- and `trace a` <-> `trace r` <-> `trace i` <-> `trace o` <-> `trace c` transitions across a promoted clause all match the oracle.
* **A generated sweep**, 480 programs: five entry settings times six target settings times sixteen shapes (branch, `ELSE`, `DO` body, `SELECT` `WHEN`, `SELECT` `OTHERWISE`, `SELECT CASE`, loop with the change on the first pass and on every pass, first clause, last clause, across a `CALL`, inside a callee, through `INTERPRET`, through `TRACE VALUE`, and a `SELECT` inside a loop). Zero IR-versus-tree-walker divergences; zero oracle divergences except the twelve `TRACE VALUE` rows of F8.
* **A boundary-focused sweep**, 43 further programs: nested `IF` in `IF`, a change inside a `DO` block branch, `LEAVE`/`ITERATE` after a change, `SIGNAL ON SYNTAX` with the change before the raising clause (`SIGL` correct on both arms), `CALL ON ERROR`, a change in a `WHEN` branch before the `END`, a change in an `OTHERWISE` branch, `DO 2`/`DO FOREVER`, a change as the last clause of a body, `SIGNAL` to a label immediately before a promoted `IF`, a change inside a `PROCEDURE`, `SELECT CASE` under `trace i` (the `>K> "CASE"` ordering), and `trace ?r`/`trace ?n`. The only two divergences are the documented interactive-prefix exclusion and the pre-existing invalid-character parse-error shape, both identical on the two engines.

The residual I would state rather than claim away: the check is only as complete as `ChunkTrace`'s two fields.
If a future `TraceMode` field ever decides whether a clause echoes, adding it to `TraceMode` without adding it to `ChunkTrace` is a wrong-output defect with nothing structural to catch it, because `ChunkTrace::of` is a hand-written projection.
Today no such field exists.

## Boundaries, not shapes

`TRACE_SETTING_CASES` puts every setting change immediately before a promoted clause, which is the right axis, and the seven rows cover: a branch (both directions), a loop body, a re-entered body, a `SELECT` header plus a listed `WHEN`, and an unpromoted assignment as the Task 7 guard.
I ran all seven programs against the oracle myself: all seven match on both engines, so the recorded bytes are measured as the table's doc claims.

What the table does not reach, and I checked each externally rather than asserting it is fine:

* No row places a setting change at a **branch end** -- the last clause of a taken `IF` branch or of a `WHEN` branch, where `Op::EndBranch` runs the boundary a flattened construct has no wrapper for. This is the 4b' family's habitat, and it is where a per-boundary observable would show it. Probed directly (a `trace r` as the last clause of an `IF` branch, of a `DO` block branch, of a `WHEN` branch and of an `OTHERWISE` branch, each followed by a promoted clause): no divergence, on either engine, against the oracle.
* No row combines a setting change with a **condition delivered at a promoted clause's boundary**, which is the other half of that family. Probed with `SIGNAL ON SYNTAX` and `CALL ON ERROR` around a promoted `IF` with the setting changed on either side: no divergence, and `SIGL` correct.
* No row uses **`INTERPRET`**, and it is the one mechanism named in the brief that can change the enclosing activation's setting from inside a clause. Probed; correct.

So the report's "None that are new, and none from the catalogued family" holds, and its reading of the family -- that the catalogued divergences need a `CALL ON` handler raising a second trapped condition -- matches `task-4bp-report.md`'s own table for the five rows that are the family.
I would still add the branch-end row to the table, because the argument that it cannot diverge is exactly the shape of argument the last three tasks got wrong; that is a suggestion rather than a finding, since the behaviour is measured correct.

## Findings

### Important

**F1. The staleness read's decision-point measurement states 40.30 billion where the shipped code measures 40.40.**

`ir/drive.rs:397`-`400`:

```
// once per pass: initialising one there cost 40.30 to
// 40.55 billion user instructions, 10 per pass, on a
// program that has nothing for the answer to decide. Asked
// here it is 40.30 billion again, ...
```

Measured, `emptyloop.rex`, `REXX_ENGINE=ir`, `perf stat -e instructions:u`, interleaved with a build of `4f98b9dc` in one sitting, two sittings:

| build | run 1 | run 2 |
| --- | --- | --- |
| `4f98b9dc` | 40,300,662,962 | 40,300,663,071 |
| `9572f527` | 40,400,662,096 | 40,400,662,133 |

The shipped shape is 40.40 billion, not 40.30.
The report's own prose says so -- "Read in the `Clause` arm it is 40.40 billion" -- so the report and the comment contradict each other about the same program on the same commit, and the comment is the one that is wrong.
Worse than a stale number: "40.30 billion again" asserts parity with the pre-task baseline, which is precisely the +0.25% the report's own table records and attributes elsewhere.
A reader of that line concludes the per-clause read is free against `4f98b9dc`, and it is not.

Fix: correct the two figures at the site, and say what the range is measured against, since "40.30 to 40.55" is only meaningful if the reader knows 40.30 is the base build rather than this one.

**F2. The four `#[inline(always)]` annotations that keep the tree-walker neutral carry no reasoning, and the report says they do.**

`trace.rs` adds `#[inline(always)]` to `ChunkTrace::of`, `ChunkTrace::echoes`, `Interp::tracing_clause` and `Interp::chunk_trace`.
None of the four doc comments mentions performance, inlining or a measurement.
The report says: "`ChunkTrace` stopped folding at the gate until `of`/`echoes`/`tracing_clause`/`chunk_trace` were annotated. **Both are recorded at the annotations with the numbers.**"
Only the first of the two is recorded at its annotation; the second is recorded nowhere in the tree.

They are load-bearing, and by exactly the amount that matters.
Measured, all four annotations removed and nothing else changed, `emptyloop.rex`, release:

| arm | with the annotations | without |
| --- | --- | --- |
| tree-walker | 38,000,657,395 | 38,100,658,898 |
| ir | 40,400,661,617 | 40,500,663,096 |

100,000,000 instructions on 25M clauses, four per clause, +0.26% -- **on the tree-walker**, which is the arm the task's own "the half that had to be true" is about.
So the one property the report singles out as non-negotiable is held up by four bare attributes that a future edit removes without anything going red and without a sentence to stop it.
This is the project's own rule (`rust/CLAUDE.md`: reasoning at the decision point, and a measurement justifying the current design earns its place), applied to the case it exists for.

Fix: put the measurement on the annotations, as `echo_stepped_clause` and `settle` already do. Correct the report's sentence, which is currently false.

**F3. Nothing pins that the running interpreter ever asks for a chunk under the setting in force, so the whole compiled-echo path can be switched off with the suite green.**

Two extra mutations, both applied to the committed tree, both `cargo test --workspace --no-fail-fast`:

| mutation | result |
| --- | --- |
| **R1** -- `run_activation` passes `ChunkTrace::of(TraceMode::NORMAL)` to `chunk_for` instead of `self.chunk_trace()` | **1373 passed, 0 failed** |
| **R2** -- `stale` forced to `true` at the `Op::Clause` arm | **1373 passed, 0 failed** |

Either one makes `Op::TraceClause` dead in production: under R1 no chunk is ever compiled under an echoing setting, so the op is never emitted; under R2 it is emitted and never run.
Every promoted clause of every traced body falls back to the run-time gate, output stays byte-correct, and the entire workspace including the corpus sweep and the new case table stays green.

This is not the same finding as N4.
`one_body_under_two_trace_settings_is_two_cached_chunks` is a good test and it does pin the key -- I checked it against the three degenerate caches by hand, and the fourth assertion (`Rc::ptr_eq` on a repeated first-setting lookup) is what rules out both a no-cache implementation and an evicting one, so it is not tautological.
But it calls `chunk_for` directly with an explicit `ChunkTrace`, and `chunk_for` has exactly one production caller, whose argument nothing checks.
No output test can close this, because the fallback is output-equivalent to the compiled path by construction -- that is the fallback's whole purpose.
It has to be closed the way the cache test itself is closed: by counting.
The crate already has the mechanism (`count_compile_call`, `count_run_chunk_entry`, `count_clause_op_entry`, all `#[cfg(test)]`), so a counter on the compiled echo, or a test that drives `Interp::run` over a body entered under `trace r` and asserts the chunk it got was compiled for `R`, would be idiomatic here.

The report's claim that the key change is "load-bearing rather than a claim in a doc comment" is true of `chunk_for` and not true of the interpreter.
Until R1 reddens, D23's benefit -- a traced chunk that emits rather than asks -- is a property of a function nothing in production is checked to call correctly.

### Minor

**F4. `echo_stepped_clause`'s `inline(always)` is justified at the site with the instrument the report declares unreliable for that program.**

The doc comment reads "`bench-programs/emptyloop.rex` runs 2.80-2.82s on the tree-walker against 2.73-2.75s with the same lines written out at the call site", about 2.5%.
The report's Measurements section says "`emptyloop.rex`'s wall clock moved about 2% between builds executing an *identical* instruction count, so a wall-clock-only reading reported a tree-walker regression that does not exist".
So the site records a 2.5% wall-clock delta on the one program the report says wall clock cannot resolve at that magnitude.

The annotation is load-bearing -- more so than either number says.
Measured, that one annotation removed: tree-walker 38,000,658,335 -> 38,525,658,925, +525,000,000 instructions, +1.38%, 21 per clause.
The report attributes "100 million instructions" to this site; 100,000,000 is the figure for the four `ChunkTrace` annotations of F2, exactly, so the two look swapped.
Put the instruction count at the site and correct the attribution.

**F5. Concern 6's stated reason is contradicted by this repository's own expectation files.**

The first half is true: an `IF` header's `*-*` echo ends in a compared trailing space, confirmed with `cat -A` on the oracle's own output (`     2 *-* if 1 = 1 $`).

The second half -- "an external data file does not survive ordinary editing with trailing blanks intact" -- is false here.
`crates/rexx-exec/tests/trace_oracle/trace_output.expected` line 18 is `     4 *-* if y > 5 ` with the trailing blank, committed and compared byte-exactly; `controlled_loop.expected` carries three more.
There is no `.editorconfig` and no pre-commit hook in the tree.
`datadriven` 0.9.0's own parser pushes each expected line verbatim and only treats a line as a terminator when `trim()` is empty, so a trailing blank on a non-blank line survives the format too.

The decision itself is defensible on its other stated ground -- it reuses `InlineCase` and `compare_inline_cases` unchanged and adds no comparison code -- and I would not ask for it to be reversed.
But this is the "false justification rides a correct decision" shape: the reason is what a later reader will act on, and it does not hold.
Either restate the reason as the reuse argument, or use `datadriven` as instructed.

**F6. `one_body_under_two_trace_settings_is_two_cached_chunks`'s doc counts three assertions and the test has four.**

The doc says "The three assertions answer three different degenerate caches, and the third is the one the key change is for", then gives three bullets.
The test asserts four things; `render(&silent) != render(&echoing)` is not among the three described, and it is not the weakest of them -- it is what says the setting decided anything at all.
A count of an in-repo aggregate in prose, wrong on the commit that wrote it, which is the failure mode `rust/CLAUDE.md` names.

**F7. `Op::TraceClause`'s two structural contracts are stated and not asserted, next to the function that asserts the neighbouring one.**

The doc states "**Only valid inside a [`Op::Clause`] region**, and it is the region's first op".
`assert_clause_regions_hold_no_clause_op` runs over exactly those regions at the end of `compile` and checks neither.
Both are pinned only by the two golden tests for the two shapes that exist today -- mutation N5 confirms the position is pinned for `IF`, and I confirmed the `SELECT` golden pins it there -- so a third promoted construct can emit the echo in the wrong place, or twice, with nothing structural to catch it.
Three lines in a function that already walks the region would state it once for every construct.

### Out of scope, pre-existing, worth recording somewhere

**F8. `TRACE VALUE expr` never emits the oracle's `>K>  "VALUE" => "<setting>"` line.**

Found in the sweep, 12 of 480 programs, both engines identical, so it is a tree-walker gap and not this diff's:

```
if 1 = 1 then trace value cv          oracle:  >V>       CV => "r"
                                               >K>       "VALUE" => "r"
                                      here:    >V>       CV => "r"
```

`run.rs`'s `exec_trace` `Trace::Value` arm evaluates the expression, classifies it and calls `trace_invocation_entry`, with no `trace_keyword` call; `PARSE VALUE`'s own `>K> "VALUE"` is implemented, so the prefix exists.
Unchanged by this commit range, absent from `KNOWN_DIVERGENCES` and from `phase-4-exclusions.txt`.
The report's "`TRACE VALUE expr` ... were not probed separately" is honest, and this is the answer to it; per the project's own rule about corrections, it belongs in the receiving task's text rather than only here.

## What I replayed and it held

### Mutation log

All nine rows reproduce exactly, whole-workspace, `--no-fail-fast`, restored and rebuilt between each.

| # | Mutation | Reported | Measured |
| --- | --- | --- | --- |
| N1 | `Op::TraceClause` never emitted (`echoes` -> `false`) | red, 1369/4, the four new tests and nothing else | **1369 passed, 4 failed** -- `both_engines_agree_when_the_trace_setting_is_not_the_compiled_one`, the two new goldens, `one_body_under_two_trace_settings_is_two_cached_chunks` |
| N2 | `stale` forced to `false` | red, branch shapes + corpus sweep + case table | **1370/3** -- `both_engines_agree_across_every_population`, `..._on_every_branch_shape`, `..._when_the_trace_setting_is_not_the_compiled_one` |
| N3 | the compiled echo op ignores staleness | red, case table only | **1372/1** -- the case table only |
| N3b | N3 with the "entered under trace r, then turns trace off" row removed | green | **1373/0** |
| N4 | cache keyed on `BodyKey` alone | red, cache test only, output stays correct | **1372/1** -- `one_body_under_two_trace_settings_is_two_cached_chunks` only |
| N5 | the `IF`'s echo emitted after its `EvalExpr` | red, traced `IF` golden + case table | **1371/2**, exactly those |
| N6 | the compiled echo prints at indent 0 | red, case table only | **1372/1** -- the case table only |
| N6b | N6 with both entered-traced rows removed | green | **1373/0** |
| N7 | `ChunkTrace::of` drops `labels` | red, `trace_labels_covers_the_labels_only_mode` | **1372/1**, exactly that |

The two adds-coverage checks are real: N3b and N6b are green, so those rows are the sole catchers rather than merely able to fail.
N1's shape is worth calling out as the strongest row here -- 1369/4 with the corpus sweep silent says the emission path is entirely new coverage, and I confirmed the four are the four this task added.

### Measurements

Reproduced on an interleaved A/B against `4f98b9dc`, `perf stat -e instructions:u`, release:

| program | engine | `4f98b9dc` | `9572f527` | delta | report |
| --- | --- | --- | --- | --- | --- |
| `emptyloop.rex` | tree-walker | 38,000,658,767 | 38,000,658,385 | **-0.000001%** | 0.00% |
| `emptyloop.rex` | ir | 40,300,662,962 | 40,400,662,096 | +0.248% | +0.25% |
| `ifloop.rex` (3M passes) | tree-walker | 13,064,878,217 | 13,064,877,971 | 0.00% | 0.00% |
| `ifloop.rex` | ir | 13,448,882,239 | 13,499,882,766 | +0.379%, 17/clause | +0.38%, ~17 |
| `toggle.rex` (300k passes) | tree-walker | 1,494,953,493 | 1,494,952,869 | 0.00% | 0.00% |
| `toggle.rex` | ir | 1,567,857,671 | 1,576,557,226 | +0.555%, +8,699,555, 29/pass | +0.55%, +8.7M, 29/pass |

**"The tree-walker is exactly neutral" is supported.**
Four hundred instructions out of thirty-eight billion on `emptyloop`, sub-thousand on the other two, in both directions across repeats -- that is measurement noise in the loader, not a per-clause cost.
It is supported *because of* the four annotations of F2, which is the finding.

The `TRACE`-in-a-loop figure the brief asked for is right and the reasoning behind it is right: `chunk_for` is consulted once per `run_activation`, which I confirmed by reading, so a `TRACE` inside a loop never reaches the cache and the 29 instructions per pass are the staleness comparison plus the gated echo, not a recompile.

My `selloop.rex` is not the report's program (mine takes the second `WHEN`), so I record it rather than compare: base 3,181,971,239 -> here 3,200,371,341, +18,400,102 over 400k passes with three promoted clauses, 15 per clause, consistent with `ifloop`'s 17.

### Exit criteria

* **The `run.rs` unit tests on the IR arm.** `Interp::new`'s `engine` flipped to `Engine::Ir` in place: **539 passed, 0 failed**, matching the report. I then checked the flip is not a no-op, because a measurement that cannot fail reads like one that passed: with the flip in place and one `return Err(...)` added to the driver's `Generic` arm, 335 passed and **204 failed**. The flip reaches them.
* **The trace oracle on both arms.** All fifteen witnesses -- the thirteen `.rex` in `tests/trace_oracle/` plus `corpus/lang/trace_output.rex` and `corpus/lang/pull_queue.rex` -- run under both `REXX_ENGINE` values: stdout, stderr and exit status byte-identical on all fifteen.
* **`KNOWN_DIVERGENCES` untouched** and its pin green.

### The `Clause` / `TraceClause` split

The split does what the interface said it must.
`Op::Clause` is unconditional in the stream -- `compile` emits it for every promoted instruction regardless of `trace`, which the untraced goldens show -- and `run_clause_region` always enters `in_stepped_clause_with`, which always invalidates the clock, decays the trace entry, sets `current_value_indent`, resolves the clause line and opens `in_clause`.
`Echo` is consumed at exactly one point inside the `in_clause` closure and decides only whether `echo_stepped_clause` runs.
Nothing about `SIGL`, the condition boundary or the failure site is reachable from the echo decision.
Confirmed behaviourally too: a promoted `IF` raising 43.1 under an untraced chunk reports the same line, the same message and the same `SIGL` on both engines and on the oracle, with the setting changed on either side of it.

The one thing worth naming, which the report names first: the compiled echo reads `clause_state.current_value_indent` rather than recomputing `printed_indent`, and that is an ordering invariant rather than a type one.
I checked the window -- `in_clause` does not write the field, and `TraceClause` is the region's first op -- so it holds today, and N6 makes it observable.
The report says nothing makes it structural, and that is accurate.

### `echo_compiled_clause` versus `echo_stepped_clause`

Byte-identical by construction, which I checked rather than assumed: `trace_stepped_clause` is `tracing_clause` plus `push_clause`, and `echo_compiled_clause` is `clause_site` plus the same `push_clause` with the same arguments.
Only the question of *whether* differs, as its doc says.
