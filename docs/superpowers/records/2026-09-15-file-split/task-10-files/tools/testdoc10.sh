#!/bin/bash
# usage: testdoc10.sh RUST_DIR TARGET OUT
# testdoc.py over the gate_table_c test target as RUST_DIR has it: the
# target's rustc invocation is captured afresh (the file touched so cargo
# recompiles it and prints the command), then replayed as rustdoc with
# private items and cfg(test) into an emptied directory. OUT gets rustdoc's
# exit, its warning count, and the warnings.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
cd $1
CARGO_TARGET_DIR=$2 cargo test -j 8 --no-run -p rexx-exec --test gate_table_c > /dev/null 2>&1
touch crates/rexx-exec/tests/gate_table_c.rs
CARGO_TARGET_DIR=$2 cargo test -j 8 --no-run -v -p rexx-exec --test gate_table_c 2>&1 | /bin/grep -a 'Running `.*--crate-name gate_table_c ' | head -1 > $3.cmd
[ -s $3.cmd ] || { echo "no rustc command captured" > $3; cat $3; exit 1; }
rm -rf $S/testdoc-out
python3 $S/tools/testdoc.py $3.cmd . $S/testdoc-out $PWD/crates/rexx-exec > $3
rm $3.cmd
head -2 $3
