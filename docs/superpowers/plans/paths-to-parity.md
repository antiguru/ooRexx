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

## Path A -- finish the IR, remove the generic fallback

Bets that the tax is per-instruction interpretive overhead. `rexxcps`'s main
body is 142 instructions, 86 promoted and **56 `Op::Generic`**, each re-entering
the tree-walker's clause unit; the delegating set is dominated by
`THEN`/`ELSE`/`END` structure and `PARSE`, and `exec_parse` measured 4.54%.

**The premise is unproven.** The IR mechanics spike measured no speedup and the
bytecode spike's figures are withdrawn -- two spikes, null or retracted.

*Cheap test:* promote `PARSE` alone and measure. A 4.5% construct that does not
yield refutes the path for a day's work rather than a phase's.

## Path B -- close the two unattacked oracle advantages

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

## The rule the paths should be run under

`0.9^n` from 1.9x is about six successive 10% wins. Two spikes in this project
have now built first and measured after, and both were retracted. **Every path
above should be preceded by an ablation that bounds it**, the way the
bookkeeping ablation bounded its own path and closed it honestly. An ablation
that comes back small is a path closed for a day's work; one that comes back
large is a mandate.
