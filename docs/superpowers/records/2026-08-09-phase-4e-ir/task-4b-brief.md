### Task 4b: The op-level interpreter, debuting on `If` and `Select`

**This task replaces the old Task 5 and takes the machinery Task 4a could not land.**

Task 4a stopped because flattening a construct needs three things that do not exist: the driver has no op program counter, a flattened construct also needs `run_bounded`'s absorption rule and a frame stack, and `Op::Generic` carries no instruction index so it cannot sit inside a jumped-to region. That is a driver redesign rather than a compiler change.

**It debuts on `If`/`Select` rather than on a loop, and the reason is not preference.** A branch condition is a single evaluation with nothing emitted between evaluations, so `Op::EvalExpr` is trace-identical there and Task 6 is not a prerequisite. A loop header is not, per the amended Decisions section. Building the hardest construct and the new machinery in the same task is what made Task 4a stop.

Deliverables: the op program counter, the absorption rule, the frame stack, `Op::Generic` carrying its instruction index, `Op::Clause`, `Op::EvalExpr`, `Op::Jump`, `Op::JumpUnless`, and the register allocator with its own test. Plus both inherited obligations: the compiler-side assertion that no `Clause` op precedes a `Generic` op, proved to fire; and removing `#[expect(dead_code)]` from `Op::Clause` and `Op::EvalExpr`.

Its `If`/`Select` steps are the old Task 5's, below, unchanged.

### Task 5: Promote `If` and `Select` -- these steps are Task 4b's

Without these, both branch bodies stay tree-walker positions and everything inside them is hidden from the IR.

`IF` splits across engines today: the true path runs `run_bounded` inline, the false path returns `Flow::Goto` and the outer loop's fallthrough runs the `ELSE` body. The compiled form has both paths as jumps in one stream.

**Predicted movement:** state it; `emptyloop` is unaffected and saying so is part of the prediction.

**Files:** `ir/compile.rs`, `ir/drive.rs`, `run.rs`; tests in `ir/golden_tests.rs`, `tests/ir_dual.rs`.

- [ ] **Step 1: Add the dual-engine cases first** -- `IF`/`THEN`/`ELSE` with and without `ELSE`, nested, `SELECT` with `OTHERWISE`, `SELECT` without a match (error **7.3**, not the 93.4 an earlier draft of this line named -- measured against the oracle, rc 249, for both `SELECT` and `SELECT CASE`), and an empty `THEN`.
- [ ] **Step 2: Write the golden test for the compiled `IF`, run it, watch it fail.**
- [ ] **Step 3: Emit the ops**, extracting the branch-selection semantics rather than duplicating it.
- [ ] **Step 4: Run the golden, dual, workspace and release suites.**
- [ ] **Step 5: Measure interleaved, record predicted versus measured, commit.**

The oracle-crashing `select; when 1 = 0 then; when 2 = 2 then nop; end` must not appear in any probe.

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
* **You own the register allocator**, whose discipline is fixed in the Decisions section. Implement
  it; do not redesign it.
* **Expression evaluation still routes through `eval.rs`.** `Op::EvalExpr` is trace-identical for a
  branch condition -- a single evaluation with nothing emitted between evaluations -- which is why
  this task debuts on `If`/`Select` and not on a loop header. Do not write a native expression op;
  Task 7 owns the first one, after Task 6's trace ops.
* **Splitting is a legitimate outcome.** Task 4 was split because it combined the hardest construct
  with new machinery, and stopping early made the split cheap. If the machinery lands cleanly and
  `Select` does not fit, say so and propose the boundary rather than half-landing it.

## Obligations this task inherits

1. **The compiler-side assertion**: no `Clause` op precedes a `Generic` op, because
   `step_in_temps_frame` echoes the clause itself and the echo is not idempotent. You are the first
   emitter of `Clause`, so you own it. **Construct the forbidden sequence and prove the assertion
   fires**, then restore.
2. **Remove `#[expect(dead_code)]` from `Op::Clause` and `Op::EvalExpr`.** You construct both.
3. **`ChunkTooLarge::what` keeps its `#[allow(dead_code)]`.** Not yours.

## A tripwire Task 4a left deliberately

`the_ir_engine_steps_a_loop_body_from_the_chunk` and
`the_ir_engine_steps_a_simple_blocks_body_from_the_chunk` are exact clause-count assertions. The
count is the observable that distinguishes a promoted body from an unpromoted one, and nothing else
in the workspace can make that distinction.

**If your work changes those counts, that is expected -- but update them by deriving the new
expected count from what the stream now contains, and say in your report why it changed.** A task
that adjusts the numbers until the tests pass has destroyed the only witness the phase has for
whether a body runs from the chunk at all.

## What Task 4a measured, and what you must not inherit from it

Task 4a's `emptyloop` -1.7% and `varlookup` -3.2% have **no named mechanism** and are explicitly not
a baseline. Do not quote them, do not predict against them, and do not treat "the IR arm was faster
before" as evidence of anything. Predict against `phase-4e-anchor.md`.

`If`/`Select` is not expected to move `emptyloop`, and saying so is part of the prediction.
