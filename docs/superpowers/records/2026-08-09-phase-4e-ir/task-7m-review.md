# Task 7-M review: find and remove the per-clause cost

Reviewed `132c3395` against `fd0ea6d1`, from `git show`, with no build, test, benchmark or profiler run.
Source read only through `git show 132c3395:<path>`.

**Spec compliance: PASS.**
All three end conditions are delivered in the form the brief asks for, nothing was promoted, and the one item whose removal would change what a promotion emits is named and stopped at.

**Quality: the verdict is not yet fit to discharge a design question.**
The itemised breakdown does not sum to its own total, the largest single row of it appears in neither the removed part nor the residual, one of the residual items the verdict calls "the shape" is one the report itself names a fix for, and an alternative to the call boundary that keeps one implementation is neither considered nor ruled out.
The code change itself is correct as far as I can establish by reading, and the invariant it now depends on is checked unconditionally at compile time, which is the right level.

## What is right, and worth saying before the findings

* The compile-time check is placed at the only producer.
  `compile` at `rust/crates/rexx-exec/src/ir/compile.rs:672` is the sole construction site of `Chunk` in the workspace -- the other `Chunk {` hits are `BodyEngine::Chunk`, a different type -- so `assert_region_ops_name_their_clause` covers every stream the driver can reach, and it is an unconditional `assert!` rather than a `debug_assert!`, so release carries it too.
* The check's `Some` arm is exactly the set of ops that read `clause` at run time.
  `TraceClause`, `EvalExpr`, `Store`, `Say`, `WhenTest`, `LoopRun` are the six ops in `run_region_ops` that now use the region's instruction, and they are the six in the `Some` arm of `compile.rs:952`.
  Every op in the `None` arm either carries no instruction index or is refused by `run_region_ops` loudly (`Generic`, `Clause`, `SelectCaseText`, `EnterWhen`, `EnterOtherwise`, `EndBranch`).
  That is a tight rather than a convenient partition.
* The run-time half compares pointers, not indices.
  `debug_assert_names_the_clause` at `rust/crates/rexx-exec/src/ir/drive.rs:1093` compares `std::ptr::from_ref` of the looked-up instruction against the region's clause, which catches an aliased index that an equality-on-`usize` check would pass.
* The coverage claim about the new assertion is measured the right way round.
  The report says the assertion adds no coverage, having reddened six tests **with the assertion removed** -- that is the mutation run against the suite without the new check, which is the discipline, and the conclusion drawn is the honest one.
* The `Driving` rejection is documented at the site, with numbers, on the instrument that refutes it.
  `drive.rs:288`-`299` is the right place for it and it will save the next reader a session.
* No `unsafe`, no em-dash in any added comment, three new tests against a suite that moves 1388 -> 1391.

## Correctness of the code change

I found nothing that changes observable behaviour, ordering or clause boundaries.
The four places where I looked hardest:

* **The instruction fetch moved out of `run_ops` and into `run_clause_region`** (`drive.rs:367`-`377`, `drive.rs:780`-`782`).
  On the non-granting path the `Loud::chunk_map_too_short` failure now surfaces from `run_clause_region` rather than from `run_ops`, but it still surfaces before `in_stepped_clause_with` opens the clause, so nothing observable happens in between.
  On the granting path the fetch precedes the grant exactly as before.
* **The region walked as a slice** (`drive.rs:863`, `ir/mod.rs:493`).
  `Chunk::ops_in` returns `None` for a range that is not a range of the stream, where the old `while pc < end` loop would have run the ops up to `ops.len()` and only then failed.
  That is strictly stricter and reaches the same `Loud` failure; for a well-formed stream the two are identical.
* **`Op::JumpUnless` inverted to fall through** (`drive.rs:964`-`972`).
  Equivalent under `for op in ops`, and the tail `Ok(RegionEnd::At(end))` at `drive.rs:1061` is unchanged, so a region that runs off its end answers the same thing.
* **`eval_chunk_expr` now takes `&Instruction`** (`rust/crates/rexx-exec/src/run.rs:6579`).
  One caller in the workspace, updated; the removed `chunk_map_too_short` arm was reachable only for a stream whose op index is outside its own body, which `run_clause_region` now rejects earlier.

Two implementations of one semantics: none introduced.
`Op::Store` still goes through `assign_evaluated`, `Op::Say` through `say_evaluated`, `Op::LoopRun` through `run_loop_with_header`, `Op::WhenTest` through `scan_when`, and the clause wrapper is still the single `in_stepped_clause_with`.

## Findings

### Important

**I1. The itemised +78 table sums to +80, and the report asserts otherwise.**
`task-7m-report.md`, section "The +78, broken down".
The column is `+20, +7, +3, -6, -3, +19, +19, -8, +26, +3`, which is 80, against a total row of `+78` and the sentence "Every row is measured; the total is +78.00 and the arithmetic is the tool's."
Each row's own internal sub-arithmetic checks out (`44+25-35-15 = 19`, `34-15 = 19`, `23-31 = -8`, `132-106 = 26`), so the error is in the itemisation rather than in a row.

**I2. The two-instruction discrepancy is localised to the `run_ops` rows, and the report's own second table says so.**
Same section.
The five rows attributed to `run_ops`' arms sum to `+21`, while the by-function table immediately below reads `run_ops::<false>` at 47 as `Generic` against 66 promoted, a delta of `+19`.
The by-function table is internally exact (`47+237+48 = 332`, `66+344 = 410`, `410-332 = 78`), which makes it the more trustworthy of the two.
So there are two instructions in `run_ops` that the itemisation over-attributes, and the breakdown does not say which row they belong to.

**I3. The first row of the marginals table is base, not head with one change undone, and its "worth 10" is not the subtraction the other two rows use.**
Same file, section "What was removed, and what each step bought", second table.
The table's header is "head with this one change undone" and its preamble says the numbers "are not inferred from the exploration order".
Row 1 is parenthesised "(base + nothing else)", reads 100, and 100 - 81 is 19, not the 10 claimed; rows 2 and 3 do use exactly that subtraction (89-81 = 8, 83-81 = 2).
The exploration table above gives a third value: the "tenth argument" configuration reads 99 per promoted clause, so head with change 1 undone but the slice kept would be about 97, making change 1 worth about 16.
Three numbers for the same change -- 19, 16, 10 -- and the one the report uses is the only one with no build behind it in the report.
This matters because change 1 is the largest of the three and carries over half of the 19.

**I4. The verdict calls the memory-returned `Result` "the shape" while the report itself names the fix for it two sections later.**
Section "What remains, and whether it is inherent", second bullet, against section "One shared cost, measured but not attempted".
The residual bullet says the call boundary "is the shape too, and specifically it is the price of running a region inside the one clause wrapper both engines share. The alternative is a second implementation of that wrapper."
That is a false dichotomy for the part of the cost that is the `sret` return: the return goes through memory because `Failure` is 104 bytes, and boxing `Raised` -- which the report measures at `Failure` 24 bytes and `Result<ClauseRegion, Failure>` 32 -- removes it without changing what a promotion emits and without a second implementation of anything.
`Raised`'s fields at `rust/crates/rexx-exec/src/error.rs:156` (a `Cow`, two `Vec`-shaped `Option`s, a `Vec`, two `u16`s and a `Delivery`) make the 104 plausible, so the fix is real.
The reason given for not taking it is also unsound as stated: "it is a cost on the path **both** arms take, so it improves absolute speed and *worsens* criterion 4, whose denominator falls with it" is contradicted one sentence earlier by "the promoted path carries one more such return per clause than the unpromoted one", which points the ratio the other way.
Neither direction is measured.
This is the "a correct decision shipping with a false reason" shape: not attempting it in this task is defensible on scope, but classifying its cost as *inherent* is not.

**I5. The largest row of the breakdown appears in neither the removed 19 nor the residual list.**
`+26`, "library helpers inlined into each function: bounds-checked indexing and `Result`/`Option` plumbing, 132 against 106", is a third of the +78.
The residual bullets name the second dispatch level (+19), the call boundary (no number), the register file (about 6), `stale` (4 to 7) and the out-of-scope `eval_chunk_expr` match (8).
The library-helper row is absent from all five, and it is precisely the category the landed 19 came out of -- a caller-side bounds-checked fetch, two op-side ones, and one per op in the region loop.
So the verdict's sentence "Every remaining item above except the last is a direct consequence of running a clause's work as a stream of dispatched ops inside the shared clause wrapper" is not true of every remaining item: it does not cover the one that is neither a dispatch level nor a call boundary nor a register, and that one is demonstrably reducible because this task reduced it.
The residual is stated as 59 but the bullets that carry numbers reach at most about 40.

**I6. An option that was not considered and not ruled out: an enter/leave pair in place of the scoped closure.**
`in_stepped_clause_with` (`run.rs:4543`) and `in_clause` (`rust/crates/rexx-exec/src/clause.rs:379`) are both `#[inline(always)]` and both take the clause's whole body as `impl FnOnce`.
Because the body must be *inside* the closure, the region's ops have to live in a callee, which is the call boundary the report measures at +20 (the nine-argument call and its `sret`) plus +19 (`run_clause_region`'s prologue, epilogue and outcome mapping) -- about half the +78.
The report treats "one closure" and "one implementation" as the same thing.
They are not: splitting the wrapper into a shared `enter` and a shared `leave` (or a guard whose `Drop` is the boundary), both called by both engines, keeps exactly one implementation of the clause unit while letting the driver run a region's ops in its own loop, with one dispatch level and no second function.
It is not free -- `in_clause`'s epilogue reads the clause's `Result` and `value.rooted()`, and the failure path would have to run the leave from the driver's own `?` site -- but "harder" is not "inherent", and the report does not mention the option at all.
This is also the load-bearing sentence in the spec text the owner wrote on the strength of this report: "`in_clause` being a scoped closure is what forces two levels, and that is semantics rather than preference" (`docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md`, criterion 4, commit `8e7ce246`).
The report never establishes that sentence.
What a scoped closure forces is a *call boundary*; the second *dispatch level* comes from a clause's ops being a sub-range of a flat stream, and a single-level driver with `Clause`/`EndClause` ops would have one match rather than two.
Neither half of the sentence is refuted here either -- the point is that it is asserted where the report only measured that one particular collapse (`#[inline(always)]` on `run_clause_region`) is worse.

**I7. Five of the seven axes carry an instruction claim with no cycle reading, against the phase's own rule.**
Section "Criterion 4, all seven axes" gives `instructions:u` for all seven; the table below it gives `cycles:u` and wall for `varlookup` and `emptyloop` only.
The rule is that both instruments are required for any performance claim, and the report itself is the demonstration of why: the rejected `Driving` variant improved instructions and moved a cycle ratio by 15 points.
So "The IR arm improves on six axes" is unsubstantiated for `arith`, `compound`, `strings`, `alloc4c` and `startup` as it stands.
The spec text the owner then wrote makes this worse rather than better: the replacement criterion 4's falsification clause now reads "a recorded ratio per axis on both instruments", which this report does not supply.

**I8. The landed change shows on `emptyloop` the same signature the report rejected `Driving` for, and does not say so.**
`Driving` was dropped because `emptyloop`'s cycle ratio went 1.00 to 1.15 while its instruction count fell.
Head takes `emptyloop`'s cycle ratio from 0.9990 to 1.0216 with its instruction count flat on both arms.
Same shape, an order of magnitude smaller, and the report reframes it as evidence that the criterion is unsound rather than noting that it is the phenomenon it had just used as a rejection ground.
Both readings can be true, but only one is stated, and the one that is stated is the one that favours the change.

**I9. "The tree-walker arm is byte-identical to base on six of the seven axes" is contradicted by the report's own table.**
`startup`'s tree-walker column reads 538,065 at base against 538,561 at head, a difference of 496 and 0.09%.
`emptyloop`'s reads 37,850,657,812 against 37,850,658,018.
`compound` is the axis the report does exclude, on the `HashMap` seeding argument, which is sound; the other two are not excluded and are not identical.
Separately, "byte-identical" is inferred from equal instruction counts ("which is what `emptyloop`'s two arms reading exactly base's counts says"), and equal counts do not imply identical bytes -- the report's own breakdown row "`run_ops`' loop head and prologue, redistributed by codegen" is an admission that this function's codegen moves.
The weaker and true claim -- the executed instruction count on the `Generic` and per-entry paths is unchanged to within a constant far below one instruction per pass -- carries the argument on its own and should replace it.

### Minor

* **M1. "to nine significant figures" is overstated.**
  40,325,665,756 against 40,325,665,822 agree to eight significant figures; 37,850,657,812 against 37,850,658,018 agree to seven.
  The conclusion survives (206 in 3.8e10 is 5e-9), but the phrase is quoted verbatim into the spec at `8e7ce246` as "**identical**", where the numbers beside it are not.
* **M2. "the nineteen-arm `match op`" miscounts.**
  `Op` has 18 variants (`ir/mod.rs`: 17 with a payload plus the unit `EndBranch`), and both `run_ops`' and `run_region_ops`' matches have 18 arms.
  Appears twice, in the breakdown row and in the first residual bullet.
* **M3. `run_ops::<true>` now fetches the instruction twice per promoted clause.**
  `drive.rs:373` under `if GRANTING` and `drive.rs:780` in `run_clause_region`.
  The trade is right -- the granting level is the activation's own and runs each clause once, where the non-granting level is every loop body pass -- but the report presents the change as a pure removal and does not mention the cost it moves onto the other path.
* **M4. `assert_region_ops_name_their_clause` can panic with the wrong message.**
  `compile.rs`, the inner `ops[at + 1..(*end as usize).min(ops.len())]`.
  The upper bound is clamped but there is no guard that `at + 1 <= end`, so a stream with `end <= at` panics with a slice-range message rather than the assertion's own.
  `assert_clause_regions_hold_no_clause_op` has the same shape, so this is consistency rather than a new defect.
* **M5. `Op::LoopRun` is the one site that passes an index and an instruction as a pair.**
  `drive.rs:1027`-`1043` hands `run_loop_with_header` the op's own `*index` and the region's `clause`.
  They agree by the new assertion, but `run_region_ops` could take the region's `index` alongside `clause` and remove the pairing entirely, which would make the assertion a check on the stream rather than on a live pair.
* **M6. `stale`'s 17-instruction figure is measured on the axis with no promoted clause.**
  The residual bullet says hoisting `stale` "was measured *before* this task at 17 instructions per `emptyloop` pass".
  `emptyloop` reaches no `Clause` op at all, so that number is the hoist's cost on a body with nothing for it to save, not the saving on a body with promoted clauses -- at 4 to 7 per clause a body with several would come out ahead.
  The behaviour objection given alongside it is the sound one and is sufficient: a `TRACE` run by one body clause must be seen by the next, and a hoisted local would not see it.
  The conclusion is right; the first half of the reason is not load-bearing and reads as if it were.
* **M7. The report is hard-wrapped rather than one sentence per line**, against the phase's markdown rule.

## Claims I cannot settle without re-measuring

I ran nothing, per the brief.
Each of these would settle a finding above.

* **The +78 total and its per-row attribution (I1, I2).**
  Build base `fd0ea6d1` and the control whose `compile` emits `Op::Generic` for `Assignment`, then for each run `valgrind --tool=callgrind --cache-sim=no --branch-sim=no --compress-strings=no --compress-pos=no` over `bench-programs/varlookup.rex` at n=1e6 and n=3e6 and difference the two lengths per function.
  Refuted if the per-function deltas do not sum to 78, or if `run_ops::<false>` does not read 47 as `Generic` against 66 promoted.
  If it does read 47/66, the itemisation is over-attributed by 2 and the row carrying the error should be named.
* **Change 1's true marginal (I3).**
  Build head with `run_clause_region` taking `instruction` again while keeping the slice and the ops-read-clause changes, then `perf stat -e instructions:u` on `varlookup` at n=1e6 and n=3e6, interleaved with head in one sitting, median of three.
  The report's own accounting predicts about 91 per promoted clause.
  Refuted -- and the "worth 10" with it -- if it reads 100, which is what the marginals table's first row already shows.
* **Whether boxing `Raised` helps or hurts criterion 4 (I4).**
  Box the `Raised` variant of `Failure`, then run both arms of both binaries on `varlookup` under `perf stat -e instructions:u` and `perf stat -e cycles:u`, rotated within one sitting.
  The report asserts the ratio worsens.
  Refuted if `varlookup`'s IR/TW ratio falls on either instrument, which is what "the promoted path carries one more such return per clause" predicts.
* **Cycle ratios for the five axes that have none (I7).**
  `perf stat -e cycles:u` on all four arms -- base and head, tree-walker and IR -- for `arith`, `compound`, `strings`, `alloc4c` and `startup`, rotated so no arm keeps a slot, five rounds, medians.
  Refuted -- and the "improves on six axes" headline with it -- if any of the five has a head cycle ratio above its base one.
* **Whether the `Generic` and per-entry paths are byte-identical (I9).**
  `objdump -d` both binaries and compare the `run_ops` symbol, or `sha256sum` the `.text` of the two.
  The report infers byte-identity from equal counts; this is what would establish or drop it.

## What I would ask for before the verdict is relied on

The verdict's *conclusion* -- that most of the residual is structural and that promotion stays monotonically worse per clause -- looks right to me, and the phase is better off knowing it now.
Three corrections would make it carry the weight the withdrawn criterion puts on it:

1. Reconcile the breakdown to 78 and carry the library-helper row into either the removed part or the residual (I1, I2, I5).
2. Restate the call-boundary bullet as "removable only by a change this task's scope excludes" rather than "the shape", naming boxing `Raised` there, and drop or measure the criterion-4-denominator reason (I4).
3. Either rule out the enter/leave restructure or record it as unexplored, and stop the spec's criterion-4 text asserting a mechanism ("a scoped closure is what forces two levels") that the report does not establish (I6).
