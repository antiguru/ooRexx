#!/bin/bash
# usage: perf8.sh BASE_BIN HEAD_BIN OUTDIR JOBS  (perf.sh on the brief's three programs)
# Two interleaved rounds (base, head, base, head per program), each run under
# callgrind with no cache or branch simulation, from an empty directory.
# Reports callgrind's summary total minus libc.so.6 and ld-linux per run.
B=$1; H=$2; OUT=$3; J=$4
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
# cg_objects.py is the one beside this script, in the committed tools/.
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p $OUT/cwd
PROGS="$R/bench-rexxcps/rexxcps.rex $R/bench-programs/nop.rex $R/bench-programs/dispatch.rex"
jobs=()
for round in 1 2; do for p in $PROGS; do for side in base head; do
  jobs+=("$round $side $p")
done; done; done
printf '%s\n' "${jobs[@]}" | xargs -P $J -L 1 bash -c '
  round=$0; side=$1; p=$2; name=$(basename $p .rex)
  bin='$B'; [ $side = head ] && bin='$H'
  f='$OUT'/$name.$side.$round
  (cd '$OUT'/cwd && valgrind --tool=callgrind --cache-sim=no --branch-sim=no --callgrind-out-file=$f.cg $bin $p > $f.stdout 2> $f.stderr)
  echo "$? $(python3 '$HERE'/cg_objects.py $f.cg | tail -1 | awk "{print \$NF}")" > $f.result
  rm $f.cg
'
for p in $PROGS; do name=$(basename $p .rex)
  echo "$name $(cat $OUT/$name.base.1.result) $(cat $OUT/$name.head.1.result) $(cat $OUT/$name.base.2.result) $(cat $OUT/$name.head.2.result)"
done
