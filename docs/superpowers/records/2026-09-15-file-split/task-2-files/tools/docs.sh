#!/bin/bash
# usage: docs.sh TREE_RUST_DIR TARGET OUTPREFIX
cd $1
CARGO_TARGET_DIR=$2 cargo doc -j 8 --no-deps -p rexx-exec > $3-doc.txt 2>&1; echo "exit $?" >> $3-doc.txt
CARGO_TARGET_DIR=$2 RUSTDOCFLAGS=--document-private-items cargo doc -j 8 --no-deps -p rexx-exec > $3-doc-private.txt 2>&1; echo "exit $?" >> $3-doc-private.txt
for f in $3-doc.txt $3-doc-private.txt; do
  echo "$f: $(/bin/grep -a -c '^warning' $f) warning lines, $(/bin/grep -a -E -c -- '--> crates/rexx-exec/src/dispatch' $f) located in dispatch files, $(tail -1 $f)"
done
