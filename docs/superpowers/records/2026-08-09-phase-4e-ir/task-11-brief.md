### Task 11: The gate

**Files:**
- Create: `docs/superpowers/plans/phase-4e-gate.md`
- Modify: `docs/superpowers/plans/2026-07-27-rust-rewrite.md` at `:442` and `:473`
- Modify: `rust/crates/rexx-exec/src/invocation.rs` (the default flips)

Landing this phase edits **two** lines of the master plan: the roadmap row at `:442` and `:473`. `:473` was already amended once by `64ee0369` as 4d-1's Task 1, so this is a second amendment to that line. `:459`'s S0 entry is left alone. This task owns those edits; an earlier draft assigned them to nobody.

- [ ] **Step 1: Flip `Engine`'s default to `Ir`**, so the IR is the default at both body-entry points. Run the whole workspace suite in both profiles and the corpus under STRICT.
- [ ] **Step 2: Run the seven exit criteria** from the spec's `## Exit gate`, and write each one's result with what it could not see. **Criterion 4 no longer carries a floor** -- it was withdrawn on 2026-08-10 at `8e7ce246` and the spec is authoritative. What it now asks for is a **recorded ratio per axis on both instruments** -- instructions and cycles, from the paired interleaved comparison between the two arms of one build -- **plus the structural residual itemised**, carried forward from Task 7-M. A ratio reported without an instrument named, or a residual without a breakdown, does not meet it. Wall clock is not an in-phase instrument.
- [ ] **Step 3: Record the resulting ratios as Phase 4f's input state**, superseding `phase-4e-anchor.md`'s tree-walker-only figures and saying so.

**This task inherits every measurement Tasks 9 and 10 no longer take (2026-08-11, Moritz), and the list is exhaustive so nothing is lost by deferring.** `rexx-arms` builds and measures any commit after the fact, so these are archaeology of the same kind Task 7-M2 ran over Task 7-M, and the commits are immutable.

- [ ] **Step 3a: Measure per-commit, not only at head.** One sitting per landed task boundary -- Task 7-M3, Task 9, Task 10 -- so a move is attributable to the task that caused it. **Head-only figures cannot satisfy this criterion's second half**, which asks whether an axis moved for the reason a promotion predicted; an aggregate cannot answer that. Each task recorded a prediction and no measurement; check each against what you measure and say which predictions held.
- [ ] **Step 3b: Delete Task 9's patch table and confirm the number moves.** Exit criterion 5's falsification: a win that survives deleting the table was static specialisation, and a dead `AtomicU32` beside a statically specialised op would otherwise pass. Task 9 was told to leave the removal as a single guard, so this is one build.
- [ ] **Step 3c: Price the `AtomicU32` load against a plain `u32` read** on the quickened path, at the same two sizes. D22's stated reason -- that routine bodies are shared across activities and `Cell` is not `Sync` -- does not bite yet, since chunks are held as `Rc` in a per-`Interp` map. The rule stands as forward-compatibility; this is what it costs, and if it is not free the number belongs to Phase 6's ledger.

  **And decide whether the bet still earns it, because Task 10 voided it (measured 2026-08-11 by Task 10's review).** `assert_sync::<Chunk>()` fails to compile at head naming **exactly one** culprit: `Cell<Option<Resolved>>` in `CallSite`. "Exactly one" is the load-bearing part -- rustc names no other field, so `PatchSlot` and everything else is `Sync`, and `Chunk` *was* `Sync` before Task 10. **So the `AtomicU32` buys nothing a `Cell<u32>` would not, for as long as `CallSite` stays a `Cell`.** Two riders for whoever revisits it: the cache hands out `Rc<Chunk>`, and `Rc<T>` is neither `Send` nor `Sync` whatever `T` is, so the bet was already contingent on an `Rc`-to-`Arc` change; and `Resolved` carries two `usize`s plus a tag, which is too wide for a lock-free atomic here. **Two locally correct choices whose combination pays a cost for nothing** -- record the decision either way rather than leaving the atomic in place by default.
- [ ] **Step 3e: Run with `CALL_SITE_CACHE = false`.** Task 10's review flipped it and got 1423 passing with both gates, so the one-line-removal claim holds -- but **nothing in the tree runs that configuration, so it will rot silently.** This is the same shape as Step 3b's `QUICKENING` switch and it costs one build.
- [ ] **Step 3f: Consider the cache check that costs one boolean.** Task 10's review suggests a `cfg`-gated assertion that re-resolves and compares on every hit, which turns each of the 10,391 sweep programs into a cache check. **The sweep's blindness to a wrong resolution is structural** -- both arms are this crate -- so a second cache gets no free coverage from it, and this is the instrument that would change that. Recording the suggestion is required; building it is Phase 4f's call.
- [ ] **Step 3d: Record the enter/leave trade at final promotion coverage.** Task 7-M3 landed it when `emptyloop` paid 24 instructions per pass against `varlookup` and `alloc4c` coming in under 1.0. Tasks 9 and 10 change the promoted-to-`Generic` ratio the trade turns on, so measure it again here and say whether the trade got better or worse.
### Oracle divergences this phase found and did not fix, which nothing else owns

**Added 2026-08-11.** Both have the same shape and it is the shape that falls through every net here: **the two engines agree with each other and the oracle differs from both.** So neither fits `ir_dual.rs`'s `KNOWN_DIVERGENCES`, which records engine-against-engine drift, and neither fits a corpus exclusions file, which lists programs rather than behaviours. Each was recorded only in the report of the task that found it, and a report is read once -- which is how the first of these was nearly lost, and why they are here instead.

**Task 11 reports both under criterion 1's "what it could not see".** Neither is a defect this phase introduced, and neither blocks the gate.

* **`TRACE VALUE expr` emits no `>K>` line** (found by Task 6, whose report has the site and the one call that would fix it). The oracle traces a `TRACE VALUE` clause's own computed setting as a keyword value; this crate does not.
* **A `CALL ON` handler delivered at a promoted loop header's boundary indents its own clauses two spaces less than the oracle** -- `13 *-*   h:` against the oracle's `13 *-*     h:` (found by Task 7-M3's review, which rebuilt `7a7f5849`'s three files and reproduced identical bytes, so the enter/leave split did not introduce it).
* **`interpret '::routine zfoo'` reaches the right error, 99.914 at rc 157, and omits the oracle's first echo line** (found by Task 10's review while probing, which notes it did not check whether this was already recorded -- it was not).

**Neither has been checked against the other engine's *own* trace fidelity work**, and whoever takes Phase 4f's trace parity should start from this list rather than rediscovering it.

- [ ] **Step 4: Run clippy from a clean target directory.** A warm target makes a green provisional.

```bash
cd rust && cargo clean && cargo clippy --workspace --all-targets -- -D warnings
```

- [ ] **Step 5: Edit the two master-plan lines and commit.**

## What this plan does not do

* **It does not reach parity.** That is Phase 4f's exit condition, and 4f runs after this phase. This plan's performance obligation is a floor, not a target.
* **It does not compile `INTERPRET` fragments**, per the decision above.
* **It does not build message sends.** Phase 5 needs the patchable call site this phase creates; it does not need this phase to anticipate it. D24's forward constraints -- selectors interned at compile time, a `SmallInt` behaviour arm, a receiver in the calling convention, a wider patch entry, and invalidation on behaviour mutation -- are recorded there, not built here.
* **It does not run on five platforms.** The inherited gate names Linux and macOS, `:35` requires five, and no platform runs the Rust suite automatically today. A dual-engine gate doubles whatever manual process exists. This is carried as an open question, not assumed away.

## Open questions

* **macOS, and CI at all**, as above.
* **How much a `TRACE` change inside a loop costs** once the setting is an input to compilation and a change invalidates the cached chunk. Task 6 measures it; if it is expensive, the answer is a decision this plan does not pre-empt.
