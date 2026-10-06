#!/bin/bash
# The SysSleep overshoot probe, per tree: release rexx-run built from
# `git archive` of each commit named, then the oracle. Usage, from anywhere:
#   bash overshoot.sh SCRATCH COMMIT... > overshoot.log
# SCRATCH holds the trees, one shared target dir and an empty run dir.
E=$(cd "$(dirname "$0")" && pwd)
W=/home/moritz/dev/repos/ooRexx-rust-rewrite
S=$1
shift
mkdir -p "$S/run"
for C in "$@"; do
  T=$S/tree-$C
  mkdir -p "$T"
  (cd "$W" && git archive "$C" rust interpreter) | tar -x -C "$T"
  find "$T" -type f -exec touch {} +
  (cd "$T/rust" && CARGO_TARGET_DIR=$S/target cargo build --release --bin rexx-run) > "$T.build" 2>&1 \
    || { echo "$C build failed"; exit 1; }
  grep -q 'Compiling rexx-exec' "$T.build" || { echo "$C: no Compiling rexx-exec line"; exit 1; }
  echo "$C $(cd "$S/run" && "$S/target/release/rexx-run" "$E/overshoot.rex" | tr '\n' ' ')"
done
echo "oracle $(cd "$S/run" && /home/moritz/dev/repos/ooRexx/build/bin/rexx "$E/overshoot.rex" | tr '\n' ' ')"
