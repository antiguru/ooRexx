#!/bin/bash
# crate.sh BIN PROG OUTDIR
B=$1; P=$2; O=$3; mkdir -p $O; R=$(mktemp -d /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-4/run.XXXX)
cd $R
( ulimit -v 4194304; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib:/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-4/forge/lib timeout 20 $B $P ) >$O/$(basename $P).out 2>$O/$(basename $P).err
echo $? > $O/$(basename $P).rc
cd / && rmdir $R
