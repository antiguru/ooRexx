#!/bin/bash
# oracle.sh PROG OUTDIR : run under the standard wrapper from a fresh empty dir
P=$1; O=$2; mkdir -p $O; R=$(mktemp -d /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-4/run.XXXX)
cd $R
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib:/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-4/forge/lib timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $P ) >$O/$(basename $P).out 2>$O/$(basename $P).err
echo $? > $O/$(basename $P).rc
cd / && rmdir $R
