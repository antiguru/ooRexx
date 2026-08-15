# Task 4b-M: the instrument, the seven candidates, and what removing them bought

Started from `401e0df5`.
This task promotes nothing: no change here alters what `ir::compile` emits.

## Summary

The instrument is `perf stat -e instructions:u`, cross-checked with `valgrind --tool=callgrind` for
exact per-function and per-line attribution.
It reads **zero across two independent builds of identical source**, which is the control no revert
over these binaries could have produced.

Of the seven named candidates, **two carry the regression and both are on the path both arms take**:
the width of `Flow` (candidate 2, whose named cause is `LeaveOrigin` held inline) and the generic
closure layers every stepped clause passes through (candidate 3).
Together they account for the whole of the shared-path instruction regression on `emptyloop`.
Candidates 1, 5 and 6 are IR-specific and jointly worth 0.7%; candidate 7 is exactly 4 instructions
per `IF` and exactly zero on both registered axes.

Removing them takes the tree-walker arm **7.0% below** the pre-driver binary in cycles on
`emptyloop` and the IR arm **3.1% below** it, and takes both axes below `phase-4e-anchor.md`'s
oracle ratio.
Exit criterion 4 now **passes on `emptyloop`** (IR/tree-walker wall ratio 0.996, IR faster in 5 of 9
paired rounds) and **fails on `varlookup`** (ratio 1.016, IR faster in 0 of 9).

**The residual `varlookup` gap is inherent to the two-level op-driver shape**, and the design
section below says why, with the callgrind numbers behind it.

## The instrument, and its demonstrated zero

`perf stat -e instructions:u` on the whole `rexx-run` process, one program per invocation, arms
interleaved within one sitting, round 0 discarded as a warm-up.
`/proc/sys/kernel/perf_event_paranoid` is `-1` here, so user-space instruction counting needs no
privilege.

**Repeatability of one cell**, `emptyloop` on the `401e0df5` binary, tree-walker arm, eight
consecutive rounds:

```
40050659234  40050658734  40050658489  40050659167
40050658434  40050658927  40050658746  40050658891
```

Spread 800 instructions out of 4.005e10, or 2 parts in 10^8.

**The zero.**
The control the brief asks for is an instrument that reads zero when nothing changed.
Two binaries were built from `401e0df5`: one in the repository checkout, one in a separate git
worktree at the same commit.
They are different files -- sha256 `dd5a93e5...` and `c6709c82...`, different embedded build paths,
different layout -- and semantically identical.

| arm | checkout build | worktree build | difference |
|---|---:|---:|---:|
| tree-walker | 40,050,658,533 | 40,050,658,583 | +50, or +1.2e-9 |
| IR | 41,075,662,430 | 41,075,661,754 | -676, or -1.6e-8 |

That is the control.
No arm is held fixed and nothing is reverted; the instrument simply does not see the layout
difference that broke Task 4b's wall-clock control.

**The instrument's own limits, stated rather than assumed.**

* It counts instructions, not the quantity exit criterion 4 names.
  An instruction ratio equals a wall ratio only if both sides retire instructions at the same rate,
  and **here they demonstrably do not**: on `emptyloop` the IR arm executes 4.14% more instructions
  than the tree-walker and finishes in 0.31% fewer cycles, because its IPC is 4.88 against 4.67.
  Every claim about the gate quantity below is therefore made on cycles and wall-clock, with
  instructions used to attribute causes.
* It has a rare heavy tail.
  One reading in roughly thirty came in at 41,200,658,879 where the other twenty-nine read
  40,050,658,xxx -- a 2.9% jump.
  The most likely cause is the per-process random hash seed changing a probe sequence; it was not
  chased.
  Every figure below is a median of at least three rounds, which is enough for a tail at that rate.
* Callgrind's counts are exact and deterministic but are taken on a shrunk axis (200,000 passes
  rather than 25,000,000), so they price *per-pass* costs and are not comparable to a full-axis
  total.

## Where the regression actually sits: shared path against IR-specific

Medians of three interleaved rounds, `perf stat -e instructions:u`.
`parent` is `736bf080`, the last commit before the op-level driver; `head` is `401e0df5`.

**`emptyloop`, 25e6 passes**

| binary | tree-walker | IR | IR - tree-walker |
|---|---:|---:|---:|
| parent | 38,850,658,059 | 39,550,659,800 | +700,001,741 (+1.80%) |
| head | 40,050,658,780 | 41,075,662,065 | +1,025,003,285 (+2.56%) |

**`varlookup`, 19e6 passes**

| binary | tree-walker | IR | IR - tree-walker |
|---|---:|---:|---:|
| parent | 72,295,651,718 | 73,302,653,127 | +1,007,001,409 (+1.39%) |
| head | 74,309,652,270 | 75,430,654,575 | +1,121,002,305 (+1.51%) |

Read as two effects:

| effect | `emptyloop` | `varlookup` |
|---|---:|---:|
| shared path, tree-walker arm parent -> head | **+3.09%** | **+2.79%** |
| IR-specific, the gap's own growth | +0.76pp | +0.12pp |
| IR arm parent -> head | +3.86% | +2.90% |

**About four fifths of the IR arm's regression is on code the tree-walker runs too.**
That is the same conclusion the brief records from the diff review, now with a number, and it is why
a control that reverts the promotion could not read zero.

## The candidates

Each row is a build with the candidate removed, measured against the same reference in the same
sitting.
Percentages are of the whole run's instruction count.

| # | candidate | verdict | the number |
|---|---|---|---|
| 1 | `op_at(start)`/`op_at(end)` and an extra call frame per `DO`-body pass | **confirmed in kind, mis-sized** | callgrind prices the two `op_at` at **3 instructions per pass** of the 74 the IR arm costs. The frame around them is 21 (10 at the call site, 11 in the prologue). Removing the frame by inlining, with candidate 6, bought 50e6 (**0.12%**) on `head` and 275e6 (**0.69%**) on the fixed tree. |
| 2 | `absorb` moves a 64-byte `Flow` across a call per clause | **confirmed; the cause is the width, not the call** | `#[inline]` on `absorb` reads **exactly 0** -- it was already inlined, so there is no call. Making `Absorbed` not carry the `Flow`: **-450e6 (-1.12%)** tree-walker, **+375e6** IR. Boxing `LeaveOrigin` instead takes `size_of::<Flow>()` from **64 to 24** and reads **-950e6 (-2.37%)** tree-walker, **-350e6 (-0.85%)** IR. |
| 3 | `step_in_temps_frame_with` routes through the generic `in_stepped_clause` with a closure | **confirmed; the largest single cause** | a monomorphic clone of the clause unit: **-750e6 (-1.87%)** tree-walker. `#[inline]` reads 0. `#[inline(always)]` on `in_stepped_clause` alone: -300e6. On `in_stepped_clause` **and** `in_clause` together: **-1,075e6 (-2.68%)** tree-walker, **-950e6 (-2.31%)** IR. |
| 4 | `BodyEngine` grew 8 to 16 bytes and is copied on every hop | **not separately confirmed** | not isolated. It is a capture of candidate 3's closure, and candidate 3's fix removes the closure. No residual is attributable to it: candidates 2 and 3 together account for the whole shared-path regression (see below). |
| 5 | `run_ops` takes eight arguments | **refuted as a separate cause** | callgrind bounds the entire call site plus prologue at **21 of 74** instructions per pass. Removing the call site outright by inlining `run_bounded_from_chunk` bought **2 instructions per pass**. |
| 6 | `granting.grants()` is a new branch per clause | **refuted** | callgrind: **2 instructions per clause**. Folded into a `const` parameter; measured jointly with candidate 1 above. |
| 7 | `if_targets` computes `skip_else` on both of `IF`'s paths | **confirmed, negligible** | callgrind, a loop whose body is one always-false `IF`: `skip_else` costs **800,000 instructions over 200,000 executions, exactly 4 per `IF`** -- 0.10% of that program. Exactly **0** on `emptyloop` and `varlookup`, neither of which holds an `IF`. |

**Candidates 2 and 3 add up to the shared-path regression, and that is the check that closes the
accounting.**
The shared-path regression on `emptyloop` is 1,200,000,443 instructions.
Candidate 2's `Absorbed` fix reads 450,000,068 and candidate 3's monomorphic clone reads
750,000,085.
They sum to 1,200,000,153, which is the regression to within 0.03%.
No fourth shared-path cause is needed to explain it, which is what leaves candidate 4 with no
residual to claim.

### On candidate 2, and what the reviewer's `size_of` observation was really about

The reviewer's `size_of::<Flow>() == size_of::<Absorbed>() == 64` is right and the diagnosis
attached to it -- a call boundary -- is not: `absorb` is inlined at both call sites and annotating
it changes nothing.
What costs is the **width**, and the width has one cause.
`Flow` is 64 bytes because `Leave` and `Iterate` hold a `LeaveOrigin` inline, `LeaveOrigin` is 48
bytes, and 24 of those are a `Vec<u8>` that only a failing `LEAVE`/`ITERATE` ever reads.
Every clause in the interpreter returns a `Flow`, and on `emptyloop` every one of them is
`Flow::Next`.

Boxing it takes `Flow` to 24 bytes.
**Both sides of that trade were measured**, as the hypothesis asked.
The allocation it adds is paid once per `LEAVE`/`ITERATE` *executed*, and a probe that executes one
`LEAVE` per pass (`do i = 1 to 2000000 / do forever / zc = zc + 1 / leave / end / end`) reads
**-4.12%** on the tree-walker arm and **-2.78%** on the IR arm boxed.
The trade is favourable on both sides, so there is no workload found here that pays for it.

### On `BodyEngine` by reference, tested as the hypothesis framed it

The suggestion was to test it as "does shrinking the closure's capture help", not as "is passing 16
bytes expensive", and that is the right framing -- but it is not separable from candidate 3 as
things stand.
The monomorphic-clone experiment removes the closure *and* passes `engine` as an ordinary argument,
and it reads -750e6; the `#[inline(always)]` pair keeps the closure and reads -1,075e6, more than
the clone.
Since the annotation that keeps the closure beats the variant that deletes it, there is no residual
capture cost left for a smaller `BodyEngine` to recover, and a by-reference variant was not built.
**This is a "not established" rather than a refutation**: it means the closure's capture is not
where the remaining cost is, not that a reference would be free.

## What was removed, and what it bought

Four changes, all of them either a type change or an inlining annotation, none of them touching what
a promotion emits:

* `Flow::Leave`/`Flow::Iterate` hold `Box<LeaveOrigin>`, and `pop_search_frame` updates through the
  box instead of rebuilding the value. `Flow` 64 -> 24 bytes.
* `#[inline(always)]` on `Interp::in_stepped_clause` and on `Interp::in_clause`, the two
  generic-over-a-closure layers every stepped clause passes through.
* `#[inline]` on `Interp::run_bounded_from_chunk`.
* `Granting` becomes a `const GRANTING: bool` parameter of `run_ops`, so the per-clause branch folds
  away at each call site. The enum is deleted and its reasoning moves to `run_ops`' own doc comment.

Instruction counts, medians of three interleaved rounds:

**`emptyloop`**

| binary | tree-walker | IR |
|---|---:|---:|
| parent `736bf080` | 38,850,658,059 | 39,550,659,800 |
| head `401e0df5` | 40,050,658,780 | 41,075,662,065 |
| fixed | 38,000,659,124 | 39,575,662,246 |
| fixed against head | **-5.12%** | **-3.65%** |
| fixed against parent | **-2.19%** | **+0.06%** |

**`varlookup`**

| binary | tree-walker | IR |
|---|---:|---:|
| parent | 72,295,651,718 | 73,302,653,127 |
| head | 74,309,652,270 | 75,430,654,575 |
| fixed | 71,820,652,236 | 73,169,655,201 |
| fixed against head | **-3.35%** | **-3.00%** |
| fixed against parent | **-0.66%** | **-0.18%** |

The shared-path regression is gone and both arms are at or below the pre-driver binary.

The binary built in the repository checkout after these changes reproduces the experimental build's
counts exactly (38,000,658,748 / 39,575,660,653 against 38,000,658,207 / 39,575,661,645), so the
committed tree is the tree that was measured.

## Wall-clock and cycles: where the phase stands

Wall-clock is what exit criterion 4 names, and cycles are what reconcile it with instructions.

**Cycles**, `perf stat -e cycles:u`, medians of three interleaved rounds:

| axis | binary | tree-walker | IR | IR arm against parent |
|---|---|---:|---:|---:|
| `emptyloop` | parent | 8,735,502,132 | 8,356,394,731 | -- |
| `emptyloop` | head | 9,788,629,385 | 9,300,121,074 | **+11.3%** |
| `emptyloop` | fixed | 8,122,145,848 | 8,097,101,071 | **-3.10%** |
| `varlookup` | parent | 15,921,167,992 | 15,356,041,628 | -- |
| `varlookup` | head | 15,704,838,496 | 16,372,009,110 | **+6.62%** |
| `varlookup` | fixed | 14,990,802,349 | 14,975,264,963 | **-2.48%** |

The regression Task 4b reported reproduces here at +11.3% and +6.6% in cycles, against its own
+7.0% and +2.7% in wall-clock -- same sign, same axes, larger.
The fix removes it and leaves the IR arm 2.5-3.1% below the pre-driver binary.

**Wall-clock**, the anchor's method: `ulimit -v 8388608` on every child, every child from one fresh
empty temporary directory, `/usr/bin/time -f %e`, medians of seven interleaved rounds, one sitting.

| axis | oracle | parent tw | parent IR | head tw | head IR | fixed tw | fixed IR |
|---|---:|---:|---:|---:|---:|---:|---:|
| `emptyloop` | 0.890 | 3.000 | 2.860 | 3.160 | 3.150 | **2.790** | **2.780** |
| `varlookup` | 1.190 | 5.380 | 5.220 | 5.340 | 5.530 | **5.010** | **5.080** |

**Against `phase-4e-anchor.md`.**
The anchor's figures are tree-walker against oracle: `emptyloop` 2.9825 s against 0.8955 s, ratio
3.33x; `varlookup` 5.3131 s against 1.2075 s, ratio 4.40x.
The oracle side in this sitting (0.890, 1.190) is within 1% of the anchor's medians, which is what
makes the ratios comparable across the two sittings.

| axis | anchor ratio | fixed tree-walker | fixed IR |
|---|---:|---:|---:|
| `emptyloop` | 3.33x | **3.13x** | **3.12x** |
| `varlookup` | 4.40x | **4.21x** | **4.28x** |

Both axes are better than the anchor on both arms.

### Exit criterion 4

The criterion is that the IR arm is not slower than the tree-walker arm.
Nine paired wall-clock rounds on the fixed binary, both arms of one binary, interleaved:

| axis | tree-walker median | IR median | IR/tree-walker | IR faster in |
|---|---:|---:|---:|---|
| `emptyloop` | 2.790 | 2.780 | **0.9964** | 5 of 9 pairs |
| `varlookup` | 5.010 | 5.090 | **1.0160** | **0 of 9 pairs** |

`emptyloop` passes.
`varlookup` fails, and it fails cleanly rather than marginally: the two samples do not overlap at
all (tree-walker 4.99-5.02, IR 5.07-5.12), so this is not a reading that more rounds would move.

**The criterion held at the parent on both axes** (parent IR 2.860 against 2.860 tree-walker 3.000;
5.220 against 5.380), so `varlookup` is a regression the driver introduced and this task did not
fully remove, not a pre-existing failure.
That the parent's IR arm was *faster* in wall-clock while executing *more* instructions on both axes
is itself worth recording: the parent's advantage was a layout effect, not work saved.

## The design finding: the residual gap is the two-level shape

After the shared-path costs are removed, the IR arm still executes **74 more instructions per
`DO`-body pass** than the tree-walker on `emptyloop`.
Callgrind on the fixed tree, 200,000 passes, totals 300,712,415 instructions on the tree-walker arm
against 315,516,004 on the IR arm -- a difference of 14,803,589, which is 74.0 per pass and matches
the sampled instrument exactly.

Where it goes, from the same run:

* `run_repeating` is **4,000,000 cheaper** on the IR arm (20 per pass), because the tree-walker's
  `run_bounded_instructions` is inlined into it outright and the IR arm's is not;
* `run_ops` costs **about 16,400,000 exclusive** across its inlined regions for 200,000 calls, and
  `run_bounded_from_chunk` a further 11 per pass at the call site;
* everything else is identical to the instruction, `step_in_temps_frame_with` included -- 20,800,000
  on both arms.

So the whole of it is the `run_bounded_from_chunk` -> `run_ops` pair: a real call per pass with a
nine-argument frame, two instruction-space-to-op-space map lookups, and a six-way op dispatch, for a
body range that contains **exactly one op**.
The tree-walker's equivalent is three instructions of inlined loop.

**This is the shape, not a defect in it.**
`run_ops` is large because it drives every op kind and the promoted-clause region; that is what
makes it not inlinable where the tree-walker's two-line loop is.
The obvious remedies are both excluded: a specialised fast path inside `run_ops` for a
single-`Generic` range is the second loop the dual-engine sweep exists to catch, and hoisting the
map lookups out of the per-pass entry means the construct's op range travelling with `BodyEngine`,
which changes what a promotion has to emit -- and this task promotes nothing.

The instructions are largely absorbed by higher IPC (4.88 against 4.67 on `emptyloop`), which is why
the arm lands at parity there and 1.6% behind on `varlookup` rather than 4% behind on both.
**That absorption is luck, not design**: it depends on the tree-walker's loop being the one with the
dependent chain, and nothing holds it in place as more constructs are promoted.

## Verification

| gate | result |
|---|---|
| `cargo test --workspace` | 1364 passed, 0 failed, 4 ignored |
| `cargo test --workspace`, all four `*_GATE` variables set | 1364 passed, 0 failed, 4 ignored |
| `cargo test --workspace --release` | 1364 passed, 0 failed, 4 ignored |
| `cargo test --workspace --release`, STRICT | 1364 passed, 0 failed, 4 ignored |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |

The baseline is unchanged, which is what a change of this kind should do: it is four annotations and
one boxed field, and none of them is reachable by a behavioural test.

## What this task did not establish

* **`BodyEngine` by reference.** Not built, for the reason given above -- the annotation that keeps
  the closure beat the variant that deletes it, so there was no residual capture cost to attribute.
  This is a gap, not a refutation.
* **Why the `Absorbed`-by-reference variant costs the IR arm.** It reads -450e6 on the tree-walker
  and +375e6 on the IR arm, and after the boxing it is redundant, so it was not shipped and the
  asymmetry was not chased.
* **Anything about `alloc4c`, `arith`, `compound` or `strings`.** Not measured on either instrument.
  The boxing and the two annotations are on every clause of every program, so they very likely move
  those axes too, in the same direction -- but that is a prediction, not a measurement.
* **Whether the `varlookup` residual can be closed at all inside the current design.** The
  attribution above says where it is; it does not prove no admissible change removes it.
* **The rare instruction-count outlier's cause.** Suspected to be the per-process hash seed;
  measured at roughly one reading in thirty and handled with medians rather than diagnosed.
* **Anything about `branchloop`.** Candidate 7 was priced with callgrind on a purpose-built
  always-false `IF` loop; no wall-clock comparison was taken on it.
