#!/bin/bash
# cmp.sh PROG... : run each on the oracle and the crate from fresh empty dirs; report three-descriptor agreement
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-5
B=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run
for P in "$@"; do
  P=$(readlink -f $P); n=$(basename $P .rex)
  for side in oracle crate; do
    R=$(mktemp -d $S/run.XXXX); cd $R
    if [ $side = oracle ]; then
      ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib:/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-5/forge timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $P ) >$S/out/$n.$side.out 2>$S/out/$n.$side.err
    else
      ( ulimit -v 4194304; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib:/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-5/forge timeout 20 $B $P ) >$S/out/$n.$side.out 2>$S/out/$n.$side.err
    fi
    echo $? > $S/out/$n.$side.rc
    cd /; rm -r $R
  done
  ok=1
  for d in out err rc; do cmp -s $S/out/$n.oracle.$d $S/out/$n.crate.$d || { ok=0; echo "$n: $d differs"; }; done
  [ $ok = 1 ] && echo "$n: identical (rc $(cat $S/out/$n.oracle.rc))"
done
