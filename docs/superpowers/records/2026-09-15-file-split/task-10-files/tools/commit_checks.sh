#!/bin/bash
# usage: commit_checks.sh N CHILD PRE_SRC [instrument options]   (runs in the dev worktree)
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
N=$1; CHILD=$2; PRE=$3; shift 3
cd $S/dev/rust
cargo fmt --all --check > /dev/null; echo "fmt $?"
CARGO_TARGET_DIR=$S/t-base cargo clippy -j 8 -p rexx-exec --all-targets -- -D warnings > $S/art/c$N-clippy.txt 2>&1; echo "clippy $?"
python3 $S/tools/instruments.py $PRE $S/dev/rust/crates/rexx-exec/src $S/c$N/removed.json $S/art/c$N "$@"
python3 $S/tools/positional.py $PRE/dispatch.rs $S/dev/rust/crates/rexx-exec/src $CHILD > $S/art/c$N-positional.txt
$S/tools/docs.sh $S/dev/rust $S/t-base $S/art/c$N
$S/tools/refusal_refresh.sh $S/dev/rust $S/t-base $S/art/c$N-refusal-sites.txt
rm -f $S/art/c$N-refusal-sites.txt.log
$S/tools/tests_links.sh $S/dev/rust $S/t-base $S/art/c$N-tests-links.txt > /dev/null; sed -n 2p $S/art/c$N-tests-links.txt
