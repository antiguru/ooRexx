#!/bin/bash
# usage: oracle.sh TAG FILE [ARGS...]   (standard wrapper, fresh empty dir; never on a cyclic program)
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t85-fix5
tag=$1; file=$2; shift 2
d=$(mktemp -d "$S/run/XXXXXX")
cd "$d" || exit 2
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$file" "$@" ) > "$S/out/$tag.out" 2> "$S/out/$tag.err"
echo $? > "$S/out/$tag.rc"
