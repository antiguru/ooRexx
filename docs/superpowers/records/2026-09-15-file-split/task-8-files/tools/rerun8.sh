#!/bin/bash
# usage: rerun8.sh N...
# Re-runs instruments 1-3 with the final tooling on each Task 8 commit pair:
# PRE is the parent commit's src, POST the commit's, both from `git archive`;
# the arguments are the commit's own (task-8-files/c<N>/args: parent,
# destinations, test file, instrument options), and each output is compared
# with the one committed for that commit.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-8
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
D=$M/docs/superpowers/records/2026-09-15-file-split/task-8-files
O=$S/art/final/rerun; mkdir -p $O
tree() { [ -d $S/trees/api-src-$1 ] || { mkdir -p $S/trees/api-src-$1; git -C $M archive $1 rust/crates/rexx-api/src | tar -x -C $S/trees/api-src-$1; }; echo $S/trees/api-src-$1/rust/crates/rexx-api/src; }
for N in "$@"; do
  mapfile -t A < $D/c$N/args
  P=${A[0]}; DEST=${A[1]}; T=${A[2]}; OPTS=("${A[@]:3}")
  C=$(git -C $M log --format=%h -F --grep=task-8-files/c$N/ 5c7173fac..HEAD); PC=$(git -C $M rev-parse --short=9 $C^)
  X=; [ "$T" != none ] && X=--tests
  SPLIT_CRATE=rexx-api SPLIT_PARENT=${P%.rs} SPLIT_PARENT_RS=$P SPLIT_DESTS=$DEST SPLIT_TESTS_RS=$T python3 $S/tools/instruments.py $(tree $PC) $(tree $C) $D/c$N/removed.json $O/c$N $X "${OPTS[@]}" > /dev/null
  same=1; for k in instrument1 instrument2 instrument3 verdict; do
    cmp -s <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $D/c$N/c$N-$k.txt) <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $O/c$N-$k.txt) || same=0
  done
  echo "c$N $C (parent $PC): $(head -1 $O/c$N-verdict.txt); outputs $([ $same = 1 ] && echo identical to || echo DIFFER from) the committed ones"
done
