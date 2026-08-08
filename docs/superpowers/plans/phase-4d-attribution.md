# Phase 4d attribution -- where this crate's time goes, by named cause

Phase 4d-1 Task 6.
Measured 2026-08-09 at commit `c445154c`, working tree clean before and after.
`rust/target/release/rexx-run` sha256
`c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967` -- byte-identical to the binary
the current baseline section of `perf-baseline.md` was measured with, so every share below is a
share of a number already on the record rather than of a re-measurement.
Oracle `bin/rexx` sha256 `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019`,
`lib/librexx.so.4` `42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb`,
`lib/librexxapi.so.4` `3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66` -- all three
match that section's Provenance exactly.

**Revised after review, same day, same tree.**
The first version of this file explained `alloc4c`'s 2.08x as a cost shared with the oracle.
That was wrong and is corrected in C6: the oracle was profiled on that axis, which the first version
never did, and it spends 56.0% of its time collecting where this crate spends none.
Step 1's refutation was also rebuilt, because the model it refuted was not the one a reader would
hold.
No measurement was withdrawn and no share changed; what changed is one explanation, one argument,
and the additions the two required.

Three prototypes were built, measured, and reverted inside the commit that publishes this file.
`sha256sum -c` against copies taken before the edits confirms `eval.rs`, `run.rs` and `value.rs`
are byte-identical to their pre-prototype state, and the rebuilt `rexx-run` has the sha256 above.
Never `git checkout --`.
No optimisation is proposed here and none survives this commit.

## Step 1: The dominant question, answered first

**The per-axis ratio spread is a property of this crate, not of the denominator.**
It was a property of the denominator at `107febcd`, and the five speedups that landed after that
measurement are what changed the answer.

Absolute throughput, from the committed baseline (2026-08-08 at `d233d1e9`), not from ratios:

| axis | oracle iters/s | this crate iters/s | oracle ns/iter | this crate ns/iter | difference |
|---|---:|---:|---:|---:|---:|
| `varlookup` | 15,659,908 | 3,598,469 | 63.9 | 277.9 | 214.0 |
| `compound` | 4,348,006 | 714,640 | 230.0 | 1399.3 | 1169.3 |
| `strings` | 3,475,037 | 327,470 | 287.8 | 3053.7 | 2765.9 |
| `alloc4c` | 886,579 | 426,957 | 1127.9 | 2342.2 | 1214.2 |
| `arith` | 432,666 | 160,219 | 2311.3 | 6241.5 | 3930.2 |

The oracle spans 36.2x from its slowest iteration to its fastest.
**This crate now spans 22.5x**, within a factor of 1.6 of the oracle's own span.
At `107febcd` the same figures over the four axes that existed then were 36.1x for the oracle and
**5.5x** for this crate -- a crate whose cost barely moved across workloads whose oracle cost moved
by a factor of 36, which is the signature of one large workload-independent constant swamping
everything else.
That constant is gone, and the hypothesis the spec recorded is now false.

Three single-cause models were tested against that table, and the third is the one that has to be
taken seriously.

* **A constant multiplier.**
  The ratios run 2.08x to 10.61x, a 5.1x range.
  Refuted.
* **A constant additive cost per *iteration*.**
  The differences run 214.0 ns to 3930.2 ns, an 18.4x range.
  Refuted, but this is a straw model and refuting it settles nothing: the axes execute different
  numbers of clauses per iteration, so nobody should expect a per-iteration constant to fit.
* **A constant additive cost per *clause*, which is the model that nearly fits.**
  Loop-body clause counts are `varlookup` 2, `compound` 2, `alloc4c` 3, `strings` 5, `arith` 7,
  giving per-clause differences of 107.0, 584.7, 404.7, 553.2 and 561.5 ns.
  **Four of the five sit inside 1.44x of each other**, and counting the stepped `DO` header as well
  (3, 3, 4, 6, 8) gives 71.3, 389.8, 303.6, 461.0 and 491.3 ns, still inside 1.62x.
  That is a materially better fit than either model above, and it is not an idle one, because C7
  below is literally a per-clause constant.

**The per-clause model is refuted by the prototypes, not by the table, and this is what carries
Step 1.**
In one interleaved run, P1 moved `compound` **-51.6%** and `arith` **-0.7%**.
No per-clause constant can produce that: `arith` executes 3.5 times as many clauses per iteration as
`compound` and moved about seventy times less.
P3 is the same argument from the other direction -- `arith` -16.3% while `strings` and `compound`
got *slower* -- and a constant cannot have two signs.
The effects are axis-specific in size and in direction, which is what a set of workload-specific
causes looks like and what no single constant produces.

**The near-fit is a consequence of the causes, not an alternative to them, and `varlookup` is where
it visibly fails.**
Most of the named causes below scale with clauses because each clause of these programs performs
about one variable write and one or two allocations, so several per-clause and per-assignment costs
aggregating to a similar per-clause figure is what should be expected.
`varlookup`, at 107 ns per clause against 405-585 for the others, is the axis the constant fits
worst and also the axis with the largest single named share in this document (C1 at 32.7%).

So the residual is workload-specific, per-axis decomposition is warranted, and the rest of this
document names causes rather than restating axes.

**The span comparison above is a description of five points, not a fitted law**, and it is not what
the conclusion rests on.
Comparing spans across axes assumes an "iteration" is comparable work between axes, which it is not;
the argument rests on the prototype split, which is measured in one run against one base and needs
no such assumption.

## Method

`samply record --save-only -o <axis>.json ./target/release/rexx-run <program>` at the default 1 kHz,
one profile per axis, each run from a fresh empty directory with `/dev/null` on standard input and
`ulimit -v 8388608` as the baseline harness applies.
Every run exited 0 and printed the bytes the baseline's "same work on both sides" table records.
Analysed through `pollard` with `expand_inlines`, because almost every interesting callee here is
inlined and the enclosing function is not the one paying.

**`unsymbolicated_pct` is a per-run reading, not a property of the binary**, and it is quoted that
way: on the five profiles this document uses it read 0 (`arith`), 0 (`varlookup`), 0.0010%
(`compound`), 0.0076% (`strings`) and 0.0095% (`alloc4c`), and the oracle profile in C6 read 0.0158%.
A re-profile of the same binary will read a different small number.
What the readings establish is that the binary is the one named above, which is a threshold
question, not that any particular figure recurs.

The profiler did not distort the workload: profile durations against the committed baseline's
medians are `alloc4c` 2.344 s against 2.3422 s, `arith` 3.086 s against 3.1207 s, `compound`
6.766 s against 6.9965 s, `strings` 9.194 s against 9.1611 s, `varlookup` 5.258 s against 5.2800 s.

**The wall-clock harness used for the prototypes below is not the baseline's harness, and the
absolute figures are not interchangeable.**
Both compute a ratio the same way -- raw median over raw median, with neither side's fixed
per-process offset netted out -- but the baseline launches each child through a `/bin/sh` `ulimit`
wrapper and times it with `Instant` inside `rexx-bench-suite`, while the runs below use a bash
subshell with the `ulimit` builtin, timed with `date +%s.%N` around it.
The baseline puts the per-process offset at no more than 0.84% of any axis on either side, so this
is a small effect, but it is not a zero one and it is part of why the oracle-interleaved run's base
medians below land between 7.2% under and 0.2% over the committed section's.
**Every claim below is a within-run comparison for that reason**, and no figure here should be
substituted into the baseline's tables.

**A share below is a subtree total unless the text says "self".**
A subtree total is the right unit for a cause, because a cause is a decision whose cost is spread
across the allocator, the hasher and the crate's own code, and self time alone would scatter one
decision across six rows.
Where two causes could overlap, the text says whether they do.

## The cause table

Each row is a claim that can be checked and can be wrong: a mechanism, the axes it predicts it
moves, its share, and the ratio that share implies if the mechanism were removed.
The implied ratio is the committed baseline's ratio multiplied by one minus the share.

| cause | axes it moves | share of this crate's time | implied ratio if removed |
|---|---|---|---|
| C1 variable binding resolved through hash maps | all five | `varlookup` 32.7%, `compound` 14.2%, `strings` 7.6%, `alloc4c` 7.0%, `arith` 4.0% | `varlookup` 2.93x, `compound` 5.22x, `strings` 9.80x, `alloc4c` 1.93x, `arith` 2.59x |
| C2 `/`, `%`, `//`, `**` have no small-integer path | `compound`, `arith` | `compound` 41.5%, `arith` 28.7% | `compound` 3.56x, `arith` 1.93x |
| C3 a number renders and reparses itself to classify itself | `arith` | `arith` 14.5% | `arith` 2.31x |
| C4 a compound access re-derives its tail at run time | `compound`, `alloc4c` | `compound` 12.7%, `alloc4c` 4.0% | `compound` 5.31x, `alloc4c` 2.00x |
| C5 a builtin call is resolved by scanning a table | `strings`, `alloc4c` | `strings` 9.9%, `alloc4c` 5.2% | `strings` 9.56x, `alloc4c` 1.97x |
| C6 every heap value is a fresh `malloc` into a never-swept arena | all five | self time in the glibc allocator: `alloc4c` 39.5%, `strings` 37.7%, `arith` 34.6%, `compound` 30.0%, `varlookup` 4.6% | see C6 below -- this row's share is not removable as stated |
| C7 every stepped clause binary-searches the source line table | all five | `alloc4c` 6.0%, `varlookup` 2.9%, `compound` 1.8%, `arith` 1.6% | `alloc4c` 1.96x, `varlookup` 4.23x, `compound` 5.97x, `arith` 2.66x |

**Do not sum these shares.**
Two overlaps exist and both would be double-counted by a bar derived from two rows at once.

* **C1 and C4 overlap on `compound` and `alloc4c`.**
  A compound access resolves its tail piece by name through the same `Plan::slot_of` C1 names, so
  C4's 12.7% on `compound` contains part of C1's 14.2% there.
  Among C1 to C5 and C7 that is the only shared call site.
* **C6 overlaps every other row, by construction.**
  C6 is *self* time in the allocator while C1 to C5 and C7 are subtree totals, and every one of
  those subtrees allocates, so part of each axis's allocator share is already inside them.
  C3 and C5 say so explicitly in their own sections; it is true of all of them.

**C6 is also the one row whose share is not a prediction**, and the difference matters enough that
it is kept in a different column shape rather than smoothed into the others.
See C6 for why.

### C1 -- variable binding is resolved through a hash map on every access, and an assignment resolves by copying and hashing the name text

Two lookups, both hash tables, on paths the oracle serves with an array index.

* A read goes through `code.slots`, a `HashMap<SymbolId, usize>` (`lib.rs:1057`), consulted at
  `lib.rs:1970`.
* An assignment does not use it at all.
  `assign_expr_target` copies the target's interned name into a fresh `Vec<u8>` (`run.rs:2529`) and
  looks *that* up in `Plan::names`, a `HashMap<Box<[u8]>, usize>` (`plan.rs:110`, read at
  `plan.rs:565`).
  So a write costs a byte-string SipHash, a `memcmp` against the stored key, and an allocation,
  where a read costs a `SymbolId` SipHash.

Measured on `varlookup` (`do i = 1 to n; x = x + 1; y = x; end`), the two halves are disjoint
subtrees and their shares add:

| path | share | reached from |
|---|---:|---|
| `Interp::slot_of` -> `Plan::slot_of` (name-keyed) | 20.2% | `assign_expr_target` (20.3%) |
| `HashMap<SymbolId, usize>::get` (id-keyed) | 12.5% | `Interp::read` (18.2%) |

The independent check is the module total: `pollard` groups 31.7% of `varlookup`'s samples under
`hashbrown`, and `Plan::slot_of`'s own 19.2% plus the id-keyed map's 12.5% is exactly that; the
remaining point of `Interp::slot_of`'s 20.2% is its `extra`/`grow_slots` fallback, which is not a
`hashbrown` frame.
The `memcmp` those lookups call is inside that figure -- 26.8% of `varlookup`'s `memcmp` samples sit
under `equivalent_key`, comparing a variable name against a stored key.

The oracle does not pay this.
The C++ profile at
`/home/moritz/.claude/projects/-home-moritz-dev-repos-ooRexx/memory/oorexx-performance-profile.md`
records that "locals are already integer slots assigned at parse time (`RexxLocalVariables`,
`locals[index]`, 0 = miss)".
This crate assigns the slots at plan-build time too, and then throws the index away at the
assignment site and re-derives it from the name.

Per-axis shares of `Interp::slot_of` plus the id-keyed map: `varlookup` 20.2 + 12.5, `compound`
10.6 + 3.6, `strings` 3.7 + 3.9, `alloc4c` 4.6 + 2.4, `arith` 2.4 + 1.6.

**Confirmed by prototype P2, below: -10.7% on `varlookup`, which is half the claim.**
P2 replaces the name-keyed lookup with the id-keyed one rather than with an array index, so it
collects the 20.2% and pays an id-keyed lookup back on every assignment.
The 12.5% still in the id-keyed map is bounded but not prototyped, so read 3.89x as the confirmed
figure for `varlookup` and 2.93x as the floor if that half went too.

### C2 -- `/`, `%`, `//` and `**` have no small-integer path

`small_int_arith` (`eval.rs:961`) matches `Plus`, `Subtract` and `Multiply` and answers `None` for
everything else, so an integer remainder of two tagged small integers takes the general decimal
path: both operands are converted to a `Number` with a digit vector, `Number::div` long-divides,
and the result is allocated as a heap object.

`compound.rex`'s inner loop is `k = i // tails; t.k = t.k + 1`, and **`Number::div` is 41.5% of
that axis** -- more than the entire compound-variable machinery the axis is named for.
On `arith`, `Number::div` is 28.7%, from `i / 3`, `i / 7` and `d = c ** 2 // 5`.

This is the sharpest instance of the brief's warning that the benchmark programs are not axis-pure.
An attribution that read `compound`'s 6.08x as a statement about stems would be wrong by a factor
of about two.

**Confirmed by prototype P1: -51.6% on `compound`.**
The measured win exceeds the 41.5% share, and the reason is that the share is the division alone
while the prototype also removes the operand conversion, the result allocation and the root pushes
that surround it -- all of which are attributed to their own frames in the profile.
`arith` moved -0.7%, inside noise, and that is the prediction rather than a miss: `arith`'s
divisions have non-integer operands, so a small-integer path cannot reach them.
`arith`'s 28.7% is therefore real and *not* addressable by this fix, which is a fact 4d-2 needs
before it schedules one task for both axes.

### C3 -- a number that is not a plain integer renders itself to a decimal string, parses it back, and throws it away

`small_int_for` (`value.rs:455`) decides whether an arithmetic result can be a tagged small integer.
It answers cheaply when `Number::plain_integer` accepts, and otherwise falls to `value.rs:468`,
which calls `format_form` -- a full decimal-to-text render into a fresh `String` -- scans the
result for `.` and `E`, parses it back to an `i64`, and discards the string.

Every arithmetic result on `arith` reaches that line, because every one of them is fractional.
`Number::format_with` under `Interp::number` accounts for **14.5% of `arith`'s samples**, and the
`malloc` and `memcpy` under it are inside that figure.

**Confirmed by prototype P3: -16.3% on `arith`.**

**The probe is not dead work everywhere, and P3 is a diagnostic rather than a proposal.**
The same prototype made `compound` 1.6% and `strings` 2.7% *slower*, and in both cases the
prototype's fastest run was slower than the base's slowest, so both are real rather than noise.
The probe therefore does accept values on those axes that `plain_integer` declines, and removing it
turns those into heap objects.
What C3 claims is that on `arith` the probe is pure cost; it does not claim the probe should go.

### C4 -- a compound access re-derives its tail from the source spelling at run time

This is Step 4's question and it has its own section below.
`Interp::tail_key` (`stem.rs:111`) is 12.7% of `compound` and 4.0% of `alloc4c`;
`rexx_parse::compound_parts`, the split, is 9.9% and 2.7% of those.

### C5 -- a builtin call is resolved by validating UTF-8, testing a hash set, and linearly scanning the builtin table

`builtin::dispatch` calls `is_builtin` (`builtin/mod.rs:622`), which runs
`std::str::from_utf8(name).is_ok_and(|name| in_scope().contains(name))`, and then scans the
implemented table with `IMPLEMENTED.iter().find(|builtin| builtin.name == name)`
(`builtin/mod.rs:680`).
Both run on every call, and both answer a question the parser already had the information to
answer.

On `strings`, whose loop calls `pos`, `substr`, `changestr` and `length` once each per iteration,
`is_builtin` is 4.7% and the linear scan is 5.2%, for **9.9%**.
They are sibling subtrees of `dispatch` and do not overlap.
For scale, `dispatch`'s whole subtree is 33.4% of that axis, so about three tenths of the time
spent in builtins is spent deciding which builtin to run.
On `alloc4c`, whose loop calls `length` once, the two are 2.1% and 3.1%.

Not prototyped.

### C6 -- every heap value is a fresh glibc allocation into a never-swept arena, and this is the cause that does *not* explain the gap

Self time in the glibc allocator family -- `int_malloc`, `int_free_chunk`, `tcache_*`,
`unlink_chunk`, `malloc_consolidate`, `sysmalloc`, `realloc`, `free`, and the `mmap`/`munmap`/
`mprotect` the allocator issues -- measured by grouping every matching frame by module:

| axis | glibc allocator, self | of which `mmap`/`munmap`/`mprotect` | this crate's arena side, self | baseline ratio |
|---|---:|---:|---:|---:|
| `alloc4c` | 39.5% | 3.5 pp | 4.6% | 2.08x |
| `strings` | 37.7% | 5.9 pp | 8.5% | 10.61x |
| `arith` | 34.6% | 1.9 pp | 4.1% | 2.70x |
| `compound` | 30.0% | 0.0 pp | not separated | 6.08x |
| `varlookup` | 4.6% | 0.2 pp | negligible | 4.35x |

"This crate's arena side" is `Heap::alloc_with_uncollected` plus `core::ptr::write::<Slot>`, the
push of a 96-byte `Slot` into `Heap.slots`.

**The syscall column is disclosed because the grouping choice raises exactly the endpoint the
finding below rests on**, and it is given so a reader can recompute without it.
Excluding the three syscalls the ordering is `alloc4c` 36.0%, `arith` 32.7%, `strings` 31.8%,
`compound` 30.0%, `varlookup` 4.4% -- `arith` and `strings` swap, and both endpoints hold: `alloc4c`
is still highest and `varlookup` still lowest.

**Read the allocator column against the ratio column, because that is the finding.**
The axis with the *highest* allocator share is the axis *closest* to the oracle, and the axis with
the *lowest* allocator share sits in the middle of the ratio range.
Across the five axes the allocator share and the ratio do not move together at all.
So a gate whose bars were derived from an allocation story would be derived from a quantity that
does not predict the thing being gated.

#### Why `alloc4c` is 2.08x, and why that is a debt rather than headroom

The finding above is about ordering, and it does not by itself explain the endpoint.
The explanation is in the *denominator*, and it required profiling the oracle, which this document's
first version did not do.

**The oracle on `alloc4c` spends most of its time collecting, and this crate spends none, because
this crate never collects.**
Profiled on the same program, same wrapper, 1057 ms and 1080 samples:

| oracle function | share |
|---|---:|
| `MemoryObject::newObject` | 69.1% total |
| `MemoryObject::collect` | 56.0% total |
| `MemoryObject::markObjects` / `markObjectsMain` | 46.6% total |
| `CompoundTableElement::live` | **20.7% self**, 30.3% total -- the largest self-time function in the profile |
| `MemorySegmentSet::sweep` | 9.4% total |
| `RexxInteger::live` | 4.4% self |

`CompoundTableElement::live` is the mark phase walking `tab.`'s tail table, and `alloc4c.rex`'s own
header says that table is "a genuinely live, growing table on any interpreter".
The oracle re-marks all of it on every collection; this crate marks nothing, ever.

**Confirmed a second way, without a profiler.**
The same program against one identical but for `tab.1 = i` in place of `tab.i = i`, so the tail table
stops growing and everything else is unchanged.
Five repetitions per cell, interleaved, same wrapper, all runs exit 0, and all four cells print
`12888896`:

| | oracle | this crate | ratio |
|---|---:|---:|---:|
| growing tail, as `alloc4c.rex` is written | 1077.4 ms | 2338.2 ms | **2.17x** |
| flat tail | 184.1 ms | 1564.8 ms | **8.50x** |

Removing the growing table makes the oracle **5.85x** faster and this crate **1.49x** faster.
So **82.9% of the oracle's time on this axis is work the growing table causes it to do**, and with
that work removed the axis reads about 8.5x -- the same neighbourhood as `strings`.

**2.08x is therefore a debt, not headroom, and it will get worse when a collector lands.**
The two sides are not paying for the same thing: the oracle pays mark and sweep over a live, growing
table, and this crate pays `malloc` and then never reclaims anything.
"A shared cost" -- which this document's first version claimed, citing
`MemoryObject::newObject` at 26% of a *different* benchmark from the C++ memory file -- inverts what
is happening, and the comparison was not sound either, since that 26% is a subtree total on another
program set against this crate's self time on this one.
The on-axis figure is 69.1%, and it takes one profile to get.

**`phase-4d-retention.md`'s collection-trigger prototype covered `strings`, `arith`, `compound` and
`varlookup`, and not `alloc4c`** -- the one axis where a collector has a large live set to re-mark
and little to reclaim, and therefore the one axis where its cost is least offset.
Whoever lands a trigger policy in 4d-2 should measure `alloc4c` before and after and expect the
ratio to move the wrong way.

This is also the direct answer to the flag `perf-baseline.md` raises under "`alloc4c`: the closest
axis, and what that means", and it is the second of the two explanations that section offers rather
than the first.
`alloc4c` does not undermeasure this crate's allocation -- it has the largest allocator share of any
axis -- but its *denominator* is inflated by collector work this crate does not perform, so 2.08x
overstates how close this crate is on that dimension.

**The share in this row is not a prediction, and no ratio is implied from it.**
Removing "the allocator" is not a change anyone can make; what can be changed is how many
allocations happen and what happens to them afterwards, and those are two separate levers with two
separate measured values:

* **Collection.** `phase-4d-retention.md` measured a trigger policy, interleaved and reverted, at
  **16% faster on `strings`** and no movement its data could distinguish from noise on `arith`,
  `compound` or `varlookup`.
  That implies `strings` 10.61x -> 8.91x and nothing on the other three.
  It measured **nothing on `alloc4c`**, which did not exist as an axis when that work was done, and
  which the oracle profile above says is the axis where collection costs most and reclaims least.
  A collector is a win on `strings` and a cost on `alloc4c`, and only the first half is measured.
* **Allocation count.** A collector does not reduce it.
  C1, C2, C3 and C4 each remove allocations as a side effect, which is why P1's measured win on
  `compound` exceeds its profiled share.

Task 7 owns the allocator-swap diagnostic and is the place to bound what remains after those.

### C7 -- every stepped clause binary-searches the source line table

`step_in_temps_frame` computes the current clause's source line on every stepped instruction
(`run.rs:4203`), through `clause_line` (`run.rs:6906`) and `ProgramSource::line_of`, which is a
binary search over the line-offset table.
It is unconditional because `SIGL` must stay correct whether or not `TRACE` is on, which the code's
own comment states.

It is small and it is everywhere: 6.0% of `alloc4c`, 2.9% of `varlookup`, 1.8% of `compound`, 1.6%
of `arith`.
The oracle's instruction objects carry their line number, so this is a per-clause constant this
crate pays and the oracle does not.

Not prototyped.

## Step 4: Was D9's compound-memoisation mandate met?

**No.**
`2026-07-27-rust-rewrite.md:389` says to build memoisation into the Rust stem and compound-variable
design "from the start rather than porting the slow shape first and optimising later".
The slow shape was ported first.

Read the code rather than the ratio.
`Interp::tail_key` (`stem.rs:110`-`125`) runs, on **every** compound access:

* `compound_parts(code.symbols.name(id))` (`stem.rs:111`), which splits the interned spelling on
  `.` and collects a fresh `Vec<Tail>`;
* a fresh `Vec<u8>` for the key, extended piece by piece;
* for each `Tail::Variable` piece, `read_by_name` (`stem.rs:145`), which resolves the piece through
  `Interp::slot_of` -- the **name-keyed** hash map of C1, not the piece's own slot -- and then
  renders the piece's value with `to_text`.

Nothing on that path is memoised: not the split, not the key buffer, not the piece's slot, not the
rendered piece.
The one thing that *is* cached is the `Plan` per body (`plan_for`, `plan.rs:575`), and the stem's
own `tails` map, neither of which is compound-access memoisation.

The split is in fact already performed once at plan-build time, and the result is discarded.
`Plan::note_compound_name` (`plan.rs:439`) calls the identical `compound_parts` to register slots
for the stem and each variable piece, and its own doc comment states the consequence plainly:
"that is exactly why `stem.rs`'s `tail_key`/`read_by_name` resolve both of these purely by name".
So this is a recorded design choice, not an oversight, and the choice is the one D9 asked not to
make.

**What it costs, measured:** `tail_key` is 12.7% of `compound` and 4.0% of `alloc4c`, of which the
split alone is 9.9% and 2.7%.
`compound`'s stem machinery proper -- `stem_get` 4.7% and `stem_set` 4.8%, whose tails-map probes
are 2.2% and 2.0% -- is smaller than the cost of working out which tail to touch.

**The prior profile D9 `:390` points at is the C++ interpreter's**, and it is not in this
repository.
It constrains where to look and it is not a profile of this crate.
Two of its findings bear on this section.
It records that the oracle's own compound tails are a balanced BST rather than a hash table, with
`memcmp` from `CompoundVariableTable::findEntry` alone at 21.6% on stem-heavy code -- so the oracle
is slow here too, which is part of why `compound` is 6.08x rather than worse.
And it records the unmerged -24% memo prototype that D9 `:389` is quoting.

## Step 5: The prototypes, published and reverted

Three prototypes, each a few lines, each built at `--release` under the pinned profile, each run
against the full test suite, each measured interleaved against the base binary, and all three
reverted in this commit.

| prototype | file | change |
|---|---|---|
| P1 | `eval.rs` | add `IntDiv` and `Remainder` to `small_int_arith`'s match, via `checked_div`/`checked_rem` |
| P2 | `run.rs` | `assign_expr_target`'s `Variable` arm resolves through `code.slots.get(id)` first, falling back to the name path, and builds the name only when tracing needs it |
| P3 | `value.rs` | `small_int_for` returns `None` immediately when `plain_integer` declines, skipping the render-and-reparse probe |

**Every prototype passed the whole suite: 1317 tests, 0 failures, exit 0, individually and in
combination.**
That is evidence and not a safety proof, and P1 and P3 both change which values become tagged small
integers, so neither is proposed as a fix here.

**Every measurement is interleaved**, base and prototype alternating within each axis, because two
separate runs on this machine have invented a 5% effect that was not there
(`benchmark-comparisons-must-interleave`).
Every run had a fresh empty working directory, `/dev/null` on standard input, `ulimit -v 8388608`,
and its stdout and exit status read as separate descriptors.
**All four binaries printed byte-identical stdout on all five axes and all exited 0**, so the
timings compare the same workload.

Five repetitions per cell, median reported, change against the base median in the same run:

| axis | base | P1 | P2 | P3 |
|---|---:|---:|---:|---:|
| `alloc4c` | 2.2931 s | -1.7% | -3.0% | -0.8% |
| `arith` | 3.0794 s | -0.7% | -2.2% | **-16.3%** |
| `compound` | 6.7913 s | **-51.6%** | -2.9% | +1.6% |
| `strings` | 8.9971 s | -1.2% | -3.2% | +2.7% |
| `varlookup` | 5.2149 s | +0.2% | **-10.7%** | +0.5% |

**Only the bold cells carry a claim.**
Within-cell spread is 0.5% to 2.6%, so at five repetitions a sub-5% difference is not separated
from noise except where one binary's whole range clears the other's, which holds for `strings` and
`compound` under P3 and for nothing else in the unbolded cells.

### The combined prototype, measured against the oracle in one interleaved run

A second run took the oracle, the base binary and a binary carrying all three prototypes,
alternating within each axis, five repetitions, same wrapper on every side.
This produces the implied ratios directly rather than deriving them.

| axis | oracle | base | ratio | combined prototype | ratio |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | 1.1813 s | 2.2765 s | 1.93x | 2.2836 s | 1.93x |
| `arith` | 1.1676 s | 3.0774 s | 2.64x | 2.5068 s | **2.15x** |
| `compound` | 1.1485 s | 6.7831 s | 5.91x | 3.2859 s | **2.86x** |
| `strings` | 0.8593 s | 9.0533 s | 10.54x | 8.7969 s | 10.24x |
| `varlookup` | 1.1970 s | 5.2147 s | 4.36x | 4.6874 s | **3.92x** |

Every run exited 0, and stdout was byte-identical across the oracle, the base and the combined
prototype on every axis.

**This run's base ratios are not the committed baseline's, and the gap is the open variance
question rather than a change in the crate.**
The binary is byte-identical to the baseline's and so are all three oracle objects.
Against the committed 2.08x / 2.70x / 6.08x / 10.61x / 4.35x, this run reads 1.93x / 2.64x / 5.91x
/ 10.54x / 4.36x -- `alloc4c` -7.2%, `arith` -2.2%, `compound` -2.8%, `strings` -0.7%, `varlookup`
+0.2%.
`perf-baseline.md` already records `compound` producing disjoint intervals across two quiet runs of
a byte-identical binary, and states that between-run variance there exceeds the within-run interval
the harness reports.
This run is a third data point for that, and it moves the finding: the axis that moved most here is
`alloc4c`, not `compound`, so the instability is not a property of one axis.
**Every prototype figure in this document is a within-run comparison for exactly that reason**, and
the ratios in the table above should be read as "this run's base against this run's prototype",
with the absolute level carrying whatever the between-run variance turns out to be.

## What this attribution says, in one paragraph

The five axes are not five instances of one problem and they are not five separate problems either.
Two mechanisms -- C1's hash-keyed variable binding and C2's missing small-integer path for the
remaining four operators -- account for the largest single share on three of the five axes, and both
are the same shape of defect: work that the parser or the plan already did, thrown away and redone
per iteration.
C3, C4, C5 and C7 are four more instances of that same shape, at 14.5%, 12.7%, 9.9% and up to 6.0%.
C6, the allocator, is the one large share that is *not* that shape, and it is also the one whose
per-axis share does not order the axes by ratio -- not because the cost is shared, but because
`alloc4c`, the axis that would otherwise anchor an allocation story, has a denominator inflated by
collector work this crate does not perform.
Read C6 before deriving any bar from an allocation figure.

## What is not attributed here

* **`arith`'s 28.7% in `Number::div`.**
  Its operands are not integers, so C2's fix cannot reach it, and nothing here separates irreducible
  decimal-division work from an implementation this crate could improve.
  The C++ profile's own `b3_decarith` row puts 73.5% of the oracle's time in arithmetic on a decimal
  benchmark, so some of this is shared cost.
* **Most of `strings`, which is the worst axis and is not ready to carry a bar.**
  Stated as the number Task 8 would need rather than as a gap: of 10.61x, the **named and removable**
  share is C1's 7.6% plus C5's 9.9%, which is **17.5%**, implying 8.75x at best.
  The only thing actually measured against it is the combined prototype's **2.8%**, a factor of six
  short, and that 2.8% is itself marginal at n=5.
  Everything else is C6 (37.7% self in the allocator, whose removability this document does not
  establish) and the builtin bodies -- `changestr` 12.2%, `pos` 5.8%, `substr` 5.0% -- which are work
  the oracle performs too and which nobody has opened.
  So roughly a sixth of `strings` is attributed to a mechanism with a named fix, and a bar derived
  from this document for that axis would be a bar over the unattributed five sixths.
* **Which of C1's two halves the oracle's array index would remove**, beyond the bound P2 measured.
* **The interaction between causes.**
  P1, P2 and P3 were measured individually and in combination; the combination's effect on
  `compound` (-51.6%) equals P1's alone, and on `arith` (-18.5%) is P3's -16.3% and P2's -2.2%
  added, so no interaction was visible, but three prototypes on five axes cannot establish that
  there is none.

## Reproducing

```sh
cd rust
cargo build --offline --release -p rexx-exec --bin rexx-run
samply record --save-only -o <axis>.json ./target/release/rexx-run bench-programs/<axis>.rex

# the oracle side of C6, which needs LD_LIBRARY_PATH and gives symbols from librexx.so.4
LD_LIBRARY_PATH=<oracle>/lib samply record --save-only -o oracle-alloc4c.json \
    <oracle>/bin/rexx <abs>/alloc4c.rex
```

Run each from a fresh empty directory -- the scratchpad is on the oracle's external-routine search
path -- and check `unsymbolicated_pct` on load before reading a share off the profile.

C6's flat-tail control is `alloc4c.rex` with `tab.i = i` replaced by `tab.1 = i` and nothing else
changed; both variants print `12888896`, which is what makes the pair comparable, and both sides must
be run interleaved because the oracle's figure moves by a factor of six between them.

The prototypes are throwaway and are not committed; each is the one edit its row in the table above
describes.
