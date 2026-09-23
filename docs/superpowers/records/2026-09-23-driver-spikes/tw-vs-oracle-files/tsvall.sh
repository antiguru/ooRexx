#!/bin/bash
# Per-instruction TSVs for every program in mi/, walker and oracle, against ctrl
# (ctrl2 is the empty loop: its extra 200000 iterations against ctrl).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$S/mi
mkdir -p "$S/tsv"
for f in "$M"/cg.*.tree-walker; do
    p=${f#"$M"/cg.}
    p=${p%.tree-walker}
    [ "$p" = ctrl ] && continue
    python3 "$S/addrtsv.py" "$S/bin/tw-rexx-run" "$M/cg.$p.tree-walker" "$M/cg.ctrl.tree-walker" > "$S/tsv/$p.tree-walker.tsv" &
    python3 "$S/addrtsv.py" /home/moritz/dev/repos/ooRexx/build/lib/librexx.so.4 "$M/cg.$p.oracle" "$M/cg.ctrl.oracle" > "$S/tsv/$p.oracle.tsv" &
done
wait
