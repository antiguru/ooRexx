# Task 21, fix round 3 re-review (scoped)

**Verdict: CHANGES REQUIRED.** Every item was done and the two instruments are real, but this round
added **three sentences that are false of the tree as it stands**, one in each of the three files
whose prose it rewrote. That is the shape this project has measured across four earlier rounds, and
all three are the same failure: a correct argument shipped with an exhibit or a quantifier that does
not survive being run.

Scope was the brief's items 1-4 plus recommendations 5 and 6. I did not re-review the task. I did
**not rebuild**; every probe below ran the committed release binary and the oracle, from a fresh
empty directory, absolute paths, three descriptors read separately, both engines.

---

## The false statements

### F1. `environment.rs:1021`-`:1022`: the `=` / `==` exhibit does not reproduce

> -- measured, `-140404878001713` and `-140404878167489` are `1` under `=` and `0` under `==`.

Typed as written, **both operators answer `1`**. Unary minus is an arithmetic operation, so each
literal is evaluated at `NUMERIC DIGITS 9` and becomes the string `-1.40404878E+14`; `==` then
compares two identical strings. Measured on the oracle, rc 0:

| probe | `=` | `==` |
| --- | --- | --- |
| `say (-140404878001713 = -140404878167489) (-140404878001713 == -140404878167489)` | `1` | **`1`** |
| `a = "-140404878001713"; b = "-140404878167489"; say (a = b) (a == b)` | `1` | `0` |
| `c = .K~method("M")~identityHash; d = .K~method("N")~identityHash; say (c = d) (c == d)` | `1` | `0` |

`say -140404878001713` prints `-1.40404878E+14`, which is the whole mechanism.

The paragraph's **argument is right** -- the answers arrive as strings from `~identityHash`, and as
strings they are `1` under `=` and `0` under `==`, which is rows 2 and 3. It is the exhibit that is
wrong, and it is the half a reader would check. `corpus/lang/class_context_package.rex`'s comment
makes the identical argument with no literals and is unaffected; that is the shape to copy.

### F2. `dispatch_seam.rs:185`-`:188`: "no gate here can see it" is false of its own exhibit

> A collection reached by any other route therefore frees a live object, and it does so **where no
> gate here can see it** -- a build whose `GC('Force')` reaches `Heap::collect` directly refuses the
> next send to `.CONTEXT` at rc 120 against the oracle's rc 0 [...]

`corpus/lang/class_context_gc.rex` -- added by this same task one round earlier -- is a gate that
sees exactly that build. `corpus.rs` runs every corpus program against the live oracle on three
descriptors under `REXX_CORPUS_GATE`, fix round 2 measured that row red under precisely this door,
and re-review 2 confirmed the redness can only be the sweep. This round's own
`class_context_reply.rex` is a second such row. The claim survives only for a door **no corpus
program reaches**, which is what the sentence should say.

This is the falsified-neighbour pattern with the arrow reversed: the neighbour that was added to
close the hole falsifies the new sentence written beside it.

### F3. `lib.rs:376`: "Non-zero under an ordinary `run_program`" replaced a universal with a universal

The three false clauses are gone and the replacement's supporting facts all check out (below). But
the bolded lead is a blanket claim, and the counter is `0` for almost every program under
`run_program`. Two statements in the tree say so, one of them thirty lines above in the same file:

* `lib.rs:337`-`:339` (`COLLECT_FLOOR`): "a program that allocates a few hundred values -- which is
  most of the corpus -- never collects at all".
* `collect_policy.rs`'s module doc: "every corpus program is far too small to reach the growth
  allowance, so under `run_program` they collect zero times".

"Always `0`" was false; "Non-zero" is false in the other direction. One word fixes it ("**Can be
non-zero**"). While there: the edit left a stranded `What` at the end of `lib.rs:379`, the tell of a
hand edit without a rewrap.

---

## Item by item

### 1. `Outcome::collections` -- fixed apart from F3

Everything the new paragraph rests on is true and I checked each against the code rather than the
report: `collect_policy.rs`'s doc names the measured **6**; its first test asserts `collections > 0`
**and** `collections <= 12`, so "a bound [...] in both directions" is accurate; `collect_if_due`
(`lib.rs:5981`) calls `collect_now` on the watermark; `alloc_with` (`lib.rs:5972`) calls
`collect_if_due` before every allocation and `collect_if_due`'s first disjunct is `self
.stress_collect`, so "under it every allocation collects" holds. Only the lead sentence is wrong.

### 2. Historical framing -- struck correctly, and no new dating in `src/`

Both sentences now read as a measured negative control on a design that is not this one, with every
word of the evidence kept, and "rc 0 twice" is gone. I verified the oracle half by running it:

| probe | oracle | ir | tree-walker |
| --- | --- | --- | --- |
| `say .context~objectName` / `say gc('force')` / `say .context~objectName` | rc 0, `a RexxContext`/`1`/`a RexxContext` | identical | identical |

So "answers `a RexxContext` twice at rc 0 on the oracle" is measured, not taken on reading. The
"refuses the second send at rc 120 here" half is a counterfactual build; I did not rebuild, so that
half stands on the round 1/2 record.

`state.rs`'s "which a bare `Heap::collect` leaves where it was" is true: `collect_now` recomputes
`self.collect_at` (`lib.rs:6050`) and `rexx-core`'s `Heap::collect` knows nothing of that field.

**Nothing in `src/` gained dated framing.** Scanning every added line for `was|were|used to|earlier|
previously|no longer|had|until|since|before` leaves three hits, all outside `src/`:

* `dispatch_seam.rs:187` "that was found by hand rather than by a test" -- that file's module doc
  already carries "the previous version of this comment claimed more", so it is within the file's
  own precedent. It is attached to F2's clause, which is the actual problem.
* `class_context_reply.rex` and `phase-5a.txt`: "it had no differential row: no corpus program
  contained both REPLY and `.context`" / "the sets were disjoint: no program held both". Apply the
  item-2 borderline test and these do **not** survive it -- strike the past tense and both become
  false, because the two rows now hold both. Corpus headers carry round stamps by convention here,
  so I raise it for consistency rather than as a violation.

Both past-tense claims are **true of their time**: over the whole corpus, the only programs holding
both `gc(` and `.context` are `class_context_gc.rex` and `class_context_reply.rex`, and the only one
holding both `reply` and `.context` is `class_context_reply.rex`. (Pattern: case-insensitive `gc(`,
`reply`, `\.context` over `corpus/**/*.rex`.)

### 3. Concern 4's paragraph -- cardinality gone, exhibits match, exhibit F1 wrong

"through two of its own methods" is now "observable", with no cardinality anywhere in the paragraph,
and it exhibits exactly two things, both measured:

| probe | oracle | ir | tree-walker |
| --- | --- | --- | --- |
| `say (.K~method("M")~identityHash == .K~method("M")~identityHash)` | rc 0, `1` | `1` | `1` |
| `.K~method("M")~objectName = "x"` then `say .K~method("M")` | rc 0, `x` | `x` | `x` |
| `say .K~method("M")` with no rename | `a Method` | `a Method` | `a Method` |

"A fresh object per send answers `0` and `a Method`" holds on this crate too and not only on the
oracle: `native_identity_hash` answers the handle, and two distinct method objects read `0` and `16`
here, so two fresh objects would compare `0`. "Address-derived" is right in the C++ --
`ObjectClass.hpp:340`, `identityHash() { return ((uintptr_t)this) ^ UINTPTR_MAX; }` -- and the
answers are 15 digits wide. The one defect is F1.

The sibling cardinality in `run/tests.rs:9015` ("two of the oracle's own methods") is **not** drift:
that test exhibits both.

### 4. The two group comments -- they cover the block, with two blemishes

Both enumerations now account for all six rows: `class_context_package.rex` (the package object),
`class_package_classes.rex` (the class tables), `class_package_addition_refused.rex` (the refusal)
and the three context rows. No other row in either block has drifted, and the retained "No row
prints `~name` for the running package" still holds -- `class_context_package.rex`'s `say
.Array~package~name` is the REXX package's name, not the running program's.

* **`phase-5a.txt` opens with a set cardinality**: "**Three rows here** are the context object's own
  identity". That is the shape item 3 was fixing, in a comment added by the same round, and a fourth
  context row falsifies it. Name the set, not its size. In the same sentence, "one per state the
  object has to survive in" does not map onto what follows: row 1 is not a state, and row 2 covers
  two of them.
* **`coverage.rs:1144` names the weaker half.** It says `class_context_gc.rex` is "surviving a
  forced collection **while running**", where `phase-5a.txt`'s clause from the same commit says
  "while its activation is running **or suspended**". The row's own header calls the suspended half
  "the half the single-activation rows cannot reach", so the coverage comment describes the part
  that is not the point. Not false; one clause short.

### 5. The new lexical assertion -- real, but it constrains a spelling, not a class

**What it actually pins**: the literal text `heap.collect(&` on a non-whole-line-comment line under
`crates/rexx-exec/src/`, exactly once and in `lib.rs`; plus that same text somewhere between
`    fn collect_now(&mut self) {` and the first `\n    }\n` after it. Both hold today on real code
(`lib.rs:6033`). The supporting claim is true and I checked it: the only writes of `Slot::Free` in
the workspace are `heap.rs:264` and `:271`, both inside `Heap::collect` (`alloc` begins after its
closing brace), so "reaches a collection" and "calls `Heap::collect`" do name one set.

**The two inversions are the right ones for the two assertions** -- a second site tests the count, a
site moved out of `collect_now` tests the inside-the-function half, and a file-level check alone
would have passed the second. But both mutations reuse the existing spelling, so neither can
discover that the needle is narrower than the class:

* `pub fn collect(&mut self, roots: &RootSet)` -- a second site written `self.heap.collect(roots)`
  where `roots` is already a `&RootSet` has **no `&`** and does not match.
* UFCS: `Heap::collect(&mut self.heap, &self.roots)` does not match (`Heap::` against `heap.`).
* A rebinding: `let h = &mut self.heap; h.collect(&self.roots);` does not match.
* Any site rustfmt wraps across lines does not match, because `occurrences` scans line by line.

Each of those leaves the count at one, in `lib.rs`, inside `collect_now` -- green, with the sweep
bypassed. Two more the scan cannot see: a collection introduced **inside `rexx-core`** (an
auto-collect in `Heap::alloc`, say), because the walker reads `rexx-exec/src` only; and a
macro-generated call spelled otherwise. A call from **another crate** is closed in practice --
`Interp::heap` is a private field (`lib.rs:2644`) -- so that one is not a live bypass.

The failure directions are otherwise safe: a `pub(crate)` on the signature panics, a reformat of the
existing call reddens the count, and a stray mention on a code line reddens rather than passes. One
contrived hole: assertion 2 searches raw text, so a whole-line comment inside `collect_now`
containing the needle satisfies it while assertion 1's single site sits elsewhere in `lib.rs`.

**What to change**: this file's module doc enumerates what its scan cannot see ("What that leaves it
unable to see, stated rather than argued away"), and the new test's doc has no such list while
claiming the needle names the whole set. Add the list, or widen the needle to `.collect(` and accept
the extra matches. The scan's own vacuity control already exists
(`the_scan_can_tell_a_present_token_from_an_absent_one`, over the same walker).

### 6. The new corpus row -- correct, deterministic, and it exercises what it claims

* `count 30` matches `wc -l`, and the sourceline oracle file below its count line is byte-identical
  to the program.
* One full three-descriptor comparison: oracle rc 0, stdout `replied`/`1`/`1`/`main done`, empty
  stderr; **both engines byte-identical on all three**. Ten further oracle runs hash to one value.
* **The route is real and the timing is right.** `park_reply` (`dispatch.rs:1791`) calls
  `activation.object_roots(&mut anchor)`, whose exhaustive destructuring emits `context_object`
  first (`activation.rs:1275`-`:1302`), and hands the anchor to `RootSet::park`.
  `run_deferred_replies` is called from `lib.rs:6409` **after the main program has finished**, so
  both `gc('force')` calls are taken while the activation is off every stack. The row reaches the
  parked route and the running/suspended rows cannot.
* "a collected object refuses the send outright" -- mechanically supported and not merely plausible:
  the sweep bumps each freed slot's generation and `Heap::resolve` (`heap.rs:445`) requires a
  generation match, so a stale handle stays unresolvable even after the `copies` loop reuses the
  slot. Not re-run; I did not rebuild.
* "a re-minted one answers the default and takes the LOST branch" -- `.context~objectName` with no
  rename is `a RexxContext` on the oracle and on both engines (measured above), so the branch fires.
* "The oracle runs a replied-to body on another thread, so anything it says races" -- **reproduced.**
  My reconstruction of the loud variant (parked body says its context unconditionally) gave **three**
  distinct stdout hashes over twenty oracle runs, 12/6/2. The race is real and the design decision
  is right. The comment records "two distinct outputs"; a count of distinct outputs over twenty
  samples is not a stable number to write down, and mine did not match it.

### The sitting

Checked against `bench-baselines/phase-5a-arms.tsv`, not against the report. The new sitting is
`21-fixround-4-comments` at `3a57b920c` (sittings continue to be numbered independently of rounds).
Its `across_builds` ir/small cells: `alloc4c` 1.003705, `arith` 0.988671, `strings` 1.011944 --
identical to `21-fixround-3`'s to six decimals -- and `rexxcps` 1.019048 against 1.019044. Every
figure in the report's paragraph is what the file holds. Measuring rather than asserting was the
right call for a comment-only round under `debug = true`.

### Comment rules

* **ASCII**: zero added lines outside the TSV match `[^ -~]` under `LC_ALL=C`, with a positive
  control (`caf\xc3\xa9` matches the same pattern). So no em-dashes and no smart quotes; the added
  prose uses `--`.
* **Set cardinalities**: one added, "Three rows here" (item 4).
* **Historical framing**: none in `src/`; three clauses outside it (item 2).
* The tree is clean (`git status --porcelain` empty), `dispatch_seam.rs` holds six `#[test]`s, and
  the round's mutations left nothing behind.

## What I did not check

* **I did not rebuild.** The rc-120 counterfactuals in `activation.rs`, `state.rs`,
  `dispatch_seam.rs` and the corpus row, and the two inversions of the new assertion, rest on the
  report and on reading the diff -- not on a build I made.
* I did not re-run the five gates, the corpus count or the 98 test binaries; the controller's
  results stand.
* The C++ citations `ClassClass.cpp:984`/`:991` are unchanged by this round and were checked in
  re-review 2; I re-checked only `ObjectClass.hpp:340`, which F1's sentence newly leans on.
* Negative claims above carry their patterns: the collection-site enumeration is `\.collect\(&`,
  `Heap::collect` and `heap\.collect` over `crates/` and `benches/`; the `Slot::Free` enumeration is
  that token over `crates/`; the corpus co-occurrence claims are `gc(`, `reply` and `\.context`,
  case-insensitive, over `corpus/**/*.rex`.
