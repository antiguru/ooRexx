#!/bin/bash
# Re-derives every census with the fold split; prints one summary line per binary and axis.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round5/spills
cd "$S" || exit 1
for b in base s1 s2 s3 s4 s5; do
  rm_=remarks.$b.log; [ "$b" = base ] && rm_=remarks.log
  for p in nop asg; do
    python3 sh/spills.py "bin/$b-rexx-run" "$rm_" "acc/cg.$b.${p}100" "acc/cg.$b.${p}50" 1000000 --rows run_ops_from > "census.$p.$b.txt"
    echo "$b $p $(/bin/grep -a -E '^(total|spill-store|spill-reload|spill-fold|spill-total|spill-extra)' "census.$p.$b.txt" | tr '\n' ' ')"
  done
done
python3 sh/spills.py bin/base-rexx-run remarks.log acc/cg.base.rexxcps 20000000 --rows "run_ops_from::<false" > census.rexxcps.base.txt
echo "base rexxcps $(/bin/grep -a -E '^(total|spill-store|spill-reload|spill-fold|spill-total|spill-extra)' census.rexxcps.base.txt | tr '\n' ' ')"
