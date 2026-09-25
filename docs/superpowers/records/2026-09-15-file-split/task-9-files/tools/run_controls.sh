#!/bin/bash
# usage: run_controls.sh OUTDIR SUFFIX
# Re-runs Task 2's five controls (its c1 pair), Task 3b's four (its c8 pair)
# and Task 4's eight (its c1, c3, c5 and seal pairs) with this task's tools,
# on trees extracted with `git archive`, plus this task's own controls
# (controls_task5.py) once its pairs exist. One output file per set.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9
M=/home/moritz/dev/repos/ooRexx-rust-rewrite
R=$M/docs/superpowers/records/2026-09-15-file-split
O=$1; X=$2; W=$S/ctl-work; mkdir -p $O $W
tree() { [ -d $S/trees/$1 ] || { mkdir -p $S/trees/$1; git -C $M archive $1 rust/crates/rexx-exec/src | tar -x -C $S/trees/$1; }; echo $S/trees/$1/rust/crates/rexx-exec/src; }
python3 $S/tools/controls.py $(tree d6dd60d90) $(tree afa268640) $R/task-2-files/c1/removed.json $W/t2 > $O/controls-task2-$X.txt 2>&1; echo "task2 exit $? $(tail -1 $O/controls-task2-$X.txt)"
python3 $S/tools/controls_run.py $(tree d37641150) $(tree 5c5380b79) $R/task-3b-files/c8/removed.json $W/run > $O/controls-run-$X.txt 2>&1; echo "run exit $? $(tail -1 $O/controls-run-$X.txt)"
: > $O/controls-lib-$X.txt
for c in "c1 2f3065b2b b4f42bbfc c1" "c3 3fc6cbd71 bdef7b8b8 c3" "c5 377d0ac3d 97ad8c334 c5" "seal b4f42bbfc 3fc6cbd71 c2"; do set -- $c
  python3 $S/tools/controls_lib.py $1 $(tree $2) $(tree $3) $R/task-4-files/$4/removed.json $W/lib >> $O/controls-lib-$X.txt 2>&1; echo "lib $1 exit $?"
done
echo "lib total caught: $(/bin/grep -a -c ': CAUGHT$' $O/controls-lib-$X.txt), missed: $(/bin/grep -a -c ': MISSED$' $O/controls-lib-$X.txt)"
if [ -f $S/tools/controls_task5.py ]; then python3 $S/tools/controls_task5.py > $O/controls-task5-$X.txt 2>&1; echo "task5 exit $? $(tail -1 $O/controls-task5-$X.txt)"; fi
