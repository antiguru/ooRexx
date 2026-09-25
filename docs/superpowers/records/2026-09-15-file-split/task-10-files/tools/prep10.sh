#!/bin/bash
# usage: prep10.sh N   (SPLIT_CRATE, SPLIT_ROOT)
# pre<N>/ is the main checkout's HEAD crates/$SPLIT_CRATE/$SPLIT_ROOT,
# extracted with git archive: the tree every instrument of commit N compares
# against.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
N=$1; B=rust/crates/$SPLIT_CRATE/$SPLIT_ROOT
[ -n "$SPLIT_CRATE" ] && [ -n "$SPLIT_ROOT" ] || { echo "SPLIT_CRATE and SPLIT_ROOT must be set"; exit 2; }
[ -z "$(git -C $M status --short -- rust)" ] || { echo "tree not clean"; git -C $M status --short; exit 1; }
[ ! -e $S/pre$N ] || { echo "pre$N exists"; exit 1; }
mkdir -p $S/pre$N.x $S/c$N
git -C $M archive HEAD $B | tar -x -C $S/pre$N.x
mv $S/pre$N.x/$B $S/pre$N; rm -r $S/pre$N.x
echo "pre$N at $(git -C $M rev-parse --short HEAD): $B" | tee $S/c$N/pre.txt
