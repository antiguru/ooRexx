#!/bin/bash
# The S2 rows against the oracle, from rust/. Usage:
#   bash s2rows.sh TARGET_DIR OUT_DIR
# Writes OUT_DIR/s2rows.log and the criterion-one table OUT_DIR/s2-table.txt.
REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE="$2/s2-table.txt" CARGO_TARGET_DIR=$1 \
  cargo test --release -p rexx-exec --test concurrency_tests -- group_runs::the_s2_rows \
  > "$2/s2rows.log" 2>&1
echo "s2rows exit $?"
