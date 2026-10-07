#!/bin/bash
# usage: prof.sh OUTDIR NAME=BIN... ; profiles sendloop dispatch fibfunc, 4 at a time
S=/tmp/claude-1000/p6-t26
B=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/bench-programs
out=$1; shift
mkdir -p $out
for arg in "$@"; do
  n=${arg%%=*}; bin=${arg#*=}
  for p in ${PROGS:-sendloop dispatch fibfunc}; do
    d=$(mktemp -d -p $S)
    ( cd $d && LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib valgrind --tool=callgrind --dump-instr=no --compress-strings=no --log-file=$out/$p.$n.vg --callgrind-out-file=$out/$p.$n.cg $bin $B/$p.rex > $out/$p.$n.out 2>&1; echo $? > $out/$p.$n.rc ) &
    while [ $(jobs -r | wc -l) -ge 4 ]; do sleep 2; done
  done
done
wait
for f in $out/*.cg; do callgrind_annotate --auto=no --inclusive=no $f > ${f%.cg}.self.txt; callgrind_annotate --auto=no --inclusive=yes $f > ${f%.cg}.incl.txt; done
echo done
