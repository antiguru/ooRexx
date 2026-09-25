#!/bin/bash
# usage: checks7.sh N PARENT_RS DESTS TESTS_RS [instrument options]
# Task 6's checks.sh for this task: fmt, clippy, instruments 1-3, the
# positional-prose scan, the comment pass (comments7.py), the cargo doc runs
# gated on docsig.py's signature (every warning's message and file) being
# BASE's, the refusal-sites refresh, the test-module intra-doc links, and the
# diff of every file outside the parent and the destinations. DESTS is comma
# separated, relative to src; TESTS_RS is the moved test module's file (or
# "none"), which switches the instruments to --tests.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; export SPLIT_PARENT_RS=$2 SPLIT_DESTS=$3 SPLIT_TESTS_RS=$4; shift 4
export SPLIT_PARENT=${SPLIT_PARENT_RS%.rs}
TESTS=; [ "$SPLIT_TESTS_RS" != none ] && TESTS=--tests
cd $R
cargo fmt --all --check > /dev/null; echo "fmt $?"
CARGO_TARGET_DIR=$S/t-check cargo clippy -j 8 -p rexx-exec --all-targets -- -D warnings > $S/art/c$N-clippy.txt 2>&1; echo "clippy $?" | tee -a $S/art/c$N-clippy.txt
python3 $S/tools/instruments.py $S/pre$N $R/crates/rexx-exec/src $S/c$N/removed.json $S/art/c$N $TESTS "$@"
python3 $S/tools/positional.py $S/pre$N $R/crates/rexx-exec/src > $S/art/c$N-positional.txt
python3 $S/tools/comments7.py $S/pre$N $R $SPLIT_PARENT_RS $SPLIT_DESTS $TESTS > $S/art/c$N-comments.txt
$S/tools/docs.sh $R $S/t-check $S/art/c$N
same=1; for k in doc doc-private doc-cfgtest; do
  diff <(python3 $S/tools/docsig.py $S/art/base/base-$k.txt) <(python3 $S/tools/docsig.py $S/art/c$N-$k.txt) > $S/art/c$N-$k-sigdiff.txt || same=0
done
echo "docs: signatures $([ $same = 1 ] && echo same as || echo DIFFER from) BASE's"
$S/tools/refusal_refresh.sh $R $S/t-check $S/art/c$N-refusal-sites.txt | sed -n 1,2p
rm -f $S/art/c$N-refusal-sites.txt.log
$S/tools/tests_links.sh $R $S/t-check $S/art/c$N-tests-links.txt | sed -n 3p
EXCL=":!crates/rexx-exec/src/$SPLIT_PARENT_RS"; for d in ${SPLIT_DESTS//,/ }; do EXCL="$EXCL :!crates/rexx-exec/src/$d"; done
git -C $R diff -- . $EXCL ':!corpus/refusal-sites.tsv' > $S/art/c$N-other-edits.diff
echo "other edits: $(/bin/grep -a -c '^diff --git' $S/art/c$N-other-edits.diff) files"
if [ -f $S/c$N/rules ]; then
  mapfile -t RULES < $S/c$N/rules
  python3 $S/tools/other_edits.py $R $S/art/c$N-other-edits-check.txt "${RULES[@]}" > /dev/null
  echo "other-edits check: $(tail -1 $S/art/c$N-other-edits-check.txt)"
fi
