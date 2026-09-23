#!/bin/bash
# Full gates on the committed tree; statuses to gates/status.txt, logs beside it.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round4/clause-overhead
G=$S/gates
W=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-accb7993007714f77
mkdir -p "$G"
echo $$ > "$G/pid"
cd "$W/rust" || exit 1
echo "$(git -C "$W" rev-parse HEAD)" > "$G/status.txt"
git -C "$W" status --short >> "$G/status.txt"
export CARGO_TARGET_DIR=$S/target
cargo fmt --all --check > "$G/fmt.log" 2>&1; echo "fmt exit=$?" >> "$G/status.txt"
CARGO_TARGET_DIR=$S/gtarget cargo clippy --workspace --all-targets -- -D warnings > "$G/clippy.log" 2>&1; echo "clippy(cold) exit=$?" >> "$G/status.txt"
rm -r "$S/gtarget"
cargo build --workspace --all-targets --release > "$G/build-release.log" 2>&1; echo "build release exit=$?" >> "$G/status.txt"
cargo test --workspace --release --no-fail-fast --no-run > "$G/build-release-tests.log" 2>&1; echo "build release tests exit=$?" >> "$G/status.txt"
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast > "$G/test-release.log" 2>&1; echo "test release exit=$?" >> "$G/status.txt"
cargo build --workspace --all-targets > "$G/build-debug.log" 2>&1; echo "build debug exit=$?" >> "$G/status.txt"
cargo test --workspace --no-fail-fast --no-run > "$G/build-debug-tests.log" 2>&1; echo "build debug tests exit=$?" >> "$G/status.txt"
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast > "$G/test-debug.log" 2>&1; echo "test debug exit=$?" >> "$G/status.txt"
echo finished >> "$G/status.txt"
