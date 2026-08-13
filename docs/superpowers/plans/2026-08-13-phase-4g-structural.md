# Phase 4g: the structural work

> **For agentic workers:** each unit below is a plan of its own to be written when the unit is reached. This document decides the *set*, the *order* and the *rules*; it deliberately contains no task steps, because every unit needs a design and a spike before it can have any.

**Goal:** close the part of the oracle gap that local optimisation cannot reach, by changing five per-clause and per-value costs rather than by finding more hot spots.

**Status:** written 2026-08-13, after the condition-promotion and compound-name plans. Not started.

## The number this phase exists for

Measured 2026-08-13, three runs each, `samples/rexxcps.rex` at `count=100`/`averaging=100`, both sides self-calibrating and the clauses-per-second figure the comparable one:

| | clauses/sec |
|---|---:|
| oracle | 16,329,757 / 16,551,003 / 16,584,147 |
| this crate at `50faeff88` | 3,593,890 / 3,605,358 / 3,609,981 |

**4.6x.** At the start of the session that produced the two most recent plans it was 5.4x, and those two plans -- four landed changes, both measured and reviewed -- moved 15.5% of runtime, which is **1.6 of the roughly fourteen successive 10% wins** parity needs.

## Why more local wins cannot get there, stated once

`phase-4d-diagnosis.md` attributed seven causes and found that **landed perfectly they leave four axes between 1.62x and 2.80x**, because every one is local: a hash lookup, a missing small-integer path, a render probe, a line-table search.
The two recent plans added four more of the same kind.

The current `rexxcps` profile says it arithmetically. Numbers is about 25% of self time, the driver about 20%, allocation and collection about 10%, expression evaluation about 11%, name hashing about 6%.
Parity needs 75% of runtime removed. **Zeroing the two largest buckets entirely gives 1.8x.**
So the remaining gap is not in any bucket; it is a tax on every clause and every value, and there are five of them.

## The five, each measured or read from the C++ rather than inferred

| the oracle | this crate | where it shows |
|---|---|---|
| `FAST_BUFFER = 48` stack buffers; at `DIGITS 9` and `20` **no allocation per arithmetic operation** | `rexx-num` allocates a `Vec<u8>` per intermediate -- `truncated_to`, `aligned_to`, the magnitude, `round_to`, `assemble` | the numbers bucket, about 25% |
| expression stack is a bump pointer into a preallocated array, scanned in place by the collector | a `RootSet` frame pushed and popped **per clause and per `eval` site** | the driver bucket, and every value produced |
| C++ exceptions cost nothing on the happy path | `Result` through five layers, each a checked branch | `Result::branch` alone was **4.6%** of the empty-loop profile |
| `this` **is** the activation, every field a direct offset | `self.activations.last().expect(...)`, six-plus times per clause | the driver bucket |
| the clause's line is a field set at parse time | a binary search in `ProgramSource::line_of`, per clause | `indent_in_range` was once **8.0%** of a profile for the same reason |

## The units, in order, and why this order

Ordered by mechanism confidence first and share second, so the cheap certainties bank early and reduce the noise the expensive ones must be measured through.

**A bound over Units 1, 2 and part of 5, measured and easy to forget.** A build with **all** per-clause bookkeeping stripped bounded the win at **19% on `varlookup` and about 5% elsewhere** -- and it could not run `rexxcps` at all, because removing `clock_stale` breaks `TIME('R')` into a divide-by-zero, which incidentally proves the bookkeeping is load-bearing rather than ceremonial.
So Units 1 and 2 are worth single digits on most axes and are ordered first for confidence and for noise reduction, **not** because they are large. Anyone who reads this order as a ranking by size has misread it.

**Unit 1 -- the clause line.** A field or a plan-side table replacing a per-clause binary search. **This is the fix this project has already made twice** (`static_indent`, and the compound-name family's four applications): a constant of the source text computed once in the pass that already walks it. Smallest, highest confidence, and it makes the driver bucket cleaner to read.

**Unit 2 -- the activation.** Hoist the current activation so a clause pays one resolution rather than six-plus. Bounded, local to the driver, no representation question.

**Unit 3 -- the error path.** The five-layer `Result` chain. **This is the first unit that may need the divergence licence** (see below): the happy-path cost is the chain's shape, and some of the shape exists to reproduce the oracle's exact failure sites. Design first, and say plainly which part is mechanism and which is semantics.

**Unit 4 -- arithmetic working storage.** The `FAST_BUFFER` analogue. Largest single share.
**The `smallvec` figures exist and are recorded here, because the repository lost them.**
`phase-4f-record.md:284` records that searches of the working tree, of `git log -S` over all refs and of `.superpowers/` could not produce them. They were held outside the repository. They are:
**measured across four inline sizes with `union` and `const_generics`, interleaved, `Vec` wins every cell** -- `arith` +18% to +30%, a digits-9 decimal loop +14% to +21%, and **N=64, which inlines every working length either workload produces, is still 18% behind.**
So inline capacity is not the variable, and the conclusion is the one this unit is built on: **the fix is not cheaper allocations, it is not allocating** -- a scratch buffer threaded through the operation, which is what `FAST_BUFFER` is.
`smallvec` 1.15.2 resolves offline from the local registry cache if anyone revisits it.
And entry's own reasoning against it stands independently and is stronger: on `strings`, `Number` allocations are reached *because* a counted answer arrives untagged and forces the general decimal path, so **removing the reason removes them wholesale where making each one cheaper leaves the path, the parse and the render in place.**

**Where the storage lives is open, and Moritz named a candidate on 2026-08-13: a thread-local pool.**
It answers the objection that makes a threaded scratch parameter unpalatable, which is that `rexx-num`'s API is value-typed and the executor calls into it from deep inside expression evaluation, so `&mut scratch` would rewrite every signature across a crate boundary.
A per-thread pool also suits where this crate is going, since the oracle is thread-per-activity and per-thread storage has no contention to design around.
Three things decide it and none is settled by reading:

* **Arithmetic nests**, so a single shared buffer corrupts when an operand's own computation is arithmetic. It has to be take-and-return, which is a small allocator rather than one cell.
* **The access may cost more than the reuse saves.** A non-`Copy` payload wants a `RefCell`, so every access pays a borrow check and a panic path, and lazy init adds a check unless the initializer is `const`. The `smallvec` result above is the standing warning: inline capacity that inlined every working length either workload produces was still 18% behind `Vec`, so cheaper-looking storage has already been wrong once on this code.
* **It needs a ceiling to be read against.** A throwaway arm that threads `&mut` scratch through the operation measures what perfect reuse buys with no access overhead at all. Without it, a thread-local arm at any figure cannot be read as "the access is eating it" rather than "the reuse was not worth much".

So the spike is three arms -- status quo, thread-local pool, threaded parameter -- on `arith` and on a digits-9 decimal loop, with the third arm existing only to bound the other two.

**Unit 5 -- rooting.** Replace the per-clause and per-`eval`-site `RootSet` frame with a bump-pointer stack the collector scans in place. Biggest, riskiest, touches the collector's contract with every value, and `2026-08-11-value-representation-design.md` already sets out options for the value layer -- **that document is this unit's starting point and must not be re-derived.**

## The divergence licence, and how it is spent

Moritz granted, 2026-08-13: **limited divergence from the reference is permitted, error semantics and trace details named as examples.**

This is a licence to spend, not a general permission, and it is spent as follows.

* **Every divergence taken lands as a row in the DEVIATIONS section of `docs/superpowers/plans/phase-4-exclusions.txt`**, which exists for exactly this: a permanent difference chosen on purpose, as against an exclusion which is work a later phase owes. That file's own rule is that adding a row is a plan amendment rather than a file edit, so each row arrives with the unit that earned it.
* **A row names what a program can observe**, not what the code does differently. "Error semantics" is not a row; "a `SYNTAX` condition raised inside a callee reports the callee's line where the oracle reports the caller's" is.
* **A divergence is justified by a measurement, not by convenience.** The unit that takes one states what identity cost, measured, and what the divergence bought. A divergence worth less than the resolution floor is not taken.
* **The corpus differential stays the gate for everything else.** The two engines must still agree with each other byte for byte, always -- divergence is from the *oracle*, never between our own engines, and never silent.

## The rules this phase inherits, and the one it changes

Inherited unchanged: the oracle is read-only and wrapped; a comment may not name a set's size; a test row is a measured witness or a labelled transcript; corrections sweep the claim's vocabulary rather than the instances named; the record is appended to, never rewritten.

**Changed: the instrument.** `phase-4f-record.md`'s configuration block fixes wall clock through `rexx-bench-suite`. Entry 27 measured the wall clock on these axes moving between -5.12% and +4.55% from layout alone, wider than most units here will produce, while the instruction counter's arm-internal spread stayed at or below 0.005%.
**So every unit reports `perf stat -e instructions:u` as its primary figure**, with each axis's own spread measured before any difference under a percent is quoted, and every arm staged at one fixed binary path -- four byte-identical copies of one binary once spanned 3.47% on wall clock purely because their `argv[0]` basenames differed in length.
Wall clock is reported only where a unit is large enough for it to resolve, and then under the accept rule with two do-nothing controls.

## What would make this phase stop

**Parity is not obviously the right target and this document does not assume it.**
4.6x against a mature C++ implementation, with byte-identical output across the corpus and two engines held in agreement, is a different thing from 4.6x against a toy.
Each unit is independently valuable and independently revertible, so the phase can stop after any of them.
The question worth re-asking after Unit 3, when the cheap certainties are banked and the two architectural units are all that is left: **is the remaining gap worth a rewrite of the value layer, or is the crate better spent on Phase 5's object model?**
That is a decision for Moritz, and this document exists partly so it can be taken with the first three units' numbers in hand rather than in advance.
