# Phase 9d Task 8.3 report: call sites, frames, routines, context

Status: DONE_WITH_CONCERNS. Base `80f0fab1c`. Code `68e7f15d5`, builtin-path perf and the define
bench `8d409845b`, records in the commit that adds this report. Records:
`docs/superpowers/records/2026-10-18-phase-9d/t8.3/`.

## What changed, per item

**2026-10-21-call-site-reads-kept-before-arguments.md (both shapes) and
2026-10-21-raise-in-routine-keeps-call-site.md.** One rule now governs every call op: a label or a
builtin is kept where it resolves (both are fixed at parse time on the oracle); anything else is a
`SiteKeep` the site takes only once the call returns, `RexxInstructionCall::execute` and
`RexxExpressionFunction::evaluate` setting `externalTarget` after `externalCall` comes back. The ops
that evaluate their own arguments (`Op::CallExpr`, `Op::Call`) read the site after the arguments
(`settle_after_arguments` re-reads the slot), so an argument that calls through the same site and
returns is seen by the outer call (Q5). Nothing is ever forgotten any more: a call that raised
simply never kept, so an outer raise cannot wipe what an inner call kept (Q6, the Task 7.1
regression). "Returns" is the callee's own end, before the function form's no-data check: measured,
the oracle keeps a routine that answered nothing to a function call (44.1 twice). The keep crosses
an entered body on `Activity::entered_keep`, which the op claims immediately (`park_call` into
`ParkedCall::keep`, or `run_entered` on the nested path); the resume paths (`resume_call`,
`resume_region`, `resume_woken`) end it with the callee's outcome. A pending keep's copy is pinned
until the call ends. `forget_site`, `Calls::forget` and `CallSite::clear` are gone.

**collect_stress.** The two Task 7.1 programs were red because under collect-on-every-allocation a
native library call parks (`native_park`) and its raise comes back through `resume_woken`, which
never reached the old forget; the plain run raised directly. Keep-on-return covers that path.
collect_stress passes (`checks-touched.txt`, 72 passed).

**2026-10-17-routine-call-frame-name.md, gate:phase-9c-gate.md:143, 2026-10-21-routine-new-call-frames.md,
2026-10-01-routine-call-context-name.md.** `Routine~call`, `~callWith` and `~[]` run under the
routine's own name (`executableName`, `RoutineClass.cpp:205-211`): a directive's name as the AST
holds it, the name `.Routine~new`/`newFile` was given, or for the main program's executable its path.
A sent primitive method entered through a `NativeTail` begin half (`NEW`, `SEND` and `SENDWITH` of
`Object` and `Message`, and `Routine`'s `CALL`, `CALLWITH` and `[]`) is now listed by
`.context~stackFrames` behind the activation it entered (`LiveLevel::Entering`, from the native tail,
which now records the activation it entered); witnessed for `NEW`, `SEND`, `SENDWITH`, `CALL`,
`CALLWITH` and `[]`. Condition frames already listed these at base. Primitives that run a Rexx body
nested rather than through a begin half (`RUN`, `SORT`, `HASITEM`, `REQUEST`, `PUT`) are still
missing from the live list, as at base. The main program's executable run by
`~call` runs synchronously, so a `MethodLevel` stands for its `CALL` frame there. The restored
routine: the image format is version 2 and carries the routine's name after the header (4-byte length
and bytes); `rexxc` writes the path it translated, and a routine installed from a restored source
takes that name (`install_executable`, `record_compiled_routine`). Witnessed in
`tests/compiled/cases/restore` (`names.rex`), which holds the oracle's block. The known-gap paragraph
beside Deviation 41 in `phase-4-exclusions.txt` is deleted.

**2026-10-21-copied-package-routine-executable.md.** `Resolved::MergedInternal` carries the
`Routine` the lookup found (`.nil` for the canonical one), `InternalLevel` holds it, and the frame and
the failure blame answer it; `copy~call` passes its receiver. The copy is rooted while its level runs.

**2026-10-21-main-program-executable-annotations.md.** The main program's `Routine` object gets an
empty annotation table when it is built.

**2026-10-19-context-name-during-install.md.** The `::CONSTANT` activation records its call
convention, so `.context~name` is `::CONSTANT` and `.context~args` empty. A package prologue now runs
under the package's own name and no arguments (`run_prologue`, `PackageClass::runProlog`,
`PackageClass.cpp:2128-2137`), for a `::REQUIRES` and for `.Package~new`; it used to inherit the
caller's name and arguments (`arg()` 1 in the prologue where the oracle has 0).

**2026-10-15-9b1-task-8b-review-minors.md.** m4: `Interp::method_object` checks that the object
cached under a class and name runs the method asked for; an instance made before a `define` gets the
object standing for its own (replaced) method, Class~method's earlier answer included. m1 (a variable
list naming a `USE ARG >z` alias) is Task 8.5's, left untouched. m5: `bench-programs/define.rex` added
(registered in `rexx-bench`, README row). Measured (`probes/define-profile.txt`):
`free_unheld_behaviours` is 7.38% inclusive, beside `define`'s own behaviour builds
(`rebuild_behaviour` 8.63%, `ClassGraph::define` 7.81%). The HashSet is not the cost: replacing it by a
sorted candidate list moved the pass from 125.08M to 124.24M instructions, so that change was not
kept. The cost is listing and clearing the freed behaviours' full dictionaries (every behaviour holds
its inherited methods), which a cheaper pass would have to change. Proposed owner: the performance
round. The file stays open (m1).

**2026-10-15-isguarded-copy-and-additional.md.** `.context~executable` in a method an `enhanced`
instance holds is the object that instance's behaviour holds (the one `instanceMethod` answers),
found by the receiver's own lookup before the "first minted" fallback. `RAISE SYNTAX ... ADDITIONAL`
with a string requests its array as `requestArray` does: the string's lines.

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file (generated by
the test's driver from copies outside the repository):
`call_site_reentry_reads_after_arguments`, `call_site_reentry_raise_keeps` (both with the
`orxfunction` `.env`), `call_site_routine_raise_forgets`, `routine_call_frame_names` (with `.d`
fixtures), `routine_copy_internal_executable`, `main_executable_annotations`, `context_name_install`
(`.d`), `instance_method_after_define`, `enhanced_unguarded_copy_executable`,
`raise_syntax_string_additional`. `witnesses/summary.tsv`: every one differs from the oracle at base
(a build of `80f0fab1c` in a scratch worktree) and is identical on all three descriptors at head; the
per-side descriptors are beside it. The compiled case: `witnesses/compiled-names.base.txt` against
`.head.txt` (base names the routine `CALL`). The two Task 7.1 programs are red at base in
collect_stress (Task 8.2's records) and green here.

I read each witness's oracle stdout for the paths it claims: the call-site witnesses print the
re-entry lines (`answered 2`, `answered 3`) and the 44.1-twice / `call answered RESULT` keep lines;
`routine_call_frame_names` prints the `METHOD CALL`/`CALLWITH`/`[]`/`NEW`/`SEND`/`SENDWITH`/`SORTWITH`
frames and every name form; `context_name_install` prints `9 ::CONSTANT 0`, `prolog 0 [] 0 q.cls`
and `pk 0 pk`.

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t83-target`, at
`8d409845b` unless stated.

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-parse -p rexx-bench --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test collect_stress --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test compiled --test rexxc` (`checks-touched.txt`) | 0: lib 1175, collect_stress 72, signals 42, dispatch_seam 6, compiled 2, rexxc 4, every table binary passed |
| `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites` | 0, 2 passed |
| `memcap 16G cargo test -j 4 --release -p rexx-bench --lib` | 0, 30 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 29 passed, 1 ignored |
| `memcap 16G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` (`checks-srcl.txt`, at `68e7f15d5`'s tree) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.3/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` at `68e7f15d5` (`suite/`) | 0; against `t8.2/r1/suite/tests.tsv` the one changed row is `base/rexxutil/SysFileTree TEST_TREE_DIRECTORY`, oracle column `failed` |
| the same at `8d409845b` (`suite-final/`) | 101; the one changed row is `TEST_TREE_DIRECTORY`, crate column `failed`: the known flake, on the other side |

## Perf

Replaced in fix round 1. `r1/callgrind-final.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o
.../cg-9d/t8.3-r1-final base=.../cg-9d/t8.3-r1/rexx-base head=.../cg-9d/t8.3-r1/rexx-head-3d9cc2da0`,
every program in the script's `PROGRAMS` (`define` included), BASE `efe8d3f40` against HEAD
`3d9cc2da0`, 3 rounds. Exit 1, from `SPREAD` alone: pingguard (head 0.0145%) and pingsem (base 0.0157%,
head 0.0159%), schedule-dependent on both sides. Head against base, libc subtracted, the largest rise
is dirread +0.0455%, then interpret +0.0235%; every other program is at most +0.0013% or falls. Whole process, pkglocal is
+0.4170% and compound +0.0592% with libc subtracted 0.0000% for both. The full table is in the record.

## Concerns

1. **Pre-existing collect_stress drift.** `NO_ALLOCATION_PROGRAMS` named programs that already collect
   at base (`probes/collections-base-head.txt`: `do_to_class` 37 collections at base and head; its
   first is its error report's `TraceObject`, `deliver_trace_line`, read from a backtrace). Base
   never reached that assertion because the divergence assertion above it failed first. I removed the
   programs that collect; the remaining list is exactly what the run observes. Three traced programs
   and `mutation_controlled_order` collect 4 more here than at base; I did not find which allocation.
2. **The site keep is an oracle property, not a rexxref rule.** rexxref's Search Order
   (`oodocs/rexxref/en-US/funct.xml:205-300`) gives the order a call searches and says nothing of a
   call keeping what it found. This task follows the item's text and
   Task 7.1's design (the oracle's `externalTarget`); no Deviation candidate is proposed.
3. **Residual, not queued: `::CONSTANT`'s frame and executable.** `probes/constant_frames.rex`: the
   oracle lists `METHOD ::CONSTANT 5` then `PROGRAM <file> 3` and `.context~executable` is a
   `Method`; this crate lists `PROGRAM ::CONSTANT 0` alone and answers a `Routine`. Proposed owner:
   Task 8.4 (install order, directives), or a queued item.
4. **Exclusions file edits.** Beside the known gap, two entries are deleted from
   `phase-4-exclusions.txt`: "A ::routine WHOSE CALL RAISED STAYS KEPT" (fixed here;
   `probes/routine_raise_addroutine.rex` differs at base, identical at head) and "A LIBRARY ROUTINE'S
   Routine~call TRACEBACK ALSO LACKS ..." (false at base already: `probes/library_routine_call_traceback.rex`
   is identical to the oracle with the base binary).
5. **A `ParkedCall` dropped without being resumed** would leave its pending copy pinned. No path drops
   one: `parked_calls` is popped only by the resume arms, which end the keep.

## New oracle defects

None found.

## Fix round 1

Brief `task-8.3-fix-r1.md`, review `task-8.3-review.md`. Code `262b6c72c`, `3d9cc2da0`; records
`t8.3/r1/`.

* **I1.** `define` is in `callgrind.sh`'s `PROGRAMS` (`262b6c72c`). Run end to end against BASE
  `efe8d3f40` (`r1/callgrind.txt`), the script showed what the five-program check had not covered:
  libc subtracted, intcall +2.83%, extcall +1.60%, fibfunc +1.45%, sendloop +0.98%, fibcall +0.69%,
  dispatchclass +0.70%, dispatch +0.65%. Causes, from `cgdiff.py`: the unkept function call went
  through one more non-inlined layer (`begin_call_over_values` taking an `Option` keep, then
  `function_started` and `function_value` outlined), `begin_subroutine_over_pushed_args` stopped
  being inlined into `begin_call_named`, and `ParkedCall` grew by the keep, which every parked send
  and call copied. `3d9cc2da0` puts the unkept path back to its base shape (the kept forms are cold
  functions of their own; `begin_call_over_values`, `begin_subroutine_over_pushed_args`,
  `function_started`, `function_value` and `run_internal_as` are `#[inline(always)]`) and moves a
  parked call's keep to `Activity::parked_keeps`, filed under its parked call's index and checked
  only where a call op resumes. The Perf section above is that run.
* **M1.** The frame claim now names the primitives entered through a `NativeTail` begin half, and the
  nested ones still missing from the live list.
* **M2.** `executable.rs` `begin_routine`'s doc and `install.rs`'s `::CONSTANT` comment are wrapped
  to 100 columns (`262b6c72c`); no added source line in `crates/*/src` is wider.

Checks at `3d9cc2da0`, from `rust/`: `cargo fmt --all --check` 0 (`r1/checks-fmt.txt`); `memcap 16G
cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` 0 (`r1/checks-clippy.txt`); the touched
binaries as in the first round, 0 (`r1/checks-touched.txt`: lib 1175, collect_stress 72, signals 42,
compiled 2, rexxc 4, the rest passed); `REXX_CORPUS_GATE=1 ... --test corpus` 0, 29 passed
(`r1/checks-corpus.txt`); the whole suite 0, no changed row against `t8.2/r1/suite/tests.tsv`
(`r1/suite/`).


## Fix round 2

Brief `task-8.3-fix-r2.md`, re-review `task-8.3-rereview-r1.md`. Code `2afe1e16c`, `3e6dc0426`,
`1bb283e1a`, `03a2a1e1a`; records `t8.3/r2/` (`head-3e6dc0426/` and `head-1bb283e1a/` are the
finishing runs; the files beside them at `r2/` are re-run at the final HEAD). Interrupted commits:
`2041eb468` (rooting the kept routine's package) and `d6dee4ef4` (its saved draft) are superseded in
place by `2afe1e16c`; `3e6dc0426` (the `SiteHold` refactor, saved unchecked) is kept, checked below.

* **Q-A.** A call site keeping a `::ROUTINE` of another program makes the package of the program it
  runs in hold that routine's package (`PackageData::site_held`, traced as an edge,
  `Interp::hold_site_program`), counted per site and dropped when the site keeps something else or
  when an `INTERPRET` fragment's chunk is dropped. Rooting instead leaked routines whose sites keep
  each other: 677 MB at 10000 iterations and OOM at the 2G cap at 100000, where the oracle stays at
  21 MB (`r2/root-pin-cycle/cycle.txt`).
* **`3e6dc0426`, kept.** `SiteHold { holder, program }` replaces the holder/program pairs and
  `Chunk::owner`; `site_keep` takes the holder from the calling activation, whose `body_key` is the
  key the running chunk is cached under, so it names the program `owner` named. `Calls::pins` is gone.
  At `2afe1e16c` the full callgrind run had nop +4.36%, assign +2.72%, varlookup +2.03%, emptyloop
  +1.30%, textnum +1.03%, compound +0.89% and rexxcps +0.52% libc subtracted
  (`cg-9d/t8.3-r2f/callgrind.txt`); at this commit each of those is within +0.0001% of base
  (`head-3e6dc0426/callgrind.txt`, the same run as below on `3e6dc0426`).
* **`1bb283e1a`.** `run_fragment` tested every slot of the fragment for a hold
  (`Chunk::holds_programs`), 13 instructions per `INTERPRET` in `run_fragment` by `cgdiff.py`, which
  put interpret at +0.0552% libc subtracted. It now tests `Interp::site_hold_count`, an upper bound
  `hold_site_program` keeps. A `Cell<bool>` on `Calls` instead brought nop back to +4.36%, assign
  +2.72%, varlookup +2.03% (`head-1bb283e1a/try-holdsflag.txt`; the count, `try-counter.txt`).
* **`03a2a1e1a`.** `run::tests::an_interpret_fragments_site_holds_end_with_it`: a main-body site
  holds one program after the loop, a fragment's site none. With `run_fragment`'s test back to
  `!pinned_copies.is_empty()` alone it fails (3 holds left, `head-1bb283e1a/unit-test-mutant.txt`),
  the growth probe's `interp` shape reaches 338 MB at 10000 iterations and OOM at 100000
  (`growth-mutant.txt`), and collect_stress stays green (74 passed), so nothing else saw it.
* **M1.** `site_call` asserts at the filing that no keep is filed at or above the new index
  (`run/call.rs:599-606`). Debug build of `3e6dc0426` with `resume_call`'s `end_parked_keep` call
  deleted: `tripwire/t2.rex` panics "an entered call's keep was left unclaimed", rc 101; unmutated it
  prints `1`, `2`, `end`, rc 0 (`head-3e6dc0426/tripwire.txt`).

**Witnesses.** `corpus/lang/call_site_kept_routine_collected.rex` (in `phase-9.txt`, with its
`sourceline_oracle` file) and collect_stress `a_routine_only_a_call_site_keeps_survives_collection`
and `routines_whose_sites_keep_each_other_survive_collection`. At `9963948e3` (release, scratch
worktree): the corpus program prints `f 1`, `f 2`, then "an activation's body selector names no
routine body", rc 120; both stress tests, copied into that tree, fail with rc 120
(`head-3e6dc0426/stress-red-9963948e3.txt`). At `1bb283e1a` all three descriptors match the oracle
(`head-1bb283e1a/witness/`); its stdout prints `f`, `g`, `h`, `m`, `s`, `i` 1 to 3 and `a 0` three
times. Per form (`head-1bb283e1a/forms.txt`): the internal function's site, the main program's
function call, `CALL`, a method's site, a `sortWith` comparator's site (a nested drive under a
native), the mutual cycle and a site reached from an `INTERPRET` then from the main program (`ix`)
differ at `9963948e3` and match at head. A site inside the fragment itself (`interpret`, `iv`)
matches at `9963948e3` as well.

**Growth** (`head-1bb283e1a/growth.txt`, the re-review's shapes: site, raise, nested, interp, cycle,
native; maxrss at 10000 and 100000 iterations): no shape rises by more than 0.04 MB.

**Checks**, from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t83f2b-target/head`:

| command | at | result |
|---|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | `03a2a1e1a` | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-parse -p rexx-bench --all-targets -- -D warnings` (`checks-clippy.txt`) | `03a2a1e1a` | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test collect_stress --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test compiled --test rexxc` (`checks-touched.txt`) | `1bb283e1a` | 0: lib 1175, collect_stress 74, signals 42, dispatch_seam 6, compiled 2, rexxc 4, every table binary passed |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --lib` (`checks-lib.txt`) | `03a2a1e1a` | 0, 1176 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | `1bb283e1a` | 101: 1182 of 1183, the one mismatch `rexxutil_file_date_time.rex` (concern 3) |
| `memcap 16G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` (`checks-srcl.txt`) | `1bb283e1a` | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.3/r2/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | `1bb283e1a` | 0; `tests.tsv` and `table.tsv` identical to `t8.2/r1/suite/` |

**Perf** (`r2/callgrind.txt`): `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.3-r2b/final base=.../t8.3-r2b/rexx-base-efe8d3f40
head=.../t8.3-r2b/rexx-head-1bb283e1a`, every program, 3 rounds, both binaries built here (base from
a scratch worktree at `efe8d3f40`). Exit 0, no SPREAD or OVER flag. `03a2a1e1a` adds only a unit
test. Delta of head against base, percent:

| program | whole | libc subtracted |
|---|---|---|
| alloc | +0.3348 | +0.0000 |
| alloc4c | -0.0806 | -0.0962 |
| arith | -0.0111 | +0.0000 |
| assign | +0.0000 | +0.0000 |
| churn | -0.1112 | -0.0788 |
| compound | +0.0707 | +0.0000 |
| decloop | -0.0003 | +0.0000 |
| decrender | -0.0519 | -0.0579 |
| define | -0.0152 | +0.0068 |
| dirread | -0.0131 | +0.0380 |
| dispatch | -0.0227 | -0.0242 |
| dispatchclass | +0.1217 | -0.0260 |
| emptyloop | +0.0001 | +0.0000 |
| extcall | -0.4936 | -0.5187 |
| fibcall | -0.2824 | -0.2083 |
| fibfunc | -0.4002 | -0.2340 |
| hashnew | -0.1057 | +0.0000 |
| heapshape | -0.0009 | +0.0000 |
| intcall | -0.1420 | -0.1754 |
| interpret | -0.2264 | +0.0447 |
| nop | +0.0001 | +0.0000 |
| parse | -0.1093 | -0.1160 |
| pkglocal | -0.0004 | +0.0000 |
| sayloop | +0.0061 | +0.0005 |
| sendloop | -0.0343 | -0.0361 |
| startup | +0.0079 | +0.0009 |
| strings | -0.1921 | -0.2063 |
| textnum | -0.1000 | -0.1045 |
| varlookup | +0.0001 | +0.0000 |
| rexxcps | -0.0277 | -0.0612 |
| pingguard | -0.1974 | -0.2249 |
| pingmsg | -0.1959 | -0.0083 |
| pingsem | -0.2134 | -0.2374 |

The largest rise with libc subtracted is interpret at +0.0447%, then dirread +0.0380%. Against
`3d9cc2da0` (fix round 1's head, `r1/callgrind-final.txt`) only interpret rises with libc subtracted.

### Concerns (fix round 2)

1. **The fragment's own site is not a red witness.** A site inside an `INTERPRET` string keeping the
   routine matches the oracle at `9963948e3` too (`forms.txt`, `interpret` and `iv`); the hold it takes
   is witnessed by the unit test and the growth mutant, not by a program that fails at the red commit.
2. **The driver's codegen moves with `Calls`.** One `Cell<bool>` field and its store in
   `Calls::remember` cost nop +4.36% libc subtracted, and `2afe1e16c`, which still had
   `Chunk::owner` and `Calls::pins`, showed the same figure. Proposed owner: the performance round.
3. **`rexxutil_file_date_time.rex` fails the corpus gate today, at base as well.** Line 9, the epoch
   of the repeated reading `2020-10-25 02:30:00`: the crate answers 1603585800 and the oracle
   1603589400, with the `efe8d3f40` build and with head (`head-3e6dc0426/date-time-diff.txt`). Run on
   2026-10-26, the day after the European DST end; the round 1 gate passed on 2026-10-22. Not from
   this round; proposed owner: the controller's queue.
4. **collect_stress `the_l0_subset_passes_again_under_collect_on_every_allocation` is timing-flaky.**
   At `3e6dc0426` it failed in 2 of 3 runs on `command_redirected_target_waits_elsewhere.rex`
   (`guard 0 3 waited 1` under stress where the plain run printed `waited 0`, a `time('e') > 0.9`
   test); at `1bb283e1a` it passed in its one run. At `efe8d3f40` it failed in 3 of 3 runs, the third on that program
   (`head-3e6dc0426/l0-subset-base.txt`; the first two runs' failing program was not captured).
5. **`site_hold_count` stays above zero once a program dies with holds.** `Interp::prune_program_rows`'s
   retain drops a dead program's chunks without releasing their holds (the holding package is gone
   with them), so later `INTERPRET` ends take the cold `unpin_chunk_copies` pass. Cost only.

### New oracle defects (fix round 2)

None found.
