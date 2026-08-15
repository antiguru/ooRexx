# Task 2 report: `Op`, `Chunk`, and the chunk cache

BASE `fb43b3c4` (Task 1's commit, plus its fix-round correction, already landed).
Commit `64ce256d`. Fix round 1: `09cba5a8` (on top of `cd529eea`, a plan-only correction to
`Op::Clause`'s annotation that this task did not need to touch).

**Fix round 1 changed two claims below; both are corrected in place rather than left standing next
to a later retraction. See "Fix round 1" at the end for what changed and the mutation evidence.**

## What was built

* `rust/crates/rexx-exec/src/ir/mod.rs` -- `Op` (`Generic`, `Clause { end: u32 }`,
  `EvalExpr { index: u32, slot: u32, dst: u16 }`), `ChunkTooLarge { what: &'static str }`,
  `Chunk { ops: Vec<Op>, op_of: Vec<u32>, registers: u16 }`, and a `#[cfg(test)]`
  `Chunk::instruction_count() -> usize` (`op_of.len() - 1`).
  `Op`, `ChunkTooLarge` and `Chunk` are `pub(crate)`; their fields carry no visibility
  modifier, so only `ir` and its submodules reach them, matching the brief's interface
  snippet exactly (no `pub(crate)` on the fields there either).
* `rust/crates/rexx-exec/src/ir/compile.rs` -- `pub(crate) fn compile(body: &CodeBody, _plan: &Plan)
  -> Result<Chunk, ChunkTooLarge>`. Walks `body.instructions` once, pushes one `op_of` entry and one
  `Op::Generic` per instruction, then pushes a final `op_of` entry at `ops.len()` after the loop. The
  one error path converts `ops.len()` to `u32` with `try_from`; `_plan` is unread in this task (no
  promoted instruction needs a name-to-slot answer yet) and is prefixed accordingly rather than
  triggering `unused_variables`. A `#[cfg(test)]` `thread_local` `Cell<usize>` counts calls
  (`compile_calls()`), for the cache test.
* `rust/crates/rexx-exec/src/ir/golden.rs` -- `pub(crate) fn render(chunk: &Chunk) -> String`, one
  `"{index}: {OpName}[ field=value]*"` line per op, exhaustive over `Op` with no catch-all arm.
* `rust/crates/rexx-exec/src/ir/golden_tests.rs` -- the three tests (below), `#[cfg(test)] mod
  golden_tests;` gated in `mod.rs`, matching this crate's own `foo/tests.rs` convention
  (`rexx-parse/src/*/tests.rs`).
* `rust/crates/rexx-exec/src/lib.rs` -- `mod ir;` declared beside `mod trace;`; `Interp` gained
  `chunks: HashMap<BodyKey, Rc<crate::ir::Chunk>>` and `chunks_refused: usize`, both zero-initialised
  in `Interp::new`.
* `rust/crates/rexx-exec/src/plan.rs` -- `Interp::chunk_for`, placed directly after `plan_for` as the
  brief's Files list specifies, matching the brief's Step 7 code exactly (cache hit clones the `Rc`;
  a miss compiles, caches, and returns; `Err(_)` bumps `chunks_refused` and returns `None`).

## Every test added, and what degenerate implementation each rules out

**Corrected in fix round 1: every claim below is now backed by an actual mutation run to red, not
just an assertion about what the test "should" catch. See "Fix round 1" for the mutation log.**

1. `every_instruction_of_an_all_generic_body_compiles_to_one_generic_op` -- the brief's own Step 1
   test, verbatim, plus `chunk.registers == 0`. Checked (fix round 1): mutating `compile` to emit a
   fixed one op regardless of the body's real instruction count turns this test red (`"0: Generic\n"`
   vs. the expected three lines), which is the shape a degenerate "ignore the body, emit a constant
   stream" implementation would produce. The `registers == 0` half is not separately checked by
   mutation -- there is no register allocator yet to mutate -- so it pins the current fact rather than
   proving it catches a future regression.
2. `the_instruction_map_has_an_entry_one_past_the_last_instruction` -- **originally reported as
   ruling out a table that stops at `len - 1`, which was false: the assertion compared
   `chunk.op_of.len()` against `chunk.instruction_count()`, and `instruction_count()` was itself
   defined as `op_of.len() - 1`. The comparison reduced to `x == x` and could not fail regardless of
   what `compile` did.** Fixed by comparing `chunk.op_of.len()` against `program.main.instructions.len() +
   1` instead -- a count taken from the parsed body, independent of anything `Chunk` computes.
   Checked (fix round 1): deleting the final `op_of.push(end)` in `compile.rs` turns the rewritten
   test red (`left: 4, right: 5`) while the other two tests stay green, confirming it now catches
   exactly the bug it names and nothing else masks the mutation. `Chunk::instruction_count()` is
   deleted; nothing else called it.
3. `chunk_for_compiles_a_body_once_across_repeated_lookups` -- calls `chunk_for` three times under
   one `BodyKey` and asserts two independent things: `compile_calls()` advanced by exactly `1` across
   all three calls, and `Rc::ptr_eq` holds between the first result and both later ones. Checked (fix
   round 1): mutating `chunk_for` to skip its cache lookup (call `compile` unconditionally) turns this
   test red (`left: 3, right: 1`) while the other two stay green -- the call-count assertion is what
   catches it; the `Rc::ptr_eq` assertion additionally guards against a *different* degenerate shape
   (calling `compile` once but reconstructing or leaking a fresh `Rc` on every lookup instead of
   returning the cached one), which the call count alone would not catch and which this mutation does
   not exercise.

## What was verified

* `cargo test -p rexx-exec --lib ir::` -- **3 passed**, read from the printed line (not the exit
  status): `3 passed; 0 failed; ... 512 filtered out`, confirming the filter matched real tests
  rather than nothing.
* `cargo test --workspace` -- **1340 passed, 0 failed** (summed every `test result:` line's `passed`
  and `failed` counts across the whole run, not read from a single suite). Baseline was 1337; this
  task added exactly 3, and the total moved by exactly 3.
* `cargo test --workspace --release` -- **1340 passed, 0 failed**, same total, the distinct gate the
  Global Constraints call out (`lto = "fat"` changes behaviour).
* `cargo fmt --all --check` -- exit 0 after running `cargo fmt --all` once to settle the new files'
  formatting (line-wrapped several `#[allow(dead_code, reason = "...")]` attributes and a couple of
  `assert!`/match-arm bodies).
* `cargo clippy --workspace --all-targets -- -D warnings`, from a **freshly removed `target/`**, per
  the Global Constraints' phase-boundary requirement -- exit 0. One real finding en route:
  `clippy::missing_const_for_thread_local` on the `Cell::new(0)` initializer, fixed by wrapping it in
  `const { .. }` per clippy's own suggestion.

## The dead_code shape, and why it needed real measurement rather than a rule of thumb

`chunk_for` has no production caller until Task 3's driver exists; its only caller today is the
cache test. Empirically (verified with small `rustc -W dead_code` repros before touching the real
crate, then confirmed against the actual crate), this makes the whole call chain
`chunk_for -> compile -> Chunk`'s fields, plus `ChunkTooLarge`, dead in the **plain `lib` target**
(no `cfg(test)`) while genuinely live in the **`unittests` target** (`cfg(test)` on, where the tests
call them) -- an inconsistency across compilations, not a uniform "always dead" state. `#[expect(dead_code)]`
demands the lint actually fire in *every* compilation that sees the item; it would be wrong here for
exactly the reason `rexx-exec/tests/owners.rs`'s own file-wide `#![allow(dead_code)]` doc gives for
its own three-binaries case. So `chunk_for`, `compile`, `Chunk`'s three fields, `ChunkTooLarge` (as a
whole -- a struct with an unconstructed value reports "never constructed" at the struct's span, which
a field-level `allow` does not cover) and `render` all carry `#[allow(dead_code, reason = "...")]`
naming Task 3 as the caller that removes it.

`Op::Clause` and `Op::EvalExpr`, by contrast, are constructed by *nothing* anywhere in this task's own
code, in either target -- consistently dead everywhere until Task 4. That consistency is what makes
`#[expect(dead_code, reason = "Task 4 is this variant's first constructor")]` the correct attribute
for exactly those two variants: unlike `allow`, it will itself start warning ("unfulfilled lint
expectation") the moment Task 4 constructs one, which is the self-invalidating property this project's
own conventions (`error.rs`'s deleted `#[expect(dead_code)]`, referenced in its own doc history) favour
over a silent, never-checked `allow`.

**Superseded by fix round 1**: `golden.rs` and its `mod golden;` declaration are now `#[cfg(test)]`-gated
in `ir/mod.rs`, the same way `golden_tests.rs` already was, rather than carrying a permanent
`#[allow(dead_code)]`. See "Fix round 1" for why. `Chunk::instruction_count()` no longer exists (deleted
in the same round, once the boundary test stopped needing it).

## Ambiguities from the brief, followed rather than re-litigated

* `Op::EvalExpr`'s `slot`/`index`/`dst` fields are declared with the exact names and types the brief
  gives, and nothing in this task assigns them meaning beyond the enum's own doc comment pointing at
  Task 4.
* `Chunk::registers` is `0` for every chunk this task compiles; the one test that inspects it
  (`chunk.registers == 0`) pins that as a current fact rather than asserting it can never change.
* The register allocator (`mark`/`alloc`/`release`) was not written. Nothing in this task's own commit
  references it.
* The chunk cache is keyed on `BodyKey` alone, unchanged from `plan_for`'s own key -- no widening, no
  trace-setting input, matching the brief's explicit "do not anticipate that here."

## What could not be done / left for later tasks

* No `testdata/ir/*.ops` golden files were added. The brief frames those as the format for "larger
  streams," and both tests given/added here use small inline string literals, matching the brief's own
  Step 1 example; nothing in this task's scope produces a stream large enough to need one.
* The `ChunkTooLarge` overflow path (`u32::MAX` ops) is untestable at this task's scale (would need
  ~4 billion instructions) and is not exercised by any test; the doc comment says so rather than
  claiming coverage that does not exist.

## Fix round 1

Two findings from the coordinator, both against `64ce256d`.

**Finding 1 (the coordinator's own defect, not this task's): the boundary test was tautological.**
`the_instruction_map_has_an_entry_one_past_the_last_instruction` asserted `chunk.op_of.len() ==
chunk.instruction_count() + 1`, and `Chunk::instruction_count()` was defined as `self.op_of.len() - 1`.
Substituting, the assertion is `op_of.len() == (op_of.len() - 1) + 1`, i.e. `x == x`: it cannot fail no
matter what `compile` does. Fixed by taking the expected count from the parsed body instead --
`program.main.instructions.len() + 1` -- a source that does not go through `Chunk` at all, so the two
sides of the comparison are independent. `Chunk::instruction_count()` is deleted; nothing else called it.

**Verification that the fix is load-bearing, and that the other two tests are too** (all three by the
same cp / mutate / run / restore / `sha256sum -c` cycle, never `git checkout --`):

* Backed up `compile.rs`, deleted the final `op_of.push(end)` (the exact bug the test names). Ran
  `cargo test -p rexx-exec --lib ir::`: the rewritten boundary test went red (`left: 4, right: 5`); the
  other two stayed green. Restored from the backup, confirmed with `sha256sum -c`, confirmed `git diff`
  showed no residual change to `compile.rs`, re-ran the three tests green.
* Backed up `compile.rs` again, changed `for _ in &body.instructions` to `for _ in 0..1` (a `compile`
  that emits a constant-length stream regardless of the body). Ran the suite: both
  `every_instruction_of_an_all_generic_body_compiles_to_one_generic_op` (`"0: Generic\n"` vs. three
  expected lines) and the boundary test (`left: 2, right: 5`) went red; the cache test stayed green.
  Restored, verified with `sha256sum -c`, confirmed via `git diff` and a green re-run.
* Backed up `plan.rs`, removed `chunk_for`'s cache-hit early return (`if let Some(chunk) =
  self.chunks.get(&key) { return ...; }`) so every lookup recompiles. Ran the suite: only
  `chunk_for_compiles_a_body_once_across_repeated_lookups` went red (`left: 3, right: 1`); the other two
  stayed green. Restored, verified with `sha256sum -c`, confirmed via `git diff` and a green re-run.

Each mutation red-lined the test whose doc comment names that exact failure mode, and none of the three
mutations turned a *different* test red that should have stayed green (the "ignore the body" mutation
tripping the boundary test too is expected, not a false positive: a stream that is always one op short
of the real body length also fails the "one entry per instruction plus one" boundary, independently of
the tautology this round fixes).

**Finding 2: `render`'s `#[allow(dead_code)]` would have been permanent.** `render` (`golden.rs`) has no
caller anywhere in the plan's ten tasks except the golden tests, so the `#[allow(dead_code, reason =
"...")]` this task gave it named no future caller -- unlike `chunk_for`/`compile`/`Chunk`'s fields/
`ChunkTooLarge`, which all name Task 3's driver as the caller that will make the annotation removable.
Fixed by gating `mod golden;` behind `#[cfg(test)]` in `ir/mod.rs`, the same treatment `golden_tests.rs`
already had, and dropping the `#[allow(dead_code)]` from `render` entirely. Re-verified from a clean
`target/`: `cargo clippy --workspace --all-targets -- -D warnings` is still exit 0.

**Re-run after both fixes:**

* `cargo test -p rexx-exec --lib ir::` -- 3 passed, 512 filtered out (non-zero run count read).
* `cargo test --workspace` -- 1340 passed, 0 failed (unchanged from before this round: the fix
  corrected an assertion and a cfg gate, it added no test and removed none).
* `cargo test --workspace --release` -- 1340 passed, 0 failed.
* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings`, from a freshly removed `target/` -- exit 0.

**What I disagree with:** nothing. Both findings are correct, the plan's own annotation fix
(`cd529eea`, `Op::Clause`) needed no changes on this task's side, and I made none.
