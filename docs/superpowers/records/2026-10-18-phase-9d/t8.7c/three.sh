#!/bin/bash
# usage: three.sh DIR FILE [args] -- o oracle (skipped with NO_ORACLE set), b 97eee8a9a, h e620097f0 (or $HEAD); outputs in DIR/out/
B=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/rexx-base-97eee8a9a
H=${HEAD:-/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/rexx-head-e620097f0}
dir=$1; f=$2; shift 2
mkdir -p "$dir/out"
cd "$dir" || exit 1
if [ -z "$NO_ORACLE" ]; then
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" "$@" > "$dir/out/$f.o.out" 2> "$dir/out/$f.o.err" < ${STDIN:-/dev/null}; echo $? > "$dir/out/$f.o.rc" )
fi
for side in b h; do
  case $side in b) bin=$B;; h) bin=$H;; esac
  [ -x "$bin" ] || continue
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 5 20 $bin "$f" "$@" > "$dir/out/$f.$side.out" 2> "$dir/out/$f.$side.err" < ${STDIN:-/dev/null}; echo $? > "$dir/out/$f.$side.rc"
done
for k in out err rc; do
  a=$(cmp -s $dir/out/$f.o.$k $dir/out/$f.b.$k && echo = || echo X); b=$(cmp -s $dir/out/$f.o.$k $dir/out/$f.h.$k && echo = || echo X)
  printf '%s %s: o~b %s o~h %s\n' $f $k $a $b
done
