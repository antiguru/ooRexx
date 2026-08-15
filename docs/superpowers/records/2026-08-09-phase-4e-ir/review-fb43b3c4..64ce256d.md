# Review: Phase 4e Task 2 -- `Op`, `Chunk`, and the chunk cache

BASE `fb43b3c4`. Commit `64ce256d`.

## Verdicts

**Spec compliance: MET.**
Every interface in the brief is produced with the exact signature and visibility given (`Op`, `Chunk`, `ChunkTooLarge`, `compile`, `Interp::chunk_for`).
`compile` walks `body.instructions` once, emits one `Op::Generic` per instruction, and pushes the boundary `op_of` entry after the loop -- verified correct by direct inspection and by an experiment that removed the final push and observed `op_of.len()` shrink by one.
`chunk_for` mirrors `plan_for` structurally, keyed on `BodyKey` alone, matching the Ambiguities section's explicit instruction not to anticipate Task 6's key widening.
The register allocator was correctly not written here.
Golden tests are `#[cfg(test)]` units inside `src/ir/`, not under `tests/`, per the brief.
All gates the brief lists pass: `cargo test -p rexx-exec --lib ir::` (3 passed, confirmed by run count), `cargo test --workspace` and `--release` (1340 passed / 0 failed, recomputed independently), `cargo fmt --all --check`, and `cargo clippy --workspace --all-targets -- -D warnings` from a genuinely clean `target/` (verified independently by deleting `target/` and rerunning: exit 0).

**Task quality: NOT CLEAN -- 1 Critical, 2 Important, 0 Minor.**
The implementation is sound; the defects are in the test suite's coverage of the one property the brief calls out by name, and in two overclaims in the report/doc comments about what later tasks will do with this task's declared shapes.

## Findings

### Critical

**C1. The `op_of` boundary test cannot fail; the exact defect the brief names is untested.**
`the_instruction_map_has_an_entry_one_past_the_last_instruction` (`ir/golden_tests.rs`) asserts
`chunk.op_of.len() == chunk.instruction_count() + 1`. `Chunk::instruction_count()` (`ir/mod.rs`,
`#[cfg(test)] impl Chunk`) is defined as `self.op_of.len() - 1`. Substituting, the assertion reduces
to `op_of.len() == (op_of.len() - 1) + 1`, an algebraic identity true for every value of
`op_of.len()`. It carries no information about whether the trailing entry exists.

Verified by experiment: backed up `ir/compile.rs`, deleted the `let end = ...; op_of.push(end);`
lines that push the boundary entry (the exact bug the brief's Step 6 describes and instructs
"watch it fail if the final push is missing"), reran `cargo test -p rexx-exec --lib ir::`. All 3
tests still pass, including this one. Restored from the backup and verified with `sha256sum -c`.

This is the brief's own named defect (`run_bounded`'s inclusive absorption guard panicking one
past the end) shipping with zero test coverage, and the report's claim that this test "rules out
... an `op_of` table that stops at `len - 1`" is false -- it does not, as demonstrated.

The implementation itself is correct (`compile` does push the boundary entry, confirmed by reading
`compile.rs` and by the experiment showing `op_of.len()` actually changes when the push is
removed), so this is a task-quality defect, not a spec-compliance one. Fix: give
`instruction_count()` an independent source, e.g. `self.ops.len()` (every instruction compiles to
exactly one op in this task, so `ops.len()` equals the instruction count without going through
`op_of` at all), or drop the helper and assert against a value computed independently in the test
(e.g. the number of instructions the test source is known to parse to, or `body.instructions.len()`
captured before compiling).

### Important

**I1. `Op::Clause`'s `#[expect(dead_code, reason = "Task 4 is this variant's first constructor")]` names a task that, per the plan, never constructs it -- and the variant itself may not survive to be constructed as declared.**
Checked against `docs/superpowers/plans/2026-08-09-phase-4e-ir.md`: Task 4's own Interfaces section
lists `Op::EvalExpr` (first construction), `Op::Jump`, `Op::JumpUnless` -- no `Op::Clause`. Task 5's
section adds no new `Op` variants at all ("extracting the branch-selection semantics" reuses
Task 4's jumps). Task 6's Interfaces section reads: "Produces: the trace op variants, and **the
split of `Clause` into an unconditional half and a conditional half**." `Op::Clause` appears
nowhere else in the plan. That is a reshape (the single variant becomes two), not an extension by
Task 4 or anyone else -- the opposite of what this task's own doc comment on `Op` claims
("Declaring both now is what lets Task 4 extend this enum rather than reshape it") and what the
report asserts ("Task 4 is this variant's first constructor... consistently dead everywhere until
Task 4"). Both statements are unverified predictions that the plan's own text contradicts.

Mechanically this is not a build defect today: `#[expect(dead_code)]` is satisfied by the variant
being deleted along with the rest of `Clause` at Task 6 just as validly as by a constructor
appearing, so no warning misfires now or is guaranteed to misfire later. But the doc comment and
the report both make a specific, checkable claim about which task acts on this variant, and it is
wrong. Whoever lands Task 6 should not be surprised to find `Op::Clause` already gone from a
comment's promise; the comment and the report should say "Task 6 replaces this variant" or
similar, or simply not name a task at all.

**I2. `render`'s `#[allow(dead_code)]` is not transitional the way the report's summary claims, and the report should say so.**
The report's summary sentence groups six items -- `chunk_for`, `compile`, `Chunk`'s three fields,
`ChunkTooLarge`, and `render` -- as all "carry[ing] `#[allow(dead_code, reason = "...")]` naming
Task 3 as the caller that removes it." That is true for the first five (confirmed: Task 3's driver,
per the plan, is `ir/drive.rs`, a descendant module of `ir`, so it can read `Chunk`'s private fields
directly and does read `chunk.registers` per the plan's own Task 3 text -- "The register region is
reserved once here, from `chunk.registers`"). It is not true for `render`. Its own `reason` string,
unlike the other five, does not name Task 3 -- it says "called only from `golden_tests.rs`'s
`#[test]` functions," which is accurate. But nothing in any of the plan's ten tasks gives `render` a
non-test caller: the plan's own file map fixes `ir/golden.rs` as "plain `pub(crate)`, no cfg gate"
for the life of the phase, and every later task's own golden test (`ir/golden_tests.rs`) is the only
consumer listed anywhere. If that holds, `render`'s `#[allow(dead_code)]` is not a Task-3-clears-it
transitional annotation; it is close to permanent, which is the same category of finding the
plan's own preamble raised about the register allocator (a caller-less item under a `-D warnings`
dead-code gate), just for a function rather than a struct, and decided differently (kept `pub(crate)`
and un-gated rather than moved). The controller checking "did Task 3 remove the annotations it was
supposed to" should not expect `render`'s to disappear, and the report should have flagged the
asymmetry instead of folding `render` into the same bucket as the other five.

## Verified positives (checked by experiment, not taken on the report's word)

* The `#[allow]`/`#[expect]` split is empirically correct. Stripped every dead-code attribute from a
  backed-up copy of the five touched files and rebuilt both the plain `lib` target and the
  `unittests` target. `chunk_for`, `compile`, `Chunk`'s three fields, `ChunkTooLarge`, `render`,
  `Interp::chunks`/`chunks_refused` are dead in `lib` and live in `unittests` -- exactly the
  inconsistent-across-compilations shape the report claims, which is why they need `#[allow]` and
  not `#[expect]` (an `#[expect]` there would misfire as "unfulfilled lint expectation" under
  `unittests`, which would itself fail `-D warnings`). `Op::Clause` and `Op::EvalExpr` are dead in
  both targets, consistent with `#[expect]` being correct for those two. Restored all five files
  from `cp` backups and confirmed with `sha256sum -c`; `git status` clean throughout.
* `cargo clippy --workspace --all-targets -- -D warnings` re-run from a freshly deleted `target/`:
  exit 0, independently reproducing the report's claim (this project's own history has at least one
  case of a stale warm-target false green, so this was worth re-running rather than trusting).
* `cargo test --workspace`: 1340 passed / 0 failed, recomputed by summing every `test result:`
  line's own count rather than trusting a single number.
* `chunk_for`'s structure mirrors `plan_for`'s exactly, field for field, as the brief requires.
* The cache test's `#[cfg(test)] thread_local! { static COMPILE_CALLS: Cell<usize> }` counter is
  sound. It is read as a delta (`compile_calls() - before`), so accumulation from other tests
  sharing the same OS thread across a run does not matter, and Rust's default test harness runs one
  test to completion per thread before reusing that thread, so there is no concurrent interleaving
  within a thread for a thread-local counter to race against. No reset between tests is needed
  because nothing depends on the counter's absolute value.
* No `unsafe`, no em-dashes, no counts of mutable in-repo aggregates in the source comments touched
  by this commit.

## Which annotations Task 3 must remove, and which it must not

**Task 3 should remove** (the driver gives each of these a real caller, per the plan's own Task 3
text):
* `plan.rs`: `Interp::chunk_for`'s `#[allow(dead_code)]`.
* `ir/compile.rs`: `compile`'s `#[allow(dead_code)]`.
* `ir/mod.rs`: `Chunk`'s `ops`, `op_of`, `registers` field-level `#[allow(dead_code)]` (three
  attributes).
* `ir/mod.rs`: `ChunkTooLarge`'s struct-level `#[allow(dead_code)]`.
* `lib.rs`: `Interp::chunks` and `Interp::chunks_refused` field-level `#[allow(dead_code)]` (two
  attributes).

**Task 4 should remove** (per its own Interfaces list, it is the first to construct `EvalExpr`; see
I1 for `Clause`, which is not Task 4's to remove):
* `ir/mod.rs`: `Op::EvalExpr`'s `#[expect(dead_code)]`.

**Nobody is specified to remove, and the controller should not expect it to disappear at Task 3**
(see I2):
* `ir/golden.rs`: `render`'s `#[allow(dead_code)]`.

**Task 6 resolves by deleting the variant, not by constructing it** (see I1):
* `ir/mod.rs`: `Op::Clause`'s `#[expect(dead_code)]` -- expect this attribute (and the variant it
  sits on) to vanish from the diff at Task 6 as part of the described split, not to start warning
  "unfulfilled" first.
