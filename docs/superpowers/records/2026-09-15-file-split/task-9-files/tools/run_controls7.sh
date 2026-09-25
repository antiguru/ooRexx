#!/bin/bash
# usage: run_controls7.sh OUTDIR SUFFIX
# Every earlier task's controls with this task's tooling: run_controls.sh
# (Task 2's five, Task 3b's four, Task 4's eight, Task 5's seven), Task 6's
# eight (controls_task6.py), Task 5's and Task 6's other-edits controls, and
# this task's own (controls_task7.py) once it exists.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
O=$1; X=$2; mkdir -p $O
$S/tools/run_controls.sh $O $X
python3 $S/tools/controls_task6.py > $O/controls-task6-$X.txt 2>&1; echo "task6 exit $? $(tail -1 $O/controls-task6-$X.txt)"
$S/tools/controls_other_edits.sh > $O/controls-other-edits-$X.txt 2>&1; echo "other-edits (task5) $(/bin/grep -a -c 'expected' $O/controls-other-edits-$X.txt) runs:"; sed 's/^/  /' $O/controls-other-edits-$X.txt
$S/tools/controls_other_edits6.sh > $O/controls-other-edits6-$X.txt 2>&1; echo "other-edits (task6):"; sed 's/^/  /' $O/controls-other-edits6-$X.txt
if [ -f $S/tools/controls_task7.py ]; then python3 $S/tools/controls_task7.py > $O/controls-task7-$X.txt 2>&1; echo "task7 exit $? $(tail -1 $O/controls-task7-$X.txt)"; fi
