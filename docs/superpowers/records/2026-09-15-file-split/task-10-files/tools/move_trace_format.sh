#!/bin/bash
# The trace.rs commit, up to the checks: pre<N>, the block from TraceMode
# through push_operator moved verbatim into trace/format.rs, `mod format;`
# declared with the re-exports that keep every `crate::trace::` path working
# and the plain imports trace.rs's own code needs, the explicit
# `use super::{..}` list, the two associated consts trace.rs's tests read
# widened to pub(super), rustfmt, and the check of every edit outside the move
# (tests/support/mod.rs's prose naming the file the formatters are in).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=$1
$S/tools/prep.sh $N || exit 1
cd $R/crates/rexx-exec/src
L=$(python3 $S/tools/span_of.py trace.rs "struct TraceMode" "fn push_operator") || exit 1
mkdir -p trace
python3 $S/tools/move_lib_child.py trace.rs trace/format.rs $S/c$N/removed.json x $L | tee $S/c$N/move.txt || exit 1
echo "block $L" >> $S/c$N/move.txt
python3 - <<'PY' || exit 1
p = 'trace.rs'; s = open(p).read()
a = "use rexx_parse::{Operator, PrefixOp, Trace};\n"; assert s.count(a) == 1
s = s.replace(a, a + "\n// The mode a `TRACE` setting parses into, and the byte layout of each trace line.\nmod format;\n")
open(p, 'w').write(s)
p = 'trace/format.rs'; s = open(p).read()
assert s.count("\n//! x\n") == 1
s = s.replace("\n//! x\n", "\n//! The mode a `TRACE` setting parses into, and the byte layout of each\n//! prefix's line: pure functions on bytes, which `trace.rs`'s emission calls.\n", 1)
for name, ty in (("LABELS", "TraceMode"), ("ALL", "TraceMode")):
    old = f"\n    const {name}: {ty} = {ty} {{\n"; assert s.count(old) == 1, name
    s = s.replace(old, f"\n    pub(super) const {name}: {ty} = {ty} {{\n")
open(p, 'w').write(s)
p = '../tests/support/mod.rs'; s = open(p).read()
# Its three mentions of trace.rs name push_clause, push_prefixed_blanks,
# push_quoted, push_quoted_tag and the formatters, which all move.
assert s.count("trace.rs") == 3
open(p, 'w').write(s.replace("trace.rs", "trace/format.rs"))
PY
python3 $S/tools/set_imports.py trace/format.rs "use super::{Number, Raised, Trace};" || exit 1
cd $R && SPLIT_PARENT=trace python3 $S/tools/apply_imports.py crates/rexx-exec/src format "" \
  "make_displayable push_operator push_tagged push_value" \
  "ChunkTrace TraceCache TraceEvent TraceMode TraceRequest applied is_whole_number literal_trace_event mode_from_setting parse_trace_request push_clause raised_invalid_trace_letter raised_numeric_trace_interactive_only" "" || exit 1
cargo fmt --all || exit 1
python3 $S/tools/other_edits.py $R $S/art/c$N-other-edits-check.txt "trace.rs=instruments" "trace/format.rs=instruments" \
  "crates/rexx-exec/tests/support/mod.rs=subst:\\btrace\\.rs=>trace/format.rs"
