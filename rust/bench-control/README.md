# Control programs

Not benchmark axes.
Each file here is a **control**: a program that differs from an axis in `../bench-programs/` by a known amount, run alongside that axis so a measurement can be shown to resolve a difference of that size.

`phase-4d-gate.md`'s escalation rule is what needs them.
An axis whose ratio lands near 1.0 earns more runs until its interval separates from 1.0, and a tight interval on its own does not distinguish a sensitive instrument from a blind one.
Running a pair that differs by a known small amount at the same time answers that: an instrument that cannot separate the control pair has not certified anything about the axis.

These are deliberately **outside** `bench-programs/`.
`rexx-bench-suite` asserts its axis list against that directory in both directions, so a program added there becomes a dimension of the committed baseline; a control is not a dimension of anything.
`rexx-bench-band` reaches them by path -- an `--axes` entry holding a `/` is taken as a path rather than as a name.

## The pair

| control | its axis | difference |
|---|---|---|
| `alloc4c-101.rex` | `../bench-programs/alloc4c.rex` | loop bound 1,010,000 against 1,000,000 -- exactly 1% more iterations, nothing else changed |

The two files are byte-identical apart from the `n = ` line, and `the_control_differs_from_its_axis_only_in_the_loop_bound` asserts both halves of that: same bytes everywhere else, and a bound exactly 1.01 times the axis's.
Neither half is safe to leave in prose.
A control that drifted from its axis in some other way would still look like a 1% control and would silently stop being one, and a bound edited to a round number would change the size of the effect the instrument is being asked to resolve without changing anything a reader would notice.

**1% of iterations is a little under 1% of wall time**, because each side pays a fixed per-process offset that does not scale with the loop.
`perf-baseline.md` puts that offset at no more than 0.84% of any axis on either side, so on `alloc4c` the expected wall-time difference is within a few hundredths of a point of 1%.
Predict it from the offset the measurement itself reports rather than assuming exactly 1%.
