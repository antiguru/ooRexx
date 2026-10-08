#!/bin/bash
# usage: compare.sh ORACLE_OUT CRATE_OUT
# Prints the number of cells, then each context with the number of its cells
# that differ from the oracle on stdout, stderr or status.
o=$1 c=$2
ls "$o"/*.rc | wc -l
for f in "$o"/*.rc; do
  n=$(basename "$f" .rc)
  for e in out err rc; do cmp -s "$o/$n.$e" "$c/$n.$e" || { echo "$n"; break; }; done
done | sed 's/.*__//' | sort | uniq -c
