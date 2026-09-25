#!/bin/bash
# usage: headers_check.sh N FILE...   (FILE relative to src)
# The native-table headers of the parent commit's tree (pre<N>) and of the
# working tree, for the files named, and the command that listed them.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src
N=$1; shift
{
  echo "command: python3 tools/table_headers.py FILE... over pre$N (the parent commit) and over the working tree"
  echo "== before"
  (cd $S/pre$N && for f in "$@"; do [ -f $f ] && python3 $S/tools/table_headers.py $f; done)
  echo "== after"
  (cd $R && for f in "$@"; do [ -f $f ] && python3 $S/tools/table_headers.py $f; done)
} > $S/art/c$N-table-headers.txt
cat $S/art/c$N-table-headers.txt
