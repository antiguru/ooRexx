#!/bin/bash
# Runs parks/*.rex, conditions/*.rex and started/*.rex on the oracle, the round-3 binary
# ("base", the code of 518613a71) and the round's binary from a fresh directory, writing
# NAME.SIDE.{out,err,rc}; each halt/*.rex and language/*.rex goes through halt/sigint.sh
# (SIGINT by PID after 0.6 s) into NAME.SIDE.txt.
# usage: run.sh BASE_BIN HEAD_BIN SCRATCH_DIR
base=$1; head=$2; scratch=$3
here=$(cd "$(dirname "$0")" && pwd)
for probe in "$here"/parks/*.rex "$here"/conditions/*.rex "$here"/started/*.rex; do
  stem=${probe%.rex}; file=$(basename "$probe")
  for side in oracle base head; do
    dir=$(mktemp -d "$scratch/p613r4.XXXX"); cp "$probe" "$dir/"
    case $side in
      oracle) ( cd "$dir" && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$file" > "$stem.$side.out" 2> "$stem.$side.err" ); echo $? > "$stem.$side.rc" ;;
      base) ( cd "$dir" && memcap 2G timeout 20 "$base" "$file" > "$stem.$side.out" 2> "$stem.$side.err" ); echo $? > "$stem.$side.rc" ;;
      head) ( cd "$dir" && memcap 2G timeout 20 "$head" "$file" > "$stem.$side.out" 2> "$stem.$side.err" ); echo $? > "$stem.$side.rc" ;;
    esac
    sed -i -e "s|$dir|DIR|g" -e '/^memcap: /d' "$stem.$side.err"; rm -rf "$dir"
  done
done
for probe in "$here"/halt/*.rex "$here"/language/*.rex; do
  stem=${probe%.rex}; file=$(basename "$probe")
  for side in oracle base head; do
    dir=$(mktemp -d "$scratch/p613r4.XXXX"); cp "$probe" "$dir/"
    case $side in
      oracle) ( cd "$dir" && "$here/halt/sigint.sh" oracle "$file" 0.6 ) > "$stem.$side.txt" ;;
      base) ( cd "$dir" && "$here/halt/sigint.sh" crate "$file" 0.6 "$base" ) > "$stem.$side.txt" ;;
      head) ( cd "$dir" && "$here/halt/sigint.sh" crate "$file" 0.6 "$head" ) > "$stem.$side.txt" ;;
    esac
    sed -i -e "s|$dir|DIR|g" -e '/^memcap: /d' "$stem.$side.txt"; rm -rf "$dir"
    sleep 2.5
  done
done
