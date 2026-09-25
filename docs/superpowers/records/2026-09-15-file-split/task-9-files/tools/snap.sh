#!/bin/bash
# usage: snap.sh save DIR | restore DIR
# save: copies every file under rust/ that differs from HEAD (modified or
# untracked) into DIR, with the list in DIR/files; restore: deletes the files
# the current working tree has untracked under rust/, puts HEAD's content
# back on every modified one, then copies DIR's files over.
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
cd $M
case $1 in
save)
  mkdir -p $2
  { git diff --name-only HEAD -- rust; git ls-files --others --exclude-standard -- rust; } | sort -u > $2/files
  while read f; do mkdir -p $2/tree/$(dirname $f); cp $f $2/tree/$f; done < $2/files
  cat $2/files ;;
restore)
  git ls-files --others --exclude-standard -- rust | while read f; do rm -- "$f"; done
  git diff --name-only HEAD -- rust | while read f; do git show HEAD:$f > $f; done
  while read f; do mkdir -p $(dirname $f); cp $2/tree/$f $f; done < $2/files
  git status --short ;;
esac
