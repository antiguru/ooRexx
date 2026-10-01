#!/bin/bash
S=${S:?set S to the task scratch dir}
G=$S/gates; mkdir -p $G; ST=$G/status.txt
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust || exit 1
echo $$ > $G/pid
git rev-parse HEAD > $ST
echo "started $(date -Is)" >> $ST
waitload() { while :; do read l1 l5 _ < /proc/loadavg; if awk -v a=$l1 -v b=$l5 'BEGIN{exit !(a<15 && b<40)}'; then echo "load $1 $(cat /proc/loadavg) $(date -Is)" >> $ST; return; fi; sleep 30; done; }
echo "load at start $(cat /proc/loadavg)" >> $ST
cargo fmt --all --check > $G/g1-fmt.txt 2>&1; echo "G1 fmt exit $?" >> $ST
rm -rf $S/target-clippy
CARGO_TARGET_DIR=$S/target-clippy cargo clippy --workspace --all-targets -- -D warnings > $G/g2-clippy.txt 2>&1; echo "G2 clippy(empty target) exit $?" >> $ST
rm -rf $S/target-clippy
# Builds exactly what G4 runs, outside the cap, with G4's environment.
REXX_CORPUS_GATE=1 cargo test --workspace --release --no-fail-fast --no-run > $G/g3-build-release.txt 2>&1; echo "G3 release build (test --no-run) exit $?" >> $ST
waitload G4
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast > $G/g4-test-release.txt 2>&1; echo "G4 release test exit $?" >> $ST
echo "G4 Compiling lines: $(grep -c '^ *Compiling' $G/g4-test-release.txt)" >> $ST
echo "load after G4 $(cat /proc/loadavg)" >> $ST
REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast --no-run > $G/g5-build-debug.txt 2>&1; echo "G5 debug build (test --no-run) exit $?" >> $ST
waitload G6
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast > $G/g6-test-debug.txt 2>&1; echo "G6 debug test exit $?" >> $ST
echo "G6 Compiling lines: $(grep -c '^ *Compiling' $G/g6-test-debug.txt)" >> $ST
echo "load after G6 $(cat /proc/loadavg)" >> $ST
CARGO_TARGET_DIR=$S/target-pinning cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings > $G/g7-clippy-pinning.txt 2>&1; echo "G7 clippy --features pinning exit $?" >> $ST
CARGO_TARGET_DIR=$S/target-pinning memcap 8G cargo test -p rexx-exec --features pinning --test concurrency_tests -- measured::a_ measured::an_ > $G/g8-test-pinning.txt 2>&1; echo "G8 pinning self-tests exit $?" >> $ST
rm -rf $S/target-pinning
git rev-parse HEAD >> $ST
git status --short >> $ST
echo "finished $(date -Is)" >> $ST
