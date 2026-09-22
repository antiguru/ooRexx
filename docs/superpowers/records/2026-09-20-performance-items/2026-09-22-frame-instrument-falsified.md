# The frame size is not the instrument, and this corrects the note that said it was

`2026-09-22-driver-frame-pressure.md`, committed the same day in this directory,
ends with a stopping rule I wrote:

> **The frame size is the direct instrument.** Read it out of the prologue's
> `sub $N, %rsp` for `run_ops_from` before and after. If the frame does not
> shrink, the change did not do the thing, whatever the benchmark says; and if
> it shrinks while instructions rise, that is the outlining price reappearing
> and the answer is no.

**The second half is falsified and the first half is unsafe.** This tree is
append-only, so that note stands as written and this one says why it is wrong.

## What falsified it

The store-fusion experiment added four op arms to `run_ops_from` and measured
`rexxcps` at **+2.32%** with nothing fused. Over the same change the driver's
frame **shrank, 1,416 to 1,368 bytes**, while the function grew from 2,897 to
3,443 instructions.

So frame size and instruction count moved in **opposite** directions on a change
that clearly hurt. A frame that moves the right way is not evidence that a
change helped, which is exactly what the stopping rule claimed.

## And the mechanism says why frame size was never going to work

The same experiment located its cost. Summing every `run_ops_from::<true>` row
on `emptyloop` accounts for the whole-program delta to within twenty thousand
instructions in 9.7 billion, at **+1.00 and -4.00 instructions per iteration**
-- whole numbers, and neither a callee nor the frame.

**That is a value moving into or out of a register across the loop.** The cost
lives in the allocator's per-value decisions, not in the size of the frame those
decisions spill into. The frame is a summary statistic over choices that
individually decide the answer, and it can shrink while the choices that matter
get worse.

## What the note still gets right

Everything before the stopping rule. `drive.rs:0` is the register allocator,
37.6% spill and reload; `run_ops_from::<true>` is 15,087 bytes and 2,894
instructions with 42.9% ever executed; Candidate E rejected the **bundled**
outlining form whose helper frame is the union of every cold arm's needs, and
the per-arm `#[cold]` variant is untested rather than refuted.

The store-fusion result makes that variant **more** interesting, not less, and
for a reason the note did not have: if a four-op difference can move an axis by
342 million instructions, then how the driver's code is divided into functions
is worth more than anything inside any one of them.

## The rule that replaces it

**There is no cheap proxy for this. Measure the program.** A source change to
`run_ops_from` has no attributable instruction cost that can be predicted from
any static property of the function -- not its size, not its frame, not its
instruction count. Retired instructions are deterministic per binary and are
**not** stable across a source change that should not have affected them.
