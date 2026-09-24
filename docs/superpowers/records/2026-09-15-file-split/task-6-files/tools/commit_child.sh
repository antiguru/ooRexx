#!/bin/bash
# usage: commit_child.sh N CHILD MSGFILE [extra paths...]
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-6
N=$1; CHILD=$2; MSG=$3; shift 3
cd $S/dev
D=docs/superpowers/records/2026-09-15-file-split/task-2-files
mkdir -p $D/c$N $D/tools/item-tool/src
cp $S/art/c$N-* $D/c$N/; cp $S/c$N/removed.json $D/c$N/removed.json
cp $S/tools/*.py $S/tools/*.sh $D/tools/; rm -f $D/tools/plan.json
cp $S/tools/plan.json $D/tools/plan.json
cp $S/item-tool/Cargo.toml $D/tools/item-tool/; cp $S/item-tool/src/main.rs $D/tools/item-tool/src/
git add rust/crates/rexx-exec/src/dispatch.rs rust/crates/rexx-exec/src/dispatch/$CHILD.rs $D "$@"
git status --short
git commit -q -F $MSG && git log --oneline -1
