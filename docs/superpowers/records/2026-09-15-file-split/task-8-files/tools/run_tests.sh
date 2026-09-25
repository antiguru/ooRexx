#!/bin/bash
# usage: run_tests.sh WORKTREE TARGET OUTPREFIX
# TARGET is ignored: the suite runs with the worktree's default target
# directory, since tests/support/arity.rs runs rust/target/release/rexx-run
# by path whatever CARGO_TARGET_DIR says.
# Instrument 4 over WORKTREE as it stands: the release rexx-exec suite, every
# binary run (--no-fail-fast), reduced to one line per test and per result
# block. OUTPREFIX.meta records the tree, the load before and after, and the
# unpiped exit status.
W=$1; T=$2; O=$3
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-8
{
  echo "head $(git -C $W rev-parse HEAD)"
  echo "rust tree $(cd $W && git add -A -- rust ':!rust/corpus-l1' ':!rust/bench-baselines/pinned' >/dev/null 2>&1; git write-tree --prefix=rust/)"
  echo "load-before $(cat /proc/loadavg)"
} > $O.meta
(cd $W/rust && env -u CARGO_TARGET_DIR memcap 8G cargo test -j 4 --release -p rexx-exec --no-fail-fast > $O.log 2>&1)
echo "exit $?" >> $O.meta
echo "load-after $(cat /proc/loadavg)" >> $O.meta
python3 $S/tools/test_results.py $O.log > $O.results
echo finished >> $O.meta
