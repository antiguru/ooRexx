# Phase 5d Task 4 — `::REQUIRES` of a program file

**BASE `310677e47`.** Tree held alone. Plan `docs/superpowers/plans/2026-09-03-phase-5d.md`
(Task 4), spec `docs/superpowers/specs/2026-09-03-phase-5d-foundation.md` (§3, D77).

**Committed before the gates**, per the plan's 2026-09-04 rule. The gate section below carries
placeholders and nothing else; the statuses are read from the background run's own file.

---

## The mechanism

**`::REQUIRES 'name'` resolves a file, parses it, installs its directives, runs its leading code
section once, and merges its public routines and classes into the requiring package.** Five parts,
each with its own site:

* `crates/rexx-exec/src/require.rs` (new) — **the search alone**, and nothing that touches the file
  system. `search_entries` builds the path entry list and `candidates` builds every name that
  entry list produces, in order. That split is what lets the route order and the extension order be
  asserted without a directory to put files in.
* `Interp::resolve_requires` (`lib.rs`) — the environment the search runs in and the test each
  candidate is put to: `require::normalize` against the current directory, then `stat` plus a
  regular-file check, which is `SysFileSystem::checkCurrentFile`.
* `Interp::load_requires` — the two-key cache, the circularity check, the parse, and the prologue.
* `Interp::load_required_packages` — the source-order walk, the installing-package marker, and the
  clause echo each level of a failing chain contributes.
* `Interp::merge_required` — `publicRoutines` then `mergedPublicRoutines`, first write wins.

**`Interp::routines` became per-program in the same change, and it had to.** It was one flat table
keyed by the upcased name, which was correct only while exactly one program ever installed a
routine. A required file's non-`PUBLIC` `::ROUTINE` would have entered it and answered a call in
the requiring program — measured, the oracle answers `Could not find routine "PRIVR".` there. The
table is now `HashMap<ProgramId, HashMap<name, InstalledRoutine>>`, beside
`package_public_routines` and `merged_public_routines`, which is the shape `package_classes` and
`package_public_classes` already had.

### The four routes, measured one at a time

`SysSearchPath` (`platform/unix/SysInterpreterInstance.cpp:123`) is the whole order, and it is not
the order the plan's sentence gives: **the requiring program's own directory comes first and the
current directory second.** Measured against the oracle, each route holding the file and the other
three empty:

| the file is in | oracle answers |
|---|---|
| the requiring program's directory | that copy |
| the current directory | that copy |
| a `REXX_PATH` entry | that copy |
| a `PATH` entry | that copy |
| all four | **the program's own directory** |
| none | `43.901` rc 213, `Could not find file "reqlib" for ::REQUIRES.` |

Also measured, and each a pair the oracle settles: the program's directory beats the current one;
`REXX_PATH` beats `PATH`; and an unset `REXX_PATH` removes its entries without shifting anything
else.

`crates/rexx-exec/tests/package_requires.rs`'s
`each_of_the_four_search_routes_finds_the_required_file` and
`the_earliest_route_wins_when_every_one_of_them_holds_a_copy` run all six rows against the oracle,
comparing three descriptors and then asserting **which** copy answered — a row that only compared
the two sides would pass on a search narrowed to one route, since no row but the last has two
candidates.

### The extensions, which are a second dimension and not part of the routes

`resolveProgramName` under `RESOLVE_REQUIRES` (`runtime/InterpreterInstance.cpp:1167`) tries
`.cls`, then the requiring program's own extension, then `.REX`, `.rex`, then the bare name — and
**each of those is a whole pass over the path**. Measured, oracle:

```text
lib, lib.cls, lib.rex, lib.REX all beside main.rex, ::requires 'lib'   ->  lib.cls
the same set with lib.cls absent, requiring 'lib' from main.rex  -> lib.rex   (the parent's)
                                                  from main.REX  -> lib.REX
                                                  from mainnoext -> lib.REX   (the first default)
lib.cls reachable only through PATH vs lib.rex in the program's own directory  ->  the .cls
::requires 'lib.rex' with only a lib.rex.cls present   ->  43.901 rc 213
::requires 'LIB' and ::requires 'LIB.REX' with only lib.rex present    ->  lib.rex
a sub/lib.rex under both the program's directory and the current one:
    ::requires 'sub/lib.rex'    -> the program's own
    ::requires './sub/lib.rex'  -> the current directory's
```

The last two are `SysFileSystem::hasDirectory`, which is true for `~`, `/`, `./` and `../` and for
nothing else — so `sub/lib.rex` is a path search and `./sub/lib.rex` is not.

### The prologue, the cache and the cycle

* **The prologue runs once however many times the file is required**, directly or through another
  required file. Measured, oracle rc 0: a `lib.rex` required twice by the program and once more by
  a `mid.rex` it also requires prints its first line once, above the requiring program's own output.
* **The cache is keyed by the name as written and by the file it resolved to, and the written name
  is asked first** (`InterpreterInstance::addRequiresFile`, `:1000`). That is a real observable and
  a surprising one: measured, oracle rc 0, two files in different directories each carrying
  `::requires 'lib.rex'` with a `lib.rex` of their own beside them, and the second gets the
  **first** file's package. This crate reproduces it.
* **The circularity check fires on a cache hit alone** (`InterpreterInstance::loadRequires`,
  `:1024`), which is why a file requiring itself loads a second copy before it is refused: measured,
  the oracle echoes that one `::REQUIRES` clause **twice** and reports `98.952` against the file's
  own path. A pair of mutually requiring files echoes three clauses, innermost first, and names the
  inner file in the `running <name> line <n>` span.

### What a required file's own frames report

Four quantities change for a program a `::REQUIRES` loaded, and all four were measured before they
were built:

```text
parse source              LINUX REQUIRES <the required file>   (COMMAND for the program itself)
.context~package~name     <the required file>
a traceback frame         Error 42 running <the required file> line 1
the 98.952 substitution   the resolved path, not the name as written
```

`CallType::Requires` is the first; `Interp::package_path` and `Interp::required_paths` are the rest,
read by `program_display_name`, `environment::package_name`, `Interp::blame_directive_in` and
`Interp::required_package_site`.

---

## The divergence this task created and closed in the same change

**`::OPTIONS NOPROLOG` had no reader**, and `options.rs`'s own comment said so and said the
`::REQUIRES` task owed it. Without it, building the loader would have made the crate run a leading
code section the oracle does not — rc 0 on both sides, stdout differing by one line, which is the
worst defect class this project recognises. Measured at HEAD-minus-the-fix:

```text
::options noprolog in a required file, whose first clause says 'lib prologue RAN'
  oracle   main / lib / wm
  crate    lib prologue RAN / main / lib / wm
```

`PackageClass::runProlog` (`classes/PackageClass.cpp:2131`) installs and stops where
`isPrologEnabled()` is false, and the top-level program never goes through `runProlog` at all — so
the keyword selects what a `::REQUIRES` of a file does and nothing about running that file. Both
halves are measured and both are asserted, in
`noprolog_suppresses_a_required_files_prologue_and_not_a_programs_own`: the same file prints its
first clause as the program and does not as a requirement, its public routine reachable either way.

---

## `::REQUIRES LIBRARY` and `::REQUIRES NAMESPACE` refuse exactly as before

Both stay in `directive_gap` and both are still raised by `staged_gap` **ahead of any file
search**, which is what leaves them failing the way they did at BASE: rc 120, stdout empty, the
program refused before its first clause. Only the message text is narrower, since
`::REQUIRES is not implemented` is no longer true of the directive as a whole:

```text
::requires zzznolib library              rexx-exec: ::REQUIRES LIBRARY is not implemented (Phase 7)
::requires 'zzznofile.rex' namespace ns  rexx-exec: ::REQUIRES NAMESPACE is not implemented (Phase 5)
```

**The oracle resolves the file before it looks at `NAMESPACE`** (`RequiresDirective::install`,
`instructions/RequiresDirective.cpp:130`), so its answer for that probe is `43.901` at rc 213 —
which this crate could now give by moving the namespace refusal behind the search. It is
deliberately not moved: table D's `requires__namespace__subkeyword` is Task 5's one open `5d` row,
and making it `agree` on a probe whose file does not exist would close the phase gate over work
Task 5 still owes.

---

## The witness

`rust/corpus/lang/package_requires.rex`, run by
**`rust/crates/rexx-exec/tests/package_requires.rs`** — the interim test binary Task 6 folds into
`corpus/phase-5d.txt` and deletes. `corpus/unfiled.txt` names the program.
**`corpus/phase-5d.txt` was not created.**

**It is the first corpus program needing a second file beside it, and the two are `.cls`.**

**The choice is not a way round Task 1's check, and the reason it could have been one is worth
recording.** A file whose first clause is a directive has no main body, so it runs as a program and
answers nothing -- measured, the brief's own `lib.rex` is rc 0 with both descriptors empty on the
oracle and on both engines. So the required file could have been an ordinary `lang/*.rex` corpus
program, filed like any other. (`corpus/lang/gate_variants.rex` is this corpus's precedent for a
program that prints nothing, but it is *not* a witness of this shape: it opens with a main body and
reaches silence by guarding every effect behind `if 0 then`.) Measured further, on the two files as
they stand: run standalone they are rc 0 printing `dep prologue` and three lines respectively, both
engines agreeing with the oracle, so the `.rex` shape was available and would have cost nothing.

**`.cls` is chosen for what it is, not for what it avoids.** These files are class libraries and
`.cls` is what ooRexx calls one -- the reference's own example is `::requires "rxregexp.cls"` -- and
naming them without an extension in the directive puts the `.cls` step, the one thing that makes a
`::REQUIRES` search differ from every other program lookup, inside the byte-for-byte differential as
well as inside the synthetic tree `each_of_the_four_search_routes_finds_the_required_file` builds.
**Both reasons in the first version of this paragraph were weaker than they were written, and the
second was the worse of the two.** The scanner reason was the leading one and should not have been a
reason at all, since the `.rex` shape was available. The extension reason then looked strong because
nobody asked what already covered that step -- and
`each_of_the_four_search_routes_finds_the_required_file` plants a `reqlib.cls` and requires
`'reqlib'`, so it was covered against the oracle before this witness existed. The correction is not
that the two reasons were in the wrong order. What the `.rex` shape would have bought instead is a
standalone differential run of each helper, which is real and small -- the witness already drives
both files through both interpreters.

The consequence for the scanners is the same either way: every corpus scan selects `*.rex`, so
`corpus/lang/package_requires_lib.cls` and `corpus/lang/package_requires_dep.cls` are outside all of
them, and the one `.rex` this task adds is named in `corpus/unfiled.txt` with its reason. Task 1's
check is exactly as strong as it was. `corpus/README.md` gains the convention.

**Task 6's flip needs nothing extra.** The differential runs the oracle side from the program's own
directory and the crate side in the test process's, and the required names carry no directory, so
the program-directory route is what finds the files on both sides -- which is exactly what the
interim binary already exercises, since it runs the two sides the same two ways.

The program covers: a public routine and a public class of the required file; a public routine and
a public class reached **transitively**, through a `::REQUIRES` in that file; a non-`PUBLIC`
`::ROUTINE` that must not be imported and a non-`PUBLIC` `::CLASS` that must render as its own
name; the requiring file's own `::ROUTINE` and `::CLASS` beating imported ones of the same name;
a name two required files both export, answered by the one the **first** `::REQUIRES` reached;
a prologue that must run exactly once for a file required three times over; and `PARSE SOURCE`'s
second word in the requiring program.

**Shown to fail at BASE.** In a `git archive` extract of `310677e47` at
`/home/moritz/dev/repos/claude-build-scratch/task-4/base/` with `CARGO_TARGET_DIR` at
`.../task-4/base-target` (sha256 `4444aa4405733e43943550116a42e0880a13b4e0de41d64d54bde26bdbe5df0f`,
distinct from the working tree's own release build, `5aa1c3ddc58e4dc824cd9b11bda0e526f55e5796a0339052a5bb2bf55406aa0f`):

```text
BASE tree-walker  rc 120  stdout empty
BASE ir           rc 120  stdout empty
                  stderr: rexx-exec: ::REQUIRES is not implemented (Phase 5)
oracle            rc 0    15 lines of stdout, empty stderr
```

---

## Controls

* **The witness fails at BASE**, on both engines, in a separate extract with its own target
  directory. Above.
* **One control per route, each run and reverted.** Removing a single entry from
  `require::search_entries` reddens exactly the row that plants its file there and nothing else:

  | removed | reddened |
  |---|---|
  | the program's own directory | `route progdir`, and `the_earliest_route_wins…` |
  | the current directory | `route cwd` |
  | `REXX_PATH` | `route rexxpath` |
  | `PATH` | `route syspath` |

  `require.rs` was restored from a copy taken before the first control and `cmp` is clean.
* **The prologue-once claim is asserted on the count**, not left to the byte comparison: a crate
  running it three times against an oracle running it once differs on every line below it too, and
  the count says which property failed.
* **The `NOPROLOG` pair**: one row would pass under a crate that ignored the keyword and the other
  under one that applied it everywhere.

---

## The method-body table

Refreshed with `REXX_METHOD_BODIES_REFRESH=1`. **`corpus/method-bodies.txt` is byte-identical**:
`regressions this run: 0. other drift from the committed table: 0.`

| verdict | BASE | HEAD |
|---|---|---|
| `loud` | 662 | 662 |
| `answers` | 676 | 676 |
| `diverge` | 7 | 7 |
| `unstable` | 2 | 2 |

**No `Package` row moved, and that is right rather than disappointing.** `addPackage`,
`findRoutine`, `importedRoutines`, `loadPackage` and `local` are *methods* on the `Package` class;
this task built the `::REQUIRES` *directive*. `Package~local` stays `loud` and stays Task 5's.

The six `DateTime` rows the spec's handover attributes to the Phase 4 `DATE()`/`TIME()` defect are
still `diverge`. I did not touch `DATE()` or `TIME()`.

---

## D77: the 110 open rows did not move

From the phase gate's own report at the committed tree:

```text
5a: 135 rows, 0 not yet agree      6:  13 rows, 13 not yet agree
5b:   6 rows, 0 not yet agree      7:  94 rows, 82 not yet agree
5c: 1225 rows, 0 not yet agree     deferred-rexxcontext-stackframes: 10 rows, 10 not yet agree
                                   never-expected-to-agree: 5 rows, 5 not yet agree
gated by this run: 0 row(s)
```

`13 + 82 + 10 + 5 = 110`, identical to Task 2's and Task 3's readings. No `class-set.txt` row was
edited and `corpus/docs/` is unmodified. Table D is unchanged: `5d: 1 rows, 1 not yet agree`, which
is Task 5's `requires__namespace__subkeyword`, so `REXX_PHASE_GATE=5d` is rc 101 on exactly that
row.

---

## Committed text this change falsified, corrected rather than left

* **`environment.rs`'s module doc** listed imported public classes among "the steps this crate has
  nothing to consult". `Interp::imported_class` is that step now, and `Interp::directive_class`
  consults it too — under the *installing* package's id, because a `::CLASS` resolves before any
  activation of its package exists. Measured, oracle rc 0: `::class K subclass Base` over an
  imported `Base` resolves, and an imported `::class Array public` makes `say .Array` print
  `The ARRAY class`.
* **`error.rs`'s `package_scope_method`** said 97.3 was "not measured on the oracle as a whole
  program, because a caller in another package needs `::REQUIRES`". It has a program now, measured
  on both engines at rc 159 and byte-identical: a `::class K public` carrying `::method m package`
  in a required file, and `o~m` in the requiring one.
  `corpus/refusal-sites.tsv`'s row moves `diverges`/`no` to `agrees`/`yes`/`97.3`.
* **`dispatch.rs`'s `package_scope_refuses_a_caller_from_another_package`** said no program in this
  phase could reach that arm. One can.
* **`ir.rs`'s `CALL_SITE_CACHE`** rested on `Interp::routines` being written once, and said so
  while naming `::REQUIRES` as what would end that. What holds now is stated instead: both routine
  tables a site reads are final before any of that package's code runs — the local one from
  `install_directives`' first walk, the imported one from `load_required_packages` in the same
  call, which is ahead of the class pass, every `::CONSTANT` expression and the first clause.
* **`options.rs`'s `Prolog` arm** said nothing read it and that `::REQUIRES` owed the field. It is
  read.
* **`lib.rs`'s `staged_gap` doc** carried `::class a / ::constant kk (1/0) / ::requires
  'helper.rex', present` as a row this crate loses. Re-measured: both sides reach the divide at
  42.3 rc 214.
* **`run/tests.rs`'s two directive tests.**
  `every_directive_this_crate_cannot_install_refuses_before_the_first_clause` now carries the two
  subkeyword forms in place of the plain one, and
  `a_gap_the_oracle_diagnoses_before_a_class_refuses_ahead_of_the_class_error` asserts the oracle's
  own 43.901 for the `::REQUIRES` pair rather than a refusal — in both orders against a failing
  `::CLASS`, which is the ordering property that test exists for.
* **`coverage.rs`'s `is_admitted_directive_kind`** refused a `::REQUIRES` outright, so a corpus
  program carrying one panics its walker. That is a landmine for Task 6, which files this witness
  into a subset file; the kind is admitted now and the negative control moved to `::RESOURCE`,
  which is still refused.
* **`docs/superpowers/plans/phase-4-exclusions.txt`**'s "::REQUIRES IS REFUSED EVEN WHEN THE FILE
  IS PRESENT" carries a CLOSED marker. Its reasoning stands as written, because the reasoning is
  what made it an over-refusal rather than an omission.

---

## Performance

**Read, and it is flat.** Two builds with their own `CARGO_TARGET_DIR`, `base` the `git archive`
extract of `310677e47` (sha256 `4444aa44…`) and `head` a copy of the working tree's own release
build at `claude-build-scratch/task-4/head-bin/rexx-run`
(sha256 `5aa1c3dd…`) that no rebuild can reach — Task 3 voided a sitting by
measuring `target/release/rexx-run` while a two-comment edit rebuilt it. Five rounds, both engine
arms, both problem sizes, committed as task `4` rows in `bench-baselines/phase-5d-arms.tsv`.

| axis | `instructions:u`, worst of its four cells | `cycles:u`, range over the four |
|---|---|---|
| `alloc4c` | -0.018% | -2.93% to +0.56% |
| `arith` | -0.097% | +1.93% to +6.09% |
| `compound` | -0.001% | -0.30% to +0.99% |
| `dispatch` | -0.001% | -0.95% to +0.17% |
| `dispatchclass` | +0.015% | -2.53% to -1.73% |
| `emptyloop` | -0.001% | -0.57% to -0.05% |
| `rexxcps` | -0.017% | -2.08% to -1.06% |
| `strings` | -0.036% | -2.09% to +4.07% |
| `varlookup` | +0.001% | -0.11% to +2.57% |

**No axis moves as much as a tenth of a percent in instructions**, which is two orders of magnitude
under the layout floor Task 3 measured with a do-nothing control (`arith` at +0.99% under a build
that did no operator dispatch at all). **No do-nothing control was built for this task and none
would have been informative**: not one of the nine axes declares a `::ROUTINE`, so none reaches
`Interp::installed_routine`, the one changed function on a hot path — the only effect available to
these programs is code layout, and the figures say it did not arrive.

The cycles column is the noisy instrument here as it was for Task 3 -- `arith` reads +1.9% to +6.1%
in cycles at -0.10% to +0.02% in instructions -- and is not the A/B instrument. The whole column is
in the committed file.

**Two sittings, and the split is on the axis rather than between the builds.** The first six axes
were measured in one run and the last three in a second, because `rexx-arms` resolves a
`bench-rexxcps` path relative to its own process directory and the first run aborted on it. Each
sitting measured `base` and `head` together, which is the comparison; no ratio here crosses the two.

---

## What I did not do

* **`::REQUIRES … NAMESPACE`, `ns:Class` lookups and the `>N>` prefix** are Task 5's and are
  untouched. The namespace refusal deliberately stays ahead of the file search, which is why table
  D's one `5d` row is still red — see the section above for the measurement that says what moving
  it would buy and what it would cost.
* **`::REQUIRES … LIBRARY`** stays Phase 7's, refusing before any search.
* **`Package~local`, `~addPackage`, `~findRoutine`, `~importedRoutines`, `~loadPackage`** are
  `loud` and did not move. They are methods on `Package`; this task built the directive.
* **A required file whose source does not parse is a loud refusal**, not the oracle's own report:
  measured, the oracle answers the syntax error under the requiring `::REQUIRES` clause and naming
  the required file, and this crate answers rc 120 `<path> does not parse here: <error>`. That is
  the same trade `execute`'s own arm takes for a top-level parse failure, and its doc says why the
  report cannot be built — `parse_program` takes the text by value and the `ProgramSource` that
  could answer the line number is dropped inside the parser. Closing it is a `rexx-parse` signature
  change and is not this task's.
* **`::OPTIONS`'s other settings across a package boundary were not swept.** `NOPROLOG` was
  measured because `::REQUIRES` is the only thing that can read it; the rest
  (`DIGITS`, `TRACE`, `NUMERIC INHERIT`, the condition escalations) are per-activation through
  `start_from_package`, which keys on the running activation's own package, and I did not build a
  cross-package probe for each.
* **`phase-4-exclusions.txt`'s `::OPTIONS IS REFUSED FOR THE SAME REASON` entry is stale and I left
  it that way.** 5c implemented `::OPTIONS` and did not mark it; it is not this task's boundary to
  move, and it sits in the same section as the one I did mark.
* **I did not build a do-nothing control for the benchmark**, and the Performance section says why
  no axis could have shown one anything.
* **I did not measure `startup`, `alloc`, `heapshape` or the `bench-control` axes** -- the sitting
  covers the nine axes `phase-5d-arms.tsv` already carries.
* **I did not run `clippy` from a clean target directory.** `rust/CLAUDE.md` asks for that at a
  phase boundary; this is not one.
* **I did not create `corpus/phase-5d.txt`** and did not add a row to any `SUBSET_FILES` list.
* **`REXX_PATH`'s compile-time default (`ORX_REXXPATH`) is not implemented.** This build defines it
  as the empty string (`build/CMakeFiles/*/flags.make`), which `SysSearchPath::addPath` drops, so
  it changes no answer here; a build that set it would need the constant.
* **A leading `~` in a required name is not expanded.** `SysFileSystem::canonicalizeName` calls
  `resolveTilde`; `require::normalize` leaves the byte alone and the candidate then fails to
  `stat`, which is a refusal rather than a wrong answer. Not probed against the oracle.
* **`require::normalize` is lexical and resolves no symlink**, which is what the oracle does; the
  crate's own `rexx-run` uses `fs::canonicalize` for the *program* path, which is one step further.
  Nothing in the corpus or in these tests runs through a symlink, so the difference is unobserved
  rather than known to agree.
* **A `REXX_PATH` or `PATH` whose bytes are not UTF-8 drops the whole variable**, because
  `Interp::resolve_requires` reads it with `std::env::var`, which answers `Err` for such a value.
  That is a narrowed search, which is the failure mode this task most had to avoid — and it is left
  as it is deliberately: the crate is `&str`-pathed throughout (`run_program` takes one, and
  `Interp::program_path` is a `String`), so an entry it cannot spell names a file it could not run
  anyway, and the arm that would drop only the offending entry cannot be tested without either
  mutating the process environment (`unsafe` in this edition) or unix-only byte conversion. Stated
  rather than guarded, since a guard nothing exercises is the weaker of the two.

---

## Files

* `rust/crates/rexx-exec/src/require.rs` (new) — the search: `search_entries`, `candidates`,
  `program_directory`, `program_extension`, `normalize`, and their tests
* `rust/crates/rexx-exec/src/lib.rs` — `load_required_packages`, `load_requires`,
  `check_not_installing`, `merge_required`, `resolve_requires`, `package_path`,
  `blame_directive_in`, `directive_clause`, `FileClasses`, the per-program routine tables, the
  narrowed `directive_gap` arms, `Loud::required_source`, and `run_loaded`'s `CallType`
* `rust/crates/rexx-exec/src/error.rs` — `Raised::requires_file_not_found`,
  `Raised::circular_requires`
* `rust/crates/rexx-exec/src/activation.rs` — `CallType::Requires`
* `rust/crates/rexx-exec/src/environment.rs` — `imported_class`, `directive_class`'s new
  parameter, `package_name`'s per-package answer
* `rust/crates/rexx-exec/src/options.rs` — `PackageOptions::suppress_prolog`
* `rust/crates/rexx-exec/src/run.rs` — `installed_routine`, `required_package_site`
* `rust/crates/rexx-exec/src/ir.rs` — the corrected `CALL_SITE_CACHE` argument
* `rust/crates/rexx-exec/src/dispatch.rs` — the corrected test doc
* `rust/crates/rexx-exec/src/run/tests.rs` — the two directive tests
* `rust/crates/rexx-exec/tests/package_requires.rs` (new, **the interim witness binary Task 6 folds
  in**), `rust/crates/rexx-exec/tests/support/oracle.rs` (`Oracle::run_in`),
  `rust/crates/rexx-exec/tests/coverage.rs`
* `rust/corpus/lang/package_requires.rex`, `rust/corpus/lang/package_requires_lib.cls`,
  `rust/corpus/lang/package_requires_dep.cls` (all new),
  `rust/crates/rexx-parse/tests/sourceline_oracle/package_requires.txt` (new),
  `rust/corpus/unfiled.txt`
* `rust/corpus/refusal-sites.tsv` (re-derived), `rust/corpus/method-bodies.txt` (refreshed, no
  change), `rust/corpus/README.md`, `rust/bench-baselines/phase-5d-arms.tsv` (task `4` rows)
* `docs/superpowers/plans/phase-4-exclusions.txt` — the CLOSED marker
* `docs/superpowers/records/2026-09-03-phase-5d/task-4-report.md` (this file; the briefed
  `.superpowers/sdd/` path is git-ignored)

**Scratch left behind, not deleted:**
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task-4/`
(the oracle and differential probe directories `probe/p001`–`p034` with their `orc.sh`, `crt.sh`
and `diff.sh` drivers, the sourceline regeneration driver under `srcgen/`, the pre-control copy of
`require.rs`, the pre-refresh copy of `method-bodies.txt`, and every gate log) and
`/home/moritz/dev/repos/claude-build-scratch/task-4/` on real disk (`base/`, the `git archive`
extract, and `base-target/`, its build tree).

---

## Gates

Run from `rust/` by a background job writing each status **unpiped** to a file as it goes, with a
pidfile. Started after the commit below; the controller reads the statuses and fills this table in.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| 7 | `REXX_PHASE_GATE=5d …` (same command) | **101** |

**G7 is expected non-zero and is not this task's**, and the run says which row rather than being
taken on the prediction. `gate_table_d.rs:809`, the only failure in either binary:

```text
1 row(s) of gate table D owned by a closing or closed phase do not `agree` with the oracle:
["gate-tables/directives/requires__namespace__subkeyword.rex"].
```

That is Task 5's row, exactly as it was after Tasks 2 and 3, and it is the row Task 5 closes.

Filled by the controller from the run's own status file, which is the protocol's design: the agent
commits with `**G1**`-`**G7**` placeholders and stops, and the gated tree is the committed tree by
construction. Two notes on this run's own instrument, neither affecting the result. The status
line's `(expected 101, Task 5's row)` suffix is written by `run.sh` unconditionally rather than
derived from the failure, so it would have read the same had a *different* row failed -- the row
above is quoted from `g7.out`, not from that suffix. And the docs correction below the gate table
(`corpus/README.md`'s two-shapes section, and the `.cls` paragraph in "The witness") was applied
after `finished`, so it is in this commit but was not in the gated tree; both are prose, and no
gate reads either file.
