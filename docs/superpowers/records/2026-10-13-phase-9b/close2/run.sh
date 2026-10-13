#!/bin/bash
# Phase 9b close: workspace gate, then the whole suite. Run from the repo root.
D=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-13-phase-9b/close2
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust
echo "head $(git rev-parse HEAD)" > $D/status.txt
echo "start $(date -u +%FT%TZ)" >> $D/status.txt
REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast > $D/workspace.log 2>&1
echo "workspace exit $? at $(date -u +%FT%TZ)" >> $D/status.txt
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=$D/suite/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture > $D/suite.log 2>&1
echo "suite exit $? at $(date -u +%FT%TZ)" >> $D/status.txt
echo finished >> $D/status.txt
