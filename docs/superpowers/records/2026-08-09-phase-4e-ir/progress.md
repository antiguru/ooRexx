# SDD ledger -- plan: docs/superpowers/plans/2026-08-09-phase-4e-ir.md

## Task 0: land the driver extractions -- complete

BASE `d45aafd8`. Commit `f9d9f46c`. Executed by the controller directly, before SDD was
requested, so there is no task reviewer report for it.

Landed `grant_procedure_permission` and `apply_flow` from spike `614388fc`, and
`reserve_temps`/`temp_at`/`set_temp` plus four tests from spike `4b285394`, both by
`cherry-pick -n` rather than by retyping.

Verified: 1337 passed / 0 failed in **both** the dev and release profiles, against 1333 before,
which is exactly the four new tests. `cargo fmt --all --check` 0, `cargo clippy --workspace
--all-targets -- -D warnings` 0 on a warm target.

**Two corrections to the plan's own text, which later tasks inherit:**

* The widening list is **eight**, not six. S1 landed `grant_procedure_permission` and
  `apply_flow` as private; a later spike commit widened them, and the plan's grep-derived list
  had put them in the "already `pub(crate)`" group.
* `pub(crate)` pushed `apply_flow` and `offer_to_trap` past the line width, so `cargo fmt`
  rewrapped both signatures. Expected, not a defect.

The `SPIKE (S1)` / `SPIKE (S2)` doc-comment labels were stripped; the reasoning under them is
unchanged and stays. `rexx-exec/tests/spike.rs` is unrelated (stack-depth tests) and untouched.

**The trap offer stays at `run_activation`'s call site**, confirmed by reading the loop rather
than by trusting the spike's note. Its position is the semantics: one offer per activation, made
by the activation that is unwinding.

## Task 1: re-establish the benchmark anchor -- complete

BASE `1b3efeb1`. Commits `27a1278b`, then fix round `fb43b3c4`.
Review: spec compliance **MET**, quality not clean -- 0 Critical, 3 Important, all in the anchor
document's prose, all fixed in one round.

**The anchor, tree-walker vs oracle, 9 pairs per axis interleaved, at `1b3efeb1`:**
`alloc4c` 2.00x, `arith` 2.67x, **`emptyloop` 3.33x**, `varlookup` 4.40x, `compound` 5.81x,
`strings` 10.47x; `rexxcps` internal cps 7.33x. `emptyloop` is new, at `n = 25000000`.

**The finding later tasks must not lose.** `alloc4c` moved -3.8% and `compound` -4.4% against
`perf-baseline.md`'s `d233d1e9` figures, with disjoint intervals, both in the same direction. Two
candidate causes and the document decides between neither: Task 0's extraction out of
`run_activation`'s per-clause loop, or ordinary cross-run noise -- `perf-baseline.md:794` already
recorded `compound` moving about 4% between two runs with no code change at all. **A later task
claiming a movement under about 4% on `alloc4c` or `compound` cannot attribute it to its own
change** without first attributing `f9d9f46c`, which nobody has done.

**A review finding that was wrong, and the process fix it forces.** The reviewer called the
anchor's "Ambiguity resolved" citation fabricated, because it appears in neither the brief nor the
plan. It came from my dispatch prompt, which the reviewer never saw. **From Task 2 on, every
ambiguity I resolve goes into the brief file, not only into the dispatch prompt** -- otherwise each
resolution reads as an invention to whoever reviews it. The residual real issue (a dispatch prompt
is not in the repo, so the citation was uncheckable) was fixed.

Verified by me independently, not taken from the agents: all six ratios recomputed from the
medians; all five movement percentages and interval-overlap verdicts recomputed; the `d233d1e9`
source figures read out of `perf-baseline.md`; the `git log d233d1e9..1b3efeb1` claim; and the
quoted precedent at `perf-baseline.md:794`. All matched.

Suite unchanged at 1337 passed / 0 failed, dev and release.

**Instrument note.** The first run of this task was parked for 70 minutes by a waiter loop
`while pgrep -f 'release/rexx-bench-suite'`, which matches its own `bash -c` command line and can
never exit. My own status checks had the same bug. Poll a captured PID with `kill -0`, or use a
bracket pattern.

## Standing obligation: the dead-code annotations Task 2 shipped

Recorded here, not in a task brief, because the risk is that "temporary" becomes permanent and no
single task owns noticing.

**Task 3 must remove:** `chunk_for` (`plan.rs`), `compile` (`ir/compile.rs`), `Chunk`'s `ops` /
`op_of` / `registers` fields, `ChunkTooLarge` (`ir/mod.rs`), and `Interp::chunks` /
`chunks_refused` (`lib.rs`). Task 3's driver is the first production caller of all of them, so if
any annotation survives Task 3, either the driver does not use what it should or the annotation is
stale.

**Task 4 must remove:** `Op::EvalExpr`'s `#[expect(dead_code)]`, and `Op::Clause`'s.

**Neither, and this is the exception that matters:** `render`'s. Nothing in the plan ever gives it
a non-test caller. It is being gated behind `#[cfg(test)]` in Task 2's fix round instead, because a
permanent `#[allow(dead_code)]` is worse than a cfg gate, and the plan's "no cfg gate" line was
written without a reason behind it.

**`Op::Clause`'s `#[expect]` was reported as contradicting the plan, and the plan was wrong.** Task
4's Produces list omitted `Op::Clause` while the Decisions section says a promoted clause emits
one. Fixed in `cd529eea`. The annotation was correct throughout.

## Task 2: `Op`, `Chunk`, and the chunk cache -- complete

BASE `fb43b3c4`. Commits `64ce256d`, plan fix `cd529eea` (mine), fix round `09cba5a8`.
Review: spec compliance **MET**, quality not clean -- 1 Critical, 2 Important, all fixed in one
round. Suite 1337 to **1340**, dev and release; clippy clean from a removed `target/`.

**The Critical was a test that could not fail, and the plan put it there.** The boundary test
compared `chunk.op_of.len()` against `Chunk::instruction_count()`, itself defined as
`op_of.len() - 1`, so it reduced to `x == x`. An earlier draft of the plan asserted the literal
`3`; I changed it to a relation to stop it being brittle and made it tautological instead. The
fix takes the count from `program.main.instructions.len()`, a source independent of the chunk,
and `Chunk::instruction_count()` was deleted for want of a caller.

**I re-ran the mutation myself rather than accepting the report.** Backed up `compile.rs` with
`cp`, replaced `op_of.push(end)`, ran `cargo test -p rexx-exec --lib ir::`: exactly one test goes
red, `left: 4 right: 5`, the other two pass. Restored from the copy, `sha256sum -c` OK, tree
clean, 1340 again. This is the check whose absence let the vacuous test ship in the first place.

**`render` is `#[cfg(test)]`, not `#[allow(dead_code)]`.** Nothing in the plan ever gives it a
non-test caller, so the allow would have been permanent. The plan's "no cfg gate" line was written
without a reason behind it and is overridden.

## Standing obligation moved to Task 4: the compiler-side `Clause`/`Generic` assertion

Task 3's Step 4 required asserting in the **compiler**, not only in prose, that no `Clause` op
precedes a `Generic` op -- because `step_in_temps_frame` echoes the clause itself and the echo is
not idempotent. Task 3 did not write it and did not report skipping it (review finding I3).

Not writing it was right; not saying so was not. `compile` emits only `Generic` today, so the
assertion cannot fail and would be exactly the vacuous-test defect this phase has already shipped
once. **Task 4 is the first emitter of `Clause`, so Task 4 owns the assertion and a mutation
proving it fires.**

## Observation, unattributed: a test that failed once under a full workspace run

`state_builtin_oracle::queued_null_line` failed in one `cargo test --workspace` run, and passed in
isolation and on a re-run. Suspected mechanism is the oracle's shared session queue under
concurrency. **Nobody has established whether it also happens at BASE**, so it is not attributed to
Task 3. If it recurs, establish that first -- otherwise it will be blamed on whichever task is in
flight when someone finally looks.

## Task 3: the driver, engine selection, and the dual-engine harness -- complete

BASE `09cba5a8`. Commits `533c48a5`, fix round `777fa4a9`.
Review: spec compliance **pass with one gap** (moved to Task 4, above), quality **good**.
3 Important, 5 Minor, all fixed in one round. Suite 1340 to **1349**, identical in dev, dev+STRICT
and release+STRICT; clippy from an empty target; corpus differential 51 of 51.

**The driver.** Two-level loop; `pc` stays an instruction index and maps through `chunk.op_of`, so
`apply_flow` is reused unchanged; `Generic` delegates to `step_in_temps_frame`; the trap offer
stays at its call site. `Op::Clause` and `Op::EvalExpr` answer Loud rather than panicking, since
nothing constructs them yet.

**Accepted deviation: selection lives inside `run_activation`, not at its two call sites.**
`run_activation` is the single function that runs a body, so both entry points reach the decision by
construction rather than by two edits that can drift. Witnessed by a test requiring a main body, a
`CALL`ed label and a `::ROUTINE` to drive exactly three chunks -- and the review closed the argument
by mutating the *other* way too: `activations.len() > 1`, meaning selection at
`resolve_and_run_call` only, is also red on that test alone. The count of three distinguishes both
one-sided placements.

**The wrong-chunk class is now caught, where before it was merely unreachable.** A `debug_assert`
pins the activation's `body_key()` against the plan it is running with, by `Rc::ptr_eq` into
`plans` -- a read, never a `plan_for` that would cache under the wrong key. **I mutated a
`program_id` construction site to `ProgramId(999)` myself: 89 failures, against green before the
assert.** Production loads one program, so every `ProgramId` is 0 and `directive` alone
discriminates; this matters the moment a second program can be loaded.

**Two review findings worth carrying as patterns.**
* A pin between two in-repo lists is not a pin. `every_population_is_present_and_non_empty`
  compared the builder's names against a literal beside it, so deleting a population from *both*
  left the sweep half-sized and green. Now derived from the tree.
* The counter that broke on a schedule. It was process-wide, so Task 11's default flip would have
  reddened two tests with no one having touched them. Now per thread, with the tests entering
  through `execute`. A rehearsal flipping the default runs 654 of 655 green, the one failure being
  the test that states the old default -- which Task 11 owns.

**What the sweep does not prove, measured rather than assumed.** Deleting engine selection outright
still leaves the four `ir_dual` tests green: every op is `Generic`, so the two engines are identical
by construction and 10,391 agreeing programs witness nothing about the IR. Not a defect at this
point in the phase, and the statement is in the tree in three places rather than only in a report.

**Residual.** The counting tests run on a `libtest` thread with an ordinary stack rather than the
512 MiB one `run_program` sizes, so a program deep enough to need that stack cannot be counted this
way. Not a limit for anything this phase counts.

## Task 4a: run a loop's body from the compiled stream -- complete

BASE `777fa4a9`. Commits `c6b1c32d`, `b6531a68`, fix round `736bf080`; plan commits `27b522bf`
and `f0dcc1dd` are mine. Review: spec compliance **pass**, quality **pass with one Important**.
Suite 1349 to **1354**, green in dev, dev+STRICT, release and release+STRICT.

**The phase's first promotion, and the question that mattered came back clean.** No loop semantics
exists twice: `run_loop`/`run_repeating`/`loop_advance` are one implementation entered from both
engines, and the only engine-dependent line in the whole promotion is `run_bounded`'s two-arm match
on `BodyEngine`, whose `Chunk` arm reaches the same `step_in_temps_frame` clause unit the other arm
calls.

**It found a defect in the plan and stopped rather than forcing it.** `Op::EvalExpr` is not
trace-identical for a loop header: `setup_controlled` interleaves evaluation and `>K>` emission in
`ctrl.order` order. Task split into 4a (landed), 4b (op-level machinery, debuting on `If`/`Select`
where a branch condition emits nothing between evaluations), 4c (flatten the header, after Task 6).
Moritz chose this over pulling trace ops forward or re-planning the phase.

**Measured and NOT claimed, by Moritz's decision.** `emptyloop` -1.7%, `varlookup` -3.2%, control
-0.2%, four interleaved rounds in one binary. **No mechanism named, and the promotion adds work per
body clause while removing none**, so the arm doing more work is the faster one. Not a baseline for
any later task. Phase 4f may chase it.

**I1: a promotion with no witness at all.** `LoopKind::Simple` passing `TreeWalker` left the
**entire workspace green**, and `LOOP_CASES`'s "simple block" case was `if 1 = 1 then do`, which
`If`'s hard-coded `TreeWalker` routed away from the chunk before the loop was reached. Fixed; **I
re-ran the mutation myself: it now reddens exactly one test and nothing else.**

**The mutation table cut both ways, and the second direction is the useful one.** Dropping
`UNTIL`'s second `DO` re-echo under `Engine::Ir` reddens **the new loop test alone** -- the
10,391-program corpus sweep misses it entirely. So the implementer's self-critical "these tests add
no coverage" was too modest, and a trace-shaped defect is exactly what a final-output corpus cannot
see. Also: `Generic` calling `step` directly does not compile, so the clause unit cannot be
bypassed.

**A tripwire deliberately left for 4b and 4c.** The two clause-count assertions will need updating
when flattening changes how many clause-op entries a loop produces. The count is the observable, so
this is intended -- but they are a tripwire on the next two tasks rather than invariants, and a
task that "fixes" them without understanding why has broken the witness.

## Task 4b: the op-level interpreter, on `If` -- complete

BASE `736bf080`. Commits `a2b6e225`, `08366e09`, `ac7b9bec`, `8f542fe1`, fix round `cfc4a688`,
`401e0df5`; mine `992ade97`. Review: spec compliance **failed on one property**, quality good.
1 Critical, 3 Important, 5 Minor, all fixed. Suite **1364**, all four gates, clippy from a clean
target.

**C1 was a real correctness defect, oracle-confirmed.** A promoted `IF` clause did not record its
failure site when a `CALL ON` handler failed at its boundary: the IR printed no clause echo where
the oracle prints one, and inside a loop printed the enclosing `DO`'s. **Fixed one level up from
where it was found** -- the recording moved into `in_stepped_clause` and out of
`step_in_temps_frame_with`, so the clause unit discharges the obligation and no promoted clause can
forget it.

**The blind spot is worth more than the bug.** The sixteen-case branch table was built around branch
*shapes* and never around the clause *boundary*: every case asked what a branch does, none asked
whether the clause unit is discharged where the promotion opens one. Recorded at `BRANCH_CASES`'
doc, in 4b''s plan section, and atop the report. **Extend with boundaries, not shapes.**

**The mechanism for the +7% is named, from two independent investigations that converged.**
I profiled with samply and analysed with pollard; the reviewer read the diff. Both landed on the
same place. `emptyloop` n=25e6, wall 3013 ms tree-walker against 3097 ms IR:

* `run_ops` **96 ms, new**, replacing `run_bounded_instructions` at 49 ms -- the op loop costs about
  twice the instruction loop.
* `step_in_temps_frame_with` self **617 to 674 ms**; its closure 151 to 111.
* `memcpy` **17 to 53 ms on an identical stack** through `run_repeating` -- same code, three times
  the time, consistent with a larger value copied per clause.
* **About 84% of the regression sits on the dispatch path itself.**

Reviewer's sharpest candidate: `run_bounded_from_chunk`'s `op_at(start)` and `run_ops`' `op_at(end)`
add two bounds-checked loads and a call frame **per `DO`-body pass** -- 50e6 loads and 25e6 frames
on a 25e6-pass axis. Plus `absorb` moving a 64-byte `Flow` per clause and `BodyEngine` growing 8 to
16 bytes.

**And it answered the instrument question, which is the bigger result.** Three of those changes sit
on the path **both** arms take -- `absorb`, the closure routing, and `if_targets` computing
`skip_else` on both paths. So **no within-binary control over these binaries could have read zero,
whatever it reverted.** 4b-M's first deliverable is therefore not "a better revert" but a control
whose arms share no changed code, or an instrument needing none.

**A verification I did that came out weaker than the agent's, stated so nobody reads it as
confirmation.** I deleted the boundary recording outright; that reddens the pre-existing
tree-walker test `a_handler_that_fails_at_a_clause_boundary_blames_that_clause`, which proves the
code is load-bearing but is **not** the agent's M13. M13 relocates the recording so the tree-walker
path still works and only the IR path breaks, and it claims exactly one catcher. I did not
reproduce that specific claim.

## Task 4b-M: the control and the mechanism -- complete

BASE `401e0df5`. Commits `6f6fce91`, `915e39cf`; mine `1535b030`. Suite unchanged at **1364**, all
four gates. No review dispatched: this task promoted nothing and its deliverable is a measurement,
which I checked directly.

**The instrument, with a demonstrated zero rather than an asserted one.** `perf stat -e
instructions:u`. Two independent builds of identical source at `401e0df5`, different worktrees,
different sha256, different layout, read **+1.2e-9** apart on the tree-walker arm and **-1.6e-8** on
the IR arm.

**And its limit bit immediately, which is why it is not the gate quantity.** On `emptyloop` the IR
arm runs **4.14% more instructions and 0.31% fewer cycles** (IPC 4.88 against 4.67). Every gate
claim is on cycles and wall-clock; instructions were the trustworthy proxy for *attribution*, not
for the verdict.

**The headline is not what the task was sent for: about four fifths of the "IR regression" was on
the path BOTH arms take.** Removing it put the **tree-walker 7.0% faster than the pre-driver binary**
on `emptyloop` and 5.9% on `varlookup`. The driver work had slowed everything down, and the IR
comparison had been measuring against an already-degraded reference. That also retires the
instrument question for good: no revert-based control could ever have read zero.

**The two causes, both from Moritz's `BodyEngine` question:**
* **`Flow` was 64 bytes because `LeaveOrigin` was inline** (`site: Option<(usize, Vec<u8>)>`), so
  every clause moved a payload only `LEAVE` and `ITERATE` read. `Box<LeaveOrigin>` takes it to 24:
  **-2.37% tree-walker, -0.85% IR.**
* **The generic `in_stepped_clause` closure was the largest single cause.** `#[inline(always)]` on
  it and on `in_clause`: **-2.68% / -2.31%**. Plain `#[inline]` read exactly 0.
* **`BodyEngine`'s width was never established as a cost** -- the two above sum to the shared
  regression within 0.03%, leaving it no residual. So the by-reference idea was correctly
  deprioritised rather than tested and found wanting.

**Three candidates refuted with numbers**, which is the part that stops them being retried:
`run_ops`' eight arguments (call site plus prologue is 21 of 74 instructions per pass; removing the
call site bought 2), `granting.grants()` (2 instructions per clause), and `if_targets`'s
`skip_else` (exactly 4 per `IF`, exactly 0 on both registered axes).

**Where criterion 4 stands.** `emptyloop` **passes** (IR 3.12x against tree-walker 3.13x);
`varlookup` **fails** (4.28x against 4.21x, IR/TW 1.016, slower in 9 pairs of 9, non-overlapping).
Anchor was 3.33x and 4.40x, so both arms improved.

**Moritz accepted the residual with a discharge condition** -- see `1535b030` and the plan section.
The prediction is that the per-body-range-entry cost dilutes once Task 7 makes `varlookup`'s two
assignment clauses native ops. **Task 7 re-measures; still above 1.0 refutes the hypothesis and puts
the three structural remedies back on the table. It is not recordable as a debt twice.**

## Task 4b': promote `Select`, and build the frame stack -- complete

BASE `1535b030`. Commits `37dacefc`, `d53e0a8f`, `67f37050`, `75b4e770`, `b52baa2b`, fix round
`f86cb0d5`. Review: spec compliance **not met** on first pass, quality mixed. 1 Critical, 1
Important, 2 Minor, all fixed. Suite 1364 to **1369**, all four gates, plus 18 new `BRANCH_CASES`
rows.

**"Boundaries, not shapes" paid for itself three times over.** The task found two oracle-confirmed
defects older than itself, both with the whole workspace green; the review then constructed a third
the task had argued was unreachable.

**C1, and the missing premise is the durable part: a boundary can leave a NEW pending trap behind
it.** A handler delivered at a clause boundary can itself `raise ... return`, queueing a trap for
the caller -- and `in_clause` delivers at most one trap and does not re-check. **So the last member
boundary is not the last boundary with work**, which is exactly what a promoted construct's single
boundary assumed. Witness: `call on user zx name h` / `call on user zy name g` / `select` / `when 1
= 1 then zq = raiser()` / `end` / `say 'after'`, with `h` doing `raise user zy return 1`. Oracle and
tree-walker print `G ran 4` then `after`; the IR printed `after` then `G ran 6`, and panicked on
`clause.rs:414` in debug.

**THE STRUCTURAL FINDING, which outlives this task.** The oracle's boundary comes from a **synthetic
end-of-branch instruction that our Phase 3 parser elides**. The tree-walker gets the equivalent from
`step_in_temps_frame`'s wrapper; a flattened construct has no wrapper, so `Op::EndBranch` and the
matched-`WHEN` frame close are its substitute. **Seven oracle divergences are now listed in the
report and all but two are the same elided instruction**, in constructs this task does not promote
(`ELSE`, `DO` blocks, a nested second delivery's `SIGL`). Task 4c and Task 7 will meet it again.

**Measuring the adjacent cases changed the fix twice**, which is the "pair a refusal with its
adjacent success" rule earning its place: an `IF` whose condition is false owes **no** such
boundary, because the oracle jumps over the instruction; nor does a branch redirected into
`OTHERWISE`, nor `OTHERWISE` itself. Each is measured and each has a case that reddens without it.

**Two things nobody predicted:**
* **A pre-existing false positive in the crate's own tripwire.** `clause.rs:414` fires at
  `1535b030` on a program with no construct in it at all, for any re-queueing handler.
  `PendingTrap::queued_during_delivery` exempts exactly that.
* **A case where the IR engine is right and the tree-walker is wrong.** It could not join the
  agreement table, so `KNOWN_DIVERGENCES` was created, with a test that is red if *either* engine
  moves.

**A disagreement I accepted.** I framed the `OTHERWISE` `SIGL` divergence as a third separate item;
the implementer argued it is the same mechanism as the false path -- the tree-walker's wrapper
running a boundary where the oracle has no instruction -- and recorded them as one family with one
fix direction. That analysis is more specific than mine and the honest fix it names (the
tree-walker should stop resolving branches inside its own `step`) is a real one.

## Task 6: trace as explicit instructions -- complete

BASE `4f98b9dc`. Commits `56ab1511`, `36c36712`, `9572f527`, fix round `de05ea58`; mine `2ff577bb`,
`d03cc3af`. Review: spec compliance **meets the spec**, quality accept with required fixes.
3 Important, 4 Minor, all fixed. Suite 1369 to **1378**, all four gates, clippy from a clean target.

**It found a hole in D23, which I wrote.** Compile-time-only trace is a **wrong-output regression**,
not an optimisation choice: `if 1 = 1 then trace r` followed by `if 1 = 1 then say 'x'` -- the
oracle echoes the second `IF` and a chunk compiled untraced would not. So the setting decides what a
chunk *emits*, and a per-clause staleness check decides whether that chunk still *applies*, making a
stale chunk slow rather than wrong. D23 amended. Cost: about 17 instructions per promoted clause,
+0.25% on `emptyloop`; `Generic` clauses and the tree-walker arm exactly neutral.

**Moritz's propagation question narrowed the refinement sharply.** `Activation::nested` (a `CALL` to
a label) inherits `trace_mode`; `Activation::routine` resets to `NORMAL`; **no callee ever writes it
back**, pinned by `a_callees_trace_setting_does_not_survive_its_return`. **So a call cannot change
the calling body's setting**, and the "could see a mid-run change" set is `TRACE` or `INTERPRET` in
the body -- not "or a call". Recorded in D23 as available, with his two-blocks proposal analysed
beside it: the cache key already gives per-setting blocks lazily; two blocks would make the
*mid-body switch* cheap, and the refinement is what makes the *detection* rare. They compose.

**F3: the mechanism the task exists to build had no witness.** Passing a constant `NORMAL` at the
entry lookup, or forcing `stale` true, **each left the whole workspace green at 1373/0** while
making `Op::TraceClause` dead in production. No output test can close it -- the fallback is
output-equivalent by design. Now counted where the echo is **emitted**, not where the op is fetched,
because R2 leaves the op in the stream and skips its work. Both mutations redden, each caught by
that test alone.

**Two false numbers at decision points**, the shape this phase keeps producing: a comment claiming a
40.30 billion staleness read that measures **40.4009**, asserting a parity the report's own table
denied; and four `#[inline(always)]` in `trace.rs` carrying no reasoning while **being the whole of
the "tree-walker is exactly neutral" claim** (removing them costs 100M instructions on the
tree-walker arm).

**My own error: I accepted a false reason and quoted it back to Moritz.** The
`InlineCase`-over-`datadriven` justification -- a compared trailing space not surviving an external
file -- is false; datadriven pushes each expected line verbatim. Now moved to
`tests/ir_dual_cases/trace-settings`.

**And moving it surfaced a hazard worth more than the rule** (recorded in
[[use-datadriven-for-case-tables]]): **`REWRITE=1` is unsafe for oracle-captured expectations.** It
would silently replace the oracle's bytes with the implementation's, converting a differential test
into a self-consistency test that passes against any behaviour. The file refuses to run under
`REWRITE` and carries the oracle capture command instead.

**A correction the implementer made to me, in my favour.** I passed on the reviewer's "12 of 480
swept programs" for F8 without verifying it; the implementer declined to restate it and reported
only what it measured. F8 -- `TRACE VALUE expr` never emitting the oracle's `>K> "VALUE"` line -- is
pre-existing, confirmed on `4f98b9dc`'s binary, and fits neither `KNOWN_DIVERGENCES` (the engines
agree; it is the oracle they differ from) nor the exclusions file, so it is in the report with the
site and the one call that would fix it.

**Residual:** `emptyloop`'s +0.25% under the compiled stream is 4 instructions per pass on a body
with **no promoted clause**, so it is the extra `Op` variant in the driver's match, not the
staleness read. Unattributed further.

## Task 7: promote `Assignment` and `Say` -- complete

BASE `caf16c90`. Commits `f330a96a`, `439e5d6c`, `3bcbcc53`, `f55ea409`, `98ac2042`, fix round
`fd0ea6d1`; mine `13a5bc91`, `7c34ffb6`. Review: spec compliance **PASS**, quality **PASS with
reservations** -- 0 Critical, 3 Important, 4 Minor, all fixed. Suite 1382 to **1388**.

**The discharge condition was refuted, and its premise was backwards.** It assumed the driver's cost
is per body-range *entry* and so dilutes as a range holds more ops. Measured: the cost is per
promoted **clause** and **zero** per entry. `varlookup`'s IR-minus-TW went **60.5 to 138.5
instructions per body clause**, +78, taking the axis 1.0321 to **1.0725** on instructions, 1.0523
wall, IR slower in 9 of 9. **So promotion makes criterion 4 monotonically worse.**

**Two of the three recorded remedies are eliminated by the number**, which is what a refuted
prediction buys: hoisting the op range onto `BodyEngine` is per-*entry* work and the per-entry delta
is zero; a single-op fast path does not apply to regions holding three and four ops. Moritz chose to
find and remove the per-clause cost first (Task 7-M, blocks 8/9/10) rather than redesign yet.

**Moritz also kept the sharing rule at a measured price.** Task 7 regresses the **tree-walker** arm
0.2 to 1.3% purely from extracting `step`'s arms into the functions the ops share, and an inline
probe restores BASE exactly -- but that probe is two implementations, which is the defect the
dual-engine gate exists to catch and has caught three times here.

**THE FALSE CLAIM THIS TASK KILLED, and it was mine.** The spec has said since the spike that a
dropped `>L>` line is visible only to an exact stderr comparison and that **no corpus instrument
here does one**. False. True of the *oracle differential*, whose `tests/support/mod.rs` normalises
that region; false of **`ir_dual.rs`'s dual-engine sweep**, whose `compare` diffs raw stderr between
arms. Verified by making `TraceLiteral` a no-op: the sweep reddens naming two corpus programs and
stays red with Task 7's own file held out. **It had been repeated through six briefs, including the
one I handed this task as fact.** Corrected in the spec at `7c34ffb6`, because the spec is what
briefs regenerate from.

**A blind spot that redirects 7-M.** `Op::Const` and `Op::TraceLiteral` execute **zero times inside
any measured loop** -- measured with a throwaway counter at `n` and `2n`: one execution on
`emptyloop`, one on `strings`, zero on the other five axes. So the headline op pair has no
performance evidence either way, **and the +78 per clause is the clause machinery (`Clause`,
`TraceClause`, `Store`, `EvalExpr`), not `Const`** -- `varlookup` has no literal in an assignment's
value position at all. Without this, 7-M would open by profiling ops that never run.

**The `>L>` reproduction is the strongest evidence in the phase**: ten independently written
programs across literal, symbol, computed compound tail, whole stem and sub-expression, under `trace
i` and `trace r`, each run three ways with all three descriptors compared as bytes; plus six indent
shapes, five staleness shapes, eight numeric-literal spellings; and all 27 stanzas re-run under the
oracle with 0 mismatches, green before the promotion.

**The implementer corrected the reviewer's numbers**, having reproduced first: the pre-existing
`SIGL`-one-clause-early divergence is oracle 5 against 4 and 6 against 5, not 6/5 and 7/6 -- the
programs differed by one line. Offsets and finding identical.

## Task 7-M: find and remove the per-clause cost -- complete, with its verdict qualified

BASE `fd0ea6d1`. Commit `132c3395`; mine `8e7ce246` (spec), `2e4f4e3d`, `9da84dc3` (plan),
`304236d6` (spec corrections), `bd61e489` (Task 7-M2). Review: spec compliance **PASS**, quality
**the verdict is not fit to discharge a design question** -- 0 Critical, 9 Important, 7 Minor.
Suite 1388 to **1391**. `varlookup` 1.07251 to **1.06256** on instructions, 1.0786 to **1.0452** on
cycles; every axis improved.

**The review confirmed the conclusion and rejected the evidence, which is the distinction that
mattered.** Most of the residual is structural and promotion stays monotonically worse per clause --
that stands. What does not stand is the itemisation offered in place of criterion 4's floor: the
+78 breakdown **sums to 80**, and its largest row -- 26 instructions of inlined bounds checks and
`Result`/`Option` plumbing -- appears in **neither** the removed 19 nor the residual bullets, which
is exactly the category the removed 19 came out of. A residual stated as 59 whose numbered bullets
reach about 40 is not an itemisation, so the replacement guard was not delivered.

**Two false statements were mine, written into the spec on the strength of the report.** "`in_clause`
being a scoped closure is what forces two levels" was asserted and never established: a scoped
closure forces a **call boundary**, and the second **dispatch level** comes from a clause's ops being
a sub-range of a flat stream. And the precision leg -- "the floor was not measurable" -- is a
**wall-clock** argument, in a commit that stopped using wall clock in the same breath; on
instructions the measurement reproduces to eight significant figures. Corrected at `304236d6`.
Moritz kept the withdrawal, now resting on the structural leg and on there being no in-phase route
to the floor.

**Two removals nobody had tried.** Boxing `Raised` takes `Failure` from 104 bytes to 24 and kills
the `sret` return without changing what a promotion emits -- so the call boundary's return half is
not inherent, whatever it measures. An enter/leave pair keeps one implementation of the clause unit
while giving the driver its own op loop; "one closure" and "one implementation" are not the same
thing, and 7-M treated them as one. Both are spiked under prototype-publish-revert in Task 7-M2.

**Task 7-M2 blocks Task 11 and nothing else**, so Tasks 8, 9 and 10 proceed on the current shape.

## Task 8: promote variable access -- complete

BASE `9da84dc3`. Commits `938de575`, `0036ca7c`, fix round `9c20430f`; mine `e294cdf8`, `bea6b2eb`,
`ced6c209`. Review: spec compliance **PASS**, quality **high** -- **0 Critical, 0 defect-class**,
2 Important and 5 Minor, all prose, all fixed. Suite 1391 to **1401**, green in dev, dev+STRICT,
release and release+STRICT; clippy clean from an empty target directory.

**The first task in this phase to move a ratio downward.** `varlookup` **1.06256 to 1.02118** and
`alloc4c` 1.02002 to 1.01026 on instructions; the four axes with no promoted read are unmoved.
**-154 instructions per promoted read**, at two problem sizes on two axes, and the split is the
useful part: **2 is the native op and 152 is not making the `HashMap<SymbolId, usize>` probe.**

**Three plan facts it falsified, two of them mine.** The `-10.7%` prediction basis was wrong twice
over -- that is P2's **wall-clock** figure for the **name**-keyed lookup at an assignment **target**,
which `phase-4d-attribution.md:207` itself calls "half the claim"; the read side is `:187`'s
**12.5%**, recorded at `:210` as bounded but not prototyped. **My amendment telling the task to
predict against a fraction of it inherited the error rather than catching it** -- I corrected a
document without checking the source it cited. The file structure listed committed `.ops` files
under `testdata/ir/`; no such directory and no `.ops` file exists anywhere in the tree.

**THE SEVENTH INSTANCE OF THE PROJECT'S MOST REPEATED ERROR SHAPE, and it was in the plan text.**
Step 3 said the unresolved-slot fallback "is not optional" because `INTERPRET` introducing a name,
`DROP (v)` and a first `CALL` before `RESULT` all reach `grow_slots`. **All three do reach
`grow_slots`. None reaches the fallback** -- they reach it only when the name occurs nowhere in the
compiled body, and then nothing compiled a read of it to fall back from. Zero hits across 1401 tests
under four gates and eight programs written for the three routes. A sound inference from a true
premise, missing a second premise, settled only by running it. Corrected at `bea6b2eb` with the
reasons that hold: `compile` must not depend on `Plan::build` being exhaustive, and a compound has
no slot of its own.

**The review falsified the sharing rule rather than reading it**, and this is the technique worth
reusing: it swapped the `>C>`/`>V>` order **inside** `echo_symbol_read`, and engine-against-engine
stayed **green** while the recorded expectation failed. A second implementation would have shown as
a divergence. It also re-measured all fourteen trace stanzas against the oracle and added three
shapes nobody asked for, including a compound read inside a `PROCEDURE EXPOSE` callee where the
value indent is offset.

**A precision bound this phase did not have.** A control build differing only in `eval_node`'s three
arms reads **+0.74% on `emptyloop`'s IR arm** -- an axis with no promoted read, so the change could
not have reached it -- larger than four of the six per-axis moves in Task 8's own table, mechanism
unestablished. It sizes 7-M2's negative control. The tree-walker arm of the same pair reproduced to
735 instructions in 37.9 billion, which is consistent with the effect being arm-specific **and**
equally consistent with it being axis-specific; the implementer marked that as belief rather than
measurement when asked.

**`size_of::<Op>() == 12` is now a `const _` assert**, verified load-bearing, and a design input to
Task 9 rather than a compile error waiting for it.

**Task 8: complete.**

## Task 7-M2: deliver the itemisation, spike the two removals -- complete

BASE `ced6c209`. Commits `bec005fd` (the `rexx-arms` harness), `84dda087` (counter guard),
`af2014e4` (size rule), `e7b8eef8` (baseline file, 1,010 rows); mine `35417b23`, `e3e5c239`,
`6efc7827` (spec), `7a7f5849` (Task 7-M3). Suite 1401 to **1411**. Verified by me: `git diff
ced6c209 HEAD -- crates/rexx-exec crates/rexx-core crates/rexx-parse` is **empty**, so neither
spike leaked into the interpreter.

**"THE RESIDUAL IS INHERENT TO THE TWO-LEVEL SHAPE" WAS REFUTED BY A BUILD.** The enter/leave split
removes **152 instructions per `varlookup` pass** -- about twice what the whole of Task 7-M removed
-- taking the axis to **0.98143 instructions and 0.96988 cycles**, with `alloc4c` at 0.99581. First
time in this phase an axis with a promoted clause has come in under 1.0, and on both instruments.
`emptyloop`, whose body reaches no promoted clause, pays 24 instructions per pass. **The sharing
rule holds by construction:** `in_clause` and `in_stepped_clause_with` are both *defined in terms
of* the pair, one body each. The failure path predicted to be the hard part does not arise.

**Boxing `Raised` settles 7-M's self-contradiction by measurement.** Absolute speed improves on both
arms, every ratio worsens, `varlookup`'s cycles going 1.00890 to **1.16911**. 7-M's "the denominator
falls with it" is right and its "the promoted path carries one more such return" is wrong; the two
sat one sentence apart pointing opposite ways. The `sret` half is removable either way, so
**inherent was wrong independently of the direction.**

**7-M's breakdown was wrong in three separate ways**, none of them the way anyone guessed. The
two-instruction discrepancy is `stale`, stated `+7` and measured `+3` -- its own residual bullet
gave that item a range of "4 to 7" and the itemisation took the top. The `-3` row describes a
codegen redistribution that did not happen. And **the `+26` library row is not a row at all**: it is
the same 78 sliced by source file, which 7-M's own arithmetic proves. **The three marginals are also
not additive** -- 16+8+2 = 26 against 19 removed -- so "additive to within one instruction" is false
and every "worth N" inherits it. Change 1 is worth **16**, not 10, exactly as the review predicted.

**INSTRUCTION COUNTS ARE NOT DETERMINISTIC BETWEEN PROCESSES, which retires a claim Task 8 made and
I repeated.** Every excursion is an exact whole number per pass (46.0, 46.0, 12.0, 80.0, and 7-M's
own 23), up to **2.85%** on `emptyloop`. Mechanism unconfirmed; `HashMap`'s per-process seed is the
candidate. **A median over rounds is load-bearing rather than a refinement.**

**It corrected its own report in flight and said so.** Its opening line read "built, tested,
measured and reverted" before sitting D had run -- drafted ahead of its evidence. It flagged that
rather than fixing it silently, which mattered because **I had already read that line and repeated
it to Moritz as fact.** It also withdrew its own earlier attribution of an excursion to `perf`
counter scaling: the guard is still right to have and it closes a different mechanism.

**And it corrected the floor I wrote into its brief.** I turned Task 8's single `+0.74%` into a
general "not distinguishable below 0.75%" bound. Measured: on `instructions:u` two builds differing
in the driver agree to **2e-9** on an axis the difference cannot reach, so there is no such floor;
on `cycles:u` there is one and it is larger, about **3%**, and it applies to cross-build comparisons
rather than to the within-binary arm ratio where both arms share a layout. Third figure I
attributed to the wrong thing this phase.

**Task 7-M2: complete.** Moritz chose to land the split next, before Tasks 9 and 10, because no
patch survived and the rebuild costs the same whenever it happens.

## Task 7-M3: land the enter/leave split -- complete

BASE `7a7f5849`. Commits `7d9cfb9a` (the split), `73b538e5` (its sitting), `74b7f4d9` (a false
comment), fix round `e63e8a00`; mine `1eb3866f`, `d6aabbfe`, `3e9c4de0`. Review: spec compliance
**PASS**, quality **good** -- 0 Critical, 1 Important, 2 Minor, all fixed. Suite **1411**, verified
by me at head rather than taken from the report: `passed=1411 failed=0 ignored=4`, exit 0.

**The split lands and both promoted axes come in under 1.0 on instructions**, the first time in this
phase. `varlookup` 1.02118 to **0.98666**, gap +81 to **-51** per pass; `alloc4c` 1.01026 to
**0.99775**, gap +164 to **-36**. `emptyloop`, whose body reaches no promoted clause, goes 1.06596
to **1.07855** -- worse, accepted, and reported as a result rather than omitted.

**Criterion 4 is met on instructions and not on cycles for `varlookup`** (1.01081 to 1.01141), and
the report says so in its own headline instead of burying it. The floor is withdrawn, so this is a
recorded ratio rather than a failure.

**87% of the spike's gap, not all of it** -- 10 instructions per promoted clause short, and
**unsettled**. The implementer ruled one hypothesis out by a build and offers register pressure in
the collapsed function as the fit; the review closed the rebuild's half by comparing all twenty arms
of the new loop line by line against the pre-split version, and says the spike's half cannot be
checked without reconstructing it. Settled by a disassembly diff against a rebuilt spike, or bounded
by an `#[inline(never)]` build.

**THE ERROR PATHS ARE CLEAN, which is where I expected the defect.** Removing the closure removed
`?` from the region walk, and failure now propagates by hand. The review searched the whole span
between `enter_stepped_clause` and `leave_stepped_clause`: no `?`, no `return`, no `continue`, one
exit besides falling off the end. **The leave runs on every path and runs once.**

**The same defect appeared twice in one file, twenty lines apart.** `74b7f4d9` removed a false "worth
10 and 8 of the 19" marginal; the review then found `drive.rs:452` quoting **-71**, which is the
*spike's* gap, where this build produces **-51** -- contradicting the report written beside it. The
fix took the durable route rather than the correct-the-number route: **the figure is gone and the
comment points at `bench-baselines/phase-4e-arms.tsv` keyed by commit hash**, because a row keyed by
a hash cannot go stale where a sentence can.

**It applied the denominator correction rather than inheriting the old rule.** The spike's cycle
figures are bracketed in its table and explicitly not treated as a target this build can hit or
miss, because two builds' cycle ratios are ratios over different denominators.

**Clippy from a clean target directory is UNVERIFIED.** The report claims it; the review did not
re-run it and flagged the claim. Task 11's step 4 runs it clean at the gate, which is where it
binds.

**Task 7-M3: complete.**

## Task 9: promote arithmetic, with the patch table as its consumer -- complete

BASE `e63e8a00`. Commits `b3345d91`, `ea17a699`, `f0d12ebe`, `dd0c4dcb`, fix round `bb0f7e80`,
`16077ea1`; mine `f712de19`. Review: spec compliance **PASS**, quality **high** -- **0 Critical, 0
correctness defects**, 1 Important (prose) and 2 Minor, all fixed. Suite 1411 to **1417**, verified
by me at head: `passed=1417 failed=0`, exit 0. Measured nothing, by decision; its prediction is
recorded in the plan for Task 11 to check.

**A case that would have made the task a silent no-op.** An unquoted number parses as
`ExprKind::Constant`, not `Literal`, so `x + 1` contains nothing promotable and would have fallen to
`Op::EvalExpr` entire. **Without `Op::LoadConstant` this task would have emitted no arithmetic op
for any arithmetic line on any benchmark axis** -- `x = x + 1`, `t.k = t.k + 1`, `a = i / 3` -- and
measured as a null result rather than a missing case. The brief did not foresee it.

**THE PLAN'S PREDICTION WAS WRONG AND IT WAS MY CORRECTION.** I had already fixed this line once,
from "predicted movement: `arith` and `compound`" down to "`compound` only", citing 4d-1's prototype
P1 -- and I checked that citation: `-51.6%` really is P1 and really is on `compound`. **What I did
not check is whether P1 is the same change.** It is not. P1 extended the small-integer path to more
*operators*; this task changes how the *operands* are produced and leaves `small_int_arith` covering
`+`, `-` and `*` exactly as before. `i / 3` has as much operand evaluation as `t.k + 1`, so `arith`
should move, and my sentence "an `arith` that does not move is this task succeeding" **inverts**.
Fourth misattributed figure of the phase, and the first where the check I ran was **the right check
for a different question**. Withdrawn at the heading by `f712de19`, not only in a section below it.

**THE SOLE-CATCHER STUDY FOUND A HOLE RATHER THAN PRUNING ONE.** Eleven mutations attempted, eight
valid, each re-run with the `arithmetic` case file moved out under `--no-fail-fast`. Seven were
caught without it. **The eighth was caught by nothing in the entire workspace**: `**`'s exponent
goes through `to_number` rather than `arith_operand`, and routing it through the latter is invisible
to a `2 ** 2.5` row, because `2.5` converts and `Number::pow` then raises the identical 26.8. **Only
a nonnumeric exponent separates the two paths and no row had one.** `2 ** 'x'`, `'x' ** 2` and
`'x' ** 'x'` close it -- an asymmetry documented in `eval_arithmetic`'s own doc comment and
untested. **Three invalid mutants are named in the report so nobody counts them as evidence.**

**The file was kept, with its real justification recorded on the file itself.** Both arms of an
engine-against-engine comparison are this crate, so a shared arithmetic path wrong the same way
twice passes it; these rows pin what the **C++ interpreter** prints for shapes no corpus program
covers. Different from mutation-catching, and now written where the next reader inherits it.

**It predicted its own feature will not pay, before measurement.** D22 forbids a hint from removing
a precondition, and for small-integer arithmetic the precondition *is* the specialisation, so a hint
can only skip a hopeless attempt. It expects exit criterion 5 to find the win surviving
`QUICKENING = false`, and says that indicts the consumer rather than D22's schema.

**And it corrected D22's justification with a stronger true one.** The stated reason is that `Cell`
is not `Sync`, which does not bite yet. What bites today: **a chunk is reached as `&Chunk` through an
`Rc`, so a site cannot record anything without interior mutability at all.** `Cell<u32>` would serve
for that; only the `Sync` half is a bet. `PatchSlot`'s three methods are the sole place the choice
appears, so Task 11's step 3c is a three-line edit.

**Task 9: complete.**

## Task 10: promote the call forms -- complete

BASE `16077ea1`. Commits `ab562714`, `0742b77b`, `6b5fac3b`, `32d83e9b`, fix round `5ab3028f`;
mine `a8c6aefe`. Review: spec compliance **PASS**, quality **good** -- **0 Critical, 0 Important**,
3 Minor, all fixed. Suite 1417 to **1423**, verified by me at head: `passed=1423 failed=0`, exit 0.
Corpus 51 of 51, `chunks_refused` zero on both arms, `size_of::<Op>() == 12` still compiles.

**The review attacked the cache and could not break it**, which was its primary assignment, and it
**corrected the implementer upward**: a wrong resolution on the hit path has **two** guards, not
one -- the case file and the new driver test. It reproduced that rather than inheriting it.

**The `>A>` experiment turned out to be the sharing-rule proof in its strongest form.** One edit
inside `invoke_call` reddens both a `CALL`-route expectation and a function-route expectation while
the engine-against-engine comparison **never notices**. One implementation entered from two routes
and two engines. It also measured what the report could not: the no-op reddens **four of 21 rows**,
unreadable from a single run because `datadriven` halts at the first failure.

**TASK 10 VOIDED TASK 9'S FORWARD-COMPATIBILITY BET, and it is measured rather than argued.**
`assert_sync::<Chunk>()` fails at head naming **exactly one** culprit, `Cell<Option<Resolved>>` in
`CallSite`. **"Exactly one" is the load-bearing part:** rustc names no other field, so `PatchSlot`
and everything else is `Sync`, and `Chunk` **was** `Sync` before this task. So Task 9's `AtomicU32`
buys nothing a `Cell<u32>` would not, while `CallSite` stays a `Cell`. **Two locally correct choices
whose combination pays a cost for nothing.** Two riders: the cache hands out `Rc<Chunk>` and `Rc<T>`
is never `Sync` whatever `T` is, so the bet was already contingent on an `Rc`-to-`Arc` change; and
`Resolved` is two `usize`s plus a tag, too wide for a lock-free atomic here.

**The cache has no axis and no evidence** -- zero clause-initial `call` in all ten
`bench-programs/*.rex`, confirmed by the review. Task 11 must not attribute movement to it in either
direction. Same shape as Task 7's `Const`/`TraceLiteral` finding.

**Three things handed to Task 11** (`a8c6aefe`): a **third** ownerless oracle divergence
(`interpret '::routine zfoo'` reaching 99.914 rc 157 while omitting the oracle's first echo line --
the review noted it had not checked whether this was recorded, and it was not, which is the argument
for the list existing); a run with `CALL_SITE_CACHE = false`, which the review verified is a real
switch giving 1423 green but which **nothing in the tree exercises, so it rots silently**; and the
review's instrument suggestion -- a `cfg`-gated assertion that re-resolves and compares on every hit,
turning each of the 10,391 sweep programs into a cache check for one boolean. **The sweep's
blindness to a wrong resolution is structural**, since both arms are this crate, so the next cache
gets no free coverage from it either.

**Task 10: complete. Every promotion in this phase has landed.**

## Phase 4e handoff: three items chased before 4f -- complete

Plan `973973f2`. Commits `8368d357` (item 2), `294469b0` (items 1 and 3); mine `0237e64d` (spec),
`131f1b40` (4f's open question). Suite 1427 to **1428**, dev and release, all four STRICT gates.

**Item 2 built the net, and its two proofs gave opposite answers**, which is why they are separate
claims. Against a promotion that stops firing **outright** it adds nothing -- the existing suite
catches that in four tests, recorded as a more direct signal rather than as coverage. Against a
promotion that stops firing **conditionally** nothing else in the workspace sees it: bounding
`native_shape`'s recursion at depth 8, a change 4f might reasonably want, reads **1427 passed, 0
failed, exit 0** without the assertion and 1 failed with it. Every promoted expression in the golden
set is hand-written and shallow, `deep_nested_expr.rex` nests three thousand terms, and **a bound
between them is invisible to every hand-written witness.** It also rejected the op-kind-count option
I suggested, with a better reason than mine: `ir_dual.rs` already diffs raw stderr over the same
population, and executed bytes beat compiled shape.

**Item 1 named the mechanism and refuted the item's own account of it.** The move is the extraction
-- restoring the pre-split body inline recovers 87% -- but it is **not two calls**: `nm -C` finds no
`arith_small_int` and no `arith_general` symbol in any binary of the range and `callgrind` records no
call edge, because both are inlined into `eval_arithmetic` outright. **What the tree-walker pays is
the code LLVM emits for the shared shape**, which is why the sharing rule's price cannot be
predicted from a call count. A refuted fix is recorded: hoisting the duplicated `digits()` read costs
**78 more** instructions per pass.

**ITEM 3 REFUTED THE GATE'S OWN CANDIDATE, IN THE WRONG DIRECTION, AND THE REFUTATION IS THE
FINDING.** Giving `Op` twelve and twenty-four extra variants with arms in both exhaustive matches --
emitted behind a never-true condition, `size_of::<Op>()` held at 12 -- grows `run_ops`' symbol from
12228 to 28354 bytes and makes the gap **cheaper every time**, 140.001 to 125.004 per pass. So **a
build differing from head only by code that never executes moves `emptyloop` by -15.000 per pass**:
11% of the gap and **71% of the whole per-phase move it was being used to attribute.**

**That refined a rule I had recorded too strongly** (`0237e64d`). 7-M2's control showed no
instruction floor near 0.75% -- true, and its control added code to a function `emptyloop` never
enters. The axis is **whether the function is entered, not whether the code runs**: code in a
function you do not execute is free to 2e-9; code in a function you do execute is not, even when it
never runs. `emptyloop` goes to 4f with that caveat attached rather than as a clean baseline.

**Moritz raised the 16-byte `Op` question** and it is recorded in 4f's open questions rather than
chased here, with the note that item 3's "stride ruled out" bullet answers a different question --
it says the stride did not move during 4e, not that 12 is the right width.
