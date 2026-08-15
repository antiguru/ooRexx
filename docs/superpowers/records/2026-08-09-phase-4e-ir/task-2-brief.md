### Task 2: `Op`, `Chunk`, and the chunk cache

One task owns the data structures; later tasks extend rather than reshape them. Caching is part of this task and not a later optimisation -- without it, compile cost fuses with execution cost and nothing downstream is measurable, which is the failure that made the spike's own A/B unresolvable.

**Files:**
- Create: `rust/crates/rexx-exec/src/ir/mod.rs`, `ir/compile.rs`, `ir/golden.rs`
- Modify: `rust/crates/rexx-exec/src/lib.rs` (the `chunks` map, `chunks_refused`), `rust/crates/rexx-exec/src/plan.rs` (`chunk_for`)
- Test: `rust/crates/rexx-exec/src/ir/golden_tests.rs`

**The golden tests are `#[cfg(test)]` unit tests inside `ir/`, not an integration test.**
They assert internal structure, so they reach `pub(crate)` directly and the crate grows no new public surface.
Routing them through `tests/` would need a second `#[doc(hidden)] pub` entry point, and `lib.rs`'s doc on `run_program_collect_every_alloc` states it is "the crate's only hidden entry point" -- a sentence this task would otherwise have to falsify.

**Interfaces:**
- Consumes: `BodyKey`, `Plan`, `Interp::plan_for` from `plan.rs`.
- Produces:

```rust
pub(crate) enum Op {
    Generic,
    Clause { end: u32 },
    EvalExpr { index: u32, slot: u32, dst: u16 },
}

pub(crate) struct Chunk {
    ops: Vec<Op>,
    op_of: Vec<u32>,   // instruction index -> op index, total, plus one entry at len
    registers: u16,
}

pub(crate) struct ChunkTooLarge { pub(crate) what: &'static str }

pub(crate) fn compile(body: &CodeBody, plan: &Plan) -> Result<Chunk, ChunkTooLarge>;

impl Interp {
    pub(crate) fn chunk_for(&mut self, key: BodyKey, body: &CodeBody, plan: &Plan)
        -> Option<Rc<Chunk>>;
}
```

`EvalExpr` is declared here and emitted from Task 4 onward. `slot` addresses the expression within its instruction; its exact meaning is Task 4's, and Task 2 only reserves the variant so later tasks extend the stream rather than reshape it.

- [ ] **Step 1: Write the failing golden test**

```rust
// rust/crates/rexx-exec/src/ir/golden_tests.rs
#[test]
fn every_instruction_of_an_all_generic_body_compiles_to_one_generic_op() {
    let chunk = compile_for_test(b"say 1\nsay 2\nn1 = 3\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Generic\n\
         1: Generic\n\
         2: Generic\n"
    );
}
```

`compile_for_test` is a `#[cfg(test)]` helper in this file that parses, plans and compiles. It is not a crate entry point.

- [ ] **Step 2: Run it and watch it fail**

```bash
cd rust && cargo test -p rexx-exec --lib ir::
```

Expected: FAIL to compile, `render` not found. Read the run count on every later invocation of this command -- a filter matching nothing exits 0.

- [ ] **Step 3: Write `Op`, `Chunk` and `compile`**

`compile` walks `body.instructions`, pushes `op_of` for each, and emits `Op::Generic` for every instruction. `op_of` gets one final entry at `ops.len()` **after** the loop, because `run_bounded`'s absorption guard is inclusive and a construct's resume point can be one past the end.

`Chunk::registers` is a field of this task and is `0` for every chunk it compiles, because nothing allocates a register until Task 4. Task 3's driver reserves it, so it is read from Task 3 onward and is not dead.

**The allocator itself is Task 4's, and writing it here would break this task's own gate.** `Registers` would have no caller until Task 4, and `cargo clippy --workspace --all-targets -- -D warnings` fails on `dead_code`. Its discipline is already decided in [Decisions](#decisions-this-plan-makes-that-the-spec-left-open) so Task 4 implements a settled design rather than inventing one; only the code moves.

- [ ] **Step 4: Write the golden serialiser**

`ir/golden.rs` exposes `pub(crate) fn render(chunk: &Chunk) -> String`, one op per line as `index: OpName field=value`, constants as escaped byte strings. It is `pub(crate)` and not `pub`: nothing outside the crate reads an op stream.

Larger streams live in `rust/crates/rexx-exec/testdata/ir/*.ops` and are compared with `include_str!`. Rewriting them is a documented manual step, not an environment variable -- `rexx-exec/src` contains no `env::var` at all and this task does not make it the first.

- [ ] **Step 5: Run the test**

Expected: PASS.

- [ ] **Step 6: Write the `op_of` boundary test**

```rust
#[test]
fn the_instruction_map_has_an_entry_one_past_the_last_instruction() {
    // `run_bounded`'s absorption guard is inclusive, so a construct's
    // resume point can be `end`, which is one past its last instruction.
    // A map that stops at `len - 1` panics there rather than at compile.
    let chunk = compile_for_test(b"if 1 = 1 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        chunk.op_of.len(),
        chunk.instruction_count() + 1,
        "one entry per instruction plus the end entry"
    );
}
```

Stated as a relation rather than as the literal `3`, so the test does not have to be re-read every time the parser's instruction count for that source changes.

Run it, watch it fail if the final push is missing, then make it pass.

- [ ] **Step 7: Add the chunk cache**

`Interp` gains `chunks: HashMap<BodyKey, Rc<Chunk>>` beside `plans`, and `chunks_refused: usize`. `chunk_for` mirrors `plan_for` exactly:

```rust
/// The chunk for one body, from the cache or compiled and cached, under the
/// same key as its plan (D16's discipline, unchanged).
///
/// `None` means the body does not fit the index widths and this activation
/// runs on the tree-walker. `chunks_refused` is what stops that being
/// silent: the dual-engine harness asserts it is zero across the corpus.
pub(crate) fn chunk_for(
    &mut self,
    key: BodyKey,
    body: &CodeBody,
    plan: &Plan,
) -> Option<Rc<Chunk>> {
    if let Some(chunk) = self.chunks.get(&key) {
        return Some(Rc::clone(chunk));
    }
    match compile(body, plan) {
        Ok(chunk) => {
            let chunk = Rc::new(chunk);
            self.chunks.insert(key, Rc::clone(&chunk));
            Some(chunk)
        }
        Err(_) => {
            self.chunks_refused += 1;
            None
        }
    }
}
```

- [ ] **Step 8: Write the cache test**

A program calling one routine in a loop compiles that routine's body once. Assert by counting compiles through a test-visible counter, not by timing.

- [ ] **Step 9: Verify and commit**

```bash
cd rust && cargo test -p rexx-exec --lib ir::
cd rust && cargo test --workspace
cd rust && cargo fmt --all --check
cd rust && cargo clippy --workspace --all-targets -- -D warnings
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

These are decisions, not suggestions. They are recorded here rather than only in the dispatch
prompt so a reviewer can check them against a file that is in the repo's own working tree.

* **`Op::EvalExpr` is declared in this task and emitted by none of it.** Task 4 is its first
  emitter. Declaring the variant now is what lets later tasks extend the stream rather than reshape
  it, which this task owns. Its `slot` field's meaning is Task 4's to define.
* **`Chunk::registers` is `0` for every chunk this task compiles**, because nothing allocates a
  register until Task 4. It is not dead: Task 3's driver reserves it.
* **The register allocator is not written in this task.** It would have no caller until Task 4 and
  `cargo clippy --workspace --all-targets -- -D warnings`, which this task runs as its own last
  step, fails on `dead_code`. Its discipline is already decided in the Decisions section above;
  Task 4 implements it.
* **The chunk cache is keyed on `BodyKey` alone, and that is correct for this task.** Task 6 makes
  the trace setting an input to compilation, at which point `BodyKey` stops identifying a chunk and
  Task 6 owns widening the key. Do not anticipate that here; do not write an abstraction for it.
* **Golden tests are `#[cfg(test)]` unit tests inside `src/ir/`, not an integration test under
  `tests/`.** Routing them through `tests/` needs a second `#[doc(hidden)] pub` entry point, and
  `lib.rs`'s doc on `run_program_collect_every_alloc` states it is "the crate's only hidden entry
  point" -- a sentence this task would otherwise have to falsify.
