#!/bin/bash
# usage: ld.sh PROG ENGINE FUNC_SUBSTRING [MIN] [CONTROL]
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$S/m1
python3 "$S/lndiff.py" "$M/cg.1.$1.$2" "$M/cg.1.${5:-ctrl}.$2" "$3" 200000 "${4:-0.5}"
