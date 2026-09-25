#!/bin/bash
# The end-of-task gates, over the committed tree, statuses written unpiped.
# G3-G6 use the default target directory, rust/target: the arity suites run
# rust/target/release/rexx-run whatever CARGO_TARGET_DIR says.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
G=$S/gates; mkdir -p $G; ST=$G/status.txt
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust
# The brief's quiet-machine condition before an oracle-backed test run:
# 1-minute load under 15 and 5-minute load under 40.
quiet() { while ! awk '{exit !($1 < 15 && $2 < 40)}' /proc/loadavg; do sleep 30; done; echo "quiet-before-$1 $(uptime)" >> $ST; }
echo "sha $(git rev-parse HEAD)" > $ST
echo $$ > $G/pid
echo "tree-changes $(git status --short | wc -l)" >> $ST
echo "load-before $(uptime)" >> $ST
echo "df $(df -h /tmp | tail -1)" >> $ST
cargo fmt --all --check > $G/fmt.txt 2>&1; echo "G1 fmt exit $?" >> $ST
rm -rf $S/t-clippy
CARGO_TARGET_DIR=$S/t-clippy cargo clippy --workspace --all-targets -- -D warnings > $G/clippy.txt 2>&1; echo "G2 clippy (empty target dir) exit $?" >> $ST
rm -rf $S/t-clippy
cargo build --workspace --all-targets --release > $G/build-release.txt 2>&1; echo "G3 build --all-targets --release exit $?" >> $ST
quiet G4
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast > $G/test-release.txt 2>&1; echo "G4 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast exit $?" >> $ST
echo "load-after-G4 $(uptime)" >> $ST
cargo build --workspace --all-targets > $G/build-debug.txt 2>&1; echo "G5 build --all-targets (debug) exit $?" >> $ST
quiet G6
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast > $G/test-debug.txt 2>&1; echo "G6 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast (debug) exit $?" >> $ST
echo "load-after $(uptime)" >> $ST
echo "tree-changes-after $(git status --short | wc -l)" >> $ST
echo finished >> $ST
