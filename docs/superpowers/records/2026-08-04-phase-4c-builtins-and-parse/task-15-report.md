# Task 15 report — the `base/bif` L1 harness, the 4c subset, `mutate-4c.sh`, and the gate

**Status: DONE_WITH_CONCERNS.**
Everything the brief asks for is built, run and committed.
The concerns are things measured along the way that the brief could not have known and that a reader should not take on trust: two of the brief's own figures did not reproduce, the `base/bif` headline is over a population this extractor chose, and the trace indent of every 4c construct is unpinned.

## What was built

Four commits, one per step.

| step | commit | what |
|---|---|---|
| 4 | `f0426f09` | the 4c subset under the allocation-stress collector |
| 1-3 | `8ac61371` | the `base/bif` extractor, its harness, and `bif-exempt.txt` |
| 5 | `1c94e50b` | `rust/scripts/mutate-4c.sh` |
| 6-7 | `89debc85` | `docs/superpowers/plans/phase-4c-gate.md` |
| fix round 1 | `70be78e2` | the pin on which subset files the stress collector reads |
| fix round 1 | `e04fb46f` | the two Step 0 figures written back into the plan |
| fix round 1 | `d0102460` | the gate document records the pin and its falsification |
| fix round 2 | `80ff9c1f` | the same pin on `corpus.rs`'s and `coverage.rs`'s union call sites |
| fix round 2 | `64ce92fe` | criterion 1's denominator recorded as pinned |

## Step 0's rulings, and the two figures in the brief that did not reproduce

The four rulings in the dispatch were followed as written.

* **The 5 `assertSame` calls in `::routine` bodies are dropped**, under their own `DropReason::RoutineBody`, and the conservation invariant carries the count.
  Measured: **1 body, 5 calls**, which is exactly the brief's figure.
* **`::options novalue` files do not get the symbol-equals-own-name resolution**; a row reading an unassigned symbol there is dropped instead.
  Measured: **10 bodies, 28 calls**.
* **The `base/bif` result is reported, not gated.**
* **`bif-exempt.txt`'s attribution column is derived from `rexx-exec`'s own loud message.**

**Two of the brief's measured figures did not reproduce, and both were re-derived because the extractor's behaviour depends on them.**

1. **The block-comment population is 25 `assertSame` calls, not "120-135".**
   24 sit inside `/* … */` (23 in `LINES.testGroup`, 1 in `CHARS.testGroup`) and 1 is behind a `--` (`DATE.testGroup:1140`).
   Two independent nested-comment blankers agree, and the brief's own reconciliation names exactly those three files ("`CHARS.testGroup`, `LINES.testGroup` ×2"), so the file set was right and the call count inside it was over-estimated.
   The consequence is that the live-method-body population is **6,261**, not the brief's 6,150-6,170, and the reconciliation still closes exactly: `6,261 + 25 + 5 + 2 = 6,293`.

2. **`::options novalue` covers 43 of 76 files, not 38.**
   `::options all <condition>` **also enables NOVALUE**, and five files use that spelling: `BEEP`, `CONDITION`, `DATE`, `LINEOUT`, `XRANGE`.
   Verified on the oracle rather than read off the grammar -- a program whose only directive is `::options all syntax` fails `say abc` with `Error 98.986, Reference to unassigned variable "ABC"`, exactly as `::options novalue error` does.
   Matching only the `novalue` spelling would have applied the symbol-equals-own-name resolution in five files where the idiom raises, which is the wrong direction the brief's own ruling 2 warns about.

**One figure differs for a reason that is not a disagreement.**
The brief gives 200 bodies containing both `expectSyntax` and `assertSame`, 199 with the assertion at or after, carrying 205 calls.
The extractor attributes **189 bodies / 195 calls** to `DropReason::ExpectedRaise`.
The difference is attribution ordering, not population: a body already blocked by a message send or an unsupported statement has its calls attributed to that stronger blocker, which is checked *before* the `expectSyntax` branch.
**No value row is ever emitted from such a body either way** -- the blocked path returns before the row-building path is reached -- so the hazard the brief's Step 2 is about is closed regardless of which bucket the call lands in.
The brief's 200 was not independently re-derived.

## Step 4: the wiring

`tests/collect_stress.rs` was the one `read_subset` call site still reading two files, and its own doc comment said so and said the fix was this task's.
Both were corrected -- the call and the sentence.

`tests/corpus.rs` (three files) and `tests/coverage.rs` (three at its union site, one each at the three per-phase pins) were **confirmed rather than redone**, as were Task 7's `phase_4c_subset_matches_the_committed_list` and Task 9's `corpus.rs` widening.

**Why the widening was worth doing beyond discharging D6:** the 4a/4b subset **calls no builtin at all**, so every allocation the 66 names make was outside `run_program_collect_every_alloc`'s reach.
The widened subset's 50 programs each perform a non-zero number of collections, asserted per program.
It did not, at this gate, catch anything -- no mutation's suite catcher was the stress run -- and the gate document says so rather than letting "the collector now covers the builtins" read as "the collector caught a builtin rooting bug".

`corpus.rs`'s dated-row convention was followed: a new row records 50 of 50 at `1c519dfc`, and the 47 the previous row carries is untouched.

`corpus/README.md` gained the five rules a `phase-4c.txt` program must obey, beside the `DO OVER` one.

## Steps 1-3: the extractor and the harness

`crates/rexx-extract/src/bif.rs` is a third module, not a third engine.
It reuses `AssertionRow`, `keyword::count_assert_same` as its independent denominator, `keyword::blank_comments`, `keyword::parse_two_args`, and keyword's `rows + dropped == calls` conservation law.
Five private helpers in `keyword.rs` became `pub(crate)` and nothing in that module changed behaviourally.

**Result: 4,999 value rows and 186 raise rows out of 6,293 `assertSame` calls; 4,920 and 184 of them pass.**
All 81 failures are on `corpus/bif-exempt.txt`.

**The one place reuse was wrong, and it was silent.**
`keyword::is_symbol_char` is `alphanumeric || '_'`, which is right for "does the method name `assertSame` end here" and wrong for reading whole Rexx symbols: it makes `.v8` scan as the symbol `v8`, and **every `.local` fixture in the corpus stopped resolving**.
The rows still balanced conservation and the harness still reported a pass rate.
Caught by dumping `WORD::test057`'s prelude and seeing `v8. = .v8` rather than the substituted text.
`bif.rs` now has its own wider predicate with the difference stated in its doc.

**Three drop reasons exist because rows were run rather than read**, and each was producing a row that asserted a value the interpreter must never produce.
All three balanced conservation, which is precisely what that invariant cannot see.

* **`NonUtf8Source` (6 bodies, 11 calls).**
  Six groups hold raw bytes above `0x7F` that are not valid UTF-8 -- `C2X.testGroup` writes 250 literal `AA` bytes -- and `String::from_utf8_lossy` turns each into a three-byte replacement character.
  `C2X::test_4` then asserted that `C2X(<250 replacement chars>)` equals `COPIES('AA',250)`, which is a different string entirely.
  Any row whose text carries `U+FFFD` is dropped.
* **`SideEffectingAssertion` (1 body, 6 calls).**
  `VALUE(name, newvalue)` assigns while `assertSame`'s own arguments are being evaluated.
  `VALUE.testGroup`'s `test051` runs `assertSame(17, value(n,'abc'))` -- true, because the setter answers the old value -- and then `assertSame('abc', value(n))`, which is true only because the previous call assigned.
  A prelude of assignments cannot carry that, so the row that assigns is emitted and everything behind it is dropped.
* **`ClockDependent` (1 body, 1 call).**
  D11 bars `RANDOM`, `DATE` and `TIME` from a differential corpus, and the same hazard applies here because the harness runs a value row's two operands as **two separate programs**.
  `DATE`/`TIME` are only clock reads when given no input value, which is why 716 `DATE` rows are deterministic and one is not; an omitted *second* argument (`date('S',,'I')`) counts as no input value, which a rule that merely counted arguments would wave through.

**The harness runs each value row as two programs and compares their stdout byte for byte.**
One program printing two lines cannot work here: `BITAND.testGroup`'s expected values contain `0A` bytes, so those rows print four lines from two `SAY`s and every one of them read as a malformed run.
Byte-for-byte rather than asking `rexx_exec` to evaluate `==` is `assertions.rs`'s reason, unchanged -- the rendering is what is under test.

**Raise rows carry both operands, in evaluation order.**
The brief's example (`assertSame(C2D(,-1), '-1')`) has the raiser first; `WORD.testGroup` writes `assertSame('one', word((v8.),1))` with the call second.
A row keeping only one of them would be checking the literal half of some bodies.

**The exempt set, 81 rows:** 47 blocked on builtins `builtin-status.txt` calls `excluded` (`STREAM`, `RXQUEUE`, `LINES`, `CHARIN`, `CHAROUT`, `CHARS`, `QUALIFY`), 22 on message sends in their own operands (Phase 5), 9 on `ExprKind::DotVariable`, which is loud and carries no owner, and 3 on `BEEP`.
The set assertion runs under a plain `cargo test`, per Step 3.

**`BEEP` is a finding.**
Measured on the oracle, `retc = beep(262, 1)` exits 0 and answers the null string; this crate raises 43.1.
`BEEP` is not a builtin function -- it is a routine the interpreter's internal package registers (`interpreter/runtime/InternalPackage.cpp:199`) -- so its absence from `rexx_inventory::builtins::NAMES` is correct, and the gap is in what this crate provides beside the builtin table.
Nothing in Phase 4's scope covers it.

## Step 5: the mutation script

Nine mutations, two instruments, **9 of 9 as declared** at exit 0, with the unmutated tree passing both instruments before the first mutation and after the last restore.
Run three times in full: once with row 7's original declaration (8 of 9), once with the correction, and once more against the committed file after a comment edit, since editing a script bash is executing is not something to draw a conclusion through.

The three additions the brief requires are all present and all fired at least once as guards:

* `--no-fail-fast` on every run.
* **The binary count is asserted, baseline against mutated, and measured in the same run.**
  The guard counts `Running`/`Doc-tests` header lines off **stderr** and the pass/fail counts off **stdout**, as separate files.
  The baseline run establishes the figure (75 binaries at this commit) and every mutated run must reproduce it; a truncated run is `INFRA_FAILURE`.
  No constant is written into the script.
* A wall-clock `timeout` around every `cargo test`, with exit 124 classified `INFRA_FAILURE`.

**One declaration was wrong and was corrected to what was measured.**
Row 7 (`::ROUTINE` resolved before the builtin table) was declared `PASSED`/`DIVERGED` on the reasoning that the resolution order had only an in-crate witness.
It measured `DIVERGED`/`DIVERGED`: `corpus/lang/routine_dispatch.rex` defines `::routine length` and `::routine 'max'` for exactly this purpose.
The row was rewritten to declare what it measures, with the correction written at the row, and the script re-run in full.

**Coverage claims are per finding.**
The per-row catcher lists are in the gate document.
The one worth repeating: **row 5, the `PARSE` comma fence, is caught in-crate by exactly one test over the whole workspace, and it is `bif_assertions::the_exempt_set_matches_the_current_failures`** -- the harness this task added.

**No row is declared `PASSED`/`PASSED`**, so the rule requiring a written justification for such a row did not have to be exercised.

**Row 8 is `TIME('R')`, and its corpus `PASSED` is D11 rather than a gap.**
The declared catchers are Task 12's own unit tests; the two that fired are `time_r_resets_relative_to_the_last_reset_not_program_start` and `a_callees_own_time_r_leaks_into_the_caller_after_it_returns`.
The brief names `two_time_r_reads_in_one_clause_answer_identically` as the declared catcher; that test exists and passes, but it is **not** what this mutation makes fail -- it asserts that two reads in one clause agree, which stays true when the anchor never moves.
The row's comment names Task 12's unit tests as the gate and the observed pair is recorded from the run.

## Step 7: every figure, with its command and unpiped exit status

All run from `rust/`, exit status read unpiped, stdout and stderr captured as separate files.

| figure | command | result | exit |
|---|---|---|---|
| workspace suite | `cargo test --workspace --no-fail-fast` | 1,289 passed, 0 failed, over 75 binaries | 0 |
| corpus gate | `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | 50 of 50 matching | 0 |
| assertions gate | `REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec --test assertions` | 4,224 of 4,259 rows | 0 |
| keyword gate | `REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions` | 888 of 896 bodies, 1,737 of 1,773 calls | 0 |
| bif gate | `REXX_BIF_GATE=1 cargo test -p rexx-exec --test bif_assertions` | 4,920 of 4,999 value rows, 184 of 186 raise rows | 0 |
| mutations | `bash scripts/mutate-4c.sh` | 9 of 9 as declared | 0 |
| subset-file pins | `cargo test --workspace --no-fail-fast`, three `*_reads_every_phase_subset_file` tests | all green; red in both falsification directions | 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | clean | 0 |
| format | `cargo fmt --all --check` | clean | 0 |

**Baseline before any change**, at `1c519dfc`'s child `284ef543`: 1,271 passed, 0 failed, 73 binaries, 74 `test result:` lines; corpus 50 of 50; keyword 888/896 and 1,737/1,773; assertions 4,224/4,259.
Every one reproduced the brief's figure exactly.
The suite grew by 15 tests and 2 binaries, which are this task's own.

## Fix round 1

Both Important findings are fixed. Neither Minor finding was touched.

### Important 1 — the stress subset's file list is now pinned, and the pin is falsified

The reviewer's measurement reproduced: deleting `&corpus_dir.join("phase-4c.txt")` from `collect_stress.rs`'s `read_subset` call left the whole workspace green.
The reason is worth stating, because it is why no existing check could see it: every assertion `the_l0_subset_passes_again_under_collect_on_every_allocation` makes -- non-empty subset, no mismatches, no zero-collection program, non-zero total -- holds just as well over a smaller union, and `coverage.rs`'s three `phase_*_subset_matches_the_committed_list` tests pin each file's **contents**, never which harness reads it.

The list is now the constant `SUBSET_FILES`, and `the_stress_subset_reads_every_phase_subset_file` asserts it against the corpus directory.
Reading the directory rather than writing the names a second time is deliberate: a hand-written expected list would be satisfied by a copy edited in the same change, and it would not catch a `phase-4d.txt` added later and never wired in.

**Falsification, run:**

```
$ python3 -c "...remove phase-4c.txt from SUBSET_FILES..."
$ cargo test --workspace --no-fail-fast     # stdout and stderr to separate files
EXIT=101
passed=1286 failed=1
75 binaries
```

```
---- the_stress_subset_reads_every_phase_subset_file stdout ----
thread 'the_stress_subset_reads_every_phase_subset_file' panicked at
crates/rexx-exec/tests/collect_stress.rs:169:5:
assertion `left == right` failed: the collect-on-every-allocation run does not
read every phase subset file in rust/corpus/. A file missing from SUBSET_FILES
is a phase whose programs never reach this harness, and nothing else in the
workspace can see that -- the run passes over whatever it was given
  left: ["phase-4a.txt", "phase-4b.txt"]
 right: ["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"]
```

The file was uncommitted at the time, so it was backed up with `cp` and the restore verified with `sha256sum -c` (`OK`, exit 0) rather than with `git checkout`.
The restored tree runs 1,287 passed / 0 failed over 75 binaries, exit 0; clippy exit 0; `cargo fmt --all --check` exit 0.

**Scope note, not fixed here.** `corpus.rs` and `coverage.rs` carry the same shape of hazard at their own union call sites. The reviewer's own finding says the consequence differs there -- a shrunken subset moves an "N of N" figure other criteria read -- and this round was scoped to `collect_stress.rs`, so they are left as they are rather than widened without a finding.

### Important 2 — both figures written back into the plan

`docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md`, in the plan's dated-row style, marked measured 2026-08-07 at `70be78e2`.

**The reconciliation** is now exact rather than a band: **6,268 outside any comment, 24 inside `/* … */` block comments, 1 behind a `--`**, closing at 6,293.
The 6,268 is broken down the way the extractor actually counts it -- **5 in `::routine` bodies, 2 outside any `test`-prefixed `::method`, 6,261 in test method bodies** -- with the `::routine` five stated as being *inside* the 6,268, since the reviewer asked which.
The two sentences the band supported were rewritten rather than left stranded: "the block-comment band is a range because …" is gone, and "**a line-oriented scan extracts over a hundred assertions that never run**" was false and is replaced by the true size and location -- **23 of the 24 are in `LINES.testGroup` alone**, in two `/* disable test … */` blocks, with the 24th in `CHARS.testGroup` and the lone `--` at `DATE.testGroup:1140`.
Blanking comments is still required; what changed is that it prevents a 25-call error, not a hundred-call one.

**The `::options` split** is **43 of 76**, with the complement corrected from 38 to **33**.
The spellings are written down, because the spelling is why the count was wrong: whitespace-normalised, **36 × `::options novalue error`, 2 × `::options novalue syntax`, 5 × `::options all syntax`** -- 38 files matching `novalue`, 5 matching `all`, **no file matching both**.
A `sort | uniq -c` that does not strip trailing whitespace splits the first group as 35 + 1; that is the same 36, and it is noted so the two renderings do not read as a discrepancy.

**Oracle evidence, recorded because this is a claim a later reader will want to re-check.**
`say nv1` as a program's only clause, from a fresh empty directory:

| directive | stdout | exit |
|---|---|---|
| none | `NV1` | 0 |
| `::options novalue syntax` | — | 158, `Error 98.986: Reference to unassigned variable "NV1"` |
| `::options all syntax` | — | 158, `Error 98.986: Reference to unassigned variable "NV1"`, byte for byte identical |

The five `all` files are `BEEP`, `CONDITION`, `DATE`, `LINEOUT` and `XRANGE`.
Matching only the `novalue` spelling would apply the symbol-equals-own-name resolution in those five, where the idiom raises.

**One figure was dropped rather than adjusted.**
The row also carried "roughly 1,325 bodies / 2,027 calls live under it".
Measured, **3,068** of the 6,293 `assertSame` calls live in the 43 files and **2,145** in the 38 `novalue`-spelling ones; 2,027 matches neither, and how it was derived is not recorded, so there was nothing to scale and the plan now says so.

**The confidence line was corrected with them.**
It listed "the reconciliation" and "the 38-of-76 `::options` split" among the figures it called *firm*; both were wrong, and a correction that left that sentence standing would have contradicted itself two paragraphs later.

### Re-run after the fixes

`mutate-4c.sh` was re-run in full rather than assumed unaffected, because the gate document quotes its baseline: **9 of 9 as declared, exit 0**, with the unmutated tree at 50 of 50 matching and 75 binaries / 1,287 passed / 0 failed both before the first mutation and after the last restore.
The script derives its binary count from its own baseline, so the suite growing by one test moved nothing in it.

## Fix round 2

One finding, from my own round-1 scope note. The reviewer measured what I had deferred, and the measurement went against the deferral.

### The finding, and why the deferral was wrong

My round-1 note said `corpus.rs` and `coverage.rs` carry "the same shape of hazard", and repeated the review's own reasoning that there a shrunken subset "moves an N of N figure other criteria read".
**That reasoning is wrong for `corpus.rs`**, and I should have measured it rather than repeating it.
Reproduced: dropping `phase-4c.txt` from `corpus.rs`'s union call site leaves **both plain mode and `REXX_CORPUS_GATE=1` at exit 0**.
The report line changes from `50 of 50 matching` to `42 of 42 matching`, and nothing asserts on it -- the gate assertion is `assert!(!gate || mismatches.is_empty())`, and a subset that has lost a whole phase has no mismatches to report.
A number a reader might eyeball is not a check, and this is the figure the gate document quotes as criterion 1.

This is the same defect I had just fixed one file over, and the thing that let it stand was my own summary of somebody else's argument.

### What was done

All three union call sites now carry the device built for `collect_stress.rs`: a named `SUBSET_FILES` constant plus a pin asserting it against `fs::read_dir` on the corpus directory.
Duplicated in each of the three rather than shared, following the reasoning each file's own module doc already gives for `read_subset` being duplicated -- these are three integration-test binaries and none can `mod` another. Those comments are untouched.

`coverage.rs` was pinned too even though it is caught today.
Its protection is real but incidental: dropping the file fails `every_in_scope_variant_is_witnessed_by_the_phase_subsets` with `4 in-scope variant(s) unwitnessed: Parse, Arg, Pull, Address::Environment`, which holds only while 4c still owns a variant no earlier phase witnesses.
That is a property of today's corpus, not an invariant, and its own `SUBSET_FILES` doc says so.

### On the gate assertion itself (the reviewer's point 5)

**Checked, and not widened.**
`corpus.rs` already has `assert!(!subset.is_empty(), ...)`, and it does **not** cover this case: a union that has lost one file is still non-empty while the others name programs, so the guard is never reached.
It covers a different thing -- every named file present and every one of them naming nothing -- and that is now written at the guard rather than left to be re-derived.
Adding a non-zero-total assertion to the gate would be a second, weaker version of the pin: it would catch a subset shrinking to *zero* and miss a subset shrinking from 50 to 42, which is the case that actually happened.
The pin is the check; the guard stays as the narrower thing it is.

### Falsification, both directions, all three harnesses

Both files were backed up with `cp` and both restores verified with `sha256sum -c` (`OK`, exit 0). No `git checkout --`.

**A. Drop `phase-4c.txt` from all three `SUBSET_FILES` constants.**

```
$ cargo test --workspace --no-fail-fast     # stdout and stderr to separate files
EXIT=101
passed=1286 failed=3        75 binaries
```

```
---- the_differential_reads_every_phase_subset_file stdout ----
panicked at crates/rexx-exec/tests/corpus.rs:502:5:
assertion `left == right` failed: the corpus differential does not read every
phase subset file in rust/corpus/. A file missing from SUBSET_FILES is a phase
whose programs are never run against the oracle, and the run stays green over
whatever is left -- the headline shrinks and nothing asserts on it
  left: ["phase-4a.txt", "phase-4b.txt"]
 right: ["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"]

---- the_union_reads_every_phase_subset_file stdout ----
panicked at crates/rexx-exec/tests/coverage.rs:545:5:
assertion `left == right` failed: the variant-coverage union does not read every
phase subset file in rust/corpus/. ...
  left: ["phase-4a.txt", "phase-4b.txt"]
 right: ["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"]
```

The third failure is `every_in_scope_variant_is_witnessed_by_the_phase_subsets`, the incidental catch described above.

**B. Add a scratch `corpus/phase-4d.txt` and wire it nowhere.**

```
$ printf '# scratch, falsification only\nlang/parse_triggers.rex\n' > corpus/phase-4d.txt
$ cargo test --workspace --no-fail-fast
EXIT=101
passed=1286 failed=3
failing: the_differential_reads_every_phase_subset_file
         the_stress_subset_reads_every_phase_subset_file
         the_union_reads_every_phase_subset_file
```

All three, and only the three pins.
This direction is also what shows the pins are not self-referential: a pin comparing the constant against a second hand-written copy would be green here.
The scratch file was removed and the three test files verified unchanged by checksum.

### Re-run after the fixes

Restored tree: `cargo test --workspace --no-fail-fast` 1,289 passed / 0 failed over 75 binaries, exit 0; `REXX_CORPUS_GATE=1` 50 of 50 matching, exit 0; clippy exit 0; `cargo fmt --all --check` exit 0.
`mutate-4c.sh` re-run in full, since criterion 6 quotes its baseline: **9 of 9 as declared, exit 0**, with the unmutated tree at 50 of 50 matching and 75 binaries / 1,289 passed / 0 failed both before the first mutation and after the last restore.

## Concerns

1. **Two of the brief's measured figures did not reproduce** (the block-comment population and the `::options novalue` file split), and both were load-bearing for the extractor. The second would have produced wrong rows in five files. Both are re-derived above with the commands.

2. **A near-miss worth recording, because I nearly reported it as a flake.**
   Mutation row 4 (`parse_template.rs`, the `.` placeholder) is caught by two `builtin::datetime` unit tests, which looks like load sensitivity in a clock test.
   It is not: both run `parse value date('T') burn() date('T') with n1 . n2`, so the placeholder is load-bearing for them, and the catch reproduced in all three sweeps.
   The first draft of this report asserted they were wall-clock flakes "for a mutation that cannot reach them" -- a sound-looking inference from a true premise, killed by reading the test.
   Nothing needs fixing; the concern is that a test's name says nothing about which code it reaches, and a coverage claim built on names is wrong in both directions.

3. **The `base/bif` headline is a measurement over a population this extractor chose.**
   4,999 of 6,293 calls became value rows; the other 1,294 are named and counted, but the largest categories (418 calls in bodies that send a message, 467 in bodies whose statements the scanner cannot carry) are Phase 5's and will not shrink at 4c.
   A reader should not read 4,920 of 4,999 as "98% of `base/bif` passes"; it is 98% of the 79% that could be lifted.

4. **The trace indent of every 4c construct is unpinned**, and the gate document says so in criterion 3 rather than in a footnote.
   Both differential harnesses normalise the run of spaces after the marker, on both sides, and the three unnormalised indent witnesses are `run.rs` unit tests over 4a-era shapes.
   Nothing in this repository would see an off-by-two indent on a `PARSE` target's value line or a `>I>` routine entry.

5. **`BEEP` works under the oracle and raises 43.1 here.**
   It is not a builtin on either side, so it is outside D4's exclusions *and* outside the 66. Nothing in Phase 4 owns it.
