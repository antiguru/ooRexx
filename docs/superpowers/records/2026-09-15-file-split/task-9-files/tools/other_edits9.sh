#!/bin/bash
# usage: other_edits9.sh N PARENT_RS DESTS(comma)   (writes art/c<N>-other-edits.diff)
# Every change under rust/ outside the parent and the destinations: the
# tracked diff (`git diff`), and every untracked file (`git ls-files
# --others --exclude-standard -- rust`), which `git diff` cannot see. Task
# 9's c4 committed four untracked instrument outputs past a diff-only check
# (review I1). Prints "other edits: K files" (tracked) and
# "untracked: U files"; verify9.sh requires both to be 0.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; P=$2; DESTS=$3
EXCL=":!crates/rexx-parse/src/$P"; KEEP="crates/rexx-parse/src/$P"
for d in ${DESTS//,/ }; do EXCL="$EXCL :!crates/rexx-parse/src/$d"; KEEP="$KEEP|crates/rexx-parse/src/$d"; done
git -C $R diff -- . $EXCL ':!corpus/refusal-sites.tsv' > $S/art/c$N-other-edits.diff
UNTRACKED=$(git -C $R ls-files --others --exclude-standard -- . | /bin/grep -a -v -x -E "$KEEP")
{ echo "untracked files under rust/ outside the parent and destinations (git ls-files --others --exclude-standard -- rust):"; [ -n "$UNTRACKED" ] && echo "$UNTRACKED" | sed 's/^/  /'; } >> $S/art/c$N-other-edits.diff
echo "other edits: $(/bin/grep -a -c '^diff --git' $S/art/c$N-other-edits.diff) files"
echo "untracked: $([ -n "$UNTRACKED" ] && echo "$UNTRACKED" | wc -l || echo 0) files"
