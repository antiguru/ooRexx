# Review of Task 3: the driver, engine selection, and the dual-engine harness

BASE `09cba5a8`, commit `533c48a5`.
Reviewed against `task-3-brief.md` and `task-3-report.md`.
Every mutation below was applied to a `cp` backup, run with `cargo test -p rexx-exec --no-fail-fast`, restored from that backup, and verified with `sha256sum -c` against a manifest of the repository paths.
No `git checkout --` was used, no commit was made, and `git status --porcelain` is empty at the end of this review.

## Verdicts

* **Spec compliance: pass with one gap.** Every interface the brief names exists with the signature it names, the four controller-resolved ambiguities are all followed, and the standing dead-code obligation is discharged with the one exception named and justified. The gap is Step 4's second sentence: "**No `Clause` op precedes a `Generic` op** ... Assert it in the compiler, not only in prose." No such assertion exists in `compile`, and the report does not say it was skipped or why.
* **Task quality: good.** The driver's three mechanics are intact, the deviation is sound and strictly stronger than what the brief asked for, and the report's measurements reproduce exactly -- every count I re-ran matched to the test. Two test-quality defects were found by mutation, one of them in a test whose own doc comment claims the property it does not hold.

## Findings

### Important

**I1. `every_population_is_present_and_non_empty` does not hold the property its doc comment claims, and half the sweep can be deleted silently.**

The test compares `POPULATION_NAMES` against the names `populations()` returns.
Both live in `ir_dual.rs`, so an edit that removes a population *and* its name satisfies the assertion.
Measured (M8'): deleting the `bif` population from the builder and `"bif"` from `POPULATION_NAMES` leaves all four `ir_dual` tests green, and the sweep runs 5206 programs instead of 10391.
The doc comment states the opposite: "The names are pinned so a population deleted from the builder is red rather than a quietly smaller sweep."

This is the exact "shrunken denominator" hazard Step 7 names, and it is live for 3 of the 4 populations -- 10340 of 10391 cases.
The file gets the same problem right one screen earlier: `phase_subset_files_on_disk` reads the corpus directory rather than a second copy of the list, and its own doc says why ("so the assertion cannot be satisfied by a copy of `SUBSET_FILES` edited in the same change").
The population half has no such external referent, and `ootest/ooRexx/base/` is one -- it is outside this repository, read-only, and already read by `suite_sources`.

Fix direction: derive the three ootest population names from the suite directories the extractors are pointed at, or pin each population's case count against the group count `find_test_groups` returns, so a population that stops extracting is red on an external fact rather than on an in-repo copy.

**I2. The two counter-delta tests are structurally incompatible with Task 11's default flip, and the `Mutex` does not protect them.**

`RUN_CHUNK_ENTRIES` is a process-wide atomic and the tests read a delta across a `run_program` call.
The module `Mutex` serialises this module against itself, which is not the exposure: the contamination comes from *other* unit-test modules in the same test binary running programs concurrently, and those never take the lock.

Measured, and the numbers are the point.
Under M11 (selection widened to `Engine::Ir | Engine::TreeWalker`, so every run drives chunks) the two tests read **7** and **28** where they assert 3 and 0.
That is precisely the world Task 11 creates by flipping the default: every unit test in `rexx-exec` that runs a program becomes a chunk driver, and the delta stops meaning "this run's work".

The report states the concern ("a later task that runs a program under `Engine::Ir` from another unit-test module has to take the same lock, and nothing enforces that") but does not connect it to the default flip the brief announces two tasks ahead, which is the case that arrives by *not* adding anything.
`the_tree_walker_drives_no_chunk_at_all` will at least go red loudly, since it names the default; `the_ir_engine_drives_every_body_the_program_enters` will go red with a number that reads like a driver defect.

Fix direction: the counter does not need to be global. `Outcome::chunks_refused` is already a per-run diagnostic filled from an `Interp` field; a `chunks_driven` beside it makes the delta a property of one `run_program` call, removes the static, the `Mutex` and the discipline nothing enforces, and is the same shape the file already uses for the refusal counter.

**I3. Step 4's compiler-side assertion was not written and not reported.**

The brief: "**No `Clause` op precedes a `Generic` op**, because `step_in_temps_frame` already echoes and the echo is not idempotent. Assert it in the compiler, not only in prose."
`compile` contains no assertion; `drive.rs`'s `Op::Generic` arm states the rule in a comment.
Nothing in `golden_tests.rs` covers it either.

The assertion is vacuously true today, which is the argument for writing it now rather than against it: its value is as a tripwire for Task 4, the first `Op::Clause` emitter, and for the six promotion tasks after it.
The report's dead-code section shows the right discipline for an obligation that cannot be met ("if one cannot be removed, say why in your report rather than leaving it") and that discipline was not applied here -- the obligation is simply absent from the report.

### Minor

**M1. `chunks_refused` counts refusal *events*, not bodies, and both its doc comments say bodies.**

`chunk_for` has no negative cache: `Err(_) => { self.chunks_refused += 1; None }` inserts nothing, so a refused body is recompiled and recounted on every activation entry.
`Outcome::chunks_refused` says "How many bodies the run declined to compile" and `Interp::chunks_refused` says "How many bodies `chunk_for` has refused".
A routine called in a loop would give the call count, not 1.
Unreachable today (the only refusal is a `u32::MAX` op stream), so this is a doc correction plus a note that the fallback path recompiles per entry.

**M2. The report's `#[derive(Default)]` justification is false.**

"`#[derive(Default)]` on the enum names the same arm a third time so no constructor can disagree."
Nothing calls `Engine::default()` or `Default::default()` for `Engine` anywhere in the workspace, so the derive constrains no constructor.
The claim is in the report only; the code carries no such comment. The derive itself is harmless on a public enum.

**M3. Two doc comments in `ir_dual.rs` name a test in `corpus.rs` that does not exist.**

`the_dual_harness_reads_every_phase_subset_file`'s doc says "`corpus.rs`'s own test of this name has the argument", and the module doc says "the shape `corpus.rs`'s own test of the same name uses".
The `corpus.rs` test is `the_differential_reads_every_phase_subset_file`; `coverage.rs` has a third under `the_union_reads_every_phase_subset_file`.
The *shape* claim is right and the `SUBSET_FILES` constants do match across all three files; only "of this name" is wrong.

**M4. Two comments count `run_activation`'s production call sites.**

`run.rs`'s selection comment ("both production entry points -- `Interp::run` and `resolve_and_run_call`") and `drive/tests.rs`'s `THREE_BODIES` doc ("The three are the two production paths into `run_activation`").
This is the shape `rust/CLAUDE.md` forbids in prose -- "It may not say how many call sites there are" -- and asks be asserted instead.
Both enumerate by name, which makes them checkable, and the count of three is asserted for its *consequence*; nothing pins "there are exactly two call sites".
Lowest severity because the enumeration is present and the brief itself asserts the same fact.

**M5. `run_chunk`'s doc states as present tense a property that is not yet true.**

"A clause spans a run of ops, and a flat stream has no scope to hang that on."
Today one clause is exactly one `Op::Generic`, and `in_clause` lives entirely inside `step_in_temps_frame` (`run.rs`), i.e. inside that single op -- so the two-level split is anticipation of `Op::Clause`, not a present requirement.
The `run_chunk` / `run_chunk_clauses` split *is* load-bearing today (the register truncation covers early returns), and that one is stated correctly.
The wording is the brief's own, which is why this is Minor rather than a correction request against the implementer.

## Observation, not a finding

The first `REXX_*_GATE=1 cargo test --workspace` run I made failed one test: `every_state_builtin_case_matches_the_oracle_except_the_declared_gaps`, on the case `queued_null_line` (`queue ''; say queued() '['queued()']'`).
The same binary passes in isolation, and a second full workspace run under the same four gates passed at 1347/0/4.
The mechanism is almost certainly the oracle's shared session data queue seen by concurrently spawned oracle processes, and this task's code is not on that path -- `state_builtin_oracle.rs` never touches `Engine` and runs on the default tree-walker.
It is recorded because the new `ir_dual` binary adds roughly 8 seconds of concurrent load to every workspace run and so may raise the flake's frequency.
**I did not establish that this test flakes at BASE**, so it is not attributed to this commit.

## The `program_id` widening: every construction site, individually

Eleven sites construct an `Activation`. All eleven are correct.

Three are production:

* `lib.rs:1907` (`Interp::run`) -- `program_id` is taken as `ProgramId(self.programs.len())` before the `push`, and the *same* local feeds `install_directives`, the `BodyKey` passed to `plan_for`, and `Activation::new`. Correct, and the rename from `id` removes the shadowing that made the old code hard to check.
* `run.rs:3783` (`Activation::nested`) -- `program_id` is read from the caller's activation at the same point `program` and `selector` are, so all three describe one frame. An internal `CALL` runs the caller's own program and body selector. Correct.
* `run.rs:3827` (`Activation::routine`) -- receives `installed.program`, the same `ProgramId` that indexes `self.programs` to produce `routine_program` and the same one used in the `BodyKey` passed to `plan_for` three lines above. Correct.

Eight are test helpers -- `queue.rs:261`, `eval.rs:1084`, `trace.rs:1129`, `stem.rs:548`, `run.rs:7839`, `builtin/numeric.rs:729`, `builtin/convert.rs:1049`, `plan.rs:747`.
Every one computes `ProgramId(interp.programs.len())` before its `programs.push` and passes that same value both to `plan_for`'s `BodyKey` and to `Activation::new`.
(The report's "Files changed" list names seven of these; `run.rs`'s own helper is covered by the `run.rs` bullet, which mentions only that file's production edits. Cosmetic.)

Nothing reassigns `Activation::program`, `program_id` or `body` after construction -- checked by grep across the workspace -- so `body_key()` cannot drift from the key the plan was cached under.

Two further checks:

* **Empirical, positive.** I added a temporary accessor on `Chunk` and asserted in `run_activation` that `chunk.op_of.len() == body.instructions.len() + 1` on every chunk selection. All four `ir_dual` tests pass with it, i.e. across 10391 programs on the IR arm no chunk cache hit ever returned a chunk for a body of a different length. Removed afterwards; sums verified.
* **The class is currently unreachable, which bounds how much the above proves.** `Interp::programs` gets exactly one entry in production (`lib.rs:1880` is the only non-test `programs.push`; `install_directives` records routine names and loads nothing). So `ProgramId` is always `ProgramId(0)` and `BodyKey` is discriminated by `directive` alone. Measured (M12): replacing the `::ROUTINE` site's `installed.program` with `ProgramId(999)` leaves the whole crate green at 653/0, because a constant wrong id is still injective when there is one program.

So: the widening is correct as written, and no test in the tree would catch a wrong constant at any of the three production sites.
That is not a defect today -- no collision is constructible -- but the plan/chunk key agreement is the thing worth a cheap pin (a `debug_assert` that `plan_for(self.activation().body_key(), ...)` is `Rc::ptr_eq` with the activation's own plan would do it) before anything can load a second program.

## The deviation: engine selection inside `run_activation`

**Verdict: sound, and strictly stronger than what the brief asked for. Accept it.**

The brief's property is that a callee must not tree-walk its whole body while the gate asserts less than it reads.
`run_activation` is the single function that runs an activation's body, and both production entry points -- `Interp::run` and `resolve_and_run_call` -- pass through it.
One decision inside it therefore covers both by construction, where two decisions at the callers is a discipline that can be broken by adding a third caller.
The implementer's second argument also holds: `Code` is built from the activation's program, plan and body selector at the top of `run_activation`, so a caller-side decision would rebuild it twice.

The cited test witnesses the property, and I confirmed that by mutating in both directions rather than one:

| mutation | what it models | result |
| --- | --- | --- |
| `&& std::hint::black_box(false)` | selection deleted outright | RED, this test alone (652/1) |
| `&& self.activations.len() == 1` | selection at `Interp::run` only | RED, this test alone (652/1) |
| `&& self.activations.len() > 1` | selection at `resolve_and_run_call` only | RED, this test alone (652/1) |

The third is mine, not the report's, and it is the one that closes the argument: the count of three distinguishes *both* one-sided placements, not just the caller-side one the report tried.

The residual cost of the deviation is that `chunk_for`'s `HashMap` lookup now runs on every activation entry under `Engine::Ir`, and a `matches!` on every entry under the tree-walker. Negligible, and correct to note on a performance phase.

## The vacuity measurement: confirmed

The report claims that deleting engine selection leaves all four `ir_dual` tests green.
**Confirmed.** With `&& std::hint::black_box(false)` on the condition in `run_activation`, `cargo test -p rexx-exec --test ir_dual` reports `4 passed; 0 failed`, including `both_engines_agree_across_every_population` over its whole population, and the whole-crate run is 652/1 with only `the_ir_engine_drives_every_body_the_program_enters` red.

This is not a defect: every op is `Op::Generic`, which delegates to the tree-walker's own clause unit, so the two arms are the same code and agree by construction.
It does decide what the sweep is worth, and the honest statement of it is in the tree rather than in a chat message -- three places, in fact: `ir_dual.rs`'s module doc under "What this proves, and what it does not", `drive/tests.rs`'s module doc, and the report's own section, which cites the mutation.
That obligation is discharged.

What the sweep does add, measured rather than argued, is M3 and M4 below: two per-clause obligations the driver discharges itself, each caught by the sweep and by nothing else in the crate.

## Test quality: the seven tests, by mutation

Unmutated crate total: **653 passed, 0 failed**.
All twelve mutations below were run against the whole crate with `--no-fail-fast`.

| test | mutation | result | unique? |
| --- | --- | --- | --- |
| `the_ir_engine_drives_every_body_the_program_enters` | M1 selection deleted | RED 652/1 | yes |
| | M2 selection at `Interp::run` only | RED 652/1 | yes |
| | M2b selection at `resolve_and_run_call` only (mine) | RED 652/1 | yes |
| `the_tree_walker_drives_no_chunk_at_all` | M11 selection widened to both arms | RED 651/2, with its sibling | no |
| `no_body_is_refused_by_either_engine` | M7 `chunk_for` refuses every body | RED 649/4 | no |
| `both_engines_agree_on_an_all_generic_program` | M6 driver clause loop skipped | RED 650/3 | no |
| `the_dual_harness_reads_every_phase_subset_file` | M9 `phase-4c.txt` dropped from `SUBSET_FILES` | RED 652/1 | yes |
| `every_population_is_present_and_non_empty` | M8 `bif` population dropped from the builder | RED 652/1 | yes |
| | M8' `bif` dropped from the builder **and** from `POPULATION_NAMES` (mine) | **GREEN** | -- see I1 |
| `both_engines_agree_across_every_population` | M3 `grant_procedure_permission` removed from the driver loop | RED 652/1 | yes |
| | M4 trap offer replaced by `return Err(failure)` | RED 652/1 | yes |
| | M6, M7 | RED with others | no |

Every count the report gives reproduced exactly, including which other tests came along.
Two additions of mine: M2b, which strengthens the deviation's witness, and M8', which is finding I1.

The eighth test's deletion was the right call, and M10 is the measurement that says so: deleting `self.roots.pop_frame(registers)` from `run_chunk` leaves the crate at **653/0**.
`chunk.registers` is `0`, `reserve_temps(0)` resizes nothing, and `pop_frame` truncates to a length that has not moved -- so a test asserting the temporaries stack balances across a chunk could not fail, whatever it asserted.
Deleting it rather than shipping it is correct, and recording the measurement rather than the reasoning is what makes the judgment checkable.

M5 could not be applied, as reported: `Op::Generic => self.step(...)` gives `error[E0624]: method 'step' is private`.
Confirmed by running it.
`step` is a private `fn` of the `run` module and `ir::drive` is not a descendant, so the delegation the brief warns about is excluded by the module system rather than by a comment.

## The three mechanics, and the trap offer

* **Two levels because `in_clause` is a scoped closure.** Present: `run_chunk_clauses` iterates clauses, `run_clause_ops` runs one clause's ops. The split that is load-bearing *today* is the other one -- `run_chunk` reserves and truncates the register region around `run_chunk_clauses`, so an early return from the loop cannot skip the truncation. See M5 above for the wording quibble.
* **`pc` stays an instruction index, so `apply_flow` is reused unchanged.** Confirmed. `run_chunk_clauses` reads `self.activation().pc` as an instruction index, indexes `code.body.instructions` with it, and maps it through `chunk.op_of` only to find the clause's first op. `apply_flow` is called unmodified and writes instruction indices back. The `INTERPRET` case that would break this does not arise: `run_fragment` builds its own `Code` and calls `run_bounded`, never `run_activation`, so the driver's outer loop only ever walks the activation's own body -- which is what makes `Flow::Signal`'s instruction target and `chunk.op_of` describe the same body.
* **`Generic` delegates to `step_in_temps_frame`, not `step`.** Confirmed, and enforced by privacy (M5).
* **The trap offer is at its call site, one per activation.** `self.offer_to_trap(code, failure)?` sits between the clause and `apply_flow`, the same position it holds in `run_activation`'s loop. M4 (replacing it with a bare re-throw) is red on the sweep and on nothing else, so the position is measured rather than asserted.

## Ordinary checks

* No `unsafe` anywhere in the diff; the workspace forbids it.
* No em-dashes in any added comment (`/bin/grep` over the added lines of the whole diff: none). The one `…` is in a format string in `excerpt`, not a comment.
* Doc comments state contracts; reasoning sits at decision points -- the engine-selection block in `run_activation` and the `Op::Generic` arm in `drive.rs` are both good examples.
* Counts of mutable in-repo aggregates in prose: two, both call-site counts, both enumerated by name. See M4.
* `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets -- -D warnings` exit 0 (warm target; the report already flags that a same-session green is provisional and asks for a clean-target re-run at the phase gate -- that flag stands).
* `REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1 REXX_BIF_GATE=1 REXX_KEYWORD_GATE=1 cargo test --workspace --no-fail-fast`: exit 0, **1347 passed, 0 failed, 4 ignored**, with all four gates reporting `mode: STRICT`. Matches the report.
* `cargo test -p rexx-exec --no-fail-fast`: 653 passed, 0 failed. Matches the report.
* The dead-code obligation is discharged: `chunk_for`, `compile`, `Chunk::ops`/`op_of`/`registers`, `ChunkTooLarge`'s struct-level annotation and `Interp::chunks`/`chunks_refused` all have their `#[allow]` removed and a production reader. The one retained annotation is narrowed to `ChunkTooLarge::what` with the reason stated where it sits, and the alternatives considered are named. That is exactly what the brief asked for.
* `Op::Clause` and `Op::EvalExpr` keep their `#[expect(dead_code)]`; the driver's `match` arms do not unfulfil them, which the clean `-D warnings` run confirms.
