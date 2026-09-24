#!/bin/bash
# usage: checks.sh N CHILD [instrument options]
# Per-commit checks over the main checkout's working tree: fmt, clippy,
# instruments 1-3 against pre<N>/, the positional-prose scan, the cargo doc
# runs, the refusal-sites refresh, the test-module intra-doc links, and the
# diff of every file outside run.rs and the new child.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-3b
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; CHILD=$2; shift 2
export SPLIT_PARENT=run
cd $R
cargo fmt --all --check > /dev/null; echo "fmt $?"
CARGO_TARGET_DIR=$S/t-check cargo clippy -j 8 -p rexx-exec --all-targets -- -D warnings > $S/art/c$N-clippy.txt 2>&1; echo "clippy $?" | tee -a $S/art/c$N-clippy.txt
python3 $S/tools/instruments.py $S/pre$N $R/crates/rexx-exec/src $S/c$N/removed.json $S/art/c$N "$@"
python3 $S/tools/positional.py $S/pre$N/run.rs $R/crates/rexx-exec/src $CHILD > $S/art/c$N-positional.txt
$S/tools/docs.sh $R $S/t-check $S/art/c$N
$S/tools/refusal_refresh.sh $R $S/t-check $S/art/c$N-refusal-sites.txt | sed -n 1,2p
rm -f $S/art/c$N-refusal-sites.txt.log
$S/tools/tests_links.sh $R $S/t-check $S/art/c$N-tests-links.txt | sed -n 3p
git -C $R diff -- . ':!crates/rexx-exec/src/run.rs' ':!corpus/refusal-sites.tsv' > $S/art/c$N-other-edits.diff
echo "other edits: $(/bin/grep -a -c '^diff --git' $S/art/c$N-other-edits.diff) files"
