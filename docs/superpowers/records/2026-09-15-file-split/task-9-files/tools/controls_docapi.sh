#!/bin/bash
# usage: controls_docapi.sh
# Controls for docapi.sh, on a copy of BASE's whole tree (git archive): each
# plant is made alone, the manifest compared with BASE's
# (art/base/base-docapi.txt), and the copy restored after it.
#   C1 a public method's doc text changes           -> manifest differs
#   C2 a public method narrows to pub(crate)        -> manifest differs
#   C3 a public item leaves the crate root's re-exports -> manifest differs
#   C4 lines shift only (a comment line inserted above every item of
#      token.rs, so every source link moves)        -> manifest the same
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
W=$S/trees/docapi-ctl; rm -rf $W; cp -r $S/trees/full-72f2bc3d8 $W
P=$W/rust/crates/rexx-parse/src
run() { # name expect(differs|same) file old new
  cp $P/$3 $S/docapi-ctl.orig
  python3 -c "import sys; p,o,n=sys.argv[1:4]; s=open(p).read(); assert s.count(o)==1,(p,o); open(p,'w').write(s.replace(o,n))" $P/$3 "$4" "$5" || { echo "$1: PLANT FAILED"; return; }
  $S/tools/docapi.sh $W/rust $S/t-check $S/art/docapi-ctl-$1.txt > /dev/null
  if diff -q $S/art/base/base-docapi.txt $S/art/docapi-ctl-$1.txt > /dev/null; then got=same; else got=differs; fi
  echo "$1: expected $2, manifest $got: $([ $got = $2 ] && echo CAUGHT || echo MISSED) ($(diff $S/art/base/base-docapi.txt $S/art/docapi-ctl-$1.txt | /bin/grep -a -c '^[<>]') lines differ; $(head -1 $S/art/docapi-ctl-$1.txt))"
  cp $S/docapi-ctl.orig $P/$3; rm $S/docapi-ctl.orig
}
run C1 differs token.rs "    /// How many distinct symbols are interned." "    /// How many distinct symbols are interned so far."
run C2 differs token.rs "    pub fn is_empty(&self) -> bool {
        self.names.is_empty()" "    pub(crate) fn is_empty(&self) -> bool {
        self.names.is_empty()"
run C3 differs lib.rs "SymbolClass, SymbolId, SymbolTable, Tag, Token," "SymbolClass, SymbolId, SymbolTable, Token,"
run C4 same token.rs "
use std::borrow::Cow;" "
// planted
use std::borrow::Cow;"
rm -rf $W
