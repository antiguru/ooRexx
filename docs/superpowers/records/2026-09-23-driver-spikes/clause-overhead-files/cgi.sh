#!/bin/bash
# Per-instruction callgrind run.
# usage: cgi.sh BIN PROG OUTFILE
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round4/clause-overhead
d=$(mktemp -d "$S/cgrun.XXXXXX")
cd "$d" || exit 99
valgrind --tool=callgrind --dump-instr=yes --compress-strings=no --compress-pos=no \
    --callgrind-out-file="$3" "$1" "$2" > "$3.out" 2> "$3.err"
echo "rc=$?" >> "$3.err"
