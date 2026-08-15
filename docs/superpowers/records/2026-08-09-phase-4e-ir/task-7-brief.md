### Task 7: Promote `Assignment` and `Say`

"Expression evaluation" is not a construct in this design: compiled expressions exist only inside promoted instructions, because an unpromoted instruction routes through `eval.rs` wholesale. Without these two, "variable access" and "arithmetic" reach almost nothing outside loop headers.

**Depends on Task 6.** The first native `Const` op is exactly what dropped `>L>`.

**Files:** `ir/compile.rs`, `ir/drive.rs`; tests in `ir/golden_tests.rs`, `tests/ir_dual.rs`.

**Interfaces:**
- Produces: `Op::Const { dst: u16, konst: u32 }`, `Op::Store { index: u32, src: u16 }`, and `Chunk::consts` with compile-time interning.

**Constants are interned**, and the reason is measured: without it a body allocates one `Vec<u8>` per literal *occurrence*, which on a straight-line workload is one heap allocation per clause paid at compile time. That artifact alone swamped the difference the spike existed to measure.

**`Store` goes through `assign_expr_target`**, which is what `step`'s own `Assignment` arm calls, so stems, compound tails and the `>=>` trace line are the same code rather than a second implementation.

- [ ] **Step 1: Add the dual-engine cases** -- assignment to a simple symbol, to a stem, to a compound with a computed tail, `SAY` of a literal, of an expression, and of nothing.
- [ ] **Step 2: Write the golden test for `n1 = 'abc'`, run it, watch it fail.**
- [ ] **Step 3: Emit `Const` and `Store` with interning and the trace ops from Task 6.**
- [ ] **Step 4: Assert the interning** -- a body with the same literal twice has one entry in `consts`.
- [ ] **Step 5: Full suites, then measure interleaved and record predicted versus measured.**
- [ ] **Step 6: Commit.**

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
* **You write the phase's first native expression op**, which is what Task 6 exists to make safe. Its
  shape is that emission is a **separate op** rather than a side effect of evaluation.
* **New case tables use `datadriven`** -- see `tests/ir_dual_cases/` for three examples. **A file
  holding oracle-measured bytes must refuse to run under `REWRITE`**, or a differential test silently
  becomes a self-consistency test. Do not retrofit `LOOP_CASES`, `BRANCH_CASES` or
  `KNOWN_DIVERGENCES`.
* **Splitting is legitimate.** Task 4 split for exactly this reason and the split was cheap because
  it stopped early.

## Why you are unblocked, and the defect that blocked you

The mechanics spike promoted an assignment and under `trace i` **it dropped the literal's `>L>`
line**: `eval.rs` emits that as a side effect of evaluating and a native `Const` op does not. Every
following line still matched, so **only an exact stderr comparison sees it** -- `tests/support/mod.rs`
normalises at `PREFIX_OFFSET` 7..10 and collapses the space run carrying nesting indent, so no
corpus instrument in this repo can.

**Promoting an expression silently drops intermediate trace unless each op re-emits it.** You are the
first task where that is live. Task 6 built `Op::TraceClause` and made the trace setting a
compilation input with a per-clause staleness check; build your expression ops the same way.

## THE DISCHARGE CONDITION THIS TASK OWNS

`varlookup` fails exit criterion 4. Moritz accepted that residual **with a condition assigned to
this task**, and it is not recordable as a debt a second time.

**The prediction:** the driver's cost is paid per body-range *entry*, not per op, so it dilutes as a
range holds more ops. `varlookup`'s body is two assignment clauses; promoting `Assignment` makes them
native ops in the same range.

**What discharges it:** re-measure `varlookup` under both arms. **At or below IR/TW 1.0 the residual
is closed.** Still above and **the amortisation hypothesis is refuted**, which puts the structural
remedies back on the table -- hoisting the op range so it travels with `BodyEngine` and is computed
once per body entry, a single-op fast path, or reconsidering the two-level shape. Say which, with the
number.

**`emptyloop` is not yours.** It also fails, at IR/TW 1.0237 wall and 1.0654 instructions, and its
body is a single `nop` so no expression promotion dilutes anything. Its +99 instructions per pass has
no owner; do not adopt it and do not let it colour your `varlookup` reading.

## Measuring, and the instrument rules this phase paid for

* **Criterion 4 is judged only within one binary**, both arms via `REXX_ENGINE`, interleaved inside
  each round, as a **paired sign test**. Building the same tree with one comment added gives a
  byte-identical `.text`, and two runs of it still differ by several per cent -- so the noise bounds
  repeated runs of one binary, not just comparisons between two.
* **Cross-binary attribution uses `perf stat -e instructions:u`**, which reproduces to eight
  significant figures where wall clock moves per cent.
* The anchor carries the measured per-pass attribution: frame stack +33, Task 4c +3, Task 6 exactly
  zero on `emptyloop`. **Predict against measured per-pass instruction deltas**, not against wall
  clock, and not against Task 4a's withdrawn figures.

## Method this phase has paid to learn

* **Boundaries, not shapes.** Tasks 4b, 4b' and 6 each shipped or nearly shipped a Critical defect in
  the gap left by testing what a construct does rather than where its clause boundaries fall. The
  premise repeatedly missed: **a boundary can leave a new pending trap behind it**, because
  `in_clause` delivers at most one and does not re-check.
* **A mutation row is a measurement of one suite.** Adding a test invalidates every row taken before
  it, invisibly -- the row still names real catchers, just not all of them. **Re-run the whole table
  once after the last test lands.**
* **Prove each case adds coverage**, not merely that it can fail: re-run the mutation with your new
  case removed and confirm green.
* **Rebuild after restoring from a `cp` backup.** A probe binary in `target/` survives a source
  revert and a clean `git status` says nothing about it.
