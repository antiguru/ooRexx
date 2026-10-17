#!/bin/bash
# Phase 9c close: workspace gate, then the whole suite, on a fresh target directory.
D=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-15-phase-9c/close
export CARGO_TARGET_DIR=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/target-close
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust
echo "head $(git rev-parse HEAD)" > $D/status.txt
echo "start $(date -u +%FT%TZ)" >> $D/status.txt
cargo fmt --all --check > $D/fmt.log 2>&1
echo "fmt exit $? at $(date -u +%FT%TZ)" >> $D/status.txt
CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings > $D/clippy.log 2>&1
echo "clippy exit $? at $(date -u +%FT%TZ)" >> $D/status.txt
REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast > $D/workspace.log 2>&1
echo "workspace exit $? at $(date -u +%FT%TZ)" >> $D/status.txt
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=$D/suite/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture > $D/suite.log 2>&1
echo "suite exit $? at $(date -u +%FT%TZ)" >> $D/status.txt
echo finished >> $D/status.txt
