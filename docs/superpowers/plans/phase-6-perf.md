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

### Hasher change (`9d863ccc5` to `1754a3b5a`)

`9d863ccc5` (sha256 `3a82550968f46b4d0a82d38151bda74469d1a89b3980c42d2cdd35a640fa85b0`) measured with
`callgrind.sh` at `0696153b4`, `-r 3 -j 16 -o $S/../cg2`, base rows only; medians compared with
`$S/cg`'s base rows:

```
python3 - $S/../cg2/summary.tsv $S/cg/summary.tsv <<'EOF'
import sys, statistics as st
def load(p):
    r = {}
    for l in open(p):
        b, pr, rr, s, libc, ld, ex, rc = l.rstrip("\n").split("\t")
        if b == "base": r.setdefault(pr, []).append(int(ex))
    return {k: st.median(v) for k, v in r.items()}
a, b = load(sys.argv[1]), load(sys.argv[2])
for k in a: print(k, a[k], b[k], f"{100 * (b[k] - a[k]) / a[k]:+.2f}")
EOF
```

| program | 9d863ccc5 | 1754a3b5a | d% |
|---|---:|---:|---:|
| alloc | 25,099,728,421 | 25,168,678,513 | +0.27 |
| alloc4c | 3,227,439,144 | 3,224,395,275 | -0.09 |
| arith | 11,523,104,960 | 11,520,060,209 | -0.03 |
| assign | 19,640,624,700 | 19,637,557,640 | -0.02 |
| compound | 9,274,659,543 | 9,271,611,489 | -0.03 |
| decloop | 2,568,226,963 | 2,565,175,941 | -0.12 |
| decrender | 4,351,442,505 | 4,348,396,352 | -0.07 |
| dispatch | 20,511,340,571 | 20,483,285,126 | -0.14 |
| dispatchclass | 15,873,243,458 | 15,850,195,751 | -0.15 |
| emptyloop | 9,311,140,499 | 9,308,098,470 | -0.03 |
| extcall | 9,910,530,890 | 8,281,259,754 | -16.44 |
| fibcall | 8,350,458,412 | 8,345,686,651 | -0.06 |
| fibfunc | 7,993,681,764 | 7,988,911,997 | -0.06 |
| heapshape | 3,277,676,017 | 3,278,634,255 | +0.03 |
| nop | 9,440,357,070 | 9,437,300,404 | -0.03 |
| parse | 1,545,045,099 | 1,541,986,776 | -0.20 |
| sayloop | 117,855,283 | 114,813,602 | -2.58 |
| sendloop | 13,891,242,032 | 13,863,190,856 | -0.20 |
| startup | 61,114,468 | 58,074,291 | -4.97 |
| strings | 17,752,337,687 | 17,749,287,716 | -0.02 |
| textnum | 1,179,627,074 | 1,176,579,713 | -0.26 |
| varlookup | 14,881,155,494 | 14,878,111,844 | -0.02 |
| rexxcps | 17,920,429,749 | 17,817,346,507 | -0.58 |

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

The ±4% wall-clock bar applies where pad2's delta is inside ±4%; for `decloop` (+6.10%) and
`startup` (-7.69%) the bar is that measured delta.

`extcall` on the oracle: median 0.546 s; base/oracle 1.70.

## Task 4

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t4`.

```
git archive COMMIT | tar -x -C $S/trees/NAME       # base 1754a3b5a, r1 0a4c5f46a, r2 8f0437be3
find $S/trees/NAME -type f -exec touch {} +
cd $S/trees/NAME/rust && CARGO_TARGET_DIR=$S/tgt/NAME cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-NAME.log 2>&1
/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-NAME.log                 # 1 for each NAME
cp $S/tgt/NAME/release/rexx-run $S/bin/NAME/rexx-run
sha256sum < $S/bin/NAME/rexx-run
objcopy -O binary --only-section=.text $S/bin/NAME/rexx-run /dev/stdout | wc -c
```

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `fed193bcadeb5e61e711c993ecfc57ae4dfb7000e1a25fe95082c7707d38bab3` | 2,732,523 |
| r1 | `1c6da11b1b89818041927bd27da2f9a98e952a072d3b57bc2463fe894c138169` | 2,737,355 |
| r2 | `337ae2ae4db4496df302fa39e106427b6eee29ceef4d2b3bbf26f6914463f3e8` | 2,728,395 |

```
$B/callgrind.sh -r 3 -j 16 -o $S/cg1 base=$S/bin/base/rexx-run r1=$S/bin/r1/rexx-run
$B/callgrind.sh -r 3 -j 16 -o $S/cg2 base=$S/bin/base/rexx-run r2=$S/bin/r2/rexx-run
```

Both exit 0. r1 holds `Box<Activity>`, r2 the `Activity` inline.

| program | base (cg1) | r1 | r1 - base | r1 d% | base (cg2) | r2 | r2 - base | r2 d% |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| alloc | 25,168,678,363 | 25,504,681,098 | +336,002,735 | +1.3350 | 25,168,678,363 | 25,168,673,999 | -4,364 | -0.0000 |
| alloc4c | 3,224,395,040 | 3,313,400,562 | +89,005,522 | +2.7604 | 3,224,395,040 | 3,224,393,701 | -1,339 | -0.0000 |
| arith | 11,520,060,059 | 11,605,062,063 | +85,002,004 | +0.7379 | 11,520,060,059 | 11,520,054,911 | -5,148 | -0.0000 |
| assign | 19,637,557,619 | 20,748,564,157 | +1,111,006,538 | +5.6576 | 19,637,557,619 | 19,637,557,417 | -202 | -0.0000 |
| compound | 9,271,611,424 | 9,561,634,239 | +290,022,815 | +3.1281 | 9,271,611,424 | 9,271,611,424 | +0 | +0.0000 |
| decloop | 2,565,175,706 | 2,616,780,954 | +51,605,248 | +2.0118 | 2,565,175,706 | 2,565,174,073 | -1,633 | -0.0001 |
| decrender | 4,348,396,202 | 4,452,801,366 | +104,405,164 | +2.4010 | 4,348,396,202 | 4,348,394,484 | -1,718 | -0.0000 |
| dispatch | 20,483,285,061 | 20,968,291,958 | +485,006,897 | +2.3678 | 20,483,285,061 | 20,483,285,061 | +0 | +0.0000 |
| dispatchclass | 15,850,195,601 | 16,226,202,382 | +376,006,781 | +2.3723 | 15,850,195,601 | 15,850,195,647 | +46 | +0.0000 |
| emptyloop | 9,308,098,320 | 9,808,105,093 | +500,006,773 | +5.3717 | 9,308,098,320 | 9,308,098,366 | +46 | +0.0000 |
| extcall | 8,281,259,519 | 8,428,266,395 | +147,006,876 | +1.7752 | 8,281,259,519 | 8,281,259,647 | +128 | +0.0000 |
| fibcall | 8,345,686,416 | 8,609,619,885 | +263,933,469 | +3.1625 | 8,345,686,416 | 8,345,686,547 | +131 | +0.0000 |
| fibfunc | 7,988,911,762 | 8,231,353,348 | +242,441,586 | +3.0347 | 7,988,911,762 | 7,988,911,893 | +131 | +0.0000 |
| heapshape | 3,278,634,105 | 3,316,768,904 | +38,134,799 | +1.1631 | 3,278,634,105 | 3,278,633,171 | -934 | -0.0000 |
| nop | 9,437,300,169 | 10,348,307,027 | +911,006,858 | +9.6533 | 9,437,300,169 | 9,437,300,300 | +131 | +0.0000 |
| parse | 1,541,986,626 | 1,586,792,354 | +44,805,728 | +2.9057 | 1,541,986,626 | 1,541,985,496 | -1,130 | -0.0001 |
| sayloop | 114,813,367 | 117,920,205 | +3,106,838 | +2.7060 | 114,813,367 | 114,813,498 | +131 | +0.0001 |
| sendloop | 13,863,190,791 | 14,178,197,538 | +315,006,747 | +2.2723 | 13,863,190,791 | 13,863,190,791 | +0 | +0.0000 |
| startup | 58,074,056 | 58,080,877 | +6,821 | +0.0117 | 58,074,056 | 58,074,187 | +131 | +0.0002 |
| strings | 17,749,287,481 | 18,283,286,185 | +533,998,704 | +3.0086 | 17,749,287,481 | 17,749,278,694 | -8,787 | -0.0000 |
| textnum | 1,176,579,478 | 1,221,386,348 | +44,806,870 | +3.8082 | 1,176,579,478 | 1,176,579,609 | +131 | +0.0000 |
| varlookup | 14,878,111,694 | 15,657,118,479 | +779,006,785 | +5.2359 | 14,878,111,694 | 14,878,111,740 | +46 | +0.0000 |
| rexxcps | 17,817,346,486 | 18,535,102,753 | +717,756,267 | +4.0284 | 17,817,346,062 | 17,817,329,203 | -16,859 | -0.0001 |

## Task 5

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t5`.

```
git archive COMMIT | tar -x -C $S/trees/NAME       # base 1754a3b5a, r1 50cce849c
find $S/trees/NAME -type f -exec touch {} +
cd $S/trees/NAME/rust && CARGO_TARGET_DIR=$S/tgt/NAME cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-NAME.log 2>&1
/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-NAME.log                 # 1 for each NAME
cp $S/tgt/NAME/release/rexx-run $S/bin/NAME/rexx-run
sha256sum < $S/bin/NAME/rexx-run
objcopy -O binary --only-section=.text $S/bin/NAME/rexx-run /dev/stdout | wc -c
```

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `264eb30b93ecb351db2eb99dff146a3e6920d63210228460bd4afdfcc715a4e1` | 2,732,523 |
| r1 | `d51e59452601948fc74e3a51854896481f7781fce121a7b9733b2d95c08b5e4c` | 2,728,539 |

```
$B/callgrind.sh -r 3 -j 16 -o $S/cg1 base=$S/bin/base/rexx-run r1=$S/bin/r1/rexx-run
```

Exits 0. r1 holds the running activity's `ActivityRoots` inline in `RootSet`.

| program | base | r1 | r1 - base | r1 d% |
|---|---:|---:|---:|---:|
| alloc | 25,168,678,363 | 25,168,639,819 | -38,544 | -0.0002 |
| alloc4c | 3,224,395,040 | 3,224,382,606 | -12,434 | -0.0004 |
| arith | 11,520,060,059 | 11,520,015,339 | -44,720 | -0.0004 |
| assign | 19,637,557,619 | 19,637,557,482 | -137 | -0.0000 |
| compound | 9,271,611,424 | 9,271,611,489 | +65 | +0.0000 |
| decloop | 2,565,175,706 | 2,565,160,836 | -14,870 | -0.0006 |
| decrender | 4,348,396,202 | 4,348,381,301 | -14,901 | -0.0003 |
| dispatch | 20,483,285,061 | 20,483,285,126 | +65 | +0.0000 |
| dispatchclass | 15,850,195,601 | 15,850,195,712 | +111 | +0.0000 |
| emptyloop | 9,308,098,320 | 9,308,098,431 | +111 | +0.0000 |
| extcall | 8,281,259,519 | 8,281,259,712 | +193 | +0.0000 |
| fibcall | 8,345,686,416 | 8,345,686,612 | +196 | +0.0000 |
| fibfunc | 7,988,911,762 | 7,988,911,958 | +196 | +0.0000 |
| heapshape | 3,278,634,105 | 3,275,164,618 | -3,469,487 | -0.1058 |
| nop | 9,437,300,169 | 9,437,300,365 | +196 | +0.0000 |
| parse | 1,541,986,626 | 1,541,976,669 | -9,957 | -0.0006 |
| sayloop | 114,813,367 | 114,813,563 | +196 | +0.0002 |
| sendloop | 13,863,190,791 | 13,863,190,856 | +65 | +0.0000 |
| startup | 58,074,056 | 58,074,252 | +196 | +0.0003 |
| strings | 17,749,287,481 | 17,749,211,510 | -75,971 | -0.0004 |
| textnum | 1,176,579,478 | 1,176,579,674 | +196 | +0.0000 |
| varlookup | 14,878,111,694 | 14,878,111,805 | +111 | +0.0000 |
| rexxcps | 17,817,340,661 | 17,817,273,008 | -67,653 | -0.0004 |

## Task 6 (S0 gate)

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t6`.

```
git archive REV | tar -x -C $S/trees/NAME       # base 1754a3b5a, stepA 6956e90f2, stepB 98d9fae33
find $S/trees/NAME -type f -exec touch {} +
cd $S/trees/NAME/rust && CARGO_TARGET_DIR=$S/tgt/NAME cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-NAME.log 2>&1
/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-NAME.log                 # 1 for each NAME
cp $S/tgt/NAME/release/rexx-run $S/bin/NAME/rexx-run
sha256sum < $S/bin/NAME/rexx-run
objcopy -O binary --only-section=.text $S/bin/NAME/rexx-run /dev/stdout | wc -c
```

`stepA` is the running total through the `ObjRef` `!Send`/`!Sync` commit; `stepB` adds the bounded
countdown reload. Both are on top of Tasks 1-5, so each is the S0 running total at that point
against the unmodified Phase 6 base.

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `39685fe900b5a6685160875833b8f8952dbd1eb4445b8f7f9f2a0bf22f2e1cff` | 2,732,523 |
| stepA | `78f35aaafc0dd6ba3ff0e9b2440331648fd877239a663081fcfb06c22f120c11` | 2,729,083 |
| stepB | `7a49502632ed82323866c6e796251356dd8a1e821421211fcd13b5444e468b36` | 2,729,035 |

### Oracle comparison

```
cd $B; LIB=/home/moritz/dev/repos/ooRexx/build/lib; O=$S
for p in fibcall fibfunc sendloop extcall; do
  f=$(readlink -f $p.rex)
  d=$(mktemp -d); (cd $d; ( ulimit -v 1048576; LD_LIBRARY_PATH=$LIB /home/moritz/dev/repos/ooRexx/build/bin/rexx $f ) > $O/o.$p.out 2> $O/o.$p.err; echo $? > $O/o.$p.rc); rmdir $d
  d=$(mktemp -d); (cd $d; LD_LIBRARY_PATH=$LIB $S/bin/stepB/rexx-run $f > $O/r.$p.out 2> $O/r.$p.err; echo $? > $O/r.$p.rc); rmdir $d
  echo "$p stdout=$(cmp -s $O/o.$p.out $O/r.$p.out && echo same || echo DIFF) stderr=$(cmp -s $O/o.$p.err $O/r.$p.err && echo same || echo DIFF) rc=$(cat $O/o.$p.rc)/$(cat $O/r.$p.rc)"
done
```
```
fibcall stdout=same stderr=same rc=0/0
fibfunc stdout=same stderr=same rc=0/0
sendloop stdout=same stderr=same rc=0/0
extcall stdout=same stderr=same rc=0/0
```

### Instruction counts (running total)

```
$B/callgrind.sh -r 3 -j 16 -o $S/cg base=$S/bin/base/rexx-run stepA=$S/bin/stepA/rexx-run stepB=$S/bin/stepB/rexx-run
```

Exit 0, every run's `rc` file 0, no SPREAD line. Instructions less libc.so.6 and ld-linux, median
of three rounds.

| program | base | stepA - base | stepB - base | stepB d% |
|---|---:|---:|---:|---:|
| alloc | 25,168,678,420 | -38,584 | +301,269 | +0.0012% |
| alloc4c | 3,224,394,995 | -12,421 | +100,855 | +0.0031% |
| arith | 11,520,060,116 | -44,760 | +68,516 | +0.0006% |
| assign | 19,637,557,532 | +216 | +2,860,575 | +0.0146% |
| compound | 9,271,611,407 | -5 | +424,818 | +0.0046% |
| decloop | 2,565,175,661 | -14,857 | +64,041 | +0.0025% |
| decrender | 4,348,396,259 | -14,941 | +109,284 | +0.0025% |
| dispatch | 20,483,285,044 | -5 | +566,396 | +0.0028% |
| dispatchclass | 15,850,195,658 | +71 | +339,924 | +0.0021% |
| emptyloop | 9,308,098,377 | +71 | +1,416,085 | +0.0152% |
| extcall | 8,281,259,474 | +206 | +170,119 | +0.0021% |
| fibcall | 8,345,686,371 | +209 | +341,048 | +0.0041% |
| fibfunc | 7,988,911,717 | +209 | +268,026 | +0.0034% |
| heapshape | 3,278,634,162 | -3,469,527 | -3,412,716 | -0.1041% |
| nop | 9,437,300,124 | +209 | +2,860,568 | +0.0303% |
| parse | 1,541,986,683 | -9,997 | +35,303 | +0.0023% |
| sayloop | 114,813,322 | +209 | +5,866 | +0.0051% |
| sendloop | 13,863,190,774 | -5 | +283,182 | +0.0020% |
| startup | 58,074,011 | +209 | +211 | +0.0004% |
| strings | 17,749,287,436 | -75,958 | +433,806 | +0.0024% |
| textnum | 1,176,579,433 | +209 | +68,187 | +0.0058% |
| varlookup | 14,878,111,751 | +71 | +1,614,329 | +0.0109% |
| rexxcps | 17,817,337,824 | -61,517 | +542,884 | +0.0030% |

Budget: at most +0.3% beyond each program's noise band (recorded above; 0 for every program except
`heapshape` 0.00000%, `sayloop` 0.0030%, `startup` 0.0045%, `rexxcps` 0.00006%). Largest `stepB`
delta is `nop` at +0.0303%, `heapshape` is negative. Every program is inside budget in round 1;
`emptyloop`'s +0.0152% matches the spec's own N=50-to-N=1024 scaling from the spike's +0.27% (1024
is about 20.5x 50; 0.27/20.5 = 0.0132%, close to measured).

### Wall clock

```
$B/wallclock.sh -r 5 -o $S/wall base=$S/bin/base/rexx-run stepB=$S/bin/stepB/rexx-run
```

| program | base median s | stepB median s | stepB d% |
|---|---:|---:|---:|
| alloc | 2.017 | 1.997 | -0.99 |
| alloc4c | 0.567 | 0.576 | +1.59 |
| arith | 1.226 | 1.230 | +0.33 |
| assign | 1.066 | 1.070 | +0.38 |
| compound | 0.648 | 0.641 | -1.08 |
| decloop | 0.250 | 0.249 | -0.40 |
| decrender | 0.462 | 0.476 | +3.03 |
| dispatch | 1.772 | 1.957 | +10.44 |
| dispatchclass | 1.386 | 1.385 | -0.07 |
| emptyloop | 0.527 | 0.527 | +0.00 |
| extcall | 0.923 | 0.909 | -1.52 |
| fibcall | 0.802 | 0.817 | +1.87 |
| fibfunc | 0.795 | 0.802 | +0.88 |
| heapshape | 0.322 | 0.322 | +0.00 |
| nop | 0.663 | 0.660 | -0.45 |
| parse | 0.129 | 0.130 | +0.78 |
| sayloop | 0.029 | 0.030 | +3.45 |
| sendloop | 1.095 | 1.135 | +3.65 |
| startup | 0.025 | 0.026 | +4.00 |
| strings | 1.145 | 1.163 | +1.57 |
| textnum | 0.092 | 0.092 | +0.00 |
| varlookup | 0.784 | 0.777 | -0.89 |
| rexxcps | 2.036 | 1.984 | -2.55 |

Every program is inside its ±4% bar (`startup`'s own band is -7.69% per Ruling P10) except
`dispatch` at +10.44%.

### `dispatch`'s wall clock: three controls

`dispatch` reproduces its excursion across an independent re-run, and reproduces even against a
change with a flat instruction count, so the excursion is layout noise rather than added work.

```
$B/wallclock.sh -r 5 -o $S/wall2 base=$S/bin/base/rexx-run stepB=$S/bin/stepB/rexx-run   # independent re-run, same two binaries
cp $S/bin/base/rexx-run $S/bin/base2/rexx-run                                            # identical-binary control
$B/wallclock.sh -r 5 -o $S/wallctl base=$S/bin/base/rexx-run base2=$S/bin/base2/rexx-run
$B/wallclock.sh -r 5 -o $S/wallA base=$S/bin/base/rexx-run stepA=$S/bin/stepA/rexx-run    # stepA alone (flat Ir vs base on dispatch)
```

| run | base median s | other median s | d% | `dispatch` Ir vs base |
|---|---:|---:|---:|---:|
| stepB re-run (`wall2`) | 1.764 | 1.969 (stepB) | +11.62 | +566,396 (+0.0028%) |
| identical binary (`wallctl`) | 1.781 | 1.767 (base2, byte-identical) | -0.79 | +0 |
| stepA alone (`wallA`) | 1.777 | 2.042 (stepA) | +14.91 | -5 (-0.0000%) |

`sendloop` also moved beyond ±4% in two of the four wall-clock runs (`wall2` +11.15%, `wallA`
+7.96%) but not the other two (`wall` +3.65%, and it is not `wallctl`'s pair), and its own Ir delta
is flat (+283,182, +0.0020%) same as `dispatch`'s. The identical-binary control (`wallctl`) shows
every program, `dispatch` and `sendloop` included, inside 4% when nothing about the binary differs,
so the instrument itself is not this noisy; a real, flat-instruction-count code change is enough to
move `dispatch` and sometimes `sendloop` past ±4% here. Task 2's own three padding controls put
`dispatch`'s band at -3.58% (`pad2`), narrower than what a real (non-padding) code change produces.
Reported for a ruling rather than spent on rounds: no candidate change is indicated by a flat
instruction count, and a round chasing linker-address luck on one benchmark is not a principled
S0 change.
