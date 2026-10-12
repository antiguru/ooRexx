# Price a removed op: fuse a producer with the `Store` that consumes it

**This task exists to measure a constant, and the commit is the instrument.**
Every remaining estimate for widening the ops multiplies an op count by a price
per removed op, and that price is known only at two points: **8 instructions**
for a dispatch that does nothing, and **54.8** once, for `Condition` +
`JumpUnless`, of which 34.8 was the value handoff between the pair. Nothing in
between has been measured, and the gap between those two numbers is the
difference between a direction worth a week and one worth an afternoon.

## The transform

An op that produces a value into a register, immediately followed by an
`Op::Store` that moves that register into a variable slot, writes straight to
the slot.

That is item 5 of the performance to-do generalised past constants: it covers
`Load` + `Store`, `Const` + `Store`, `LoadConstant` + `Store` and `Arith` +
`Store`, which are the commonest shapes on the axes below.

## Why the axis is `varlookup` and not `rexxcps`

Rendered with `target/release/rexx-ir`, trace ops excluded, at the `9a2eb522f`
tree:

| program | clauses | shapes | trace ops | hot shapes |
|---|---:|---:|---:|---|
| `varlookup` | 6 | 5 | **0** | `Load Store LoopNext`, `Load LoadConstant Arith Store` |
| `arith` | 11 | 8 | **0** | `Load LoadConstant Arith Store` (x2), `Load Load Arith LoadConstant Arith Store` |
| `emptyloop` | 4 | 4 | **0** | `LoopNext`, `Const Say` |
| `rexxcps` | 129 | 57 | 207 | its `Const Store` pairs have a live `TraceLiteral` between them |

**`varlookup`'s loop body is `Load Store LoopNext` with no trace op between the
pair**, and `varlookup` reproduces to **0.0001%** where `rexxcps` carries about
0.01% intrinsic spread from rendering `TIME()` into its own output. So
`varlookup` both exercises the transform in its hot loop and can resolve the
answer.

**On `rexxcps` this will bank almost nothing and that is expected**, because its
constant-to-store pairs are separated by a `TraceLiteral` that still has to
emit, which is what killed item 5's two-op form at six executed pairs. Report
`rexxcps` anyway, always, and do not treat a small number there as a failure.

## The number I want out of this

**Instructions removed divided by ops removed**, on `varlookup` and on `arith`,
each stated separately. Count ops by summing execution counts at the dispatch
jump addresses; the method is in `2026-09-20-instructions-per-op.md` and has
been reproduced independently twice.

If it lands near 8, widening the ops is worth an afternoon and the answer is to
stop after this. If it lands near 54.8, the clause-shape distribution says ten
shapes cover 65% of `rexxcps`' timed body and the direction is worth a plan.
**Either answer is a result. Say which one you got, and do not average the two
axes into one figure** -- if they disagree, the disagreement is the finding.

### A prediction, recorded before the measurement exists

The implementer who decomposed the driver predicts **8 per removed op on this
shape**, reasoning from the base dump that the region dispatch preamble is five
instructions at `drive.rs:664` and the latch three. **That is a prediction to
falsify, not a result**, and it is written here because this project has twice
had a prediction come out wrong in **sign**, and the only reason anyone knows is
that it was written down first.

Do not let it anchor you. If you get 40, report 40 and say the prediction was
wrong; if you get 8, the prediction was right and the direction is settled
cheaply. Write your own prediction beside it before you measure, and report both
against the result.

## The hazards, from the analysis that killed the narrower version

1. **The register must be dead after the `Store`.** Fusing writes the value
   straight to the slot, so anything that reads the producer's `dst` afterwards
   reads a register that was never written. Scan the rest of the emitted region
   for a read of that register and refuse the fusion when you find one.
2. **Nothing may branch to the `Store`.** A jump whose target is the `Store`
   would enter a fused op in the middle. Refuse when any target in the chunk
   names that index.
3. **A trace op between the two disables it**, which is self-limiting and
   correct: the echo still has to happen.
4. **Emit the fusion where the ops are emitted, not as a pass over a finished
   stream.** That is how the `Condition` + `JumpUnless` fusion avoided
   renumbering every jump target and every side table keyed by an op index, and
   it is the reason that commit had no remapping hazard at all.

## What would make me stop reading

**If the design needs a conditional in the per-clause path, stop and report
it.** Any conditional there costs about 0.5% on `rexxcps` and 1.25% on
`emptyloop` whatever it does, measured in five shapes, and a commit was reverted
on exactly that arithmetic.

**And watch the driver's frame.** It is currently **1,416 bytes** (`sub $0x588,
%rsp` in the prologue) with 42.9% of its instructions never executed on
`rexxcps`, and 37.6% of its unattributed cost is spill and reload. A new op arm
is more code in that function. Read the frame size before and after with
`objdump`, and **report it whether or not it moved** -- if instructions fall
while the frame grows, that interaction is worth more than the percentage.

## How it is judged

**The oracle differential is the arbiter.** `corpus_differential` has been 604
of 604 STRICT for five commits; keep it there. The golden op-stream tests will
move, and that movement is the instrument that shows the fusion happened: quote
the before and after for a program whose stream you can read.

Gates as `rust/CLAUDE.md` defines them. **Build `--all-targets` outside the cap,
then test under `memcap 8G`** -- a gate run this week exited 137 because the cap
was wrapping the build and OOM-killed `rustc` with zero `test result:` lines.
Expected at BASE: 133 binaries, 2651 passed / 0 failed / 4 ignored release,
2652 / 0 / 4 debug. A sub-second clippy against a warm target is provisional;
re-run it cold.

Commit before any long run and leave the tree frozen until the status file says
finished.

## Measurement

`valgrind --tool=callgrind`, interleaved arms, own `CARGO_TARGET_DIR` per build,
and confirm each measured binary against a rebuild by `.text` hash:
`objcopy -O binary --only-section=.text <bin> <out> && sha256sum <out>`. A
whole-file hash cannot do it, because `debug = true` puts the target directory's
path into the debug info.

**Measure in the shipping profile.** Settled 2026-09-22: at `-O2` without LTO
three of five landed cells flip sign, and an `-O2` profile reports 5.319% of
cost in calls that fat LTO inlines to exactly zero.

## Report

`.superpowers/sdd/2026-09-22-store-fusion-report.md`, or `.txt` if your harness
refuses `.md`. **Replies truncate at about 8 KB: gate statuses first**, then the
price per removed op, then the measurements, then concerns.
