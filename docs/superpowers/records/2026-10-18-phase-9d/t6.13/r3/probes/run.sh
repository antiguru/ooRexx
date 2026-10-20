#!/bin/bash
# Runs parks/*.rex "$here"/delayed/*.rex on the oracle, the round-2 binary and the round's binary
# from a fresh directory, writing NAME.SIDE.{out,err,rc}, and each halt/*.rex
# through halt/sigint.sh (SIGINT by PID after 0.6 s) into NAME.SIDE.txt.
# usage: run.sh ROUND2_BIN HEAD_BIN SCRATCH_DIR
parent=$1; head=$2; scratch=$3
here=$(cd "$(dirname "$0")" && pwd)
for probe in "$here"/parks/*.rex "$here"/delayed/*.rex; do
  base=${probe%.rex}; file=$(basename "$probe")
  for side in oracle parent head; do
    dir=$(mktemp -d "$scratch/p613r2.XXXX"); cp "$probe" "$dir/"
    case $side in
      oracle) ( cd "$dir" && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
      parent) ( cd "$dir" && memcap 2G timeout 20 "$parent" "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
      head) ( cd "$dir" && memcap 2G timeout 20 "$head" "$file" > "$base.$side.out" 2> "$base.$side.err" ); echo $? > "$base.$side.rc" ;;
    esac
    sed -i "s|$dir|DIR|g" "$base.$side.err"
  done
done
for probe in "$here"/halt/*.rex; do
  base=${probe%.rex}; file=$(basename "$probe")
  for side in oracle parent head; do
    dir=$(mktemp -d "$scratch/p613r2.XXXX"); cp "$probe" "$dir/"
    case $side in
      oracle) ( cd "$dir" && "$here/halt/sigint.sh" oracle "$file" 0.6 ) > "$base.$side.txt" ;;
      parent) ( cd "$dir" && "$here/halt/sigint.sh" crate "$file" 0.6 "$parent" ) > "$base.$side.txt" ;;
      head) ( cd "$dir" && "$here/halt/sigint.sh" crate "$file" 0.6 "$head" ) > "$base.$side.txt" ;;
    esac
    sed -i -e "s|$dir|DIR|g" -e '/^memcap: /d' "$base.$side.txt"
    sleep 2.5
  done
done
