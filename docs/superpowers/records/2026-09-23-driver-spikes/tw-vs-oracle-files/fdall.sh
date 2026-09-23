#!/bin/bash
# Function diffs for every program and engine, round 1, into fd/.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$S/m1
mkdir -p "$S/fd"
for f in "$S"/progs/*.rex; do
    p=$(basename "$f" .rex)
    c=ctrl
    [ "$p" = stem_read ] && c=ctrlfill
    [ "$p" = ctrl ] && continue
    for e in oracle tree-walker ir; do
        python3 "$S/fndiff.py" "$M/cg.1.$p.$e" "$M/cg.1.$c.$e" 200000 0.5 > "$S/fd/$p.$e.txt" &
    done
done
wait
