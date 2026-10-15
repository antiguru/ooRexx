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

## Step 2: retired behaviours

Programs: `task-8/memory/define.rex` (`.k~define('BAR', m)`), `defineobj.rex` (`.k~delete('BAR')`),
and `hide.rex`, which is `do i = 1 to n; .k~define('BAR'); end` on an empty `::class k`. The crate
binary `rexx-run-step2` is the Step 2 commit.

| program | n | oracle | base | step2 |
|---|---|---|---|---|
| define | 5000 / 20000 / 80000 / 320000 | 20352 / 20668 / 20308 / 20300 | 57068 / 170352 / 628364 / - | 57224 / 93156 / 92376 / 92260 |
| hide | 5000 / 20000 / 80000 / 320000 | 20336 / 19724 / 20592 / 20256 | 54812 / 160444 / 582576 / - | 54980 / 76240 / 76084 / 75784 |
| defineobj | 5000 / 20000 / 80000 / 320000 | 20368 / 19804 / 20692 / 20724 | 54500 / 157088 / 573136 / - | 53764 / 76448 / 76576 / 76592 |

From 20000 on, step2 is flat within 1%. The rise from 5000 to 20000 is the first collection: a
retired behaviour is charged 128 bytes per method entry against the 32 MiB byte floor, which 5000
defines do not reach. That gap is collector tuning, which spec section 6 licenses.

Three mechanisms were each needed, measured in turn at 5000 / 20000 / 80000 / 320000 on `define`:
- freeing alone: 57848 / 170172 / 519344 (nothing charges a retired behaviour toward a collection);
- plus the charge: 57332 / 82968 / 104292 / 217212 (the class still owns every `Method` copy
  `define` made);
- plus letting go of those copies: the table above.

## Step 3: variable-reference cells

Programs: `task-8/memory/varref3.rex` (`r = >y` in a procedure) and `varref.rex` (`call q >y`, `use
arg >z`). The crate binary `rexx-run-step3` is the Step 3 commit.

| program | n | oracle | base | step3 |
|---|---|---|---|---|
| varref3 | 20000 / 80000 / 320000 | 20092 / 20328 / 20388 | 44972 / 112172 / 386440 | 47328 / 49580 / 50260 |
| varref | 20000 / 80000 / 320000 | 20500 / 20332 / 20624 | 23320 / 33224 / 41788 | 25868 / 29564 / 29516 |

`varref3` is within 10% from 20000 to 320000 (+6.2%). `varref` is +14.3% from 20000 to 80000 and
flat after it (-0.2% to 320000); an earlier run of an equivalent build measured 26704 / 28728 /
28564, so the first figure moves with where the first collection falls. The holders, the shapes
and the mutations are in `cell-holders.md`.
