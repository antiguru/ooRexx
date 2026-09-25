#!/bin/bash
# usage: verify8.sh N PARENT_RS DESTS TESTS_RS [instrument options]
# For commit N, over the main checkout's working tree: the per-commit checks
# (checks8.sh), a snapshot of every changed file under rust/, instrument 4 on
# a test worktree holding exactly HEAD plus that snapshot, and the results.
# Stops at the first failure; commit_task8.sh commits afterwards.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
N=$1
df -h /tmp | tail -1
$S/tools/checks8.sh "$@" > $S/art/checks-c$N.out 2>&1
cat $S/art/checks-c$N.out
/bin/grep -q '^fmt 0' $S/art/checks-c$N.out && /bin/grep -q '^clippy 0' $S/art/checks-c$N.out && /bin/grep -q '^PASS' $S/art/c$N-verdict.txt || { echo "CHECKS FAILED c$N"; exit 1; }
/bin/grep -q '^docs: signatures same as BASE.s' $S/art/checks-c$N.out || { echo "DOCS CHANGED c$N"; exit 1; }
/bin/grep -q '^  identical, ' $S/art/c$N-refusal-sites.txt && /bin/grep -q '^  identical$' $S/art/c$N-refusal-sites.txt || { echo "REFUSAL-SITES COLUMNS CHANGED c$N"; exit 1; }
/bin/grep -q '^doc exit: .*Generated' $S/art/c$N-tests-links.txt || { echo "TESTS-LINKS DOC FAILED c$N"; exit 1; }
/bin/grep -q "^unsafe: NONE" $S/art/checks-c$N.out || { echo "UNSAFE IN MOVED CODE c$N"; exit 1; }
[ -f $S/c$N/comment-pass.md ] || { echo "NO COMMENT PASS c$N"; exit 1; }
[ ! -f $S/c$N/rules ] || /bin/grep -q '^other-edits check: PASS' $S/art/checks-c$N.out || { echo "OTHER EDITS FAILED c$N"; exit 1; }
rm -rf $S/snap$N; $S/tools/snap.sh save $S/snap$N
$S/tools/i48.sh $N > /dev/null
/bin/grep -q '^compare exit 0' $S/art/i4-c$N.compare && /bin/grep -q '^exit 0' $S/art/i4-c$N.meta || { echo "INSTRUMENT 4 FAILED c$N"; cat $S/art/i4-c$N.compare; exit 1; }
cat $S/art/i4-c$N.compare $S/art/i4-c$N.meta
echo "VERIFIED c$N"
