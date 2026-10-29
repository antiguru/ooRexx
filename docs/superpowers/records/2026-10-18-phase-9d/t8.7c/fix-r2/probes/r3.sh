#!/bin/bash
# usage: r3.sh DIR FILE -- o oracle (skip with NO_ORACLE), b round 1 head (246ab583d), h this round (H=binary); prints each side's out/err/rc
B=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/rexx-head-246ab583d
H=${H:-/tmp/claude-1000/t87c-fix2-target/release/rexx}
dir=$1; f=$2
mkdir -p "$dir/out"; cd "$dir" || exit 1
sides="b h"; [ -z "$NO_ORACLE" ] && sides="o b h"
[ -n "$NO_HEAD" ] && sides="${sides% h}"
for s in $sides; do
  case $s in
  o) ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" > "$dir/out/$f.o.out" 2> "$dir/out/$f.o.err" < ${STDIN:-/dev/null}; echo $? > "$dir/out/$f.o.rc" ) ;;
  b|h) bin=$B; [ $s = h ] && bin=$H
     LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 5 20 $bin "$f" > "$dir/out/$f.$s.out" 2> "$dir/out/$f.$s.err" < ${STDIN:-/dev/null}; echo $? > "$dir/out/$f.$s.rc" ;;
  esac
done
for s in $sides; do echo "--- $s rc $(cat $dir/out/$f.$s.rc)"; cat $dir/out/$f.$s.out; [ -s $dir/out/$f.$s.err ] && { echo "  [stderr]"; cat $dir/out/$f.$s.err; }; done
