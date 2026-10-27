# Task 8.4 fix round 4 re-review

Range 4d5b702b1..76255f5e8. Read-only; mutants ran on a detached worktree under /tmp, since removed.

## Verdicts

1. Coverage (TRANSLATION_REFUSED): ADDRESSED.
2. define axis: ADDRESSED.
3. directive_options do-repeat-count: ADDRESSED.

## Evidence

Tests run at HEAD with a fresh target dir: coverage 21 passed, directive_options 24 passed, rexx-bench-suite
10 passed (rust/target/release/rexx present).

### 1. Coverage

- The proof is kept and in one respect sharper. Every program that parses still feeds the variant sets;
  the refused programs contribute nothing, and the test still passes, so no variant relied on a refused
  program alone. The coverage report assertion is unchanged.
- Exactness: `(e.code(), e.sub())` is compared with the listed pair (coverage.rs:1398-1405).
- Stale entries fail loudly. Mutants on the worktree copy, each run with `every_in_scope`:
  - entry deleted (digits_below_fuzz): panics "failed to parse" (33.1);
  - non-existent entry added: panics "listed in TRANSLATION_REFUSED and in no phase subset";
  - wrong sub-code (902 for the 901 program): panics "refused with a different error";
  - an entry naming a program that parses (hello.rex substituted for class_duplicate_class): the removed
    program then panics "failed to parse", so the swap is caught from the other side.
- The parsed-and-listed arm (`(Ok(_), Some(_))`, coverage.rs:1406) was not exercised by a mutant; it is
  reached by reasoning only.

### 2. define axis

- `Role::Loop` is correct: the program has a loop bound, and the other Loop axes (dirread, dispatch) declare
  `n = <digits>` the same way. Adding `n = 3000` and `to n` keeps the work identical.
- Oracle and crate both print `3000 2999`, rc 0, from a fresh empty dir.
- It still exercises what 8d409845b added it for: the define/new/collect loop body is untouched.

### 3. do-repeat-count

- Without the directive both sides answer 26.2 rc 230; with `::options lostdigits syntax` both answer
  98.972 rc 158, identical on stdout, stderr and status (run on both). This matches rexxref
  (LOSTDIGITS truncation raising, escalated by the directive), so Raises is right.
- `Expect::Diverges` had exactly one row; with that row moved, the variant and its arm are dead and
  removing them loses no check. The Raises arm asserts the oracle's stderr carries 98.972, so the
  moved row is held to that too.

## New problems

Critical: none.

Important: none.

Minor:
- coverage.rs:1398-1408: a refused program skips `assert_program_has_only_admitted_directives`, so a
  refused program could carry a directive outside the admitted set unnoticed. These programs are class
  and options directive programs by nature; low risk.

## Out of scope

The report's workspace run (datetime_zone, collect_stress waited timing flake) was not re-run.
