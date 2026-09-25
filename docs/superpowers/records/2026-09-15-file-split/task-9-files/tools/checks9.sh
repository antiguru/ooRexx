#!/bin/bash
# usage: checks9.sh N PARENT_RS DESTS [instrument options]
# checks8.sh for rexx-parse (SPLIT_CRATE): fmt, clippy over the whole
# workspace (rexx-parse's public paths are named by other crates), instruments
# 1-3, the positional-prose scan, the comment pass listing (comments7.py,
# widened word list), the unsafe count (unsafe_count.sh), the cargo doc runs
# gated on docsig.py's signature being BASE's, the public-API doc manifest
# (docapi.sh) compared with BASE's, the refusal-sites refresh, the test-module
# intra-doc links, and the diff of every file outside the parent and the
# destinations.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; export SPLIT_PARENT_RS=$2 SPLIT_DESTS=$3 SPLIT_TESTS_RS=none SPLIT_CRATE=rexx-parse; shift 3
export SPLIT_PARENT=${SPLIT_PARENT_RS%.rs}
SRC=$R/crates/rexx-parse/src
cd $R
cargo fmt --all --check > /dev/null; echo "fmt $?"
CARGO_TARGET_DIR=$S/t-check cargo clippy -j 8 --workspace --all-targets -- -D warnings > $S/art/c$N-clippy.txt 2>&1; echo "clippy $?" | tee -a $S/art/c$N-clippy.txt
python3 $S/tools/instruments.py $S/pre$N $SRC $S/c$N/removed.json $S/art/c$N "$@"
python3 $S/tools/positional.py $S/pre$N $SRC > $S/art/c$N-positional.txt
python3 $S/tools/comments7.py $S/pre$N $R $SPLIT_PARENT_RS $SPLIT_DESTS > $S/art/c$N-comments.txt
$S/tools/unsafe_count.sh $S/pre$N $SRC $S/c$N/removed.json $SPLIT_PARENT_RS $SPLIT_DESTS > $S/art/c$N-unsafe.txt; tail -1 $S/art/c$N-unsafe.txt
$S/tools/docs.sh $R $S/t-check $S/art/c$N
same=1; for k in doc doc-private doc-cfgtest; do
  diff <(python3 $S/tools/docsig.py $S/art/base/base-$k.txt) <(python3 $S/tools/docsig.py $S/art/c$N-$k.txt) > $S/art/c$N-$k-sigdiff.txt || same=0
done
echo "docs: signatures $([ $same = 1 ] && echo same as || echo DIFFER from) BASE's"
$S/tools/docapi.sh $R $S/t-check $S/art/c$N-docapi.txt
diff $S/art/base/base-docapi.txt $S/art/c$N-docapi.txt > $S/art/c$N-docapi-diff.txt && echo "docapi: manifest same as BASE's" || echo "docapi: manifest DIFFERS from BASE's"
$S/tools/refusal_refresh.sh $R $S/t-check $S/art/c$N-refusal-sites.txt | sed -n 1,2p
rm -f $S/art/c$N-refusal-sites.txt.log
$S/tools/tests_links.sh $R $S/t-check $S/art/c$N-tests-links.txt | sed -n 3p
$S/tools/other_edits9.sh $N $SPLIT_PARENT_RS $SPLIT_DESTS
