#!/bin/bash
# usage: gate6_rerun.sh
# G6 again, over the same commit and the same default target directory,
# after a first run whose only failure was an oracle `TimedOut` at a load
# average above 200 (the plan's quiet-machine bullet: a timeout is machine
# state until a quiet rerun says otherwise). Statuses written unpiped.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
G=$S/gates; ST=$G/status-g6-rerun.txt
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust
echo "sha $(git rev-parse HEAD)" > $ST
echo "tree-changes $(git status --short | wc -l)" >> $ST
echo "load-before $(uptime)" >> $ST
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast > $G/test-debug-rerun.txt 2>&1; echo "G6 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast (debug) exit $?" >> $ST
echo "load-after $(uptime)" >> $ST
echo "tree-changes-after $(git status --short | wc -l)" >> $ST
echo finished >> $ST
