#!/bin/bash
# usage: tests_links.sh RUST_DIR TARGET OUT
# rustdoc never renders a #[test] fn's doc, so dispatch/tests.rs's intra-doc
# links are unchecked by every `cargo doc`. This strips the `#[test]` lines
# from a copy (restored afterwards), documents with cfg(test) on, and reports
# the warnings located in dispatch/tests.rs.
F=$1/crates/rexx-exec/src/dispatch/tests.rs
# BASE keeps the module inline in dispatch.rs.
[ -f $F ] || F=$1/crates/rexx-exec/src/dispatch.rs
REL=${F#$1/}
cp $F $3.orig
sed -i 's/^\( *\)#\[test\]$//' $F
(cd $1 && CARGO_TARGET_DIR=$2 RUSTDOCFLAGS="--cfg test --document-private-items" cargo doc -j 8 --no-deps -p rexx-exec > $3.log 2>&1)
cp $3.orig $F && rm $3.orig
{ echo "doc exit: $(tail -1 $3.log)"; echo "warnings located in $REL: $(/bin/grep -a -c -- "--> $REL" $3.log)"; /bin/grep -a -B1 -A4 -- "--> $REL" $3.log; } > $3
rm $3.log
cat $3
