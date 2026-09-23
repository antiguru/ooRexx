#!/bin/bash
# The same release test run on base source in this worktree, for the failing-set comparison.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/spikes/cold-arms
W=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-a179fbec2e954af52/rust
cp "$S/drive.rs.base" "$W/crates/rexx-exec/src/ir/drive.rs" || exit 1
cd "$W" || exit 1
export CARGO_TARGET_DIR=$S/target-base
st=$S/tests-base-status.txt
git diff --stat 0ba0f3876 -- crates > "$st"
cargo build --release -p rexx-exec --all-targets > "$S/test-base-build.log" 2>&1
echo "build $?" >> "$st"
memcap 8G cargo test -p rexx-exec --release --no-fail-fast > "$S/test-base.log" 2>&1
echo "test $?" >> "$st"
cp "$S/drive.rs.full" "$W/crates/rexx-exec/src/ir/drive.rs"
git status --short >> "$st"
echo finished >> "$st"
