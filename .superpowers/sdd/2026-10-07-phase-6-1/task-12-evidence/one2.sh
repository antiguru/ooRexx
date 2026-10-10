#!/bin/bash
# one.sh LABEL BINARY TEST: run one GUARD test on BINARY (or "oracle") from a fresh template copy.
L=$1; B=$2; T=$3; G=/tmp/claude-1000/p61/t12/gb
d=$(mktemp -d $G/runs.$L.$T.XXXXXX); cp -r $G/tpl2/. $d/
lib=/home/moritz/dev/repos/ooRexx/build/lib
if [ "$B" = oracle ]; then
  ( cd $d; ulimit -v 1048576; LD_LIBRARY_PATH=$lib timeout -k 5 120 /home/moritz/dev/repos/ooRexx/build/bin/rexx testOORexx.rex -f ooRexx/base/rexxutil/SysSleep.testGroup -U -V 2 -t $T > out 2> err; echo $? > rc )
else
  mkdir $d/bin; ln -s $B $d/bin/rexx
  ( cd $d; LD_LIBRARY_PATH=$lib PATH=$d/bin:$PATH memcap 2G timeout -k 5 120 $B testOORexx.rex -f ooRexx/base/rexxutil/SysSleep.testGroup -U -V 2 -t $T > out 2> err; echo $? > rc )
fi
a=$(grep -a 'Assertions:' $d/out | head -1 | tr -s ' '); f=$(grep -a 'Failures:' $d/out | head -1 | tr -s ' '); e=$(grep -a 'Errors:' $d/out | head -1 | tr -s ' ')
echo "$L $T rc=$(cat $d/rc) $a $f $e | $(grep -a 'rexx-exec:' $d/err | head -1 | cut -c1-120)"
