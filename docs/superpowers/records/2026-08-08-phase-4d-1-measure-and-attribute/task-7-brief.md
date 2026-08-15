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

