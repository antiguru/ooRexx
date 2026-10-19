#!/bin/bash
# usage: run.sh RED_BIN GREEN_BIN OUT_DIR -- runs each Task 6.9 witness from a
# fresh directory on the oracle, the parent (red) and the task (green), writing
# stdout, stderr and rc separately, then summary.tsv.
RED=$1; GREEN=$2; OUT=$3
LANG_DIR=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/lang
names="${NAMES:-condition_call_on_argument signal_on_missing_label raise_result_and_sender error_report_line_ends condition_object_origin}"
printf 'program\tred vs oracle\tgreen vs oracle\n' > "$OUT/summary.tsv"
for name in $names; do
  for side in oracle red green; do
    dir=$(realpath "$(mktemp -d "$OUT/../run.XXXXXX")")
    cp "$LANG_DIR/$name.rex" "$dir/$name.rex"
    mkdir -p "$OUT/$side/$name"
    case $side in
      oracle) ( cd "$dir" && ulimit -v 1048576 && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$name.rex" >"$OUT/$side/$name/stdout" 2>"$OUT/$side/$name/stderr"; echo $? >"$OUT/$side/$name/rc" ) ;;
      red) ( cd "$dir" && memcap 2G timeout 20 "$RED" "$name.rex" >"$OUT/$side/$name/stdout" 2>"$OUT/$side/$name/stderr"; echo $? >"$OUT/$side/$name/rc" ) ;;
      green) ( cd "$dir" && memcap 2G timeout 20 "$GREEN" "$name.rex" >"$OUT/$side/$name/stdout" 2>"$OUT/$side/$name/stderr"; echo $? >"$OUT/$side/$name/rc" ) ;;
    esac
    sed -i "s#$dir/##g" "$OUT/$side/$name/stdout" "$OUT/$side/$name/stderr"
    rm -r "$dir"
  done
  verdict() {
    local d=""
    for f in stdout stderr rc; do cmp -s "$OUT/oracle/$name/$f" "$OUT/$1/$name/$f" || d="$d $f"; done
    [ -z "$d" ] && echo identical || echo "differs:$d"
  }
  printf '%s\t%s\t%s\n' "$name" "$(verdict red)" "$(verdict green)" >> "$OUT/summary.tsv"
done
