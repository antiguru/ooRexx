#!/bin/bash
# usage: controls_checkout10.sh PARENT_RS DESTS
# Controls for the Task 10 tools that read the main checkout itself, run on
# a clean tree at HEAD: each plant is made alone and removed by explicit
# path (or its file restored from a copy) before the next.
#   O0 clean tree                                        -> other edits 0, untracked 0
#   O1 an untracked .rs under tests/table_c/, not a destination -> untracked 1
#   O2 an untracked tool output beside the crate's tests/ -> untracked 1
#   O3 a tracked edit to another test file (gate_table_d.rs) -> other edits 1
#   O4 clean again                                       -> 0 and 0
#   D1 a broken intra-doc link in the moved code (table_c/rows.rs)
#      -> testdoc10.sh reports a warning (BASE: none)
#   D2 unplanted -> none
#   A1 a public doc line of rexx-extract's docs/classes.rs changes -> manifest differs
#   A2 a public fn of it narrows to pub(crate)            -> manifest differs
#   A3 lines shift only (a comment inserted)              -> manifest the same
#   E1 corpus/gate-tables/README.md edited exactly as a declared rule says -> extra_edits10 passes
#   E2 the same, and a second word of the file changed    -> fails
#   E3 a rule whose pattern matches twice                 -> fails
#   E4 a declared rule with the file left as HEAD has it  -> fails
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
T=$R/crates/rexx-exec/tests
[ -z "$(git -C $R status --short -- .)" ] || { echo "tree under rust/ not clean"; exit 1; }
chk() { SPLIT_CRATE=rexx-exec SPLIT_ROOT=tests $S/tools/other_edits10.sh ctl $1 $2 > $S/art/ctl-oe.out; tr '\n' ';' < $S/art/ctl-oe.out; /bin/grep -q '^untracked: 0 files' $S/art/ctl-oe.out && /bin/grep -q '^other edits: 0 files' $S/art/ctl-oe.out && echo " verify10 would PASS" || echo " verify10 would FAIL"; }
echo "O0 clean tree (expected PASS): $(chk $1 $2)"
P=$T/table_c/planted.rs; echo '// planted' > $P
echo "O1 untracked tests/table_c/planted.rs (expected FAIL): $(chk $1 $2)"; rm $P
P=$R/crates/rexx-exec/tests-out-verdict.txt; echo planted > $P
echo "O2 untracked crates/rexx-exec/tests-out-verdict.txt (expected FAIL): $(chk $1 $2)"; rm $P
cp $T/gate_table_d.rs $S/ctl-gtd.orig; echo '// planted' >> $T/gate_table_d.rs
echo "O3 tracked edit to tests/gate_table_d.rs (expected FAIL): $(chk $1 $2)"; cp $S/ctl-gtd.orig $T/gate_table_d.rs; rm $S/ctl-gtd.orig
echo "O4 after the plants (expected PASS): $(chk $1 $2)"
rm -f $S/art/ctl-oe.out $S/art/cctl-other-edits.diff
cp $T/table_c/rows.rs $S/ctl-rows.orig
python3 -c "import sys; p=sys.argv[1]; s=open(p).read(); o='/// Reads one committed row file as tab-separated fields.\n'; assert s.count(o)==1; open(p,'w').write(s.replace(o, o + '/// Planted: see [\`no_such_item\`].\n'))" $T/table_c/rows.rs
$S/tools/testdoc10.sh $R $S/t-check $S/art/ctl-testdoc-D1.txt > /dev/null
cp $S/ctl-rows.orig $T/table_c/rows.rs; rm $S/ctl-rows.orig
echo "D1 a broken intra-doc link in table_c/rows.rs (expected a warning): $(head -2 $S/art/ctl-testdoc-D1.txt | tr '\n' ' ') $(/bin/grep -a -c 'no_such_item' $S/art/ctl-testdoc-D1.txt) lines name it"
$S/tools/testdoc10.sh $R $S/t-check $S/art/ctl-testdoc-D2.txt > /dev/null
echo "D2 unplanted (expected none): $(head -2 $S/art/ctl-testdoc-D2.txt | tr '\n' ' ')"
C=$R/crates/rexx-extract/src/docs/classes.rs
run() { # name expect(differs|same) old new
  cp $C $S/ctl-classes.orig
  python3 -c "import sys; p,o,n=sys.argv[1:4]; s=open(p).read(); assert s.count(o)==1,(p,o); open(p,'w').write(s.replace(o,n))" $C "$3" "$4" || { echo "$1: PLANT FAILED"; return; }
  $S/tools/docapi10.sh $R $S/t-check $S/art/ctl-docapi-$1.txt rexx-extract > /dev/null
  cp $S/ctl-classes.orig $C; rm $S/ctl-classes.orig
  if diff -q $S/art/base/base-docapi-rexx-extract.txt $S/art/ctl-docapi-$1.txt > /dev/null; then got=same; else got=differs; fi
  echo "$1: expected $2, manifest $got: $([ $got = $2 ] && echo CAUGHT || echo MISSED) ($(diff $S/art/base/base-docapi-rexx-extract.txt $S/art/ctl-docapi-$1.txt | /bin/grep -a -c '^[<>]') lines differ; $(head -1 $S/art/ctl-docapi-$1.txt))"
}
run A1 differs "/// The inverse of [\`render_method\`], for a harness reading the committed file." "/// The inverse of [\`render_method\`], for a harness reading a committed file."
run A2 differs "pub fn parse_method(field: &str) -> String {" "pub(crate) fn parse_method(field: &str) -> String {"
run A3 same "
/// Every \`cls*\` section across the books" "
// planted
/// Every \`cls*\` section across the books"
G=$R/corpus/gate-tables/README.md
ex() { # name expect(PASS|FAIL) rule [second-edit]
  cp $G $S/ctl-readme.orig
  [ -n "$4" ] && python3 -c "import sys; p=sys.argv[1]; s=open(p).read(); s2=s.replace(sys.argv[2], sys.argv[3], 1); assert s2 != s; open(p,'w').write(s2)" $G "$4" "$5"
  python3 $S/tools/extra_edits10.py $R $S/art/ctl-extra-$1.txt "$3" > /dev/null; rc=$?
  cp $S/ctl-readme.orig $G; rm $S/ctl-readme.orig
  got=$([ $rc = 0 ] && echo PASS || echo FAIL)
  echo "$1: expected $2, got $got: $([ $got = $2 ] && echo CAUGHT || echo MISSED) ($(head -1 $S/art/ctl-extra-$1.txt | cut -c1-200))"
}
R1='corpus/gate-tables/README.md=`tests/table_c/probes\.rs`=>`tests/table_c/checks.rs`'
ex E1 PASS "$R1" '`tests/table_c/probes.rs`' '`tests/table_c/checks.rs`'
cp $G $S/ctl-readme2.orig; python3 -c "import sys; p=sys.argv[1]; s=open(p).read(); s2=s.replace('**Those last three are derived', '**Those three are derived', 1); assert s2 != s; open(p,'w').write(s2)" $G
ex E2 FAIL "$R1" '`tests/table_c/probes.rs`' '`tests/table_c/checks.rs`'
cp $S/ctl-readme2.orig $G; rm $S/ctl-readme2.orig
ex E3 FAIL 'corpus/gate-tables/README.md=probe=>Probe' 'probe' 'Probe'
ex E4 FAIL "$R1"
echo "tree afterwards: [$(git -C $R status --short -- . | tr '\n' ' ')] (expected empty)"
