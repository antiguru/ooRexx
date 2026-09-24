#!/bin/bash
# usage: tests_links.sh RUST_DIR TARGET OUT
# rustdoc never renders a #[test] fn's doc, so the intra-doc links of run's
# test modules (run/tests.rs, run/tests/*.rs) are unchecked by every
# `cargo doc`. This strips the `#[test]` lines from copies (restored
# afterwards), documents with cfg(test) on, and reports the warnings located
# in those files.
D=$1/crates/rexx-exec/src/run
FILES="$D/tests.rs $(ls $D/tests/*.rs)"
mkdir -p $3.orig
for F in $FILES; do cp $F $3.orig/$(basename $F); sed -i 's/^\( *\)#\[test\]$//' $F; done
(cd $1 && CARGO_TARGET_DIR=$2 RUSTDOCFLAGS="--cfg test --document-private-items" cargo doc -j 8 --no-deps -p rexx-exec > $3.log 2>&1)
for F in $FILES; do cp $3.orig/$(basename $F) $F; done
rm -r $3.orig
{ echo "doc exit: $(tail -1 $3.log)"; echo "files: $(echo $FILES | wc -w) (run/tests.rs and run/tests/*.rs)"; echo "warnings located in run/tests.rs or run/tests/: $(/bin/grep -a -c -E -- '--> crates/rexx-exec/src/run/tests' $3.log)"; /bin/grep -a -B1 -A4 -E -- '--> crates/rexx-exec/src/run/tests' $3.log; } > $3
rm $3.log
cat $3
