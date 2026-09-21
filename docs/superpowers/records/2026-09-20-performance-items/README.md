# Performance items, September 2026

Not an SDD plan record. This is the working set of an ordered performance to-do
worked item by item with a brief per item, and it is here for the same reason
the plan records are: the reasoning, the measurements and the rejected
candidates exist nowhere else, and `.superpowers/sdd/` is git-ignored scratch.

| file | what it is |
|---|---|
| `2026-09-20-performance-todo.md` | the ordered item list, and the reports for items 1 and 2 written back into it as appended sections rather than as separate files |
| `2026-09-20-instructions-per-op.md` | the baseline: how ops are counted by summing execution counts at the driver's dispatch jump addresses, and the per-op instruction figures every later candidate is sized against |
| `2026-09-19-driver-control-flow-spike.md` | the control-flow spike, including the padding control that separates a layout effect from a real one |
| `trace-emission-brief.md` | item 1's dispatch brief |
| `trace-quickening-brief.md` | item 2's dispatch brief |

## What these files get wrong, recorded rather than edited

Append-only, per this tree's policy. Two things in the files above were later
measured to be wrong, and both are corrected in appended sections of
`2026-09-20-performance-todo.md` rather than in the documents that state them.

**The 3.89% ceiling in both briefs is not a ceiling.** It was measured by
deleting the `TRACE` instructions from `rexxcps`, which measures what the
feature costs rather than what a fix can recover. The most any promotion of that
shape can reach is **-1.706%**, measured with an unguarded speculation of every
clause. The difference is the `TRACE` instructions that still execute: line 76's
`trace value trace()` runs 280,000 times. The same error was made twice, once
for each item, because the second item inherited the number from the first
item's A/B.

**Item 2's entry describes a mechanism that does not work.** Keying the chunk on
an observed outcome banks nothing here, because the chunk is chosen when an
activation starts and `rexxcps`' timed body runs inside one activation. And
`drive.rs`'s staleness check cannot serve as the deoptimisation: it moves the
clause echo between the stream and a run-time gate, and an op that is not in the
stream cannot be gated back on.

## What landed, with the figures

| commit | what | measured |
|---|---|---|
| `7a28f68e6`, `7fbadfecb` | item 1: a per-clause `TRACE` setting from a forward dataflow analysis, then emission wired to it | reaches its ceiling where the setting is a top-level literal; **zero on `rexxcps`**, whose events are `trace value <expr>` and bottom to any static analysis |
| `941b5fb82` | a loop's back edge and zero-trip edge, and `LEAVE`/`ITERATE` reaching every enclosing block | **-1.78%** on the in-loop literal shape (766 ops to 536); inside the noise band on `rexxcps` |
| `3c2e6825a`, reverted by `4ef6af5bb` | item 2: speculate that an event keeps the setting it was entered with, behind a guard | emission worth **-0.543%**, guard costs **+0.501%**, and the guard is paid by programs containing no `TRACE` at all: `emptyloop` +1.25%, `varlookup` +1.08% |

`3c2e6825a`'s message contains two statements its own follow-up disproves. It
was not amended; `4ef6af5bb`'s message carries the correction, and the wrong
commit stays readable beside it.

## The general result, which outlives the items

A conditional in the driver's `Op::Clause` arm costs about 0.5% on `rexxcps` and
about 1.25% on `emptyloop` whatever it does. Item 2 found it in five shapes: a
call into the chunk, the same behind an out-of-line `#[cold]` helper, settling
the returned flow inside the branch, splicing alternative ops into the stream so
the guard is one assignment to the program counter, and that splice with the
accessor inlined as a slice read. The control that separates the branch from the
restructuring around it: the same arm with the branch deleted measures
`emptyloop` at 9,998,565,949 against 9,998,565,263.

So an optimisation that buys under about 1% by adding a per-clause test does not
pay here, and the items that remain on the list are the ones that **remove
dispatches** rather than adding a test.
