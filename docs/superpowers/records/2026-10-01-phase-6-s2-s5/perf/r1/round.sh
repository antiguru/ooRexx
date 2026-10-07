#!/bin/bash
# usage: round.sh NAME REV JOBS : build REV from git archive into bin/NAME, then callgrind every program with base, s1, NAME
set -u
S=/tmp/claude-1000/p6-t26
R=/home/moritz/dev/repos/ooRexx-rust-rewrite
n=$1; rev=$2; j=$3
mkdir -p $S/trees/$n $S/bin/$n
cd $R && git archive $rev | tar -x -C $S/trees/$n
find $S/trees/$n -type f -exec touch {} +
( cd $S/trees/$n/rust && CARGO_TARGET_DIR=$S/tgt/$n memcap 8G cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-$n.log 2>&1 )
echo "$n exit=$? compiling=$(/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-$n.log)" >> $S/progress.txt
cp $S/tgt/$n/release/rexx-run $S/bin/$n/rexx-run
echo "$n $(sha256sum $S/bin/$n/rexx-run | cut -d' ' -f1) $(objcopy -O binary --only-section=.text $S/bin/$n/rexx-run /dev/stdout | wc -c)" >> $S/hashes.txt
cd $R
bash rust/bench-programs/callgrind.sh -r 3 -j $j -o $S/cg-$n base=$S/bin/base/rexx-run s1=$S/bin/s1/rexx-run $n=$S/bin/$n/rexx-run > $S/cg-$n-table.txt 2> $S/cg-$n-err.txt
echo "exit $?" > $S/cg-$n-exit.txt
