#!/bin/bash
# usage: ad.sh PROG ENGINE FUNC_SUBSTRING [CONTROL] [BIAS]
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$S/mi
B=$S/bin/tw-rexx-run
[ "$2" = oracle ] && B=/home/moritz/dev/repos/ooRexx/build/lib/librexx.so.4
python3 "$S/addrdiff.py" "$B" "$M/cg.$1.$2" "$M/cg.${4:-ctrl}.$2" "$3" 200000 "${5:-0}"
