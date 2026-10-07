#!/bin/bash
set -u
S=/tmp/claude-1000/p6-t26
R=/home/moritz/dev/repos/ooRexx-rust-rewrite
cd $R
mk() { # name rev
  mkdir -p $S/trees/$1 $S/bin/$1
  git archive $2 | tar -x -C $S/trees/$1
}
mk base 1754a3b5a
mk s1 1a81353e3
mk head c484f4516
for n in base s1 head; do
  find $S/trees/$n -type f -exec touch {} +
  ( cd $S/trees/$n/rust && CARGO_TARGET_DIR=$S/tgt/$n memcap 8G cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-$n.log 2>&1 )
  echo "$n exit=$? compiling=$(/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-$n.log)" >> $S/progress.txt
  cp $S/tgt/$n/release/rexx-run $S/bin/$n/rexx-run
done
cd $S/bin
for n in base s1 head; do
  echo "$n $(sha256sum $n/rexx-run | cut -d' ' -f1) $(objcopy -O binary --only-section=.text $n/rexx-run /dev/stdout | wc -c)" >> $S/hashes.txt
done
echo done >> $S/progress.txt
