## Task 3: the extractors, and the committed row sets

**Goal.** Build the extractors, **run them**, and commit the row sets they produce, with each
extractor beside its data file and each file stamped with the revision it was derived at (D56).

**Why this is a task and not spec text (R32).** Three consecutive review rounds each found their
defects in the derivation rules the previous round had written, and the worst was found in one pass
by *running* the rule: as specified it produced rows naming things the oracle answers on neither arm,
and no row at all for the operator methods. A derivation procedure is a program, and prose review is
the wrong instrument for a program.

**Build**, in `rexx-extract` (a new module and a `rexx-extract-docs` binary, following the four modes
already there), emitting to `rust/corpus/docs/`:

* `provide-sections.txt` -- one row per `<section id>` under `provide.xml`'s `provide` chapter, at
  every nesting level. Table C's concept row set.
* `class-set.txt` -- every `cls*` section across the four books, minus `RegularExpression`, with
  `RexxInfo` marked as an instance row and `ArgUtil` added with the XML comment as its citation.
* `hierarchy-edges.txt` -- the `chi` `<simplelist>`, five `&nbsp;` per level. **Three rules:** strip
  XML comments before reading `<member>`s; the root emits no self-edge; excluded names emit no edge.
  Rules 2 and 3 are witnessed by the end-to-end oracle run; **rule 1 is not**, so the derivation
  additionally **asserts `ArgUtil` is absent from the derived edge set** -- an extractor that skips
  comment stripping emits an extra edge the oracle confirms, and the end-to-end run stays at zero
  failures over a wrong member set.
* `class-methods.txt` -- one row per (class, method, arm, **status**). Every hard case the spec lists
  must be handled and each is a named case in the code: entity-reference prefixes glued to names
  (`&added50;size`) with **no DTD-resolving escape available**, since `provide.xml` uses four
  entities `rexxref.ent` does not define and the DTD is an `http` URL; group-heading titles; the
  operator methods those headings document, which have no title anywhere; method sets pulled in by
  `xi:include` of a `*classmethods.xml`, where neither the class's own section span nor an
  `mth<Class>` name prefix is correct alone (the prefix collides: `mthString*` matches
  `StringTable`'s set); the arm suffix with its case wobble, its one `(Inherited Class Method)`, its
  one non-marker parenthetical, and the two bare-`new` titles that are class-side with no suffix;
  slashed titles naming two methods; and `objectclassmethods.xml`, which nothing includes.

  **The status column is the spec's three row statuses and it is not optional** -- without it this
  task's own completeness evidence is unmeetable, which is how an earlier draft read. `covered` is a
  class in the set a bare `~new` constructs, or one opted in with a committed construction program;
  `not-covered` is a class with no construction program, **a statement about this project's coverage
  carrying no claim about the oracle**; `unreachable` is grounded in a sentence of the reference
  itself and belongs to exactly the two classes that carry one -- `clsBuffer` and `clsPointer`, both
  reading "can only be created using the native code application programming interfaces"
  (`utilityclasses.xml:429`, `:6910`). The constructible set is measured, not derived: iterating
  `.environment` and sending a bare `~new` to every entry answering `~isA(.Class)` gives **62 class
  entries, 38 constructible, 24 raising**, re-run for this plan and agreeing with the spec's figure
  and the plan review's. Each `not-covered` row carries its stated reason, which is the only thing
  standing behind it.
* `directive-options.txt` -- table D's row set: per `dire.xml` section, the union of `<option>` names
  and `NAME subkeyword` indexterms, unioned with `DirectiveParser.cpp`'s per-function
  `SUBDIRECTIVE_*` arms. Row key **(directive, keyword, position)**, position being `subkeyword` or
  `value-of(subkeyword)`; a name in one source only is a row marked with its side; a keyword the
  directive's own parser has no arm for is marked `cross-reference` with the parser evidence.

**The task's own completeness evidence, and the scope it has to carry.** Every **`covered`** method
row is run against the oracle and **must answer on one arm or the other** -- `.X~hasMethod("M")` or
`.X~new~hasMethod("M")`. A `covered` row answering on neither is a defect in the extractor, not a
finding about the crate. **An arm-agreement measurement cannot witness this class of defect at all**,
because a name the oracle does not have has no arm to disagree about; that is why the previous round's
415-of-417 evidence scored well on a row set that was partly wrong. This sweep is **oracle-only**, so
it is runnable now: `~new` works there, and no crate capability is involved.

**The scope to `covered` is what makes the evidence achievable, and `Alarm` is the proof.** Measured:
`.Alarm~hasMethod("CANCEL")`, `("TRIGGERED")` and `("SCHEDULEDTIME")` are all **`0`**, and `.Alarm~new`
raises `93.901` -- so every one of Alarm's documented instance methods answers on **neither** arm, and
`Alarm` is the spec's own worked example of `not-covered`. An unscoped neither-arm gate is therefore
unmeetable by any correct implementation, which is the shape this project has learned to treat as
worse than no gate at all. A `not-covered` or `unreachable` row runs no program, has no verdict and is
gated on nothing; the spec says so and this plan adds no check, because the obvious one was withdrawn
for being unable to tell `String` from `Buffer`.

**Runnable now, and its dependency on Task 1 is real for a different reason than an earlier draft
gave.** The bare-`~new` sweep does **not** hang: re-run for this plan, 62 class entries, 38
constructible, 24 raising, none hanging, well inside a 25-second bound -- `.Ticker~new` bare is an
argument error, not a hang. The hang the spec records is `.Ticker~new(1000, msg)`, which is an
**opt-in construction program**, and this task writes none. Task 1 precedes it because this task alone
launches an oracle process per class and per probe, and because nothing downstream should be the first
task to discover the harness cannot bound a run.

**The both-directions check.** A `#[test]` re-derives each file and compares against the committed
copy **in both directions** -- a row that stops being derived is as red as one that appears. It
**fails when `oodocs/` is absent unless the run is explicitly marked docs-less** (D56), because a
check that silently skips is a failure mode this project has shipped. It also asserts each file's
stamp against the checkout's revision, which is where Task 2's ledger staleness rule is enforced.

**What this cannot see.** The both-directions check fires when one side moves and not when both move
together in one commit -- a regeneration. That is a diff for a human, deliberately, and it is why the
committed row sets are reviewed as artifacts rather than waved through as generated output. And every
one of these checks needs `oodocs/`, which none of CI's five platforms has: they fire on one developer
machine per gate, not continuously.

**Done when** every file is committed with its extractor and its stamp, **every `covered` method row
answers on one arm or the other with the neither-arm set empty**, every row carries a status and every
`not-covered` row carries a reason, and the `ArgUtil` assertion is in place -- demonstrable here, since
an extractor run without comment stripping emits the extra edge and the assertion fires. No sitting:
no `src/` of a measured crate changes.

---

