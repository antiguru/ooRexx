#!/bin/bash
# usage: t67-run3.sh OUTDIR NAME... ; runs each corpus/lang/NAME.rex on oracle, parent and task
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad
L=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/lang
out=$1; shift
mkdir -p "$out"
for n in "$@"; do
  for side in oracle parent task; do
    d=$(mktemp -d "$S/run.XXXX")
    cp "$L/$n.rex" "$d/"
    case $side in
      oracle) ( cd "$d" && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$n.rex" ) > "$out/$n.$side.out" 2> "$out/$n.$side.err" ;;
      parent) ( cd "$d" && memcap 2G timeout 20 /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.6-r2/rexx-head "$n.rex" ) > "$out/$n.$side.out" 2> "$out/$n.$side.err" ;;
      task) ( cd "$d" && memcap 2G timeout 20 "$S/t67-target/release/rexx" "$n.rex" ) > "$out/$n.$side.out" 2> "$out/$n.$side.err" ;;
    esac
    echo $? > "$out/$n.$side.rc"
    rm -rf "$d"
  done
  sed -i "s|$S/run\.[A-Za-z0-9]*/|DIR/|g" "$out/$n".*.err
  for side in parent task; do
    o=same; cmp -s "$out/$n.oracle.out" "$out/$n.$side.out" || o=differs
    e=same; cmp -s "$out/$n.oracle.err" "$out/$n.$side.err" || e=differs
    printf '%s\t%s\trc %s/%s\tstdout %s\tstderr %s\n' "$n" "$side" "$(cat $out/$n.oracle.rc)" "$(cat $out/$n.$side.rc)" "$o" "$e"
  done
done
