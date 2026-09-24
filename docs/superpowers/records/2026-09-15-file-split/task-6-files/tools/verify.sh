#!/bin/bash
# usage: verify.sh N PARENT_RS DESTS TESTS_RS [instrument options]
# For commit N, over the main checkout's working tree: the per-commit checks
# (checks.sh), a snapshot of every changed file under rust/, instrument 4 on
# a test worktree holding exactly HEAD plus that snapshot, and the results.
# Stops at the first failure; commit_task6.sh commits afterwards.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-6
N=$1
df -h /tmp | tail -1
$S/tools/checks.sh "$@" > $S/art/checks-c$N.out 2>&1
cat $S/art/checks-c$N.out
/bin/grep -q '^fmt 0' $S/art/checks-c$N.out && /bin/grep -q '^clippy 0' $S/art/checks-c$N.out && /bin/grep -q '^PASS' $S/art/c$N-verdict.txt || { echo "CHECKS FAILED c$N"; exit 1; }
rm -rf $S/snap$N; $S/tools/snap.sh save $S/snap$N
$S/tools/i4.sh $N > /dev/null
/bin/grep -q '^compare exit 0' $S/art/i4-c$N.compare && /bin/grep -q '^exit 0' $S/art/i4-c$N.meta || { echo "INSTRUMENT 4 FAILED c$N"; cat $S/art/i4-c$N.compare; exit 1; }
cat $S/art/i4-c$N.compare $S/art/i4-c$N.meta
echo "VERIFIED c$N"
