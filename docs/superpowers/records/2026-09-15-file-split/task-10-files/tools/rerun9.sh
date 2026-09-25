#!/bin/bash
# usage: rerun9.sh N...
# rerun8.sh for this task: instruments 1-3 with the final tooling on each
# commit pair, PRE the parent commit's rexx-parse src and POST the commit's,
# both from `git archive`; the arguments are the commit's own
# (task-9-files/c<N>/args: parent, destinations, instrument options), and
# each output is compared with the one committed for that commit.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
D=$M/docs/superpowers/records/2026-09-15-file-split/task-9-files
O=$S/art/final/rerun; mkdir -p $O
tree() { [ -d $S/trees/parse-src-$1 ] || { mkdir -p $S/trees/parse-src-$1; git -C $M archive $1 rust/crates/rexx-parse/src | tar -x -C $S/trees/parse-src-$1; }; echo $S/trees/parse-src-$1/rust/crates/rexx-parse/src; }
for N in "$@"; do
  mapfile -t A < $D/c$N/args
  P=${A[0]}; DEST=${A[1]}; OPTS=("${A[@]:2}")
  C=$(git -C $M log --format=%h -F --grep=task-9-files/c$N/ 72f2bc3d8..HEAD); PC=$(git -C $M rev-parse --short=9 $C^)
  SPLIT_CRATE=rexx-parse SPLIT_PARENT=${P%.rs} SPLIT_PARENT_RS=$P SPLIT_DESTS=$DEST SPLIT_TESTS_RS=none python3 $S/tools/instruments.py $(tree $PC) $(tree $C) $D/c$N/removed.json $O/c$N "${OPTS[@]}" > /dev/null
  same=1; for k in instrument1 instrument2 instrument3 verdict; do
    cmp -s <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $D/c$N/c$N-$k.txt) <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $O/c$N-$k.txt) || same=0
  done
  echo "c$N $C (parent $PC): $(head -1 $O/c$N-verdict.txt); outputs $([ $same = 1 ] && echo identical to || echo DIFFER from) the committed ones"
done
