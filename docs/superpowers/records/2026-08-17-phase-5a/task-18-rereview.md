# Task 18, fix round 1: re-review

Range `4761541c0..fe8d51cb0`. Scoped to what this round changed. Everything below that says
"measured" was run by me, fresh directory per batch, absolute paths, three descriptors compared
separately, both sides bounded.

**Verdict: CHANGES REQUESTED.** One load-bearing false statement in a shipped doc comment, repeated
in the report, plus two report-only number claims that its own TSV contradicts. **The work itself is
sound**: the duplicate-member check refuses exactly what the oracle refuses across every boundary I
could build, both discriminating mutations reproduce, and the eight-axis sitting is complete and its
figures come from the file it names.

---

## Defect 1 (source): the reversed duplicate/`EXTERNAL` pair is rc 120 here, not 98.903

`crates/rexx-exec/src/lib.rs:4213`-`:4218`:

> **It runs before [`directive_gap`] in the walk**, because a directive that is both a duplicate and
> an `EXTERNAL` gets the duplicate: measured, `::method m` followed by
> `::method m external "LIBRARY nosuchlib nosuchfn"` is 99.902 at rc 157, **where the reverse order is
> the `EXTERNAL`'s own 98.903 at rc 158 because the file reaches it first.**

Measured, that reversed file under `::CLASS A`:

```
oracle          rc 158   Error 98.903:  Unable to load library "nosuchlib".
crate ir / tw   rc 120   rexx-exec: ::METHOD EXTERNAL is not implemented (Phase 7)
```

The forward direction is a genuine byte-for-byte match and I confirmed it. The reverse is not: this
crate never reaches 98.903 for any `EXTERNAL` directive, because `directive_gap` refuses it as a
Phase 7 gap. The paragraph's subject is this crate's walk, the paragraph before it marks its oracle
claim explicitly (*"A translation error on the oracle"*) and the paragraph after it marks another
(*"measured, oracle rc 0"*), so an unlabelled sentence here reads as both engines -- and the sentence
is offered as the **reason** the check sits ahead of `directive_gap`. In the reverse direction it is
not evidence for that ordering at all: the crate's answer there is decided by a refusal neither
number appears in.

Report section 12.2 carries the same row under a table headed `| program | oracle and crate |`:
*"the same pair reversed | rc 158, 98.903 -- the file reaches the `EXTERNAL` first"*.

**Fix:** say that the reverse order is the oracle's 98.903 and this crate's Phase 7 refusal, or drop
the clause. The ordering claim stands on the forward direction alone, which is measured and matches.

## Defect 2 (report): the `dispatchclass` `tw` outlier description contradicts its own median

Section 12.1: *"One base run in that sample reaches 13,156,140,568 where the other four sit near
13,096,103,237."*

The TSV row it comes from, `18-fixround-1-contribution` / `dispatchclass` / `base` / `absolute` /
`tw` / `small` / `instructions:u`:

```
median 13,122,125,431   min 13,096,103,237   max 13,156,140,568   k=5
```

With five samples the median is the third sorted value, so **three of the five base runs are at or
above 13,122,125,431**, about 26M above the min -- not one. The `changed` side is the mirror image:
median 13,096,076,134 with max 13,122,006,485, so at least one of its runs is in the high mode too.
This axis is **bimodal** at roughly 0.2%, both builds sample both modes, and which mode carries the
median is what the 0.998019 cell is reporting.

**The conclusion is unaffected and is if anything understated.** `base>changed` is changed/base, so
0.998019 says the changed build used *fewer* instructions on the noisy arm; the other three
`dispatchclass` cells are 0.999987/0.999997/0.999997 with tight ranges, and `rexxcps` `tw` is
1.000004. The flat reading does rest on the absolutes and on the seven quiet axes, as the report
says. What is wrong is the sentence describing the sample, and "per-run ranges overlapping" -- which
is true, base min 13,096,103,237 against changed max 13,122,006,485 -- is the part that carries the
argument.

**Fix:** call it a bimodal axis both builds sample, not a single high run.

## Defect 3 (report): "to five decimals on every one of them" is four decimals on two cells

Section 12.1: *"My figures reproduce the reviewer's to five decimals on every one of them."* Against
the six cells I measured independently last round:

```
cell                      mine       theirs     agree to
dispatchclass ir small    1.018376   1.018375   5 dp
dispatchclass ir large    1.018414   1.018412   5 dp
dispatchclass tw small    1.016303   1.016293   4 dp
dispatchclass tw large    1.016322   1.016325   5 dp
rexxcps       ir small    1.020306   1.020307   5 dp
rexxcps       tw small    1.016592   1.016600   4 dp
```

The two `tw` cells agree to four. They are also the two arms defect 2 shows are bimodal, so four is
the honest figure rather than a discrepancy to explain. Separately, the two sittings did not measure
the same binary: mine was the head at `4761541c0`, theirs the head at `d7020ed9f`, which carries the
new check -- so this is agreement across a code change the axes cannot see, which is a stronger
result than "reproduce" and should be stated as what it is.

**Fix:** four decimals, and name the two builds.

---

## What I verified and found sound

### The check's edges: 32 shapes, 30 byte-identical on both engines

Two batches, oracle against `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, three descriptors
compared separately. Every one of these matched exactly, exit status included:

**Must stay rc 0, and do:** `::METHOD m` beside `::METHOD m CLASS` in one class; `::ATTRIBUTE p GET`
beside `::ATTRIBUTE p SET`; `::ATTRIBUTE p` in class `A` and again in class `B`; `::ATTRIBUTE p CLASS`
beside `::ATTRIBUTE p`; an unattached `::METHOD m` above a `::CLASS` that also declares `M`;
`::METHOD m` beside `::ROUTINE m`.

**Refusals, each the oracle's own code and clause:** `::METHOD "p="` then `::ATTRIBUTE p` is the
attribute's 99.931, and reversed it is the method's 99.902 -- the setter key nothing spells, in both
directions; `::CONSTANT c` beside an instance-side `::METHOD c` is 99.902 and reversed is 99.932;
`::CONSTANT c` beside `::ATTRIBUTE c` is 99.931; `::METHOD m ATTRIBUTE` beside `::ATTRIBUTE m` is
99.931 and beside `::METHOD m` is 99.902; `::ATTRIBUTE p` beside `::ATTRIBUTE p GET` is 99.931;
`::ATTRIBUTE p SET` beside `::METHOD "p="` is 99.902; `::METHOD m DELEGATE p ATTRIBUTE` beside
`::METHOD "m="` is 99.902; `::METHOD m ABSTRACT` beside `::METHOD m` is 99.902; `::METHOD m` beside
`::METHOD M` is 99.902; unattached `::METHOD m` twice is 99.902 and unattached `::ATTRIBUTE p` twice
is 99.931; `::METHOD m` then `::METHOD m CLASS` with no `::CLASS` is 99.905, and `::ATTRIBUTE p CLASS`
alone is 99.905 with the message naming `::METHOD`.

**Orderings:** `::constant c 5` then `::constant c (1+2)` with no `::CLASS` is 99.932, and
`::constant sep (1+2)` twice is 99.906 on the **first** -- both as reported; a duplicate `::CONSTANT`
pair ahead of a `::CLASS` naming an unresolvable superclass is 99.932 and not the class error; a
duplicate `::ROUTINE` still takes 99.903.

**The key sets are the C++'s own, checked rather than assumed.** `methodDirective` claims
`internalname` (`parser/DirectiveParser.cpp:822`), plus the setter under `DELEGATE`+`ATTRIBUTE`
(`:841`) and under `ATTRIBUTE` alone (`:855`) -- exactly `method_dictionary_keys`' four arms.
`attributeDirective` claims both names for `ATTRIBUTE_BOTH` (`:1664`, `:1667`), the getter name for
`ATTRIBUTE_GET` (`:1725`) and the setter name for `ATTRIBUTE_SET` (`:1790`) -- exactly
`attribute_dictionary_keys`' three arms. `constantDirective` claims the instance side unconditionally
(`:1926`) and the class side only under `activeClass != OREF_NULL` (`:1929`), which is what the
`constant && class_side` skip reproduces. `checkDuplicateMethod` itself is `:507`-`:530`, its
`classMethod` refusal `:512`-`:515` and its unattached table `:518`, all exact; the per-side dictionary
test is `ClassDirective.cpp:434`-`:444`, exact.

**The one place the crate's key order differs from the C++ is unobservable.** `methodDirective`
checks `internalname` before the setter; `method_dictionary_keys`' `DELEGATE`+`ATTRIBUTE` arm returns
the setter first. Both keys carry the same side and the same error code on the same directive, so no
descriptor can tell them apart -- and the order is the one `install_method` already had.

### The two discriminating mutations, run by me

Applied to the committed tree, corpus gate read, tree restored from a byte-identical copy. `sha256`
of `crates/rexx-exec/src/lib.rs` before and after both:
`5297769b8247c2bfc942145e30c560e56cfa86556820b2c15f6589064c324b35`, `git status --porcelain` empty,
`target/release/rexx-run` rebuilt from the restored source afterwards.

* **A member's keys forget which side** (`Method` and `Attribute` mapped to `false`, `Constant` left
  alone): **200 of 202**, mismatching set exactly `class_member_class_keyword_needs_class.rex` and
  `class_member_names_per_side.rex`. That is the pair the report names, and it is what says the
  negative control is doing work.
* **A `::CONSTANT` claims only one side**: **201 of 202**, mismatching set exactly
  `class_duplicate_constant_and_method.rex`. Again as reported.

### The sitting

`phase-5a-arms.tsv` now carries `18-fixround-1` and `18-fixround-1-contribution`, **each over all
eight axes**, both at `d7020ed9f`. The original six-axis `18` rows are left in place, which is right.
Every figure in section 12.1's two tables is the TSV's own value, checked cell by cell, and every
absolute quoted in the decomposition paragraph is in the file at the value quoted. Nothing in the
contribution arm is at or above 1%; the loudest is the bimodal `dispatchclass` `tw` `small` cell at
0.998019, in the safe direction.

### The three corrected claims

All three are now right, and all three citations re-read:

* `record_literal_constants` cites `:1911` for the literal arm (`value = token->value();`) and
  `:1875` for the value-omitted arm (`value = name;`), with the `isEndOfClause` test at `:1873`
  choosing between them. All three exact.
* `install_constant` now names `setUnguarded` (`:2523`) and `setConstant` (`:2525`) and rests the
  claim on `MethodClass::isSpecial()` (`classes/MethodClass.hpp:118`), which reads
  `PROTECTED_FLAG, PRIVATE_FLAG, PACKAGE_FLAG` and neither of those two. Exact.
* `ClassGraph::refresh_parent_has_uninit`'s doc no longer names a caller that does not exist, says
  plainly that nothing witnesses the call, and gives the reason it is kept.

### Constraints

No `unsafe` and no `forbid` claim among the added lines; zero non-ASCII bytes in the added lines
under `rust/`; no historical framing; no comment names the size of a set (`"which of the three"` is
an anaphor for the codes the same doc enumerates two lines above).

---

## Found in passing, not this round's: duplicate `::CLASS` is a silent wrong answer

```
say 'prolog'
::CLASS A
::METHOD m
  return 1
::CLASS A
::METHOD m
  return 2
```

Oracle rc 157, `Error 99.901: Duplicate ::CLASS directive instruction.` This crate runs it at rc 0 on
both engines, printing `prolog`. **Pre-existing**: the binary I built from `dd83eecdc` for the last
round answers rc 0 too, and `install_directives`' `declared` map has always taken first-wins with no
refusal.

It is the same family as the divergence this round fixed and sits one subcode below it, it is the
worst failure mode this project names, and no task in the plan owns it -- the same disposition as the
previous round's finding 5. Worth a ledger line and an owner, not a change to this round.

## Gates, run by me at `fe8d51cb0`, tree clean

```
cargo fmt --all --check                                              FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                CLIPPY_EXIT=0, zero warning/error lines
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   EXIT=0
```

98 `test result: ok`, zero `FAILED`, zero `panicked`, **202 of 202 matching**.

## State the tree is left in

`git status --porcelain` empty at `fe8d51cb0ae6c900185ac92ed8334f892df9a2a0`.
`crates/rexx-exec/src/lib.rs` restored from a byte-identical copy after each of the two mutations
(`sha256` verified both times) and `target/release/rexx-run` rebuilt from the restored source.
`bench-baselines/phase-5a-arms.tsv` was not written to. Probe files live in the scratchpad.
