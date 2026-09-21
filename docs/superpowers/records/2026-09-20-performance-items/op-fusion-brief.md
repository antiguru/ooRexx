# Items 3 and 5: fuse two adjacent op pairs, each in its own commit

BASE is whatever `git rev-parse HEAD` answers when you start; the tree is clean
and gated at `4ef6af5bb`. Read items 3, 4 and 5 of
`.superpowers/sdd/queued/2026-09-20-performance-todo.md`, and the section
"Item 2 was built and reverted, 2026-09-21" in the same file, before starting.
That section is the shape of failure this brief exists to avoid.

## Why these two and not the others

Every dispatched op costs about **20 instructions before it does any work**,
derived in `.superpowers/sdd/queued/2026-09-20-instructions-per-op.md`.
`rexxcps` dispatches 127,886,118 ops for 21.1 billion instructions.

Item 2 failed because it **added a per-clause branch** to buy a reduction, and
the branch cost what the reduction saved. Project memory records the constant it
ran into: any conditional in the driver's `Op::Clause` arm costs about 0.5% on
`rexxcps` and 1.25% on `emptyloop` whatever it does.

**These two items add nothing to the hot path.** They remove dispatches by
emitting one op where two are emitted now. A program that never hits the pattern
pays one more arm in a jump table and nothing else. That is the whole reason they
are next.

## Item 3, first commit: `Condition` + `JumpUnless`

`crates/rexx-exec/src/ir.rs:261` and `:265`. On `rexxcps` both execute exactly
5,900,006 times.

**Equal executed counts do not prove adjacency**, and adjacency is what the
fusion needs. Fuse a *statically adjacent* pair in the emitted stream and the
soundness question does not arise; then report how many static pairs you found
and how many `Condition` and `JumpUnless` ops were left unfused. If the fusion
does not cover essentially all of the 5.9M, say so with the count rather than
quoting the item's 0.56%.

The fused op does everything the pair does: validate the value, emit the `>>>`
line it owes, and branch. Nothing is skipped, so no trace behaviour changes.

## Item 5, second commit: a constant load with the store that consumes it

`Op::Const { dst, konst }` at `:154`, `Op::LoadConstant { symbol, dst }` at
`:158`, `Op::Store { index, at, src }` at `:209`. Both pairs, both directions:
`Const`+`Store` and `LoadConstant`+`Store`.

**Size it before building it.** The item's ~1.4% assumes every constant load
feeds a store. Count the adjacent pairs in the emitted stream for `rexxcps`,
weight them by their executed counts, and put that number in the report. If it
comes out under about 0.3%, say so and stop rather than building it.

### Two hazards, and they are the whole of the correctness risk

1. **The register must be dead after the store.** Fusing writes the constant
   straight to the slot, so anything that reads the load's `dst` afterwards
   reads a register that was never written. Check it rather than assuming a
   temp is single-use: scan the rest of the emitted region for a read of that
   register, and refuse the fusion when you find one.
2. **Nothing may branch between the two ops.** A jump whose target is the
   `Store` enters a fused op in the middle. Refuse the fusion when any `Jump`,
   `JumpUnless` or any other target in the chunk names that index.

## The hazard both items share, and it is the one that will bite

Collapsing two ops into one **renumbers every op after it**. Every `target: u32`
in the stream, and every side table keyed by an op index, has to be remapped.
`Chunk`'s hint tables are dense over sites rather than over the stream, which is
exactly the kind of table that renumbers silently and wrongly.

**Enumerate what is keyed by an op index before you write the pass**, and put
that enumeration in the report. Do not answer it from memory or from a grep for
`u32`: ask the compiler by changing the index type's spelling if that is what it
takes, or name each table and say how you checked it.

A remapping error is a program that jumps to the wrong op. Some of those crash
and some of them silently compute the wrong answer, which is why the corpus
differential below is the arbiter.

## How it is judged

**The oracle differential is the arbiter, not reasoning.** Run the corpus.
Control flow and variable assignment are what these two items touch, so almost
every corpus program exercises them.

The golden op-stream tests (`crates/rexx-exec/src/ir/golden.rs`,
`golden_tests.rs`, `corpus_shape_tests.rs`) will move. **That movement is the
instrument that shows the fusion happened**: quote the before and after op counts
for a program whose stream you can read, rather than only regenerating them.

Gates as `rust/CLAUDE.md` defines them, with the split the earlier items found:

    cargo build --workspace --all-targets --release                              # outside the cap
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast # under it
    cargo build --workspace --all-targets                                        # outside
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast           # under it

plus `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
-D warnings`. **`memcap 8G` kills a cold release compile**: `rustc` peaks over
the cap, rc 137, tests never start, and the last log line is a `Compiling`. A
status-only reading calls that a red gate. Build outside the cap, test under it.

Commit before any long run and leave the tree frozen while one is in flight.

## Measurement

`valgrind --tool=callgrind` on the pinned `rust/bench-rexxcps/rexxcps.rex`, not
`samples/`. BASE is 127,886,118 ops and about 21.148 billion instructions; the
last run measured the noise band at 0.0210% between two rounds of one build, so
take two rounds per build and give the spread.

Count ops by summing the execution counts at the dispatch `jmp *` addresses in
every `Interp::run_ops_from` instantiation; the method and the expected
per-address rows are in `2026-09-20-instructions-per-op.md`.

**Also measure `bench-programs/emptyloop.rex` and `varlookup.rex`.** They are the
control for the failure item 2 hit: a program that does not benefit must not pay.
An added jump-table arm is not free by assumption, it is free by measurement.

**Give separate revisions separate target directories.** Two revisions once
produced the same binary sha256 here because one build was stale; print the
sha256 of each binary you measure beside its figure.

## House rules

`rust/CLAUDE.md` governs. Comments are minimal: a one-sentence overview, then
parameters, returns, panics and non-obvious properties only. **A comment may
never state the size of a set.** No em-dashes. Never drop or re-wrap an existing
comment. Inserting an item under a doc block silently reassigns that doc to your
new item, and neither `fmt` nor `clippy` sees it, so check what sits above every
insertion point.

No `unsafe`. No new dependencies. No process-global state.

## Report

Write to `.superpowers/sdd/2026-09-21-op-fusion-report.md`; if your harness
refuses to write `.md` files, return it as text and say so.

Return only: status, commits, the static and executed pair counts for each item,
the instruction and op measurements against BASE with the spread, the
`emptyloop` and `varlookup` control figures, gate results with exit statuses, the
enumeration of what is keyed by an op index and how you checked it, and any
concern.

Quote the command beside every figure and give the exit status you observed. If a
run is still going when you report, say so rather than describing what it will
say. Never amend a commit; a follow-up commit is the answer.
