## Unit 0 -- characterise the noise, then reduce it

**Nothing in this phase is decidable until this closes, and it blocks the gate as much as it blocks the loop.**

`phase-4d-gate.md` currently declares an undecidable band of 7.2%.
That figure is **the largest of three observations**, adopted as a lower bound because three runs cannot support anything better.
A 7.2% band means a change worth 5% is unmeasurable, and most individual optimisations are worth less than that.
It is a broken instrument, not a fact about the machine.

**Step 1: characterise.**
30 to 50 repetitions per side per axis, same binary, same program, same machine state, so the *distribution* of the ratio is known rather than its observed extreme.
Report the distribution, not a single number: median, spread, and the shape, because a long tail and a wide symmetric spread need different responses.

**Step 2: reduce.**
Most of the variance is likely attackable and none of it requires touching the interpreter:

* pin to a core with `taskset`, so a migration mid-run stops being a measurement,
* fix the CPU governor, so frequency scaling is not part of the signal,
* measure cycles rather than wall time where the comparison allows it,
* control residency and page-cache state between runs,
* remove the harness difference `phase-4d-attribution.md` records between the suite and the ad-hoc scripts, so one harness produces every number.

**Step 3: re-derive the band from the reduced distribution**, and amend `phase-4d-gate.md` with both wordings and the reason.

**The target is a band under one per cent**, because the loop below cannot accept a change it cannot measure.

**Do not flip UNDECIDED into a pass while the band is wide.**
"Within noise" is the right bar only when noise is small; with a 7.2% band it would mean 7.2% slower passes, and a worse instrument would make the gate easier. That is the vacuity shape this project has shipped before. The band shrinks first.
