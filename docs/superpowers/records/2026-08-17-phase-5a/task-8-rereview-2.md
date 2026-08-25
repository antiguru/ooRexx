# Task 8, re-review 2 (fix round 2, prose only)

Base `c9523023c`, head `4729e5d3a`. One commit, four files, prose only -- confirmed independently
(`git diff -U0 c9523023c..4729e5d3a -- rust/crates | grep -E '^[+-]' | grep -v '^+++|^---' | grep
-vE '^\+\s*///|^\+\s*//|^-\s*///|^-\s*//'` is empty).

## N1 -- write order, six copies

C++ read directly, `ClassClass.cpp:1562`-`:1620`, execution order confirmed:
`:1579` `new_class = meta_class->sendMessage(NEW, class_id)` -> inside `newRexx`, `this` is
`meta_class`, so `new_class->metaClass = this` (non-primitive branch) sets `metaClass := meta_class`.
`:1586`-`:1591`: `if (isMetaClass())` -- `this` here is the **superclass** (the object `subclass()`
was called on) -- `new_class->metaClass = this` overwrites `metaClass` to the superclass. `:1613`
`createClassBehaviour` runs after that overwrite. `:1615` `setOwningClass(meta_class)` reads the
local parameter, never touched by `:1590`'s write to the `metaClass` *field*. So `:1590` runs
**before** `:1615`, confirmed.

Also independently verified the M3/S counterexample cited only in lib.rs's new prose
(`::class S mixinclass class` + `::CLASS M3 SUBCLASS S METACLASS S`) on the oracle: answers `S` to
both, matching the claim.

Per-copy verdict:
* `rust/crates/rexx-classes/src/class_graph.rs:147-151` (metaclass field doc) -- ADDRESSED, correct order.
* `rust/crates/rexx-classes/src/class_graph.rs:280-285` (define_class override doc) -- ADDRESSED, correct order.
* `rust/crates/rexx-classes/src/registry.rs:180-205` (class_of doc) -- ADDRESSED by deletion: the
  order claim is dropped entirely rather than restated (the doc now states the iff rule and points
  elsewhere for mechanism). Not false, just silent on order here -- acceptable, mechanism is stated
  correctly in class_graph.rs and lib.rs.
* `rust/crates/rexx-exec/src/lib.rs:5417-5426` -- ADDRESSED, correct order, and cites `:1579`'s
  `newRexx` role too (verified above, accurate).
* `docs/superpowers/plans/phase-4-exclusions.txt:607-608` -- ADDRESSED, correct order.
* Report FR1.1 (`task-8-report.md:659-675`) -- ADDRESSED, order stated correctly with an
  execution-order C++ excerpt that checks out line-for-line against the source.
* `8a88dc63d`'s commit message -- correctly left uncorrected (unfixable) and the report says so
  (line 673-675). Grepped the whole tree for the old backwards phrasing and for the old false
  general-rule sentence: no surviving copies anywhere outside the two acknowledged commit messages.

## N2 -- the `iff`, all five copies

**Corrected rule stated and verified.** Re-measured on the oracle from a fresh directory
(`probe1.rex`, six-row table -- `MC mixinclass Class`, `Z subclass Class`,
`M3 subclass MC metaclass MC`, `T subclass MC metaclass M1`, `T2 subclass MC`, `K metaclass M1`):
output matched every row in every copy exactly (`MC`: Class/Class same; `Z`: Class/Class same; `M3`:
MC/MC same; `T`: MC/M1 part; `T2`: MC/Class part; `K`: M1/M1 same). Walked the iff's truth table
against all six rows by hand against `RexxClass::subclass`'s actual logic (superclass-is-metaclass
test at `:1586`, named-or-inherited metaclass defaulting via `getMetaClass()` at `:1566`-`:1568`) --
holds in both directions, no row is a coincidence of the test program rather than the rule.

Per-copy verdict, all ADDRESSED, correct form (necessity-not-sufficient, "and is not the
named-or-inherited metaclass" clause present, counterexample rows kept beside it):
* `class_graph.rs:158-176` (owning_class field doc) -- `iff` stated, table of all six rows, M3/MC/Z
  named as the necessity-not-sufficient counterexamples.
* `registry.rs:185-201` -- `iff` stated with the same clause, T/T2 as parting examples, M3/MC/Z as
  coinciding, plus the "superclass not a metaclass" branch (fixing N4, see below).
* `lib.rs:5417-5447` -- `iff` stated in the test's own doc comment, full six-row table with an
  "asserted" column, plus the M3/S oracle-verified counterexample outside the committed program.
* Report FR1.1 (`task-8-report.md:630-657`) -- `iff` stated, six-row table, and explicitly names
  where the counterexample was hiding (the test's own first directive) rather than just fixing the
  words.

No copy states necessity as sufficiency, no copy drops the "and is not" clause, no copy keeps the old
shape in a subordinate sentence.

## N2b, N3-N9

* **N2b** -- ADDRESSED. `phase-4-exclusions.txt:604`-`610`: "discards ... altogether" replaced with
  "A named METACLASS is not discarded by that, though: it survives as the class's `~class`."
* **N3** -- ADDRESSED. `task-8-report.md:927`-`934`: sitting counts deleted, replaced by the span
  `[1.001283..1.001284]` and the six-commit list (`c351fa473, 56c842cb0, 6aa432f19, c35ba11cc,
  bb6d46466, 8a88dc63d`), stated as re-derived from the TSV rather than corrected from the wrong
  number. `c9523023c`'s commit message copy correctly left as unfixable.
* **N4** -- ADDRESSED. `registry.rs:198-201`: "Every class `native_classes` builds names `.Object`
  as its superclass, or is `.Object` itself and names none."
* **N5** -- ADDRESSED. `lib.rs:5449-5453`: `~request` / `ObjectClass.cpp:1916` added, citation
  verified against source (`Protected<RexxString> class_id = behaviour->getOwningClass()->getId()
  ->upper();`) -- exact line and exact mechanism.
* **N6** -- ADDRESSED. `task-8-report.md:938`-`945`: claim narrowed to the six measured axes
  (`alloc4c, arith, compound, emptyloop, strings, varlookup`), `dispatch.rex` named as the
  counterexample to the wider claim.
* **N7** -- ADDRESSED (acknowledged, not edited). `task-8-report.md:985`: both uneditable commit
  messages (`8a88dc63d`, `c9523023c`) named as carrying the errors.
* **N8** -- ADDRESSED by a structural fix, not a line-number correction: `lib.rs`'s test doc now
  cites assertion rows by label (`T~class`, `T~metaClass`) rather than by line number, and the report
  (line 712-714) explains why. Confirmed current line numbers have already drifted again (assertions
  now at `:5485`-`:5486`, not `:5467`/`:5466` or `:5469`/`:5468`), which is exactly why the label
  citation is the right fix.
* **N9** -- ADDRESSED. `lib.rs:5459`: "Either way of collapsing..." replaced with "Collapsing the
  fields back into one fails it whichever field is made to stand in for the other" -- no enumeration
  of the set size remains.

## Comment-prose pass (house method)

Collapsed each contiguous `///`/`//` comment block to one line for base and head across the three
Rust files, and each paragraph for `phase-4-exclusions.txt`; diffed the collapsed files; read every
new line in full (not filtered through a keyword list). Changed blocks: `class_graph.rs`'s
`metaclass`/`owning_class` field docs and `define_class`'s doc, `registry.rs`'s `class_of` doc,
`lib.rs`'s test doc, `phase-4-exclusions.txt`'s one paragraph -- exactly the five sites already
covered above, nothing else moved. No new "used to"/"no longer"/"previously" framing (grepped for
it across the diff, one hit -- "is not discarded" -- which is present-tense and correct, not
historical framing). No non-ASCII bytes in any added line. No new set-cardinality or enumeration
violation in the changed comments: the removed enumerations (N4, N9) were fixed, no new one
introduced.

## C++ citations, all printed and checked

`ClassClass.cpp:1562`, `:1566`-`:1568`, `:1572`, `:1579`, `:1586`-`:1591`, `:1613`, `:1615` -- read
directly, all match what the comments say. `ObjectClass.cpp:1916` -- read directly, matches.
`ClassClass.cpp:1123`-`:1127` (`createClassBehaviour`'s merge, cited unchanged from before this
round) -- spot-checked, still accurate. No wrong citation found; this is the second round running
with none.

## Verification run myself

* `cargo fmt --all --check` from `rust/` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- first run was suspiciously fast (0.07s,
  warm cache); re-ran after `touch`ing the three changed source files -- recompiled `rexx-classes`
  and `rexx-exec`, exit 0.
* Did not re-run the five gates or the corpus suite (instructed not to; report's numbers treated as
  unverified).

## Round verdict

**PASS.** Both substantive findings (N1, N2) are corrected in every one of their copies, in the
exact logical form required, and independently re-measured against the oracle rather than taken on
the implementer's word -- all six table rows in the corrected `iff` match, including the three
coinciding counterexamples that refute the previous false rule. All seven minors (N2b, N3-N9) are
addressed, several by deleting the fragile claim (counts, line numbers) rather than patching it,
which is the right move given this project's repeated history of corrections rotting. No new
set-cardinality, enumeration, historical-framing, or citation defect found in the diff. `fmt` and
`clippy` both clean on a cold recompile. Nothing left to send back.
