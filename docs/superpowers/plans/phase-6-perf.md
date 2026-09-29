# Phase 6 performance record

Base `1754a3b5a`, measured 2026-09-29. `callgrind.sh` and the runs of `wallclock.sh` at `44c465281`;
`wallclock.sh -T` and `cgdiff.py` at the commit adding this file. rustc 1.98.1,
valgrind-3.27.1, `nproc` 32. `S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t2/r2`,
`B` is `rust/bench-programs`.

## Builds

```
git archive 1754a3b5a | tar -x -C $S/trees/NAME          # NAME = base pad1 pad2 pad3
python3 $B/layout-pad.py $S/trees/pad1 8
python3 $B/layout-pad.py $S/trees/pad2 48
python3 $B/layout-pad.py $S/trees/pad3 192
find $S/trees/NAME -type f -exec touch {} +
cd $S/trees/NAME/rust && CARGO_TARGET_DIR=$S/tgt/NAME cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-NAME.log 2>&1
/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-NAME.log                 # 1 for each NAME
cp $S/tgt/NAME/release/rexx-run $S/bin/NAME/rexx-run
sha256sum < $S/bin/NAME/rexx-run
objcopy -O binary --only-section=.text $S/bin/NAME/rexx-run /dev/stdout | wc -c
```

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `e500b628ff3ee3de6bd2b17741e2cbd95317cc81816ac0a01b9c6cefd22de39b` | 2,732,523 |
| pad1 | `95a6104d28371660c9c47401c4859312631c0e1913617e0a48d86b23897f2699` | 2,733,691 |
| pad2 | `ae258873812332f3641a6fae87a593b5968e4c1200b2e2dfb0fcf02bad87c1b1` | 2,740,075 |
| pad3 | `f161b53b187d3a5e219ed258cc40d045aca769c786c6fa81a7c66c4a05c30386` | 2,763,115 |

Symbol sizes, pads excluded, against base: 0 differing lines of 3865 for each of pad1, pad2, pad3.

```
diff <(nm -S $S/bin/base/rexx-run | awk 'NF==4{print $2}' | sort) \
     <(nm -S $S/bin/padN/rexx-run | awk 'NF==4 && $4 !~ /layout_pad|LAYOUT_PAD/{print $2}' | sort) | wc -l
```

## Oracle comparison

```
cd $B; LIB=/home/moritz/dev/repos/ooRexx/build/lib; O=$S/oracmp
for p in fibcall fibfunc sendloop extcall; do
  f=$(readlink -f $p.rex)
  d=$(mktemp -d); (cd $d; ( ulimit -v 1048576; LD_LIBRARY_PATH=$LIB /home/moritz/dev/repos/ooRexx/build/bin/rexx $f ) > $O/$p.o.out 2> $O/$p.o.err; echo $? > $O/$p.o.rc); rmdir $d
  d=$(mktemp -d); (cd $d; LD_LIBRARY_PATH=$LIB $S/bin/base/rexx-run $f > $O/$p.r.out 2> $O/$p.r.err; echo $? > $O/$p.r.rc); rmdir $d
  echo "$p stdout=$(cmp -s $O/$p.o.out $O/$p.r.out && echo same || echo DIFF) stderr=$(cmp -s $O/$p.o.err $O/$p.r.err && echo same || echo DIFF) rc=$(cat $O/$p.o.rc)/$(cat $O/$p.r.rc)"
done
```
```
fibcall stdout=same stderr=same rc=0/0
fibfunc stdout=same stderr=same rc=0/0
sendloop stdout=same stderr=same rc=0/0
extcall stdout=same stderr=same rc=0/0
```

## Instruction counts

```
$B/callgrind.sh -r 3 -j 16 -o $S/cg base=$S/bin/base/rexx-run pad1=$S/bin/pad1/rexx-run pad2=$S/bin/pad2/rexx-run pad3=$S/bin/pad3/rexx-run
```

Exit 0, no SPREAD line. Instructions less libc.so.6 and ld-linux, median of three rounds. Noise
band: largest control difference. Base spread: max minus min of base's rounds.

| program | base | pad1 - base | pad2 - base | pad3 - base | noise band | base spread |
|---|---:|---:|---:|---:|---:|---:|
| alloc | 25,168,678,513 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| alloc4c | 3,224,395,275 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| arith | 11,520,060,209 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| assign | 19,637,557,640 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| compound | 9,271,611,489 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| decloop | 2,565,175,941 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| decrender | 4,348,396,352 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| dispatch | 20,483,285,126 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| dispatchclass | 15,850,195,751 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| emptyloop | 9,308,098,470 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| extcall | 8,281,259,754 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| fibcall | 8,345,686,651 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| fibfunc | 7,988,911,997 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| heapshape | 3,278,634,255 | +0 | +0 | +100 | 100 (0.00000%) | 100 |
| nop | 9,437,300,404 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| parse | 1,541,986,776 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| sayloop | 114,813,602 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| sendloop | 13,863,190,856 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| startup | 58,074,291 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| strings | 17,749,287,716 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| textnum | 1,176,579,713 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| varlookup | 14,878,111,844 | +0 | +0 | +0 | 0 (0.00000%) | 0 |
| rexxcps | 17,817,346,507 | +5,722 | -9,942 | -11,030 | 11,030 (0.00006%) | 20,659 |

### Repeat runs

```
$B/callgrind.sh -r 32 -j 16 -p "extcall rexxcps" -o $S/rep32 base=$S/bin/base/rexx-run
awk -F'\t' '{k=$2; v=$7; if(!(k in mn)||v<mn[k])mn[k]=v; if(v>mx[k])mx[k]=v; c[k]++} END{for(k in mn) print k, c[k], "runs exlibc min", mn[k], "max", mx[k]}' $S/rep32/summary.tsv
```
```
extcall 32 runs exlibc min 8281259754 max 8281259754
rexxcps 32 runs exlibc min 17817340311 max 17817354515
```

`rexxcps` and `heapshape` call `TIME()`. Between rexxcps's min and max runs, and between two
heapshape base runs 100 apart:

```
python3 $B/cgdiff.py $S/rep32/rexxcps.base.r25.cg $S/rep32/rexxcps.base.r3.cg
python3 $B/cgdiff.py $S/cg/heapshape.base.r1.cg $S/cg/heapshape.base.r3.cg
```
excerpt:

```
rexxcps calls: +33 each to chrono Source::new, getenv, statx, try_statx; self +15477 getenv
heapshape self: +58 Formatter::pad_integral, +56 String::write_char, -14 u64::_fmt_inner; calls: +2 String::write_char
```

## Wall clock

```
$B/wallclock.sh -r 5 -x extcall -o $S/wall base=$S/bin/base/rexx-run pad2=$S/bin/pad2/rexx-run
$B/wallclock.sh -T -o $S/wall base=$S/bin/base/rexx-run pad2=$S/bin/pad2/rexx-run
```

Load average (1, 5, 15 min) 2.27 7.95 10.55 at start, 1.20 4.68 8.76 at end. Stderr empty on
every run; stdout identical across rounds and arms except `heapshape` and `rexxcps`; `extcall`
stdout identical to the oracle's.

| program | base median s | pad2 median s | pad2 d% |
|---|---:|---:|---:|
| alloc | 2.014 | 2.010 | -0.20 |
| alloc4c | 0.575 | 0.567 | -1.39 |
| arith | 1.235 | 1.208 | -2.19 |
| assign | 1.069 | 1.074 | +0.47 |
| compound | 0.653 | 0.644 | -1.38 |
| decloop | 0.246 | 0.261 | +6.10 |
| decrender | 0.455 | 0.455 | +0.00 |
| dispatch | 1.770 | 1.751 | -1.07 |
| dispatchclass | 1.385 | 1.358 | -1.95 |
| emptyloop | 0.527 | 0.531 | +0.76 |
| extcall | 0.930 | 0.927 | -0.32 |
| fibcall | 0.791 | 0.800 | +1.14 |
| fibfunc | 0.785 | 0.811 | +3.31 |
| heapshape | 0.325 | 0.321 | -1.23 |
| nop | 0.655 | 0.661 | +0.92 |
| parse | 0.130 | 0.131 | +0.77 |
| sayloop | 0.028 | 0.029 | +3.57 |
| sendloop | 1.072 | 1.070 | -0.19 |
| startup | 0.026 | 0.024 | -7.69 |
| strings | 1.140 | 1.138 | -0.18 |
| textnum | 0.094 | 0.091 | -3.19 |
| varlookup | 0.780 | 0.784 | +0.51 |
| rexxcps | 2.028 | 1.972 | -2.76 |

`extcall` on the oracle: median 0.546 s; base/oracle 1.70.
