#!/bin/bash
# usage: mem.sh BIN PROG... ; each program in a fresh dir with ext.rex
bin=$1; shift
for p in "$@"; do
  n=$(basename $p .rex); d=$(mktemp -d /tmp/t7rev/run.XXXXXX); cp /tmp/t7rev/mem/$n.rex $d/; cp /tmp/t7rev/mem/ext.rex.src $d/ext.rex; cd $d
  if [ "$bin" = oracle ]; then
    r=$( ( ulimit -v 1048576 -t 120; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /usr/bin/time -f '%M' /home/moritz/dev/repos/ooRexx/build/bin/rexx $n.rex ) 2>&1 | tr '\n' ' ')
  else
    r=$( ( ulimit -v 4194304 -t 120; /usr/bin/time -f '%M' $bin $n.rex ) 2>&1 | tr '\n' ' ')
  fi
  echo "$(basename $bin) $n $r"; cd /tmp
done
