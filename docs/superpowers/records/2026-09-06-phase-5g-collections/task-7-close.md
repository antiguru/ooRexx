# Phase 5g Task 7 — the sweep, the instrument, and the close

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
Spec: `docs/superpowers/specs/2026-09-06-collections.md`. BASE `f5b383d6f`.

---

## What the phase moved

Diffed from `corpus/method-bodies.txt` at BASE, not summed from the task
reports:

| class | arm | rows |
|---|---|---|
| `Queue` | instance | 33 |
| `CircularQueue` | instance | 30 |
| `List` | instance | 28 |
| `Array` | instance | 27 |
| `Supplier` | instance | 6 |
| `Monitor` | instance | 2 |
| `Queue`, `List`, `CircularQueue` | class | 1 each |

**129 rows, every one `loud` -> `answers`.** No row was added or removed, none
started diverging, and none that answered stopped. The table has the same 1347
rows it had at BASE.

**Every ordered-collection instance row now answers**: `Array`, `Queue`,
`List`, `CircularQueue` and `Supplier` are at zero `loud`.

`Monitor`'s two are the phase reaching past its own classes, the way 5f reached
`DateTime` — it forwards to a collection.

## The instrument, which is the honest number

`corpus/collection-arity.tsv`, re-run and diffed against the run Task 0
committed:

| verdict | Task 0 | now |
|---|---|---|
| `agree` | 21 | **183** |
| `send-differs` | 90 | 56 |
| `setup-differs` | 321 | 193 |
| `exempt` | 1 | 1 |

**162 rows moved from disagreeing to agreeing under a real argument list**,
against `method-bodies.txt`'s 129. The two numbers differ because they measure
different things and that was the point of building the second one: the
verdict column agrees about an arity error, and the instrument does not.

The 193 rows still reading `setup-differs` are the mapped classes — `Table`,
`IdentityTable`, `Relation`, `Set`, `Bag`, `Directory`, `StringTable`,
`Properties`, `Stem` — which is Phase 5h and exactly the population the spec
scoped it on.

## The receiver sweep, and a control that mattered

`RECEIVER_OVERRIDES` gained populated receivers for `Array`, `Queue`, `List`
and `CircularQueue`, which retires the 5c follow-up's open question (105 rows
across thirteen classes) for this phase's four.

**It moved nothing, and the control is what makes that a finding rather than a
blank.** An override that is silently inert looks exactly like one that is
applied and changes no verdict. Pointing `Array`'s override at `.Queue~new` —
a receiver that is not an array at all — moved five `Array` rows from `rc 0` to
`rc 159`, so the mechanism is live. The sweep changes no verdict because the
bodies now agree at an empty receiver and a populated one alike, which is what
being implemented means.

Phase 5f's Task 0 predicted a populated receiver would "sharpen `Queue` 10 of
43, `Array` 11 of 44, `List` 7 of 38". Measured now, it sharpens none of them —
that prediction was about rows that were unimplemented, and the discrimination
it offered was against a refusal rather than against an answer.

## `Queue~of` and `List~of`

Needed by the sweep — a populated receiver has to be one expression — and
worth their own rows. One body: each creates a collection of the **receiver's
own** class and appends the arguments (`classes/ListClass.cpp:1010`), so
`.Queue~of('a')~class~id` is `Queue` and a subclass's is the subclass. An
omitted argument is refused at its own position: `.List~of('a',,'c')` is
93.903 `argument 2 is required`.

They are class methods, so they are registered in `NATIVE_CLASS_METHODS` and
not in the chained instance slice — the build panics on the difference, which
is how it was found.

Witness `corpus/lang/collection_of.rex`. **M12 — `of` always answers an
`Array`** — predicted red on it and **confirmed**: 417 of 418.

## The phase's corpus

Strict corpus **418 of 418 matching**, from 405 at BASE. Thirteen witnesses
added, each filed in `corpus/phase-5c.txt`, `EXPECTED_SUBSET_5C` and
`sourceline_oracle/`.

## What this phase found that outlives it

* **A new oracle crash.** `Stem~index` and `Stem~hasItem` sent no argument are
  a deterministic SIGSEGV: both reach `findByValue(nullptr)` behind an arity
  that is a maximum. Filed in `corpus/oracle-crashes.txt`, no upstream ticket.
* **A use-after-free in this crate**, caught only by collect-on-every-allocation
  — index objects allocated into an unrooted `Vec`. The release run, the corpus
  differential and both engines all agreed before the fix.
* **Four new error constructors**, each measured rather than guessed:
  93.918 `Incorrect list index`, 93.954 `single-dimensional array only`,
  93.937 `No more supplier items`, 98.975 `Missing array element`.
* **`RexxObject::makeArrayRexx` is a virtual**, so the one entry point that
  crosses the 5g/5h split is the one that must be written twice: items for
  `Array` and `List`, indexes for the hash classes, tails for `Stem`.

## What is left, and who owns it

* **Phase 5h**, the mapped classes: 193 rows reading `setup-differs`, which is
  the object-keyed store the spec's D95–D98 scope.
* **`Properties`**, which needs 5h's `Directory` and the design branch spec D90
  records — `native_directory_new` gives a `Directory` subclass a plain
  instance *on purpose*, and this phase's `collection_store` is the shape that
  would let it have both halves.
* **`RexxQueue`**, out of scope by D93 and Phase 7's.
* **`Object~copy`**, which blocks `Collection`'s Rexx-level set operations —
  found by Task 1's first probe and owned by no phase.
* **`ARG(1,'A')`** (spec D100), which blocks the mapped classes' `of` class
  rows. Not collection work.

No `CLOSED_PHASES` change and no `class-set.txt` edit: `5g` is these documents'
name, not a value anything reads.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
