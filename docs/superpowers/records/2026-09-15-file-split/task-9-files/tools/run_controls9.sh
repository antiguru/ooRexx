#!/bin/bash
# usage: run_controls9.sh OUTDIR SUFFIX
# Every earlier task's controls with this task's tooling (run_controls8.sh:
# Task 2's five, Task 3b's four, Task 4's eight, Task 5's seven, Task 6's
# eight, Task 5's and Task 6's other-edits controls, Task 7's ten and Task
# 8's fifteen), and this task's own (controls_task9.py).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
O=$1; X=$2; mkdir -p $O
$S/tools/run_controls8.sh $O $X
python3 $S/tools/controls_task9.py > $O/controls-task9-$X.txt 2>&1; echo "task9 exit $? $(tail -1 $O/controls-task9-$X.txt)"
