#!/bin/bash
# usage: controls_untracked.sh
# Controls for the fix-round-1 tooling (review I1), on the main checkout at
# HEAD with c7's parent and destination. Each plant is removed by explicit
# path afterwards.
#   U0 clean tree                                   -> untracked: 0 (verify passes)
#   U1 an untracked file under the crate root, named as I1's were
#      (rust/crates/rexx-parse/src-out-verdict.txt) -> untracked: 1 (verify fails)
#   U2 an untracked .rs file under src/ that is not a destination
#                                                   -> untracked: 1 (verify fails)
#   U3 controls_task9.py --pre 7 (the invocation that caused I1) leaves
#      nothing untracked or changed under rust/
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
chk() { $S/tools/other_edits9.sh ctl instruction.rs instruction/parse.rs | tee $S/art/ctl-untracked.out | tr '\n' ';'; /bin/grep -q '^untracked: 0 files' $S/art/ctl-untracked.out && /bin/grep -q '^other edits: 0 files' $S/art/ctl-untracked.out && echo " verify9 would PASS" || echo " verify9 would FAIL"; }
echo "U0 clean tree (expected PASS): $(chk)"
P=$R/crates/rexx-parse/src-out-verdict.txt; echo planted > $P
echo "U1 untracked src-out-verdict.txt (expected FAIL): $(chk)"; rm $P
P=$R/crates/rexx-parse/src/instruction/planted.rs; echo '// planted' > $P
echo "U2 untracked src/instruction/planted.rs (expected FAIL): $(chk)"; rm $P
python3 $S/tools/controls_task9.py --pre 7 > $S/art/ctl-pre7.txt 2>&1; echo "U3 controls_task9.py --pre 7: $(tail -1 $S/art/ctl-pre7.txt); git status under rust/ afterwards: [$(git -C $R status --short -- . | tr '\n' ' ')] (expected empty)"
echo "U4 after the controls: $(chk)"
rm -f $S/art/ctl-untracked.out $S/art/cctl-other-edits.diff
