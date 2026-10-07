#!/bin/bash
S=/tmp/claude-1000/p6-t26
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
echo "start $(date -Is) $(cat /proc/loadavg)" > $S/wall-load.txt
bash rust/bench-programs/wallclock.sh -r 5 -x extcall -o $S/wall base=$S/bin/base/rexx-run s1=$S/bin/s1/rexx-run r1=$S/bin/r1/rexx-run r3=$S/bin/r3/rexx-run base2=$S/bin/base2/rexx-run > $S/wall-out.txt 2> $S/wall-err.txt
echo "exit $?" >> $S/wall-load.txt
echo "end $(date -Is) $(cat /proc/loadavg)" >> $S/wall-load.txt
