#!/bin/bash
# usage: run.sh oracle|crate PROBEDIR OUTDIR [BIN]
# Runs every probe from its own fresh empty directory under OUTDIR/run and
# writes NAME.out, NAME.err and NAME.rc to OUTDIR.
side=$1 probes=$2 out=$3 bin=${4:-}
mkdir -p "$out/run"
for f in "$probes"/*.rex; do
  n=$(basename "$f" .rex)
  d=$(mktemp -d "$out/run/p.XXXXXX")
  cd "$d" || exit 9
  if [ "$side" = oracle ]; then
    ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" ) > "$out/$n.out" 2> "$out/$n.err"
  else
    ( timeout -k 5 20 "$bin" "$f" ) > "$out/$n.out" 2> "$out/$n.err"
  fi
  echo $? > "$out/$n.rc"
  cd "$out" || exit 9
  rmdir "$d"
done
