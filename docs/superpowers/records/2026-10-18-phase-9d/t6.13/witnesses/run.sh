#!/bin/bash
# Runs each Task 6.13 witness on the oracle, the parent and the task's binary,
# each from a fresh directory, and writes the three descriptors and summary.tsv.
# usage: run.sh PARENT_BIN HEAD_BIN SCRATCH_DIR
parent=$1; head=$2; scratch=$3
here=$(cd "$(dirname "$0")" && pwd)
corpus=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/lang
printf 'program\toracle\tparent\thead\n' > "$here/summary.tsv"
for name in context_rs_command command_redirected_off_baton; do
  for side in oracle parent head; do
    dir=$(mktemp -d "$scratch/w613.XXXX")
    cp "$corpus/$name.rex" "$dir/"
    case $side in
      oracle) ( cd "$dir" && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$name.rex" > "$here/$name.$side.out" 2> "$here/$name.$side.err" ); echo $? > "$here/$name.$side.rc" ;;
      parent) ( cd "$dir" && memcap 2G timeout 20 "$parent" "$name.rex" > "$here/$name.$side.out" 2> "$here/$name.$side.err" ); echo $? > "$here/$name.$side.rc" ;;
      head) ( cd "$dir" && memcap 2G timeout 20 "$head" "$name.rex" > "$here/$name.$side.out" 2> "$here/$name.$side.err" ); echo $? > "$here/$name.$side.rc" ;;
    esac
  done
  verdict() {
    if cmp -s "$here/$name.oracle.out" "$here/$name.$1.out" && cmp -s "$here/$name.oracle.err" "$here/$name.$1.err" && cmp -s "$here/$name.oracle.rc" "$here/$name.$1.rc"; then echo identical; else echo "differs (rc $(cat "$here/$name.$1.rc"))"; fi
  }
  printf '%s\trc %s\t%s\t%s\n' "$name" "$(cat "$here/$name.oracle.rc")" "$(verdict parent)" "$(verdict head)" >> "$here/summary.tsv"
done
