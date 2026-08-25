# Scoped re-review of the final fix round — `433ac60fc..8863e2943`

Scope: the nine findings of `final-review.md` (H1, M1-M4, L1-L4), the sitting, and the prose this
round added. No task re-reviewed, no earlier finding re-derived. Read-only on the tree apart from
this file; nothing rebuilt.

**Verdict: CHANGES REQUIRED.** The *behaviour* is complete and correct — H1 is fixed everywhere it
can be reached, and nothing the oracle allows is now refused. Four sentences this round wrote are
false: two in the tree, two in the report. None of them changes what the interpreter does; two of
them are contradicted by other sentences a reader meets in the same doc block or the same report.

---

## Per-finding verdicts

| finding | verdict |
|---|---|
| **H1** the lock, as behaviour | **complete** — 620 differential cells, 2 differ, both the known `.RexxQueue~delete` |
| **H1** did the fix over-refuse? | **no** — nothing the oracle allows is refused; user classes and shadowing classes still mutate |
| **H1** sentence 1, `class_graph.rs:225` | **true** (and L4's paragraph is gone, replaced by the bypass) |
| **H1** sentence 2, `dispatch.rs:3345`-`:3348` | **FALSE — new** (see F1) |
| **H1** sentence 3, `lib.rs:3129` | **true** |
| **M1** `~publicClasses` exhibit | **addressed** — new `ARRAY` exhibit measured true, both copies corrected, no third copy |
| **M2** bench suite's emitted paragraph | **addressed** — emitted text and `Role::Offset` both corrected; ~21 ms reproduced |
| **M3** `DO OVER` order divergence | **addressed** — plan bullet added; measurement *and* mechanism reproduce exactly |
| **M4** `::ATTRIBUTE EXTERNAL` | **addressed** — probe reproduces byte for byte; recorded, not re-filed, as the brief required |
| **L1** two laziness justifications | **addressed** — "only caller" claim verified; no unreachability claim made |
| **L2** ownership sentence | **partly** — ownership clause fixed, but the rewrite kept a false clause (see F2) |
| **L3** `check_package` reachability | **addressed** — one production call site, pattern reproduces 0/0/0 |
| **L4** `class_graph.rs`'s "Nothing inside this crate reads it" | **addressed** — deleted |
| **the sitting** | **reads as claimed** — every quoted figure matches the TSV exactly; two prose slips (F3, F4) |
| **the new corpus rows** | **sound** — all three byte-identical to the oracle, both engines, three descriptors |

---

## The four false sentences

### F1 — tree, `rust/crates/rexx-exec/src/dispatch.rs:3345`-`:3348`

> A build that dropped the check would let `.Array~define(...)` and `.Alarm~inherit()` succeed at rc 0
> where the oracle raises -- a divergence a differential row sees, unlike a refusal the oracle does
> not share.

`.Alarm~inherit()` would **not** succeed at rc 0 without the check. `~inherit` with no argument
raises `88.901` at **rc 168** once the lock stops answering first — measured on an unflagged class,
`.K~inherit()` with `::class K` is rc 168 / `88.901` on the oracle and on both engines. This round's
own report records the same number (`_library_inherit` "without the fix: rc 168 (`88.901`) against
oracle 158"), and so does the review's sizing table.

The sentence is contradicted twice inside the tree, both times by prose a reader meets first:

* two paragraphs above it in the *same doc block* — "`.Array~inherit()` reports 98.985 where the
  same send to a class a `::CLASS` declared reports 88.901";
* `error.rs:696` — "`.Array~inherit()` reports this and not the 88.901 the same send to a `::class`
  of one's own reports".

The `.Array~define(...)` half is true (`.K~define('ZZZ')` is rc 0 on both sides), and the
conclusion — that a differential row sees the regression — survives either way, at rc 168 against
158 with differing stderr. What is wrong is the exhibit, and `.Alarm~inherit()` is the row this
round added. Dropping the second conjunct restores the sentence.

### F2 — tree, `rust/crates/rexx-exec/src/dispatch.rs:1044`-`:1046`

> A name `.Array`'s behaviour here does not hold is the oracle's 97.1, which is the same answer a
> `String` receiver gets for a name a mixin the prologue inherits declares.

A `String` receiver does **not** get 97.1 for such a name. The only mixin the prologue makes
`.String` inherit is `Comparable`, whose only declared method is `compareTo`, and measured:

* `say 'abc'~compareTo('abd')` — oracle rc 0 `-1`; crate rc **120**,
  `rexx-exec: method "COMPARETO" of class "String" is not implemented (Phase 5)`.
* control, an unheld name: `say 'abc'~zzznosuch()` is rc 159 / `97.1` on both sides, and
  `a~zzznosuch()` on a `~superClasses` array is rc 159 / `97.1` on both sides. So the array half of
  the sentence is right and the String half is not.

The same doc block says the true thing three sentences later — "`'abc'~compareTo('abd')` is `-1` on
the oracle and a Phase 5 refusal here" — so the paragraph now asserts both. The false clause is
inherited from the pre-round text ("the same answer a `String` receiver *already gets* for a name
the prologue donates and this crate has not"); L2's fix rewrote this sentence and kept it.

### F3 — report, `final-fix-report.md:293`-`:295`

> The pinned build's `per_pass` `instructions:u` figures are identical to the last Task 23 sitting's
> on all fourteen axis-arm cells, to six significant figures, five days apart

The fourteen cells and the six significant figures are exactly right — I re-derived every one from
`rust/bench-baselines/phase-5a-arms.tsv` and every delta is `-0.00%` or `+0.00%`. **"five days
apart" is not.** The two sittings are about **fourteen hours** apart:

* `23-attempt-2` was appended by `87cf62507`, `2026-08-25 07:46:42 +0200`;
* `23-fixround-3` by `8863e2943`, `2026-08-25 21:39:58 +0200`.

Five days is the age of the *pinned binary*: `bench-baselines/pinned/rexx-run-15a1ffa98` is dated
Aug 20 21:22. The control is as strong as claimed; the interval it spans is not. This is the fourth
claim about the world in this plan written without checking the world.

### F4 — report, `final-fix-report.md:313`-`:314`

> and `emptyloop` and `varlookup` now sit just under 1.0 on both arms

True for the **large** size, false for the small. The `pinned>head` `across_builds` ratios at
`23-fixround-3`, `instructions:u`:

| axis | arm | large | small |
|---|---|---:|---:|
| emptyloop | ir | 0.999829 | **1.015609** |
| emptyloop | tw | 0.997567 | **1.011102** |
| varlookup | ir | 0.996334 | **1.005305** |
| varlookup | tw | 0.995348 | 0.999784 |

Three of the eight cells the sentence quantifies over are above 1.0. The sentence names the arm
dimension and omits the size dimension, which is the one that decides it. The sentence beside it —
"the `across_builds` `pinned>head` ratios move the same way, -1.13% to +0.10%" — is **exactly
right**: the movement between the two sittings runs from `-1.13%` (emptyloop tw large) to `+0.10%`
(dispatchclass tw large).

---

## Two narrower claims, worth a line each rather than a change

* **`rust/crates/rexx-exec/src/lib.rs:7887`** — "The route to a class a `::CLASS` without `PUBLIC`
  declares is `~package~classes`", and the report's stronger "**The only route** the oracle offers
  to a class declared without `PUBLIC` is `~package~classes`". There is a second:
  `.Array~package~findClass('SETMIXIN')` is `The SetMixin class` at rc 0 on the oracle, and
  `c~inherit()` on what it answers is rc 158 / `98.985`. The **conclusion is unaffected** —
  `findClass` is refused here too (`rexx-exec: method "FINDCLASS" of class "Package" is not
  implemented (Phase 5)`, rc 120), so the non-public classes still have no differential witness and
  the in-crate assertion is still the whole instrument. A false premise carrying a true conclusion;
  "the route" wants to be "the routes … are `~package~classes` and `~package~findClass`".
* **`dispatch.rs:1425`'s L3 pattern** is stated only over `::METHOD ... PACKAGE`, but
  `::ATTRIBUTE ... PACKAGE` reaches `Access::Package` too, and the embedded files carry 17
  `::ATTRIBUTE` directives. Measured, `/bin/grep -acinE '^\s*::attribute[^;]*\bpackage\b'` is also
  `0` for all three files, so the conclusion holds; the recorded pattern is narrower than the claim
  it supports.

---

## What I ran

**H1, the whole reachable population.** Five class mutators — `~inherit()`, `~define('ZZZ')`,
`~delete('ZZZNOSUCH')`, `~uninherit(.Object)`, `~defineMethods(.NIL)` — over all **62** classes the
oracle's `.Array~package~publicClasses` names, one program per cell, oracle against both engines,
comparing exit status and whole stdout and stderr with `cmp`: **620 cells, 2 differing**, both the
`.RexxQueue~delete` cell the round already names (`rexx_delete_queue`, a Phase 7 entry point, never
reaches the lock). Every one of the 34 classes the two embedded files declare `PUBLIC` is covered,
and every mutator, not only the two tested at dispatch.

**The five non-public ones have no reachable mutator, on either side.** `say .SetMixin~id` is rc 159
/ `97.1` identically on all three interpreters; `say .SupplierMixin` and `say .LocalServer` are the
dotted-text fallback `.SUPPLIERMIXIN` / `.LOCALSERVER` at rc 0 identically. I swept every one of the
62 public classes' `~superClasses` on the oracle — **no non-public mixin appears in any of them**
(the distinct superclass set is 18 names, all public). `.local~class~id` is `Directory` on both. So
the in-crate assertion really is the only instrument, as claimed.

**Nothing the oracle allows is now refused.** `::class K` + `.K~inherit(.Comparable)` is rc 0 / `2`
identically; `.K~uninherit(.Comparable)` under `::class K inherit Comparable` is rc 0 identically; a
program that *shadows* a library class (`::class Alarm`, `::class Array`) still mutates its own —
rc 0 / `2` identically on both engines. The prologue's own `~inherit` clauses all ran:
`.String~superClasses` is `Object Comparable`, `.Array~superClasses` is `Object OrderedCollection`,
`.DateTime~superClasses` is `Object Comparable Orderable`, `.InputOutputStream~superClasses` is
`Object InputStream OutputStream`, `.Stream~superClasses` is `InputOutputStream` — all
byte-identical to the oracle on both engines. 220 non-mutator class-object sends across ten library
classes: 180 identical, 40 differing, and all 40 are the two pre-existing loud rc-120 refusals
`QUERYMIXINCLASS` and `DEFAULTNAME`, which behave the same on native classes.

**The blast radius is provably the five mutators.** `is_rexx_defined` has exactly one production
reader, `dispatch::rexx_defined_lock` (`dispatch.rs:3366`), reached from five call sites;
`install_class` (`lib.rs:5224`) has exactly one caller, `install_class_at`; and
`define_unregistered_class` has exactly one caller in `rexx-exec`. So every class either embedded
file declares goes through the flagging site, and nothing else reads the flag.

**The three new corpus rows** — `class_rexx_defined_library_{inherit,define,no_mutation}.rex` — are
byte-identical to the oracle on exit status, stdout and stderr, on both engines: rc 158, rc 158,
rc 0 with stdout `0`. The two `.Array` controls beside them are identical too.

**M1.** `.Array~package~publicClasses["ARRAY"]` is `The Array class`, oracle rc 0. `ARRAY` is in no
table this crate could answer from: `package_classes` / `package_public_classes` are written only by
`record_package_class` (from `install_class`) and `add_installed_class` (`~addClass`), and a
`Setup.cpp` class reaches neither. `completeSystemClass` is `memory/Setup.cpp:199`-`:206` with
`TheRexxPackage->addInstalledClass` at `:205`, as cited. No third copy of the retired sentence
survives — a collapsed-comment sweep of every `.rs` file under `crates/` for
`ORDEREDCOLLECTION|publicClasses|no class this crate registers|Nothing inside this crate reads`
returns the corrected pair and nothing stale.

**M2.** Independently, interleaved, two rounds of `rexx-time --warmup 10 --runs 50` on
`rust/bench-programs/startup.rex`: oracle min/median `4.414 / 6.859` then `4.240 / 6.841` ms; crate
`ir` `25.812 / 28.153` then `26.114 / 28.398` ms. About **21 ms**, mins and medians alike — the same
answer the review and the report got, under D2's `~50 ms` absolute delta
(`2026-07-27-rust-rewrite.md:157`, verified verbatim; the pinned `median 5.1 ms` is `:159`).
`build/lib/rexx.img` exists, so the emitted paragraph's "the oracle restores its saved image" is
true. The four remaining "no bootstrap / starts fast" strings in the repository are all inside dated
Phase 4 records, which is where they belong.

**M3.** The divergence reproduces exactly: oracle `MANGO ALPHA KIWI ZEBRA DELTA`, both engines
`ALPHA DELTA KIWI MANGO ZEBRA`, rc 0 everywhere. **The bullet's mechanism claim reproduces too**,
which is worth saying because mechanism claims in this plan usually do not: computing
`h = 31*h + byte` over each name and taking `h % 17` gives buckets MANGO 1, ALPHA 8, KIWI 8,
ZEBRA 9, DELTA 11 — bucket-ascending, chain order within bucket 8, which is the oracle's printed
order character for character. `MinimumBucketSize = 17` is `HashCollection.hpp:126` and
`h = 31 * h + stringData[i]` is `StringClass.hpp:341`. `Interp::hash_collection_indexes` exists and
is cited from `environment.rs` as well.

**M4.** `rust/corpus/gate-tables/directives/attribute__external__subkeyword.rex` on the oracle is
rc **166**, `Error 90.998:  Unable to find external method "GETzzz_no_entry".`; the crate is
`rexx-exec: ::ATTRIBUTE EXTERNAL is not implemented (Phase 7)` at rc 120 on both engines. The
`::METHOD` sibling is byte-identical to the oracle (rc 166, `"zzz_no_entry"`), which is the
mechanism the doc says Task 22 landed. `directive_gap`'s `::ATTRIBUTE` arm is one blanket refusal
covering both spellings, as the doc says. The gate row is not re-filed.

**L1.** `ObjectModel::bootstrap` has exactly one caller in the workspace,
`Interp::object_model`'s `get_or_insert_with` (`dispatch.rs:905`). `bootstrap_library` is called
unconditionally before `interp.run(program)` (`lib.rs:6710`), so `environment.rs`'s replacement
reason holds. Neither replacement makes an unreachability claim.

**L3.** One production call site for `check_package` — the `Access::Package` arm at
`dispatch.rs:1213`; the other four are in-crate tests.
`/bin/grep -acinE '^\s*::method[^;]*\bpackage\b'` is `0` for `CoreClasses.orx`,
`StreamClasses.orx` and `PlatformObjects.orx`, as recorded.

**The sitting.** Every figure the report quotes matches the TSV. The pinned control: 14 axis-arm
cells, all `-0.00%` or `+0.00%` against `23-attempt-2` (`alloc4c ir` 3594.6860 vs 3594.6867,
`strings ir` 5369.5251 vs 5369.5255, `emptyloop ir` 376.0000 vs 376.0000). The head `per_pass`
table: `alloc4c` -0.21/-0.28, `arith` -0.08/+0.11, `compound` -0.28/-0.37, `dispatchclass`
-0.18/+0.32, `emptyloop` -0.80/-1.15, `strings` -0.21/-0.24, `varlookup` -0.69/-0.63 — every cell
reproduces to the digit. `across_builds` movement `-1.13%` to `+0.10%` reproduces. `dispatchclass
tw`'s `fixed` is 89.6M against 173.2M, and `strings`/`emptyloop`/`varlookup`/`compound`/`arith` all
sit at 148-150M on both arms, as stated. The pinned binary's sha256 is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, which is what `PINNED.md:16`
records; `target/release/rexx-run`'s is `683faa64e0...`, the report's head hash, and no file under
`rust/crates` or `rust/corpus` is newer than it. `rexxcps_path` does resolve
`bench-rexxcps/rexxcps.rex` and `bench-programs/rexxcps.rex` does not exist. Concern 6 checks out
in full: the README's uniqueness command prints **480** at HEAD *and* at `433ac60fc`, and all 480
duplicates are task label `20` at `301842eaa`, `6808a1901` and `7e253fae1`, 160 each.

## One observation, not a finding

The sitting is labelled `23-fixround-3` and the three corpus rows open `/* Task 23 fix round 3: */`,
but this round's brief is `final-fix-brief.md` — the whole-branch review's, not Task 23's. There is
no `task-23-fixround-3-brief.md` for a later reader to find. The label is unique in the TSV and the
commit column pins it, so nothing is ambiguous in the data; it is the paper trail that has a gap.

## What this re-review could not see

* **Anything needing a rebuild.** The red witnesses the report records for the three corpus rows and
  the in-crate assertion are its own; I confirmed the *forward* direction on every one and the
  arithmetic of the reverse (an unflagged class's answer to each of the five mutators, measured), but
  I did not remove the fix and rebuild.
* **The gate chain.** Not re-run — the team lead verified fmt, clippy and
  `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` at HEAD, with 251 of 251
  matching and zero FAILED.
* **Instance-side behaviour.** `~new` is 5b's, so every H1 check here is a class-object mutation, as
  the review and the report both already state.
