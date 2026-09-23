#!/bin/bash
# usage: rep.sh AXIS N NAME... -- N callgrind runs of AXIS per binary, in parallel; prints ex-libc per run
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round4/clause-overhead
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-accb7993007714f77/rust
axis=$1; n=$2; shift 2
if [ "$axis" = rexxcps ]; then P=$R/bench-rexxcps/rexxcps.rex; else P=$R/bench-programs/$axis.rex; fi
M=$S/rep; mkdir -p "$M"
for b in "$@"; do for i in $(seq "$n"); do
  ( d=$(mktemp -d "$S/rrun.XXXXXX"); cd "$d" && valgrind --tool=callgrind --compress-strings=no --callgrind-out-file="$M/cg.$b.$axis.$i" "$S/bin/$b-rexx-run" "$P" >/dev/null 2>&1; rmdir "$d" 2>/dev/null ) &
done; done
wait
for b in "$@"; do for i in $(seq "$n"); do
  echo "$b $axis $i $(python3 "$S/sh/sumobj.py" "$M/cg.$b.$axis.$i" | cut -f4)"
  rm "$M/cg.$b.$axis.$i"
done; done
