#!/bin/bash
# usage: trial7.sh N
# The brief's trial of a production move before it is committed: the test
# worktree at HEAD plus the working tree's changes, and refusal-sites.tsv
# refreshed there; the report shows every column other than 4 against HEAD.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
N=$1
rm -rf $S/trial$N; $S/tools/snap.sh save $S/trial$N > /dev/null
$S/tools/sync_wt.sh $S/trial$N
$S/tools/refusal_refresh.sh $S/wt/rust $S/t-check $S/art/c$N-trial-refusal-sites.txt
rm -f $S/art/c$N-trial-refusal-sites.txt.log
git -C $S/wt checkout -q -- rust/corpus/refusal-sites.tsv
