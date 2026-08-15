# Task 7-M2: deliver the itemisation, and price the two removals -- report

BASE `ced6c209`.
Commits, in order: `bec005fd` (the harness), `84dda087` (the counter guard the harness's own first sitting earned) and `af2014e4` (the size rule its second sitting earned).
Neither spike landed; both were built, tested, measured and reverted, and after each revert the rebuilt `rexx-run` was byte-identical to `ced6c209`'s by `sha256sum`.

(An earlier draft of that last sentence claimed the spikes had been measured while their sitting was still queued. It was written ahead of its own evidence, which is the defect this task exists to correct, and it is recorded here rather than silently fixed. It is true as of the sitting reported under steps 5 and 6.)

Suite **1401 -> 1411**, 0 failed, 4 ignored, each exit status read unpiped.
`cargo fmt --all --check` 0 and `cargo clippy --workspace --all-targets -- -D warnings` 0, both read unpiped.
The ten added tests are the harness's; both spike builds ran the same suite green at 1410 before the eleventh landed with the size rule.

This report is not one of the commits: `.superpowers` is in `.gitignore`, so the ledger is on disk and outside version control.

## The instrument, before any number taken with it

`rexx-arms`, built at step 0a and used for every figure below.
`rust/crates/rexx-bench/src/arms.rs` is the measurement and the reductions, `rust/crates/rexx-bench/src/bin/rexx-arms.rs` is the command line and the two output streams, and `rust/bench-baselines/` is the committed record.

**Five rules, as types rather than as a methodology section.**
Each is a defect this phase has shipped or nearly shipped while the rule was already written down in prose.

* **Both instruments.** `Reading` holds `instructions:u` and `cycles:u` together and has no single-counter constructor. The `perf stat` reply is refused whole when either event is missing.
* **One sitting.** `measure` takes the builds and the workload and runs every cell itself. `Sitting` cannot be constructed from outside the module, so there is no way to hand it two result sets.
* **One binary, both arms.** `Sitting::arm_ratio` takes one build's index and reads that build's own two `REXX_ENGINE` arms. A two-binary ratio is `across_builds`, a differently named method whose doc says what it is worth.
* **Two sizes.** `Workload::classify` derives the size from the program's own single `n = <integer>` line and runs it at that bound and twice it. A program with no such line becomes `Workload::Fixed`, whose sitting is a plain `Sitting` with no `per_pass` method on it at all -- so the one exception cannot produce a per-pass figure rather than producing a misleading one. A program with two such lines is refused.
* **No bare number.** `Figure`'s fields are private and both of its renderings carry the range and the round count.

**One further rule the events themselves needed.**
`Wrapper`'s `counters: bool` became `Counted`, whose variant names the two event strings; the same value builds the `-e` argument and parses the reply.
Asking for `cycles:u` and being answered by `cycles` differs by a few per cent on these axes rather than by an order of magnitude, so nothing downstream could have noticed.

## Step 0b: the negative control, and what it says about the floor

**The control.** A `black_box` spin -- eight `wrapping_add`s -- at the top of `run_region_ops`' `for op in ops` loop, in the driver only.
`varlookup` runs two promoted clauses of three ops each per pass, so the spin executes six times per pass there; `emptyloop` reaches no `Op::Clause` at all, so on that axis the spin **cannot execute**, which makes the same binary pair its own null.
Built, saved, reverted, `sha256sum -c` on the source and the rebuilt `rexx-run` byte-identical to `ced6c209`'s.

Five rounds, both arms, two sizes, one sitting, rotated.

| | head | control | difference |
|---|---:|---:|---:|
| `varlookup` IR arm, instructions per pass | 3905.00208 | 3967.00212 | **+62.00** |
| `varlookup` IR/TW, `instructions:u` | 1.02118 | 1.03740 | +0.01622 |
| `varlookup` IR/TW, `cycles:u`, large | 1.01113 | 1.04797 | +0.0368 |
| `varlookup` TW arm, instructions per pass | 3824.00210 | 3824.00208 | -0.00002 |
| `emptyloop` IR arm, instructions per pass | 1616.00203 | 1616.00204 | +0.00001 |
| `emptyloop` IR/TW, `instructions:u` | 1.06596 | 1.06596 | 0.00000 |
| `emptyloop` IR/TW, `cycles:u`, large | 1.04049 | 1.06484 | +0.0244 |

**The control passes on both halves.**
The introduced cost is read as +62.00 instructions per pass with a five-round range of 0.00006, and 62 over six spins is 10.3 instructions each, which is the right size for eight `wrapping_add`s behind a `black_box`.
It reaches only the arm and only the axis it can reach: the tree-walker arm moves by 2e-5 instructions per pass on `varlookup`, and the axis with no promoted clause moves by 1e-5.

**And the floor the brief asked me to size against is instrument-specific, which the brief's wording does not allow for.**
Task 8 recorded `+0.74%` on `emptyloop`'s IR arm between a committed build and a control that could not reach that axis, and the brief takes that as a bound below which a per-axis move "is not yet distinguishable".
On this pair the same quantity is **0.0000002%**: `emptyloop`'s IR arm reads 40,400,668,841 on head and 40,400,668,757 on the control, 84 instructions apart in 4.04e10.

* **On `instructions:u` there is no 0.75% floor.** Two builds that differ in the driver agree to 2e-9 on an axis the difference cannot reach. An instruction figure from this harness resolves far below 0.75%.
* **On `cycles:u` there is one, and it is larger than 0.74%.** The same two binaries, on the same axis, with instruction counts agreeing to 2e-9, read `emptyloop`'s IR arm cycle ratio 1.04049 against 1.06484 and its cross-build cycle ratio at **1.0235** and **1.0213** on the two sizes. So about **2.3%** of code-placement sensitivity, measured with the executed work held exactly constant, and in the same direction on both sizes.

This does not show Task 8's `+0.74%` was wrong: it is a difference between two *other* binaries and reproducing it needs those binaries.
What it shows is that the bound cannot be carried as a property of the instrument. See "What I could not settle" for the one reading of it I could not close.

**The instrument found a defect in itself on its first sitting, which is what a control is for.**
One round of `emptyloop`'s head IR cell read 41,550,668,546 against a median of 40,400,669,204 -- **+2.85%** on a quantity that is otherwise stable to eight significant figures.
The median absorbed it, and nothing in the reduced output said which run it was.
`perf stat` prints the percentage of the run an event was scheduled for and scales a short count *up*, printing the estimate in the same column and format as an exact count; that column is now read, and a short one refuses the whole reading (`84dda087`).
`--raw` now writes every individual run for the same reason.
Twelve repeats of that exact cell afterwards all read 100.00% enabled and landed within 1,500 instructions of each other, so the excursion did not recur and I cannot attribute it; the guard closes the mechanism that would produce exactly this shape whether or not it was this one.

## The builds this report measures, and why archaeology needed its own

Every binary below is `rexx-run`, release profile, built from the commit or the patched commit named.
The old commits were built in a detached worktree under the scratchpad; the two spikes were built in the working tree under prototype-publish-revert, with the source restored from a `cp` copy and checked with `sha256sum -c`, never `git checkout --`.

| label | what it is |
|---|---|
| `base` | `fd0ea6d1`, Task 7-M's base |
| `generic` | `fd0ea6d1` with `compile`'s `Assignment` arm guarded off, so an assignment emits `Op::Generic` -- 7-M's own control |
| `m7head` | `132c3395`, Task 7-M's head |
| `c1undone` | `132c3395` with `run_clause_region` taking `instruction` again alongside `index`, and the caller fetching it unconditionally -- change 1 undone, the slice and the ops-read-clause changes kept |
| `head` | `ced6c209`, the tree at hand-off |
| `control` | `head` with the step 0b spin |
| `spike-box` | `head` with `Failure::Raised(Box<Raised>)` |
| `spike-enterleave` | `head` with the clause wrapper split |

**One binary carries two commits, and it is worth saying because it removes a build.**
`132c3395` and `9da84dc3` differ only in `docs/`, and their `rexx-run` binaries have the same sha256.
Task 8's base binary and Task 7-M's head binary are therefore the *same* binary, so step 3's head column and step 8's base column are one measurement rather than two.

## Steps 1 and 4: the per-promoted-clause figures, rebuilt

Five builds, `varlookup`, both arms, both instruments, two sizes, five rounds, one sitting, rotated so no build and no arm keeps a slot.
`varlookup` runs one body-range entry and two promoted assignment clauses per pass, which is the decomposition 7-M established and this sitting reproduces.

| build | IR-minus-TW per pass | per promoted clause | IR/TW `instructions:u` | IR/TW `cycles:u` (large) |
|---|---:|---:|---:|---:|
| `base` `fd0ea6d1` | 277.00001 | 100 | 1.07251 | 1.07765 |
| `generic` control | 121.00003 | 22 | 1.03168 | 1.00979 |
| `c1undone` | 271.00000 | 97 | 1.07094 | 1.09725 |
| `m7head` `132c3395` | 238.99998 | 81 | 1.06256 | 1.03873 |
| `head` `ced6c209` | 81.00002 | 40.5 | 1.02118 | 1.01443 |

Every per-pass figure has a five-round range under 0.0001 instructions.
The per-promoted-clause column is `(per pass - 77) / 2` with 77 the per-entry cost 7-M solved for; on `head` the two clauses are no longer alike, since Task 8 promoted the read in one of them, so its cell is the pair's mean and is marked as such rather than quoted as a per-clause constant.

**7-M's own figures reproduce exactly.**
277, 121 and 239 per pass, and 1.07251 and 1.06256 as ratios, are 7-M's numbers to the digit, taken on a different day by a different instrument-of-record.
That is the strongest thing this sitting says about 7-M: its measurements were right.

**Change 1 is worth 16, and the review's arithmetic predicted it (I3).**
`c1undone` is `132c3395` with `run_clause_region` taking `instruction` again alongside `index` and the caller fetching it unconditionally -- change 1 undone, the slice and the ops-read-clause changes kept.
It reads **97** per promoted clause against head's **81**, so change 1 is worth **16**.
The report used **10**, which is `100 - 90` against a row its own table labels "(base + nothing else)"; the review predicted "about 97 ... about 16" from the exploration table, and the build agrees.

**And the three changes are not additive, which the report claims they are.**
7-M writes "The three marginals total 20 against the 19 actually removed, so they are additive to within one instruction."
With change 1 measured rather than inferred the three marginals are **16 + 8 + 2 = 26** against the **19** actually removed, so they overlap by about 7.
The mechanism is 7-M's own "argument-count wall" finding read one step further: with the instruction passed down as an argument it is already in a register, so the region's ops reading the region's clause saves much less than it does once that argument is gone.
The same sitting shows it from the other side -- `base` at 100 against `c1undone` at 97 says the slice and the ops-read-clause changes are worth **3** in the presence of the old calling convention, where against head they are worth 10.
Both decompositions are correct and they attribute the same 19 instructions very differently; the report presents only one and calls it additive.

**A fourth build shows the instructions-fall-cycles-rise signature again, and this one is not small.**
`c1undone` executes **fewer** instructions than `base` -- 155.4587e9 against 155.6867e9 on the IR arm at the large size, 6 fewer per pass -- and its cycle ratio is **worse**: 1.09725 against `base`'s 1.07765, about two points.
That is the shape `Driving` was rejected for and the shape `emptyloop` shows at head, on a third pair, with the direction of the instruction change known.
It is recorded here because three instances make it a property of this tree rather than an anomaly of one variant, and because it is the reason a cycle figure in this phase needs its arm-paired form.

**On the tree-walker arm, `base`, `generic`, `m7head` and `c1undone` are indistinguishable and `head` is not.**
At the large size their tree-walker arms read 145,160,693,849 / 145,160,693,310 / 145,160,693,751 / 145,160,694,116 -- a spread of 806 instructions in 1.45e11, 6e-9.
`head` reads 145,312,693,672, which is +4 instructions per pass and is Task 8's sharing-rule cost rather than noise.

### I9: "byte-identical" is wrong, and the true claim is stronger than the report needs

7-M writes "The tree-walker arm is byte-identical to base on six of the seven axes", and infers byte-identity from equal instruction counts.
Equal counts do not imply identical bytes, and the counts are not equal either: on `varlookup` at the large size `base` and `m7head` read 145,160,693,849 against 145,160,693,751, 98 instructions apart.

The claim that is both true and sufficient is the weaker one:

> The tree-walker arm's executed instruction count is unchanged to within a constant far below one instruction per pass -- 98 in 1.45e11 on `varlookup`, 7e-10.

Byte-identity was never needed for the argument it was carrying, and 7-M's own breakdown row "`run_ops`' loop head and prologue, **redistributed by codegen**" is an admission that the codegen moves.

## Step 6: the enter/leave split -- what was built, and whether it keeps one implementation

The numbers are below, under "Steps 5 and 6: the two spike numbers"; this section is the shape and the sharing-rule verdict.

**The shape.** `Interp::in_clause` and `Interp::in_stepped_clause_with` are each rewritten as three lines:

```rust
self.enter_clause(line);
let ran = body(self);
self.leave_clause(code, ran)
```

and

```rust
let entry = self.enter_stepped_clause(echo, code, index, instruction, source);
let ran = work(self);
self.leave_stepped_clause(entry, code, index, instruction, source, ran)
```

Everything the closure form used to do sits in the two halves, in the same order, with `SteppedClause` carrying the temps frame and the watermark across.
`run_clause_region` is then deleted outright and its body opened into `run_ops`' `Op::Clause` arm, which now calls `enter_stepped_clause`, runs the region's ops, and calls `leave_stepped_clause`.

**The sharing rule holds, and it holds by construction rather than by inspection.**
The tree-walker reaches the clause unit through `in_stepped_clause_with`, which is *defined in terms of* the same `enter`/`leave` pair the driver calls.
There is one implementation of the clause boundary and two entry shapes into it, which is the distinction the review drew and 7-M's report collapses: "one closure" and "one implementation" are not the same thing.
If the two had been separate copies that merely agree, this spike would have answered a different question and the number below would be worthless -- so it is worth saying exactly where the single copy is: `enter_stepped_clause` and `leave_stepped_clause` in `run.rs`, and `enter_clause` and `leave_clause` in `clause.rs`, each with one body.

**The hard part the brief names does not arise, and that is a finding rather than luck.**
The brief expects "the failure path has to run the leave from the driver's own `?` site".
It does not, because the clause's `Result` is passed *into* `leave_stepped_clause` as a value rather than taken by a `?` before it -- which is exactly what the closure form already did: `in_clause`'s epilogue begins `let Ok(value) = &ran else { return Ok(ClauseOutcome::Ran(ran)) }`.
A failure travels into the leave as data on both shapes, so there is no leave to remember on an error path and no second place that has to know about failures.
The `?` in the driver is on the *leave's own* result, which is the boundary's failure and not the clause's -- the same distinction `ClauseOutcome` already exists to make.

**What the compiler did with it.** `run_clause_region` is gone as a symbol and `run_region_ops` no longer exists as one either: the whole region collapsed into `run_ops`.
`run_ops::<false>` grows from 5,302 to 11,190 bytes, against 5,302 + 7,750 = 13,052 for the pair it replaces.
Suite green, 1410 passed, 0 failed, 4 ignored.

## Step 5: boxing the `Raised` variant -- what was built

The numbers are below, under "Steps 5 and 6: the two spike numbers".

`Failure::Raised(Raised)` becomes `Failure::Raised(Box<Raised>)`, and `From<Raised> for Failure` boxes.
The whole crate needed **two** further edits, both in `run.rs`: one `raised.into()` that becomes `Failure::Raised(raised)` because `raised` is already a box, and one `ActiveCondition { raised }` that becomes `raised: *raised`.
Every other site -- 120 of them, almost all `let Failure::Raised(x) = failure else` in tests -- compiles unchanged, because a `Box<Raised>` derefs to a `Raised` for field access.

The sizes are what the report and the review both predicted, confirmed here by a compile-time probe rather than taken on trust:

| | at `ced6c209` | boxed |
|---|---:|---:|
| `Raised` | 104 | 104 |
| `Failure` | 104 | **24** |
| `Result<ClauseRegion, Failure>` | 104 | **32** |

Suite green, 1410 passed, 0 failed, 4 ignored.

**It changes nothing a promotion emits and introduces no second implementation of anything**, which is the part of the review's I4 that does not depend on the number: whatever the measurement says, the `sret` half of the call boundary is *removable*, so classifying it as inherent was wrong independently of whether removing it is worth it.

## The correction to the plan, landed at `35417b23`

The plan's step 0b said a per-axis move under about 0.75% "is not yet distinguishable from this effect", treating Task 8's `+0.74%` as a property of the instrument.
This task's negative control splits it by instrument, and the controller applied the replacement verbatim at **`35417b23`**.
That commit is the text; it is not restated here, because restating a measurement is authorship rather than quotation and this report has already caught itself doing that once.

The two readings behind it, for the record: `head` against `control` differ in `run_region_ops`, which `emptyloop` never enters, and on that axis they read a cross-build ratio of **1.000000** on `instructions:u` and **1.0235**/**1.0213** on `cycles:u`.

## What this task did not touch

* **Nothing a promotion emits.** `compile` is unchanged at `ced6c209`, and neither spike landed.
* **No `unsafe`**, no em-dashes in added comments, no `git checkout --` anywhere: every revert was `cp` from a copy, verified with `sha256sum -c` over a 145-file manifest of `rust/crates`, and followed by a rebuild whose `rexx-run` matched `ced6c209`'s by sha256.
* **Wall clock is not used anywhere in this report.**

## The instrument's real precision limit, which is not the one anybody has been quoting

**Instruction counts on these axes are not deterministic between processes, and the variation is discrete.**
Every excursion this task saw is an exact whole number of instructions **per loop pass**, never a fraction:

| cell | excursions above the cell's own floor, per pass |
|---|---|
| `arith` `base` TW small | 0, 0, 0, 0, **+46.0** |
| `arith` `base` TW large | 0, 0, 0, 0, **+46.0** |
| `arith` `m7head` IR small | 0, 0, 0, 0, **+80.0** |
| `arith` `m7head` IR large | 0, 0, 0, 0, **+12.0** |
| `emptyloop` `head` IR small (step 0b sitting) | 0, 0, 0, 0, **+46.0** |

A counter artifact does not produce whole numbers per pass and does not produce the *same* whole number on two different lengths of the same program.
A per-process decision that changes how many instructions each pass executes does exactly that, and it stays fixed for the life of the process.

**This corrects two earlier statements, one of them mine.**

* Mine: I first attributed the `emptyloop` excursion to `perf stat` scaling a short-scheduled counter. That guard is still right to have -- scaling is real, silent, and printed identically to an exact count -- but it is **not** what produced this excursion, because scaling would not land on 46.0 instructions per pass.
* Task 8's: "Instruction counts are deterministic: the seven-round spread is +/-0.00% on every axis but `compound` (+/-0.002%) and `startup` (+/-0.08%)". They are deterministic *within* a process and vary between processes in discrete steps of tens of instructions per pass -- 0.075% on `arith`, 2.85% on `emptyloop`, whose per-pass cost is smallest and therefore most exposed.
* And 7-M's: "A sample was seen to come back low by ~0.7%, which on a differenced quantity read as **23 instructions per pass**". That is a whole number per pass as well, so it is this phenomenon rather than the sampling error 7-M took it for. Its remedy -- median of three, re-take the affected rounds -- was right for the wrong reason.

**What follows for every figure in this phase.**
A median over rounds is not a refinement here, it is load-bearing: one run in five carries a whole-number-per-pass offset, and a mean would fold it in rather than reject it.
`compound`'s "not deterministic to better than about 5e-6" is the same effect seen on the axis where 7-M happened to look; it is 150 times larger on `arith` and it is not confined to stem lookup or to one arm.

## The harness's second self-caught defect, and the size rule it changed

The first run of the six-axis sitting **died** on `strings`: doubled to n = 6,000,000 it asks for 6,442,450,944 bytes and `ADDRESS_SPACE_LIMIT_KIB` kills it.

Doubling was the wrong rule and the failure is the smaller half of why.
On the allocation-shaped axes a doubled program is a *different* program rather than a longer one -- `alloc4c` builds a table of twice as many live tails, `strings` holds twice as much -- so the difference between the two lengths stops being "the cost of n more passes".
And the committed bound is the length every published figure in this phase was taken at, so a rule that never measures it makes every whole-program ratio comparable only with a doubled program nobody else ran.

The two lengths are now the committed bound and **half** of it (`af2014e4`).
`bound - bound / 2` is the same number of passes either way, so the per-pass reduction is untouched; what changes is that the `large` cell is now the axis as committed.
`the_benchmark_programs_all_classify` asserts that direction, so a future rule that stops measuring the committed bound is a red test rather than a quiet incomparability.

## Step 3: the five missing cycle readings, and what they do to the headline

Three builds -- `base` `fd0ea6d1`, `m7head` `132c3395`, `head` `ced6c209` -- both arms, both instruments, two lengths, five rounds, one sitting, rotated.
Ratios are IR over tree-walker within each build, at the axis's committed bound.

| axis | base instr | m7head instr | base cycles | m7head cycles |
|---|---:|---:|---:|---:|
| `arith` | 1.01018 | **1.00862** | 1.02276 [1.01878..1.03053] | **1.02078** [1.01632..1.02287] |
| `compound` | 1.01791 | **1.01546** | 1.02072 [0.99503..1.02825] | **1.05057** [1.04300..1.05409] |
| `strings` | 1.02032 | **1.01697** | 1.03514 [1.01322..1.04319] | **1.02799** [1.01301..1.04391] |
| `alloc4c` | 1.02359 | **1.02002** | 1.02194 [0.99694..1.03013] | **1.01845** [1.01241..1.02863] |
| `emptyloop` | 1.06539 | 1.06539 | 1.00125 [0.99879..1.00151] | 1.00400 [1.00075..1.00793] |
| `startup` | 1.04367 | 1.04435 | 0.97937 [0.87437..1.52145] | 1.04316 [0.86838..1.14856] |

**Every instruction figure of 7-M's reproduces**, to five or six significant figures, from a different instrument on a different day: 1.01017/1.00861, 1.01793/1.01546, 1.02032/1.01698, 1.02359/1.02002, 1.06539/1.06539.

**And the headline does not survive the second instrument.**
The review's stated refutation was "if any of the five has a head cycle ratio above its base one".
`compound` does: **1.02072 at base against 1.05057 at head**, three points worse, and the two five-round ranges are **disjoint** -- `m7head`'s whole range [1.04300..1.05409] sits above `base`'s median and above the top of `base`'s range.
Its instruction ratio improves over the same pair, 1.01791 to 1.01546.

So `compound` shows exactly the instructions-fall-cycles-rise signature 7-M rejected the `Driving` variant for, at a fifth of `Driving`'s size and thirty times the `emptyloop` move 7-M did report -- on an axis 7-M reported on instructions alone.
**"The IR arm improves on six axes" is not supportable as stated.**
What the six axes support is the narrower claim, which is still worth having:

> On `instructions:u` the IR arm improves on six axes. On `cycles:u` three of the five previously unreported axes improve within overlapping ranges, `emptyloop` is unmoved, and `compound` regresses by three points with disjoint ranges.

`startup` is not a reading on either instrument: its cycle ranges span [0.87..1.52] and [0.87..1.15], which is half a million instructions dominated by dynamic loading.
It is reported here so that the row exists, not because it decides anything.

### I8: `emptyloop`'s cycle rise is real and it is a tenth of what 7-M reported

7-M gives `emptyloop`'s cycle ratio as 0.9990 at base and 1.0216 at head, a rise of 2.3 points, and reads it as evidence that the criterion is unsound.
Measured here at the same two commits it is **1.00125 to 1.00400** -- a rise of 0.3 points, in the same direction, with barely disjoint ranges.

Both readings are of the same pair of binaries, and the difference between 2.3 points and 0.3 points is the size of the cross-build cycle artifact this task's own control puts at about 2.3% on this very axis.
The review's finding stands -- the change does show the signature it rejected `Driving` for, and 7-M states only the reading that favours it -- but the magnitude 7-M attaches to it is mostly instrument.
That cuts both ways and both are worth recording: the movement is real and much smaller than reported, and `emptyloop`'s cycle ratio is the least reproducible number in this phase.

## Step 8: Task 8 re-measured, and where it does not reproduce

`9da84dc3` and `132c3395` build the same `rexx-run`, so Task 8's base column and 7-M's head column are one binary and the comparison below came out of the sittings above rather than needing a third.
`varlookup`'s row is from the five-build sitting, the rest from the six-axis one.

**`instructions:u` -- every figure reproduces.**

| axis | Task 8 base | measured | Task 8 head | measured |
|---|---:|---:|---:|---:|
| `varlookup` | 1.06256 | **1.06256** | 1.02118 | **1.02118** |
| `alloc4c` | 1.02002 | **1.02002** | 1.01026 | **1.01026** |
| `emptyloop` | 1.06539 | **1.06539** | 1.06596 | **1.06596** |
| `arith` | 1.00862 | **1.00862** | 1.00882 | **1.00882** |
| `compound` | 1.01547 | 1.01546 | 1.01570 | 1.01569 |
| `strings` | 1.01698 | 1.01697 | 1.01733 | **1.01733** |
| `startup` | 1.04389 | 1.04435 | 1.04300 | 1.04496 |

Six of seven agree to the last digit or one off it, on an independent instrument.
`startup` disagrees by 0.002, which is inside its own instruction spread of [1.04126..1.04584] and far inside its cycle spread; it is not a reproduction failure, it is an axis that does not resolve.

Task 8's headline pair -- **`varlookup` 1.02118 and `alloc4c` 1.01026** -- reproduce exactly.
Its per-pass claim reproduces too: the IR-minus-TW gap is 81 on `varlookup` (this task's sitting B) and **164.000** on `alloc4c` against `m7head`'s **320.089**, a saving of 156 per pass where Task 8 measured 154.
The two-instruction difference is inside what a per-pass figure taken on a different pair of lengths can be expected to hold.

**`cycles:u` -- two reproduce, two do not, and the pattern is informative.**

| axis | Task 8 base | measured | Task 8 head | measured |
|---|---:|---:|---:|---:|
| `arith` | 1.02337 | 1.02078 | 1.01314 | 1.01318 |
| `compound` | 1.04612 | 1.05057 | 1.02908 | 1.02014 |
| `alloc4c` | 1.01410 | 1.01845 | 1.03385 | 1.02029 |
| `strings` | 1.01068 | 1.02799 | 1.02842 | 1.02886 |
| `emptyloop` | 1.02752 | 1.00400 | 1.02854 | 1.04106 |
| `varlookup` | 1.04239 | 1.03873 | 1.01685 | 1.01443 |

`arith` and `compound` land within a point and keep their direction.
`emptyloop` does not reproduce at all: Task 8 reads the change as +0.001 and this sitting reads it as +0.037.
That is the same axis whose cycle ratio this task's own control moved 2.3 points with the executed instruction count held constant to 2e-9, and the same axis Task 7 read at 1.0526 and 0.9887 in two sittings of one session.

**The finding this step was set up to produce.**
A hand-measured figure the harness cannot reproduce is a finding about one of the two, and here it is a finding about neither and about the axis: **`emptyloop`'s cycle ratio is not a reproducible quantity in this tree**, on any instrument, at a resolution finer than about 4 points, and three independent sessions now say so.
Every instruction figure Task 8 reported reproduces; every cycle figure it reported on an axis with real work in it reproduces to about a point; the one that does not is the axis with the least work per clause and therefore the highest proportion of driver in it.

## Steps 5 and 6: the two spike numbers

Three builds -- `head` `ced6c209`, `box`, `enterleave` -- both arms, both instruments, two lengths, five rounds, one sitting, rotated.
Ratios are IR over tree-walker **within each build**; the per-pass columns are that build's own two arms differenced.

| axis | | `head` | `box` | `enterleave` |
|---|---|---:|---:|---:|
| `varlookup` | IR/TW `instructions:u` | 1.02118 | 1.02728 | **0.98143** |
| | IR/TW `cycles:u` | 1.00890 | **1.16911** | **0.96988** |
| | IR-minus-TW per pass | +81.00 | +101.00 | **-71.00** |
| `alloc4c` | IR/TW `instructions:u` | 1.01026 | 1.01189 | **0.99581** |
| | IR/TW `cycles:u` | 1.02882 | 1.03984 | 1.02208 |
| | IR-minus-TW per pass | +164.02 | +209.74 | **-66.99** |
| `emptyloop` | IR/TW `instructions:u` | 1.06596 | 1.06842 | **1.08185** |
| | IR/TW `cycles:u` | 1.03842 | 1.08321 | 1.04524 |
| | IR-minus-TW per pass | +100.00 | +99.00 | **+124.00** |

### Step 5: boxing `Raised` improves absolute speed on both arms and makes criterion 4 worse on every axis and both instruments

**Absolute, per pass, and both arms gain:**

| axis | arm | `head` | `box` |
|---|---|---:|---:|
| `varlookup` | TW | 3832.00 | **3711.00** |
| `varlookup` | IR | 3913.00 | **3812.00** |
| `emptyloop` | TW | 1516.00 | **1447.00** |
| `emptyloop` | IR | 1616.00 | **1546.00** |

About 70 instructions per pass off `emptyloop` on **both** arms and 121/101 off `varlookup`, which is a real and cheap win in absolute terms -- a `Result` that fits in registers rather than going through memory, on every `?` in the crate.

**And the ratio moves the wrong way on all six cells**, because the tree-walker arm gains more than the compiled one: `varlookup` 1.02118 to 1.02728 on instructions and **1.00890 to 1.16911 on cycles**, `alloc4c` and `emptyloop` both worse on both instruments.
The per-pass gap widens from +81 to +101 on `varlookup` and from +164 to +210 on `alloc4c`.

**This settles I4, and it settles it in 7-M's favour on the half the review left open.**
7-M gave two reasons that point opposite ways and measured neither.
The one that is right is "it is a cost on the path **both** arms take, so it improves absolute speed and *worsens* criterion 4, whose denominator falls with it".
The one that is wrong is "the promoted path carries one more such return per clause than the unpromoted one" -- if that dominated, the ratio would improve, and it does not on any axis.

**What the review was right about stands, and it is the part that matters for the itemisation.**
Boxing changes nothing a promotion emits and needs no second implementation of anything, so the `sret` half of the call boundary is **removable** and describing it as *inherent* was wrong.
It is a cost this phase declines to remove because removing it makes the phase's own gate worse -- which is a different sentence from "it is the shape", and the difference is exactly what criterion 4's itemisation is for.

### Step 6: the enter/leave split removes 152 instructions per pass on `varlookup`, and takes it below 1.0

**On the two axes with promoted clauses it is not a small effect.**
`varlookup`'s IR-minus-tree-walker gap goes from **+81 to -71** instructions per pass, a swing of **152**, and its IR/TW ratio goes **1.02118 to 0.98143** on instructions and **1.00890 to 0.96988** on cycles.
`alloc4c` goes from **+164 to -67**, a swing of 231, and **1.01026 to 0.99581**.
Both instruments move the same way on both axes, which is what step 0b's control was built to make readable.

**Criterion 4 is `IR/TW <= 1.0`. On this spike `varlookup` and `alloc4c` both pass it, on both instruments.**
Neither has ever passed it before in this phase.

**And `emptyloop` pays for it.** Its gap goes from +100 to +124 per pass and its ratio from 1.06596 to 1.08185.
The mechanism is visible in the symbol table: with `run_clause_region` gone, `run_ops::<false>` grows from 5,302 bytes to 11,190, and the `Op::Generic` arm -- which is all `emptyloop` executes -- now sits inside a much larger function.
So the split trades the clause-dispatch floor for the promoted-clause cost, which is the same trade the report's "driver's own op loop over a `stop`-bounded slice" row records at a twentieth the size.

**The trade is nothing like the one 7-M measured, and that is the finding.**
7-M's verdict is that the call boundary and the second dispatch level "are the shape", worth about half the +78, removable only by a second implementation of the clause wrapper.
This spike removes the call boundary while keeping **one** implementation -- `in_stepped_clause_with` and `in_clause` are both *defined in terms of* the enter/leave pair the driver calls -- and it removes 152 instructions per `varlookup` pass, which is about twice what 7-M's whole task removed.

## Steps 1 and 2: the breakdown reconciled

`valgrind --tool=callgrind --cache-sim=no --branch-sim=no --compress-strings=no --compress-pos=no`, IR arm, `base` `fd0ea6d1` against its `generic` control, over the `varlookup` body at n = 1,000,000 and n = 3,000,000, differenced between the two lengths and divided by the two promoted clauses a pass runs.
Callgrind is a simulator, so these are exact rather than sampled, and the per-process variation that moves `perf` counts does not reach them.

**The by-function table reproduces to the instruction, and the total is 78.**

| function | as `Generic` | promoted | delta |
|---|---:|---:|---:|
| `run_clause_region` | 0 | 344 | **+344** |
| `step_in_temps_frame` | 237 | 0 | **-237** |
| `assign_evaluated` | 48 | 0 | **-48** |
| `run_ops::<false>` | -- | -- | **+19** |
| **total** | | | **+78.00** |

7-M's own by-function figures are +344, -237, -48 and 47 against 66. Every delta is identical.
(`run_ops`' absolute cells are 78 and 97 here against 7-M's 47 and 66, a constant 31 apart, because this slice charges `run_ops` for the loop's own header work as well; the delta, which is what the table is for, is +19 either way.)

### Step 1: the row that carries the two instructions is `stale`

Per-line, inside `run_ops`' own `drive.rs` cost, per promoted clause:

| `drive.rs` line at `fd0ea6d1` | source | 7-M's row | measured |
|---|---|---:|---:|
| 405 + 409 | `match self.run_clause_region(` and its argument setup | +20 | **+20** |
| 404 | `let stale = chunk.trace() != self.chunk_trace();` | +7 | **+3** |
| 355 | the `Op::Clause` arm's own payload read | +3 | **+1** |
| 349 + 342 | less the `Op::Generic` arm's call and payload read | -6 | **-6** |
| 290, 300, 323, 324, 326, 329, 523 | the loop head and prologue | -3 | **0** |
| 0 | unattributed | -- | +1 |
| **total** | | **+21** | **+19** |

**The two rows anchored to a call site reproduce exactly**: the nine-argument call and its `sret` at +20, and the `Op::Generic` arm it replaces at -6.
The error is in the three rows that are not anchored to one, and the largest single discrepancy is **`stale`, stated at +7 and measured at +3**.

**7-M knew `stale` was a range and the itemisation quotes the top of it.**
Its own residual bullet says "`stale`, at **4 to 7** per clause across the builds measured".
A range put into a column that is then summed and asserted to total 78 is where the two instructions came from.

The other two unanchored rows partly cancel it: the payload-read row is 2 high, and the "-3, loud head and prologue, **redistributed by codegen**" row describes a redistribution that **did not happen** -- those seven lines cost the same in both builds, to the instruction.
`-4 -2 +3 +1 = -2`, which is the whole of it.

**And that -3 row has a much better home.** `run_ops`' own library-attributed delta is exactly **-3** per promoted clause (39.5 against 36.5). The -3 is the library plumbing slice of `run_ops`, mislabelled as codegen redistribution.

### Step 2: the +26 library row belongs in the residual, and it is not separately reducible

**It is not an eleventh item. It is the same 78 sliced along a different axis, and 7-M's own arithmetic says so.**
Rows 6, 7, 8 and 10 -- the ones describing `run_clause_region`, the region loop, the op bodies and the shared clause unit -- sum to +33, and +33 plus the +26 library row is **+59**, which is exactly the by-function delta for those three functions (`344 - 237 - 48`).
So the library row is the library-attributed *remainder* of the same three functions the other rows slice by code region, not a tenth thing beside them.
Measured here that remainder is **+17** rather than +26 (152.5 against 135.5 per promoted clause), the difference being where the crate/library line is drawn -- 7-M evidently excluded `rexx-parse` and `rexx-core` code that this slice counts as crate.

The library cost sits **inside** each function rather than beside it, which is the whole point:

| function | crate-attributed | library-attributed | total |
|---|---:|---:|---:|
| `run_clause_region` (promoted) | 228 | 116 | 344 |
| `step_in_temps_frame` (`Generic`) | 150 | 87 | 237 |

`core/src/slice/mod.rs` and `core/src/slice/index.rs` are 65 of `run_clause_region`'s 116 and 56 of `step_in_temps_frame`'s 87 -- bounds-checked indexing, exactly as 7-M describes it -- with `core/src/result.rs` and `core/src/option.rs` carrying most of the rest.

**Where it goes, and the reason it was not reduced.**
It goes in the **residual**, and the reason is now demonstrable rather than arguable: *there is nothing here to reduce on its own*.
Every instruction in it is the plumbing of a crate-level operation that some other row already names -- a bounds check on an op fetch, an `Option` on an instruction lookup, a `Result` through a `?`.
Reducing the library row means removing the crate-level work it plumbs, which is what the landed 19 did (a bounds-checked fetch removed took its bounds check with it) and what step 6's spike does at eight times the scale.
**A task that set out to reduce "the +26" as an item would be looking for something that does not exist**, and that is the correction the review's I5 is entitled to: not that the row is irreducible, but that it is not a row.

## The corrected itemisation, which criterion 4 asks for

Per promoted assignment clause at `fd0ea6d1`, the `+78` a promotion cost over the same clause run as `Op::Generic`, reconciled to its own total and sliced two ways because one slice cannot carry it.

**By code region** -- what the promoted path does that the unpromoted one does not:

| item | instructions |
|---|---:|
| the nine-argument call to `run_clause_region` and its `sret` return | +20 |
| `run_clause_region`'s prologue, epilogue and `ClauseOutcome` -> `RegionEnd` -> `ClauseRegion` mapping, against `step_in_temps_frame`'s same two | +19 |
| the region loop -- `pc < end`, the op fetch, the second `match op` -- against `step`'s own `mem::take` and single `InstructionKind` match | +19 |
| `stale`, i.e. `chunk.trace() != self.chunk_trace()` | +3 |
| the `Op::Clause` arm's own payload reads and unattributed remainder | +2 |
| the shared clause unit -- `in_stepped_clause_with`, `in_clause`, the printed indent, the `SIGL` line | +3 |
| less the `Op::Generic` arm it replaces -- its instruction fetch and its whole five-slot call | -6 |
| the `Op::EvalExpr` and `Op::Store` bodies against `assign_evaluated` ceasing to be a call of its own | -8 |
| library plumbing of the crate work above -- bounds-checked indexing and `Result`/`Option` -- **counted inside each row rather than beside it** | (+26 as 7-M sliced it, +17 as this one does) |
| **total** | **+78.00** |

**By function** -- the same 78 with no line attribution in it, and the table to trust:

| function | as `Generic` | promoted | delta |
|---|---:|---:|---:|
| `run_clause_region` | 0 | 344 | +344 |
| `step_in_temps_frame` | 237 | 0 | -237 |
| `assign_evaluated` | 48 | 0 | -48 |
| `run_ops::<false>` | | | +19 |
| **total** | | | **+78.00** |

**What of it is removable, with a build behind each answer rather than an argument:**

| item | removable? | evidence |
|---|---|---|
| the call boundary and its `sret`, plus `run_clause_region`'s prologue and epilogue -- about half the 78 | **yes, and with one implementation of the clause unit** | step 6: `varlookup` +81 to -71 per pass, `alloc4c` +164 to -67, both arms of one build, both instruments |
| the `sret` half on its own | **yes** | step 5: `Failure` 104 to 24; but the ratio worsens on every axis because the tree-walker gains more |
| the second dispatch level | not separately measured | it survives step 6's spike, which keeps two `match`es and still removes 152 |
| the register file, about 6 | no, at this level | it is the same fact as "the program counter is an op index" |
| `stale`, +3 | no, and it is nearly free | a `TRACE` run by one body clause must be seen by the next |
| `eval_chunk_expr`'s `(kind, slot)` re-derivation, 8 | yes, and out of scope | it changes what a promotion emits and gives `Chunk` a lifetime |
| library plumbing | **not as an item** | it is the plumbing of the rows above, and it leaves when they do |

**The verdict this replaces.**
7-M's is "the residual is inherent to the two-level shape", with the call boundary "the price of running a region inside the one clause wrapper both engines share" whose only alternative is "a second implementation of that wrapper".
That is refuted by a build: the alternative is an enter/leave pair, there is exactly one implementation of the clause unit in it, and it removes about twice what 7-M's whole task did.
What survives is the narrower and still useful claim -- **the residual is dominated by the call boundary, and the call boundary is a consequence of the wrapper being a scoped closure rather than of the clause unit being shared.**

## What follows for 4f, which is this task's only forward-looking claim

The brief asks whether either spike removes enough to change what 4f should do first.
**One does.**

The enter/leave split takes `varlookup` and `alloc4c` below 1.0 on both instruments -- the first time any axis with a promoted clause has passed criterion 4 in this phase -- and it costs `emptyloop` 24 instructions per pass.
Landing it is a separate decision and not this task's, and there are three things a task that takes it would owe:

* **`emptyloop` gets worse and the mechanism is known**, so it is a trade rather than a mystery: `run_ops::<false>` doubles in size and the `Op::Generic` arm sits inside the bigger function. Whether that is worth it depends on how many bodies are all-`Generic` once Tasks 9 and 10 land, which is a question those tasks answer and this one cannot.
* **The spike is a spike.** It moves the failure-site recording and the temps-frame discipline into two functions that must be called in pairs, and nothing in the type system yet says they are. A guard whose `Drop` is the leave is the obvious way to make that unforgettable and was not tried here.
* **It interacts with Tasks 9 and 10 rather than composing with them.** Every further promotion changes the ratio of promoted to `Generic` clauses, which is exactly the ratio this trade turns on.

Boxing `Raised` should **not** be taken for criterion 4's sake -- it makes every axis worse on both instruments -- but it is worth about 70 instructions per pass on both arms of `emptyloop` and 121 on `varlookup`'s tree-walker arm, which is a Phase 4f matter rather than a Phase 4e one: 4f measures this binary against the oracle, where absolute speed is the deliverable and the arm ratio is not.
**Recorded here so 4f does not have to rediscover it**, and so nobody reads 7-M's "not attempted" as "not worth attempting".

## What I could not settle

* **Which of `stale`, the payload-read row and the codegen row is "the" wrong one.** I can say the two anchored rows reproduce exactly, that the unanchored three are stated at +7 and measure at +5, that `stale` is the largest single discrepancy at 7 against 3, and that the loop-head redistribution did not happen. Line attribution inside an optimised function is not exact enough to divide the remaining instruction between them, and I have not pretended otherwise.
* **Why `compound`'s cycle ratio is three points worse at `132c3395`** with its instruction ratio improving and disjoint five-round ranges. The *family* is now named -- see "Instructions fall, cycles rise", where the moving quantity is the tree-walker arm's own cycles per instruction -- but that says which term moved, not why a layout change moves it. Naming that needs a disassembly and cache-behaviour comparison this task did not do.
* **Whether Task 8's `+0.74%` reproduces.** It is a difference between two binaries I did not build, and building them is Task 8's control rather than mine. What I can say is that it is not a property of the instrument: a different pair, on the same axis, with the change unable to reach it, reads 1.000000 on instructions.
* **The mechanism behind the whole-number-per-pass excursions.** They are real, discrete, per process, and up to 2.85% on `emptyloop`. `HashMap`'s per-process random seed changing a probe count is the obvious candidate and would explain the constant-per-pass shape, but I did not confirm it, and 7-M's `compound` note attributes the same shape to exactly that.
* **Whether the enter/leave split holds up as a design** rather than as a number. The suite is green on it (1410 passed) and the sharing rule holds by construction, but it was built to be measured and reverted, not to be reviewed.

## Which of 7-M's "worth N" claims survive the additivity failure

The additivity assumption is what broke, not one number, so every "worth N" in 7-M inherits the question.
Sorting them by whether a build stands behind them:

| 7-M claim | verdict |
|---|---|
| the three changes removed **19** per promoted clause, 100 to 81 | **survives.** Both endpoints reproduce here to the instruction: `base` 100, `m7head` 81 |
| change 1 "is worth **10** on its own" | **fails.** It is **16**, measured on a build of exactly the configuration the row names |
| change 2, "the region's ops resolve their own `index`": **8** | **survives as a marginal against head**, which is what 7-M built and what the number means. It is not "8 of the 19" |
| change 3, "the region walked with `for pc in at..end`": **2** | **survives, same reading** |
| "The three marginals total 20 against the 19 actually removed, so they are **additive** to within one instruction" | **fails.** They total **26** against 19; the overlap is 7 |
| "that change is worth 10 on its own **and the 8 is kept on top of it**" | **fails**, and it is the additivity claim in prose |
| the argument-count wall: passing the instruction as a tenth argument is a **net 1** | **survives.** A built configuration, and the exploration table's 99 agrees |
| `stale` "at **4 to 7** per clause across the builds measured" | **survives**, and it is the honest form; the itemisation quoting **7** from that range is what carries the two-instruction error |

**What a marginal against head means, since that is the whole of the disagreement.**
"Undo change X from head and see what it costs" is well defined and 7-M measured two of them correctly.
What does not follow is that the three marginals partition the 19, and the mechanism is 7-M's own argument-count wall read one step further: with the instruction passed down as an argument it is already in a register, so the region's ops reading the region's clause saves much less than it does once that argument is gone.
Measured from the other side in the same sitting: `base` 100 against `c1undone` 97 says changes 2 and 3 together are worth **3** in the presence of the old calling convention, where against head they are worth 10.
Both numbers are right. Neither is "the" value of those changes, because there isn't one.

**One landed comment is now false and I have not edited it.**
`rust/crates/rexx-exec/src/ir/drive.rs`, on `run_clause_region`, ends: "the two together -- this lookup moved in, and the region's ops reading it -- are worth **10 and 8** of the 19 instructions per promoted clause that came off this path."
The 10 is 16, and "10 and 8 of the 19" states the additivity that fails.
By this tree's own rule a comment that states something false must be corrected rather than hedged, but `drive.rs` is what Tasks 9 and 10 are editing and a docs edit landing under a live agent is a collision this plan has already paid for once.
Suggested replacement, for whoever owns the file next:

> Fetching it here costs no slot. Undoing this one change on top of the other two is measured at **16** instructions per promoted clause, and undoing the other two on top of the old calling convention is measured at **3** -- the three do not partition the 19 they removed together, because an instruction already in an argument register is one the region's ops do not save by reading it off the region.

## Instructions fall, cycles rise: six instances, and what they have in common

The both-instruments rule has been resting on one anecdote. It is six.

| # | change | instructions | cycles | source |
|---|---|---|---|---|
| 1 | the `Driving` struct variant | `emptyloop` -7 per pass | ratio 1.00 -> 1.15 | 7-M, rejected on this |
| 2 | `132c3395` itself, on `emptyloop` | flat on both arms | ratio 0.9990 -> 1.0216 (7-M), 1.00125 -> 1.00400 (here) | 7-M's landed change |
| 3 | Task 8, on `alloc4c` | IR arm -0.945% | ratio 1.01410 -> 1.03385 | Task 8's own table |
| 4 | `132c3395`, on `compound` | ratio 1.01791 -> 1.01546 | ratio 1.02072 -> **1.05057**, disjoint ranges | this task, step 3 |
| 5 | `c1undone`, on `varlookup` | IR arm 6 fewer per pass than `base` | ratio 1.07765 -> 1.09725 | this task, sitting B |
| 6 | boxing `Raised`, on `varlookup` | **both** arms faster, -121 and -101 per pass | ratio 1.00890 -> **1.16911** | this task, step 5 |

**What they have in common is measurable, and it is not the compiled arm.**
In every one of them the quantity that moves is the **tree-walker arm's cycles per instruction** -- the denominator of every criterion-4 ratio, and an arm that several of these changes never enter at all.

Tree-walker arm, instructions per cycle, across the builds of this task, at a near-constant instruction count:

| axis | instructions per pass | IPC range across builds |
|---|---|---|
| `varlookup` | 3820 to 3832 | **4.683 to 4.862** (3.8%) |
| `emptyloop` | 1514 to 1516 | **4.464 to 4.755** (6.5%) |

Both ranges exclude the `box` build, whose instruction count genuinely falls.
The same binary measured in three different sittings varies far less -- `emptyloop`'s tree-walker IPC reads 4.595, 4.610 and 4.579 for `head` in sittings A, C and D, a spread of 0.7% -- so the remaining 6% is between builds rather than between occasions.

**So the mechanism family is: a build's layout changes how fast the *unchanged* arm retires its own instructions, and the ratio moves because its denominator did.**
That is why instance 6 is the clearest case: boxing `Raised` makes the tree-walker arm 12% faster in cycles for a 3% instruction saving, and the ratio blows up to 1.169 without the compiled arm having got worse in absolute terms at all.

**Three consequences, and they sharpen the both-instruments rule rather than restating it:**

* **A cycle ratio is comparable within a build and not across builds.** Both arms of one binary share a layout, so the artifact cancels inside the ratio; comparing one build's ratio against another's does not cancel anything, because the denominators are different code.
* **"Instructions fell and cycles rose" is not by itself evidence that a change is bad.** In four of the six the compiled arm's absolute cycles barely moved and the tree-walker's fell. `Driving` may well have been rejected on an artifact -- which does not make rejecting it wrong, since 7 instructions was not worth the risk either way, but it does mean the phase should stop citing it as a demonstrated hazard of the change itself.
* **What the rule actually buys is a tripwire, not a second opinion.** Its value is that a large cycle move with no instruction move behind it says *look at the denominator*, which is what this section did.

**A caveat on the per-pass method, found while checking this.**
`varlookup`'s tree-walker arm reads 3824.00 instructions per pass in sitting B (n = 19,000,000 and 38,000,000) and 3832.00 in sitting D (n = 9,500,000 and 19,000,000), on the **same binary**.
The per-pass cost is not exactly constant in n, because `x` grows and its decimal rendering gets longer, so a slope taken between two lengths depends slightly on which two.
0.2% on the absolute slope -- and **the IR-minus-tree-walker gap reads 81.00 in both sittings**, because the effect is common to the arms and cancels.
The gap is the robust quantity; a single arm's absolute per-pass figure carries this and should not be quoted across sittings that used different lengths.
