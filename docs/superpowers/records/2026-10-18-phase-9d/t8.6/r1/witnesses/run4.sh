#!/bin/bash
# run3.sh PROBE.rex [pipe|file]: oracle, base, prior head and new head, each in a fresh dir
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/r1fix
P=$(readlink -f "$1"); name=$(basename "$P" .rex); mode=${2:-pipe}
mkdir -p $S/runs
for who in oracle base prior head; do
  d=$S/runs/$name.$who; rm -rf "$d"; mkdir -p "$d"; cp "$P" "$d/p.rex"
  pushd "$d" >/dev/null
  case $who in
    oracle) cmd=( env LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx p.rex );;
    base) cmd=( memcap 2G timeout -k 5 20 /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.6/rexx-base-0fd10a074 p.rex );;
    prior) cmd=( memcap 2G timeout -k 5 20 /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.6/rexx-head-b53b1de0c p.rex );;
    head) cmd=( memcap 2G timeout -k 5 20 /tmp/claude-1000/t86-target/release/rexx p.rex );;
  esac
  if [ "$mode" = file ]; then
    ( if [ $who = oracle ]; then ulimit -v 1048576; fi; "${cmd[@]}" > ../$name.$who.out 2> ../$name.$who.err < /dev/null ); echo $? > ../$name.$who.rc
  else
    ( if [ $who = oracle ]; then ulimit -v 1048576; fi; "${cmd[@]}" 2> ../$name.$who.err < /dev/null | cat > ../$name.$who.out; exit ${PIPESTATUS[0]} ); echo $? > ../$name.$who.rc
  fi
  popd >/dev/null
done
cd $S/runs
for who in oracle base prior head; do echo "== $who rc=$(cat $name.$who.rc) out: $(cat -v $name.$who.out | tr '\n' ' ') err: $(cat -v $name.$who.err | tr '\n' ' ')"; done
