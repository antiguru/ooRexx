#!/bin/bash
# Runs each probe here on the oracle, the parent and the task's binary from a
# fresh directory, the crate with the oracle's library directory as the suite
# harness gives it; writes NAME.SIDE.{out,err,rc} beside the probe.
# usage: run.sh PARENT_BIN HEAD_BIN SCRATCH_DIR
parent=$1; head=$2; scratch=$3
here=$(cd "$(dirname "$0")" && pwd)
lib=/home/moritz/dev/repos/ooRexx/build/lib
for probe in "$here"/interpret/*.rex "$here"/hostemu/*.rex; do
  base=${probe%.rex}; file=$(basename "$probe")
  for side in oracle parent head; do
    dir=$(mktemp -d "$scratch/p613.XXXX"); cp "$probe" "$dir/"
    case $side in
      oracle) ( cd "$dir" && ulimit -v 1048576 && LD_LIBRARY_PATH=$lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
      parent) ( cd "$dir" && LD_LIBRARY_PATH=$lib memcap 2G timeout 20 "$parent" "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
      head) ( cd "$dir" && LD_LIBRARY_PATH=$lib memcap 2G timeout 20 "$head" "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
    esac
  done
done
