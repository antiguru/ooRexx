#!/bin/bash
# usage: cand.sh NAME -- remark build of the current tree (checked against bin/NAME's .text), then the per-clause census
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round5/spills
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-a5b211e2b657b43ee/rust
cd "$R" || exit 1
CARGO_TARGET_DIR=$S/rtarget RUSTFLAGS="-Cremark=stack-frame-layout" cargo build --release -p rexx-exec --bin rexx-run > "$S/remarks.$1.log" 2>&1 || exit 2
objcopy -O binary --only-section=.text "$S/rtarget/release/rexx-run" "$S/rtext.$1"
a=$(sha256sum < "$S/rtext.$1" | cut -d' ' -f1); b=$(sha256sum < "$S/bin/$1.text" | cut -d' ' -f1)
rm "$S/rtext.$1"
echo "remark build $a, measured $b"
[ "$a" = "$b" ] || { echo MISMATCH; exit 3; }
bash "$S/sh/acc.sh" "$1" > /dev/null
bash "$S/sh/perclause.sh" "$1" | /bin/grep -a "per clause"
cd "$S" || exit 1
python3 sh/spills.py "bin/$1-rexx-run" "remarks.$1.log" "acc/cg.$1.nop100" "acc/cg.$1.nop50" 1000000 --rows run_ops_from > "census.nop.$1.txt"
python3 sh/spills.py "bin/$1-rexx-run" "remarks.$1.log" "acc/cg.$1.asg100" "acc/cg.$1.asg50" 1000000 --rows run_ops_from > "census.asg.$1.txt"
for p in nop asg; do echo "$p $(/bin/grep -a -E '^(total|spill-total|obj|arg)' "census.$p.$1.txt" | tr '\n' ' ')"; done
bash "$S/sh/cgi.sh" "$S/bin/$1-rexx-run" "$R/bench-rexxcps/rexxcps.rex" "$S/acc/cg.$1.rexxcps"
python3 sh/spills.py "bin/$1-rexx-run" "remarks.$1.log" "acc/cg.$1.rexxcps" 20000000 > "census.rexxcps.$1.txt"
echo "rexxcps $(/bin/grep -a -E '^(total|spill-total|obj|push/pop)' "census.rexxcps.$1.txt" | tr '\n' ' ')"
rm "acc/cg.$1.rexxcps"
