# Phase 4d retention -- what a loop retains, and by what

Measured 2026-08-08 at commit `c9a90906`, `rust/target/release/rexx-run`
sha256 `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967` -- the same binary the
re-measured baseline in `perf-baseline.md` was taken with.
Oracle `bin/rexx` sha256 `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019`,
`lib/librexx.so.4` sha256 `42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb`.
Every figure is `/usr/bin/time -v` peak resident set under `ulimit -v 8388608` unless a row says
otherwise, taken from a fresh empty directory with absolute paths, with the exit status and the
stdout of every run checked.
No optimisation is proposed here and none survives this commit.

## Summary

**A loop's memory still grows without bound, and there are two independent causes, not one.**
The per-iteration constant has collapsed since it was last measured -- 8 bytes on the loop shape the
task brief names, down from about 216 -- but the growth is still linear and still unbounded, and one
of the two causes is a genuine root leak that a garbage collector would not touch.

* **Cause A, the arena is never collected.**
  Every heap object a program allocates is retained for the process's whole life, because nothing
  triggers a collection.
  A retained object costs 96 bytes of arena slot plus its own payload -- 128 bytes for a short
  string, 144 for a wide decimal.
* **Cause B, a counted loop's control temp is rooted until the loop ends.**
  `DO i = 1 TO n` pushes one root per iteration in `Interp::loop_advance` and pops none of them
  until the enclosing `DO` clause finishes, so the root set grows 8 bytes an iteration.
  A collector cannot reclaim through it: the roots are live by definition, so when the control
  variable's values are heap objects the loop pins every one of them.

Both are defects rather than performance properties, and both are recorded in
`phase-4-exclusions.txt` under KNOWN GAPS.

**Fixing both is a win on time as well as memory.**
Measured by prototype, then reverted: peak RSS falls 42x on `arith`, 60x on `varlookup` and 298x on
`strings`, and `strings` gets 16% *faster*.

## The stale figures, and what replaces them

The task brief quoted retention of roughly 216 bytes per iteration on
`do i = 1 to n; x = x + 1; y = x; end`, 213 MB at one million iterations, and peak RSS 51 to 218
times the oracle's.
Those were measured at or before `107febcd`.
Five speedups landed afterwards, two of them (`b6b1d8a9`, `e1d50dda`) directly on that loop shape.

| figure | at or before `107febcd` | at `c9a90906` |
|---|---:|---:|
| bytes retained per iteration, `x = x + 1; y = x` in `do i = 1 to n` | ~216 | 8.0 |
| peak RSS at 1,000,000 iterations of that loop | 213,104 KB | 9,900 KB |
| `varlookup.rex` (19,000,000 iterations of it) | 4,009,200 KB | 147,600 KB |
| `strings.rex` | 4,337,108 KB | 3,727,092 KB |
| `arith.rex` | 1,072,888 KB | 438,176 KB |
| `compound.rex` | 1,056,848 KB | 40,048 KB |
| `startup.rex` | 2,864 KB | 2,500 KB |
| RSS multiple against the oracle, over those five | 51x to 218x | 0.30x to 185x |

The oracle side was re-measured on the same host at the same time and has not moved: `arith` 20,736
KB, `compound` 20,720 KB, `strings` 20,188 KB, `varlookup` 20,992 KB, `startup` 8,416 KB.
So the multiples now read `compound` 1.9x, `varlookup` 7.0x, `arith` 21.1x, `strings` 184.6x, and
`startup` 0.30x -- this crate uses *less* than the oracle at startup, because it has no
`CoreClasses.orx` bootstrap yet.

**The linearity did not change, only the constant.**
Peak RSS on `do i = 1 to n; zz = zz + 1; yy = zz; end`, every run exiting 0 and printing `n`:

| n | peak RSS |
|---:|---:|
| 100,000 | 3,048 KB |
| 200,000 | 4,312 KB |
| 500,000 | 6,296 KB |
| 1,000,000 | 9,900 KB |
| 2,000,000 | 18,112 KB |
| 4,000,000 | 33,756 KB |
| 8,000,000 | 64,724 KB |
| 16,000,000 | 127,124 KB |
| 19,000,000 | 149,488 KB |

The 8,000,000 to 16,000,000 step is 62,400 KB for 8,000,000 iterations, which is 7.99 bytes each.
The relationship is straight from 100,000 up, which is what makes the small-scale instruments below
usable.

## The two call sites still read as Step 2 recorded them

Confirmed at `c9a90906`.
`Heap::collect` has exactly two production callers: `Interp::alloc_with` in
`crates/rexx-exec/src/lib.rs`, gated on `self.stress_collect`, which only
`run_program_collect_every_alloc` sets; and the user-callable `GC('Force')` builtin in
`crates/rexx-exec/src/builtin/state.rs`.
There is no allocation-count threshold and no heap-size threshold.

## What the retention is proportional to

**Not to iterations, not to clauses, and not to assignments.**
It is proportional to two things separately: the number of heap objects allocated, and the number of
iterations of a *counted* loop specifically.
Every row below is 8,000,000 iterations, every run exited 0 with the expected stdout, and the
per-iteration column subtracts the 2,500 KB floor a trivial program occupies.

| loop header | body | peak RSS | bytes per iteration |
|---|---|---:|---:|
| `do n` | `nop` | 2,568 KB | 0.0 |
| `do n` | `zz = zz + 1` | 2,516 KB | 0.0 |
| `do n` | `if 1 then nop` | 2,796 KB | 0.0 |
| `do n` | `if k then nop`, `k = 1` | 2,744 KB | 0.0 |
| `do i = 1 to n` | `nop` | 64,268 KB | 7.9 |
| `do i = 1 to n` | `nop; nop` | 64,232 KB | 7.9 |
| `do i = 1 to n` | `zz = zz + 1` | 64,432 KB | 7.9 |
| `do i = 1 to n` | `yy = zz` | 64,284 KB | 7.9 |
| `do i = 1 to n` | `zz = zz + 1; yy = zz` | 64,724 KB | 8.0 |
| `do i = 1 to n by 1` | `nop` | 64,700 KB | 8.0 |
| `do i = 1 to n` | `yy = 7` | 65,176 KB | 8.0 |
| `do n` | `yy = 'abc'` | 1,002,964 KB | 128.1 |
| `do n` | `yy = (k = 5)` | 1,002,736 KB | 128.0 |
| `do n` | `if k = 5 then nop` | 1,001,700 KB | 127.9 |
| `do n` | `yy = 12345678901234567890123456789` | 1,127,088 KB | 144.0 |
| `do n` | `yy = 'a' \|\| 'b'` | 3,002,748 KB | 384.0 |
| `do i = 1 to n` | `if i = n then nop` | 1,063,640 KB | 135.8 |
| `do i = 1.5 to n by 1` | `nop` | 1,064,416 KB | 135.9 |
| `do while k < n` | `k = k + 1` | 1,063,580 KB | 135.8 |
| `do forever` | `k = k + 1; if k = n then leave` | 1,001,456 KB | 127.9 |

Read the table as four constants: **0, 8, 128 and 384**, plus 8-and-128 added together as 136.

* **0 is the negative control that matters most.**
  `do n` -- a repeat-count loop with no control variable -- retains nothing at all, at any body that
  allocates nothing.
  Eight million iterations, eight million clause steps, and a flat 2.5 MB.
  So neither "iterating" nor "stepping a clause" retains anything, and any explanation that says
  otherwise is refuted by this row.
* **8 is the counted loop's control variable**, and only the control variable.
  It appears whenever the header is `DO i = ...` and never when it is `DO n`, `DO WHILE` or
  `DO FOREVER`, and it does not move when the body is doubled from one `nop` to two.
* **128 is one retained string object**, and it does not care what produced it: a literal
  assignment, a comparison's `0`, an `IF`'s test.
  `if 1 then nop` and `if k then nop` cost nothing because a literal `1` is inlined (`3799692d`) and
  a variable read allocates nothing, which is what isolates the 128 to the *comparison's result*
  rather than to `IF`.
* **384 is three of them**: `'a'`, `'b'` and the concatenation.
* **`zz = zz + 1` costs nothing** inside a `do n` loop, which is `b6b1d8a9` visible from the memory
  side: a small-integer sum is a tagged immediate and never reaches the heap.
  Give it an operand that cannot be one -- `do i = 1.5 to n`, or a 25-digit addend -- and the 128
  comes back.

**The two causes are released at different times, which is the cleanest evidence that they are two
things rather than one.**
Eight outer iterations of a 1,000,000-iteration inner loop:

| program | peak RSS |
|---|---:|
| `do i = 1 to 8; do j = 1 to 1000000; nop; end; end` | 9,868 KB |
| `do i = 1 to 8; do j = 1 to 1000000; yy = 'abc'; end; end` | 1,009,888 KB |

The first is the 1,000,000-iteration figure, not the 8,000,000-iteration one: the inner loop's
8-byte-per-iteration growth is released every time the inner loop ends.
The second is the full 8,000,000-iteration figure: the heap objects are never released at all.

## The two retained things, named

Both were located with `valgrind --tool=massif --stacks=no --detailed-freq=1`, read with `ms_print`,
at 100,000 iterations, which the linearity above licenses.

### Cause A -- `Heap.slots`, grown by `alloc_with_uncollected`, never swept

For `do n; yy = 'abc'; end` at 100,000 iterations, massif's peak snapshot has a 12,894,105-byte
useful heap and puts **12,582,912 of those bytes in one allocation site**:

```
12,582,912B  <alloc::raw_vec::RawVec<rexx_core::heap::Slot>>::grow_one
             <- <rexx_core::heap::Heap>::alloc_with_uncollected (heap.rs:271)
             <- alloc_with (lib.rs:2101)
             <- text_owned (value.rs:89) <- <rexx_exec::Interp>::text (value.rs:47)
             <- eval_node <- <rexx_exec::Interp>::eval (eval.rs:152)
             <- step (run.rs:1033) <- step_in_temps_frame (run.rs:4206)
             <- run_bounded (run.rs:4795) <- run_repeating (run.rs:5239)
```

12,582,912 bytes over a 131,072-element capacity is **96 bytes per `Slot`**, which with the string's
own `Vec<u8>` and the allocator's per-block overhead is the 128 bytes the table measures.
This is `Heap.slots` in `crates/rexx-core/src/heap.rs`, and nothing ever sweeps it because nothing
ever calls `Heap::collect`.

The root that holds these objects is **nothing**.
They are not reachable from `RootSet` at all -- the per-clause temps frame that rooted them was
popped when the clause ended, and the variable they were bound to has been rebound many times since.
They are retained because the sweep that would notice never runs.

### Cause B -- `RootSet.temps`, grown by `loop_advance`, popped only at loop exit

For `do i = 1 to n; nop; end` at 100,000 iterations, massif's peak snapshot has a 1,075,070-byte
useful heap and puts **1,048,576 of those bytes in one allocation site**:

```
1,048,576B   <alloc::raw_vec::RawVec<rexx_core::handle::ObjRef>>::grow_one
             <- push_temp (roots.rs:146)
             <- loop_advance (run.rs:5654)
             <- run_repeating (run.rs:5168)
             <- step (run.rs:1734) <- step_in_temps_frame (run.rs:4206)
             <- <rexx_exec::Interp>::run_activation (run.rs:882)
```

1,048,576 bytes over a 131,072-element capacity is **8 bytes per `ObjRef`**, which is the table's
8-byte constant exactly.

`Interp::loop_advance` reads the control variable's previous value, calls
`self.roots.push_temp(previous)` to root it across the arithmetic that produces the next value, and
never pops it.
The only thing that releases it is the enclosing `step_in_temps_frame`'s unconditional
`pop_frame` -- and the instruction that frame belongs to is the whole `DO`, so the release happens
when the loop finishes rather than when the iteration does.
That is precisely the nested-loop result above: the inner loop's temps go when the inner loop ends.

**This one is a root leak, and a collector cannot fix it.**
The entries are live roots, so a mark phase must trace them.
When the control values are tagged small integers they cost only the 8 bytes of the vector entry;
when they are not -- `do i = 1.5 to n by 1` -- each one pins a heap object as well, which is why
that row costs 136 bytes and not 8.
`RootSet::pop_frame` truncates to a watermark and its own doc comment says a caller may open a frame
and rely on the outer truncation, so the fix is a frame taken and popped inside `loop_advance`, not
a change to the root set.

## Whether the arena is a high-water mark

**It is, and that turns out not to be the objection it looked like.**
`Heap::collect` marks, then replaces each unreachable `Slot::Live` with a `Slot::Free` threaded onto
a free list.
`Heap.slots` is only ever pushed and indexed -- there is no `truncate`, no `shrink_to_fit`, no
`clear` -- so the vector's length never falls and the pages are never returned to the allocator.
Peak RSS is therefore a measure of the largest interval between collections rather than of live
data, which is what Step 2 of the brief hypothesised.

**But the free list is reused, so the peak is bounded by the interval rather than by the program's
total allocation.**
The same program at 8,000,000 iterations of `yy = 'abc'`, structured as an outer loop over an inner
loop with a call at the outer level, three repetitions each, `gc()` (which does not collect) as the
control and `gc('Force')` as the treatment:

| outer call | inner loop length | peak RSS | wall |
|---|---:|---|---|
| `gc()` | 10,000 | 1,002,976 / 1,002,700 / 1,002,728 KB | 2.91 / 2.97 / 2.91 s |
| `gc()` | 100,000 | 1,003,528 / 1,003,452 / 1,003,720 KB | 2.88 / 2.83 / 2.94 s |
| `gc('Force')` | 10,000 | 4,036 / 3,724 / 4,052 KB | 2.44 / 2.36 / 2.39 s |
| `gc('Force')` | 100,000 | 15,796 / 16,088 / 16,344 KB | 2.64 / 2.61 / 2.60 s |
| `gc('Force')` | 1,000,000 | 136,072 KB (one run) | 2.54 s |

Peak RSS tracks the collection interval linearly and nothing else: ten times the interval is ten
times the peak.
So **a trigger policy would bound the observed figures, not merely delay them**, and the brief's
worry that it would only postpone the problem is refuted.

The brief also asked why forcing a collection every hundred thousand iterations still left 123 MB.
That figure was taken before `b6b1d8a9` and `e1d50dda`; the equivalent measurement here leaves 16 MB
on a loop that allocates far more per iteration than the one it was taken on.

The control column is worth reading on its own: `gc()` costs the same 1,003 MB whether the inner
loop is 10,000 or 100,000 long, which is the point -- with no collection the peak is the whole
program's allocation and the structure around it is irrelevant.

## What the fix costs, by prototype

Two throwaway prototypes were built, measured, and reverted in the commit that publishes this file.
Both are gone from the tree; `sha256sum -c` against copies taken before the edits confirms the two
touched files are byte-identical to their pre-prototype state, and the rebuilt `rexx-run` has the
same sha256 as the baseline binary.

* **Prototype A, a trigger policy.**
  A watermark on `Heap::live_count()` in `Interp::alloc_with`: collect before allocating once the
  live count passes it, then set the next watermark to twice what survived, floor 65,536.
* **Prototype B, the loop temp.**
  `push_frame` before `loop_advance`'s `push_temp` and `pop_frame` after the increment stores its
  result.

Every measurement below is interleaved -- base, prototype, base, prototype -- because two separate
suite runs on this machine have invented a 5% effect that was not there.
Every run exited 0 and printed the expected bytes.

**Prototype A alone**, three interleaved repetitions:

| axis | base RSS | A RSS | base wall | A wall |
|---|---|---|---|---|
| `arith` | 439,440 / 439,440 / 439,952 KB | 21,516 / 21,664 / 22,024 KB | 3.12 / 3.12 / 3.11 s | 3.27 / 3.29 / 3.23 s |
| `compound` | 41,072 / 40,144 / 40,204 KB | 40,852 / 40,844 / 40,636 KB | 6.90 / 6.92 / 6.94 s | 6.78 / 6.85 / 6.82 s |
| `strings` | 3,728,072 / 3,728,100 / 3,728,036 KB | 82,808 / 82,324 / 82,560 KB | 9.23 / 9.04 / 9.33 s | 9.39 / 10.14 / 10.82 s |
| `varlookup` | 150,208 / 148,740 / 149,688 KB | 150,192 / 149,956 / 150,740 KB | 5.24 / 5.25 / 5.27 s | 5.41 / 5.42 / 8.05 s |

`varlookup` and `compound` do not move at all under A, and that is the finding rather than a
disappointment: their retention is cause B, which A does not touch.

**Both prototypes**, interleaved, two repetitions except `strings` at seven:

| axis | base RSS | A+B RSS | base wall (median) | A+B wall (median) |
|---|---|---|---:|---:|
| `arith` | 439,468 / 439,732 KB | 10,452 / 9,876 KB | 3.10 s | 3.27 s |
| `compound` | 40,324 / 40,136 KB | 2,472 / 2,388 KB | 6.83 s | 6.78 s |
| `strings` | 3,726,796 - 3,728,796 KB | 12,372 - 13,072 KB | 9.24 s | 7.79 s |
| `varlookup` | 150,272 / 150,284 KB | 2,500 / 2,436 KB | 5.24 s | 5.42 s |
| `startup` | 2,548 KB | 2,512 KB | -- | -- |

So the two together are worth **42x on `arith`, 16x on `compound`, 298x on `strings` and 60x on
`varlookup`** in peak resident set, at a wall cost between -16% and +6%.
`varlookup` lands at 2,436 KB against the oracle's 20,992 KB, and `compound` at 2,388 KB against
20,720 KB.

**`strings` gets faster, by 16%, and that is the headline for 4d-2.**
Its base peak is 3.7 GB; holding 3.7 GB of never-reused arena destroys locality, and collecting into
a free list that fits in cache is cheaper than the allocator's next fresh page.
The smoke profile that put about a third of `arith`'s self time in the glibc malloc family was
reading the same thing from the allocator's side.

**The whole test suite passes with both prototypes in place**: 1,317 tests, 0 failures, exit 0.
That is evidence and not a safety proof.
`Heap::collect`'s own doc records one known under-rooted window -- `EXIT`'s result, from the
wrapper's `pop_frame` through to `exit_code_for` -- and argues that nothing in that window
allocates.
A real trigger policy in 4d-2 must sweep `rexx-exec` for that shape rather than inherit this
prototype's silence, because every corpus program is small enough that a rare unrooted window need
not be hit even once.

## The ruling: this is a defect

**Both causes are defects, not performance properties, and a user sees them as an abort.**
A Rexx program whose live set is two integers should not exhaust memory, and this one does: at
`c9a90906` under the project's standard `ulimit -v 1048576`, `arith.rex` and `strings.rex` both die
with

```
memory allocation of 402653184 bytes failed
```

on stderr, exit status 134, and nothing at all on stdout.
No Rexx condition is raised, so `SIGNAL ON SYNTAX` cannot catch it, no traceback is printed, and the
partial output the program had already produced is lost.
`compound.rex`, `varlookup.rex` and `startup.rex` now complete under that cap; at `107febcd` four of
the five aborted, so the symptom has narrowed without going away.

The severity is a function of runtime, not of program size.
A four-line loop that is correct today fails after enough iterations, and the iteration count at
which it fails depends on how much the body allocates -- 8 bytes an iteration for a counted loop
that allocates nothing, 128 for one that touches a string.

Both are recorded in `docs/superpowers/plans/phase-4-exclusions.txt` under KNOWN GAPS.

## The coverage gap that let this reach Phase 4

**Nothing in the differential suite can see unbounded growth, and this is the project's primary
instrument.**
Two independent reasons, either of which alone is sufficient:

* **The harness compares output, not memory.**
  Neither the corpus harness nor the trace harness reads peak resident set, `getrusage`, or any
  memory figure at all.
  A program that retains a gigabyte and prints the right bytes passes.
* **Every corpus program is far too small.**
  The largest loop bound anywhere in `rust/corpus` is `to 1010`.
  At the 8 bytes an iteration measured above, that loop retains about 8 KB -- four orders of
  magnitude below anything a peak-RSS check could distinguish from noise, even if one existed.

The two compound: adding an RSS assertion to the existing corpus would catch nothing, and adding a
long-running program without an RSS assertion would catch nothing either.
Closing this needs both -- a program whose iteration count is large enough for the linear term to
dominate the fixed cost, and a bound on its peak resident set rather than on its output.

That is a gap in the instrument the whole project's definition of correctness rests on, and it is
why unbounded growth survived three phases of differential testing with a green suite.

## Reproducing

The probe programs are throwaway and are not committed.
Each is the loop in the tables above, written into a fresh empty directory, run with an absolute
path:

```sh
( ulimit -v 8388608; /usr/bin/time -v /abs/path/rexx-run /abs/path/probe.rex )
```

Check the exit status and stdout of every run before reading the resident set.
A program that dies on line 1 reports a small, stable and entirely meaningless figure, and that
mistake has already been made once against this very finding.

The two allocation sites:

```sh
valgrind --tool=massif --stacks=no --detailed-freq=1 --massif-out-file=out /abs/path/rexx-run /abs/path/probe.rex
ms_print --threshold=1.0 out
```

100,000 iterations is enough for both, because the growth is linear from there up.
