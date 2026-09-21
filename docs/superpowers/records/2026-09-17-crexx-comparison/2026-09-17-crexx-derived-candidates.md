# Queued: three performance candidates derived from CREXX, each unsized

Produced 2026-09-17 by a read-only design comparison against
`https://github.com/adesutherland/CREXX` (Adrian Sutherland's from-scratch Rexx in
C, compiler to bytecode plus VM). Full report assembled at
`scratchpad/crexx-compare/report.md` in this session's scratchpad, with the clone,
the build and every probe program beside it. **Copy the report into
`docs/superpowers/records/` when the tree is free**; it is currently only in
scratch.

Nothing here is scheduled. Every candidate is sized from CREXX's per-instruction
cost or from op counts, never from a measurement of ours, and each carries the run
that would size it. **Size before scheduling.**

## What did not transfer, so nobody re-asks

* **Direct threading.** Their `rxtvm` stores each handler's address in the
  instruction cell and dispatches with `goto *next_inst`. Unreachable from safe
  Rust: computed goto has no equivalent, `become` is unstable, and no `unsafe`
  grant would produce one anyway. Measured internally at ~13% of retired
  instructions, 4 host instructions out of ~32 per VM instruction.
* **Typed opcodes and static types.** Their `IADD` handler performs no type check
  at all because their compiler proves the type, and it can do that only because
  `INTERPRET` and `DROP` are commented out of their scanner and a variable may not
  be retyped. `a = 1; a = "hello"` fails to compile. `1 / 3` is `0` and `0.1 + 0.2
  = 0.3` is false. We already have the dynamic equivalent in `Op::Arith`'s
  per-site hint with permanent demotion, which is the right shape for a language
  that keeps `INTERPRET`.
* **An ahead-of-time-linked library image.** Their `rxvme` links the library into
  the executable and costs ~580M instructions to print `1`, against ordinary
  `rxvm`'s 3.8M and the C++ oracle's ~15M. Packing the library in did not make it
  cheap. The evidence points at lazy construction instead, and at the oracle's
  `rexx.img` as the design that actually wins startup.
* **Their hand-tiered handler panel.** A layout experiment with no control, at a
  granularity Rust does not offer, on a project that has already measured dead
  code alone moving cycles up to 4% per axis.

## A. Per-send method cache, generation-guarded

The one the tree was already built for. **D28 declined a per-call-site cache and
D29 laid down `Behaviour::version` precisely so D28 would be cheap to revisit**:
`rexx-classes/src/class_graph.rs:64-72`, written at three sites, read only through
the accessor at `:914`, whose consumers are tests. This candidate executes a
recorded revisit condition rather than importing a foreign idea.

What CREXX supplies is the part D28 left open: a two-way type-keyed site cache
with a generation stamp, and a slot table dense over specialisable sites rather
than over the stream, which is the trick `Chunk::hints` already uses. The shape to
copy on our side is the routine cache we already run, `ir.rs:441-459`.

Two details of their shape are worth having over a naive monomorphic cache:
**two ways rather than one**, because a send site alternating between two receiver
classes is common and thrashes a one-way cache, and **sparse slot allocation**, so
sites that can never specialise cost nothing.

Their class graph is flat and ours is not, which looks like it should sink the
whole idea and does not: `MethodDict` is already a flattened dictionary with
`scope_orders` recording the cascade, so the flattening has happened by the time a
send arrives and a behaviour handle plus its version is still a sound key. That is
the argument the candidate rests on, so check it before building anything on it.

Three hazards, in the order they will bite. **The first two are independent
problems and fixing one does not touch the other.**

1. **The object-own dictionary, which is structural.** `own_method_entry` reads a
   dictionary attached to a single object, not to its class, and is consulted
   first. A behaviour-level version stamp **cannot** witness a change to it, no
   matter how correctly every mutating site bumps the stamp, because the change is
   not to a behaviour at all. The key must include "this receiver has no own
   dictionary", or the cache must be skipped when it does. Getting this wrong is a
   silently wrong method call, not a crash, and no existing gate would catch it.
2. **`~name:scope` override sends take a different path** and must bypass the
   cache; a key on receiver behaviour alone answers the wrong method.
3. **`Op::Message` carries no site id**, so it grows a `u16` against an `Op`
   asserted at 16 bytes.

**Check first, before any of it:** that `Behaviour::version` is bumped at every
operation that can change a flattened method dictionary, `inherit`, `define`,
`~setMethod` and mixin folds among them. What has been established is *where* it
is written, three sites, and that only tests read it. That is not the same claim
as the set of writers being complete, and the completeness is what the candidate
needs. If one mutator does not bump, the candidate is a silently wrong method call
rather than a slow one.

**Sizing run:** callgrind `bench-programs/dispatch.rex` and `dispatchclass.rex`.
Under ~3% of instructions in `MethodDict::slot` and its hash, the candidate dies.

## B. Move `Op::Message` / `Say` / `Parse` / `Call` off the AST re-entry path

These ops carry an instruction index and re-enter `eval.rs` or the clause
interpreter; `EvalExpr`'s own doc says it is "still dispatched through `eval.rs`
rather than reimplemented here". The tree-walker was retired and `Op::Generic`
deleted, but this is the same residue under a different name. CREXX has no such
escape hatch.

**`Op::Exec` is the `INTERPRET` op and must stay an AST re-entry**, because the
instruction it runs does not exist until run time. Write that exclusion into any
plan rather than discovering it.

**Sizing run:** callgrind `dispatch.rex`. If `exec_message` and its callees are
already thin with the cost inside `Interp::lookup`, candidate A subsumes this and
it should not be done separately.

## C. Fuse comparison with the branch that consumes it

`do while i <= n` is one instruction for them and three ops for us: `Binary`,
`Condition`, `JumpUnless`. With trace off, `Op::Condition`'s only job is to raise
when the value is not exactly `0` or `1`, and a Rexx comparison always answers one
of those, so when the `IF`/`WHEN` root is a comparison operator the `Condition` is
provably a no-op and the intermediate register write is dead.

Cleanest of the three against the object model: the property is a fact about the
operator, not about name resolution. The trace complication is already solved,
since we emit a different op stream per `ChunkTrace`.

**Run this before anything else:** enumerate every comparison operator's path to
`logical()`. `logical()` was verified to produce only `b'0'` and `b'1'`; the
enumeration over operators was not done. If any comparison can answer something
else, the fusion is unsound and the candidate dies on the spot.

## For the Phase 6 plan, not for Phase 6 code

Level G structured concurrency does not transfer: its safety rests on no shared
mutable object graph and on static proof of transferability, and `~start` alone
breaks both by handing a Message object back into the shared graph. Two habits
from how they run it are worth having anyway.

* **An explicit "deliberately absent" list**, naming what the model will not
  expose before anyone asks. `GUARD`, `REPLY` and the activity model are exactly
  where Phase 6 scope creep will come from, and "we did not decide that" reads
  identically to "we forgot" six weeks later.
* **A status vocabulary with evidence requirements per state**: theirs is
  implemented / locally qualified / portable / published initial / stable, with
  the explicit sentence that an implemented capability is not automatically any of
  the others. This is our hollow-shell problem stated as policy rather than caught
  by a reviewer: the same distinction as "the method exists" against "the method
  works", which is how gate table C came to count `hasMethod` readbacks as method
  coverage.

## The comparability rule this produced

No CREXX benchmark is comparable to `rexxcps` or to any of our axes, because every
number was taken on a program their compiler type-checked end to end: no send, no
method lookup, no class-library call, no `INTERPRET`, no `DROP`. The one figure
worth carrying is their **internal** A/B, `rxtvm` against `rxbvm`, same tree, same
build, same machine, same program, because it isolates one technique and holds
everything else fixed.

## Revisit note, 2026-09-19: candidate E's verdict was reasoned on the wrong axis

E was rejected as "a layout experiment with no control". That reasoning stands
against their *hand-maintained hotness table*. It does not settle the concern the
table exists to address, and Moritz has since stated that this interpreter is
suffering L1i misses.

If that is confirmed by measurement, then CREXX's handler panel is aimed at a
constraint we actually have, and the question is no longer whether to adopt their
mechanism but whether there is a principled way to get the same effect: `#[cold]`
or `#[inline(never)]` on the rare arms of the op match, or moving infrequent op
handlers out of the driver body entirely, so that what stays resident is the arms
a hot loop actually executes.

Two measurements make this concrete and both are cheap. `run_ops_from` is 15,377
and 15,432 bytes in its two instantiations, so the driver alone occupies about
30 KB of instruction footprint, which is the order of a typical L1i. And the
`GRANTING` A/B merges those into one 21,026-byte function, which is a ~9.8 KB
reduction in exactly the text a hot loop keeps resident.

**The instrument matters more than usual here.** Retired-instruction counts cannot
see an I-cache effect at all. The `GRANTING` A/B's first round came back *worse*
on instructions -- `varlookup` +4.00%, `decrender` +0.64%, `emptyloop` +0.25% --
which on that instrument alone reads as a rejection. Whether it is one depends on
a frontend measurement that had not been taken when those numbers were read.

### Candidate E, settled 2026-09-19: measured here, and rejected on our own numbers

The revisit note above was written on the strength of "71.5M misses on
`dispatchclass` is not layout noise". That sentence needs splitting, because the
control-flow spike measured both halves of it.

The **baseline** misses are real: 71.5M per run at a 14.17% rate. What is not
supportable is attributing any *improvement* on that axis to a footprint change.
Four builds with the driver byte-identical and only its address moved swing
hardware L1i from -56.6% to +66.8% and cycles from -5.07% to +6.38%, `Ir` flat.
Anything a footprint change claims there lands inside that envelope.

And cachegrind at two associativities says why shrinking was never the lever:
`dispatchclass` fits in 32 KiB at 24.94 KiB and drops to **zero** misses at
64-way, so its misses are **conflict**, not capacity. Only `decrender` is capacity
bound, at 35.81 KiB.

**V1 of the spike was the principled version of the handler panel** -- cold arms
moved to one `#[inline(never)]` non-generic helper -- and it failed on the
instrument the control does not wash out: `+2.80%`, `+3.70%` and `+2.53%` retired
instructions on three axes, with 64-way I1 unmoved at +0.16%, and `.text` up 7,328
bytes because the extracted arms cost 9,965 standalone against roughly 1,900
inlined per instantiation.

So candidate E is rejected twice over: once on their evidence not isolating
outlining, and now on ours showing the outlining that is available to us costs
instructions and does not move the conflict misses it was aimed at.

### Candidate A's sizing run, narrowed 2026-09-19 but not settled

The control-flow spike's `--dump-instr` pass over `dispatchclass` (marginal, 40k
against 80k iterations) prints a share of instructions whose denominator is the
**hot-line subset**, not the whole run:

* total marginal: 4,374 Ir per iteration
* `MethodDict::slot`: 11 lines, **2.85%**
* `Interp::lookup`: 9 lines, **1.44%**

**This is a third quantity and not the kill threshold's currency.** The threshold
above is "under about 3% of instructions in `MethodDict::slot` and its hash". The
denominator here excludes marginal instructions on lines executed less than once
per iteration, and the numerator excludes that function's own such lines, so the
figure can move either way against a clean per-iteration count.

The hash half is **unidentified**. Two hash symbols appear in the same table:
`hashbrown::raw::RawTable<(String, Vec<method_dict::Me...>)>` at 14 lines / 2.48%
non-marginal, and `HashMap<Box<[u8]>, ObjRef, rustc_hash::FxB...>` at 8 lines /
5.74% marginal. The second looks like a name-to-object table rather than the
method dictionary, and neither was verified.

So the sizing run is narrowed, not replaced: `slot` and `lookup` together are in
the low single digits in a related currency, which is close enough to the
threshold that the clean per-iteration measurement is what has to decide it.
Whoever runs it must also establish which hash symbol belongs to the method dict
rather than assuming from the type name.
