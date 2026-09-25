# SDD ledger — plan: docs/superpowers/plans/2026-09-14-phase-8-surface.md

Spec: docs/superpowers/specs/2026-09-14-phase-8-native-api.md (evidence base
docs/superpowers/specs/2026-09-14-phase-8-scoping.md). The L2 slice's plan, its final review and fix
rounds are recorded in docs/superpowers/records/2026-09-14-phase-8/. BASE at start: `d7eafeb54`.

Plan state at start: amended at `2e0590b40`, corrected at `509225578` after a review of the
amendment (`docs/superpowers/records/2026-09-14-phase-8/docs-review.md`, scope 2).

## Pre-flight scan

### Pairs of tasks sharing a file or an interface

| tasks | shared | produces / consumes | finding |
|---|---|---|---|
| 1, 9 | `tests/closed_phases.rs` | 1 may add `"Phase 5"`; 9 adds `"Phase 8"` | compatible; both append to one list |
| 2, 3 | `rexx-api/src/invoke.rs` | 2 adds the routine protocol; 3 skips the too-many check for `ARGLIST` in `invoke::method` | sequential; 3 must extend whichever entry point 2 builds for routines too, see ruling S2 |
| 2, 5 | `rexx-api/src/layout.rs` | 2 populates `CallContextInterface`; 5 the thread and method tables | disjoint tables; sequential |
| 4, 5 | `ffi.rs` | 4 gives `Contexts` the interpreter's lifetime and resolves the current frame from the owner; 5 fills thread-table callbacks that rely on it | order right (4 before 5) |
| 5, 6 | `ffi.rs`, `layout.rs`, `RexxInstanceInterface` | 5 Step 1 fills "the pointers `orxmethod.cpp` and `orxfunction.cpp` call"; `orxfunction.cpp` calls `AddCommandEnvironment`, which is 6 Step 1 | **conflict**, ruling S1 |
| 7, 8 | the group list | 7's test holds the list; 8 runs the groups it names | order right |
| 2, 7, 8 | `corpus/phase-8.txt` | 2 adds witness rows; 8 adds witnesses | append-only; no conflict |
| 1, 8 | the framework loading | 1 Step 5 walks `ooTest.frm` to its next stop and does not fix it; 8 needs the groups to run | 8 inherits whatever 1 Step 5 finds; ruling S4 |

### Each task against itself

| task | finding |
|---|---|
| 1 | Files omit `tests/closed_phases.rs` (Step 6), `rust/corpus/phase-8.txt` (Step 3's rows) and `docs/superpowers/plans/phase-8-l2.md` (Step 5's successor section). Step 6's condition will not be met: `/bin/grep -rln --include=*.rs '"Phase 5"' rust/crates/*/src` names owners in files beyond the directory seam at `d7eafeb54` (listed by that command), most unrelated to `Directory`; so Step 6 is a report of what remains. |
| 2 | Files omit `corpus/phase-8.txt`, the `.env`/`.d` fixtures and `rexx-parse/tests/sourceline_oracle/`; the conventions are in the L2 records. |
| 3 | Consistent. |
| 4 | Step 3's unloader at termination lives on the interpreter's termination path, which is `rexx-exec/src/lib.rs` rather than the files listed. |
| 5 | Step 1 overlaps 6 (S1). Step 4's `sigcheck.py` was written against the pre-X1 macro and must be re-derived, not transcribed. |
| 6 | Consistent once S1 applies. |
| 7 | Step 2 places the derived test beside `rexx-api/tests/load.rs`, which has no oracle-root helper; the test needs the oracle checkout's `build/lib`. Ruling S3. |
| 8 | Step 1 runs groups "with the single-group form, on both sides", and Step 2 enumerates the failing set, but nothing makes that a committed, re-runnable instrument, and Task 9's gate cannot read a manual run. `testOORexx.rex` writes fixtures into its working tree. Ruling S4. |
| 9 | Consistent. |

### Rulings

* **S1:** Task 5 Step 1 excludes `RexxInstanceInterface::AddCommandEnvironment` and the exit and
  IO-redirector tables, even though `orxfunction.cpp` calls them; they are Task 6's. -- Task 6's
  whole content is that path, and a partial body in Task 5 would be the second code site Task 6
  then has to find. -- If wrong: Task 5's derived "no stub remains" test must list those as
  deferred to Task 6, which is cheap.
* **S2:** Task 3 Step 2's `usedArglist` applies to every entry point that runs the two-call
  protocol after Task 2, the routine one included. -- The oracle's `processArguments` is shared by
  methods and routines. -- If wrong: a routine taking `ARGLIST` refuses too-many where the oracle
  does not.
* **S3:** Task 7's derived test lives in `rexx-exec/tests/`, reusing `tests/support/oracle.rs`'s
  oracle root, not beside `rexx-api/tests/load.rs`. -- One definition of where the oracle checkout
  is. -- If wrong: a file move.
* **S4:** Task 8 commits an instrument, not a transcript: a test in `rexx-exec/tests/` that runs
  each group Task 7's list names through `testOORexx.rex`'s single-group form on both sides, from a
  scratch copy of the group's directory and the framework (never `ootest/` itself, since the suite
  writes fixtures), and asserts the crate's group result matches the oracle's. It may be
  gate-only (behind `REXX_CORPUS_GATE`) if it is slow. Whatever Task 1 Step 5 finds between the
  framework and a running group is Task 8's first step to measure and route. -- Task 9's gate has
  to read the group result from a run, and the L2 walk showed the suite dirtying `ootest/`. -- If
  wrong: Task 8 grows by one harness it would have needed at Task 9 anyway.

## Task 1: dispatched

BASE `d7eafeb54`. Brief `task-1-brief.md` (the plan's Task 1 plus the inherited-context and Global
Constraints sections, the scan's rulings for this task, and the operational rules), model opus,
report `task-1-report.md`. Steps 1-2 (find the blocker, choose the representation) are to be
written into the report before any code; the implementer stops and asks if the blocker invalidates
every representation.

### Task 1: implementer returned DONE_WITH_CONCERNS, `6bf1401fd`

The blocker Moritz remembered is in `records/2026-08-*-phase-5a*/task-23-report.md:364`:
membership, then order. Neither holds now: `.environment` keeps its contents at 69 buckets (the
only count between 17 and 4999 consistent with the observed order; on the oracle a
`.Directory~new(68)` filled in that order matches it through a resize), `LOCAL` is last because it
is a method-table entry (`Setup.cpp:1781`), and bootstrap membership is exact. Candidate (c), the
never-shrinking unbuilt map, was a live bug (`defineMethods(.environment)` refused naming Phase 5);
(d) the security manager sees no check on sends to either directory; (e) class references never
reach the directories. Both are now store-backed like `.Directory~new`, unbuilt names hold a
placeholder a later `put` replaces in place, `.local`'s streams, monitors and `SYSCARGS` are minted
at the end of bootstrap as `LocalServer~initInstance` does, `STDQUE` value reads refuse naming
Phase 10. Beyond the brief: string-keyed collections take their index as a string argument
(88.909, `HashCollection.cpp:1112`); `allIndexes`/`makeArray`/`DO OVER` no longer run `setMethod`
methods; two unrooted values in the new `.local` setup fixed. Corpus 548 of 548. Controls: A
(refuse in `owns`) 0 of 548 since bootstrap writes `.environment` itself; A2 (refuse after
bootstrap) 539 of 548 with one unpredicted red; B (reversed order) exactly the two new witnesses.
Step 5: `ooTest.frm` loads on both sides; the driver next stops at `worker.rex:1074`,
`cmdLine~copy`, `'abc'~copy` refusing naming Phase 5 -- to Task 8 under S4; the driver also needs
`rxregexp.cls` beside the framework copy. Step 6: `"Phase 5"` not added.

Concerns: `.NAME` reaching the directories ~1.61x base in instructions (callgrind, `rexxcps`
+0.03%, startup +0.5%); other `Body::Native` directories (condition objects, `Package~local`,
security manager argument directories) still refuse the same methods, and the framework reads
condition objects (`OOREXXUNIT.CLS:1595`); the two failing tests are the known `collect_stress` L0
and `concept_and_class_gate_table`; a probe wrote 47 files into `rust/crates/rexx-exec/` and they
were removed by list before the commit (tree clean, checked by the controller).

Task review dispatched (opus), package `review-d7eafeb54..6bf1401fd.diff`, report `task-1-review.md`,
with seven named risks: rooting, order through `put`/`remove`/resize, `STDQUE` on every value
route, the two out-of-brief changes, the D45 chokepoint, the controls' reach, the `.NAME` path.

### Task 1 review: Needs fixes (1 Critical, 1 Important, 6 Minor)

`task-1-review.md`. Held under probes past the witnesses: order through 150 puts with removals and
growth on both directories (197 names in the oracle's order), 88.909 message text on ten routes and
survival under stress, an auditing security manager seeing identical events on both sides, every
other new handle held across an allocation rooted. Findings:

* **Critical:** `VALUE(name, new, '')` panics `a live value` under collect-on-every-allocation at
  HEAD and not at BASE: `old` held across `set_directory_entry`, which now allocates.
* **Important:** `.context~package~findClass('STDQUE')` answers `.nil` (`environment.rs:1854`,
  `Owed` falls through), oracle `SESSION`; older than the change but on the line it rewrote, and it
  falsifies "every value read refuses".
* Minor: `VALUE(name, , '')` through `dot_variable` gains a case; `STDQUE` `setEntry` with no value
  and `setMethod` over-refuse; a `.NAME` missing both directories costs ~1.88x base in instructions
  (not in the report); no witness for "no method run"; the method-table test passes a `.nil`
  degenerate; two doc comments over-claim. Inherited `equivalent` refuses on both directories.

Fix round 1 sent to the implementer: the crash (plus a sweep of every value held across the now
allocating write), `findClass` refusal, `STDQUE` removal on `setEntry`/`setMethod`, the miss-path
cost, the missing witness, the degenerate-proof test, the doc. Recorded not fixed: `VALUE(name, ,
'')` and `equivalent`.

### Task 1 fix round 1: DONE_WITH_CONCERNS, `81c927160`..`24d459c82`

`81c927160` VALUE's old value rooted, a five-case `collect_stress` row red without it, every caller
of the directory write path checked and tabled; `24f8879e3` `findClass` of an owed name refuses
(Phase 10) and now propagates a raising `setMethod` entry (42.3, stderr `running RUN` pre-existing),
`setEntry` with no value / `setMethod` / `removeItem` take `STDQUE` without reading it, doc
corrected; `7bfe5d428` witness that index reads run no method; `3cd228b8b` the method-table test
compares transcripts against a `Directory` copy and reddens under the `.nil` degenerate; `96bf51e3e`
a per-directory reading of the pool stamped with a generation counter moved by `install_store`,
`set_free`, `set_unknown_method`, checked by a `debug_assert` -- miss path 1.086x base (was 1.88x),
`dotname.rex` 1.147x (was 1.617x), startup 1.004x, `rexxcps` 1.0005x, the remainder the bucket
probe itself; `24d459c82` three comments narrowed. Corpus 551 of 551 release and debug; the two
failing test binaries are the known pair. Recorded not fixed: `VALUE(name, , '')` through `.NAME`,
`equivalent` refusing (the operator refusal on `.RexxInfo` and `SYSCARGS`'s array).

Re-review dispatched (opus), package `review-6bf1401fd..24d459c82.diff`, report
`task-1-rereview.md`, first named risk the cache's invalidation: every pool write that can reach
the two directories, probed in release where the `debug_assert` is off.

### Task 1 re-review: Needs another round (small)

`task-1-rereview.md`. Critical 1, Important 1, Minors 2-5 closed by running; the `.nil` degenerate
control re-run and red as predicted; the pool-reading cache clean -- every pool write goes through
four `Interp` methods, the one store write that does not move the counter writes a copy, `EXPOSE`
cannot alias the store (`define`/`inherit` on `Table` refused 98.985 both sides), probes identical
in release and debug, the `set_unknown_method` control reddens as predicted. Minor 6 partly: a
rewritten sentence still false. New in the fix, all Minor and prose: `store_generation`'s doc, the
collector field audit's `environment: _` (now holds `views`), `EMPTY`'s doc (oracle leaves method
entries), `Removal`'s doc against `removeItem`, `views`' "last reading", the report's three false
sentences and stale 1.62x. Out of scope but the same class, reached through Task 1's rewrite:
`directive_class` swallows a raising `UNKNOWN` on `.environment` (oracle rc 214 42.3, crate rc 158
98.909, pre-existing at the fix base).

Fix round 2 sent: `directive_class` propagation plus a sweep of the same shape, the six comments
(checking the `views` rooting claim by running the stress harness first), the test's `SETENTRY`
row, the report corrections under a new heading. Recorded not fixed: the two-`VALUE` route with a
same-named `.local` method entry (Minor 1's).

### Task 1 fix round 2: DONE_WITH_CONCERNS, `d9262d474`..`35dc46110`; reviewed by the controller

`d9262d474` `directive_class` answers `Result` and propagates the `.environment` lookup's raise
(reviewer's probe now rc 214 42.3 both sides), witness `directive_class_environment_raises.rex`
with a three-package `.d/`, control (`if let Ok` restored) reddens exactly its two raising rows;
the other error-swallowing reads tabled in the report, only this one needed a fix. `3c38f62a4` the
six comments, the `views` rooting claim checked first under a debug collect-on-every-allocation
build over three directory witnesses (all rc 0, byte-equal to the oracle). `35dc46110` the
`SETENTRY` row. Report corrections under "Fix round 2". Corpus 552 of 552; failing binaries the
known pair. Recorded not fixed: the two-`VALUE` route; a trapped error's `POSITION` naming the
caller's line; a failed `::REQUIRES` under an external call missing its `call` traceback line; a
non-class `.environment` entry as a `::CLASS` target (oracle 99.949, crate 98.909) -- the last three
the same at BASE.

**Controller's review of round 2**, by reading its code diff (80 insertions): the propagation is
the `findClass` shape, blames the directive on both the miss and the raise, and the comments now
state what the code does. Accepted without a further dispatched re-review: small, controlled, and
every claim in it either ran or is a comment. **The new oracle crash** the round found
(`.local~setMethod('TRACEOUTPUT', ... exit 7)` then a traced clause, SIGSEGV rc 139, run once) added
to `rust/corpus/oracle-crashes.txt` as entry 11d, with its raising variant (rc 207, both
descriptors empty); the licence covers the crash, and where the crate writes the raising variant's
trace lines is recorded as its own answer.

**Task 1: complete.** Commits `6bf1401fd`, `81c927160`, `24f8879e3`, `7bfe5d428`, `3cd228b8b`,
`96bf51e3e`, `24d459c82`, `d9262d474`, `3c38f62a4`, `35dc46110`; crash list entry 11d at the
commit after (`refusal_sites` and `method_bodies`, the tests that mention the file, pass: 23 and 5).
`ooTest.frm` loads; the driver next stops at `worker.rex:1074`, `'abc'~copy` refusing (Phase 5),
Task 8's under S4. Carried to Task 8 as well: other `Body::Native` directories (condition objects,
`Package~local`, security-manager argument directories) still refuse the store family, and the
framework reads condition objects (`OOREXXUNIT.CLS:1595`).

## Task 2: dispatched

BASE `e7cb210d9`. Brief `task-2-brief.md`: the plan's Task 2, the inherited context, and rulings --
S2's shared too-many check, the witness files and refusal table the task also touches, **a second
Phase 8 refusal the plan did not name** (`::ROUTINE EXTERNAL naming REXX or REGISTERED`, pinned in
`run/tests.rs`: measure, implement or re-home), the shared-code model and its witnesses kept green,
B7's probes as the reproduction, entry 15 never run, Miri for `ffi.rs`/`load.rs` changes. Model
opus, report `task-2-report.md`.

### Task 2 pre-flight: NEEDS_CONTEXT, ruled

No `rxmath` routine answers without rows and slots other tasks own: `RxCalcSqrt` is
`RexxRoutine2(RexxObjectPtr, RxCalcSqrt, double, x, OPTIONAL_positive_wholenumber_t, precision)`
(`extensions/rxmath/rxmath.cpp:409`) and calls `GetContextDigits` (call context) and
`DoubleToObjectWithPrecision` (thread table, refusing); `double` to_native and `RexxObjectPtr`
from_native are unfilled. Steps 1-5 as written would move B7's probes from a silent 43.1 to a loud
conversion refusal, not the oracle's answer.

* **Ruling Q1 (A):** Task 2 fills `double` and `positive_wholenumber_t` to_native, `RexxObjectPtr`
  from_native and `DoubleToObjectWithPrecision`, each with per-row tests, a row-deletion control on
  one of them, Miri for the slot, and witnesses at several precisions; stop and list if more is
  pulled in. -- A routine protocol witnessed only by a refusal proves the dispatch and not the
  answer. -- If wrong: Task 3 and Task 5 lose a few rows they would have built.
* **Ruling Q2 (b):** Task 2 populates `CallContextInterface`, wires a call context, fills
  `GetContextDigits`/`Fuzz`/`Form`; the other members refuse naming Phase 8. **Plan amendment owed
  between tasks:** Task 5 takes the `CallContextInterface` members `orxfunction.cpp` calls, beside
  the thread and method tables. -- Only rxmath's path is witnessable now. -- If wrong: Task 5 grows.

### Task 2: implementer returned DONE_WITH_CONCERNS, `4c7d65e97`, `c23c214f3`

`invoke::routine` and `invoke::method` share one private run (S2); `ROUTINE_CLASSIC_STYLE` refused
naming Phase 10 before the stub runs; `CallContextInterface` populated with
`GetContextDigits`/`Fuzz`/`Form`; `DoubleToObjectWithPrecision` filled; `double` and
`positive_wholenumber_t` to_native (88.921, 88.905 measured), `RexxObjectPtr` from_native. Every
library load registers its routines in `settle_library`; B7's r1/r5/r6/r7/r8/r10 and the
`loadExternalMethod` and failing `::METHOD EXTERNAL` sites match the oracle. `::REQUIRES ...
LIBRARY` merges routines into the requiring package as `PackageClass::mergeLibrary` does (found
before the security manager, run under the name as written, answered by `findRoutine`).
`Routine~call` on directive-bound and `loadExternalRoutine` routines; `~define`/`~defineMethods` of a
`loadExternalMethod` answer run. A boundary refusal is lineless only where a directive bound the
shared code. The call-site cache no longer keeps `Resolved::External`. `LIBRARY REXX` routines
implemented; `REGISTERED` re-homed to Phase 10 (the oracle writes into the rxapi daemon's function
registry, visible to a later process). Corpus 571 of 571; Miri Stacked Borrows 20 passed, 3
ignored; the known `collect_stress` L0 failure. Controls C1, C2, C4, C6 exact; C3, C5 partly
falsified in extent, recorded.

Concerns: **`RxCalcSin(30, 3, 'X')` aborts the process rc 134, losing earlier `SAY` output**, where
the oracle gives 88.916 -- a refusing thread-table slot (`NewStringFromAsciiz`, Task 5's) panics
across `extern "C"`, now reachable from a shipped extension; `MathLoadFuncs` refuses on the
`CSTRING` result row (Task 3's); name-collision precedence (library vs internal routine, merged
library vs merged public) unreproduced and unwitnessed; routine object identity not shared (hidden
by `==` on `Routine` refusing, Phase 5); **the `REGISTERED` probe left `ZZT2REGPROBE` registered in
the machine's rxapi daemon** (clears on daemon restart; cannot be dropped from another process);
`phase-4-exclusions.txt` edited outside the file list; `call .context~...` parse error rendering
and `~define` of a source-built `Method` refusing, seen in passing.

Review: diff 96 files, 2675 insertions, so two slices on the one package
`review-e7cb210d9..c23c214f3.diff`: boundary (fable, `task-2-review-boundary.md`) and integration
(opus, `task-2-review-integration.md`).

### Task 2 review, boundary slice: Needs fixes (0 Critical, 3 Important, 3 Minor)

`task-2-review-boundary.md`. The routine half built as asked: shared `run` holds the one too-many
check, style refused before the stub, call context derived from the whole `Owned`, Miri 20 passed;
formatting (31 rows: NaN, infinities, negative zero, denormals, precision above digits, digits 1
and 20, engineering form) and argument refusals (30 rows) identical on all three descriptors; a
fuzz-answers-digits mutant reddens exactly its test.

* **I1:** `NativeRoutineEntry::signature`/`call` are safe `pub(crate)` fns whose soundness rests on
  `invoke::routine`'s style check in another module; a classic row transmuted to the typed stub.
  Fix: `stub()` answers `None` unless typed.
* **I2, predates the task:** `RexxThreadContext_.instance` is null; `orxmethod`'s
  `TestInterpreterVersion`/`TestLanguageLevel` and `orxfunction`'s `TestAddCommandEnvironment` go
  through it. Oracle rc 0 `328448`; crate **rc 139, both descriptors empty**.
* **I3, the refusing-slot design:** slots reachable from a program listed per shipped library
  (`slots.txt`): `rxmath`'s trig routines with a bad units byte reach `NewStringFromAsciiz`,
  `ArrayOfThree`, `RaiseException`; `rxregexp` none; `orxfunction` and `orxmethod` many. Record-and-
  return is sound **in a per-call, per-thread cell**, not on the `Activation` (whose invariant the
  refusing tables' bare contexts do not keep); `ThrowException*`/`ThrowCondition` cannot return and
  stay aborting; pointer-returning accessors answered `NULL` need per-slot judgement; a refusal must
  win over a `pending` condition raised after the harmless value.
* Minor: `Failure::error_number(method)`'s routine 40.918 is dead code (host renders 93.968 for
  both); `routine` re-implements `signature`'s bound; one test's `answered == b"4"` is not the
  load-bearing assertion.

Waiting on the integration slice before the fix round.

### Task 2 review, integration slice: Needs fixes (0 Critical, 4 Important, 5 Minor)

`task-2-review-integration.md`. Registration SAME on every probe past the witnesses (loads inside a
required package's routine seen from main, a third package, a nested required package, `INTERPRET`,
a later `loadPackage`, a loop call site before the load, the `::ATTRIBUTE` site, two spellings, a
version-refused library registering nothing); the lineless rule SAME in eight shapes; `rexxcps`
+0.05%; witnesses exist that fail with registration only at `::REQUIRES`.

* **I1:** a library routine called twice from one expression call site runs under `""`
  (`ir/drive.rs:250` on a cache hit): tracebacks `Compiled routine ""`, the security manager's second
  `CALL` event `NAME` empty (that half wrong at base for internal routines too).
* **I2, pre-existing but claimed:** a merged library routine is found after required packages'
  public routines; the oracle merges libraries first. `RxCalcSqrt(16)` answers `pk 16` against `4`,
  rc 0; `importedRoutines['RXCALCSQRT']` `.nil` against a Routine.
* **I3:** dropping `Resolved::External` from the call-site cache costs +60% per external-file call
  (657,860 vs 1,052,950 instructions); base could not call an external file twice from one
  function-form site (43.1 `""`), which the task fixed unwitnessed.
* **I4:** `double_of` expands a number digit by digit: `RxCalcSqrt('1E+999999999')` peaks 1.97 GB
  and aborts rc 134 under a 1 GB cap; oracle `+infinity`.
* Minor: cached library-routine resolution survives `addPackage` (comment "Nothing invalidates one"
  false); exclusions CLOSED wording; `phase-8.txt` load-site list and "name as passed"; two
  citations; a `refusal-sites.tsv` witness column. Step 6: the `::ATTRIBUTE` load site unwitnessed.

## Task 2 fix round 1: sent

Both slices in one round to the implementer. **Rulings:** (R1) boundary I3 is fixed now, not in
Task 5 -- a refusing slot records the slot's name in a per-call cell on the calling thread, saved
and restored around nested native calls, returns the oracle's failure-path value, and `invoke::run`
refuses loudly after the stub; the refusal beats a `pending` condition; `ThrowException*` and
`ThrowCondition` stay aborting; pointer-returning accessors decided per slot and listed. -- shipped
extensions reach refusing slots from ordinary programs (`rxmath` trig routines), and an abort loses
output with no name. -- The thread-local cell is accepted against "no process-global state" because
it is set and cleared inside one native call on the calling thread; if wrong, it becomes a field on
whatever owns the call. (R2) boundary I2 fills `InterpreterVersion` and `LanguageLevel` now with a
populated instance context; `AddCommandEnvironment` stays Task 6's. Order: I4 (memory), boundary I2,
I3, boundary I1, integration I1, I3 (cache restored with invalidation on library registration and,
if the same mechanism, `addPackage`), I2 (one ordered merged lookup, libraries first), then minors.
Recorded not fixed: `setMethod` of a loaded method refusing (Phase 5), a condition object's
`TRACEBACK` without `Compiled routine`, a loop's parse error rendering `rexx-exec: 47.2`.

**Side effect outside the tree, recorded:** Task 2's `REGISTERED` probe left `ZZT2REGPROBE` in the
machine's rxapi daemon function registry; it clears on daemon restart. Every later dispatch forbids
registering with the daemon.

### Task 2 fix round 1: DONE_WITH_CONCERNS, `a873b0678`..`bd64f3197`

`a873b0678` `double_of` parses from mantissa and exponent (matches the oracle on huge exponents,
no longer exhausts memory); `fa7c51301` an instance context with `InterpreterVersion` and
`LanguageLevel`, record-and-return for refusing slots (loud rc 120 after the stub; `Throw*` and the
pointer-returning members still abort), `stub()` typed-only; `6245fe1ca` a kept call site passes the
routine's name, the security manager's check included; `25bdb2dd5` external files cached per site
again, dropped when a library registers routines or a merge adds a name (`cae7d5382` table
re-derive); `2804a8986` one merged table per package, libraries first, never replacing, read by
`importedRoutines` and namespace calls; `79f08bed6` routine signature refusal 40.918 (measured on a
forged scratch library, unit-test witnessed), the `::ATTRIBUTE` witness, minor wording;
`bd64f3197` an inert sidecar the gate caught. Corpus 584 of 584; the known `collect_stress` L0
failure; Miri passes; external-file call 1,064,166 -> 664,907 instructions, `rexxcps` +0.1%.

Concerns carried: `RxCalcSin(30, 3, 'X')` loud rc 120 against the oracle's 88.916 (Task 5's slots);
`orxmethod`'s version test reaches Task 3's `size_t` row; **cache invalidation does not mark a
`Resolved::Routine` found through a parent package shadowable, unmeasured**; two early gate runs
reused an earlier revision's binaries (caught by the Compiling count, re-run, script fixed);
routine objects still not shared.

Scoped re-review in two slices on `review-c23c214f3..bd64f3197.diff`: boundary (fable,
`task-2-rereview-boundary.md`: the refusal cell, failure-path values, the abort list derived from the
tables, the instance context, Miri with a save/restore mutant) and integration (opus,
`task-2-rereview-integration.md`: every event that can change a call site's resolution against the
generation, a stale answer being Critical; names on hits; the merged order; `double_of`; the
removed sidecar).

### Task 2 re-review, boundary: Needs another round (small)

`task-2-rereview-boundary.md`. I1, I2, M1-M3 closed; I3's design holds: the cell is per call and
per thread, a save/restore mutant reddens exactly the nested test, the refusal beats `pending` and
is untrappable, the abort set derived from the header equals the report's, no recording slot
returns a pointer (enforced by a test and, for `CSTRING`, by the compiler), Miri Stacked Borrows 24
passed. The instance context has the thread context's provenance; `InterpreterVersion` 328448 and
`LanguageLevel` 1542 as the oracle answers; the routine's 40.918 reproduces on three forged probes.

* **I-A:** `DisplayCondition`'s failure value is `Error_Interpretation/1000` = 49
  (`RexxErrorCodes.h:456`); the tree returns 48 and a test pins 48 -- the only non-zero failure
  literal across 220 stub functions, and the one that is wrong. Unobservable today.
* Minor: `InterpreterInstanceStubs.cpp:105` a blank line (`:106`); a nonexistent
  `InterpreterInstance::LanguageLevel`; a comment naming only a condition raised after the refused
  member; a non-1/2 routine style answering 40.918 where the oracle builds a typed routine, worth a
  sentence.
* **Owed to the plan, not this round:** the cell's doc holds only while no stub runs outside
  `invoke::run`; Task 4 Step 3's package `loader` hook, run with a thread context, is the first such
  path, and its text must say that a refusal recorded there is read, not cleared unread.

Waiting on the integration re-review before sending round 2.

### Task 2 re-review, integration: Needs another round (1 Critical, 2 Important, 4 Minor)

`task-2-rereview-integration.md`, complete although its agent was then stopped by a session limit.
I1, I4, M2-M5 and the `::ATTRIBUTE` site closed; external-file cost -37.7% per call.

* **C1, regression:** a kept `External` survives `~addRoutine` (`install_routine` does not move the
  generation): oracle and base `2 added`, head `2 ext 2`, rc 0.
* **I-A, regression:** a kept merged library routine is re-resolved on any generation move; the
  oracle keeps a `findRoutine` hit for good. `p3`: oracle and base `2 4`, head `2 pk 16`.
* **I-B, regression:** a context-built package's merged library routine is found before its
  parent's own routine; the oracle walks `routines` up the parent chain first
  (`PackageClass.cpp:822-911`). Oracle and base `main 16`, head `4`.
* Minor: the invalidation prose false in four places; `importedRoutines` leaks a `Routine` per send
  (400,000 sends rc 134 under the cap); `double_of`'s exponential-form sentence; two `run/tests.rs`
  tests say they load the oracle's build and load the worktree's.

## Task 2 fix round 2: sent

To the implementer, both re-reviews. **Ruling:** the call-site cache takes the oracle's rule --
keep `Routine` and `MergedLibraryRoutine` for good, drop every other kept kind on a generation move,
and move the generation in `install_routine` -- checked against the C++ before building, with a
sweep of every write that should move it. I-B: `findLocalRoutine` up the parent chain, then
`findPublicRoutine`, in both lookup functions. m2: one `Routine` per library code row, which also
makes `findRoutine` answer the same object twice as the oracle does and closes the identity
concern. Boundary I-A: 49. Prose after behaviour. Recorded not fixed: a call in a method's argument
list re-resolving, resolution before argument evaluation, an external file's error losing the
caller's traceback line, `interpret "::requires"`'s traceback, `.Routine~new` with a context
(Phase 5), `Refused::Unfilled`'s doubled suffix.

### Task 2 fix round 2: DONE_WITH_CONCERNS, `eab19ac3b`..`9ff11e465`

`eab19ac3b` call sites keep `Routine` and `MergedLibraryRoutine` for good, drop `Library`,
`Internal`, `LibraryRoutine`, `External` on a generation move, `~addRoutine`/`~addPublicRoutine`
move it (checked against `ExpressionFunction.cpp:179-215`, `CallInstruction.cpp:159-198`,
`RexxActivation.cpp:3062-3105`; a writes table in the report); `c08f4badd` one `find_routine`,
`routines` up the parent chain then merged tables up the chain, for calls and `~findRoutine`;
`a15a8603a` one rooted `Routine` per `library_codes` row, a capped 400,000-send binary test (peak
flat ~125 MB); `5d08b1bce` `DisplayCondition` 49; `9ff11e465` prose (the "oracle's build" text was in
five places, not two). Each revision gated in its own target dir: corpus 586, 587, 588, 588, 588
matching; the known L0 failure; Miri 24 passed. `call ext i` unchanged (~664k), `rexxcps` +0.02%.

Concerns: a routine put into `.ROUTINES` from Rexx is never found by a call (recorded, not fixed);
Step 3 answers kept between routine-table writes rest on the writes table; **`/tmp` filled and voided
four gate runs**. The implementer attributed the space to other sessions; measured by the controller,
**this session's scratchpad held 39 GB of the 45 GB used**, almost all cargo target directories from
finished reviews and fix rounds. The controller deleted the 77 target directories it found (each
identified by its `CACHEDIR.TAG` and `.rustc_info.json`, deleted by explicit path) with no agent
live: scratchpad 39 GB -> 13 GB, `/tmp` 72% -> 30%. Every later dispatch checks `df` before
building and deletes its own targets at the end.

Re-review dispatched (opus), `review-bd64f3197..9ff11e465.diff`, report `task-2-rereview2.md`:
attack the kept-resolution rule itself on both sides (a kept `Routine` followed by `~addRoutine`,
a later merge, a `newFile` context), the writes table against the code, the one lookup walk,
identity and rooting of the shared `Routine`, the memory test's failure reason and cost.

### Task 2 re-review 2: Needs another round (1 Critical, 5 Minor)

`task-2-rereview2.md`. C1, I-B, m2-m4, boundary I-A and M-A..M-D closed by running; I-A partly:
keeping `MergedLibraryRoutine` for good is the oracle's Step 2 rule (`RexxActivation.cpp:3069`) and
holds past `~addRoutine`, same-name merges, parent writes and `newFile` contexts, but it opened C-1.
The writes table is complete. The root deletion reddens under collect-on-every-allocation. The
memory test fails at `bd64f3197` for its stated reason, passes at head in 1.41 s and has ~300 MiB
headroom under the cap. The `.ROUTINES` put never found predates the task (same at `e7cb210d9`).

* **C-1, regression:** a kept merged library routine can be one resolved before the call's own
  arguments merged a same-name routine (`o3`: oracle `helper 16` every pass, head `4` every pass,
  base wrong on pass 1 only); also `o3c`, `o3a`, `o2`; `o2r` wrong at base.
* Minor: the identity witness compares `identityHash` numerically (distinct objects can compare
  equal); no running gate catches deleting the new root (L0 stress stops first); four prose lines;
  `loadExternalRoutine` and `EXTERNAL "LIBRARY"` answer different objects from `findRoutine` (predates,
  and **the ledger's "closes the identity concern" above claimed more than was built**); the memory
  test needs the oracle checkout on every plain `cargo test`.
* Out of scope, same at base: a kept answer lives in one compiled chunk per trace setting (`a9`);
  a namespace call does not search the namespace package's parent (`ns1`); `.Routine~new(name,
  source)` without context gets no parent (`nf2`); `~addRoutine` refuses a library `Routine` (`ar1`).
  **Every recorded-not-fixed item of this task exists only in gitignored files.**

## Task 2 fix round 3: sent (round 3 of 5; the last intended for this task)

C-1 first, checking whether the oracle simply resolves after argument evaluation (then `o1` closes
too); m-1 identity test; m-4 extended if no new mechanism, else recorded; m-5 the oracle-absent
convention; m-3 prose; **and every recorded divergence of this task appended to
`phase-4-exclusions.txt`'s Phase 8 section with both sides' output and an owner**, its reader tests
run.

### Task 2 fix round 3: DONE_WITH_CONCERNS, `c55caa62f`..`cb13f0033`

`c55caa62f` **every call now resolves its routine after its arguments run**, the oracle's order
(`ExpressionFunction.cpp:185`, `CallInstruction.cpp:166`: only a label and a builtin index are
fixed before): `resolve_fixed_call` then `resolve_routine_call`; a kept Step 3 answer reused only if
the arguments moved no generation; `o1`, `o2`, `o2r`, `o3`, `o3a`, `o3c` and new `o4`-`o6` match the
oracle, `o1`'s recorded divergence closes. `668c73827` the identity witness compares hash strings
with `==` plus a distinct-objects control; `loadExternalRoutine` answers the shared object; the
directive route recorded (its object carries the directive's record). `a1ee6579c` the memory test
uses `support::oracle::locate()`, failing by name, never skipping. `273e8f608` prose. `3f48151ce`,
`cb13f0033` **a "RECORDED BY SURFACE TASK 2 AND ITS FIX ROUNDS, NOT FIXED" block in
`phase-4-exclusions.txt`** with fresh transcripts on both sides and owners (`Refused::Unfilled`
Task 3's, the rest no owner with reasons); reader tests green. Corpus 591 of 591 at every revision;
the known L0 failure; external calls unchanged within 0.1%, `rexxcps` -0.57%, unattributed.

Re-review 3 dispatched (opus), `review-9ff11e465..cb13f0033.diff`, `task-2-rereview3.md`: the
resolution-order change on every call kind (error precedence between a missing routine and a
raising argument, side-effect counts, traps, trace order, labels, builtins, INTERPRET, both
engines), kept answers under argument-side generation moves, the cost of internal and builtin
calls, the shared `loadExternalRoutine` object's retroactive package, five exclusions transcripts.
If it needs another round, round 4 goes to a fresh implementer on a more capable model, per the
skill.

### Task 2 re-review 3: no Critical; 1 Important (cost), minors -- closed by the controller

`task-2-rereview3.md`. C-1 fixed in every call form the round changed (function, `CALL`, dynamic,
qualified, `CALL ON`), with head right and base wrong on exactly the pass the arguments changed
across fifteen probes; error precedence, evaluation counts, `NOVALUE`/`SYNTAX` traps, labels, trace
output byte-identical to base; of 107 earlier review probes none regressed. m-1, m-3, m-5 closed;
m-4 built for non-REXX libraries.

**Ruling on I-1, the cost:** calls whose arguments are evaluated inside `invoke_call` got slower --
`length(length(i))` +1.2%, `f(f(i))` +0.7%, `call f f(i)` +1.0%, `filespec('N', length(i))` +2.3%,
`r(r(i))` +1.2% to +2.0% -- while direct forms are -0.6% to +0.1% and `rexxcps` -0.60%/-0.57%.
**Accepted and recorded here, not fixed:** the order is the oracle's and a correctness requirement
(C-1 was a silent wrong answer), and `rexxcps` moved the other way. -- **Recorded so cumulative drift
stays visible** (per-step thresholds hide it); the phase close must carry these figures into the
gate document beside the other axes. -- If wrong: a performance pass recovers the nested-call cost.

**Closed by the controller at `5d84dd8cb`** rather than a fourth fix round, as the reviewer
suggested: n-1's four false comments (and the `resolved_after_arguments` doc now cites
`ExpressionFunction.cpp:184` and `CallInstruction.cpp:166`, printed); n-2's exclusions corrections
(the `~addRoutine` mechanism sentence, the nested-call extent, the root entry's revision, the `j2`
elided oracle lines, the header); n-3 and the out-of-scope items as new entries (namespace-qualified
call resolving first, `LIBRARY REXX` routine identity, a raised call kept at its site, the missing
`>>>` for an external function, the `Compiled method "CALL"` traceback line, three loud Phase 5
refusals sourced from the report and not re-run, the unmeasured library-versus-internal collision).
Reader tests green. A stale check-profile artifact made `cargo check -p rexx-exec` fail on a method
that exists (`ScopePools::entries`); touching `body.rs` rebuilt it and it passed.

**Plan amended at the same commit:** Task 5 takes the call-context members `orxfunction.cpp` calls
(Task 2's Q2 ruling); Task 4 Step 3 says a refusal recorded inside a package hook is read there.

**Task 2: complete.** Commits `4c7d65e97`, `c23c214f3`, `a873b0678`, `fa7c51301`, `6245fe1ca`,
`25bdb2dd5`, `cae7d5382`, `2804a8986`, `79f08bed6`, `bd64f3197`, `eab19ac3b`, `c08f4badd`,
`a15a8603a`, `5d08b1bce`, `9ff11e465`, `c55caa62f`, `668c73827`, `a1ee6579c`, `273e8f608`,
`3f48151ce`, `cb13f0033`, `5d84dd8cb`. Three fix rounds; the first two each introduced silent wrong
answers that their re-reviews caught. Corpus 591 of 591 at `cb13f0033`.

## Task 3: dispatched (Moritz: "kick off task 3")

BASE `5d84dd8cb`. Brief `task-3-brief.md`: the plan's Task 3, the inherited context, the state after
Tasks 1-2, S2 for routines, `ResultSignature` for special return codes, `Refused::Unfilled`'s doubled
suffix, the rows shipped extensions reach (`rxmath` `CSTRING` result, `orxmethod` `size_t` result),
forged extensions for unit tests only, the never-load and never-register rules, Miri, the `/tmp`
rule. Model opus, report `task-3-report.md`.

**Dispatched while the post-Task-2 gates are still running** (G1 0, G2 0, G3 101 read so far):
the implementer is told not to edit the worktree, build in `rust/target/`, or write with `git` until
`scratchpad/gate8t2/status.txt` says `ALLDONE`, and to read and probe meanwhile.

### Gates at `5d84dd8cb` (after Task 2), clean before and after, HEAD unmoved

G1 0, G2 0, G3 101 (2582 passed / 2 failed), G4 101 (2582 / 3), G5 0 (591 of 591). Report-mode
figures unchanged. `scratchpad/gate8t2/`.

* **G3's failing set changed.** Left: the three `ir::drive` counter tests
  (`a_call_site_resolves_once_and_answers_from_what_it_kept`,
  `a_long_constant_is_built_once_however_many_passes_read_it`,
  `the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk`) and
  `a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`. No commit since `e7cb210d9` touched
  their test files, so a source change during surface Tasks 1-2 made them pass; **the commit is not
  yet identified -- attribute it before the phase gate document is written.** Remaining:
  `the_l0_subset_passes_again_under_collect_on_every_allocation` (known) and
  **`the_table_holds_every_constructor_the_source_defines`, joined at `5d84dd8cb`: the controller's own
  comment edits moved `run.rs` lines under `refusal-sites.tsv`'s definition column.** G4 is G3's set
  plus `concept_and_class_gate_table` (known).
* The table's re-derive sent to the Task 3 implementer as its first, separate commit, since the
  worktree is its now.

### Task 3: implementer returned DONE_WITH_CONCERNS, `e0b30d156`..`034c1c7d7`

`e0b30d156` the refusal table's column 4 re-derived (the controller's drift, fixed first as asked);
`edcf1ff40` every `REXX_VALUE_*` row both ways, `Failure::Unfilled` gone (its doubled suffix and
exclusions entry closed), special result codes `ResultSignature` (40.918 routine, 93.968 method,
forged lib), `ARGLIST` lifts the too-many check in both (S2), integers through new
`Number::int64_value`/`unsigned_int64_value` copying `NumberString::int64Value` including two
measured oracle quirks (a sign dropped on carry, an undetected overflow wrap), `POINTERSTRING` per
`sscanf "0x%p"`, stems resolved in a routine's caller, `float`/`double` results at 9 digits, the
`CSTRING` result copied in `load.rs` (Miri 27 passed), a Task 2 silent divergence fixed (a native
refusal's `found` for an array); `c00d18052` frames reused to recover +280 instructions per native
call, now ~2.6-2.7% below pre-task on 20,000-send loops; `034c1c7d7` exclusions. Corpus 603 of 603;
the known L0 failure. Row-deletion control on `uint16_t` exact.

Concerns: `MutableBuffer`/`VariableReference` success and special result codes unit-test and
forged-lib only; `POINTERSTRING` overflow value unmeasured; 9-digit rendering not measured under
`::OPTIONS DIGITS`; a trapped native refusal's condition object names the caller (not run against a
pre-task binary); **`signal on syntax; y = 1/0; ...; say condition('O')~message` panics under
collect-on-every-allocation with no native call**, at the L0 failure's assertion, so the stress test
reads codes only; C2 (removing new `push_temp`s) did not redden its test; `positive_wholenumber_t`
host path now `int64_value`, "differ only past 18 digits" read not measured.

Review in two slices on `review-5d84dd8cb..034c1c7d7.diff` (316 KB): rows and `rexx-num` (fable,
`task-3-review-rows.md`) and host, frame reuse and records (opus, `task-3-review-host.md`), the
latter asked to reduce the `condition('O')~message` panic and say whether an ordinary run reaches it,
Critical if so.

### Task 3 review, rows slice: Needs fixes (2 Important, 3 Minor)

`task-3-review-rows.md`. Every header code has one row, none twice; each row read against its
`processArguments`/`valueToObject` case with every citation landing; the integer semantics
(`int64_value`/`unsigned_int64_value` step for step with both quirks), `NUMERIC DIGITS` 3/5/12/40,
`FORM ENGINEERING`, `::OPTIONS DIGITS` in caller and required package, routine and method alike
(`double` at 9 digits on every path), integer edges to 41 digits, results at every width's extreme,
positions beside special parameters, `ARGLIST` lifting 88.922 -- all SAME. A forged echo measured the
`POINTERSTRING` overflow value (concern 2 answered) and the `MutableBuffer`/`VariableReference` success
paths. Miri 27 passed; a `CSTRING` read past its allocation is UB under Miri and green on stable.
Controls: deleting `int8_t`, deleting `SUPER`, a wrong class and a wrong object for the instance rows
each redden exactly the predicted tests. Concern 6's premise wrong (base used `whole_value` at 20
digits) and conclusion right.

* **I1:** a `logical_t` refusal names the argument (`found "a M"`); the oracle names the string
  `truthValue` tested (`found "2"`), for any object reaching it through `MAKESTRING`/`STRING`.
* **I2:** `pointer_string` lacks glibc's `(nil)` form: `0x(nil)`, `0x (nil)`, `0x(nil)zz`, `0x(NIL)`,
  `0x0x(nil)` convert to 0 on the oracle, 88.919 here.
* Minor: `RESULT_DIGITS`'s comment cites a non-discriminating measurement; the report's concern 6
  says 18 digits; `int_from_native` bypasses the host seam.

Waiting on the host slice before the fix round.

### Task 3 review, host slice: Needs fixes (1 Critical pre-existing, 2 Important, 6 Minor)

`task-3-review-host.md`. All twelve new witnesses and the reviewer's probes byte-identical; stems
from every caller kind; specials through `FORWARD`, lower-case `~send`, scope override; frame reuse
leaks nothing (refusal then success, omitted flags, argument arrays, names and scopes, nested
calls), RSS flat to 300k iterations, two mutants each redden the reuse unit test; the cost claim
reproduced (-2.6% sends and routines, `rexxcps` within noise); `refusal-sites.tsv`'s column-4 commit,
new rows and `SHARED_ANSWERS` all derived and right.

* **Critical, pre-existing, and the root cause of the long-red L0 stress test:**
  `build_condition_object` (`condition.rs:70-146`) holds the entry values in a plain `Vec` until the
  `PUT` loop, unrooted. **An ordinary run panics**: a loop growing an array while a procedure traps
  `1/0` and reads `c~message c~errortext` -- oracle `done 20001` rc 0, head rc 101 at
  `dispatch.rs:1510`, deterministic, the same at base. A `push_temp` per entry fixes it (mutant
  verified), and **uncovers six more stress-mode mismatches** (`condition_object.rex`,
  `address_with_stream.rex`, `executable_context.rex`, `sys_file_functions.rex`,
  `security_manager.rex`, `call_miss_not_cached.rex`), none recorded, ordinary-run reachability
  unmeasured.
* **I1:** `logical_t`'s `found` (also the rows slice's I1).
* **I2, pre-existing and widened by this task:** `found` never sends `OBJECTNAME`/`DEFAULTNAME`; the
  new comment says the oracle's `stringValue` sends nothing, but it sends `OBJECTNAME`
  (`ObjectClass.cpp:1157`) which sends `DEFAULTNAME` (`:1713`).
* Minor: `phase-8.txt:205` "every conversion row"; the POSITION/PROGRAM exclusions entry is
  pre-existing (reviewer ran base); a `refusal_sites.rs` comment; "emptied" three times; a witness
  header; a `PROCEDURE` not trapping a `raise syntax` inside `DEFAULTNAME` (pre-existing).

## Task 3 fix round 1: sent

To the implementer, both slices. **Rulings:** the Critical is fixed in this round although it
predates the phase -- an ordinary-run crash with a verified local fix -- and the six mismatches it
uncovers are reduced one by one: local missing roots fixed with witnesses, anything else recorded
with its ordinary-run reachability; bounded, stopping on a design problem. Host I2: match the
oracle's `stringValue` (send `OBJECTNAME`/`DEFAULTNAME`) for a native refusal's `found`, after
listing and measuring every caller of any shared helper so no other path changes where the oracle
does not. Rows I2: glibc's `(nil)` forms, checked against the reviewer's C program first. Order:
Critical, the six, `logical_t`, `OBJECTNAME`, `(nil)`, minors.

## Queued by Moritz, 2026-09-15: split every Rust file to at most 1,000 lines

*"queue a task to split files into smaller ones. No more than 1k LoC."* Drafted as its own plan at
`.superpowers/sdd/queued/2026-09-15-file-split.md` (gitignored until the worktree is free; it moves to
`docs/superpowers/plans/` with its first commit). **Runs after this plan's Task 3 closes and before
Task 4**, when no agent holds the worktree. Measured at `5d84dd8cb`: 48 `.rs` files over 1,000
physical lines, 110,195 lines between them. Assumptions stated in the plan: physical lines as `wc -l`
counts them, tests and comments included; pure moves proved by the four instruments the `run.rs`
split used; the three file-granular invariant tests (`unsafe_sites.rs` for D-U1, `environment_seam.rs`
for D45, `dispatch_seam.rs`) kept meaning what they mean, and **if `ffi.rs`/`load.rs` cannot reach the
limit by moving only safe code, it stops and asks** (widening D-U1 is Moritz's); a shrinking allowlist
test first so nothing grows back while the work proceeds. It supersedes the 2026-08-15 ruling that
declined carving `impl Interp`.

**Refined by Moritz the same day:** *"1k LoC is definitely approximate, and up to negotiation. It's
more a 'keep modules self-contained and single-purpose', without encoding too much opinion."* The
queued plan now treats ~1,000 lines as a trigger for reading a file, not a limit: no length test and
no allowlist; its first task is a survey that proposes, per candidate, a split by responsibility or a
reason the file is one thing, ruled by the controller before anything moves; `ffi.rs`/`load.rs` stay
as they are if self-containment would move `unsafe` out of them. The pure-move instruments,
file-granular invariants, derived-table re-derives and performance measurements stay.

### Task 3 fix round 1, part 1: `2d155158c`, and the L0 stress test is green

The Critical and the six mismatches it uncovered are three missing roots, each reduced and fixed:
a condition object's entries while it is built; **`Interp::resolve_stream`'s name, sent to `.Stream~new`
unrooted** (five of the six, and reached in an ordinary run: 40,000 `stream(name, 's')` calls are rc
120 before the fix, rc 0 and identical to the oracle after); **a `CALL ON` handler's condition object**,
which `Interp::object_roots` did not walk for a running activation. Each has a unit-test witness with
a control that reddens it; the stream witness was withdrawn from the corpus because
collect-on-every-allocation ran it for 46.8 s. **`collect_stress`'s L0 test passes -- exit 0, 32
passed -- for the first time since before this phase.** `NO_ALLOCATION_PROGRAMS` was stale by
seventeen programs (sixteen Phase 8 `library_*` and `trace_debug_skip.rex`, each dated by
`git log --diff-filter=A`) and one program that now collects; the list is the observed set again.
Control C8 closed the exclusions entry claiming no gate sees the imported-routine root.

**The agent was stopped by a session limit mid-round** (reset at midnight) with items 3-6 uncommitted
in six files; the controller copied the tracked diff to `scratchpad/t3-fix-backup-0416/` without
touching the tree and resumed it at 04:16, asking it to name the commit where the L0 test turns green
and to say in `NO_ALLOCATION_PROGRAMS`' doc how a reader re-derives the list.

## Task 3, fix round 1, part 2 (items 3-7)

Commit `6c96144d8`, "Report a native refusal's found through the stringValue
protocol". Status DONE_WITH_CONCERNS.

- Item 3: `Failure::NotLogical` carries the object tested; `Host::logical`
  answers `Result<Result<bool, ObjRef>, Raised>`; the rexx-exec host answers the
  converted string. Corpus witnesses for the oracle's four cases.
- Item 4: the shared `string_value_text` was left alone. The native refusal got
  its own `Interp::native_found`, sending `OBJECTNAME` for an instance and
  leaving a buffer's or pointer's own string value alone. The guard came from a
  measured regression: sending made a MutableBuffer argument report
  `found "a MutableBuffer"` where the oracle reports `found "buf"`.
- Item 5: `(nil)` accepted case-insensitively after the whitespace run, no sign,
  and not after an inner `0x`/`0X` prefix. Measured with `sscanf` in C first.
- Item 6: minors done. `int_from_native` deliberately stays direct: routing it
  through `Host::whole_number` makes
  `a_collection_inside_the_call_leaves_the_cself_intact` see a second collection.
- Item 7: the `PROCEDURE`/`DEFAULTNAME` trap recorded in the exclusions with its
  transcript, plus a second pre-existing divergence (an instance whose `STRING`
  answers a non-string converts to its own default name here, where the oracle
  converts that answer again).

Gates at `6c96144d8`: crates `--release --no-fail-fast` exit 0, 71
`test result: ok`, no failures; `REXX_CORPUS_GATE=1 ... --test corpus` exit 0,
604 of 604. fmt, clippy and Miri (`-Zmiri-strict-provenance`, rexx-api --lib,
27 passed) green before the commit.

**The L0 stress test becomes green at `2d155158c`**:
`the_l0_subset_passes_again_under_collect_on_every_allocation` in
`crates/rexx-exec/tests/collect_stress.rs`. Cite that commit in the phase gate
document. `NO_ALLOCATION_PROGRAMS`' doc now says how to re-derive it.

Concerns relayed: the two recorded-not-fixed divergences; `gate_table_c` failing
under `REXX_CORPUS_GATE=1` with 82 unanswered closed-phase rows; the 88.914
MutableBuffer and VariableReference rows witnessed only on the refusal side.

Ruling: the implementer's argument that `gate_table_c` is pre-existing rests on a
pristine tree at `2d155158c`, which is this round's own commit and therefore
proves nothing about "pre-existing". The re-review is dispatched with an
independent check at `e0b30d156`, the commit before Task 3's first. Cost if
wrong: one extra pristine build.

Scoped re-review dispatched on opus with
`.superpowers/sdd/2026-09-14-phase-8-surface/task-3-fix1-rereview-brief.md` and
the package `review-034c1c7d7..6c96144d8.diff`.

## Task 3, fix round 1, re-review

CHANGES REQUESTED, prose only. Report:
`.superpowers/sdd/2026-09-14-phase-8-surface/task-3-fix1-rereview.md`.

Findings 1-7 all closed, each re-measured by the reviewer rather than read:
`collect_stress` rebuilt at head (32 passed); the `(nil)` rule diffed against
real `sscanf` on glibc 2.43 over generated inputs with zero divergences and a
live control that reddens when the acceptance arms are deleted; a counting
`DEFAULTNAME` across four shapes showing findings 3 and 4 compose with no double
run and no absence; oracle-versus-crate byte comparisons over the receiver
shapes the `native_found` guard classifies, so the guard is faithful rather than
a special case for the two shapes that were measured.

The refusal-side-only witnesses for the 88.914 MutableBuffer and
VariableReference rows are accepted: the reviewer ran all eight shipped
declarations reaching those rows, six refuse loudly at rc 120 and two abort the
process at rc 134 (`MutableBufferData`, `rexx-api/src/layout.rs:382`, identical
at `034c1c7d7`), so a success-side corpus row would have aborted the harness.

**`gate_table_c` at `e0b30d156` fails identically (82 rows, `21 passed; 1
failed`), so pre-existing is confirmed.** The report's description of those rows
was wrong: they are phase 7's alone, File instance 50, Stream instance 24,
StreamSupplier instance 8. Alarm and Ticker are phase 6 and are not gated.

New findings, all prose except N5: N1 the refusal match's head comment contradicted
by its own `NotLogical` arm; N2 the exclusions calling three corpus programs
in-crate and replacing their names with a count; N3 phase-8.txt's false split of
the conversion rows; N4 a negative extent clause in `lib.rs`; N5 one rule in two
non-exhaustive copies (`native_found` against `redirect_of`); N6 the stream-name
root's second, louder witness unrecorded.

Ruling: N5 becomes a type-level fix rather than a comment. One inherent method on
`NativeState` matching its variants exhaustively, called from both sites, so a new
variant is a compile error instead of a silent classification. Cost if wrong: one
method and two call sites to revert. Rationale: this project has shipped the same
defect three times where the mitigation was dispatch prose, and only the
type-level fix held.

Fix round 2 dispatched to the same implementer with
`.superpowers/sdd/2026-09-14-phase-8-surface/task-3-fix2-brief.md`: N1-N6, a
stale comment pair at `value.rs:999-1004` that I found while reading N5's sites,
and the report's own wrong description of the 82 rows.

Separately dispatched `gate-table-c-probe` (sonnet) to establish since when
`gate_table_c` fails under `REXX_CORPUS_GATE=1`, whether anything licenses the
rows, and whether the rows or the table's expectation are wrong. It is read-only
on the worktree and builds only from pristine archives, so it can run beside the
fix implementer. This matters beyond Task 3: the whole-workspace corpus-gated
run is one of the five gates, and the per-crate gate runs this phase has been
using would not have shown it red.

## `gate_table_c` under the corpus gate: probe result

Report: `.superpowers/sdd/2026-09-14-phase-8-surface/gate-table-c-probe.md`.

First failing commit `98a0db498`, "Repair the eleven test binaries Phase 7 left
red, and the cadence that hid them" (2026-09-12), verified against its parent
`5bcb28edb` by pristine builds in separate target directories: parent exit 0
(22 passed), candidate exit 101 (21 passed, 1 failed, the same 82 rows). The row
set has not changed since 2026-09-03. What changed is that `"7"` entered
`CLOSED_PHASES` in `gate_tables/mod.rs` while those 82 rows were already
`unanswered`.

`REXX_CORPUS_GATE=1` changes nothing about what is measured: every probe runs
and every verdict is computed either way. It decides only whether a non-`Agree`
verdict on a row owned by a closed phase turns the test red.

**The crate is not wrong.** Two rows checked against the oracle from fresh empty
directories on three descriptors: `file__instance.rex` and
`streamsupplier__instance.rex` are byte-identical to the oracle in exit code,
stdout and stderr (163/93.901 and 159/97.1). `unanswered` means the oracle's own
probe raised before reaching the question, because gate table C's row set never
commits a working construction expression for File, Stream and StreamSupplier.
The sibling gate `method_bodies.rs` already has one (`RECEIVER_OVERRIDES`, with
`.File~new('/')`), gets oracle-agreeing answers from it, and its cross-check
explicitly skips reconciling with gate table C for exactly these three classes.

Not licensed by the mechanism this project uses: neither
`phase-4-exclusions.txt` nor `licensed_divergences.rs` mentions the rows. It is
on the record in `docs/superpowers/plans/phase-7-gate.md` section 8, which
documents the whole-workspace corpus gate exiting 101 with eight failures at
Phase 7's own close commit, names `concept_and_class_gate_table` among them, and
declares the phase's exit criterion met by a sentence that does not cover that
gate. That contradicts `rust/CLAUDE.md`'s gate rule, which expects the same
command to exit 0.

Ruling: this becomes its own task, taken before the queued file-split plan and
before Phase 8's gate document is written, not folded into Task 3. The crate is
correct, so the work is to give gate table C's row set a construction expression
per the sibling gate, and then to decide each row that remains non-`Agree` on its
merits. Cost if wrong: a task's worth of rework, and the file-split plan starts
later. Rationale: a gate that has been red since 2026-09-12 hides every
regression it would otherwise catch, and Phase 8 cannot claim a green gate set
over it.

Open question for that task, not answerable yet: the probe reports eight
failures under that gate at Phase 7's close, of which this is one. The red set
today is unmeasured. Measure it with one whole-workspace corpus-gated run once
Task 3's fix round 2 has committed and the tree is free.

## Task 3, fix round 2

Commit `a442e0b5b`. Status DONE, no observable changed, all eight items.

Item 5 was the only code change: `NativeState::renders_its_own_string_value`,
an exhaustive match over the variants, answering for both `native_found` and
`Interp::redirect_of`. Held observable-free by two checks: the corpus witness
byte-identical on all three descriptors before and after and matching the oracle
on both sides, and `-p rexx-api --test values` at `83 passed` either way.

Item 6's number was measured before it was written, as asked. Control C9,
prediction first: removing the stream name's `push_temp` reddens exactly
`address_with_stream`, `executable_context`, `sys_file_functions`,
`security_manager` and `call_miss_not_cached`, each stress run rc 120 against a
matching plain run, and no other program. Confirmed on all four parts, `run.rs`
restored with an empty diff.

Implementer's gates at the commit: crates `--release --no-fail-fast` under
`memcap 16G` exit 0, 82 `test result: ok`, L0 stress test ok; corpus exit 0,
604 of 604; fmt and clippy exit 0 beforehand.

Scoped re-review dispatched on sonnet against
`review-6c96144d8..a442e0b5b.diff`, framed as a neighbourhood review rather than
a findings review, because this project has measured correction rounds
introducing new false statements at 5, 0, 4, 0 across four previous rounds.

Controller running the five gates at `a442e0b5b` from a clean tree:
`scratchpad/gates-a442e0b5b/`, each exit code recorded separately, no pipes on
the gate lines. This is also the run that measures the true red set under the
whole-workspace corpus gate, which the probe showed has been red since
2026-09-12.

## Task 3, fix round 2, re-review

APPROVED. Report:
`.superpowers/sdd/2026-09-14-phase-8-surface/task-3-fix2-rereview.md`.
All eight items closed, no new blocking findings.

Verified rather than trusted: the reviewer extracted both `6c96144d8` and
`a442e0b5b` with `git archive` into its own directories with separate target
directories, and measured `-p rexx-api --test values` at 83 passed on both, and
the corpus differential at 604 of 604 matching on both. It checked the
exhaustive match variant by variant against the two old spellings, confirmed the
`MutableBufferLength`/`VariableReferenceValue` rc 120 description against
`layout.rs` and `testbinaries/orxmethod.cpp`, and found no other site
open-coding the `buffer()`/`pointer()` rule and no other duplicated comment head.

Parked, not fixed: `dispatch/library.rs:250-251`'s new comment is imprecise
about mechanism for the `Decoded::SmallInt` short-circuit in
`Refused::NotLogical`, where `found` is the raw argument rather than the string
the conversion answered. The displayed text is identical either way, the wording
mirrors approved corpus-header prose, and the brief directed it. Ruling: leave
it. Cost if wrong: a comment that names the right observable for a slightly
wrong reason, in an arm whose output is already witnessed on both sides.

Gates at `a442e0b5b` (controller's own run, `scratchpad/gates-a442e0b5b/`):
g1 fmt exit 0, g2 clippy exit 0. g3, g4, g5 still running.

## Gates at `a442e0b5b`: first attempt was blind to its own subject

The first run reported all five gates exit 0, including the whole-workspace
corpus gate that the probe had just shown red since 2026-09-12. The
contradiction was the tell: **my script omitted `REXX_CORPUS_GATE=1` from both
corpus-gated lines**, so gates 4 and 5 ran the ordinary suite twice and could not
see the thing they exist to check. A green reading from a command that cannot
observe its subject is indistinguishable from a real pass, which is why the
probe's independent measurement is what caught it rather than the gate output.

Standing at `a442e0b5b`, from a clean tree, rev unchanged before and after:
g1 fmt exit 0, g2 clippy exit 0, g3 release workspace exit 0 (59 test binaries
`ok`, no failures). g4 and g5 are void and re-running as `run2.sh`, which sets
the variable and, before the gate lines, records `printenv REXX_CORPUS_GATE`
from a child process so the run carries proof the variable reached one.

## Gates at `a442e0b5b`, corrected run

`scratchpad/gates-a442e0b5b/run2.sh`, clean tree, rev unchanged before and
after, and `printenv REXX_CORPUS_GATE` from a child recorded as `1` beside the
gate lines so the reading carries proof the variable reached a child process.

- g1 fmt exit 0
- g2 clippy exit 0
- g3 release workspace exit 0, 59 test binaries `ok`, no failures
- g4 `REXX_CORPUS_GATE=1` workspace exit **101**, 132 `test result: ok`, exactly
  one binary failing: `concept_and_class_gate_table` (`gate_table_c.rs:1795`),
  21 passed 1 failed
- g5 `REXX_CORPUS_GATE=1 --release -p rexx-exec --test corpus` exit 0

**Task 3: complete.** Commits `e0b30d156`, `edcf1ff40`, `c00d18052`,
`034c1c7d7` (task), `2d155158c`, `6c96144d8` (fix round 1), `a442e0b5b` (fix
round 2). Both fix rounds re-reviewed; round 2 APPROVED with no new blocking
findings. The only gate not green at the close commit is the pre-existing
`gate_table_c` failure, which predates Task 3 by four days and is taken next as
its own task.

## The red set under the corpus gate, measured

Today it is one test, not the eight `phase-7-gate.md` section 8 recorded at
Phase 7's close: seven of those have since been fixed. `gate_table_c`'s own
report gives the shape, so the repair does not need rediscovery:

- verdicts over the whole table: agree 1378, unanswered 110, loud 0
- unanswered rows by owning phase: phase 7 has 94 rows of which 82 are not yet
  `agree`; phase 6 has 13 rows, all 13 not yet `agree`;
  `deferred-rexxcontext-stackframes` 10; `never-expected-to-agree` 5
- gated by the run: the 82 whose owning phase is closed. Phase 6's 13 are red by
  the same cause and are **not** gated, because `"6"` is absent from
  `CLOSED_PHASES` while `"7"` is present. Whoever takes the repair should say
  whether that absence is deliberate.
- the rows carry the cause in their own transcript: both sides answer rc 163
  with a `Method INIT with scope "Stream"` / `"TICKER"` traceback, so oracle and
  crate agree that the probe raised in construction before any documented name
  was asked. `unanswered` here is a property of the row set, not a divergence.

## Inserted task: gate table C repair

Dispatched to the same agent that investigated it read-only, resumed with its
write constraint lifted, so the terrain it already measured is not rediscovered.
Brief: `.superpowers/sdd/2026-09-14-phase-8-surface/gate-table-c-repair-brief.md`.

Ruling on where it sits: before the queued file-split plan and before surface
Task 4. A gate red since 2026-09-12 hides every regression it exists to catch,
and Phase 8 cannot claim a green gate set over it. Cost if wrong: the file-split
plan and Task 4 both start later.

Ruling on `phase-7-gate.md` section 8: a dated correction beside the original
sentence, not a rewrite. A gate document records what was believed at the time,
and silently correcting it destroys the evidence of how the belief failed. Cost
if wrong: one paragraph of clutter in a record nobody rereads.

Ruling on phase 6: its 13 non-agreeing rows are ungated because `"6"` is absent
from `CLOSED_PHASES`. The repair reports what adding it would turn red but does
not add it in the same commit, so the repair's own effect stays measurable.

## Gate table C repair: reported

Commits `a4b6a37ce` (the construction expressions) and `5757857a4` (the
exclusions entry and the `phase-7-gate.md` correction). Report:
`.superpowers/sdd/2026-09-14-phase-8-surface/gate-table-c-repair-report.md`.

The agent went idle after both commits without reporting, and the report file
did not exist; I chased it. Same shape as the four silent finishes already on
record: an idle notification is not a report, and the trigger for checking is
the event, not the clock.

Claimed at `5757857a4`: the corpus-gated workspace run exits 0, 133 of 133
binaries ok, 2636 passed 0 failed, and gate table C's own line reads
"gated by this run: 0". Verdicts move agree 1378 -> 1460, unanswered 110 -> 15,
loud 0 -> 13, gated 82 -> 0.

**The 13 new loud rows are the interesting half.** Giving Alarm and Ticker a
construction expression turned their rows from `unanswered` into a measured
divergence: the oracle constructs and answers every `hasMethod` at rc 0, this
crate refuses loudly at rc 120 because `alarm_startTimer` and
`ticker_createTimer` are not implemented. So the repair did not only make 82
rows agree, it made a real gap visible that the missing receiver had been
hiding. Recorded in the exclusions with its transcript and `OWNER: Phase 6`,
which is the owned-gap mechanism rather than the licensed-divergence one; the
entry ends by saying this row is what closing Phase 6 would have to answer for
first.

Phase 6's absence from `CLOSED_PHASES` is correct, not a second instance of the
same hole: the master plan carries no CLOSED marker for it and its exit gate is
plainly unmet. Measured what closing it would cost with `REXX_PHASE_GATE=6`
rather than by editing the constant: exactly those 13 rows.

The two diff surprises are answered. `class-methods.txt` and `class-set.txt` are
regenerated by the command each file's own header names, and a hand edit would be
rejected by `extract_docs.rs`'s round-trip test; the 214 and 10 changed lines are
107 and 5 rows doubled by diff, and they move because status and reason are
stamped from the class onto every row of that class in both arms. The
`rexx-extract/src/docs/classes.rs` change is the task itself:
`CONSTRUCTION_PROGRAMS` is the only place gate table C's row set can take a
construction expression from.

Controller's independent corpus-gate run at `5757857a4` is in flight
(`scratchpad/gates-5757857a4/`), with the variable's arrival at a child recorded
beside it. Verifying rather than accepting, because this repair edits the gate's
own input table, which is the one change shape that can turn a gate green by
altering the question it asks.

## Gate table C repair: independently confirmed

Controller's own run at `5757857a4`, clean tree, rev unchanged before and after,
`printenv REXX_CORPUS_GATE` from a child recorded as `1`:

    REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8
    exit 0, 133 `test result: ok`, 0 `test result: FAILED`
    gate table C: "gated by this run: 0"
    verdicts: agree 1460, diverge-both 13, unanswered 15, loud 13

Every figure matches the implementer's report. The gate that was red from
`98a0db498` (2026-09-12) to `a4b6a37ce` (2026-09-16) is green.

Checked the one soft spot in the report before accepting it: the five probe
`.rex` files were hand-updated to match the derivation, and hand-maintained
derived files are where drift hides. It is enforced, not trusted.
`gate_table_c.rs` calls `check_probe_text` for every method group against text
derived from the row set including its construction expression, so a hand edit
that did not match the derivation fails the gate.
`every_committed_construction_program_has_an_instance_arm_to_run_it` and
`every_method_rows_class_and_status_come_from_the_class_set` close the other two
directions.

Scoped review dispatched on sonnet (`gate-repair-review`) against
`review-a442e0b5b..5757857a4.diff`, framed around one question: did the gate go
green because the crate agrees with the oracle, or because the question changed?
That is the failure mode a repair to a gate's own input table has, and my own
green run cannot distinguish the two.

## Next

Phase 8 surface Task 3 and the gate repair are both closed. The file-split plan
is committed at `ec7824e4a` and its Task 1 survey is dispatched
(`file-split-survey`, opus, read-only, produces a proposal for a controller
ruling before anything moves). Its ledger is
`.superpowers/sdd/2026-09-15-file-split/progress.md`, with the pre-flight scan
recorded there. Surface Task 4 follows the file-split work.

## Gate table C repair: review found one real defect

CHANGES REQUESTED. Report:
`.superpowers/sdd/2026-09-14-phase-8-surface/gate-repair-review.md`.

Items 1 through 6 verified by running, not reading: File's and Stream's
constructions checked against the oracle from a fresh directory; the 82 rows'
probes re-run against both sides on three descriptors and byte-identical with
real, non-refusal answers, so the agreement is not two refusals matching; the
extractor command re-run from a pristine tree producing all five outputs
byte-identical to the committed ones, with every changed row moving
not-covered to covered and no status moving in a direction that excuses a
failure; `method_bodies.rs`'s change confirmed a forced consequence of the
`RECEIVER_OVERRIDES` fallback rather than an independent edit; the 13 loud rows
reproduced with the exact refusal text; section 9 confirmed a pure addition with
its dates and commits checked.

**The defect the review existed to look for was there.**
`rexx-extract/src/docs/classes.rs:219` embeds
`/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/gate-tables/fixtures/streamsupplier_seed.txt`
into StreamSupplier's construction program. I confirmed it by reading the entry.
That row is green only because this worktree sits at that path. The project
builds pinned worktrees at specific commits to verify gates and three exist now;
the next one built at or after `a4b6a37ce` will not find the file, and the row
regresses into something that reads like a code defect. The contrast inside the
same subsystem is the tell: `gate_table_c.rs:30` resolves the corpus through
`env!("CARGO_MANIFEST_DIR")`, and `FILE` and `STREAM` use absolute paths chosen
to be nonexistent, so both are portable.

So the green was real for File, Stream, Alarm and Ticker, and environmental for
StreamSupplier. My own confirming run could not have told the difference: it ran
in the one directory where the path resolves.

Fix dispatched to the same implementer: resolve the construction wherever the
repository is checked out, re-derive the row sets and the five probes by the
committed command rather than by hand, correct or delete the README and
`method_bodies.rs` sentences that describe the fixture's reference, and add a
test that fails if any construction program contains the repository's own
location, run in both directions. Verification must include a pristine tree
extracted to a **different path**, which is the condition the defect depends on
and the one this worktree's gate cannot observe.

## Gate table C repair, fix round 1: re-review APPROVED

Commit `5b0b69170`. Report:
`.superpowers/sdd/2026-09-14-phase-8-surface/gate-repair-rereview.md`.

The construction now resolves the fixture from the running probe's own location
(`filespec('path', .context~package~name)` and a relative hop) with the fixture
still committed. The finding closes, verified by the reviewer in two fresh
locations unrelated to this worktree and to each other: the committed probe run
in place against a freshly built crate and the oracle, rc 0 on both sides, eight
stdout lines byte-identical, stderr identical.

The new guard was shown to generalize rather than to match this machine: the
reviewer patched a pristine copy's own `classes.rs` to hardcode **that copy's**
absolute path and watched the guard fail naming it, then restored and re-ran
green. Grepping the touched files for a checkout-specific path found one
remaining site, `tests/support/oracle.rs`'s `oracle_root()`, which is
pre-existing, documented in `rust/CLAUDE.md` as a deliberate exception, and
untouched by either commit.

The reviewer also answered the question the implementer's account had not:
`run_probe` and the oracle wrapper both run the committed `.rex` in place,
canonicalized, with no staging, which is why the relative resolution works. Not
a coincidence of the two layouts tried, unconditional in the harness today.

Ruling: the residual becomes an assertion, not a note. `method_bodies.rs`
already stages probes into a temp directory for a different table, so the pattern
that breaks this row exists in the same subsystem; a comment does not fail when
the invariant does, and this project has shipped one defect three times where the
mitigation was prose. The harness checks that the program it is about to run
resolves inside the corpus directory, placed where the path is chosen so it
cannot drift from the runner, and proved both ways. Cost if wrong: one assertion
in a harness that never stages.

Ruling: the guard's scope stays as it is. It rejects this checkout's own path,
and a construction embedding some other machine's path would pass everywhere.
Accepted, because these entries are written in the checkout the test then runs
in, so an author's own mistake is caught where it is made. A second guard over a
case the workflow does not produce buys little and obscures which check is
load-bearing. Cost if wrong: a foreign path reaches the tree and fails in
whatever checkout first tries to resolve it.

## Ruling reversed: the guard gets the exists-check too

I ruled an hour ago that the checkout-root guard was sufficient, on the grounds
that entries are written in the checkout the test then runs in, so an author's
own mistake is caught where it is made. The reviewer, asked for a judgment rather
than a change, argued the other way and was right.

What my reasoning missed: this table's convention is that every entry is checked
against the oracle from a fresh directory **before** it is committed, so the
author runs the construction locally pre-commit. An exists-check fires in that
same run, in the author's own environment, where a mistaken absolute path is
exactly the kind that does exist. It closes the defect class at authoring time,
for any contributor's path rather than only this checkout's, and earlier than the
checkout-root check can.

Both checks stay: the checkout-root one is deterministic whatever the filesystem
holds and still catches a self-referential path after the file it names is
deleted; the exists-check is environment-dependent but catches a path never
derived from any repository root.

Scope is `CONSTRUCTION_PROGRAMS` only, and deliberately not
`method_bodies.rs`'s `RECEIVER_OVERRIDES`, whose `/`, `/dev/null` and
`/etc/hostname` are real OS paths by design because it sends documented methods
that need something to open. The reason the rule fits one table and not the other
is gate table C's own principle that it should never need a real file except
through the portable pattern this fix established, and the test's failure message
is to say so.

Zero false positives against the table as committed: the only absolute literals
left are the two deliberately nonexistent placeholders, and StreamSupplier's
literal is now relative. Folded into the same round as the in-place assertion.

## Gate table C repair: closed

`e6d9d1231`. Five gates, started 11:51:51Z, finished 12:08:21Z, all exit 0; 267
`test result: ok` blocks, no `FAILED`. Revision on the status file's first line,
tree clean before and after, no build processes left behind. I read the status
file directly rather than wait for the agent, which had gone idle without
reporting for the second time on this task.

Commits: `a4b6a37ce` (construction expressions), `5757857a4` (exclusions entry
and the `phase-7-gate.md` correction), `5b0b69170` (portable StreamSupplier
construction), `42530000e` (the in-place assertion), `e6d9d1231` (the
exists-check). Review at `gate-repair-review.md`, re-review at
`gate-repair-rereview.md`, report at `gate-table-c-repair-report.md`.

The agent flagged `docs/superpowers/plans/2026-09-15-file-split.md` showing
modified in the shared worktree and declined to fold it into its close-out. That
was mine, mid-amendment, committed since at `44bbf0959`. Declining to absorb
another writer's uncommitted file into your own result is the right instinct in a
shared tree.

## Phase 8 surface: where it stands

Task 3 closed at `a442e0b5b`. The gate table C repair is closed. The file-split
plan is committed and amended (`ec7824e4a`, `44bbf0959`) with its survey ruled.
Surface Task 4 follows the file-split work.

Standing correction to the roadmap's reading, noted 2026-09-16: **Phase 6
(Concurrency) has never been started** -- no plan, no spec, no gate document, no
CLOSED marker on its row. Skipping it was legitimate by the table's own
dependency column, since Phase 7 depends on 5 alone and Phase 8 on 5 and 7, but
Phase 9 depends on 6, and the debt is now visible in the exclusions: the
Alarm/Ticker rows this repair exposed, and an earlier entry saying a real `GUARD`
rather than a silent no-op is Phase 6's.

## Task 4 dispatched: thread context with the interpreter's lifetime, loader/unloader hooks; BASE `91f6afaba`

File-split plan code-complete (Task 11's report in progress, docs only). Brief
`task-4-brief.md` extracted and checked against the tree: `Contexts` at
`ffi.rs:200`, `layout.rs` `loader`/`unloader` fields at :314-315, the refusal
read at `invoke.rs:144`. Miri toolchain must be reinstalled (the L2 scratch
install is gone). Implementer `surface-4` (Opus).

### Task 4 implemented `e6dcc65d0`; gates red; fix round 1 sent

Thread context owned by the interpreter, callbacks resolve the innermost
native frame; kept context answers `x 42` as the oracle; no-frame callback
aborts rc 134 as the oracle (measured with a destructor); loader after
routine registration, never for a version-refused library, a raise becomes
the load's error; unloaders at termination for every held library in the
oracle's package-table order (modelled then matched), a raise stops the
rest. Miri Stacked Borrows 29/0/7. Hooks declared only by `hostemu`,
`orxinvocation` (need librexx), `rxsock`, `rxunixsys` (empty loaders).
Gates: G4 exit 137, memcap OOM while compiling (G3 did not build what G4
tests); G6 `refusal_sites`: `Raised::library_version` column 3
`body+send` -> `body`, a finding. Implementer concern: no in-repo test
reddens when the loader call is removed.
Ruling: fix round 1 -- gate script, library_version's sites before/after
with a run, and an in-repo loader/unloader witness through an in-memory
package entry in ffi.rs/load.rs tests (no C compiler, no new dependency,
no new unsafe file). Cost if wrong: a seam added for testing.

### Task 4 fix round 1: `61be5ecfb`; gates green

Gates at `61be5ecfb` (G3 now `test --no-run`, G4/G6 0 Compiling lines):
release 2671/0/4, debug 2672/0/4, 604 of 604 STRICT both. Review next.

### Task 4 reviewed: spec PASS, quality PASS with one Important; fix round 2 sent

Reviewer ran Miri (SB and TB) green, reproduced the old design's
use-after-free deterministically, compared 25 forge probes with the oracle
(loader/unloader order, raises, version refusal all match), confirmed
`library_version`'s sites unchanged and hook refusals loud. I1: libraries are
closed when the `Libraries` HashMap drops (random order), the oracle closes
each right after its unloader; the report's "identical" claim is false for
destructor lines. M1: nothing tests that execution calls `terminate`. M2
`load::hooks_only` pub doc(hidden), accepted. M3 abort stderr differs, M4
unloader context identity and SAY/printf interleaving: record as divergences.
Ruling: fix the close order in code (the oracle's order is cheap to match and
makes ours deterministic), add the terminate witness, record M3/M4. Cost if
wrong: one more gate cycle.

### Task 4 fix round 2: `fb860a6f2` (close order), `5a5c462ac` (divergences); re-reviewed

Gates at `5a5c462ac` green: 2673/0/4, 2674/0/4, 604/604. Re-review: all
findings addressed; forge destructor lines match the oracle on 26 probes and
are deterministic; closed code never entered; terminate witness live; Miri
SB green (the closed-hook test is Miri-ignored). Minors: m1 no witness for
`Drop for Libraries`; m2 exclusions entry narrower than its cause
(termination UNINITs too). Ruling: fix round 3, targeted checks only.
Records note: this plan's reports and reviews are still only in
`.superpowers/` (git-ignored); commit them under
`docs/superpowers/records/2026-09-14-phase-8-surface/` when Task 4 closes.

### Task 4 fix round 3: `aacd284d7`; complete

`dropping_the_libraries_closes_what_termination_left_open` reddens when the
drop's close is removed (predicted, 851/1). Exclusions entry widened to
termination UNINITs, citing `Interpreter.cpp:277-281`. rexx-exec release
1585/0/1.

**Task 4 complete at `aacd284d7`.** Records committed under
`docs/superpowers/records/2026-09-14-phase-8-surface/`.
