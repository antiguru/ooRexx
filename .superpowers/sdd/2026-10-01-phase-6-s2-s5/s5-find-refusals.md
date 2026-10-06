# S5 findings: refusals (Task 24 Step 1 measurement, Step 2 prep)

Agent `refusals`, HEAD 540e6a72ea7c485972ac1d7c4f36b97dea322ad4, 2026-10-06. Evidence under
`s5-evidence/refusals/` (E below). Our side: release `rexx-run` built at HEAD's working tree
(`Compiling rexx-exec` seen, own target dir). Oracle: the rules' wrapper, fresh empty dir per run
(`E/run.sh`, `E/run30.sh`).

## 1. The S0/S1 enumeration at HEAD

Commands (from `docs/superpowers/plans/phase-6-gate.md:65-67`), from the repository root:

```
/bin/grep -a -rn 'Phase 6' rust/crates --include=*.rs | /bin/grep -av '/tests\.rs\|/tests/' | /bin/grep -av ':[0-9]*:\s*//'
/bin/grep -a -rPzoc 'Phase\s+6' rust/crates --include=*.rs | /bin/grep -v ':0$'
/bin/grep -a -c 'Phase 6' rust/corpus/refusal-sites.tsv
```

- Command 1: **0 lines** (`E/cmd1.txt`, empty).
- Command 2 (`E/cmd2.txt`): `rexx-exec/src/run.rs`, `rexx-core/src/handle.rs`, and six test files
  (`gate_table_c.rs`, `internal_routines.rs`, `api_group_tests.rs`, `closed_phases.rs`,
  `concurrency_tests.rs`, `program_end.rs`).
- Command 3: 0.

The two non-test hits are comments (unfiltered first stage):
`rust/crates/rexx-exec/src/run.rs:2375` (`// **LEGALITY, and Phase 6 keeps it.**`) and
`rust/crates/rexx-core/src/handle.rs:50` (`/// Phase 6 says so (design section 2.5).`). Neither is an
owner and `closed_phases` reads string literals only, so neither reddens it (section 4 run).

Other spellings checked, none an owner: `grep -rn -i 'phase[ _:]*6\|phase6\|P6\b'` over non-test
`rust/crates` finds only `spec 2026-09-29 P6-3/P6-4` citations (baton.rs:13, scheduler.rs:466,
:1953, dispatch/library.rs:314, rexx-api values.rs:728). Owner literals in non-test sources:
`"Phase 10"` 60, `"Phase 5"` 15, `"Phase 9"` 6, no other.

The criterion-8 list items, probed at HEAD (E/probes2, E/out2, all identical to the oracle):
GUARD WHEN wait (`guard_when.rex`), REPLY inside DO (`reply_in_do.rex`), Alarm/Ticker
construction and an Alarm firing (`alarm_construct.rex`, `ticker_construct.rex`,
`alarm_fires.rex`), unnamed Sys*Sem routines (`E/probes3/syssem_empty.rex`; `SysCreateMutexSem()`
with no argument segfaults the oracle, rc 139, known in `oracle-crashes.txt:1121`).
`Message~result`/`~wait` of an unsent message: oracle blocks for ever (killed at 20 s, rc 137);
ours `a wait that nothing left to run can end is not implemented`, rc 120, **no phase named**
(`lib.rs:775-781`, `Loud::unsatisfiable_wait`). See open question Q2.

## 2. Message, EventSemaphore, MutexSemaphore methods

Oracle tables (exact lines): `interpreter/memory/Setup.cpp` Message `:1050-1082` (methods
`:1052`, `:1056-1070`, `:1073-1076`), EventSemaphore `:1325-1341` (`:1327`, `:1331-1335`),
MutexSemaphore `:1348-1362` (`:1350`, `:1354-1356`). The brief's `:1055-1076`/`:1325-1356` are close
but Message `New` is at `:1052`.

Cross-check (`E/rows.py` extracts dispatch rows into `E/dispatch-rows.txt`; `E/crosscheck.py`
extracts Setup.cpp rows and joins, output `E/crosscheck.txt`, re-run and diffed equal):
**30 methods, 0 missing**. Each has a row in `rust/crates/rexx-exec/src/dispatch.rs`:

| class | method | Setup.cpp | dispatch.rs row |
|---|---|---|---|
| Message | class NEW | 1052 | 1022 RESUMABLE_CLASS_METHODS |
| Message | COMPLETED | 1056 | 696 NATIVE_METHODS |
| Message | HASERROR | 1057 | 709 NATIVE_METHODS |
| Message | HASRESULT | 1058 | 715 NATIVE_METHODS |
| Message | NOTIFY | 1059 | 727 NATIVE_METHODS |
| Message | RESULT | 1060 | 952 RESUMABLE_METHODS |
| Message | TARGET | 1061 | 742 NATIVE_METHODS |
| Message | MESSAGENAME | 1062 | 721 NATIVE_METHODS |
| Message | ARGUMENTS | 1063 | 690 NATIVE_METHODS |
| Message | ERRORCONDITION | 1064 | 702 NATIVE_METHODS |
| Message | SEND | 1065 | 953 RESUMABLE_METHODS |
| Message | START | 1066 | 735 NATIVE_METHODS |
| Message | REPLY | 1067 | 728 NATIVE_METHODS |
| Message | SENDWITH | 1068 | 954 RESUMABLE_METHODS |
| Message | STARTWITH | 1069 | 736 NATIVE_METHODS |
| Message | REPLYWITH | 1070 | 729 NATIVE_METHODS |
| Message | MESSAGECOMPLETE | 1073 | 940 RESUMABLE_METHODS |
| Message | TRIGGERED | 1074 | 946 RESUMABLE_METHODS |
| Message | WAIT | 1075 | 964 RESUMABLE_METHODS |
| Message | HALT | 1076 | 708 NATIVE_METHODS |
| EventSemaphore | class NEW | 1327 | 982 RESUMABLE_CLASS_METHODS |
| EventSemaphore | UNINIT | 1331 | 680 NATIVE_METHODS |
| EventSemaphore | POST | 1332 | 670 NATIVE_METHODS |
| EventSemaphore | RESET | 1333 | 671 NATIVE_METHODS |
| EventSemaphore | WAIT | 1334 | 965 RESUMABLE_METHODS |
| EventSemaphore | ISPOSTED | 1335 | 664 NATIVE_METHODS |
| MutexSemaphore | class NEW | 1350 | 984 RESUMABLE_CLASS_METHODS |
| MutexSemaphore | UNINIT | 1354 | 756 NATIVE_METHODS |
| MutexSemaphore | RELEASE | 1355 | 750 NATIVE_METHODS |
| MutexSemaphore | ACQUIRE | 1356 | 966 RESUMABLE_METHODS |

Probes: one program per method (`E/probes/*.rex`, 31 files, `NOTIFY` twice: before and after the
send), run once each on both sides by `bash E/run.sh probes out` (`E/summary.txt`, outputs
`E/out/`). Result: 29 of 31 identical on all of stdout, stderr and rc, none answers `not
implemented`. The two that differed (`msg_reply`, `msg_replywith`) were a bad probe: `m~reply` answers
a started **copy**, so `m~wait` blocks for ever on the oracle (rc 137) and ours refused it as
`a wait that nothing left to run can end` (no phase). Rewritten to wait on the answered copy
(`E/probes2/msg_reply.rex`, `msg_replywith.rex`): identical (`E/summary2.txt`). `UNINIT` probes
rewritten as instructions (`E/probes2/ev_uninit.rex`, `mx_uninit.rex`): identical. A started
`Message~halt` (`E/probes2/msg_halt_started.rex`) shows the same lines and rc on both, in a
different stdout/stderr interleaving (one run; concurrent, no claim beyond "not refused").

**No Message or semaphore method is still refused. Nothing to build or re-home under this item.**

## 3. Recorded divergences and their exclusions rows

File: `docs/superpowers/plans/phase-4-exclusions.txt`, DEVIATIONS section from `:713`. Existing
Phase 6 rows (all `OWNER: none`): 9 `:1440-1466`, 10 `:1468-1516`, 11 `:1518-1528`, 12
`:1530-1546`, 13 `:1548-1572`. Searched for the others with
`grep -n -i -- KEY docs/superpowers/plans/phase-4-exclusions.txt` for `invert`, `immovable`,
`wrapper`, `busy`, `pinned`, `nested scheduler`, `nohup`, `callback`, `baton`, `reply`, `owner: none`
(`E/kw.sh`, output `E/kw.out`): no row for any of the six marked **add** below.

| divergence | row | status |
|---|---|---|
| stale C writes to reallocated lent storage | 9, `:1440` | exists, owner none (writes covered by "Rexx does not see its writes"; reads after a collection the measured part) |
| the `nohup` signal difference | 10, `:1468`, nohup at `:1474-1480` | exists, owner none |
| P63 SIGPIPE in ADDRESS children | 11, `:1518` | exists, owner none |
| P69 no activity runs during a stdin read | 13, `:1548` | exists, owner none |
| inverted pinned waits | none | **add** |
| immovable REPLY | none | **add** |
| wrappers blocking on the baton (beyond stdin, row 13) | none | **add** |
| HALT under nesting | none | **add** |
| a pinned busy-waiter (P31) | none | **add** |
| P57 callback waits for a baton-keeping call (and P55 pool-bound fallback) | none | **add** |

Measured for the new rows (30 runs each side, `E/probes5`, counts `E/out5/counts.txt`):

- `inverted.rex` (the `HIDDEN_INVERSION` program of `scheduler/tests.rs:125` with main's wait in
  INTERPRET): oracle rc 0, 30/30, two orders (23 and 7); ours 30/30 `s3 sent m0` then
  `rexx-exec: a pinned wait for a message's completion that only an activity pinned below it can
  end is not implemented`, rc 120.
- `immovable.rex` (`do label l; reply 1; leave l; end`): oracle `1` rc 0, 30/30; ours 30/30
  `rexx-exec: a REPLY its method body runs on a nested Rust frame is not implemented`, rc 120.
- `busy2.rex` (two busy-waiters in INTERPRET needing each other, `concurrency_tests.rs:679`'s
  program): oracle `A ended` / `B ended` rc 0, 30/30; ours 30/30 hangs until `timeout` (rc 124,
  three 4.1 HALT tracebacks from the SIGTERM). The 30 md5s differ only in the temp dir name in the
  traceback.

Row texts to add after row 13 (`:1572`), in the file's register:

```
 14. A PINNED WAIT ONLY AN ACTIVITY BELOW IT CAN END IS REFUSED (spec
     2026-09-29 2.6, rulings P23, P30). OWNER: none.

     A pinned activity that parks runs a nested scheduler round on its own
     stack. Where what it waits for can come only from an activity whose
     continuation lies below it, nothing the round can run ends the wait,
     and with nothing in flight it refuses: "a pinned wait for WHAT that
     only an activity pinned below it can end is not implemented", rc 120.
     The oracle runs every activity on its own thread and completes.

     Measured 2026-10-06, 30 runs each: scheduler/tests.rs HIDDEN_INVERSION
     with main's wait inside INTERPRET: oracle rc 0 in every run, ours the
     refusal after "s3 sent m0" in every run. Witnesses: rexx-exec's
     scheduler::tests::an_inverted_pinned_wait_is_refused and
     an_inversion_the_refusing_loop_set_aside_is_refused_as_inverted.

 15. A REPLY WITH RUST FRAMES INSIDE ITS METHOD BODY IS REFUSED (spec
     2026-09-29 section 5). OWNER: none.

     A REPLY moves the rest of its method to a new activity by moving its
     frames. Inside a non-flattened loop, INTERPRET or a CALL ON handler
     there are Rust frames between the body's driver and the REPLY, which
     cannot move: "a REPLY its method body runs on a nested Rust frame is
     not implemented", rc 120. The oracle continues on a new thread.

     Measured 2026-10-06, 30 runs each: `do label l; reply 1; leave l; end`
     prints 1 at rc 0 on the oracle and refuses here. Witness: rexx-exec's
     concurrency_tests an_immovable_reply_is_counted_with_its_frames.

 16. A WRAPPER THAT KEEPS THE BATON BLOCKS EVERY OTHER ACTIVITY (spec
     2026-09-29 2.1). OWNER: none.

     The stream builtins, SAY to a routed .output, trace output delivery,
     ADDRESS WITH redirection, condition handling, package loaders and
     Routine~call of a library routine run on nested Rust frames and keep
     the baton. While one blocks (a pipe, a console read) the interpreter's
     other activities wait; the oracle's run. Row 13 is the measured case of
     the default input stream. Counted by the pinning report.

 17. HALT UNDER NESTING WAITS FOR THE INNER REGION (spec 2026-09-29 2.6).
     OWNER: none.

     An activity that traps HALT and parks again while nested keeps the
     enclosing pinned activities waiting, and a HALT targeted at an
     enclosing activity is taken only when the inner region ends. The
     oracle's activities each have a thread and take it at their next
     clause.

 18. A PINNED BUSY-WAITER IS NOT PREEMPTED (spec 2026-09-29 2.6, rulings
     P29, P31). OWNER: none.

     A pinned activity's slices are deferred, and its second slice takes a
     pinned yield that runs others in a nested round on its stack. Two
     pinned busy-waiters that need each other never end: the inner one's
     yields cannot run the outer, buried below it. It is not refused, since
     a busy wait cannot be told from long pinned work; each round is counted
     as an inverted yield. The oracle preempts both and completes.

     Measured 2026-10-06, 30 runs each: two INTERPRETed busy-waiters on
     each other's flags print "A ended" / "B ended" at rc 0 on the oracle
     and run here until killed. Witness: rexx-exec's concurrency_tests
     pinned_busy_waiters_that_need_each_other_count_inverted_yields.

 19. A CALLBACK FROM A THREAD RUNNING NO NATIVE CALL WAITS FOR A
     BATON-KEEPING CALL (rulings P55, P57). OWNER: none.

     With the pool at its bound, or for a lone call, a native call keeps
     the baton. A callback into the interpreter from another thread then
     waits for that call to return; if the native joins that thread the
     run hangs, where the oracle aborts, rc 134 (30 runs). Both fail.
```

(Row 19's "30 runs" is P57's own figure, `progress.md:272`; I did not re-run it. Row 17 has no
witness I could find: `grep -rn 'fn .*halt.*\(nest\|pinned\|inner\|enclos\)'` over
`rust/crates/rexx-exec/{src,tests}` finds none.)

Thread migration and Error 11 depth on a pool thread (spec `:551-553`, "Both are recorded
divergences") also have no row; see Q3.

## 4. Task 24 Step 2 prep

### closed_phases

`rust/crates/rexx-exec/tests/closed_phases.rs:38` `const CLOSED: &[&str] = &["Phase 7", "Phase 8"];`
Negative control `:322-347` (doc :322-325, fn :327), line `:335`:
`if line.contains("\"Phase 6\"") || line.contains("\"Phase 10\"") {` asserting `open >= 2`.

Measured in a scratch `git archive` of HEAD (`E/scan.sh`, `E/scan.log`): CLOSED with `"Phase 6"`
added and `:335` reduced to `"Phase 10"` alone: **5 passed, 0 failed** (Compiling rexx-exec seen).

**Defect found: the exclusions check passes over an open Phase 6 owner.** The Alarm/Ticker row
(`phase-4-exclusions.txt:5641-5668`) says `OWNER: Phase 6` (`:5661-5662`) with no resolution, and
`no_open_exclusions_row_names_a_closed_phase` passes only because the row's own sentence `neither is
gated today because Phase 6 is not in `CLOSED_PHASES`` (`:5667`) contains `CLOSED`, which
`word_at` (`closed_phases.rs:291-296`) accepts as a resolution word since `_` is not in its
`joined` set. Shown by `E/scan2.sh` (`E/scan2.log`): the same scratch tree with that one
`CLOSED_PHASES` spelled `XLOSED_PHASES` fails with
`"OWNER: Phase 6, which the refusal itself names -- `alarm_startTimer` and `ticker_createTimer` are
native timer entry points its concurrency work has not built"`.

The implementer must:
1. `closed_phases.rs` `word_at`: count `_` as joined (`c.is_ascii_alphanumeric() || c == '-' ||
   c == '_'`), and add a case to `the_row_check_tells_an_open_owner_from_a_resolved_one`
   (`:315`), e.g. `H. OWNER: Phase 8.\n\nnot in `CLOSED_PHASES`.\n` expected open.
2. Alarm/Ticker row `:5641-5668`: add a DELIVERED note. Measured: construction answers
   `instance 1` on both (`E/out2/alarm_construct.*`, `ticker_construct.*`), and gate table C at
   HEAD reports `Alarm instance loud=no 6 7 row(s): agree=7` and `Ticker instance loud=no 6 6
   row(s): agree=6` (`E/gate_table_c-alarm-ticker.txt`, `cargo test --release -p rexx-exec --test
   gate_table_c` in the scratch tree, 22 passed). The `CLOSED_PHASES` sentence becomes false
   once 6 is closed.
3. CLOSED gains `"Phase 6"`; `:335` counts `"Phase 10"` alone; the doc comment at `:322-325`
   unchanged in meaning.
4. `rust/crates/rexx-exec/tests/internal_routines.rs:107` `PHASES = ["Phase 6", "Phase 10"]`:
   drop `"Phase 6"` (no internal routine row names it: command 1 empty).
5. Other exclusions prose naming Phase 6 (found by `grep -n -i 'phase 6'` and
   `grep -P -z -o '[Pp]hase\s+6\b' ... | wc -l` = 11 occurrences):
   - `:4077` GUARDED directive option "That is Phase 6's": now real. Measured 30 runs each
     (`E/probes4/guarded.rex`, `E/guarded-30.txt`): two guarded bodies never overlap on either side,
     unguarded ones interleave; oracle 4 orders (16/9/3/2), ours 30/30 the oracle's most common.
     Needs a DELIVERED note citing `corpus/lang` witnesses of Phase 6 S3 Task 11
     (`phase-8.txt:496-508`).
   - `:4739` "each naming Phase 9 or Phase 6": false, `REFUSING_MEMBERS`
     (`rexx-api/src/layout.rs:378-384`) names Phase 9 only. Delete "or Phase 6".
   - `:4706-4716`, `:4930`, `:4945`: history already resolved ("AT PHASE 6 S3", "FIXED"). Keep.
6. gate table C: `rust/crates/rexx-exec/tests/gate_tables/mod.rs:195-197` `CLOSED_PHASES` must
   gain `"6"` in the commit that creates `phase-6.txt`: class rows Alarm and Ticker carry owner 6
   (`rust/corpus/docs/class-set.txt:75`, `:103`), and
   `gate_table_c.rs:386` `every_closed_phase_this_table_owns_rows_for_is_gated` fails for an owner
   with a `phase-{owner}.txt` not in `CLOSED_PHASES`. Their rows agree today (item 2).

### phase-8.txt to phase-6.txt (P17)

P17 is in the S0/S1 workspace: `.superpowers/sdd/2026-09-29-phase-6-s0-s1/progress.md:86`, not in
`$W/progress.md`.

`rust/corpus/phase-8.txt` lines 366-557 (to end of file) are the Phase 6 block, starting at
`# Phase 6 Task 8: Rexx-to-Rexx calls ... Filed here because a phase-6.txt would count phase 6 as
closed to gate table C.` Command: `awk 'NR>=366 && !/^#/ && NF' rust/corpus/phase-8.txt | wc -l`
= **132 programs** (31 comment headers); lines 1-365 hold 139 programs, all Phase 8 (headers
`Task 8`..`Final fix round (I4)`, `:14-364`). Every header from `:366` names Phase 6 work (Tasks
8-10 of S0/S1, S2-S5 Tasks 2-14 and their fix rounds); `grep -n -i 'phase 6'` over every
`rust/corpus/*.txt` finds no other phase-file block (`oracle-crashes.txt` hits are history).
Move 366-557 with its headers; drop the "Filed here because ..." sentence.

The phase-file lists that must gain `"phase-6.txt"` (each asserted equal to the files on disk or
read explicitly): `tests/corpus.rs:574-583` `SUBSET_FILES` (checked by
`the_differential_reads_every_phase_subset_file`), `tests/ir_recorded.rs:818-827`,
`tests/coverage.rs:497-506`, `tests/trace_oracle.rs:457-466`,
`tests/collect_stress.rs:54-63`. `src/ir/corpus_shape_tests.rs:497` and
`corpus.rs:594` glob `phase-*.txt`. `src/dispatch/library/tests.rs:177` documents that
`collect_stress` reads `phase-8.txt` last; check its reasoning still holds if `phase-6.txt` sorts
before `phase-7.txt`.

## Open questions for the lead

- Q1. Rows 17 (HALT under nesting) and 16 (wrappers) have no witness in the suite and I did not
  build one. Accept rows without a witness (as row 11 does), or require one?
- Q2. `Loud::unsatisfiable_wait` (an unsent message's `~result`/`~wait`, and any deadlock): the
  oracle blocks for ever (20 s, rc 137, one run each), ours rc 120 naming no phase. Recorded only
  in `oracle-crashes.txt`. A DEVIATIONS row too?
- Q3. Spec `:551-553` (activities migrate between OS threads; Error 11 depth on a pool thread
  depends on scheduling) and the program-end rulings P39/P40, P53's leak, P38/P41 scheduling
  licences: none has a DEVIATIONS row. In or out of criterion 8's list?
- Q4. The two comments (`run.rs:2375`, `handle.rs:50`) do not trip `closed_phases`. Reword them
  for criterion 8's "gone", or leave as history?
