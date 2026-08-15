# Review: Task 4c, flatten the loop header (`de05ea58..4d8e913a`)

**Spec compliance: pass.**
**Task quality: pass, with one Important finding and six Minor.**

The range reviewed is `de05ea58..4d8e913a`.
The checkout has since gained `bc231a4b` and `68f29913`, both documents only, and the second one corrects the cause of the noise Task 4c measured -- see m3b.
Nothing in this review is affected, because every reading here is between two arms of one binary, interleaved.

Everything below was run rather than read.
Every build was made in a `git worktree` under the session scratchpad, never in the repository checkout, and each worktree's `git log` was read before it was built.
No tracked file in the repository was modified by this review.

## Question 1: which reading of criterion 4 holds, measured within one binary

**Criterion 4 fails on `emptyloop` at head, and it also fails at Task 4b-M's own commit.**
The two figures do not describe the same tree only because one of them does not reproduce: on this instrument the IR arm is slower than the tree-walker arm at `1535b030`, in two independently built binaries, on two instruments.
So nothing broke between 4b-M and now in the sense of crossing the line -- the line was already crossed at 4b-M's own commit, and 4b-M's `0.9964` is the outlier.

Method, per the amendment: both arms of **one** binary, selected with `REXX_ENGINE`, interleaved within one sitting, rounds outermost so machine drift hits every cell.
`ulimit -v 8388608`, one program per invocation, from one fresh empty directory, warm-up round discarded, medians of nine pairs.
Wall clock is `perf`'s own `duration_time`; the cross-check below uses 4b-M's instrument (`/usr/bin/time -f %e`, no `perf`) so that the wrapper is not the explanation.
No cross-binary wall-clock or cycle comparison is made anywhere in this section.

**Every wall-clock claim here is a paired one, which is what survives `68f29913`'s correction.**
That commit shows the one-comment 7.8% is run-to-run variance rather than codegen, and therefore bounds repeated runs of a single binary rather than only comparisons between two.
A median would not survive that; the pair count does, because the two arms of a pair run seconds apart inside one round and the answer is the same in every round.
The reading is a sign test over rounds, not a difference of medians, and the medians are quoted only to say how large the effect is.

### At head, `4d8e913a`

| quantity | tree-walker | IR | IR/TW | IR faster in |
|---|---:|---:|---:|---|
| `instructions:u` | 37.8507e9 | 40.3257e9 | **1.0654** | -- |
| `cycles:u` | -- | -- | **1.0236** | -- |
| wall, build A | 2.895 s | 2.964 s | **1.0237** | **0 of 9** |
| wall, build B (same source, different path) | 2.850 s | 2.976 s | **1.0440** | **0 of 9** |
| wall, `/usr/bin/time -f %e`, no `perf` | 2.89 s | 2.95 s | **1.021** | **0 of 7** |

The tree-walker figures reproduce Task 4c's own head numbers exactly (37.8507e9 / 40.3257e9), so this is the same instrument reading the same tree.

### At `1535b030`, which is Task 4b-M's own "fixed" tree

My build of `1535b030` reproduces 4b-M's recorded instruction counts (38.0007e9 / 39.5757e9 against its 38,000,658,748 / 39,575,660,653), so it is that binary semantically.

| quantity | tree-walker | IR | IR/TW | IR faster in |
|---|---:|---:|---:|---|
| `instructions:u` (both builds) | 38.0007e9 | 39.5757e9 | **1.0414** | -- |
| wall, build A | 2.762 s | 2.845 s | **1.0299** | **0 of 9** |
| wall, build B | 2.890 s | 2.966 s | **1.0265** | **0 of 9** |
| wall, `/usr/bin/time -f %e`, no `perf` | 2.77 s | 2.85 s | **1.029** | **1 of 7** |

That single pair is the one round whose tree-walker arm read 3.04 s where its other six read 2.75 to 2.78 -- the heavy tail 4b-M documented at about one run in thirty and this sitting saw once in seven.
It is the only pair in 34 measured pairs at this commit in which the IR arm was faster.

4b-M recorded `2.790` against `2.780`, IR faster in 5 of 9 pairs.
I read the IR arm 2.7% to 3.0% slower, 0 of 9 pairs under `perf`, in two builds and on 4b-M's own instrument.
The gap is seven times the granularity of that instrument on a 2.8 s program, so it is not a rounding of my figures, and the tree-walker arm agrees with 4b-M's to within 1%: it is the **IR** arm's reading that differs.
Nothing here explains what 4b-M measured; what it establishes is that the reading does not survive re-measurement at its own commit.

`varlookup` at head, same method, five pairs: instructions 71.6307e9 against 73.9297e9 (ratio **1.0321**, which is Task 4c's own figure), cycles 1.0523, wall 5.050 s against 5.308 s (**1.0511**), IR faster in 0 of 5.
It fails too, as the plan already records.

### What moved, on instruction counts

Instruction counts reproduce to 1e-8 across builds of one commit, so the IR-against-tree-walker gap can be tracked across commits without a same-source control build beside it.
All figures are within-binary, `emptyloop`, 25e6 passes, medians of three pairs.

| commit | tree-walker | IR | IR/TW | IR - TW, per pass |
|---|---:|---:|---:|---:|
| `1535b030` (4b-M's head) | 38.0007e9 | 39.5757e9 | 1.0414 | 63 |
| `37dacefc` | 38.0007e9 | 39.5757e9 | 1.0414 | 63 |
| `67f37050` | 38.0007e9 | 39.5757e9 | 1.0414 | 63 |
| `75b4e770` (the frame stack) | 38.0007e9 | 41.8257e9 | **1.1007** | **153** |
| `b52baa2b` (4b' head) | 38.0007e9 | 40.4007e9 | 1.0632 | 96 |
| `9572f527` (Task 6 head) | 38.0007e9 | 40.4007e9 | 1.0632 | 96 |
| `de05ea58` (4c's BASE) | 38.0007e9 | 40.4007e9 | 1.0632 | 96 |
| `4d8e913a` (head) | 37.8507e9 | 40.3257e9 | 1.0654 | 99 |

* **Task 4b' owns +33 instructions per pass** of the IR arm's excess (63 to 96), and the tree-walker arm does not move at all across those commits.
  The frame-stack commit itself is +90 and `b52baa2b`, its own follow-up, gives 57 back.
* **Task 6 costs exactly zero on this axis**: both arms read the same at `b52baa2b` and at `9572f527`, to 1e-8.
  The `+0.25%` the brief and the anchor offer as a candidate is not one here, whatever it measured elsewhere.
* **Task 4c adds +3 per pass** to the gap (96 to 99, +0.22pp of ratio): it saves 6 per pass on the tree-walker arm and 3 on the IR arm, so the arms move by different amounts and the ratio worsens slightly.

So the honest statement is that the criterion's quantity was already failing on this axis at 4b-M's commit, and the instruction-level gap behind it grew by 2.18 percentage points at Task 4b' with no task recording it.
Task 4c's own contribution is +0.22pp.
I did not attribute the wall-clock and cycle deltas to any commit and no such attribution is offered.

## Question 2: the refused-loop defect, its fix, and whether the class is closed

**The defect was real and is exactly as described.**
Built `08137f3a` and ran the three programs on both arms:

| program | `08137f3a` tree-walker | `08137f3a` IR | head, both arms |
|---|---|---|---|
| `do with index i over 5` | rc 120 | **rc 101**, `unreachable!` at `run.rs:5604` | rc 120 |
| `do counter c i = 1 to 2` | rc 120 | **rc 101**, `a controlled loop's plan always names its initial value` | rc 120 |
| `do qq over a.` | rc 120 | **rc 101**, `a DO OVER's plan always names its target` | rc 120 |

**The fix holds and the new rows are the only thing in the workspace that catch it.**
`M5b` -- the refusal moved back into `run_loop` alone, which is the arrangement `08137f3a` shipped -- turns exactly one test red, `both_engines_agree_on_every_case_file`.
With `loop-header-boundaries` and `loop-refusals` moved **out of the case directory**, the same mutation is fully green: 1382 passed, 0 failed.
That control had to be built twice: renaming a case file in place does not remove it, because `datadriven::walk` reads every file in the directory whatever its name.

**`COUNTER`, `DO WITH` and a stem `OVER` target are the only refusals**, and the guard is now the enumeration itself rather than a copy of it: `run_loop_with_header` refuses when `loop_header_plan(body).is_none()`, and `loop_header_plan` is the one function that answers `None`, for those three reasons only.
A refusal added there in future refuses on both arms without anything else being edited, which is the property the shipped defect lacked.

**The class -- a construct the tree-walker refuses but the compiled stream tries to run -- is closed for the constructs that are promoted.**
Only `If`, `Select` and `Do`/`Loop` compile to anything but `Op::Generic`, and a `Generic` reaches `step`, so its refusal is the tree-walker's own by construction.
Neither `If` nor `Select` has a refusal on its path: the three `Loud::instruction` sites in `step` are `Call::Qualified`, `Address` with a command or redirection, and the catch-all, all of which are `Generic` instructions.
The residual is that `run_loop_with_header` still holds one `unreachable!` and two `expect`s that the guard above them is what makes unreachable -- a prose guard rather than a type-level one -- but it is now a guard both engines pass through, and the enumeration it consults is the same one the compiler emits from.

## Question 3: extraction, not a second implementation

The five `LoopKind` arms, the `WHILE`/`UNTIL` conditional, the `LEAVE`/`ITERATE` search and every per-pass trace echo are in `run_loop_with_header` and `run_repeating`, entered by both engines; `Op::LoopRun` differs from the tree-walker's entry in one argument, the `BodyEngine`.
The header's order is `loop_header_plan`, iterated by `eval_loop_header` and emitted from by `compile`; each value's validation is `accept_header_value` and each echo is `echo_header_value`, both entered from the tree-walker's loop and from an op.
`setup_controlled` and `step_in_temps_frame_with` are gone, and `step` no longer takes a `BodyEngine` at all, which makes "anything reaching `step` is stepped by the tree-walker" a fact about the signature rather than a claim about call sites.
`M7` -- `Op::LoopRun` passing `BodyEngine::TreeWalker` -- turns both clause-count tests red, so the promotion still has its witness.

The one duplicated encoding is inside `compile`'s own arm: the region's length is summed from `2 + u32::from(role.keyword().is_some())` per role, independently of the loop that emits those two or three ops.
A change to the group's shape has to be made in both places.
It is caught -- `M3` leaves the `end` one op too long and five golden tests go red -- and it is small, but it is a second statement of the same rule, so it is listed as Minor below.

## Question 4: boundaries the new table does not cover

I built seven probes around loop-header boundaries the eleven rows do not reach and ran each under the oracle and under both arms, from a fresh empty directory per run, with stdout, stderr and status kept apart.
The programs live outside the scratchpad root, because the root is on the oracle's external-routine search path.

**Nothing new is introduced by the flattening.**
Five probes are byte-identical across the oracle and both arms: a `FOR` count's own `>K>` echo under `trace r`; a bare `DO 2`'s repeat count echoed under the `FOR` tag; a `DO OVER ... FOR` whose count queues a handler; a nested loop whose inner header queues a handler on both outer passes; and a `WHILE` re-test reached through an `ITERATE`, whose handler reports the `ITERATE`'s own line.

**Two probes diverge from the oracle, and both diverge identically on both arms and at `de05ea58` and `1535b030` as well as at head.**

* A **zero-pass** counted loop whose `TO` expression queues a handler that queues a second condition: ours prints `G ran 3` then `after`, the oracle prints `after` then `G ran 6`.
  The second delivery lands at the loop's own per-pass header boundary, which reports the `DO`'s line, where the oracle waits for the next real clause.
* A `DO UNTIL` whose condition queues a handler that queues again: ours reports `G ran 6` (the `END`), the oracle reports `G ran 5` (the body clause).

Both are the family `task-4bp-report.md` catalogues -- our clause boundary has no equivalent of the oracle's synthetic end-of-construct instruction -- and `KNOWN_DIVERGENCES`' first row records the same shape one construct over, with the tree-walker's answer being exactly the shape ours takes here.
So they are pre-existing, they are engine-agreeing, the flattening did not move them, and no claim in the report is false because of them.
What is missing is that nothing records them: the two requeue rows the task chose both happen to land where we agree with the oracle, and the neighbouring shapes do not.
Listed as Minor, against the phase record rather than against this task.

## Question 5: the clause-count tripwire

**The derivation is sound.**
`count_clause_op_entry` fires on `Op::Generic` and `Op::Clause`, which are the two ops that open a clause of an instruction.
A three-pass counted loop over a one-instruction body emits one `Clause` for the `DO` and one `Generic` for the body instruction; the `Clause` op is reached once, because the loop's repetition happens inside `Op::LoopRun` rather than by re-entering the region, and the body's `Generic` is reached once per pass.
1 + 3 = 4, and before the change the `DO` contributed one `Op::Loop` entry in place of that one `Op::Clause` entry.
The block's 1 + 2 = 3 follows the same way.
The per-pass header re-evaluation is `run_repeating`'s own `in_clause`, which opens no clause of an instruction and calls nothing that counts.

The doc comment does name the change that would move the number -- "the loop's per-pass header becoming a region of its own" -- and `M7` shows the numbers are still load-bearing.

## Question 6: mutation replay and the `REWRITE` refusal

Every mutation was applied in a worktree, run with `cargo test --workspace --no-fail-fast`, restored from a `cp` backup, checked against the committed blob's `sha256`, and the worktree rebuilt afterwards.
Baseline in that worktree: **1382 passed, 0 failed, 4 ignored**, matching the report; `de05ea58` reads **1378**, also matching.

| mutation | red here | report | agrees |
|---|---|---|---|
| M1, defer every `LoopHeaderValue` past every `EvalExpr` | case file + **5** golden | case file + 4 golden | count differs |
| M2, echo after validate | case file + 4 golden | same | yes |
| M3, drop the `TraceKeyword` op | 3 dual-engine sweeps + 5 golden + 1 clause-count test (9) | same set | yes |
| M4, release the header's registers at the region's end | the nested-loop golden test alone | same | yes |
| M5, no refusal in either engine's path | 4 pre-existing `run::tests` + case file | same | yes |
| M5b, refusal in the tree-walker's entry only | case file alone | same | yes |
| M6b, `release_to` keeps the highest mark | the two-constructs golden test alone | same | yes |
| M7, `Op::LoopRun` passes `BodyEngine::TreeWalker` | both clause-count tests | same | yes |

Adds-coverage controls, which is the step "can fail" does not cover:

* M1 with the two new case files out of the directory: only the five new golden tests are red, so **no pre-existing behavioural test catches it**.
* M1 with the case files out **and** the five new golden tests cut: 1377 passed, 0 failed -- fully green, so the new tests are the only catchers.
* M5b with the case files out: fully green, as above.

**`REWRITE` is refused.**
`REWRITE=1 cargo test -p rexx-exec --test ir_dual both_engines_agree_on_every_case_file` panics at `ir_dual.rs:875` with the message naming the oracle-capture command, before any file is touched, and the harness also asserts the directory produced at least one stanza so an emptied directory cannot pass vacuously.
Both new files carry a header saying what their bytes are; `loop-refusals` says plainly that its bytes are this crate's own refusal and not the oracle's, which is the right exception to state rather than to leave implicit.

## Independent checks of the report's other claims

* All eleven `loop-header-boundaries` rows re-run outside the harness: both engines agree at head, both agree at `de05ea58`, head equals base, and each row's rendered answer equals its expected block byte for byte.
  So the "would this row ever have failed" recovery is real, and the rows are a regression net rather than a self-consistency check -- M1, M2 and M5b turn them red.
* `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean from a fresh target directory; the four gated `ir_dual` runs pass; `cargo test --workspace --release` reads 1382 / 0 / 4.
* `4d8e913a` changes comments only and `38e3dc89` changes one document only, so the tree measured at `b6d54856` is the tree at head.

## Findings

### Important

**I1. The phase still records criterion 4 as passing on `emptyloop`, and it does not pass.**
`docs/superpowers/plans/2026-08-09-phase-4e-ir.md` asserts the pass twice, at the 4b-M summary and again in the 4b-M narrative, and Task 11's gate step reads those lines.
Within one binary the IR arm is slower on this axis at head (1.024 and 1.044 in two builds, 0 of 9 pairs each), and at `1535b030` itself (1.030 and 1.027, 0 of 9 each), on 4b-M's own instrument as well as under `perf`.
Task 4c saw the contradiction and recorded it honestly -- but it recorded it in its own "what I could not verify" and in the anchor, while the plan the next task reads still says "passes".
This project's own rule is that a correction goes into the plan rather than into the message carrying the work.
The correction should also carry the attribution, which is now measured: Task 4b' is +33 instructions per pass of the IR arm's excess on this axis, Task 6 is zero, Task 4c is +3.

### Minor

**m1. The M1 row of the report's mutation table undercounts its own catchers.**
It records "`loop-header-boundaries` + 4 golden"; the mutation turns five golden tests red, `two_constructs_ending_at_one_instruction_release_to_the_lower_mark` included.
The substantive claim of the row -- the case file is the only behavioural catcher -- is correct and now has its control.

**m2. A new comment claims what every other call site on the page does.**
`step`'s `Do`/`Loop` arm says "`BodyEngine::TreeWalker`, like every other `run_bounded` call on this page".
`rust/CLAUDE.md` forbids exactly that shape: a comment "may not say ... what every other site does", because the referent changes without the sentence being reread.
The sentence after it states the property instead, and that property is now enforced by `step`'s signature, so the first clause can simply go.

**m3. The anchor's Task 4c entry mis-cites 4b-M and points at a candidate that measures zero.**
"Task 4b-M recorded parity at `401e0df5` (IR 3.12x against 3.13x)" attaches the *fixed* tree's oracle ratios to `401e0df5`, which is 4b-M's base; and "Task 6 measured its own cost at +0.25%" is offered as part of the explanation, where Task 6's net cost on this axis is exactly zero in both arms.
Neither changes the entry's conclusion, which was that the question was open.
**m3b. The report and the anchor both attribute the one-comment 7.8% to code layout, and that attribution is refuted.**
`68f29913` -- landed after this review range, in the spec only -- built the same tree with and without one comment and found the two `.text` sections byte-identical, so the difference is run-to-run variance in the environment rather than codegen.
The report's "layout is worth about 8% there" and the anchor's "different layout" and "what layout is worth on this axis" are all the retired reading.
The conclusion each of them draws from it -- that a cross-binary wall-clock claim at that magnitude is worthless -- survives the correction and gets stronger, because the variance now bounds two runs of one binary as well.

**m4. Two oracle divergences at loop-header boundaries are recorded nowhere.**
A zero-pass loop and a `DO UNTIL`, each with a handler that queues a second condition, differ from the oracle in `SIGL` and in ordering, identically on both arms and unchanged since `1535b030`.
They belong to the catalogued end-of-construct family and are not this task's, but the new table's two requeue rows both land where we agree with the oracle, so the table reads as if the family did not reach here.

**m5. The region's length is a second encoding of the emission's shape.**
`compile` sums `2 + u32::from(role.keyword().is_some())` per role and then emits those ops in a separate loop; a change to the group has to be made twice.
Caught by the golden tests, and small, but it is the one place in this promotion where the same rule is written down twice.

**m6. `Op::TraceKeyword` builds the value's text before the `>K>` gate.**
`echo_header_value` calls `to_text(value).to_vec()` and only then `trace_keyword`, which returns immediately when `trace_mode().results` is false.
Both engines share the function so nothing diverges, and it is once per loop entry rather than per pass, but the allocation is unconditional where the line it feeds is not.

**m7. Not this task's, recorded because it was true throughout this review:** the repository checkout carried one uncommitted line in `rust/crates/rexx-exec/src/ir/drive.rs` -- `// layout probe: one comment line, no behaviour change` -- from another process.
I did not create it, touch it, or build from it; every measurement here came from a worktree at a named commit.

## What this review did not establish

* Why 4b-M's `emptyloop` wall figures read as they did.
  Two builds of that commit and two instruments say the IR arm is slower there; nothing here reconstructs the reading that said otherwise, and per the inadmissibility rule no cross-binary comparison was attempted to try.
* The mechanism of Task 4c's own 6-instructions-per-clause saving.
  I reproduced the numbers exactly and did not chase the cause; the report's own candidate is refuted by its own control.
* Whether the header registers' rooting is load-bearing.
  Nothing collects yet, so M4 is caught only by a golden test on register indices, exactly as the report says.
