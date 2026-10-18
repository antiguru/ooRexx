# Task 6.1 report: `::REQUIRES` from dynamic code

Status: DONE.

## Commits

* `5b483709d` Phase 9d Task 6.1: a ::REQUIRES in source-built code searches from its own name, then its parents
* `37f18b39f` Phase 9d Task 6.1: inline the one-use closure in resolve_requires (no behaviour change)
* the records commit that carries this report

## Root cause

`Interp::resolve_requires` (`rust/crates/rexx-exec/src/install.rs`) searched from `package_path(id)`,
which for a package without a file (one built by `.Routine~new`, `.Method~new` or
`.Package~new(name, source)`) is the main program's path. The oracle's
`PackageClass::resolveProgramName` (`classes/PackageClass.cpp:926`) searches from the directory and
extension of the package's own `programName`, the name the code was given, and on a miss repeats the
search from each `parentPackage` up the chain (`parser/LanguageParser.cpp:603`-`:726` set the parent
before `installPackage`). The fix walks `package_chain` and searches each package from
`program_display_name`. The REXX package searches the global context. `ooTestCase~runDynamicSource`
names the routine after the test group's file and passes that package as the parent, so every test
in the cluster searched from the harness's directory instead of the group's.

## Witnesses

`rust/corpus/lang/requires_dynamic_search_start.rex`, added to `rust/corpus/phase-9.txt`, with its
`crates/rexx-parse/tests/sourceline_oracle/requires_dynamic_search_start.txt` from the oracle driver.
The witness has four cases: the name's own directory, the name's own extension (`.zzz`), a bare name
that falls through to the parent package's directory, and a miss (43.901 on both sides). Records are in
`docs/superpowers/records/2026-10-18-phase-9d/t6.1/probes/`.

* Red (binary at `a38297da2`): rc 213, `Error 43.901: Could not find file "rqd_own" for ::REQUIRES.`
  `only_ext` and `only_parent` show the extension case and the parent case red on their own (rc 213,
  43.901 for `rqd_ext` and `rqd_par`).
* Green (binary at `5b483709d`): stdout, stderr and rc byte-identical to the oracle
  (`own`, `ext`, `parent`, `syntax 43.901 ...`, rc 0). The same holds for `only_ext` and `only_parent`.
* Mutations:
  * Walking only the first package of the chain (`.take(1)`) prints `own`, `ext`, then rc 213 on
    `rqd_par`.
  * Searching from `package_path` instead of the given name gives rc 213 on `rqd_own`. Under this
    mutant `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast` exits
    101 with `1067 of 1068 matching`, and the witness is the only program that differs.

  Each mutant was reversed by an exact edit and checked with `git diff --stat`.

## The cluster's tests

These 25 tests pass now. All of them failed at the 9c close with SYNTAX 43.901.

* `base/class/Package`: `TEST_NEW01`.
* `base/directives/CLASS`: `TEST_INHERIT`, `TEST_METACLASS`, `TEST_MIXINCLASS`, `TEST_SUBCLASS`.
* `base/directives/REQUIRES`: `TEST_ACTIVATE`, `TEST_BASIC_GOOD_REQUIRES`, `TEST_CASCADED_REQUIRES`,
  `TEST_CASCADED_REQUIRES2`, `TEST_CASCADED_REQUIRES3`, `TEST_MULTIPLE_NAMESPACE`,
  `TEST_MULTIPLE_NAMESPACE2`, `TEST_NAMESPACE_GOOD_REQUIRES`, `TEST_NAMESPACE_NO_PUBLIC_CALL`,
  `TEST_NAMESPACE_NO_PUBLIC_CLASS`, `TEST_NAMESPACE_NO_PUBLIC_ROUTINE`, `TEST_NO_PUBLIC_CLASS`,
  `TEST_NO_PUBLIC_ROUTINE`, `TEST_PROLOG`, `TEST_PROLOG2`, `TEST_PROLOG3`, `TEST_SEARCH_ORDER_CLS`,
  `TEST_SEARCH_ORDER_NONE`, `TEST_SEARCH_ORDER_REX`, `TEST_SEARCH_ORDER_TESTGROUP`.

Their lines are removed from `rust/corpus/suite-9d-set.txt`. This instance closes no queued item and
no exclusion row.

## Whole-suite changed rows

The suite ran at `5b483709d`. The table is in `docs/superpowers/records/2026-10-18-phase-9d/t6.1/suite/`.

* Against `task-4/suite/tests.tsv` (`changed-vs-task-4.tsv`), exactly the 25 cluster tests changed,
  each from `crate failed` to `crate passed`. The set equals the cluster's rows of `clusters.tsv`,
  checked with `diff`.
* Against `2026-10-15-phase-9c/close/suite/tests.tsv` (`changed-vs-9c-close.tsv`), the same 25 rows
  changed, plus the json, yaml and dateparser rows. Those rows went from `Phase10(Extension)` to `-`
  with the crate passing, and they already show in Task 4's table. They include the two yaml tests
  that went from blocked to passed in Task 4.

## Perf

The table is `docs/superpowers/records/2026-10-18-phase-9d/t6.1/callgrind.txt`, measured at `37f18b39f`.
Its raw outputs are in `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.1/`. The previous perf-checked
task is Task 4 (`task-4/callgrind.txt`).

| program | vs pin | Task 4 vs pin | own delta |
|---|---|---|---|
| startup | +0.0198 | +0.0188 | +0.0010 |
| interpret | -2.1904 | -2.1922 | +0.0018 |
| parse | -0.0874 | -0.0876 | +0.0002 |
| churn | +0.1276 | +0.1526 | -0.0250 |
| sayloop | +0.0346 | +0.0336 | +0.0010 |

Each own delta is at most +0.05% and each figure against the pin is under +0.5%.

## Checks

All runs used `CARGO_TARGET_DIR=<scratchpad>/target-9d-t6`, `CARGO_INCREMENTAL=0`, `memcap 8G` and
`-j 4`. Exit statuses were read unpiped.

| command (from `rust/`) | exit |
|---|---|
| `cargo fmt --all --check` (at `5b483709d` and `37f18b39f`) | 0 |
| `cargo clippy -p rexx-exec -p rexx-parse --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib require` (16 passed) | 0 |
| `cargo test --release -p rexx-exec --test package_requires --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (`1068 of 1068 matching`) | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.1/suite/table.tsv cargo test --release -p rexx-exec --test suite l3_core_suite` (at `5b483709d`) | 101 (expected while the phase runs) |
| `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn sayloop" -o .../cg-9d/t6.1 pin=... head=...` | 0 |

Clippy, fmt and the corpus gate ran at `5b483709d`. `37f18b39f` only inlines a closure, and fmt and the
perf run were taken there.

## Concerns

* **The same rule fails in two more paths, and this task does not widen to them.** A `CALL` of an
  external routine from source-built code (`Interp::external_program`, `lib.rs`) and `Package~findProgram`
  on a source-built package (`Interp::resolve_program_name`, `environment/identities.rs`) still search
  from the main program's path, or from the global context, and never walk the parent chain. Probe
  `t6.1/probes/same-rule-other-paths/`: the oracle prints `findProgram 1` and `call external`, while
  the crate at `37f18b39f` prints `findProgram 0` and `syntax 43.1`. The oracle reaches both through
  `PackageClass::resolveProgramName` and `findProgramRexx` with the same chain. The fix would reuse
  `resolve_requires`' walk with `requires = false`. The `resolve_program_name` doc comment ("A package
  compiled from source text has no directory") holds only for a bare name.
