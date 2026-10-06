#!/bin/bash
# usage: run.sh oracle|ours probe N
side=$1; p=$2; n=$3
P=/tmp/claude-1000/p6-s5-verify/probes/$p.rex
L=/tmp/claude-1000/p6-s5-verify/logs/$side-$p.log
mkdir -p /tmp/claude-1000/p6-s5-verify/logs
: > $L
cd /tmp/claude-1000/p6-s5-verify/empty || exit 1
for i in $(seq 1 $n); do
  s=$(date +%s.%N)
  if [ $side = oracle ]; then
    out=$( ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $P ) 2>/tmp/claude-1000/p6-s5-verify/err.txt ); rc=$?
  elif [ $side = every ]; then
    out=$( ( ulimit -v 4194304; REXX_SWITCH_MODE=every timeout -k 5 20 /tmp/claude-1000/p6-s5-verify/target/release/rexx-run $P ) 2>/tmp/claude-1000/p6-s5-verify/err.txt ); rc=$?
  else
    out=$( ( ulimit -v 4194304; timeout -k 5 20 /tmp/claude-1000/p6-s5-verify/target/release/rexx-run $P ) 2>/tmp/claude-1000/p6-s5-verify/err.txt ); rc=$?
  fi
  e=$(date +%s.%N)
  err=$(tr '\n' '~' < /tmp/claude-1000/p6-s5-verify/err.txt)
  printf '%s|rc=%s|stderr=%s\n' "$(printf '%s' "$out" | tr '\n' '|')" "$rc" "$err" >> $L
  printf '%s %.2f\n' "$i" "$(echo "$e - $s" | bc)" >> $L.times
done
echo "== $side $p ($n runs)"
sort $L | uniq -c
awk '{if($2>m)m=$2} END{print "max seconds", m}' $L.times
