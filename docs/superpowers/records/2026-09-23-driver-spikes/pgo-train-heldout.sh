#!/usr/bin/env bash
# Runs the held-out training set (the differential corpus, minus the six
# measured axis programs) through a -Cprofile-generate rexx-run binary, one
# process per program, from a fresh empty directory so the scratchpad's own
# .rex files never land on the oracle/interpreter external-routine search
# path. Each run is capped by timeout so a hang cannot stall training.
set -u
BIN="$1"
RUST_DIR="$2"
LIST="$3"
RUNDIR="$4"
mkdir -p "$RUNDIR"
count=0
while IFS= read -r rel; do
  [ -z "$rel" ] && continue
  f="$RUST_DIR/corpus/$rel"
  timeout 5 "$BIN" "$f" < /dev/null > /dev/null 2>&1
  count=$((count + 1))
done < "$LIST"
echo "ran $count programs"
