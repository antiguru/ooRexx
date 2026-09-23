# Spike: cold-arms -- per-arm outlining of the driver's never-executed arms

Read `common.md` beside this file first; it binds you.

## The idea

`run_ops_from::<true>` has 57% of its instructions never executed on `rexxcps`,
and the allocator plans one frame and one set of spill decisions across all of
it. `2026-09-22-driver-frame-pressure.md` tested outlining in the **bundled**
form (Candidate E: one helper whose frame is the union of every cold arm's
needs) and rejected it; the **per-arm** form -- each cold arm's body moved into
its own `#[cold] #[inline(never)]` method, the arm reduced to a call -- was
never tried. `2026-09-22-frame-instrument-falsified.md` argues it is the more
interesting direction after store fusion: how the driver is divided into
functions matters more than anything inside one.

## Doing it

1. Find which arms execute. Use `callgrind --dump-instr=yes` on the base with
   `rexxcps`, `varlookup`, `arith`, `emptyloop`, `dispatch`, `compound`, and
   map executed addresses back to arms (the jump table dispatch addresses, per
   `2026-09-20-instructions-per-op.md`). An arm is hot if any axis executes it.
   Commit the list and the command that produced it.
2. Outline every arm that is cold on all axes, one method per arm. Leave the
   hot arms untouched.
3. **Control**: the outlined arms are unreachable on the measured axes by
   construction, so any change on those axes is purely the hot code's
   allocation changing. That is the thing being measured, so the control here
   is the opposite direction: also build a variant that outlines **only half**
   the cold arms (pick deterministically, e.g. every other one in source
   order). If full and half move the axes in the same direction with roughly
   proportional size, the effect is real; if half moves more than full, or in
   the opposite direction, it is allocator noise and say so.
4. If an arm's body needs so many locals from the driver that the call site
   becomes large, say which and leave it inline; do not contort it.

Remember the measured precedent: an out-of-line store tail in the store-fusion
experiment came out **worse** than inline on both axes. Your prediction should
engage with that.

Spike name: `cold-arms`.
