#!/bin/bash
# usage: perf_builds10.sh TAG=COMMIT...
# The brief's reduced performance record: the release `rexx-run`'s `.text`
# sha256 (objcopy -O binary --only-section=.text BIN OUT && sha256sum OUT)
# at each revision given. Each is built from a `git archive` of the whole
# repository at that commit, with every file stamped (Task 9's
# build-freshness trap), in a target directory of its own that is deleted
# by path once its hash is recorded; the build log is kept and must show
# `Compiling rexx-exec`.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
O=$S/art/perf; mkdir -p $O
: > $O/builds.txt
echo "load-before $(cat /proc/loadavg)" >> $O/builds.txt
for tc in "$@"; do
  TAG=${tc%%=*}; C=${tc#*=}
  T=$S/trees/perf-$C; rm -rf $T; mkdir -p $T; git -C $M archive $C | tar -x -C $T
  find $T/rust -type f -exec touch {} +
  TD=$S/t-perf-$TAG
  echo "df $(df -h /tmp | tail -1)" >> $O/builds.txt
  (cd $T/rust && CARGO_TARGET_DIR=$TD cargo build --release -p rexx-exec --bin rexx-run > $O/build-$TAG.log 2>&1); echo "build $TAG $C exit $?; 'Compiling rexx-exec' lines: $(/bin/grep -a -c 'Compiling rexx-exec' $O/build-$TAG.log)" >> $O/builds.txt
  objcopy -O binary --only-section=.text $TD/release/rexx-run $O/text-$TAG && echo "$TAG $C .text $(sha256sum $O/text-$TAG | cut -c1-64) $(stat -c %s $O/text-$TAG) bytes" >> $O/builds.txt
  rm -f $O/text-$TAG
  rm -rf $TD $T
done
echo "load-after $(cat /proc/loadavg)" >> $O/builds.txt
echo finished >> $O/builds.txt
cat $O/builds.txt
