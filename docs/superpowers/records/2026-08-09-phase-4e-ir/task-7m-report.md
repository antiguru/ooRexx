# Task 7-M: find and remove the per-clause cost -- report

BASE `fd0ea6d1`.
One commit: `132c3395`.

Suite **1388 -> 1391**, 0 failed, 4 ignored, green in dev, dev+STRICT, release and release+STRICT, each
exit status read unpiped.
`cargo fmt --all --check` 0.
`cargo clippy --workspace --all-targets -- -D warnings` 0, from a warm target directory -- provisional
by this tree's own rule, and owed a clean-target run at the phase boundary rather than here.

Headline: the +78 instructions per promoted clause is broken down below with a number against each
item, **19 of them are removed**, and the residual is judged inherent to the two-level shape with one
named exception that the brief puts out of scope.
`varlookup`'s IR-minus-tree-walker per body clause goes **138.5 -> 119.5**, its instruction ratio
**1.07251 -> 1.06256**, its cycle ratio **1.0786 -> 1.0452** and its wall ratio **1.0766 -> 1.0454**.
`varlookup` still fails criterion 4 and `emptyloop`'s status flips -- **on an axis whose executed
instruction count this change provably does not touch**, which is the second finding and is about the
criterion rather than about the change. Both are laid out below.

## The instrument, and the two corrections it needed

`perf stat -e instructions:u`, as the brief specifies, but taken as a **per-pass difference between two
lengths of the same program** rather than as a whole-program count: the same body at `n = 1e6` and
`n = 3e6`, so program startup cancels exactly and the answer is instructions per loop pass.
Solving that across bodies of one, two and three clauses splits the IR-minus-tree-walker delta into
three quantities that are each exact:

| | instructions |
|---|---:|
| per body-range entry | 77 |
| per `Op::Generic` clause | 22 |
| per promoted assignment clause | 100 |

`varlookup` is one entry and two promoted clauses per pass: `77 + 2 x 100 = 277`, which is the figure
the brief hands over, and `277 / 2 = 138.5` per body clause.
`emptyloop` is one entry and one `Generic` clause: `77 + 22 = 99`, which is the earlier per-pass
attribution.
Both reproduce to the instruction, and the intercept comes out at 77 from the `nop` bodies and 77 from
the assignment bodies independently.

**A single `perf` run is not reliable enough for this, and that was measured rather than assumed.** A
sample was seen to come back low by ~0.7%, which on a differenced quantity read as 23 instructions per
pass -- a third of the number being chased, and it invented a movement in a binary that had not changed.
Repeats of one binary otherwise agree to eight significant figures. Every figure below is a median of
three, and the two rounds taken before that was noticed were re-taken.

**`callgrind` is what produced the breakdown, and it agrees with `perf` exactly.** `valgrind
--tool=callgrind --cache-sim=no --branch-sim=no --compress-strings=no --compress-pos=no`, differenced
between two lengths the same way, gives 277 instructions per pass on `varlookup`'s shape against
`perf`'s 277, and it attributes them per function and per source line with no sampling error at all.
Sampled `perf record` was tried first and abandoned: at 20,000 samples a 10-instruction-per-pass line
is 50 samples, and the resulting per-line percentages disagreed with the exact numbers by enough to
mislead.

**The control that makes the breakdown a breakdown.** A probe build whose `compile` emits `Op::Generic`
for `Assignment` recreates the pre-Task-7 shape on this axis: it reads per promoted clause **21-22**,
exactly the `Generic` figure, with the tree-walker arm byte-identical. Diffing that build against head
line by line, on the IR arm only, is what the +78 is.

## The +78, broken down

`callgrind`, base binary, IR arm, one promoted assignment clause against the same clause run as
`Op::Generic`. Every row is measured; the total is +78.00 and the arithmetic is the tool's.

| item | instructions |
|---|---:|
| `run_ops`' `Op::Clause` arm: the nine-argument call to `run_clause_region` and its `sret` return | +20 |
| `run_ops`' `Op::Clause` arm: `stale`, i.e. `chunk.trace() != self.chunk_trace()` | +7 |
| `run_ops`' `Op::Clause` arm: the `ClauseRegion` match and the op's own payload reads | +3 |
| less the `Op::Generic` arm it replaces -- its instruction fetch and its whole five-slot call | -6 |
| `run_ops`' loop head and prologue, redistributed by codegen | -3 |
| `run_clause_region`'s prologue, epilogue and `ClauseOutcome` -> `RegionEnd` -> `ClauseRegion` mapping (+44) with its unattributed line-0 code (+25), against `step_in_temps_frame`'s same two (-35, -15) | +19 |
| `run_region_ops`' region loop -- `pc < end`, `chunk.op_at_index(pc)`, the nineteen-arm `match op` (+34) -- against `step`'s own `mem::take` and single `InstructionKind` match (-15) | +19 |
| the `Op::EvalExpr` and `Op::Store` bodies: instruction re-fetch, kind re-match, `eval_chunk_expr`'s `(kind, slot)` match, the register write and the register read (+23), against `assign_evaluated` ceasing to be a call of its own (-31) | -8 |
| library helpers inlined into each function: bounds-checked indexing and `Result`/`Option` plumbing, 132 against 106 | +26 |
| the shared clause unit itself -- `in_stepped_clause_with`, `in_clause`, the printed indent, the `SIGL` line | +3 |
| **total** | **+78** |

By function, which is the same number with no line attribution in it at all:

| function | as `Generic` | promoted |
|---|---:|---:|
| `run_ops::<false>` | 47 | 66 |
| `step_in_temps_frame`, with `step`, `in_stepped_clause` and `in_clause` inlined | 237 | 0 |
| `assign_evaluated`, its own call | 48 | 0 |
| `run_clause_region`, with `in_stepped_clause_with`, `in_clause`, `run_region_ops`, `eval_chunk_expr`'s match and `assign_evaluated` inlined | 0 | 344 |
| **total** | **332** | **410** |

**Where it is not.** Not in `Op::Const` or `Op::TraceLiteral`, which the brief already established and
which this task did not open on. Not in `grant_procedure_permission`, which is `const`-eliminated on
every body clause. Not in branch prediction: 70,000 misses out of 6.3 billion branches on `emptyloop`,
either engine, either binary.

## What was removed, and what each step bought

Every row is a built binary measured on the instrument above. **Only the last one landed**; the middle
three are the exploration, and the variant they share is rejected for the reason the next table gives.
Each run also read the tree-walker arm's own absolute count, so a change to that arm could not hide: at
head it is 1506, 2323 and 3382 instructions per pass on the three probe shapes, which is base exactly.

| configuration | per entry | per `Generic` | per promoted | `varlookup` /pass |
|---|---:|---:|---:|---:|
| base `fd0ea6d1` | 77 | 22 | 100 | 277 |
| region ops read the region's clause, passed in as a tenth argument | 77 | 22 | 99 | 275 |
| a `Driving` struct for the four range-invariant arguments, plus the above | 71 | 21 | 90 | 251 |
| `Driving` plus the region walked as a slice, ops still re-fetching | 71 | 21 | 96 | 263 |
| `Driving` plus the slice plus ops reading the clause | 71 | 21 | 86 | 243 |
| **head** -- no struct; `run_clause_region` fetches the clause from `index`; region as a slice; ops read the clause | **77** | **22** | **81** | **239** |

Three changes landed. Each was then measured **as a marginal against head**, by building head with that
one change undone, so the numbers are not inferred from the exploration order:

| head with this one change undone | per promoted clause | so the change is worth |
|---|---:|---:|
| `run_clause_region` takes `instruction` again instead of `index` (base + nothing else) | 100 | **10** |
| the region's ops resolve their own `index` instead of reading the region's clause | 89 | **8** |
| the region walked with `for pc in at..end` and one `op_at_index` per op | 83 | **2** |
| head | 81 | |

The three marginals total 20 against the 19 actually removed, so they are additive to within one
instruction.

**The argument-count wall, which is the finding that made the second row worth anything.** Not looking
the region's own instruction up again in `Op::EvalExpr` and `Op::Store` -- two bounds-checked lookups of
an instruction the enclosing `Clause` op had already found -- is worth 8. Passing the instruction down as
an extra argument gives almost all of it straight back, because `run_clause_region` already took ten
argument slots counting the receiver and this ABI passes six in registers: measured at a **net 1**. The
fix is to pass **`index` instead of `instruction`** and let `run_clause_region` do the single lookup
itself. The argument list gets one slot shorter rather than one longer, and the caller's own fetch
disappears on the path that does not grant the first-instruction permission -- which is every clause
inside a construct's body -- so that change is worth 10 on its own and the 8 is kept on top of it.

**The region as a slice.** `run_region_ops` walked `[at, end)` with a `while pc < end` and a
`chunk.op_at_index(pc)` per op, which is a bounds check the loop guard had already made. A region's
counter only ever advances -- every op inside one either falls through to the next or ends the region --
so the range is settled once on the way in and the ops are iterated. Worth 2 on `varlookup`, whose
regions hold three ops; it scales with region length rather than with clauses.

**Nothing else.** Per body entry and per `Generic` clause the head is byte-identical to base, which is
what `emptyloop`'s two arms reading exactly base's counts says. The tree-walker arm is byte-identical
to base on six of the seven axes and within 5e-6 on `compound`.

### Rejected variants, each measured

| variant | result |
|---|---|
| `#[inline(always)]` on `run_region_ops` | zero on every cell: LLVM already inlines it into `run_clause_region` |
| `#[inline]` on `run_clause_region` | zero on every cell |
| `#[inline(always)]` on `run_clause_region` | promoted clause 100 -> **143**, and it moved the tree-walker arm too (+10 per `emptyloop` pass) |
| the driver's own op loop over a `stop`-bounded slice | entry +7, `Generic` clause -5, promoted -5: `emptyloop` +2 per pass against `varlookup` -3, so it trades one axis for another |
| a `Driving` struct that `run_ops` itself takes | -7 instructions per `emptyloop` pass and **`emptyloop`'s cycle ratio 1.00 -> 1.15**; see below |
| locals for `code`/`source` in the `Op::Generic` arm, on top of `Driving` | zero instructions, cycles slightly worse -- refutes "the struct's pointer loads are the cause" |

**The `Driving` rejection is the one worth reading.** It does what it promises to `instructions:u`, and
on `cycles:u` it moves both arms of `emptyloop` far more than that, in opposite directions: the
tree-walker arm -- which never enters `run_ops` at all -- from 8.61 to 7.96 billion cycles, and the
compiled arm from 8.63 to 9.12, taking that axis' cycle ratio from 1.00 to 1.15 while its instruction
count *falls*. Reproduced across two sittings and two builds, with branch misses unchanged, so it is
neither noise nor prediction. No mechanism below "the layout changed" was found, and the variant was
dropped rather than shipped for 7 instructions. It is recorded on `run_ops` itself, with the numbers, so
the next reader does not spend the session rediscovering it.

## Criterion 4, all seven axes

`instructions:u`, base against head, both arms of each binary, interleaved within one sitting, median of
three, under `ulimit -v 8388608`.

| axis | base TW | base IR | base IR/TW | head TW | head IR | head IR/TW | head IR arm |
|---|---:|---:|---:|---:|---:|---:|---:|
| `emptyloop` | 37.8507e9 | 40.3257e9 | 1.06539 | 37.8507e9 | 40.3257e9 | 1.06539 | 0.000% |
| `varlookup` | 72.5807e9 | 77.8437e9 | 1.07251 | 72.5807e9 | 77.1217e9 | **1.06256** | -0.928% |
| `arith` | 30.4631e9 | 30.7730e9 | 1.01017 | 30.4631e9 | 30.7255e9 | **1.00861** | -0.154% |
| `compound` | 77.2629e9 | 78.6480e9 | 1.01793 | 77.2633e9 | 78.4580e9 | **1.01546** | -0.242% |
| `strings` | 85.1798e9 | 86.9107e9 | 1.02032 | 85.1798e9 | 86.6257e9 | **1.01698** | -0.328% |
| `alloc4c` | 15.9843e9 | 16.3613e9 | 1.02359 | 15.9843e9 | 16.3043e9 | **1.02002** | -0.348% |
| `startup` | 538,065 | 562,259 | 1.04496 | 538,561 | 562,454 | 1.04436 | +0.035% |

The IR arm improves on six axes and `emptyloop` is unchanged on both arms by construction: nothing on
the `Op::Generic` path changed, and its body holds no promoted clause. `startup`'s numbers are half a
million instructions dominated by dynamic loading, and its +0.035% is inside its own spread.

Cycles and wall clock, four arms of one sitting -- base and head, tree-walker and IR -- rotated so no arm
keeps a fixed slot, five rounds for cycles and seven for wall, medians:

| axis | instrument | base IR/TW | head IR/TW |
|---|---|---:|---:|
| `varlookup` | `cycles:u` | 1.0786 | **1.0452** |
| `varlookup` | wall | 1.0766 | **1.0454** |
| `emptyloop` | `cycles:u` | 0.9990 | 1.0216 |
| `emptyloop` | wall | 0.9990 | 1.0228 |

### `emptyloop`'s criterion-4 status flips, and it cannot be this change

Criterion 4 is `IR/TW <= 1.0` on wall clock, which is how 4b recorded `emptyloop` **passing** at 3.12x
against 3.13x and `varlookup` **failing** at 1.016. On the four-arm sitting above, `emptyloop` reads
0.9990 at base and 1.0228 at head: it passes before and fails after.

**And base and head execute the same instruction count on that axis, on both arms, to nine significant
figures** -- 40,325,665,756 against 40,325,665,822 on the IR arm, 37,850,657,812 against 37,850,658,018
on the tree-walker's. Nothing on the `Op::Generic` path changed and `emptyloop`'s body holds no promoted
clause, so there is no work for this change to have added or removed there. Both arms got faster in
absolute wall time and the tree-walker arm got faster still, so the ratio rose.

**So a criterion-4 reading taken on wall clock carries at least plus-or-minus 2.3 points of
code-placement sensitivity, demonstrated with the executed work held exactly constant.** The
pre-existing record already straddles the threshold from both sides: Task 7 measured `emptyloop` at
1.0526 in one sitting of that session and 0.9887 in another, on binaries with identical instruction
counts. This task's contribution is to show the same spread between two *different* binaries whose
`emptyloop` instruction counts agree, which is the stronger form of the same statement.

**What follows for the phase, and it is not this task's to decide.** A criterion whose pass/fail flips
on function placement cannot discharge a design question, and `emptyloop` -- the clause-dispatch floor,
the axis with the least work per clause and therefore the highest proportion of driver in it -- is where
that bites hardest. On `instructions:u` `emptyloop` reads 1.06539 at base and 1.06539 at head, unmoved
and failing on both.

## What remains, and whether it is inherent

Per promoted assignment clause the head pays **81** against **22** for the same instruction run as
`Op::Generic`: the gap Task 7 opened is now **+59** rather than +78. Item by item, of the breakdown
above:

* **the second dispatch level** -- the nineteen-arm `match op` inside the region, on top of the driver
  loop's own match that reached the `Clause` op -- **is the shape**. It is what "two-level" means. It
  cost +19 net against `step`'s single `InstructionKind` match and it is still there. Collapsing it by
  inlining `run_clause_region` into `run_ops` was measured and is 43 instructions per clause *worse*.
* **the extra call boundary** -- `run_clause_region` as a function, with a `Result<ClauseRegion,
  Failure>` returned through memory -- **is the shape too**, and specifically it is the price of running
  a region *inside* the one clause wrapper both engines share. The alternative is a second
  implementation of that wrapper, which is the defect the dual-engine gate exists to catch.
* **the register file** -- a value produced by `Op::EvalExpr` and consumed by `Op::Store` travels through
  a bounds-checked `Vec<ObjRef>` where `step` keeps it in a machine register, about 6 instructions per
  clause -- **is the shape**, and it is the same fact as "the program counter is an op index".
* **`stale`, at 4 to 7 per clause across the builds measured, is not removable at this level and is
  nearly free anyway.** It replaces the
  per-clause `tracing_clause` read the tree-walker's own `echo_stepped_clause` makes, which a promoted
  clause skips. Hoisting it into a local was measured *before* this task at 17 instructions per
  `emptyloop` pass and would also change behaviour: a `TRACE` executed by one body clause would not be
  seen by the next.
* **one item is removable and this task must not remove it.** `Op::EvalExpr` names an instruction and a
  slot, so `eval_chunk_expr` re-derives *which* expression from `(kind, slot)` -- 8 instructions per
  clause -- where `step`'s own match already had the `&Expr` in hand. Carrying the expression in the op
  removes that outright. It changes what a promotion emits and it gives `Chunk` a lifetime over the
  body, which is exactly the case the brief says to name and stop at. **Named, and stopped at.**

**Verdict: the residual is inherent to the two-level shape.** Every remaining item above except the last
is a direct consequence of running a clause's work as a stream of dispatched ops inside the shared clause
wrapper, and the one that is not is out of this task's scope by the brief's own rule. What was removable
without touching either the shape or what a promotion emits was 19 of the 78, and it is removed.

**And the shape's cost is per clause, so Task 7's finding stands unchanged.** Each further promotion
still adds 59 instructions per clause it promotes rather than amortising anything. Tasks 8, 9 and 10 make
criterion 4 monotonically worse on instructions by that amount times the clauses they promote, and this
task does not change that -- it lowers the per-clause constant by a quarter.

## One shared cost, measured but not attempted

`Failure` is **104 bytes**, and `Raised` is the variant that makes it so. Every `Result<_, Failure>` in
the crate is therefore returned through memory rather than in registers, and the promoted path carries
one more such return per clause than the unpromoted one. Boxing `Raised` would take `Failure` to 24
bytes and `Result<ClauseRegion, Failure>` from 104 to 32.

Not attempted, for two reasons, both worth recording. It touches every site that raises, which is not a
"removable part of the +78". And it is a cost on the path **both** arms take, so it improves absolute
speed and *worsens* criterion 4, whose denominator falls with it -- which is the trap on the far side of
4b-M's precedent, and the same trap `emptyloop`'s wall row above walked into by accident.

## What I could not establish

* **Why a refactor that cannot change semantics moves `cycles:u` by 8% in opposite directions on the two
  arms.** Measured four times on two binaries, with instruction counts and branch misses held constant.
  This is the same family as the extraction cost this phase already records with no mechanism named, and
  no mechanism is named here either. Naming it needs a disassembly and layout comparison this task did
  not do.
* **Whether the new compile-time assertion catches anything the suite would miss.** It does not, for the
  one mutation tried: making an assignment's value op name the instruction after it reddens six tests
  with the assertion removed -- the dual-engine population sweep, the branch, loop and case-file
  harnesses and the known-divergence table. What the assertion adds is the *shape* of the failure, a
  compile-time refusal naming the op and both instructions instead of a divergence in a program's
  output. Recorded on the assertion itself so nobody reads it as coverage.
* **`compound`'s tree-walker arm is not deterministic to better than about 5e-6**: 77,262,940,772 against
  77,263,349,983 between two binaries whose tree-walker path is untouched. `HashMap` is randomly seeded
  per process and stem lookup goes through one, so probe counts differ run to run. It is far below
  anything decided here, but it is why `compound`'s row is not quoted as byte-identical the way the other
  five are.
* **No wall-clock reading of `emptyloop` means anything in either direction**, and this task strengthens
  rather than repeats that: it is now measured against two binaries with identical instruction counts
  rather than inferred from repeats of one.
