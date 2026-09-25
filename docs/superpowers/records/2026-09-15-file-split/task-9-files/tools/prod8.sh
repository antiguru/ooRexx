#!/bin/bash
# usage: prod8.sh N PARENT_RS DESTS TESTS_RS [instrument options]
# One commit after the move is in the working tree: verify8.sh (checks,
# snapshot, instrument 4 on the test worktree), then commit_task8.sh. Stops
# at the first failure. Writes art/prod-cN.out, ending in DONE or FAILED.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
N=$1
O=$S/art/prod-c$N.out
{
  $S/tools/verify8.sh "$@" || { echo FAILED; exit 1; }
  $S/tools/commit_task8.sh $N || { echo FAILED; exit 1; }
  echo DONE
} > $O 2>&1
