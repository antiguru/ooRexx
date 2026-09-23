#!/bin/bash
G=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round5/spills/gates
for f in test-release test-debug; do
    echo "$f: binaries=$(/bin/grep -a -c '^test result' "$G/$f.log") $(/bin/grep -a '^test result' "$G/$f.log" | awk '{p+=$4; fl+=$6; i+=$8} END {print "passed="p, "failed="fl, "ignored="i}')"
    /bin/grep -a -E "[0-9]+ of 604 matching|STRICT" "$G/$f.log" | head -3
done
