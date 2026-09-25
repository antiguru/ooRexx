#!/bin/bash
# usage: build.sh TAG COMMIT
# Release rexx-run from a `git archive` of the repository at COMMIT, every
# extracted file touched, in a fresh CARGO_TARGET_DIR of its own. Keeps the
# build log and the binary; records the log's `Compiling rexx-exec` line and
# the .text sha256.
B=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-11b
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
TAG=$1; C=$2; TREE=$B/trees/$TAG; TGT=$B/target-$TAG; O=$B/builds
mkdir -p $O/bins
[ -e $TREE ] && { echo "$TREE exists"; exit 1; }
[ -e $TGT ] && { echo "$TGT exists"; exit 1; }
mkdir -p $TREE && git -C $M archive $C | tar -x -C $TREE
find $TREE -type f -exec touch {} +
echo "build $TAG $C load-before $(cat /proc/loadavg) df $(df -h /tmp | tail -1)" >> $O/builds.txt
(cd $TREE/rust && CARGO_TARGET_DIR=$TGT cargo build --release -p rexx-exec --bin rexx-run > $O/build-$TAG.log 2>&1)
echo "build $TAG $C exit $?" >> $O/builds.txt
echo "build $TAG Compiling rexx-exec: $(/bin/grep -a 'Compiling rexx-exec' $O/build-$TAG.log)" >> $O/builds.txt
cp $TGT/release/rexx-run $O/bins/rexx-run-$TAG
objcopy -O binary --only-section=.text $O/bins/rexx-run-$TAG $O/bins/text-$TAG
echo "$TAG $C .text $(sha256sum $O/bins/text-$TAG | cut -c1-64) $(stat -c %s $O/bins/text-$TAG) bytes" >> $O/builds.txt
rm $O/bins/text-$TAG
echo "build $TAG load-after $(cat /proc/loadavg)" >> $O/builds.txt
echo "build $TAG done" >> $O/builds.txt
