#!/bin/bash
# Negative controls for this task's other_edits.py rules. For each of c7
# (`imports:`), c13 (`narrow+insert`) and c14 (several `subst:` joined by
# `;;`), a scratch worktree at the commit's parent with the commit's files
# checked out over it reproduces the commit's working tree; the check passes
# with the commit's own rules, and fails for each planted defect.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-8
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
D=$M/docs/superpowers/records/2026-09-15-file-split/task-6-files
W=$S/oe6-wt
commit() { git -C $M log --format=%h -F --grep=task-6-files/c$1/ ed8cb3f03..HEAD; }
reset() { C=$(commit $1); [ -d $W ] || git -C $M worktree add -q --detach $W $C^; git -C $W checkout -q -f $C^; git -C $W clean -q -fd -- rust/crates; git -C $W checkout -q $C -- rust/crates; git -C $W reset -q; }
run() { mapfile -t RULES < $D/c$1/rules; python3 $S/tools/other_edits.py $W/rust $S/oe6.out "${RULES[@]}" > /dev/null; r=$?; echo "c$1 $2: exit $r (expected $3); $(/bin/grep -a "FAIL \\|BAD " $S/oe6.out | /bin/grep -v FAILURES | tail -1)"; }
F=$W/rust/crates/rexx-exec/src
reset 7; run 7 "as committed" 0
reset 7; sed -i 's/^    string_method_argument, usize_or_refuse, whole_method_argument,$/    string_method_argument, whole_method_argument,/; s/^    native_length, native_reverse, native_string_make_string, native_string_makearray,$/    native_length, native_reverse, native_string_make_string, native_string_makearray, usize_or_refuse,/' $F/dispatch.rs; run 7 "a name moves to a module other than the declared one" 1
reset 7; sed -i "s/^\/\/ \`Class\`'s own methods: its readers, the mutators, the class factory\.$/\/\/ \`Class\`'s own methods: its readers and mutators, the class factory./" $F/dispatch.rs; run 7 "a third comment line edited beside the two declared" 1
reset 13; run 13 "as committed" 0
reset 13; sed -i 's/^fn put_native(/fn put_native_(/' $F/dispatch.rs; run 13 "a code line changes beside the narrowing" 1
reset 14; run 14 "as committed" 0
reset 14; sed -i 's/^\/\/\/ A list.s items, its handles and its free stack\.$/\/\/\/ A list'"'"'s items, handles and free stack./' $F/dispatch/collection/list.rs; run 14 "a second edit in a file with two declared substitutions" 1
reset 14; sed -i 's/        \.chain(array::sort::NATIVE_METHODS)$/        .chain(array::surface::NATIVE_METHODS)/' $F/dispatch/tests.rs; run 14 "the test's chain names a slice twice" 1
git -C $M worktree remove --force $W
