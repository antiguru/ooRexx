#!/bin/bash
# usage: rss.sh file n  -> maxrss KB and wall per side (oracle, head, base 3b24a6445)
f=$1; n=$2
OL=/home/moritz/dev/repos/ooRexx/build/lib
o=$( ( ulimit -v 1048576; LD_LIBRARY_PATH=$OL timeout -k 5 300 /usr/bin/time -f 'rss=%MKB t=%es' /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" $n ) 2>&1 | tr '\n' ' ')
h=$(LD_LIBRARY_PATH=$OL memcap 2G timeout 300 /usr/bin/time -f 'rss=%MKB t=%es' /tmp/claude-1000/t7-target/release/rexx "$f" $n 2>&1 | tr '\n' ' ')
b=$(LD_LIBRARY_PATH=$OL memcap 2G timeout 300 /usr/bin/time -f 'rss=%MKB t=%es' /tmp/claude-1000/t7-base-target/release/rexx "$f" $n 2>&1 | tr '\n' ' ')
echo "$(basename $f) n=$n | oracle: $o | head: $h | base: $b"
