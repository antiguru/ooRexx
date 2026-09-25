#!/bin/bash
# usage: docapi10.sh RUST_DIR TARGET OUT CRATE
# docapi.sh (Task 9, whose header says what the manifest holds and leaves
# out) with the crate a parameter: `rexx-extract` here, whose public API
# `docs::classes` is, so a manifest identical to BASE's says every public
# item, path, signature and doc of it is unchanged.
C=$4; CU=${C//-/_}
cd $1
rm -rf $2/doc
CARGO_TARGET_DIR=$2 cargo doc -j 8 --no-deps -p $C > $3.log 2>&1; echo "exit $?" >> $3.log
{
  echo "doc: $(tail -1 $3.log); $(/bin/grep -a -c '^warning' $3.log) warning lines"
  (cd $2/doc && find . -type f ! -path './src/*' ! -path './static.files/*' ! -path './search.index/*' ! -name 'src-files.js' | while read f; do
    if /bin/grep -q 'http-equiv="refresh"' "$f"; then
      echo "STUB -> $(sed -n 's#.*URL=\([^"]*\)".*#\1#p' "$f" | sed 's#^\(\.\./\)*##')"
    elif [ "${f%.js}" != "$f" ]; then
      echo "$(sed -E "s#$CU(::[a-z_]+)+::([A-Z])#$CU::~::\2#g; s#\"fragment_lengths\":\[[0-9,]*\]#\"fragment_lengths\":[~]#g" "$f" | sha256sum | cut -c1-16) $f"
    else
      echo "$(sed -E "s#href=\"[^\"]*src/$CU/[^\"]*\"#href=\"SRC\"#g" "$f" | sha256sum | cut -c1-16) $f"
    fi
  done | sort -k2)
} > $3
rm $3.log
echo "docapi: $(head -1 $3); $(($(wc -l < $3) - 1)) files, $(/bin/grep -a -c '^STUB' $3) of them redirect stubs"
