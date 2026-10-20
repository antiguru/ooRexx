#!/bin/bash
# usage: sigint.sh SIDE PROGRAM DELAY -- runs PROGRAM (oracle or crate BIN), sends SIGINT
# to the interpreter process after DELAY seconds; prints stdout, stderr, rc.
side=$1; prog=$2; delay=$3; bin=$4
out=$(mktemp); err=$(mktemp)
if [ "$side" = oracle ]; then
  ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib exec timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$prog" >"$out" 2>"$err" ) &
  w=$!; target=
else
  ( exec memcap 2G timeout 20 "$bin" "$prog" >"$out" 2>"$err" ) &
  w=$!
  target=
fi
sleep "$delay"
if [ "$side" = oracle ]; then
  target=$(pgrep -P $w)
else
  t=$(pgrep -P $w timeout); target=$(pgrep -P "$t")
fi
kill -INT "$target"
wait $w; rc=$?
echo "--- stdout"; cat "$out"; echo "--- stderr"; cat "$err"; echo "--- rc $rc"
rm -f "$out" "$err"
