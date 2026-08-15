# Phase 4e final review -- Task 11's diff, and cross-task coherence

Reviewed at `e303e7a2`, working tree clean before and after.
Every build and every mutation ran in a private worktree under the session scratchpad, removed afterwards.

**Verdict: fit to close and hand to 4f.**
No behavioural defect was found, the flip is complete, and every number I checked in the gate document reproduces from `rust/bench-baselines/phase-4e-arms.tsv`.
What is wrong is prose, in nine places, and one of those nine is a forward hazard rather than a wording slip: a harness that silently changed which engine it measures.

## Scope 1: Task 11's own diff

### The flip is complete, and it is verified by running rather than by reading

The gate's criterion-1 claim is that three places name an engine, two were flipped, and `Interp::new` staying at `TreeWalker` is inert for production.
That claim holds, and it holds for a reason stronger than the one the gate gives.

`struct Interp` at `rust/crates/rexx-exec/src/lib.rs:1337` is **private**.
The crate's whole public surface for running a program is `run_program` (`:2395`) and `run_program_collect_every_alloc` (`:2428`), and both are one line each that calls the private free function `execute`.
`execute` writes `interp.engine = engine` at `:2521`, before `interp.run(program)` at `:2539`, with nothing between them that runs a body.
So no caller outside the crate can construct an `Interp` at all, and no caller inside it reaches a body without passing through that assignment.

I did not rest on that inference.
In a private worktree I put `eprintln!("PROBE-ENGINE {:?}", self.engine)` at `run.rs`'s engine-selection point and a second line on the fall-through past it, built `rexx-run`, and ran two programs from fresh empty directories covering the main body, an internal `CALL` label, a `::ROUTINE`, a `PROCEDURE` frame, a `SIGNAL ON NOVALUE` handler, a nested `DO`/`SELECT`, and an `INTERPRET` fragment.
Seven activations across the two programs, **every one `Ir`, zero fall-throughs**.
With `REXX_ENGINE=tree-walker` the same program reads `TreeWalker` and takes the fall-through.
Criterion 1 is not overstated.

`run_fragment` passes `BodyEngine::TreeWalker` unconditionally and enters no activation, which is the plan's own decision that a fragment compiles to no chunk; it is not a missed route.

### `6257baa4` reverses D22, and the reasoning is right while the sentence it left behind is wrong

The decision is sound and I reproduced the measurement it rests on.
At `5ab3028f`, `const _: () = assert_sync::<Chunk>();` fails with **exactly one** `E0277`, naming `Cell<Option<Resolved>>` in `CallSite`.
So the `AtomicU32` really did buy a property the struct around it did not have, and the change is correct.

Nothing depended on the patch table being `Sync`.
`rust/crates/rexx-exec/src/lib.rs:2448` is the only `Send` bound in the crate, on the closure `on_interpreter_thread` spawns; that closure captures a path, a text and an `Invocation`, and every `Chunk` is created and dropped inside the thread it runs on.
There is no `Sync` bound anywhere in `rexx-exec`.

**But the same commit falsified the sentence it wrote.**
At head the identical assertion fails with **two** errors, and the first one named is `Cell<u32>` in `PatchSlot`.
Three documents assert the one-culprit result in the present tense at head, and all three are now false:

* `rust/crates/rexx-exec/src/ir/mod.rs:837` -- "**This field is the whole of what stops a [`Chunk`] being `Sync`**", and "this is the type that would have to change first if a chunk ever crossed a thread". Both fields now stop it, and both would have to change.
* `docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md:218` -- "`assert_sync::<Chunk>()` fails naming `Cell<Option<Resolved>>` in `CallSite` and nothing else".
* `docs/superpowers/plans/phase-4e-gate.md:314` -- "fails **at head** naming exactly one culprit". The measurement belongs to `5ab3028f`; at the gate's own head it names two.

The fix is one clause in each: say the measurement was taken before the change, which is what makes it evidence for the change.
`rust/CLAUDE.md`'s rule is that a false comment is corrected or removed, not hedged, and this is the shape it names -- a correction round writing about the code's context from a line that cannot see it.

### `e303e7a2` as documentation

The two master-plan edits are exactly the ones the spec's `## Exit gate` calls for, at `:442` and `:473`, and the anchor's supersession note is scoped correctly ("and only in that role").
The floor's second copy is retired in place with its old wording quoted, which is the right treatment.

D22 itself is still self-contradictory after the amendment.
`docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md:477` opens "patch state lives in a parallel **atomic** table" and closes "at `Cell<u32>` (amended `6257baa4`)".
The amendment changed the tail of the bullet and left its head, which is the same failure the commit message describes one paragraph earlier.

## Scope 2: cross-task coherence

### 1. Three `Op`-adjacent doc comments describe a shape the tree left behind

All three are in `rust/crates/rexx-exec/src/ir/mod.rs`, and all three were true at Task 2 and false from Task 4a or Task 6 on.

* `:15` -- "`compile` walks a body once and emits **one `Op` per instruction**". A promoted instruction is a `Clause` region of many ops; `Op`'s own doc at `:66` says so, twenty lines below.
* `:924` -- `Chunk::ops`, "**One entry per instruction, in instruction order**". Same defect, and it reads as if copied from `op_of`'s doc at `:929`, where it is true and where the whole reason that map exists is that `ops` is *not* one entry per instruction.
* `:22` -- "`Interp::chunk_for` (beside `plan_for`, **under the same `BodyKey`**) is the cache: a body compiles **once, on first entry**". D23 widened the key at Task 6. `chunk_for`'s own doc in `plan.rs` calls the narrow key "a wrong-output defect, not a slow one", the field is `chunks: HashMap<(BodyKey, ChunkTrace), _>`, and `one_body_under_two_trace_settings_is_two_cached_chunks` pins two chunks for one body.

I checked the `Op` variants' own doc comments against `drive.rs` arm by arm and against `compile.rs`'s five `assert_*` functions.
Those are current: the enter/leave pair, the labelled `'region` block replacing the `?`, the no-`Generic`-inside-a-region rule, the "last op of the region" claims for `LoopRun` and `Call`, and the "only valid inside a `Clause` region" claims all match what the driver does.
The staleness is in the module header and one field, not in the variants.

### 2. Spec, plan, gate and code

Beyond the `Sync` and D22 items above:

* **The plan still mandates `AtomicU32`, in three places, unamended.** `docs/superpowers/plans/2026-08-09-phase-4e-ir.md:1024` ("Produces: ... `Chunk`'s parallel `Vec<AtomicU32>` patch table"), `:1030` ("patch state lives in the parallel atomic table"), `:1032` ("**`AtomicU32`, not `Cell<Op>`**, because ooRexx shares routine bodies across activities and `Cell` is not `Sync`. This survives Phase 6."). `:1034` hedges it and `:1111` records the decision to void it, but `:1032` states the rule flatly and is what a regenerated brief would carry. The spec was fixed at `e303e7a2` and the plan was not, which is the same one-of-two-copies pattern that commit is about.
* **The withdrawn floor has a third live copy.** `docs/superpowers/plans/2026-08-09-phase-4e-ir.md:1137`: "This plan's performance obligation is a floor, not a target." `:626` and `:953` both carry an explicit withdrawal marker; `:1137` carries none and reads as current. `emptyloop` ships at 1.09223, so this sentence, like the one `e303e7a2` retired, would have failed the phase the gate just closed.
* **Criterion 1 is worded against a structure the code no longer has.** The spec says the IR must be the default "at **both body-entry points**" (`:377`) and `## Body entry` (`:323`) names `resolve_and_run_call` and `Interp::run`. The code moved selection *inside* `run_activation` (`run.rs:1108`, with a comment explaining why), so there is one point rather than two, and it is strictly better -- a caller that was missed can no longer tree-walk a whole body. The spec was never amended and the gate answers a differently-worded criterion without saying so.

### 3. The gate's numbers

I checked every table in the gate against `rust/bench-baselines/phase-4e-arms.tsv` and they reproduce.
The head ratios (all seven axes, both instruments, `large` bound except `startup`'s `small`), the five-build `instructions:u` table (35 cells), the per-pass gap table (30 cells), criterion 5's five rows and its 157.75 / 48.73 / 16.11 / 11.00 / -4.00 differences, the 62045.85 tree-walker control across all four spike builds, the `Cell` price of 8 / 2 / 1 / 0, the `CALL_SITE_CACHE = false` reproduction, and Task 9's and Task 10's per-axis prediction figures all match the rows.

The `+22` and the `+21` in the same document are **not** an inconsistency and I checked this specifically: `+22` is the gap's movement across Tasks 8, 9 and 10 (1 + 13 + 8), `+21` is its movement since the split (13 + 8).

Two small imprecisions:

* `:101` -- "the two agree **exactly** on ... `alloc4c` (0.99682)". The rows are 0.996826 and 0.996825; `:207` already reports them as 0.99682 / 0.99683. They agree at the quoted precision, not exactly.
* `:105` -- "it read [0.89617..1.58778] in the five-build sitting". That is `5ab3028f`'s own range in that sitting. The five builds' range there is [0.71542..1.58778]. The point being made survives either way.

`strings` really does compile no arithmetic op, so `:212`'s "work held constant" demonstration stands: `total = total + length(joined)` has a call as its right operand, and `Op::Arith` requires both operands to compile native.

### 4. The two holes, and whether they are sized right

**Criterion 6 is sized right, and its second clause has the same weakness the gate flags only for criterion 7.**
The golden set bounds the hole as the gate says: a promotion that stopped firing for a covered shape reddens a golden test, and `arith_hint_skips` and `call_site_hits` back that for two ops.
But the gate says plainly that what falsifies the degenerate implementation is "the measurement rather than a test" -- which makes the second clause a build taken once, exactly like criterion 7's, and the gate hands 4f only the first clause's hole.
That is worth saying out loud in the handover rather than treating as discharged.

**Criterion 7 is not worse than the gate says. It is cheaper to close than the gate says, and that is a sizing error in the direction that gets work dropped.**
`:337` says the cheapest fix for `QUICKENING` and `CALL_SITE_CACHE` is a `#[cfg]`-gated second run, but that the third needs "a dual-engine helper in `run.rs`'s test module, which is a larger change because `Interp::new` has well over a hundred callers".
`Interp::new`'s engine is a literal in a constructor exactly as those two are literals in a `const`; the same `#[cfg]`-gated second run closes all three, and the gate's own table at `:331` already lists them as one shape.
No helper and no call-site edit is needed.

What the criterion-7 discharge does **not** show, and what I would want before calling it closed: `run_source` driving 1 chunk proves the constructor decides the engine, and a green 1423 proves nothing regressed -- but neither says how many of the exact-stderr trace tests actually ran a *promoted* clause under trace.
A build where they all fell to `Op::Generic` looks identical.
The phase already owns the instrument that settles it: read `trace_op_echoes()` across `run.rs`'s trace tests in that configuration, and a non-zero count proves the compiled echo path was exercised. One build.

### 5. The one finding that is not prose

**`rexx-bench-suite`, `rexx-bench-band` and `rexx-time` silently changed which engine they measure at `d9f68dd6`, and nothing records it.**
`rust/crates/rexx-bench/src/child.rs:75`, `Side::rust`, hands the child `env: Vec::new()`.
Those three binaries spawn `rexx-run` through it, and `rexx-run` with `REXX_ENGINE` unset now answers `Ir`.
`rexx-arms` is unaffected -- `arms.rs:697` sets the variable on every run, which is why the gate's own figures are safe and why `:102`'s "the default flip itself is measurement-neutral" is true *of `rexx-arms`*.

But `docs/superpowers/plans/phase-4e-anchor.md:138` records that its figures were "produced by `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`, run with no arguments", and the gate hands that document to 4f as "the last tree-walker-against-oracle figures".
`docs/superpowers/plans/perf-baseline.md` has the same provenance and is pinned by a test in `rexx-bench-suite.rs` itself.
So 4f's first re-run of the anchor's own command produces an **IR**-against-oracle number, compares it to a **tree-walker**-against-oracle baseline, and nothing in the tree or in the gate says the arms changed under it.
The cheapest fix is for `Side::rust` to name the arm explicitly and for the harness to print it in its provenance block, which is where every other identity in that document already lives.

### 6. Smaller coherence items

* `docs/superpowers/plans/phase-4e-gate.md:26` -- "what it actually decides is the engine for `run.rs`'s own unit tests" is under-inclusive: `eval.rs`, `trace.rs`, `stem.rs`, `value.rs`, `plan.rs`, `queue.rs`, `builtin/*` and `ir/golden_tests.rs` all construct an `Interp` directly too. It errs toward under-claiming coverage.
* `docs/superpowers/plans/phase-4e-gate.md:24` -- the harness list omits `builtin_status.rs`, `input_oracle.rs`, `loud.rs`, `parse_version_oracle.rs` and `state_builtin_oracle.rs`, which also build from `Invocation` and also flipped.
* `docs/superpowers/plans/phase-4e-gate.md:276` -- "`ir_dual.rs` diffs raw stderr ... across **every** population" is scoped to that file's four populations (corpus, expressions, bif, keyword). The 14-program `tests/trace_oracle/` set is not among them, so after the flip it is IR-against-oracle only, with `tests/support/mod.rs` normalising the prefix region and no arm-to-arm comparison. `tests/ir_dual_cases/trace-settings` is what actually carries unnormalised trace between the arms.
* `rust/crates/rexx-exec/src/bin/rexx-run.rs:48` -- the shipped binary's unset default has no test anywhere. Its doc says it defaults "to the same engine `Invocation::none` picks", and nothing checks the two agree; `invocation.rs:309` asserts the library half only. Of the three places the gate names, this is the one with no witness.
* `rust/crates/rexx-exec/src/ir/mod.rs:41` -- "`render` ... has no caller outside `golden_tests.rs` and **the plan's ten tasks** never give it one". That is a call-site count in prose, which `rust/CLAUDE.md` forbids, and "ten tasks" is wrong: the plan has Tasks 0, 1, 2, 3, 4a, 4b, 4b', 4b-M, 4c, 5, 6, 7, 7-M, 7-M2, 7-M3, 8, 9, 10 and 11. The gate cites this comment as its evidence at `:265`.
* `rust/crates/rexx-exec/src/ir/compile.rs:1032` -- `assert_clause_regions_hold_no_clause_op` checks only `Op::Generic`. The doc header and the panic message are accurate; the name promises more than the body checks.
* `rust/crates/rexx-exec/src/lib.rs:1962` -- `Interp::new`'s doc carries both a call-site count and a task-scoped "files this task is not permitted to touch", with an under-inclusive list of four files. Measured: 402 sites across 16 files. Pre-existing, not Phase 4e's, but the gate leans on the count.

## What is unsettled, and what would settle it

* **Whether the exact-stderr trace tests exercised promoted clauses under `Interp::new = Engine::Ir`.** One build, reading `trace_op_echoes()`.
* **The `+22` per pass an all-`Generic` body now pays.** The gate refuses to attribute it and names the driver's `match` widening as an unclaimed candidate. Settling it needs a build that isolates dispatch width -- the gate's own suggestion of a jump table or a two-level encoding, measured as a spike, not an argument.
* **`arith`'s tree-walker arm moving +186.6 instructions per pass at the Task 9 boundary**, on an arm Task 9 does not touch. Recorded in the task report, absent from the gate, unexplained. Visible in the per-pass absolutes: 61844.27 at `e63e8a00` against 62030.85 at `16077ea1`.
* **The enter/leave trade at final coverage.** Correctly reported as unevaluated; the counterfactual build is a hand-written pre-split driver carrying the current op set, and nothing cheaper answers it.
* **Five platforms.** Untouched, unchanged, correctly carried.

## What I did not do

No benchmark and no profiler was run, per the brief.
No oracle benchmark sitting.
I did not re-run the suite: the lead's 1423 in both profiles at `e303e7a2` stands and re-deriving it would have added nothing.
