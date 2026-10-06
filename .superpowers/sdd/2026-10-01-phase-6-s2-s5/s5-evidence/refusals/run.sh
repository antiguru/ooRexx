#!/bin/bash
# usage: run.sh probesdir outdir
P=$1; O=$2; S=/tmp/claude-1000/p6-s5-refusals
OURS=$S/target/release/rexx-run
mkdir -p $O
for f in $P/*.rex; do
  b=$(basename $f .rex)
  d=$(mktemp -d $S/run.XXXXXX)
  cp $f $d/p.rex
  cd $d
  ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx p.rex ) > $O/$b.oracle 2>&1; echo "rc $?" >> $O/$b.oracle
  timeout -k 5 20 $OURS p.rex > $O/$b.ours 2>&1; echo "rc $?" >> $O/$b.ours
  cd $S
  rm -f $d/p.rex; rmdir $d
  if cmp -s $O/$b.oracle $O/$b.ours; then r=SAME; else r=DIFF; fi
  if grep -q 'not implemented' $O/$b.ours; then r="$r REFUSED"; fi
  echo "$b $r"
done
