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
is dirread +0.0455%; every other program is at most +0.0013% or falls. Whole process, pkglocal is
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

