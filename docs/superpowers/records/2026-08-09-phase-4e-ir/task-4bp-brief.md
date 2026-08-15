### Task 4b': Promote `Select`, and build the frame stack it needs

Split out of Task 4b, which landed the machinery and stopped at the boundary rather than
half-landing `Select`.

`Select` needs three things `If` does not, and all three are the same missing piece:

* it **consumes** a `LEAVE` naming its own label and raises 28.5 on a matching `ITERATE`, so a flow
  escaping a matched `WHEN`'s body must reach `leave_select` rather than the enclosing range;
* it **rewrites** every `LEAVE`/`ITERATE` it forwards, resetting `origin.indent` through
  `pop_search_frame`;
* F-EX1's redirect -- an absorbed `WhenCase`'s false-branch `Goto` landing exactly on this
  `SELECT`'s `OTHERWISE` marker must enter `run_otherwise` -- is a decision the construct makes about
  a flow that would otherwise escape it.

Today the Rust call stack is that mechanism: `run_bounded(...)?` followed by `leave_select`.
Flattened, it becomes an explicit stack of open construct frames in the driver, each with its label,
its resume and its outcome handler, and an escaping `Flow` walks that stack applying each handler.
`indent_offset`, `current_case_text` and `test_case_when`'s per-value trace lines all ride on it.

`BRANCH_CASES` (`tests/ir_dual.rs`) already carries `SELECT` shapes, added by Task 4b before
anything was promoted and measured against the oracle first, so part of the regression net is in the
tree before this task starts. **It is not sufficient and must be extended, and the useful axis is
not more shapes.** Task 4b shipped a defect no shape case could see: a promoted clause
that recorded its own failure site but not its *boundary's*, which diverged from the oracle only
when a `CALL ON` handler failed at an `IF`'s clause boundary. Every shape case asks what a construct
does; none asks whether the clause unit is discharged exactly once where the promotion opens one.
`SELECT` opens a clause per listed `WHEN` as well as for its own header, so it has more of those
boundaries than `IF` does, not fewer.

Order against Task 4c is open: neither blocks the other, and 4c additionally waits on Task 6.

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
* **Expression evaluation still routes through `eval.rs`.** A `WHEN` condition is a single
  evaluation with nothing emitted between evaluations, so `Op::EvalExpr` is trace-identical for it,
  exactly as it was for `IF`. Do not write a native expression op; Task 7 owns the first one, after
  Task 6's trace ops.
* **You do not touch performance.** 4b-M closed the driver's cost investigation and Moritz accepted
  the one open residual with a discharge condition that belongs to Task 7. Measure and report, but
  do not optimise, and do not treat the residual as yours.
* **Splitting is a legitimate outcome** if the frame stack lands and `SELECT`'s remaining shapes do
  not fit. Say so and propose the boundary.

## The blind spot Task 4b shipped, which is your first design input

Task 4b's sixteen-case branch table was built around branch **shapes**. Every case asked what a
branch *does*; none asked whether the clause unit is discharged where the promotion opens a clause.
**A Critical defect landed in exactly that gap**: a promoted `IF` clause did not record its failure
site when a `CALL ON` handler failed at its boundary, an oracle-confirmed byte divergence between
engines.

It was fixed one level up -- the recording moved into `in_stepped_clause` and out of
`step_in_temps_frame_with`, so the clause unit itself owes the obligation and no promoted clause can
forget it. **Extend the table with boundaries, not shapes.** `SELECT` has more clause boundaries
than `IF`, which is why this warning is at the top of your brief rather than in a footnote.

## What `SELECT` needs that `IF` did not

`IF` owns no label, consumes no `LEAVE`/`ITERATE` and rewrites no escaping flow, so it needed jumps
and nothing else. `SELECT` needs the frame stack: label consumption, error 28.5, `pop_search_frame`,
and the `OTHERWISE` redirect. That is the deliverable, and it is why the split was drawn here.

**A `SELECT` with no matching `WHEN` and no `OTHERWISE` is error 7.3, not 93.4** -- measured against
the oracle, rc 249, for both `SELECT` and `SELECT CASE`. An earlier draft of this plan said 93.4.

**Never run `select; when 1 = 0 then; when 2 = 2 then nop; end` against the C++ oracle.** It is one
of three known oracle-crashing programs, and it is `SELECT`-shaped, so you are likelier than most
to reach for it.
