#!/bin/bash
# usage: natives_check.sh RUST_DIR TARGET OUT
# Patches dispatch.rs with natives_dump.py's temporary dump (restored from a
# copy afterwards), builds a debug rexx-run, bootstraps it on a one-line
# program, and compares the resolved table with natives-base.txt.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-2
R=$1; T=$2; OUT=$3
F=$R/crates/rexx-exec/src/dispatch.rs
cp $F $OUT.dispatch.orig
python3 $S/tools/natives_dump.py patch $F
(cd $R && CARGO_TARGET_DIR=$T cargo build -j 8 -p rexx-exec --bin rexx-run 2>&1 | grep -E '^error')
cp $OUT.dispatch.orig $F && rm $OUT.dispatch.orig
mkdir -p $S/probe-natives && printf 'say 1\n' > $S/probe-natives/p.rex
(cd $R && $T/debug/rexx-run $S/probe-natives/p.rex 2> $OUT.stderr > /dev/null)
python3 $S/tools/natives_dump.py resolve $T/debug/rexx-run $OUT.stderr > $OUT
rm $OUT.stderr
if cmp -s $OUT $S/natives-base.txt; then echo "natives table identical to BASE: $(wc -l < $OUT) entries"; else echo "NATIVES DIFFER"; diff $S/natives-base.txt $OUT | head; fi
