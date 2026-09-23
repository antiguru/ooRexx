#!/bin/bash
# usage: flips.sh A B -- nm presence of the helpers whose inlining has flipped before, and per-function self deltas on rexxcps, varlookup, nop, assign
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round5/spills
for b in "$1" "$2"; do
  echo "$b: $(nm -C "$S/bin/$b-rexx-run" | /bin/grep -a -c -E 'arith_small_int|Interp>::read_at$|drop_in_place<core::option::Option<rexx_exec::run::LoopHeaderValues>>|drop_glue::<core::option::Option<rexx_exec::run::LoopHeaderValues>>') helpers: $(nm -C "$S/bin/$b-rexx-run" | /bin/grep -a -o -E 'arith_small_int|Interp>::read_at$|Option<rexx_exec::run::LoopHeaderValues>>' | sort | tr '\n' ' ')"
done
for a in rexxcps varlookup nop assign; do
  echo "== $a"
  python3 "$S/sh/fncmp.py" "$S/meas/$2/cg.$1.$a.r1" "$S/meas/$2/cg.$2.$a.r1" 8
done
