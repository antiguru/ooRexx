#!/bin/bash
# rungroup.sh SIDE COPYDIR OUT : runs the RexxInteger group of COPYDIR/run on
# SIDE (oracle|crate), writing OUT.stdout, OUT.stderr and OUT.status.
set -u
side=$1
copy=$2
out=$3
run=$copy/run
group=$run/ooRexx/base/class/RexxInteger.testGroup
ORACLE=/home/moritz/dev/repos/ooRexx/build
TARGET=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/target-9d-t6
cd "$run" || exit 99
if [ "$side" = oracle ]; then
  ( ulimit -v 1048576
    LD_LIBRARY_PATH=$ORACLE/lib PATH=$ORACLE/bin:$PATH \
      timeout -k 5 300 $ORACLE/bin/rexx "$run/testOORexx.rex" -f "$group" -U -V 2 \
      >"$out.stdout" 2>"$out.stderr"
    echo $? >"$out.status" )
else
  mkdir -p "$copy/bin-rexx"
  rm -f "$copy/bin-rexx/rexx"
  ln "$TARGET/release/rexx" "$copy/bin-rexx/rexx"
  [ -e "$copy/samples" ] || ln -s $ORACLE/samples "$copy/samples"
  LD_LIBRARY_PATH=$ORACLE/lib PATH=$copy/bin-rexx:$PATH \
    memcap 2G timeout -k 5 300 "$copy/bin-rexx/rexx" "$run/testOORexx.rex" -f "$group" -U -V 2 \
    >"$out.stdout" 2>"$out.stderr"
  echo $? >"$out.status"
fi
