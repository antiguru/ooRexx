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

Budget: at most +0.3% beyond each program's noise band (recorded above). Largest `stepB`
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

## Task 7

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t7`.

```
git archive REV | tar -x -C $S/trees/NAME       # base 1754a3b5a, prev 8b253ef97, t7 31a394482, t7b a0a82f2b7
find $S/trees/NAME -type f -exec touch {} +
cd $S/trees/NAME/rust && CARGO_TARGET_DIR=$S/tgt/NAME cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-NAME.log 2>&1
/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-NAME.log                 # 1 for each NAME
cp $S/tgt/NAME/release/rexx-run $S/bin/NAME/rexx-run
sha256sum < $S/bin/NAME/rexx-run
objcopy -O binary --only-section=.text $S/bin/NAME/rexx-run /dev/stdout | wc -c
cp $S/bin/base/rexx-run $S/bin/base2/rexx-run                                  # identical-binary control
```

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `d59c32248068af0bb927e2f2b714b1bffca4b17d32231c884f812b4bd40005b6` | 2,732,523 |
| prev | `a1bb5533770cd845c8ec80e3ab189ee9243ed5f9cd6a8c24e4baf436d836bbbe` | 2,729,035 |
| t7 | `9af11a0218e9d87244f147465cb5de008dee3c952798af7c7893a6cf2db3a2d9` | 2,725,499 |
| t7b | `22cdd7a6d7f16d0b5ba23fe6e2f8170b849d81d84d4ac96791dafcb42068cc0d` | 2,725,563 |

### Oracle comparison

```
cd $B; LIB=/home/moritz/dev/repos/ooRexx/build/lib; O=$S/oracmp
for p in fibcall fibfunc sendloop extcall; do
  f=$(readlink -f $p.rex)
  d=$(mktemp -d); (cd $d; ( ulimit -v 1048576; LD_LIBRARY_PATH=$LIB /home/moritz/dev/repos/ooRexx/build/bin/rexx $f ) > $O/o.$p.out 2> $O/o.$p.err; echo $? > $O/o.$p.rc); rmdir $d
  d=$(mktemp -d); (cd $d; LD_LIBRARY_PATH=$LIB $S/bin/t7b/rexx-run $f > $O/r.$p.out 2> $O/r.$p.err; echo $? > $O/r.$p.rc); rmdir $d
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
$B/callgrind.sh -r 3 -j 12 -o $S/cg2 base=$S/bin/base/rexx-run prev=$S/bin/prev/rexx-run t7=$S/bin/t7/rexx-run t7b=$S/bin/t7b/rexx-run
```

Exit 0, no SPREAD line. Instructions less libc.so.6 and ld-linux, median of three rounds. `t7` is
round 1, `t7b` round 2.

| program | base | prev - base | t7 - base | t7b - base | t7b d% |
|---|---:|---:|---:|---:|---:|
| alloc | 25,168,678,363 | +301,269 | +301,152 | -104,700,691 | -0.4160% |
| alloc4c | 3,224,395,040 | +100,717 | +1,100,684 | -3,901,073 | -0.1210% |
| arith | 11,520,060,059 | +68,516 | -4,931,607 | -7,933,454 | -0.0689% |
| assign | 19,637,557,619 | +2,860,430 | -96,139,931 | -198,141,606 | -1.0090% |
| compound | 9,271,611,424 | +424,894 | +5,424,713 | -19,578,129 | -0.2112% |
| decloop | 2,565,175,706 | +63,903 | -8,336,125 | -9,137,885 | -0.3562% |
| decrender | 4,348,396,202 | +109,284 | -9,890,829 | -10,692,672 | -0.2459% |
| dispatch | 20,483,285,061 | +566,472 | -79,394,732 | -189,396,578 | -0.9246% |
| dispatchclass | 15,850,195,601 | +339,924 | -99,652,412 | -135,654,255 | -0.8559% |
| emptyloop | 9,308,098,320 | +1,416,085 | -23,584,020 | -73,585,862 | -0.7906% |
| extcall | 8,281,259,519 | +169,981 | +81,169,955 | +12,168,197 | +0.1469% |
| fibcall | 8,345,686,416 | +340,910 | -78,741,339 | -79,602,838 | -0.9538% |
| fibfunc | 7,988,911,762 | +267,888 | -74,516,009 | -76,237,248 | -0.9543% |
| heapshape | 3,278,634,105 | -3,412,716 | +18,611,708 | -13,456,840 | -0.4104% |
| nop | 9,437,300,169 | +2,860,430 | -196,139,590 | -198,141,349 | -2.0996% |
| parse | 1,541,986,626 | +35,303 | +5,435,148 | -1,566,696 | -0.1016% |
| sayloop | 114,813,367 | +5,728 | -94,290 | -296,049 | -0.2579% |
| sendloop | 13,863,190,791 | +283,258 | -34,716,950 | -144,718,793 | -1.0439% |
| startup | 58,074,056 | +73 | +72 | -1,686 | -0.0029% |
| strings | 17,749,287,481 | +433,668 | -2,566,378 | -23,568,139 | -0.1328% |
| textnum | 1,176,579,478 | +68,049 | -1,731,983 | -3,133,743 | -0.2663% |
| varlookup | 14,878,111,694 | +1,614,329 | -17,385,784 | -93,387,627 | -0.6277% |
| rexxcps | 17,817,347,083 | +530,994 | -21,681,769 | -49,692,262 | -0.2789% |

### Wall clock

```
$B/wallclock.sh -r 5 -o $S/wall base=$S/bin/base/rexx-run t7b=$S/bin/t7b/rexx-run
```

| program | base median s | t7b median s | t7b d% |
|---|---:|---:|---:|
| alloc | 2.019 | 1.995 | -1.19 |
| alloc4c | 0.565 | 0.574 | +1.59 |
| arith | 1.229 | 1.229 | +0.00 |
| assign | 1.070 | 1.089 | +1.78 |
| compound | 0.646 | 0.638 | -1.24 |
| decloop | 0.246 | 0.267 | +8.54 |
| decrender | 0.450 | 0.450 | +0.00 |
| dispatch | 1.789 | 1.792 | +0.17 |
| dispatchclass | 1.378 | 1.305 | -5.30 |
| emptyloop | 0.529 | 0.519 | -1.89 |
| extcall | 0.932 | 0.867 | -6.97 |
| fibcall | 0.805 | 0.789 | -1.99 |
| fibfunc | 0.790 | 0.774 | -2.03 |
| heapshape | 0.323 | 0.333 | +3.10 |
| nop | 0.654 | 0.563 | -13.91 |
| parse | 0.130 | 0.129 | -0.77 |
| sayloop | 0.029 | 0.028 | -3.45 |
| sendloop | 1.077 | 1.057 | -1.86 |
| startup | 0.024 | 0.025 | +4.17 |
| strings | 1.144 | 1.142 | -0.17 |
| textnum | 0.092 | 0.092 | +0.00 |
| varlookup | 0.782 | 0.780 | -0.26 |
| rexxcps | 2.007 | 1.971 | -1.79 |

```
$B/wallclock.sh -r 5 -o $S/wall2 base=$S/bin/base/rexx-run t7b=$S/bin/t7b/rexx-run base2=$S/bin/base2/rexx-run
```

| program | base median s | t7b median s | base2 median s | t7b d% | base2 d% |
|---|---:|---:|---:|---:|---:|
| alloc | 2.013 | 1.993 | 2.016 | -0.99 | +0.15 |
| alloc4c | 0.568 | 0.570 | 0.568 | +0.35 | +0.00 |
| arith | 1.258 | 1.228 | 1.211 | -2.38 | -3.74 |
| assign | 1.069 | 1.089 | 1.070 | +1.87 | +0.09 |
| compound | 0.647 | 0.639 | 0.658 | -1.24 | +1.70 |
| decloop | 0.249 | 0.255 | 0.250 | +2.41 | +0.40 |
| decrender | 0.451 | 0.451 | 0.450 | +0.00 | -0.22 |
| dispatch | 1.823 | 1.832 | 1.779 | +0.49 | -2.41 |
| dispatchclass | 1.388 | 1.297 | 1.387 | -6.56 | -0.07 |
| emptyloop | 0.527 | 0.519 | 0.529 | -1.52 | +0.38 |
| extcall | 0.924 | 0.866 | 0.923 | -6.28 | -0.11 |
| fibcall | 0.789 | 0.785 | 0.789 | -0.51 | +0.00 |
| fibfunc | 0.782 | 0.775 | 0.794 | -0.90 | +1.53 |
| heapshape | 0.320 | 0.334 | 0.324 | +4.37 | +1.25 |
| nop | 0.660 | 0.571 | 0.663 | -13.48 | +0.45 |
| parse | 0.129 | 0.129 | 0.130 | +0.00 | +0.78 |
| sayloop | 0.030 | 0.029 | 0.029 | -3.33 | -3.33 |
| sendloop | 1.073 | 1.048 | 1.089 | -2.33 | +1.49 |
| startup | 0.026 | 0.024 | 0.025 | -7.69 | -3.85 |
| strings | 1.136 | 1.144 | 1.146 | +0.70 | +0.88 |
| textnum | 0.092 | 0.093 | 0.091 | +1.09 | -1.09 |
| varlookup | 0.786 | 0.777 | 0.779 | -1.15 | -0.89 |
| rexxcps | 2.011 | 1.984 | 2.057 | -1.34 | +2.29 |

### Round 3 (fix round 1, `d112d68f9`)

```
git archive d112d68f9 | tar -x -C $S/trees/t7c                                 # then the build and cp steps above, NAME = t7c
$B/callgrind.sh -r 3 -j 12 -o $S/cg3 base=$S/bin/base/rexx-run t7b=$S/bin/t7b/rexx-run t7c=$S/bin/t7c/rexx-run
$B/wallclock.sh -r 5 -o $S/wall3 base=$S/bin/base/rexx-run t7c=$S/bin/t7c/rexx-run base2=$S/bin/base2/rexx-run
```

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| t7c | `22cea71ac7fdc3c6daf50af2497bcb625d9b51f4f66aa432ef4b9267061de28e` | 2,725,275 |

Callgrind exit 0, no SPREAD line.

| program | base | t7b - base | t7c - base | t7c d% |
|---|---:|---:|---:|---:|
| alloc | 25,168,678,363 | -104,700,691 | -74,706,383 | -0.2968% |
| alloc4c | 3,224,395,040 | -3,901,073 | -9,891,783 | -0.3068% |
| arith | 11,520,060,059 | -7,933,454 | -2,939,160 | -0.0255% |
| assign | 19,637,557,619 | -198,141,606 | +2,852,702 | +0.0145% |
| compound | 9,271,611,424 | -19,578,129 | +20,418,682 | +0.2202% |
| decloop | 2,565,175,706 | -9,137,885 | -8,343,577 | -0.3253% |
| decrender | 4,348,396,202 | -10,692,672 | -20,298,364 | -0.4668% |
| dispatch | 20,483,285,061 | -189,396,578 | -29,402,152 | -0.1435% |
| dispatchclass | 15,850,195,601 | -135,654,255 | -3,659,887 | -0.0231% |
| emptyloop | 9,308,098,320 | -73,585,862 | -48,591,556 | -0.5220% |
| extcall | 8,281,259,519 | +12,168,197 | +30,162,506 | +0.3642% |
| fibcall | 8,345,686,416 | -79,602,838 | -9,973,296 | -0.1195% |
| fibfunc | 7,988,911,762 | -76,237,248 | -10,046,396 | -0.1258% |
| heapshape | 3,278,634,105 | -13,456,840 | -9,497,770 | -0.2897% |
| nop | 9,437,300,169 | -198,141,349 | -197,147,043 | -2.0890% |
| parse | 1,541,986,626 | -1,566,696 | -172,387 | -0.0112% |
| sayloop | 114,813,367 | -296,049 | -201,743 | -0.1757% |
| sendloop | 13,863,190,791 | -144,718,793 | +15,275,574 | +0.1102% |
| startup | 58,074,056 | -1,686 | -7,383 | -0.0127% |
| strings | 17,749,287,481 | -23,568,139 | -26,573,830 | -0.1497% |
| textnum | 1,176,579,478 | -3,133,743 | -339,435 | -0.0288% |
| varlookup | 14,878,111,694 | -93,387,627 | +1,606,681 | +0.0108% |
| rexxcps | 17,817,337,154 | -49,697,346 | -42,077,825 | -0.2362% |

| program | base median s | t7c median s | base2 median s | t7c d% | base2 d% |
|---|---:|---:|---:|---:|---:|
| alloc | 2.031 | 2.038 | 2.036 | +0.34 | +0.25 |
| alloc4c | 0.575 | 0.578 | 0.570 | +0.52 | -0.87 |
| arith | 1.228 | 1.267 | 1.231 | +3.18 | +0.24 |
| assign | 1.069 | 1.114 | 1.071 | +4.21 | +0.19 |
| compound | 0.645 | 0.649 | 0.645 | +0.62 | +0.00 |
| decloop | 0.247 | 0.251 | 0.243 | +1.62 | -1.62 |
| decrender | 0.448 | 0.435 | 0.455 | -2.90 | +1.56 |
| dispatch | 1.774 | 1.744 | 1.787 | -1.69 | +0.73 |
| dispatchclass | 1.385 | 1.374 | 1.401 | -0.79 | +1.16 |
| emptyloop | 0.527 | 0.520 | 0.527 | -1.33 | +0.00 |
| extcall | 0.918 | 0.917 | 0.923 | -0.11 | +0.54 |
| fibcall | 0.794 | 0.821 | 0.793 | +3.40 | -0.13 |
| fibfunc | 0.783 | 0.798 | 0.781 | +1.92 | -0.26 |
| heapshape | 0.319 | 0.324 | 0.322 | +1.57 | +0.94 |
| nop | 0.660 | 0.558 | 0.660 | -15.45 | +0.00 |
| parse | 0.130 | 0.133 | 0.133 | +2.31 | +2.31 |
| sayloop | 0.030 | 0.028 | 0.029 | -6.67 | -3.33 |
| sendloop | 1.079 | 1.083 | 1.069 | +0.37 | -0.93 |
| startup | 0.025 | 0.025 | 0.025 | +0.00 | +0.00 |
| strings | 1.140 | 1.181 | 1.147 | +3.60 | +0.61 |
| textnum | 0.094 | 0.094 | 0.091 | +0.00 | -3.19 |
| varlookup | 0.781 | 0.785 | 0.780 | +0.51 | -0.13 |
| rexxcps | 2.024 | 1.940 | 2.055 | -4.15 | +1.53 |

Verdict at `d112d68f9`: every program inside +0.3% except `extcall` at +0.3642% (round 3 of 3, reported for a ruling); wall clock inside each bar except `assign` at +4.21% (Ir +0.0145%).

## Task 8

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t8`; `T7` is Task 7's.

```
cp $T7/bin/base/rexx-run $S/bin/base/rexx-run      # Task 7's base build, sha checked against its table
cp $T7/bin/base/rexx-run $S/bin/base2/rexx-run     # identical-binary control
cp $T7/bin/t7c/rexx-run $S/bin/prev/rexx-run       # Task 7's d112d68f9 build; 8d26923bb changed only docs
git archive REV | tar -x -C $S/trees/NAME           # t8 4822d01e0, t8r1 1f976e121
find $S/trees/NAME -type f -exec touch {} +
cd $S/trees/NAME/rust && CARGO_TARGET_DIR=$S/tgt/NAME cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-NAME.log 2>&1
/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-NAME.log                 # 1 for each NAME
cp $S/tgt/NAME/release/rexx-run $S/bin/NAME/rexx-run
sha256sum < $S/bin/NAME/rexx-run
objcopy -O binary --only-section=.text $S/bin/NAME/rexx-run /dev/stdout | wc -c
```

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `d59c32248068af0bb927e2f2b714b1bffca4b17d32231c884f812b4bd40005b6` | 2,732,523 |
| prev | `22cea71ac7fdc3c6daf50af2497bcb625d9b51f4f66aa432ef4b9267061de28e` | 2,725,275 |
| t8 | `7ccbbe2c4d879db7b813a743b02c84f85b75c937aaa4db259cc3b59219ba0025` | 2,763,803 |
| t8r1 | `0c9a7a3b6bc9d1d5db4c1fa0654f7f7090fb2a97ea53bdc063e1e9d569876423` | 2,772,475 |

### Instruction counts

```
$B/callgrind.sh -r 3 -j 8 -o $S/cg2 base=$S/bin/base/rexx-run prev=$S/bin/prev/rexx-run t8=$S/bin/t8/rexx-run t8r1=$S/bin/t8r1/rexx-run
```

Exit 0, no SPREAD line. Instructions less libc.so.6 and ld-linux, median of three rounds, delta
against base. `t8` is the implementation, `t8r1` round 1.

| program | base | prev d% | t8 d% | t8r1 d% |
|---|---:|---:|---:|---:|
| alloc | 25,168,678,399 | -0.2968% | +0.0369% | -0.1061% |
| alloc4c | 3,224,395,037 | -0.3068% | +0.4066% | -1.5782% |
| arith | 11,520,060,095 | -0.0255% | -0.0125% | -0.3640% |
| assign | 19,637,557,681 | +0.0145% | -2.5326% | -5.0777% |
| compound | 9,271,611,428 | +0.2202% | +0.1122% | -0.9662% |
| decloop | 2,565,175,703 | -0.3253% | -0.4968% | -0.8085% |
| decrender | 4,348,396,238 | -0.4668% | -0.1265% | -1.3222% |
| dispatch | 20,483,285,065 | -0.1435% | +0.9790% | +0.3201% |
| dispatchclass | 15,850,195,637 | -0.0231% | +1.2638% | +0.1535% |
| emptyloop | 9,308,098,356 | -0.5220% | +0.2827% | -1.0592% |
| extcall | 8,281,259,516 | +0.3642% | +1.5959% | +0.9077% |
| fibcall | 8,345,686,413 | -0.1195% | +2.8674% | +4.6498% |
| fibfunc | 7,988,911,759 | -0.1258% | +4.6319% | +5.2548% |
| heapshape | 3,278,634,141 | -0.2897% | -0.2285% | -0.3506% |
| nop | 9,437,300,166 | -2.0890% | +1.0878% | -4.2082% |
| parse | 1,541,986,662 | -0.0112% | +1.5581% | -0.4649% |
| sayloop | 114,813,364 | -0.1758% | +0.4346% | -0.2599% |
| sendloop | 13,863,190,795 | +0.1102% | +4.9792% | +3.0677% |
| startup | 58,074,053 | -0.0129% | -0.0106% | -0.0069% |
| strings | 17,749,287,478 | -0.1497% | +1.7262% | -1.5019% |
| textnum | 1,176,579,475 | -0.0289% | +1.0759% | -1.6604% |
| varlookup | 14,878,111,730 | +0.0108% | -0.7559% | -2.6710% |
| rexxcps | 17,817,347,549 | -0.2362% | +1.9752% | +0.7581% |

### Wall clock

```
$B/wallclock.sh -r 5 -o $S/wall base=$S/bin/base/rexx-run t8r1=$S/bin/t8r1/rexx-run base2=$S/bin/base2/rexx-run
```

Load average 1.09 4.13 7.35 at start, 1.15 2.23 5.64 at end.

| program | base median s | t8r1 median s | base2 median s | t8r1 d% | base2 d% |
|---|---:|---:|---:|---:|---:|
| alloc | 2.045 | 2.137 | 2.027 | +4.50 | -0.88 |
| alloc4c | 0.570 | 0.570 | 0.569 | +0.00 | -0.18 |
| arith | 1.261 | 1.260 | 1.247 | -0.08 | -1.11 |
| assign | 1.074 | 1.049 | 1.071 | -2.33 | -0.28 |
| compound | 0.644 | 0.637 | 0.637 | -1.09 | -1.09 |
| decloop | 0.249 | 0.252 | 0.249 | +1.20 | +0.00 |
| decrender | 0.445 | 0.441 | 0.447 | -0.90 | +0.45 |
| dispatch | 1.764 | 1.961 | 1.768 | +11.17 | +0.23 |
| dispatchclass | 1.392 | 1.563 | 1.365 | +12.28 | -1.94 |
| emptyloop | 0.528 | 0.518 | 0.527 | -1.89 | -0.19 |
| extcall | 0.925 | 0.932 | 0.917 | +0.76 | -0.86 |
| fibcall | 0.793 | 0.852 | 0.793 | +7.44 | +0.00 |
| fibfunc | 0.784 | 0.869 | 0.794 | +10.84 | +1.28 |
| heapshape | 0.322 | 0.331 | 0.328 | +2.80 | +1.86 |
| nop | 0.666 | 0.646 | 0.660 | -3.00 | -0.90 |
| parse | 0.132 | 0.128 | 0.131 | -3.03 | -0.76 |
| sayloop | 0.029 | 0.028 | 0.030 | -3.45 | +3.45 |
| sendloop | 1.096 | 1.172 | 1.093 | +6.93 | -0.27 |
| startup | 0.026 | 0.024 | 0.025 | -7.69 | -3.85 |
| strings | 1.131 | 1.102 | 1.136 | -2.56 | +0.44 |
| textnum | 0.093 | 0.091 | 0.092 | -2.15 | -1.08 |
| varlookup | 0.775 | 0.765 | 0.776 | -1.29 | +0.13 |
| rexxcps | 2.028 | 2.036 | 2.019 | +0.39 | -0.44 |

Verdict: over the S1 budget. t8r1 exceeds +0.3% on fibcall (+4.65%), fibfunc (+5.25%), sendloop
(+3.07%), extcall (+0.91%, of which +0.36% is Task 7's), rexxcps (+0.76%) and dispatch (+0.32%);
wall clock is over its bar on fibcall, fibfunc, sendloop, dispatchclass and alloc.

### Rounds 2 and 3

`t8r2` `7d7ace817`, `t8r3` `5b8c2af83`, each built by the recipe above.

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| t8r2 | `03e58dc39f25357721d4bf89d21784c18e0ab24561362ac585575fc707875b34` | 2,773,371 |
| t8r3 | `8f0c7a431a9ce76e6fc0c65565714ae5de5d721cdcb800f96303c2cd018e2fec` | 2,773,163 |

```
$B/callgrind.sh -r 3 -j 8 -o $S/cg4 base=$S/bin/base/rexx-run t8r2=$S/bin/t8r2/rexx-run t8r3=$S/bin/t8r3/rexx-run
```

Exit 0, no SPREAD line. Same measure as above; base as in the table above except rexxcps
17,817,346,217.

| program | t8r2 d% | t8r3 d% |
|---|---:|---:|
| alloc | -0.1061% | -0.1061% |
| alloc4c | -1.5782% | -1.5782% |
| arith | -0.3640% | -0.3640% |
| assign | -5.0777% | -5.0777% |
| compound | -0.9662% | -0.9662% |
| decloop | -0.8085% | -0.8085% |
| decrender | -1.3222% | -1.3222% |
| dispatch | -1.1204% | -1.1204% |
| dispatchclass | -1.2093% | -1.2093% |
| emptyloop | -1.0592% | -1.0592% |
| extcall | +0.9077% | -0.9761% |
| fibcall | +4.2376% | +2.9603% |
| fibfunc | +4.8242% | +3.6404% |
| heapshape | -0.3507% | -0.3507% |
| nop | -4.2082% | -4.2082% |
| parse | -0.4649% | -0.4649% |
| sayloop | -0.2600% | -0.2600% |
| sendloop | +1.1201% | +1.1201% |
| startup | -0.0072% | -0.0072% |
| strings | -1.5019% | -1.5019% |
| textnum | -1.6604% | -1.6604% |
| varlookup | -2.6710% | -2.6710% |
| rexxcps | +0.7251% | +0.6277% |

```
$B/wallclock.sh -r 5 -o $S/wall3 base=$S/bin/base/rexx-run t8r3=$S/bin/t8r3/rexx-run base2=$S/bin/base2/rexx-run
```

Started once the one-minute load average was under 2: load 1.92 9.13 13.31 at start, 1.31 4.39
10.15 at end. An earlier run (`$S/wall2`, load 4.50 11.14 14.16 at start) had base2 at +10.75% on
compound and +10.25% on dispatch and is not used.

| program | base median s | t8r3 median s | base2 median s | t8r3 d% | base2 d% |
|---|---:|---:|---:|---:|---:|
| alloc | 2.022 | 2.074 | 2.036 | +2.57 | +0.69 |
| alloc4c | 0.585 | 0.594 | 0.580 | +1.54 | -0.85 |
| arith | 1.242 | 1.243 | 1.235 | +0.08 | -0.56 |
| assign | 1.081 | 1.056 | 1.069 | -2.31 | -1.11 |
| compound | 0.649 | 0.641 | 0.644 | -1.23 | -0.77 |
| decloop | 0.252 | 0.251 | 0.245 | -0.40 | -2.78 |
| decrender | 0.457 | 0.443 | 0.452 | -3.06 | -1.09 |
| dispatch | 1.768 | 1.784 | 1.764 | +0.90 | -0.23 |
| dispatchclass | 1.395 | 1.433 | 1.398 | +2.72 | +0.22 |
| emptyloop | 0.528 | 0.517 | 0.527 | -2.08 | -0.19 |
| extcall | 0.922 | 0.899 | 0.941 | -2.49 | +2.06 |
| fibcall | 0.791 | 0.867 | 0.813 | +9.61 | +2.78 |
| fibfunc | 0.805 | 0.890 | 0.790 | +10.56 | -1.86 |
| heapshape | 0.318 | 0.331 | 0.323 | +4.09 | +1.57 |
| nop | 0.654 | 0.638 | 0.656 | -2.45 | +0.31 |
| parse | 0.132 | 0.129 | 0.131 | -2.27 | -0.76 |
| sayloop | 0.029 | 0.028 | 0.029 | -3.45 | +0.00 |
| sendloop | 1.099 | 1.113 | 1.070 | +1.27 | -2.64 |
| startup | 0.025 | 0.025 | 0.025 | +0.00 | +0.00 |
| strings | 1.130 | 1.124 | 1.131 | -0.53 | +0.09 |
| textnum | 0.092 | 0.092 | 0.092 | +0.00 | +0.00 |
| varlookup | 0.777 | 0.764 | 0.780 | -1.67 | +0.39 |
| rexxcps | 2.030 | 2.025 | 2.077 | -0.25 | +2.32 |

Verdict: over the S1 budget. t8r3 exceeds +0.3% on fibcall (+2.96%), fibfunc (+3.64%), sendloop
(+1.12%) and rexxcps (+0.63%); wall clock is over its bar on fibcall (+9.61%), fibfunc (+10.56%)
and heapshape (+4.09%, base2 +1.57%).

### Fix round 1

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t8-fix`.
`base` `1754a3b5a`, `t8r3` `5b8c2af83`, `fix1` `8428ca61b`, each built by the recipe above in its own
`CARGO_TARGET_DIR` (`$S/tgt/NAME`), one `Compiling rexx-exec` line each.

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `9845d61cd833e7b21a7f64c7a13c7da492b2e5ede145b25c98942f7e69dfab53` | 2,732,523 |
| t8r3 | `a0086755882c1b2b6cb5539b70574a2bd2f1c1c71ff81c64bd6dd83b73bcbbce` | 2,773,163 |
| fix1 | `ef62c584ca23128752ce664899379cb1b5b580ae356f42de0d6178fa931ae285` | 2,771,147 |

```
$B/callgrind.sh -r 3 -j 8 -o $S/cg -p "fibcall fibfunc sendloop extcall rexxcps dispatch" base=$S/bin/base/rexx-run t8r3=$S/bin/t8r3/rexx-run fix1=$S/bin/fix1/rexx-run
```

Exit 0, no SPREAD line. Load 11.04 at start.

| program | base | t8r3 | fix1 | t8r3 d% | fix1 d% |
|---|---:|---:|---:|---:|---:|
| fibcall | 8,345,686,512 | 8,592,742,497 | 8,592,742,497 | +2.9603% | +2.9603% |
| fibfunc | 7,988,911,858 | 8,279,742,617 | 8,279,742,617 | +3.6404% | +3.6404% |
| sendloop | 13,863,190,779 | 14,018,470,066 | 14,018,470,066 | +1.1201% | +1.1201% |
| extcall | 8,281,259,615 | 8,200,425,304 | 8,200,425,304 | -0.9761% | -0.9761% |
| rexxcps | 17,817,333,657 | 17,929,181,388 | 17,905,659,928 | +0.6277% | +0.4957% |
| dispatch | 20,483,285,049 | 20,253,789,393 | 20,253,789,393 | -1.1204% | -1.1204% |

## Task 9

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t9`;
`B` is `rust/bench-programs`. Each binary built by Task 8's recipe from `git archive REV` into
`$S/trees/NAME`, touched, in its own `CARGO_TARGET_DIR` (`$S/tgt/NAME`), one `Compiling rexx-exec`
line each: `base` `1754a3b5a`, `prev` `4b0128186` (Task 8 fix round 1), `t9` `805a1fb5d` (the
implementation), `r1` `31f4a5485`, `r2` `54b07d745`, `r3` `f00c8921e` (perf rounds 1 to 3).

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| base | `8486d7ab22f995eaf47ba750a4a18987f7aac55892071c42ca57b3f48b271876` | 2,732,523 |
| prev | `9cc21fe6a599f13f83584933ee7677292fb417b2a64280d962c0e2f367ed1ac8` | 2,771,147 |
| t9 | `80b8774fcb705b75fd501b6cfd31983b3cdc2a19ad992155221c056d92f5594c` | 2,799,323 |
| r1 | `938d5a1187fc1d081e53e0a1ad04034e4d90b804d3593d41acc3b94ea84d16e9` | 2,799,579 |
| r2 | `edfa53ae8b625aade70a62a4886778e6e5bca3c16bc564c00a64050b24c06294` | 2,799,995 |
| r3 | `b9b262261d36b48659bc97b37b0c478910975812aa400a61735d7b404022924f` | 2,800,331 |

### Instruction counts

```
$B/callgrind.sh -r 3 -j 8 -o $S/cg-full1 base=$S/bin/base/rexx-run prev=$S/bin/prev/rexx-run t9=$S/bin/t9/rexx-run r1=$S/bin/r1/rexx-run
$B/callgrind.sh -r 3 -j 8 -o $S/cg-full2 base=$S/bin/base/rexx-run r2=$S/bin/r2/rexx-run r3=$S/bin/r3/rexx-run
```

Both exit 0, no SPREAD line. Median of three, libc and ld-linux subtracted, delta against base.

| program | base | prev d% | t9 d% | r1 d% | r2 d% | r3 d% |
|---|---:|---:|---:|---:|---:|---:|
| alloc | 25,168,678,345 | -0.1061% | -14.1948% | -18.5217% | -18.6170% | -18.6170% |
| alloc4c | 3,224,394,975 | -1.5782% | +0.2829% | -0.9583% | -1.6716% | -1.6716% |
| arith | 11,520,060,041 | -0.3640% | +0.1746% | -0.2161% | -0.4331% | -0.4331% |
| assign | 19,637,557,664 | -5.0777% | -1.0048% | -3.0408% | -4.5685% | -4.5685% |
| compound | 9,271,611,173 | -0.9662% | +1.3528% | -0.0494% | -0.6426% | -0.6426% |
| decloop | 2,565,175,641 | -0.8085% | +0.1906% | -0.4806% | -0.9017% | -0.9017% |
| decrender | 4,348,396,184 | -1.3222% | +0.2422% | -0.8896% | -1.4323% | -1.4323% |
| dispatch | 20,483,284,810 | -1.1204% | +9.0587% | -0.1682% | -0.1926% | -1.1690% |
| dispatchclass | 15,850,195,583 | -1.2093% | +8.1030% | -1.8402% | -1.9663% | -2.9758% |
| emptyloop | 9,308,098,302 | -1.0592% | -0.5229% | -0.7906% | -1.0592% | -1.0592% |
| extcall | 8,281,259,454 | -0.9761% | -0.4687% | -0.7226% | -1.0124% | -1.0124% |
| fibcall | 8,345,686,351 | +2.9603% | +3.9723% | +3.1173% | +2.6435% | +2.6435% |
| fibfunc | 7,988,911,697 | +3.6404% | +4.7423% | +4.1504% | +3.8706% | +3.8706% |
| heapshape | 3,278,634,087 | -0.3507% | -22.0866% | -27.2255% | -27.3480% | -27.3480% |
| nop | 9,437,300,104 | -4.2082% | -2.0909% | -3.1486% | -4.2083% | -4.2083% |
| parse | 1,541,986,608 | -0.4649% | +0.7040% | +0.0284% | -0.5941% | -0.5941% |
| sayloop | 114,813,302 | -0.2600% | +0.3642% | +0.0856% | -0.3498% | -0.3499% |
| sendloop | 13,863,190,540 | +1.1201% | +16.3404% | +3.0316% | +2.9955% | +1.5529% |
| startup | 58,073,991 | -0.0072% | +0.0221% | -0.0127% | -0.0125% | -0.0126% |
| strings | 17,749,287,416 | -1.5019% | +0.4760% | -0.9268% | -1.9240% | -1.9240% |
| textnum | 1,176,579,413 | -1.6604% | +1.0435% | -0.8958% | -2.0177% | -2.0177% |
| varlookup | 14,878,111,676 | -2.6710% | +0.6490% | -1.5217% | -2.6710% | -2.6710% |
| rexxcps | 17,817,352,350 | +0.4956% | +2.0317% | +1.2287% | +0.4998% | +0.4997% |

(`rexxcps`' base is 17,817,341,052 in the second run; its deltas there are against that.)

Verdict: over the S1 budget on fibcall (+2.64%), fibfunc (+3.87%), sendloop (+1.55%) and rexxcps
(+0.50%); every other program is below base.

- `t9`: a stackless send cost about 422 instructions more than Task 8's recursive one on sendloop,
  and the new region arm cost clauses that run no send (nop +2.21%, varlookup +1.18%, rexxcps
  +1.53% against prev, measured on the work tree with the SELF/SUPER change).
- Round 1 (`r1`): SELF/SUPER slots on the plan (-216 per send), the behaviour computed once per
  send and a per-chunk send-site name table (-154 per send), one Park construction per region
  (nop +2.21% -> +1.11% against prev). Variants measured and not committed: without the List arm
  (no change), the send arm placed after CallArgs (no change), a merged CallArgs/Send arm (worse
  on fibfunc +1.24%), the header boxing out of line (fibcall +0.41%), the arm passing the op's
  fields (sendloop worse).
- Round 2 (`r2`): the send arm answering `Started<ObjRef>` like the call arms: nop, varlookup and
  rexxcps back to prev's counts.
- Round 3 (`r3`): `plan_for` answering its last entry first (-40 per send).
- alloc's sends no longer evaluate through the tree: against prev, `eval_node`, `message_term` and
  `eval_chunk_expr` retire 5.76 billion fewer instructions between them (`cgdiff.py` on
  `cg-full1/alloc.prev.r1.cg` and `cg-full2/alloc.r3.r1.cg`). heapshape was not broken down.

### Final commit and wall clock

`fin` is `54c4c28e5` (the seam fix after round 3; gates ran on it), built by the same recipe:
sha256 `3f780e78b64f6764001890228aa2f220653d10ab12757a5251a1bcbb910d1b08`, `.text` 2,800,203 bytes.
`$B/callgrind.sh -r 1 -j 8 -o $S/cg-fin -p "sendloop dispatch dispatchclass fibcall fibfunc rexxcps nop" r3=... fin=...`:
sendloop -0.3197%, dispatch -0.2223%, dispatchclass -0.2341% against r3, the others within 6,000
instructions. So against base: sendloop 14,033,461,969 (+1.23%).

```
$B/wallclock.sh -r 5 -o $S/wall base=$S/bin/base/rexx-run fin=$S/bin/fin/rexx-run base2=$S/bin/base2/rexx-run
```

`base2` is a copy of `base`. Started once the one-minute load was under 2: load 1.96 4.06 8.54 at
start, 1.95 2.76 6.75 at end, exit 0.

| program | base median s | fin median s | base2 median s | fin d% | base2 d% |
|---|---:|---:|---:|---:|---:|
| alloc | 2.028 | 1.651 | 2.036 | -18.59 | +0.39 |
| alloc4c | 0.605 | 0.603 | 0.601 | -0.33 | -0.66 |
| arith | 1.225 | 1.306 | 1.238 | +6.61 | +1.06 |
| assign | 1.076 | 1.098 | 1.082 | +2.04 | +0.56 |
| compound | 0.644 | 0.640 | 0.647 | -0.62 | +0.47 |
| decloop | 0.255 | 0.246 | 0.249 | -3.53 | -2.35 |
| decrender | 0.460 | 0.445 | 0.454 | -3.26 | -1.30 |
| dispatch | 1.804 | 2.146 | 1.781 | +18.96 | -1.27 |
| dispatchclass | 1.371 | 1.492 | 1.364 | +8.83 | -0.51 |
| emptyloop | 0.525 | 0.531 | 0.531 | +1.14 | +1.14 |
| extcall | 0.970 | 0.907 | 0.984 | -6.49 | +1.44 |
| fibcall | 0.809 | 0.832 | 0.802 | +2.84 | -0.87 |
| fibfunc | 0.789 | 0.854 | 0.787 | +8.24 | -0.25 |
| heapshape | 0.321 | 0.190 | 0.326 | -40.81 | +1.56 |
| nop | 0.667 | 0.569 | 0.663 | -14.69 | -0.60 |
| parse | 0.129 | 0.127 | 0.131 | -1.55 | +1.55 |
| sayloop | 0.029 | 0.029 | 0.029 | +0.00 | +0.00 |
| sendloop | 1.073 | 1.284 | 1.064 | +19.66 | -0.84 |
| startup | 0.024 | 0.025 | 0.026 | +4.17 | +8.33 |
| strings | 1.134 | 1.129 | 1.129 | -0.44 | -0.44 |
| textnum | 0.093 | 0.096 | 0.095 | +3.23 | +2.15 |
| varlookup | 0.782 | 0.776 | 0.785 | -0.77 | +0.38 |
| rexxcps | 2.043 | 2.006 | 2.026 | -1.81 | -0.83 |

Wall is over its +-4% bar on sendloop (+19.66%), dispatch (+18.96%), dispatchclass (+8.83%),
fibfunc (+8.24%) and arith (+6.61%, instructions -0.43%; base2 +1.06%).

`perf stat -x, -e EVENT rexx-run bench-programs/sendloop.rex`, one event per run, base, fin, base:

| event | base | fin | base (again) |
|---|---:|---:|---:|
| cycles | 3,146,932,706 | 3,693,099,344 | 3,160,010,451 |
| instructions | 14,991,267,637 | 15,150,479,569 | 14,991,035,956 |
| L1-icache-load-misses | 838,924 | 55,633,195 | 954,665 |
| branch-misses | 340,844 | 358,712 | 3,097,434 |

About eleven L1 instruction-cache misses per send against none on base.

## S1 front-end round

`S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-perf1`;
`B` is `rust/bench-programs`. Each binary built from `git archive REV` into `$S/trees/NAME`, touched,
in its own `CARGO_TARGET_DIR` (`$S/tgt/NAME`), one `Compiling rexx-exec` line each (`$S/mkrev.sh
NAME REV`). `base` is Task 9's `1754a3b5a` binary (sha256 `8486d7ab...`, as recorded there),
`head` `2050300f4`, `r1` `49a55bea9`.

| name | sha256 of `rexx-run` | `.text` bytes |
|---|---|---:|
| head | `466cb65a579f1bc414c23d1c4d361ddf9e33775c7820e1e5da46ef091e870adb` | 2,801,371 |
| r1 | `64304a7794c4e18ff4569d4a5606c1468e01f26a068ecf32059b546dc8610509` | 2,822,475 |

### Round 1 (`49a55bea9`)

What changed: a region holding `CallExpr`, `CallArgs` or `Send` compiles as `Op::CallingClause`;
the op loop runs `Op::Clause` through one region expansion and `Op::CallingClause` plus every
resumed region through a second, in `ops_loop_steady` (`resume_region` only delivers the answer).
A body's first non-label clause is found at entry and reached through the loop's own bound check
(`grant_for`); the granting instance is a cold fallback. `begin_invoke` keeps the Rexx and native
arms inline.

```
$B/callgrind.sh -r 3 -j 8 -o $S/cg-r1 base=$S/bin/base/rexx-run head=$S/bin/head/rexx-run r1=$S/bin/r1/rexx-run
```

Exit 0, no SPREAD. Median of three, libc and ld-linux subtracted.

| program | base | head d% | r1 d% |
|---|---:|---:|---:|
| alloc | 25,168,678,669 | -19.8924% | -19.5468% |
| alloc4c | 3,224,394,975 | -1.6717% | -1.5785% |
| arith | 11,520,060,365 | -0.4331% | -0.4244% |
| assign | 19,637,557,677 | -4.5685% | -4.0593% |
| compound | 9,271,611,408 | -0.6427% | -0.9123% |
| decloop | 2,565,175,641 | -0.9017% | -1.2446% |
| decrender | 4,348,396,508 | -1.4324% | -1.7082% |
| dispatch | 20,483,285,045 | -1.3887% | +0.1248% |
| dispatchclass | 15,850,195,907 | -3.2029% | -2.2439% |
| emptyloop | 9,308,098,626 | -1.0592% | -0.5220% |
| extcall | 8,281,259,454 | -1.0124% | -0.6501% |
| fibcall | 8,345,686,351 | +2.6435% | +2.1182% |
| fibfunc | 7,988,911,697 | +3.8706% | +4.2995% |
| heapshape | 3,278,634,411 | -27.9298% | -27.5934% |
| nop | 9,437,300,104 | -4.2083% | -2.0890% |
| parse | 1,541,986,932 | -0.5942% | -0.5680% |
| sayloop | 114,813,302 | -0.3510% | -0.0861% |
| sendloop | 13,863,190,775 | +1.2283% | -1.0800% |
| startup | 58,073,991 | -0.0148% | -0.0076% |
| strings | 17,749,287,416 | -1.9240% | -1.4677% |
| textnum | 1,176,579,413 | -2.0178% | -2.0854% |
| varlookup | 14,878,112,000 | -2.6710% | -2.7987% |
| rexxcps | 17,817,347,405 | +0.4997% | +0.0939% |

```
$S/pstat.sh L1-icache-load-misses "sendloop dispatch dispatchclass fibfunc fibcall rexxcps alloc" base head r1
$S/pstat.sh r20000048F "sendloop dispatch dispatchclass fibfunc fibcall rexxcps alloc" base head r1
```

`perf stat -x, -e EVENT`, one run per program per binary, output in `$S/r1-pstat2.txt`; load 0.58
at start, 0.85 at end. (A first run at load 3.11 read base's sendloop at 34.6M L1i misses and is
discarded.)

| program | L1i base | L1i head | L1i r1 | op-cache base | op-cache head | op-cache r1 |
|---|---:|---:|---:|---:|---:|---:|
| sendloop | 1.0M | 41.2M | 69.9M | 268.2M | 471.8M | 735.5M |
| dispatch | 145.3M | 253.4M | 163.8M | 998.2M | 1560.9M | 1160.4M |
| dispatchclass | 106.7M | 123.0M | 117.9M | 738.4M | 826.9M | 906.4M |
| fibfunc | 44.9M | 89.2M | 93.8M | 436.5M | 557.8M | 606.5M |
| fibcall | 51.0M | 75.1M | 65.0M | 447.0M | 454.5M | 444.8M |
| rexxcps | 175.3M | 218.8M | 205.4M | 1711.3M | 1674.5M | 1803.2M |
| alloc | 3.6M | 26.9M | 7.0M | 544.5M | 540.0M | 325.5M |

```
$B/wallclock.sh -r 5 -o $S/wall-r1 base=$S/bin/base/rexx-run head=$S/bin/head/rexx-run r1=$S/bin/r1/rexx-run
```

Load 1.68 at start, 0.97 at end; exit 0.

| program | base s | head s | r1 s | head d% | r1 d% |
|---|---:|---:|---:|---:|---:|
| alloc | 2.059 | 1.669 | 1.588 | -18.94 | -22.88 |
| alloc4c | 0.566 | 0.561 | 0.561 | -0.88 | -0.88 |
| arith | 1.229 | 1.315 | 1.319 | +7.00 | +7.32 |
| assign | 1.068 | 1.084 | 1.036 | +1.50 | -3.00 |
| compound | 0.640 | 0.646 | 0.636 | +0.94 | -0.63 |
| decloop | 0.255 | 0.241 | 0.250 | -5.49 | -1.96 |
| decrender | 0.451 | 0.437 | 0.468 | -3.10 | +3.77 |
| dispatch | 1.772 | 2.106 | 1.876 | +18.85 | +5.87 |
| dispatchclass | 1.379 | 1.454 | 1.476 | +5.44 | +7.03 |
| emptyloop | 0.526 | 0.517 | 0.512 | -1.71 | -2.66 |
| extcall | 0.971 | 0.866 | 0.866 | -10.81 | -10.81 |
| fibcall | 0.794 | 0.814 | 0.842 | +2.52 | +6.05 |
| fibfunc | 0.790 | 0.862 | 0.860 | +9.11 | +8.86 |
| heapshape | 0.320 | 0.201 | 0.199 | -37.19 | -37.81 |
| nop | 0.664 | 0.568 | 0.553 | -14.46 | -16.72 |
| parse | 0.128 | 0.128 | 0.130 | +0.00 | +1.56 |
| sayloop | 0.029 | 0.028 | 0.028 | -3.45 | -3.45 |
| sendloop | 1.067 | 1.248 | 1.249 | +16.96 | +17.06 |
| startup | 0.026 | 0.025 | 0.024 | -3.85 | -7.69 |
| strings | 1.140 | 1.125 | 1.139 | -1.32 | -0.09 |
| textnum | 0.090 | 0.098 | 0.095 | +8.89 | +5.56 |
| varlookup | 0.772 | 0.778 | 0.782 | +0.78 | +1.30 |
| rexxcps | 1.994 | 2.007 | 2.094 | +0.65 | +5.02 |

Verdict: round 1 moves dispatch's wall from +18.85% to +5.87% and leaves sendloop at +17.06%,
fibfunc +8.86%, dispatchclass +7.03%, and puts rexxcps at +5.02% and fibcall at +6.05%. Ir is
inside +0.3% of base everywhere except fibcall (+2.12%) and fibfunc (+4.30%), both over at head
too. The variants behind it are in the round's report
(`.superpowers/sdd/2026-09-29-phase-6-s0-s1/perf1-report.md`).


## Task 10

Plain DO flattened and the `Op::Exec` outcome channel, head `2fb2bd50c`. Instructions (Ir), libc
and ld-linux subtracted, median of 3 rounds, every spread 0.0001% or less:

```
bash rust/bench-programs/callgrind.sh -r 3 -o $S/cg base=<1754a3b5a>/rexx-run \
  prev=<92ac5c054>/rexx-run head=<2fb2bd50c>/rexx-run
```

Each binary was built from `git archive` of its commit (`rust` and `interpreter`) into its own
target dir.

| program | base Ir | prev Ir | head Ir | prev vs base % | head vs base % | head vs prev % |
|---|---|---|---|---|---|---|
| alloc | 25168678552 | 20249018559 | 20285019284 | -19.55 | -19.40 | +0.18 |
| alloc4c | 3224395138 | 3173498684 | 3195499409 | -1.58 | -0.90 | +0.69 |
| arith | 11520060248 | 11471165757 | 11531154772 | -0.42 | +0.10 | +0.52 |
| assign | 19637557611 | 18840415350 | 19541123107 | -4.06 | -0.49 | +3.72 |
| compound | 9271611629 | 9187029588 | 9307007547 | -0.91 | +0.38 | +1.31 |
| decloop | 2565175804 | 2533249310 | 2549646127 | -1.24 | -0.61 | +0.65 |
| decrender | 4348396391 | 4274115353 | 4301707487 | -1.71 | -1.07 | +0.65 |
| dispatch | 20483285266 | 20508850258 | 20658792415 | +0.12 | +0.86 | +0.73 |
| dispatchclass | 15850195790 | 15494532692 | 15558509978 | -2.24 | -1.84 | +0.41 |
| emptyloop | 9308098509 | 9259510005 | 9359364239 | -0.52 | +0.55 | +1.08 |
| extcall | 8281259617 | 8227425066 | 8233425791 | -0.65 | -0.58 | +0.07 |
| fibcall | 8345686514 | 8522465879 | 8411530492 | +2.12 | +0.79 | -1.30 |
| fibfunc | 7988911860 | 8332398587 | 8200838404 | +4.30 | +2.65 | -1.58 |
| heapshape | 3278634294 | 2373947480 | 2376968928 | -27.59 | -27.50 | +0.13 |
| nop | 9437300267 | 9240156702 | 9540864449 | -2.09 | +1.10 | +3.25 |
| parse | 1541986815 | 1533228145 | 1540224191 | -0.57 | -0.11 | +0.46 |
| sayloop | 114813465 | 114714602 | 115514728 | -0.09 | +0.61 | +0.70 |
| sendloop | 13863190996 | 13713470957 | 13758471676 | -1.08 | -0.76 | +0.33 |
| startup | 58074154 | 58069747 | 58070444 | -0.01 | -0.01 | +0.00 |
| strings | 17749287579 | 17488789122 | 17548737120 | -1.47 | -1.13 | +0.34 |
| textnum | 1176579576 | 1152043463 | 1167239499 | -2.09 | -0.79 | +1.32 |
| varlookup | 14878111883 | 14461721676 | 14822611074 | -2.80 | -0.37 | +2.50 |
| rexxcps | 17817335255 | 17834081656 | 17844231924 | +0.09 | +0.15 | +0.06 |

Over +0.3% against `92ac5c054`: assign +3.72, nop +3.25, varlookup +2.50, textnum +1.32,
compound +1.31, emptyloop +1.08, dispatch +0.73, sayloop +0.70, alloc4c +0.69, decloop +0.65,
decrender +0.65, arith +0.52, parse +0.46, dispatchclass +0.41, strings +0.34, sendloop +0.33.
