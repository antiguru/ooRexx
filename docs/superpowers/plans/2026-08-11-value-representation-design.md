# The value representation decision

**Written 2026-08-11.**
**This document decides nothing by itself.** It sets out options, their blast radius, what the profile suggests each is worth, and an order.
The 4f loop's accept rule decides which of them land, one at a time, against the immediately preceding binary.

**Why it exists.** `2026-08-09-phase-4f-optimisation-loop.md` says under "What this phase does not do": *"It does not decide representation questions on its own authority... That is a decision to be taken deliberately, with both wordings recorded, not absorbed into an optimisation attempt."*
Two independent consultations, recorded as entry 11 of `phase-4f-record.md`, both concluded the loop has been optimising the wrong layer.
A representation change is not one change; it is a decision about the value layer from which several candidates follow, and the accept rule cannot judge a decision.

**Nothing here was built, benchmarked or profiled.** Another agent held the machine for the expression spike throughout.
Every share below is quoted from `phase-4f-record.md` -- chiefly entry 11's single profiled run per axis per side, with the entry named wherever it is not entry 11 -- and every figure derived from source is labelled as an estimate.
`rust/crates/rexx-exec/src/` was read through `git show HEAD:` because that crate had uncommitted spike changes while this was written; `rexx-core` and `rexx-num` were read from a clean working tree.
**`HEAD` moved three times during the writing**, the spike landing at `b946381e0` and the `unsafe` policy changing at `140acfc37` and `320c2232d`.
Every `rexx-exec` line number cited below was re-checked against `b946381e0` after that, and the two changes that bear on the argument are recorded in their own sections rather than folded in silently.

## What the source material establishes

The reference's values are cheap, and both consultations cite the same C++:

* `RexxString` ends in `char stringData[4]` and `NumberString` in `char numberDigits[4]` -- trailing flexible arrays, so a string or a number is **one allocation with its bytes inline**.
* Arithmetic runs in `char resultBufFast[FAST_BUFFER]` with `FAST_BUFFER = 48` -- a stack buffer, no heap traffic for intermediates.
* `RexxInteger::plus` adds machine integers when both operands are valid under the current `DIGITS`, and `-10..100` are interned.
* `CompoundVariableTail` builds a tail in a stack buffer, allocating nothing in the common case.

Ours, verified at `HEAD`:

* A `Body::Text` is a slot in `Heap::slots` **plus** a separate `malloc` for its `Vec<u8>` (`rust/crates/rexx-core/src/body.rs:66`), reached through a generation-checked handle (`rexx-core/src/handle.rs`, `ObjRef::decode`).
* `Number` is `{ negative: bool, digits: Vec<u8>, exponent: i32 }` -- **one heap byte per decimal digit** (`rust/crates/rexx-num/src/lib.rs:407`).
* `Interp::to_number` (`rexx-exec/src/value.rs:282`) returns an owned clone -- its `Body::Num` arm is `Ok(value.clone())` -- so reading a numeric variable allocates.
* `Interp::text` is `self.text_owned(bytes.to_vec())` (`value.rs:46`), so any caller holding bytes pays a second full-size copy to make a value of them.
* `Interp::to_text` on a tagged integer is `Cow::Owned(n.to_string().into_bytes())` (`value.rs:191`) -- a `String` allocation to render a value already in a register.

**And one mechanism neither the record nor Phase 4e had named.**
`to_text(&mut self, value: ObjRef) -> Cow<'_, [u8]>` borrows the whole interpreter, because it lazily fills the `num`/`text` caches, so a caller needing two operands at once buys its way out with an owned copy -- visible in a profile only as `malloc`.
Its own doc comment states this as an interface fact: *"`&mut self` and not `&self`... because filling that cache mutates the heap object the first time this is asked about it."*

**A finding this document adds, read out of that function at `HEAD` and not measured.**
Of `to_text`'s five arms, exactly one mutates.
`Decoded::Nil` returns a borrow of a literal, `Body::Text` returns `Cow::Borrowed(bytes.as_slice())`, and `Body::Stem` returns `Cow::Borrowed(&**name)` -- none of the three writes anything, and the function reaches them through `heap.get_mut` only because the `Body::Num` arm's `text.get_or_insert_with` needs it.
So the `&mut self` that forces the copies is paid by every reader of a string on behalf of the one arm that renders a number.
That makes option D below much cheaper than "put the caches behind `RefCell`" implies, and it is the single most actionable thing in this document that is not already on the queue.

## What D1 says, quoted

D1 is `2026-07-27-rust-rewrite.md:111-132` with its measurement in `docs/superpowers/plans/d1-decision.md`.
It is closed as (a), arena with tagged generation-checked handles, and nothing here reopens that.
Four passages bear on this decision.

**The pre-registration, `2026-07-27-rust-rewrite.md:2391`:**

> **Pre-registered: the string-representation problem.** C++ stores string bytes *inline with the object header* -- `RexxString` ends in `char stringData[4]`, the flexible-array-member idiom, so a string is one variable-sized allocation with its header and bytes contiguous... The arena as specified in Task 1.2 uses `Body::String(String)`, which is a fixed-size slot *plus* a separate heap buffer: **two allocations and a pointer chase for the single most common object in the language.**

**The fix D1 named, `:2393`:**

> The fix, if the measurement demands it, is a **side byte-arena**: `Body::String { offset: u32, len: u32 }` indexing a `Vec<u8>` string heap held alongside the slot vector... It works because `RexxString` is **immutable** in ooRexx -- strings are never modified in place, so no slot ever needs to grow.

**The guidance against boxing, `:2388-2389`:**

> * *The `Body` enum is too wide.* Every slot costs `size_of::<Body>()`, set by the largest variant. Box the large, rare variants and re-measure.
> * *Strings are paying two allocations where C++ pays one.* **This is the more likely cause, and boxing makes it worse.**

**The trigger, `:2395` and `d1-decision.md`:**

> Do not build the byte-arena speculatively. Build `Body::String(String)` first because it is simpler, measure, and reach for this only if the number says to.

and, from `d1-decision.md`, *"If the Phase 4 re-measurement misses parity, this is the first thing to try."*

### Verdict: this refines D1 and does not contradict it

**The trigger D1 set has fired.**
`d1-decision.md`'s Phase 4 addendum re-measured full-GC pause at 28.5-29.1 ms against the oracle's 17.966 ms, **1.58x-1.66x**, and records it as *"a genuine miss, not a number massaged to clear a bar"* -- missing not only parity but the Phase 1 threshold of 1.5x the stale figure had been inside.
D1's own instruction for that outcome is to reach for the representation fix.
Doing so now is following D1, not overriding it.

**Three refinements, each stated so a later reader can see what changed.**

* **D1 named one fix; the profile and the reference point at a different one.** D1's side byte-arena removes the second `malloc` but keeps two dependent loads and adds compaction to the sweep. An inline payload in the slot removes the second `malloc` *and* the second load for short values, needs no compaction, and is what Ruby, CPython and ooRexx itself all do. The byte-arena is not withdrawn; it is option E below, ranked behind the inline payload and conditional on a measurement nobody has taken.
* **D1's "boxing makes it worse" is about string bytes and remains correct as stated.** `d1-decision.md`'s Phase 4 addendum already says so: *"that note is about boxing string bytes specifically... not about whether a large, cold variant like `Stem` should stay inline in the same enum as the hot ones."* The 4f plan records this as a tension. It resolves by splitting the sentence into two populations rather than by overruling either half -- see "what the object model needs" below.
* **D1's file already pre-registers the numeric half, in a place the 4f plan does not cite.** `d1-decision.md`'s Phase 2 addendum says closing arithmetic's gap *"means attacking the digit-per-`u8` representation and the fresh `Vec` per result -- the same class of change this document already pre-registers for strings, and for the same reason: one allocation per value where the C++ has none."* Option B is therefore also a D1 item, not a new one.

**Where there is genuine tension, and it is not about boxing.**
D1's evidence rule is *"Decision closes on numbers, not argument"*, and this document is argument.
That is why it produces options and an order rather than a chosen representation: each option lands through the loop's paired accept rule, with a pre-stated falsifier, and the decision is what the runs say.

## What the expression spike measured, and what it does to every share below

**This landed at `b946381e0` while this document was being written, and it is the most recent evidence about what representation work has to close.**
Entry 11 named expression-level compile-time resolution as the structural change and priced it at 26.8 points of `strings` and 14.5 of `rexxcps` by summing shares the same entry says are not additive.
The spike measured it instead: `strings` **-28.24% wall** over 7 of 7 rounds, and `rexxcps` **+3.76% clauses per second**, about a third of the estimate.

**Both consultations predicted this shape, and it is the strongest argument for the options below.**
Expression promotion is necessary and it is measured, and it leaves `strings` at roughly 4.5x and `rexxcps` at roughly 6.2x -- both still walls.
Entry 11's own closing sentence was that *"what is left after it is the object model -- how a value is allocated, held and freed -- rather than any block a profile names"*, and the spike is that sentence with a number under it.

**And it moves the denominators of every share quoted in this document.**
The 4f plan's own opening argument applies: *"An attribution has a shelf life measured in landed changes."*
Every figure below is from entry 11's profile at `2bbecac9`, taken before the spike.
If expression promotion lands, `strings` shrinks by about a quarter and every share of that axis grows accordingly, while `rexxcps` barely moves.
**So the per-axis rankings here are stable and the absolute ceilings are not**, and a third profiling pass belongs between the spike landing and any option below being pitched on a `strings` number.

## The options

Every option below is entered from both engines by construction, and this is checked rather than assumed.
`ir/drive.rs`'s own comments at `HEAD` record that its native expression ops go *"through the same `Interp::read_symbol` that `eval_node`'s `Variable`/`Stem`/`Compound` arms enter"* and *"through the same `Interp::arith_small_int` and `Interp::arith_general` that `eval_arithmetic` enters"*.
So `Heap`, `Body`, `Number` and the `Interp` value accessors sit below both arms, which is entry 11's own *"Risk to the sharing rule: none. `Heap`, `Slot` and `Number` sit below both engines."*
**The one option where the rule bites is D**, because it changes an interface rather than a representation, and a borrowing accessor added for the IR while the tree-walker keeps the copying one is two implementations of one semantics.

**None of the five needs `unsafe`, and that is the point of the option set rather than a happy accident.**
Every shape below -- an inline-buffer enum, `Cell`/`RefCell`, `Rc<[u8]>`, a side byte-arena -- is ordinary safe Rust.
**What is not available in safe Rust is the reference's own trick**: `char stringData[4]` is a flexible array member, so the header and a run-time-sized payload share one allocation, and there is no safe spelling of that.
The option set is deliberately the set of things that reach the same *effect* without it: an inline arm bounds the payload at compile time instead of at run time, which is what makes it expressible.

**The project's `unsafe` policy changed on 2026-08-11, while this document was being written, and it changed because of this work.**
`140acfc37` and `320c2232d` record it: `unsafe` is now discouraged rather than banned, may be allowed in a specific self-contained situation by Moritz's decision per site, and the workspace lint is `unsafe_code = "deny"` (`rust/Cargo.toml:17`) rather than `forbid` -- `deny` still fails the build on any unapproved `unsafe`, but unlike `forbid` it leaves an approved per-site exception expressible.
`140acfc37`'s message names the trigger as this decision: *"whether that shape is reachable in safe Rust is now an open question rather than a closed one."*

**Nothing here asks for it, and none of the five options should be read as opening the door.**
`rust/CLAUDE.md` states the bar a site must clear: the unsafety confined to one module behind a safe interface, the invariant stated at the site and checkable by reading that module alone, a safe implementation described and *its cost measured rather than assumed*, and a test that fails if the invariant breaks -- and *"a performance argument alone is not enough -- the measurement has to exist first."*
**Read against that bar, the correct sequence is exactly the one recommended below**: the safe options are the ones whose cost has to be measured before any `unsafe` alternative can even be proposed.
If A measures well, the question closes; if A measures badly *because* the inline arm's fixed bound is the wrong shape, that measurement is the evidence a flexible-array site would need, and it is Moritz's decision and not a candidate's.

Entry 8's allocator swap, lever (c) of candidate 1, is an adjacent decision of the same kind but not the same one: `libmimalloc-sys` and `tikv-jemalloc-sys` compile and link a C library in a clean-room Rust reimplementation on five platforms, which is a policy question about the clean-room claim rather than about `unsafe`.

### A -- inline the payload of a short `Body::Text`

**What it changes.** `Body::Text { bytes: Vec<u8>, num: ... }` becomes `Body::Text { bytes: Bytes, num: ... }` where `Bytes` is an enum with an inline arm carrying a length and a fixed byte array, and a heap arm holding the `Vec<u8>` for anything longer.
Construction goes through `Interp::text_owned`, which is already the one place a `Body::Text` is built (`value.rs:123`).

**Blast radius.** `rexx-core` (`body.rs`, `heap.rs`) and `rexx-exec` (`value.rs`, `stem.rs`, `eval.rs`).
Both engines, automatically, through `Interp`.
`rexx-num` does not depend on `rexx-core` and is untouched.

**What it is worth, estimated from entry 11 and not measured.**
On `rexxcps` the allocator family is 40.0% of our run against the oracle's `newObject` at 22.2%, so roughly 18 points of excess -- but **nobody has decomposed that 40% into `Text`, `Number` and tail keys**, so the share of it this option reaches is unknown.
On `compound` the allocator family is 12.1%.
On `strings` it is 27.5% against the oracle's 35.9%, which is **negative excess**: entry 11 says in terms that *"a candidate aimed at `strings` through this lever is aimed at the wrong thing"*, and this option must not be sold on that axis.
It also removes the per-payload `free()` for every short string, which is candidate 7's `drop_glue::<Slot>` at 4.7% to 7.4% -- **and that must not be added to the allocator row**, because entry 11 records that the `free()` is already counted inside the allocator family's self time.

**What it costs.** Medium.
No `unsafe`.
**It probably costs nothing in slot width, and that is an estimate from recorded figures rather than a measurement.**
`d1-decision.md` measured `size_of::<Body>() == 80` at `ad7c36f0`, dominated by `Stem`, with `Text`'s own payload *"around 40 bytes"*.
An inline arm of about 22 bytes keeps `Bytes` the same width as the `Vec<u8>` it replaces, so `Text` stays near 40 and `Body` stays at `Stem`'s 80.
**Falsified by one line I was not permitted to run**: print `size_of::<Body>()` and `size_of::<Bytes>()` at `HEAD`, before and after.

**Landable and measurable alone.** Yes.
Pre-stated falsifier: a paired run moving `rexxcps` clauses per second by less than about 5%, or peak resident set not falling.

### B -- an inline digit buffer for a small `Number`

**What it changes.** `Number::digits: Vec<u8>` becomes an enum with an inline arm sized for the default `NUMERIC DIGITS 9` and comfortably beyond it, and a heap arm for the rest.
This is the reference's `FAST_BUFFER = 48` answer, in the one place it belongs.

**Blast radius.** `rexx-num` alone.
`digits` is `pub(crate)` (`rexx-num/src/lib.rs:411`), so no other crate can name it, and the interpreter is untouched unless a constructor is added.
That makes B the most contained option here.
Both engines get it for free, since both reach arithmetic through `Interp::arith_small_int` and `Interp::arith_general`.

**What it is worth, estimated from entry 11 and not measured.**
On `arith`, `drop_glue::<rexx_num::Number>` is 12.2% and `Number::clone` 7.7%, and entry 11 names both as the digit `Vec<u8>`.
That sits on an axis where our allocator family is 39.9% against the oracle's whole `newObject` subtree at 8.3% -- so on `arith`, unlike `strings`, nearly the whole share is a differentiator.
**What it is not.** Our decimal kernels are already about **1.65x faster than the oracle's in absolute self time** (entry 11: ours 0.527 s of a 2486 ms run, the oracle's 0.871 s of a 1175 ms run), so this is not a kernel change and must not be pitched as one.

**What it costs.** Medium.
No `unsafe`.
It is a rewrite of the digit representation reaching every module of `rexx-num` that does arithmetic, comparison or formatting, and its helper binaries.
**Do not inherit the "five files" figure from entry 2, and do not inherit a list from this sentence either**; scope it against the tree at the time, because that set moves.

**Landable and measurable alone.** Yes, and most cleanly of the five.
Entry 11 already states the falsifier: *"a paired run of lever (a) moving `arith` by less than about 15%; or peak resident set not falling."*

**One warning this option carries and cannot discharge.**
Entry 2 records that `smallvec` for `Number::digits` was *"measured and rejected"* -- and that citation exists nowhere in this repository.
I re-checked on 2026-08-11: a case-sensitive search of the working tree for `smallvec` across `.md`, `.rs`, `.toml` and `.lock` finds it only in prose sentences that cite it and in one Phase 4e report noting the same absence.
**`Cargo.lock` does not contain it**, so it is not a dependency and not a transitive one, and there is no measurement behind it.
**Treat B as unmeasured, not as previously refuted.**

### C -- intern small integers: do not do this, and here is why the profile looks like it says otherwise

**Recommendation: no.**
The reference interns `-10..100` as `RexxInteger` objects because every integer it does not intern is a heap allocation.
We do not allocate for integers at all: `ObjRef` tags them inline over `SMALL_INT_MIN..=SMALL_INT_MAX`, which is `-(1 << 61)` to `(1 << 61) - 1` (`rexx-core/src/handle.rs:49-50`), and `Interp::counted` and `Interp::number` already route results onto that tag at construction (`value.rs`).
Entries 3, 4 and 9 are all accepted changes that widened what reaches it.
**Interning would add a table lookup to a path that currently costs a shift and a mask, and it would cover the reference's 111 values where the tag already covers plus or minus 2^61.**

**What the profile shows that looks like this option, and what it actually is.**
`spec_to_string::<i64>` is 4.6% of `compound` and 3.4% of `rexxcps`.
That is **rendering** a tagged integer to bytes, not allocating one, and interning removes none of it.
Its source is `to_text`'s `Decoded::SmallInt(n) => Cow::Owned(n.to_string().into_bytes())` (`value.rs:191`) and the compound tail builder, which reaches it through `Interp::tail_key` (`stem.rs:110`) to append an integer subscript to a key it is building anyway.

**Do this instead, and it is small.**
Render an integer into a caller-supplied buffer rather than into a fresh `String`, so `tail_key`'s `key.extend_from_slice(&self.to_text(value))` writes the digits straight into the key it already owns.
That is `CompoundVariableTail`'s own answer with the stack buffer moved onto the caller.
**Blast radius:** `rexx-exec` (`value.rs`, `stem.rs`), both engines.
**Worth:** the two rows above, minus whatever remains; estimated, and no oracle-side comparison exists for them.
**Cost:** low, no `unsafe`, and it is adjacent to but not the same as queue candidate 3, which is the tail *split* rather than the tail *render*.
**Landable and measurable alone:** yes, and independent of every other option here.
Falsifier: a paired run moving `compound` by less than about 3%.

### D -- change the borrow shape that forces owned copies

**What it changes.** Two separable sub-changes, and they should not be run together.

**D(i): `to_number` stops returning an owned clone.**
`Body::Num { value, .. } => Ok(value.clone())` (`value.rs:282`) allocates a digit `Vec` on every numeric read of a heap number.
Either return a borrow, or compute in place, or -- after B -- accept that the clone is a fixed-size copy with no allocator call in it.
**That last point is the interaction: B largely defeats D(i).** If `Number`'s digits are inline for the common case, the clone stops calling `malloc` and D(i)'s remaining value is the copy itself.

**D(ii): a `&self` accessor for bytes.**
Verified above: only `to_text`'s `Body::Num` arm mutates.
So a `try_text(&self, value) -> Option<&[u8]>` can answer for `Body::Text`, `Body::Stem` and `Nil` with an ordinary shared borrow, returning `None` for an unrendered `Body::Num` and for a `SmallInt`, whose bytes do not exist anywhere to borrow.
Callers that get `None` fall back to today's `&mut` path.
**This is much cheaper than the "put the caches behind `RefCell`" shape both consultations proposed**, and it needs no interior mutability at all.
The `Rc<[u8]>` shape the practitioner offered as an alternative should be weighed against a constraint already recorded in this crate: `rexx-exec/src/lib.rs` notes that `Outcome` *"has to be `Send` -- which `Vec<u8>` is and an `Rc`-flavoured sink would not be"*, so `Rc` has been ruled out here once already, for a reason that will recur when ooRexx's threading model arrives.

**Blast radius.** `rexx-exec` -- the accessor in `value.rs`, and callers in `eval.rs`, `builtin/`, `stem.rs` and `ir/`.
**This is the option where the sharing rule binds**, and it binds on the interface rather than on the data: change `to_text`'s shape for both arms, never add a borrowing accessor the IR uses while `eval.rs` keeps copying.

**What it is worth, and this is the option whose measured worth has already fallen.**
Entry 4 measured `required_string`'s `to_vec` at **15.5% of `strings`**.
Entry 11 re-measured it at **about 3.5%** and struck it off the queue, because two accepted changes shrank the axis around it.
That is the largest single instance of the mechanism, and it is now small.
**So the mechanism is real and its headline instance is not worth a candidate on its own** -- which is the honest reading, and it is why D ranks below A and B despite being the mechanism the consultations found most interesting.
`Interp::concat` (`eval.rs:831`) is a second instance with an independent argument behind it: it builds `bytes`, then calls `self.text(&bytes)`, which is `text_owned(bytes.to_vec())`, copying the whole result a second time -- and `text_owned`'s own doc records a measured case where that second copy is the difference between running and aborting.

**Landable and measurable alone.** D(i) yes; D(ii) yes, but only after A, because a peek accessor written against `Vec<u8>` has to be rewritten against A's inline enum.
Falsifier: a paired run moving `strings` by less than about 3%.

### E -- D1's side byte-arena

**What it changes.** `Body::Text` becomes `{ offset: u32, len: u32 }` into a `Vec<u8>` held beside the slot vector, with compaction folded into the sweep.
D1 specifies this in full and gives the argument that makes it sound: `RexxString` is immutable, so no slot ever needs to grow.

**Blast radius.** `rexx-core` including the collector, and `rexx-exec`'s value accessors.
Larger than A's, because it changes what a sweep does.

**What it is worth.** The same allocator rows as A, plus the long strings A does not reach, plus the whole of the per-payload `free()` rather than the short-string share of it.

**What it costs.** High, and the cost is correctness surface in the collector rather than lines.
No `unsafe`.

**Landable and measurable alone.** Yes, but it is mutually exclusive with A rather than sequenced after it: both decide what a `Body::Text` holds, and doing one makes the other a rewrite rather than an increment.

**Recommendation: hold, and make it conditional on a measurement nobody has.**
It is D1's own pre-registered fix and it is not withdrawn.
A dominates it for short values at a fraction of the risk, and the fact that decides between them -- **the length distribution of the strings these benchmarks actually allocate** -- has never been measured.
If that histogram says long strings carry the allocation, E moves ahead of A.

## Recommended order

**B, then the string-length histogram, then A, then D(ii), then C's replacement.**
D(i) is folded into B's measurement rather than run separately.
E is held unless the histogram picks it over A. C is declined.

**The reasoning.**

* **B first**, because it is the most contained option here (one crate, no interpreter surface), it is aimed at the axis whose excess the profile decomposes least ambiguously (`arith`: ours 39.9% against the oracle's 8.3%), entry 11 has already written its falsifier, and it replaces a cited-but-absent `smallvec` result with a real number. It also does not move `Body`'s width, so it cannot perturb A.
* **The histogram next**, because it is not a candidate at all -- it is one instrumented run, it decides A against E, and getting it wrong means building the larger of the two twice.
* **A third**, aimed at `rexxcps` and `compound`, explicitly not at `strings`.
* **D(ii) fourth**, after A, because A changes the payload type its accessor would borrow from.
* **C's replacement last**, because it is small and it is the only one whose worth is two profile rows with no oracle-side comparison at all.

**Independent of each other:** A and B touch different crates and different variants; either can land first.
C's replacement is independent of all four.
**Ordered:** D(ii) after A; D(i) after B, or not at all.
**Mutually exclusive:** A and E, since both decide what a `Body::Text` holds -- and the length histogram is what chooses between them, so it comes before either.
**Superseded:** C by the existing `ObjRef` tag.

**What would change this order.** One measurement: decompose `rexxcps`' 40.0% allocator family into `Text`, `Number` and tail keys.
`rexxcps` is one of the two walls, B does nothing for it if its allocations are strings, and A does nothing for it if they are numbers.
Nobody has taken that decomposition, and it is the cheapest thing that would reorder this list.

## What the object model needs from this

**Phase 5 builds 32 classes on whatever value layer exists, and the reason to do this before Phase 5 rather than after is not "inline the payload".**
It is that **`Body`'s width is set by its widest variant, and Phase 5 adds variants.**

That is not a prediction; it is measured history in this repository.
`d1-decision.md`'s Phase 4 addendum records `size_of::<Body>()` at **32 bytes** at `0fa62ca8`, the last commit before `a3178cff`, and **80 bytes** at `ad7c36f0`, and identifies the cause: *"The dominant term is `Body::Stem`, added in the same commit for an unrelated value kind... A Rust enum's size is its largest variant, so every `Body` value, including one that only ever holds text, now carries the footprint `Stem` needs, whether or not the graph being collected contains a single stem."*
The same passage records that `Text`'s own payload grew from a `String`'s 24 bytes to around 40 in that commit, so part of the growth is `Text`'s own and the dominant remainder is `Stem`'s.
One cold variant, added once for a value kind most programs never use, widened every string in the language.
Phase 5 does that thirty-two more times unless a rule is in place first.

**So the thing to hand Phase 5 is a rule, and it is D1's own sentence split into its two populations.**

* **Hot variants stay inline and narrow.** `Text` and `Num` are what the benchmarks allocate. D1's "boxing makes it worse" is about exactly these, and it stands.
* **Cold variants are boxed, and a new class's instance state arrives boxed by default.** D1's own "box the large, rare variants and re-measure" is about exactly these. `Body::Instance(Vec<(String, ObjRef)>)` already has this shape; a class whose state does not fit that pattern should carry its own indirection rather than widen the enum.

**The two halves of D1's guidance were read as one sentence and are two.** The 4f plan records them as a tension; splitting them by variant temperature is what resolves it, and `d1-decision.md`'s Phase 4 addendum had already reached the same reading for `Stem` specifically.

**Boxing `Stem` on its own is not recommended as a candidate.**
It buys roughly a tenth of `Body`'s width, because `Num` becomes the next widest variant at an estimated 72 bytes -- an estimate reached independently by the practitioner consultation and by summing `Num`'s fields here, and measured by neither.
It adds a pointer chase to every stem access, and it does not unlock option A, which fits inside today's headroom without it.
It becomes worth doing when the rule above is adopted for Phase 5, as part of adopting it, rather than as an optimisation attempt.

## Two cautions this document is bound by

**Shares are not additive, and no total is given here on purpose.**
Entry 11 says its own ceiling construction *"is the loosest possible bound, the shares are not additive, and the sampled share is one sample of a distribution nobody characterised"*, and the consultations' specific challenge is that the argument copy, the payload `malloc`, the slot write and the collection the churn triggers may be **one cost with four names**.
If that is right, entry 11's 4.35x and 5.50x residuals are inflated and the real post-fix figures are better.
**No combined figure appears above, and the two rows that describe one object are deliberately left unsummed**: `drop_glue::<rexx_num::Number>` at 12.2% and `Number::clone` at 7.7% are both the digit `Vec`, and adding them is the move entry 11 forbids.
If a combined figure is ever built from these options, **what would falsify it** is this: land B, measure `arith`, and compare the delta against those two rows.
If the measured move is materially larger than either, the blocks were overlapping and entry 11's residuals are withdrawn; if materially smaller than both, the attribution was wrong and this document's ordering is wrong with it.

**A citation with no measurement behind it has bitten this project twice.**
Every figure above is quoted from `phase-4f-record.md` with the entry named, from `d1-decision.md`, or from source read at `HEAD` with the file and line given.
The one figure this document declines to pass on is the `smallvec` result, checked again here and still absent from the repository in every form.

## What could not be settled, and what would settle it

* **Whether the residual is one cost with four names.** Settled by landing B or A and comparing the measured delta against the profile's prediction. This is the experiment both consultations asked for, and it is the first thing the next candidate produces.
* **`size_of::<Body>()` and `size_of::<Number>()` at `HEAD`.** The 80-byte figure is `d1-decision.md`'s, measured at `ad7c36f0`; commits since may have moved it. One line of `std::mem::size_of`, which I was not permitted to run, and A's "costs nothing in slot width" claim rests on it.
* **The length distribution of allocated strings.** Decides A's inline-arm size, decides whether A or E is the right shape, and nobody has measured it. A histogram of the lengths reaching `Interp::text_owned` per axis settles it.
* **How `rexxcps`' 40.0% allocator family splits between `Text`, `Number` and tail keys.** Named above as the one measurement that would reorder the recommendation.
* **Whether every `to_text` caller can take the `&self` peek.** The accessor is provably available for three of five arms; whether the call sites want it is a reading of `eval.rs`, `builtin/` and `ir/` that the spike's uncommitted state made unsafe to do now.
* **Whether `Rc<[u8]>` is compatible with the eventual threading model.** `Outcome` must be `Send` and the crate has already declined `Rc` once for that reason; whether a heap value itself ever crosses a thread boundary is not established anywhere I could find.
* **Every share here predates the expression spike.** A third profiling pass, taken after expression promotion lands, is what makes the `strings` figures above usable again; the per-axis *rankings* should survive it and the *ceilings* will not. Settled by re-profiling, which the loop's own cadence requires anyway.
* **What the `Op` width budget costs elsewhere.** Entry 11 records that `assert!(size_of::<Op>() == 12)` is defended by an assertion rather than a measurement, and that it has already distorted two design choices. It is not a value-representation question and is not decided here, but it is the same kind of unmeasured constraint and belongs in the same conversation.
