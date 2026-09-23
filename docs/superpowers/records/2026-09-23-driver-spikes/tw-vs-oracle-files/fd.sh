#!/bin/bash
# usage: fd.sh PROG ENGINE [MIN] [CONTROL] [ROUND]
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$S/m1
r=${5:-1}
python3 "$S/fndiff.py" "$M/cg.$r.$1.$2" "$M/cg.$r.${4:-ctrl}.$2" 200000 "${3:-0.5}"
