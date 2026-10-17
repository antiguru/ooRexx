#!/usr/bin/env bash
# Usage: run.sh BIN OUTDIR. Runs each program from a fresh directory on the
# oracle and on BIN, and compares stdout, stderr and rc separately.
set -u
bin=$1; out=$2
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
oracle=/home/moritz/dev/repos/ooRexx/build
for p in "$here"/progs3/*.rex; do
    n=$(basename "$p" .rex)
    for side in oracle crate; do
        # Both sides run from the same path, so the messages naming it agree.
        d=$out/$n
        mkdir -p "$d" && cp "$p" "$d/$n.rex" || exit 1
        if [ $side = oracle ]; then
            ( cd "$d" && ulimit -v 1048576 && LD_LIBRARY_PATH=$oracle/lib timeout -k 5 20 "$oracle/bin/rexx" "$n.rex" >out 2>err; echo $? >rc )
        else
            ( cd "$d" && timeout -k 5 20 memcap 2G "$bin" "$n.rex" >out 2>err; echo $? >rc )
        fi
        mv "$d" "$out/$n-$side" || exit 1
    done
    o=$out/$n-oracle; c=$out/$n-crate
    s=same; for f in out err rc; do cmp -s "$o/$f" "$c/$f" || s="DIFF($f)"; done
    echo "$n rc oracle $(cat "$o/rc") crate $(cat "$c/rc") $s"
done
