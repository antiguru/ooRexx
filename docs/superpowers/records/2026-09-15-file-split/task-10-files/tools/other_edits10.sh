#!/bin/bash
# usage: other_edits10.sh N PARENT_RS DESTS(comma)   (writes art/c<N>-other-edits.diff)
# other_edits9.sh with the crate and its root directory parameters
# (SPLIT_CRATE, SPLIT_ROOT: `rexx-exec` and `tests`, or `rexx-extract` and
# `src`). Every change under rust/ outside the parent and the destinations:
# the tracked diff (`git diff`), and every untracked file (`git ls-files
# --others --exclude-standard -- rust`), which `git diff` cannot see. Prints
# "other edits: K files" (tracked) and "untracked: U files"; verify10.sh
# requires both to be 0.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; P=$2; DESTS=$3
B=crates/$SPLIT_CRATE/$SPLIT_ROOT
[ -n "$SPLIT_CRATE" ] && [ -n "$SPLIT_ROOT" ] || { echo "SPLIT_CRATE and SPLIT_ROOT must be set"; exit 2; }
EXCL=":!$B/$P"; KEEP="$B/$P"
for d in ${DESTS//,/ }; do EXCL="$EXCL :!$B/$d"; KEEP="$KEEP|$B/$d"; done
git -C $R diff -- . $EXCL ':!corpus/refusal-sites.tsv' > $S/art/c$N-other-edits.diff
UNTRACKED=$(git -C $R ls-files --others --exclude-standard -- . | /bin/grep -a -v -x -E "$KEEP")
{ echo "untracked files under rust/ outside the parent and destinations (git ls-files --others --exclude-standard -- rust):"; [ -n "$UNTRACKED" ] && echo "$UNTRACKED" | sed 's/^/  /'; } >> $S/art/c$N-other-edits.diff
echo "other edits: $(/bin/grep -a -c '^diff --git' $S/art/c$N-other-edits.diff) files"
echo "untracked: $([ -n "$UNTRACKED" ] && echo "$UNTRACKED" | wc -l || echo 0) files"
