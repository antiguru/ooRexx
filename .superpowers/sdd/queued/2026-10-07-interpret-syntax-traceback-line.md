# 2026-10-07-interpret-syntax-traceback-line

Found by the Phase 6 final review (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/final-review.md` M4).

`interpret "x=1; y=(; z=2"`: the oracle's traceback starts `1 *-* y=(;` and says `Incorrect
expression detected at "("`; ours omits the line and says `at "&1"`. A REPLY inside INTERPRET
(99.924) omits the same line.
