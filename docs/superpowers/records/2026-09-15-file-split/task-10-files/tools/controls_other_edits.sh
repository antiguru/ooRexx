#!/bin/bash
# Negative controls for other_edits.py on c7 (the version constants): a
# scratch worktree at c7's parent with c7's files checked out over it
# reproduces the commit's working tree; the check passes as committed, and
# fails for each planted defect: a renamed path that also changes the name
# it reaches, a stray edit in a file declared insert-only, and a changed
# file with no rule.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
W=$S/oe-wt; C7=$(git -C $M log --format=%h -F --grep=task-5-files/c7/)
[ -d $W ] || git -C $M worktree add -q --detach $W $C7^
RULES=("parse_template.rs=instruments" "version.rs=instruments" "lib.rs=insert" "parse_template/tests.rs=insert"
  "dispatch/rexx_info.rs=substsort:\\bparse_template\\b=>version"
  "trace.rs=subst:crate::parse_template::PLATFORM=>crate::version::PLATFORM"
  "environment.rs=subst:crate::parse_template::LINE_END=>crate::version::LINE_END")
reset() { git -C $W checkout -q -f $C7^; rm -f $W/rust/crates/rexx-exec/src/version.rs; git -C $W checkout -q $C7 -- rust/crates; git -C $W reset -q; }
run() { python3 $S/tools/other_edits.py $W/rust /dev/null "${RULES[@]}" > $S/oe.out; echo "$1: exit $? (expected $2); $(/bin/grep -a 'FAIL' $S/oe.out | head -1)"; }
reset; run "as committed" 0
reset; sed -i 's/version::RELEASE/version::MODIFICATION/' $W/rust/crates/rexx-exec/src/dispatch/rexx_info.rs; run "a renamed path reaches another constant" 1
reset; sed -i 's/^mod parse_template;$/pub mod parse_template;/' $W/rust/crates/rexx-exec/src/lib.rs; run "an insert-only file changes a line" 1
reset; echo "// stray" >> $W/rust/crates/rexx-exec/src/value.rs; run "a changed file with no rule" 1
reset
