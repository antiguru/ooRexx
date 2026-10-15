# Task 8b loop witnesses

Peak RSS in kB from `/usr/bin/time -f %M` on the binary itself, through
`docs/superpowers/records/2026-10-14-phase-9b1/task-8/memory/memloop.sh BIN PROGRAM N...`, run from a
fresh empty directory. The crate side runs under `memcap 2G timeout -k 5 120`, the oracle under
`ulimit -v 1048576` with `LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib`. Binaries are kept
in `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/t8b/`: `rexx-run-base` is `efbf0fa1d`, `rexx-run-step1`
is the Step 1 commit.

## Step 1: MethodId rows

Programs: `task-8/memory/copy.rex`, `copypriv.rex`, `nativecopy.rex`, and
`.superpowers/sdd/2026-10-14-phase-9b1/task-8-review-files/methnewpriv.rex`.

| program | n | oracle | base | step1 |
|---|---|---|---|---|
| copy | 20000 / 80000 / 320000 | 12692 / 18976 / 20288 | 54764 / 55812 / 55644 | 54276 / 55416 / 55884 |
| copypriv | 20000 / 80000 / 320000 | 12728 / 19536 / 20188 | 55328 / 58796 / 71712 | 55584 / 56560 / 57004 |
| nativecopy | 20000 / 80000 / 320000 | 12188 / 19536 / 20572 | 54292 / 57404 / 68868 | 54436 / 55340 / 55532 |
| methnewpriv | 20000 / 80000 / 320000 | 20128 / 20384 / 20324 | 46104 / 48608 / 55400 | 45268 / 44912 / 45240 |

Step 1 is within 10% from 20000 to 320000 on every program. On base, `copypriv` 20000 to 80000 is
+6.3% and 320000 is +29.6%, which is why the table carries a third count.
