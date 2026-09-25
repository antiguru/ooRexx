#!/bin/bash
# usage: move_env_child.sh N CHILD KEYSFILE "MOD COMMENT" "MODULE DOC" "USE LINE"
# An environment.rs child commit, up to the checks: pre<N>, the units named
# in KEYSFILE (item-tool keys, one per line) moved into environment/CHILD.rs
# (impl Interp members in one impl block), `mod CHILD;` declared with its
# comment, the module doc, the explicit `use super::{..}` list, rustfmt, any
# member the rest of the crate cannot reach widened, and the check that no
# file outside the parent and the child changed.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-7
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1; C=$2
$S/tools/prep.sh $N || exit 1
cp $3 $S/c$N/keys.txt
cd $R/crates/rexx-exec/src
mapfile -t K < $3
python3 $S/tools/move_lib_child.py environment.rs environment/$C.rs $S/c$N/removed.json x "${K[@]}" | tee $S/c$N/move.txt || exit 1
python3 $S/tools/declare_env_child.py . $C "$4" "$5" || exit 1
python3 $S/tools/set_imports.py environment/$C.rs "$6" || exit 1
cd $R && cargo fmt --all || exit 1
CARGO_TARGET_DIR=$S/t-check python3 $S/tools/widen_methods.py $R $S/t-check environment/$C.rs | tee -a $S/c$N/move.txt
cargo fmt --all
python3 $S/tools/other_edits.py $R $S/art/c$N-other-edits-check.txt "environment.rs=instruments" "environment/$C.rs=instruments"
