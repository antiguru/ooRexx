# Task 1 -- the clause line, from a table rather than a search

**Status: complete, change accepted.**
Base `9b2d416d3`, code at `5b3be9536`, record at `a8ea4f5be`, all read back from `git log`.
Working tree clean.

## The short version

Seven axes down between 1.026% and 7.366% on `perf stat -e instructions:u`, none up, every arm pair non-overlapping by at least eighty times the wider of its two spans.
Nothing a program observes changed, on either engine, anywhere.
The plan's Step 5 stop condition did not fire and was not close to firing.

**One thing the task found that the plan did not expect: the plan's own design finding is wrong.**
It is stated in the plan as "the design fact this unit turns on", it comes from the spike arm the plan itself disowns two paragraphs earlier, and the landed fix measures the opposite.
The unit's *decision* is unaffected. The reason given for it was not the reason it is right.

## Steps

All six completed.

### Step 1 -- the base, each axis's own spread, and the call count

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, six runs of the one base binary staged at one fixed path, from a fresh empty directory.

| axis | base, minimum of six | same-binary span | span as % |
|---|---:|---:|---:|
| `emptyloop` | 27,150,609,168 | 1,405 | 0.00001% |
| `varlookup` | 44,802,636,405 | 1,620 | 0.00000% |
| `arith` | 20,413,489,649 | 1,693 | 0.00001% |
| `strings` | 44,913,588,182 | 36,001,118 | 0.08016% |
| `compound` | 25,027,490,216 | 1,680,896 | 0.00672% |
| `alloc4c` | 8,183,660,084 | 129,445 | 0.00158% |
| `rexxcps` | 25,259,114,646 | 19,581 | 0.00008% |

`strings`' span is one run 36 million high, the interference signature entries 29, 30 and 32 all record on that axis.

**The call count, and it is the reason no axis here is a bound.**
`Interp::clause_line`'s call into `line_of` instrumented with a counter, release, both engines.
The two engines answer **identically on every axis** -- the compiled engine's promoted clauses do not reach `Interp::step` and reach `enter_stepped_clause` anyway.

| axis | searches | body lines |
|---|---:|---:|
| `emptyloop` | 50,000,005 | 7 |
| `varlookup` | 57,000,006 | 8 |
| `strings` | 18,000,007 | 12 |
| `compound` | 15,001,011 | 14 |
| `arith` | 4,000,006 | 14 |
| `alloc4c` | 4,000,006 | 51 |
| `rexxcps` | 10,161,437 | 198 |

`emptyloop`'s 50,000,005 is exactly two per pass of 25,000,000: the `nop`'s own step, and the loop header's re-read in `run_repeating`. Both are sites this change fixes.

The spike's `rexxcps` count was 10,000,000; the real figure is 10,161,437, which does not change anything the spike concluded from it.

### Step 2 -- the table, in `indents`' own shape

`Plan::lines: Box<[usize]>`, one entry per instruction, filled by a walk in `Plan::build`, read through `Plan::line_at`.
`Plan::build` now takes `source: Option<&ProgramSource>` and `Interp::plan_for` takes `&ProgramSource`; `fragment_plan` passes `None`, so an `INTERPRET` fragment grows no table -- its clauses report the enclosing clause's line through `clause_line_override`, its `Code::plan` is `None`, and its plan is discarded with it.

`Interp::clause_line_at(code, index, instruction, source)` is the read side: `source?`, then the override, then the table, then the search.
Three call sites, all of which already held an index:

* `enter_stepped_clause` (`run.rs`), the per-clause site;
* `run_repeating`'s `do_line`, the per-pass site;
* `run_repeating`'s `end_line`, once per loop.

`Interp::clause_line` is unchanged and still serves `clause_site`, whose callers do not all hold an index and which is on the trace and failure paths rather than the hot one.

**The loop-header question the plan asked is yes**: `run_repeating` already carries `do_index` and `end_index` beside the instructions, so both sites read the table with no plumbing.

Blast radius, from the compiler rather than from a grep: the two production `plan_for` callers, the `plan_for` and `Plan::build` test helpers, and `ir/corpus_shape_tests.rs`'s `check_body`, which was given the source so the corpus shape sweep exercises a filled table rather than an empty one.

Four tests added: `line_at_answers_what_line_of_answers_at_every_index`, `line_at_falls_back_when_the_plan_was_built_without_a_source`, `clause_line_at_answers_what_clause_line_answers_and_the_override_still_wins`, and `build_fills_what_line_of_computes_for_every_corpus_program`.

### Step 3 -- the tripwire, and the inversion that proves it live

`Plan::line_at` carries `debug_assert_eq!(cached, source.line_of(span.start))`.
Whole workspace, `--no-fail-fast`, `memcap 8G`: **1487 passed, 0 failed, 4 ignored, zero firings.**
Inverted to `debug_assert_ne!`: **1084 passed, 403 failed, 400 distinct names.** The zero is a live zero.

| mutation | result | distinct failing names |
|---|---|---:|
| the accessor tripwire inverted | 1084 passed, 403 failed | 400 |
| the index/instruction pairing assert inverted | 1085 passed, 402 failed | 399 |
| **T1** the table filled one line high | 1083 passed, 404 failed | 401 |
| **T1'** the same, tripwire deleted | 1453 passed, 34 failed | 34 |
| **T2** the table never filled at all | 1485 passed, **2** failed | 2 |
| **T3** the table consulted before `clause_line_override` | 1486 passed, **1** failed | 1 |

* **T1 minus T1' is 367 distinct names** the tripwire alone catches.
* **T1' still fails 34**, including `both_engines_agree_on_every_case_file` and `both_engines_agree_on_every_branch_shape`, so a wrong line is observable to the differential gates without it. The tripwire buys the margin, not the whole catch, and I ran the pair rather than assuming either half.
* **T2's two catchers are this task's own tests.** Correct: a table that misses falls back and is right.
* **T3's one catcher is this task's own wiring test**, and see the honesty note under Step 4.

**Harness discipline, and it caught something.**
Each mutation was snapshotted from the live tree immediately before it was applied and verified after restore against `D0`, the `sha256` of `git diff` recorded once when the change was complete -- a derivation from `HEAD` plus the tree, not from the backup the restore came out of.
A later probe modified `bin/rexx-run.rs`, which was clean when its snapshot was taken and so was never in it; the restore left the file modified and **the `D0` check caught it**. A same-snapshot comparison would have passed. It was restored from the `HEAD` snapshot taken at the start and re-verified.

### Step 4 -- the `BodyKey` question, answered by running

`Plan` was given a probe field recording the `ProgramSource` address it was built from, and `plan_for` was made to assert, on a cache hit, that the source it is being asked with is that same one.

* **Zero mismatches** over 381 corpus, bench and oracle-sample programs on both engines, and over the whole workspace suite. `Interp::programs` never held more than one program in any run.
* **The probe is live.** Inverted to `assert_ne!` it fires on a program that calls one `::ROUTINE` twice, on one that calls a routine from inside an `INTERPRET`, and on a nested-`INTERPRET`-plus-loop shape.
* **The corpus is a weak witness for this, and I can put a number on how weak.** Inverted, only **2 of those 381 programs** and **7 of 1487 tests** reach the plan-cache hit path at all. The hand-written probes are what exercised it, and a sweep that never reaches the path proves nothing about it.
* Structurally, the two production `plan_for` callers take body, symbols and source out of one `Rc<Program>` reached through the id the key carries. But `::REQUIRES` is a Phase 5 gap and an external routine file a Phase 7 one, and **either is a route by which a second program could be loaded.** The tripwire is what will still be standing when one lands.

**I could not construct the failing case, and I am not claiming it cannot happen.**

**The other unreachability I will not claim.** T3 says the override outranking the table has one catcher, this task's own test. I instrumented `clause_line_at` to count clauses stepped with a plan in hand *and* the override in force, and ran it over the corpus, the bench programs and eight hand-written shapes including a routine called from inside an `INTERPRET` and an `INTERPRET` inside a routine inside an `INTERPRET`: **zero on every one, both engines.** So the state that test guards was not reached by anything I ran. That is not the same as unreachable, and the test stays.

### Step 5 -- the measurement

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, six interleaved rounds per arm, both arms staged at one fixed binary path, from a fresh empty directory, minimum of each arm.

| axis | base `9b2d416d3` | head `5b3be9536` | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `emptyloop` | 27,150,609,938 | 25,150,609,522 | -2,000,000,416 | **-7.366%** | 550 | 1,521 |
| `varlookup` | 44,802,636,286 | 42,408,615,813 | -2,394,020,473 | **-5.343%** | 1,296 | 647 |
| `rexxcps` | 25,259,112,106 | 24,296,505,166 | -962,606,940 | **-3.811%** | 11,497,720 | 4,212,461 |
| `alloc4c` | 8,183,701,509 | 7,882,695,240 | -301,006,269 | **-3.678%** | 55,696 | 85,333 |
| `compound` | 25,027,491,124 | 24,233,277,999 | -794,213,125 | **-3.173%** | 1,679,877 | 2,822,324 |
| `strings` | 44,913,587,939 | 43,950,588,042 | -962,999,897 | **-2.144%** | 1,317 | 1,005 |
| `arith` | 20,413,489,949 | 20,203,990,984 | -209,498,965 | **-1.026%** | 681 | 1,914 |

Every `rexxcps` run of both arms self-calibrated to `100 x 100`, checked on each arm's own `Averaged:` line.
Every arm pair is non-overlapping; the tightest ratio is `rexxcps` at 83.7 times its own wider span.
**No figure here is a bound.** Every axis executes the changed code, established by counting rather than by reading, and every axis moved past its own spread.

**Measured twice.** A comment-only correction after the first sitting changed the binary's bytes, and this project has measured layout alone moving these axes. The two sittings agree to three decimal places on six axes and to 0.004 percentage points on `compound`. The table above is the second, taken with the binary that was committed, `sha256` checked against `target/release/rexx-run` after the commit.

**The plan's own honest expectation was about 5.6% on `emptyloop`. It came in at 7.366%.**

#### The profile

`perf record -F 999`, `REXX_ENGINE=ir`, `samples/rexxcps.rex`, three runs per arm, one sitting, self time.

| | base | head |
|---|---|---|
| `Interp::run_ops::<false>` | 12.41 / 12.00 / 12.25% | 9.02 / 9.44 / 10.12% |
| `Interp::step_in_temps_frame` | 2.56 / 2.77 / 2.13% | 1.10 / 1.51 / 1.21% |

Both fall with non-overlapping ranges.
**`ProgramSource::line_of` appears as a symbol in neither arm.** It is inlined into both, so the sampled instrument could never have named this cost -- the phase document's "driver bucket, about 20%" had a per-clause binary search inside it, invisible to the tool that produced that share.

#### What a program observes

Base and head binaries compared byte for byte on stdout, stderr and exit status, **both engines**, over the corpus, `bench-programs/` and the oracle's `samples/` tree.
Identical everywhere except `samples/rexxcps.rex`, whose `Performance:` line is the clauses-per-second figure it exists to print.
Two infinitely recursive samples differed only in the PID inside `memcap`'s own kill message; filtered, identical at rc 137.
`tests/ir_dual.rs` and `tests/corpus.rs` green at head. **No divergence licence spent.**

### Step 6 -- the record

Entry 35, appended. `git diff --numstat` reads `136 0`, deletion count **0**.
Hunk header: `@@ -3223,0 +3224,136 @@ Entry 32's successor stands unchanged: \`PARSE\` targets, whose call site in \`pars`

The entry carries the contaminated-arm finding, and carries it one step further than the plan did -- see below.

## What the task found that the plan did not

**The plan's stated design fact is wrong, and it comes from the arm the plan itself disowns.**

The plan says, under "What survives, and it is the design fact this unit turns on": *the search depth is not the cost*, and *a faster search would be worthless*.
Dividing each axis's measured saving by its own count of removed searches:

| body lines | saved per removed search |
|---:|---:|
| 7 | 40.0 |
| 8 | 42.0 |
| 12 | 53.5 |
| 14 | 52.9 and 52.4 |
| 51 | 75.3 |
| 198 | 94.7 |

Monotone in the body's line count and more than doubling across the range. **The depth is most of the spread.**

The spike read otherwise by comparing 105.0 instructions per pass on `emptyloop` against 52.8 per clause on `rexxcps` -- and the `rexxcps` arm is the one the plan disowns in the paragraph above, because both probe arms return wrong line numbers that reach downstream consumers.
The plan disowned that arm's *table* and kept its *design conclusion*. The conclusion was the part built on it.

**The decision is unaffected and I did not change it.** A table is preferred to a faster search not because the search is shallow, but because a search that is not made costs neither its call nor its depth, which bounds anything a faster search could return.
The plan's finding is left standing in the plan file with a pointer to entry 35, rather than rewritten, because it is what the task was given.

**A second, smaller thing the table also buys, which the plan's framing hides.** Because the saving tracks depth, this change is worth *more* on long programs, not less. The plan's expectation of "about 5.6% on an axis that does almost nothing else per clause, and less everywhere that does more" is right about the shape and wrong about the mechanism: `rexxcps` does far more per clause than `emptyloop` and still gives up 94.7 instructions per removed search against `emptyloop`'s 40.0.

## Concerns

1. **No control axis exists for this unit and I did not build one.** Every axis executes the changed code, so entry 31's standing ask -- separate this change's semantics from drift -- cannot be discharged by an axis that sees none of it. What stands in for it is that the saving is monotone in the depth removed across seven axes, which drift has no reason to be. It is weaker than a control and I am not calling it one.

2. **The `BodyKey` answer is a negative from a thin witness.** 2 of 381 programs and 7 of 1487 tests reach the plan-cache hit path at all. The zero mismatches are real and the probe is live, but the sweep's power against this specific question is small, and the hand-written probes are doing the work. When `::REQUIRES` or an external routine file lands, this question is open again and the tripwire is the thing that will answer it.

3. **T3's guard has no reachable witness today.** The wiring test is the only catcher for the override outranking the table, and I could not reach the state it guards from any program. It guards a future arrangement.

4. **`Plan::lines` costs eight bytes per instruction, always, including for a body that never raises, traces or reads `SIGL`.** Nothing measures that here. `Plan::indents` already carries the same cost with the same justification, so this doubles an existing footprint rather than introducing a new kind of one -- but a body-size measurement for the pair does not exist and neither entry took one.

5. **`Interp::clause_line` survives beside `clause_line_at`.** Two accessors for one question, differing only in whether the caller holds an index. The remaining `clause_line` callers are on the trace and failure paths, where the search is cold, so collapsing them is a restructuring rather than a win -- but a future caller reaching for the wrong one gets the search silently.
