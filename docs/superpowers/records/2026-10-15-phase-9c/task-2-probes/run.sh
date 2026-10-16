#!/bin/bash
# usage: run.sh PROBEDIR OUTDIR CRATEBIN
P=$1; O=$2; BIN=$3
mkdir -p "$O"
for f in "$P"/*.rex; do
  n=$(basename "$f" .rex)
  ( cd "$P" && ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" >"$O/$n.o.out" 2>"$O/$n.o.err"; echo $? >"$O/$n.o.rc" )
  ( cd "$P" && memcap 2G timeout -k 5 20 "$BIN" "$f" >"$O/$n.c.out" 2>"$O/$n.c.err"; echo $? >"$O/$n.c.rc" )
  same=yes
  for k in out err rc; do cmp -s "$O/$n.o.$k" "$O/$n.c.$k" || same="no($k)"; done
  echo "$n $same"
done
