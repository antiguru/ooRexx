#!/bin/sh
# Runs each seed probe named on the command line on this crate and on the oracle,
# from the fresh empty directory $1, writing <probe>.crate.{out,err,rc} and
# <probe>.oracle.{out,err,rc} beside the probe. Usage: run.sh RUN_DIR REXX PROBE...
run_dir=$1; rexx=$2; shift 2
cd "$run_dir" || exit 1
for probe in "$@"; do
  base=${probe%.rex}
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 2 10 "$rexx" "$probe" >"$base.crate.out" 2>"$base.crate.err"
  echo $? >"$base.crate.rc"
  ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$probe" >"$base.oracle.out" 2>"$base.oracle.err" )
  echo $? >"$base.oracle.rc"
done
