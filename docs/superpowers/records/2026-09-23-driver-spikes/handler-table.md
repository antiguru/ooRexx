# Spike: handler-table

Branch `spike/handler-table`, based on `0ba0f3876` (verified with `git rev-parse HEAD`
after `git switch -c spike/handler-table 0ba0f3876`; the worktree had first been cut
at `c2edef977` on the master lineage, nothing was built there).

Base binary: `bin/base-rexx-run`, `.text` sha256
`748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32`.

## Prediction (written before any head measurement)

Mechanism priced per region op in the head:

* added: the tag read and table load, an indirect call and return, the handler's
  own prologue/epilogue (callee-saved pushes for whatever it keeps live), reloading
  the region context (`registers`, `chunk`, `clause`, `code`) from memory instead of
  from registers, the handler's `let Op::X = op else` check (the uniform signature
  forces it), and the return-code test in the walk. Estimate 10 to 15 instructions
  per region op.
* removed: whatever part of `run_ops_from`'s spill/reload traffic comes from the
  region arms' register demand. 37.6% of its line-0 cost is spill/reload
  (`2026-09-22-driver-frame-pressure.md`); the driver is about a quarter of the
  program, so the removable ceiling is a few percent, and only part of it moves.
* the cheapest ops pay the most relatively: a gated trace op is about 20
  instructions to dispatch today (`2026-09-20-instructions-per-op.md`) and rexxcps
  dispatches about 6.4 ops per clause, a third of them trace ops.

| axis | predicted delta (instructions ex-libc) |
|---|---|
| rexxcps | +3% (plausible range -2% to +8%) |
| varlookup | +3% (6 region ops per iteration, all cheap) |
| arith | +1% (numeric work dominates) |
| emptyloop | 0% +/- 2.5% (region nearly empty; only the driver's shape moves) |
| dispatch | +1% (message sends dominate) |
| compound | +2% |

Predicted verdict: does not stick. The bet is that per-op frames are cheaper than
one big frame; I expect the call/return and context reload to cost more than the
spill traffic they remove, because the ops that dominate the counts are the ones
with almost no body.

## What was built

Head is commit `8409b817b` on `spike/handler-table`, and its binary's `.text` sha256 is
`16fc472f27d59fd56f362b8ed4d8cb132d95ae2ce9d5307ec34c1c935b0fbe4a`. The change is
`rust/crates/rexx-exec/src/ir/drive/handlers.rs` plus a 28-line hook in `drive.rs`'s
region walk:

* `RegionCtx` holds what the walk kept in locals: `code`, `chunk`, `clause`, `source`,
  `registers`, `index`, `stale`, `end`, the loop `header`, and a `cold` slot for a
  `Result<RegionEnd, Failure>`. It is built once per `Clause` op.
* Every handler has the same signature, `fn(&mut Interp, &mut RegionCtx, &Op) -> u32`,
  and answers `CONTINUE` (`u32::MAX`), `COLD` (`u32::MAX - 1`, with the outcome left
  in `cx.cold`), or the op index the region jumps to.
* The dispatch is `handler(op)(self, &mut cx, op)`. `handler` is an exhaustive
  `match` that returns function pointers, and LLVM lowered it to one table load and an
  indirect call: `movzbl (%r14),%eax; call *(%rcx,%rax,8)` at `0x1369e2`.
* Handlers exist for the ops the six axes execute inside a region, as measured by
  callgrind call counts on a first build (`head1`). The seven handlers that no axis
  executed (TraceClause, EvalExpr, Prefix, TracePrefix, Say, JumpUnless, Jump) were
  folded into the fallback before the head was committed.
* Every other op sits in the table's `h_other` slot, which calls
  `region_op_fallback`. That function holds the original arms. There is no per-op test
  and no per-region decision.
* The `debug_assert!(chunk.holds_register(..))` tripwires were not carried into the
  handlers. They are debug-only, so release behaviour is unchanged.
* `handlers::TABLE = false` is the control.

Correctness at `8409b817b`:
* `cargo test -p rexx-exec --release --no-fail-fast`: exit 0, 1578 passed, 0 failed.
  This needed the untracked `build/` and `ootest/` symlinks and `rust/target` pointing
  at the test build. Without them, 14 and then 18 tests failed on missing fixture
  paths, and none of those failures involved the interpreter.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus
  corpus_differential` printed `604 of 604 matching` in `mode: STRICT`.

## Results (callgrind `summary:` minus libc.so.6 and ld-linux, mean of 2 rounds, interleaved)

| axis | base ex-libc | head ex-libc | delta |
|---|---|---|---|
| rexxcps | 18,638,733,571 | 20,510,344,622 | **+10.04%** |
| varlookup | 17,122,896,896 | 19,345,913,576 | +12.98% |
| arith | 11,764,230,174 | 12,070,606,087 | +2.60% |
| emptyloop | 9,685,885,286 | 10,360,890,268 | +6.97% |
| dispatch | 20,711,081,273 | 21,101,084,998 | +1.88% |
| compound | 9,914,435,956 | 10,684,480,602 | +7.77% |

Within one binary, the two rounds differed by at most 10,612 instructions on every
axis, except the control's rexxcps, which differed by 4.77M (0.03%). Every run
reported rc=0. Each result row carries the measured binary's `.text` hash prefix:
base `748b2f0679dc`, head `16fc472f27d5`, control `f7ac8c98d58c`, noinline
`707355a2329d`.

### Control: table present, unused (`TABLE = false`, handlers kept alive by a `#[used]` static)

| axis | control delta vs base |
|---|---|
| rexxcps | +0.01% |
| varlookup | +0.00% |
| arith | -0.00% |
| emptyloop | +0.00% |
| dispatch | -0.00% |
| compound | +0.00% |

Having the handlers and the table in the binary costs nothing. The whole +2% to +13%
comes from running ops through them.

### Second control: head with `#[inline(never)]` on `run_ops_from`

In the head, the shrunk `run_ops_from::<true>` was inlined into `run_activation`, and
`nm` has no `run_ops_from` symbol. I pinned it out of line (`noinline.patch`, not
committed) to separate that inlining from the handler cost:

| axis | noinline delta vs base |
|---|---|
| rexxcps | +9.64% |
| varlookup | +11.65% |
| arith | +2.43% |
| emptyloop | +5.42% |
| dispatch | +2.46% |
| compound | +7.16% |

The inlining accounts for less than 1.6 points on every axis, and on dispatch it
helped the head (+2.46% without it, +1.88% with it). The handler structure itself
costs the rest.

### Symbols that are sensitive to inlining (from the cold-arms finding)

* `Interp::arith_small_int`: an out-of-line copy exists in all four binaries, but it
  has zero self-Ir on varlookup, rexxcps and emptyloop in both base and head. It is
  inlined at the hot site in both: into `run_ops_from` in the base and into `h_arith`
  in the head. So the +342M flip did not happen here.
* `drop_glue::<Option<LoopHeaderValues>>`: base rexxcps spends 28,201,025 Ir in it
  (5,640,205 calls); the head spends none, because the header now lives in
  `RegionCtx`. On varlookup and emptyloop it is 20 Ir in both.
* On both loop axes, `drop_glue::<ControlValue>` in the head and
  `drop_glue::<Option<Number>>` in the base cost the same amount (57M on varlookup,
  75M on emptyloop). The same code is attributed to a different symbol, so the net
  is 0.

### Where the head's extra instructions go

* **emptyloop executes no region op at all**: 288 handler calls over the whole run.
  The noinline build still adds +549,997,067 Ir to `run_ops_from::<true>`, which is
  **22 Ir per clause** over 25M clauses. That is the cost of building and dropping
  `RegionCtx` on every region entry: the stores, `header: None`, `cold: None`, and the
  drop checks for both. Across rexxcps's 20M clauses the same price would be about
  440M, or 2.4 of its 10 points.
* **varlookup**: the head retires 2,223,016,680 more instructions over 114,000,290
  region ops, **+19.5 Ir per region op**.

Dispatch counts, measured with `--dump-instr=yes` at the dispatch instructions:

| axis | dispatch site | base | head |
|---|---|---|---|
| varlookup | outer `jmp *%rdx` (drive.rs:544) | 57,000,257 | 57,000,257 |
| varlookup | region dispatch | 114,000,290 (`jmp *%rax`, 0x13d387) | 114,000,290 (`call *(%rcx,%rax,8)`, 0x1369e2) |
| emptyloop | outer | 50,000,256 | 50,000,256 |
| emptyloop | region | 288 | 288 |

Instructions per dispatched op (ex-libc total over outer plus region dispatches):

| axis | base | head |
|---|---|---|
| varlookup | 100.1 | 113.1 |
| emptyloop | 193.7 | 207.2 |

The head's handler call counts equal the base's jump-table counts exactly, so both
builds dispatch the same ops.

The head's self-Ir per call, on varlookup:
* `h_store`: 52.0
* `h_load`: 40.0
* `h_arith`: 73.0
* `h_load_constant`: 32.0

The body of `run_ops_from` that these replaced fell from 10.45G (base) to 7.20G
(inlined into `run_activation` in the head). It did not fall by enough to pay for the
handlers.

### Handler frames (description only: prologue pushes / `sub rsp`)

* Most handlers are small: push_arg, trace_argument, trace_literal, trace_read,
  trace_operator, call_named, parse, exec, expose, return, message and condition_jump
  each take 2 pushes and 56 to 88 bytes.
* load: 4 pushes / 104. binary: 4 / 72. store: 3 / 80. arith: 6 / 120. const: 5 / 64.
  load_constant: 5 / 64. call_args: 5 / 80. trace_function: 3 / 64.
* loop_header_value takes 6 / 232 and loop_run takes 6 / 1160, because
  `flat_loop_start` and `run_loop_with_header` are inlined into them.
* `region_op_fallback` takes 6 / 408.
* For comparison, the base's `run_ops_from::<true>` takes 6 pushes / 1416. The head
  inlines the driver into `run_activation`, which takes 6 / 936. With `inline(never)`,
  the head's own `run_ops_from::<true>` is 6,023 bytes against the base's 15,061.
* Even the trace handlers, which only test a flag, pay a full prologue and epilogue
  (`push r14; push rbx; sub $0x38,%rsp`). The `let Op::X = op else` check costs a
  `cmpb` and a `jne` per op.

## Prediction vs result

| axis | predicted | measured |
|---|---|---|
| rexxcps | +3% | +10.04% |
| varlookup | +3% | +12.98% |
| arith | +1% | +2.60% |
| emptyloop | 0% +/- 2.5% | +6.97% |
| dispatch | +1% | +1.88% |
| compound | +2% | +7.77% |

The direction was right and the size was about three times too small. The part I did
not predict is the per-clause cost of building the context (22 Ir per clause with no
op run). I also underpriced the per-op cost: I guessed 10 to 15 Ir, and varlookup
measured 19.5.

## Verdict

Does not stick. Call-threading the region walk costs +10.04% on rexxcps and makes
every axis worse. The table-present-unused control is flat (+0.01% or less), so the
cost comes from running ops through small functions. The driver shape changing does
not explain it: pinning `run_ops_from` out of line accounts for less than 1.6 points
on any axis.

## Concerns

* Much of the loss is the context build, not the calls. A version that passes
  `registers` and `clause` as arguments, or keeps `RegionCtx` alive across clauses,
  would cost less per clause. The varlookup per-op figure (+19.5) says the calls alone
  would still lose, though.
* `Op` is 16 bytes with the discriminant in byte 0, so dispatch is one load. A
  pre-resolved `Vec<fn>` would not save anything there, and it would add a second
  stream.
* I did not try a nightly `become` variant.
* Only two rounds per binary, but the spreads are four orders of magnitude below the
  effect.
