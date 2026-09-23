#!/bin/bash
# usage: perclause.sh NAME -- per-clause ex-libc for nop and x = y, plus per-function split
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round5/spills
A=$S/acc
cd "$A" || exit 1
for p in nop asg; do
    a=$(python3 "$S/sh/sumobj.py" "cg.$1.${p}100" | cut -f4)
    b=$(python3 "$S/sh/sumobj.py" "cg.$1.${p}50" | cut -f4)
    echo "$1 $p per clause: $(echo "scale=3; ($a - $b) / 1000000" | bc)"
    python3 "$S/sh/addrtsv.py" "$S/bin/$1-rexx-run" "cg.$1.${p}100" "cg.$1.${p}50" 1000000 > "$p.$1.tsv"
    awk -F'\t' '{a[$3]+=$1} END {for (k in a) printf "   %.2f\t%s\n", a[k], k}' "$p.$1.tsv" | sort -rn
done
