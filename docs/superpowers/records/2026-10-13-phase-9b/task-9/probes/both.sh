#!/bin/bash
# usage: both.sh file.rex  -- runs oracle and crate from fresh empty dirs
f=$(readlink -f "$1"); in=$(readlink -f "${2:-/dev/null}")
base=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t9/runs
n=$(date +%s%N)
mkdir -p $base/o$n $base/c$n
cp "$f" $base/o$n/p.rex; cp "$f" $base/c$n/p.rex
cd $base/o$n
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 4G timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx p.rex >out 2>err <"$in"; echo $? >rc )
cd $base/c$n
( memcap 4G timeout -k 5 20 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run p.rex >out 2>err <"$in"; echo $? >rc )
for s in o c; do d=$base/$s$n; echo "== $s rc=$(cat $d/rc)"; echo "-- stdout"; cat $d/out; echo "-- stderr"; cat $d/err; done
