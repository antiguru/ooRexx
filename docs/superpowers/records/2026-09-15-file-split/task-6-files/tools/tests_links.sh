#!/bin/bash
# usage: tests_links.sh RUST_DIR TARGET OUT
# rustdoc never renders a #[test] fn's doc, so the intra-doc links of a test
# module are unchecked by every `cargo doc`. This strips the `#[test]` lines
# from copies of SPLIT_PARENT_RS and every file under its directory
# (restored afterwards), documents with cfg(test) on, and reports the
# warnings located in those files: the test module's links whether it is
# inline (before the move) or a file of its own (after).
D=$1/crates/rexx-exec/src
P=${SPLIT_PARENT_RS%.rs}
FILES="$D/$P.rs"; [ -d $D/$P ] && FILES="$FILES $(find $D/$P -name '*.rs' | sort)"
mkdir -p $3.orig
i=0; for F in $FILES; do cp $F $3.orig/$i; sed -i 's/^\( *\)#\[test\]$//' $F; i=$((i+1)); done
(cd $1 && CARGO_TARGET_DIR=$2 RUSTDOCFLAGS="--cfg test --document-private-items" cargo doc -j 8 --no-deps -p rexx-exec > $3.log 2>&1)
i=0; for F in $FILES; do cp $3.orig/$i $F; i=$((i+1)); done
rm -r $3.orig
PAT="--> crates/rexx-exec/src/$P(\\.rs|/)"
{ echo "doc exit: $(tail -1 $3.log)"; echo "files: $(for F in $FILES; do echo ${F#$D/}; done | tr '\n' ' ')"; echo "warnings located in $P.rs or $P/: $(/bin/grep -a -c -E -- "$PAT" $3.log)"; /bin/grep -a -B1 -A4 -E -- "$PAT" $3.log; } > $3
rm $3.log
cat $3
