# Consult: what the literature says about an IR that loses to a tree-walker

Written without reading `phase-4f-record.md` or `2026-08-08-phase-4e-ir-design.md`; the
agree/disagree section at the end was added after reading them.

## 1. The named causes

**The folklore this IR was built on is empirically contested.** Larose, Kaleba, Burchell and
Marr, *AST vs. Bytecode: Interpreters in the Age of Meta-Compilation* (OOPSLA 2023), built
matched AST and bytecode interpreters for the same language and found the AST versions "on par
with, or even slightly faster than their bytecode counterparts" in the interpreter-only case.
**Caveat, and it is a large one:** their bytecode interpreters ran on RPython and Graal, and the
authors attribute the loss partly to Graal supporting only structured control flow, so a flat op
stream with arbitrary jumps compiles badly. That is a host artifact, not a law. The paper does
*not* license "bytecode is slower"; it licenses "an IR is not automatically faster, and the
burden of proof is on the IR."

**Cause A -- the interpreter was never dispatch-bound.** Romer, Lee, Voelker, Wolman, Wong,
Baer, Bershad and Levy, *The Structure and Performance of Interpreters* (ASPLOS 1996), found
interpreter cost is a function of work-per-instruction, not of the dispatch loop. Ertl and
Gregg, *The Behavior of Efficient Virtual Machine Interpreters on Modern Architectures*
(Euro-Par 2001) and *The Structure and Performance of Efficient Interpreters* (JILP 2003),
reached the *opposite* conclusion -- indirect branches are 3.2--13% of instructions and up to
61--79% of cycles -- and explicitly said Romer measured inefficient interpreters. The dispute
resolves on work-per-instruction: Ertl and Gregg measured Forth and JVM, where an instruction is
~10 native instructions. Here a clause is a decimal add plus a heap materialisation. The
reference's per-clause loop is ~5 ns of 60 ns. **Dispatch was under 10% of the thing being
replaced, so the IR could not have paid even if it were free.**

**Cause B -- abstraction level bounds what dispatch tricks are worth.** Brunthaler,
*Virtual-Machine Abstraction and Optimization Techniques* (BYTECODE 2009) and *Efficient
Interpretation using Quickening* (DLS 2010): the payoff of dispatch optimisation is inversely
proportional to the VM's abstraction level, and for high-level VMs the cost is **operand access
and boxing**, not dispatch. Rexx is about as high-abstraction as a VM gets -- every value is
conceptually a string.

**Cause C -- the hybrid pays both taxes.** Every `Op::Generic` costs an IR fetch, bounds check
and `match` *and then* the whole tree-walk it delegates to. Every non-promotable expression
costs an op *and* a recursive `eval`. A partial IR is strictly additive over the AST walk on
exactly the constructs it has not promoted.

**Cause D -- flattening gives back what the host stack gave for free.** `SelectFrame` in
`rust/crates/rexx-exec/src/ir/drive.rs` is, by its own doc comment, "the tree-walker's own Rust
call frame, flattened". The AST walker got nesting, unwinding and scoping from the machine call
stack at zero marginal cost; the op stream re-implements them in a `Vec` with a bounds-checked
push/pop and a linear search on escape.

**Cause E -- register machines trade instruction count for memory traffic.** Shi, Casey, Ertl
and Gregg, *Virtual Machine Showdown: Stack Versus Registers* (VEE 2005 / TACO 2008), measured a
~47% drop in executed VM instructions and ~26.5% in running time for a register JVM -- but that
gain comes from *eliminating* loads and stores. An AST walker holds an operand in a machine
register (a Rust local); a register IR that spills every intermediate to a `Vec<ObjRef>`
temporaries stack with a bounds check per access moves in the wrong direction.

**Cause F -- object-table indirection.** Every value here is reached by resolving a
generation-checked handle through `Heap::slots` (`rust/crates/rexx-core/src/heap.rs`): a bounds
check, a discriminant test and a generation compare before the payload pointer. Smalltalk-80
used an object table and the Smalltalk/Self line abandoned it for direct pointers for exactly
this reason (Ungar's Berkeley Smalltalk work; Deutsch and Schiffman, POPL 1984). The handle is
the right call for a no-`unsafe` GC, but it is a per-access cost the reference does not pay.

## 2. But none of A--F is the main number

Reading the code against the reference makes the ranking unambiguous, and it is not a dispatch
story at all. It is representation.

* `Number` is `{ negative: bool, digits: Vec<u8>, exponent: i32 }` --
  **one heap-allocated byte per decimal digit** (`rust/crates/rexx-num/src/lib.rs:407`).
  `add_magnitudes` does `vec![0u8; n]` per operation (`addsub.rs:223`).
* `Interp::to_number` returns an **owned clone**: `Body::Num { value, .. } => Ok(value.clone())`
  (`rust/crates/rexx-exec/src/value.rs`), so merely *reading* a numeric variable allocates.
* `Body::Text { bytes: Vec<u8>, .. }` -- a 96-byte arena slot **plus** a separate `malloc` for
  every string, however short.

The reference: `NumberString` "directly includes the string data" (`NumberStringClass.hpp:96`),
one variable-length object; arithmetic works in `char resultBufFast[FAST_BUFFER]` with
`FAST_BUFFER = 48` (`NumberStringClass.hpp:414`, `NumberStringMath2.cpp:138`) -- **a stack
buffer, zero heap traffic for intermediates**; `RexxInteger::plus` does `value + other->value`
in machine integers whenever both operands are valid under the current DIGITS
(`IntegerClass.cpp:552`); and `-10..100` are interned (`IntegerClass.hpp:185`).

So on `i = i + 1` with one operand off the small-int path, this system performs on the order of
four allocations where the reference performs zero-to-one. That is the 155 ns against 13 ns, and
it also explains the residual the brief reports: **"remove every profiled block the reference
does not also pay, still 4.4x" is the signature of a representation cost, not a block cost.**
Cloning a digit `Vec`, chasing a handle, and looping one byte per digit are charged to whichever
block happens to be running. A flat profile after the obvious blocks are removed is the standard
symptom, and the literature's answer to it is the data representation every time.

What everyone else does for a decimal tower: decNumber (Cowlishaw, IBM -- the author of Rexx)
packs `DECDPUN` digits into each coefficient unit rather than one per byte; libmpdec, which
became CPython's `_decimal`, uses base-10^9 limbs and is reported at **30x--80x** over the pure
Python `decimal` (bytereef.org, mpdecimal project; CPython 3.3). That factor is a
different-language number and does not transfer -- but the change that produced it is exactly
the change available here. Ruby stores strings up to 23 bytes inline in the 40-byte `RVALUE`
slot; CPython's compact unicode objects do the same; ooRexx does the same. The pattern is
universal and this system is the outlier.

## 3. Techniques the constraints rule out

* **Threaded code / computed goto / replication / superinstructions** (Ertl and Gregg, PLDI
  2003; Piumarta and Riccardi, PLDI 1998, ~2x on Forth and OCaml). Needs `unsafe` or unstable
  `become` in Rust, *and* attacks the one cost that is under 10% here. Rule out on both counts.
* **NaN boxing.** Technically safe in Rust via `f64::to_bits` with an index payload, but the
  values are decimal strings, not doubles; there is nothing to box. The existing tagged small
  int already covers the case that matters.
* **JIT, meta-tracing, copy-and-patch** (Xu and Kjolstad, PLDI 2021). All need `unsafe`. Larose
  et al. is also the argument for fixing the interpreter first regardless.
* **Full deoptimisation machinery** (Hölzle, Chambers, Ungar, PLDI 1992). Correct principle,
  far too heavy: the chunk-cache-keyed-on-trace-setting already buys the same thing.
* **Ropes / lazy concatenation** (Boehm, Atkinson, Plass 1995). Trace-when-on forces
  materialisation, and short strings dominate; the inline-payload fix is strictly better.

## 4. The three I would attempt, in order

**1. Collapse allocations per value.** Three sub-changes, in increasing effort: (a) make
`to_number` borrow or compute in place rather than clone a `Vec` on every numeric read -- this
is the cheapest large win in the codebase and needs no design; (b) give `Number` an inline
digit buffer for the common case, which is the reference's own `FAST_BUFFER = 48` answer and
covers every program running at default `DIGITS 9`; (c) store short `Text` bytes inline in the
slot, as Ruby, CPython and ooRexx all do. Expected: the 155 ns block toward the reference's
13 ns, *plus* an unquantified but larger amount smeared across every other block. Sources:
decNumber DECDPUN; libmpdec; ooRexx's own `NumberStringClass.hpp`. **Transferable as a design,
not as a factor** -- nobody has measured this change on this codebase.

**2. Let promoted expressions keep intermediates unboxed in registers.** This is the *only*
thing an op stream can do that an AST walker cannot, and it is the reason to keep the IR. Today
an expression that is not a literal, a bare symbol, or a promotable arithmetic node calls back
into the recursive evaluator, and every intermediate becomes a heap object. If the register file
can hold an unboxed `Number` or `i64`, `a = b*c + d*e` allocates once instead of three times.
Sources: Brunthaler's operand-access argument (DLS 2010); Shi et al. (VEE 2005) for why register
machines win when they actually eliminate the memory round trip. **This depends on (1):** an
unboxed `Number` that still owns a `Vec` has moved the allocation, not removed it.

**3. Quicken the op stream: inline caches for builtin and variable dispatch, and compiled-in
observability.** Brunthaler, *Inline Caching Meets Quickening* (ECOOP 2010), reports **up to
1.71x** on CPython 3.1 -- an interpreter of comparable abstraction level, which makes it the
most transferable published number here. Deutsch and Schiffman (POPL 1984) is the origin;
CPython's PEP 659 is the modern production instance. For the trace facility specifically, PEP
669 (`sys.monitoring`, CPython 3.12) is the exact shape: **swap instrumented op variants in when
monitoring is enabled rather than testing a flag per instruction**, to the point that "code run
under a debugger on 3.12 should outperform code run without a debugger on 3.11". Keying the
chunk cache on the trace setting is already this idea; the remaining work is making the untraced
chunk carry *no* trace ops and no clause obligations beyond error reporting.

Ordering rationale is Amdahl, not preference: (1) addresses the 142 ns of the 327 ns per-clause
gap that is visibly allocation plus the residual that the block-subtraction arithmetic cannot
account for; (2) and (3) are multiplicative on what is left. Attempting (3) first is the classic
mistake the 1996/2003 dispute describes -- optimising dispatch in an interpreter that is not
dispatch-bound.

## 5. The headline

**Do not try to make the IR faster. The IR is not why this is slow, and a better IR will not fix
it.** Keep it -- it is the necessary carrier for (2) and (3), and neither is expressible in a
tree-walker. But the 5--7x against a tree-walker is a representation gap: every value costs two
to four allocations where the reference costs zero to one, and a decimal digit costs a byte in a
`Vec` where the reference costs a byte in a stack buffer. Fix the representation and the IR
starts earning; leave it and no amount of promotion will close the gap.

## 6. After reading `phase-4f-record.md` and `2026-08-08-phase-4e-ir-design.md`

**The premise I was given to react to is not this project's position.** The brief frames the
question as "an IR that is 5x slower than a tree-walker". The spec's first line is *"The IR is a
foundation, not a speedup"* (`2026-08-08-phase-4e-ir-design.md:10`), it withdraws the
`spike/bytecode-vm` figures outright, and it records at `:45` that no measurement shows the IR
arm faster. Its four stated reasons are: a patchable call site for inline caches, a shape a
compiler backend consumes, trace as an emission decision rather than a mode flag, and *"it gives
quickening and specialisation somewhere to live"*.

That is Deutsch and Schiffman (POPL 1984) and Brunthaler (DLS 2010) as the justification for an
IR, arrived at without citing them, and it is the justification the literature would give. **My
section 5 headline -- keep the IR, stop expecting it to pay by itself, use it as the carrier --
is the spec's own position restated, not a correction of it.** The one thing I would soften is
`:42`, "the win collapsing as work per clause rises is the signature of a dispatch cost, and
that is the reason to keep going": the same observation is also the reason *not* to expect much,
and Romer 1996 and Brunthaler 2009 are the sources that read it the second way.

Entry 11's own diagnosis -- *"a fact the parser knew is re-derived at run time"*, unifying the
line binary-search, the twice-run builtin name lookup, the re-split compound tail and the hashed
variable slot -- is a better statement of the problem than anything I wrote, and it is exactly
PEP 659's premise. Entry 11's closing observation that **a fixed per-clause cost gets more
valuable every time something else is removed** (11.4% -> 32.0%, 9.1% -> 17.4%, 6.2% -> 11.2%)
is a real Amdahl result and is the strongest argument against my ordering.

### Where I agree, with nothing to add

* Rejecting cheaper marking, generational and incremental collection because marking is not what
  is paid (`Heap::collect` self under 0.6%; `drop_glue::<Slot>` 4.7--7.4%). Correct method:
  measure where collector time goes rather than assume. Blackburn, Cheng and McKinley, *Myths
  and Realities* (SIGMETRICS 2004), is the paper that established doing it this way.
* Disposing of "widen `Op` to 16 bytes" on a 1.1--4.9% self-time measurement of the decode loop.
  That is the dispatch-optimisation question and the answer matches Romer and Brunthaler.
* Refusing `alloc4c` as an acceptance axis after a known +1% difference failed to hold its sign.
  Mytkowicz, Diwan, Hauswirth and Sweeney, *Producing Wrong Data Without Doing Anything
  Obviously Wrong!* (ASPLOS 2009), is the citation for why that caution is not excessive.
* Entry 11's conclusion that *"what is left after it is the object model -- how a value is
  allocated, held and freed -- rather than any block a profile names"*. This is my section 2,
  reached from the profile rather than from the source, and it is the correct reading of a
  residual that block-subtraction cannot account for.

### Where I disagree

**1. The ordering, and it is my only substantive disagreement.** Entry 11 makes expression-level
compile-time resolution the structural change and hands the object model on as *"a Phase 4f
finding to hand on rather than a candidate to attempt in it"*. I would bring the object model
forward, for three reasons the record's own numbers supply.

*(a)* On `arith`, the axis nearest the bar, the record's table gives allocation 39.9 of the 42.7
identified points and lands the axis at 1.22x. Expression resolution contributes 2.8. On the
axis where the queue nearly wins, the object model **is** the candidate.

*(b)* **The two changes interact in one direction only.** The larger half of what expression-level
compilation buys is not name resolution -- it is keeping an intermediate unboxed in a register.
That half is unreachable while `Number` owns a `Vec<u8>` and every `Text` owns a separate
`malloc`: a register-held intermediate that still allocates has moved the allocation, not removed
it. Doing the representation first makes expression compilation worth strictly more; doing
expression compilation first delivers only its smaller half and has to be revisited. Brunthaler's
own sequence is the precedent -- quickening and inline caching came first, and *Multi-Level
Quickening* exists because the boxing had to be attacked separately afterwards.

*(c)* The record already shows the object model swallowing candidates one at a time. Entry 11
lever (b) was *"set aside because the builtins build their result in a `Vec` before `Body` sees
it"* -- that is not a reason the lever fails, it is a description of the change. **The reference
does exactly this and the pattern is in its source**: `NumberBuilder(RexxString *s) :
current(s->getWritableData())` (`interpreter/classes/NumberStringClass.hpp:123`) allocates the
result object at its known length and writes into it. A builtin returning `Vec<u8>` cannot; a
builtin handed a writable destination can.

**2. `strings`' 4.35x "wall" is the same finding, and calling it a wall is premature.** Entry 11
says that after every candidate lands, `builtin::dispatch`'s subtree is about 2.5 s against the
oracle's 0.52 s *for the same iterations with name resolution already removed*. That 5x on the
bodies alone is not decomposed anywhere. My prediction from the code is that most of it is the
same object model: a result built in a fresh `Vec`, moved into a `Body::Text`, given a 96-byte
slot, and read back through a generation-checked handle. It is testable cheaply -- count
allocations per `SUBSTR` call on both sides -- and it should be tested before "wall" is written
down, because a wall is the one finding that stops work.

**3. Candidate 6 (`line_of`) has a better fix than the queued one.** The queue's answer is to put
the line on the instruction, as the oracle does. But the oracle does that because it is a
tree-walker with nowhere else to put it; an op stream does not need to. **`SIGL` is written at
control transfers and the line is read on a raised condition -- neither is per clause.** PEP
669's rule applies directly: do not compute, and do not even store, what nothing is observing;
recover the line from the pc through `Chunk::op_of` at the two moments it is actually read. That
removes the cost rather than making it cheaper, and the record's own counted probe -- this
crate's per-clause cost growing 16.1% with a program's line count, the oracle's not growing --
says the cost is worth removing rather than relocating.

**4. One caution on candidate 2.** The record is already appropriately sceptical that 32% of
`varlookup` in `leave_clause` is a mis-attributed stall. Worth adding only that a large `E` in
`Result<T, E>` penalising the success path is a well-known Rust codegen pathology rather than an
interpreter-design cost, the oracle pays 1.8--5.3% at the same boundary, and the change is one
type. It should be attempted regardless of where it ranks, because its cost is near zero and its
falsification test is cheap.

### The one-line answer to "is the technique the one the literature would pick"

**Yes.** Compile-time resolution carried by a patchable stream is quickening, and quickening is
what the literature picks for an interpreter at this abstraction level. The disagreement is only
that the literature would put the value representation *beside* it rather than after it, because
at this abstraction level the two are the same programme: Brunthaler's own line is that dispatch
and boxing are separate costs and the second is the larger one.

## Sources

- Larose, Kaleba, Burchell, Marr, *AST vs. Bytecode: Interpreters in the Age of Meta-Compilation*, OOPSLA 2023. https://stefan-marr.de/downloads/oopsla23-larose-et-al-ast-vs-bytecode-interpreters-in-the-age-of-meta-compilation.pdf and https://stefan-marr.de/2023/10/ast-vs-bytecode-interpreters/
- Romer et al., *The Structure and Performance of Interpreters*, ASPLOS 1996. https://alecw.azurewebsites.net/work/papers/asplos-1996.pdf
- Ertl, Gregg, *The Behavior of Efficient Virtual Machine Interpreters on Modern Architectures*, Euro-Par 2001. https://link.springer.com/chapter/10.1007/3-540-44681-8_59
- Brunthaler, *Virtual-Machine Abstraction and Optimization Techniques*, BYTECODE 2009. https://publications.sba-research.org/publications/bytecode09.pdf
- Brunthaler, *Efficient Interpretation using Quickening*, DLS 2010. https://publications.sba-research.org/publications/dls10.pdf
- Brunthaler, *Inline Caching Meets Quickening*, ECOOP 2010. https://publications.sba-research.org/publications/ecoop10.pdf
- Shi, Casey, Ertl, Gregg, *Virtual Machine Showdown: Stack Versus Registers*, VEE 2005 / TACO 2008. https://dl.acm.org/doi/10.1145/1064979.1064998
- Deutsch, Schiffman, *Efficient Implementation of the Smalltalk-80 System*, POPL 1984.
- Hölzle, Chambers, Ungar, *Debugging Optimized Code with Dynamic Deoptimization*, PLDI 1992.
- PEP 659, *Specializing Adaptive Interpreter*. https://peps.python.org/pep-0659/
- PEP 669, *Low Impact Monitoring for CPython*. https://peps.python.org/pep-0669/
- Blackburn, Cheng, McKinley, *Myths and Realities: The Performance Impact of Garbage Collection*, SIGMETRICS 2004.
- Mytkowicz, Diwan, Hauswirth, Sweeney, *Producing Wrong Data Without Doing Anything Obviously Wrong!*, ASPLOS 2009.
- Cowlishaw, *The decNumber C library* (DECDPUN). https://speleotrove.com/decimal/decnumber.pdf
- mpdecimal / libmpdec project (30x--80x over `_pydecimal`). https://www.bytereef.org/mpdecimal/
- This repository: `interpreter/classes/NumberStringClass.hpp:96,414`, `interpreter/classes/NumberStringMath2.cpp:138`, `interpreter/classes/IntegerClass.cpp:552`, `interpreter/classes/IntegerClass.hpp:185`, `rust/crates/rexx-num/src/lib.rs:407`, `rust/crates/rexx-num/src/addsub.rs:223`, `rust/crates/rexx-core/src/heap.rs`, `rust/crates/rexx-exec/src/ir/drive.rs`, `rust/crates/rexx-exec/src/value.rs`, `rust/crates/rexx-exec/src/eval.rs:742`
