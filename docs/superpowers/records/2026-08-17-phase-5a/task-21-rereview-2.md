# Task 21, fix round 2 re-review (scoped)

**Verdict: CHANGES REQUIRED**, and every finding is prose. The regression is closed **as a class and
not only at the one door** -- I enumerated the collection sites from the type rather than from the
report, and there is exactly one function in this workspace that frees a slot, exactly one caller of
it in `rexx-exec`, and both callers of *that* now sweep. The new corpus row witnesses what the report
claims, including the suspended half, and I confirmed the half it cannot reach by hand. What is left
is three sentences: one in the tree that is flatly false and that the class enumeration is what
found, one rule the round enforced in one file and broke in two others, and one claim the round's own
deletion left without its exhibit.

Everything below was run from a fresh empty probe directory with absolute paths, three descriptors
read separately, both engines. **I did not rebuild**, so no claim here rests on a build I made.

---

## 1. Is the rooting hole closed as a class?

**Yes, for the tree as it stands, and I checked it by enumeration rather than by reading the fix.**

**One function frees a slot.** `Heap`'s public API is `new`, `slot_capacity`,
`collections_performed`, `collect`, `alloc`, `alloc_immortal`, `set_uninit`, `clear_uninit`,
`immortal_count`, `alloc_with_uncollected`, `get`, `get_mut`, `live_count`, `will_grow`. The only
writes of `Slot::Free` are `heap.rs:264` and `:271`, both inside `Heap::collect`. So "reaches a
collection" and "calls `Heap::collect`" are the same question.

**One call of it in `rexx-exec`.** Scanning every `.rs` file under `rust/` except `target/` for
`.collect(&`, `Heap::collect` and `::collect(`: the only live call sites are
`crates/rexx-exec/src/lib.rs:6027` (inside `Interp::collect_now`), `crates/rexx-core/src/heap.rs:531`
(inside a `#[cfg(test)] mod retire_tests`), and `rexx-core`'s own `tests/` and `benches/`. Nothing
outside `rexx-core` reaches `Heap::collect` except through `Interp::collect_now`.

**Two callers of `collect_now`, and both sweep by construction** -- the sweep is inside it:

| path | reaches a collection via | sweeps? |
| --- | --- | --- |
| `Interp::alloc_with` (`lib.rs:5966`) | `collect_if_due` -> `collect_now` | yes |
| `Interp::alloc_immortal_with` (`lib.rs:6065`) | `collect_if_due` -> `collect_now` | yes |
| `builtin::state::gc` (`state.rs:232`) | `collect_now` directly | yes, this round |
| stress mode | `collect_if_due`'s `stress_collect` arm -> `collect_now` | yes |

`alloc_with` is the crate's single allocation entry point, so there is no fourth door in the
allocation direction either.

**The four holders of an `Activation` are still exactly four** (`Interp::running`,
`Interp::suspended`, `Interp::spare_activations`, `DeferredReply::activation` at
`activation.rs:925`), the sweep covers the first two, `park_reply`'s anchor covers the fourth through
`Activation::object_roots`, and `push_activation` overwrites a spare wholesale (`*spare =
activation`, `activation.rs:1442`). `object_roots` is an **exhaustive destructuring of every field**,
so a new field on `Activation` cannot be added without this function failing to compile -- that is a
real type-level control and it covers the field class.

**What is not closed is the future.** Nothing but a paragraph stops a fifth door. The paragraph is
good and says the right thing ("a new collection site is the thing to check against this
paragraph"), but this task has now spent two rounds on a defect whose whole shape was that the
mechanism hangs off the site. `crates/rexx-exec/tests/dispatch_seam.rs` already carries the exact
instrument: a `source_files()` walker over `crates/rexx-exec/src/` and an `occurrences(needle)`
helper that returns `path:line` pairs, used there to assert a function is called from the seam and
nowhere else, with the same argument in its module doc ("no program can tell that branch from its
absence and the lexical assertion is the whole instrument"). One assertion -- `occurrences("heap
.collect(")` is one entry and it is in `lib.rs` -- turns this paragraph into a check.
`rexx-core/tests/unsafe_sites.rs` is the second precedent for the shape. **Recommended, not
required.**

### Witnesses I ran

| probe | oracle | ir | tree-walker |
| --- | --- | --- | --- |
| the corpus row itself | rc 0, `1`/`outer`/`1`/`inner`/`outer` | identical | identical |
| suspended half in isolation: outer renames, inner forces the only collection | rc 0, `1`/`x`/`outer` | identical | identical |
| three nested class methods, only the deepest forces | rc 0, `1`/`d1`/`m1`/`o1` | identical | identical |
| stem of 50 tails + string + held `.context`, two forced collections around them | rc 0, `1`/`zzzzzzz`/`40`/`held`/`held`/`1`/`1`/`held` | identical | identical |
| `REPLY` then two forced collections in the main body, parked body reads its own context back | rc 0, `inner`/`1`/`1`/`main done`/`inner` | identical | identical |

The third row is the one that says the sweep walks the whole of `suspended` and not just its top.
The fifth says routing through `collect_now` did not disturb the parked mechanism -- and it came back
byte-identical on all three descriptors including line order, which matters for finding 6 below.

## 2. Did routing `GC('force')` through `collect_now` change anything else?

**No answer a program can see changed, and I checked the return value directly.** `gc()` is `0`;
`gc('force')`, `gc('Force')`, `gc('f')`, `gc('F')` and `gc('fnord')` are all `1`; `gc('x')` is rc 216
with the 40.904 message naming the four spellings. All byte-identical to the oracle on both engines,
stdout and stderr read separately. The builtin's `Ok(interp.text(b"1"))` is untouched and unreachable
from the change.

Three differences exist and none of them is observable:

* **The sweep runs.** That is the fix.
* **`collect_at` is now recomputed after a forced collection** (`COLLECT_FLOOR.max(live * 2)`).
  This changes *when future collections happen*, nothing else. It cannot change an answer, because a
  collection produces no program-visible effect: `stats.pending_uninit` is asserted empty and nothing
  in the crate sets `Object::has_uninit`, so no finalizer is owed. The direction is toward the
  natural path rather than away from it -- a forced collection now leaves the watermark exactly where
  a natural collection at the same moment would. No storm follows a lowered watermark either, because
  `will_grow()` is `free_head.is_none()` and a collection that lowers `collect_at` also fills the
  free list.
* **A root frame is pushed and popped around the collect.** `alloc_with` already did this at every
  allocation, so no invariant is new.

`builtin/state.rs:688`'s own unit test (`GC()` must not collect, `GC('Force')` must) still measures
the right thing: `Heap::collect` increments `collections` and `collect_now` still calls it.

## 3. The new corpus row

**It witnesses what the report claims, and it discriminates in both directions.**

* **A build that mints a context per evaluation fails it.** `.context~objectName` with no rename is
  `a RexxContext` -- measured on the oracle and on `ir`. The row's first output row is `outer`, so a
  per-evaluation build prints the default and diverges. The comment's claim is true.
* **A build whose sweep covered `running` only would fail it too**, at the last line rather than the
  third: `Interp::context_object` (`environment.rs:692`) answers the cached `ObjRef`
  **unconditionally** and never re-mints on a miss, so a freed cache is a `rc 120` refusal and not a
  fresh object. The inner method's `gc('force')` is taken while the outer activation is in
  `suspended`. My isolated probe (row 2 of the table above) removes the confound entirely and passes.
* **Could the inversion redden for a reason other than the hole?** No. Replacing `interp
  .collect_now()` with `interp.heap.collect(&interp.roots)` differs in exactly two ways: no sweep,
  and `collect_at` left alone. The second cannot redden this program -- `COLLECT_FLOOR` is 65,536
  slots and this program allocates three orders less, so no natural collection happens under either
  value and `collect_at` is never read to a different answer. The redness the report measured can
  only be the sweep. I did not rebuild, so this is an argument from the diff rather than a re-run of
  the control; the working tree is clean, so the report's restore-from-copy did land.
* The `sourceline_oracle` expectation is byte-identical to the program below its `count 25` line, and
  `count 25` matches `wc -l`. `class_context_identity.txt`'s `count 32` matches its own file too.
* The row is also picked up by `collect_stress.rs`, which reads **every** phase subset file and
  asserts plain-vs-stress agreement per program on all three descriptors. So the row runs a second
  time with the collector on every allocation.

## 4. The three prose fixes and concern 4

* **Historical framing struck from `unbuilt_collection_owner`.** Only that sentence went; the
  paragraph before it and the owner-ordering sentence after it are untouched and the paragraph still
  reads. `Loud::unreadable_collection` (`lib.rs:874`) does carry the measurement in a present-tense
  framing -- "measured, `.K~defineMethods(.local)` is oracle rc 163, `93.974`". Correct as asked.
* **The corpus comment now describes the plain run, and what it says is true.** `COLLECT_FLOOR` is
  65,536 (`lib.rs:345`) and the trigger also needs `will_grow()`; 300 iterations cannot reach either.
  The retained sentence is true as well: `collect_stress.rs` reads every phase subset file, and
  `phase-5a.txt` lists `class_context_identity.rex`, so that program really does run under
  collect-on-every-allocation.
* **The sixth-decimal slip is corrected and the correction is right.** Report `:747`-`:748` now reads
  ir/large `1.017380` against `1.017378`, tw/large `1.015308` against `1.015311`, tw/small `1.015515`
  against `1.015511`. All four pairs are what `bench-baselines/phase-5a-arms.tsv` holds for
  `21-fixround-2` and `21-fixround-2-inlined`.
* **Concern 4's deletion removed only what was asked.** The `=` row and its `0` are gone along with
  the `1` that was the removed row's own result; the `~objectName` row, the `a Method`
  counterfactual, the `ClassClass.cpp:984`/`:991` citations and the rooting paragraph are all
  untouched. I checked the two citations in the C++ -- `:984` is `MethodClass
  *RexxClass::method(RexxString *method_name)` and `:991` the `instanceMethodDictionary->getMethod`
  retrieval -- and I re-ran the retained measurement: oracle rc 0, prints `x`. **But see finding 3
  below**: the sentence the paragraph opens with counts two methods and now exhibits one.

## 5. The sitting

Every figure in the report's sitting section reads as written, checked against the TSV rather than
against the report: `alloc4c` 1.004808 -> 1.003705, `arith` 0.989743 -> 0.988671, `strings` 1.013408
-> 1.011944, `rexxcps` 1.020337 -> 1.019044, `compound` 1.010472 both, `emptyloop` 0.992023 both,
`varlookup` 0.994260 both, `dispatchclass` 1.017572 -> 1.017568 (four millionths, as claimed). The
absolute pair is exact too: `alloc4c` head/ir/small is 1671167082 against 1673002829. The new sitting
is `21-fixround-3` at commit `ae4681ad1`, which continues the existing convention of numbering
sittings rather than rounds (`21-fixround-2`/`-2-inlined`/`-2-committed` were all fix round 1's).

The baseline the report calls "the round before" is `21-fixround-2` and not `21-fixround-2-committed`
(which reads 1.004807 and 1.020307 on two of the four axes). That is the same baseline the committed
doc comment quotes, so it is the consistent choice and it changes no conclusion.

The report attributes nothing, which is right: the only codegen-affecting change this round is the
one call site, everything else is comments, and a tenth of a percent on the allocating axes with a
do-nothing control unrun is a layout claim nobody can make.

---

## Findings

### 1. `Outcome::collections`'s doc is false, and the class enumeration is what found it

`lib.rs:374`-`:381`:

> How many times `Heap::collect` ran during this program. **Always `0` under `run_program`**, which
> does not enable Task 16's stress mode and **nothing else in this crate calls `collect` at all** --
> a non-zero value here is **only ever possible through `run_program_collect_every_alloc`** [...]

Three clauses, all false at HEAD, and one of them is the exact claim this round's class question
turns on. `collect_policy.rs:121` asserts `outcome.collections > 0` from a plain `run_program` and
its doc names the measured counts, **6** and **7**. `gc('force')` under a plain `run_program`
collects at least once. And "nothing else in this crate calls `collect`" was already wrong before
this round -- `collect_if_due` has called it since the watermark policy landed.

This is **pre-existing rather than introduced**, and it is inside the round's scope only because the
class question is "how many doors are there" and this paragraph answers it wrongly. It is also the
paragraph a reader would consult to decide whether a new collection site is possible. One paragraph
to fix; nothing to re-measure, the numbers are in `collect_policy.rs`'s own doc.

### 2. The round struck one historical framing and added two

`activation.rs:763`-`:768`:

> `GC('Force')` **was** such a door -- **it called** `Heap::collect` directly, and a forced collection
> **freed** the running activation's own context object. Measured: `say .context~objectName` either
> side of `gc('force')` **was** oracle rc 0 twice and rc 120 here on the second.

and `state.rs:226`-`:229`:

> **Reaching past it left** `gc('force')` freeing the running activation's own `.CONTEXT` --
> measured, `say .context~objectName` either side of a forced collection **was** oracle rc 0 twice
> and rc 120 here on the second.

Apply the global constraints' own borderline test -- "strike the historical framing and see whether
the sentence still says the same thing about the code as it is". Strike both and what remains
("the rooting hangs off the collection site, not off the state [...] a collection reached by some
other door sees neither mechanism. So every collection goes through `Interp::collect_now`" and
"through `Interp::collect_now` and not `Heap::collect` directly, because the root set the collector
is handed is not `Interp::roots` alone") says exactly the same thing about the code as it is. By that
test the framing is decoration, and it is the same shape as the `environment.rs` sentence this round
was told to strike, in the same commit. The constraint also says a before state belongs in the report
and the ledger, and the report has it.

**In fairness, the tree has precedent both ways**: `Interp::alloc_with`'s doc carries "An earlier
version of this method collected *after* allocating" with its measured failure, and
`activation.rs`'s own module doc ends "An earlier version of this comment claimed no definition
existed". So this is a ruling rather than an open-and-shut violation. If the framing is kept, keep it
knowingly.

The cheap fix keeps every word of the evidence and drops the dating: "a door that called
`Heap::collect` directly frees the running activation's own context object -- measured against such a
build, `say .context~objectName` either side of `gc('force')` is oracle rc 0 both times and rc 120
here on the second." That is `alloc_with`'s accepted shape (a measured negative control on a design
that is not this one) rather than a date stamp on this code.

Separately, "**was oracle rc 0 twice**" is loose: a process has one exit status. It means the oracle
answered the name twice and exited 0.

### 3. The concern 4 deletion left its own opening sentence unsupported

`environment.rs:1008`-`:1014` now reads:

> **One object per dictionary entry, because the oracle's identity is observable through two of its
> own methods.** [...] Measured, oracle rc 0: `.K~method("M")~objectName = "x"` then `say
> .K~method("M")` prints `x`. A fresh object per send answers `a Method`.

The paragraph promises **two** methods and exhibits one. The struck row was the second one, and it
was struck because the comparison it used does not discriminate. This is the pattern where the change
that fixes a sentence falsifies the one beside it. It is also a set cardinality in a comment, which
is the same rule m3 was about. One word: "through two of its own methods" -> "through its own
`~objectName`", or restore the second exhibit as the `==` comparison that does discriminate.

### 4. Two group comments do not describe the row inserted under them

`corpus/phase-5a.txt`'s last block is headed "The Package object: `RexxContext~package` and the two
class tables", with an added sentence for `class_context_identity.rex` because that row is not about
the Package object either. `class_context_gc.rex` went in beside it with no such sentence, and
`coverage.rs:1143`'s "the Package object -- `RexxContext~package`, the two class tables, and the
refusal the REXX package gives an addition" now covers a fifth member it does not name. Not false;
incomplete, and the same insertion-under-a-doc-block shape that has bitten this plan before. One
clause each.

### 5. Recommended: make the site class a check rather than a paragraph

Section 1 above. `dispatch_seam.rs` already has the helper and the precedent.

### 6. Recommended: the parked half is still witnessed only by hand

No corpus program contains both `REPLY` and `.context` -- the five programs naming `.context` are
`class_context_gc.rex`, `class_context_identity.rex`, `class_context_package.rex`,
`class_package_classes.rex` and `environment_symbols.rex`, and none of them replies. That is the same
disjoint-sets shape that let NEW-1 through, one mechanism over: the parked route
(`park_reply` -> `RootSet::park` -> `Activation::object_roots`) has no differential row at all. It is
**writable now**: my `REPLY` probe came back byte-identical to the oracle on stdout, stderr, exit
status and line order on both engines. Not this round's debt, and not a blocker.

---

## Comment rules and citations

* **ASCII, and I checked the pattern rather than trusting it.** Zero of the 119 added non-TSV lines
  match `[\x80-\xff]` under `LC_ALL=C`, with a positive control (a UTF-8 `caf\xc3\xa9` line matches).
  So no em-dashes and no smart quotes either; the added prose uses `--`.
* **No set cardinality is added.** The round in fact removes one: "Those are the **two** states an
  activation can be in" is gone. Finding 3's "two of its own methods" is pre-existing prose the
  deletion undermined.
* **No new C++ citation is added by this round.** The two the edited paragraph retains
  (`ClassClass.cpp:984`, `:991`) land on the code they name, checked in `interpreter/`.
* **The intra-doc links resolve.** `activation.rs` has `use crate::Interp;` (`:33`), and
  `Interp::collect_now` exists at `lib.rs:6003`, so the rewritten `[`Interp::collect_now`]` is a live
  link and not the silent-warning shape the constraints name. `[`Activation::object_roots`]` is in
  the same file.
* `resume_reply`'s new sentence is accurate: between `roots.release(parked)` (`dispatch.rs:1932`) and
  `push_activation` (`:1945`) the only call is `save_clause_state`, which takes `&self` and copies
  three scalars. Nothing there allocates a Rexx object, so the window is latent exactly as stated.
* `dispatch.rs`'s rewritten dead-handle arm is accurate and no longer overclaims.

## What I did not check

* **I did not rebuild.** So the inverted control is argued from the diff, not re-run, and no claim
  above depends on a build I made. The tree is clean (`git status --porcelain` empty), which is what
  says the report's restore landed.
* I did not re-run the gates, the corpus count or the 98 test binaries; the controller's results
  stand as reported.
* I did not re-take the concern 4 measurement that the ruling rests on, as instructed.
* The negative claims above have their patterns beside them: "one function frees a slot" is from
  `Slot::Free` writes in `heap.rs`; "one call site" is from `.collect(&`, `Heap::collect` and
  `::collect(` over every `.rs` under `rust/` except `target/`; "no corpus row replies and reads
  `.context`" is from `reply` and `\.context` over `corpus/**/*.rex`.
