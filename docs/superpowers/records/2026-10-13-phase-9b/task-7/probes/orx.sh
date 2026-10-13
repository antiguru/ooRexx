#!/bin/bash
# usage: orx.sh DIR FILE -- runs the oracle on DIR/FILE from DIR, three descriptors separately
cd "$1" || exit 99
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 4G timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$2" ) >"$1/o.out" 2>"$1/o.err" </dev/null
echo "rc=$?"
echo "--- stdout"; cat -v "$1/o.out"; echo "--- stderr"; cat -v "$1/o.err"
