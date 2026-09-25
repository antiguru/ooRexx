#!/bin/bash
# usage: prod.sh N PARENT_RS DESTS TESTS_RS [instrument options]
# A production commit after the move is in the working tree: verify.sh
# (checks, snapshot, instrument 4 on the test worktree), then the native
# table instrument on that same worktree, then commit_task6.sh. Stops at
# the first failure. Writes art/prod-cN.out, ending in DONE or FAILED.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-8
N=$1
O=$S/art/prod-c$N.out
{
  $S/tools/verify.sh "$@" || { echo FAILED; exit 1; }
  $S/tools/natives_check.sh $S/wt/rust $S/art/c$N-natives.txt | tee $S/art/c$N-natives-verdict.txt
  /bin/grep -q '^natives table identical to BASE' $S/art/c$N-natives-verdict.txt || { echo FAILED; exit 1; }
  $S/tools/commit_task6.sh $N || { echo FAILED; exit 1; }
  echo DONE
} > $O 2>&1
