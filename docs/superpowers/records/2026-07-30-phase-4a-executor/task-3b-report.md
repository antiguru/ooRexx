# Task 3b report: `rexx-parse` drops a deep expression by recursion

Status: DONE. Commit: 0f33843a "Make three recursive Expr-tree walks iterative, not just Drop"

## Resolution (team-lead's reply)

Team-lead confirmed the Step-1 finding was correct and widened this task's
scope to cover both real causes: `block.rs`'s `visit_expr` (fix first, since
it's what actually blocks the corpus test) and `Expr`'s `Drop` (fix second,
real on its own merits). The parenthesis-descent parser recursion (a third,
much deeper and differently-shaped cliff, ~85,000 terms on a sized thread)
stays a separate Task 3c, explicitly not mine: the fix there is a depth
counter that raises 11.1, not an iterative rewrite, because the oracle
raises a condition at its own cliff rather than crashing.

While verifying against all three test binaries team-lead named, found a
**fourth** recursion, not mentioned in either message, in shared test
infrastructure: `rust/crates/rexx-parse/tests/gate_walk/mod.rs`'s `each_expr`
(used by both `tiling.rs` and `variants.rs`) has its own hand-written
recursive `walk`, same shape as `visit_expr`, same 3000-term corpus program
tripping it. Fixed it too (see below) rather than stopping again to ask,
since: the fix is mechanically identical to the two already approved, it
touches only test-only code with no behavioural surface, `git status` showed
no other agent had uncommitted work there, and leaving it unfixed would
contradict the explicit instruction to get exactly these three tests green.
Flagging this decision explicitly in this report and in the completion
message rather than treating it as pre-approved.

## Step 1: find the actual cliff

Built a throwaway example, `rust/crates/rexx-parse/examples/depth_probe.rs`
(not part of the deliverable, deleted before the final commit), that builds
`total = 1 + 1 + ... + 1` with N terms and runs one of four modes on a
`std::thread::Builder` with no explicit `stack_size` (the std default, same
as what a `cargo test` test thread runs on):

- `parse` / `parse_leak`: call `parse_program`, then `drop` or
  `std::mem::forget` the result.
- `build_drop` / `build_leak`: build a left-leaning `Binary` chain directly
  via `Expr::binary` in a plain iterative loop (no parser involved at all),
  then `drop` or `forget` it.

Binary search on `parse` (drop the `Program` normally): **the cliff is
exactly N=2450 terms.** 2449 succeeds, 2450 aborts, reproduced 3x each side.
So the corpus file's 3000 is indeed past the cliff, not at it, matching the
observed test failure.

**This contradicts the brief's diagnosis that recursive Drop is the cause.**
Evidence:

1. `parse_leak` (parse, then `forget` instead of `drop`, so the recursive
   `Drop` impl never runs at all) crashes at the *same* depth, 2449-2450, not
   deeper. If recursive Drop were the binding constraint, leaking should have
   let much deeper inputs parse successfully.
2. `build_drop`/`build_leak` (an iteratively-built tree of the identical
   shape, `Expr`'s current derived/compiler-generated recursive Drop, no
   parser involved) survive fine up to 10,000 and only start aborting
   between 10,000 and 20,000. So recursive Drop's own cliff is roughly
   6-8x deeper than the parse cliff, not the same thing.
3. Ran the actual failing test under `gdb` (`handle SIGSEGV stop nopass`,
   the test binary itself, not the throwaway example) and got a real
   backtrace at the moment of the guard-page hit. It is not in any `Drop`
   glue at all:

   ```
   #0 rexx_parse::ast::ExprKind::for_each_child<...> (ast.rs:215)
   #1 rexx_parse::block::visit_expr<...> (block.rs:984)
   #2 rexx_parse::block::visit_expr::{closure#0}<...> (block.rs:984)
   #3 rexx_parse::ast::ExprKind::for_each_child<...> (ast.rs:226)
   #4 rexx_parse::block::visit_expr<...> (block.rs:984)
   ... (repeats, one frame per chain element)
   ```

   `block.rs`'s `visit_expr` (`block.rs:976`-`985`) is a hand-written
   recursive walk over `Expr`'s children (via `ExprKind::for_each_child`),
   called from `BlockBuilder::add_clause` (`block.rs:342`-`360`) on every
   instruction as it's appended to the chain, to populate `self.referenced`
   (read by `GUARD`/exposed-variable handling per the comment at
   `block.rs:347`-`353`). This runs at **parse time**, on the left spine of
   the freshly built expression tree, one recursive call per chain element.
   It is what overflows, and it does so before the `Program` is ever
   returned to a caller who might drop it.

So: `Expr` deriving Drop by recursion is a real, separate exposure (it does
start aborting somewhere between 10,000 and 20,000 terms on a default
2 MiB stack, confirmed by `build_drop` above), but it is **not** what makes
`cargo test -p rexx-parse --test program
the_corpus_exercises_at_least_one_directive_with_a_body` abort today. That
crash is `block.rs`'s `visit_expr`, a plain recursive function outside the
file the brief scopes me to touch (`ast.rs`). Giving `Expr` an iterative
`Drop` impl, by itself, will not turn the corpus test green: parsing itself
aborts at 2450 terms, before any `Program` value exists to drop.

Messaged team-lead (`main`) with this finding before proceeding, per the
brief's "ask me before implementing if anything looks wrong."

## Step 3 (done, independently of the Step-1 scope question)

Implemented the iterative `Drop` for `Expr` in `rust/crates/rexx-parse/src/ast.rs`
anyway, since it is real (confirmed by `build_drop` above: the *current*
derived recursive Drop does start aborting between 10,000 and 20,000 terms)
and is needed regardless of how the `visit_expr` question above is resolved.

Shape: a `for_each_child_mut` twin of the existing `for_each_child` (same
variants, same order, on `ExprKind`), plus `impl Drop for Expr` that takes
each child out of its `Box`/`Vec` slot with `mem::replace` into a `Vec<Expr>`
worklist (leaving a childless `cheap_leaf()` behind so the slot's own,
automatic drop is O(1)) and drains the worklist in a `while` loop. No
`unsafe`. Verified with the `build_drop` mode of the throwaway probe: a tree
built the same way, then dropped, now survives 200,000 terms on a default
2 MiB thread (previously aborted between 10,000 and 20,000).

**This alone does not turn the corpus test green.** Ran
`cargo test -p rexx-parse --test program
the_corpus_exercises_at_least_one_directive_with_a_body` with the `Drop` fix
in place: it still aborts with "has overflowed its stack / fatal runtime
error: stack overflow", exit signal 6. Confirms the Step-1 finding directly
on the real test, not just the throwaway probe: `parse_program` itself
still aborts on the 3000-term corpus file, in `block.rs`'s `visit_expr`,
before a `Program` ever exists to drop.

## Step 4, done ahead of the scope answer (uses the "build" construction, not the parser, so it is independent of the `visit_expr` question)

Tested three of `Expr`'s four derives at depth, using the same
iteratively-built left-leaning `Binary` chain `build_drop` uses (so a
derive's own cliff is isolated from parsing and from `Drop`, which is now
fixed):

- **`Debug`** (`format!("{e:?}")`): cliff between 2000 and 2050.
- **`PartialEq`** (`e == e.clone()`, which exercises both `Clone` and `PartialEq`):
  cliff between 2100 and 2200.
- **`Clone`** alone (clone then drop the clone): cliff between 2100 and 2200.
- **`Eq`**: not tested separately. It is a marker trait with no method of
  its own here (`Expr` has no custom `Eq`), so it has no runtime behaviour
  beyond what `PartialEq` already exercises.

All three tested derives are recursive (as expected, since none has a
custom impl) and share the same shallow exposure class as `visit_expr` and
the pre-fix `Drop` had before this task, roughly 2000-2500 terms on a
default 2 MiB stack, i.e. well below the corpus file's 3000 and far below
the oracle's 100,000. **None of these three is fixed by this task**, since
the brief scopes Task 3b to `Drop` alone (Step 3: "Implement an iterative
Drop for Expr", not the other derives) and fixing a derive would mean
replacing it with a hand-written impl, which is a bigger, separate change
per derive. Recording this as a known limit rather than working around it,
per the brief's own instruction for Step 4.

## Step 2 (revised): `visit_expr` fixed, plus the fourth recursion in `gate_walk`

`block.rs`'s `visit_expr` rewritten to an explicit `Vec<&Expr>` stack instead
of recursing through `for_each_child`. `for_each_child`'s own signature had
to change too, from an elided (fresh-per-call, effectively higher-ranked)
closure lifetime to `pub(crate) fn for_each_child<'a>(&'a self, f: &mut impl
FnMut(&'a Expr))`, because an explicit worklist needs to stash a yielded
`&Expr` past the call that produced it, which the original signature's
higher-ranked closure bound did not allow (`E0521`). Confirmed this widens
rather than narrows: every existing call site (`Expr::new`'s span widening,
the old recursive `visit_expr`) only reads a child inside the closure body
and still compiles unchanged. `referenced` (what `visit_expr` feeds) is a
`BTreeSet` read only through `.contains`, so visitation order is not
observable and the stack-based traversal is free to differ from the
recursive one's order.

While confirming this against the three test binaries team-lead named,
found `rust/crates/rexx-parse/tests/gate_walk/mod.rs`'s `each_expr` has the
same shape of hand-written recursive walk (its local `walk` function,
calling itself through `children_of`), shared by `tiling.rs` and
`variants.rs`, and it was also aborting on the same corpus file. This is a
**fourth** recursion neither of team-lead's two messages named. Fixed it the
same way (explicit stack), with one added wrinkle: `each_expr`'s own doc
comment promises "parents before children", i.e. preorder, so the fix pushes
each node's children in reverse before popping, which reproduces the
recursive version's exact traversal order (preorder, left-to-right among
siblings), not just "the same set of nodes" the way `visit_expr`'s fix could
get away with.

Reasoning for fixing this without asking first, recorded here rather than
assumed pre-approved: the change is test-only code with no shipped
behavioural surface, mechanically identical to the two changes already
explicitly approved, `git status` showed no other agent had uncommitted work
in `tests/gate_walk/`, and leaving it broken would have left two of the
three tests team-lead explicitly asked me to verify by name still red.

## Step 3: `Drop` -- see above, unchanged by the scope widening.

## Verification (Step 5)

All three named tests, individually:
```
cargo test -p rexx-parse --test program the_corpus_exercises_at_least_one_directive_with_a_body
  -> ok, 1 passed
cargo test -p rexx-parse --test tiling every_corpus_program_tiles
  -> ok, 1 passed
cargo test -p rexx-parse --test variants every_variant_is_constructed_by_the_corpus_and_samples
  -> ok, 1 passed
```

`cargo test -p rexx-parse --no-fail-fast`: every binary green, including the
new `tests/deep.rs` (2 passed) and the two gate-walk-dependent binaries
above.

`cargo test --workspace --exclude rexx-exec --no-fail-fast`: every binary
green, 0 failed anywhere. `rexx-exec` itself does not currently compile
(`missing field stack_entry in initializer of Interp`, `lib.rs:595`) --
another agent's in-progress work per the shared-worktree note in the brief;
excluded rather than touched, exactly as instructed. Also found (untracked,
not mine) `rust/crates/rexx-exec/examples/m1check.rs`, confirming that
crate is actively being edited elsewhere; left entirely alone.

`cargo clippy --workspace --exclude rexx-exec --all-targets -- -D warnings`:
clean. `cargo fmt -p <pkg> --check` for every non-`rexx-exec` crate: clean
(ran per-package since `cargo fmt --workspace` would need `rexx-exec` to
parse too; it does, `cargo fmt` only needs syntax not a full build, but
per-package avoids any ambiguity about which crate a diff belongs to).

`examples/depth_probe.rs`: kept and given a doc comment recording what it
measures and the four cliffs as measured when it was written (three from
the brief's table, plus the three derive cliffs from Step 4), per
team-lead's explicit instruction not to delete it. Re-ran `parse` mode after
all three fixes: now handles 500,000 terms cleanly on a default 2 MiB
thread, comfortably past the oracle's own 100,000-term working point.

`tests/deep.rs`: two tests, 5,000 terms (past the original ~2450 cliff) and
100,000 terms (the oracle's own depth), both parse and drop cleanly.

## Known limits, carried forward rather than fixed here

- `Debug`, `PartialEq`/`Clone` on `Expr` are still recursive derives with
  their own ~2000-2500-term cliff (Step 4 above). Not this task's scope
  (team-lead's widened scope was named as exactly two: `visit_expr` and
  `Drop`); flagged as a known limit.
- The parenthesis-descent recursion in `parse_subterm` (~85,000 terms on a
  sized thread) is Task 3c, explicitly not mine, per team-lead.

## Files touched

- `rust/crates/rexx-parse/src/ast.rs`: `for_each_child`'s lifetime named;
  `for_each_child_mut` added; `impl Drop for Expr` added; `cheap_leaf()`
  helper added. No existing code deleted.
- `rust/crates/rexx-parse/src/block.rs`: `visit_expr` rewritten iteratively.
- `rust/crates/rexx-parse/tests/gate_walk/mod.rs`: `each_expr`'s local
  `walk` rewritten iteratively.
- `rust/crates/rexx-parse/tests/deep.rs`: new, two tests per Step 2.
- `rust/crates/rexx-parse/examples/depth_probe.rs`: new, the measurement
  instrument, kept per team-lead's instruction.

Not touched: anything under `rust/crates/rexx-exec/` or `rust/corpus/`.

## Commit

`0f33843a`, "Make three recursive Expr-tree walks iterative, not just Drop".
Staged and committed exactly the five files above (nothing under
`rexx-exec`, nothing under `rust/corpus/`). `git status` after the commit
still shows two other agents' in-progress files
(`docs/superpowers/plans/2026-07-30-phase-4a-executor.md`,
`rust/crates/rexx-exec/src/lib.rs`), untouched by me.
