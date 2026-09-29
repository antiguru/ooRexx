# Phase 6 performance record

Base `9d863ccc5`, three layout controls and `0d911ab7c`, measured 2026-09-29.

## Base

rustc 1.98.1, valgrind-3.27.1, `nproc` 32. Programs and `callgrind.sh` as at `f0e3e727e`.
`REXX_LIB_DIR` default (`/home/moritz/dev/repos/ooRexx/build/lib`) on every run, oracle included.

### Builds

`S` is the task scratch directory. Each build log has one `Compiling rexx-exec` line.

```
git archive 9d863ccc5 | tar -x -C $S/trees/9d863ccc5
find $S/trees/9d863ccc5 -type f -exec touch {} +
cd $S/trees/9d863ccc5/rust && CARGO_TARGET_DIR=$S/tgt/base cargo build --release -p rexx-exec --bin rexx-run
```

Controls: the same archive, then `rust/bench-programs/layout-pad.py $S/trees/padN N` (N = 8, 48,
192), touched and built as above into `$S/tgt/pad1`, `pad2`, `pad3`. `0d911ab7c` built as above
into `$S/tgt/t1pa`. `nm -S`: symbol sizes, pads excluded, identical to base in all three controls.

| name | source | sha256 of `rexx-run` | `.text` bytes |
|---|---|---|---:|
| base | `9d863ccc5` | `3a82550968f46b4d0a82d38151bda74469d1a89b3980c42d2cdd35a640fa85b0` | 2,737,579 |
| pad1 | `9d863ccc5` + 8 pads | `1070e97a9bd5624fd00eae2fad777cf836bce2542bbbbadaa1a9edc02b1ca9be` | 2,738,747 |
| pad2 | `9d863ccc5` + 48 pads | `a8bef57c1fa6dc8d10ae7216e266567d69b32865558bba1a626acf3ede6f27af` | 2,745,131 |
| pad3 | `9d863ccc5` + 192 pads | `4dc74e8c313914226bda2d05f0c1153b39eb9d449c515f5ac7ffd053af93cf40` | 2,768,171 |
| t1pa | `0d911ab7c` | `39b588c01897934d77b38d2243ffc315504e48dddd6086c05262dfe38f1f4d1b` | 2,737,787 |

### Instruction counts

```
# sitting 1, script at 55ee6c08b: exit 1, stdout differed on heapshape and rexxcps, every run exited 0
rust/bench-programs/callgrind.sh -r 3 -j 16 -o $S/cg1 base=$S/bin/base/rexx-run pad1=$S/bin/pad1/rexx-run pad2=$S/bin/pad2/rexx-run pad3=$S/bin/pad3/rexx-run t1pa=$S/bin/t1pa/rexx-run
# sitting 2, script at 0696153b4, exit 0
rust/bench-programs/callgrind.sh -r 3 -j 16 -o $S/cg2 base=... (same arguments)
# both tabulated by median with the script at f0e3e727e
rust/bench-programs/callgrind.sh -T -o $S/cg1 base=... (same arguments)
rust/bench-programs/callgrind.sh -T -o $S/cg2 base=... (same arguments)
```

Instructions less libc.so.6 and ld-linux, median of three rounds. Control columns: sitting 2.
Band 1, band 2: largest control delta in sitting 1, 2. `0.0000`: under 0.00005%.

| program | base Ir ex-libc (sitting 2) | pad1 d% | pad2 d% | pad3 d% | band 1 | band 2 | noise band | 0d911ab7c d% (1 / 2) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| alloc | 25,099,728,421 | +0.0000 | -0.0000 | +0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / -0.0000 |
| alloc4c | 3,227,439,144 | -0.0000 | +0.0001 | +0.0002 | 0.0001 | 0.0002 | **0.0002** | +0.0000 / -0.0000 |
| arith | 11,523,104,960 | +0.0000 | +0.0000 | +0.0000 | 0.0001 | 0.0000 | **0.0001** | -0.0000 / +0.0000 |
| assign | 19,640,624,700 | -0.0000 | +0.0000 | -0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / -0.0000 |
| compound | 9,274,659,543 | +0.0000 | +0.0000 | +0.0000 | 0.0001 | 0.0000 | **0.0001** | -0.0001 / +0.0000 |
| decloop | 2,568,226,963 | -0.0003 | -0.0003 | -0.0002 | 0.0002 | 0.0003 | **0.0003** | +0.0003 / -0.0002 |
| decrender | 4,351,442,505 | +0.0001 | +0.0000 | +0.0000 | 0.0001 | 0.0001 | **0.0001** | +0.0001 / +0.0001 |
| dispatch | 20,511,340,571 | -0.0000 | -0.0000 | -0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / -0.0000 |
| dispatchclass | 15,873,243,458 | -0.0000 | -0.0000 | +0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / +0.0000 |
| emptyloop | 9,311,140,499 | -0.0000 | -0.0000 | +0.0000 | 0.0000 | 0.0000 | **0.0000** | +0.0000 / +0.0000 |
| extcall | 9,910,530,890 | -0.0000 | -0.0000 | +0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / +0.0000 |
| fibcall | 8,350,458,412 | -0.0000 | -0.0000 | -0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / -0.0000 |
| fibfunc | 7,993,681,764 | +0.0000 | +0.0000 | -0.0001 | 0.0001 | 0.0001 | **0.0001** | -0.0000 / -0.0000 |
| heapshape | 3,277,676,017 | -0.0001 | -0.0002 | -0.0001 | 0.0003 | 0.0002 | **0.0003** | +0.0000 / -0.0001 |
| nop | 9,440,357,070 | +0.0000 | -0.0000 | -0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / -0.0000 |
| parse | 1,545,045,099 | -0.0006 | -0.0004 | -0.0004 | 0.0002 | 0.0006 | **0.0006** | -0.0000 / -0.0004 |
| sayloop | 117,855,283 | +0.0028 | -0.0009 | -0.0006 | 0.0030 | 0.0028 | **0.0030** | -0.0018 / +0.0043 |
| sendloop | 13,891,242,032 | -0.0000 | -0.0000 | -0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / -0.0000 |
| startup | 61,114,468 | +0.0008 | -0.0045 | +0.0030 | 0.0039 | 0.0045 | **0.0045** | +0.0026 / +0.0142 |
| strings | 17,752,337,687 | -0.0000 | -0.0000 | -0.0000 | 0.0000 | 0.0000 | **0.0000** | -0.0000 / -0.0000 |
| textnum | 1,179,627,074 | -0.0001 | +0.0004 | +0.0002 | 0.0002 | 0.0004 | **0.0004** | -0.0003 / +0.0001 |
| varlookup | 14,881,155,494 | +0.0000 | +0.0000 | -0.0000 | 0.0000 | 0.0000 | **0.0000** | +0.0000 / +0.0000 |
| rexxcps | 17,920,429,749 | +0.0000 | +0.0000 | +0.0000 | 0.0000 | 0.0000 | **0.0000** | +0.0221 / +0.0221 |

Single-run excursions, sitting 2: one pad2 `extcall` run 42,000,341 above the higher of that
binary's other two runs (+0.42%), 42,000,000 of it in `rexx_api::handles::Table::register`; one
pad2 `rexxcps` run 7,554,142 above (+0.042%), in `Interp::resolve_fixed_call` and 280,003 extra
`memcmp` calls. Twelve further callgrind runs of base on `extcall`: none.

### `0d911ab7c` against the base

`rexxcps`: -0.0221% at the base in both sittings (17,920,429,749 against 17,924,390,887 in
sitting 2). Beyond band by more than 0.0001%: `startup` +0.0142% (sitting 2; band 0.0045%, t1pa
spread 0.0139%) and `sayloop` +0.0043% (sitting 2; band 0.0030%, t1pa spread in sitting 1
0.0109%).

### Wall clock

Five interleaved rounds, base and pad2 on every program, oracle on `extcall`, each `rexx-run`
copied to one path before each run. One-minute load average 5.97 at start, 6.28 at end.
`$S/wall.sh $S/wall1`:

```
PROGRAMS=<callgrind.sh's PROGRAMS line>
TIMEFORMAT=%R
run() { # arm program round
  local d t; d=$(mktemp -d $S/fresh.XXXXXX)
  if [ $1 = oracle ]; then
    t=$( { cd $d; ( ulimit -v 1048576; export LD_LIBRARY_PATH=$LIB; time /home/moritz/dev/repos/ooRexx/build/bin/rexx $(prog $2) > $out/$2.$1.r$3.out 2> $out/$2.$1.r$3.err ); } 2>&1 )
  else
    cp $S/bin/$1/rexx-run $S/stage/rexx-run
    t=$( { cd $d; ( export LD_LIBRARY_PATH=$LIB; time $S/stage/rexx-run $(prog $2) > $out/$2.$1.r$3.out 2> $out/$2.$1.r$3.err ); } 2>&1 )
  fi
  rmdir $d
  printf '%s\t%s\tr%s\t%s\n' $1 $2 $3 "$t" >> $out/wall.tsv
}
for r in 1 2 3 4 5; do
  for p in $PROGRAMS; do
    arms="base pad2"; [ $p = extcall ] && arms="base pad2 oracle"
    set -- $arms; n=$#; arr=($arms)
    for ((k = 0; k < n; k++)); do run ${arr[$(((k + r - 1) % n))]} $p $r; done
  done
done
```

Stderr empty on every run; stdout identical across rounds and arms except `heapshape` and
`rexxcps` (timings); `extcall` stdout identical to the oracle's.

| program | base median s | base runs | pad2 median s | pad2 runs | pad2 d% |
|---|---:|---|---:|---|---:|
| alloc | 2.051 | 2.051 2.113 2.056 2.047 2.048 | 2.119 | 2.124 2.137 2.060 2.119 2.043 | +3.32 |
| alloc4c | 0.699 | 0.651 0.727 0.699 0.721 0.660 | 0.703 | 0.648 0.774 0.703 0.706 0.663 | +0.57 |
| arith | 1.341 | 1.379 1.365 1.341 1.310 1.323 | 1.341 | 1.376 1.382 1.341 1.324 1.298 | +0.00 |
| assign | 1.089 | 1.086 1.104 1.084 1.135 1.089 | 1.091 | 1.091 1.105 1.080 1.089 1.096 | +0.18 |
| compound | 0.648 | 0.644 0.648 0.648 0.651 0.650 | 0.654 | 0.649 0.661 0.654 0.673 0.644 | +0.93 |
| decloop | 0.262 | 0.280 0.261 0.262 0.269 0.246 | 0.266 | 0.262 0.299 0.266 0.280 0.265 | +1.53 |
| decrender | 0.444 | 0.442 0.443 0.444 0.459 0.477 | 0.459 | 0.459 0.490 0.452 0.448 0.477 | +3.38 |
| dispatch | 1.789 | 1.789 1.829 1.778 1.801 1.761 | 1.725 | 1.744 1.723 1.720 1.759 1.725 | -3.58 |
| dispatchclass | 1.453 | 1.453 1.452 1.631 1.461 1.442 | 1.459 | 1.472 1.465 1.445 1.454 1.459 | +0.41 |
| emptyloop | 0.564 | 0.561 0.568 0.564 0.568 0.563 | 0.571 | 0.567 0.581 0.563 0.571 0.582 | +1.24 |
| extcall | 0.992 | 1.001 1.037 0.955 0.992 0.966 | 0.996 | 0.996 1.008 1.047 0.976 0.971 | +0.40 |
| fibcall | 0.875 | 0.875 0.852 0.846 0.876 0.990 | 0.866 | 0.862 0.851 0.866 0.914 0.876 | -1.03 |
| fibfunc | 0.816 | 0.816 0.813 0.810 0.817 0.824 | 0.819 | 0.824 0.814 0.830 0.809 0.819 | +0.37 |
| heapshape | 0.329 | 0.329 0.332 0.326 0.331 0.321 | 0.331 | 0.331 0.328 0.341 0.330 0.335 | +0.61 |
| nop | 0.675 | 0.675 0.670 0.698 0.689 0.675 | 0.676 | 0.668 0.676 0.673 0.699 0.677 | +0.15 |
| parse | 0.133 | 0.133 0.137 0.133 0.128 0.130 | 0.130 | 0.139 0.130 0.132 0.129 0.129 | -2.26 |
| sayloop | 0.030 | 0.033 0.030 0.029 0.028 0.030 | 0.030 | 0.030 0.030 0.027 0.028 0.030 | +0.00 |
| sendloop | 1.081 | 1.099 1.076 1.081 1.086 1.076 | 1.105 | 1.105 1.093 1.112 1.125 1.095 | +2.22 |
| startup | 0.026 | 0.026 0.024 0.026 0.026 0.025 | 0.026 | 0.026 0.027 0.026 0.025 0.023 | +0.00 |
| strings | 1.173 | 1.125 1.192 1.173 1.182 1.142 | 1.168 | 1.168 1.141 1.182 1.172 1.155 | -0.43 |
| textnum | 0.094 | 0.117 0.099 0.093 0.094 0.094 | 0.094 | 0.094 0.099 0.094 0.096 0.094 | +0.00 |
| varlookup | 0.802 | 0.801 0.796 0.804 0.804 0.802 | 0.805 | 0.826 0.796 0.824 0.805 0.804 | +0.37 |
| rexxcps | 2.085 | 2.120 2.014 2.191 2.085 2.037 | 2.051 | 2.042 2.051 2.063 2.084 2.050 | -1.63 |

oracle extcall: median 0.563 s, runs 0.563 0.559 0.528 0.578 0.566; base/oracle 1.76x
