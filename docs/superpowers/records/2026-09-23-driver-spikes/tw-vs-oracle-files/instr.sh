#!/bin/bash
# Per-instruction callgrind runs for a few programs on the walker and the oracle.
# usage: instr.sh PROG...
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$S/mi
mkdir -p "$M"
for p in "$@"; do
    for e in tree-walker oracle; do
        d=$M/run.$p.$e
        mkdir "$d" || exit 1
        (
            cd "$d" || exit 1
            if [ "$e" = oracle ]; then
                ulimit -v 1048576
                LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
                valgrind --tool=callgrind --dump-instr=yes --compress-strings=no --compress-pos=no \
                    --callgrind-out-file="$M/cg.$p.$e" /home/moritz/dev/repos/ooRexx/build/bin/rexx "$S/progs/$p.rex"
            else
                REXX_ENGINE=$e valgrind --tool=callgrind --dump-instr=yes --compress-strings=no --compress-pos=no \
                    --callgrind-out-file="$M/cg.$p.$e" "$S/bin/tw-rexx-run" "$S/progs/$p.rex"
            fi
        ) >"$M/out.$p.$e" 2>"$M/err.$p.$e" &
    done
done
wait
