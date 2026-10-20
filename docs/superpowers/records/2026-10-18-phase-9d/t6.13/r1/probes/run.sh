#!/bin/bash
# Runs each probe under targets/ and libraries/ on the oracle, the parent and
# the round's binary from a fresh directory, with the oracle's library
# directory on LD_LIBRARY_PATH, writing NAME.SIDE.{out,err,rc}; and each halt/
# probe through halt/sigint.sh (SIGINT by PID after 0.6 s) into NAME.SIDE.txt.
# usage: run.sh PARENT_BIN HEAD_BIN SCRATCH_DIR
parent=$1; head=$2; scratch=$3
here=$(cd "$(dirname "$0")" && pwd)
lib=/home/moritz/dev/repos/ooRexx/build/lib
for probe in "$here"/targets/*.rex "$here"/libraries/*.rex; do
  base=${probe%.rex}; file=$(basename "$probe")
  for side in oracle parent head; do
    dir=$(mktemp -d "$scratch/p613r1.XXXX"); cp "$probe" "$dir/"
    case $side in
      oracle) ( cd "$dir" && ulimit -v 1048576 && LD_LIBRARY_PATH=$lib timeout -k 5 30 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
      parent) ( cd "$dir" && LD_LIBRARY_PATH=$lib memcap 2G timeout 30 "$parent" "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
      head) ( cd "$dir" && LD_LIBRARY_PATH=$lib memcap 2G timeout 30 "$head" "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
    esac
    sed -i "s|$dir|DIR|g" "$base.$side.err"
  done
done
for probe in "$here"/halt/*.rex; do
  base=${probe%.rex}; file=$(basename "$probe")
  for side in oracle parent head; do
    dir=$(mktemp -d "$scratch/p613r1.XXXX"); cp "$probe" "$dir/"
    case $side in
      oracle) ( cd "$dir" && "$here/halt/sigint.sh" oracle "$file" 0.6 ) > "$base.$side.txt" ;;
      parent) ( cd "$dir" && "$here/halt/sigint.sh" crate "$file" 0.6 "$parent" ) > "$base.$side.txt" ;;
      head) ( cd "$dir" && "$here/halt/sigint.sh" crate "$file" 0.6 "$head" ) > "$base.$side.txt" ;;
    esac
    sed -i "s|$dir|DIR|g" "$base.$side.txt"
    sleep 3.5
  done
done
