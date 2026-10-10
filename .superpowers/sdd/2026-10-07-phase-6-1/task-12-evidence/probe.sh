#!/bin/bash
# probe.sh FILE [RUNS]: run FILE on the oracle and on HEAD's rexx-run, each from a fresh
# empty directory, RUNS times each (default 1). Prints stdout, stderr and rc per run.
f=$(readlink -f "$1"); n=${2:-1}; [ -n "$STDIN" ] && STDIN=$(readlink -f "$STDIN")
ours=${OURS:-/tmp/claude-1000/p61/t12/bin/head/rexx-run}
lib=/home/moritz/dev/repos/ooRexx/build/lib
for ((i = 1; i <= n; i++)); do
  d=$(mktemp -d /tmp/claude-1000/p61/t12/runs/o.XXXXXX)
  ( cd "$d"; ( ulimit -v 1048576; LD_LIBRARY_PATH=$lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" > out 2> err < "${STDIN:-/dev/null}"; echo $? > rc ) )
  echo "--- oracle run $i rc $(cat $d/rc)"; echo "[stdout]"; cat "$d/out"; echo "[stderr]"; cat "$d/err"
  d=$(mktemp -d /tmp/claude-1000/p61/t12/runs/r.XXXXXX)
  ( cd "$d"; LD_LIBRARY_PATH=$lib memcap 2G timeout -k 5 20 "$ours" "$f" > out 2> err < "${STDIN:-/dev/null}"; echo $? > rc )
  echo "--- ours run $i rc $(cat $d/rc)"; echo "[stdout]"; cat "$d/out"; echo "[stderr]"; cat "$d/err"
done
