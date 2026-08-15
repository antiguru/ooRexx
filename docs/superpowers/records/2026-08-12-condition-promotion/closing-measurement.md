# Closing measurement -- condition/header/return promotion

Base **02c6b9b67** ("Widen the message to the ops that now report it"), head **de6efa3b1**
("Stop a test's note explaining itself by a promotion that happened"), 20 commits between them.
Taken 2026-08-13, 10:34-11:50 local, on the 32-CPU host, in four detached worktrees under the
session scratchpad, each with its own `CARGO_TARGET_DIR`. The main working tree was not modified
and nothing was committed. The worktrees were removed at the end.

## What the four arms are

| arm | commit | source difference |
|---|---|---|
| A | 02c6b9b67 | none |
| A' | 02c6b9b67 | one trivial edit, below |
| B | de6efa3b1 | none |
| B' | de6efa3b1 | the same trivial edit |

**The trivial edit, exactly.** In `rust/crates/rexx-exec/src/ir/drive.rs`, inside
`Interp::run_ops`, the two independent statements

```
        let mut frames: Vec<SelectFrame> = Vec::new();
        let mut pc = at;
```

were swapped (the three-line comment moved with `frames`). Neither statement reads the other's
binding; `Vec::new()` allocates nothing and `at` is a `u32` parameter, so neither has a side
effect and the order between them is not observable. It is the second of the two forms the brief
offered.

**The edit does not reach the machine code.** `.text` is byte-identical between A and A', and
between B and B':

```
f6257921fe251315a355701c10862274  A .text
f6257921fe251315a355701c10862274  Ap .text
f6d0516622485ca4a3bf7142bbcf9f07  B .text
f6d0516622485ca4a3bf7142bbcf9f07  Bp .text
```

and so is every other allocated section (`.rodata`, `.data`, `.data.rel.ro`, `.init_array`,
`.got`, `.got.plt`, `.plt`), at identical addresses and sizes. Only the non-allocated `.debug_*`
sections differ, which is why the files differ in md5 and (for B/B') by 8 bytes in size. **A' and
A load the same image; B' and B load the same image.**

This was not the first edit tried. Under this workspace's release profile (`lto = "fat"`,
`codegen-units = 1`) four separate behaviour-preserving perturbations all produced byte-identical
`.text`: swapping two adjacent methods (`when_frame`/`otherwise_frame`) inside `impl Interp`;
swapping `mod clause;` and `mod run;` in `rexx-exec/src/lib.rs`; renaming a private method; and
the statement swap above. Under this profile a trivial source edit is not a layout perturbation.
That has a consequence stated in full under "What the controls do and do not bound".

## Method

* Wall clock, `$EPOCHREALTIME` immediately around the child, staging excluded.
* `ulimit -v 8388608` in every run's subshell.
* `REXX_ENGINE=ir` in every run.
* A fresh empty working directory created per run and removed after it. The 28 correctness-gate
  runs were additionally scanned for files left behind in that directory; none left any, so the
  scratchpad-on-the-search-path hazard is not in play for these programs.
* One discarded warm-up pass (every arm x every program) before round 1.
* Arm order rotated between rounds through eight permutations, so no arm keeps a fixed slot or a
  fixed neighbour; over 15 rounds each arm sat in each of the four slots 3 or 4 times.
* Host-idle gate before each measurement block: six consecutive five-second `/proc/stat` samples
  at or above 90% idle. Passed at 98.4-99.4% idle each time.
* 15 rounds, all four arms, all seven programs -- 420 timed runs. Round-by-round win counts are
  reported, not only medians.

**rexxcps counts.** `count` and `averaging` were fixed at **100 and 100**, which are the file's
own defaults; the copy timed is byte-identical to `/home/moritz/dev/repos/ooRexx/samples/rexxcps.rex`
(sha256 `b86b1232a3747bacdeba64da16eb79cb0e0115bc3d91da3c2b0b08a80772c8f4`). That gives ~3.3 s per
run, and `total > 1` on trial 1 on every arm, so the program's self-calibration never fired. This
was not assumed: every timed run's stdout was checked for the line
`Averaged: 100 x 100 iterations`, and a run without it would have been voided. **None was** --
0 of 420 rows voided, in both timing sittings.

## The programs, and why six of them are controls

`Role::Loop` in `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs` names six: `alloc4c`,
`arith`, `compound`, `emptyloop`, `strings`, `varlookup`. All six were read in full, and then
checked mechanically with block comments stripped. **None contains an `IF`, a `WHEN`, a `SELECT`
or a `CALL` -- zero occurrences of each, in all six.** Their `DO` headers are:

```
alloc4c    do i = 1 to n
arith      do i = 1 to n
compound   do i = 1 to n   and   do k = 0 to tails - 1
emptyloop  do i = 1 to n
strings    do i = 1 to n
varlookup  do i = 1 to n
```

No header holds a call. Two qualifications the brief's claim does not carry, both checked rather
than assumed:

1. `compound.rex`'s second header holds an *expression*, `tails - 1`, which Task 2 does promote.
2. Every header's `1`/`0` and `n` are header values, which Task 2 also promotes.

Both are one-time: a `DO` header's values are evaluated once per header execution, not per
iteration -- `b8e9db0e5` compiles the *slots*, and the per-iteration step and test are not
expression evaluations. Each of these programs enters its loop once, so the plan promotes two
header slots per program (four in `compound`) against 10^6 to 2.5x10^7 iterations. The claim
stands: **the plan cannot make these six measurably faster.** Movement on them is layout.

## Correctness gate

Every program was run on all four arms and stdout, stderr and exit status compared.

| program | stdout | stderr | exit |
|---|---|---|---|
| alloc4c | byte-identical on all four (`12888896`) | empty on all four | 0 |
| arith | byte-identical on all four (`4629643519330627.7808`) | empty on all four | 0 |
| compound | byte-identical on all four (`5000000`) | empty on all four | 0 |
| emptyloop | byte-identical on all four (`done`) | empty on all four | 0 |
| strings | byte-identical on all four (`138000000`) | empty on all four | 0 |
| varlookup | byte-identical on all four (`19000000`) | empty on all four | 0 |
| rexxcps | identical on all four except its two timing figures -- see below | empty on all four | 0 |

**No program diverged. Nothing is voided.**

`rexxcps` prints its own elapsed time and its own clauses-per-second, so byte-identity across arms
is not available for those two lines and would be a defect in the harness if it were. Everything
else is byte-identical:

```
----- REXXCPS 2.2 -- Measuring REXX clauses/second -----
 REXX version is: REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026
       System is: LINUX
        Averaged: 100 x 100 iterations of 1000 clauses (over TIMEs)

     Performance: CPS REXX clauses per second
```

The only two lines the mask touches are the two it names, per arm, and the check is therefore
complete rather than partial: the program's whole remaining output surface is its ten
`say 'FailedN'` guards and `say 'NoValue raised'`, and **none appears on any arm** (grep count 0
in all four captures). The version line is a compile-time constant and is identical across arms.

## A measurement artifact found and removed before the numbers below

The first 15-round sitting ran each arm from its own path -- `bin/rexx-run-A`, `bin/rexx-run-Ap`,
`bin/rexx-run-B`, `bin/rexx-run-Bp`. It produced control spreads of up to **3.25%** with lopsided
win counts (A' beat A 13/15 on rexxcps) **between two byte-identical images**. That is not
possible from the code, so it was chased rather than reported.

Four byte-identical copies of the A binary, under the names `rexx-run-P`, `rexx-run-Q`,
`rexx-run-Pz`, `rexx-run-Qz`, timed interleaved over 8 rounds on rexxcps:

```
rexx-run-P   (10-char basename)  median 3.4866
rexx-run-Q   (10-char basename)  median 3.4407
rexx-run-Pz  (11-char basename)  median 3.3823
rexx-run-Qz  (11-char basename)  median 3.3696

spread across four byte-identical images: 3.47%
P->Pz (10 vs 11 chars) -2.99%      Q->Qz (10 vs 11 chars) -2.06%
P->Q  (both 10 chars)  -1.32%      Pz->Qz (both 11 chars) -0.37%
```

The dominant term is **the length of `argv[0]`**: one character shifts the initial process stack
and moves the whole run by 2-3%. `A` and `Ap` differ by exactly one character, and so do `B` and
`Bp`; `A` and `B` do not, and neither do `A'` and `B'`. So the artifact inflated the two control
spreads and left the effect comparison alone -- the safe direction, but a contaminated control.
The instruction counter confirms the mechanism: A and A' differ by **0.0000%** on
`instructions:u`, so nothing about the executed instruction stream changed; only where it landed
did.

**Fix:** every arm is staged to one fixed path, `runbin/rexx-run`, by a `cp` outside the timed
window, so `argv[0]` and the file path are identical for every run of every arm. Re-running the
same four-way null test under that design cut the identical-image spread from 3.47% to 1.33%.
**Every number in the sections below is from the re-run under the fixed-path design.** The
contaminated sitting is kept at the end as raw data, because it is the evidence for the artifact.

## Wall clock -- 15 rounds, fixed-path staging (the measurement)

Medians of 15 runs per cell, seconds:

```
program           A med      Ap med       B med      Bp med
rexxcps          3.3416      3.3686      3.2425      3.2442
alloc4c          1.1517      1.1721      1.1635      1.1674
arith            2.3095      2.2994      2.2308      2.2391
compound         2.7647      2.7562      2.8550      2.8061
emptyloop        1.5524      1.5470      1.4729      1.4763
strings          3.2844      3.3220      3.4337      3.4334
varlookup        2.5607      2.5569      2.5939      2.6000
```

`d(med)%` is the change between the two medians; `med paired%` is the median of the 15 per-round
paired changes; `y wins` counts the rounds in which the second arm was faster.

```
program    comparison    median x  median y  d(med)% med paired%   y wins
rexxcps    A vs A'         3.3416    3.3686     0.81        0.90     4/15
rexxcps    B vs B'         3.2425    3.2442     0.05        0.18     7/15
rexxcps    A vs B          3.3416    3.2425    -2.97       -3.37    15/15
rexxcps    A' vs B'        3.3686    3.2442    -3.69       -4.18    15/15
alloc4c    A vs A'         1.1517    1.1721     1.77        1.68     5/15
alloc4c    B vs B'         1.1635    1.1674     0.34       -0.32     8/15
alloc4c    A vs B          1.1517    1.1635     1.02        0.23     5/15
alloc4c    A' vs B'        1.1721    1.1674    -0.40       -0.55    10/15
arith      A vs A'         2.3095    2.2994    -0.44       -0.07     8/15
arith      B vs B'         2.2308    2.2391     0.37        0.36     7/15
arith      A vs B          2.3095    2.2308    -3.41       -3.73    14/15
arith      A' vs B'        2.2994    2.2391    -2.62       -2.30    15/15
compound   A vs A'         2.7647    2.7562    -0.31       -0.50    10/15
compound   B vs B'         2.8550    2.8061    -1.71       -2.17    11/15
compound   A vs B          2.7647    2.8550     3.27        3.97     1/15
compound   A' vs B'        2.7562    2.8061     1.81        2.02     4/15
emptyloop  A vs A'         1.5524    1.5470    -0.35       -0.08     8/15
emptyloop  B vs B'         1.4729    1.4763     0.23        0.08     6/15
emptyloop  A vs B          1.5524    1.4729    -5.12       -5.33    15/15
emptyloop  A' vs B'        1.5470    1.4763    -4.57       -4.75    15/15
strings    A vs A'         3.2844    3.3220     1.15        0.35     5/15
strings    B vs B'         3.4337    3.4334    -0.01       -0.85     9/15
strings    A vs B          3.2844    3.4337     4.55        4.40     0/15
strings    A' vs B'        3.3220    3.4334     3.35        3.45     0/15
varlookup  A vs A'         2.5607    2.5569    -0.15       -0.30    10/15
varlookup  B vs B'         2.5939    2.6000     0.24        0.24     4/15
varlookup  A vs B          2.5607    2.5939     1.29        1.36     1/15
varlookup  A' vs B'        2.5569    2.6000     1.69        1.79     2/15
```

### The accept rule as stated

> Report no effect as real unless it exceeds both control spreads.

```
program      A-vs-B %  ctrl A/A'%  ctrl B/B'%   verdict
rexxcps         -2.97       0.81        0.05    exceeds both control spreads
alloc4c          1.02       1.77        0.34    INSIDE a control spread -- not real
arith           -3.41      -0.44        0.37    exceeds both control spreads
compound         3.27      -0.31       -1.71    exceeds both control spreads
emptyloop       -5.12      -0.35        0.23    exceeds both control spreads
strings          4.55       1.15       -0.01    exceeds both control spreads
varlookup        1.29      -0.15        0.24    exceeds both control spreads
```

By the rule as stated, six of the seven axes carry a real A-vs-B wall-clock difference, and
`alloc4c` does not.

### What the controls do and do not bound

The A/A' and B/B' controls are pairs of **byte-identical loaded images**. They therefore bound
run-to-run and harness noise honestly -- 0.01% to 1.77%, with win counts between 4/15 and 10/15,
which is what two copies of one program should look like. What they cannot bound is **code
layout**, because the trivial edit did not change the code. Under this release profile no trivial
edit does; four were tried.

The second kind of control the brief specifies is what closes that gap, and it is the binding
one. Five of the six `Role::Loop` programs -- which contain no `IF`, no `WHEN`, no call in a
header, and which the plan therefore cannot speed up -- moved past their own control spreads, in
**both directions**:

```
emptyloop  -5.12%  (B faster, 15/15)
arith      -3.41%  (B faster, 14/15)
varlookup  +1.29%  (B slower,  1/15)
compound   +3.27%  (B slower,  1/15)
strings    +4.55%  (B slower,  0/15)
```

That is a layout swing of **-5.12% to +4.55%** on axes where no promoted work exists. It is the
same phenomenon `echo-op-price.md` recorded on `arith`, where its two identical-behaviour arms
spanned 9.04% and head beat the do-nothing control 20/20. Here it is **larger than the effect on
rexxcps**. On the wall clock alone, rexxcps's -2.97% is not
distinguishable from the layout movement this build produced on programs it cannot have touched.

## Instructions (`perf stat -e instructions:u`, one run per arm)

```
program                   A               B   A->B %   A->A' %   B->B' %
rexxcps      29,605,462,015  28,594,977,579    -3.41    0.0000   -0.0225
alloc4c       9,342,657,574   9,392,748,982     0.54    0.0008   -0.0008
arith        20,349,640,032  20,397,974,269     0.24    0.0000   -0.0000
compound     37,064,852,737  37,163,438,975     0.27   -0.0024    0.0049
emptyloop    27,525,609,300  27,100,610,015    -1.54   -0.0000   -0.0000
strings      44,565,576,116  44,787,576,200     0.50    0.0000    0.0000
varlookup    44,384,636,468  44,574,636,225     0.43   -0.0000    0.0000
```

Reported **beside** the wall clock, not instead of it.

The A/A' and B/B' columns are the instrument's own noise: **0.0000% to 0.0049%**, four orders of
magnitude below the effect. (rexxcps's -0.0225% is the largest and has a cause: the program
formats its own elapsed times, so digit strings of different lengths do slightly different
arithmetic.) The instrument is exact enough to read the op stream directly.

Two things it says:

1. **rexxcps loses 1.010 billion user instructions, -3.41%.** No other axis comes within a factor
   of two of that, and the arm-internal jitter is 0.02%. This is the promoted work: the only
   program measured that holds `IF`s, `WHEN`s and per-execution header values.
2. **The six control axes move -1.54% to +0.54%.** This is not zero, and it should not be read as
   "the op stream changed" for them: they hold no promotable construct. It is codegen drift in the
   shared driver -- `run_ops` grew arms and the surrounding inlining moved with it -- which
   `instructions:u` does see. So this instrument is layout-blind but not codegen-blind, and
   -1.54% (emptyloop) is the floor any instruction claim about this plan must clear. rexxcps's
   -3.41% clears it by 2.2x.

## rexxcps's own clauses-per-second figure

Usable across arms **here**, for a reason that has to be stated rather than assumed: `count` and
`averaging` are pinned at 100 and 100 and were verified from every run's own `Averaged:` line, so
the figure is a wall-clock throughput over one fixed workload rather than over a self-adjusted
one. It is not the same quantity as whole-process wall clock -- it excludes process start, parse
and the calibration loop, and it subtracts the measured empty-loop time from its denominator,
which is exactly the shape that flatters a ratio.

Measured on the **same runs**, so the two instruments are not from two sittings (10 rounds,
fixed-path staging):

```
arm   wall med     cps med
A       3.3804   2,963,192
Ap      3.3558   2,985,008
B       3.2302   3,101,346
Bp      3.2480   3,084,576

wall A->B  -4.44%      cps A->B  +4.66%      B faster on both, 10/10 rounds each
wall A->A' -0.73%      cps A->A' +0.74%
wall B->B' +0.55%      cps B->B' -0.54%
```

The two agree to about 0.2 points, so on this program the subtraction is not doing meaningful
inflation at these magnitudes. The figure is reported as a cross-check, not as the measurement.

**But note the sitting-to-sitting spread it exposes.** Three interleaved sittings on rexxcps:

| sitting | design | rounds | wall A->B | rounds B won |
|---|---|---|---|---|
| 1 | per-arm paths | 15 | -6.48% | 15/15 |
| 2 | fixed-path staging | 15 | -2.97% | 15/15 |
| 3 | fixed-path staging | 10 | -4.44% | 10/10 |

**Direction unanimous in all 40 rounds. Magnitude between 3.0% and 6.5%.** The magnitude is not
pinned by this host at this resolution.

## Verdict

**The plan's aim -- a faster rexxcps -- is supported, and the support comes from the instruction
counter rather than from the wall clock.**

* By the rule as stated, rexxcps's -2.97% exceeds both control spreads (0.81% and 0.05%) and is
  real. So do five other axes, which is the tell that the rule's controls are not the binding
  constraint here.
* The wall clock cannot attribute the rexxcps gain: the same head build moved `emptyloop` by
  -5.12% and `strings` by +4.55%, unanimously, on programs holding no `IF`, no `WHEN` and no call
  in a header. A -3% to -6% wall move on rexxcps sits inside that layout swing.
* The instruction counter can: -3.41% on rexxcps against -1.54% to +0.54% on the six control axes,
  with 0.005% arm-internal noise. That is the op stream, it is specific to the one program that
  holds the promoted constructs, and it is what the plan set out to remove.

Two findings that are not the plan's aim and should not be lost:

* **The head build is slower than base on three control axes** -- `strings` +4.55% (0/15),
  `compound` +3.27% (1/15), `varlookup` +1.29% (1/15) -- and faster on two, `emptyloop` -5.12%
  (15/15) and `arith` -3.41% (14/15). None of it can be promoted work. It is layout, and it is
  not free: whatever the plan bought on rexxcps, it also reshuffled every other axis by several
  percent in whichever direction the linker chose.
* **Instructions rose slightly on four control axes** (+0.24% to +0.54%) and fell 1.54% on
  `emptyloop`. The shared driver's codegen moved. Small, but it is the cost side of the ledger
  and it is measured rather than assumed.

## Raw data

Every run is in the session scratchpad and is reproduced below.

### Wall clock, fixed-path staging, 15 rounds (seconds)
-- rexxcps
     round          A         Ap          B         Bp
         1     3.3106     3.3404     3.1958     3.2243
         2     3.4138     3.3313     3.2881     3.1921
         3     3.3644     3.4656     3.2155     3.3369
         4     3.3416     3.3683     3.2292     3.2252
         5     3.4082     3.3280     3.2681     3.1643
         6     3.2931     3.3686     3.2531     3.1702
         7     3.3412     3.4232     3.2967     3.3025
         8     3.3274     3.4168     3.2354     3.2442
         9     3.3552     3.3502     3.2070     3.3030
        10     3.3446     3.4616     3.2941     3.2453
        11     3.3624     3.3719     3.1932     3.2885
        12     3.3347     3.3917     3.2841     3.3087
        13     3.3282     3.3508     3.2425     3.2663
        14     3.3375     3.3857     3.2255     3.1911
        15     3.4404     3.3517     3.2837     3.2079
-- alloc4c
     round          A         Ap          B         Bp
         1     1.2085     1.1739     1.2045     1.1907
         2     1.1865     1.1562     1.1424     1.1621
         3     1.1580     1.1774     1.1602     1.1565
         4     1.1713     1.1858     1.1732     1.1743
         5     1.1812     1.1721     1.1504     1.1611
         6     1.1479     1.1631     1.1287     1.2294
         7     1.1608     1.1445     1.1635     1.1885
         8     1.1383     1.1709     1.1634     1.1534
         9     1.1359     1.1761     1.1943     1.1708
        10     1.1302     1.1512     1.1617     1.1449
        11     1.1517     1.1721     1.1672     1.1683
        12     1.1753     1.1696     1.1525     1.1604
        13     1.1493     1.1714     1.1872     1.1743
        14     1.1296     1.1729     1.1771     1.1524
        15     1.1393     1.1806     1.1986     1.1674
-- arith
     round          A         Ap          B         Bp
         1     2.3303     2.3045     2.2342     2.2463
         2     2.3368     2.2575     2.2322     2.2403
         3     2.2590     2.3113     2.2182     2.2747
         4     2.3305     2.2724     2.2005     2.2685
         5     2.3006     2.3273     2.2110     2.2052
         6     2.3437     2.3268     2.1932     2.2690
         7     2.2618     2.2909     2.2964     2.2382
         8     2.2597     2.2981     2.2200     2.2624
         9     2.3049     2.2854     2.2369     2.1865
        10     2.3469     2.2830     2.2376     2.2327
        11     2.3136     2.3540     2.2778     2.2391
        12     2.2848     2.3188     2.2308     2.1812
        13     2.2656     2.2718     2.1532     2.2246
        14     2.3193     2.2994     2.2631     2.2260
        15     2.3095     2.3079     2.2234     2.2706
-- compound
     round          A         Ap          B         Bp
         1     2.8566     2.7684     3.1687     2.7806
         2     2.7465     2.9111     2.9354     2.8498
         3     2.7647     2.7290     2.8118     2.7808
         4     2.7461     2.8194     2.8550     2.7908
         5     2.8058     2.8166     2.8164     2.7746
         6     2.8031     2.7891     2.8051     2.7703
         7     2.7423     2.7322     2.7970     2.8110
         8     2.7792     2.7118     3.1453     3.0770
         9     2.7422     2.7388     3.0355     2.8061
        10     2.7283     2.7392     2.7910     2.8555
        11     2.7803     2.7337     2.7896     2.8841
        12     2.7595     2.7063     2.9770     2.7933
        13     2.8755     2.7640     3.1752     2.8199
        14     3.0159     2.7562     2.7875     3.2925
        15     2.7472     2.7725     2.9693     2.7823
-- emptyloop
     round          A         Ap          B         Bp
         1     1.5989     1.5465     1.4899     1.4763
         2     1.5450     1.5671     1.4745     1.4996
         3     1.6030     1.5444     1.4653     1.4857
         4     1.5524     1.5550     1.4676     1.4856
         5     1.5556     1.5579     1.4792     1.4800
         6     1.5415     1.5515     1.4670     1.4681
         7     1.5412     1.5429     1.4751     1.4662
         8     1.5921     1.7543     1.4808     1.4868
         9     1.5558     1.5439     1.4729     1.4689
        10     1.5887     1.5446     1.5052     1.4713
        11     1.5485     1.5408     1.4655     1.4733
        12     1.5499     1.5413     1.4639     1.4824
        13     1.5635     1.5471     1.4732     1.4783
        14     1.5445     1.5470     1.4699     1.4633
        15     1.5504     1.5491     1.4699     1.4695
-- strings
     round          A         Ap          B         Bp
         1     3.2794     3.3314     3.4236     3.3894
         2     3.3190     3.3220     3.4337     3.3947
         3     3.2653     3.3108     3.4309     3.4135
         4     3.2844     3.2600     3.3839     3.5244
         5     3.2737     3.3616     3.5716     3.3767
         6     3.2639     3.2987     3.6071     3.4123
         7     3.2894     3.2540     3.4122     3.5352
         8     3.2708     3.2709     3.3564     3.4957
         9     3.3151     3.3448     3.4923     3.4334
        10     3.2964     3.2821     3.4300     3.6698
        11     3.2761     3.2875     3.4717     3.4011
        12     3.2624     3.3698     3.6558     3.5704
        13     3.3550     3.3390     3.4875     3.5136
        14     3.3677     3.3610     3.4144     3.7810
        15     3.3019     3.3296     3.4553     3.4260
-- varlookup
     round          A         Ap          B         Bp
         1     2.5439     2.5660     2.6009     2.6054
         2     2.5643     2.5567     2.5900     2.6498
         3     2.5567     2.5465     2.6453     2.5973
         4     2.5895     2.5641     2.5922     2.5869
         5     2.5596     2.5488     2.5943     2.6048
         6     2.5476     2.6098     2.6002     2.5993
         7     2.5499     2.5569     2.5963     2.6027
         8     2.5999     2.5639     2.5918     2.5963
         9     2.5564     2.5506     2.5939     2.5968
        10     2.5688     2.5620     2.5938     2.6000
        11     2.5610     2.6069     2.6088     2.5926
        12     2.5476     2.5559     2.5934     2.6049
        13     2.5607     2.5498     2.5845     2.5969
        14     2.5831     2.5575     2.5928     2.6003
        15     2.5722     2.5527     2.5981     2.6138

### Wall clock, contaminated per-arm-path sitting, 15 rounds (seconds)

Kept because it is the evidence for the `argv[0]` artifact, not because it measures the change.
-- rexxcps
     round          A         Ap          B         Bp
         1     3.4430     3.3826     3.2286     3.2057
         2     3.4311     3.3892     3.2341     3.1956
         3     3.4316     3.3795     3.2217     3.1997
         4     3.4758     3.4124     3.2602     3.3029
         5     3.4688     3.4251     3.2560     3.2048
         6     3.4390     3.4243     3.2200     3.2013
         7     3.4629     3.3687     3.2486     3.2075
         8     3.5925     3.4312     3.2631     3.2744
         9     3.4504     3.5060     3.3199     3.3128
        10     3.5614     3.3977     3.2318     3.3096
        11     3.5651     3.5063     3.2546     3.2238
        12     3.5104     3.5170     3.2748     3.3575
        13     3.4583     3.3846     3.2218     3.2008
        14     3.4200     3.3461     3.1916     3.2298
        15     3.4308     3.3703     3.2063     3.1933
-- alloc4c
     round          A         Ap          B         Bp
         1     1.2091     1.1623     1.1814     1.1748
         2     1.1826     1.1689     1.2285     1.1994
         3     1.2339     1.2067     1.2037     1.2531
         4     1.2877     1.9060     2.2285     1.2993
         5     1.2179     1.3204     1.2655     1.2824
         6     1.3071     1.2280     1.2141     1.2917
         7     1.2392     1.1619     1.2679     1.1647
         8     1.2773     1.3001     1.2608     1.3015
         9     1.3463     1.3210     1.3412     1.3539
        10     1.3272     1.2946     1.3425     1.3818
        11     1.4022     1.3555     1.4157     1.3720
        12     1.2713     1.2742     1.3001     1.2831
        13     1.1932     1.1900     1.2423     1.2320
        14     1.2179     1.1454     1.1696     1.1527
        15     1.1811     1.1572     1.1752     1.1518
-- arith
     round          A         Ap          B         Bp
         1     2.3043     2.3039     2.2043     2.2467
         2     2.3031     2.3264     2.2005     2.2495
         3     2.3107     2.3059     2.2167     2.2984
         4     2.9983     3.1193     3.1048     2.9983
         5     2.3249     2.3174     2.2771     2.2525
         6     2.3163     2.3219     2.2018     2.2565
         7     2.3289     2.3098     2.2227     2.2554
         8     2.3157     2.3238     2.2121     2.2642
         9     2.3073     2.3530     2.2205     2.2569
        10     2.3217     2.3098     2.2556     2.3418
        11     2.3163     2.3202     2.2437     2.2964
        12     2.3178     2.3289     2.2105     2.2594
        13     2.3159     2.3211     2.2178     2.2677
        14     2.3029     2.3051     2.2026     2.2421
        15     2.3008     2.3170     2.2096     2.2354
-- compound
     round          A         Ap          B         Bp
         1     2.7357     2.8350     2.8094     2.7762
         2     2.7280     2.8289     2.8114     2.7600
         3     2.7319     2.8560     2.8058     2.7807
         4     4.3608     4.3506     3.1877     4.5311
         5     2.7621     2.8394     2.8139     2.7559
         6     2.7504     2.8643     2.8310     2.7909
         7     2.7571     2.8822     2.8071     2.7900
         8     2.7321     2.8279     2.8014     2.8044
         9     2.8855     2.8429     2.8168     2.7638
        10     2.7807     2.8374     2.8160     2.7734
        11     2.7742     2.8843     2.8805     2.8364
        12     2.7423     2.8643     3.1710     2.8087
        13     2.7534     2.8393     2.8192     2.7705
        14     2.7549     2.9025     2.8077     2.7621
        15     2.7447     2.8213     2.8375     2.7585
-- emptyloop
     round          A         Ap          B         Bp
         1     1.5464     1.5526     1.4900     1.4670
         2     1.5418     1.5492     1.4743     1.4764
         3     1.5664     1.5697     1.4804     1.4870
         4     1.5463     1.6047     1.4818     1.4832
         5     1.5355     1.5472     1.4734     1.4769
         6     1.5485     1.5553     1.4804     1.4871
         7     1.5416     1.5881     1.4686     1.4986
         8     1.5741     1.5388     1.4751     1.4743
         9     1.5511     1.5574     1.4729     1.4756
        10     1.5534     1.5532     1.4743     1.4809
        11     1.5716     1.6283     1.5321     1.4910
        12     2.2204     1.6189     1.6612     2.3085
        13     1.5516     1.5483     1.5019     1.4906
        14     1.5535     1.5475     1.4922     1.4689
        15     1.5495     1.5911     1.4965     1.4665
-- strings
     round          A         Ap          B         Bp
         1     3.3836     3.3001     3.4869     3.4081
         2     3.3690     3.3333     3.4823     3.4271
         3     3.4130     3.3218     3.5068     3.4348
         4     3.4030     3.2827     3.5198     3.4659
         5     3.3364     3.3393     3.5136     3.4112
         6     3.3631     3.3029     3.4955     3.4075
         7     3.3725     3.2886     3.5225     3.4533
         8     3.3577     3.3437     3.4587     3.4955
         9     3.4231     3.3610     3.5471     3.4861
        10     3.4285     3.3156     3.5789     3.7066
        11     3.3742     3.3680     3.5681     3.4701
        12     3.4633     3.3547     3.5213     3.8097
        13     3.3343     3.2793     3.4662     3.4082
        14     3.3411     3.2643     3.4460     3.4219
        15     3.3453     3.3207     3.5508     3.4492
-- varlookup
     round          A         Ap          B         Bp
         1     2.5769     2.5679     2.6109     2.6002
         2     2.5660     2.5993     2.6257     2.6010
         3     2.5854     2.5561     2.6311     2.6163
         4     2.5846     2.5588     2.6253     2.6219
         5     2.5955     2.5683     2.6054     2.5999
         6     2.5653     2.5610     2.6276     2.6058
         7     2.6876     2.5707     2.7572     2.6305
         8     2.6736     2.5648     2.6512     2.6695
         9     2.5971     2.5479     2.6095     2.6037
        10     2.6428     2.6115     2.6514     2.7027
        11     2.5622     2.6104     2.6425     2.6179
        12     2.5960     2.5866     2.5963     2.6051
        13     2.5539     2.5547     2.6012     2.5936
        14     2.5685     2.5567     2.6106     2.5974
        15     2.5657     2.5712     2.6080     2.6002

### Wall clock and rexxcps's own cps, same runs, 10 rounds

```
  r 1  A=3.3725/2,970,074  Ap=3.3361/3,002,308  B=3.2553/3,076,865  Bp=3.2953/3,040,792
  r 2  A=3.3801/2,963,514  Ap=3.3553/2,985,030  B=3.2211/3,109,950  Bp=3.3084/3,027,736
  r 3  A=3.4456/2,907,924  Ap=3.3823/2,961,652  B=3.2169/3,114,344  Bp=3.1682/3,161,682
  r 4  A=3.3067/3,029,292  Ap=3.3563/2,984,986  B=3.2444/3,087,360  Bp=3.2494/3,083,155
  r 5  A=3.3394/2,999,554  Ap=3.3799/2,964,100  B=3.2005/3,130,668  Bp=3.2231/3,108,711
  r 6  A=3.3808/2,962,871  Ap=3.3523/2,987,865  B=3.2424/3,089,610  Bp=3.2597/3,073,362
  r 7  A=3.3456/2,994,134  Ap=3.3286/3,009,401  B=3.1943/3,136,212  Bp=3.2515/3,081,178
  r 8  A=3.3958/2,950,149  Ap=3.3642/2,977,606  B=3.2149/3,116,298  Bp=3.2203/3,110,276
  r 9  A=3.4535/2,900,092  Ap=3.3701/2,972,723  B=3.2393/3,092,743  Bp=3.2238/3,107,808
  r10  A=3.4535/2,900,563  Ap=3.3489/2,991,654  B=3.2816/3,052,952  Bp=3.2466/3,085,997
```

### Four byte-identical copies of the A image, separate paths, 8 rounds (seconds)

```
rexx-run-P   3.480 3.482 3.500 3.469 3.501 3.497 3.485 3.489   median 3.4866
rexx-run-Q   3.445 3.487 3.428 3.425 3.445 3.428 3.438 3.443   median 3.4407
rexx-run-Pz  3.346 3.383 3.393 3.349 3.382 3.391 3.387 3.361   median 3.3823
rexx-run-Qz  3.420 3.357 3.367 3.353 3.389 3.376 3.372 3.355   median 3.3696
```

### The same four copies, staged to one fixed path, 8 rounds (seconds)

```
P    3.346 3.442 3.336 3.419 3.413 3.379 3.324 3.349   median 3.3640
Q    3.332 3.436 3.342 3.364 3.428 3.417 3.375 3.374   median 3.3748
Pz   3.269 3.430 3.380 3.420 3.421 3.385 3.356 3.413   median 3.3987
Qz   3.322 3.419 3.403 3.417 3.415 3.336 3.363 3.417   median 3.4087

spread 1.33%, against 3.47% for the same four under separate paths
```
