# Task 8, fix round 1 -- re-review

Base `6754ae2a2`, head `c9523023c`. Four commits: `121bc720d` and `8fb97527c` (doc/citation),
`8a88dc63d` (the fix), `c9523023c` (the sitting's rows).

**Round verdict: the code is right and the instrument is real; the prose is not.** Every finding is
addressed in behaviour, the field split matches the oracle on every shape I could build, and both
collapses are red at the claimed assertions in a plain `cargo test`. But the round shipped two false
statements about the oracle, each in several copies, and both are load-bearing for Task 9: the
write order inside `RexxClass::subclass` is stated backwards in six places, and "they part wherever
a class derives from a metaclass" is false -- falsified by `::class S mixinclass class`, which is
the first line of the round's own test program and a row in the round's own report table. One more
prose-only round.

---

## Finding verdicts

**1. The latent bug, fixed in the code -- ADDRESSED.**
`ClassDef::owning_class` added beside `metaclass` (`rust/crates/rexx-classes/src/class_graph.rs:169`),
bound before the override and `metaclass` after (`:282`-`:283`), read by `ClassGraph::owning_class`
(`:702`) and `ClassRegistry::class_of` (`rust/crates/rexx-classes/src/registry.rs:203`).
`cascade_build` still reads `metaclass` (`class_graph.rs:443`-`:445`), which is right:
`createClassBehaviour` reads the `metaClass` field at `ClassClass.cpp:1123`/`:1127`, and that read
happens at `:1613` -- after the `:1590` override. Only one `ClassDef` literal exists, so no
construction site can miss the new field. `class_of` has no production caller yet
(`/bin/grep -rn class_of rust/crates --include=*.rs` finds only tests), so the behaviour change is
inert today, exactly as the brief said.

**2. `docs/superpowers/plans/phase-4-exclusions.txt:604`-`:611` -- ADDRESSED, with residue.**
"`.Class` is then the metaclass whatever the directive said" is gone and the superclass is named.
Two residues, below as N1 and N2b.

**3. `rust/corpus/phase-5a.txt:254`-`:258` -- ADDRESSED.** No count, no enumeration; membership is
stated as a rule ("its send resolves to nothing under the donation rule the first program
measures"). Checked against the two programs it governs: `class_metaclass_class_method_does_not_
donate.rex` and `class_metaclass_superclass_wins.rex` are both 97.1 sends, so the rule is true of
the group it describes.

**Minors.**

* `rust/crates/rexx-exec/src/lib.rs:3733` now cites `ClassDirective.cpp:180` against `:191` and
  `:225` -- ADDRESSED and verified by printing the lines: `:180` `Error_Execution_nometaclass`,
  `:191` and `:225` `Error_Execution_noclass`. Every other C++ citation new in this diff also
  checks out: `:222`/`:230` (error.rs `class_not_found`), `:247`-`:249` (`abstract_metaclass`),
  `ClassClass.cpp:1754`-`:1761` (`makeAbstract`), `:1123`-`:1127`, `:1119`-`:1127`, `:1572`,
  `:736`/`:803`, `ClassDirective.cpp:200`/`:205`/`:230`, `StemClass.cpp:280`. **No wrong citation
  this round.**
* `ir_dual` reason -- ADDRESSED (report:549-560), with a positive control on the grep (38 files
  under `corpus/lang/`, 0 under `ir_dual_cases/`).
* Commit count -- ADDRESSED and **verified exactly**: `git log --oneline 15a1ffa98..REV --
  rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml | wc -l` gives 28 at `d0b7bd151`, 29 at
  `bb6d46466`, 32 at `8fb97527c`, matching report:47 stamp for stamp.
* `arith` bimodality -- ADDRESSED in section 9, but the FR1.8 restatement introduces a new false
  count (N3).
* Table D's `ABSTRACT` probe -- ADDRESSED (report:118-131), and it says where enforcement lands
  (`~new`, 5b's, `abscla`'s row).
* Committed witness as a superset -- ADDRESSED (report:65-68), naming
  `corpus/lang/class_metaclass.rex:27`.
* The implementer's own find -- `a_bare_class_directive_subclasses_object`'s "still `directive_gap`
  above" clause is gone (`lib.rs:5403`-`:5406`). Correct: all three keywords install.

---

## 1. The field split against the oracle

Fourteen shapes measured on the oracle, then the same shapes read out of `ClassGraph` through
`install_directives` in a copied tree. **Every one agrees.**

Oracle (`~metaClass` / `~class`), rc 0 throughout:

```text
::class M1 mixinclass class          Class   Class     isMeta 1
::class S  mixinclass class          Class   Class     isMeta 1
::class T  subclass S metaclass M1   S       M1        isMeta 1
::class T2 subclass S                S       Class     isMeta 1
::class K  metaclass M1              M1      M1        isMeta 0
::class P                            Class   Class     isMeta 0

::class MC mixinclass class          Class   Class     isMeta 1   metaclass from a metaclass
::class M2 subclass MC               MC      Class     isMeta 1     "  derived from a metaclass
::class M3 subclass MC metaclass MC  MC      MC        isMeta 1   METACLASS names the superclass
::class G1                           Class   Class     isMeta 0   three-generation chain,
::class G2 subclass G1 metaclass MC  MC      MC        isMeta 0     middle names a metaclass
::class G3 subclass G2               MC      MC        isMeta 0
::class LATER metaclass LM           LM      LM        isMeta 0   metaclass declared later
::class LM mixinclass class          Class   Class     isMeta 1
::class MX mixinclass MC metaclass M2  MC    M2        isMeta 1
::class N  subclass M2               M2      MC        isMeta 1   grandparent on the class side
::class Z  subclass Class            Class   Class     isMeta 1
::class ZM subclass Class metaclass Class  Class Class isMeta 1
.Object / .Class / .String / .Method  Class  Class
```

Our graph answers all eighteen rows identically (probe test inserted into a copy of `rust/` outside
the repository, `installed()` -> `install_directives`, printing `metaclass`, `class_of`,
`is_metaclass` for each).

**The rule, read off the C++ rather than inferred.** `~class` is
`behaviour->getOwningClass()` (`ObjectClass.cpp:1814`-`:1816`), written unconditionally at
`ClassClass.cpp:1615` with the local `meta_class` -- the named metaclass, or `getMetaClass()` of
the superclass when the directive names none (`:1566`-`:1569`). `~metaClass` is the field
(`:421`), written in `newRexx` to the metaclass (`:1806`-`:1814`) and then moved to the superclass
at `:1590` when that superclass `isMetaClass()`. `mixinClass` forwards to `subclass` (`:1518`), so
`MIXINCLASS` obeys the same rule; nothing else writes `behaviour->owningClass` for a class object,
so `INHERIT` cannot disturb it. That is exactly what `define_class` implements.

**No divergence found.** The one place worth naming as checked-and-clean: `newRexx`'s
`isPrimitiveClass()` branch (`:1806`) writes `TheClassClass` rather than `this`, which our code has
no counterpart for -- it cannot bite, because `.Class` is the only primitive class with
`META_CLASS` set (`:744`-`:747`; `Setup.cpp` sets it nowhere else), so the branch can only ever
write the value the other arm would have written.

## 2. Is the in-crate test real? YES

Copy of `rust/` outside the repository, `target` deleted, symlinks for `interpreter/`, `oodocs/`,
`ootest/`, `docs/`, `samples/`. `env -u REXX_ENGINE cargo test -p rexx-exec --lib
a_class_objects_metaclass_and_its_class_are_separate_fields`, no gate variable set:

```text
unmutated                          running 1 test ... ok          1 passed; 0 failed
owning_class() reads .metaclass    running 1 test ... FAILED      lib.rs:5469  "T~class"
override dropped in define_class   running 1 test ... FAILED      lib.rs:5468  "T~metaClass"
```

Each run printed `running 1 test`, so no filter matched nothing, and each printed
`Compiling rexx-classes` so no cache hit masqueraded as a result. Both collapses fail
**unconditionally**, at the assertions the implementer named. The test goes through
`install_directives` as claimed. `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` both exit 0 in the copy, unpiped, with
every workspace crate re-checked.

**Note for the record:** the report's own transcript cites `lib.rs:5467` and `lib.rs:5466`; the
assertions at head are `:5469` and `:5468` (N8).

## 3. The wider claim -- STILL TOO NARROW IN ONE DIRECTION AND TOO WIDE IN THE OTHER

The claim: *"the split is a property of deriving from a metaclass, and naming `METACLASS` only
chooses which value the `~class` side holds."*

* **Necessity holds.** `:1590` is the only write that can separate the two, and it fires only when
  the superclass `isMetaClass()`. Re-measured: `::CLASS T2 SUBCLASS S` with `S` a metaclass and no
  `METACLASS` keyword anywhere answers `~metaClass` `S`, `~class` `Class`. So yes, the split needs
  no keyword.
* **Sufficiency does not.** Deriving from a metaclass does *not* part them whenever the superclass
  is its own metaclass or is the metaclass named. Measured: `::class MC mixinclass class` answers
  `Class` to both; `::class Z subclass Class` answers `Class` to both; `::CLASS M3 SUBCLASS MC
  METACLASS MC` answers `MC` to both.

**Exact rule.** Write `meta` for the named metaclass, else the superclass's own `metaClass`. Then
`~class` = `meta` always; `~metaClass` = superclass when the superclass is a metaclass, else
`meta`. **They part iff the superclass is a metaclass and is not `meta`** -- in practice, deriving
from any metaclass other than `.Class`, unless `METACLASS` names that same superclass.

That is the sentence Task 9 should build on. What is committed says something weaker and false in
five places -- N2.

## 4. New false statements in the rewritten prose

House method: contiguous comment blocks collapsed to one line for base and head on all five changed
`.rs` files, `diff`ed, every new line read. Citations checked by printing the C++ lines.

### N1 (moderate) -- the oracle's write order is stated backwards, in six copies

`ClassClass.cpp:1590` (`new_class->metaClass = this`) runs **before** `:1615`
(`new_class->behaviour->setOwningClass(meta_class)`). Printed, numbered. The round says the
opposite:

* `rust/crates/rexx-classes/src/class_graph.rs:148`-`:150` -- "moves this field to the superclass
  (`:1590`) after it has already handed the metaclass it was given to `setOwningClass` (`:1615`)"
* `rust/crates/rexx-classes/src/class_graph.rs:271`-`:272` -- "the oracle has already given that
  value to `setOwningClass` (`:1615`) by the time `:1590` runs"
* `rust/crates/rexx-classes/src/registry.rs:192`-`:194` -- "moves the `metaClass` field to the
  superclass (`:1590`) after handing `setOwningClass` the metaclass it was given"
* `rust/crates/rexx-exec/src/lib.rs:5420`-`:5423` -- "hands the metaclass it was given to
  `setOwningClass` (`ClassClass.cpp:1615`), and only *then* moves the `metaClass` field"
* `docs/superpowers/plans/phase-4-exclusions.txt:609`-`:610` -- "because `setOwningClass` is handed
  the named metaclass before the field is moved"
* commit `8a88dc63d`'s message -- "hands the metaclass it was given to setOwningClass at :1615 and
  only then moves the metaClass field to the superclass at :1590". **Uneditable.**

The report repeats it too: FR1.1, "`:1590` is `new_class->metaClass = this` -- the *superclass*,
and it runs after", under a heading that says "read at the source rather than inferred".

The brief the round was working from had it right ("`:1590` ... has *already* moved the `metaClass`
field"), so this is an inversion introduced by the round, not inherited.

**Why it matters even though the code is correct.** The stated mechanism is not the real one. The
real one is that `:1590` writes a *different location*: the field `new_class->metaClass`, while the
local `meta_class` handed to `setOwningClass` at `:1615` is never assigned again. Order is
irrelevant; independence of the two locations is the reason. A reader who takes the committed
sentence and reorders the two writes in our `define_class` would conclude the binding order is
load-bearing, which is precisely the trap the next task walks into.

### N2 (moderate) -- "they part wherever a class derives from a metaclass" is false

Falsified by `::class MC mixinclass class`, `::class Z subclass Class`, and `::CLASS M3 SUBCLASS MC
METACLASS MC`, all measured above. Sites:

* `rust/crates/rexx-classes/src/registry.rs:190`-`:192` -- "They come apart **wherever** a class is
  derived from a metaclass". Unambiguous sufficiency claim, false.
* `rust/crates/rexx-classes/src/class_graph.rs:147`-`:148` -- "The two differ for a class derived
  from a metaclass".
* `rust/crates/rexx-classes/src/class_graph.rs:158`-`:159` -- "a build reading one for the other is
  wrong on **every** class derived from a metaclass". False: on `MC`, `Z` and `M3` such a build
  answers correctly.
* `rust/crates/rexx-exec/src/lib.rs:5417`-`:5418` -- "a class derived from a metaclass is where
  they part". Survives on a necessity reading; reads as sufficiency beside the three above.
* report FR1.1 -- "So **any** class derived from a metaclass parts the two." The counterexample is
  two lines above it in the report's own table: `S metaclass Class  S class Class`, where `S` is
  `::CLASS S MIXINCLASS Class`.

The same `S` and `M1` are the first two directives of the committed test's program, so the round
had the falsifying data in hand at every stage.

**N2b.** `phase-4-exclusions.txt:604` still opens "deriving from a metaclass discards a named
METACLASS **altogether**", and lines `:608`-`:610`, added this round, show the named metaclass
surviving as `~class`. The paragraph now contradicts itself. (The same "discards" framing sits in
`corpus/lang/class_metaclass_superclass_wins.rex`'s header comment -- pre-existing, not this
round's, but it is the same sentence and will read as wrong once `~class` lands.)

### N3 (minor) -- new false counts about the sitting table

The table is exact (not re-derived, per the brief). The prose about it is not:

* report FR1.8 -- "`ir small` bimodal across `[1.001283..1.001284]` -- **fourth sitting** for that
  cell". The TSV has the cell straddling both values at `c351fa473`, `56c842cb0`, `6aa432f19`,
  `c35ba11cc`, `bb6d46466` and `8a88dc63d`: **six**, and the cell has rows in eight sittings.
* commit `c9523023c` -- "for the fourth time" (same, uneditable) and "the values **three** earlier
  sittings recorded" for `arith`'s four values, which appear in **five** earlier sittings
  (`c351fa473`, `56c842cb0`, `6aa432f19`, `c35ba11cc`, `bb6d46466`).

Everything else in the FR1.8 prose checks out against the TSV: 312 rows; `cycles:u` from `0.946087`
(`alloc4c tw small`) to `1.019660` (`compound ir small`) exactly; `strings tw small` flat and "a
third independent reading" is right, because the confirmation sitting was written to a throwaway
file (report:509-512) and so is absent from the TSV; `/bin/grep -c "::" bench-programs/arith.rex`
is 0.

### N4 (minor) -- `registry.rs:188`-`:190`

"`native_classes` gives **every** class it builds `.Object` as its superclass". `.Object` itself is
built with `None` (`native_classes.rs:297`). The conclusion drawn from it ("none of them is derived
from a metaclass") still holds.

### N5 (minor) -- `lib.rs:5434`-`:5436`

"Every program that could observe the split has to send `~class`." In the oracle,
`RexxObject::requestRexx` also reads `behaviour->getOwningClass()` (`ObjectClass.cpp:1916`), so
`~request` observes it too. Harmless today -- `REQUEST` is not in our dispatch table -- but the
negative claim is wider than its evidence.

### N6 (minor) -- "no axis program installs a `::CLASS` directive at all"

report FR1.8 and commit `c9523023c`. True of the six axes every sitting on this plan runs. Not true
of the harness's own axis list: `rexx-bench`'s `PROGRAMS` includes `dispatch`, and
`bench-programs/dispatch.rex:9` is `::class counter` -- the one axis program where this round's
change would execute.

### N7 (minor) -- `8a88dc63d`'s message closes with a sitting exemption that does not hold

"No sitting. The release binary's .text is byte-identical across a comments-only change ... so no
axis can move in instructions." `8a88dc63d` adds a struct field; the report's own measurement puts
`.text` at 1205001 -> 1205177 bytes across it, and `c9523023c` exists because a sitting *was*
owed. FR1.6 records the error well and names the general shape ("a stale *reason* rather than a
stale list"), but never says the false sentence landed in a commit message, so a reader of the log
meets it uncorrected.

### N8 (minor) -- stale line numbers in the collapse transcript

report FR1.1 cites `lib.rs:5467` (`T~class`) and `lib.rs:5466` (`T~metaClass`); at head they are
`:5469` and `:5468`, which is what my re-runs printed.

### N9 (judgement, low) -- `lib.rs:5441`-`:5446`

"**Either way** of collapsing the fields back into one fails it" names the size of a set (two) and
then enumerates its members. Nothing enforces that there are only two collapses -- collapsing both
directions at once, or writing the superclass into `owning_class`, are others. Defensible as a
report of two mutations actually run; flagged because this plan has ruled on the shape before.

## 5. Things I checked that are clean

* ASCII only across the whole diff and both commit messages.
* No `unsafe` introduced.
* No historical framing in the new comments; the one historical clause in the neighbourhood was
  struck by the round itself.
* The `blame_native_method` rule rewritten in `121bc720d` -- re-measured on the oracle rather than
  trusted. `::CLASS S MIXINCLASS Class METACLASS Object` gives 99.927 with **no** `Compiled method`
  frame; `::class d inherit c` over a non-mixin `c` gives 98.942 opening `*-* Compiled method
  "INHERIT" with scope "Class".` The discriminating pair is real and the cited lines (`:200`,
  `:205`, `:230`) are the right ones.
* `cargo fmt --all --check` exit 0, `cargo clippy --workspace --all-targets -- -D warnings` exit 0,
  both unpiped, both in the copy at head sources.

## 6. Cosmetic

* `registry.rs:190`-`:192` wraps as "derived / from a / metaclass" -- a two-word line left by the
  edit.
* report FR1.1's opening table pairs directive rows with result rows that do not correspond
  (`::CLASS M1 ...` sits beside `T metaclass S`); the values are right, the layout invites a
  misread.
