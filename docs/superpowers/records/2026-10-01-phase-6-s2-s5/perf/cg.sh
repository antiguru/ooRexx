#!/bin/bash
S=/tmp/claude-1000/p6-t26
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
bash rust/bench-programs/callgrind.sh -r 3 -j 16 -o $S/cg base=$S/bin/base/rexx-run s1=$S/bin/s1/rexx-run head=$S/bin/head/rexx-run > $S/cg-table.txt 2> $S/cg-err.txt
echo "exit $?" > $S/cg-exit.txt
