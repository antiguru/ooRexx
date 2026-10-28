#!/bin/bash
# run3.sh PROBE.rex [stdout-mode]  -- runs oracle, base, head each in a fresh dir; stdout piped through cat unless mode=file
S=/home/moritz/dev/repos/ooRexx-rust-rewrite/.superpowers/sdd/2026-10-18-phase-9d/t86-review-scratch
P=$(readlink -f "$1"); name=$(basename "$P" .rex); mode=${2:-pipe}
for who in oracle base head; do
  d=$S/runs/$name.$who; rm -rf "$d"; mkdir -p "$d"; cp "$P" "$d/p.rex"
  pushd "$d" >/dev/null
  case $who in
    oracle) cmd=( env LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx p.rex );;
    base) cmd=( memcap 2G timeout -k 5 20 /tmp/claude-1000/t86-review-target-base/release/rexx p.rex );;
    head) cmd=( memcap 2G timeout -k 5 20 /tmp/claude-1000/t86-review-target-head/release/rexx p.rex );;
  esac
  if [ "$mode" = file ]; then
    ( if [ $who = oracle ]; then ulimit -v 1048576; fi; "${cmd[@]}" > ../$name.$who.out 2> ../$name.$who.err < /dev/null ); echo $? > ../$name.$who.rc
  else
    ( if [ $who = oracle ]; then ulimit -v 1048576; fi; "${cmd[@]}" 2> ../$name.$who.err < /dev/null | cat > ../$name.$who.out; exit ${PIPESTATUS[0]} ); echo $? > ../$name.$who.rc
  fi
  popd >/dev/null
done
cd $S/runs
for who in oracle base head; do echo "== $who rc=$(cat $name.$who.rc)"; echo "-- out"; cat -v $name.$who.out; echo "-- err"; cat -v $name.$who.err; done
for x in out err rc; do cmp -s $name.oracle.$x $name.head.$x && echo "head==oracle $x" || echo "head!=oracle $x"; done
