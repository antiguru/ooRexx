### Task 6: Trace as explicit instructions in the stream

Lands before the first native expression op, and Tasks 7 onward depend on it.

**It is a correctness requirement, not a performance idea, and the spike proved it (`70b1c5cc`).** Under `trace i` a promoted assignment dropped the literal's `>L>` line, because `eval.rs` emits it as a side effect of evaluating and a native `Const` op does not. Every following line still matched, so only an exact stderr comparison sees it, and no corpus instrument here does. **Promoting an expression silently drops intermediate trace unless each op re-emits it.**

The naive fix is worse than it looks: emitting inline cost a full render and allocation per clause on an *untraced* run until the gate was hoisted, measured at about 39 ns/clause. That branch is what the op form removes.

**Files:** `ir/compile.rs`, `ir/drive.rs`; tests in `ir_dual.rs` plus a new exact-stderr test.

**Interfaces:**
- Produces: the trace op variants, and the split of `Clause` into an unconditional half and a conditional half -- the clause *line* is semantics rather than trace, feeding `SIGL`, condition objects and syntax error messages, so it cannot be elided with the echo.

- [ ] **Step 1: Write the failing test that the spike's defect would pass today**

An exact stderr comparison of `trace i` output between the two arms, on a program with a literal in an assignment. Corpus instruments cannot see this: `tests/support/mod.rs` normalises at `PREFIX_OFFSET` 7..10 and collapses the space run that carries nesting indent.

- [ ] **Step 2: Make the trace setting an input to compilation**

Per D23. The setting decides which trace ops are emitted; an untraced chunk pays nothing at all, not even a flag test per clause.

**This invalidates Task 2's cache key, and fixing it is part of this step rather than a later cleanup.** `BodyKey` identified a chunk while trace was not an input; now two chunks for one body can differ, and `chunk_for` handing back whichever was compiled first is a wrong-output defect, not a performance one. Widen the key or evict on a setting change.

- [ ] **Step 2b: Write the test that a stale chunk would fail**

A body entered once untraced and once under `trace i`, in that order, in one process. The second entry must produce trace output. A cache still keyed on `BodyKey` alone returns the untraced chunk and the assertion goes red -- which is the point: this test is what makes the key change load-bearing rather than a claim in a doc comment.

**What this does not reach, stated so nobody assumes it does:** `TRACE` is dynamic -- `TRACE VALUE expr`, a mid-program `trace`, inheritance across activations -- which relocates the deopt problem rather than solving it. Interactive trace is control flow rather than an emit. Under `Generic` the two regimes coexist until expressions are promoted. The plan's answer to dynamism is that a setting change invalidates the body's cached chunk; measure whether that is cheap enough on a program that changes `TRACE` inside a loop, and record the figure.

- [ ] **Step 3: Run the trace oracle under both arms.**
- [ ] **Step 4: Run the `run.rs` unit tests that assert exact stderr under both arms.** They construct `Interp` directly, which makes them the tests most at risk of never running on the IR arm -- exit criterion 7 names them for that reason.
- [ ] **Step 5: Measure that an untraced chunk did not get slower**, interleaved, and commit.

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
* **Use the `datadriven` crate for any new case table** (dev-dependency, allowed as of `4f98b9dc`,
  0.9.0 resolves from the local registry cache offline). **Do not retrofit** `LOOP_CASES`,
  `BRANCH_CASES` or `KNOWN_DIVERGENCES` -- they hold witnesses nothing else in the workspace
  catches, and rewriting them for format risks those for no coverage gain.
* **You promote nothing.** Task 7 owns the first native expression op. Your job is to make trace an
  emission decision so that Task 7 *can* write one without silently dropping lines.
* **Splitting is legitimate** if the cache-key change and the op set do not fit together. Say so and
  propose the boundary.

## What this task exists to prevent, measured rather than argued

The mechanics spike promoted an assignment and, under `trace i`, **it dropped the literal's `>L>`
line**. `eval.rs` emits that as a side effect of evaluating; a native `Const` op does not. Every
following line still matched, so **only an exact stderr comparison sees it** -- `tests/support/mod.rs`
normalises at `PREFIX_OFFSET` 7..10 and collapses the space run carrying nesting indent, so no
corpus instrument here can.

**Promoting an expression silently drops intermediate trace unless each op re-emits it.** That is
why you land before Task 7 and not with it.

**The naive fix is worse than it looks.** Emitting inline cost a full render and allocation per
clause on an *untraced* run until the gate was hoisted -- about 39 ns/clause. The op form is
precisely what removes that branch.

## The cache-key change is yours, and it is a wrong-output defect if missed

Task 2 keyed the chunk cache on `BodyKey` alone, which was correct while trace was not a compilation
input. **Once it is, two chunks for one body can differ and `chunk_for` handing back whichever was
compiled first is a wrong-output defect, not a performance one.** Widen the key or evict on a
setting change, and write the test that fails against the old key: one body entered untraced and
then under `trace i`, in one process, asserting the second entry traces.

## What this task cannot reach, so do not claim it

`TRACE` is dynamic -- `TRACE VALUE expr`, a mid-program `trace`, inheritance across activations --
which **relocates** the deopt problem rather than solving it. Interactive trace (`?` prefix) is
control flow, not an emit: it reads standard input and executes what it is given. And under
`Generic` the two regimes coexist until expressions are promoted.

Measure what a `TRACE` change inside a loop costs once a setting change invalidates the body's
cached chunk, and record the figure. If it is expensive, say so; the answer is a decision this plan
does not pre-empt.

## A family of divergences you will meet, already catalogued

Task 4b' established that **the oracle's clause boundary at a branch end comes from a synthetic
end-of-branch instruction our Phase 3 parser elides.** The tree-walker gets the equivalent from
`step_in_temps_frame`'s wrapper; a flattened construct has none. Seven oracle divergences are listed
in `.superpowers/sdd/2026-08-09-phase-4e-ir/task-4bp-report.md` and all but two are that same
instruction, in `ELSE`, `DO` blocks and a nested second delivery's `SIGL`.

**Trace output is exactly where this family shows itself.** Read that list before you start, and
when you meet a divergence check it against the list before treating it as new.

`KNOWN_DIVERGENCES` in `tests/ir_dual.rs` is where a divergence goes when the two engines genuinely
differ and one of them is right -- it has a test that is red if *either* engine moves. Use it rather
than widening a normalisation.
