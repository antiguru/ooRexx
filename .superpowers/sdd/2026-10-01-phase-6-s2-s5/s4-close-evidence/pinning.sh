#!/bin/bash
# The pinning report's measured run, from rust/. Usage:
#   bash pinning.sh TARGET_DIR OUT_DIR
RAYON_NUM_THREADS=4 CARGO_TARGET_DIR=$1 memcap 8G cargo test --release -p rexx-exec \
  --features pinning --test concurrency_tests -- measured:: --nocapture --test-threads=1 \
  > "$2/pinning.log" 2>&1
echo "pinning exit $?"
