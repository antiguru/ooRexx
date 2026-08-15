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

