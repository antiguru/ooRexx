#!/bin/bash
# Re-runs instruments 1-3 with the final tooling on every commit pair: PRE is
# the parent commit's tree, POST the commit's, both from `git archive`; the
# outputs are compared with the ones committed for that commit.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
D=$M/docs/superpowers/records/2026-09-15-file-split/task-5-files
O=$S/art/final/rerun; mkdir -p $O
tree() { [ -d $S/trees/$1 ] || { mkdir -p $S/trees/$1; git -C $M archive $1 rust/crates/rexx-exec/src | tar -x -C $S/trees/$1; }; echo $S/trees/$1/rust/crates/rexx-exec/src; }
while read N P DEST T OPT; do
  C=$(git -C $M log --format=%h -F --grep=task-5-files/c$N/); PC=$(git -C $M rev-parse --short=9 $C^)
  X=; [ "$T" != none ] && X=--tests; [ "$OPT" = - ] && OPT=
  [ "$OPT" = E4 ] && OPT="--expect-edit=tests::fn the_small_int_fast_path_answers_what_the_general_path_answers"
  SPLIT_PARENT=${P%.rs} SPLIT_PARENT_RS=$P SPLIT_DESTS=$DEST SPLIT_TESTS_RS=$T python3 $S/tools/instruments.py $(tree $PC) $(tree $C) $D/c$N/removed.json $O/c$N $X ${OPT:+"$OPT"} > /dev/null
  same=1; for k in instrument1 instrument2 instrument3 verdict; do
    # tmp paths in cmp messages differ run to run
    cmp -s <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $D/c$N/c$N-$k.txt) <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $O/c$N-$k.txt) || same=0
  done
  echo "c$N $C (parent $PC): $(head -1 $O/c$N-verdict.txt); outputs $([ $same = 1 ] && echo identical to || echo DIFFER from) the committed ones"
done <<'L'
1 value.rs value/tests.rs value/tests.rs -
2 plan.rs plan/tests.rs plan/tests.rs -
3 builtin.rs builtin/tests.rs builtin/tests.rs -
4 eval.rs eval/tests.rs eval/tests.rs E4
5 eval.rs eval/object_operand_tests.rs eval/object_operand_tests.rs -
6 parse_template.rs parse_template/tests.rs parse_template/tests.rs -
7 parse_template.rs version.rs none -
8 environment.rs environment/tests.rs environment/tests.rs -
9 environment.rs environment/route.rs none -
10 environment.rs environment/identities.rs none -
11 trace.rs trace/format.rs none -
L
