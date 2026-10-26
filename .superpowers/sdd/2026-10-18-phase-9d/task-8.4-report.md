# Phase 9d Task 8.4 report: install order, directives, scanner

Status: DONE_WITH_CONCERNS. Base `f0164456d`. Code `44f1cb56b`, `f84e64ee4`, `e54ed5724`,
`7fe50bedd`, `bb9cda7b5`; records and closings in the commit that adds this report. Records:
`docs/superpowers/records/2026-10-18-phase-9d/t8.4/`.

## What changed, per item

**2026-10-17-scanner-reports-later-clause-error.md and gate:phase-9c-gate.md:145 (`44f1cb56b`).** The
scan stops at the clause holding its error (`scan_to_error`, tokens truncated before that clause's
first token) and the parse reports the error only once the cursor is asked for a clause past the last
one (`ClauseCursor::read_past_end`), which is where the oracle's clause-at-a-time `nextClause` meets
it; an earlier parse error wins. A `::RESOURCE` without its end marker records no body, and the
directive parser raises 99.943 after its own checks (so a duplicate resource name, 99.942, now wins
as on the oracle). This closes the Phase 3 "eager scan" deviation (plan
`2026-07-28-phase-3-parser.md:464`): its tests in `tests/errors.rs` now assert the oracle's answer
(37.2 on line 1). The phase 3 plan and gate prose naming that deviation are historical and were not
edited.

**2026-10-18-duplicate-member-checked-at-install.md (`f84e64ee4`).** The directive parser keeps what
the directives so far declare (`Declared`) and refuses, at the oracle's own point in each directive:
a duplicate class (99.901), routine (99.903), resource (99.942), member key on a side (99.902,
99.931, 99.932, the `unattachedMethods` table before any `::CLASS`), a class-less `::CONSTANT`
expression (99.906), a missing `::ANNOTATE` target (99.945, before the pairs are read), and a package
DIGITS not above its FUZZ (33.1). All of these had to move together: moving the member check alone
would have reordered it against the others, which stayed in the install walk. The walk keeps only the
key tables `::ANNOTATE` staging needs; `Raised::duplicate_*` and `constant_needs_class` are gone
(refusal-sites re-derived, those rows removed and nothing else).

**2026-10-17-library-bind-before-options.md and gate:phase-9c-gate.md:142 (`f84e64ee4`).** A parse
that fails carries the directives whose `EXTERNAL` it passed the oracle's resolution point for
(`Rejected::binds`: after the body check `checkDirective`, and not when that check read past a scanner
error). `Interp::raise_parse_failure` resolves them first (`resolve_external`, factored out of the
walk) and reports a refusal in place of the parse error; the main-program path routes such a
rejection through the interpreter. Install rows added to `parse-errors.tsv` (the classification test
now expects 12): the item's pair, its `::METHOD ... ATTRIBUTE EXTERNAL` form, and a bind before a
later duplicate.

**2026-10-18-install-error-names-caller-file.md (`e54ed5724`).** `blame_directive` names the package
the directive is in (`program_display_name`) wherever it is not the main program, so a `CALL`, a
`::REQUIRES`, `.Routine~new` and `.Package~new` each report their own file or name, and a cyclic
`SUBCLASS` pair names it in 98.911's message. The translation-time refusals left in the walk (a
missing library or entry, a cycle) close the `COMPILE` frame with the package as its origin, as the
oracle's do, where a missing superclass still closes `ROUTINE`. The duplicate-routine instance
(`superclass_e`-like) is fixed by the parser move above. The `hs9`/`qpull` paragraph is not fixed:
Deviation candidate A below. The file is left open with a status line.

**2026-10-18-options-trace-constant-expression.md and 2026-10-22-constant-frame-and-executable.md
(`7fe50bedd`).** Derived from rexxref (`dire.xml:535`-`:556`): a `::CONSTANT` "defines a method", and
the expression form may name `self` (the example `::constant eSquare (self~e ** 2)`), so the expression
runs as a method of its class. It now runs in a `METHOD` level named `::CONSTANT` with the class as
receiver and scope, stopped on the directive's own clause, above a level for the package being
installed (`PROGRAM` for the main program, `ROUTINE` otherwise, stopped on the `::CLASS` installed
last, `Interp::install_call`). Its executable is a `Method` of its own
(`ExecutableSource::ConstantExpression`): guarded, every other flag clear, its directive clause as its
source, scoped to the class, not the accessor `~method` answers. The package's `::OPTIONS TRACE`
applies (the `PackageStart` now carries the trace setting), so the level announces `>I>`, echoes the
directive, traces values and `>>>`, and ends `<I<`, under each setting as the oracle does (I, R, A,
L measured; interactive `?R` not checked, Phase 11.1).

**Perf (`bb9cda7b5`).** The full callgrind run at `7fe50bedd` had startup +0.2683%, sayloop +0.1428%
and interpret +0.0536% libc subtracted (`callgrind-7fe50bedd.txt`), from the declaration tables in
every parse (the library bootstrap's above all) and a `Declared` built per `INTERPRET`. Now: names are
borrowed from the source, the tables are made at the first directive, an `INTERPRET` fragment and a
program with no directive build no declaration state, bound directives are kept as indices until a
parse fails, the interpreter's own library parses with `parse_known_program` (unchecked;
`tests::every_library_program_passes_the_declaration_checks` parses it checked), and a directive with
no `EXTERNAL` skips `resolve_external`. A `::METHOD "p="` files as the setter of `P`
(`member_key`), which `class_duplicate_attribute.rex` caught when the setter flag was first split out.

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file generated from a
copy outside the repository: `scan_error_waits_for_its_clause`, `translation_checks_in_source_order`,
`translation_error_in_called_file`, `install_error_in_called_file`, `install_error_in_routine_new`,
`install_error_in_required_file` (`.d` with `r.cls`), `constant_expression_level`,
`constant_expression_trace`, `constant_expression_trace_labels`. Each differs from the oracle with the
base binary (`f0164456d`, built before any change) and matches on all three descriptors at head:
`t8.4/scanner/witness-oracle-base-head.txt`, `t8.4/declarations/witness-*.txt`, the summaries in
`t8.4/*/probes-summary.txt`; the two called-file witnesses write their files into the run directory and
differ only by that directory's path in stderr. New `parse-errors.tsv` rows: `t8.4/scanner/rows-*.txt`
and `t8.4/declarations/rows-*.txt` give the base and head `rexxc` answer per row; the rows "same" at
base are the adjacent successes (the scan error still winning when the parse reaches it, a bind ahead
of a later duplicate, each check's plain form).

I read each witness's stdout: every scanner shape prints its parse error code, every declaration shape
its own code and line, the called-file programs the `COMPILE` frames, the constant program both
`METHOD ::CONSTANT` lists and the executable's flags, the trace programs the `>I>`/`<I<` lines.

Test lists updated because their subject moved: `program.rs` (`REFUSED_CORPUS_PROGRAMS`, the corpus
programs whose subject is a declaration error), `tiling.rs`/`variants.rs`/`ir/corpus_shape_tests.rs`
(skip `DECLARATION_ERRORS`), `collect_stress` `NO_ALLOCATION_PROGRAMS` (those programs now refuse
before running), `scanner.rs` (99.943 left the scanner).

## Commands

From `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t84-target`, at `bb9cda7b5`.

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-parse -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test collect_stress --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test compiled --test rexxc` (`checks-touched.txt`) | 101: lib 1178, signals 42, dispatch_seam 6, compiled 2, rexxc 4, every table binary passed; collect_stress 73 of 74, the failure the l0 subset on `command_redirected_target_waits_elsewhere.rex` (`waited 1` plain, `waited 0` under stress), run while callgrind loaded the machine |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress`, rerun (`checks-collect-stress-rerun.txt`) | 0, 74 passed |
| `memcap 16G cargo test -j 4 --release -p rexx-parse` (`checks-parse.txt`, includes `sourceline_oracle`, `errors`, `scanner`, `program`, `tiling`, `variants`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites` (`checks-unsafe.txt`) | 0 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: 1191 of 1192, the one mismatch `rexxutil_file_date_time.rex` (queued DST item) |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.4/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv` and `oracle-differences.tsv` identical to `t8.3/r2/suite/` |

A debug build of the fix round 1 tree ran every Task 8.4 witness (`r1/debug-witnesses.txt`).

## Perf

`callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.4/final base=.../t8.4/rexx-base-f0164456d head=.../t8.4/rexx-head-bb9cda7b5`, every program, 3 rounds. Exit 1, from one OVER flag: dispatch whole process +0.7247%, libc subtracted +0.0001% (the first full run at `7fe50bedd` had dispatch +0.0015% whole, so the whole-process figure is libc's). No SPREAD flag. Libc subtracted, the largest rises are startup +0.0214%, sayloop +0.0114%, churn +0.0035%, intcall +0.0020%, interpret +0.0019%; every other program is at most +0.0011% or falls (pingmsg -0.0083%).

## Deviation candidates

**A. Each report of a condition re-raised through `Message~result` names the line it was raised at.**
* Rule: rexxref says `result` "also raises an error condition" when the message raised one
  (`fundclasses.xml:1661`). The started activity's own report is of its own raise; the waiter's is of
  the raise in its own clause.
* Measurement: `t8.4/naming/probes/r1.rex` (`1/0` in a started method, `~result` on line 3). Oracle:
  two reports interleaved line by line, both `Error 42 running ... line 3` in 20 of 20 runs
  (`r1/candidate-a-oracle-20.txt`), while the review saw 1 of 5 give line 8 then line 3; it is
  Deviation 67's race.
  Crate: the target's report names line 8, then main's names line 3. `r2.rex` (HALT, the `hs9` shape)
  is the same: oracle `line 2` twice, crate `line 9` then `line 2`. In the C++,
  `MessageDispatcher::handleError` posts the condition to the message (`message->error(c)`) before the
  target activity displays it, and the waiter's `reraiseException` updates the same condition object's
  position first.
* Witnessed by: no corpus witness, the crate is unchanged; the probes and their three-way outputs are
  `t8.4/naming/result-reraise-r1.txt` and `-r2.txt`.

## New oracle defects

1. **A `::CONSTANT` expression method's `~source` answers another directive's clause.** Repro
   `t8.4/constant/probes/f8.rex`: three `::CONSTANT` expressions in one class, each answering
   `.context~executable`; `~source` of each is `[::constant g (.context~executable)]`, the third's.
   Expected: each its own clause (this crate answers that). Not read in the C++ beyond the observable;
   the constant's code location looks shared.

## Concerns

1. **Residual frames under a `::CONSTANT` level.** For a main program's `::REQUIRES` (`f1.rex`) the
   oracle lists the main `PROGRAM` level below the required package's `ROUTINE` level; the main
   program has no activation while its own directives install here, so it is missing. For
   `.Package~new`/`.Routine~new` (`f3.rex`, `f4.rex`) the oracle lists `METHOD NEW` between the levels:
   that is `2026-10-22-live-frames-omit-nested-primitives.md` (Task 8.7). Records:
   `t8.4/constant/residual-*.txt`. Proposed owner for the first: a queued item.
2. **The `::CONSTANT` method object cannot be run.** Its record has no installed `MethodId`, so
   sending it to `run` or `setMethod` takes the paths for an unrecorded method. Not probed against the
   oracle. The level also takes no guard on the class, where the oracle's guarded method would.
3. **A compile failure's condition object names the caller's package.** `condition('o')~program` and
   `~package` after a parse or translation error in a called file name the calling program here; the
   oracle names the file being translated and hands out a package object for it
   (`t8.4/naming/probes/compile-condition/m.rex`, run from a directory holding its two callees). Proposed
   owner: a queued item; it needs a package object for a source that did not translate.
4. **Known flakes seen**: `command_redirected_target_waits_elsewhere.rex` differed once in the corpus
   gate (the crate side) and once in collect_stress, both on its `waited` timing (queued
   `2026-10-26-redirected-target-waits-timing-flake.md`); each rerun passed.
5. **`install-error-names-caller-file.md` stays open** for candidate A only.

## Closed

`**CLOSED 2026-10-26 by Phase 9d Task 8.4 (<sha>).**` added to
`2026-10-17-scanner-reports-later-clause-error.md` (`44f1cb56b`),
`2026-10-18-duplicate-member-checked-at-install.md` and `2026-10-17-library-bind-before-options.md`
(`f84e64ee4`), `2026-10-18-options-trace-constant-expression.md` and
`2026-10-22-constant-frame-and-executable.md` (`7fe50bedd`); their inventory rows read `closed:<sha>`.
Gate lines: `gate:phase-9c-gate.md:142` closed by `f84e64ee4`, `gate:phase-9c-gate.md:145` by
`44f1cb56b` (`t8.4/closed.tsv`, and their inventory rows).

## Fix round 1

Brief `task-8.4-fix-r1.md`, review `task-8.4-review.md`. Code `402a6c753`, `2ea743b36`; records
`t8.4/r1/`.

* **I1.** A level that runs no body carries the directive it is stopped on
  (`ActivationCold::directive_clause`): the `::CONSTANT` level its own directive, the level installing
  the package the `::CLASS` installed last. Its clause index names no instruction (`usize::MAX`, set by
  `enter_directive_clause`), and only then does `trace_line_text` look the directive up
  (`directive_clause_text`, cold): the first form, reading it on every frame, cost pingmsg +0.073%
  libc subtracted. Live frames and a trapped condition's `~stackframes` now echo the `::CONSTANT` and
  the `::CLASS` (the review's `k4`, `tb1`; `tl1` differs only by the `METHOD NEW` level, Task 8.7's
  item). Witness `corpus/lang/constant_expression_traceline.rex` (in `phase-9.txt`, with its
  `sourceline_oracle` file): differs at `f0164456d`, identical on all three descriptors at head
  (`r1/three-way-constant_expression_traceline.txt`).
* **M1.** `parse-errors.tsv`: an `install` row is accepted, or rejected for a later error having
  recorded the directive the oracle names as one whose `EXTERNAL` it reached first; the sentence on
  the decision being unchanged is deleted. `derive.sh` re-derives the file byte for byte (rc 0,
  `diff` empty).
* **M2.** `directive_clause_lines`'s comment states the ruled answer. Deviation candidate B below,
  witnessed by `tests::a_constant_expressions_method_answers_its_own_clause_as_source` (red at
  `f0164456d`: that build answers the main program's three lines for each, `r1/three-way-src.txt`).
* **M3.** The candidate A sentence above reads the measured race; the debug sentence now names its
  record, `r1/debug-witnesses.txt` (a debug build of the fix round tree over every Task 8.4 witness,
  each identical to the oracle except `translation_error_in_called_file`, whose stderr carries its run
  directory).
* **M4 and the review's queue candidate 2.** rexxref says nothing of the method a `::CONSTANT`
  expression runs as beyond its being a method (`dire.xml:535`); a method that can be listed by
  `.context~executable` can be sent, copied and defined like any other. The object now carries a
  method identity minted for it alone (`GeneratedKind::ConstantExpression`, freed with the object):
  run again it evaluates its own expression on its receiver and answers nothing, so a `define`d copy
  sent for a result is 91.999 and as an instruction leaves `RESULT` unset, as on the oracle; `run` is
  the private-method 97.2 for it as for any object. The level is guarded (it reserves the receiver's
  guard in its class's scope, `reserve_for_method`, and releases it at its end), so the review's
  `g2.rex` deadlock is 98.905 rc 158 as on the oracle. Witnesses:
  `corpus/lang/constant_expression_method_runs.rex` (red at `f0164456d`, the loud "no executable
  record" refusal; identical at head), `tests::a_constant_expression_holds_its_classs_guard` (`g2`'s
  shape; `f0164456d` prints `g ran` and `5`, rc 0, `r1/three-way-g2.txt`; 5 of 5 runs pass) and
  `tests::a_constant_expressions_method_runs_its_own_expression`. Two differences remain, both
  chosen: the oracle evaluates another `::CONSTANT`'s expression when the method is run (`run2.rex`
  prints `note ran` again; part of candidate B's oracle bug), and `g2`'s two deadlock reports name the
  started activity's line and the constant's clauses where the oracle's interleave on the waiter's line
  (Deviation 67's shape).
* **Deviation 67, HALT.** `run::tests::conditions::a_started_messages_halt_report_names_the_line_that_raised`,
  the `hs9`/`r2.rex` shape: the started report names line 9, the waiter's line 2. It passes at
  `f0164456d` as well: the behaviour predates the ruling, and the test pins it.

**Checks**, from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t84-target`, at
`2ea743b36`:

| command | result |
|---|---|
| `cargo fmt --all --check` (`r1/checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-parse -p rexx-exec --all-targets -- -D warnings` (`r1/checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test collect_stress --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test compiled --test rexxc` (`r1/checks-touched.txt`) | 0: lib 1182, collect_stress 74, every other binary passed |
| `memcap 16G cargo test -j 4 --release -p rexx-parse` (`r1/checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites` (`r1/checks-unsafe.txt`) | 0 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`r1/checks-corpus.txt`) | 101: 1193 of 1194, the one mismatch `rexxutil_file_date_time.rex` |
| whole suite, `REXX_SUITE_TABLE=.../t8.4/r1/suite/table.tsv` (`r1/suite/`) | 0; the four tables identical to `t8.4/suite/` |

**Perf** (`r1/callgrind.txt`): `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.4/r1final base=.../t8.4/rexx-base-f0164456d
head=.../t8.4/rexx-head-2ea743b36`, every program, 3 rounds, exit 0, no flag. Libc subtracted, against
`f0164456d`: startup +0.0263%, pingmsg +0.0252%, sayloop +0.0140%, churn +0.0044%, define +0.0038%;
every other program is at most +0.0025% or falls.

### Deviation candidate B

**A `::CONSTANT` expression's method reports and runs its own clause** (Moritz 2026-10-26: "Own
clause, oracle bug").
* Rule: a `::CONSTANT` directive "defines a method" (rexxref `dire.xml:535`-`:536`); the method a
  parenthesised value is evaluated by is that directive's, so its `~source` is the directive's own
  clause and running it evaluates the directive's own expression.
* Measurement: `t8.4/constant/probes/f8.rex` and `r1/probes/src.rex`: three expressions in one class,
  each answering `.context~executable`; the oracle answers the third's clause as `~source` of each, this
  crate each its own (`r1/three-way-src.txt`). `r1/probes/run2.rex`: the method of `::constant e
  (.context~executable)`, `define`d and sent, evaluates every `::CONSTANT` expression of the class in
  order on the oracle (`r2/probes/run3.rex`: constants `a`, `e`, `z`, `e` the one run, print `note a`,
  `keep`, `note z` at install and again for the run, `r2/three-way-run3.txt`) and its own expression
  here (`keep` alone).
* Witnessed by `tests::a_constant_expressions_method_answers_its_own_clause_as_source` and
  `tests::a_constant_expressions_method_runs_its_own_expression`.

### Concerns (fix round 1)

1. **The started report under `g2`'s deadlock** names its own line and the constant's clauses, where
   the oracle's two reports both name the waiter's line 6; the same shape as Deviation 67, not
   separately ruled.
2. **A run of the constant's method takes the guard on its receiver in the method's scope**, as any
   guarded method does; not probed against the oracle beyond the install-time deadlock.

## Fix round 2

Brief `task-8.4-fix-r2.md`, re-review `task-8.4-rereview-r1.md`. Code `d2e226538`; records
`t8.4/r2/` (the re-review's probes rebuilt under `r2/probes/`, each run three ways in
`r2/three-way-*.txt`).

* **C1.** A run's `SUPER` is `super_scope_of(receiver, scope)`, the receiver's behaviour's super
  scope as `Interp::begin_method` computes it, which answers a `.nil` scope too; the install-time level
  answers what it did (`class_super_scope(class, class)` for a class receiver). `p1` (`self~run`, as
  an instruction), `p2` (`say self~run(...)`, 91.999 `Message "RUN"`), `p3` (`.object~enhanced`) and
  `g14` (`setMethod` in a class method) are identical to the oracle on all three descriptors; each
  panicked at `b3875ce42` (`r2/red-at-b3875ce42.txt`). Witness
  `corpus/lang/constant_expression_method_scopes.rex`, each class holding one `::CONSTANT` so the
  oracle's re-run of every expression of the class (candidate B) is out of the picture; its one other
  difference on the way there was the `METHOD RUN` level a nested primitive leaves out (Task 8.7's
  item), avoided by keeping the frame listing off the `run` cases.
* **I1.** After `reserve_for_method` the level serves its own wait (`serve_guard_wait`), since it
  runs no clause where a queued wait would be: a run waits for a guard another activity holds. `g4` and
  `g11`, 5 runs each side, identical every time (`r2/guard-wait-runs.txt`). Witness
  `tests::a_constant_expressions_method_waits_for_a_held_guard` (`g11`'s shape; 5 of 5 runs pass;
  `b3875ce42` printed `note ran` before `go still holds`). A deadlock in that wait blames the
  directive (`blame_first_clause` reads `ActivationCold::directive_clause`), where it named the main
  program's first clause: `g15.rex`.
* **I2.** `ConstantRun::name`: `::CONSTANT` while the package installs, the message name on a run.
  `tb3` (`METHOD ZZ 6`) and `m4` (`>I> Method "ZZ" with scope "C"`) identical. Witness
  `corpus/lang/constant_expression_method_trace.rex` (`m4`'s shape; differs at `b3875ce42`).
* **Positions.** rexxref: POSITION is "the line number in source code at which a condition was
  raised" (`condtra.xml:769`-`:771`), and a `::CONSTANT` defines a method (`dire.xml:535`), so a
  condition raised below its level keeps the line it was raised at, and only the expression's own
  failure is blamed on the directive and its `::CLASS`. The failure path now leaves a condition raised
  again by a `result` the expression waited on (`FailureSite::Reraised`) as it stands: no site is
  recorded for the directive or the class. Measured, 10 runs per side (`r2/race-runs.txt`): `g2` oracle
  `line 6 line 6`, crate `line 8 line 6`; `g9` oracle `line 7 line 7`, crate `line 9 line 7`; `g10`
  (no constant) oracle `line 6 line 6`, crate `line 8 line 6`, all ten identical per side. Each
  residual is the started activity's report naming its own line, Deviation 67's crate answer; the
  waiter's report now names the oracle's line with the oracle's echoes. The review's `k2`, `k3`,
  `t_err` (errors in the expression and in a method it calls) stay identical on all three descriptors.
  Witness `tests::a_condition_raised_again_in_a_constant_expression_keeps_its_position` (`g9`'s
  shape; `b3875ce42` named line 3 with the directive and class echoes).
* **Candidate B's measurement** above now reads the oracle's re-run of every expression of the class
  (`run3.rex`).

**Checks**, from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t84-target`, at
`d2e226538`:

| command | result |
|---|---|
| `cargo fmt --all --check` (`r2/checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-parse -p rexx-exec --all-targets -- -D warnings` (`r2/checks-clippy.txt`) | 0 |
| touched binaries as in round 1 (`r2/checks-touched.txt`) | 0: lib 1184, collect_stress 74, every other binary passed |
| `memcap 16G cargo test -j 4 --release -p rexx-parse` (`r2/checks-parse.txt`) | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` (`r2/checks-corpus.txt`) | 101: 1195 of 1196, the one mismatch `rexxutil_file_date_time.rex` |
| whole suite (`r2/suite/`) | 0; the four tables identical to `t8.4/r1/suite/` |

A debug build of `d2e226538` ran every Task 8.4 witness (`r2/debug-witnesses.txt`): each identical
to the oracle except `translation_error_in_called_file`, whose stderr carries its run directory.

**Perf** (`r2/callgrind.txt`): `callgrind.sh -B 0.5 -j 4`, every program, `rexx-head-d2e226538`
against `rexx-base-f0164456d`, exit 0, no flag. Libc subtracted: startup +0.0269%, pingmsg +0.0259%,
sayloop +0.0143%, churn +0.0045%; every other program at most +0.0026% or falls.

### Concerns (fix round 2)

1. **A deadlock in a run's guard wait** (`g15.rex`) now blames the directive, but the reports
   differ from the oracle beyond Deviation 67: the oracle prints two reports, both echoing the
   directive and naming the waiter's line 10; here the started report names line 6 and the waiter's
   report echoes nothing. The same deadlock with an ordinary method (`d2.rex`) differs too, and more:
   the oracle reports once (`Compiled method "RESULT"`, `10 *-* m~result`, `2 *-* o~go`), this crate
   twice. Pre-existing in the deadlock report shape, not this task's; proposed owner: a queued item.

