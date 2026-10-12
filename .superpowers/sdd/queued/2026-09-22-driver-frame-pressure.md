# The driver's frame, and the one outlining variant nobody has run

Measured 2026-09-22 by the driver-gap task, while identifying `drive.rs:0`:

* `run_ops_from::<true>` is **15,087 bytes, 2,894 instructions**, and **42.9% of
  those instructions ever execute** on `rexxcps`.
* The costs with no line attribution, **555,387,711 Ir over 246 addresses**, are
  **37.6% spill and reload against a 1,416-byte stack frame**, with the rest
  register copies, block-boundary jumps and position-less loads.
* Both dispatch tables carry source lines, `:664` for the region walk and
  `:543`/`:544` for the outer one, so **the jump table is not in this figure**.
  Dispatch is already counted elsewhere.

## The mechanism, and why it is not the one already rejected

A stack frame is allocated once at function entry for the union of every arm's
needs. **So the arms that never run on a given program still set the frame size
that the arms that do run pay for**, in prologue and epilogue and in how many
registers the allocator has left for the hot path.

This is **register pressure, not instruction-cache footprint**, and that
distinction matters because the footprint argument has been measured and
rejected here twice: their evidence did not isolate outlining, and ours showed
`dispatchclass` fits in 32 KiB and drops to zero misses at 64-way, so its misses
are conflict rather than capacity.

## What was tried, what it cost, and what it did not try

The control-flow spike moved the cold arms **into one `#[inline(never)]`
non-generic helper**: **+2.80%, +3.70% and +2.53%** retired instructions on three
axes, 64-way I1 unmoved at +0.16%, `.text` **up** 7,328 bytes, because the
extracted arms cost 9,965 standalone against roughly 1,900 inlined per
instantiation.

**That experiment bundled every cold arm into one function.** A single helper
taking the union of what all the cold arms need has a large frame of its own and
a wide argument list, and every cold arm pays for every other cold arm's state.

**The variant it did not run: leave the arms separate.** `#[cold]` plus
`#[inline(never)]` on each cold arm individually, so each keeps its own small
frame, and the hot function's frame shrinks to what the hot arms need. That is a
different experiment from the one that failed, and the failure of the bundled
form is not evidence against it.

## How to tell whether it worked, before believing any percentage

**The frame size is the direct instrument.** Read it out of the prologue's `sub
$N, %rsp` for `run_ops_from` before and after. If the frame does not shrink, the
change did not do the thing, whatever the benchmark says; and if it shrinks
while instructions rise, that is the outlining price reappearing and the answer
is no.

Retired instructions are the arbiter as always, on `emptyloop` and `varlookup`
where they reproduce to 0.0001%, with `rexxcps` always reported.

## What would make this not worth doing

If the spill traffic is dominated by state genuinely live **across** the hot
arms -- the program counter, the stop bound, the chunk, the code, the register
frame -- rather than by the union of cold arms' locals, then no amount of
outlining helps and the answer is to carry less state across the loop. **Read
which values are being spilled before choosing between the two**, since they
point at opposite changes and the profile alone does not distinguish them.
