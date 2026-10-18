#!/usr/bin/env bash
# probe-r1.sh OUTDIR: as probe.sh, with t6.6 meaning the fix round 2 build
# and parent the build
# (ebeab2d35) and this task's build, each from OUTDIR itself, and writes
# OUTDIR/<name>.<side>.{out,err,rc} and OUTDIR/summary.tsv.
set -u
out=$1
oracle=/home/moritz/dev/repos/ooRexx/build
parent=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.6/rexx-base-ebeab2d35
task=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.6-r2/rexx-head
cd "$out" || exit 1
printf 'program\toracle rc\tparent rc\tparent stdout\tparent stderr\tt6.6 rc\tt6.6 stdout\tt6.6 stderr\n' > summary.tsv
for f in *.rex; do
  n=${f%.rex}
  case $n in *_callee) continue ;; esac
  ( ulimit -v 1048576; LD_LIBRARY_PATH=$oracle/lib timeout -k 5 20 $oracle/bin/rexx "$f" > $n.oracle.out 2> $n.oracle.err; echo $? > $n.oracle.rc )
  for side in parent task; do
    bin=${!side}
    timeout 20 memcap 2G "$bin" "$f" > $n.$side.out 2> $n.$side.err; echo $? > $n.$side.rc
  done
  row="$n\t$(cat $n.oracle.rc)"
  for side in parent task; do
    cmp -s $n.oracle.out $n.$side.out && o=same || o=differs
    cmp -s $n.oracle.err $n.$side.err && e=same || e=differs
    row="$row\t$(cat $n.$side.rc)\t$o\t$e"
  done
  printf "$row\n" >> summary.tsv
done
