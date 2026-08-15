# Handoff item 2 -- the corpus-wide assertion over the compiled op stream

Working notes.
BASE `973973f2`, branch `plan/rust-rewrite`.

## The shape chosen, and why

**The invariant, and a stronger one than the item proposed.**

The item offered "no instruction the minimum promotion set covers emitted `Op::Generic`".
That is one direction of one level.
What is built states the promotion set at both levels it has, computed from each body's own parse tree:

* **Instruction level.** An instruction of a kind the minimum promotion set covers must have opened an `Op::Clause` region and must not have emitted `Op::Generic`.
  An instruction *outside* the set carries no claim at all, so promoting something new cannot redden this.
* **Expression level.** For an `Assignment` or a `SAY` -- the two constructs whose value expression can compile natively -- the last value-producing op inside the clause region must be exactly the one the expression's shape calls for: `Op::Const` for a literal, `Op::LoadConstant` for a constant symbol, `Op::Load` for a bare read, `Op::Arith` for an arithmetic tree, `Op::EvalExpr` for anything else.
  This direction is bidirectional, which is what lets it see arithmetic ceasing to promote as well as promoting where it should not.

### Why not the transcript

A committed op stream per corpus program regenerates on every promotion, and Phase 4f is a loop that changes what promotes.
That is the maintenance cost the item exists to avoid handing it.

### Why not the per-program op-kind count summary

Judged on the same terms and rejected.
It is still a committed artefact with a regeneration step, and it churns in a worse place than the transcript does: a count row moves whenever *any* op is added, removed or split for that program, including changes that leave the promotion set exactly where it was.
What it buys over the invariant is noticing a changed number of `TraceLiteral`/`TraceRead`/`TraceOperator` ops -- which is trace emission, and `ir_dual.rs` already diffs raw stderr between the two arms over the corpus population, on executed bytes rather than on compiled shape.
So its marginal catch is largely held by a stronger instrument, and its cost is paid on every 4f iteration.

### What the invariant cannot see, stated plainly

* **Op operands.** It reads an op's *kind* and never its fields, so a wrong register, jump target, region `end`, constant index or `SymbolId` passes.
  A transcript catches those; this does not.
* **Ops it does not name.** A dropped `TraceLiteral`, `TraceRead`, `TraceOperator`, `EndBranch` or `Jump` passes.
* **Order** inside a region, beyond which value op is last.
* It is a claim about what compilation *emitted*, not about what running it does.

The item's own framing -- "it cannot see an op emitted wrongly, only one not emitted at all" -- is right at the instruction level and too pessimistic at the expression level, where the check is bidirectional and does see the wrong op.
It is right about operands at both levels.

## Where it lives, and one correction to the item's premises

`rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs`, a unit-test module inside the crate.

**The item says `ir/golden.rs` is "plain `pub(crate)`, no `cfg` gate". It is not.**
`ir/mod.rs` declares `mod golden;` under `#[cfg(test)]`, with a comment saying why.
So `render` is unreachable from an integration test, and the assertion cannot live beside `corpus_cases()` in `tests/ir_dual.rs`.
Inside the crate it can match on `Op` variants directly, which is better than reading `render`'s text -- `render` deliberately omits the `symbol` field, and a check written against its output would inherit that.
`render` is still used, for the failure message: a red assertion prints the whole offending stream under the program's name.

The population is read from `rust/corpus/phase-*.txt` by directory listing rather than from a third copy of `SUBSET_FILES`.
`corpus.rs` and `ir_dual.rs` each keep a literal pinned against that same listing; a third literal would be a third thing to keep in step, and the listing is what both pins resolve to anyway.
Every body is swept, not only `main`: `::ROUTINE`, `::METHOD` and `::ATTRIBUTE` bodies go through the same check, over an exhaustive `match` on `DirectiveKind` so a directive form that gains a body has to be routed rather than dropping out.

## The two anti-vacuity controls

Both are assertions in the test rather than sentences here.

* The subset-file listing is non-empty, the program list has more than one entry, and the body count is at least the program count.
  An empty population is the failure mode where the harness runs nothing and every check inside the loop passes.
* Every construct of the minimum promotion set and every expression root is observed at least once across the sweep.
  `DO` and `LOOP` are one row -- same construct, two spellings.
  Without this, a row of the classifier that nothing reaches would be a check that holds over an empty set.

## Why the expectations are restated rather than asked of `compile`

`root_of` restates what `native_shape` and `push_native` decide; `promoted_as` restates which instruction arms `compile` has; `arithmetic` spells out the seven operators that `compile` gets from `eval::is_arithmetic`.

Calling into `compile` for any of them would make the expectation move with the code under test, and the assertion could never redden.
The price is drift between the two statements, and the drift is loud in both directions: an operator dropped from the promoted set leaves an `EvalExpr` where an `Arith` is expected, one added leaves an `Arith` where an `EvalExpr` is expected, and either names the program.

## Proof 1: it can fail, and it names the program

Mutation: `compile.rs`'s `InstructionKind::Call(call) if matches!(&**call, Call::Named { .. })` guard made `if false && matches!(...)`, so `CALL name` falls through to `Op::Generic`.

```
thread 'ir::corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops'
panicked at crates/rexx-exec/src/ir/corpus_shape_tests.rs:
lang/address_env.rex main: instruction 31 (CALL name) is in the minimum promotion set and compiled to Op::Generic
```

Restored from a `cp` backup, `sha256sum -c` OK on all three touched files, rebuilt, re-run green.

## Proof 2: does it add coverage

**Two mutations, and they give opposite answers. Both are recorded.**

### The whole-construct mutation: no, it adds nothing

Same `CALL name` mutation, the module registration removed from `ir/mod.rs` so the assertion does not exist, `cargo test --workspace --no-fail-fast`:

* `ir::golden_tests::a_call_compiles_to_a_clause_region_and_one_call_op`
* `ir::golden_tests::a_traced_call_carries_its_clause_echo_in_front_of_the_call`
* `ir::golden_tests::each_call_site_takes_a_resolution_site_of_its_own`
* `ir::drive::tests::a_call_site_resolves_once_and_answers_from_what_it_kept`
* `tests/spike.rs` aborts on a stack overflow, SIGABRT, so that binary reports no result line at all

**For a promotion that stops firing outright, the existing suite catches it identically and this assertion adds no coverage.**

### The shape-conditional mutation: yes, and nothing else sees it

The class the existing suite structurally cannot cover is a promotion that keeps firing for every shape the golden set writes and stops firing for one only the corpus contains.

Mutation: `native_shape` given a depth parameter and made to refuse nesting deeper than 8.
`push_native` recurses once per operator, so a compile-time recursion bound is a change Phase 4f might reasonably want; this is not a contrivance with no motive.

| suite | result |
|---|---|
| with the assertion, `--no-fail-fast` | 1427 passed, **1 failed** -- the new assertion, and nothing else |
| without the assertion, `--no-fail-fast` | **1427 passed, 0 failed, exit 0** |

```
lang/deep_nested_expr.rex main: instruction 0 (assignment) has an expression calling for
Some(Arith) and a compiled clause ending in Some(EvalExpr)
0: Clause index=0 end=3
1: EvalExpr index=0 slot=0 dst=0
```

Every promoted expression in `golden_tests.rs` is hand-written and shallow; `corpus/lang/deep_nested_expr.rex` is a single assignment whose own header says it nests three thousand terms deliberately.
A bound anywhere between the two is invisible to every hand-written witness and visible here.

**So the honest summary is: it adds no coverage against a promotion that stops firing outright, and it adds coverage that nothing else in the workspace has against a promotion that stops firing conditionally.**
Both mutations are recorded on the file itself, so the next reader gets the limit and not only the win.

## Gates

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace`, dev | 1428 passed, 0 failed, 4 ignored, exit 0 |
| `cargo test --workspace --release`, all four STRICT gates | 1428 passed, 0 failed, 4 ignored, exit 0, four `mode: STRICT` banners |

BASE was 1427 in both profiles, so the delta is the one test added.

## What was not done

* No benchmark, `perf`, `samply` or profiler run -- the two sibling items own the machine.
  Nothing here makes a performance claim.
  The one that would need settling if anyone wants it: the sweep parses and compiles every corpus body on every `cargo test` run, and whether that is a cost worth naming is a `cargo test` wall-clock question, not an interpreter one.
* No oracle run. The assertion is over compilation and never executes a program, so the oracle has nothing to say about it.
