# Task 2 report: the interleaved two-interpreter harness, and the baseline

**Status: DONE_WITH_CONCERNS.** Fix rounds 1 and 2 complete.
The harness and the baseline are committed and every gate is green, including a clean-target
clippy run.
The concerns are findings, not incomplete work: the project's standard memory cap could not be
used, and this crate's memory behaviour on the benchmark axes is worse than any figure in the
brief anticipated. Neither is a defect in this task's output; both are things later tasks must be
told.

**Commits, in order, on `plan/rust-rewrite`:**

| commit | what |
|---|---|
| `107febcd87489ef9b097e57db0db3dd396393e93` | the harness, the timing core, the first baseline |
| `7735aac4b393852ac9bd9088697d027b6b9c1f63` | fix round 1: roles checked, whole oracle fingerprinted, `PROGRAMS` asserted, joint coverage stated |
| `2fae9a5b3b8c84b7396aa412cfce21b02b01731d` | the re-measured baseline, pasted whole |
| `9bcbfeda9e2cc0829ae5764b5cddb4bf02afcec9` | fix round 2: exemptions made checkable, Bonferroni bound clamped, emitted prose pinned |

**The live baseline is the one in `2fae9a5b`**, measured at `7735aac4`, which is the commit that
carries the harness that produced it. The figures in this report are that run's, and fix round 2
did not move them -- see below for why it did not need to.

## Fix round 1

Four Important findings and one Minor, all addressed. What each one changed, and how I checked it.

**Important 1 -- the "verbatim" claim was false.** The committed block said it was the harness's
output; four sentences had been edited and two paragraphs inserted into the Provenance range.
Rather than narrow the claim around hand edits, I committed the harness first and **re-ran the
suite**, then pasted the block whole. The boundary is now stated exactly ("every heading from
Provenance to Axes this crate cannot run ... is that program's output byte for byte; the prose
before and after is written here") and it is checked by `diff` against the harness output, not
asserted -- exit 0, no differences. That is the cleanest resolution for 4d-2, which diffs a fresh
run against this block: every remaining delta is a number.

The re-run moved the figures within their own intervals and changed no conclusion. `arith` 3.54x
to 3.52x, `compound` 12.00x to 12.15x, `strings` 13.84x to 13.70x, `varlookup` 23.19x to 23.07x,
`rexxcps` 9.33x to 9.26x. The oracle and this crate are the same binaries by hash as before, and
the three intervening commits from other tasks touched only the plan document -- verified,
`git diff --name-only 107febcd..ea6d5966 -- 'rust/**/*.rs' 'rust/**/*.toml'` is empty.

**Important 2 -- the pin asserted names, not roles.** Correct, and the failure mode is exactly as
described: when Phase 5 lands message sends the blocked axes exit 0 and would keep printing under
"Axes this crate cannot run" with status 0 and an empty message, timed by nothing. Fixed in two
places rather than one, because they catch it at different moments. `every_blocked_axis_still_fails_on_this_crate`
runs each blocked axis through this crate and fails if it succeeds; the suite itself does the same
check at run time and turns the run red, so the *report* cannot go green over it either. Both
mutation-checked, each with the mutation that actually reaches it:

* **The test.** Declare `startup` (which exits 0) `Role::Blocked`. Red, with
  `startup is declared Role::Blocked but exited 0`.
* **The suite.** Declare `arith` (which exits 0 and is a `Role::Loop` axis) `Role::Blocked`. The
  run exits 1, the report carries
  `**arith no longer fails on this crate and is still declared Role::Blocked.**` and a
  "This run is not a baseline" section, and stderr says `1 problem(s)`.

**The `startup` mutation does not reach the suite path**, and the first version of this report said
it did. `startup` is the only `Role::Offset` axis, so removing that role makes `main()` panic at
`.expect("the axis list has an offset program")` before `write_blocked` ever runs. Corrected in fix
round 2 after the reviewer caught it; the check itself was never in doubt, only the recipe, and a
wrong recipe is how the next person concludes a working check is broken.

The test needs a `rexx-run` binary. It takes release or debug, whichever exists, because the
property is profile-independent, and it **fails loudly** rather than skipping if neither is there.
Verified that `cargo test --workspace --no-run` does build `target/debug/rexx-run` -- by moving the
file aside, running it, and confirming the file came back byte-identical.

**Important 3 -- the `PROGRAMS` gap was prose.** Replaced with an assertion.
`rexx_bench::NOT_BENCHMARKED` now carries `heapshape` as a named exemption with its reason (it
prints two timings of its own and only one is comparable, which is why `d1-decision.md` runs it
directly), and `the_benchmark_list_accounts_for_every_program` asserts `PROGRAMS` plus the
exemptions against the directory. `PROGRAMS` itself is unchanged. The prose sentences in
`perf-baseline.md` are **deleted**, not corrected. Mutation-checked: emptying `NOT_BENCHMARKED`
makes the test fail.

**Important 4 -- the fingerprint was short.** Right, and my own argument for adding `librexx.so.4`
did apply to `librexxapi.so.4` verbatim. Rather than add the second name, the set is now **derived
from `ldd`** and filtered to the oracle build root, so an object the build gains later is covered
without anyone remembering it. A resolution that finds nothing is a hard failure that prints no
table at all, since the entire output of the phase is ratios against that build. The committed
Provenance now carries three hashes, and the section says why: the C++ tree holds an uncommitted
local patch, so these hashes are the only identity that build has.

**Minor -- the ratio interval's coverage was unstated.** The harness now prints, above the table,
that the interval is indicative, that its joint coverage is at least 92.2% by Bonferroni rather
than the 96.1% either side carries alone, and that the verdict is not taken from it. The per-side
figure is computed from the sample size rather than restated, so it cannot drift from `PAIRS`.

**Two corrections to this report itself**, both accepted. The "under 1% on three of the four axes"
claim was wrong at the time (0.96, 1.11, 3.40, 0.72 -- two, not three) and is now moot: on the
re-run this crate's spreads are 1.30, 1.04, 2.40, 1.02 and none is under 1%, so the sentence is
gone rather than corrected. The sizing cross-check is re-stated below against the coordinator's
dispatch as its source, since that is where those figures came from.

## Fix round 2

**The finding -- `NOT_BENCHMARKED` was a place to put a name.** Correct, and the reviewer's live
reproduction is the right way to have found it: my `Role::Blocked` fix asserts a behaviour and this
one asserted membership, so the two are siblings with the pattern missing from one.

**I did not take the suggested property, and the reason matters.** The suggestion was to assert
that every `NOT_BENCHMARKED` entry fails on this crate. It would have passed today and been the
wrong claim:

* `dispatch` and `alloc` exit 120 on this crate exactly as `heapshape` does, **and both are in
  `PROGRAMS`**. So "fails on this crate" does not separate the two lists at all -- it would have
  been satisfied for `heapshape` by coincidence, and satisfied equally by two names that belong in
  the other list.
* `PROGRAMS` is the criterion harness's list, and that harness runs whatever `REXX_BENCH_BINARY`
  names -- the C++ oracle for the committed baseline, where all eight programs run. Failing on
  *this crate* is not a fact about that harness at all.
* At Phase 5 it would have forced the wrong decision: `heapshape` starts running, the assertion
  goes red, and the obvious way to clear it is to benchmark it -- while its real reason for
  exemption still holds.

That is the "false justification riding a correct decision" shape: the fix would have been right
and its stated reason wrong, and the wrong reason is what a later reader acts on.

**The property I used instead is `heapshape`'s actual reason: it reports a timing of its own.**
Checked in both directions -- an exempt program that does not call `TIME()` is red, and a
benchmarked program that does is red too. It happens to be exactly discriminating on the current
tree: `heapshape` calls `TIME()` four times and no other benchmark program calls it once.
Appending a name cannot make it green.

**Mutation-checked in the direction that counts**, as asked. Dropping a dummy program into
`bench-programs/` turns the set-equality assertion red; **appending its name to `NOT_BENCHMARKED`
turns that one green and leaves `the_exemptions_are_true_of_the_programs_they_name` red**, naming
the dummy and saying `it does not call TIME()`. The other direction was checked too, and my first
attempt at it was a bad test that I caught and redid: emptying `NOT_BENCHMARKED` went red on the
`is_empty` guard before reaching the loop I meant to exercise, so it proved nothing about that
loop. Adding `heapshape` to `PROGRAMS` while leaving it exempt is the mutation that actually
reaches it, and it fails with `belongs in NOT_BENCHMARKED`.

**The `interval_coverage` degenerate case.** Fixed, and it is worse than "cosmetic garbage in an
obviously broken report": it fires in **`--self-check`**, which is a mode people run deliberately
and which exits 0. Every self-check run printed "joint coverage is at least -100.0%". The bound is
now clamped, the function takes the row rather than a slice so the empty case cannot reach it at
all, and `write_axes` says plainly when no axis produced samples.

**Fixing that would have broken Important 1, and I did not want to pay for it twice.** The clamp
changes a sentence *inside* the emitted block, so the committed block would have stopped being the
harness's output byte for byte. Re-running would have cost a third set of numbers and another pass
of recomputed prose percentages -- which is precisely where a correction round introduces false
statements. Instead the non-degenerate sentence is worded **exactly as committed** and the vacuous
case gets its own, so the block is untouched; verified, the `diff` against the committed block is
still empty. `the_caveat_matches_the_committed_baseline` now pins that agreement, so a future
wording change costs a re-paste rather than a silent divergence into 4d-2's drift diff. It is
mutation-checked: changing "at least" to "at least about" turns it red.

**The report recipe correction.** You were right and I verified it rather than taking it on trust.
With `startup` mutated, `main()` panics at `.expect("the axis list has an offset program")` before
`write_blocked` runs, because `startup` is the only `Role::Offset` axis. Mutating `arith` from
`Loop` to `Blocked` produces the claimed behaviour exactly: exit 1, the report carries
`**arith no longer fails on this crate and is still declared Role::Blocked.**` and a
"This run is not a baseline" section, and stderr says `1 problem(s)`. The recipe below is corrected;
the mechanism was never in doubt but the reproduction was, and a wrong recipe is how the next
person concludes a working check is broken.

## What was delivered

* `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`, the harness.
* `rust/crates/rexx-bench/src/timing.rs`, the timing core factored out of `rexx-time.rs`.
* `rust/crates/rexx-bench/src/bin/rexx-time.rs` rewired onto it.
* `docs/superpowers/plans/perf-baseline.md`, new dated section
  "Phase 4d-1 -- the interleaved two-interpreter baseline, measured 2026-08-08".

## Reuse: what was done with `rexx-time`, and why

**The timing core was factored into the bench crate's library and both binaries call it.**
Not "invoke `rexx-time` as a subprocess", for three reasons that each independently rule it out.
The suite has to alternate the two sides run by run, and `rexx-time` owns its own N-run loop, so
delegating to it would produce exactly the block-per-side layout the task forbids.
The suite needs each run's stdout -- `rexxcps` reports its result by printing it, and the
"same work on both sides" check needs the bytes -- and `rexx-time` sends both descriptors to
`/dev/null`.
The suite needs the raw samples to compute an interval, and `rexx-time` only prints a reduction.

`rexx-time`'s measured quantity is unchanged: `Summary::of` reproduces its `samples[len / 2]`
median exactly, and `Capture::Discard` reproduces its `Stdio::null()` on both output descriptors,
so the committed Phase 0 cold-start numbers remain numbers that binary still produces.
One behaviour did change: the shared core gives every child `/dev/null` on standard input where
`rexx-time` previously let it inherit. Nothing it times reads input, so this is a guard rather than
a change of quantity, and it is stated in the timing module's doc.

## What the baseline says

One line: **this crate is between 3.5x and 23.2x slower than the oracle across the four runnable
loop axes, the spread between those axes is real rather than measurement noise, and this crate uses
51x to 218x the oracle's memory.**

| axis | oracle | this crate | ratio | ratio interval |
|---|---:|---:|---:|---|
| `arith` | 1.1545 s | 4.0694 s | 3.52x | 3.49 - 3.56 |
| `compound` | 1.1440 s | 13.8938 s | 12.15x | 11.95 - 12.33 |
| `strings` | 0.8648 s | 11.8497 s | 13.70x | 13.54 - 13.92 |
| `varlookup` | 1.2140 s | 28.0086 s | 23.07x | 22.83 - 23.50 |
| `rexxcps` (internal cps) | 16,822,174 | 1,817,623 | 9.26x | 9.08 - 9.50 |

All four axes are SLOWER on Global Constraints' rule (this crate's median outside the oracle's
interval on the slow side). The ratio intervals are indicative only -- their joint coverage is at
least 92.2% by Bonferroni, not the 96.1% either side carries alone -- and the verdict does not use
them. Absolute throughput, the offset line, the per-side spreads and the blocked axes are in the
committed baseline section.

### The open hypothesis is settled, and refuted

The phase asked whether the wide per-axis ratio spread is a property of the *oracle's* variation
rather than this crate's. It is neither.
The four ratio intervals do not come close to overlapping each other -- they span 3.49 to 23.50 --
while the widest single-side run-to-run spread in the axes table is the oracle's 7.11% on
`varlookup`.
Per-workload difference dominates measurement variation by roughly two orders of magnitude, so
per-axis attribution is well founded. That is what Tasks 3 and 4 rest on.

Incidentally, the *direction* the hypothesis guessed is right about which side is noisier: the
oracle's run-to-run spread exceeds this crate's on all four axes (2.66 against 1.30, 2.29 against
1.04, 3.31 against 2.40, 7.11 against 1.02). It is just far too small to explain anything.

### `arith` against Phase 2's parity debt

`d1-decision.md`'s Phase 2 addendum recorded 1.22x and said in the same entry that it was a
**lower bound**, because it timed Rust arithmetic alone against a C++ figure that already included
parsing, dispatch and variable lookup, and that it would get worse once the Rust side paid those.
The scheduled Phase 4 re-measurement is **3.52x**, worse by a factor of 2.9.
The entry's own prediction is confirmed rather than contradicted, and `arith` remains the *best*
of the four axes in ratio terms.

## What I checked, not only what I found

The brief asked me to ask what would make each number wrong and check that thing.

* **Is the timing window measuring the workload, or the harness?**
  The per-process offset is measured with the identical wrapper (`/bin/sh` `exec` included) and is
  at most 0.84% of any axis (the oracle on `strings`) and 0.041% on this crate's side (`arith`).
  Reported as its own line, not smeared.
* **Did the workload actually run?**
  Every sampled run's stdout is compared against every other run on the same side, and the two
  sides against each other. All four axes: stable and identical. A run that died on its first
  clause is very fast, and wall time alone cannot tell that apart from a fast interpreter.
* **Is the interleaving real?**
  `the_two_sides_alternate_and_run_in_the_working_directory` reads back the order the children
  actually ran in, via two `Side`s differing only in an environment variable appending to a file.
  Mutated to a side-at-a-time loop, it fails with `left: "ooooRRRR"`, `right: "oRoRoRoR"` -- the
  exact shape it exists to exclude. Nothing in the report itself could have distinguished the two.
* **Can an axis silently drop out?**
  `AXES` is a literal asserted against `bench-programs/` at run time and by a test. Mutated by
  deleting the `heapshape` entry, the test fails (status 101, 1 failed).
* **Can an axis stay in the list under a role that has stopped being true?**
  It could, until fix round 1. Both the test and the suite now run the blocked axes and go red if
  one succeeds; both were mutation-checked (see "Fix round 1" above). This is the finding I would
  not have found on my own: the pin I wrote guards the list, and I read that as guarding the
  measurement.
* **Is the oracle the one I think it is?**
  Fingerprinted, and **not only `bin/rexx`**. That file is 60 KB of `main` and the interpreter is
  in the shared objects it loads; `ldd` resolves two under the build root and they carry different
  dates, so a hardcoded pair would have been a list that can fall behind. The set is derived from
  `ldd` and a resolution that finds nothing refuses to print a table. This matters for the
  end-of-4d-2 re-check, and more than usual because the C++ tree carries an uncommitted local
  patch, so these hashes are that build's only identity.
* **Is the Rust binary the one the pinned profile produces?**
  Forcing a rebuild of `rexx-exec` and its dependents reproduced
  `77f6680275f1ed4bdd68fab93bccb2568d3c2a440e0741ca53f2f97e361ae6ac` byte for byte. The tree had
  been warm and `cargo build` had reported "Finished in 0.03s", which is exactly the situation
  `rust/CLAUDE.md` says to distrust.
* **Do my numbers match the sizing figures I was given?**
  Yes, within spread: `varlookup` 28.01 vs 28.04, `compound` 13.89 vs 13.73, `strings` 11.85 vs
  11.95, `arith` 4.07 vs 4.11; oracle 1.214/1.144/0.865/1.155 vs 1.22/1.13/0.88/1.17.
  `rexxcps` 1,817,623 vs 1,822,252 and 16,822,174 vs 16,982,729.
  Those reference figures came from the coordinator's dispatch prose rather than from the brief or
  the plan, so this check is not reproducible from the committed record as it stands; the
  coordinator is putting them into the plan.
* **Was anything retyped?**
  No. The committed block is the harness's markdown pasted whole, and that is checked by `diff`
  against the harness output rather than asserted -- exit 0. Every derived figure in the
  surrounding prose (percentages, RSS multiples, the 6.6x ratio span, the 2.9x debt growth) was
  computed rather than eyeballed.
* **Were the mutations restored properly?**
  Every mutation was applied to a `cp` backup and restored from it, never `git checkout --`, and
  every restore was verified with `sha256sum -c` and a `diff`. Every mutation run was read for its
  run count as well as its status, because `cargo test <name>` exits 0 when it matches nothing.

## Concerns

### 1. The standard `ulimit -v 1048576` could not be used, and that is a finding

The brief states "The standard cap clears both". **It does not.** That claim holds for `say 1`,
which is what it was measured on; it does not hold for the workloads.

Measured on this tree, under `ulimit -v 1048576`, this crate aborts with SIGABRT and
`memory allocation of N bytes failed` on `varlookup`, `compound`, `strings`, `arith` and
`samples/rexxcps.rex`. It completes only `startup`. The oracle completes all six.

I applied `ulimit -v 8388608` (8 GiB) to **both sides on every axis**, verified before choosing it
that this clears every axis on both sides. That satisfies the brief's symmetry rule but deviates
from its literal number, and the deviation is stated in the report, in the constant's doc comment,
and here. Not "no cap": the cap exists so a runaway cannot take the machine's memory with it.

**If a later task re-measures under the 1 GiB cap it will get four aborts, not four numbers.**

### 2. This crate's memory use is the largest single divergence found

Peak resident set, `/usr/bin/time -v`, both sides:

| program | this crate | oracle | multiple |
|---|---:|---:|---:|
| `varlookup.rex` | 4,009,200 KB | 20,152 KB | 199x |
| `strings.rex` | 4,337,108 KB | 19,892 KB | 218x |
| `rexxcps.rex` | 2,686,360 KB | 20,736 KB | 130x |
| `arith.rex` | 1,072,888 KB | 20,868 KB | 51x |
| `compound.rex` | 1,056,848 KB | 20,180 KB | 52x |
| `startup.rex` | 2,864 KB | 8,380 KB | 0.34x |

This is resident memory, so it is **not** the `INTERPRETER_STACK_BYTES` reservation asymmetry -- a
reservation costs nothing in RSS, and four of these are two to eight times the size of that
reservation in RSS alone. `varlookup.rex` is
`do i = 1 to 19000000; x = x + 1; y = x; end`, whose live set is two integers, and it reaches 4 GB.

I did not investigate the cause, and I deliberately did not measure whether it scales with the
loop count -- that is an attribution question and attribution is Tasks 3 and 4. I flag it because
a 23x time ratio on an axis whose memory is 199x the oracle's may not be a *time* problem at all,
and a task that profiles instruction counts without knowing this could attribute the gap wrongly.
**Tasks 3 and 4 should be told this before they start.** It is not in their briefs.

### 3. One smaller note

**`rexxcps` moved without anyone changing it on purpose.** 9.26x here against 10.02x and 10.09x
measured 2026-08-06 in the section above it in the same file. The oracle barely moved; this crate
gained about 8%. Recorded as an observation in the baseline, not attributed. Worth a glance if a
later task expects the 10.0x figure.

The `rexx_bench::PROGRAMS` gap that stood here in the first version of this report is closed: the
set is asserted against the directory rather than described, `heapshape`'s exemption is a claim
about the program that is itself checked in both directions, and the prose about it is deleted.
`PROGRAMS` itself is unchanged.

## Constraints honoured

No optimisation was attempted or landed. The C++ tree was never written to; `rexxcps.rex` was run
from its own path and, where I needed to inspect it, copied to scratch. No `unsafe`. All probes ran
from fresh empty scratch subdirectories. stdout, stderr and exit status were read as three separate
descriptors throughout; no `2>&1` in any measurement. No cargo command was run from the repo root
or read from a pipeline. The three oracle-crashing programs were never run.

Gates, all read unpiped from `rust/`:

| gate | result |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --workspace --no-fail-fast` | 0 -- **1311 passed, 0 failed**, 4 ignored |

Baseline before this task was 1,289 passed. The 22 new tests are 10 in `timing`, 10 in the suite
binary and 2 in the `rexx-bench` lib; 1289 + 22 = 1311 exactly.

One count in the brief does not reconcile and is worth stating rather than papering over: the brief
gives the baseline as "75 binaries" and the first full run showed 77 lines matching `test result:`.
Of those 77, 68 were `Running` lines and 8 were `Doc-tests` lines, totalling 76 actual harnesses;
the 77th is a `test result:` line *echoed* by `corpus.rs`'s test that re-executes its own binary as
a child and prints the child's output. 76 = 75 + the one new `rexx-bench-suite` unit-test binary,
which is exactly what this task adds. The passed count reconciling to the unit is the stronger
evidence.

`rust/CLAUDE.md` treats a same-session clippy green on a warm target directory as provisional, so
it was re-run from a genuinely empty one at the end of each fix round:
`CARGO_TARGET_DIR=<empty> cargo clippy --offline --workspace --all-targets -- -D warnings`, exit 0,
61 `Compiling`/`Checking` lines both times, no `warning` or `error` line on stderr. The reviewer's
independent fresh-target run agrees.

The committed baseline block was re-diffed against the harness output after fix round 2 and is
still byte-identical -- that round deliberately left the emitted 9-pair wording untouched, and
`the_caveat_matches_the_committed_baseline` now holds it there.
