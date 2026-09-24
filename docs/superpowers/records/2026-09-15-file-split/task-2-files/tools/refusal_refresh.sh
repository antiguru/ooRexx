#!/bin/bash
# usage: refusal_refresh.sh TREE_RUST_DIR TARGET OUT
cd $1
BEFORE=$(stat -c %y corpus/refusal-sites.tsv); sleep 1
REXX_REFUSAL_SITES_REFRESH=1 CARGO_TARGET_DIR=$2 cargo test -j 8 -p rexx-exec --test refusal_sites -- --test-threads=1 > $3.log 2>&1
echo "refresh exit $?; table mtime before $BEFORE, after $(stat -c %y corpus/refusal-sites.tsv)" > $3
git diff --stat -- corpus/refusal-sites.tsv >> $3
git diff -U0 -- corpus/refusal-sites.tsv >> $3
echo "column 3 (surface) of every row, before (HEAD) vs after refresh:" >> $3
diff <(git show HEAD:rust/corpus/refusal-sites.tsv | awk -F'\t' '!/^#/{print $2"\t"$3}') <(awk -F'\t' '!/^#/{print $2"\t"$3}' corpus/refusal-sites.tsv) >> $3 && echo "  identical, $(awk -F'\t' '!/^#/' corpus/refusal-sites.tsv | wc -l) rows" >> $3
cat $3
