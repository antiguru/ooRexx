#!/usr/bin/env bash
# probe-dirs.sh SETDIR NAME=BINARY...: each SETDIR/<probe>/ holds main.rex and its
# helpers. Each side runs main.rex in a fresh copy of the probe directory; the
# oracle is always the first side. Writes SETDIR/<probe>/<side>.{out,err,rc}
# (paths replaced by DIR) and SETDIR/summary.tsv.
set -u
set_dir=$1; shift
oracle=/home/moritz/dev/repos/ooRexx/build
sides=(oracle "$@")
{ printf 'probe\toracle rc'; for s in "$@"; do printf '\t%s rc\t%s vs oracle' "${s%%=*}" "${s%%=*}"; done; printf '\n'; } > "$set_dir/summary.tsv"
for p in "$set_dir"/*/; do
  p=${p%/}; n=$(basename "$p")
  for side in "${sides[@]}"; do
    name=${side%%=*}; bin=${side#*=}
    run=$(mktemp -d -p /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t66/r2); cp -r "$p"/. "$run"/; rm -f "$run"/*.out "$run"/*.err "$run"/*.rc
    if [ "$name" = oracle ]; then
      ( cd "$run"; ulimit -v 1048576; LD_LIBRARY_PATH=$oracle/lib timeout -k 5 20 $oracle/bin/rexx main.rex > o 2> e; echo $? > r )
    else
      ( cd "$run"; timeout 20 memcap 2G "$bin" main.rex > o 2> e; echo $? > r )
    fi
    sed "s|$run|DIR|g" "$run/o" > "$p/$name.out"; sed "s|$run|DIR|g" "$run/e" > "$p/$name.err"; cp "$run/r" "$p/$name.rc"
    rm -r "$run"
  done
  row="$n\t$(cat $p/oracle.rc)"
  for s in "$@"; do
    name=${s%%=*}
    if cmp -s "$p/oracle.out" "$p/$name.out" && cmp -s "$p/oracle.err" "$p/$name.err" && cmp -s "$p/oracle.rc" "$p/$name.rc"; then v=same; else v=DIFFERS; fi
    row="$row\t$(cat $p/$name.rc)\t$v"
  done
  printf "$row\n" >> "$set_dir/summary.tsv"
done
