set -e
S=/tmp/claude-1000/p6-s5-refusals
rm -rf /tmp/claude-1000/p6-s5-refusals/tree
mkdir -p $S/tree
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
git archive 540e6a72ea7c485972ac1d7c4f36b97dea322ad4 | tar -x -C $S/tree
find $S/tree -exec touch {} +
cd $S/tree/rust/crates/rexx-exec/tests
sed -i 's/const CLOSED: &\[&str\] = &\["Phase 7", "Phase 8"\];/const CLOSED: \&[\&str] = \&["Phase 6", "Phase 7", "Phase 8"];/' closed_phases.rs
sed -i 's/line.contains("\\"Phase 6\\"") || line.contains("\\"Phase 10\\"")/line.contains("\\"Phase 10\\"")/' closed_phases.rs
grep -n 'const CLOSED\|line.contains' closed_phases.rs
cd $S/tree/rust
CARGO_TARGET_DIR=$S/target memcap 8G cargo test --release -p rexx-exec --test closed_phases 2>&1 | grep -a 'Compiling rexx-exec\|test \|panicked\|Phase\|result\|^ *"' 
