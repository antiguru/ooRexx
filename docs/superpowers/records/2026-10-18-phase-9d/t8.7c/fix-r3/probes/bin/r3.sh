#!/bin/bash
# usage: r3.sh DIR FILE...   runs o (oracle), b (rexx-head-8dd9a4ae0), h (rexx-head-cd070fd13) from DIR, outputs in DIR/out
d=$1; shift
mkdir -p "$d/out"
for f in "$@"; do
  n=$(basename "$f")
  ( cd "$d" && ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" ) > "$d/out/$n.o.out" 2> "$d/out/$n.o.err" < /dev/null; echo $? > "$d/out/$n.o.rc" )
  ( cd "$d" && memcap 2G timeout -k 5 20 /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/rexx-head-8dd9a4ae0 "$f" > "$d/out/$n.b.out" 2> "$d/out/$n.b.err" < /dev/null; echo $? > "$d/out/$n.b.rc" )
  ( cd "$d" && memcap 2G timeout -k 5 20 /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/rexx-head-cd070fd13 "$f" > "$d/out/$n.h.out" 2> "$d/out/$n.h.err" < /dev/null; echo $? > "$d/out/$n.h.rc" )
  for s in b h; do
    r=""
    for x in out err rc; do cmp -s "$d/out/$n.o.$x" "$d/out/$n.$s.$x" || r="$r $x"; done
    echo "$n o~$s: ${r:- equal}"
  done
done
