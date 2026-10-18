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

None open. The two other paths of the same rule are fixed in fix round 1.

## Fix round 1

The controller ruled that `Interp::external_program` (an external `CALL`) and
`Interp::resolve_program_name` (`Package~findProgram`) belong here, because they follow the same oracle
rule. The probe in `t6.1/probes/same-rule-other-paths/` shows them diverging at `37f18b39f`.

### Commit

* `325afb5b7` Phase 9d Task 6.1 fix round 1: an external CALL and findProgram search from the package chain too

`resolve_requires`' walk is now `Interp::resolve_in_package(id, name, requires)` (`install.rs`).
`external_program` (`lib.rs`) and `resolve_program_name` (`environment/identities.rs`) call it with
`requires = false`. `resolve_program_name` with no package keeps the global search, which
`call_program` (`dispatch/library/surface.rs`) uses. The old doc's `compiled_method_names` test is
gone. Its measured fact, that a package built under a bare name searches the current directory first,
still holds under the new rule and stays in the doc.

### Witness

`rust/corpus/lang/external_call_dynamic_search_start.rex`, added to `phase-9.txt`, with its
sourceline expectation file. Each of the cases (own directory, own extension, bare name through the
parent) does an external `CALL` and a `findProgram`. A miss gives `findProgram` `.nil` and 43.1 on both
sides. Records are in `t6.1/probes/fix-round-1/`.

* Red (binary at `37f18b39f`): rc 213, `Error 43.1: Could not find routine "EXT_OWN".`
  `findprogram_only` prints `0` for each of the three cases, where the oracle prints `1`.
  `call_parent_only` stops with rc 213 on `EXT_PAR`.
* Green (binary at `325afb5b7`): all three runs match the oracle byte for byte on stdout, stderr and rc.
  The first witness, `requires_dynamic_search_start.rex`, also still matches.

### Checks

These ran at `325afb5b7` with the same environment as above. Exit statuses were read unpiped.

| command (from `rust/`) | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec -p rexx-parse --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib require` (16 passed) | 0 |
| the refusal-table and `package_requires` command above | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (`1069 of 1069 matching`) | 0 |
| whole suite, `REXX_SUITE_TABLE=.../t6.1/suite-r1/table.tsv` | 101 (expected) |
| `callgrind.sh ... -o .../cg-9d/t6.1-r1` | 0 |

### Whole suite

No row of `t6.1/suite-r1/tests.tsv` differs from `t6.1/suite/tests.tsv`
(`suite-r1/changed-vs-t6.1.tsv`). So against Task 4 the changed rows are still exactly the 25
cluster tests.

### Perf

The table is `t6.1/callgrind-r1.txt`. The previous perf-checked figure is this task's first round
(`t6.1/callgrind.txt`).

| program | vs pin | round 0 vs pin | own delta | task total vs Task 4 |
|---|---|---|---|---|
| startup | +0.0194 | +0.0198 | -0.0004 | +0.0006 |
| interpret | -2.1904 | -2.1904 | 0.0000 | +0.0018 |
| parse | -0.0874 | -0.0874 | 0.0000 | +0.0002 |
| churn | +0.1275 | +0.1276 | -0.0001 | -0.0251 |
| sayloop | +0.0344 | +0.0346 | -0.0002 | +0.0008 |

## Fix round 2

This round covers the three items of `task-6.1-review.md`.

### Commit

* `ade6fd223` Phase 9d Task 6.1 fix round 2: only a name's first byte is exempt from the directory and extension scans

`require.rs`: `program_directory` and `program_extension`, and through them `has_extension`, now skip
only the first char of the whole name, as `SysFileSystem::extractDirectory`, `extractExtension` and
`hasExtension` do (`platform/unix/SysFileSystem.cpp:224`, `:256`, `:322`). The first char is one
widened byte. The old unit test `a_leading_dot_is_not_an_extension` asserted the per-component rule,
which was wrong, and is replaced.

### Cases and witnesses

1. A dot-led last component after a directory is an extension. In
   `external_call_dynamic_search_start.rex`, a routine named `rqx/.zzz` runs `CALL ext_dot`. The binary
   at `e27a7f843` stops with rc 213 and `43.1 EXT_DOT`, where the oracle prints `dot 1`. The same rule
   applies to a `::REQUIRES` name. In `requires_dynamic_search_start.rex`, `::requires 'rqd/.hid'`
   with only `rqd/.hid.cls` present prints `hidden` at `e27a7f843`, where the oracle prints
   `syntax 43.901`. The review's `prog.rex` now matches the oracle on stdout, stderr and rc 213,
   including `T18 dot-only ext own-zzz`.
2. `/x` has no directory. This is covered by the unit test `the_first_byte_is_never_a_directory_separator`,
   which failed before the change (`left: Some("/") right: None`). The new extension unit test failed too.
   Both pass after the change.
3. The current directory comes before the parent package's directory. In
   `requires_dynamic_search_start.rex`, `rqd_cwd.cls` sits in both the current directory and the parent
   package's directory. That case is green at `e27a7f843` as well, since the order already held.
   Under a mutant that walks the chain from the outermost parent inward, it prints
   `parent directory` instead of `current directory`. Every other line stays the same. The mutant was
   reversed by an exact edit and checked with `git diff --stat`.

At `ade6fd223` both witnesses match the oracle byte for byte on stdout, stderr and rc. Both sourceline
expectation files were regenerated. Records are in `t6.1/probes/fix-round-2/`.

### Checks

These ran at `ade6fd223`. Exit statuses were read unpiped.

| command (from `rust/`) | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec -p rexx-parse --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib require` (17 passed) | 0 |
| the refusal-table and `package_requires` command above | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (`1069 of 1069 matching`) | 0 |
| whole suite, `REXX_SUITE_TABLE=.../t6.1/suite-r2/table.tsv` | 101 (expected) |
| `callgrind.sh ... -o .../cg-9d/t6.1-r2` | 0 |

### Whole suite

No row of `t6.1/suite-r2/tests.tsv` differs from `t6.1/suite-r1/tests.tsv`
(`suite-r2/changed-vs-r1.tsv`).

### Perf

The table is `t6.1/callgrind-r2.txt`. The previous perf-checked figure is fix round 1.

| program | vs pin | round 1 vs pin | own delta | task total vs Task 4 |
|---|---|---|---|---|
| startup | +0.0173 | +0.0194 | -0.0021 | -0.0015 |
| interpret | -2.1860 | -2.1904 | +0.0044 | +0.0062 |
| parse | -0.0877 | -0.0874 | -0.0003 | -0.0001 |
| churn | +0.1525 | +0.1275 | +0.0250 | -0.0001 |
| sayloop | +0.0331 | +0.0344 | -0.0013 | -0.0005 |

Every own delta is at most +0.05%. Churn's +0.0250 returns it to Task 4's figure (+0.1526).
