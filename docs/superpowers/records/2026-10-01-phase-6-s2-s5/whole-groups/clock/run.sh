#!/bin/bash
# The elapsed-clock probes, each N times on the oracle and on REXX_RUN, from an
# empty directory. Usage, from the repository root: bash run.sh REXX_RUN SCRATCH N
R=$1; S=$2; N=$3
P=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$S/empty"; cd "$S/empty" || exit 1
for p in a b d e; do
  for side in oracle ours; do
    for i in $(seq 1 "$N"); do
      if [ "$side" = oracle ]; then
        ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$P/$p.rex" 2>&1; echo "rc=$?" )
      else
        ( ulimit -v 4194304; timeout -k 5 20 "$R" "$P/$p.rex" 2>&1; echo "rc=$?" )
      fi | tr '\n' '|'; echo
    done > "$S/$side-$p.log"
    echo "== $side $p ($N runs)"; sort "$S/$side-$p.log" | uniq -c
  done
done
