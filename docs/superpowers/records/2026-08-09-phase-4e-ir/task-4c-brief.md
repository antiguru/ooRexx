### Task 4c: Flatten the loop header -- after Task 6

Blocked on trace ops, per the amended Decisions section: the header interleaves evaluation and `>K>` emission, so it cannot use a general `EvalExpr`. Takes Task 4b's machinery and Task 6's trace ops and finishes what Task 4a began.

**Predicted movement:** `emptyloop` and `varlookup`. State it against `phase-4e-anchor.md` before measuring, and remember that Task 4a's own unexplained movement is not a baseline.

### Task 4 (original): Promote `Do` and `Loop` -- superseded by 4a, 4b and 4c

Kept for its steps, which 4b and 4c draw on. Without it no benchmark moves, and there is nothing to measure the clause unit with, because compile cost only amortises across repeated execution.

**Predicted movement:** `emptyloop` and `varlookup` are the axes this touches; state a predicted percentage against `phase-4e-anchor.md` **before** measuring, and record predicted beside measured whichever way it comes out.

**Files:**
- Modify: `rust/crates/rexx-exec/src/ir/compile.rs`, `ir/drive.rs`, `run.rs`
- Test: `rust/crates/rexx-exec/src/ir/golden_tests.rs`, `rust/crates/rexx-exec/tests/ir_dual.rs`

**Interfaces:**
- Produces: `Op::Clause { end }` and `Op::EvalExpr` emitted for the first time; `Op::Jump { target: u32 }`, `Op::JumpUnless { cond: u16, target: u32 }`, and whatever the loop's control-value step needs. The shared step is extracted from `run_loop` so both engines call it.

**`Op::Clause` is on that list because a promoted clause emits one**, per the Decisions section: "a promoted clause takes a mark when its `Clause` op is emitted and releases to it at `end`". An earlier version of this line omitted it, which made Task 2's `#[expect(dead_code, reason = "Task 4 is this variant's first constructor")]` look wrong against the plan when the annotation was right and the plan was incomplete. Task 6 later *splits* `Clause`; it does not introduce it.

**Expression evaluation stays on `eval.rs` in this task.** `Op::EvalExpr` calls the same function `step` calls, so trace output is identical by construction and Task 6's trace ops are not a prerequisite.

- [ ] **Step 1: Read `run_loop` before touching anything**

It dispatches five `LoopKind` variants and refuses `COUNTER` and `LoopKind::With` before its match; `WHILE`/`UNTIL` is an orthogonal `Loop.conditional` threaded through it and `run_repeating`; there is a `LEAVE`/`ITERATE` label search; and there is a per-step control-variable re-read that a reviewer once caught as a false unreachability claim. `LoopKind::Over` is implemented only in its documented single-iteration form for non-stem targets.

- [ ] **Step 2: Write the golden test for the compiled loop**

Assert the exact op stream for `do i = 1 to 3; nop; end`. It fails until the compiler emits it.

- [ ] **Step 3: Write the dual-engine behaviour tests first**

Before any compiler change, add the loop programs to `ir_dual.rs`: counted, `WHILE`, `UNTIL`, `FOREVER` with `LEAVE`, nested with a labelled `ITERATE`, and `DO OVER`. They pass now (both arms delegate) and must still pass after.

- [ ] **Step 4: Extract the loop semantics**

Whatever the native ops need -- the control-value step, the condition test, the label search -- becomes a function on `Interp` that `run_loop` also calls. Promotion extracts shared semantics; it does not write a second implementation.

- [ ] **Step 5: Write the register allocator**

It lands here, with its first caller, rather than in Task 2, where it would have none and `-D warnings` would fail on `dead_code`. The discipline is already decided; implement it, do not redesign it.

```rust
/// Compile-time register allocation, stack-disciplined.
///
/// A clause releases to its entry mark at its `end`. A construct whose
/// state outlives its member clauses allocates in the enclosing scope,
/// before emitting them, so those releases cannot reclaim it. The chunk
/// reserves the high-water mark once, on entry to the driver.
struct Registers {
    live: u32,
    high: u32,
}

impl Registers {
    fn mark(&self) -> u32 {
        self.live
    }

    fn alloc(&mut self) -> Result<u16, ChunkTooLarge> {
        let index = u16::try_from(self.live)
            .map_err(|_| ChunkTooLarge { what: "registers" })?;
        self.live += 1;
        self.high = self.high.max(self.live);
        Ok(index)
    }

    fn release(&mut self, mark: u32) {
        self.live = mark;
    }
}
```

- [ ] **Step 6: Write the allocator's own test before using it**

```rust
#[test]
fn a_release_reclaims_registers_but_not_the_high_water_mark() {
    // The chunk reserves `high`, so a release that lowered it would hand
    // the driver a region smaller than the stream addresses.
    let mut r = Registers { live: 0, high: 0 };
    let mark = r.mark();
    r.alloc().unwrap();
    r.alloc().unwrap();
    r.release(mark);
    assert_eq!(r.alloc().unwrap(), 0, "reuses the reclaimed index");
    assert_eq!(r.high, 2, "the reservation still covers the peak");
}
```

- [ ] **Step 7: Emit the ops**

The loop's control registers allocate in the enclosing scope, before the member clauses are emitted, so the release at each clause boundary cannot reclaim them.

- [ ] **Step 8: Run everything**

```bash
cd rust && cargo test -p rexx-exec --lib ir:: && cargo test -p rexx-exec --test ir_dual
cd rust && REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1 REXX_BIF_GATE=1 REXX_KEYWORD_GATE=1 \
  cargo test -p rexx-exec --test ir_dual
cd rust && cargo test --workspace && cargo test --workspace --release
```

- [ ] **Step 9: Measure, interleaved, both arms of the same binary**

Both arms are the same build by construction, so the build-identity problem does not arise. Interleave arm A, arm B, arm A within one sitting. Per Phase 4f's rule, **if a handful of paired runs cannot show a movement, there is none** -- record that outcome rather than adding runs.

- [ ] **Step 10: Falsify any claimed speedup by reverting**

- [ ] **Step 11: Record predicted versus measured, and commit**

Append to `phase-4e-anchor.md`. A movement that arrives by a route other than the stated hypothesis has not confirmed the hypothesis, and says so.

---

## Decisions this plan makes that the spec left open

These are the items the spec left to the plan, decided here so no task rediscovers them. The golden serialisation format and the engine-comparison method are decided inside Tasks 2 and 4 respectively, at the point they are first used. The platform question is not decided and is carried to the end of this document.

**Register allocation is a compile-time stack, and the chunk records its high-water mark.**
The spike's whole-chunk monotonic counter is withdrawn: it allocates a register per assignment and never reuses one, so a long body reserves a region proportional to its length.
The allocator exposes `mark()`, `alloc()` and `release(mark)`.
A promoted clause takes a mark when its `Clause` op is emitted and releases to it at `end`.
A construct whose state outlives its member clauses -- a loop's control value is the case that exists -- allocates in the *enclosing* scope before emitting those clauses, so the release at each clause boundary cannot reclaim it.
`Chunk::registers` is the maximum the counter ever reached.
This is the discipline that survives nesting, which the per-clause reset does not and the monotonic counter does only by wasting space.

**Compilation is whole-body and lazy: one body at a time, on first entry, cached.**
Not whole-program ahead of time, and not ad hoc per clause.
Within a body every instruction compiles and nothing refuses, which is D21; across bodies, a body that never runs is never compiled.

Four reasons, and the last is the one that decides it rather than merely favouring it:

* It is `plan_for`'s discipline exactly, under the same key, so there is one cache shape rather than two.
* `startup` is a benchmark axis. A program with many `::ROUTINE` directives that calls two of them would pay for all of them ahead of time, and pay it on the axis that measures exactly that.
* An `INTERPRET` fragment is not knowable before it runs, so a lazy entry point has to exist whatever happens. Ahead-of-time compilation would be an addition to it, not a replacement for it.
* **The trace setting is an input to compilation (D23), and it is not known at parse time.** `TRACE` is dynamic: `TRACE VALUE expr`, a mid-program `trace`, and inheritance across activations. A body compiled at parse time under the initial setting would have to be recompiled the first time it is entered under a different one, which is the lazy path with an extra wasted compile in front of it.

**A consequence Task 6 owns, and it is a change to Task 2's work rather than an addition to it.** Once the trace setting is a compilation input, `BodyKey` alone no longer identifies a chunk, and `chunk_for` returning a cached chunk without consulting the current setting is wrong. Task 2 writes the cache keyed on `BodyKey`, which is correct while trace is not yet an input; Task 6 widens the key or evicts on a setting change, and measures what that costs.

**A fragment does not compile to a chunk in this phase, and `run_fragment` keeps its tree-walker path.**
D16 declined to cache fragment *plans* because "any per-parse key misses every lookup while retaining every entry, and `do 1000000; interpret s; end` would accumulate a million dead plans", leaving text-keyed caching to a later phase "and only if it can show a hit rate".
Keying on text fixes the miss half and not the retention half, and this plan does not have a retention bound to offer.
Declining to cache settles the rest by itself: an uncached chunk is compiled once per execution and thrown away, which is pure added cost over running the fragment on the tree-walker, so there is nothing to gain by compiling it.
`INTERPRET` is on the spec's "may remain delegated" list, so this costs the phase no promotion.
**The consequence, stated so it is not discovered later:** `run_fragment`'s re-entrancy question does not arise in this phase, and the spec's `## INTERPRET` section describes a later phase's work rather than this one's.

**The compiler has one error, and it is a machine width rather than a language construct.**
"Every instruction compiles, nothing refuses" is about instructions. It is not a claim that every body fits in the index widths, and the two must not be conflated.
Register indices are `u16`; op, constant and instruction indices are `u32`.
`compile` returns `Result<Chunk, ChunkTooLarge>`, and the one way to get the error is a body whose live register count exceeds `u16::MAX` or whose op stream exceeds `u32::MAX`.
Engine selection treats `ChunkTooLarge` as "run this body on the tree-walker" and bumps `Interp::chunks_refused`.
**The counter is what stops that being a silent fallback:** the dual-engine harness asserts it is zero across the whole corpus, so a compiler that starts refusing ordinary bodies goes red instead of quietly running everything on the tree-walker and passing.

**Trace ops land before the first native expression op, not with it.**
S4 measured a promoted assignment dropping the literal's `>L>` line under `trace i`, because `eval.rs` emits it as a side effect of evaluating and a native `Const` op does not.
So Task 6 lands the trace instructions, and Tasks 7 onward -- the ones that replace an `eval.rs` call with a native op -- depend on it.
Tasks 4 and 5 promote control flow only and keep expression evaluation on `eval.rs` through `Op::EvalExpr`.

**Amended 2026-08-09, and the amendment re-sequenced the phase.** The sentence above used to end "which is trace-identical by construction because it is the same call". That is false for a loop header, and Task 4a found it by reading the code rather than by arguing. `setup_controlled` interleaves evaluation and trace: it evaluates `TO`, emits `>K>`, evaluates `BY`, emits `>K>`, in `ctrl.order` order. An op that only evaluates into a register cannot reproduce that ordering. An op that *does* reproduce it is the whole evaluate-and-trace unit, which makes it loop-header-specific rather than the general expression op Tasks 7 to 9 need, and bolting a trace keyword onto `EvalExpr` would pre-empt D23's decision that trace becomes explicit instructions.

So `EvalExpr` is trace-identical only where the evaluation it wraps emits nothing between evaluations. That holds for a branch condition and does not hold for a loop header, which is why the loop header's flattening now waits for Task 6 and `If`/`Select` goes first.

## Global Constraints

* **The C++ tree at `/home/moritz/dev/repos/ooRexx/` is the oracle and is read-only.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Wrap every oracle invocation:** `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`. Use `ulimit -v 8388608` for benchmark workloads.
* **Three programs crash the oracle; never run them:** `select; when 1 = 0 then; when 2 = 2 then nop; end`; `say date('M','0','D')`; any `NUMERIC DIGITS` above 1000.
* **Run every oracle probe from a fresh empty subdirectory you `mkdir` yourself, with absolute paths.** The scratchpad root is on the oracle's external-routine search path and a leftover `.rex` file gets called as an external routine.
* **A symbol named `x` or `b` followed by a quoted string parses as a hex or binary literal.** Use `n1`, `cv`, `zz` in probes.
* **Use `/bin/grep -a`, never bare `grep`** -- the wrapper is ugrep with `-I` and silently skips binary files.
* **Read stdout, stderr and exit status as separate descriptors. Never `2>&1`.** Never read a cargo exit code from a pipeline.
* **Never run cargo from the repo root.** Run from `rust/`.
* **`cargo fmt --all --check`**, not `cargo fmt --edition 2024 --check`, which is an error rather than a check.
* **`cargo clippy --workspace --all-targets -- -D warnings`.** A warm target directory makes a green provisional; run from a clean target at the phase gate.
* **`cargo test <name>` exits 0 when it matches nothing.** Read the run count.
* **`cargo test --release` is a distinct gate** -- `lto = "fat"` changes behaviour, which `5253a674` recorded.
* **Mutation runs need `--no-fail-fast`**, or the suite stops at the first catcher and "nothing else caught it" is unmeasured.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
* **No em-dashes in comments; use `--`.** No counts of mutable in-repo aggregates in prose.
* **Markdown: one sentence per line, `*` bullets, first-word-only capitalisation including after a colon.**
* **Back up with `cp`, restore from the backup, verify with `sha256sum -c`.** Never `git checkout --`, never `git add -A`, never `git reset --hard`, never force-push.
* **Benchmark comparisons interleave between arms within one sitting.** Never read a comparison across two separate runs.
* **Commit first, then read the hash back with `git log`, then quote it.**


---

## Ambiguities the controller resolved for this task

* **`Engine::TreeWalker` stays the default.** Task 11 flips it.
* **Task 6 has landed, so your blocker is gone.** Trace is an emission decision: the setting decides
  which trace ops a body compiles to, and a per-clause staleness check makes a stale chunk slow
  rather than wrong. `Op::TraceClause` is the compiled echo site.
* **New case tables use `datadriven`** -- see `tests/ir_dual_cases/trace-settings` for the shape.
  **A file holding oracle-measured bytes must refuse to run under `REWRITE`**, and carry the oracle
  capture command in its header instead: `REWRITE` would replace the oracle's bytes with the
  implementation's and silently turn a differential test into a self-consistency test. Do not
  retrofit `LOOP_CASES`, `BRANCH_CASES` or `KNOWN_DIVERGENCES`.
* **Splitting is legitimate.** Task 4 split for exactly this construct and the split was cheap
  because it stopped early.

## Why this task was blocked, which is also its main hazard

Task 4a tried to flatten the loop header and stopped. `setup_controlled` **interleaves evaluation
and trace emission**: it evaluates `TO`, emits `>K>`, evaluates `BY`, emits `>K>`, in `ctrl.order`
order. A general "evaluate into a register" op cannot reproduce that ordering, and an op that can is
the whole evaluate-and-trace unit.

Task 6 is what unblocks you, and the shape it establishes is that **the emission is a separate op**
rather than a side effect of the evaluation. Build the header the same way: the order in
`ctrl.order` is the order of ops.

## The tripwire two earlier tasks left you deliberately

`the_ir_engine_steps_a_loop_body_from_the_chunk` and
`the_ir_engine_steps_a_simple_blocks_body_from_the_chunk` are **exact clause-count assertions**, and
they are the only thing in the workspace that can distinguish a promoted body from an unpromoted
one.

**Flattening the header will change those counts. That is expected.** Derive the new count from what
the stream now contains and state in your report why it changed. **A task that adjusts the numbers
until the tests pass has destroyed the phase's only witness**, and the counts are the observable
precisely because nothing else sees it.

## Boundaries, not shapes -- three tasks in a row

Tasks 4b, 4b' and 6 each shipped or nearly shipped a Critical defect in the gap left by testing what
a construct *does* rather than where its clause boundaries fall. Two were oracle-confirmed byte
divergences that the whole workspace passed.

The premise that keeps being missed: **a boundary can leave a new pending trap behind it.** A
handler delivered at a clause boundary can itself `raise ... return`, queueing a trap for the
caller, and `in_clause` delivers at most one trap and does not re-check -- so the last member
boundary is not the last boundary with work.

**A loop header has more boundaries than either construct that found this.** Build the table around
boundaries.

## A catalogued divergence family you will meet

Task 4b' established that the oracle's clause boundary at a branch end comes from a **synthetic
end-of-branch instruction our Phase 3 parser elides**; the tree-walker gets the equivalent from
`step_in_temps_frame`'s wrapper and a flattened construct has none. Seven divergences are listed in
`task-4bp-report.md`, and `DO` blocks are explicitly among them. **Read that list before treating a
divergence as new.**

## Measuring

Predict against `docs/superpowers/plans/phase-4e-anchor.md`. **Do not predict against Task 4a's
-1.7%/-3.2%** -- those have no named mechanism and are explicitly not a baseline.

`varlookup` currently **fails exit criterion 4** (IR 4.28x against tree-walker 4.21x). Moritz
accepted that residual with a discharge condition assigned to **Task 7**, not to you. Report what
you measure on it; do not treat closing it as your job.

Wall clock disagreed with an identical instruction count by about 2% on this machine, so prefer
`perf stat -e instructions:u` for attribution and say so when you use it. A cycle or instruction
ratio equals a wall-clock ratio only if both sides retire at the same rate.
