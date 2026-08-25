# Task 24 review -- the 5a gate

**Spec compliance: CHANGES REQUIRED.** The brief's first "Done when" clause is not met, and I
reproduced that independently. It is not meetable by editing these four commits, and the report says
so; the remedy is that 5a stays open and the flip stays out.

**Task quality: CHANGES REQUIRED.** One major finding, three minor. The central conclusion --
"5a is not closed, on exactly these five rows" -- is correct and I could not find a sixth.

**Findings: 1 major, 3 minor, 3 observations.**

---

## 1. Is the set of five complete? Yes, and I re-derived it without taking a number from the report

I did not read the tallies out of the report or its quoted output. Three independent derivations,
all at `059b38124` on a clean tree:

**(a) The report-mode run, tallied by me.** `cargo test --release -p rexx-exec --test gate_table_c
--test gate_table_d --no-fail-fast`, exit 0. Extracting every row line from the two reports and
histogramming `(verdict, phase)` myself:

| table | phase | rows | not `agree` |
|---|---|---:|---:|
| C | 5a | 135 | **4** |
| C | 5b | 6 | 6 |
| C | 5c | 1347 | 1213 |
| D | 5a | 36 | **1** |
| D | 5b | 2 | 0 |
| D | 5c | 38 | 35 |
| D | 7 | 1 | 1 |
| D | `deferred-parse-error-rendering` | 2 | 2 |

Table C's verdict totals cross-check: `agree` 265 = 131 + 0 + 134; `diverge-both` 725 = 4 + 5 + 716;
`diverge-stdout` 1 = `obdes`; `unanswered` 497 = 5c. Table D: 79 rows, `agree` 40 = 35 + 2 + 3.
The report's section 3 tables are right in every cell.

**(b) The gating arm, run.** `verdict_is_gated` reads `REXX_PHASE_GATE` at run time, so the flip's
effect is observable without touching the tree or rebuilding:

```
$ REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a cargo test --release -p rexx-exec \
      --test gate_table_c --test gate_table_d --no-fail-fast
rc=101
gated by this run: 4 row(s) ...
gated by this run: 1 row(s) ...
4 row(s) of gate table C ... ["gate-tables/concepts/usingcl.rex", "gate-tables/concepts/xscope.rex",
  "gate-tables/concepts/methna.rex", "gate-tables/classes/rexxinfo.rex"]
1 row(s) of gate table D ... ["gate-tables/directives/attribute__external__subkeyword.rex"]
```

Exit 101, and exactly the five rows the report names. **The set of five is complete and correct.**

**(c) The structural surface, read rather than assumed.** `gate_tables/mod.rs:344` is the only
`CLOSED_PHASES` definition and `gate_table_c.rs` / `gate_table_d.rs` are its only consumers, so
nothing else in the workspace can hide a gated row. `assert_no_structural_failures` runs before any
verdict assertion in both tables, `run_on_both_engines` asserts engine agreement and
`chunks_refused == 0` unconditionally, and the `ArgUtil` assertion pushes a `Structural` -- so none
of those can be relaxed by a mode. All four are as the report describes them.

## 2. Phase attribution

**Table C's bulk assignments are correct.** `WIRING_PHASE = "5a"` covers all 63 class and all 57 edge
rows; `METHOD_PHASE = "5c"` covers all 1347 method rows, which is the spec's split. The 15 5a concept
rows and 6 5b concept rows each carry a per-row authority string, printed in the report; the six 5b
ones are `objcla`, `abscla`, `usesem`, `creo`, `obdes`, `methodsbyclass`, five of which stop at
`~new` and one at `UNINIT` -- both of which the brief's own handover puts in 5b. No misfiling.

**The `::ATTRIBUTE EXTERNAL` reading is correct.** `gate_table_d.rs:236`-`:247`, catch-all at `:244`:
`("::ANNOTATE" | "::ATTRIBUTE" | "::CLASS" | "::METHOD", _) => Some("5a")`. The arm's doc gives an
authority for each of the four exceptions above it -- `::OPTIONS`/`::RESOURCE`/`::REQUIRES`/`::ROUTINE`
to 5c, `::ROUTINE EXTERNAL` to 7, `DELEGATE` to 5b, the two cross-reference rows to the deferral --
and never mentions `::ATTRIBUTE EXTERNAL`, while `lib.rs:1527` refuses it naming **Phase 7**. So the
row set files it 5a and the code files it 7. The report's "filed 5a by omission" is the right reading,
and its decision not to re-file it (which would close a gate row by narrowing the gate) is right.

Its supporting measurement also holds: `/bin/grep -n "INTERNAL_METHOD(GET" NativeMethods.h` and the
`SET` form both match nothing, case-insensitively too, so no `::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX x'`
can resolve on either side and the whole reachable behaviour is one 90.998. Confirmed by running
`::method p attribute external 'LIBRARY REXX Filespec'`: oracle 90.998 rc 166 naming `GETFilespec`.

**The two `deferred-parse-error-rendering` rows are correctly not-5a.** Run here:
`::class k class` is 25.901 on both sides, `::resource r library` is 25.926 on both sides; only the
rendering and the exit status differ. `phase-4-exclusions.txt:3975`ff records that as a decided
deferral, with the number/sub-number gated in both directions by `rexx-parse/tests/errors.rs`.

**One filing the report did not audit: see finding m3.**

## 3. The decision not to commit the flip -- sound

Committing it would put five standing reds under G4 and G5, so the task could not have reported the
five gate commands passing either; and `global-constraints.md`'s "any red is a regression ... rather
than matching it against a name it was told to expect" would become false for every later task. Both
options fail the brief's clause 1; only one leaves the tree usable. The five rows are mechanism-sized
work belonging to Tasks 7, 9, 21, 22 and a standing deferral, none of which is this task's.

**The handover is sufficient to perform the flip later.** `ae7d291f0` puts in the plan's Task 24
section: the exact line (`pub const CLOSED_PHASES: &[&str] = &["5a"];`), its file
(`rust/crates/rexx-exec/tests/gate_tables/mod.rs`), the five rows with what each stops at and the task
each belongs to, that `REXX_PHASE_GATE=5a` reddens the identical set, and the cost if the call is
wrong. Line 344 is not given, but the constant name is unique in the workspace.

## 4. The negative control -- it discriminates, and I confirmed the split from the oracle

I could not rebuild, so I derived the predicted split instead. Asking the oracle for
`~superClasses~makeString('L', ' ')` of all 62 class rows and applying `.take(1)` on paper:

* **17 classes have more than one superclass**, so 17 of the 63 class wiring rows redden on the
  `superclasses` line. The report says 17.
* **17 of the 57 edges have their documented parent at a position other than the first**, so 17 edge
  rows redden and **40 stay green**. The report says 17 and 40.
* The two sets coincide exactly: there is no edge whose child has multiple superclasses and whose
  documented parent is already first (so "an edge whose parent is already first is untouched" holds),
  and none whose parent is not first but which has a single superclass.
* The 17 the report names -- 8 `<- MapCollection`, 3 `<- OrderedCollection`, 4 `<- Comparable`,
  `InputOutputStream <- InputStream`, `Message <- MessageNotification` -- are **exactly** my 17.

4 already-red + 17 class + 17 edge + 2 concept = **40**, matching the report's `+36` delta. The
control's arithmetic is right in every cell.

**The report's own claim about what a nameable-set control witnesses is correct.** A control that
reddens everything is consistent with "the arm fires when the binary is broken"; this one is
consistent only with "the arm reads *this row's* verdict", because it leaves 1448 of 1488 rows
unchanged and the 40 it moves are predictable from the mutation without running it -- which is what I
just did. Its stated limit is also honest: it moves one mechanism, so it says nothing about a concept
or directive row, and what covers those is the flip run's own five rows in three families.

`edge_probe_text`'s change is an instrument change and not a narrowing: `check_documented_edge` still
requires the **oracle's** `documented-edge` answer to be `"1"` and files a `Structural` otherwise, so
a row cannot pass by both sides answering `0`. Verified by running
`corpus/gate-tables/hierarchy/array__orderedcollection.rex` against the oracle and both engines: rc 0,
byte-identical on all three descriptors. `~hasItem` is still refused today
(`method "HASITEM" of class "Array"`, rc 120), and `==` on a class object is still refused with the
message the doc quotes -- so both halves of the justification stand.

## 5. Arithmetic and citations checked

Everything below I re-ran or re-read. Correct unless a finding says otherwise.

* Corpus: 31/12/12/**51** union **106** at `15a1ffa98`; 31/12/12/**193** union **248** at HEAD.
  `phase-5a.txt` gained 142.
* Section 2's before/after: 135 rows both sides, 61 = 4 + 57 before, 4 after; 74 = 12 + 62 agree
  before, 131 = 12 + 62 + 57 after. Self-consistent.
* Loud-row breakdown: the 31 constructs sum to **1222**, and 1222 - 32 = **1190** name a `~new`
  refusal. Both figures right.
* Section 8's ten `git log --oneline 15a1ffa98..HEAD -- <path>` counts: 2, 0, 0, 3, 40, 2, 52, 0, 0,
  15. All ten reproduce. `owners.rs`'s three commits are exactly `ebe91e39b`, `071835c9c`,
  `f4b21eadb`. `bif-exempt.txt` 76 = 51 + 22 + 3; `keyword-exempt.txt` 8 = 2 + 3 + 3 with no Phase 5
  row; `builtin-status.txt` 81 = 66 + 15; `PREFIX_COVERAGE` 19 = 15 + 2 + `+++` + `>N>`.
  **`assertions.rs`'s EXEMPT figures are wrong -- finding m1.**
* Section 7: one `unsafe {` block (`bytes.rs:184`), one `#[allow(unsafe_code)]`
  (`rexx-core/src/lib.rs:23`), no `deny`/`forbid` in any crate root, `unsafe_code = "deny"` at
  `Cargo.toml:31`, ten crates all carrying `[lints] workspace = true`. All ten checked one at a time.
* Section 12's pin checks: `git diff --stat 571275f0b..059b38124 -- 'rust/crates/*/src' ...` empty;
  `merge-base --is-ancestor` true; sha256 `141c3fa9...` matches `PINNED.md`; the foreign-commit list
  is **124**, of which **88** appear in `progress.md` by SHA and **36** do not, and all 36 are dated
  2026-08-22..2026-08-25 with this plan's own task subjects. Every one of the 30 per-axis ratios in
  section 12's table matches `phase-5a-arms.tsv`'s `23-attempt-2` rows to four decimals.
* Section 14: 28.657/7.437 = 3.853; reversed 28.102/7.254 = 3.874; 39.662/17.983 = 2.206;
  148,922,066 - 587,955 = 148.33M; 148,922,066/9,124,478 = 16.32; 587,955/9,124,478 = 0.0644.
  `perf-baseline.md`'s section heading is at `:1337` and the subsection at `:1388`, and the 14.380 ms
  and the "Not comparable, and not a pass" quote are both verbatim. **One fraction is wrong --
  finding m2.**
* C++ citations, all read: `Setup.cpp:740` (`AddMethod("HasItem", ArrayClass::hasItemRexx, 1);`),
  `:1285` (`EndSpecialClassDefinition(RexxInfo);`), `:396`-`:397` (the macro, expanding to
  `addToSystem(#name, currentClass);`), `:1737` (`addToEnvironment("REXXINFO", info);` over
  `RexxInfo *info = new RexxInfo;`), `NativeMethods.h`. All correct.
* Rust and row-set citations: `eval.rs:3458`, `dispatch.rs:3116`, `lib.rs:1510`-`:1511` and `:1527`,
  `gate_table_d.rs:236`-`:247` with the catch-all at `:244`, `class-methods.txt:486`,
  `provide.xml:838`, `utilityclasses.xml:7934`, `2026-08-17-phase-5-object-model.md:328` and `:1245`
  (D48, which does name `2026-07-27-rust-rewrite.md:453` as its amendment target), roadmap `:453`.
  All correct.
* `class-set.txt` is 63 rows, 62 `class` + 1 `instance`; `hierarchy-edges.txt` is 57. Three deferrals
  stand in `native_classes.rs` -- `RexxInteger`, `NumberString`, `RexxInfo` -- and the first two match
  nothing in `class-set.txt`, so the report's "two outside the criterion, one inside" is right.
* D56: `svn info` answers r13198 / r13198 / r13178; `extract_docs` is 8 tests and all 8 pass, from the
  binary hash the report quotes.

## 6. Exhibits in the four commits' prose, and in the report's boundary -- all run

Every one reproduced exactly as stated, from a fresh empty directory, three descriptors:

| exhibit | result |
|---|---|
| `::requires "rxregexp.cls"` | oracle 43.901 rc 213 |
| `.Array~superClasses~hasItem(.Object)` | crate rc 120, `method "HASITEM" of class "Array"`, both engines |
| `if c == .OrderedCollection` over `~superClasses` | crate rc 120, `the operator ==  applied to a class object`, both engines |
| `/bin/grep -acE "^::[Cc][Ll][Aa][Ss][Ss]" CoreClasses.orx` | **32** |
| `.environment` entries answering `~isA(.Class)` | **classes 62 / entries 69** |
| plain `::CLASS` with `::METHOD uninit CLASS` | oracle rc 0 `main`/`uninit ran`; crate rc 0 `main`; **silent** |
| `SUBCLASS` of one carrying it | oracle two `uninit ran`; crate none; **silent** |
| `INHERIT` of a mixin carrying it | oracle two `uninit ran`; crate none; **silent** -- the plan's prediction that this one would still be loud is correctly withdrawn |
| `USE STRICT ARG` on a user class method | oracle 93.901 rc 163, crate 40.3 rc 216, **and the pinned `rexx-run-15a1ffa98` answers 40.3 rc 216 too** |
| `Package~findRoutine` | oracle `The NIL object` rc 0; crate rc 120 |
| `.Array~package~local~class~id` | oracle `Directory` rc 0; crate rc 120 |
| `.Array~package~name` | `REXX` rc 0 on both |
| `.routines["R"]~class~id` and `.routines~r~class~id` | `Routine` twice, rc 0, both sides |
| `do i over .local` | oracle `local-entries 10` rc 0; crate rc 120, OVER-target refusal |

Comment rules: no non-ASCII byte and no em- or en-dash anywhere in the four commits' added lines; the
new `edge_probe_text` doc states no set size; the historical framing in the roadmap and plan is in
documents rather than source, which is where `global-constraints.md` puts it.

---

## Findings

### M1 (major). The control build the report says is still needed was already run, at this source, into the file section 12 reads

Section 12: "Separating the two needs the control build, and section 14 says what it would take and
what this task did not do." Section 14: "Separating a fixed cost from a per-pass one on that axis
**still needs the control build** -- a head build with the bootstrap disabled, measured on the axes --
which this task did not run".

That control exists. `bench-baselines/phase-5a-arms.tsv`'s **last 156 rows** are task
`23-attempt-2-control-no-bootstrap` at commit `cfffc7899` -- the *same* commit as the `23-attempt-2`
rows section 12 quotes -- on `strings`, `alloc4c` and `emptyloop`, both arms, both sizes, both
instruments. And `git diff --stat cfffc7899..HEAD -- 'rust/crates/*/src' 'rust/Cargo.toml'
'rust/Cargo.lock'` is empty, so it is a control build of *this* tree's source.

It already separates the two. `instructions:u`, ir arm, per pass:

| | pin | head, bootstrap suppressed | head as shipped |
|---|---:|---:|---:|
| `strings` | 5,369.53 | 5,406.48 | 7,136.86 |

So the `strings` axis's movement is **per-pass and arrives with the resident library**, not the
bootstrap's fixed 148M -- which is also arithmetically obvious, since 34% of 16.1e9 instructions
cannot be a 148M fixed cost. Task 23's own report says this in terms ("It is the change, by design,
and it is not a fixed cost. Task 24's cold-start framing covers the 148 million; it does not cover
this"), three lines below the +31.35% figure section 12 cites.

Three consequences, all of which need fixing:

1. The negative claim is false. "Still needs the control build" is a statement about the world, and
   the world already contains it, committed, in the file the section is reading.
2. "The last sitting recorded in `bench-baselines/phase-5a-arms.tsv` is task `23-attempt-2`" is wrong
   about the file. The last rows are the control's.
3. Task 23's Concern 1 -- "the per-pass regression is the one that needs a decision, and it is not
   mine to take ... it is a question about the collector's cost model with a large resident set" --
   was handed forward explicitly and does not appear in section 16's Concerns or in section 10's
   boundary. A phase gate is where an open, measured, undecided cost should be carried to 5b and 5c.
   Instead it is carried as an unresolved measurement question, which is a weaker and wrong version
   of something already answered.

None of this touches the gate verdict. It is a false statement in the section the brief specifically
asked for, resting on a negative that one `awk` over the cited file would have falsified.

### m1 (minor). Section 8's EXEMPT row counts are each one too many

"**36 rows -> 14.** 22 retired ... counted by `/bin/grep -ac 'ExemptRow {'`". That grep also matches
`struct ExemptRow {` at `assertions.rs:313`. The array holds **13** rows now and **35** at
`15a1ffa98` (`/bin/grep -ac '^    ExemptRow {'`). The derived "22 retired" survives because the
off-by-one cancels; the two absolute figures do not. Every remaining row does carry `"Phase 5"` --
13 of 13.

### m2 (minor). "Under an eighth" is the wrong fraction, and the true one is in the same sentence

Section 14: "the pin, `rexx-run-15a1ffa98`, starts in **0.970 ms**, under an eighth of the oracle's
median (7.437 / 0.970 = 7.7)". An eighth of 7.437 ms is 0.9296 ms, and 0.970 > 0.9296, so the pin is
*over* an eighth -- it is under a seventh. The parenthetical gives the correct ratio. Worth naming
because commit `059b38124` -- this task's own last commit -- exists to fix exactly this shape of error
one paragraph away ("32 of 62 is not a third").

### m3 (minor). The catch-all audit runs in one direction only, and the other direction hides a measured divergence

The report checks the row filed *into* 5a by omission. It does not check the row filed *out* of it.
`owning_phase`'s `("::ROUTINE", "EXTERNAL") => Some("7")` covers both spellings, because a row's
identity is (directive, keyword, position) and the probe picks the shared-library form
(`'LIBRARY zzznolib zzzr'`). Measured here:

```
::routine r external 'LIBRARY REXX Filespec'
  oracle: rc 168, prints `main`, compiles routine "R", raises 88.901 on my deliberate bad call
  crate:  rc 120, `rexx-exec: ::ROUTINE EXTERNAL is not implemented (Phase 7)`, both engines
```

That is the *same* `LIBRARY REXX` spelling D37 moved into 5a for `::METHOD`, and it is the argument
the report itself uses to keep `::ATTRIBUTE EXTERNAL` in 5a ("the probe names `LIBRARY REXX`, which is
present, and that is the exact spelling D37 moved into 5a"). Applied symmetrically it says this row is
at least arguable too -- and unlike `::ATTRIBUTE EXTERNAL`, no row of either table can see it.

`gate_table_d.rs`'s own doc records this in full, including the measurement, so nothing is concealed
and Phase 7 does own the row. But the brief asked this task to hand 5b and 5c a stated boundary, and
neither section 3 ("The Phase 7 row is `::ROUTINE EXTERNAL`"), nor section 10's lists, nor section 16
carries it. It should be named in the boundary as a divergence no phase gate row currently sees.

### Observations (no change required)

* Two quoted terminal blocks are abridged without an ellipsis. Section 1's `REXX_PHASE_GATE=5a` block
  shows `rc=101` and the two `gated by this run:` lines; the run also prints the two assertion
  messages, as the `CLOSED_PHASES` block below it shows and as my own run reproduced. Section 5's
  `extract_docs` block shows 3 of the 8 `test ... ok` lines above a summary reading `8 passed`. Both
  are genuine output, both trimmed to the lines the section is about.
* `5b4302f59`'s commit message carries the same wrong fraction its successor fixed in the document
  ("satisfiable with a third of the environment missing"; 30 of 62 is about half). History, not
  actionable.
* Cargo relinked `rexx-exec`'s test binaries on my first invocation despite a clean tree at HEAD.
  Nothing in the tree changed and the outputs are deterministic from HEAD's source, but the run took
  a fresh compile rather than reusing the team lead's.

## What these checks could not see

* **The five-row set** rests on the phase strings the row sets and `owning_phase` carry. I checked
  every filing that could hide a non-`agree` row (table C's 6 5b rows, table D's 7 and deferral rows,
  and the two bulk assignments) and found one arguable case, m3. What I did not check is the mirror
  image the report also declines: a row filed 5a that *passes* for a reason other than its own
  mechanism. Each such row is a three-descriptor differential against a live oracle, so it would need
  coincidental agreement on all three.
* **The negative control** I verified by prediction from the oracle's superclass lists, not by
  rebuilding. If `.take(1)` had a side effect beyond `~superClasses` -- on `~superClass`,
  `~baseClass` or `~isA` -- my 17/17 would understate what reddens and the report's would too. The
  report's own `xmixin` and `chi` rows suggest it does reach `~baseClass`, and those two are counted.
  Had the report's split been wrong, my computation would have produced a different set of names, and
  it produced the identical 17.
* **The guard's per-axis standing** I checked against the TSV, not by taking a sitting. A sitting is
  correctly not owed: no `src/`, `Cargo.toml` or `Cargo.lock` byte changed in the range.
