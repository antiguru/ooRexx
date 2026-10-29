#!/bin/bash
# usage: four.sh DIR FILE [args]  -- runs o/b/c/h in DIR, writes DIR/out/FILE.{o,b,c,h}.{out,err,rc}
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/r2
dir=$1; f=$2; shift 2
mkdir -p "$dir/out"
cd "$dir" || exit 1
B=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b/rexx-head-5053fbd9b
if [ -z "$NO_ORACLE" ]; then
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" "$@" > "$dir/out/$f.o.out" 2> "$dir/out/$f.o.err"; echo $? > "$dir/out/$f.o.rc" )
fi
for side in b c h; do
  case $side in b) bin=$B;; c) bin=$S/rexx-cca;; h) bin=${HEAD:-$S/rexx-h1};; esac
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 5 20 $bin "$f" "$@" > "$dir/out/$f.$side.out" 2> "$dir/out/$f.$side.err"; echo $? > "$dir/out/$f.$side.rc"
done
for side in o b c h; do [ -f "$dir/out/$f.$side.out" ] || continue; echo "--- $side rc=$(cat $dir/out/$f.$side.rc)"; cat "$dir/out/$f.$side.out"; sed 's/^/ERR: /' "$dir/out/$f.$side.err"; done
