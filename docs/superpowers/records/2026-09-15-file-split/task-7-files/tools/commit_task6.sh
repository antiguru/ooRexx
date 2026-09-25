#!/bin/bash
# usage: commit_task6.sh N
# Copies commit N's artifacts and the tooling into the records directory,
# stages the snapshot's files and those records by explicit path, checks the
# staged rust/ tree is the one instrument 4 ran on, and commits with msgN.txt.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-7
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
N=$1
cd $M
D=docs/superpowers/records/2026-09-15-file-split/task-6-files
mkdir -p $D/c$N $D/instrument4 $D/tools/item-tool/src
cp $S/art/c$N-* $D/c$N/; cp $S/c$N/removed.json $D/c$N/removed.json
[ -f $S/c$N/move.txt ] && cp $S/c$N/move.txt $D/c$N/
cp $S/art/i4-c$N.meta $S/art/i4-c$N.results $S/art/i4-c$N.compare $D/instrument4/
[ -f $D/instrument4/i4-base.meta ] || cp $S/art/i4-base.meta $S/art/i4-base.results $D/instrument4/
cp $S/art/controls-*.txt $S/art/path-pins-*.txt $D/ 2>/dev/null
mkdir -p $D/base; cp $S/art/base/* $S/art/natives-base.txt $D/base/ 2>/dev/null
cp $S/tools/*.py $S/tools/*.sh $D/tools/
cp $S/tools/item-tool/Cargo.toml $D/tools/item-tool/; cp $S/tools/item-tool/src/main.rs $D/tools/item-tool/src/
for f in $(cat $S/snap$N/files); do cmp -s $f $S/snap$N/tree/$f || { echo "DIFF $f from snapshot"; exit 1; }; done
[ -z "$(comm -23 <({ git diff --name-only HEAD -- rust; git ls-files --others --exclude-standard -- rust; } | sort -u) <(sort $S/snap$N/files))" ] || { echo "a changed file outside the snapshot"; exit 1; }
git add $(cat $S/snap$N/files) $D
WANT=$(/bin/grep -a '^rust tree' $S/art/i4-c$N.meta | awk '{print $3}')
GOT=$(git write-tree --prefix=rust/)
echo "staged rust tree $GOT, instrument 4 tree $WANT"
[ "$GOT" = "$WANT" ] || { echo "TREE MISMATCH, not committing"; exit 1; }
git status --short | /bin/grep -v '^[AM] '
git commit -q -F $S/msg$N.txt && git log --oneline -1
