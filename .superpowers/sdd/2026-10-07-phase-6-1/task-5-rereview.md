# Task 5 re-review: fix round 1 (including I2)

Fix base `a46465a9f`, head `883a7f4b2` (SDD/plan commits 1d56fd4be, a877498d1, e78e7efc8, ff982eb46
ignored). Reviewer t5-rereview, 2026-10-08. Read-only on the tree. Two release builds from
`git archive` copies (`rust`, `interpreter`, `extensions`), each in its own target directory with a
`Compiling rexx-exec` line: HEAD `883a7f4b2` and fix base `a46465a9f`. Each build also carried a
scratch-only example, `stressrun`, which calls `run_program_collect_every_alloc` and honours
`REXX_SWITCH_MODE=every`. Every probe ran from a fresh directory under the scratchpad's `t5rr/fresh/`:
the oracle under the standard wrapper, and `rexx-run` and `stressrun` under `memcap 2G`. Each was
run in four modes: normal, `every`, collect-every-alloc, and collect-every-alloc with `every`. The
`stressrun` runs reported non-zero collections on every program that fails. stdout, stderr and rc
were compared separately. Where stderr differed, it was compared again with the `Error NN.M:` line
dropped (R3). Target directories deleted.

### Finding Verdicts

- **I1. A parse failure in a `::REQUIRES` that a loader loads diverges and is not recorded:**
  ADDRESSED.
  - The fix is at `install.rs:1899-1902` (`seal_package_level`, called from
    `install_executable`), `lib.rs:2281-2282` (`CallType::Requires` seal on directive failure),
    `lib.rs:2393-2395` and `:2432-2434`, `:2458` (`own_level`), and `dispatch/context.rs`
    `read_snapshot` (a `Requires` frame is `ROUTINE`, named for its package).
  - Checks run (all in four modes; `t5rr/round1.txt` and the `cases2`-`cases4` runs):
    - Loaders `Routine~newFile`, `Method~newFile`, `Package~new(file)` and `loadPackage`, each with
      a nested file that does not parse (35.1 at line 3) and one whose prologue raises (42.3).
      Each was run at one and at two levels of `::REQUIRES` nesting, trapped and untrapped. Trapped:
      stdout identical to the oracle, with the traceback (5 items at depth 2) and `STACKFRAMES`
      (`COMPILE`, `ROUTINE` per level, `METHOD <loader>`, `PROGRAM`). Untrapped: rc and stdout
      identical. stderr is identical apart from the R3 insert line.
    - `::REQUIRES` cycles through each loader:
      - a cycle closed before a parse failure (98.952, agrees);
      - a cycle whose member does not parse at its first instruction (99.916, agrees);
      - a cycle whose member does not parse on line 2 (35.1, agrees apart from R3);
      - the same cycle from a top-level `::requires`.
    - `Package~new` over lines whose `::requires` prologue raises, and the CLOSED row's own probe
      (`Package~new('src', 'call nosuchroutine_zz')`) trapped and untrapped: identical.
    - External call of a file whose `::requires` does not parse, or raises: identical apart from R3.
    - At the fix base `a46465a9f`, every trapped loader probe differed from the oracle on stdout and
      every untrapped one on stderr. At HEAD they agree. The fix is what closes them.
    - I read the stdout of each witness shape: every level the probe claims prints.
  - The exclusions record is now true: the agreement row (`phase-4-exclusions.txt:1958-1962`, the sentence from `:1959`) and
    the CLOSED family row match the measurements above.
- **I2. `whole_groups` expectation rows quote a message HEAD cannot produce:** ADDRESSED. Judged
  from the diff, gate record and report, as instructed, plus one probe.
  - No `does not parse here` text remains in `tests/concurrency_tests.rs`.
  - Every changed or deleted row has a cause in the gate record's table
    (`phase-6-1-gate.md`, `### Task 5 fix round 1, I2`), consistent with the diff:
    - the refusals Task 5 removed (ATTRIBUTE, CONSTANT, METHOD, CALL, GUARD, TRACE);
    - the ones Task 4 removed (Class, TRACE_TraceObject, and Method/Object rest +1/+2 assertions,
      failing sets unchanged);
    - the rest rows deleted because their whole run no longer refuses.
  - The harness change (`concurrency_tests.rs` `rows_of`, REST_LEFT_OUT now left out of every part on
    both sides) is documented on the constant and explains Class whole's new key.
  - No row enshrines a regression:
    - Each new failing set is the old rest row's set, plus tests that refused before.
    - ATTRIBUTE TESTMISPLACEDCLASSMETHOD is one of those. I measured it on the two-line program
      `::attribute 'foo' class` / `say hi`: oracle rc 157 99.905; HEAD rc 157 99.937; fix base rc 157
      99.937. The claim holds, and the divergence predates this round.
    - TRACE_TraceObject TEST_OBJECT_AND_SCOPE previously refused, so it never passed here.
  - `whole_groups` was not run, as instructed. The two green runs are in the gate record.
- **M1. `end` saturates silently above 4 GiB:** ADDRESSED. `token.rs:40-43` states both
  consequences (cut, or empty).
- **M2. Invented user-visible fallback text:** ADDRESSED. `error.rs:1871-1876` now uses
  `debug_assert!` and an empty echo.
- **M3. Three copies of parse-and-raise:** ADDRESSED.
  - `run.rs:3259` `Interp::parse_file` now serves `lib.rs:2127`, `install.rs:654` and
    `install.rs:1829`.
  - Each site passes the same name it did before (`resolved`, `resolved`, `name`) and keeps its
    `Rc` wrapping.
  - Behaviour is unchanged at all three sites. The `ext-*`, top-level `::requires` and `*nf-*`
    probes agree with the oracle, and their outputs match the fix base everywhere except the levels
    I1 added.
- **M4. Gate sentence "the pad prints the same as base":** ADDRESSED. `phase-6-1-gate.md:502`
  states both deltas. +0.0395 against +0.0388 is +0.0007.

### New Breakage in the Fix Diff

- **Minor.** `install.rs:2526-2532`: `seal_package_level` was inserted between
  `blame_directive_in`'s doc block and its `fn`.
  - `seal_package_level`'s rustdoc now opens with "[`Interp::blame_directive`] for a directive in
    the package `id`, whose report names that package's own file...". That sentence is false for
    this function.
  - `blame_directive_in` (`:2538`) is left with no doc.
  - Fix: move the two orphaned lines back above `fn blame_directive_in`.

No other breakage found:
- Directive-time failures (98.909, `::class a subclass nosuchclass_zz`) in a nested file now go
  through `seal_package_level` too. For every loader, trapped, the traceback and `STACKFRAMES` now
  agree with the oracle, where at the fix base they did not.
- Under `trace r`, a successful prologue loaded from an indented `DO`, from an internal routine
  (where the fix base indented wrongly and HEAD matches), and from a top-level `::requires` traces
  identically to the oracle. The only difference is the `>I>`/`<I<` pair, which is already recorded
  (`phase-4-exclusions.txt` "THREE TRACE AND BLAME DIVERGENCES").

### Checks run beyond the findings

- **The new KNOWN GAP is true** (`phase-4-exclusions.txt`, "A PROLOGUE'S LIVE STACK FRAMES LACK THE
  LEVELS THAT LOADED IT"). Measured with `.context~stackframes` in a prologue:
  - Top-level `::requires 'p.cls'`: oracle `ROUTINE p.cls 1`, `PROGRAM main.rex 2`; here
    `ROUTINE p.cls 1` alone.
  - `.Package~new('p.cls')`: oracle `ROUTINE`, `METHOD NEW`, `PROGRAM`; here `METHOD NEW` is
    missing.
  - "A condition raised in the prologue carries every level on both" holds: see the `run` probes in
    I1.
  - Not stated in the row, but within its headline: in nested and `newFile` shapes the intermediate
    `ROUTINE mid.cls` frames and `METHOD NEWFILE` are missing too.
- The `collect_stress` zero-collection list edit is consistent with the new frames being fresh
  allocations. Under collect-every-alloc, every probe above collected and matched its plain run.

### Out-of-Scope Observations

- The untrapped directive-time error in a required file names the program in its `Error 98 running
  <file>` line. That is `main.rex` where the oracle names `bad.cls`, and it is identical at the fix
  base. It is already recorded (`phase-4-exclusions.txt`, "THREE TRACE AND BLAME DIVERGENCES", the
  k2.cls example), so not new.
- ATTRIBUTE 99.937/99.905 has no owner. The controller has queued it to Task 12 (`progress.md:119`).
- CALL whole now agrees with the oracle, including TEST_4. The CALL rest run, which left out only
  TEST_INVALID, failed TEST_4 (the elapsed-clock defect). Nothing records why running TEST_INVALID
  first makes TEST_4 pass. If that depends on order or timing, the deleted CALL whole rows could flake
  later.
- The tree's HEAD moved to `f056301ca` during this review. Everything here is against the
  `883a7f4b2` archive.

### Verdict

**Fix round:** All findings addressed, no new Critical/Important breakage. One new Minor: the
orphaned doc block at `install.rs:2526-2532`.
