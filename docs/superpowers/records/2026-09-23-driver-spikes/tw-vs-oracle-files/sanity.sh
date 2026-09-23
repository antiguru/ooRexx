#!/bin/bash
# Runs every program once on each engine without valgrind and compares stdout.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
P=$S/progs
D=$S/sanity.$$
mkdir "$D" || exit 1
cd "$D" || exit 1
for f in "$P"/*.rex; do
    n=$(basename "$f" .rex)
    o=$( ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx "$f" ) 2>"$D/$n.oerr"); orc=$?
    t=$(REXX_ENGINE=tree-walker "$S/bin/tw-rexx-run" "$f" 2>"$D/$n.terr"); trc=$?
    i=$(REXX_ENGINE=ir "$S/bin/tw-rexx-run" "$f" 2>"$D/$n.ierr"); irc=$?
    echo "$n oracle=[$o]$orc tw=[$t]$trc ir=[$i]$irc"
done
