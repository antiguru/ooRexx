#!/bin/bash
# The version-constants commit, up to the checks: pre<N>, the block from
# PLATFORM through `part` moved verbatim into version.rs (a sibling of
# parse_template.rs), its module doc, BIT_WIDTH widened for parse_template's
# tests, `mod version;` in lib.rs, the imports and path renames
# (version_edits.py), rustfmt, and the check of every edit outside the move.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-5
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1
$S/tools/prep.sh $N || exit 1
cd $R/crates/rexx-exec/src
L=$(python3 $S/tools/span_of.py parse_template.rs "const PLATFORM" "fn part") || exit 1
python3 $S/tools/move_lib_child.py parse_template.rs version.rs $S/c$N/removed.json x $L | tee $S/c$N/move.txt
echo "block $L" >> $S/c$N/move.txt
python3 - <<'PY' || exit 1
s = open('version.rs').read()
assert s.count("\n//! x\n\nuse super::*;\n\n") == 1
s = s.replace("\n//! x\n\nuse super::*;\n\n", "\n//! The interpreter's version string and the fields `RexxInfo` cuts from it, the\n//! platform name and the line terminator.\n\n", 1)
assert s.count("#[cfg(test)]\nconst BIT_WIDTH") == 1
s = s.replace("#[cfg(test)]\nconst BIT_WIDTH", "#[cfg(test)]\npub(super) const BIT_WIDTH", 1)
open('version.rs', 'w').write(s)
l = open('lib.rs').read()
a = "\nmod parse_template;\n"; assert l.count(a) == 1
l = l.replace(a, a + "\n// The interpreter's version string, platform name and line terminator:\n// what `PARSE VERSION`, `PARSE SOURCE`, `.ENDOFLINE` and `RexxInfo` answer.\nmod version;\n")
open('lib.rs', 'w').write(l)
PY
python3 $S/tools/version_edits.py . || exit 1
cd $R && cargo fmt --all || exit 1
python3 $S/tools/other_edits.py $R $S/art/c$N-other-edits-check.txt \
  "parse_template.rs=instruments" "version.rs=instruments" "lib.rs=insert" "parse_template/tests.rs=insert" \
  "dispatch/rexx_info.rs=substsort:\\bparse_template\\b=>version" \
  "trace.rs=subst:crate::parse_template::PLATFORM=>crate::version::PLATFORM" \
  "environment.rs=subst:crate::parse_template::LINE_END=>crate::version::LINE_END"
