# Item 1: do not emit trace ops where the setting is statically known

BASE `f9ffe9a8b`. Tree clean. This is item 1 of
`.superpowers/sdd/queued/2026-09-20-performance-todo.md`; read that file's item 1
and its Kildall section before starting.

## What is wrong

`rexxcps` dispatches 127,886,118 ops, and **38,461,666 of them (30.1%) are trace
ops in a run with tracing off**. Their arm advances the pointer and re-dispatches.

The mechanism is chunk-wide and all-or-nothing.
`crates/rexx-exec/src/ir/compile.rs:192`:

    let echoes_values = trace.intermediates() || !plan.never_retraces();

`Plan::never_retraces` is one `bool` for the whole body, "whether nothing this
body runs can change the `TRACE` setting in force while it runs". So **one
`TRACE` instruction anywhere in a body makes every clause in it carry value-echo
ops**, whatever the setting actually is.

Minimal, with `rexx-ir`:

* `zw = 1` / `zv = zw + 1` / `say zv` compiles to **11 ops, no trace ops**.
* The same three clauses with `trace off` prepended compile to **18 ops** carrying
  `TraceLiteral`, `TraceRead` and `TraceOperator`.

`TRACE OFF` is a literal. Its setting is known at compile time for every clause
that follows, and we emit as though it were not.

**Moritz's reason, which outranks the percentage:** *"it would still be a good
habit to avoid work, i.e., don't emit trace instructions after trace off."*

## What to build

A per-clause answer where there is now a per-body `bool`, computed as a forward
dataflow analysis in Kildall's framework (1973, "A Unified Approach to Global
Program Optimization"). The paper is at
`<session scratchpad>/reading/kildall1973.pdf` with text beside it as
`kildall.txt`.

* **Pool:** the `ChunkTrace` in force, plus a top for "not yet reached" and a
  bottom for "not statically known".
* **Meet:** two settings that agree give that setting; two that disagree give
  bottom; top meets anything to that thing.
* **Optimizing function** `f(clause, pool)`: a clause that is not a `TRACE`
  passes its pool through; `TRACE <literal>` produces that literal's
  `ChunkTrace`; `TRACE VALUE <expr>` produces bottom.
* **Entry pool:** the `trace` argument `compile` already receives.
* **Iterate to a fixpoint**, meeting at every join. Kildall proves this correct
  and terminating, and the order successors are processed in does not matter. The
  lattice here is tiny, so it converges almost at once.
* **Emission:** a clause whose pool is a single known setting is emitted for that
  setting. A clause at bottom is emitted exactly as today.

`Plan::build` already walks the body once and produces per-index tables
(`indents`, `lines`), so the natural home is another table of the same shape.
`echoes(trace, instruction)` has 19 call sites in `compile.rs` and
`trace.intermediates()`/`trace.results()` one each; all of them become a question
about a clause rather than about the chunk.

## Hazards, in the order they will bite

1. **`INTERPRET` is the one that breaks the analysis.** `Op::Exec` runs a fragment
   compiled at run time, in the current activation, and that fragment can contain
   a `TRACE`. The enclosing graph never sees it. **Establish against the oracle
   whether a `TRACE` inside an `INTERPRET` changes the setting for clauses after
   it in the enclosing body.** If it does, `Op::Exec` must drive the pool to
   bottom for everything reachable after it. Do not assume either way, and do not
   reason from the Rexx books alone: measure it.
2. **Control-flow edges must include the ones that are not fallthrough.**
   `SIGNAL` to a label, `CALL ON`/`SIGNAL ON` trap entry, loop back edges,
   `SELECT` branches. A missing edge is an unsound "known" answer, which is a
   program that silently stops tracing. Prefer bottom wherever the edge set is
   uncertain.
3. **The staleness check must keep working.** `drive.rs:576` compares
   `chunk.trace().clause_echoes()` against the setting in force, and its comment
   says it is "the whole of what makes a compiled-in emission decision safe". The
   chunk's identity for recompilation is its **entry** setting; per-clause values
   are derived from it. Say in the report what the chunk's key is after the change
   and why a stale chunk is still detected.
4. **Internal routine calls** get the caller's setting in Rexx, and a change
   inside the routine does not propagate back to the caller. That is a claim to
   verify, not to assume.

## Sequencing

**Do not change emission in the first commit.** Land the analysis first with the
emission decision unchanged, plus an assertion that the analysis is conservative:
wherever `never_retraces()` says the body can retrace, the analysis must not claim
a known setting for a clause unless it can justify it. Then wire it into emission
in a second commit. Two commits, each gated.

## How it is judged

**The oracle differential is the arbiter, not reasoning.** Trace output is
observable on all three descriptors and the corpus has programs that trace. Run
the corpus differential and the trace programs; do not argue from the code.

The five gates as `rust/CLAUDE.md` defines them. Commit before any long run and
leave the tree frozen while one is in flight.

**Measurement, on `rexxcps` via `rust/bench-rexxcps/rexxcps.rex`** (the pinned
copy, not `samples/`):

* baseline at `f9ffe9a8b`: 21,147,250,696 instructions, 127,886,118 ops
  dispatched, 6.39 ops per clause.
* the ceiling, measured by stripping every `TRACE` instruction: 20,325,640,274
  instructions, 86,884,257 ops, 4.34 ops per clause, **-3.89%**.
* a real fix lands between the two. Report ops dispatched and instructions, both
  from `valgrind --tool=callgrind --dump-instr=yes`, and say what fraction of the
  ceiling was reached.

Count ops by summing the execution counts at the dispatch jump addresses, which is
how the baseline was derived; the method is in
`2026-09-20-instructions-per-op.md`.

## Report, and two things not to do

Write the full report to
`.superpowers/sdd/2026-09-20-trace-emission-report.md`.

**Do not fix the orphaned doc comment you will find at
`crates/rexx-exec/src/plan.rs:178`.** It reads "How the compound `id` names
splits, if this plan's pass saw it" and sits above `never_retraces()`, which is
not what it describes; the text belongs to `compound()` below it. Report it, leave
it, and keep the diff about one thing.

**Do not quote a figure you did not watch a command print.** Quote the command
beside every number and give the exit status you observed. If a run is still going
when you report, say it is still going.
