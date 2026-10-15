# Task 8 memory loops

Peak RSS in kB, `/usr/bin/time -f %M` on the interpreter itself, one run each, from a fresh
directory holding the programs in this folder:

    memloop.sh BIN PROGRAM N...      # BIN is a crate binary or "oracle"

which runs `memcap 2G timeout -k 5 120 /usr/bin/time -f %M BIN PROGRAM N` for a crate binary and
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -k 5 120 /usr/bin/time -f %M .../build/bin/rexx PROGRAM N )`
for the oracle. Binaries (in /home/moritz/dev/repos/ooRexx-p9-pin/): `rexx-run-9b1-t8base` is
d965c149a (Task 8's base), `rexx-run-9b1-t8h` is 882c826eb (sha256 8a5321491a9c9c1a...).

| loop | oracle 20000 / 80000 kB | base `t8base` 20000 / 80000 | head `t8h` 20000 / 80000 / 320000 | head 80000 vs 20000 |
|---|---|---|---|---|
| `copy` | 12288 / 19816 | 54912 / 57592 | 55168 / 56268 / 56220 | +2.0% |
| `copypriv` | 12692 / 20048 | 54860 / 60904 | 55284 / 59116 / 70724 | +6.9% |
| `nativecopy` | 13216 / 19488 | 54812 / 57556 | 54452 / 57472 / 71760 | +5.5% |
| `setmethod` | 20116 / 20328 | 37364 / 54332 | 35984 / 51856 / 51640 | +44.1% |
| `nativesetmethod` | 20324 / 20580 | 35444 / 50484 | 36492 / 52460 / 66056 | +43.8% |
| `cm4` | 29756 / 56196 | 60260 / 72064 | 54096 / 54940 / 55628 | +1.6% |
| `cm5` | 28480 / 58344 | 325044 / 1174604 | 53780 / 53012 / 53068 | -1.4% |
| `cm1` | 26328 / 56544 | 190860 / 639156 | 48012 / 48164 / 47508 | +0.3% |
| `constmeth` | 25272 / 50712 | 191128 / 635808 | 48376 / 48796 / 48084 | +0.9% |
| `litmeth2` | 20376 / 20864 | 54888 / 73136 | 50016 / 50856 / 50716 | +1.7% |
| `interp` | 20336 / 20804 | 36396 / 53464 | 36544 / 36588 / 36552 | +0.1% |
| `cn3` | 20324 / 20076 | 314564 / 985820 | 47968 / 48320 / 47292 | +0.7% |

| loop | oracle 5000 / 20000 kB | base 5000 / 20000 | head 5000 / 20000 |
|---|---|---|---|
| `define` | 20328 / 20460 | 57708 / 170640 | 57644 / 170956 |
| `defineobj` | 20120 / 20328 | 54424 / 157780 | 54164 / 158192 |

`setmethod` reaches its steady state between 20000 and 80000 on both binaries and is flat from
80000 to 320000, the msgloop warm-up. `copypriv`, `nativecopy` and `nativesetmethod` grow about
50 bytes per minted identity past 80000: `special_methods` and the primitive table `natives` are
vectors indexed by MethodId, whose length follows the newest identity holding a row (reported).
`define` and `defineobj` (`.k~define`/`.k~delete` on a live class) keep a full behaviour copy per
call for the class's lifetime (reported).

Further loops, same commands:

| loop | oracle 20000 / 80000 kB | head `t8h` 20000 / 80000 / 320000 |
|---|---|---|
| `sub3` (`.object~subclass`) | 25416 / 53576 | 58656 / 59380 / 57960 |
| `pcopy` (`Package~copy`) | 18552 / 20592 | 59820 / 61028 / 60768 |
| `varref` (`call q >y` from a procedure) | 20072 / 20604 | 24200 / 33564 / 40688 |
| `varref3` (`r = >y` over a 1000-byte value) | 20340 / 20328 | 46652 / 112748 / 385864 |

`varref` and `varref3` are `RootSet::cells` (reported): a variable a `>name` reference was taken to
moves to a cell that is never freed and whose value stays a root, about 1.1 kB per iteration of
`varref3`.
