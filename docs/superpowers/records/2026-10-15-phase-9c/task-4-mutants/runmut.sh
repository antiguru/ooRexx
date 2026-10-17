#!/usr/bin/env bash
# Usage: runmut.sh BATCH...  Applies each batch, runs the errors test, restores.
set -u
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/9c-t4
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust || exit 1
git diff --stat -- crates > "$S/mut-base-stat.txt"
for batch in "$@"; do
    python3 "$S/mutants.py" apply "$batch" > "$S/mut-applied-$batch.txt" || exit 1
    CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --profile mutation -p rexx-parse --test errors --no-fail-fast > "$S/mut-out-$batch.txt" 2>&1
    echo "batch $batch exit $?"
    python3 "$S/mutants.py" restore
    git diff --stat -- crates > "$S/mut-after-stat.txt"
    cmp "$S/mut-base-stat.txt" "$S/mut-after-stat.txt" || { echo "restore mismatch"; exit 1; }
done
