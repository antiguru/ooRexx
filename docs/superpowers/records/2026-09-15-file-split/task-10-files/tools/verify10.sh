#!/bin/bash
# usage: verify10.sh N PARENT_RS DESTS [instrument options]   (SPLIT_CRATE, SPLIT_ROOT)
# verify9.sh for Task 10: the per-commit checks (checks10.sh), a snapshot of
# every changed file under rust/, instrument 4 on a test worktree holding
# exactly HEAD plus that snapshot, and the results. Stops at the first
# failure; commit_task10.sh commits afterwards.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
N=$1
df -h /tmp | tail -1
FREE=$(df --output=avail -B1G /tmp | tail -1 | tr -d ' '); [ "$FREE" -ge 10 ] || { echo "UNDER 10 GB FREE IN /tmp c$N"; exit 1; }
$S/tools/checks10.sh "$@" > $S/art/checks-c$N.out 2>&1
cat $S/art/checks-c$N.out
/bin/grep -q '^fmt 0' $S/art/checks-c$N.out && /bin/grep -q '^clippy 0' $S/art/checks-c$N.out && /bin/grep -q '^PASS' $S/art/c$N-verdict.txt || { echo "CHECKS FAILED c$N"; exit 1; }
/bin/grep -q '^docs: signatures same as BASE.s' $S/art/checks-c$N.out || { echo "DOCS CHANGED c$N"; exit 1; }
/bin/grep -q '^docapi: manifest same as BASE.s' $S/art/checks-c$N.out || { echo "PUBLIC API DOCS CHANGED c$N"; exit 1; }
/bin/grep -q '^  identical, ' $S/art/c$N-refusal-sites.txt && /bin/grep -q '^  identical$' $S/art/c$N-refusal-sites.txt || { echo "REFUSAL-SITES COLUMNS CHANGED c$N"; exit 1; }
if [ $SPLIT_ROOT = tests ]; then
  /bin/grep -q '^testdoc: rustdoc exit 0; warnings same as BASE.s' $S/art/checks-c$N.out || { echo "TEST-TARGET RUSTDOC CHANGED c$N"; exit 1; }
  /bin/grep -q '^targets: same as BASE.s' $S/art/checks-c$N.out || { echo "TEST TARGETS CHANGED c$N"; exit 1; }
else
  /bin/grep -q '^doc exit: .*Generated' $S/art/c$N-tests-links.txt && /bin/grep -q ': 0$' <(sed -n 3p $S/art/c$N-tests-links.txt) || { echo "TESTS-LINKS DOC FAILED c$N"; exit 1; }
fi
/bin/grep -q "^unsafe: NONE" $S/art/checks-c$N.out || { echo "UNSAFE IN MOVED CODE c$N"; exit 1; }
/bin/grep -q '^other edits: 0 files' $S/art/checks-c$N.out || { echo "OTHER FILES EDITED c$N"; exit 1; }
/bin/grep -q '^untracked: 0 files' $S/art/checks-c$N.out || { echo "UNTRACKED FILES UNDER rust/ c$N"; exit 1; }
[ -f $S/c$N/comment-pass.md ] || { echo "NO COMMENT PASS c$N"; exit 1; }
rm -rf $S/snap$N; $S/tools/snap.sh save $S/snap$N
$S/tools/i410.sh $N > /dev/null
/bin/grep -q '^compare exit 0' $S/art/i4-c$N.compare && /bin/grep -q '^exit 0' $S/art/i4-c$N.meta || { echo "INSTRUMENT 4 FAILED c$N"; cat $S/art/i4-c$N.compare; exit 1; }
cat $S/art/i4-c$N.compare $S/art/i4-c$N.meta
echo "VERIFIED c$N"
