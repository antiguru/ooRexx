#!/bin/bash
set -u
S=/tmp/claude-1000/p6-s5-perfbase
R=/home/moritz/dev/repos/ooRexx-rust-rewrite
B=$R/rust/bench-programs
cd $R
mk() { # name rev
  mkdir -p $S/trees/$1 $S/bin/$1
  git archive $2 | tar -x -C $S/trees/$1
}
mk base 1754a3b5a
mk s1 1a81353e3
for p in 1:8 2:48 3:192; do n=${p%%:*}; a=${p##*:}; mk pad$n 1754a3b5a; python3 $B/layout-pad.py $S/trees/pad$n $a; done
for n in base s1 pad1 pad2 pad3; do
  find $S/trees/$n -type f -exec touch {} +
  mkdir -p $S/bin/$n
  ( cd $S/trees/$n/rust && CARGO_TARGET_DIR=$S/tgt/$n memcap 8G cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-$n.log 2>&1 )
  echo "$n compiling=$(/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-$n.log)" >> $S/progress.txt
  cp $S/tgt/$n/release/rexx-run $S/bin/$n/rexx-run
done
echo done >> $S/progress.txt
