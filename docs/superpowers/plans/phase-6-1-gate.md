# Phase 6.1 gate record

Performance base: `e6af1198b` (the commit Task 1 starts from). Budget (spec section 5): a running
total of at most +0.5% beyond the noise band on every program; wall clock within ±4%.

## Task 1

Commits: `66b0f6854` (layout), `16b67c6cc` (behaviour), `12239c974` (perf round 1). `S` is
`/tmp/claude-1000/p61/t1`; each binary from `git archive <sha> rust interpreter`, built with
`CARGO_TARGET_DIR=$S/target-NAME memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`,
one `Compiling rexx-exec` line each.

| binary | source | sha256 |
|---|---|---|
| base | `e6af1198b` | `2ea19b3ea2875fada8718eaf5b89f828d5ae07587a74feaf9b34681c879c891e` |
| pad1 | base + `layout-pad.py 8` | `f7c3a9d41e0646fc1931281a32aaf002b7a3f762a87386349508c70fe61a5aa1` |
| pad2 | base + `layout-pad.py 48` | `c03d30015583f749cf4344438901f65af16dc34746a67ff6d9c7d9e2549cfac0` |
| pad3 | base + `layout-pad.py 192` | `91653253d9f2c1c7ab8ca690b17460dca4ad4615a3a284401ebec17195f6e6bb` |
| t1 | `16b67c6cc` | `6a31c6fb78068741f10d9ded837051f4c2334cbba04b4b2cb4285f835990a78b` |
| r1 | `12239c974` | `b44aaf39eeacc09b6fe1bcacfb8235d4fb0d3db02ac51e5c000e855c18d8ec3c` |

Noise band, the layout controls against base:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $S/cg-pad -p "fibcall fibfunc dispatch dispatchclass sendloop" base=$S/bin/base/rexx-run pad1=$S/bin/pad1/rexx-run pad2=$S/bin/pad2/rexx-run pad3=$S/bin/pad3/rexx-run
```

Exit 0. Every control delta on every program prints as `0.0000`% (largest absolute difference 136
instructions, `sendloop` pad3), so the band is 0 and the budget is +0.5%.

Instructions:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 10 -o $S/cg-final -p "fibcall fibfunc dispatch dispatchclass sendloop" base=$S/bin/base/rexx-run t1=$S/bin/head/rexx-run r1=$S/bin/r1c/rexx-run
```

Exit 0, every spread 0.0000%.

| program | t1 % | r1 % | verdict |
|---|---:|---:|---|
| fibcall | +0.6131 | +0.2044 | inside |
| fibfunc | +0.5642 | +0.2717 | inside |
| dispatch | +0.7929 | +0.0961 | inside |
| dispatchclass | +0.8357 | +0.1013 | inside |
| sendloop | +1.1774 | +0.1427 | inside |

t1 was over on every program: `pop_activation`, inlined into its callers at base, went out of line
(`cgdiff.py` on `sendloop`: `finish_send` -245,000,269, `pop_activation` +365,005,475). Round 1
moved the termination restore to a cold helper, forced `pop_activation` inline and kept
`TraceCache` at its base shape.

Wall clock:

```
PROGRAMS="fibcall fibfunc dispatch dispatchclass sendloop" bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall-final base=$S/bin/base/rexx-run pad2=$S/bin/pad2/rexx-run r1=$S/bin/r1c/rexx-run
```

Exit 0; load average 2.59 at start, 1.47 at end.

| program | pad2 % | r1 % |
|---|---:|---:|
| fibcall | +0.13 | -0.13 |
| fibfunc | +0.51 | +0.13 |
| dispatch | +0.63 | -3.16 |
| dispatchclass | +1.71 | -2.99 |
| sendloop | +0.51 | -2.40 |

Every program inside ±4%.

## Task 2

Commits: `5b7acef35` (behaviour), `9777db504` (perf round 1), `fe765db7b` (witness), `4d46b76b0` (whole-group lines). `S` is
`/tmp/claude-1000/p61/t2`; base and pads are Task 1's binaries (sha256 above, re-checked). Each
Task 2 binary from `git archive <sha> rust interpreter`, built with
`CARGO_TARGET_DIR=$S/target-NAME memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`,
one `Compiling rexx-exec` line each.

| binary | source | sha256 |
|---|---|---|
| t2 | `5b7acef35` | `b21c6a8e7f2e201312009c1f6be339db75f1d8ab918f9bb5b4c23bdbfd18e1a3` |
| r1 | `9777db504` | `0d08596f68c918280afca14c1d019b6ca00618ee1884b651bd8ab606b6d93d5d` |

Instructions:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $S/cg-final -p "emptyloop decloop rexxcps" base=$B/base/rexx-run pad1=$B/pad1/rexx-run pad2=$B/pad2/rexx-run pad3=$B/pad3/rexx-run t2=$S/bin/t2/rexx-run r1=$S/bin/r1/rexx-run
```

(`B` is `/tmp/claude-1000/p61/t1/bin`.) Exit 0. The layout controls print `+0.0000`% on every
program, so the band is 0 and the budget is +0.5%.

| program | t2 % | r1 % | verdict |
|---|---:|---:|---|
| emptyloop | +18.6211 | -0.3229 | inside |
| decloop | +3.1105 | +0.4373 | inside |
| rexxcps | +0.4163 | +0.2741 | inside |

t2 was over: the fast flat header's closure stopped being inlined into `ops_loop_steady`
(`cgdiff.py` on `emptyloop`: `flat_loop_header::{closure#0}` +875,001,750 as a symbol of its own,
`ops_loop_steady` +624,854,607). Round 1 spells the clause out (`enter_clause`, the advance,
`leave_clause`), keeps WHILE/UNTIL/COUNTER as bits of one `PassTest` byte, and moves the driver's
`LoopHeaderValue` work into one filing call.

Wall clock:

```
PROGRAMS="emptyloop decloop rexxcps" bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall base=$B/base/rexx-run pad2=$B/pad2/rexx-run r1=$S/bin/r1/rexx-run
```

Exit 0; load average 0.16 at start, 0.57 at end.

| program | pad2 % | r1 % |
|---|---:|---:|
| emptyloop | -0.23 | +4.16 |
| decloop | +0.84 | -0.84 |
| rexxcps | -2.03 | +0.81 |

**`emptyloop` is outside ±4% on wall clock.** Two more emptyloop-only runs of the same command
gave +4.18 and +4.42. `perf stat -e cycles`, three runs each: base 1.262-1.288 G, Task 1's `r1c`
1.281-1.291 G, Task 2's r1 1.316-1.348 G; branch misses equal (370,386 against 384,551). The cost is
cycles at fewer instructions, and `perf annotate` puts it in `loop_advance` (20.3% of cycles at base,
37.4% at r1, where `cgdiff.py` gives it 49,999,953 fewer instructions). Two further rounds moved it the wrong way:
`DO WITH`'s state out of `LoopState` (+6.5% cycles) and the counter boxed in `FlatLoop` (+10%).
Neither landed. Three rounds are spent; this stops for Moritz's ruling.
