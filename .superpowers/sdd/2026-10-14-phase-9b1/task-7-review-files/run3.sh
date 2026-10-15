#!/bin/bash
# usage: run3.sh PROBE.rex -> runs oracle, head (plain), head (stress) each in a fresh dir
p=$1; n=$(basename $p .rex)
for mode in oracle plain stress; do
  d=$(mktemp -d /tmp/t7rev/run.XXXXXX); cp $p $d/; cd $d
  case $mode in
    oracle) ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $n.rex ) >out 2>err; rc=$? ;;
    plain) timeout -k 5 60 memcap 4G /tmp/t7rev/rexx-run-head $n.rex >out 2>err; rc=$? ;;
    stress) timeout -k 5 300 memcap 4G /tmp/t7rev/stress $n.rex >out 2>err; rc=$? ;;
  esac
  echo "=== $n $mode rc=$rc"; cat out; echo "--- stderr"; head -c 600 err
  cd /tmp
done
