# whole_groups memory at 8G

Investigator wg-mem, 2026-10-08. Measuring only; nothing in the tree changed.

## Verdict

One group grew: `base/class/Class.testGroup`, part `whole`. Commit `7d233521f` (Task 4, "every
Method object runs"). Before it, the whole run refuses at TEST_CLASS_DEFINE (`method "TEST1" of
class "TESTDEFINE1" is not implemented`, rc 120, 41 MB). From it on, TEST_CLASS_DEFINE runs, the
run goes on to TEST_SUBCLASSES_GC, and that test allocates about 1 GB a second until killed.
The growth is legitimate in the sense the brief names (the group runs further because a refusal
no longer stops it) and lands on a defect that predates Phase 6.1: the collector triggers on the
arena's slot count, not bytes, so large dead strings are never collected.

## How whole_groups runs

`concurrency_tests.rs` `group_runs::whole_groups::each_group_of_the_derived_list_in_one_run_in_both_modes`.
Each (group, part) is a row; `rows_in_parallel` runs rows on a rayon pool of `REXX_GROUP_ROWS`
(default 6), WALL_CLOCK rows alone afterwards. A row runs the oracle as a subprocess (5, or 30 when
unsettled) and our side **in process**, `run_crate_within` via `watchdog::run_bounded_with` on a
thread, normal and every modes at once (`at_once`). So up to 12 interpreters share the test
process, and an in-process run is bounded only by its deadline (`max(4 x slowest oracle, 60 s)`),
not by memory. Across rows the harness holds only the `Row` strings and the normal run's
stdout/stderr; per-row copies under `CARGO_TARGET_TMPDIR` are removed after each row.
`REST_LEFT_OUT` (TEST_SUBCLASSES_GC, TEST_UNINIT, TEST_UNINIT_CLASS) exists for exactly this
hazard, "allocates here until memory runs out, which takes the shared test process with it", but
is applied only to the `rest` rerun after a whole run refuses.

## Per-group table

Commands. Trees: `git archive` of `ec97e4190` and `a46465a9f` into `wgmem/old`, `wgmem/new`,
`find -exec touch`, `ootest` symlinked in, `CARGO_TARGET_DIR=wgmem/t-old|t-new memcap 8G cargo
build -j 4 --release -p rexx-exec --bin rexx-run` (each log has `Compiling rexx-exec`). Group
copies made by the harness's own `copy()` (rxapi tests renamed out, derived part reduced, starts
marked) from a scratch-only test added to `wgmem/new`. Each cell is one subprocess run of the copy:

    cd COPY; PATH=bin-COMMIT:$PATH LD_LIBRARY_PATH=$ORACLE/lib [REXX_SWITCH_MODE=every] \
      memcap 3G /usr/bin/time -v timeout -k 5 200 rexx-run testOORexx.rex -f COPY/ooRexx/GROUP -U -V 2

Cell: max RSS KB / exit status, `OOM` = memcap killed it at 3G (no RSS line), `ref` = a
`rexx-exec:` refusal line. old = `ec97e4190`, new = `a46465a9f`. rc 124 on MutexSemaphore every
is `timeout` at 200 s on both commits.

| group | part | old normal | old every | new normal | new every |
|---|---|---|---|---|---|
| base/bif/STREAM.testGroup | whole | 41916/2 | 41440/2 | 42076/2 | 41448/2 |
| base/bif/STREAM.testGroup | derived | 38796/0 | 38388/0 | 38508/0 | 38592/0 |
| base/bif/TIME.testGroup | whole | 77272/0 | 77160/0 | 77136/0 | 78080/0 |
| base/bif/TIME.testGroup | derived | 54764/0 | 54064/0 | 54076/0 | 54556/0 |
| base/class/Alarm.testGroup | whole | 42112/0 | 42380/0 | 41592/0 | 42256/0 |
| base/class/Alarm.testGroup | derived | 42160/0 | 42168/0 | 41976/0 | 41920/0 |
| base/class/Class.testGroup | whole | 41836/120/ref | 41740/120/ref | none/137/OOM | none/137/OOM |
| base/class/Class.testGroup | derived | 42220/0 | 41292/0 | 41452/0 | 41540/0 |
| base/class/DateTime.testGroup | whole | 53032/0 | 53008/0 | 52972/0 | 52760/0 |
| base/class/DateTime.testGroup | derived | 38712/0 | 38340/0 | 39068/0 | 38744/0 |
| base/class/EventSemaphore.testGroup | whole | 37648/0 | 36960/0 | 36828/0 | 37620/0 |
| base/class/EventSemaphore.testGroup | derived | 37472/0 | 37516/0 | 37176/0 | 37648/0 |
| base/class/Message.testGroup | whole | 42476/120/ref | 41960/120/ref | 42080/120/ref | 41988/120/ref |
| base/class/Message.testGroup | derived | 40820/120/ref | 40336/120/ref | 40216/120/ref | 40572/120/ref |
| base/class/Method.testGroup | whole | 41416/120/ref | 41140/120/ref | 40232/120/ref | 40828/120/ref |
| base/class/Method.testGroup | derived | 39244/0 | 39124/0 | 39596/0 | 39120/0 |
| base/class/MethodArgs.testGroup | whole | 58384/0 | 58648/0 | 58288/0 | 58348/0 |
| base/class/MethodArgs.testGroup | derived | 58532/0 | 58836/0 | 58104/0 | 58656/0 |
| base/class/MutexSemaphore.testGroup | whole | 36600/0 | 37328/124 | 36492/0 | 37028/124 |
| base/class/MutexSemaphore.testGroup | derived | 36484/0 | 37416/124 | 36804/0 | 36732/124 |
| base/class/Object.testGroup | whole | 46972/120/ref | 47004/120/ref | 46556/120/ref | 46992/120/ref |
| base/class/Object.testGroup | derived | 47040/0 | 47384/0 | 46996/0 | 47416/0 |
| base/class/RexxContext.testGroup | whole | 35820/120/ref | 36104/120/ref | 36100/120/ref | 35968/120/ref |
| base/class/RexxContext.testGroup | derived | 37160/0 | 36500/0 | 37424/0 | 36480/0 |
| base/class/Ticker.testGroup | whole | 40408/0 | 40720/0 | 40168/0 | 41112/0 |
| base/class/Ticker.testGroup | derived | 40416/0 | 40800/0 | 40616/0 | 41100/0 |
| base/directives/ATTRIBUTE.testGroup | whole | 39168/120/ref | 39048/120/ref | 46220/2 | 46608/2 |
| base/directives/ATTRIBUTE.testGroup | derived | 40400/1 | 40920/1 | 40824/1 | 40920/1 |
| base/directives/CONSTANT.testGroup | whole | 37148/120/ref | 36668/120/ref | 36772/120/ref | 37972/120/ref |
| base/directives/CONSTANT.testGroup | derived | 38036/0 | 37312/0 | 37564/0 | 38120/0 |
| base/directives/METHOD.testGroup | whole | 38516/120/ref | 38888/120/ref | 42436/120/ref | 42248/120/ref |
| base/directives/METHOD.testGroup | derived | 40300/1 | 41216/1 | 40972/1 | 40436/1 |
| base/keyword/CALL.testGroup | whole | 38424/120/ref | 38760/120/ref | 70636/0 | 71180/0 |
| base/keyword/CALL.testGroup | derived | 38076/0 | 38348/0 | 38472/0 | 38388/0 |
| base/keyword/GUARD.testGroup | whole | 36056/120/ref | 36068/120/ref | 38908/0 | 39256/0 |
| base/keyword/GUARD.testGroup | derived | 38376/0 | 38896/0 | 38352/0 | 38164/0 |
| base/keyword/RAISE.testGroup | whole | 39980/1 | 39416/1 | 39584/1 | 39200/1 |
| base/keyword/RAISE.testGroup | derived | 38216/1 | 38216/1 | 38372/1 | 38504/1 |
| base/keyword/REPLY.testGroup | whole | 39828/0 | 39652/0 | 39620/0 | 39496/0 |
| base/keyword/REPLY.testGroup | derived | 39308/0 | 38268/0 | 39816/0 | 38964/0 |
| base/keyword/TRACE.testGroup | whole | 43204/120/ref | 43404/120/ref | 47984/2 | 47868/2 |
| base/keyword/TRACE.testGroup | derived | 39016/0 | 39092/0 | 39084/0 | 38416/0 |
| base/keyword/TRACE_TraceObject.testGroup | whole | 35984/120/ref | 36692/120/ref | 39584/2 | 39772/2 |
| base/keyword/TRACE_TraceObject.testGroup | derived | 38540/2 | 37704/2 | 38812/2 | 39112/2 |
| base/rexxutil/SysSleep.testGroup | whole | 37572/0 | 36820/0 | 36804/0 | 37124/0 |
| base/rexxutil/SysSleep.testGroup | derived | 36824/0 | 36664/0 | 36796/0 | 36992/0 |
| base/special.variables/RESULT_RC_SIGL.testGroup | whole | 36876/0 | 36940/0 | 37460/0 | 36848/0 |
| base/special.variables/RESULT_RC_SIGL.testGroup | derived | 36820/0 | 36596/0 | 36772/0 | 37068/0 |
| doc/rexxref/chapter5/Section1.testGroup | whole | 34992/120/ref | 34692/120/ref | 35244/120/ref | 35532/120/ref |
| doc/rexxref/chapter5/Section1.testGroup | derived | 35524/0 | 36708/0 | 36988/0 | 36492/0 |
| regressions/bug2003_guard_when.testGroup | whole | 37276/0 | 36816/0 | 36660/0 | 37108/0 |
| regressions/bug2003_guard_when.testGroup | derived | 35852/0 | 36540/0 | 36536/0 | 36364/0 |

Only Class whole moved from tens of MB. CALL whole grew 38 MB to 71 MB because it now finishes
instead of refusing; ATTRIBUTE, GUARD, TRACE, TRACE_TraceObject whole likewise finish now, all
under 50 MB.

## The commit

Range bisected by code commit (`git log ec97e4190..a46465a9f -- rust/crates`; the first code
commit is `7d233521f`, whose parent `e5f465ddc` changes only the ledger after `ec97e4190`).
Built `7d233521f` the same way (`wgmem/t-t4`, `Compiling rexx-exec` present):

| run at 7d233521f, normal | result |
|---|---|
| Class whole | memcap OOM at 3G, rc 137 |
| Object whole | 46988 KB, rc 120, refuses at TEST_RUN_ARRAY_ARGUMENT (MAKEARRAY, Phase 9) |
| Class whole, TEST_SUBCLASSES_GC renamed out of the copy | 50944 KB, rc 1, 102 tests ran |
| same at a46465a9f | 51032 KB, rc 1, 102 tests ran |

So the first bad commit is `7d233521f` and the sole cause in Class whole is TEST_SUBCLASSES_GC.
Object whole still refuses before TEST_UNINIT on both commits; it will hit the same wall the day
MAKEARRAY stops refusing.

## Mechanism, with evidence

1. Where the memory goes. `gdb -batch -p` on the a46465a9f Class whole run at 600 MB RSS (RSS
   0.5 GB at 0.5 s, 1.07 GB at 1 s): the interpreter thread is in `builtin::string::copies_bytes`
   (`string.rs:726`) extending a 3-byte source, under `builtin::run` from the IR loop. The only
   3-byte COPIES in the group is TEST_SUBCLASSES_GC's `x = copies('abc', 100000)` inside
   `loop while ref~value \= .nil` (Class.testGroup line 1167). No stdout or stderr appears before
   the kill because nothing parks first, so the buffered `started` lines never reach the sinks.
2. Why the loop never ends: the weak reference is to a class, and classes are never collected
   (D59). That alone would be a spin until the deadline.
3. Why it allocates without bound: `Interp::collect_if_due` (`lib.rs:2637`) collects only when
   `heap.will_grow() && heap.slot_capacity() >= collect_at`, `collect_at` starting at
   `COLLECT_FLOOR` = 65 536 slots. A 300 KB string is one slot, so the first collection is due
   after about 19.7 GB of dead strings. Counting probe (scratch test calling `run_program` on
   `do i = 1 to N; x = copies('abc', SIZE); end`, reading `Outcome::collections`, release, a46465a9f):

   | N | string bytes | collections |
   |---|---|---|
   | 1000 | 300000 | 0 |
   | 4000 | 300000 | 0 |
   | 60000 | 999 | 0 |
   | 70000 | 999 | 1 |
   | 200000 | 999 | 3 |

   Max RSS of the same loop through `rexx-run`: N=1000 312 MB, 2000 606 MB, 4000 1192 MB at
   300 KB on both ec97e4190 and a46465a9f (identical to 0.1%); at 999 bytes it plateaus at 89 MB
   from N=100000 to N=1000000. The oracle (`ulimit -v 1048576`, from an empty dir) runs N=4000
   and N=20000 at 300 KB in 13 MB each; ours OOMs at 3G on N=20000.

Not a leak in the Task 4, 4a or 5 sense: no `ActivationCold.executable` rooting, truth temps or
trace accumulation is involved. RSS is flat across commits for every group that runs the same
tests on both, and the growth needs no live objects, only unreclaimed dead ones. The slot-count
trigger is already recorded (`docs/superpowers/records/2026-09-04-phase-5c-followup/task-1-report.md`
3.4; `phase-4f-record.md` Cause A argues the peak is bounded by a multiple of the live set, which
holds in slots and not in bytes).

## Recommendation

1. Harness, now (unblocks Task 5): leave the `REST_LEFT_OUT` tests out of every part, not only
   the rest rerun, on both sides, so the oracle comparison stays like for like. In `rows_of`,
   after `let mut left_out = reaching_rxapi(dir, &[group]);`:

       for (_, test, _) in REST_LEFT_OUT.iter().filter(|(listed, ..)| *listed == file) {
           left_out.insert(format!("{group}.{test}"));
       }

   Class whole's expectation rows then change (it no longer refuses; the scratch run above gives
   rc 1 with failures TEST_ACTIVATE and TEST_METHODS, the rest row's failures). Its `rest` rows
   in DIFFERING become unused. No cap change is needed: see the validation below.
2. Interpreter, a later task for Moritz to place: make the collection trigger account bytes held
   by string bodies (collect when bytes allocated since the last collection exceed a floor or a
   multiple of live bytes), so a loop of large dead strings runs in bounded memory as the
   oracle's does. This does not make TEST_SUBCLASSES_GC finish (D59 keeps the class), so item 1
   stays either way. It touches the per-allocation test in `collect_if_due`, so it needs the
   perf budget.

## Validation of recommendation 1

The patch above applied to the scratch copy of `a46465a9f` only, then:

    REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_TABLE=wgmem/wg-table.tsv memcap 8G /usr/bin/time -v \
      cargo test -j 1 --release -p rexx-exec --test concurrency_tests whole_groups

Result: no OOM; test binary max RSS 2 423 492 KB (2.3 GiB), wall 5:14; exit 101 on expectation
rows only. Six pass; the main test lists 20 (group, part, mode) rows whose key moved, all
"listed as X, is Y", none "an inverted wait or a hang": Class whole; Method rest; Object rest;
ATTRIBUTE whole; CONSTANT whole and rest; METHOD whole and rest; TRACE whole; TRACE_TraceObject
whole; each in both modes. These are the Task 4, 4a and 5 expectation updates Task 5 owes. The
table and the full failure list are `whole-groups-memory-table.tsv` and
`whole-groups-memory-failing.txt` beside this file. The 8G cap holds with room; no raise is needed.

Scratch trees and target dirs deleted after the runs.
