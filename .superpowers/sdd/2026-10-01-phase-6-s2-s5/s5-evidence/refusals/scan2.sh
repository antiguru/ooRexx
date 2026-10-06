S=/tmp/claude-1000/p6-s5-refusals
cd $S/tree
grep -n 'CLOSED_PHASES' docs/superpowers/plans/phase-4-exclusions.txt
sed -i 's/`CLOSED_PHASES`/`XLOSED_PHASES`/' docs/superpowers/plans/phase-4-exclusions.txt
cd $S/tree/rust
CARGO_TARGET_DIR=$S/target memcap 8G cargo test --release -p rexx-exec --test closed_phases 2>&1 | grep -a 'test \|panicked\|OWNER\|result'
cd $S/tree
sed -i 's/`XLOSED_PHASES`/`CLOSED_PHASES`/' docs/superpowers/plans/phase-4-exclusions.txt
