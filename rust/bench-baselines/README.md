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
| `task` | the task that took the reading, and which of its sittings -- see below |
| `commit` | the commit the reading describes; for a multi-build sitting, the build column names which |
| `axis` | the benchmark program, by its `bench-programs/` stem |
| `build` | the `--build` label, or `from>to` for an `across_builds` row |
| `scope` | `arm_ratio`, `absolute`, `per_pass`, `per_pass_gap`, `fixed` or `across_builds` |
| `arm` | `tw`, `ir`, `ir/tw` for a ratio, `ir-tw` for a difference |
| `size` | `small` or `large`, or `-` where the figure spans both |
| `instrument` | `instructions:u` or `cycles:u` -- every figure is emitted on both |
| `value_median`, `value_min`, `value_max`, `value_rounds` | the median and the spread of the rounds behind it |

## What the `task` column holds in `phase-5a-arms.tsv`

**A task's number, optionally suffixed**, where the suffix names one particular sitting of that task
rather than the task: `<number>-fixround-N` for a reviewer-driven fix round, and other suffixes where
a task took more than one sitting of the same revision (`11-bisection`, `12-prev`).

Which labels the file actually holds, and against which commits, is what this lists rather than
anything written here:

```
awk -F'\t' 'NR>1 {print $1"\t"$2}' bench-baselines/phase-5a-arms.tsv | sort -u
```

**A bare number does not mean a single sitting**, and neither does a `(task, commit)` pair: several
tasks re-measured the same axes against more than one revision under one label. The transcription
that used to stand here in place of that command was wrong within a day of being written, which is
why the command is what the section offers.

**Together, `task` and `commit` distinguish every sitting in the file**, and that is a property to
keep rather than an observation:

```
awk -F'\t' 'NR>1 {k=$1"|"$2"|"$3"|"$4"|"$5"|"$6"|"$7"|"$8; c[k]++; if(c[k]>1) d++} END {print d+0}' \
    bench-baselines/phase-5a-arms.tsv
```

prints `0`. It printed `312` between Task 12's two sittings of `ee6ebbf64` and the relabelling of the
second to `12-prev`, and a run that leaves it non-zero has produced a table whose rows cannot be told
apart by anything but their position in the file.

**The `task` column is a label and the columns after `commit` are measurements**, and the two are not
edited under the same rule. A measured value is never rewritten -- editing one to match a conclusion
drawn afterwards would be editing data. A label may be corrected, and the correction above is the
case that arises: when one task takes two sittings of one revision, one of them is relabelled so the
key stays distinct, rather than a sitting being dropped.

## What a row is and is not

An `arm_ratio` row is **within one binary**: both arms are `REXX_ENGINE` settings of the build its `build` column names.
That is the only form of arm comparison this phase treats as valid, because Task 4c built the same source twice differing by one comment and read 7.8% between the two on a `.text` with an identical sha256.

An `across_builds` row is between two binaries and is the weaker kind.
Interleaving removes the machine's drift from it; it does not remove the code placement's, which Task 8 bounded at `+/-0.74%` on an axis the change it was measuring could not reach.
