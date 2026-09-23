#!/bin/bash
# Accounting runs: per-instruction callgrind of 100- and 50-clause bodies on one binary.
# usage: acc.sh NAME   (binary bin/NAME-rexx-run; outputs acc/cg.NAME.*)
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round4/clause-overhead
A=$S/acc
mkdir -p "$A"
python3 "$S/gen/gen2.py" nop 20000 100 > "$A/nop100.rex"
python3 "$S/gen/gen2.py" nop 20000 50 > "$A/nop50.rex"
python3 "$S/gen/gen2.py" assign 20000 100 > "$A/asg100.rex"
python3 "$S/gen/gen2.py" assign 20000 50 > "$A/asg50.rex"
for p in nop100 nop50 asg100 asg50; do
    bash "$S/sh/cgi.sh" "$S/bin/$1-rexx-run" "$A/$p.rex" "$A/cg.$1.$p" &
done
wait
tail -n1 "$A"/cg."$1".*.err
