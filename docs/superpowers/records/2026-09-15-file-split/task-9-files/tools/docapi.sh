#!/bin/bash
# usage: docapi.sh RUST_DIR TARGET OUT
# The public API rustdoc documents: `cargo doc --no-deps -p rexx-parse` into
# a doc directory emptied first (so no page of another run is left over), then
# one line per generated file outside `src/`, `src-files.js` and
# `static.files/`, with the sha256 of its content once every link to a
# source page (`href="...src/rexx_parse/..."`, which a move shifts by design)
# is replaced by `SRC`. Two trees with the same manifest document the same
# public items, paths, signatures and docs.
cd $1
rm -rf $2/doc
CARGO_TARGET_DIR=$2 cargo doc -j 8 --no-deps -p rexx-parse > $3.log 2>&1; echo "exit $?" >> $3.log
{
  echo "doc: $(tail -1 $3.log); $(/bin/grep -a -c '^warning' $3.log) warning lines"
  (cd $2/doc && find . -type f ! -path './src/*' ! -path './static.files/*' ! -name 'src-files.js' | sort | while read f; do
    echo "$(sed -E 's#href="[^"]*src/rexx_parse/[^"]*"#href="SRC"#g' "$f" | sha256sum | cut -c1-16) $f"
  done)
} > $3
rm $3.log
echo "docapi: $(head -1 $3); $(($(wc -l < $3) - 1)) files"
