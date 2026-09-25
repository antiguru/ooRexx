#!/bin/bash
# usage: checks10.sh N PARENT_RS DESTS [instrument options]   (SPLIT_CRATE, SPLIT_ROOT)
# checks9.sh for Task 10's two crates. For every commit: fmt, clippy over
# the whole workspace, instruments 1-3 (PRE and POST are
# crates/$SPLIT_CRATE/$SPLIT_ROOT), the positional-prose scan, the comment
# pass listing (comments10.py), the unsafe count, `cargo doc` of rexx-extract
# three ways with its warning signatures compared with BASE's, the
# rexx-extract public-API manifest compared with BASE's, the refusal-sites
# refresh, and the diff of every file outside the parent and destinations,
# tracked and untracked, and each declared extra edit (S/c<N>/extra)
# against its rule (extra_edits10.py). For the test crate (SPLIT_ROOT=tests): rustdoc over
# the gate_table_c target (testdoc10.sh) and the rexx-exec target list
# against BASE's. For rexx-extract (SPLIT_ROOT=src): the test-module
# intra-doc links (tests_links.sh).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; export SPLIT_PARENT_RS=$2 SPLIT_DESTS=$3 SPLIT_TESTS_RS=none; shift 3
[ -n "$SPLIT_CRATE" ] && [ -n "$SPLIT_ROOT" ] || { echo "SPLIT_CRATE and SPLIT_ROOT must be set"; exit 2; }
export SPLIT_PARENT=${SPLIT_PARENT_RS%.rs}
SRC=$R/crates/$SPLIT_CRATE/$SPLIT_ROOT
cd $R
cargo fmt --all --check > /dev/null; echo "fmt $?"
CARGO_TARGET_DIR=$S/t-check cargo clippy -j 8 --workspace --all-targets -- -D warnings > $S/art/c$N-clippy.txt 2>&1; echo "clippy $?" | tee -a $S/art/c$N-clippy.txt
python3 $S/tools/instruments.py $S/pre$N $SRC $S/c$N/removed.json $S/art/c$N "$@"
python3 $S/tools/positional.py $S/pre$N $SRC > $S/art/c$N-positional.txt
python3 $S/tools/comments10.py $S/pre$N $R $SPLIT_PARENT_RS $SPLIT_DESTS > $S/art/c$N-comments.txt
$S/tools/unsafe_count.sh $S/pre$N $SRC $S/c$N/removed.json $SPLIT_PARENT_RS $SPLIT_DESTS > $S/art/c$N-unsafe.txt; tail -1 $S/art/c$N-unsafe.txt
# rexx-extract's docs at every commit (the brief's `cargo doc --no-deps -p
# rexx-extract` before and after), with its own parent for the "located in"
# column.
(export SPLIT_CRATE=rexx-extract SPLIT_PARENT_RS=docs/classes.rs; $S/tools/docs.sh $R $S/t-check $S/art/c$N)
same=1; for k in doc doc-private doc-cfgtest; do
  diff <(python3 $S/tools/docsig.py $S/art/base/base-$k.txt) <(python3 $S/tools/docsig.py $S/art/c$N-$k.txt) > $S/art/c$N-$k-sigdiff.txt || same=0
done
echo "docs: signatures $([ $same = 1 ] && echo same as || echo DIFFER from) BASE's"
$S/tools/docapi10.sh $R $S/t-check $S/art/c$N-docapi.txt rexx-extract
diff $S/art/base/base-docapi-rexx-extract.txt $S/art/c$N-docapi.txt > $S/art/c$N-docapi-diff.txt && echo "docapi: manifest same as BASE's" || echo "docapi: manifest DIFFERS from BASE's"
$S/tools/refusal_refresh.sh $R $S/t-check $S/art/c$N-refusal-sites.txt | sed -n 1,2p
rm -f $S/art/c$N-refusal-sites.txt.log
if [ $SPLIT_ROOT = tests ]; then
  $S/tools/testdoc10.sh $R $S/t-check $S/art/c$N-testdoc.txt > /dev/null
  diff <(sed 1d $S/art/base/base-testdoc.txt) <(sed 1d $S/art/c$N-testdoc.txt) > /dev/null && echo "testdoc: $(head -1 $S/art/c$N-testdoc.txt); warnings same as BASE's" || echo "testdoc: $(head -1 $S/art/c$N-testdoc.txt); warnings DIFFER from BASE's"
  cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c "
import json,sys
d=json.load(sys.stdin)
for p in d['packages']:
  if p['name'] in ('rexx-exec','rexx-extract'):
    for t in sorted(p['targets'], key=lambda t:(t['kind'],t['name'])): print(p['name'], ','.join(t['kind']), t['name'], t['src_path'].split('/rust/')[1])
" > $S/art/c$N-targets.txt
  diff $S/art/base/base-targets.txt $S/art/c$N-targets.txt > /dev/null && echo "targets: same as BASE's ($(wc -l < $S/art/c$N-targets.txt))" || echo "targets: DIFFER from BASE's"
else
  $S/tools/tests_links.sh $R $S/t-check $S/art/c$N-tests-links.txt | sed -n 3p
fi
$S/tools/other_edits10.sh $N $SPLIT_PARENT_RS $SPLIT_DESTS
if [ -f $S/c$N/extra ]; then mapfile -t X < $S/c$N/extra; python3 $S/tools/extra_edits10.py $R $S/art/c$N-extra-edits.txt "${X[@]}" | tail -1; fi
