# Practitioner consult: why the instruction stream is not winning

Written from the code alone, before reading `phase-4f-record.md` or the 4e design spec. A
second section comparing against those two follows at the end. No builds, no benchmarks,
no profiling were run; every claim below is either read out of source (with the file and
line) or flagged as a hypothesis to test.

---

## 1. Why an instruction stream is losing to a tree-walk

### The framing is the first thing I would change

"5-7x slower than a tree-walker while having an instruction stream" reads as a paradox and
it is not one. An instruction stream and a tree-walk differ in *how the interpreter arrives
at an operation*. They do not differ in what the operation costs. The reference's arrival
cost is a few nanoseconds out of a 60 ns clause; ours is a few nanoseconds out of 387 ns.
The IR is competing for a slice that was never large on either side.

What makes the C++ fast is not that it walks a tree. It is that its values are cheap:

* `RexxString` ends in `char stringData[4]` (`interpreter/classes/StringClass.hpp:817`) --
  a trailing flexible array. **A string is one allocation with its bytes inline**, from a
  segment the interpreter bump-allocates out of.
* `NumberString` ends in `char numberDigits[4]`
  (`interpreter/classes/NumberStringClass.hpp:416`) -- same shape. A number is one
  allocation with its digits inline.
* `CompoundVariableTail` is a **stack** builder with an inline buffer of
  `MAX_SYMBOL_LENGTH`, `tail = buffer`
  (`interpreter/classes/support/CompoundVariableTail.hpp:150-158`). Building a compound
  tail allocates nothing at all in the common case.

Against that, one of our short strings is a 96-byte arena slot **plus** a separate `malloc`
for the `Vec<u8>`, reached through a generation-checked handle. That is the 155 ns against
13 ns, and neither engine can do anything about it, because both engines share it.

So the question I would actually answer is: *why is the value layer 10x the reference's?*
The engine choice is close to orthogonal.

### The mechanisms, strongest evidence first

**M1 -- Every value read that is followed by any other interpreter call is a heap copy, and
the borrow checker is why.** This is the mechanism I think nobody has named, and I rate it
the single largest item.

`Interp::to_text(&mut self) -> Cow<'_, [u8]>` (`rexx-exec/src/value.rs:188`) borrows the
*whole interpreter*, because it lazily fills the `num`/`text` caches. So any code that needs
two operands' bytes at once, or bytes plus a later allocation, cannot hold the borrow --
it must buy its way out with an owned copy. Counted mechanically over `rexx-exec/src`
(`grep` for `to_text(` and for `to_vec()`/`into_owned()` on the same line, tests and doc
comments included, so the figure is soft): **77 of 135 mentions copy**. The pattern is not
soft. Three sites make it concrete:

* `builtin::required_string` is literally `interp.to_text(value).into_owned()`
  (`builtin/mod.rs:790`). **Every string argument of every builtin call is fully copied
  before the builtin looks at it.** `pos("fox", s)` mallocs and memcpys 3 bytes and 43 bytes
  to run a search that needs neither; `changestr("fox", s, "cat")` copies three arguments,
  then builds its result into a `Vec::new()` with no reservation
  (`builtin/string.rs:747`). This is the 2.5 s against 0.52 s. **It is not dispatch.** The
  builtin-dispatch subtree contains argument marshalling, and the marshalling is the cost.
* `Interp::concat_values` (`eval.rs:855-865`) builds the result into `bytes`, then calls
  `self.text(&bytes)` -- and `text` is `text_owned(bytes.to_vec())`. **Every `||` copies its
  full result twice.** One malloc, one memcpy and one free of the whole result length, per
  concatenation, for nothing. `self.text_owned(bytes)` is the entire fix.
* `Interp::stem_get` (`stem.rs:263-282`) computes `(resolved, name.clone())` -- it clones the
  stem's `Box<[u8]>` name **on every compound read, including hits**, where the clone is
  never used. The comment says it is cloned "rather than borrowed, for the same reason", the
  reason being borrow overlap. So the `compound` axis pays an allocation per access for a
  value it discards.

This mechanism is supported by the shape of the table, not just by reading. It predicts
that string- and parse-heavy axes are worst and string-free axes are best: `strings` 7.1x
and `rexxcps` 7.3x against `varlookup` 2.2x. That is what the table says.

It is also nearly invisible to review, because each site *looks locally necessary* and the
comment at each site explains the copy as a borrow requirement -- which reads as a
justification rather than as a price.

**M2 -- The object is 96 bytes with its payload somewhere else.** `Body`'s width is set by
`Body::Stem`, which holds a `HashMap<Vec<u8>, Option<ObjRef>>` (48 bytes) and a `Box<[u8]>`
inline (`rexx-core/src/body.rs:100-104`). Every `Body::Text` pays that width. So a 3-byte
string costs: a 96-byte write into the arena, a separate `malloc` for the `Vec`, and two
dependent cache misses to read it back. The reference costs one bump and one line. Boxing
`Stem`'s payload alone should take `Body` from ~80 bytes to ~24-32.

**M3 -- The handle is generation-checked, so every dereference is a bounds check, an enum
discriminant test, a generation compare and a dependent load** (`heap.rs:273-280`). The
reference dereferences a pointer. This is a fixed tax on every value touch, and value
touches are the inner loop of everything. I would call this a hypothesis rather than a
finding: I'd guess 5-15% overall, not the headline, but it compounds with M2 because the
two loads are dependent and neither is prefetchable.

**M4 -- Amdahl on the promotion set.** `compile.rs` promotes `Do`/`Loop`, `If`, `Select`,
`When`/`WhenCase`, `Assignment`, `Say` and `Call(Named)`; everything else falls to
`Op::Generic` (`ir/compile.rs:693`). Now look at what `rexxcps` executes per pass
(`samples/rexxcps.rex:107-149`): four `parse var`, a `parse value`, a `parse upper arg` in
the callee, `trace value`, `address value`, `iterate`, `leave`. **Every one of those is
`Generic`** -- the tree-walker's clause unit, with the driver's own loop on top of it.

So **on the benchmark with the worst ratio, the IR is closest to being pure overhead**, and
on the benchmark with the best ratio (`varlookup`, whose body is two promotable assignments)
it is closest to being fully exercised. The promotion set was chosen by what was tractable
to compile, not by what the bad benchmarks execute. That is a checkable claim and I would
check it before anything else in section 2: count `Generic` against promoted op executions
per axis. I expect `varlookup` near 100% promoted and `rexxcps` well under half.

**M5 -- Per-clause work that is a compile-time constant is recomputed at run time.**
`enter_stepped_clause` calls `clause_line` (`run.rs:7793`), which calls
`ProgramSource::line_of` (`rexx-parse/src/source.rs:266`), which is a `partition_point` --
a binary search over the line table, **once per clause, on both engines**, for a number
fixed when the instruction was parsed. The reference stores the line on the instruction
object. This is the *same defect class* as the seven already landed, and it is still there.

**M6 -- The IR removed the cheaper half of the clause and left the expensive half.** Per
promoted clause the driver still pays: `frames.pop_if` on an empty `Vec`, `pc >= stop`,
`op_at_index`'s bounds check, a 20-way match, an instruction bounds check,
`grant_procedure_permission`, the `chunk.trace() != self.chunk_trace()` staleness compare,
then `enter_stepped_clause` (clock invalidation, `>I>` decay, `printed_indent`, the
`line_of` binary search, `enter_clause`, `push_frame`), then the region loop, then
`leave_stepped_clause` (`pop_frame`, `leave_clause`, two `record_failure_site` guards).
The clause boundary is intact and it is the bigger half. The IR's ceiling on a clause-bound
benchmark was therefore always small -- and, importantly, most of that ceiling was
reachable *without* an IR, by attacking the boundary directly.

**M7 -- `Number` is a `Vec<u8>` of one decimal digit per byte** (`rexx-num/src/lib.rs:411`),
and `to_number` hands back an owned clone (`value.rs:305`), and `Number::from_i64` allocates
`Vec::with_capacity(20)`. So a general-path arithmetic operation is 3-5 mallocs. `arith`
survives at 2.2x only because `arith_small_int` covers the tagged case; `arith.rex`
deliberately switches to `DIGITS 20` and divides, so the general path is exercised, and the
reference does the same work with the digits inline in the object.

**M8 -- The compound tail is built on the heap where the reference builds it on the stack.**
`Interp::tail_key` (`stem.rs:110-126`) allocates a fresh `Vec<u8>` per compound access, and
for an integer subscript it goes through `to_text`, which for a `SmallInt` is
`Cow::Owned(n.to_string().into_bytes())` (`value.rs:191`) -- **a `String` allocation to
render an integer that is already in a register**. So `t.k = t.k + 1` is at least four
allocations per iteration where the reference has zero. `compound` is 2.9x and this is
essentially the whole of it.

---

## 2. What I would do next, in priority order

Value figures are my expectations, not measurements. Cost is engineering effort.

**P0. The three free ones, today (value: small but certain; cost: an hour).**
`self.text(&bytes)` to `self.text_owned(bytes)` in `concat_values` (`eval.rs:864`); hoist
`name.clone()` in `stem_get` into the miss arm only (`stem.rs:281`); reserve `changestr`'s
output buffer (`builtin/string.rs:747`). No interface change, no design decision, and they
are a *calibration*: measure the delta and compare it against what the profile predicted.
If the profile did not predict them, the residual figure in section 4 is wrong.

**P1. Stop copying bytes to satisfy the borrow checker (value: largest single item, I would
expect `strings` from ~7x toward 3-4x and a visible dent in the 155 ns; cost: medium-high,
but mechanical).** Change the value-read interface so a reader gets a view that does not
borrow `Interp` mutably. Two shapes, either works:
  * put the lazy caches behind `Cell`/`RefCell` so `to_text(&self) -> &[u8]` -- then
    `required_string` returns a borrow and the 77 copy sites mostly become borrows; or
  * make `Body::Text`'s payload an `Rc<[u8]>` so `to_text` returns a refcount clone.
  The differential harness is exactly the right instrument for a change of this shape: it
  is a pure representation change with no observable output. This is where the effort should
  go, and it is not an optimiser.

**P2. Make an allocation cheap (value: attacks the 155 ns directly, second-largest; cost:
medium).** Box `Body::Stem`'s payload so `Body` stops being 80 bytes for everyone. Then add
small-string optimisation: an inline payload of ~24-40 bytes in the slot covers the
overwhelming majority of Rexx values, including every rendered small integer and every
compound tail, and removes the second `malloc` and the second cache miss. This is the direct
analogue of `stringData[4]`.

**P3. Precompute what is per-clause-constant (value: small, certain, both engines; cost:
low).** Put the source line on the instruction, or in a parallel `Vec<u32>` in the plan, and
delete the `line_of` binary search from the clause boundary.

**P4. Build the compound tail without allocating (value: most of `compound`'s 2.9x, plus
`alloc4c` and `rexxcps`; cost: low-medium).** A reusable scratch buffer on `Interp` --
cleared, never reallocated -- with an integer rendered into it directly rather than through
`to_text`, and a `HashMap` lookup on the borrowed key. This is `CompoundVariableTail`, with
the stack buffer moved onto `Interp` because Rust will not let you keep it on the frame
across a `&mut self` call.

**P5. Count the Generic/promoted split per axis, then widen promotion where the count says
(value: unknown, which is the point; cost: an hour to count, high to act).** If `rexxcps` is
mostly `Generic`, promoting `PARSE` is the highest-value construct left -- and "promoting"
it need not mean writing a template matcher in ops. The already-proven trick is the one
`ExprCallSite` uses: resolve the template once at compile time and keep a per-site cache, so
the run-time path stops re-deciding what it already knows.

**P6. Only then, consider a real representation change** -- NaN-boxing, or a wider tagged
word so short strings avoid the heap entirely. This is the one item that is a rewrite. I
would not begin it until P1-P3 have landed, because P1-P3 are what tell you how much of the
gap was *representation* and how much was merely *copying*, and those need different
answers.

**What I would not do**, and would push back on: adding op variants, fusing ops into
superinstructions, computed-goto/tail-call threading of the dispatch loop, or widening the
register file. Every one of them buys a slice the measurements say is a few nanoseconds
wide, and every one enlarges the correctness surface that the differential harness has to
hold.

---

## 3. Where the line is between removing redundant work and writing an optimiser

The line I use: **an optimiser changes which operations run, by reasoning about the
program**; removing redundant work changes what one operation costs, without changing the
sequence. Constant folding, escape analysis, loop-invariant motion, type specialisation and
inline caching are on the optimiser side. Not copying a string, not re-deriving a constant,
not allocating 96 bytes for three are on the other side.

**This system is not at that line, and I do not think the seven landed changes moved it
toward the line -- they moved it *along* the line.** Every one of them, and every one of
P0-P4 above, shares a property: a correct implementation of *this same design* would never
have done the work in the first place. Copying a string to end a borrow, rendering an
integer to text to build a hash key, binary-searching for a value fixed at parse time,
cloning a name you then discard -- these are not optimisations foregone. They are the cost
of having built the value layer before there was a performance model to build it against.

Exactly one thing in the tree crosses the line today: `Hints`/`PatchSlot` quickening, which
is genuine inline caching. And the `QUICKENING` const that lets you price it in one edit is
the right instinct -- it is the discipline the rest of this list needs.

But there is a sharper distinction available than "same kind or different kind", and I think
it is the one that matters here. **The seven landed changes were all *local*: they removed
work at a site without changing an interface.** The remaining ones are not. P1 is an
interface change to the value model; P2 is a representation change. That is the real
transition this project is standing at, and it will *feel* like a rewrite while still not
being optimisation.

The failure mode I would guard against is concluding "we have exhausted the redundant work,
so the next phase is optimisation." That conclusion sends you to build op fusion and a
peephole pass, and the measurements say those are worth almost nothing here. The honest
version is: "the local redundant work is exhausted; the non-local redundant work is not, and
it is bigger."

---

## 4. What I think is being got wrong

**(a) The headline framing, restated.** "We have an instruction stream and we are still 5-7x
a tree-walker" makes the IR look like the thing to iterate on. It is not. The IR is roughly
neutral; the gap lives in the value layer, which both engines share, and which no amount of
IR work touches. This is the most expensive wrong idea in the brief.

**(b) The "removing every identified cost still leaves 4.4x and 5.5x" summation is the
conclusion I would most want re-derived, and I think it is currently unsupported.** Summing
independent removals assumes the costs are additive and disjoint. The ones I found are
neither: the argument copy, the payload `malloc`, the 96-byte slot write, and the collection
that the churn eventually triggers are **one cost with four names**. If the profile
attributed them to four separate blocks and the summation subtracted all four, the residual
is *inflated* and the real post-fix number is better than 4.4x. If the profile grouped them,
the residual is a genuine unexplained remainder and there is a mechanism nobody in this
consultation has named -- which would be much more interesting and would change the plan.
Those two cases lead to opposite decisions about whether P1/P2 are worth doing, so the
question is worth an hour. **P0 is the experiment**: land the three free fixes, measure, and
compare the delta against what the profile predicted. If they disagree, withdraw the 4.4x
and 5.5x figures.

**(c) The mechanism nobody has named, in one sentence: in this codebase the borrow checker
is a performance decision, and it has never been priced.** Every `&mut self` on a value
accessor forces its callers to buy an owned copy. It shows up in a profile only as `malloc`,
it shows up in review as a locally-justified line, and the comments at those sites *explain*
the copy rather than flagging it. `stem.rs:281` and `builtin/mod.rs:790` are the two clearest
instances, and the second one is on the hot path of the worst benchmark.

**(d) The `Op` width budget is optimising the wrong thing and may be obstructing.** The
`assert!(size_of::<Op>() == 12)` and the reasoning around it ("every op pays for the widest
variant") is correct discipline *in a bytecode VM where dispatch dominates*. Here dispatch
does not dominate, and that budget is already distorting design: it is the stated reason
`PlanSlot` is a sentinel `u32` rather than an `Option`, and the reason `ExprCallSite`'s
`base`/`argc` had to be pushed off the op into a side table. A width budget defended by an
assertion is a strong constraint; it should be defended by a measurement, and there isn't
one.

**(e) The most informative pair in the table is not being used.** `varlookup` at 2.2x and
`rexxcps` at 7.3x are both clause-rate benchmarks. What separates them is *which constructs
run* and *whether strings are touched*. That single contrast localises the gap to the value
layer plus the unpromoted constructs, and it does so with no profiler at all. I would put it
at the top of any writeup of this work, because it is the cheapest piece of evidence
available and it points at the right two things.

**(f) A smaller one, offered because it is easy to fix.** `builtin::dispatch` hashes the name
into a `HashSet` (`is_builtin`) and then does `IMPLEMENTED.iter().find(...)`, a **linear scan
with byte-slice comparison over the whole builtin table** (`builtin/mod.rs:707`). The
`ExprCallSite` spike caches past it, so this may already be off the hot path for promoted
call sites -- but every unpromoted call, and the tree-walker arm entirely, still pays it. If
the "one builtin-dispatch subtree at 2.5 s" measurement was taken on a path that reaches
`dispatch` rather than `dispatch_at`, part of that number is the scan and not the
marshalling, and the two want different fixes. Worth separating before acting on it.

---

---

# Section 2: after reading the project's own conclusions

*(Written after the above, having then read `docs/superpowers/plans/phase-4f-record.md`.
Section 1 is left exactly as written; the corrections are here.)*

The record is more rigorous than the consultation brief suggested, and it has already reached
several of my conclusions by better routes. **The brief handed me entry 1's table.** Entry 11
measures head at `alloc4c` 1.27x, `arith` 2.13x, `varlookup` 2.16x, `compound` 2.42x,
`strings` 6.27x, `rexxcps` 6.44x -- so parts of section 1 are aimed at a state that no longer
exists. Where that changes an answer I say so below rather than leaving it to be inferred.

## Where I was wrong

**1. My headline mechanism -- M1, the borrow-checker copies -- is measured and has largely
evaporated, and my ranking of it is wrong.** Entry 4 measured `required_string`'s `to_vec` at
**15.5% of `strings`** and explicitly set it aside as "a borrow-structure change and not a
representation", deserving a candidate of its own. Entry 11 re-measured it at **about 3.5%**
and struck it off. So it is not "the single largest item"; it is a class that two accepted
changes diluted. What survives of the mechanism is narrower and I state it as C below: the
*class* is real and unnamed, and two of its instances are still in the tree and unqueued.

**2. My M2/P2 -- "make an allocation cheap" -- is aimed at the wrong axis, and the record's
oracle-side measurement is what kills it.** On `strings`, this crate spends 27.5% in
allocation and **the oracle spends 35.9% of its own run in `newObject`**. Allocation is not
that axis's differentiator at all, and entry 11 says so explicitly ("this is the one place
the queue was aimed at the wrong axis"). It *is* the differentiator on `arith`: 38.9% here
against the oracle's whole `newObject` subtree at 8.3%. The record's candidate 1 states that
split per axis; my section 1 did not have it and would have sent effort at `strings`.

**3. My "box `Body::Stem`" suggestion is smaller than I implied and I should not have led
with it.** Entry 4 records `Body` at 80 bytes "dominated by the cold `Stem` variant". Boxing
`Stem` leaves `Body::Num` -- a `Number` (32) plus `created_digits`, `created_form` and an
`Option<Vec<u8>>` -- as the next widest, around 72. So the change buys roughly a tenth, not a
half. Getting a genuinely small uniform slot needs the object-model work entry 11 hands on,
not one `Box`.

**4. My criticism (b), that the residual figures are an unsound summation, is largely
answered and I withdraw most of it.** Entry 11 does the three things I asked for: it sums only
"the identified blocks the oracle does **not** also pay", it deliberately excludes the
reclamation row to avoid double-counting the `free()` already inside the allocator's self
time, and it labels the whole construction "the loosest possible bound". More importantly it
**corroborates the two wall figures with absolute comparisons that do not depend on
additivity at all**: on `strings`, about 3.9 s of string work against the oracle's 0.55 s, and
`dispatch`'s subtree at 2.5 s against 0.52 s; on `rexxcps`, 387 ns per clause against 60, with
the allocation term substituted at the oracle's own 13 ns still leaving 4.1x. Those are the
load-bearing figures and they are sound. My proposed experiment (P0 as calibration) is
therefore not needed for that purpose, though the two fixes in C are still worth taking.

**5. My M4 is directionally right and badly scaled.** I argued the IR is close to pure
overhead on `rexxcps` because its clause mix is unpromoted. The promotion analysis is
correct -- `PARSE`, `TRACE VALUE`, `ADDRESS VALUE` all compile to `Op::Generic` -- but entry
1 measures the IR arm on `rexxcps` at **+3.24%** against the tree-walker, not at some large
fraction. It is a three-per-cent story, not a seven-times one. `rexxcps`' 6.44x is per-clause
and per-call costs both engines share, which entry 11's profile shows directly.

**6. My criticism (a), the headline framing, is aimed at the brief and not at the record.**
Entry 1's own "four things worth carrying forward" leads with "the IR arm is behind the
tree-walker on more axes than it is ahead", and entry 11 closes with "what is left after it
is the object model -- how a value is allocated, held and freed -- rather than any block a
profile names". The project is not confused about this. I withdraw it as a criticism and keep
it only as a caution about how the question was put to me.

**7. Three of my priorities are already the queue, with better mechanisms than mine.** My P3
is candidate 6, with a control I could not have proposed -- padding a program with 0, 1,000
and 20,000 comment lines and watching this crate's instruction count rise 8.8% and 16.1%
while the oracle's does not. My P4 is candidate 3, and the record's mechanism is sharper than
mine: not `tail_key`'s `Vec` but `compound_parts` **re-splitting the symbol's source
spelling** on every access, measured by lengthening the stem's name by 34 characters for
+687 instructions per pass here and +0.001% on the oracle. My (f) is candidate 4, measured
far better than I guessed: `is_builtin` runs *twice* per call, and the table scan costs about
3.3 instructions per entry, so `SUBSTR` at index 42 pays roughly 135 instructions of pure
walk per call.

**8. Where we independently agree, which I record as confirmation rather than contribution.**
My section 3 -- that what remains is the same kind of thing as the seven landed changes, but
non-local rather than local -- is entry 11's own closing finding reached from profiling
instead of from reading: "every candidate 3 to 6 above is the same defect wearing a different
coat: a fact the parser knew is re-derived at run time", and "the structural change is to
extend compile-time resolution from statements to expressions". Two routes, one answer.

## Where I still think I have something

**A. The one I would act on: `find_forward` is naive, the oracle's is not, and the record
classifies the difference as shared work.** Entry 11 reads `_memcmp_evex_movbe` at 14.8% of
`strings` and splits it "6.3 points is that table scan and **7.7 is genuine string searching
inside `CHANGESTR` and `POS`**". "Genuine" reads as a cost both sides pay. It is not:

* Ours (`builtin/string.rs:149-162`) is `window.windows(needle.len()).position(|candidate|
  candidate == needle)` -- a slice comparison, lowered to a `memcmp`, **at every offset**.
* The oracle's `StringUtil::pos`
  (`interpreter/classes/support/StringUtil.cpp:206-246`) calls **`memchr` for the needle's
  first byte**, then tests the second byte inline as a scalar, and only then calls `memcmp`
  on `needle_length - 2` bytes. For `pos("fox", <43 bytes>)` that is one SIMD pass and two
  byte compares against our ~41 `memcmp` calls.

It compounds in `CHANGESTR`, which runs `count_occurrences` -- itself a chain of
`find_forward`s -- over the whole haystack and *then* re-runs `find_forward` per change, so
the subject is scanned naively twice over, and builds its result into an unreserved
`Vec::new()` (`builtin/string.rs:747`).

This sits **inside the 4.35x residual entry 11 calls a wall**, and it is not on the queue
because it was classified as shared. Cheap to falsify with the record's own instruments:
mirror the oracle's algorithm and count instructions on a `pos`-only probe. **And it needs no
new dependency decision** -- `memchr` is already in `Cargo.lock` and five versions sit in the
offline cache, and it is pure Rust, so it does not raise entry 8's C-library question at all.
Even without it, the oracle's own two-step is five lines.

**B. A method point the queue's own ranking line invites.** Entry 11 ranks "by the largest
share on a bar-bound axis", and separately computes, per axis, the blocks "the oracle does
not also pay". On the two axes that are walls those are different orderings, and the second
is the right one. A block at 27.5% of our `strings` time against the oracle's 35.9% has
*negative* excess and should never have been ranked first there -- which the entry discovers
by inspection two paragraphs later. Conversely a block at 3.5% of our time and 0% of the
oracle's is 4.2% of the excess, which is the number a candidate aimed at closing a gap should
be judged on. **Rank by share of the excess, not share of the axis.** The data is already in
the entry; only the sort key changes.

**C. Two concrete allocations that are free to remove and are on no queue.** Both are the
class from my M1, surviving in the two places the accepted changes did not reach.

* **`Interp::concat_values` copies its whole result twice** (`eval.rs:855-865`): it builds
  `bytes`, then calls `self.text(&bytes)`, which is `text_owned(bytes.to_vec())`. One malloc,
  one memcpy and (since entry 6) one real `free` of the full result length, per `||`.
  `self.text_owned(bytes)` is the whole fix. `strings` concatenates 46 bytes 3,000,000 times;
  `alloc4c` once per pass; `rexxcps` in `1.0''loop` and in its `CALL` argument.
* **`Interp::stem_get` clones the stem's name on every read, including hits**
  (`stem.rs:263-282`): the block computes `(resolved, name.clone())` unconditionally and the
  clone is used only on the miss path. A `Box<[u8]>` clone is a malloc, a memcpy and a free,
  once per compound access -- twice per pass on `compound`, 5,000,000 passes. It is *adjacent*
  to candidate 3 and is not candidate 3: that one is the tail **split**, this is the stem's
  **name**, and hoisting it into the `None` arm is a two-line change with no sharing-rule
  surface. Its comment even states the reason -- "cloned rather than borrowed" so the borrow
  does not overlap -- which is the class named.

**D. Four "risk to the sharing rule: real" notes that are one design decision.** Candidates 3,
4, 5(ii) and 6 each carry a warning that the resolved fact "belongs on the plan or the
instruction, where both engines read it, not on an `Op`". That is one mechanism written four
times: **a per-node resolution side-table on the `Plan`, keyed by expression-node identity and
read by both engines**, which is the generalisation of what `ExprCallSite` already does for a
single node kind. Building it once discharges all four risks with one correctness argument
instead of four, needs no `Op` field (so the 12-byte assertion stays out of the way), and is
exactly entry 11's own "extend compile-time resolution from statements to expressions" given
a shape. I would build that before any of the four as local fixes.

**E. "The object model" is four separable decisions and naming them now is worth doing.**
Entry 11 hands the residual on as one thing. It is at least four, and only two of them are
what "inline the payload" addresses:

1. **How many allocations per value** -- two today, a slot and a payload `malloc`. The oracle
   has one: `RexxString` ends in `char stringData[4]` (`StringClass.hpp:817`) and
   `NumberString` in `char numberDigits[4]` (`NumberStringClass.hpp:416`).
2. **How wide the slot is** -- 80-byte `Body` for a `Text` that needs 40.
3. **How many dependent loads to read a value's bytes** -- two, plus a bounds check, an enum
   discriminant test and a generation compare (`heap.rs:273-280`). The oracle dereferences a
   pointer. Nothing on the queue touches this and no profile will show it as a block; it is
   spread over every value touch.
4. **Whether reading a value's bytes borrows the interpreter mutably** -- it does, which is
   the class in C above and the reason `required_string` was ever written that way.

A phase that attacks (1) and (2) will find (3) and (4) waiting, and (4) is the one that can
be attacked independently and cheaply, by moving the `num`/`text` lazy caches behind
`Cell`/`RefCell` so `to_text` can take `&self`.

## One thing I would not change

The record's method. The ABBA ordering, the `/proc/stat` idle gate, re-counting BASE rather
than inheriting it, entry 10 withdrawing entry 9's displacement figures on the ground that
three reps do not resolve half a per cent, entry 8 withdrawing entry 6's own headline as
understated by half -- that is better measurement discipline than most production VM work
gets. My section 4(b) was written on the assumption it was absent. It is not.
