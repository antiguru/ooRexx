#!/bin/bash
# Criterion 10's UNINIT-ordering probes, each run N times on the oracle, on
# REXX_RUN and on REXX_RUN under every opportunity, from an empty directory.
# Usage, from the repository root: bash run.sh REXX_RUN SCRATCH N > summary.txt
# Each run's line: stdout lines joined by |, the exit status, stderr.
R=$1; S=$2; N=$3
P=$(cd "$(dirname "$0")/probes" && pwd)
mkdir -p "$S/empty"
cd "$S/empty" || exit 1
for p in u1 u2 u3 u4 u5 t3 t7 corpus_uaea; do
  for side in oracle ours every; do
    : > "$S/$side-$p.log"
    for i in $(seq 1 "$N"); do
      if [ "$side" = oracle ]; then
        out=$( ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$P/$p.rex" ) 2>"$S/err.txt" ); rc=$?
      elif [ "$side" = every ]; then
        out=$( ( ulimit -v 4194304; REXX_SWITCH_MODE=every timeout -k 5 20 "$R" "$P/$p.rex" ) 2>"$S/err.txt" ); rc=$?
      else
        out=$( ( ulimit -v 4194304; timeout -k 5 20 "$R" "$P/$p.rex" ) 2>"$S/err.txt" ); rc=$?
      fi
      printf '%s|rc=%s|stderr=%s\n' "$(printf '%s' "$out" | tr '\n' '|')" "$rc" "$(tr '\n' '~' < "$S/err.txt")" >> "$S/$side-$p.log"
    done
    echo "== $side $p ($N runs)"
    sort "$S/$side-$p.log" | uniq -c
  done
done
