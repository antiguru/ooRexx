# Task 9 probes

Run with the Task 5 forge's `compare.sh` (`../task-5-forge/compare.sh`, its `S` set to a scratch
directory) after building both forges into `$S/forge`: `../task-5-forge/build.sh $S/forge` and
`./build.sh $S/forge`. `outer9.rex` and `stale.rex` load `libouter9.so`; `ov.rex` loads the oracle's
`orxmethod`; the rest load nothing or `rxmath`. Results at `2ae06085c` are in
`docs/superpowers/plans/phase-4-exclusions.txt` and `task-9-report.md`.
