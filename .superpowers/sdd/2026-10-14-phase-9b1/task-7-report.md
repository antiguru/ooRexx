# Phase 9b1 Task 7 report: executable and message maps, and the collector's trigger

Status: DONE_WITH_CONCERNS. Commits: `f9341ef33` (the change, tests, witness, tables),
`5fda502fb` (refusal-dispositions row and its probes), `5958a6f63` (records). This report is not committed.

## Step 1: the Method/Routine and Message maps

`Interp::prune_dead_object_rows` (`lib.rs`), called from `collect_now` after the sweep, drops every
row keyed by an object the sweep freed. `Activities::prune_senders` became `prune_messages` and drops
`message_ids` rows too. A swept class's `Annotated::Class`/`Member` rows go in the `freed_classes`
block beside `method_objects`.

Every map takes state 2 (keyed by an ObjRef, pruned at sweep) or state 3 (compiled data of a program,
pruned when Task 6 frees it). None takes state 1: each value is an `rexx-exec` type (`ExecutableRecord`,
`Raised`, `MethodFlagWrites`), so a field would need a new `rexx-core` store in `NativeObject`, whose
size is pinned at 104 bytes for hashnew's sake. That is a shared path, so not widened here.

Enumerated from scout-leaks' "Method / Routine object" and "Message object" groups, each name checked
with `grep -rn "\b<name>\b" crates --include=*.rs`.

| map | key | state | pruned where |
|---|---|---|---|
| `executable_sources` | Method/Routine ObjRef | 2 (and 3) | `prune_dead_object_rows`; `prune_program_rows` |
| `method_flag_writes` | Method ObjRef | 2 | `prune_dead_object_rows` |
| `executable_objects` | MethodId, value ObjRef | 2 by value (and 3) | `prune_dead_object_rows` drops rows whose object died; `prune_program_rows` |
| `message_outcomes` | Message ObjRef | 2 | `prune_dead_object_rows` (and its `remove` on reuse) |
| `started_messages` | Message ObjRef | 2 | `prune_dead_object_rows` |
| `unnotified_messages` | Message ObjRef | 2 | `prune_dead_object_rows` |
| scheduler `message_ids` | Message ObjRef | 2 | `prune_messages` (and its `remove` at completion) |
| scheduler `senders` | Message ObjRef | 2 | `prune_messages` (was `prune_senders`) |
| `annotations` `Class`/`Member` | class ObjRef | 2 | `collect_now`'s `freed_classes` block |
| `annotations` `Package`/`Unattached`/`Routine` | ProgramId | 3 | `prune_program_rows` (Task 6) |
| `annotations` `LibraryRoutine`/`InternalRoutine` | row index | 3 | never: global roots for the run, as the oracle keeps library routine objects; bounded by the row count |
| `routine_objects`, `library_routine_codes`, `rexx_routine_rows` | `InstalledRoutine` | 3 | `prune_program_rows` (Task 6) |
| `method_bodies`, `generated_methods`, `native_externals`, `guarded_externals`, `library_externals`, `special_methods` | MethodId | 3 | `prune_program_rows` (Task 6). See concern 1 |
| `defined_library_codes` | MethodId | 3 by intent | never pruned: its rows name a library code row, whose package is rooted for the run. See concern 1 |
| `library_routine_objects` | library code row | rooted | never: global roots, bounded by the row count |
| `internal_routine_objects` | `INTERNAL_ROUTINES` index | rooted | never: one slot per static row |
| `rexx_routine_objects`, `rexx_routine_executables` | | | no such field in the tree (scout names) |

Test: `run::tests::a_collection_drops_the_rows_of_swept_executables_and_messages` (`src/run/tests.rs`).
It keeps the main program loaded through `.local['KEEP'] = .context~package` (so Task 6's program
prune cannot remove the rows instead), runs three passes of `Message~new`+`send`, `Message~new`+`start`,
`.k~method('FOO')~copy`+`setPrivate` and `.object~subclass('t')~annotations`, finishes the started
activities with `run_started_to_end`, asserts the rows exist, calls `collect_now`, and asserts every key
(every value for `executable_objects`) is live.

Mutations (each by exact edit from a saved copy, restored by copy, `git diff --stat` checked after;
harness scratch `t7/mut/mutate.py`; debug, `cargo test -p rexx-exec --lib -- a_collection_drops_the_rows`):

| id | mutation | result |
|---|---|---|
| P1 | no `executable_sources` retain | red |
| P2 | no `method_flag_writes` retain | red |
| P3 | no `executable_objects` retain | red |
| P4 | no `message_outcomes` retain | red |
| P5 | no `started_messages` retain | red (green before `run_started_to_end` was added: an unrun started message is live) |
| P6 | no `unnotified_messages` retain | green: unwitnessed. A row exists only while a notification runs or after one failed |
| P7 | no `message_ids` retain | green: unwitnessed. Its rows are removed at completion, so only a waited-on message dropped before it completes leaves one |
| P8 | `Member`/`Class` rows kept | red |

### Memory witness (section 6), `memory.txt`

| loop | oracle 20000 / 80000 | head `t7b` 20000 / 80000 | 80000 vs 20000 | Task 6 `t6e` 20000 / 80000 |
|---|---|---|---|---|
| `Method~new` | 20112 / 20588 kB | 44720 / 45180 | +1.0% | 175420 / 176060 |
| `Routine~new` | 20572 / 20184 | 44912 / 44792 | -0.3% | 175400 / 174568 |
| source-string `setMethod` | 20268 / 20588 | 45288 / 45936 | +1.4% | 163524 / 164080 |
| `Package~new` | 20584 / 20588 | 54196 / 54692 | +0.9% | 419460 / 421280 |
| msgloop | 16220 / 20356 | 35824 / 43500 | **+21.4%** | 34180 / 45052 |
| `call ext` | 20376 / 20608 | 54840 / 54028 | -1.5% | 423892 / 426156 |

msgloop misses the 10% bar, and so does the oracle (+25.5%). Both reach their steady state between
20000 and 80000: `t7b` is 42852 kB at 160000 and 43548 kB at 320000, the oracle 20328 and 20460.
`t6e` kept growing: 46740 at 160000, 55628 at 320000. So the leak is gone and what is left is the
warm-up both interpreters share. msgloop is `o = .array~of(1,2)` then
`m = .Message~new(o,'ITEMS'); m~send` (scout-leaks does not name its `o`).

## Step 2: the allocation charge

- A program: `Interp::load_program` (`lib.rs`) charges `PROGRAM_BYTES` (24 kB) plus
  `PROGRAM_BYTES_PER_SOURCE_BYTE` (40) per source byte, then inserts. Every non-test
  `programs.insert` goes through it (`git grep -n 'programs.insert(' rust/crates/rexx-exec/src`, test modules aside) except the library
  bootstrap's, which is never freed. Measured on `t6e` before any collection: 30.5 kB per one-line
  `Package~new`, 113.8 kB per 101-line one (2000 and 4000 iterations, `pkr_*`, `pk101_*`).
- A package: `package_object` charges `PACKAGE_BYTES` (3 kB) when it builds a program's package.
  Measured: `Package~copy` costs about 3 kB each before a collection (`pcopy*`).
- A `::CLASS` class: `install_class` calls `charge_class`, as `~subclass` does, outside the library
  bootstrap.

Witness: `corpus/lang/package_class_collected.rex` (in `phase-9.txt`, sourceline file generated by the
driver from a scratch copy), pinned by `class_lifetime::a_loop_of_package_new_collects_its_classes_and_packages`.
Each loop is bounded and prints whether a WeakReference to a dropped object cleared: a
one-class `Package~new` (bound 5000), a classless one (5000), and a 50-class one (300). Oracle 3 of 3
runs rc 0, `class cleared: 1` / `package cleared: 1` / `one of many classes cleared: 1`. The oracle
clears at iteration 243, 651 and 9 (`clsweak`, `pkgweak`, `cls50weak`); `t7b` at 815, 1130 and 53;
`t6e` at 10863, 13036 and 1184 (1.3 GB peak on the last).

| id | mutation | `class_lifetime` (debug) |
|---|---|---|
| C1 | `load_program` charges nothing | red |
| C2 | `package_object` charges nothing | green: unwitnessed, every program package follows its program's charge |
| C3 | `install_class` charges nothing | red (line 3). Release binary of C3: `cls50weak` clears at 632, 703584 kB |

The queued item `.superpowers/sdd/queued/2026-10-12-gc-trigger-blind-to-class-weight.md` is closed
for `::CLASS` classes, packages and programs. Its other two clauses stay open: the emptied `Behaviour`
slots (`clsloop` grows about 180 B per class on `t7b`, 68756 / 79528 / 93972 kB at 20000 / 80000 /
160000; the oracle grows faster, 23340 / 56532 / 92060), and `Package~copy`, which Task 6's program
lifetime made flat (`pcopy` 60504 / 60400 kB, oracle 18784 / 20328).

### The floor (reported, not changed)

An experiment binary with `COLLECT_BYTES_FLOOR = 8 << 20` (`memory.txt`, `callgrind-floor8/`) peaks
at 25784 / 26040 / 26368 / 28560 / 28820 kB for the `Method~new`, `Routine~new`, `setMethod`,
`Package~new` and `call ext` loops at 80000, against 44.7 to 54.7 MB at 32 MB, and the oracle's
20 MB. msgloop does not move (43680 kB). Its callgrind cost on alloc, hashnew, startup, strings,
dirread, heapshape, compound and pingmsg, one round: within 0.0003% except hashnew -0.1286%.

## Step 3: budget

```
timeout 590 bench-programs/callgrind.sh -B 0.5 -j 16 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-7-a pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t7b
```

Exit 0, full set, 3 rounds, no OVER, no SPREAD. Records: `task-7/callgrind/`. Whole-process: alloc
-0.7165%, hashnew +0.1097%, startup +0.3362%, pingmsg -1.1209%, sendloop +0.0715%, dispatch -0.1564%,
compound +0.4347% (-libc +0.0031%), sayloop +0.2258%. Task 6 fix 1 had startup +0.3389%, hashnew
+0.1064%, alloc -0.4156%.

## Representation refusals

The `ALLOWED` rows tagged "Task 7" are removed with their sites, and "Task 7" is gone from
`OWNERS`. Each site (`executable.rs` source_of and begin_routine twice, `class_protocol.rs`
scoped_method, define, defineMethods, the class-method table and defineClassMethod,
`object_protocol.rs` copy) now refuses with `Loud::unrecorded_executable()`: "a Method or Routine
object with no executable record", an internal guard. Every builder of a `Method` or `Routine`
object records it (`record_executable`, `record_native_executable`, `record_loaded_executable`,
`record_rexx_routine_object`, the copy paths), and its record is now dropped only with the object.
Probes `task-7/probes/executable_a.rex` and `executable_b.rex` (copies, Method and Routine subclass
instances, setMethod, define and defineMethods with method objects, natives included) agree with the
oracle on stdout, stderr and rc. `refusal-sites.tsv` re-derived in `f9341ef33`, the new row filled
by hand. `refusal-dispositions.tsv` gained a GUARD row in `5fda502fb`, because the first commit's
table run did not include `refusal_dispositions` and it went red there.

## Files outside the brief's list

`install.rs` (the program loads and the class charge), `environment/identities.rs` (the package
charge), `dispatch.rs`/`class_protocol.rs` (exporting `charge_class`), `executable.rs`,
`object_protocol.rs` (the refusal sites). Each is a call-site change of this task's own mechanism.

## Checks (from `rust/`, `CARGO_INCREMENTAL=0 -j 4 timeout 590 memcap 8G`, statuses unpiped, outputs in scratch `t7/checks/`)

At `f9341ef33` unless noted.

| command | exit | result |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy -p rexx-exec --all-targets -- -D warnings` | 0 | |
| `cargo test --release -p rexx-exec --lib` | 0 | 1124 passed |
| `cargo test -p rexx-exec --lib` (debug) | 0 | 1124 passed |
| `cargo test --release -p rexx-exec --test collect_stress` | 0 | 55 passed |
| `cargo test -p rexx-exec --test collect_stress` (debug) | 0 | 55 passed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 29 passed, 1 ignored |
| the same, debug | 0 | 29 passed, 1 ignored |
| `cargo test --release -p rexx-exec --lib -- sim::tests` | 0 | 24 passed |
| `cargo test --release -p rexx-exec --test representation_refusals --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test package_requires --test class_lifetime --test class_weak_reference` | 101 | `refusal_dispositions` red (no row for `unrecorded_executable`), the rest ok |
| `cargo test --release -p rexx-exec --test refusal_dispositions --test refusal_sites` at `5fda502fb`'s tree | 0 | 3 and 5 passed |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 | 1 passed |
| suite, Task 6's groups plus `base/class/Message` and `base/class/WeakReference` (below), at `5fda502fb` | 101 | 11 groups, 5 red. `tests.tsv` against the 9b close2: one row changed, `TESTRECLAIM01` blocked (timeout) to passed. Against Task 6: identical on the shared rows |

```
REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package,base/class/Package_Options,base/directives/REQUIRES,API/oo/METHOD,base/directives/METHOD,base/directives/ROUTINE,base/class/Method,base/class/MethodArgs,base/class/Routine,base/class/Message,base/class/WeakReference REXX_SUITE_TABLE=<records>/task-7/suite/table.tsv CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite -- --nocapture
```

`rexx-core` was not touched and its tests were not run.

## TDD evidence

The prune test and the witness were written after the code. Against the base: the witness program
prints `class cleared: 0` / `package cleared: 0` on `t6e` (rc 0), so the test is red there. The
mutation tables above are the evidence each part is load-bearing.

## Concerns

1. **Copy identities still leak.** `copy_method_identity` mints a MethodId per `Method~copy`,
   `newScope` copy and Method subclass instance copy, and files rows under it in `method_bodies`,
   `generated_methods`, `native_externals`, `library_externals`, `special_methods` and dispatch's
   `natives`, which go only when the template's program is freed. The main program's never is.
   Loops on `t7b` at 20000 / 80000 / 320000: `.k~method('FOO')~copy` 54864 / 58168 / 73892 kB,
   with `setPrivate` too 55160 / 61344 / 82132, `.object~method('STRING')~copy` 55144 / 56752 /
   68676 (oracle flat at about 20 MB; `t6e` 117672 / 138940 / 116800 at 320000). Under 10% at
   80000, about 50 to 90 bytes a copy. Pruning them when the copy's object dies is not safe as it
   stands: a frame can run a method after its `Method` object is swept
   (`run::tests::an_executable_rebuilt_under_the_nil_scope_answers`), so it needs a rule for who
   holds a copy identity. That is a new identity rule, reported rather than built.
   `defined_library_codes` is the same shape for `loadExternalMethod` answers.
2. msgloop is +21.4% from 20000 to 80000 against the 10% bar. The oracle is +25.5%, and both are
   flat from 80000 to 320000 (above). The criterion as written cannot be met by either interpreter
   on this program.
3. P6 and P7 (`unnotified_messages`, `message_ids` prunes) and C2 (the package charge) have no test
   that fails without them.
4. Absolute peaks are 2.1x to 2.7x the oracle's at the 32 MB floor; an 8 MB floor brings them to
   1.3x to 1.4x at no measured instruction cost on the programs tried.
5. The first commit's table run missed `refusal_dispositions`, so `f9341ef33` alone is red on it;
   `5fda502fb` fixes it.

## Fix round 1

Commits: `00a96524a` (loaded-method prune, class charge placement, witnesses), `8d14e52b8` (churn
bench program), `06febe9ab` (records in `task-7/fix1/`).

### (1) loadExternalMethod identity rows

`prune_dead_object_rows` (`lib.rs`) reads each swept object's `ExecutableRecord` before dropping
it. If the method identity it ran under is a `loadExternalMethod` identity, that identity's rows go
too: `defined_library_codes`, `library_externals`, `native_externals`, `guarded_externals` and its
`special_methods` row. A `loadExternalMethod` identity is one with a `defined_library_codes` row (a
shared library's procedure, or a copy of one), or a `Native` record whose identity has a
`native_externals` row (`LIBRARY REXX`, minted by `record_rexx_external_method`, which the review
did not name and which leaked the same way). No program owns these identities, and every installer
holds the object (`define` and `defineMethods` through `hold_method_object`, `setMethod` through
`ObjectMethod::executable`, a scoped copy through its own new identity). A directive method's
identity is never in either set: its record's source is `Directive`/`External`, and a builtin's
identity has no `native_externals` row.

Memory (`fix1/memory.txt`), `m = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Init')`:

| iterations | oracle | `t7e` | `t7b` (before) |
|---|---|---|---|
| 20000 | 19920 kB | 30996 | 31304 |
| 80000 | 20124 | 54888 | 55012 |
| 160000 | 20332 | 54288 | |
| 320000 | 20328 | 54928 | 80532 |

Flat from 80000 to 320000 (+0.07%). **20000 to 80000 is +77%**, and that is warm-up, not growth:
the first collection comes between 40000 and 80000 iterations (`t7c` 10000 / 40000: 26640 / 39180
kB), and nothing in this loop is charged beyond its strings. This is the msgloop shape. The bar as
written (80000 within 10% of 20000) is not met. The 80000 vs 320000 judgement the msgloop ruling
uses is met.

Safety: the review's m1 to m5 on `t7e` against the oracle, plain, stdout sorted: m1, m2, m4 and m5
agree, rc 0. m3 stops at rc 159 on `o~instanceMethod('Y')~isGuarded`, the queued pre-existing
`instanceMethod` divergence (identical stdout and rc on `t7b`). The m-shapes are Rexx methods,
which this prune does not touch, so a loaded-method shape was added:
`collect_stress::an_installed_loaded_method_survives_its_dropped_object`. It covers a `LIBRARY REXX
file_separator` answer installed by `define`, one flagged private then installed and dropped, a
copy installed by `setMethod`, and fresh answers flagged protected and dropped. Oracle, the same
program with a 300-pass `copies('x', 100000)` filler, rc 0: `define /` / `private / 1 1` / `own /` /
`again / / /` (probe `fix1/probes/loaded_method_dropped.rex`; `t7c` plain agrees).

Test: `run::tests::a_collection_drops_the_rows_of_swept_loaded_methods`. Three passes of a `LIBRARY
REXX stream_uninit` answer and a `LIBRARY rxregexp RegExp_Match` answer, each `setPrivate`, plus a
copy of the second, all dropped. It asserts the four row counts grew, then after `collect_now`
returned to the post-bootstrap baseline. It points `library_search` at the oracle's lib directory,
the same hard-coded path `tests/support/oracle.rs` uses.

### (2) Witnesses

`run::tests::a_collection_drops_the_unnotified_and_waiter_rows_of_swept_messages` (P6, P7) and
`run::tests::building_a_programs_package_charges_it` (C2), from the review's text, with names and
messages of their own.

### (3) The class charge

`install_class` no longer charges. `install_class_at` charges after the instance-side members are
installed (the class side was installed earlier, before `INIT`). Test:
`run::tests::a_class_directive_is_charged_with_its_methods`, the same source with twenty `::METHOD`s
attached to `::class c` and with them ahead of it. The attached version must be charged at least
20 x 128 bytes more, with no collection in between. Measured: `cls50weak` stays at 53, and
`clsmweak` (20 classes of 20 methods) clears at 64 against 71 before the move (oracle 7), with a
peak of 265136 kB. Divided over 64 packages of 20 classes, that peak is about 200 kB per class. In
the test's C4 reading, a class with no methods of its own plus its package's 3 kB came to 16096 bytes.
So the per-class charge is about a tenth of the measured weight, and recalibrating it is collector
tuning.

### (4) The churn program

`bench-programs/churn.rex`: `n = 100000` strings of 10,000 bytes made and dropped while 20,000 objects
are held. Registered in `callgrind.sh` PROGRAMS, `rexx-bench` PROGRAMS, the suite's AXES (`Role::Loop`)
and the README. Sized against this crate: 0.11 s and 54896 kB. The oracle takes 2.14 s and 18004 kB.
The `rexx-bench` tests pass, `every_loop_axis_has_a_loop_bound` among them (it needs the `n = `
line). One round, `-p churn`, `t7b` against the 8 MB floor binary: +49.63% whole-process, +81.93%
without libc (`fix1/callgrind-churn-floor8/`).

### Mutations (exact edits from a saved copy, restored by copy, `git diff --stat` checked)

| id | mutation | result |
|---|---|---|
| P6 | no `unnotified_messages` retain | red, the unnotified test |
| P7 | no `message_ids` retain | red, the same test's second assertion |
| C2 | no `PACKAGE_BYTES` charge | red, `building_a_programs_package_charges_it` |
| L1 | orphaned `defined_library_codes` row kept | red, the loaded-methods test |
| L2 | orphaned `library_externals` row kept | red |
| L3 | orphaned `native_externals` row kept | red |
| L4 | orphaned `special_methods` row kept | red |
| L5 | the `defined_library_codes` test of an orphan disabled | red |
| L6 | orphans taken from live records too (`!keep` dropped) | red, `an_installed_loaded_method_survives_its_dropped_object` (collect_stress, debug) |
| C4 | the class charge moved back before the methods | red, `a_class_directive_is_charged_with_its_methods` (`16096 against 16096`) |

P6 to L5 ran as `cargo test -p rexx-exec --lib -- run::tests::` (debug). Each was red only in the
test named. `guarded_externals` in the orphan prune is not witnessed: no loaded method has a row
there.

### Budget

```
timeout 595 bench-programs/callgrind.sh -B 0.5 -j 16 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-7-fix1 pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t7e
```

Exit 0, full set with churn, 3 rounds, no OVER, no SPREAD (`fix1/callgrind/`). Whole-process:
startup +0.3399%, churn +0.4128%, compound +0.4347%, hashnew +0.1349%, sayloop +0.2280%, alloc
-0.7036%, pingmsg -1.1118%. `t7e` is the release build at `8d14e52b8`. It is byte-identical to `t7d`,
which was built at `00a96524a`'s source.

### Checks (from `rust/`, `CARGO_INCREMENTAL=0 -j 4 timeout 590 memcap 8G`, statuses unpiped)

| command | exit | result |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy -p rexx-exec -p rexx-bench --all-targets -- -D warnings` | 0 | |
| `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites -- --test-threads=1` | 0 | tsv unchanged |
| `cargo test -p rexx-bench` | 0 | all binaries ok |
| `cargo test --release -p rexx-exec --lib` | 0 | 1128 passed |
| `cargo test --release -p rexx-exec --test collect_stress` | 0 | 56 passed |
| `cargo test -p rexx-exec --test collect_stress` (debug) | 0 | 56 passed |
| the refusal tables and lifetime binaries (the first round's command) | 0 | 9 binaries ok |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 29 passed, 1 ignored |

The debug corpus, the debug lib run and the suite groups were not re-run this round.

### Concerns

1. The loadExternalMethod loop misses 80000-vs-20000 for the msgloop reason, which is warm-up below
   the 32 MB floor. It is flat from 80000 to 320000. It needs the msgloop ruling's exception, or a
   different bound.
2. `CLASS_BYTES_PER_METHOD` (128) charges about a tenth of what a method-heavy `::CLASS` weighs
   (`clsmweak`: 265 MB peak, oracle 21 MB). That is collector tuning, and this round did not change it.
3. churn sits at +0.41% against the pin, which leaves little headroom on it.

## Fix round 2

Commits: `99404aa00` (class charge recalibrated), `780ece09d` (records,
`task-7/fix2/`).

Ruling noted: the loadExternalMethod loop takes the msgloop warm-up exception and is judged 80000 vs
320000 (54888 / 54928 kB, +0.07%; oracle 20124 / 20328).

### The class charge

Measured on `t7e` with both collection floors at `1 << 40` (no collection; binary not committed).
Each program keeps N `Package~new` results, and a package's weight is the peak-RSS slope from N = 20
to N = 60 (`fix2/memory.txt`, programs included):

| shape | per package | per class |
|---|---|---|
| 20 `::CLASS`es of 20 `::METHOD`s (A) | 3816 kB | |
| the same 400 methods unattached, then 20 bare classes (B) | 615 kB | |
| the 400 unattached methods alone (D) | 223 kB | |
| 20 bare classes, (B - D) / 20 | | 19.6 kB |
| each own `::METHOD`, (A - B) / 400 | | 8.0 kB |
| 20 `subclass Array` (E) | 624 kB | 31.2 kB |
| 20 `subclass Object` (F) | 465 kB | 23.2 kB |

One per-name constant cannot fit both shapes: an own method weighs about 40 times an inherited name.
So `charge_class` (`class_protocol.rs`) now charges `CLASS_BYTES_PER_NAME` (200, was
`CLASS_BYTES_PER_METHOD` 128) per name the two behaviours resolve, plus `CLASS_BYTES_PER_OWN_METHOD`
(8 kB) per name in the class's own two method dictionaries. The second count comes from a new
`ClassRegistry::own_method_names` / `ClassGraph::own_method_names` in `rexx-classes`, a one-line
accessor and a file outside the brief. At 200 per name, a bare `Object` subclass (about 101 names,
from the C4 reading) is charged about 20 kB and an `Array` subclass proportionally more, which
matches E and F. The comment cites the measurement.

Witness (`fix2/memory.txt`). Iteration at which a dropped class first clears, and peak RSS:

| loop | oracle | `t7f` (this round) | `t7e` (before) |
|---|---|---|---|
| `clsmweak`: 20 classes of 20 methods | 7, 20632 kB | **8**, 52356 kB | 64, 265792 kB |
| `cls50weak`: 50 bare classes | 9, 22716 kB (fix 1) | 35, 58748 kB | 53, 77244 kB |
| `clsweak`: one bare class | 243, 20644 kB (fix 1) | 705, 54576 kB | 815, 60736 kB |

`clsmweak` is now within one iteration of the oracle. `clsloop` 20000 / 80000: 59780 / 72120 kB, the
`Behaviour`-slot residue already recorded.

`run::tests::a_class_directive_is_charged_with_its_methods` now requires the 20 attached methods to
add at least 20 x 8 kB. Mutant C5 (own-method term multiplied by 0): red. Mutant C4 (charge before
the methods) still has the test as its witness.

### Budget

```
timeout 595 bench-programs/callgrind.sh -B 0.5 -j 16 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-7-fix2 pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t7f
```

Exit 0, full set including churn, 3 rounds, no OVER, no SPREAD (`fix2/callgrind/`). Whole-process:
compound +0.4347% (unchanged; -libc +0.0031%), churn +0.4124%, startup +0.3379%, sayloop +0.2269%,
hashnew +0.1192%, alloc -0.7036%. `t7f` is the release build at `99404aa00`, byte-identical to a
rebuild after the commit.

### Checks (from `rust/`, `CARGO_INCREMENTAL=0 -j 4 timeout 590 memcap 8G`, statuses unpiped)

| command | exit | result |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy -p rexx-exec -p rexx-classes --all-targets -- -D warnings` | 0 | |
| `cargo test --release -p rexx-exec --lib` | 0 | 1128 passed |
| `cargo test --release -p rexx-classes` | 0 | all binaries ok |
| `cargo test --release -p rexx-exec --test class_lifetime --test class_weak_reference --test refusal_sites --test representation_refusals --test refusal_dispositions` | 0 | all ok |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 29 passed, 1 ignored |

collect_stress was not re-run this round: the change is a charge amount, which the stress mode
(collect at every allocation) does not read.
