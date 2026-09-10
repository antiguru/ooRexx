# Paths to parity with the oracle

Measured 2026-09-10 at `8565dcfa8`, `rustc 1.98.1`. Wall clock, interleaved,
three reps, median, the same file given to both sides, oracle under
`ulimit -v 1048576` from an empty directory.

| axis | crate (ms) | oracle (ms) | ratio |
| --- | ---: | ---: | ---: |
| `emptyloop` | 551 | 896 | **0.61x** |
| `alloc4c` | 643 | 1034 | **0.62x** |
| `compound` | 723 | 1153 | **0.63x** |
| `varlookup` | 907 | 1195 | **0.76x** |
| `arith` | 1360 | 1160 | 1.17x |
| `startup` | 26 | 15 | 1.73x |
| `dispatch` | 2334 | 1309 | 1.78x |
| `strings` | 1567 | 870 | 1.80x |
| `dispatchclass` | 1857 | 1025 | 1.81x |
| `alloc` | 2093 | 1158 | 1.81x |
| `rexxcps` | 8,779,990 cps | 17,283,590 cps | 1.97x |

Four axes are ahead of the oracle. `alloc` was 3.5x, `dispatchclass` 2.74x and
`dispatch` 2.26x on 2026-09-05; the hashing and dispatch round of 2026-09-09/10
is what moved them.

## The observation the paths are drawn from

**The remaining cluster is uniform.** String building, two dispatch shapes,
allocation churn and a mixed workload -- structurally unrelated axes -- all land
between 1.73x and 1.97x. Axis-specific defects do not produce that; a per-clause
or per-evaluation tax does. A flat profile with no lever is what one looks like
from the inside, and `rexxcps`'s profile is flat: its largest self-time symbol is
the driver loop itself at 9.3%.

This is a hypothesis with an obvious test, not a conclusion. See Path B.

## Already ruled out, so it is not re-proposed

* **Replacing the execution core.** The oracle is a tree/object walker, not a
  bytecode VM -- same architectural family as ours, ~5 ns per clause. The claim
  that it must be doing something categorically different was asserted once and
  was false.
* **`spike/bytecode-vm`'s speedups are withdrawn** -- measured on a design that
  cannot run a `CALL`, `INTERPRET` or `DROP`.
* **`smallvec` for `Number::digits`** -- `Vec` won every cell across four inline
  sizes. The answer is not allocating, not allocating more cheaply.
* **Stripping per-clause bookkeeping** -- ablated, bounded at 19% on
  `varlookup` and about 5% elsewhere, and it cannot run `rexxcps` at all.
* **Pointer-based GC addressing** -- ablated at low single digits. Object shape
  was the large half and `Bytes` inlining already took it.
* **Columnar heap layout** -- see `heap-representation-spike-gate.md`.

## Path A -- finish the IR, remove the generic fallback  [RE-SCOPED, see below]

**Superseded**: re-measured at `873488fc2` the body has **9** `Op::Generic`
ops, not 56, and `PARSE` is promoted -- see the re-measurement section. What
survives of this path is the value-echo ops, not the fallback. The original
framing is kept below because the note it came from is still quoted elsewhere.

Bets that the tax is per-instruction interpretive overhead. `rexxcps`'s main
body was recorded in a 2026-09-02 note as 142 instructions, 86 promoted and
**56 `Op::Generic`**, each re-entering
the tree-walker's clause unit; the delegating set is dominated by
`THEN`/`ELSE`/`END` structure and `PARSE`, and `exec_parse` measured 4.54%.

**The premise is unproven.** The IR mechanics spike measured no speedup and the
bytecode spike's figures are withdrawn -- two spikes, null or retracted.

*Cheap test:* promote `PARSE` alone and measure. A 4.5% construct that does not
yield refutes the path for a day's work rather than a phase's.

## Path B -- close the two unattacked oracle advantages  [CLOSED, see below]

`oorexx-oracle-is-a-tree-walker` names four differences read from source. Two
are still unattacked, and both are uniform per-clause taxes:

* **The expression stack is a preallocated array with a bump pointer.** `push`
  is `*(++top) = value`, `clear` resets `top`, and the collector scans the live
  region in place. This crate pushes and pops a `RootSet` frame per clause *and*
  per `eval` site.
* **`this` is the activation.** Every field is a direct offset. This crate goes
  through `self.activations.last().expect(...)` -- load, bounds check, branch --
  six-plus times per clause.

**Neither was in the bookkeeping ablation**, which stripped `clock_stale`,
`trace_entry` and `pending_trap`. So that ablation's 19%/5% bound does not
apply to these.

*Cheap test, and the one to run first:* an unsound build that skips the
per-`eval` root frame and caches the activation pointer, read for its ceiling.
It either explains the uniform cluster or eliminates the best theory for it,
and it is the only experiment here that tests the hypothesis directly.

## Path C -- stop allocating in arithmetic and string building

`FAST_BUFFER = 48`, and at `DIGITS 9` and `20` the oracle **allocates nothing
per arithmetic operation**. `rexx-num` allocates a `Vec<u8>` per intermediate,
several per operation, and half the `rexx-num` bucket was measured as friction
rather than computation.

Deferred by decision 2026-08-08 with "do not re-derive it". The fix is a scratch
buffer threaded through the operation, which is an invasive signature change.

## Path D -- startup, via the image

The class library is built eagerly at every process start. The oracle ships
`rexx.img`, a flattened pre-built heap.

**This is the one path where this architecture is cheaper than the oracle's.**
Under D1(a) heap objects are plain data in a `Vec` and references are indices,
so an image is a serialization of a flat array **with no pointer fixups at
all**, where the C++ needs 105 `flatten` implementations and vtable repatching
(D2). Closes startup and nothing else -- but startup is what a script pays.

## Path E -- narrow what parity means

Four axes are ahead and `arith` is 1.17x. Declare parity per axis, licence what
is structurally out of reach, and stop. A decision rather than an engineering
effort, named here because the alternatives cost weeks and this costs an
afternoon of writing down what the word means.

## The measurement that reframes all of it (2026-09-10)

**The gap is retired instructions, and there is no microarchitectural headroom
left to find.** Better IPC is not a consolation and not a win: it is a
constraint. It says the entire 2x must come out of instruction count, because
the per-cycle half is already spent.
`perf stat`, single event per run so nothing is multiplexed:

| axis | crate IPC | oracle IPC | instructions | cycles | our IPC advantage |
| --- | ---: | ---: | ---: | ---: | ---: |
| `alloc` | 4.573 | 3.918 | 2.07x | 1.77x | **+16.7%** |
| `dispatch` | 3.911 | 3.421 | 2.15x | 1.88x | **+14.3%** |
| `dispatchclass` | 3.742 | 3.362 | 2.25x | 2.02x | **+11.3%** |
| `rexxcps` | 3.383 | 3.057 | 2.17x | 1.96x | **+10.6%** |
| `strings` | 4.718 | 4.402 | 2.04x | 1.90x | +7.2% |
| `arith` | 3.151 | 3.219 | 1.16x | 1.18x | -2.1% |
| `varlookup` | 6.498 | 4.785 | 1.01x | 0.75x | +35.8% |

On every axis the instruction ratio accounts for the whole gap, and on six of
seven this crate already retires more instructions per cycle than the oracle
does -- so there is nothing to recover there. Running at 2.1x the instructions
is the whole problem, and a better IPC does not help us do it.
**`varlookup` is the clinching case**: instruction count is at parity, and we
finish in 0.75x the cycles purely because IPC is 36% better.

**Branch prediction is not the problem and cannot become the fix.** On
`rexxcps` the crate takes 4,926,508 branch misses against the oracle's
5,729,709 -- **fewer in absolute terms**, on 1.8x the branches, so half the
miss rate. The driver's `Op` dispatch compiles to one shared indirect jump
(`movzx eax, byte [r12]` / table lookup / `jmp rax`, 16-byte stride), which is
the classic interpreter mispredict trap in the folklore; measured, the
predictor handles it.

**So a whole family of candidate work is ruled out at once**: enum variant
reordering, jump-table layout, dispatch replication or computed goto,
prefetching, cache-line packing. Every one of them buys IPC, and IPC is
the axis with no headroom left. This is not an argument that those techniques
are weak -- on a stalled interpreter they are worth multiples, and enum
ordering can be worth 2x there. It is a measurement that this interpreter is
not stalled, so they would buy nothing here.

**Parity means executing about 2.1x fewer instructions on the slow axes.**
That is the whole statement of the problem, and it re-reads the paths above:

* **Path A** removes instructions -- still valid.
* **Path B is closed by ablation, below.**
* **Path C** removes instructions, and is now the best-motivated of the set:
  the oracle allocates *nothing* per arithmetic operation below
  `FAST_BUFFER = 48`, and every `Vec<u8>` we build instead is pure instruction
  count.
* **Path D** removes instructions from startup.
* **Path E** is unaffected.

## Path B, measured and closed (2026-09-10)

Two of its premises were already stale. `Interp::running` is
`Option<Box<Activation>>`, so reaching the activation is a null check and a
pointer load, not `activations.last()` with a bounds check; and
`RootSet::push_frame`/`pop_frame` are already a length read and a `truncate`,
not an allocation. What remained was `push_temp`, a `Vec::push` with a capacity
check, against the oracle's `*(++top) = value` into a preallocated array.

**Ablated: `push_temp` made a no-op, removing the entire temp-rooting tax.**
Output stayed byte-identical on all nine bench programs, so the numbers are
comparable. Retired instructions, ablated over real:

    alloc 0.9863   rexxcps 0.9885   strings 0.9910
    compound 0.9924   dispatch 0.9972   varlookup 1.0022

**A ceiling of 1.4%, and that is with the rooting removed entirely rather than
made cheaper.** A sound implementation would recover some fraction of 1.4% and
cost a redesign of the root discipline. Closed, on an hour's measurement rather
than a phase's work -- which is the rule this document ends with, applied to
the first path it was applied to.

## Sample on the event you are bound by (2026-09-10)

Because the gap is instruction count, a **cycle-sampled** profile ranks the
wrong things. The same `rexxcps` run, cycles against `instructions:u`:

| symbol | by cycles | by instructions |
| --- | ---: | ---: |
| `run_ops_from::<true>` | 9.3% | **16.0%** |
| `exec_parse` | ~4.5% | **6.5%** |
| `__memmove_avx512_unaligned_erms` | not in top | **6.1%** |
| string conversion (`required_string_answer` + `_dispatch` + `classify_`) | 6.1% | **7.5%** |

Cheap instructions that retire at high IPC -- stack traffic above all -- are
nearly invisible to a cycle profile and are exactly what an instruction-bound
program needs to remove. **Use `perf record -e instructions:u` for this work**,
and `perf annotate` for the per-line view; samply's recorder is cycles.

### What the driver's assembly shows

`run_ops_from::<true>` is 2908 instructions and 15,278 bytes. Of those
instructions **901 touch `[rsp + …]` and 208 are stores** -- 31% stack traffic,
which is register pressure in a function this size.

### Skid: the first reading of this was wrong

A non-precise profile put **6.41% on a single 16-byte spill**,
`movaps %xmm0,0x200(%rsp)`, and that figure was an artifact. With a deep
pipeline the sample lands a few instructions past the one that stalls, so a
cheap instruction downstream collects them. The same instruction under a
precise event is **0.07%**:

| sampling | that `movaps` |
| --- | ---: |
| `instructions:u`, not precise | 6.41% |
| `cycles:pp`, precise | **0.07%** |

**Use a precise event before attributing anything to a single instruction.**
`instructions:pp` is unavailable on this AMD part and `cycles:pp` is the one
that works -- worth writing down, since concluding from one refusal that
precise sampling is unavailable is how a measurement gets retired for the
wrong reason.

### What precise sampling shows

Flat, no line above 2.84%, dominated by the per-op preamble -- the dispatch
plus restoring state from the stack after each handler returns:

```
2.84%  cmp    0x1b0(%rsp),%rax     loop bound, from stack
2.24%  mov    0x28(%rsp),%rbp      reload
2.15%  mov    0x8(%rsp),%r14       reload `self`
2.08%  jmp    117912               back to the top
2.04%  lea    -0xe595b(%rip),%rcx  jump table base
1.64%  add    %rcx,%rax
1.14%  movslq (%rcx,%rax,4),%rax
1.02%  jmp    *%rax
1.02%  testb  $0x1,0x1346(%r14)    the `intermediates` flag
```

Together the preamble is **about 17% of the function**. The reloads of `self`
and its neighbours after every handler call are the register-pressure symptom
the 31% stack traffic already suggested. The handlers are already out of line
-- 186 `call` sites, and every hot source line is a `match self.<handler>(…)`
-- so those 2908 instructions are the driver's own dispatch and marshalling,
not inlined handler bodies.

**Candidates this opens, all instruction removal and all in the 0.3-1% band
that compounds:** the per-op preamble's reloads; the `intermediates` flag test
at `+0x1346`, which candidate A below shows is the visible edge of 238
value-echo ops; and `memmove` at 6.1%, whose callers are `append_tails` 22%,
`concat_values` 21%, `to_text` 15%, `exec_parse` 11%, `push_activation` 5.8%
and `text_built` 5.8%. Most of that is string building the program asked for.
The one defect in it is `push_activation`: `Activation` is 456 bytes and is
taken by value, so `*spare = activation` copies all of it per call even on the
pooled path.

## Candidate A, re-measured: the Generic fallback is nearly gone, the value
## echoes are not (2026-09-10)

**The 56-`Op::Generic` figure is stale.** Dumping `rexxcps`'s compiled stream
with `rexx-ir` at this commit: **9 `Generic` ops**, and `PARSE` is promoted --
it has its own `Op::Parse` inside an `Op::Clause` region, so `exec_parse`'s
6.5% of retired instructions is the promoted handler doing real work, not a
fallback. Path A as written aims at something largely already done.

**What the same dump shows instead: 238 `Trace*` ops in a stream compiled
under trace setting `n`.** About one op in three exists only to be skipped:

    23: Const dst=0 konst=1
    24: TraceLiteral src=0            <- nothing, untraced
    25: Load read=Simple at=0 dst=1
    26: TraceRead read=Simple src=1   <- nothing
    27: Binary op=blank lhs=0 rhs=1 dst=0
    28: TraceOperator op=blank src=0  <- nothing

Each pays the jump-table dispatch, the `intermediates` test and the loop-back.
That test is the `testb $0x1,0x1346(%r14)` the assembly section names:
`trace_cache` is at `0x1344`, so `0x1346` is byte 2 of `TraceMode`.

**The elision already exists and is already measured** --
`compile.rs`'s `let echoes_values = trace.intermediates() || !plan.never_retraces();`,
whose comment records `rexxcps` -1.78% and streams "between 29% and 38%
value-echo ops". 238 of 763 is 31%, so the comment is still accurate.

**It is defeated by the second disjunct.** A body containing a `TRACE`
instruction has `never_retraces() == false` and emits every value-echo op
defensively, forever, whether or not tracing is ever switched on. `rexxcps`
has three (`trace value tracevar`, `trace value trace()`, `trace off`) and
enables none of them.

Measured by forcing `echoes_values = trace.intermediates()`, output identical:

| program | ratio | has a `TRACE` instruction |
| --- | ---: | --- |
| `rexxcps` | **0.9844** | yes, 3 |
| `alloc` | 1.0000 | no |
| `dispatch` | 1.0000 | no |
| `varlookup` | 1.0000 | no |

**The three controls are exactly 1.0000**, which is what says the cost is the
defensive emission and nothing else: the change cannot reach a body with no
`TRACE` in it. `rexxcps` is the only bench program that has one -- and it is
the headline parity axis.

**Why the existing staleness trick does not extend.** A stale *clause* echo is
handled at `drive.rs:805` by passing `Echo::Gated` and moving the decision back
to a runtime gate, in both directions. That works because the op exists either
way. A *value* echo has no op to gate when the chunk was compiled without one,
so the same trick cannot produce the line.

**Three ways to make the 1.56% sound**, none yet built:

1. Let the staleness check see `INTERMEDIATES` -- `ChunkTrace` already carries
   the bit (`trace.rs:298`, `:316`) and `Interp::chunk_for` already keys on it;
   what is deliberately masked off today is exactly this bit
   (`trace.rs:319`). Resuming needs an instruction boundary, which `op_of`
   provides.
2. Abandon the chunk and finish the activation on the tree-walker when
   intermediates actually turns on. The engines are equivalent by
   construction, which `ir_dual` asserts, so the rare path degrades rather
   than diverging.
3. Narrow `never_retraces` -- it is per body, and a `TRACE` anywhere in a body
   taints all of it.

**A note in `ir.rs` is false and should go with whichever lands.** `Op::TraceLiteral`'s
doc says these ops are emitted unconditionally because the gate is
`trace_mode().intermediates`, "which `ChunkTrace` does not carry".
`ChunkTrace` does carry it: `trace.rs:268` defines the bit, `:298` sets it
from the mode and `:316` reads it.

## The rule the paths should be run under

`0.9^n` from 1.9x is about six successive 10% wins. Two spikes in this project
have now built first and measured after, and both were retracted. **Every path
above should be preceded by an ablation that bounds it**, the way the
bookkeeping ablation bounded its own path and closed it honestly. An ablation
that comes back small is a path closed for a day's work; one that comes back
large is a mandate.

## The `SELF`/`SUPER` plan slots, closed 2026-09-10

`Interp::enter_method_body` binds both names on **every** method send through
`Interp::slot_of`, whose third source grows the frame and inserts a boxed key
into `Activation::extra` -- for two names a method body usually never mentions.
`Plan::build` already registers `RESULT`, `RC` and `SIGL` against exactly this
cost and these two were missed.

Registering them in every plan was measured, and reverted: the tree's own plan
tests hold `build`'s key set exactly, and a `::ROUTINE` or main body would have
carried two slots nothing there ever writes. The design that survives is
`Plan::build` taking a `BodyKind`, derived from the cache key by
`Interp::body_kind` rather than passed in beside it -- a plan is stored under
that key and handed to whatever asks for it next, so a caller-supplied kind
could disagree with the cache.

Measured by retired instructions, ablated arm against the change, three runs
each, interleaved, both binaries hashed:

    dispatch       25,321,997,864 -> 21,697,000,608   -14.32%
    dispatchclass  19,637,792,886 -> 16,737,793,895   -14.77%
    alloc, alloc4c, varlookup, compound, strings, arith,
    emptyloop, rexxcps                                 1.0000

**Eight controls at exactly 1.0000**, `rexxcps` among them. The change can
alter a `::METHOD` or `::ATTRIBUTE` body's plan and nothing else, which is what
makes eight flat axes a control rather than a coincidence -- and what says the
win is the two names rather than the two slots that came with them.

**Two attempts at the same cluster were measured and discarded first**, and
neither is in the tree: recycling the `extra` map's capacity across pooled
activations cost +0.18% on both send axes, and boxing `extra` cost +1.95%.

## Re-measured 2026-09-10 at `86ca524e6`: the cluster was not one shape

Three instruments, run back to back on an otherwise idle machine, single perf
event per run, three reps, median, interleaved, oracle under
`ulimit -v 1048576` from an empty directory.

### The tree-walker is slower than the IR on every axis

Same binary, `REXX_ENGINE=tree-walker` against the default, retired
instructions:

| axis | tree-walker | IR | IR/TW |
| --- | ---: | ---: | ---: |
| `varlookup` | 31,908,577,449 | 17,259,617,918 | **0.5409** |
| `strings` | 32,895,825,492 | 22,728,924,169 | 0.6909 |
| `alloc4c` | 5,735,758,346 | 4,257,811,188 | 0.7423 |
| `compound` | 14,435,175,816 | 11,175,019,955 | 0.7742 |
| `rexxcps` | 27,317,694,742 | 23,074,852,970 | 0.8447 |
| `emptyloop` | 11,096,562,336 | 9,446,603,134 | 0.8513 |
| `arith` | 13,751,991,059 | 12,855,008,303 | 0.9348 |
| `dispatch` | 22,946,939,335 | 21,696,993,863 | 0.9455 |
| `alloc` | 28,932,633,488 | 27,396,681,793 | 0.9469 |
| `dispatchclass` | 16,889,740,832 | 16,737,789,136 | 0.9910 |

**No axis is above 1.0000.** Retiring the IR in favour of the tree-walker
would cost between 0.9% and 85% depending on the axis. The representation is
not what stands between this crate and the oracle, and the question "is the IR
the wrong approach" is answered: it is not.

What the table also says is where the IR has already given what it has. On
`dispatchclass`, `alloc` and `dispatch` it is worth 0.9%-5.5%, because on those
axes the time is not in the clause loop at all.

### Every axis is instruction-bound. None is stall-bound.

Crate against oracle, instructions and cycles measured separately:

| axis | crate IPC | oracle IPC | instr. ratio | cycle ratio |
| --- | ---: | ---: | ---: | ---: |
| `startup` | 2.66 | 1.20 | **13.34** | 6.03 |
| `rexxcps` | 3.42 | 3.10 | 2.17 | 1.97 |
| `alloc` | 4.60 | 3.97 | 2.06 | 1.78 |
| `strings` | 5.00 | 4.40 | 2.04 | 1.79 |
| `dispatchclass` | 4.67 | 3.34 | 1.72 | 1.23 |
| `dispatch` | 4.64 | 3.45 | 1.65 | 1.23 |
| `arith` | 3.31 | 3.26 | 1.16 | 1.14 |
| `compound` | 5.27 | 3.09 | 1.08 | 0.63 |
| `varlookup` | 6.49 | 4.85 | 1.01 | 0.76 |
| `emptyloop` | 6.01 | 4.80 | 0.75 | 0.60 |

**Our IPC is higher than the oracle's on all ten axes**, by 1.5% (`arith`) to
122% (`startup`). Cache, branch prediction, code layout and enum ordering are
therefore not where the gap lives -- on every axis we retire more instructions
and retire them faster. Parity is a matter of emitting fewer instructions and
nothing else.

The cycle ratio tracks the wall-clock ratio to within a few percent on every
axis except `startup`, which is the cross-check that says these two instruments
agree.

### The 1.7x cluster has split into three

The uniformity noted at `8565dcfa8` -- six axes between 1.73x and 1.97x --
does not survive the measurement.

1. **`startup` never belonged.** Its 1.73x wall clock is 26 ms against 15 ms,
   and most of both is process spawn that `:u` counters do not see. In user
   space it is **13.3x the instructions** and 6.0x the cycles. Grouping it with
   the others was pattern-matching on a ratio.
2. **`dispatch` and `dispatchclass` have left**, from targeted work rather than
   from anything the flat-tax hypothesis predicted: 1.78x/1.81x wall clock at
   `8565dcfa8`, **1.20x/1.22x now**. Their shape is also distinct -- 1.65x-1.72x
   instructions but only 1.23x cycles, on the largest IPC advantage of any axis.
3. **`alloc`, `strings` and `rexxcps` are the real class**, and they are
   genuinely uniform: 2.04x-2.17x instructions, 1.78x-1.97x cycles, IPC ahead
   by 13%-16%. One tax, ~2x the instructions for the same work.

That third group is what "parity" now means, and `startup` is a separate and
much larger multiple that no path above was drawn to address.

## The `rexxcps` parity program, opened 2026-09-10

Moritz set the goal: `rexxcps` to 1.00x. The budget, from the fixed 20,000,000
clauses both sides run: **1153 instructions per clause against the oracle's
531**. Two landed this day, both from one ablation:

* `46480a4ee` / `46845ce88` -- the required-string protocol answered where the
  answer is the value. `rexxcps` 23.07G -> 20.79G (**2.17x -> 1.96x**
  instructions), `strings` 1.81x -> 1.50x wall clock.

### What the profile says after them, and it is the important part

Sampled on `instructions:u` at `46845ce88`, with DWARF inline frames expanded
so self-time lands on the deepest inlined callee rather than its host:

| leaf | self |
| --- | ---: |
| `run_ops_from::<true>` | 11.5% |
| `memcpy` | 4.1% |
| `ObjRef::decode` | 3.2% |
| `slice::Iter<Op>::next` + `NonNull<Op>::eq` | 2.9% |
| `Digits::as_slice` | 2.8% |
| `int_free_chunk` | 1.8% |
| `RootSet::frame_slot` + `set_temp` | 1.8% |

**Nothing above 4.1% outside the driver, and every item opened so far is real
work rather than waste.** Each of these was chased to its source and none is a
defect: `Digits::as_slice` is a two-arm match called constantly by decimal
arithmetic; the op-iteration pair is `Chunk::ops_in`, which exists precisely to
hoist the per-op bounds check; `memcpy` is 48 distinct stacks whose top three
are 64, 64 and 61 samples of 557; `concat_values` already has an inline-bytes
path, a lent buffer and one conditional allocation.

### Candidates ruled out this day, each on evidence

* **The op-dispatch loop.** `emptyloop` is 0.75x on instructions -- bare clause
  dispatch already beats the oracle by a quarter. Whatever the gap is, it is
  above dispatch.
* **Compound tail allocation.** `join_tails`' `Vec::new` is not on the hot
  path; the three live compound sites already lend through
  `take_key_buffer`/`give_key_buffer`.
* **PARSE buffers.** `exec_parse` pools through `give_parse_buffer` and
  `give_parse_strings`.
* **An integer arithmetic fast path.** `arith_small_int` exists and is tried
  first, and `spelled_int_arith` already admits a text operand. `rexxcps`'s hot
  arithmetic is genuinely decimal (`do j=1.1 to 2.2 by 1.1`,
  `avar.1.2 * 1.1`), which the oracle also does in decimal.
* **Allocation volume.** `heaptrack` on the shortened program: 344,336
  allocation calls against 800,000 clauses -- **0.43 per clause**, about one
  per 2,400 instructions, which is the ~4% the malloc/free/calloc symbols
  measure. Allocation count is not the lever; the copies are the larger half
  of that theme.
* **A missed-inline family.** Briefly believed, then disproved: those
  accessors carry `#[inline(always)]` and are absent from the non-expanded
  profile, so they are inline frames being attributed accurately.

### What that means for the goal

`0.9^n` from 1.96x needs n≈7 more wins of 10%, and the profile no longer
offers one. Parity is a program of many 1-3% steps against a noise floor of
about 1% per axis -- the `emptyloop` control moved 1.0079 in an arm whose
ablated code cannot run at all. The next honest step is not another symbol from
this table but a structural question: what the oracle does per clause that we
do twice, at 531 instructions against 1153.

`startup` remains untouched and is the largest multiple in the tree at 13.3x
instructions, hidden behind a 1.65x wall clock that is mostly process spawn.

## The driver's code density, measured 2026-09-10 at `20b078177`

Prompted by Moritz: check the driver's asm structure, and see whether the
individual instruction bodies are short or should be outlined.

### The counter nobody had looked at

Same fixed `rexxcps` work, both interpreters:

| counter | crate | oracle | ratio |
| --- | ---: | ---: | ---: |
| instructions | 20.80G | 10.63G | 1.96x |
| cycles | 6.31G | 3.46G | 1.82x |
| **L1i misses** | **202.1M** | **58.2M** | **3.47x** |
| branch misses | 4.60M | 5.43M | 0.85x |

**9.71 icache misses per 1000 instructions against the oracle's 5.48.** It is
the most disproportionate counter measured on this axis -- worse than the
instruction ratio it should track -- and branch misses are *lower* than the
oracle's, so the frontend problem is fetch and not prediction.

### The bodies are already outlined; the bulk is cleanup

Sampled on `instructions:u`, the hot source lines inside `run_ops_from` are
almost all call sites -- `apply_binary`, `exec_parse`, `run_call_named`,
`run_call_args`, `run_loop_with_header`, `arith_general`, `read_symbol`. The
dispatch itself (`for region_op in ops` / `match region_op`) is about half the
function's own self samples. So the op arms are not fat inline bodies.

**What the function is instead: 87% cold.** 2529 of 2908 instructions carry
no sample in a full run. The largest cold run, 518 instructions, sits at the
end (LLVM sank it) and opens with `movups` stack shuffling and `-0x2`
drop-flag stores -- unwind landing pads and drop glue. Across all cold code the
mnemonics are 1075 `mov`, 263 `lea`, 96 `movups`: cleanup, not error
construction. Marking every `Loud` constructor and both `Failure` conversions
`#[cold] #[inline(never)]` shrank the function by **121 bytes of 15,278**, so
the error constructors were already outlined calls and are not the bloat.

The remaining ~2000 cold instructions are interleaved with the hot ones through
the first 85% of the function: the 50-55% band holds 4 hot instructions
carrying 1527 samples among 141 cold ones. Effective hot footprint is ~379
instructions spread over ~13KB of address range.

### `const GRANTING` -- measured both ways, and it earns its keep

`run_ops_from` is monomorphised twice on it, 15,278 + 15,127 bytes, and the
const is read exactly once: `let mut granting = GRANTING;`. Collapsing it to a
parameter gives one function of 17,767 bytes, **12,638 bytes less driver**.
Measured, five reps, interleaved, both binaries hashed:

    axis            ins       cyc       l1i
    dispatch     1.0069    0.8721    0.2305
    rexxcps      1.0064    0.9848    0.7762
    alloc        0.9984    0.9802    3.4961
    compound     1.0108    0.9918    0.9678
    varlookup    1.0187    1.0023    1.0557
    strings      1.0094    1.0290    1.2916
    emptyloop    1.0397    1.0348    1.0521

**Not landed: it is a trade, and both sides are mechanisms rather than
layout.** In the `<false>` monomorphisation `granting` starts false and is
never set true, so LLVM proves the two `if granting` branches dead and deletes
them; a parameter restores about four instructions per clause, which is
`emptyloop`'s +3.97% **instructions**. The other side is equally real:
`dispatch` takes -12.8% cycles on +0.69% instructions, from 12.6KB less code.
The doc comment on `run_ops` claims the const "turns a per-clause branch into
no code at all" and it is correct; a reading of it as vacuous was wrong.

The design that takes both halves is not this one. It has to shrink the cold
bulk so that *both* monomorphisations are small -- which means fewer droppable
locals live across `?` in the driver, not a different signature.

### Two instrument findings, both of which invalidate earlier method

* **Retired instructions cannot measure density work.** Code that never
  executes retires nothing, so the primary instrument used all day reports
  `1.0064` for a change worth -12.8% cycles on `dispatch`. Anything about
  layout, duplication or icache has to be measured on `cycles:u`.
* **An L1i *ratio* is not evidence where the absolute count is small.** The
  same change reads 0.2305 on `dispatch` and 3.4961 on `alloc` while cycles
  move -12.8% and -2.0%. The ratio was called a clean control here and it is
  not one; absolutes are required before reading it.

### The layout control for the driver, measured 2026-09-10

The section above rejected the `GRANTING` collapse on a cycle measurement, and
the note above `run_ops` rejected an earlier change for having "no mechanism
below the layout changed". Neither had a control. This is it.

Four binaries differing **only** in the size of a `#[cold] #[inline(never)]`
function in the driver's own translation unit, never called (env-gated and
`black_box`ed so it survives DCE and never runs). Identical semantics,
byte-identical 15,278-byte driver in all four, four distinct hashes. Retired
instructions across the extremes: 21,701,992,708 against 21,701,987,585, a
0.00002% difference -- which is what says the pad never executes and the hot
code is the same code.

Pad sizes 22, 8,583, 19,543 and 39,274 bytes, bracketing the 12,638-byte delta
the real change produced. `cycles:u`, five reps, median:

| axis | pad1 | pad400 | pad900 | pad1800 | spread |
| --- | ---: | ---: | ---: | ---: | ---: |
| `dispatch` | 1.0000 | 0.9677 | 1.0231 | 0.9642 | **5.9 pp** |
| `strings` | 1.0000 | 1.0402 | 0.9999 | 1.0010 | 4.0 pp |
| `emptyloop` | 1.0000 | 1.0290 | 1.0048 | 1.0351 | 3.5 pp |
| `rexxcps` | 1.0000 | 0.9943 | 0.9749 | 0.9843 | 2.5 pp |
| `alloc` | 1.0000 | 1.0108 | 0.9977 | 1.0060 | 1.3 pp |

**Dead code alone moves a cycle measurement by up to 4% on one axis, and
`dispatch` spans 5.9 points end to end.** The movement is not monotonic in pad
size -- `dispatch` is 0.9677 at 8.5KB, 1.0231 at 19.5KB and 0.9642 at 39KB --
which is what distinguishes a placement artifact from an effect.

**A single-axis cycle difference under about 4% is not evidence.** That
threshold governs the rest of this document's cycle work, and it retires two
claims made in the section above:

| axis | `GRANTING` change | band | verdict |
| --- | ---: | --- | --- |
| `dispatch` | 0.8721 | 0.9642-1.0231 | **outside, better by 9.2 pp** |
| `alloc` | 0.9802 | 0.9977-1.0108 | outside, better by 1.8 pp |
| `rexxcps` | 0.9848 | 0.9749-1.0000 | inside -- **not established** |
| `strings` | 1.0290 | 0.9999-1.0402 | inside -- **not established** |
| `emptyloop` | 1.0348 | 1.0000-1.0351 | inside on cycles |

`rexxcps -1.5%` was reported as a win and `strings +2.9%` as a regression.
Neither is distinguishable from layout and both are withdrawn.

**What survives is the important part.** `dispatch` at -12.8% clears the whole
band by 9.2 points, and its cost is `emptyloop`'s **+3.97% instructions**,
which is real for the opposite reason: instructions are layout-insensitive, as
this control's own 0.00002% shows. So the trade is a 12.8% cycle cut on the
send axis against roughly four instructions per clause on the tightest loop --
both real, neither noise. That is a genuine engineering choice rather than the
measurement artifact the earlier note assumed, and it is evidence that driver
code size matters for send-heavy work, which is what the "shrink the cold bulk"
direction rests on.

## The per-primitive parity table, one instrument, 2026-09-10 at `bbe86427d`

Retired instructions throughout, three reps, median, oracle under
`ulimit -v 1048576` from an empty directory. **Every earlier table in this
document that mixes wall-clock ratios with instruction ratios should be read
against this one.**

| axis | crate | oracle | ratio |
| --- | ---: | ---: | ---: |
| `emptyloop` | 9,446,594,802 | 12,650,857,189 | **0.75x** |
| `compound` | 10,194,963,616 | 10,342,231,098 | **0.99x** |
| `varlookup` | 17,259,616,380 | 17,015,851,635 | 1.01x |
| `arith` | 12,356,911,967 | 11,076,071,132 | 1.12x |
| `textnum` | 1,384,896,185 | 1,234,553,685 | 1.12x |
| `parse` | 1,871,040,268 | 1,261,877,196 | 1.48x |
| `dispatch` | 21,702,000,878 | 13,111,514,876 | 1.66x |
| `dispatchclass` | 16,737,784,005 | 9,750,642,549 | 1.72x |
| `strings` | 20,166,927,819 | 11,162,817,324 | 1.81x |
| **`rexxcps`** | **20,799,776,442** | **10,625,947,299** | **1.96x** |
| `alloc` | 27,396,714,532 | 13,280,207,704 | 2.06x |

### What it says, and what it corrects

**`rexxcps` is exceeded only by `alloc`.** It also performs **no method sends
at all**, so `dispatch` and `dispatchclass` -- where two of the three wins
landed on 2026-09-10 -- cannot move it. The levers for the goal are
**allocation (2.06x), string materialisation (1.81x) and `PARSE` (1.48x)**, in
that order.

**A claim made and withdrawn the same hour.** "The mix is worse than every one
of its parts" was built by comparing the *wall-clock* ratios (`compound` 0.58x,
`emptyloop` 0.60x, `varlookup` 0.73x) against `rexxcps`'s *instruction* ratio.
In consistent units `compound` is 0.99x and `varlookup` 1.01x, and the mix sits
inside the range of its parts. There was no anomaly; there was a unit error,
and an architectural argument was built on it before it was checked.

### The interpreter layer is not the tax base

Four ablations on 2026-09-10 bounded it, and `hot-core-size.md` carries them:
removing the error channel outright is worth 0.9% on this axis, and removing
**32.8% of all executed ops** is worth 1.73%. An op costs about 8.6
instructions and there are ~146M of them against 20.8G. Dispatch, op count,
fusion and trace emission are collectively worth single digits.

We are also better than the oracle on every cache and predictor measured --
**87x fewer L1d misses on `varlookup`, 1.6x fewer on `rexxcps`**, fewer branch
misses, higher IPC on all axes -- and worse only on L1i. The gap is retired
instructions in the value operations, and nowhere else.

## Why the mix exceeds its parts: decimal loop control (2026-09-10)

`rexxcps` is 1.96x while every primitive it exercises is at or below 1.48x.
`alloc` (2.06x) and the two dispatch axes are message-send workloads and
`rexxcps` performs no sends, so they are not its components -- and **`alloc4c`,
the no-sends allocation axis, is 0.82x: this crate is faster than the oracle at
allocation.** Allocation is not a lever for the goal.

A profile cannot resolve "the mix exceeds its parts", because it reports shares
*within* the mix. Bisecting the workload can. `rexxcps`' inner body, one
construct removed at a time, every variant agreeing with the oracle:

| variant | ratio |
| --- | ---: |
| full | 2.014x |
| **without the `j`-loop** | **1.832x** |
| without `SELECT` | 2.018x |
| without `PARSE` | 2.058x |
| without the compound work | 2.064x |
| without the `CALL` | 2.140x |

Only the `j`-loop's removal improves the ratio; every other construct is one
this crate handles *better* than the average of the rest. The `j`-loop's own
contribution is **2.31x**.

Isolated, and now in `bench-programs/`:

    intloop     1.22x   integer-controlled DO
    decloop     1.63x   decimal-controlled DO
    decrender   1.99x   ... plus rendering the control value to text

### Two hypotheses raised and killed

**The `text_numbers` cache is not thrashing.** 85.3% hit with 16 cold misses on
`rexxcps`; **100.0% hit with 3 misses in 2,000,000** on the decimal loops. Its
11.32% of `decrender` is therefore *probe* cost -- hash, compare, and a
`.clone()` of a 40-byte `Result<Number, NotNumeric>` -- and not parsing.

**Pre-parsing numeric literals is not a lever for this crate.** The conversion
counts said it should be: `decloop` performs exactly 3 conversions per outer
iteration and its loop header holds exactly three literals. But binding the
bounds to already-`Number` values first makes the ratio **worse, 1.63x ->
1.92x**: this crate's instructions rise 3.2% while the oracle's fall 12.5%.
The oracle pays for re-parsing literals and we largely do not, because the
cache answers them.

### Open, and the next thing to chase

`decloop` prints nothing, yet `rexx_num::format::resolve_exponential_state` and
`Number::format_with` together are 5.3% of `decrender`, with malloc/free at
9.6%. `Interp::number` stores `text: None`, so the rendering is not eager at
construction -- something else on the decimal loop path renders a number to
text. **That is localised but not diagnosed, and it should not be guessed at:
the mechanism needs a caller-attributed profile of `decloop` specifically.**

### The decimal loop's mechanism, and two more closed leads (2026-09-10)

**`decloop` renders nothing.** The 5.3% of number *formatting* recorded above
is `decrender`'s, where `length(j)` and `j='foobar'` genuinely require the
control value as text. A caller-attributed profile of `decloop` contains no
`format` frame at all; the earlier note conflated the two programs.

**What the 1.63x actually is: the controlled-loop fast path cannot fire.**
`Interp::loop_advance`'s `LoopState::Controlled` arm has an integer fast path
guarded on `by_int` and `to_int` -- both `Number::plain_integer` -- and on the
control value being `ControlValue::Small`. For `do j=1.1 to 2.2 by 1.1` the
`BY` is not a plain integer, so `by_int` is `None` and the guard can never
pass. Every step then takes the general path: a genuine variable read (the
oracle's own semantics, measured), a `Number` conversion, a decimal add, a
fresh arena object, and a numeric compare.

The loop state is not at fault -- `to` and `by` are already kept as `Number`s
with `to_int`/`by_int` caches, so nothing is re-parsed per step. `decloop`
performs exactly 3 conversions per *outer* iteration, which is its three
literals at loop entry.

**Closed: `Number`'s size is not a lever, and cannot be changed.** The profile
suggested it -- `Result<Number, Failure>::branch` 2.5%, `Number::clone` 1.5%,
`Digits::as_slice` 5.1%, all consequences of a 40-byte value type moved through
`Result`s where the oracle passes a pointer. Shrinking `INLINE_DIGITS` from 30
to 14 made every axis **worse** (`decloop` +2.3%, `arith` +34%), because
`Number::from_i64` takes up to nineteen digits and a 14-digit buffer spills
every integer conversion to the heap. That bound is written in `digits.rs`, and
the ablation went under it.

Checking it properly closes the line: `digits.rs` records that **every capacity
from twenty to thirty gives `Digits` 32 bytes and `Number` 40**, because the
`Vec` arm is 24 bytes. Twenty is the floor `from_i64` sets. So `Number` cannot
be smaller than 40 bytes, and its size was never available to trade.

**What is left, and it is a real design question.** A decimal-controlled `DO`
is a supported and not-rare construct that falls off a fast path built only for
integers. The fix is not to widen that guard -- `by_int` exists because
integer stepping is exactly representable -- but to give the decimal case its
own stepping state, keeping the control value as a `Number` across iterations
instead of round-tripping it through a variable read and an arena allocation
each step. The oracle's `DoBlock` keeps its control value likewise. That is a
design change, and it should be bounded by an ablation first, like everything
else here.

### Correction: it is the *spelling*, not the fractional step (2026-09-10)

The section above says the fast path cannot fire because `BY` is fractional.
That is wrong, and one experiment shows it. Two loops of identical trip count:

| program | crate | oracle | ratio |
| --- | ---: | ---: | ---: |
| `intloop` -- `do j=1 to 3` | 1,531,220,879 | 1,264,758,094 | 1.21x |
| `decspell` -- `do j=1.0 to 3.0 by 1.0` | 3,631,517,991 | 2,252,634,550 | 1.61x |
| `decfrac` -- `do j=1.1 to 3.3 by 1.1` | 3,631,517,397 | 2,189,811,584 | 1.66x |

**`decspell` and `decfrac` differ by 594 instructions out of 3.6 billion.** An
integral-but-decimal-spelled bound takes exactly the same path as a fractional
one, so the guard is not about the step being fractional.

`Number::plain_integer` opens with `u64::try_from(self.exponent).ok()?`, so it
rejects any negative exponent. `1.0` is digits `[1, 0]` with exponent `-1` and
is rejected although it is numerically an integer; `1` is digits `[1]` with
exponent `0` and is accepted. **The guard turns on how the literal was
written.**

Both interpreters pay for decimal loop control -- per iteration the oracle goes
1,054 -> 1,877 instructions (+78%) and this crate 1,276 -> 3,026 (+137%). The
penalty is real on both sides and larger here, which is the whole of the
1.21x -> 1.61x move.

**Two candidate changes, and they are not the same size.**

1. Normalise `plain_integer` over trailing zeros, so `1.0` is accepted as `1`.
   Small and safe, but it only reaches loops written with integral decimal
   literals -- not `rexxcps`, whose step really is `1.1`.
2. Give the fractional case its own stepping state, keeping the control value
   as a `Number` across iterations instead of reading the variable back,
   re-converting it and allocating a fresh arena value each step. This is the
   one that reaches the goal axis, and it is a design change: the read-back is
   load-bearing (`do ii = 1 to 3; ii = 10; end` prints `11` on the oracle, and
   reusing the loop's own value prints `4`), so any such state has to be
   invalidated by a body that assigns to the control variable.
