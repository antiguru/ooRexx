# Phase 5j — the growth witness, and what it actually found

D59a's fourth consequence is OOM under class accumulation. The program is 200,000 `~subclass`
calls, each dropped immediately. Resident set measured on the interpreter itself, not on a wrapper.

## The readings, in the order they were taken

| build | retained classes | max RSS |
|---|---|---|
| before the expunge freed behaviours | — | 4,001,512 kB |
| with a dead class's behaviours emptied | 3,814 | 1,396,296 kB |
| the same, forcing a collection every 1000 iterations | **0** | **82,748 kB** |
| the C++ oracle, same program | 1 | 116,812 kB |

**A control says the growth is class-attributable.** The identical loop building the same strings
and creating no class is flat: 20,580 kB at 20,000 iterations and 21,300 kB at 200,000.

## What it means

**There is no permgen and no leak.** 196,186 of the 200,000 classes were collected without any
forcing, and with collection driven the resident set is *below* the oracle's — 82.7 MB against
116.8 MB — with nothing retained at all.

**What the 1.4 GB is, is the collector's scheduling.** `collect_at` is
`COLLECT_FLOOR.max(live * 2)`, a count of arena objects. A class is one small arena object that
owns two method dictionaries living in `rexx-classes`, off the arena entirely, so the heuristic
cannot see the weight a class carries and lets the arena run far past where a class-heavy program
should have collected.

**That is a follow-on, not a phase blocker**, and it belongs with the heap-representation spike
rather than here: the fix is for the collection trigger to account for off-arena weight, which is a
question about the trigger and not about class lifetime.

## Two corrections to my own work, recorded because they were wrong on the way through

**Emptying the dead behaviours is worth 4.0 GB → 1.4 GB, and that is measured.** The dictionaries
of a collected class are where the bulk was.

**But my stated reason for one edit was false.** I replaced `MethodDict::clear()` with a fresh
`MethodDict::new()` on the theory that `clear()` retains capacity and that the capacity "is the
whole of the weight". Measured either way: 1,395,816 kB against 1,393,692 kB, which is noise. The
theory was wrong, the edit bought nothing, and it is reverted to `clear()` — the oracle's own
`clearMethodDictionary` — rather than kept with a justification the measurement does not support.

**And the first attribution was wrong too.** I read the residual as a per-class leak of about 5 kB
and went looking for the table holding it. The forced-collection run says there is no such table.
The lesson is the one the control taught: measure the attribution, do not infer it from a slope.
