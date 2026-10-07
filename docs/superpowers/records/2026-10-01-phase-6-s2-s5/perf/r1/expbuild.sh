#!/bin/bash
# usage: expbuild.sh NAME : build trees/exp into bin/NAME
S=/tmp/claude-1000/p6-t26
find $S/trees/exp -type f -newer $S/trees/exp/rust/Cargo.toml -exec touch {} + 2>/dev/null
( cd $S/trees/exp/rust && CARGO_TARGET_DIR=$S/tgt/exp memcap 8G cargo build --release -p rexx-exec --bin rexx-run > $S/logs/build-$1.log 2>&1 )
echo "$1 exit=$? compiling=$(/bin/grep -a -c 'Compiling rexx-exec' $S/logs/build-$1.log)"
mkdir -p $S/bin/$1 && cp $S/tgt/exp/release/rexx-run $S/bin/$1/rexx-run
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
bash rust/bench-programs/callgrind.sh -r 1 -j 4 -p "${PROGS:-sendloop dispatch dispatchclass fibfunc fibcall}" -o $S/cgx-$1 head=$S/bin/head/rexx-run $1=$S/bin/$1/rexx-run
