#!/bin/bash
# usage: perf_builds.sh
# Release rexx-run for BASE (its own target dir) and for c5 then c6 (one
# shared target dir, built in that order), each from a `git archive` of the
# whole repository at that commit; each binary copied out with its .text
# sha256 (objcopy -O binary --only-section=.text BIN OUT && sha256sum OUT).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
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
build base 964a6a8db $S/t-perf-base
build c5 ecd4fc295 $S/t-perf-final
build c6 e59418808 $S/t-perf-final
echo finished >> $O/builds.txt
