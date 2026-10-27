#!/bin/bash
# every cyclic probe on BIN=$1 under TAGPREFIX=$2, 8 at a time
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t85-fix4
bin=$1; pre=$2
P=$S/probes
jobs=()
run() { "$S/run.sh" "$@" & jobs+=($!); if [ ${#jobs[@]} -ge 8 ]; then wait "${jobs[0]}"; jobs=("${jobs[@]:1}"); fi; }
run $pre-sd1 $bin $P/sd1.rex
run $pre-sd_via $bin $P/sd_via.rex
run $pre-sd_tr $bin $P/sd_tr.rex
run $pre-sd_msg $bin $P/sd_msg.rex
run $pre-sd0 $bin $P/sd0.rex
for i in 1 2 3 4 5; do run $pre-stemdef$i $bin $P/stemdef.rex $i; done
for i in 1 2 3 4 5 6 7; do run $pre-sub$i $bin $P/sub$i.rex; done
for i in 1 2 3 4 5 6 7 8 9 10 11 12; do run $pre-sd_ops$i $bin $P/sd_ops.rex $i; done
wait
for f in "$S"/out/$pre-*.rc; do t=$(basename "$f" .rc); printf '%s rc=%s out=%s err=%s\n' "$t" "$(cat "$f")" "$(tr '\n' '|' < "$S/out/$t.out")" "$(tr '\n' '|' < "$S/out/$t.err" | cut -c1-160)"; done
