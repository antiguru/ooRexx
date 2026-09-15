# Phase 8 docs-pass and surface-plan review

Reviewer: read-only, no subagents. Worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`, HEAD `476347f52`. A gate run is
live in `rust/target/`; nothing was built there and no writing `git` command
was run. Crate runs, where needed, use a `git archive 476347f52 rust` copy in
the scratchpad with its own `CARGO_TARGET_DIR`.

Scope 1: commits `313807bc5..6be99d610` (the documentation pass; brief at
`docs-pass-brief.md`, report at `docs-pass-report.md`).
Scope 2: commit `2e0590b40` (the amended surface plan
`docs/superpowers/plans/2026-09-14-phase-8-surface.md`, D-L2 and row 8 in the
roadmap).

Each finding carries: severity, file:line, what, the command and its output,
and whether the check was run or inferred. Findings are appended in the order
found; the summary at the end is written last.

## Findings

Resumed after an API session limit. HEAD is now `6a167e462`; the gate run has finished, so
`rust/target/release/rexx-run` (built from HEAD's code) is usable. Commits `5f09e0e7b` and
`6a167e462` (section 8 of `phase-8-gate.md`) join scope 1.

### R1. Important -- the roadmap `:2638` citation was moved again, by the surface-plan commit (I3 repeated)

`docs/superpowers/plans/2026-07-27-rust-rewrite.md:254` and
`docs/superpowers/specs/2026-09-14-phase-8-scoping.md:167` both cite the section 6 Phase 8 bullet
at `:2638`. The docs pass re-printed it at `6be99d610` and it landed there. Commit `2e0590b40`
then inserted the D-L2 ruling block (9 lines added, 1 deleted; `git show 2e0590b40 --numstat`)
at `:566-573`, above the bullet, so at HEAD:

    $ sed -n 2638p docs/superpowers/plans/2026-07-27-rust-rewrite.md
      What Phase 7 has to decide, each already read out of the draft rather than left to be discovered:
    $ /bin/grep -n 'rebuilds `testbinaries/` unchanged' docs/superpowers/plans/2026-07-27-rust-rewrite.md
    2646:- **Phase 8** rebuilds `testbinaries/` unchanged against the frozen headers. ...
    $ git show 6be99d610:docs/superpowers/plans/2026-07-27-rust-rewrite.md | sed -n 2638p | cut -c1-60
    - **Phase 8** rebuilds `testbinaries/` unchanged against the

Run. Both citations are off by eight lines. The same commit added to the surface plan the
sentence "Check every citation by printing that line, including citations into files this plan
itself edits: the roadmap line a decision cited moved under the phase's own insertions"
(`2026-09-14-phase-8-surface.md:113-114`), and is the commit that moved it. Fix: cite the bullet
by its text or by section, not by line, in both places; a line number into a file the phase keeps
inserting into cannot stay true.

### R2. Important -- "`build/lib` holds one product per `testbinaries/` target" is false; two targets are executables in `build/bin`

Stated three times: `2026-09-14-phase-8-scoping.md:113-114` ("the oracle checkout's `build/lib`,
which holds one product per `testbinaries/` target"), `phase-4-exclusions.txt` (the FIVE OF THE
EIGHT entry: "oracle checkout's build/lib, which holds one product per testbinaries/ target"),
and the surface plan Task 7 Step 1 ("the oracle's `build/lib` holds every `testbinaries/`
product").

    $ /bin/grep -nE "add_(library|executable)" testbinaries/CMakeLists.txt
    53:   add_library(${target} SHARED        # orxfunction, orxmethod, orxclassic, orxclassic1
    74:add_library(orxexits SHARED
    86:add_library(orxinvocation SHARED
    95:add_executable(rexxinstance
    107:add_executable(provoke_locks
    $ ls /home/moritz/dev/repos/ooRexx/build/lib/liborx*.so    # six libraries
    $ ls /home/moritz/dev/repos/ooRexx/build/bin/ | grep -E "rexxinstance|provoke_locks"
    provoke_locks
    rexxinstance

Run. `build/lib` holds the six library products; `rexxinstance` and `provoke_locks` are in
`build/bin`. The docs-pass report (`docs-pass-report.md:70-72`) has it right ("`build/bin` holds
`provoke_locks` and `rexxinstance`: one product per `testbinaries/` target") and the documents
compressed it wrong. The partition conclusion does not depend on it; the sentence does, and the
surface plan asks Task 7 to record it "with the commands".

### R3. Minor -- "both import the function, subcom, queue and macro-space registries" is true of the pair and false of each

`2026-09-14-phase-8-surface.md:107-108`: "`CLASSIC` loads `orxclassic` and registers `orxclassic1`
through `rxfuncadd`, and both import the function, subcom, queue and macro-space registries".

    $ nm -D --undefined-only /home/moritz/dev/repos/ooRexx/build/lib/liborxclassic.so | grep ' Rexx' | awk '{print $NF}' | tr '\n' ' '
    RexxAddMacro RexxAddQueue RexxAllocateMemory RexxClearMacroSpace RexxClearQueue RexxCreateQueue RexxDeleteQueue RexxDeregisterFunction RexxDropMacro RexxFreeMemory RexxLoadMacroSpace RexxOpenQueue RexxPullFromQueue RexxQueryFunction RexxQueryMacro RexxQueryQueue RexxQueueExists RexxRegisterFunctionExe RexxReorderMacro RexxSaveMacroSpace RexxVariablePool
    $ nm -D --undefined-only .../liborxclassic1.so | grep ' Rexx' | awk '{print $NF}' | tr '\n' ' '
    RexxDeregisterFunction RexxDeregisterSubcom RexxQueryFunction RexxQuerySubcom RexxRegisterFunctionDll RexxRegisterFunctionExe RexxRegisterSubcomDll RexxRegisterSubcomExe

Run. `liborxclassic.so` imports no `*Subcom*` symbol; `liborxclassic1.so` imports no queue or
macro-space symbol. The spec's and exclusions' wording ("`orxclassic` and `orxclassic1`, which
import ...") reads as the union and is fine; "both import" is per-library and false. The docs-pass
brief (`docs-pass-brief.md:66-67`) carried the same "both importing".

### R4. Minor -- scoping §5 keeps `dispatch/native.rs:217` for `handle_set`, which now lands on `implemented(`

`2026-09-14-phase-8-scoping.md:150`: "`handle_set` (`dispatch/native.rs:217`) ... re-homed to
Phase 10 at `08d232ecc`". The docs pass edited this bullet and kept the citation.

    $ sed -n 217p rust/crates/rexx-exec/src/dispatch/native.rs
        implemented(
    $ sed -n 223p rust/crates/rexx-exec/src/dispatch/native.rs
        deferred("handle_set", Family::Stream, "Phase 10"),

Run. The pass's own report (`docs-pass-report.md:101`) prints `:223`; the document was not
updated to it. The brief required every citation in an edited paragraph to be printed.

### Checks that held (scope 1, partition and citations)

Run, all landing as written unless noted:

* Every `.testGroup` under `ootest/ooRexx/API` calls `.context~package~loadPackage(...)` at its
  line 52; the five `.cls` files each name one library (`orxclassic`, `orxmethod` x2,
  `orxfunction`, `orxinvocation`); `CLASSIC.testGroup:822,861,875` are the three `rxfuncadd
  ... 'orxclassic1'` calls.
* `readelf -d`: `liborxmethod.so`, `liborxfunction.so` NEED `libc.so.6` alone; `liborxinvocation.so`
  NEEDs `liborxexits.so`, `libc.so.6`; `liborxexits.so` NEEDs `librexx.so.4`, `librexxapi.so.4`,
  `libc.so.6`; `liborxclassic.so` NEEDs `librexx.so.4`, `librexxapi.so.4`, `libc.so.6`;
  `liborxclassic1.so` NEEDs `librexxapi.so.4`, `libc.so.6`. `nm -D --undefined-only`: no `Rexx*`
  in `liborxmethod`, `liborxfunction`, `liborxinvocation`; `liborxexits` imports
  `RexxCreateInterpreter RexxDeregisterExit RexxDeregisterSubcom RexxFreeMemory RexxQueryExit
  RexxRegisterExitDll RexxRegisterExitExe RexxRegisterSubcomExe RexxStart RexxVariablePool`.
* `diff -rq` of `api/`, `testbinaries/`, `extensions/rxregexp` against
  `/home/moritz/dev/repos/ooRexx`: all three empty, rc 0.
* `RUNPATH` on the oracle's `bin/rexx` and `lib/librexx.so.4`:
  `[/home/moritz/dev/repos/ooRexx/build/lib:]`, the empty element present.
* `ThreadContextStubs.cpp:1863/1875/1887` are `RaiseException0/1/2`, `:1897` the closing brace,
  `:1899` `APIRaiseException`. `oorexxapi.h:635-638` declares the four. `rxregexp.cpp:73,130`
  raise `Rexx_Error_Incorrect_method`, `:83` `Rexx_Error_Invalid_template`, `:224` the `NULL`
  hook. `orxmethod.cpp:2729`, `orxfunction.cpp:969` `NULL` hooks; `orxinvocation.cpp:473-478`
  define `packageLoader`/`packageUnloader`, `:532-533` install them. `orxfunction.cpp:750-768`
  is `TestAddCommandEnvironment`. `orxinstance.cpp:909` `RexxCreateInterpreter`;
  `orxclassicexits.cpp:808` `RexxStart`.
* `LibraryPackage.cpp:166-175` (`unload`, runs `unloader`), `:211` (`RexxGetPackage`), `:232-235`
  (version check), `:237` `loadRoutines`, `:239-246` loader. `PackageManager.cpp:642-650`,
  `:947`, `:968`; `Interpreter.cpp:281`; `PackageClass.cpp:1042-1049` (`parentPackage->
  findPublicClass`); `DirectiveParser.cpp:1682,1689`; `NativeActivation.cpp:219` (`processArguments`),
  `:680` (`usedArglist`), `:1787` (`checkConditions`); `ActivationApiContexts.hpp:64-68`;
  `Activity.hpp:458` (`contextToActivation`); `oorexxapi.h:259` (`RexxPackageLoader`),
  `:190-208`, `:135-174`; `ootest/misc/dlOpenTest.c:49` `#include <oorexxapi.h>`.
* Crate: `ffi.rs:168,172`; `layout.rs:313-314,514,592-595`; `native.rs:99-105,223`;
  `gate_tables/mod.rs:209,215,222,307`; `unsafe_sites.rs:89-91,107-109`; `dispatch.rs:1506,3231,
  3242,8266,8278`; `error.rs:1794,1952,1976`; `dispatch/library.rs:264-265,563`; `run.rs:3842`
  at `362f50453` and at HEAD; `sidecar.rs:183`; `oracle.rs:57`; `rexx-api/tests/load.rs:30,109`,
  `invoke.rs:49`, `context.rs:43`; `rexx-api/tests/layout.rs` exists.
* Records: `phase-7-gate.md:65-79` is section 4 and names `method-bodies.txt`'s `answers`
  blindness; `unanswered` occurs 0 times in that file. `E-external-resolution.md:305-312`
  names `parentPackage`, `findRoutine` / `findClass`.
  `records/2026-09-07-phase-5h-mapped-collections/task-4-report.md:47` says `.environment` and
  `.local` stay `Body::Native`.
* Commits: every hash named in the six commits and the surface plan resolves, with the subject
  the document ascribes to it. `git log --oneline 04a286913~1..cf92ff4fb | wc -l` is 11.
  `fix-rereview-integration.md:11` reads "Critical 0, Important 1 (R1), Minor 9 (R2-R10)".

### R5. Important -- the D-L2 ruling was written into two places and left the old owner in five

Commit `2e0590b40` records Moritz's ruling (Phase 8 takes the `Directory` methods) in the D-L2
body (`2026-07-27-rust-rewrite.md:566-573`) and at the tail of roadmap row 8. At HEAD the old
statement stands in:

    $ /bin/grep -n "no open phase\|HAS NO OWNER\|no longer this phase's\|no open phase holds" \
        docs/superpowers/plans/2026-07-27-rust-rewrite.md docs/superpowers/plans/phase-8-gate.md \
        docs/superpowers/plans/phase-4-exclusions.txt docs/superpowers/plans/phase-8-l2.md
    2026-07-27-rust-rewrite.md:543  | 8 | ... **L2 is still not reached, and the blocker is no longer this phase's.** ... D-L2 below recorded that no open phase owned them, and Moritz then ruled that this phase does ...
    2026-07-27-rust-rewrite.md:550  **D-L2, 2026-09-14: L2's blocker is a closed phase's leftovers, and no open phase owns them.**
    phase-8-gate.md:108             * **L2 is still not reached, and the blocker is no longer this phase's.**
    phase-8-gate.md:116               D-L2 records that no open phase owns them.
    phase-4-exclusions.txt:4537     L2 IS NOT REACHED AND ITS BLOCKER HAS NO OWNER. ...
    phase-4-exclusions.txt:4546-4548  Decision D-L2 in the roadmap records that the Rung column cannot be re-homed until a phase takes those methods.
    phase-4-exclusions.txt:4741-4742  OWNER: the same unpaid Phase 5 debt D-L2 records for the Directory methods; no open phase holds it.
    phase-8-l2.md:300-304           **No phase currently in the plan owns the work that would unblock it.**

Run. Row 8 contradicts itself inside one cell (the bold lead says the blocker is not this
phase's; the sentence after it says Moritz ruled it is). `phase-8-gate.md` §5 and the exclusions
file are where the project looks for a gap's owner, and both still say none. The `Method~new`
entry (`:4741`) borrows the Directory debt as its precedent, so its "no open phase holds it" is
now false by analogy as well. `phase-8-l2.md` §4 and the D-L2 heading are records of the finding
as made and may stand if labelled; the other four are live statements.

### R6. Important -- surface plan Task 3 Step 3 names the pre-F9 variant for a result-side 93.968

`2026-09-14-phase-8-surface.md:216-217`: "The special codes as *return* types answer `Signature`
(93.968), which is `valueToObject`'s `default:`, rather than `Unfilled`." This is the final
review's A8 (`final-review-a-boundary.md:369-370`), written before F9. F9 split the two:

    $ /bin/grep -n "ResultSignature\|Failure::Signature" rust/crates/rexx-api/src/values.rs | head
    116:    ResultSignature,
    974:  ... Err(Failure::Signature),          # to_native, parameter side
    1013:   return Err(Failure::ResultSignature); # from_native, result side
    1073/1090/1101: return Err(Failure::ResultSignature);
    $ sed -n 570,575p rust/crates/rexx-exec/src/dispatch/library.rs
        assert!(lineless(Refused::Signature));
        assert!(!lineless(Refused::ResultSignature));

Run. A special code as a *return* type is refused inside `from_native`, the same `valueToObject`
`default:` F9 measured for the `OPTIONAL|int` return word, whose delivery keeps its line against
the sender (spec §4, l2 §1d, `refusal-sites.tsv:143`). `Failure::Signature` is the lineless,
declaring-package delivery. A subagent following the step as written ships the wrong delivery for
a case the phase measured. The step should name `ResultSignature`.

### R7. Important -- surface plan Task 7 Step 2 asserts over a set nothing populates

`2026-09-14-phase-8-surface.md:300-304`: a derived test "asserts the partition: the groups whose
libraries need no interpreter library are the ones `corpus/phase-8.txt` runs".

    $ /bin/grep -rln testOORexx rust/crates rust/corpus
    rust/crates/rexx-exec/src/lib.rs            # a comment; no harness runs a group
    $ /bin/grep -v '^#' rust/corpus/phase-8.txt | /bin/grep -v '^$' | /bin/grep -vc '^lang/library_'
    0                                            # 29 rows, every one a lang/library_* program

Run. No corpus program or harness runs an ooTest group, `phase-8.txt` lists no group, and Task 8
runs the three groups "with the single-group form, on both sides", not through `phase-8.txt`.
Task 7 precedes Task 8. As written the assertion's right-hand set is empty when the test is
written, so it either fails on `METHOD`/`CONVERSION`/`FUNCTION` or is rewritten by the subagent
into something the plan did not specify. The step needs to say what "runs" reads (a list the
test owns, or rows Task 8 adds) and the order has to make that list exist first.

### R8. Minor -- "all four load sites" names a subset of the load sites the tree has

`2026-09-14-phase-8-surface.md:193-195` (Task 2 Step 3): "so all four load sites make the routines
callable". The fix re-review enumerated nine load sites for a first ask
(`fix-rereview-integration.md:184-187`: `loadLibrary`, `loadExternalMethod`,
`loadExternalRoutine`, `::requires ... LIBRARY` in the program, `::method ... external`,
`::routine ... external`, `::attribute ... get external`, and the two required-package forms),
and B7 (`final-review-b-integration.md:126-129`) names `loadExternalMethod` and the `::METHOD`/
`::ATTRIBUTE` sites as loading and registering nothing. Run. The step's own first clause ("the one
resolution path every load site shares") is the right one; "four" is the amendment bullet's list
plus `::REQUIRES` and undercounts.

### R9. Minor -- gate §2 puts the retired number beside `::REQUIRES`

`phase-8-gate.md:60`: "`b83ed8312` (the load-failure numbers: 98.982 for `::REQUIRES` and the two
entry-missing paths)".

    $ git show -s --format=%B b83ed8312 | sed -n 3,5p
    ... A `::REQUIRES ... LIBRARY` naming a library
    that is not there raises 98.903 and not 98.982, ...

Run. The commit corrected 98.982 to 98.903; the sentence reads as though 98.982 is the number.
Spec §3 `:91` has it right.

### R10. Minor -- "Amended 2026-09-14" cites commits from 2026-09-15

`2026-09-14-phase-8-surface.md:24` "Amended 2026-09-14, before execution"; `:41` and `:305` cite
`313807bc5` and `3bb1d34d2`.

    $ git log -1 --format='%h %ad' --date=iso 313807bc5    -> 2026-09-15 00:26:05 +0200
    $ git log -1 --format='%h %ad' --date=iso 2e0590b40    -> 2026-09-15 00:31:52 +0200

Run. The ruling was 2026-09-14; the amendment that cites the pass's commits was not.

### R11. Minor -- `M-F5b there` lands on the wrong document

`phase-4-exclusions.txt:4770-4774`: "(the ledger's final-fix-report.md F5, and nine load sites SAME
on the fix re-review's own forge). Package~loadLibrary's mapping of Version to 98.982 has no
committed test that fails without it (M-F5b there)."

    $ /bin/grep -rn "M-F5b" docs/superpowers/records/2026-09-14-phase-8/*.md
    final-fix-report.md:413, :422, :841      # nowhere in either fix-rereview-*.md

Run. `there` follows "the fix re-review", which does not carry M-F5b; the fix report does. The
"nine load sites" count is sourced (`fix-rereview-integration.md:187`, "all nine SAME x3").

### R12. Minor -- "Tree Borrows was run once, in the fix round" misattributes the measurement

`phase-4-exclusions.txt` INSTRUMENTS OUTSIDE THE GATE: "Tree Borrows was run once, in the fix
round, and passes the unfixed tree too, so Stacked Borrows is the one witness."

    $ /bin/grep -n "Tree Borrows" docs/superpowers/records/2026-09-14-phase-8/fix-rereview-boundary.md | head -3
    129:| M2 | pre-F8 | TB | **12 passed, exit 0** ... Tree Borrows accepts the unfixed shape |
    281:   Tree Borrows also passes the unfixed `13268f0e1` (M2/TB, 12 passed) and the thread-side mutant

Run. That Tree Borrows passes the unfixed tree was measured by the fix re-review, which ran it on
the pre-F8 tree and on a mutant; the fix round ran it on the fixed tree only. Neither "once" nor
"in the fix round" holds for the fact the sentence rests on.

### R13. Minor -- two step lines in the executed plan still call this worktree's build "the oracle's own"

`2026-09-14-phase-8.md:178` "Tests against the oracle's own binary. `build/lib/librxregexp.so`, by
absolute path from the workspace root" and `:319` "`RegExp_Init` from the oracle's own
`librxregexp.so`". The pass corrected I7 in the same file's Global Constraints (`:42-46`: the
in-crate tests open this worktree's `build/lib`) and left these two, which describe the same
tests. Run (`sed -n 178p; sed -n 319p`). The brief's "record of what was asked" reading covers
the step's content, not the false label on the file.

### R14. Minor -- "the Task 1 store `Directory~new` already uses" cannot be resolved

`2026-09-14-phase-8-surface.md:153-154`, inside Task 1. Within this plan Task 1 is the task the
sentence is in; if it means Phase 5h's Task 1, that is "the object-keyed store, `IdentityTable`
and `Table`" (`2026-09-06-phase-5h-mapped-collections.md:65`), and `Directory` is string-keyed.
Run (grep); a subagent has no way to pick. Name the store by its type.

### R15. Minor -- `liborxinvocation.so` does not NEED `librexx.so` in its own dynamic section

`2026-09-14-phase-8-surface.md:108-111`: "Never load a prebuilt extension that NEEDs `librexx.so`
or `librexxapi.so` ... `liborxinvocation.so`, `liborxexits.so`, `liborxclassic.so` and
`liborxclassic1.so` do ... Check `readelf -d` before the first load".

    $ readelf -d /home/moritz/dev/repos/ooRexx/build/lib/liborxinvocation.so | grep NEEDED
     (NEEDED) Shared library: [liborxexits.so]
     (NEEDED) Shared library: [libc.so.6]

Run. It reaches `librexx.so.4` through `liborxexits.so`; the prescribed `readelf -d` check on the
library itself passes it. The spec and exclusions say "through it"; the constraint should too, or
say the check is transitive.

### R16. Minor -- "(Task 7 records the commands)" is prospective and Task 7 does not

`2026-09-14-phase-8-surface.md:34`. Task 7's Step 1 records `diff -rq`; Step 2 derives a test.
The `readelf -d`/`nm -D` commands are recorded in the scoping survey §3, roadmap rows 9-10 and the
exclusions (`313807bc5`, `3bb1d34d2`, `6db4f085f`). Inferred from the plan text.

## Checks that held (scope 1, continued)

### Section 8 of `phase-8-gate.md` (`5f09e0e7b`, `6a167e462`)

Every figure traced to `scratchpad/gate8docs/`:

    $ cat status.txt                 -> G1 exit=0 G2 exit=0 G3 exit=101 G4 exit=101 G5 exit=0 ALLDONE
    $ cat rev.txt rev-after.txt      -> 476347f52 476347f52 ; dirty.txt and dirty-after.txt empty
    $ grep -a "^test result" g3.txt | awk '...sum...'   -> passed 2524 failed 5 ignored 4
    $ grep -a "^test result" g4.txt | awk '...sum...'   -> passed 2524 failed 6 ignored 4
    $ grep -a "of 545" g5.txt        -> 545 of 545 matching
    $ grep -a -E "of 896|of 4999|of 186|of 4259" g3.txt g4.txt
       892 of 896 bodies ... 4992 of 4999 value rows passing, 186 of 186 raise rows ... 4247 of 4259 rows

Failing sets, from the `failures:` blocks: G3 = `ir::drive::tests::{a_call_site_resolves_once_...,
a_long_constant_is_built_once_..., the_ir_engine_steps_an_ifs_chosen_branch_...}`,
`a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`,
`the_l0_subset_passes_again_under_collect_on_every_allocation`; G4 = those plus
`concept_and_class_gate_table`. The commands in the table are `run.sh`'s, each status read from
`$?` unpiped. Section 6 says 531 of 531 (`phase-8-gate.md:136`). `progress.md:1146-1156` and
`:1225-1231` record the same failing sets at `cf92ff4fb` and `233d2766d`. All run.

### Severity tallies quoted in gate §7

Counted from each report's "Findings by severity" section: `final-review-a-boundary.md:333-380`
Important 1-4, Minor 5-9 (0/4/5); `final-review-b-integration.md` headings B1, B2, B5, B7, B6
Important and B3, B12, B4, B10, B8, B9, B11 Minor (0/5/7); `final-review-c-claims.md:129-157`
C1; I1-I7; M1-M10 (1/7/10); `fix-rereview-boundary.md:261-294` 1; 2-4 (0/1/3);
`fix-rereview-integration.md:11` "Critical 0, Important 1 (R1), Minor 9 (R2-R10)";
`residual-rereview.md:412-455` 1; 2-4 (0/1/3). All six match. "closed B1-B6, B8 and B10 on their
original probes" matches `fix-rereview-integration.md:42-46`. R6's two counts (`:377-385`), R9
(`:410`), the "13/13 under both models" one-witness finding (`fix-rereview-boundary.md:280-283`),
`e8a6b7667`'s stat (ffi.rs, invoke.rs, library.rs, ir_recorded.rs, sidecar.rs, corpus.rs,
environment.rs, lib.rs, one `.env`) all as the section states.

### KNOWN GAPS probes, re-run both sides (fresh directory per side, standard wrapper, three files)

Crate binary: a `cargo build --release -p rexx-exec` of `git archive 476347f52 rust` in the
scratchpad (`docs-review/target/release/rexx-run`; HEAD's `rust/` is unchanged since). Programs in
`docs-review/probes/src/`, outputs in `docs-review/probes/out/`. Every transcript matches the
exclusions entry it belongs to:

* `b12`: oracle `code 88.901 / position The NIL object / program <DIR>/re.cls / traceback items 2`
  (`Compiled method "DOPARSE" with scope "RE".`, `3 *-* say r~doparse()`); crate `position 3 /
  program <DIR>/main.rex / traceback items 1`. rc 0 both.
* `r9`: oracle `request STRING` / `pos 2` rc 0; crate rc 168, `Error 88.909:  Argument 1 must have
  a string value.`
* `f4`: oracle `The K class` / `answered a Package`; crate `.K` / `answered a Package`; rc 0 both.
* `mnew`: oracle `a Method` rc 0; crate rc 120 `method "NEW" of class "Method" is not implemented
  (Phase 5)`.
* `tb`: rc 213 both; oracle stderr has `*-* Compiled method "NEW" with scope "Package".` and
  `1 *-* p = .Package~new(...)` between the first line and `Error 43 running src line 1`; the
  crate's has the first line and the last two only.
* `cwd`: oracle `cwd copy 1`, crate `cwd copy 0`, rc 0 both.
* `c1`: `Error 90 running <DIR>/pk.cls line 3` against `<DIR>/main.rex line 3`, 90.998, rc 166
  both. `c2`: `<DIR>/k2.cls line 1` against `<DIR>/main.rex line 1`, 98.909, rc 158 both.
* `b7r1`: oracle `load 1` / `sqrt 4` rc 0; crate `load 1` then 43.1 rc 213. `b7r6`: oracle
  `main 4`; crate 43.1 rc 213.
* `ll0`: 88.901 rc 168 both; `ll1`: 98.984 rc 158 both. `allidx`: oracle ten lines `SYSCARGS
  INPUT TRACEOUTPUT DEBUGINPUT STDOUT OUTPUT STDERR STDIN STDQUE ERROR`; crate rc 120
  `method "ALLINDEXES" of class "Directory" is not implemented (Phase 5)`.
* Not run, by rule: `oracle-crashes.txt` entry 15.

### Scope 2 checks that held

* `dispatch/hash.rs:144-153` `owns` returns false for `Body::Native`; `not_this_task` at `:171`;
  `closed_phases.rs:36` `CLOSED = &["Phase 7"]`; `environment.rs:240-244` `EnvironmentModel` with
  `unbuilt`; `D45` named in `environment.rs`; `minted_local_name` `:223`; `dispatch/package.rs`
  exists; `libraries.rs:36-55` `Libraries` with `get` and `hold` in its own module;
  `lib.rs:4934,4945` `blame_directive` / `blame_directive_in` side by side; `invoke.rs:56`
  `pub fn method`; `ffi.rs:153` `Contexts` owning `RexxThreadContext_`; `rexx-api/tests/layout.rs`
  exists; `unsafe_sites.rs:25-30` walks every `.rs` under `crates/`, so `tests/` is scanned.
* `records/2026-09-07-phase-5h-mapped-collections/task-4-report.md:47` keeps `.environment` and
  `.local` `Body::Native`; `progress.md:880` records the nine methods answering on a plain
  `.Directory~new`; my `dir9` probe (hasEntry, hasIndex, entry, index, items, allIndexes, remove,
  setEntry with and without a value, supplier) is byte-identical on both sides, rc 0.
* `.environment~allIndexes` on the oracle: 69 lines, `LOCAL` last, two runs `cmp` identical.
* `final-review-a-boundary.md:144-145,280-281`: `b3_stash_shallow_use_deep` oracle `x 42` rc 0,
  crate rc 134, ASan `stack-use-after-return` in `rexx_api::ffi::whole_number_to_object`;
  `:37-47` sigcheck `mismatches: 0` over six tables. `final-review-b-integration.md:117-160` B7's
  `r1`/`r6`/`r7` with `rxmath` as Task 2 quotes; my `lext` probe: oracle `obj a Routine` /
  `call 4`, crate `obj a Routine` then rc 120 `a routine whose body this crate does not hold is
  not implemented (Phase 5)`, so "shells whose use refuses naming Phase 5" holds.
* `testbinaries/orxfunction.cpp:530,540,564`: `dHandler`/`rHandler` take `RexxExitContext *`,
  `ioHandler` also `RexxIORedirectorContext *`; `oorexxapi.h:409,412` `REGISTERED_EXITS`,
  `DIRECT_EXITS`; `rexx.h:654` `RexxRegisterExitExe`. `LibraryPackage.cpp:232-246`: the version
  `reportException` precedes `loadRoutines` and the loader, so "never for a library refused for
  its version" holds.
* Items the records route to the surface plan, and where the plan carries them: A1 (thread
  context lifetime) Task 4; B7 (routines at every load site, loaded objects that run) Task 2;
  A7 (signature-level layout test) Task 5 Step 4; A8 (special return codes, `usedArglist`) Task 3
  Steps 2-3, with R6 above; I2 (`testbinaries/` build) Task 7; C1 (Task 6 could not run without
  D-L2) Task 1; M8 (211 pointers grouped) Task 5 Step 1; loader/unloader Task 4 Step 3. The
  `::ATTRIBUTE` getter-first order is routed by the exclusions to "a forged-extension instrument,
  if one is ever built" and by no record to the surface plan.

### Observation, not a finding

The review brief's range `313807bc5..6be99d610` excludes `313807bc5`; the docs-pass report counts
it among "six commits". Both were reviewed here.

## Not reached

* The twenty L1 cases' driver (`phase-8-l2.md` §1a, §1f) and the L2 walk itself were not re-run;
  their statements stand as the run they record.
* Miri was not run; the `rexx-api` test suites and the exclusions file's four test readers were
  not re-run (the pass's report records exit 0; the gate run at `476347f52` covers them).
* The deletion of the `mem::replace` sentence from the surface plan's inherited facts was not
  checked against the tree or the ledger.
* Residual re-review finding 4's other context shapes (bound library object, loaded package,
  `Routine~newFile`) were not re-probed; only the `.context~package` shape was.
* `phase-4-exclusions.txt` outside the passages the six commits touched, and the Phase 8 section's
  entries the pass did not edit, were not re-read.
* The `sourceline_oracle` companions and the corpus harness under `{oraclelib}` were not run.
* Whether `e8a6b7667` carries exactly "three doc corrections" was not counted.

## Summary

Critical 0. Important 5: R1 (the `:2638` roadmap citation moved again, by `2e0590b40`), R2
(`build/lib` does not hold every `testbinaries/` product; two are executables in `build/bin`,
stated in three documents), R5 (the D-L2 ruling left the old owner in row 8's own lead sentence,
gate §5, two exclusions entries and l2 §4), R6 (Task 3 Step 3 names `Signature` where the
result-side variant is `ResultSignature`), R7 (Task 7 Step 2 asserts over a set no task
populates, before the task that would). Minor 11: R3, R4, R8-R16.
