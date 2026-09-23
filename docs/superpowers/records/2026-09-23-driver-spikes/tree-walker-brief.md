# Bake-off: the tree-walker against the IR

Read `worktree-setup.md` beside this file first. Branch: `spike/tree-walker`.

## Why

The C++ oracle is a tree walker at 534 instructions per clause on `rexxcps`;
our IR is at about 1,000. Round 1 showed the IR driver's code generation alone
is worth about 13% (PGO ceiling). Moritz wants a bake-off: bring the Rust
tree-walker back and measure it against the IR on today's tree.

**Prior art you must read and cite, not repeat:** project memory records that
on 2026-09-10 at `86ca524e6` the tree-walker was slower than the IR on all ten
axes, 0.9% (`dispatchclass`) to 85% (`varlookup`), same binary under
`REXX_ENGINE`, measured in cycles/wall. It was deleted 2026-09-11 by
`47ae2a18d` (`Op::Generic`), `64cc47c81` (dual harness), `8657503a9` (the
engine) and `458f49f76` (INTERPRET compiled, remainder deleted). The IR has
had twelve days of optimisation since; the tree-walker had none before or
after. So the question is not "which was faster on 2026-09-10" but **where
each engine's instructions per clause go**, and whether the tree shape has a
structural advantage the IR's driver cannot get.

## Hard constraint (Moritz)

**No IR generic op ever comes back.** No `Op::Generic`, no op, arm or fallback
by which IR execution delegates a clause, instruction or body to the
tree-walker. The two engines are alternatives for a whole run, never mixed.
Make this structural, not a comment:
* the restored engine lives behind a cargo feature, `tree-walker`, **off by
  default**, so the default build contains no tree-walker at all and any IR
  code that referenced it would fail to compile;
* engine choice is made once per run (an `Invocation`/construction field,
  plus a `rexx-run` flag for measurement), never per clause or per body;
* a test (feature-on) asserts the IR sources under `rust/crates/rexx-exec/src/ir`
  and `ir.rs` do not name the tree-walker module, and fails if they do. Prove
  it can fail by adding a reference and watching it go red, then remove it.

Shared code the IR already uses (`exec_instruction`, `eval`, `Op::Exec`'s
path) is fine for the tree-walker to call; the direction that is forbidden is
IR into tree-walker.

## Phase A: the historical pair, cheap, first

Find the last commit where both engines ran under `REXX_ENGINE` (at or just
before `47ae2a18d`; verify the switch works there). Build it once in its own
target dir (a temporary `git worktree add --detach` in your scratch; remove it
with `git worktree remove` afterwards), and run callgrind on both engines for
`rexxcps` (pinned copy if it exists there, else say what you used) and the six
common axes. Report ex-libc instructions and **instructions per clause** on
`rexxcps` for each engine. This is a number nobody has: the 2026-09-10
figures were cycles.

## Phase B: restore on today's tree

Restore the engine onto `b5dd351d6` behind the feature, by reverting or
re-applying the deletion commits' engine parts, adapted to twelve days of
change. It must be **correct**, or its numbers mean nothing:
`corpus_differential` 604 of 604 STRICT with the tree-walker selected for the
whole corpus (add a feature-gated way to run the corpus under it). If some
construct added since the deletion has no tree path, the tree-walker refuses
it loudly, you list which corpus rows fail and why, and the bake-off uses only
axes that run -- but try to reach 604 first; a stub is a last resort.

The default build must be unchanged: build the default (feature-off) release
and confirm its `.text` hash equals base's. If it does not, find out why
before measuring anything.

## Phase C: the bake-off

Same binary, feature on, IR against tree-walker, callgrind ex-libc, two
interleaved rounds, the six axes plus `rexxcps`. Also the feature-on binary's
IR against the feature-off base binary's IR, so we know whether merely
compiling the tree-walker in moves the IR (it can: round 1 showed unrelated
code moves allocation).

Then the part that answers the question: on `rexxcps`, split each engine's
cost into the categories of
`docs/superpowers/records/2026-09-20-performance-items/2026-09-22-category-rollup.md`
(use its method and scripts if committed; the callgrind line column is
subposition-compressed, see that record) so the table reads: interpretation
machinery, variable access, arithmetic, PARSE, allocation, ... for IR, for the
tree-walker, and the oracle's column from that record. Where does the
tree-walker spend less than the IR, and where more?

## Report

Per the common contract. Lead with Phase C's table and the category split;
then Phase A; then correctness and the no-generic guarantee (with the red
run); then concerns. Commit the restored engine, the tests and the report to
`spike/tree-walker`. Do not merge anything.
