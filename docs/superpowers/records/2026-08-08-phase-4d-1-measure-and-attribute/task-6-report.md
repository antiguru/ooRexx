# Task 6 report: profile every axis and attribute the gap

Status: DONE, after one review round.
Commits `7b643c59acad69d7a535cecfdc475bded69e9089` (the attribution) and
`9ce83f141dac94787e858200595ccf805cdfad1e` (the review corrections), branch `plan/rust-rewrite`.
Parent of the first is `c445154c`.
Deliverable: `docs/superpowers/plans/phase-4d-attribution.md`.
Working tree clean before and after; three prototypes applied and reverted inside that commit.

## Step 1: the dominant question, answered before anything else

**The per-axis ratio spread is a property of this crate, not of the denominator, and the hypothesis
the spec records is now false.**
It was true at `107febcd`; the five speedups that landed after that measurement are what changed it.

From absolute throughput, not ratios (committed baseline, 2026-08-08 at `d233d1e9`):

| axis | oracle ns/iter | this crate ns/iter | difference |
|---|---:|---:|---:|
| `varlookup` | 63.9 | 277.9 | 214.0 |
| `compound` | 230.0 | 1399.3 | 1169.3 |
| `strings` | 287.8 | 3053.7 | 2765.9 |
| `alloc4c` | 1127.9 | 2342.2 | 1214.2 |
| `arith` | 2311.3 | 6241.5 | 3930.2 |

The oracle spans 36.2x across the five; **this crate spans 22.5x**, within a factor of 1.6 of it.
At `107febcd`, over the four axes that existed then, the same figures were 36.1x and **5.5x** -- a
crate whose cost barely moved across workloads whose oracle cost moved by 36x, which is the shape a
single large workload-independent constant makes.

A constant multiplier is refuted by the ratios (2.08x to 10.61x).
A constant additive cost per *iteration* is refuted by the differences (18.4x range), but that is a
straw model, since the axes run different numbers of clauses per iteration.

**The model that had to be refuted is a constant per *clause*, and it nearly fits.**
Loop-body clause counts are 2, 2, 3, 5, 7, giving per-clause differences of 107.0, 584.7, 404.7,
553.2 and 561.5 ns -- four of five inside **1.44x**, and inside 1.62x if the stepped `DO` header is
counted too.
What refutes it is the prototype split rather than the table: in one interleaved run **P1 moved
`compound` -51.6% and `arith` -0.7%**, though `arith` runs 3.5 times as many clauses per iteration,
and **P3 moved `arith` -16.3% while making `strings` and `compound` slower**.
No constant produces effects that differ in size by seventy-fold and in sign by axis.
The near-fit is a consequence of several per-clause and per-assignment causes aggregating, not an
alternative to them, and `varlookup` at 107 ns per clause is where even that model fails.

So decomposition is warranted rather than one task attacking one thing, which is what determined the
shape of everything below it.

## The named causes, with shares

Full evidence, per-axis shares, and call-site citations are in the deliverable.
Shares are subtree totals from `samply` at 1 kHz through `pollard` with `expand_inlines`;
`unsymbolicated_pct` was 0 on two profiles and at most 0.0095% on the other three.

| cause | mechanism | share | implied ratio if removed |
|---|---|---|---|
| C1 | variable binding resolved through hash maps; an assignment resolves by copying and hashing the name text (`run.rs:2529`, `plan.rs:110`, `lib.rs:1057`) | `varlookup` 32.7%, `compound` 14.2%, `strings` 7.6%, `alloc4c` 7.0%, `arith` 4.0% | `varlookup` 2.93x, `compound` 5.22x, `strings` 9.80x, `alloc4c` 1.93x, `arith` 2.59x |
| C2 | `/`, `%`, `//`, `**` have no small-integer path (`eval.rs:961`) | `compound` 41.5%, `arith` 28.7% | `compound` 3.56x, `arith` 1.93x |
| C3 | a number renders and reparses itself just to classify itself (`value.rs:468`) | `arith` 14.5% | `arith` 2.31x |
| C4 | a compound access re-derives its tail from the source spelling (`stem.rs:111`) | `compound` 12.7%, `alloc4c` 4.0% | `compound` 5.31x, `alloc4c` 2.00x |
| C5 | a builtin call is resolved by UTF-8 validation, a hash-set test and a linear table scan (`builtin/mod.rs:622`, `:680`) | `strings` 9.9%, `alloc4c` 5.2% | `strings` 9.56x, `alloc4c` 1.97x |
| C6 | every heap value is a fresh `malloc` into a never-swept arena | glibc allocator self time: `alloc4c` 39.5%, `strings` 37.7%, `arith` 34.6%, `compound` 30.0%, `varlookup` 4.6% | not implied -- see concern 1 |
| C7 | every stepped clause binary-searches the source line table (`run.rs:4203`) | `alloc4c` 6.0%, `varlookup` 2.9%, `compound` 1.8%, `arith` 1.6% | `alloc4c` 1.96x, `varlookup` 4.23x, `compound` 5.97x, `arith` 2.66x |

C1 and C4 overlap on `compound` and `alloc4c`; among C1 to C5 and C7 that is the only shared call
site, and C6 overlaps all of them by construction, being self time inside their subtree totals.
The shares must not be summed.
C1, C2, C3, C4, C5 and C7 are all the same shape of defect -- work the parser or the plan already
did, thrown away and redone per iteration.

## The three prototypes, measured and reverted

Each is a few lines, each built at `--release` under the pinned profile, each measured interleaved
against the base binary with five repetitions, each reverted.
**All four binaries printed byte-identical stdout on all five axes and all exited 0.**
**Each prototype passed the whole suite -- 1317 tests, 0 failures -- individually and combined.**

| axis | base median | P1 (small-int `//`, `%`) | P2 (assign by symbol id) | P3 (drop the render probe) |
|---|---:|---:|---:|---:|
| `alloc4c` | 2.2931 s | -1.7% | -3.0% | -0.8% |
| `arith` | 3.0794 s | -0.7% | -2.2% | **-16.3%** |
| `compound` | 6.7913 s | **-51.6%** | -2.9% | +1.6% |
| `strings` | 8.9971 s | -1.2% | -3.2% | +2.7% |
| `varlookup` | 5.2149 s | +0.2% | **-10.7%** | +0.5% |

Only the bold cells carry a claim; within-cell spread is 0.5% to 2.6%, so at n=5 a sub-5%
difference is not separated from noise unless one binary's whole range clears the other's, which
holds for `strings` and `compound` under P3 and nowhere else unbolded.

A second interleaved run took the oracle, the base, and a binary carrying all three, five
repetitions, same wrapper on every side, so the resulting ratios are measured rather than derived:

| axis | oracle | base | ratio | all three | ratio |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | 1.1813 s | 2.2765 s | 1.93x | 2.2836 s | 1.93x |
| `arith` | 1.1676 s | 3.0774 s | 2.64x | 2.5068 s | **2.15x** |
| `compound` | 1.1485 s | 6.7831 s | 5.91x | 3.2859 s | **2.86x** |
| `strings` | 0.8593 s | 9.0533 s | 10.54x | 8.7969 s | 10.24x |
| `varlookup` | 1.1970 s | 5.2147 s | 4.36x | 4.6874 s | **3.92x** |

Revert discipline: backups taken with `cp`, restored from the backups, verified with `sha256sum -c`
(three files, all OK), and the rebuilt `rexx-run` has sha256
`c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967` -- byte-identical to the binary
the committed baseline was measured with, which is also what makes every share above a share of a
number already on the record.
Never `git checkout --`.

## Step 4: D9's compound-memoisation mandate was not met

Answered by reading the code, not the ratio.
`Interp::tail_key` (`stem.rs:110`-`125`) runs on **every** compound access: `compound_parts` splits
the interned spelling on `.` into a fresh `Vec<Tail>`, a fresh `Vec<u8>` key is built piece by
piece, and each `Tail::Variable` piece is resolved through `read_by_name` -> `Interp::slot_of` --
the **name-keyed** hash map, not the piece's own slot -- and then rendered with `to_text`.
Nothing on that path is memoised.

The split is already performed once at plan-build time and discarded: `Plan::note_compound_name`
(`plan.rs:439`) calls the identical `compound_parts` to register slots, and its own doc comment
states the consequence -- "that is exactly why `stem.rs`'s `tail_key`/`read_by_name` resolve both of
these purely by name".
So this is a recorded design choice, and it is the one D9 `:389` asked not to make.
Cost: `tail_key` is 12.7% of `compound` and 4.0% of `alloc4c`, the split alone 9.9% and 2.7% -- more
than the stem machinery proper (`stem_get` 4.7%, `stem_set` 4.8%).

The prior profile D9 `:390` points at is the C++ interpreter's and is outside this repository.
Two of its findings bear here: the oracle's own compound tails are a balanced BST with `memcmp` from
`CompoundVariableTable::findEntry` at 21.6% on stem-heavy code, which is part of why `compound` is
6.08x rather than worse; and "locals are already integer slots assigned at parse time
(`RexxLocalVariables`, `locals[index]`)", which is exactly the thing C1 says this crate throws away.

## Concerns

### 1. `alloc4c`'s 2.08x is a denominator artifact, and my first explanation of it was wrong

The ordering finding survives, and the reviewer reproduced both endpoints: the glibc allocator's
share of *self* time is **highest** on `alloc4c` (39.5%), the axis **closest** to the oracle
(2.08x), and **lowest** on `varlookup` (4.6%), an axis mid-range at 4.35x.
Allocator share does not order the axes by ratio, so a bar derived from an allocation figure would
be derived from a quantity that does not predict the gate.

**The reason I gave was wrong, and the corrected reason points the opposite way.**
I wrote that this is a shared cost, citing `MemoryObject::newObject` at 26% of a *different* C++
benchmark -- a subtree total on another program, set against this crate's self time on this one.
I have now profiled the oracle on the axis, which the first version never did: `MemoryObject::collect`
is **56.0%** of its time, `markObjectsMain` 46.6%, and the largest self-time function in the whole
profile is `CompoundTableElement::live` at **20.7% self** -- the mark phase walking `tab.`'s growing
tail table.
This crate spends none of that, because it never collects.

Confirmed a second way, without a profiler: five interleaved repetitions, identical stdout, all exit
0. Replacing `tab.i = i` with `tab.1 = i` takes the oracle from 1077.4 ms to 184.1 ms and this crate
from 2338.2 ms to 1564.8 ms, moving the ratio from **2.17x to 8.50x**.
So **82.9% of the oracle's time on this axis is work the growing table causes**, and without it the
axis reads about 8.5x -- `strings` territory.

**2.08x is a debt, not headroom, and it will get worse when a collector lands.**
`phase-4d-retention.md`'s trigger prototype covered `strings`, `arith`, `compound` and `varlookup`
and **not** `alloc4c`, which I verified rather than assumed: `alloc4c.rex` was added at `d233d1e9`,
after `c9a90906` where that work was measured.
`alloc4c` is precisely the axis where a collector has a large live set to re-mark and little to
reclaim, so a collector is a measured 16% win on `strings` and an unmeasured cost here.
Task 7 owns bounding what remains once allocation *count* falls, which C1 to C4 each reduce.

### 2. Between-run variance is worse than `perf-baseline.md`'s record of it, and it moved axes

The baseline records `compound` producing disjoint intervals across two quiet runs of a
byte-identical binary and flags it for Task 8.
My oracle-interleaved run is a third data point, taken with a byte-identical `rexx-run` and all
three oracle objects hash-matched, and it reads 1.93x / 2.64x / 5.91x / 10.54x / 4.36x against the
committed 2.08x / 2.70x / 6.08x / 10.61x / 4.35x -- `alloc4c` -7.2%, `arith` -2.2%, `compound`
-2.8%, `strings` -0.7%, `varlookup` +0.2%.
**The axis that moved most is `alloc4c`, not `compound`**, so the instability is not a property of
one axis and Task 8 cannot close the question by re-measuring `compound` alone.
Every prototype figure I report is a within-run comparison for exactly this reason; the absolute
levels carry whatever the variance turns out to be.

### 3. `strings` cannot carry a bar, and here is the number that says so

Of 10.61x, the **named and removable** share is C1's 7.6% plus C5's 9.9% = **17.5%**, implying 8.75x
at best.
The only thing measured against it is the combined prototype's **2.8%**, a factor of six short and
itself marginal at n=5.
The rest is C6 (37.7% self in the allocator, whose removability this document does not establish)
and the builtin bodies -- `changestr` 12.2%, `pos` 5.8%, `substr` 5.0% -- which are work the oracle
performs too and which nobody has opened.
Roughly a sixth of `strings` is attributed to a mechanism with a named fix, so a bar derived from
this document for that axis would be a bar over the unattributed five sixths.

### 4. Two shares are real but not addressable by the fix that looks obvious

* **`arith`'s 28.7% in `Number::div`** is not reachable by C2's small-integer path -- its operands
  are not integers -- which P1 confirmed by moving `arith` only -0.7% while moving `compound`
  -51.6%. A 4d-2 task that schedules one fix for both axes would half-miss.
* **P3 is a diagnostic, not a proposal.** Removing the render-and-reparse probe made `compound` 1.6%
  and `strings` 2.7% *slower*, both real at n=5 (the prototype's fastest run was slower than the
  base's slowest), so the probe does accept values `plain_integer` declines. C3's claim is that on
  `arith` it is pure cost, not that it should go.

### 5. Three shares are unconfirmed by prototype

C4 (12.7% on `compound`), C5 (9.9% on `strings`) and C7 (up to 6.0%) are profile shares only.
Their implied ratios are arithmetic on a measured share, not a measured win, and C2's prototype
shows the two can differ by a lot in the *favourable* direction (41.5% share, 51.6% measured).

## Verification

All read unpiped from `rust/`, after the prototypes were reverted and the tree confirmed clean.
Run twice: once before the first commit, and again after the review corrections.

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` from a freshly created empty target dir | exit 0, 61 `Compiling`/`Checking` lines, 0 `warning`/`error` lines on stderr |
| `cargo test --offline --workspace --no-fail-fast` | exit 0, **1317 passed, 0 failed**, 77 result lines |
| `git status --short` after commit | empty |
| source hashes against the pre-prototype backups | `eval.rs`, `run.rs`, `value.rs` all OK, both times |
| `rust/target/release/rexx-run` sha256 | `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967`, the baseline binary |

## Constraints honoured

No optimisation was kept; all three prototypes are reverted and the binary hash proves it.
The C++ tree was never written to.
Every probe and every timed run used a fresh empty directory it created itself, absolute paths, and
`/dev/null` on standard input; stdout, stderr and exit status were read as three separate
descriptors, never `2>&1`.
No cargo exit status was read from a pipeline.
No `git add -A`, no `git reset --hard`, no force-push, no `git checkout --`.
The address-space cap was `ulimit -v 8388608` on both sides of every measurement, matching the
baseline harness rather than the project's standard 1 GiB, which this crate cannot complete
`strings` under.
