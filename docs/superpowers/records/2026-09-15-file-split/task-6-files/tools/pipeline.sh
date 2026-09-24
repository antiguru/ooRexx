#!/bin/bash
# usage: pipeline.sh FIRST LAST
# For each commit N: restore snapshot N over HEAD, re-run the per-commit
# checks, confirm the working tree is byte-identical to the snapshot, run
# instrument 4 on it, and commit. Stops at the first failure.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-6
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
CHILD=(x indent raised select interpret settings condition call loops)
EXPECT=(x "--expect-edit=fn indent_in_range" "--expect-edit=fn raised_when_not_logical" "" "--expect-edit=impl Interp::fn exec_instruction" "" "--expect-edit=impl Interp::fn signal_to_label" "" "")
cd $M
for N in $(seq $1 $2); do
  echo "== c$N $(date +%T)"
  $S/tools/snap.sh restore $S/snap$N > /dev/null
  if [ -n "${EXPECT[$N]}" ]; then $S/tools/checks.sh $N ${CHILD[$N]} "${EXPECT[$N]}"; else $S/tools/checks.sh $N ${CHILD[$N]}; fi > $S/art/checks-c$N.out 2>&1
  /bin/grep -q '^fmt 0' $S/art/checks-c$N.out && /bin/grep -q '^clippy 0' $S/art/checks-c$N.out && /bin/grep -q '^PASS' $S/art/c$N-verdict.txt || { echo "checks failed c$N"; cat $S/art/checks-c$N.out; exit 1; }
  bad=0; for f in $(cat $S/snap$N/files); do cmp -s $f $S/snap$N/tree/$f || { echo "DIFF $f"; bad=1; }; done
  [ -z "$(comm -23 <({ git diff --name-only HEAD -- rust; git ls-files --others --exclude-standard -- rust; } | sort -u) <(sort /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-6/snap$N/files))" ] || { echo "a changed file outside the snapshot"; git status --short; bad=1; }
  [ $bad = 0 ] || exit 1
  $S/tools/i4.sh $N > /dev/null
  /bin/grep -q '^compare exit 0' $S/art/i4-c$N.compare && /bin/grep -q '^exit 0' $S/art/i4-c$N.meta || { echo "instrument 4 failed c$N"; cat $S/art/i4-c$N.compare; exit 1; }
  python3 $S/tools/test_results.py $S/art/i4-c$N.log > $S/art/i4-c$N.results
  $S/tools/commit_run.sh $N || exit 1
done
echo "pipeline finished $(date +%T)"
