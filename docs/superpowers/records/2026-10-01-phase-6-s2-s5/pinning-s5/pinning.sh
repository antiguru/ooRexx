#!/bin/bash
# Criterion 9's pinning reports, from rust/. Usage: bash pinning.sh TARGET_DIR OUT_DIR
# The tables land in TARGET_DIR/tmp/: pinning-table*.md, pinning-corpus*.md.
T=$1; O=$2
RAYON_NUM_THREADS=4 CARGO_TARGET_DIR=$T memcap 8G cargo test --release -p rexx-exec --features pinning \
  --test concurrency_tests -- measured:: --nocapture --test-threads=1 > "$O/pinning.log" 2>&1
echo "measured exit $?"
CARGO_TARGET_DIR=$T memcap 8G cargo test --release -p rexx-exec --features pinning --test corpus \
  -- --exact pinning_report_over_the_corpus --nocapture > "$O/pinning-corpus.log" 2>&1
echo "corpus exit $?"
REXX_CORPUS_SWITCH=every CARGO_TARGET_DIR=$T memcap 8G cargo test --release -p rexx-exec --features pinning \
  --test corpus -- --exact pinning_report_over_the_corpus --nocapture > "$O/pinning-corpus-every.log" 2>&1
echo "corpus every exit $?"
