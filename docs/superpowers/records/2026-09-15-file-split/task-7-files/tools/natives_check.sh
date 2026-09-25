#!/bin/bash
# usage: natives_check.sh RUST_DIR OUT
# Patches RUST_DIR's dispatch.rs with natives_dump.py's temporary dump
# (restored from a copy afterwards), builds a debug rexx-run in RUST_DIR's
# default target directory, bootstraps it on a one-line program run from an
# empty directory, and compares the resolved table with natives-base.txt
# (written by the first run, at BASE, when it does not exist yet).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-7
R=$1; OUT=$2
F=$R/crates/rexx-exec/src/dispatch.rs
cp $F $OUT.dispatch.orig
python3 $S/tools/natives_dump.py patch $F
(cd $R && env -u CARGO_TARGET_DIR cargo build -j 8 -p rexx-exec --bin rexx-run 2>&1 | grep -E '^error')
cp $OUT.dispatch.orig $F && rm $OUT.dispatch.orig
mkdir -p $S/probe-natives && printf 'say 1\n' > $S/probe-natives/p.rex
(cd $S/probe-natives && $R/target/debug/rexx-run p.rex 2> $OUT.stderr > /dev/null)
python3 $S/tools/natives_dump.py resolve $R/target/debug/rexx-run $OUT.stderr > $OUT
rm $OUT.stderr
[ -f $S/natives-base.txt ] || { cp $OUT $S/natives-base.txt; echo "natives-base.txt written: $(wc -l < $OUT) entries"; }
if cmp -s $OUT $S/natives-base.txt; then echo "natives table identical to BASE: $(wc -l < $OUT) entries"; else echo "NATIVES DIFFER"; diff $S/natives-base.txt $OUT | head; fi
