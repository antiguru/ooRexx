#!/bin/bash
S=/tmp/claude-1000/p6-s5-perfbase
cd /home/moritz/dev/repos/ooRexx-rust-rewrite
bash rust/bench-programs/callgrind.sh -r 3 -j 16 -o $S/cg base=$S/bin/base/rexx-run s1=$S/bin/s1/rexx-run pad1=$S/bin/pad1/rexx-run pad2=$S/bin/pad2/rexx-run pad3=$S/bin/pad3/rexx-run > $S/cg-table.txt 2> $S/cg-err.txt
echo "exit $?" > $S/cg-exit.txt
