#!/bin/bash
# usage: docs.sh TREE_RUST_DIR TARGET OUTPREFIX
# cargo doc for rexx-exec three ways (public, --document-private-items, and
# that with --cfg test), each log kept, with its warning count and the count
# located in the files this commit touches: SPLIT_PARENT_RS and every file
# under its directory (`eval.rs` and `eval/*.rs`).
P=${SPLIT_PARENT_RS%.rs}
PAT="--> crates/rexx-exec/src/$P(\\.rs|/)"
cd $1
CARGO_TARGET_DIR=$2 cargo doc -j 8 --no-deps -p rexx-exec > $3-doc.txt 2>&1; echo "exit $?" >> $3-doc.txt
CARGO_TARGET_DIR=$2 RUSTDOCFLAGS=--document-private-items cargo doc -j 8 --no-deps -p rexx-exec > $3-doc-private.txt 2>&1; echo "exit $?" >> $3-doc-private.txt
CARGO_TARGET_DIR=$2 RUSTDOCFLAGS="--cfg test --document-private-items" cargo doc -j 8 --no-deps -p rexx-exec > $3-doc-cfgtest.txt 2>&1; echo "exit $?" >> $3-doc-cfgtest.txt
for f in $3-doc.txt $3-doc-private.txt $3-doc-cfgtest.txt; do
  echo "$f: $(/bin/grep -a -c '^warning' $f) warning lines, $(/bin/grep -a -E -c -- "$PAT" $f) located in $P.rs or $P/, $(tail -1 $f)"
  /bin/grep -a -E -- "$PAT" $f | sed 's/^/    /'
done
