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

