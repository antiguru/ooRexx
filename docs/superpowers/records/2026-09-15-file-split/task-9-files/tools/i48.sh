#!/bin/bash
# usage: i48.sh N
# i4.sh for this task: the test worktree at the main checkout's HEAD plus
# snapshot N, run_tests8.sh, and the comparison with BASE's results.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
N=$1
$S/tools/sync_wt.sh $S/snap$N > $S/art/i4-c$N.sync
$S/tools/run_tests8.sh $S/wt $S/art/i4-c$N
python3 $S/tools/test_results.py --compare $S/art/i4-base.log $S/art/i4-c$N.log > $S/art/i4-c$N.compare
echo "compare exit $?" >> $S/art/i4-c$N.compare
cat $S/art/i4-c$N.compare
