#!/bin/bash
# run1.sh LABEL BIN [args...] : run BIN with args from cwd, print the three descriptors.
label=$1; shift
bin=$1; shift
out=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/9c-t5/tools/out.txt
err=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/9c-t5/tools/err.txt
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 "$bin" "$@" >"$out" 2>"$err" </dev/null )
rc=$?
printf '=== %s\n--- argv:' "$label"
for a in "$@"; do printf ' [%s]' "$a"; done
printf '\n--- stdout\n'; cat -v "$out"
printf '%s\n' '--- stderr'; cat -v "$err"
printf '%s %s\n\n' '--- rc' "$rc"
