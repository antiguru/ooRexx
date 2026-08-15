### Task 3: The driver, engine selection, and the dual-engine harness

**Files:**
- Create: `rust/crates/rexx-exec/src/ir/drive.rs`
- Modify: `rust/crates/rexx-exec/src/invocation.rs`, `lib.rs`, `run.rs`
- Test: `rust/crates/rexx-exec/tests/ir_dual.rs`

**Interfaces:**
- Consumes: `Chunk`, `chunk_for`, `apply_flow`, `grant_procedure_permission`, `step_in_temps_frame`.
- Produces: `Engine` (`TreeWalker` | `Ir`), `Invocation::with_engine(self, engine: Engine) -> Invocation`, `Interp::run_chunk(&mut self, code: &Code<'_>, chunk: &Chunk, source: Option<&ProgramSource>) -> Result<Ended, Failure>`.

**Engine selection is not an environment variable.** `rexx-exec/src` contains no `env::var` at all; every gate variable lives in `tests/`. A variable read once per process also cannot give two arms in one in-process `cargo test` run, and the corpus runner calls `rexx_exec::run_program` directly rather than spawning. The carrier is `Invocation`, which is already a parameter of `run_program` and already reaches `execute`.

- [ ] **Step 1: Write the failing dual-engine test**

```rust
// rust/crates/rexx-exec/tests/ir_dual.rs
#[test]
fn both_engines_agree_on_an_all_generic_program() {
    let text = b"n1 = 2\nsay n1 + 3\n".to_vec();
    let tw = run(text.clone(), Engine::TreeWalker);
    let ir = run(text, Engine::Ir);
    assert_eq!(tw.stdout, ir.stdout);
    assert_eq!(tw.stderr, ir.stderr);
    assert_eq!(tw.exit, ir.exit);
}
```

- [ ] **Step 2: Run it, watch it fail**

Expected: FAIL, `Engine` not found.

- [ ] **Step 3: Add `Engine` and thread it**

`Engine` on `Invocation`, defaulting to `TreeWalker` for this phase; `execute` stores it on `Interp` the way it already handles `collect_every_alloc`. Task 11 flips the default.

- [ ] **Step 4: Write the driver**

Two levels, and the reason is `in_clause`. It is a scoped closure: it sets the clause line, runs the whole clause, and then -- only on the success path -- delivers a queued `CALL ON` handler, which can end the program. A clause spans a run of ops and a flat stream has no scope to hang that on.

```rust
/// Runs `chunk` for the activation on top of the stack.
///
/// The outer loop iterates clauses; the inner one runs that clause's own
/// ops. The `pc` stays an **instruction** index throughout, mapped through
/// `chunk.op_of` at the top of each clause, which is what lets `apply_flow`
/// be reused unchanged -- `Flow::Goto` and `Flow::Signal` both carry
/// instruction indices, and `Signal`'s resolve against the activation's own
/// body rather than against the body being stepped.
pub(crate) fn run_chunk(
    &mut self,
    code: &Code<'_>,
    chunk: &Chunk,
    source: Option<&ProgramSource>,
) -> Result<Ended, Failure> {
```

The register region is reserved once here, from `chunk.registers`, via `reserve_temps`, and truncated on the way out.

A `Generic` op runs `step_in_temps_frame` on the instruction the `pc` addresses. **No `Clause` op precedes a `Generic` op**, because `step_in_temps_frame` already echoes and the echo is not idempotent. Assert it in the compiler, not only in prose.

- [ ] **Step 5: Run the dual test**

Expected: PASS. At this point the IR arm executes an outer loop and delegates everything.

- [ ] **Step 6: Wire both body-entry points**

The production `run_activation` call sites are **two**: `resolve_and_run_call` and `Interp::run`. `deliver_pending_trap` reaches `resolve_and_run_call` and is not a third; `run_fragment` enters no activation; there is no `Interp::execute` -- `execute` is a private free function that calls `Interp::run`.

Selection happens at both, or a callee tree-walks its whole body and the gate asserts less than it reads.

- [ ] **Step 7: Build the dual-engine corpus harness**

`ir_dual.rs` runs the existing corpus and assertion populations under both arms and compares. Two requirements, each of which has bitten this project before:

* **Every gate variable STRICT in both arms.** `REXX_CORPUS_GATE`, `REXX_ASSERTIONS_GATE`, `REXX_BIF_GATE`, `REXX_KEYWORD_GATE` all default to REPORT, which exits 0 whatever it finds. Two green REPORT runs prove nothing.
* **Both arms derive their run list from one source**, asserted, in the shape `the_differential_reads_every_phase_subset_file` already uses -- so an arm that silently skips a harness goes red rather than passing with a shrunken denominator.

Also assert `chunks_refused == 0` across the population.

- [ ] **Step 8: Run it, then verify and commit**

```bash
cd rust && REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1 REXX_BIF_GATE=1 REXX_KEYWORD_GATE=1 \
  cargo test -p rexx-exec --test ir_dual
cd rust && cargo test --workspace && cargo test --workspace --release
cd rust && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
```

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
Tasks 4 and 5 promote control flow only and keep expression evaluation on `eval.rs` through `Op::EvalExpr`, which is trace-identical by construction because it is the same call.

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

Decisions, not suggestions, recorded in the repo so a reviewer can check them.

* **The carrier is `Invocation`, not an environment variable and not a second entry point.**
  `rexx-exec/src` contains no `env::var` at all; every gate variable lives in `tests/`. A variable
  read once per process also cannot give two arms in one in-process `cargo test` run, and the
  corpus runner calls `rexx_exec::run_program` directly rather than spawning. `execute` already
  threads a mode (`collect_every_alloc`) from `run_program` to `Interp`; follow that shape.
* **`Engine::TreeWalker` stays the default through this task.** Task 11 flips it. A task that
  flips it early makes every later task's failures ambiguous between "the promotion is wrong" and
  "the driver is wrong".
* **`run_fragment` is not in scope.** Fragments do not compile to chunks in this phase, so the
  re-entrancy question does not arise. Leave `run_fragment` on its tree-walker path and do not add
  a chunk path to it.
* **Both body-entry points, and there are exactly two:** `resolve_and_run_call` and `Interp::run`.
  `deliver_pending_trap` reaches `resolve_and_run_call` and is not a third; `run_fragment` enters
  no activation; there is no `Interp::execute` -- `execute` is a private free function that calls
  `Interp::run`.

## A standing obligation this task discharges

Task 2 shipped dead-code annotations because its items had no production caller. **Your driver is
that caller.** Remove every one of these, and if one cannot be removed, say why in your report
rather than leaving it:

* `chunk_for` (`plan.rs`)
* `compile` (`ir/compile.rs`)
* `Chunk`'s `ops`, `op_of` and `registers` fields (`ir/mod.rs`)
* `ChunkTooLarge` (`ir/mod.rs`)
* `Interp::chunks` and `Interp::chunks_refused` (`lib.rs`)

Not yours to remove: `Op::Clause` and `Op::EvalExpr` carry `#[expect(dead_code)]` and Task 4 is
their first emitter. `render` is `#[cfg(test)]` and stays that way.
