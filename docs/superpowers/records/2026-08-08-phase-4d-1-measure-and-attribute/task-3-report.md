# Task 3 report: diagnose the unbounded per-iteration retention

Status: DONE.
Worked from `c9a90906`, all measurements at that commit with
`rust/target/release/rexx-run` sha256
`c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967`, the same binary the re-measured
baseline was taken with.
Deliverable: `docs/superpowers/plans/phase-4d-retention.md`.

## The headline

**Memory still grows without bound, and there are two independent causes rather than one.**
The brief's warning was correct and the old figures are all stale: the per-iteration constant on
`do i = 1 to n; x = x + 1; y = x; end` is now **8.0 bytes**, not 216, and 1,000,000 iterations peak
at 9,900 KB, not 213,104 KB.
The growth is still linear and still unbounded, so the finding survives the constant collapsing.

* **Cause A, the arena is never collected.**
  Every heap object is retained for the process's life; 96 bytes of `rexx_core::heap::Slot` plus its
  payload, 128 bytes for a short string.
* **Cause B, a loop header's temps are a root leak, at two sites.**
  `Interp::loop_advance` (a counted loop's control variable) and `Interp::eval_condition`
  (`run.rs:6017`, every `WHILE`/`UNTIL` test) each push one `RootSet` temp per pass and pop none
  until the whole `DO` clause ends, 8 bytes a pass, and **a collector cannot reclaim through
  either**.

Cause B is new -- `phase-4d-diagnosis.md`'s Cause 3 states "not a root leak", which is false at this
commit, and that document now carries inline corrections at the two false claims.

**Review round: three majors, four minors, all addressed.** The first major was mine and was
substantive: the deliverable claimed the 8-byte constant appears "never" for `DO WHILE`, and
specified a fix to `loop_advance` alone. That would have left `DO WHILE` and `DO UNTIL` growing
without bound after 4d-2 built exactly what the document said, and no benchmark axis could have
caught it because all four are `do i = 1 to n`.

## Against the stale figures

Peak RSS, `/usr/bin/time -v`, `ulimit -v 8388608`, oracle re-measured the same day on the same host.

| program | `107febcd` | `c9a90906` | oracle now | multiple now |
|---|---:|---:|---:|---:|
| `varlookup.rex` | 4,009,200 KB | 147,600 KB | 20,992 KB | 7.0x |
| `strings.rex` | 4,337,108 KB | 3,727,092 KB | 20,188 KB | 184.6x |
| `arith.rex` | 1,072,888 KB | 438,176 KB | 20,736 KB | 21.1x |
| `compound.rex` | 1,056,848 KB | 40,048 KB | 20,720 KB | 1.9x |
| `startup.rex` | 2,864 KB | 2,500 KB | 8,416 KB | 0.30x |

The band is now 0.30x to 185x rather than 51x to 218x.

## What the review changed, with the evidence I re-measured

* **Cause B is two sites, not one.** `do n while zz; nop; end` with `zz = 1` -- no control variable,
  nothing on either side of the test that allocates -- retains **7.93 bytes a pass** (64,468 KB at
  8,000,000) against the same loop without the condition at **0.0** (2,532 KB; 2,568 KB in an
  earlier batch, disclosed in the deliverable). `UNTIL` matches at
  7.83. Massif on that shape names `eval_condition (run.rs:6017)` directly. The tree's own doc at
  `run.rs:4073`-`4082` already said this and I missed it; the deliverable now cites it.
* **The fix specification was wrong and is rewritten.** Both sites need the frame treatment, and the
  document now names `do n while zz; nop; end` as 4d-2's acceptance test instead of `varlookup.rex`.
* **A methodological warning is recorded**, because 4d-2 will reuse massif: on
  `do while k < n; k = k + 1; end` massif attributes the temps vector's growth to
  `eval_arithmetic (eval.rs:620)` -- the *body's* push, which crosses the capacity boundary -- while
  the entries accumulating are `eval_condition`'s. Verified in the profile. Isolate a suspect push
  by removing the others, not by reading the deepest frame.
* **The exclusions row was in the wrong section** -- it landed inside `CLOSED DEFECTS` (starts 1787)
  rather than `KNOWN GAPS`. Moved, not rewritten in place. Verified by reconstruction: the file
  minus the moved block is byte-identical to the previous content, `diff` exit 0.
* **The abort claim overstated by one axis.** Raising the cap by exactly the 512 MiB
  `INTERPRETER_STACK_BYTES` reserves (`ulimit -v 1572864`) takes `arith` from rc 134 to rc 0 with
  correct output; `strings` still dies at 805,306,368 bytes. So one axis dies with the reservation
  free, not two. The ruling stands on the linearity, and `rust/CLAUDE.md`'s required caveat is now in
  all three documents.
* **Minors:** the discarded 18.94 s run is disclosed in the deliverable; the prototype-A wall table
  now flags its outliers and states that no claim rests on a sub-10% wall difference; the diagnosis
  document carries inline markers at both false statements and its header now says two false and one
  answered; the heading capitalisation is fixed.

## How the two causes were separated

The decisive probe is a negative control the brief did not suggest: **`do n` -- a repeat-count loop
with no control variable -- is flat at 2,568 KB over 8,000,000 iterations with a `nop` body**, while
`do i = 1 to n` with the same body reaches 64,268 KB.
So neither iterating nor stepping a clause retains anything; only a *counted* loop's control
variable does.
Doubling the body from one `nop` to two does not move it, and `do while` and `do forever` do not
show it at all.

Nesting separates the release times: `do i = 1 to 8; do j = 1 to 1000000; nop; end; end` peaks at
the 1,000,000-iteration figure (the temps go at inner-loop exit), and the same shape with
`yy = 'abc'` peaks at the full 8,000,000-iteration figure (the heap objects never go).

Both sites were named with `valgrind --tool=massif`, not by reading code:
`Vec<rexx_core::heap::Slot>::grow_one` under `Heap::alloc_with_uncollected` at 96 bytes a slot, and
`Vec<rexx_core::handle::ObjRef>::grow_one` under `push_temp` under `loop_advance` at 8 bytes an
entry.
Both byte counts reproduce the measured per-iteration constants exactly.

## The two open questions from Step 2, answered

* **The arena is a high-water mark** -- `Heap.slots` is only pushed and indexed, never truncated or
  shrunk -- **but the free list is reused, so a trigger policy bounds the peak rather than delaying
  it.** Measured on one 8,000,000-iteration program with `gc()` as the control and `gc('Force')` as
  the treatment: 1,002,700 KB with no collection, 15,796 KB every 100,000 iterations, 4,036 KB every
  10,000. Peak RSS tracks the interval linearly.
* **The 128 bytes** is 96 bytes of arena slot plus the payload `Vec<u8>` and allocator overhead.

## Step 4, quantified by prototype and reverted

Two prototypes, built, measured interleaved, reverted in this commit.
`sha256sum -c` against pre-edit copies confirms `lib.rs` and `run.rs` are byte-identical to their
prior state, and the rebuilt `rexx-run` has the same sha256 as the baseline binary.
Never `git checkout --`.

| axis | base RSS | both prototypes | base wall | both wall |
|---|---:|---:|---:|---:|
| `arith` | 439,468 KB | 10,452 KB | 3.10 s | 3.27 s |
| `compound` | 40,324 KB | 2,472 KB | 6.83 s | 6.78 s |
| `strings` | 3,727,312 KB | 12,488 KB | 9.24 s | 7.79 s |
| `varlookup` | 150,272 KB | 2,500 KB | 5.24 s | 5.42 s |

42x, 16x, 298x and 60x on memory, at a wall cost of -16% to +6%.
`strings` gets **16% faster**, which is the largest single lever this task found for 4d-2.
`varlookup` and `compound` do not move under the trigger policy alone, which is what proves cause B
is separate.

The full test suite passes with both prototypes in place: 1,317 tests, 0 failures, exit 0.

## Step 5, the ruling and the coverage gap

**Defect.** Under the standard `ulimit -v 1048576` at this commit, `arith.rex` and `strings.rex` die
with `memory allocation of 402653184 bytes failed`, exit 134, empty stdout, no Rexx condition and no
traceback.
With the 512 MiB reservation freed, `strings.rex` alone dies.
Four of five aborted at `107febcd`, so the symptom narrowed without going away.
Recorded in `phase-4-exclusions.txt` under KNOWN GAPS, which is the section whose own text says
adding a row needs no amendment.

**The coverage gap is recorded in both files.**
Nothing in the differential suite can see this, for two sufficient reasons: neither harness reads
peak resident set or any memory figure, and the largest loop bound anywhere in `rust/corpus` is
`to 1010`, which at 8 bytes an iteration retains about 8 KB.
Closing it needs both a long-running program and an RSS bound; either alone catches nothing.

## Concerns

* **The prototypes' green suite is evidence, not a safety proof.** `Heap::collect`'s own doc records
  an under-rooted window at `EXIT`'s result. 1,317 tests passing with a collector firing every
  65,536 live objects does not establish that no such window exists, because every corpus program is
  small enough to miss a rare one. 4d-2 must sweep `rexx-exec` for the shape rather than inherit
  this prototype's silence. This is stated in both the diagnosis document and the KNOWN GAP row.
* **`clippy` finished in 1.63 s off a warm target directory.** It did re-check `rexx-exec`, the only
  crate touched, so the restored file was linted; but per `rust/CLAUDE.md` a same-session green is
  provisional and a clean-target run at the phase boundary is still owed.
* **One `strings` wall measurement was contaminated and discarded.** An early non-interleaved run
  read 18.94 s for the prototype against 9.20 s for the base, which reversed under interleaving to
  7.79 s against 9.24 s. Every figure reported is interleaved; the discarded run is now disclosed in
  the deliverable so a re-measurement can be reconciled against it.
* **`phase-4d-diagnosis.md` was edited**, which is outside the brief's file list. Two of its
  statements are false at `c9a90906` and one is answered, and it is the document a reader reaches
  first, so it carries a header note plus an inline marker beside each false claim; its own
  measurements at `9bcbfeda` are untouched.
* **What closing `eval_condition`'s site is worth is unquantified.** Prototype B covered
  `loop_advance` only, and every prototype figure therefore bounds the counted form. I did not build
  a second prototype for the `WHILE` site: the mechanism is measured and located, but 4d-2 will need
  its own number.
