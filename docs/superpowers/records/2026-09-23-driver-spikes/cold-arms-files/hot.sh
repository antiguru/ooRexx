#!/bin/bash
# Callgrind instruction dumps of the base binary on every axis, for the hot-arm list.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/spikes/cold-arms
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-a179fbec2e954af52/rust
BIN=$S/bin/base-rexx-run
mkdir -p "$S/hot/run"
cd "$S/hot/run" || exit 1
for ax in rexxcps varlookup arith emptyloop dispatch compound; do
    if [ "$ax" = rexxcps ]; then f=$R/bench-rexxcps/rexxcps.rex; else f=$R/bench-programs/$ax.rex; fi
    valgrind --tool=callgrind --dump-instr=yes --dump-line=yes --compress-pos=no \
        --compress-strings=no --callgrind-out-file="$S/hot/cg.$ax" "$BIN" "$f" \
        >"$S/hot/$ax.out" 2>"$S/hot/$ax.err" &
done
wait
echo done
