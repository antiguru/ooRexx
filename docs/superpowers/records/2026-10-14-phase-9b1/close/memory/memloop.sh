#!/bin/sh
# usage: memloop.sh BIN PROGRAM N...   prints program n peak_kB (last stderr line) for crate BIN or "oracle"
bin=$1; prog=$2; shift 2
for n in "$@"; do
  if [ "$bin" = oracle ]; then
    r=$( ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 120 /usr/bin/time -f %M /home/moritz/dev/repos/ooRexx/build/bin/rexx "$prog" "$n" 2>&1 >/dev/null ) )
  else
    r=$( memcap 2G timeout -k 5 120 /usr/bin/time -f %M "$bin" "$prog" "$n" 2>&1 >/dev/null )
  fi
  echo "$prog $n $(echo "$r" | tr '\n' ' ')"
done
