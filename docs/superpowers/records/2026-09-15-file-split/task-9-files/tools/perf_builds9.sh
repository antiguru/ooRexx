#!/bin/bash
# usage: perf_builds9.sh C1 C2 ... (this task's commits, in order)
# Release rexx-run for BASE (its own target dir) and for each commit in turn
# (one shared target dir, built in the order given), each from a `git
# archive` of the whole repository at that commit; each binary's .text
# sha256 (objcopy -O binary --only-section=.text BIN OUT && sha256sum OUT).
# BASE's and the last commit's binaries are kept for the callgrind runs.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
O=$S/art/perf; mkdir -p $O/bins
build() { # tag commit target
  T=$S/trees/full-$2; [ -d $T ] || { mkdir -p $T; git -C $M archive $2 | tar -x -C $T; }
  (cd $T/rust && CARGO_TARGET_DIR=$3 cargo build --release -p rexx-exec --bin rexx-run > $O/build-$1.log 2>&1); echo "build $1 $2 exit $?" >> $O/builds.txt
  cp $3/release/rexx-run $O/bins/rexx-run-$1
  objcopy -O binary --only-section=.text $O/bins/rexx-run-$1 $O/bins/text-$1 && echo "$1 $2 .text $(sha256sum $O/bins/text-$1 | cut -c1-64) $(stat -c %s $O/bins/text-$1) bytes" >> $O/builds.txt
  rm $O/bins/text-$1
}
: > $O/builds.txt
echo "load-before $(cat /proc/loadavg)" >> $O/builds.txt
build base 72f2bc3d8 $S/t-perf-base
n=0; for c in "$@"; do n=$((n+1)); build c$n $c $S/t-perf-final; [ $n -lt $# ] && rm $O/bins/rexx-run-c$n; done
echo "load-after $(cat /proc/loadavg)" >> $O/builds.txt
echo finished >> $O/builds.txt
