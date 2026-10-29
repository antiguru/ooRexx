#!/bin/bash
# usage: try.sh NAME -- build the working tree's rexx, pin it as rexx-head-try-r2NAME, callgrind a subset one round
set -u
n=$1
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust || exit 1
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/t87c-fix2-target memcap 8G cargo build --release -j 4 --bin rexx 2>&1 | grep -E '^(warning|error)|Finished'
P=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c
cp /tmp/claude-1000/t87c-fix2-target/release/rexx $P/rexx-head-try-r2$n
bench-programs/callgrind.sh -r 1 -j 8 -o $P/fix2-exp-$n -p "${PROGS:-rexxcps compound pingguard parse arith alloc4c pingsem}" base=$P/rexx-base-97eee8a9a head=$P/rexx-head-try-r2$n 2>&1 | tail -9 | cut -f1,7
