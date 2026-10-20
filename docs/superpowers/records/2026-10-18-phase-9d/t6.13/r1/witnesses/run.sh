#!/bin/bash
# Runs each fix-round-1 witness on the oracle, the parent and the round's
# binary, each from a fresh directory with the oracle's library directory on
# LD_LIBRARY_PATH, RUNS times per side, and writes the first run's three
# descriptors, the distinct-output count per side, and summary.tsv.
# usage: run.sh PARENT_BIN HEAD_BIN SCRATCH_DIR RUNS
parent=$1; head=$2; scratch=$3; runs=$4
here=$(cd "$(dirname "$0")" && pwd)
corpus=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/lang
lib=/home/moritz/dev/repos/ooRexx/build/lib
printf 'program\toracle\tparent\thead\tdistinct outputs (oracle/parent/head)\n' > "$here/summary.tsv"
for name in command_library_environments_unregistered command_redirected_targets_mid_command command_redirected_targets_collected trace_commands_address_clause command_redirected_target_raises; do
  declare -A seen=()
  for side in oracle parent head; do
    sums=()
    for run in $(seq 1 "$runs"); do
      dir=$(mktemp -d "$scratch/w613r1.XXXX"); cp "$corpus/$name.rex" "$dir/"
      o=$dir/out; e=$dir/err
      case $side in
        oracle) ( cd "$dir" && ulimit -v 1048576 && LD_LIBRARY_PATH=$lib timeout -k 5 30 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$name.rex" > "$o" 2> "$e" ); rc=$? ;;
        parent) ( cd "$dir" && LD_LIBRARY_PATH=$lib memcap 2G timeout 30 "$parent" "$name.rex" > "$o" 2> "$e" ); rc=$? ;;
        head) ( cd "$dir" && LD_LIBRARY_PATH=$lib memcap 2G timeout 30 "$head" "$name.rex" > "$o" 2> "$e" ); rc=$? ;;
      esac
      sed "s|$dir|DIR|g" "$e" > "$e.n"
      if [ "$run" = 1 ]; then cp "$o" "$here/$name.$side.out"; cp "$e.n" "$here/$name.$side.err"; echo $rc > "$here/$name.$side.rc"; fi
      sums+=("$( { cat "$o"; echo ---; cat "$e.n"; echo "rc $rc"; } | md5sum | cut -c1-32)")
    done
    seen[$side]=$(printf '%s\n' "${sums[@]}" | sort -u | wc -l)
  done
  verdict() {
    if cmp -s "$here/$name.oracle.out" "$here/$name.$1.out" && cmp -s "$here/$name.oracle.err" "$here/$name.$1.err" && cmp -s "$here/$name.oracle.rc" "$here/$name.$1.rc"; then echo identical; else echo "differs (rc $(cat "$here/$name.$1.rc"))"; fi
  }
  printf '%s\trc %s\t%s\t%s\t%s/%s/%s\n' "$name" "$(cat "$here/$name.oracle.rc")" "$(verdict parent)" "$(verdict head)" "${seen[oracle]}" "${seen[parent]}" "${seen[head]}" >> "$here/summary.tsv"
  unset seen
done
