#!/bin/bash
# usage: sync_wt.sh SNAPDIR
# Puts the test worktree at the main checkout's HEAD plus the snapshot's
# files: every file the snapshot lists is copied in; nothing else differs.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-5
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
W=$S/wt
git -C $W reset -q --mixed
git -C $W checkout -q -- rust
git -C $W ls-files --others --exclude-standard -- rust/crates | while read f; do rm -- "$W/$f"; done
git -C $W checkout -q --detach $(git -C $M rev-parse HEAD) || exit 1
while read f; do mkdir -p $W/$(dirname $f); cp $1/tree/$f $W/$f; done < $1/files
git -C $W status --short | /bin/grep -v '^?? \(build\|oodocs\|ootest\|rust/corpus-l1\|rust/bench-baselines/pinned\|rust/target\)$'
