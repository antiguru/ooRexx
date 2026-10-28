#!/usr/bin/env bash
# three.sh FILE : oracle (o), 5053fbd9b (b), new (h) outputs beside FILE
f=$1; d=$(dirname $f); n=$(basename $f)
cd $d || exit 1
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $n >$d/$n.o.out 2>$d/$n.o.err; echo $? >$d/$n.o.rc )
LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 5 20 /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b/rexx-head-5053fbd9b $n >$d/$n.b.out 2>$d/$n.b.err; echo $? >$d/$n.b.rc
LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 5 20 /tmp/claude-1000/t87b-target/release/rexx $n >$d/$n.h.out 2>$d/$n.h.err; echo $? >$d/$n.h.rc
for k in out err rc; do
  a=$(cmp -s $d/$n.o.$k $d/$n.b.$k && echo = || echo X); b=$(cmp -s $d/$n.o.$k $d/$n.h.$k && echo = || echo X)
  printf '%s %s: o~b %s o~h %s\n' $n $k $a $b
done
