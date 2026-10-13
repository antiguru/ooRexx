#!/bin/bash
# usage: simrun.sh DIR FILE EXPECTED_STDOUT -- runs the crate under several simulation policies
cd "$1" || exit 99
for mode in sim:1 sim:2,uniform:0.2 sim:3,uniform:1 sim:4,pct:3,k=12 sim:5,pre:2,k=12 sim:6,uniform:0.5,gc=0.5 sim:7,pre:1,k=12 sim:8,pct:2,k=12; do
  REXX_SWITCH_MODE=$mode memcap 4G timeout -k 5 60 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run "$2" >"$1/s.out" 2>"$1/s.err" </dev/null
  rc=$?
  if cmp -s "$1/s.out" "$3"; then same=same; else same=DIFFERS; fi
  echo "$mode rc=$rc stdout $same | $(grep '^rexx-sim' "$1/s.err" | cut -d' ' -f2-6)"
done
