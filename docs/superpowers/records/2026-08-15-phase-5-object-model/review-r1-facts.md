# R1 -- factual falsification of the Phase 5 spec

Reviewer R1. Lens: every claim of fact in
`docs/superpowers/specs/2026-08-15-phase-5-object-model.md`.

Every oracle transcript below was taken in this session through the wrapper the ground rules
mandate, from fresh empty directories under the session scratchpad, with absolute paths, reading
stdout, stderr and exit status separately. Our side is
`rust/target/release/rexx-run` as staged (`b029abe77`'s code).

Findings are ordered by severity. Each carries CONFIRMED (I ran something that would have come out
differently had the finding been wrong) or PLAUSIBLE (I reasoned from documents I read).

---

## HIGH

### H1. Exit criterion 1 cannot be met inside this phase as the phase is scoped. CONFIRMED.

**Spec's words**, exit criterion 1: *"`CoreClasses.orx`, `StreamClasses.orx` and
`PlatformObjects.orx` all run to completion, on both engines."* And the directives section:
*"`::CLASS`, `::METHOD` and `::ATTRIBUTE` are this phase's, in that order"*, with `::REQUIRES`
the only construct named as landing late.

**What I ran.** A `::` census over `StreamClasses.orx` from the language's own directive marker
rather than from a line-start regex:

```
/bin/grep -aon "::[A-Za-z]*" interpreter/RexxClasses/StreamClasses.orx | sed 's/.*:://' | sort | uniq -c
      5 ATTRIBUTE
      7 CLASS
      2 constant
     33 method
    106 METHOD
```

and `/bin/grep -an "::" ... | /bin/grep -av "^[0-9]*:[[:space:]]*::"` returned nothing, so none of
those is inside a comment or a string.

**What it answers.** `StreamClasses.orx` needs two things the spec's directive list does not
contain:

* **`::CONSTANT`**, at `StreamClasses.orx:548` and `:549`:
  `::constant separator (.File~getSeparator)` and `::constant pathSeparator
  (.File~getPathSeparator)`. Both evaluate an expression at directive-install time.
* **`::METHOD ... EXTERNAL 'LIBRARY REXX ...'`**, from `:127` through `:552`, including
  `stream_init`, `stream_open`, `stream_charin`, `file_separator`, `file_path_separator`,
  `rexx_push_queue` and the rest.

`::METHOD EXTERNAL` is **already a declared Phase 7 refusal in this crate**
(`rexx-exec/src/lib.rs:1020`, `gap("::METHOD EXTERNAL", "Phase 7")`). Isolated probe:

```
::class Foo
::method getSeparator private class external "LIBRARY REXX file_separator"
```
with a `say 'main'` body -- oracle rc 0 stdout `main`; ours rc 120,
stderr `rexx-exec: ::METHOD EXTERNAL is not implemented (Phase 7)`.

So criterion 1 as written requires Phase 7 work. The spec never says so, and its `.File`/L2
paragraph says the opposite (see H10). This is the single most important finding: criterion 1 is
the phase's headline, and it is not reachable by the phase's own scope.

**Not a finding, and worth recording because the spec leaves it open:**
`interpreter/platform/unix/PlatformObjects.orx` is one line, `-- Nothing to do currently`, and
already runs at rc 0 on our interpreter today. The spec's open question *"Whether
`PlatformObjects.orx` is in scope ... nothing has read it yet"* is answered: as far as content
goes, there is nothing in it.

### H2. REPLY and GUARD outside a method are checked at **run time**, not translation time. CONFIRMED.

**Spec's words:** *"Re-measured this session: outside a method the oracle answers at
**translation** time and exits 157"*, and **D32**: *"`REPLY` and `GUARD` get their
**translation-time legality check only**."*

**What I ran.** The spec's own two transcripts reproduce exactly -- `reply` alone gives rc 157 with
`Error 99.919`, `guard on` alone gives rc 157 with `Error 99.911`. Then three programs that
separate translation from execution, all on the oracle:

| program | rc | stdout |
|---|---:|---|
| `if 1=0 then reply` / `say "done"` | 0 | `done` |
| `if 1=0 then guard on` / `say "done"` | 0 | `done` |
| `signal skip` / `reply` / `skip:` / `say "done"` | 0 | `done` |
| `say "hi"` / `exit` / `reply` | 0 | `hi` |
| `say "hi"` / `exit` / `::routine foo` / ` reply` | 0 | `hi` |

A translation-time check refuses all five. The oracle runs all five.

**The mechanism, read directly.** `reportException(Error_Translation_reply)` sits inside
`RexxInstructionReply::execute` (`interpreter/instructions/ReplyInstruction.cpp:72`), guarded by
`if (!context->inMethod())`, after `context->traceInstruction(this)`.
`reportException(Error_Translation_guard_guard)` sits inside `RexxInstructionGuard::execute`
(`interpreter/instructions/GuardInstruction.cpp:123`) in the same shape. The word "Translation" is
in the message *name*, not in when it fires -- and the spec read the name as the timing.

**Consequence.** D32 implemented as written over-refuses every program with an unreachable `REPLY`
or `GUARD`, which is the exact defect class the spec spends its directives section deleting. The
correct statement is: *outside a method, both raise 99.919 / 99.911 when the instruction is
executed*, and that is what is testable here.

### H3. The spec edits D24's text to remove the half D28 reverses. CONFIRMED.

**Spec's words:** *"**D24 already settled the shape and this phase does not revisit it**: dispatch
is `resolve` and `invoke`, two operations, not one fused `send`. The IR's `Send` op calls the pair;
the tree-walker and `eval.rs` call the same pair."*

**D24 itself** (`docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md:495`): *"Dispatch is
implemented once as **`resolve` and `invoke`**, not one fused `send`. **The IR's `Send` op caches
the resolution** and calls the invocation; the tree-walker and `eval.rs` call the same pair
uncached."*

The spec drops "caches the resolution" and drops "uncached", producing a sentence in which D24 says
nothing about caching -- and then asserts *"this phase does not revisit it"* immediately before
**D28** states *"Message resolution is **dynamic, with no per-call-site cache**"*, which is a
reversal of the clause it removed. D28 may well be right; the misquote hides that it is an
amendment rather than an application, and a reader checking D24 will find the spec's version of its
words rather than its words.

The spec's paraphrase of D24's *later* amendment (`2026-08-09-phase-4e-ir.md:1082`) is faithful,
near-verbatim, and is sound.

Also unaddressed: `2026-08-09-phase-4e-ir.md:1141` records D24's forward constraints for Phase 5 as
"selectors interned at compile time, a `SmallInt` behaviour arm, a receiver in the calling
convention, a wider patch entry, and invalidation on behaviour mutation". D28 disposes of the last
two by declining the cache. The first three are Phase 5's by D24's own text and the spec is silent
on all three.

### H4. The four-answer wiring assertion is blind to the wiring hazard the spec names. CONFIRMED.

**Spec's words:** *"**Wiring is asserted against the oracle, not against the file.** `~class`,
`~superClass`, `~isA` and `~metaClass` on every class in the native set, byte for byte against the
oracle. Those four answers are the observable shadow of the wiring"*, offered as the response to the
risk row *"the discovered native set is wired wrong"*.

**What I ran**, on the oracle:

```
say .DateTime~superClass~id                              -> Object
say .DateTime~superClasses~makestring("LINE", ",")       -> The Object class,The Comparable class,The Orderable class
say .Array~superClass~id                                 -> Object
say .Array~superClasses~makestring("LINE", ",")          -> The Object class,The OrderedCollection class
```

`~superClass` answers `Object` for `.DateTime` -- the mixin edges the spec itself cites two
paragraphs later (`::CLASS 'DateTime' public inherit Comparable Orderable`) are **invisible to it**.
`.Array`'s `OrderedCollection` edge, installed by `CoreClasses.orx`'s prologue, is likewise
invisible. A build that wired no mixin at all passes all four assertions on both classes.

`~superClasses` (plural) is the answer that sees them, and it is not in the spec's set. The
mitigation, taken as written, cannot fail on the case it exists for.

For completeness, the rest of the four are takeable and the metaclass self-reference is real:
`.class~class~id` is `Class`, `.class~class == .class` is `1`, `.Array~metaClass~id` is `Class`,
`.Array~isA(.class)` is `1`, `.Object~superClass` is `The NIL object`.

### H5. `CoreClasses.orx` has an executable prologue that *is* object-model wiring, and the spec never mentions it. CONFIRMED.

**Spec's words:** *"`CoreClasses.orx` does not bootstrap an object model; it runs on top of one that
already exists"*, *"the file's content is not the object model"*, and *"Its file-scope `::METHOD`
block ... exists to be attached to primitive classes that the native layer has already created"*.
On that basis it dismisses the survey's Q1(b) as *"a category error"*.

**What I read.** `CoreClasses.orx:39` through `:126` is a main program body, not directives. It:

* takes `use arg rexxPackage` (`:39`);
* calls `rexxPackage~addClass` for `LOCALSERVER`, `SUPPLIERMIXIN`, `ManyItemMixin`, `SetMixin`,
  `BagMixin` (`:47`-`:52`);
* sets `.environment~objectname` and `rexxPackage~objectname` (`:55`-`:56`);
* enumerates `.context~package~publicClasses` and does `.environment~put` plus
  `rexxPackage~addPublicClass` for each (`:61`-`:67`);
* installs sixteen class methods on `.String` via `.String~defineClassMethod(name~upper,
  .methods[...])` (`:70`-`:74`);
* does `.supplier~inheritInstanceMethods(.SupplierMixin)`, and the same for `.relation`, `.bag`,
  `.set` (`:80`-`:87`) -- the file's own comment calls this *"a phony inherit to add those directly
  to the class instance dictionary"*;
* does **real** `~inherit` on the primitives: `.string~inherit(.Comparable)`,
  `.array`/`.list`/`.queue~inherit(.OrderedCollection)`,
  `.identityTable`/`.table`/`.stringTable`/`.directory`/`.relation`/`.set`/`.bag`/`.stem~inherit(.MapCollection)`,
  `.set`/`.bag~inherit(.SetCollection)`, `.message~inherit(.MessageNotification)` and
  `~inherit(.AlarmNotification)` (`:93`-`:119`);
* then `call 'StreamClasses.orx' rexxPackage` (`:122`), `call 'PlatformObjects.orx' rexxPackage`
  (`:124`), `exit` (`:126`).

Those `~inherit` calls mutate the primitive classes' superclass lists and fire the
`updateSubClasses` cascade the spec's own dispatch section describes. Saying the file "runs on top
of" the object model is true of the directives and false of the prologue: the prologue *is* part of
the model's construction, and it is the part `rexx-classes` has to expose an API for
(`~inherit`, `~inheritInstanceMethods`, `~defineClassMethod`, `~addClass`, `~addPublicClass`,
`.methods`, `.context~package~publicClasses`, `~objectname=`).

This also removes the ground under the Q1(b) dismissal. Q1(b) is "4,193 lines of behaviour hand-
ported" (survey `:545`); it is a bad trade for the reasons the survey gives, but it is not a
category error, because the file's content includes model wiring rather than only sitting on top of
it.

### H6. The stated crate interface cannot express `FORWARD CLASS(SUPER)`, which `CoreClasses.orx` uses throughout. CONFIRMED.

**Spec's words**, the interface table: *"resolve | given a receiver's behaviour and an uppercased
message name, which method body"*, plus *"reaching past it is a spec amendment, not a refactor"*,
and *"a send is one hash lookup in the receiver's behaviour"*.

**What I ran and read.** `RexxObject::messageSend` has two overloads in
`interpreter/classes/ObjectClass.cpp`. The ordinary one (`:866`) is
`behaviour->methodLookup(msgname)` -- one hash lookup, exactly as the spec says. The
**scope-override** one (`:919`), used by `FORWARD` and by `SUPER`, is
`superMethod(msgname, startscope)`, reaching `RexxBehaviour::superMethod`
(`interpreter/behaviour/RexxBehaviour.cpp:584`) and `MethodDictionary::findSuperMethod`. The
dictionary is not name-to-method alone: `MethodDictionary` (`interpreter/behaviour/MethodDictionary.hpp`)
carries `ArrayClass *scopeList; // the list of scope value order use for lookups` and
`IdentityTable *scopeOrders`, plus `StringTable *instanceMethods` for per-object methods.

`CoreClasses.orx` uses the construct that needs the start scope heavily --
`/bin/grep -ain "~super\|^[[:space:]]*forward" CoreClasses.orx` returns `forward class(super)`
and `forward class (super)` at `:1779`, `:1804`, `:1823`, `:1836`, `:1886`, `:1915`, `:2168`,
`:2180`, `:2512`, `:3234` among others, plus `forward to (...)` and `forward message ...`.

So the "flattened dictionary, one hash lookup" model is right for the ordinary send and incomplete
as a specification: the flattened dictionary must additionally retain scope ordering, and `resolve`
must take an optional start scope and an optional per-object method table. The spec makes the
interface a contract whose breach is "a spec amendment", so the omission is load-bearing.

### H7. `::OPTIONS`'s effect is not `digits()`, `form()` and `fuzz()`. CONFIRMED.

**Spec's words:** *"`::OPTIONS`'s measured effect is `digits()`, `form()` and `fuzz()` -- package
settings, with no connection to the object model"*.

**What I read.** `LanguageParser::optionsDirective`
(`interpreter/parser/DirectiveParser.cpp:948` onward) switches on the subdirective and accepts, each
with its own `// ::OPTIONS X` comment in the source: `DIGITS`, `FORM`, `FUZZ`, `TRACE`, `NOVALUE`,
`ERROR`, `FAILURE`, `LOSTDIGITS`, `NOSTRING`, `NOTREADY`, `ALL`, `NOPROLOG`, `PROLOG`, `NUMERIC`.
Its own opening comment reads *"except for (NO)PROLOG all options are of a keyword/value pattern"*.

`phase-4-exclusions.txt`, which the spec leans on here, names the `TRACE` route itself twice:
`:246` (*"`::options trace labels` in the package ... both lines fire"*) and `:329`-`:330`
(*"THE ::OPTIONS TRACE LABELS ROUTE IS NOT IMPLEMENTED"*). So does this crate's own code:
`rexx-exec/src/lib.rs:1036` reads *"`::options trace labels` makes every `::ROUTINE` in the file
emit its own `>I>`/`<I<` pair"*.

The exclusions file's sentence is *"its whole effect is to change package settings,
unconditionally"* with `digits` as the worked example. The spec converted one example into an
enumeration of the whole effect. D31's conclusion (this is a package-settings unit, unrelated to
the object model) is not damaged by this; the sentence supporting it is false, and `NOPROLOG` in
particular is an execution-order option rather than a numeric setting.

### H8. Criterion 2 requires retiring a refusal D31 removes from the phase. CONFIRMED.

**Spec's words**, criterion 2: *"It contains at minimum: ... and the four `directive_gap`
over-refusals this phase retires."* And **D31**: *"`::OPTIONS` and the `OPTIONS` instruction
**leave this phase**."*

**What I read.** `directive_gap` (`rexx-exec/src/lib.rs:1006`-`1066`) has exactly these Phase 5
arms: `::REQUIRES` (`:1033`), `::OPTIONS` (`:1038`), `::CLASS naming another class` (`:1048`),
`::ANNOTATE naming a target` (`:1056`). Its other gaps are `::ROUTINE EXTERNAL`,
`::METHOD EXTERNAL` and `::ATTRIBUTE EXTERNAL`, all Phase 7.

So "the four" is `::REQUIRES`, `::OPTIONS`, `::CLASS naming another class`,
`::ANNOTATE naming a target` -- and D31 hands `::OPTIONS` to another unit. Criterion 2 and D31
contradict each other. Either the criterion means three, or D31's unit has to land inside this
phase after all.

### H9. The performance "floor" the guard is read against does not exist in the form the spec describes, and the tree says not to use the number it points at. CONFIRMED.

**Spec's words:** *"The pinned baseline is what such a measurement compares against, and
`bench-baselines/README.md` states the floor to read an `across_builds` movement against."* And:
*"interleaved, read against the floor rather than against zero"*. And criterion 4: *"No classic axis
regressed against the pinned baseline beyond the floor"*.

**What I read.** `rust/bench-baselines/README.md` does not contain the word "floor". Its closing
sentence is: *"An `across_builds` row is between two binaries and is the weaker kind. Interleaving
removes the machine's drift from it; it does not remove the code placement's, which Task 8 bounded
at `+/-0.74%` on an axis the change it was measuring could not reach."*

`docs/superpowers/plans/2026-08-09-phase-4e-ir.md:927` addresses that number directly: *"Task 8's
`+0.74%` was an instruction figure across a different binary pair and is not reproduced or refuted
here; **nothing licenses carrying it as a general floor**"*. The same paragraph sets the two
instrument-specific bounds the spec collapses into one: an `instructions:u` figure from `rexx-arms`
*"resolves far below 0.75%"*, while *"a **cross-build** `cycles:u` figure below about 3% is not"* a
reading. The spec's guard run is a cross-build sitting (`--build pinned=... --build head=...`) and
never says which instrument criterion 4 is read on.

Worse for this phase specifically: `docs/superpowers/plans/phase-4f-record.md` entry 17 records a
**null control** -- `Repr` collapsed to a single arm, *"by construction it does what BASE did"*,
byte-identical stdout, instruction counts across three builds spreading 0.0001%-0.0863% -- moving
`varlookup` by **+6.98%** and **+6.78%** on two independent harnesses, and `emptyloop` by +2.37% and
+3.07%. Its own conclusion: *"The cost does not follow the enum. It follows **touching `Body` at
all**."* Phase 5 is a phase whose value-representation section is about adding `Body` variants and
whose D34 puts a real Array in the heap. The instrument the spec proposes has a demonstrated
false-positive amplitude on the axis and the change class this phase consists of, and the spec
names no floor that accounts for it.

### H10. `.File`, criterion 1 and the L2 paragraph disagree. CONFIRMED.

**Spec's words:** *"the roadmap also records at `2026-07-27-rust-rewrite.md:304` that the ooTest
framework cannot start without `SysFileExists` and `.File`, which are Phase 7's."*

`:304` says what the spec says it says (I read the line; it also names `SysFileTree` for the other
runner). But `.File` is defined by `StreamClasses.orx:506`
(`::CLASS "File" public inherit Comparable Orderable`), and criterion 1 requires
`StreamClasses.orx` to run to completion in **this** phase. So either `.File` is not Phase 7's, or
criterion 1 is not this phase's. Combined with H1, the second is the live reading -- and then the
L2 paragraph's premise ("a gate criterion that cannot be reached is worse than no criterion")
applies to criterion 1 itself.

---

## MEDIUM

### M1. D30 states the opposite of what `ir.rs`'s comment says, on the code it cites. CONFIRMED.

**Spec's words**, D30: *"`::REQUIRES` lands last in this phase ... It is **the one construct that
breaks the two append-only arguments** the `CallSite` table and the plan cache rest on."*

**What I read.** `rexx-exec/src/ir.rs:1197`-`1201`: *"**The property the table needs is that one,
and not 'the map is written once'**, which is a stronger thing that happens to be true today and
would stop being the reason if a `::REQUIRES` or an external-file call ever installed a routine
mid-run. Append-only is what the refusal enforces, and append-only is enough."* The append-only
property is stated at `:1189` as *"`Interp::routines` never rebinds a name it has already bound"*,
enforced by the 99.903 duplicate-`::ROUTINE` refusal, and the comment says it holds *"however many
programs are loaded"*.

So the cited comment says (a) append-only is **not** what `::REQUIRES` would break -- the stronger
"written once" is -- and (b) `::REQUIRES` is **not** alone: an external-file call is named beside
it. And an external-file call is precisely what `CoreClasses.orx:122` and `:124` are, so this
phase meets that construct at criterion 1 whatever D30 does about `::REQUIRES`.

The plan cache's own argument (`plan.rs:41`-`:47`, `lib.rs:1599`-`1605`) is that `ProgramId`s are
never reused because `Interp::programs` holds an `Rc` for every id it issued. Loading more programs
is what that argument is built to survive. D30's conclusion may still be the right ordering; the
reason given for it is not the reason the code gives.

### M2. "Comparator and its six subclasses" -- there are seven. CONFIRMED.

`/bin/grep -ain "^[[:space:]]*::class" CoreClasses.orx` lists, at `:1312`, `:1317`, `:1322`,
`:1327`, `:1338`, `:1351`, `:1369`: `DescendingComparator`, `CaselessComparator`,
`CaselessDescendingComparator`, `ColumnComparator`, `InvertingComparator`, `NumericComparator`,
`CaselessColumnComparator`. The spec's *set* is otherwise complete -- with seven, its mixin list
plus its utility list accounts for every `::CLASS` directive in the file. Only the cardinal is
wrong, and per the project's own rule the cardinal should not have been written.

### M3. Four of the classes the spec calls "mixins" are not `MIXINCLASS`. CONFIRMED.

**Spec's words:** *"Its `::CLASS` directives are mixins (`Collection`, ... `SupplierMixin`,
`ManyItemMixin`, `SetMixin`, `BagMixin`, ...)"*.

Read at the file: `:172` `::class "SupplierMixin"`, `:218` `::class "ManyItemMixin"`, `:411`
`::CLASS 'SetMixin'`, `:557` `::CLASS 'BagMixin'` -- no `MIXINCLASS` keyword, no `PUBLIC`, on any
of the four. Every other class the spec calls a mixin does carry `MIXINCLASS` (`:729`, `:733`,
`:744`, `:1011`, `:1234`, `:1296`, `:1299`, `:1307`, the Comparator subclasses, `:3840`, `:3974`).

The four are consumed by `~inheritInstanceMethods` at `:80`-`:87`, which the file calls *"a phony
inherit"*, not by `~inherit`. The distinction is not cosmetic: it decides whether the class appears
in the target's `superClasses` list, and therefore whether the flattened dictionary carries its
scope. `rexx-classes` needs both operations and they are different.

### M4. D33 names a different `VALUE` form from the one the body measures. CONFIRMED.

**Spec's body:** *"`VALUE`'s local-pool read of a leading-dot name ..."*, with the `.LOCAL`/`.ARRAY`
transcripts. **Spec's D33:** *"the class registry, environment-symbol lookup and `VALUE`'s
**empty-selector form** all resolve through them."*

`phase-4-exclusions.txt` records these as two separate KNOWN GAPs: the local-pool read at `:2530`
("VALUE ON A DEFINED ENVIRONMENT NAME IS SILENTLY WRONG AT RC 0"), and the external-selector form
at `:2569`, whose branch (a) is *"an EMPTY selector reads/writes `TheEnvironment`"*.

**I re-ran all of the spec's figures rather than quoting them**, and measured the fourth form:

| program | oracle | ours |
|---|---|---|
| `say value('.LOCAL')` | rc 0, `The Local Directory` | rc 0, `.LOCAL` |
| `say value('.ARRAY')` | rc 0, `The Array class` | rc 0, `.ARRAY` |
| `say .LOCAL` | rc 0, `The Local Directory` | rc 120, `rexx-exec: an environment symbol is not implemented` |
| `say value('.LOCAL',,'')` | rc 0, `..LOCAL` | rc 120, `rexx-exec: VALUE's external-selector form is not implemented` |

Every figure the spec quotes reproduces. But the empty-selector form D33 names answers `..LOCAL`,
not `The Local Directory` -- it is a different subsystem with a different oracle answer, and D33 as
written commits the phase to the one the body never measured while leaving the one it did measure
un-named in the decision.

### M5. "the three existing `run.rs` indent tests" names a set by a size that is wrong, in a file that no longer holds them. CONFIRMED.

`/bin/grep -c "fn .*indent" crates/rexx-exec/src/run/tests.rs` is **16**, and they are in
`run/tests.rs`, not `run.rs`. They are also not one shape: `the_corrected_28x_indent_rule_matches_
all_fourteen_probed_shapes` asserts an integer computed by `static_indent` over a table of sources,
while `one_two_and_three_enclosing_dos_indent_by_two_four_and_six` and its neighbours are the
stderr-shaped ones. A task told to copy "the three existing indent tests" has to guess which three
and which shape.

### M6. The `apply_binary` figure is not from `perf-baseline.md` and predates the pinned baseline. CONFIRMED.

**Spec's words:** *"It is 13.5% of the pinned `rexxcps`' allocations"*, in a section that opens
*"the standing is pinned at `b029abe77` in `perf-baseline.md`"*.

`/bin/grep -rn "apply_binary" docs/` finds the figure at
`2026-07-27-rust-rewrite.md:2452` and `2026-08-14-parse-source-buffers.md:95`. It is **not** in
`perf-baseline.md`, and `/bin/grep -rn "13\.5[^0-9]" docs/superpowers/plans/perf-baseline.md`
returns nothing. `parse-source-buffers.md:95` says *"13.5% of **that program's** allocations"*, with
"pinned" appearing on the neighbouring `render_integer_padded` row. The reading is a profile of
`samples/rexxcps.rex` taken on 2026-08-14; `b029abe77` was pinned 2026-08-15. Placed where the spec
places it, the sentence reads as a figure from the pinned baseline. It is a carried-forward number
with no citation, which is the thing `bench-baselines/README.md`'s own "Why the file rather than the
report" section exists to stop.

### M7. `emptyloop` is missing from the guard, and `rexxcps` is not an axis. CONFIRMED.

**Spec's words**, D35 / Q13: *"**The guard is the classic axes only**: `arith`, `compound`,
`strings`, `varlookup`, `alloc4c` and `rexxcps`."*

`rexx-bench/src/bin/rexx-bench-suite.rs:145`-`186`, the `AXES` literal, has
`Role::Loop` on: `alloc4c`, `arith`, `compound`, **`emptyloop`**, `strings`, `varlookup`.
`Role::Blocked` on `alloc`, `dispatch`, `heapshape`. `Role::Offset` on `startup`. There is no
`rexxcps` axis; `samples/rexxcps.rex` is measured in its own section of the harness.

`emptyloop` is measured in the pinned baseline (oracle 0.8794 s, this crate 1.3192 s, 1.50x) and is
the axis entry 17's null control moved by +2.37%/+3.07%. Under criterion 4 as written it is not
guarded.

### M8. "the baseline's last table" is not the last table. CONFIRMED.

**Spec's words:** *"all three exit 120 on a message send today, which the baseline's last table
records."* The table is real and its contents are exactly as the spec says -- `alloc`, `dispatch`,
`heapshape`, each `120`, each `rexx-exec: a message send is not implemented (Phase 5)`, under
"Axes this crate cannot run" in `perf-baseline.md`. It is followed by the whole "The regression
guard -- a pinned binary and `rexx-arms` instruction counts" section and its tables. Trivial in
itself; it matters because a reader sent to "the last table" lands on the guard's provenance block
instead.

### M9. `Body` is asserted at **most** 80 bytes, not at 80. CONFIRMED.

`rexx-core/src/body.rs:134` is `const _: () = assert!(size_of::<Body>() <= 80);`, and its own doc at
`:130` says *"Upper bounds rather than equalities: the claim is that nothing widened, and shrinking
needs no decision."* The spec's *"`Body` is asserted at 80 bytes"* is not what the assertion says.
The quoted comment fragment -- *"Phase 5 adds variants and is expected to trip this"* -- is verbatim
and correct, and so is the spec's conclusion.

---

## LOW

### L1. The GUARD refusal message is not the one the spec quotes. CONFIRMED.

*"This crate exits 120 on both with `REPLY is not implemented (Phase 5)`."* Measured: `guard on`
gives `rexx-exec: GUARD is not implemented (Phase 5)`.

Also unremarked, and relevant to D32's scope: bare `guard` with no ON/OFF is oracle rc **231**,
`Error 25.913: GUARD must be followed by the keyword ON or OFF; found ";"`, and ours already
produces `rexx-exec: 25.913: Invalid subkeyword found.` at rc 120. So one of GUARD's two
legality checks is already half-built here and diverges on status and transcript.

### L2. The roadmap quote at `:82` is truncated without a mark. CONFIRMED.

Spec: *"...the single highest-leverage milestone in the plan."* Source `:82`: *"...the single
highest-leverage milestone in the plan (Phase 5), and everything before it is scaffolding for that
moment."*

### L3. "in that order, because that is the order `CoreClasses.orx` needs them" is unsupported. PLAUSIBLE.

The file's first directive is `::METHOD string_cls_nl` at `:138`; its first `::CLASS` is at `:172`;
all directives are installed before the prologue at `:39` runs a clause. There is no execution order
among them for the file to "need". The first *refusal* we hit is `::CLASS naming another class`
(measured, below), which is a different statement, and the spec makes it separately and correctly in
its open questions.

### L4. `createImage()` is the image-**build** path, not startup. CONFIRMED, and not a spec error.

`MemoryObject::createImage` runs only from `RexxMemory.cpp:205` under `if (!restoringImage)`, and
ends at `Setup.cpp:1812`-`:1814` with `saveImage(imageTarget)` and `exit(0)`. The oracle at ordinary
startup restores `/home/moritz/dev/repos/ooRexx/build/lib/rexx.img`; it does not run
`CoreClasses.orx`. The spec never claims otherwise, and D26 and criterion 6 are consistent with it.
Recorded because the neighbouring `perf-baseline.md` sentence *"it starts fast by not doing the
work the oracle does at startup"* describes work the oracle does at **image-build** time, and
criterion 6's hyperfine comparison is against an image restore -- which is exactly the deliberately
unfavourable comparison D2 sets up and warns about, so criterion 6 should say so where D2 does.

---

## Claims I checked and found sound

Named individually, not counted.

**The C++ citations.**

* `Setup.cpp:1785` is `RexxString *symb = getGlobalName(BASEIMAGELOAD);`. The surrounding block
  creates the `LOCAL` method on `TheEnvironment` (`:1777`-`:1781`), resolves the program name, and
  runs it with `RexxObject *args = TheRexxPackage;` and `argCount 1` (`:1795`-`:1798`) -- so *"runs
  it as a program with `TheRexxPackage` as its single argument"* is exact, and *"`Setup.cpp` puts
  the `LOCAL` method on `TheEnvironment` before `CoreClasses.orx` runs"* is exact.
* `platform/unix/PlatformDefinitions.h:99` is
  `#define BASEIMAGELOAD "CoreClasses.orx" /* MHES 29122004 */`. The windows file defines the same
  string at its `:78`.
* `ClassClass.cpp:1036` is `void  RexxClass::updateSubClasses()`, `:1071` is
  `void RexxClass::updateInstanceSubClasses()`, `:1148` is
  `void RexxClass::createInstanceBehaviour(RexxBehaviour *target_instance_behaviour)`. All three
  line numbers land on the function the spec attributes to them.
* `updateSubClasses` does clear both dictionaries, rebuild via `createInstanceBehaviour` and
  `createClassBehaviour`, and then recurse over `getSubClasses()` -- the cascade the spec
  describes.
* `createInstanceBehaviour` merges superclasses in reverse order into one target behaviour, guarded
  by `hasScope`, so the result is flat.

**The build order.** I did not trust the spec's grep. `/bin/grep -rn "::createInstance\b"
interpreter/` finds the definition of `X::createInstance` in `interpreter/classes/` for exactly the
set the spec lists, and no others; the only other `createInstance` in the tree is
`Interpreter::createInstance` (`runtime/Interpreter.cpp:326`, reached from
`api/InterpreterAPI.cpp:398`), which creates an interpreter instance, is not a class creation, and
is **not** called from `createImage()` -- `createImage` calls
`Interpreter::createInterpreterInstance()` instead, so the spec's list is not contaminated by it.
Reading `createImage()` end to end (`Setup.cpp:227`-`:1815`), the `createInstance()` call sequence
is exactly the spec's list in exactly the spec's order, and the two quoted comments are the file's:
*"Class and integer has some special stuff, so get them created first"* and *"string and object are
fairly critical"*. Nothing else in `createImage` creates a class: `TheNilObject`, `TheEnvironment`,
`TheSystem`, `TheCommonRetrievers`, the cached integers, the `RexxInfo` at `:1736` and
`createRexxPackage()` all create instances. The macro section (`:448`-`:1723`) *defines* the same
classes in a different order and adds nothing new; that difference is worth the plan knowing, since
the spec's hazard paragraph says `createInstance()` fixes *"the order the behaviours are built in"*
and it is `CompleteClassDefinition`/`buildFinalClassBehaviour` in the macro section that does that.

**`CoreClasses.orx`'s directive vocabulary.** Enumerated from `::` rather than from line starts:
`/bin/grep -aoc "::" CoreClasses.orx` is 348 and
`/bin/grep -ao "::" CoreClasses.orx | wc -l` is 348, so no line carries two. Every one of the 348 is
`::` followed by `METHOD`, `CLASS` or `ATTRIBUTE` in some case. Exactly one is not a directive --
`:42`, inside a comment: *"needed is to ensure the PUBLIC keyword is specified on the ::CLASS
directive"*. None is indented (`/bin/grep -an "^[[:space:]]\+::"` returns nothing). So *"its
directives are `::METHOD`, `::CLASS` and `::ATTRIBUTE`, and nothing else"* and *"No `::REQUIRES`,
no `::ROUTINE`, no `::OPTIONS` anywhere in it"* are both **true**, checked the way the ground rules
ask. (`StreamClasses.orx` adds `::CONSTANT` -- see H1. Neither file has `::REQUIRES` or
`::ROUTINE`.)

**The classification of the `::CLASS` directives** is otherwise right: none of the 32 is a
primitive class being redefined in Rexx. The nearest cases are `TraceObject subclass StringTable`,
`CircularQueue subclass queue`, `Properties subclass Directory` and `Alarm SUBCLASS object`, all
subclasses of primitives rather than primitives. The utility list (`Alarm`, `Ticker`, `Monitor`,
`CircularQueue`, `Properties`, `DateTime`, `TimeSpan`, `ArgUtil`, `Validate`, `LocalServer`,
`TraceObject`) matches the file. The quoted file comment *"unattached METHOD definitions for the
various enhanced objects created above"* is at `:130`-`:131`, and the file-scope `::METHOD` block
it introduces (`:138`-`:162`) is exactly the `string_cls_*` group, which `:70`-`:74` attaches to
`.String` as class methods.

**The three-file reachability.** `call 'StreamClasses.orx' rexxPackage` is `CoreClasses.orx:122`
and `call 'PlatformObjects.orx' rexxPackage` is `:124`, both exactly as quoted, both before `exit`
at `:126`. Running the first does run all three.

**Flat lookup at send time.** `RexxObject::messageSend` (`ObjectClass.cpp:866`) is
`behaviour->methodLookup(msgname)`; `RexxBehaviour::methodLookup`
(`RexxBehaviour.cpp:438`) is one `methodDictionary->getMethod(messageName)`; and
`MethodDictionary::getMethod` (`MethodDictionary.hpp:69`) is `get(methodName)` on the hash
collection. One hash lookup, no chain walk. (H6 is the qualification, not a contradiction.)

**A chain walk gives a different answer, with a witness.** I built one:

```rexx
say .C~new~m
::class A subclass Object
::method m
  return "A"
::class B subclass A
::class Mix mixinclass A
::method m
  return "Mix"
::class C subclass B inherit Mix
```

Oracle: rc 0, stdout `Mix`. A superclass-chain walk from `C` following a single `superclass` link
visits `C`, `B`, `A` and answers `A`. The merge answers `Mix` because `createInstanceBehaviour`
processes `superClasses` last-to-first and `Mix` overlays `A`'s dictionary after `B` contributes
nothing. A second program confirms the list itself: with `B` also defining `m`,
`.C~superClasses~makestring("LINE", ",")` is `The B class,The MIX class` and `~m` is `B`.

**`BehaviourTable` cannot express a mixin.** `rexx-core/src/behaviour.rs:24` is
`superclass: Option<BehaviourId>`, singular, and `lookup` (`:65`-`:81`) walks that link with a
visited set. Verbatim as the spec describes. (It is reachable only from
`rexx-core/tests/behaviour.rs` today -- no production caller -- which strengthens rather than
weakens D29.)

**Our first refusal on `CoreClasses.orx`.** Run this session: rc **120**, stdout empty, stderr
`rexx-exec: ::CLASS naming another class is not implemented (Phase 5)`. Byte for byte what the spec
says.

**The REPLY transcript's text.** Reproduced exactly, including `Error 99.919:  REPLY can only be
issued in an object method invocation.` at rc 157, and `99.911` for `guard on`. Only the *timing*
claim fails (H2).

**Our own repository citations.** `2026-07-27-rust-rewrite.md:517` is
`    rexx-classes/             # Phase 4-5`; `:518` is
`    rexx-lib/                 # Phase 5: loads CoreClasses.orx / StreamClasses.orx`; `:304` says
what the spec says it says. `ls rust/crates/` shows neither `rexx-classes` nor `rexx-lib`, so
*"nothing in Phase 4 created it"* holds.

**`ir.rs`'s `CallSite`.** `struct CallSite(Cell<Option<Resolved>>)` at `:1227`, a classic-call
cache, with `Interp::routines`' non-rebinding property at `:1189` as the spec says. (M1 is about
D30's gloss on it, not about this description.)

**`phase-4-exclusions.txt`'s VALUE KNOWN GAP.** At `:2530`. It does say *"No owner is assigned to
closing it, because closing it is building the subsystem, which is Phase 5's to do"*, and its
account of the fallback (`RexxDotVariable::getValue`, `ExpressionDotVariable.cpp:195`, reached only
after a package-environment lookup and a reflection-name table come back empty) is what the spec
paraphrases. All four measured figures re-run and reproduce (M4's table).

**The bare `OPTIONS` instruction.** `phase-4-exclusions.txt:2087` onward records it, with our
message `rexx-exec: OPTIONS is not implemented (Phase 5)` and the oracle running the program. The
spec's use of it is accurate.

**D24's amendment**, `2026-08-09-phase-4e-ir.md:1082` -- the spec's *"correct **because it needs no
guard**, which is precisely the property a send cache lacks. The classic case validates the seam
and not the caching discipline"* is that paragraph nearly verbatim.

**The blocked-axis assertion.** `rexx-bench-suite.rs:1190`-`:1210` asserts every `Role::Blocked`
axis exits non-zero, with a guard against the list being empty. It does go red the moment a refusal
is removed, so the spec's "re-role them in the same commit" is a real requirement rather than a
formality.

**`ir_dual` diffs raw stderr.** `crates/rexx-exec/tests/ir_dual.rs:152` is
`assert_eq!(tw.stderr, ir.stderr)`, and its module doc at `:55` says so. (It compares the two
engines to each other, so it cannot see an indent both engines get wrong; the spec's in-crate
assertions are what covers that, and the spec's own "two instruments, because they fail
differently" acknowledges the split.)

**`tests/support` normalisation.** `crates/rexx-exec/tests/support/mod.rs`'s `normalize_stderr`
(`:155`) and `normalize_line` (`:222`) do collapse indent, with a positive test
(`two_clause_lines_differing_only_in_indent_width_normalise_equal`) and a negative one
(a banner line passes through unchanged), so the hole the spec describes is real.

**The `>N>` correction.** The survey does record it at `:298`, `:334`, `:339`-`:342` and `:519`:
`>N>` from `ExpressionClassResolver.cpp:135` via `traceClassResolution`, carrying the qualified
name, while `QualifiedCall` traces `>F>` with the bare name. The spec reports the survey faithfully.

**`ExprKind::List`.** `crates/rexx-parse/src/ast.rs:209` carries the discriminator verbatim:
*"measured, `(1,)~size` is 2 and `(1,,)~size` is 3, where `f(1,)` passes one argument"*, citing
`LanguageParser.cpp:3145`. Re-measured on the oracle this session: `say (1,)~size` prints `2` and
`say (1,,)~size` prints `3`, rc 0. `ExprKind::List` is a Phase 5 loud refusal today
(`rexx-exec/src/lib.rs:1249`), so nothing approximate ships now either.

**D1's root-set criterion.** `2026-07-27-rust-rewrite.md:121` names the global tables
(`.environment`, `.local`, class registry) as part of the enumerable root set, so *"class objects
live in the arena like everything else, which is D1's own criterion"* is D1's own criterion.

**D2.** `2026-07-27-rust-rewrite.md:140`-`:161`. *"Phase 5 exit measures cold start for (a) with
hyperfine against `build/bin/rexx`"* and *"build (b) only if (a) costs more than ~50 ms of wall
clock over the C++ startup"* are both there verbatim, so the spec's criterion 6 states D2
faithfully.

**The pinned artifacts exist** as described: `rust/bench-baselines/pinned/rexx-run-pre-phase-5`
(14191256 bytes, the sha256 `perf-baseline.md`'s provenance block records) and
`rust/bench-baselines/pre-phase-5-arms.tsv`. `perf-baseline.md`'s "The pre-Phase-5 baseline"
section is at `:854`, dated 2026-08-15 at `b029abe77`. (H9 is about the "floor", not about these.)

---

## What I searched *for*, and what my terms could not reach

* For directives I enumerated `::` occurrences byte-wise with `/bin/grep -a`, not line-start
  matches, and separately checked for indentation and for non-line-start occurrences. A directive
  spelled across a line continuation (`,` at end of line) would evade both my census and the
  spec's; I did not check for one, and ooRexx does not permit it for the `::` token itself.
* For class creation I searched `::createInstance\b` over the whole of `interpreter/`. A class
  created by a differently named factory, or by direct `new RexxClass(...)` outside
  `CLASS_CREATE`, would not appear. I read `createImage()` end to end as a control on that, which
  is how I established the non-`createInstance` creations are instances.
* For the scope-override path I searched `~super` and line-initial `forward` in `CoreClasses.orx`,
  plus `superMethod` across `ObjectClass.cpp`, `RexxBehaviour.cpp` and `MethodDictionary.cpp`. A
  scope override reached through `.methods` or through `Method~setUnguarded`-style reflection would
  not be in those terms.
* For the `apply_binary` figure I searched `apply_binary` across `docs/` and `13\.5[^0-9]` across
  `perf-baseline.md`. A restatement of the figure in different words would evade both.
* I did **not** re-measure the 13.5% or 9.4% allocation shares -- that needs the profiling harness
  `2026-08-14-parse-source-buffers.md` describes, which is out of a review's budget. M6 is about
  provenance and placement, not about whether the number is right.
* I did **not** run any performance sitting. H9 and M7 rest on reading `AXES`, `README.md`,
  `perf-baseline.md` and `phase-4f-record.md` entry 17, all of which I quote.
* No program from `rust/corpus/oracle-crashes.txt` was run. Nothing outside this file and
  `$SCRATCHPAD/r1/` was written.
