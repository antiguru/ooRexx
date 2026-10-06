# Task 26 Step 1, fixed-commit half (perfbase)

Evidence: `.superpowers/sdd/2026-10-01-phase-6-s2-s5/s5-evidence/perfbase/` (`cg-table.txt` is the
full table, `summary.tsv`/`binaries.txt` are callgrind.sh's own outputs, `build.sh` and `cg.sh`
are the exact scripts run, build logs, `hashes.txt`, `script-hashes.txt`, `program-hashes.txt`).
Measured 2026-10-06 with the working tree at `811610f63`; scripts and bench programs read from that tree.
Wall clock not done (waits for HEAD and a quiet machine; load was about 29 during these runs, which
does not affect callgrind counts).

## Builds

Base `1754a3b5a`, S1 close `1a81353e3`, controls = base + `layout-pad.py` (8, 48, 192 pads).
Each from `git archive REV | tar -x` into its own tree, `find TREE -type f -exec touch {} +`, own
`CARGO_TARGET_DIR`, `memcap 8G cargo build --release -p rexx-exec --bin rexx-run`
(`build.sh`). Every build log has exactly one `Compiling rexx-exec` line (`progress.txt`).

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `dbce1a524b030eed574a74711d213bf37f37ef837428cedc657bfcac09779981` | 2,732,523 |
| s1 | `d851f268744533bf704ccce2479973945c30b99aa78365b4a036120a44bb9e64` | 2,833,355 |
| pad1 | `d03c08a09fd9bd24253d2c89d067e67f533095748c226fbacb4120f7e65d4e41` | 2,733,691 |
| pad2 | `a9b12e91f24be2f372ffdd28b67e22a38cbe7fec5135aec882df800f6ac8e96f` | 2,740,075 |
| pad3 | `c772227271f91817f76ba2709c4e82244741a6aa0eb1ae510b304e331bb90d8d` | 2,763,115 |

Base/pad `.text` sizes equal the Task 2 record. The sha256s differ from the earlier records (paths
differ per scratch directory); only the `.text` sizes are comparable across records.

## Instruction counts

Command (`cg.sh`), exit 0, empty stderr apart from the outputs line, no SPREAD flag:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 16 -o $S/cg base=$S/bin/base/rexx-run s1=$S/bin/s1/rexx-run pad1=$S/bin/pad1/rexx-run pad2=$S/bin/pad2/rexx-run pad3=$S/bin/pad3/rexx-run
```

Median of 3 rounds, libc.so.6 and ld-linux subtracted. Every spread 0.0000% except pad3 rexxcps
0.0001%.

| program | base Ir | s1 Ir | s1 vs base % | pad1-base | pad2-base | pad3-base |
|---|---:|---:|---:|---:|---:|---:|
| alloc | 25168677522 | 20270017677 | -19.4633 | +0 | +0 | +0 |
| alloc4c | 3224394091 | 3176497924 | -1.4854 | +0 | +0 | +0 |
| arith | 11520059218 | 11498164875 | -0.1901 | +0 | +0 | +0 |
| assign | 19637556640 | 18840414548 | -4.0593 | +0 | +0 | +0 |
| compound | 9271610392 | 9187028618 | -0.9123 | +0 | +0 | +0 |
| decloop | 2565174757 | 2536048161 | -1.1355 | +0 | +0 | +0 |
| decrender | 4348395361 | 4276914082 | -1.6439 | +0 | +0 | +0 |
| dispatch | 20483284029 | 20493849276 | +0.0516 | +0 | +0 | +0 |
| dispatchclass | 15850194760 | 15490531810 | -2.2691 | +0 | +0 | +0 |
| emptyloop | 9308097479 | 9259509123 | -0.5220 | +0 | +0 | +0 |
| extcall | 8281258570 | 8230424306 | -0.6138 | +0 | +0 | +0 |
| fibcall | 8345685467 | 8324735269 | -0.2510 | +0 | +0 | +0 |
| fibfunc | 7988910813 | 8138966437 | +1.8783 | +0 | +0 | +0 |
| heapshape | 3278633276 | 2373954723 | -27.5932 | +0 | +0 | +0 |
| nop | 9437299220 | 9240155942 | -2.0890 | +0 | +0 | +0 |
| parse | 1541985785 | 1534027263 | -0.5161 | +0 | +0 | +0 |
| sayloop | 114812418 | 114713842 | -0.0859 | +0 | +0 | +0 |
| sendloop | 13863189759 | 13703469981 | -1.1521 | +0 | +0 | +0 |
| startup | 58073107 | 58068980 | -0.0071 | +0 | +0 | +0 |
| strings | 17749286532 | 17503788362 | -1.3831 | +0 | +0 | +0 |
| textnum | 1176578529 | 1152442703 | -2.0514 | +0 | +0 | +0 |
| varlookup | 14878110853 | 14461720794 | -2.7987 | +0 | +0 | +0 |
| rexxcps | 17817350141 | 17663738442 | -0.8621 | +1777 | +2558 | -3388 |

Noise band (largest control difference from base): 0 Ir for every program except `rexxcps`
(pad1 +1,777, pad2 +2,558, pad3 -3,388; band 3,388 Ir, 0.00002%). `s1` reproduces the Task 11
percentages (e.g. `fibfunc` +1.8783%, `heapshape` -27.5932%).

## Budget (Global Constraints, `docs/superpowers/plans/2026-10-01-phase-6-s2-s5.md`, quoted)

> running totals against the Phase 6 base `1754a3b5a` at most +1.0% beyond the noise band;
> `fibfunc`'s bar is its accepted S1 figure (+1.88%, P22) plus 0.7%. Wall clock is measured and
> recorded; no layout iteration and no build-level layout control (function ordering, PGO) (P19).
> Over budget: at most three rounds, then stop for Moritz's ruling (P16).

So the bar is +1.0% for every program but `fibfunc` (+1.88% + 0.7% = +2.58%), each plus the band
above (zero except `rexxcps`).

## HEAD measurement: exact commands

1. Tree: `git archive HEAD | tar -x -C \$S/trees/head`, `find \$S/trees/head -type f -exec touch {} +`,
   `cd \$S/trees/head/rust && CARGO_TARGET_DIR=\$S/tgt/head memcap 8G cargo build --release -p rexx-exec --bin rexx-run`,
   require one `Compiling rexx-exec` line, copy to `\$S/bin/head/rexx-run`, record sha256 and
   `objcopy -O binary --only-section=.text \$S/bin/head/rexx-run /dev/stdout | wc -c`.
2. Callgrind, same flags and script (callgrind.sh sha256 `5fd181ce...343d`, last changed in
   `44c465281`; bench programs hashed in `program-hashes.txt`; if either changed by HEAD, re-run base
   with the HEAD version rather than compare):
   `bash rust/bench-programs/callgrind.sh -r 3 -j 16 -o \$S/cg2 base=\$S/bin/base/rexx-run s1=\$S/bin/s1/rexx-run head=\$S/bin/head/rexx-run`
   Reusing the binaries above needs them kept; they are in `/tmp/claude-1000/p6-s5-perfbase/bin/`
   until deleted (they are deleted at the end of this task, so rebuild base and s1 from the build.sh
   recipe, or run base and s1 again alongside head; counts are deterministic, so the table above
   serves as the check: base and s1 must reproduce these Ir exactly).
3. Delta against S1 close per program: `head - s1` from the same table.
4. Named risks: Task 2's pin depth is in every build (it is in `head` and not in `base`, so it is
   inside the running total); Task 12's variable barrier is likewise inside `head`. Neither can be
   separated without extra builds; attribute only by program shape (variable-heavy: `assign`, `compound`,
   `varlookup`, `alloc`).

## Open questions for the lead

- Binaries deleted with the scratch; say if you want them kept for the HEAD run.
