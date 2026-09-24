#!/bin/bash
# usage: tests_links.sh RUST_DIR TARGET OUT
# rustdoc never renders a #[test] fn's doc, so the intra-doc links of the
# crate root's test module (inline in lib.rs at BASE, tests.rs after the
# move) are unchecked by every `cargo doc`. This strips the `#[test]` lines
# from copies of lib.rs and tests.rs (restored afterwards), documents with
# cfg(test) on, and reports the warnings located in those files.
D=$1/crates/rexx-exec/src
FILES="$D/lib.rs"; [ -f $D/tests.rs ] && FILES="$FILES $D/tests.rs"
mkdir -p $3.orig
for F in $FILES; do cp $F $3.orig/$(basename $F); sed -i 's/^\( *\)#\[test\]$//' $F; done
(cd $1 && CARGO_TARGET_DIR=$2 RUSTDOCFLAGS="--cfg test --document-private-items" cargo doc -j 8 --no-deps -p rexx-exec > $3.log 2>&1)
for F in $FILES; do cp $3.orig/$(basename $F) $F; done
rm -r $3.orig
{ echo "doc exit: $(tail -1 $3.log)"; echo "files: $(for F in $FILES; do basename $F; done | tr '\n' ' ')"; echo "warnings located in lib.rs or tests.rs: $(/bin/grep -a -c -E -- '--> crates/rexx-exec/src/(lib|tests)\.rs' $3.log)"; /bin/grep -a -B1 -A4 -E -- '--> crates/rexx-exec/src/(lib|tests)\.rs' $3.log; } > $3
rm $3.log
cat $3
