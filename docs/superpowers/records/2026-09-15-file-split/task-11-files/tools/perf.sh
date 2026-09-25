#!/bin/bash
# usage: perf.sh BASE_BIN HEAD_BIN OUTDIR JOBS RUST_DIR
# Task 9's perf9.sh on the brief's eleven programs: two interleaved rounds
# (base, head per program, per round), each under callgrind with no cache or
# branch simulation, from a fresh empty directory per run. Reports
# callgrind's summary total minus libc.so.6 and ld-linux per run.
B=$1; H=$2; OUT=$3; J=$4; [ -d "$5" ] || { echo "RUST_DIR missing"; exit 1; }
R=$5
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p $OUT
PROGS="$R/bench-rexxcps/rexxcps.rex"
for p in nop assign emptyloop varlookup arith compound dispatch strings startup parse; do PROGS="$PROGS $R/bench-programs/$p.rex"; done
jobs=()
for round in 1 2; do for p in $PROGS; do for side in base head; do
  jobs+=("$round $side $p")
done; done; done
printf '%s\n' "${jobs[@]}" | xargs -P $J -L 1 bash -c '
  round=$0; side=$1; p=$2; name=$(basename $p .rex)
  bin='$B'; [ $side = head ] && bin='$H'
  f='$OUT'/$name.$side.$round
  cwd='$OUT'/cwd-$name.$side.$round; mkdir $cwd
  (cd $cwd && valgrind --tool=callgrind --cache-sim=no --branch-sim=no --callgrind-out-file=$f.cg $bin $p > $f.stdout 2> $f.stderr)
  echo "$? $(python3 '$HERE'/cg_objects.py $f.cg | tail -1 | awk "{print \$NF}")" > $f.result
  echo "leftover files in cwd: $(ls -A $cwd | wc -l)" >> $f.result
  rmdir $cwd
  rm $f.cg
'
for p in $PROGS; do name=$(basename $p .rex)
  echo "$name $(head -1 $OUT/$name.base.1.result) $(head -1 $OUT/$name.head.1.result) $(head -1 $OUT/$name.base.2.result) $(head -1 $OUT/$name.head.2.result)"
done
