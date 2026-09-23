#!/bin/bash
# usage: save.sh NAME -- copies the worktree's release rexx-run/rexx-ir to bin/NAME-*, prints .text hash
set -e
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round4/clause-overhead
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-accb7993007714f77/rust
cp "$S/target/release/rexx-run" "$S/bin/$1-rexx-run"
cp "$S/target/release/rexx-ir" "$S/bin/$1-rexx-ir" 2>/dev/null || true
objcopy -O binary --only-section=.text "$S/bin/$1-rexx-run" "$S/bin/$1.text"
echo "$1 $(sha256sum < "$S/bin/$1.text" | cut -d' ' -f1)" | tee -a "$S/bin/hashes.txt"
