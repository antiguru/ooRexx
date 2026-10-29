#!/bin/bash
# usage: four.sh DIR FILE [args]  -- runs o/p/c/h in DIR, writes DIR/out/FILE.{o,p,c,h}.{out,err,rc}
# o oracle (skipped with NO_ORACLE set), p d1f8b387f, c cca0956e0 (rexx-head-10b0f5081), h this round (HEAD)
P=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b
dir=$1; f=$2; shift 2
mkdir -p "$dir/out"
cd "$dir" || exit 1
if [ -z "$NO_ORACLE" ]; then
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" "$@" > "$dir/out/$f.o.out" 2> "$dir/out/$f.o.err"; echo $? > "$dir/out/$f.o.rc" )
fi
for side in p c h; do
  case $side in p) bin=$P/rexx-head-d1f8b387f;; c) bin=$P/rexx-head-10b0f5081;; h) bin=$HEAD;; esac
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 5 20 $bin "$f" "$@" > "$dir/out/$f.$side.out" 2> "$dir/out/$f.$side.err" < /dev/null; echo $? > "$dir/out/$f.$side.rc"
done
for side in o p c h; do [ -f "$dir/out/$f.$side.out" ] || continue; echo "--- $side rc=$(cat $dir/out/$f.$side.rc) lines=$(wc -l < $dir/out/$f.$side.out)"; head -12 "$dir/out/$f.$side.out"; head -5 "$dir/out/$f.$side.err" | sed 's/^/ERR: /'; done
