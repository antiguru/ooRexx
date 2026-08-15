# Task 7-M3: land the enter/leave split -- report

BASE `7a7f5849`.
Commits, in order: `7d9cfb9a` (the split) and `73b538e5` (the sitting, appended to `rust/bench-baselines/phase-4e-arms.tsv`).
Two docs commits by the controller, `1eb3866f` and `d6aabbfe`, landed interleaved between them; both touch `docs/` only and neither reaches anything this task built or measured.

Suite **1411**, 0 failed, 4 ignored, in all four combinations -- dev, dev+STRICT, release, release+STRICT -- each exit status read unpiped.
The count is unchanged from BASE, which is what a task adding no test and changing no behaviour should read; the BASE run was taken first, before anything was edited, so the two are a before and an after rather than one number quoted twice.
`cargo fmt --all --check` 0, and `cargo clippy --workspace --all-targets -- -D warnings` 0.
**The clippy run's target directory was clean**, and that is recorded as evidence rather than as a claim, because a warm directory produces a green that has linted nothing: the final run used a `CARGO_TARGET_DIR` whose path did not exist beforehand (`ls` reports `No such file or directory`, and `stat` gives a creation time inside the run), it compiled every dependency from scratch, and `rexx-exec` is in the crate list it checked.

This report is not one of the commits: `.superpowers` is in `.gitignore`.

## The headline, and the part of it that did not reproduce

**The split lands, and it takes both promoted axes below 1.0 on `instructions:u` for the first time in this phase.**
On `cycles:u` this build reads `varlookup` at **1.01141**, so criterion 4's `<= 1.0` is met there on one instrument and not the other.
And it removes about **87%** of the per-pass gap the spike removed rather than all of it.

| axis | instrument | `base` `7a7f5849` | `split` `7d9cfb9a` | 7-M2's spike |
|---|---|---:|---:|---:|
| `varlookup` | `instructions:u` | 1.02118 | **0.98666** | 0.98143 |
| | IR-minus-TW per pass | +81.00 | **-51.00** | -71.00 |
| | `cycles:u` (within its own build) | 1.01081 | **1.01141** | (0.96988) |
| `alloc4c` | `instructions:u` | 1.01026 | **0.99775** | 0.99581 |
| | IR-minus-TW per pass | +164.07 | **-36.00** | -66.99 |
| | `cycles:u` (within its own build) | 1.03648 | **0.99736** | (1.02208) |
| `emptyloop` | `instructions:u` | 1.06596 | **1.07855** | 1.08185 |
| | IR-minus-TW per pass | +100.00 | **+119.00** | +124.00 |
| | `cycles:u` (within its own build) | 1.03711 | **1.10779** | (1.04524) |

Ratios are IR over tree-walker **within each build**, at the axis's committed bound, five rounds, one sitting, both builds rotated through it.

**The cycle column is bracketed in the spike's column on purpose, and the rule behind that is a correction to the one my brief carries** (`1eb3866f`).
A cycle ratio cancels the layout artifact only *inside* one binary, because both arms of one binary share a layout.
Two builds' cycle ratios are ratios over different denominators, so a difference between them is not by itself real work, and the spike's cycle figures are therefore not a target this build can hit or miss.
What each bracketed number does say, soundly, is what criterion 4 read on that build.
The instruction rows carry no such caveat: an instruction count does not move with layout, and both the ratios and the per-pass gaps above are directly comparable across builds and sittings.

**`emptyloop` gets worse and it is a result of this task rather than an omission from it.**
Its instruction gap goes +100 to +119 per pass and its ratio 1.06596 to 1.07855, and those are the comparable figures.
Its cycle ratio reads 1.10779 on this build against 1.03711 on `base`, with disjoint five-round ranges ([1.10528..1.11229] against [1.03437..1.05039]) -- but that pair is two builds, so the ratio delta is not the finding.
**The finding is underneath it, in absolutes**, and it is in the section below: `emptyloop`'s compiled arm gets 1.7% *faster* in cycles here and its tree-walker arm 7.8% faster, so the ratio moved because its denominator did.

## The instrument checked itself before anything was read off it

`base` is a build of `7a7f5849` measured **in the same sitting** as `split`, not a figure carried over.
Between `ced6c209` -- 7-M2's `head` -- and `7a7f5849` only `rexx-bench` changed, so this binary is the same `rexx-exec` code 7-M2 measured, and its column is directly comparable to 7-M2's.

**Every instruction figure of 7-M2's `head` column reproduces to the digit.**

| quantity | 7-M2 `head` | measured here |
|---|---:|---:|
| `varlookup` `instructions:u` | 1.02118 | **1.02118** |
| `varlookup` IR-minus-TW per pass | +81.00 | **+81.00** |
| `alloc4c` `instructions:u` | 1.01026 | **1.01026** |
| `alloc4c` IR-minus-TW per pass | +164.02 | **+164.07** |
| `emptyloop` `instructions:u` | 1.06596 | **1.06596** |
| `emptyloop` IR-minus-TW per pass | +100.00 | **+100.00** |

**And the symbol sizes reproduce too**, which matters because they are the one thing that can say whether two builds of the same shape are the same build.
7-M2 gives `run_ops::<false>` at 5,302 bytes and the pair it replaces at 5,302 + 7,750 = 13,052.
Measured here on the `base` binary: `run_ops::<false>` is 0x14b6 = **5,302** and `run_clause_region` is 0x1e46 = **7,750**, with `run_region_ops` not a symbol at all because it was already inlined into `run_clause_region`.

**The `split` column reproduces across two sittings of this task on `instructions:u`**, taken either side of a `cargo fmt` pass: 0.98666 / -51.00, 0.99775 / -36.0, 1.07855 / +119.00 both times.
Its cycle figures move about a point between the two (`varlookup` 1.00731 against 1.01141, `alloc4c` 1.00808 against 0.99736, `emptyloop` 1.11708 against 1.10779), which is the resolution 7-M2 puts on that instrument and no better.

## Where the 20 instructions went, as far as a build can say

`varlookup` runs two promoted clauses per pass, so the 20-instruction shortfall against the spike is about **10 per promoted clause**.
`alloc4c`'s shortfall is 31 per pass, which is the same order once its own clause count is allowed for.
The direction, the sign and the criterion-4 verdict on `instructions:u` are the spike's; the magnitude is 87% of it.

**The one element I added beyond 7-M2's described shape is measured at exactly zero, so it is not the cause.**
`enter_clause` hands back a `ClauseEntry` that only `leave_clause` spends -- see "what the token buys" below.
A build with the token removed entirely, measured against the landed one in a sitting of its own, reads `varlookup` at **0.98666** and **-51.00000** per pass: the same figures to five decimal places, with `run_ops::<false>` the same 0x29e7 bytes in both.
A zero-sized type costs nothing here, which was the expectation, and it is now a measurement rather than an expectation.

**What is left is a difference in codegen between two compilations of one shape, and the symbol table is the evidence for it.**
7-M2 reports its spike's `run_ops::<false>` at **11,190** bytes; this one is 0x29e7 = **10,727**, 463 bytes smaller.
Two builds of the same shape that differ by 463 bytes in the one function that grew are not the same build, and the mechanism that would produce a per-clause difference of this size is the obvious one: with the region's ops in `run_ops`' own frame, the outer loop's live set -- `frames`, `pc`, `stop`, `depth`, `registers`, `chunk`, `code`, `source`, `start`, `end`, and now `clause` -- stays live across every op of every region, where a callee had a fresh register file for that walk.
That is exactly the pressure the collapse trades for the call boundary, and it is the same trade `emptyloop` pays visibly.

**What I did not do is tune toward the spiked figures.**
No patch of the spike survives, so there is nothing to diff against, and a change made to recover 10 instructions per clause with no mechanism behind it would be fitting to a number rather than measuring one.
The difference is named here and left.

## Instructions fall, cycles rise -- and this time both arms got faster

7-M2's "instructions fall, cycles rise" section says the quantity that moves in these cases is the **tree-walker arm's** cycles per instruction, which is the denominator of every criterion-4 ratio.
This sitting is a clean instance of it, and it is worth recording because it is the reading criterion 4 cannot give.

Absolute cycles per pass, both arms, at the committed bound.
**The two builds ran in one sitting at identical lengths**, which is what makes these figures comparable at all: an arm's absolute per-pass cost is not constant in n -- `varlookup`'s tree-walker arm reads 3824 and 3832 instructions per pass in two sittings of *one* binary -- so a per-pass absolute travels within a sitting and the IR-minus-tree-walker gap travels between them.

| axis | arm | `base` | `split` |
|---|---|---:|---:|
| `varlookup` | tree-walker | 811.49 | **751.92** |
| `varlookup` | IR | 817.89 | **761.86** |
| `emptyloop` | tree-walker | 329.70 | **303.94** |
| `emptyloop` | IR | 342.45 | **336.48** |

On `varlookup` **both arms get about 7% faster in cycles** and the ratio is flat, because the denominator fell with the numerator.
On `emptyloop` the compiled arm gets 1.7% faster and the tree-walker arm 7.8% faster, and that -- not a compiled arm that got slower -- is the whole of the seven-point ratio move.
The tree-walker arm's instruction count barely moves on either axis (`varlookup` 3832 to 3831 per pass, `emptyloop` 1516 to 1515), so this is layout, exactly as 7-M2 describes.

`alloc4c`'s cycle cells are the noisy ones and I am not reading anything off them: the `split` tree-walker arm reads 6387.84 in the first sitting and 6596.40 in the second, 3% apart on one binary, which is wider than the ratio move it would be used to explain.

## What was built, and why it keeps one implementation

**The shape, which is 7-M2's step 6 rebuilt from its description.**

`clause.rs`:

```rust
let entry = self.enter_clause(line);
let ran = body(self);
self.leave_clause(entry, code, ran)
```

`run.rs`:

```rust
let entry = self.enter_stepped_clause(echo, code, index, instruction, source);
let ran = work(self);
self.leave_stepped_clause(entry, code, index, instruction, source, ran)
```

`enter_clause`/`leave_clause` and `enter_stepped_clause`/`leave_stepped_clause` each have exactly one body, and `in_clause` and `in_stepped_clause_with` are the three lines above.
Everything the closure form did sits in the two halves in the same order: the clock invalidation, the `>I>` decay, `current_value_indent`, the `SIGL` line and its fourth-site tripwire, the `*-*` echo, the temps frame and its watermark, both failure-site records, and the `CALL ON` delivery.
`SteppedClause` carries the frame, the watermark and the `ClauseEntry` across, and nothing else -- every other thing the leave does is computed from arguments its caller already holds.

**`run_clause_region` and `run_region_ops` are both gone**, and the region's ops run in a labelled block inside `run_ops`' own `Op::Clause` arm.
`run_ops::<false>` grows 5,302 -> 10,727 bytes against the 13,052 of the pair it replaces.

**The sharing rule holds by construction and not by inspection.**
The tree-walker reaches the clause unit through `in_stepped_clause_with`, which is *defined in terms of* the same pair the driver calls, so there is one implementation of the clause boundary and two entry shapes into it.
Two implementations that agree today is what the dual-engine gate exists to catch, and it has caught four Critical defects in this phase; this change does not create one to be caught.

**The failure path is not hard, and 7-M2 is right about why.**
The clause's `Result` travels *into* `leave_stepped_clause` as a value, which is what `in_clause`'s epilogue already did with it (`let Ok(value) = &ran else { return Ok(ClauseOutcome::Ran(ran)) }`).
So there is no `?` between the two halves to take a failure past the boundary that owes it a site, and the region's ops leave their labelled block carrying the failure rather than returning it.
The `?` in the driver is on the *leave's own* result, which is the boundary's failure and not the clause's -- the distinction `ClauseOutcome` already exists to make.
The cost of that is the one visible ugliness in the diff: `?` is not available inside a labelled block, so the region's fallible ops each spell their propagation out.

### What the token buys, and what it does not

7-M2's own "what I could not settle" says the spike "moves the failure-site recording and the temps-frame discipline into two functions that must be called in pairs, and nothing in the type system yet says they are", and names a `Drop` guard as the obvious answer.
A `Drop` guard cannot be it here: the leave needs `&mut self` and returns a value, neither of which `Drop` can give.

What is available and costs nothing is a token.
`enter_clause` returns a `ClauseEntry` whose field is private to `clause.rs`, `leave_clause` consumes one, and `SteppedClause` carries it.

* **A leave with no enter in front of it does not compile**, because nothing outside `clause.rs` can build the value it needs.
* **An enter whose token is dropped rather than spent warns**, by `#[must_use]` on the type and by `unused_variables` on the binding.
* **A token deliberately discarded with `let _ =` is reached by neither**, and that is stated in `clause.rs`'s module doc beside the rest of what that module's `pub(crate)` surface admits rather than left for a re-review to find.

The module doc's old claim -- that `in_clause`'s closure makes the "two halves came apart" family inexpressible -- was true of one entry shape and is now false of the other, so it is rewritten rather than left standing.

## Step 3: the boundary cases, run directly

Eight programs, each on both engines through `target/release/rexx-run` under `REXX_ENGINE`, with stdout, stderr and exit status compared as three separate byte streams.
Four shapes, each in a plain and a traced form, because the boundary carries the echo and the indent as well as the delivery.

| probe | what it pins | rc |
|---|---|---:|
| `c1` | a `CALL ON` handler delivered at a promoted assignment's boundary; prints `set V 3` | 0 |
| `c2` | a handler whose own `RAISE` leaves a second trap queued behind it | 0 |
| `c3` | `LEAVE` naming a `SELECT` from inside `OTHERWISE` | 0 |
| `c4` | `SIGNAL ON NOVALUE` whose `SIGL` must be the failing clause's own line; prints `novalue at 3` | 0 |

**All eight agree between the engines, and all eight agree with the C++ oracle**, byte for byte on all three descriptors.

**`c2` is the case that has caught implementations twice, and it still behaves.**
`h` runs at the assignment's boundary and its own `RAISE` queues `zy`; the clause unit delivers at most one trap and does not re-check, so `g` waits for the *next* clause's boundary.
Its stdout is `after V` / `G ran 4` / `second` -- the handler for the second trap runs after the following `SAY` has already printed, which is what says the last member boundary is not the last boundary with work.
The traced form reads `SIGL 5` and shows `g`'s clauses interleaved between line 5's `say` and line 6's, on both engines and on the oracle.

**And the comparison is not vacuous, which is the half that needed its own control.**
A build with `run_ops`' `Op::Clause` arm made loud on entry turns **every one of the eight** from rc 0 into rc 120 on the IR arm.
So each probe does reach a promoted clause, and the agreement above is agreement about the path this task changed rather than about a path it never entered.
That build was reverted from a `cp` copy, verified with `sha256sum -c` on all three sources, and the rebuilt `rexx-run` was byte-identical to the pre-control one.

## Three false figures in one file, and the third was mine

The first two are below.
The third is the one worth leading with, because I wrote it and because it is the same defect I had just finished removing.

**The `Op::Clause` arm shipped saying the shape "runs the compiled arm 71 instructions per pass below the tree-walker".**
**-71 is 7-M2's spike. This build measures -51**, which is what its own committed baseline row says (`per_pass_gap ir-tw instructions:u -50.999974`) and what `7d9cfb9a`'s commit message says.
I wrote that sentence while building, from the figure the brief told me to expect, and never returned to it after the measurement came back different -- which is a claim written ahead of its own evidence, in a task whose entire "where the 20 instructions went" section exists *because* that figure did not reproduce.
So the comment contradicted the report being written beside it, and it did so twenty lines above the comment I had corrected one commit earlier for being a measurement attributed to code that does not produce it.

**Fixed by dropping the figure rather than by correcting it to -51**, which is the more durable of the two options the review offered.
A per-pass gap moves with every promotion landed after it, so a number written into that line is falsified by the next task rather than by a mistake; the comment now says what quantity the shape is worth and points at `bench-baselines/phase-4e-arms.tsv`, where a row keyed by an immutable hash records it per commit.
That is the file's stated purpose, and pointing at a row is quotation where restating it in a comment is authorship.

**What I can say about the rest of the code I added**: `git diff 7a7f5849..HEAD` over `rust/crates` has exactly **one** added line matching a figure, and it is that one.
That is a check on the whole change rather than on the finding, which is the form this repository's own rule asks for -- a correction round is where false statements get introduced, so the neighbourhood was re-searched rather than the finding re-read.

## The false comment 7-M2 left for this task, and one it did not know it was leaving

**The named one is gone with its subject rather than corrected.**
`run_clause_region`'s doc ended "are worth **10 and 8** of the 19 instructions per promoted clause that came off this path", where the 10 is 16 and the phrasing asserts an additivity 7-M2 measured false.
That function no longer exists, so the whole paragraph went with it, including the sentence in front of it pricing a nine-slot argument list against a tenth slot -- a call with no call left.
Nothing in `rust/crates` now matches `10 and 8`, `of the 19`, or `nine slots`.

**And carrying the other number forward was the same defect one level down, so it is out too.**
My first version of the `Op::Clause` arm kept "removes two bounds-checked lookups of the same instruction per promoted assignment, worth **8** instructions per clause on `bench-programs/varlookup.rex`", on the strength of 7-M2's verdict that the 8 survives *as a marginal against head*.
It does -- but "head" there is a build whose `run_clause_region` took the instruction as an argument, and this change deletes that calling convention.
7-M2's own demonstration is that the same two changes are worth **3** under the old convention and **10** under the new one, both correct, so a marginal is a number about a pair of builds rather than a property of a change.
Quoting 8 beside code that no longer has the configuration it was measured against would be handing one change a number the measurement does not support, which is exactly what the sentence I deleted was doing.
The comment now states the mechanism and the two checks that police it, and carries no figure.
That is a figure removed rather than re-measured, and I did not re-measure it.

## What this task did not do

* **Nothing a promotion emits changed.** `compile.rs` is untouched, and `ir/mod.rs`'s `const _: () = assert!(size_of::<Op>() == 12);` is untouched and did not fire.
* **`Raised` is not boxed.** Out of scope by name, and 7-M2 measured why.
* **No `unsafe`, no `git checkout --`.** Every revert was `cp` from a copy checked with `sha256sum -c`, and every one was followed by a rebuild.
* **Wall clock is not used anywhere in this report.**

## What I could not settle

* **Why this rebuild removes 10 instructions per promoted clause less than the spike did.** The token is ruled out by a build. The symbol sizes say the two compilations differ, and register pressure in the collapsed function is the mechanism that fits, but no patch of the spike survives to diff against and I did not confirm it by disassembly.
* **What this build's `emptyloop` cycle ratio would be under the spike's layout, which is the only form of that question a cycle figure can answer.** This build's own numbers are internally consistent -- 1.10779 across two sittings with tight ranges, and a tree-walker denominator falling 7.8% with its instruction count flat -- so what happened here is measured. What cannot be settled by comparing it to the spike's 1.04524 is whether the two builds differ in work or only in layout, because that comparison spans two denominators and the artifact does not cancel across them.
* **Whether the trade is worth it once Tasks 9 and 10 land**, which is the question 7-M2 leaves open and which depends on how many bodies are all-`Generic` afterwards. Nothing here answers it, and this task's `emptyloop` figure is the input to it rather than the answer.
* **Whether the labelled block is the right long-term shape for the region walk.** It is what makes `?` unavailable, and the propagation it forces into each fallible op is the least pleasant thing in the diff. A `try` block would remove it and is unstable.
