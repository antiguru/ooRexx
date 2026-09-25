#!/bin/bash
# usage: tests_move.sh N PARENT_RS MODNAME
# One tests-only commit end to end: pre<N>, the move (PARENT/<MODNAME>.rs),
# rustfmt, verify.sh, and commit_task6.sh with msgN.txt.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; P=$2; MOD=$3; CHILD=${P%.rs}/$MOD.rs
$S/tools/prep.sh $N || exit 1
(cd $R/crates/rexx-exec/src && python3 $S/tools/move_tests_mod.py $P $P $CHILD $MOD $S/c$N/removed.json | tee $S/c$N/move.txt) || exit 1
(cd $R && cargo fmt --all) || exit 1
$S/tools/verify.sh $N $P $CHILD $CHILD || exit 1
$S/tools/commit_task6.sh $N
