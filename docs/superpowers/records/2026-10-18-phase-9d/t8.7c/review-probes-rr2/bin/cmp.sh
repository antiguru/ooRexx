#!/bin/bash
# usage: cmp.sh FILE.rex  -> o~b, o~h differences after r.sh
f=$(readlink -f "$1"); d=$(dirname "$f")/out; n=$(basename "$f" .rex)
for w in ${2:-b h}; do s=""; for x in out err rc; do cmp -s $d/$n.o.$x $d/$n.$w.$x || s="$s $x"; done; echo "$n o~$w differ:${s:- none}"; done
