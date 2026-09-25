#!/bin/bash
# usage: prep8.sh N
# prep.sh for rexx-api: pre<N>/ is the main checkout's HEAD
# crates/rexx-api/src, extracted with git archive, the tree every instrument
# of commit N compares against.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
N=$1
[ -z "$(git -C $M status --short -- rust)" ] || { echo "tree not clean"; git -C $M status --short; exit 1; }
[ ! -e $S/pre$N ] || { echo "pre$N exists"; exit 1; }
mkdir -p $S/pre$N.x $S/c$N
git -C $M archive HEAD rust/crates/rexx-api/src | tar -x -C $S/pre$N.x
mv $S/pre$N.x/rust/crates/rexx-api/src $S/pre$N; rm -r $S/pre$N.x
echo "pre$N at $(git -C $M rev-parse --short HEAD)" | tee $S/c$N/pre.txt
