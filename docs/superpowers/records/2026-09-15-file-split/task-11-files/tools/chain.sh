#!/bin/bash
# usage: chain.sh OUTDIR JOBS RUST_DIR TAG...
# One callgrind round per program per task-BASE binary (builds/bins/rexx-run-TAG),
# each from a fresh empty directory; prints summary minus libc.so.6 and ld-linux.
B=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-11b
OUT=$1; J=$2; R=$3; shift 3; mkdir -p $OUT
PROGS="$R/bench-rexxcps/rexxcps.rex"
for p in nop assign emptyloop varlookup arith compound dispatch strings startup parse; do PROGS="$PROGS $R/bench-programs/$p.rex"; done
for t in "$@"; do for p in $PROGS; do echo "$t $p"; done; done | xargs -P $J -L 1 bash -c '
  t=$0; p=$1; name=$(basename $p .rex); f='$OUT'/$name.$t
  cwd='$OUT'/cwd-$name.$t; mkdir $cwd
  (cd $cwd && valgrind --tool=callgrind --cache-sim=no --branch-sim=no --callgrind-out-file=$f.cg '$B'/builds/bins/rexx-run-$t $p > $f.stdout 2> $f.stderr)
  echo "$? $(python3 '$B'/tools/cg_objects.py $f.cg | tail -1 | awk "{print \$NF}") leftover $(ls -A $cwd | wc -l)" > $f.result
  rmdir $cwd; rm $f.cg
'
