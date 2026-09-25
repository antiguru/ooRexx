#!/bin/bash
# usage: refusal_refresh.sh TREE_RUST_DIR TARGET OUT
# Re-derives corpus/refusal-sites.tsv, then shows (1) that the refresh ran
# (the table's mtime moved), (2) the rows it changed, and (3) that every
# column other than column 4 (the location) is identical, row for row, to
# the committed table's.
cd $1
BEFORE=$(stat -c %y corpus/refusal-sites.tsv); sleep 1
REXX_REFUSAL_SITES_REFRESH=1 CARGO_TARGET_DIR=$2 cargo test -j 8 -p rexx-exec --test refusal_sites -- --test-threads=1 > $3.log 2>&1
echo "refresh exit $?; table mtime before $BEFORE, after $(stat -c %y corpus/refusal-sites.tsv)" > $3
git diff --stat -- corpus/refusal-sites.tsv >> $3
git diff -U0 -- corpus/refusal-sites.tsv >> $3
echo "every column but column 4, before (HEAD) vs after refresh:" >> $3
cut4() { awk -F'\t' 'BEGIN{OFS="\t"} !/^#/{$4="<loc>"; print}'; }
diff <(git show HEAD:rust/corpus/refusal-sites.tsv | cut4) <(cut4 < corpus/refusal-sites.tsv) >> $3 && echo "  identical, $(awk -F'\t' '!/^#/' corpus/refusal-sites.tsv | wc -l) rows" >> $3
echo "header (comment) lines, before vs after:" >> $3
diff <(git show HEAD:rust/corpus/refusal-sites.tsv | /bin/grep -a '^#') <(/bin/grep -a '^#' corpus/refusal-sites.tsv) >> $3 && echo "  identical" >> $3
echo "column 4 changes, by row (constructor: before -> after):" >> $3
join -t$'\t' <(git show HEAD:rust/corpus/refusal-sites.tsv | awk -F'\t' '!/^#/{print $1"/"$2"\t"$4}' | sort) <(awk -F'\t' '!/^#/{print $1"/"$2"\t"$4}' corpus/refusal-sites.tsv | sort) | awk -F'\t' '$2!=$3{print "  "$1": "$2" -> "$3}' >> $3
cat $3
