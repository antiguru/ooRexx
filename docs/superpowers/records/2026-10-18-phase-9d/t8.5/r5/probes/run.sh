#!/bin/bash
# usage: run.sh TAG BIN FILE [ARGS...]   (crate binary, memcap 2G, timeout -k 5 20, fresh empty dir; wall time in TAG.secs)
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t85-fix5
tag=$1; bin=$2; file=$3; shift 3
d=$(mktemp -d "$S/run/XXXXXX")
cd "$d" || exit 2
s=$(date +%s.%N)
LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib memcap 2G timeout -k 5 20 "$bin" "$file" "$@" > "$S/out/$tag.out" 2> "$S/out/$tag.err"
echo $? > "$S/out/$tag.rc"
echo "$(date +%s.%N) - $s" | bc > "$S/out/$tag.secs"
