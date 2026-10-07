#!/bin/bash
# Background gates on a frozen copy of one commit: bggates.sh SHA
# Status: $W/bg/<short sha>/status.txt; target dirs deleted at the end.
set -u
REPO=/home/moritz/dev/repos/ooRexx-rust-rewrite
W=$REPO/.superpowers/sdd/2026-10-07-phase-6-1
SHA=$(git -C $REPO rev-parse --short=9 "${1:?commit}") || exit 1
D=$W/bg/$SHA; ST=$D/status.txt; G=$D/logs
[ -e $D ] && { echo "$D exists"; exit 1; }
mkdir -p $D/tree $G || exit 1
git -C $REPO archive $SHA | tar -x -C $D/tree || exit 1
find $D/tree -exec touch {} +
for p in ootest oodocs build rust/corpus-l1 rust/bench-baselines/pinned; do
  [ -e $REPO/$p ] && ln -s $REPO/$p $D/tree/$p
done
cd $D/tree/rust || exit 1
export CARGO_TARGET_DIR=$D/tree/rust/target  # tests read rust/target in the tree
tables() { export REXX_TIMER_TABLE=$G/$1-timer-table.txt REXX_CRITERION_ONE_TABLE=$G/$1-criterion-one-table.txt REXX_CRITERION_ONE_S3_TABLE=$G/$1-criterion-one-s3-table.txt; }  # P48 reruns show in these
echo $$ > $D/pid
echo "$SHA started $(date -Is)" > $ST
waitload() { while :; do read l1 l5 _ < /proc/loadavg; if awk -v a=$l1 -v b=$l5 'BEGIN{exit !(a<15 && b<40)}'; then echo "load $1 $(cat /proc/loadavg) $(date -Is)" >> $ST; return; fi; sleep 30; done; }
cargo fmt --all --check > $G/g1-fmt.txt 2>&1; echo "G1 fmt exit $?" >> $ST
CARGO_TARGET_DIR=$D/target-clippy cargo clippy --workspace --all-targets -- -D warnings > $G/g2-clippy.txt 2>&1; echo "G2 clippy exit $?" >> $ST
REXX_CORPUS_GATE=1 cargo test --workspace --release --no-fail-fast --no-run > $G/g3-build-release.txt 2>&1; echo "G3 release build exit $?" >> $ST
waitload G4
tables g4
REXX_CORPUS_GATE=1 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast > $G/g4-test-release.txt 2>&1; echo "G4 release test exit $?" >> $ST
REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast --no-run > $G/g5-build-debug.txt 2>&1; echo "G5 debug build exit $?" >> $ST
waitload G6
tables g6
REXX_CORPUS_GATE=1 timeout 3000 memcap 8G cargo test -j 4 --workspace --no-fail-fast > $G/g6-test-debug.txt 2>&1; echo "G6 debug test exit $?" >> $ST
CARGO_TARGET_DIR=$D/target-pinning cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings > $G/g7-clippy-pinning.txt 2>&1; echo "G7 clippy pinning exit $?" >> $ST
CARGO_TARGET_DIR=$D/target-pinning memcap 8G cargo test -p rexx-exec --features pinning --test concurrency_tests -- measured::a_ measured::an_ > $G/g8-test-pinning.txt 2>&1; echo "G8 pinning self-tests exit $?" >> $ST
RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=$D/target-loom memcap 8G cargo test -p rexx-exec --test loom > $G/g9-loom.txt 2>&1; echo "G9 loom exit $?" >> $ST
/bin/grep -a -c "^test .* FAILED$\|^test result: FAILED" $G/g4-test-release.txt $G/g6-test-debug.txt >> $ST
echo "P48 reruns: $(cat $G/*-table.txt 2>/dev/null | /bin/grep -a -c 'P48 rerun')" >> $ST
rm -rf $D/tree $D/target-clippy $D/target-pinning $D/target-loom
echo "finished $(date -Is)" >> $ST
