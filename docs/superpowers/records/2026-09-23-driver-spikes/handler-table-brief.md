# Spike: handler-table -- one function per hot op, dispatched through a table

Read `common.md` beside this file first; it binds you.

## The idea

The driver is one giant function, so every op pays for register decisions made
for every other op (`2026-09-22-store-fusion-report.md`: a four-op difference
moved `varlookup` by 342 million instructions). The oracle, a tree walker,
executes each clause kind in its own small C++ method with its own small frame.

Try the structural opposite of the big match: **each hot op becomes its own
small function** with a uniform signature, e.g.
`fn(&mut Interp, &mut Ctx, &Op) -> Next` where `Ctx` holds what the region walk
keeps in locals (registers base, pc, chunk, etc.), and the region walk becomes
`loop { next = HANDLERS[op.tag()](self, &mut ctx, op); ... }` or a pre-resolved
`Vec<fn ...>` built beside the op stream when the chunk is compiled
(closure/"call-threaded" compilation). Rust has no stable guaranteed tail
calls, so this is call-threading through a trampoline, not direct threading;
the bet is that small per-op frames beat one big frame by more than the
call/return costs. `#![feature(explicit_tail_calls)]` / `become` is nightly
only; the shipping build is stable, so do not depend on it, but if you have
time a nightly-only variant is a fair extra data point, clearly labelled.

## Scope

* Only the **inner region walk** (`for region_op in ops {` in
  `run_ops_from`), and only the ops that the six axes actually execute there.
  Find them with `callgrind --dump-instr=yes` against the dispatch jump table,
  per `2026-09-20-instructions-per-op.md`.
* Every other op keeps its existing path. A region containing an op that has no
  handler can take the existing match, decided once per region or per chunk,
  **not** by a per-op test added to the hot path.
* Behaviour must be byte-identical: correctness floor in `common.md`.

## What to measure, beyond the common contract

* Op counts (execution counts at the dispatch sites) for base and head, so the
  per-op price before and after is stated: base instructions per dispatched op
  vs head instructions per dispatched op on `varlookup` and `emptyloop`.
* **Control**: build the handler table and the trampoline but route every op
  through the existing match (table present, unused). That separates "the
  driver function changed shape" from "ops ran in small functions".
* The frame size of each handler you add, as description only.

Spike name: `handler-table`.
