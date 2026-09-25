#!/bin/bash
# usage: tmove8.sh N PARENT_RS MODNAME
# tmove7.sh for rexx-api: pre<N>/ from HEAD, move_tests_mod.py over the main
# checkout, rustfmt, then the comment pass listing (comments7.py).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
export SPLIT_CRATE=rexx-api
N=$1; P=$2; MOD=$3; C=${P%.rs}/$MOD.rs
$S/tools/prep8.sh $N || exit 1
(cd $S/tools && python3 move_tests_mod.py $S/pre$N/$P $R/crates/rexx-api/src/$P $R/crates/rexx-api/src/$C $MOD $S/c$N/removed.json) | tee $S/c$N/move.txt
(cd $R && cargo fmt --all && git status --short)
python3 $S/tools/comments7.py $S/pre$N $R $P $C --tests > $S/art/c$N-comments.txt
SPLIT_PARENT_RS=$P SPLIT_DESTS=$C python3 $S/tools/positional.py $S/pre$N $R/crates/rexx-api/src > $S/art/c$N-positional.txt
cat $S/art/c$N-comments.txt $S/art/c$N-positional.txt
