# Phase 4d-1 implementation plan -- measure, attribute, write the gate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure this crate against the C++ oracle on every axis that can be measured, attribute the gap to named causes carrying numbers, and write `phase-4d-gate.md` -- before any optimisation lands.

**Architecture:** Nothing is optimised in 4d-1. Its output is three artifacts: a reproducible measurement, an attribution document, and a gate. 4d-2 is planned from the attribution and is not planned here.

**Tech stack:** The existing `rexx-time` wall-clock harness (`rust/crates/rexx-bench/src/bin/rexx-time.rs`), criterion for in-crate benches, `samply` 0.13.1 for sampling, `pollard` 0.0.9 for profile analysis.

**Spec:** `docs/superpowers/specs/2026-08-08-phase-4d-performance-design.md`. Read it before Task 1. Where this plan and the spec disagree, the spec governs and the disagreement is a finding.

## Global constraints

Every task's requirements implicitly include this section.

* **The C++ tree at `/home/moritz/dev/repos/ooRexx/` is read-only.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Nothing in this phase optimises anything.** A prototype built to confirm an attribution is published and reverted in the same task. If you find yourself keeping a speedup, stop: that is 4d-2's work and landing it here destroys the bar-before-optimisation property this unit exists to provide.
* **Correctness is not a tradeoff.** No committed suite figure regresses. `cargo test --workspace --no-fail-fast` from `rust/`, clippy clean under `-D warnings`, `cargo fmt --all --check` clean, all read unpiped.
* **Read stdout, stderr and exit status as three separate descriptors.** Never `2>&1`. Never read an exit code from a pipeline.
* **Never run cargo from the repo root** -- exit 101, "could not find Cargo.toml", which reads exactly like a test failure. Run from `rust/`.
* **`grep` is a `ugrep -I` wrapper that silently skips non-UTF-8 files.** Always `/bin/grep -a`.
* **Never report a single figure.** N runs and the spread, every time.
* **`ulimit -v` is applied equally to both sides or to neither, and which is stated.** It caps address space, and **this crate reserves 512 MiB before running anything** (`INTERPRETER_STACK_BYTES`, `rexx-exec/src/lib.rs:309`) while the oracle reserves nothing comparable. Measured 2026-08-08: `say 1` exits 0 on the oracle at `ulimit -v 100000` and fails here (rc 101) at both 100000 and 400000, succeeding at 600000.
  **The project's standard cap of 1048576 does NOT clear the benchmark workloads, and an earlier version of this line said it did.** That claim was checked against `say 1` and generalised, which is the error, not the number: measured at Task 2, this crate aborts under that cap on `varlookup`, `compound`, `strings`, `arith` and `rexxcps`, completing only `startup`. Task 2 used `ulimit -v 8388608` on both sides and verified it clears everything. **Use a cap you have verified against the workload you are running, state it, and apply it to both sides.**
* **Three programs crash the oracle deterministically; never run them:** `select; when 1 = 0 then; when 2 = 2 then nop; end`; `say date('M','0','D')`; any `NUMERIC DIGITS` above 1000.
* **Run oracle probes from a fresh empty subdirectory** of `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad`, absolute paths, fresh **directory** per batch.
* Markdown: one sentence per line, `*` bullets, headers capitalise only the first word and proper nouns, `--` not em-dashes.
* Comments state the contract at the top and reasoning at the decision point. **Do not write counts of mutable in-repo aggregates into comments or prose** -- assert them or omit them (`rust/CLAUDE.md`).

## File structure

| path | responsibility | task |
|---|---|---|
| `docs/superpowers/plans/perf-baseline.md` | corrected; gains the Rust arm | 1, 2 |
| `docs/superpowers/plans/2026-07-27-rust-rewrite.md` | roadmap amended for 4d, "the ratio bar" defined | 1 |
| `docs/superpowers/plans/phase-4-exclusions.txt` | reservation reversal corrected | 1 |
| `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs` | the interleaved two-interpreter harness | 2 |
| `docs/superpowers/plans/phase-4d-retention.md` | the per-iteration retention diagnosis | 3 |
| `rust/bench-programs/alloc4c.rex` | allocation axis, 4c surface | 4 |
| `rust/crates/rexx-core/benches/heap.rs` | GC arm rebuilt like-for-like | 5 |
| `docs/superpowers/plans/phase-4d-attribution.md` | named causes with numbers | 6, 7 |
| `docs/superpowers/plans/phase-4d-gate.md` | the criteria and their derivations | 8 |

---

### Task 1: Correct the record before anything reads it

Three documents carry a claim that is backwards, and later tasks read them. This task runs first so no measurement is designed against a false premise.

**Files:**
* Modify: `docs/superpowers/plans/perf-baseline.md`, `docs/superpowers/plans/phase-4-exclusions.txt`, `docs/superpowers/plans/2026-07-27-rust-rewrite.md`
* Modify: `.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/rexxcps-measurement.md` and `task-5-report.md` if the claim appears there

- [ ] **Step 1: Reproduce the reversal yourself**

Do not take it on faith. From a fresh scratch directory, with `say 1` in a file:

```
for cap in 100000 400000 600000; do
  ( cd "$D" && ulimit -v $cap; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
      /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/t.rex" >/dev/null 2>&1; echo "oracle $cap -> $?" )
  ( cd "$D" && ulimit -v $cap; \
      /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run "$D/t.rex" >/dev/null 2>&1; echo "rust $cap -> $?" )
done
```

Expected: oracle 0 at all three caps; rust 101, 101, 0.

Confirm the constant is ours and not the oracle's: `/bin/grep -rn INTERPRETER_STACK_BYTES` in both trees. It is at `rust/crates/rexx-exec/src/lib.rs:309` and has no hits in the C++ tree.

- [ ] **Step 2: Correct every occurrence**

`perf-baseline.md` around `:262` is the origin; the others follow it. The corrected statement is that **this crate** reserves 512 MiB of address space before running anything, so under a shared `ulimit -v 1048576` this crate has roughly 500 MB of room and the oracle roughly 1000 -- which is what `rust/CLAUDE.md` already says correctly, and is the wording to align to.

**Correct the claim; do not delete the paragraph.** It is load-bearing for how memory findings under that cap are read.

- [ ] **Step 3: Amend the master plan for 4d's existence**

`2026-07-27-rust-rewrite.md` currently says at `:473` "The Phase 4 row closes when 4c closes", its roadmap row at `:442` uses the phrase "the ratio bar" without ever defining it, and 4d appears nowhere.

Make three changes: say the Phase 4 row closes when 4d closes; **define "the ratio bar" at `:442` as the parity shipping gate of Global Constraints `:39`**, not as 4a's R2 threshold of 1.5, which `:40` scopes to Phase 1; and add 4d to the roadmap as a sub-phase alongside 4a, 4b and 4c.

- [ ] **Step 4: Verify no occurrence survives, and commit**

`/bin/grep -arn "512 MiB\|INTERPRETER_STACK_BYTES\|512 \* 1024" docs/ .superpowers/` and read each hit. Commit.

---

### Task 2: The interleaved two-interpreter harness, and the baseline

**Files:**
* Create: `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`
* Modify: `docs/superpowers/plans/perf-baseline.md`

- [ ] **Step 1: Read what already exists before writing anything**

`rust/crates/rexx-bench/src/bin/rexx-time.rs` already times an arbitrary command over N runs after M warmups and reports a median. **Reuse it rather than reimplementing timing.** Its interface is `rexx-time [--warmup N] [--runs N] <command> [args...]`.

`perf-baseline.md` already holds the oracle's Phase 0 wall times. **Do not reuse them.** They were taken at a different commit of the oracle and the phase's own rule is that a baseline is measured at gate time.

- [ ] **Step 2: Write the harness**

`rexx-bench-suite` runs, for each axis: the oracle then this crate then the oracle then this crate, alternating, for N pairs -- **not all of one side then all of the other**. Frequency drift across minutes on this 32-core part exceeds several of the effects being measured, and alternating makes drift common-mode.

It reports per axis:

* **wall time**, median and spread, both sides
* **iterations per second**, both sides, taken from the program's own loop count, which is exact
* **the ratio**, oracle over rust for throughput measures
* the **fixed per-process offset** as its own line, measured by timing a program whose body is `nop`, so a change to startup is visible rather than distributed across every axis

The axis list is a **literal constant in the source**, asserted against the contents of `rust/bench-programs/` so that an axis cannot silently disappear. This is the device `tests/corpus.rs` uses for its subset files; copy that shape.

- [ ] **Step 3: Pin N, the statistic, and the tool, in the source**

Adopt Global Constraints `:39`'s definition of "slower": the point estimate falling outside the C++ baseline's confidence interval on the slow side. Ratios are reported because they are legible; **the gate is decided on interval overlap**, so the harness must emit an interval, not only a median.

- [ ] **Step 4: Fingerprint the oracle**

Record `size`, `mtime` and `sha256` of `/home/moritz/dev/repos/ooRexx/build/bin/rexx` into the report. The project declined a corpus-harness fingerprint deliberately, on the grounds that rebuilds are rare and the user's own; that reasoning does not carry to a phase whose entire output is ratios against that binary, measured once here and again at the end of 4d-2.

- [ ] **Step 5: Run it, commit the report**

Both sides under `ulimit -v 1048576`, stated in the report. Release build both sides; the oracle is a CMake `Release` build and `rust/Cargo.toml` now pins `[profile.release]`.

Record the `arith` result explicitly against **Phase 2's outstanding parity debt** (`d1-decision.md:76`, recorded at 1.22 times and warned to be a lower bound). This is that debt's scheduled re-measurement.

---

### Task 2b: Re-establish the baseline, because five speedups landed after it

Task 2's baseline was committed at `107febcd`. Five speedups landed after it -- `3799692d`, `b6b1d8a9`, `e1d50dda`, `c428ec8a`, `04ab4af6` -- so `perf-baseline.md`'s Phase 4d-1 section no longer describes this interpreter, and every task below that reads it would read a stale number.

**That those speedups landed at all contradicts this phase's own no-optimisation rule** (Global Constraints, `:18`). They were measured and they were the user's call, and the consequence is recorded rather than argued: this unit can produce a *current* baseline and attribution, and cannot produce a pre-optimisation one. The bar-before-optimisation property is partly spent and no document should claim otherwise.

**Staleness is already confirmed and the direction is known.** An indicative re-run on 2026-08-08 gave `arith` 2.60x against the committed 3.52x, `compound` 6.32x against 12.15x, `strings` 10.68x against 13.70x, `varlookup` 4.28x against 23.07x, and an internal cps ratio of 6.21x against 9.33x. **Those figures are not a baseline and must not be quoted as one**: their spreads reached 82.77 per cent on `strings` and 32.84 per cent on `arith`, against the committed baseline's 1.04 to 2.40 per cent, and an interval of 6.32x to 13.37x decides nothing. The machine was not quiet.

**Files:**
* Modify: `docs/superpowers/plans/perf-baseline.md`

- [ ] **Step 1: Re-run the committed harness on a quiet machine**

`./target/release/rexx-bench-suite`, built from `rust/`. Nothing else running -- no background build, no other benchmark, no editor indexing. The harness already interleaves and reports per-side spread; that spread is the check on whether the run is usable.

- [ ] **Step 2: Reject the run if its spread is worse than the committed baseline's**

The committed section's spreads are 1.04 to 2.40 per cent. A run whose spread is materially worse is not a baseline, and re-running it is cheaper than reasoning about which of two bad numbers to trust. Say in the document what spread the accepted run had.

- [ ] **Step 3: Record both figures, with their commits**

Follow the amendment discipline `phase-4c-gate.md` used for its criterion 4: carry **both wordings and the reason**. The old figures stay, marked as measured at `107febcd` and superseded, with the five speedup commits named as what moved them. A reader must be able to see that the numbers changed and why, rather than finding one set silently replaced.

- [ ] **Step 4: Commit**

---

### Task 3: Diagnose the unbounded per-iteration retention

Task 2 measured peak resident set alongside wall time and found this crate between 51 and 218 times the oracle's, which sits near 20 MB on every axis.
Follow-up measurement, 2026-08-08, on `do i = 1 to n; x = x + 1; y = x; end`: retention grew linearly with iterations at roughly 216 bytes each -- 213 MB at one million, 1.06 GB at five, 2.11 GB at ten -- and a literal loop bound behaved identically to a variable one (213,104 KB against 213,028 KB).

**Every figure in the paragraph above is stale, and this task re-measures rather than confirms.** All of it was taken at or before `107febcd`. Two of the speedups that landed afterwards remove per-iteration allocation on exactly the loop shape named here: `b6b1d8a9` does small-integer arithmetic without building a decimal, and `e1d50dda` steps a counted loop without building a `Number` per iteration. The per-iteration constant, the linearity and the RSS multiple may all have moved, and an implementer who sets out to *confirm* 216 bytes will either confirm a number that no longer exists or find a mismatch with no guidance on which figure governs. **Establish the current numbers first, report them against the ones above, and treat the difference as a finding rather than an error.**
The oracle stays flat because it collects.

**This is pulled ahead of the profiling task because it may not be a performance property at all.**
A loop whose live set is two integers should not grow without bound; a long-running Rexx program would exhaust memory.
If that is right it is a defect, its fix is 4d-2's largest single lever, and it plausibly explains a large share of the timing gap on every loop axis -- the smoke profile of `arith` put about a third of self time in the glibc malloc family, which is what unreclaimed per-iteration allocation looks like from the allocator's side.

**This task diagnoses. It does not fix.** The phase's no-optimisation rule binds here exactly as elsewhere.

**Files:**
* Create: `docs/superpowers/plans/phase-4d-retention.md`

- [ ] **Step 1: Reproduce and characterise, before reading any code**

Confirm the linear growth and the per-iteration constant yourself. Vary the loop body and find what the retention is proportional to: iterations, clauses executed, assignments, distinct variables, or arithmetic operations. A body of `nop` against one of `x = x + 1` against one of `y = x` separates several of these in three runs.

**Check the exit status and the output of every probe.** A program that dies on line 1 reports a small, stable, entirely meaningless resident set -- that mistake was made while checking this very finding, and the wrong number looked like a refutation of it.

- [ ] **Step 2: Already answered -- confirm it, do not re-derive it**

**Measured 2026-08-08, before this task was dispatched: nothing triggers a collection automatically.**

`heap.collect` has exactly two production callers. `Interp::alloc_with` (`rexx-exec/src/lib.rs:2091`) calls it **only when `self.stress_collect` is set**, which is the test-only stress mode. The other is the user-callable `GC('Force')` builtin (`builtin/state.rs:234`). There is no allocation-count threshold, no heap-size threshold, and no other trigger anywhere in the crate, so a normal program never collects and the heap grows monotonically until the process dies.

**The collector itself works.** On a two-million-iteration loop of `x = x + 1; y = x`, both runs exiting 0 with correct output: plain peaks at 423,892 KB, and the same loop calling `gc('Force')` every hundred thousand iterations peaks at 122,880 KB.

So this is **a missing trigger policy, not a broken collector and not a root leak**, and the three-outcome question this step used to ask is settled at the first branch.

Confirm the two call sites still read that way at the commit you are working from, then spend the effort on what is *not* known:

* **What the 216 bytes per iteration actually are.** The loop's live set is two integers, so name what is allocated per iteration and why. `x = x + 1` and `y = x` between them allocate a number and rebind a variable; that should not cost 216 bytes retained.
* **Why forcing a collection every hundred thousand iterations still leaves 123 MB** rather than the roughly 21 MB those iterations should account for. The likely answer is that the arena is a high-water mark and `collect` reclaims into free lists without returning pages, which would make peak RSS a measure of the largest interval between collections rather than of live data. **Confirm or refute that** -- it decides whether a trigger policy alone would fix the observed figures or only bound them.
* **What a trigger policy would cost in time**, per Step 4. This is the number 4d-2 needs and the one nobody has.

- [ ] **Step 3: Name the retained object and the root that holds it**

Whatever Step 2 says, end with a specific answer: which allocation, held by which root, released by what if anything. `roots.push_temp` and the frame discipline in `run.rs` are the obvious places to look, and `crates/rexx-core/src/roots.rs` defines the root set.

**Use a heap profiler rather than reading code and guessing.** `samply` answers where time goes and is the wrong instrument here; the question is what is held and by whom.

`valgrind --tool=massif` is installed and needs no setup. It reports heap size over time plus the allocation tree with call stacks at each peak snapshot, which is exactly the shape of this question. Use `--stacks=no --detailed-freq=1` and read the result with `ms_print`.

**Valgrind's 20-to-50-times slowdown does not matter here, and the reason is worth understanding rather than working around.** The retention is *linear* in iterations, so it reproduces at any scale: 100,000 iterations retain roughly 21 MB, which is ample signal. Run the small loop under the slow instrument rather than trying to make the big loop fast. Confirm linearity holds at the small end before relying on it.

If massif's C-level stacks are hard to attribute to Rust call sites, the `dhat` crate is a pure-Rust alternative -- a dev-dependency plus a feature-gated global allocator, no system install -- and it names Rust frames directly. `heaptrack` would be better than either and is **not installed**; installing it needs root, so ask rather than attempting it.

- [ ] **Step 4: Quantify the time cost, by prototype, then revert**

Confirm the attribution the way a bug fix is confirmed: change it, show both the retention and the wall time move, revert it. **Publish the measured win in `phase-4d-retention.md` and revert the prototype in the same commit.** Back up with `cp` and restore from the backup, verified with `sha256sum -c`; never `git checkout --`.

If a prototype is not tractable within this task, say so and record what you would need -- an unquantified cause is still a finding, and a wrong number is worse than none.

- [ ] **Step 5: Rule on whether this is a defect, and record the coverage gap either way**

If it is a defect, say what a user would see and record it in `docs/superpowers/plans/phase-4-exclusions.txt` in that file's own style.

**Regardless of the ruling, record this:** nothing in the differential suite can see unbounded growth, because every corpus program is small and the harness compares output rather than resident set. That is a coverage gap in the project's primary instrument, and it is why this reached Phase 4 unnoticed.

- [ ] **Step 6: Commit**

---

### Task 4: Make the allocation axis measurable

`alloc.rex` stops at `rc=120`, "a message send is not implemented (Phase 5)". Allocation throughput does not need message sends, and 4d-2 will land representation changes -- **landing them with this axis unmeasured is the worst available ordering.**

**Files:**
* Create: `rust/bench-programs/alloc4c.rex`
* Modify: `rust/bench-programs/README.md`, `docs/superpowers/plans/perf-baseline.md`

- [ ] **Step 1: Read `alloc.rex` and find what it uses that 4c lacks**

Run it on the oracle and here, and record both. Identify the exact construct that blocks.

- [ ] **Step 2: Write `alloc4c.rex` over the 4c surface**

It must allocate on the same order and in the same proportions as `alloc.rex` does, using only constructs this crate implements -- string concatenation, compound-variable creation and numeric temporaries all allocate and are all available.

**State in the file's own header what it does and does not preserve from `alloc.rex`.** A variant that avoids the blocking construct by measuring something else is not the same axis, and the honest handling is to say which parts of the original axis it covers.

- [ ] **Step 3: Confirm it runs on both sides and produces identical output**

Byte-identical stdout, exit 0 both sides. An interpreter that computes something different is not faster.

- [ ] **Step 4: Add it to the harness's axis list and re-run, then commit**

---

### Task 5: D1's Phase 4 re-measurement -- the GC arm, rebuilt

`d1-decision.md:19-22` records the Phase 1 heap result as a debt rather than a pass and says it "must be re-measured at Phase 4 when a real interpreter exists to measure on equal footing". This task is that re-measurement.

**Files:**
* Modify: `rust/crates/rexx-core/benches/heap.rs`, `docs/superpowers/plans/d1-decision.md`

- [ ] **Step 1: Establish that the recorded figure is not reproducible, and say so**

The C++ arm is `TIME('E')` around `GC('F')` on `heapshape.rex` (`d1-decision.md:61`). The Rust arm is `heap.rs`'s `collection`/`full_gc_1m_graph` criterion bench.

Commit `a3178cff` replaced `Body::String(String)` -- the variant D1's risk analysis names -- with `Body::Text`. **So re-running the bench today measures a different representation than the recorded number, and the comparison must be rebuilt rather than re-run.** Record this in `d1-decision.md` rather than quietly quoting a new number against an old one.

- [ ] **Step 2: Rebuild the two arms to be like-for-like**

The C++ arm is a whole-program GC pause on a graph `heapshape.rex` builds. The Rust arm must build a graph of the same shape and count, and time a full collection. Where the shapes cannot be made identical, say which and in which direction the difference cuts.

`heapshape.rex` builds its strings with `"e" || j` rather than a bare literal (`d1-decision.md:67`); preserve that, because it determines whether the strings are distinct heap objects.

- [ ] **Step 3: Measure, and record against parity**

Write the result into `d1-decision.md` as the Phase 4 re-measurement the document schedules, with its date and commit. If it misses parity, say so plainly -- `d1-decision.md:50-56` pre-registers the side byte-arena as the first thing to try, and that recommendation becomes live input to 4d-2 rather than a conclusion drawn here.

- [ ] **Step 4: Commit**

---

### Task 6: Profile every axis and attribute the gap

**Files:**
* Create: `docs/superpowers/plans/phase-4d-attribution.md`

- [ ] **Step 1: Settle the dominant question first**

The spec records a hypothesis that must be answered before any decomposition: **the per-axis ratio spread may be a property of the denominator rather than of this crate.** A smoke reading put the Rust side in a narrow band while the oracle spans an order of magnitude.

Answer it from Task 2's absolute throughput numbers, not from the ratios. If the Rust side is roughly flat across axes, there is one dominant cause and a per-axis decomposition would produce several tasks attacking the same thing. **Write the answer first**, because it determines the shape of everything below it.

- [ ] **Step 2: Profile each axis**

```
samply record --save-only -o <axis>.json ./target/release/rexx-run <program>
```

Analyse through `pollard`: `load_profile`, then `top_functions` with `expand_inlines` where inlining hides the callee, then `call_tree` for the hot paths. Check `unsymbolicated_pct` on load -- it should be near zero now that `[profile.release]` sets `debug = true`; a high value means the binary is not the one you think.

- [ ] **Step 3: Name causes, not axes**

**A cause is the unit, and it carries the set of axes it predicts it will move.** The benchmark programs are not axis-pure: `varlookup.rex`'s inner loop is `x = x + 1`, which is decimal addition inside the axis named "variable lookup". One-cause-per-axis is wrong by construction.

**Each cause is a falsifiable claim with a number:** what share of self time it accounts for, on which axes, and the ratio predicted if it were removed. A cause without a number is not a cause -- "varlookup is slow because variable lookup is slow" satisfies the letter of this step and is worthless.

- [ ] **Step 4: Answer D9's outstanding question about compound access**

D9 `:389` says memoisation was to be built into the Rust stem design "from the start rather than porting the slow shape first and optimising later". Read the stem and compound-variable code and say whether that was done. This is a question about this crate, answerable by reading it.

D9 `:390` also points at a prior performance profile. **It is not in this repository** -- it lives at `/home/moritz/.claude/projects/-home-moritz-dev-repos-ooRexx/memory/oorexx-performance-profile.md`, and it describes the **C++** interpreter. It constrains where to look; it is not a profile of this crate, and re-deriving this crate's profile is exactly this task's job.

- [ ] **Step 5: Prototype where an attribution needs confirming -- then publish and revert**

A cause is confirmed the way a bug fix is: change it, show the number moves, revert it. **Publish the prototype's measured win in the attribution document and revert the prototype in the same commit.** The bar written in Task 7 is then visibly downstream of a number already on the record, rather than one invented to match what 4d-2 was going to achieve.

Back up with `cp` before mutating and restore from that backup, verifying with `sha256sum -c`. Never `git checkout --`.

- [ ] **Step 6: Commit the attribution**

---

### Task 7: The allocator diagnostic

A smoke profile of `arith` -- the axis closest to the oracle -- put roughly a third of self time in the glibc malloc family with the crate's own allocation path a further five per cent on top. One short run of one axis, so Task 6 supersedes it.

This task exists for what it **rules out**, not for what it wins.

**Files:**
* Modify: `docs/superpowers/plans/phase-4d-attribution.md`

- [ ] **Step 1: Swap the global allocator, temporarily**

A `#[global_allocator]` change, on a scratch branch or an uncommitted edit backed up with `cp`.

- [ ] **Step 2: Re-measure every axis and interpret the result as a fork**

* Recovers most of that share -> the cost is allocator **quality**, and adoption is a 4d-2 candidate.
* Recovers little -> the cost is allocation **count**, and the fix is not to call the allocator at all, which is D1's pre-registered side byte-arena.

**Expect the second.** The oracle does not call libc malloc per object; it allocates from its own pools (`MemoryObject`, `DeadObjectPool`, segment allocator). A per-value malloc is a difference in kind, and a faster malloc narrows it without removing it. Record the result either way -- the informative outcome is the one that contradicts this expectation.

- [ ] **Step 3: Check adoption feasibility before recommending it**

The parity gate names Linux and macOS; CI runs five platforms. An allocator that does not build everywhere the interpreter ships is not a candidate. **Check, do not assume.**

- [ ] **Step 4: Revert the swap, verify the tree, publish the finding**

This is a runtime behaviour change, so unlike `lto` it does **not** go into the baseline. Verify the revert with `sha256sum -c` and confirm the suite is green.

---

### Task 8: Write the gate

**Files:**
* Create: `docs/superpowers/plans/phase-4d-gate.md`

- [ ] **Step 1: Write the criteria before reading Task 6's numbers again**

Model the document on `docs/superpowers/plans/phase-4c-gate.md`, which renders **MET** or not per criterion in a table.

**The bar is parity, unamended** -- Global Constraints `:39`, decided 2026-08-08. Not 4a's R2 threshold of 1.5, which `:40` scopes to Phase 1's viability check.

- [ ] **Step 2: Write each bar's derivation, not only its value**

"Axis X's bar is the ratio implied by removing cause C's measured self-time share" is checkable by a later reader. A bare number is not. This is the step that makes the bar a prediction rather than a description.

- [ ] **Step 3: Apply the vacuity test to every criterion**

**Ask of each: what degenerate execution satisfies this, and would deleting its subject leave it green?** This project has shipped criteria satisfied by shrinking their own denominator, and one whose stated falsification procedure was measured and did not falsify.

Two specific to this gate:

* **A criterion that gates on a ratio containing process startup is improvable by cutting startup with nothing landing in the interpreter.** The spec makes the offset visible; visibility is not closure. **Decide here** whether the gated measure excludes the fixed offset, and say which.
* **`startup` is not comparable at 4d and must not be recorded as passing.** This crate has no `CoreClasses.orx` bootstrap, so it starts fast by doing none of the work the oracle does. Record it as not comparable, name D2's absolute target of about 55 ms for 5,203 lines (`plan:157`), and gate nothing on it. D2's decision is already made and is **(a), no saved image** (`plan:151`).

- [ ] **Step 4: Record what cannot be measured, by name**

`dispatch` goes to Phase 5: a dispatch benchmark that avoids message sends is a different benchmark. State it as a D9 dimension this phase does not cover.

**macOS is in the parity gate's text** and is not available in this session. Record Linux parity as met or not, and macOS as outstanding, with the gate explicitly incomplete on that axis rather than silently Linux-only.

- [ ] **Step 5: Record the stopping rule and the amendment rule for 4d-2**

4d-2 ends when every axis meets its bar, or when a task's measured result contradicts the attribution -- whichever comes first. Any later change to a bar carries **both wordings and the reason**, the way `phase-4c-gate.md` recorded its criterion 4 amendment, which is the only reason that amendment survived review.

- [ ] **Step 6: Commit, and state that 4d-2 is not planned here**

---

## Explicitly not in scope

* **Any optimisation.** Prototypes are published and reverted.
* **`dispatch`**, which needs Phase 5.
* **Planning 4d-2**, which is planned from the attribution once this unit closes.
* **Adopting an allocator**, which is 4d-2's decision against the bar.
