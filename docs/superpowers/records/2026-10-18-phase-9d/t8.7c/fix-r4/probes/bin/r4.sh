#!/bin/bash
# usage: r4.sh FILE.rex [which...]  (o oracle, b rexx-head-cd070fd13, h rexx-head-d8614a7c9)
f=$(readlink -f "$1"); shift
d=$(dirname "$f"); n=$(basename "$f" .rex)
P=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c
run=/tmp/claude-1000/t87c-fix4-run
for w in ${*:-o b h}; do
  rm -rf /tmp/claude-1000/t87c-fix4-run; mkdir $run
  out=$d/out/$n.$w; mkdir -p $d/out
  cp "$f" $run/
  pushd $run >/dev/null
  case $w in
    o) ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $n.rex ) </dev/null >$out.out 2>$out.err; echo $? >$out.rc;;
    b) memcap 2G timeout -k 5 20 $P/rexx-head-cd070fd13 $n.rex </dev/null >$out.out 2>$out.err; echo $? >$out.rc;;
    h) memcap 2G timeout -k 5 20 $P/rexx-head-d8614a7c9 $n.rex </dev/null >$out.out 2>$out.err; echo $? >$out.rc;;
  esac
  popd >/dev/null
  rm -r /tmp/claude-1000/t87c-fix4-run
done
for w in b h; do s=""; for x in out err rc; do cmp -s $d/out/$n.o.$x $d/out/$n.$w.$x || s="$s $x"; done; echo "$n o~$w differ:${s:- none}"; done
