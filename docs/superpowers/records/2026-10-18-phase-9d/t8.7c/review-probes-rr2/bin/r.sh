#!/bin/bash
# usage: r.sh FILE.rex [which...]  (o oracle, b base d6720f350, h head 3344571cc)
f=$(readlink -f "$1"); shift
d=$(dirname "$f"); n=$(basename "$f" .rex)
which="${*:-o b h}"
for w in $which; do
  run=/tmp/claude-1000/t87c-rr2-run; rm -rf /tmp/claude-1000/t87c-rr2-run; mkdir $run
  out=$d/out/$n.$w; mkdir -p $d/out
  cp "$f" $run/
  [ -e "$d/$n.stdin" ] && in="$d/$n.stdin" || in=/dev/null
  pushd $run >/dev/null
  case $w in
    o) ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $n.rex ) <$in >$out.out 2>$out.err; echo $? >$out.rc;;
    b) memcap 2G timeout -k 5 20 /tmp/claude-1000/t87c-rr2-target-d6720f350/release/rexx $n.rex <$in >$out.out 2>$out.err; echo $? >$out.rc;;
    h) memcap 2G timeout -k 5 20 /tmp/claude-1000/t87c-rr2-target-3344571cc/release/rexx $n.rex <$in >$out.out 2>$out.err; echo $? >$out.rc;;
    n) memcap 2G timeout -k 5 20 /tmp/claude-1000/t87c-rr2-target-mut2/release/rexx $n.rex <$in >$out.out 2>$out.err; echo $? >$out.rc;;
    m) memcap 2G timeout -k 5 20 /tmp/claude-1000/t87c-rr2-target-mut/release/rexx $n.rex <$in >$out.out 2>$out.err; echo $? >$out.rc;;
  esac
  popd >/dev/null
  rm -r "$run"
done
for w in $which; do echo "== $n.$w rc $(cat $d/out/$n.$w.rc)"; cat $d/out/$n.$w.out; echo "-- err"; cat $d/out/$n.$w.err; done
