#!/bin/bash
# usage: docapi.sh RUST_DIR TARGET OUT
# The public API rustdoc documents: `cargo doc --no-deps -p rexx-parse` into
# a doc directory emptied first (so no page of another run is left over), then
# one line per generated file outside `src/`, `src-files.js`,
# `static.files/` and `search.index/`:
#  * a page: the sha256 of its content once every link to a source page
#    (`href="...src/rexx_parse/..."`, which a move shifts by design) is
#    replaced by `SRC`;
#  * a redirect stub (`http-equiv="refresh"`), which rustdoc writes at an
#    item's defining module when the item is re-exported from a private one:
#    `STUB -> <target>` without the stub's own path, since a move changes the
#    defining module by design and the target is the public page;
#  * a `.js` index: its sha256 once a defining path through private modules
#    (`rexx_parse::token::symbols::SymbolId`) is collapsed to
#    `rexx_parse::~::SymbolId`, and its `fragment_lengths` (the byte
#    lengths of those strings) dropped.
# `search.index/` is left out: it records each item's defining module too, in
# a length-prefixed encoding a text substitution cannot normalise; every item
# it indexes has its page above. Two trees with the same manifest document the
# same public items, paths, signatures and docs.
cd $1
rm -rf $2/doc
CARGO_TARGET_DIR=$2 cargo doc -j 8 --no-deps -p rexx-parse > $3.log 2>&1; echo "exit $?" >> $3.log
{
  echo "doc: $(tail -1 $3.log); $(/bin/grep -a -c '^warning' $3.log) warning lines"
  (cd $2/doc && find . -type f ! -path './src/*' ! -path './static.files/*' ! -path './search.index/*' ! -name 'src-files.js' | while read f; do
    if /bin/grep -q 'http-equiv="refresh"' "$f"; then
      echo "STUB -> $(sed -n 's#.*URL=\([^"]*\)".*#\1#p' "$f" | sed 's#^\(\.\./\)*##')"
    elif [ "${f%.js}" != "$f" ]; then
      echo "$(sed -E 's#rexx_parse(::[a-z_]+)+::([A-Z])#rexx_parse::~::\2#g; s#"fragment_lengths":\[[0-9,]*\]#"fragment_lengths":[~]#g' "$f" | sha256sum | cut -c1-16) $f"
    else
      echo "$(sed -E 's#href="[^"]*src/rexx_parse/[^"]*"#href="SRC"#g' "$f" | sha256sum | cut -c1-16) $f"
    fi
  done | sort -k2)
} > $3
rm $3.log
echo "docapi: $(head -1 $3); $(($(wc -l < $3) - 1)) files, $(/bin/grep -a -c '^STUB' $3) of them redirect stubs"
