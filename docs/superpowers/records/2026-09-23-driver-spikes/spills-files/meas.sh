#!/bin/bash
# Two interleaved callgrind rounds of every axis on two binaries.
# usage: meas.sh TAG BASE_NAME HEAD_NAME   (binaries bin/NAME-rexx-run)
# round 1 runs base then head, round 2 head then base; each round's runs are parallel across axes.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round5/spills
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-a5b211e2b657b43ee/rust
M=$S/meas/$1
mkdir -p "$M"
AXES="nop assign rexxcps varlookup emptyloop arith dispatch compound"
prog() {
    if [ "$1" = rexxcps ]; then echo "$R/bench-rexxcps/rexxcps.rex"; else echo "$R/bench-programs/$1.rex"; fi
}
one() { # name axis round
    local d
    d=$(mktemp -d "$S/mrun.XXXXXX")
    (cd "$d" && valgrind --tool=callgrind --compress-strings=no --callgrind-out-file="$M/cg.$1.$2.r$3" \
        "$S/bin/$1-rexx-run" "$(prog "$2")" > "$M/out.$1.$2.r$3" 2> "$M/err.$1.$2.r$3"; echo "rc=$?" >> "$M/err.$1.$2.r$3")
}
for r in 1 2; do
    if [ $r = 1 ]; then order="$2 $3"; else order="$3 $2"; fi
    for b in $order; do
        for a in $AXES; do one "$b" "$a" "$r" & done
        wait
    done
done
for b in "$2" "$3"; do
    for a in $AXES; do
        for r in 1 2; do
            printf '%s\t%s\tr%s\t%s\t%s\n' "$b" "$a" "$r" "$(python3 "$S/sh/sumobj.py" "$M/cg.$b.$a.r$r")" "$(tail -n1 "$M/err.$b.$a.r$r")"
        done
    done
done > "$M/summary.tsv"
python3 "$S/sh/table.py" "$M/summary.tsv" "$2" "$3" | tee "$M/table.txt"
