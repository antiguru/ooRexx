#!/bin/bash
# Re-runs instruments 1-3 with the final tooling on every Task 6 commit pair:
# PRE is the parent commit's src, POST the commit's, both from `git archive`;
# each output is compared with the one committed for that commit. c6 is also
# run with `--expect-drop=hash::`, which the per-commit run did not have.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-8
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
D=$M/docs/superpowers/records/2026-09-15-file-split/task-6-files
O=$S/art/final/rerun; mkdir -p $O
tree() { [ -d $S/trees/$1 ] || { mkdir -p $S/trees/$1; git -C $M archive $1 rust/crates/rexx-exec/src | tar -x -C $S/trees/$1; }; echo $S/trees/$1/rust/crates/rexx-exec/src; }
run() {
  N=$1; P=$2; DEST=$3; T=$4; shift 4
  C=$(git -C $M log --format=%h -F --grep=task-6-files/c$N/ ed8cb3f03..HEAD); PC=$(git -C $M rev-parse --short=9 $C^)
  X=; [ "$T" != none ] && X=--tests
  SPLIT_PARENT=${P%.rs} SPLIT_PARENT_RS=$P SPLIT_DESTS=$DEST SPLIT_TESTS_RS=$T python3 $S/tools/instruments.py $(tree $PC) $(tree $C) $D/c$N/removed.json $O/c$N $X "$@" > /dev/null
  same=1; for k in instrument1 instrument2 instrument3 verdict; do
    cmp -s <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $D/c$N/c$N-$k.txt) <(sed 's#/tmp/tmp[a-z0-9_]*/#/tmp/X/#g' $O/c$N-$k.txt) || same=0
  done
  echo "c$N $C (parent $PC): $(head -1 $O/c$N-verdict.txt); outputs $([ $same = 1 ] && echo identical to || echo DIFFER from) the committed ones"
}
run 1 dispatch/library.rs dispatch/library/tests.rs dispatch/library/tests.rs
run 2 dispatch/native.rs dispatch/native/tests.rs dispatch/native/tests.rs '--expect-edit=tests::fn check_binds' '--expect-edit=tests::fn an_entry_point_resolves_whatever_case_it_is_asked_for'
run 3 dispatch.rs dispatch/package.rs none
run 4 dispatch.rs dispatch/files.rs none
run 5 dispatch.rs dispatch/string.rs none
run 6 dispatch.rs dispatch/hash.rs none '--expect-edit=fn native_hash_at' '--expect-edit=fn native_hash_put' '--expect-edit=fn native_hash_unknown'
mkdir -p $O/c6-drop; D0=$D; O0=$O; O=$O/c6-drop
run 6 dispatch.rs dispatch/hash.rs none '--expect-edit=fn native_hash_at' '--expect-edit=fn native_hash_put' '--expect-edit=fn native_hash_unknown' '--expect-drop=hash::'
O=$O0
run 7 dispatch/buffer.rs dispatch/method_arguments.rs none "--expect-line=//! \`MutableBuffer\`'s primitive methods, and the argument and byte-search" "--expect-line=//! helpers the other primitive methods share."
run 8 dispatch/hash.rs dispatch/hash/stem.rs none
run 9 dispatch/hash.rs dispatch/hash/relation.rs none
run 10 dispatch/collection.rs dispatch/collection/list.rs none
run 11 dispatch/collection.rs dispatch/collection/queue.rs none
run 12 dispatch/collection.rs dispatch/collection/supplier.rs none
run 13 dispatch/collection.rs dispatch/array/sort.rs none
run 14 dispatch/collection.rs dispatch/array/surface.rs none "--expect-gone=" "--expect-gone=/// The collection classes' primitive methods, chained into" "--expect-gone=/// \`ObjectModel::build\`." "--expect-gone=pub(super) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[" "--expect-gone=];"
