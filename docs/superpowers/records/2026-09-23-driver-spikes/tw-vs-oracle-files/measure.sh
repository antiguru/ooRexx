#!/bin/bash
# Callgrind every program on the oracle, the tree-walker and the IR, two rounds,
# the second in reverse engine order, each run from a fresh empty directory.
# usage: measure.sh OUTDIR [program...]
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$1
shift
mkdir "$M" || exit 1
progs="$*"
[ -z "$progs" ] && progs=$(cd "$S/progs" && ls *.rex | sed 's/\.rex$//')
sha256sum "$S/bin/text.bin" > "$M/text-hash.txt"
run() {
    local round=$1 prog=$2 eng=$3
    local d=$M/run.$round.$prog.$eng
    mkdir "$d" || return 1
    cd "$d" || return 1
    local f=$S/progs/$prog.rex
    local cg=$M/cg.$round.$prog.$eng
    if [ "$eng" = oracle ]; then
        ( ulimit -v 1048576
          LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
          valgrind --tool=callgrind --compress-strings=no --callgrind-out-file="$cg" \
          /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" ) >"$M/out.$round.$prog.$eng" 2>"$M/err.$round.$prog.$eng"
    else
        REXX_ENGINE=$eng valgrind --tool=callgrind --compress-strings=no --callgrind-out-file="$cg" \
          "$S/bin/tw-rexx-run" "$f" >"$M/out.$round.$prog.$eng" 2>"$M/err.$round.$prog.$eng"
    fi
    echo $? > "$M/rc.$round.$prog.$eng"
}
for round in 1 2; do
    if [ "$round" = 1 ]; then engs="oracle tree-walker ir"; else engs="ir tree-walker oracle"; fi
    for p in $progs; do
        for e in $engs; do
            run "$round" "$p" "$e" &
        done
    done
    wait
done
for f in "$M"/cg.*; do
    n=${f##*/cg.}
    printf '%s\t%s\t%s\n' "$n" "$(cat "$M/rc.$n")" "$(python3 "$S/sumobj.py" "$f")"
done > "$M/results.tsv"
echo finished > "$M/status"
