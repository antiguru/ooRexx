# Committed measurement records

**The pinned builds these are measured against are described in `PINNED.md`**, which also says which of the two `phase-5a` files belongs to which plan and which pin. `pinned/` itself is git-ignored.

One machine-readable record per landed task, written by `rexx-arms`
(`crates/rexx-bench/src/bin/rexx-arms.rs`).

## Why the file rather than the report

A summary of a measurement is a new claim.
Restating a ratio in a brief is authorship rather than quotation, and the restatement can be false while every word around it is true -- which is how this phase has twice carried a number forward wrong.
A brief that cites `phase-4e-arms.tsv` and a task that appends to it are quoting the same bytes.

## Reading `phase-4e-arms.tsv`

One row per figure, tab separated, with a header row.
Long rather than wide: a reduction added later is a new value of `scope`, and no column moves.

| column | meaning |
|---|---|
| `task` | the task that took the reading |
| `commit` | the commit the reading describes; for a multi-build sitting, the build column names which |
| `axis` | the benchmark program, by its `bench-programs/` stem |
| `build` | the `--build` label, or `from>to` for an `across_builds` row |
| `scope` | `arm_ratio`, `absolute`, `per_pass`, `per_pass_gap`, `fixed` or `across_builds` |
| `arm` | `tw`, `ir`, `ir/tw` for a ratio, `ir-tw` for a difference |
| `size` | `small` or `large`, or `-` where the figure spans both |
| `instrument` | `instructions:u` or `cycles:u` -- every figure is emitted on both |
| `value_median`, `value_min`, `value_max`, `value_rounds` | the median and the spread of the rounds behind it |

## What a row is and is not

An `arm_ratio` row is **within one binary**: both arms are `REXX_ENGINE` settings of the build its `build` column names.
That is the only form of arm comparison this phase treats as valid, because Task 4c built the same source twice differing by one comment and read 7.8% between the two on a `.text` with an identical sha256.

An `across_builds` row is between two binaries and is the weaker kind.
Interleaving removes the machine's drift from it; it does not remove the code placement's, which Task 8 bounded at `+/-0.74%` on an axis the change it was measuring could not reach.
