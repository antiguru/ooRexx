#!/bin/bash
# derive-replay.sh LOG OUT : the replay table from the oracle's log-mode run
# (runs/log-oracle.tsv: line, argument, dataType("W"), answer). One row per
# "line|argument", then one row per call site whose oracle calls all
# answered alike, keyed by the line alone.
set -eu
log=$1
out=$2
awk -F'\t' '{print $1"|"$2"\t"$3"\t"$4}' "$log" | sort -u > "$out"
cut -f1,3,4 "$log" | sort -u | cut -f1 | uniq -c | awk '$1==1{print $2}' > "$out.sites"
cut -f1,3,4 "$log" | sort -u | awk -F'\t' 'NR==FNR{u[$1]=1; next} ($1 in u){print}' "$out.sites" - >> "$out"
rm -f "$out.sites"
