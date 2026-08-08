# Phase 4d diagnosis -- where the interpreter's time and memory go

Measured 2026-08-08 at `9bcbfeda`, release build with the pinned profile, this machine.
Driven directly rather than through the task plan, because the work is iterative and a linear task list was the wrong shape for it.

**Cause 3 and its two open questions are superseded by `phase-4d-retention.md`, measured at
`c9a90906`.**
Three statements below are false at that commit, and the corrections are there with their evidence:
this is a missing trigger policy **and also** a root leak, in `Interp::loop_advance`, which a
collector cannot fix; the standard `ulimit -v 1048576` now aborts two of the five axes rather than
four; and both "what is not diagnosed" items about the arena and the 128-byte figure are answered.
Everything else here, including the four causes and the per-axis attribution, stands as measured at
`9bcbfeda`.

## Summary

**Four causes, three of them compounding.**
The headline per-axis ratios -- `varlookup` 23x, `strings` 14x, `compound` 12x, `arith` 3.5x -- are mostly one cause wearing four names.

| cause | cost | evidence |
|---|---|---|
| 1. `DO` control-variable arithmetic | **612 ns/iteration, 92% of an empty counted loop** | repeat-count vs counted loop, below |
| 2. Literal evaluation allocates per evaluation | ~170 ns and 128 B each | body-variant table, below |
| 3. Nothing triggers a collection | turns 1 and 2 from churn into unbounded RSS | two call sites, below |
| 4. Residual interpretation overhead | ~5.2x | repeat-count loop, which isolates it |

Causes 1 and 2 produce garbage; cause 3 means it is never reclaimed; cause 4 is what remains once the others are gone.
**Fixing memory alone would take a counted loop from about 23x to about 13x, not to parity**, because cause 1 is arithmetic, not allocation -- although cause 1's fix removes most of the allocation too, since the allocation is a *consequence* of doing decimal arithmetic.

## Cause 1 -- every `DO i = 1 TO n` iteration does two arbitrary-precision decimal operations

The single largest lever, in the most common construct in the language.

Measured at five million iterations with a body of `nop`:

| loop form | rust | oracle | ratio | peak RSS |
|---|---|---|---|---|
| `do 5000000` | 0.26 s | 0.05 s | 5.2 | 2,816 KB |
| `do i = 1 to 5000000` | 3.32 s | 0.19 s | 17.5 | 41,452 KB |

The control variable costs **3.06 s of the 3.32 s**, or 612 ns per iteration.
The oracle pays 28 ns for the same thing, so this crate is roughly **22 times worse specifically at stepping a counter**.

**Why.** The profile's stacks show two independent decimal operations per iteration:

```
run_repeating -> compare_decoded -> add_signed -> round_to  -> malloc
run_repeating -> add_signed      -> aligned_to -> malloc -> int_malloc
```

The first is the bound test: `compare_decoded` is implemented as a subtraction.
The second is the increment.
Each runs the general `rexx-num` path -- exponent alignment through `aligned_to`, rounding through `round_to` -- and each allocates.

On a `nop` loop, `Number::add_signed` accounts for **49% of total samples** and the glibc malloc family for about **42% of self samples**, with `memcpy` at 12.4% and `render_integer_padded` at 9.5% total.
An empty loop is parsing, adding, rounding, rendering and allocating.

**There is no integer fast path.** Rexx semantics do not require one to be visible: under the default `NUMERIC DIGITS 9` a control variable stepping by a whole number stays a whole number, and the oracle exploits exactly this.

## Cause 2 -- evaluating a literal allocates a fresh object every time

`eval.rs:314`:

```rust
ExprKind::Literal(bytes) => Ok(self.text(bytes)),
```

`text` calls `text_owned(bytes.to_vec())`, so each evaluation allocates a heap object **and** copies the bytes, even though `bytes` is already in the AST and Rexx strings are immutable.
`ExprKind::Constant` at `:317` does the same.

Measured at one million iterations, isolating each effect:

| loop body | peak RSS | per iteration | wall | delta over `nop` |
|---|---|---|---|---|
| `nop` | 10,188 KB | ~0 | 0.66 s | -- |
| `y = x` | 10,496 KB | ~0 | 0.73 s | +70 ns |
| `y = 5` | 134,868 KB | ~128 B | 0.83 s | +170 ns |
| `x = x + 1` | 213,192 KB | ~208 B | 1.34 s | +680 ns |
| `z = 1 + 1` | 415,384 KB | ~415 B | 1.50 s | +840 ns |

`y = x` is the control that makes this readable: **assigning an existing value allocates nothing**, so the cost is in producing values, not in binding them.

## Cause 3 -- nothing ever triggers a collection

`heap.collect` has exactly two production callers:

* `Interp::alloc_with` (`rexx-exec/src/lib.rs:2091`), **gated on `self.stress_collect`**, which is the test-only stress mode.
* the user-callable `GC('Force')` builtin (`builtin/state.rs:234`).

There is no allocation-count threshold, no heap-size threshold, and no other trigger.
A normal program never collects and the heap grows monotonically until the process dies.

**The collector works.** Two million iterations of `x = x + 1; y = x`, both runs exiting 0 with correct output: 423,892 KB plain against 122,880 KB when forcing `gc('Force')` every hundred thousand iterations.

This is a **missing trigger policy, not a broken collector and not a root leak.**

**Consequence for the standard memory cap.** Under the project's `ulimit -v 1048576` this crate aborts on four of the five runnable benchmark axes, completing only `startup`.
The previously recorded set of seven SIGABRT programs is therefore not an exotic large-string edge case: the crate exhausts memory on ordinary loops, and any memory finding taken under that cap was measured on an interpreter already out of room.

## Cause 4 -- the residual

The repeat-count loop isolates interpretation with no control variable and no allocation: **5.2x**, at a flat 2,816 KB.
That is the floor the other three sit on top of, and it is the only one of the four this diagnosis does not explain further.

## What this predicts, so it can be wrong

* Fixing cause 1 alone should move `varlookup` from about 23x to near the repeat-loop's 5.2x, because `varlookup.rex`'s body is two assignments against a counted loop's overhead.
* Fixing cause 3 alone should bound RSS without moving wall time much, since the allocations still happen.
* Fixing cause 2 should move `strings` more than the other axes.
* No fix should move `arith` much, since its ratio is already 3.5x and its body does real arithmetic the oracle also pays for.

Each is a prediction against a measured baseline, and each is falsifiable by 4d-2's own before-and-after.

## What is not diagnosed

* **Why forcing a collection every hundred thousand iterations still leaves 123 MB** rather than the roughly 21 MB those iterations account for. The likely answer is that the arena is a high-water mark and `collect` reclaims into free lists without returning pages, which would make peak RSS a measure of the largest interval between collections rather than of live data. Unconfirmed.
* **`compound` at 12x specifically.** D9 required stem memoisation to be built in from the start; whether that happened has not been checked.
* **The 128-byte figure per literal.** That is large for a short string object and nobody has looked at the object layout.

## Method note

This was diagnosed by iterating -- hypothesis, probe, revise -- not by executing a plan.
The task plan had it as one linear task with ordered steps, and that shape was wrong: the second experiment invalidated the first hypothesis, and the fourth was only reachable because the third failed.
The first probe run against this question was also wrong in a way worth recording: it omitted an initialisation, so the program raised on line 1 and reported a small, stable, entirely meaningless resident set that read as a refutation of a correct finding.
**Check the exit status and the output of every probe** before reading its measurement.
