# Task 3 report — the extractors, and the committed row sets

Base `68a44aa1c`, branch `plan/rust-rewrite`. Two commits:

* `a8a3b7f2223bd2530be05e381e836cbce28cd72f` — the extractors and the five committed row sets.
* `fd8cb5cd063cf95438e4258124e980d57714f9b3` — drops `hierarchy::linkends`, a helper nothing called
  whose doc named a user that does not exist. Found by my own post-commit read of the diff, not by a
  tool: a `pub fn` is not dead code to the compiler, and its doc asserted a relationship between two
  modules that is not there.

Five row sets derived, run and committed under `rust/corpus/docs/`, with their extractors in
`rexx-extract` and a `rexx-extract-docs` binary that writes them. Every file carries a D56 stamp
(`# derived-at: oodocs/rexxref/en-US r13198`) and is re-derived and compared **in both directions**
by `crates/rexx-extract/tests/extract_docs.rs`.

**The `covered` sweep's neither-arm set is empty**: 793 `covered` method rows over 37 classes, every
one answering `1` on `.X~hasMethod("M")` or on `.X~new~hasMethod("M")`.

---

## What was built

| file | rows | module |
|---|---|---|
| `rust/corpus/docs/provide-sections.txt` | 21 | `src/docs/provide.rs` |
| `rust/corpus/docs/hierarchy-edges.txt` | 57 | `src/docs/hierarchy.rs` |
| `rust/corpus/docs/class-set.txt` | 63 | `src/docs/classes.rs` |
| `rust/corpus/docs/class-methods.txt` | 1345 | `src/docs/classes.rs` |
| `rust/corpus/docs/directive-options.txt` | 79 | `src/docs/directives.rs` |

Plus `src/docs.rs` (the module root, the `RowSet` renderer, `derive`, `svn_revision`),
`src/docs/xml.rs` (the shared text-level DocBook scanner), `src/bin/rexx-extract-docs.rs`, and
`tests/extract_docs.rs`.

```
cargo run -p rexx-extract --bin rexx-extract-docs -- \
    --oodocs ../oodocs --interpreter ../interpreter --out corpus/docs
```

`--check` compares without writing.

### `provide-sections.txt` — 21 rows

`id<TAB>depth<TAB>line<TAB>parent<TAB>title`, every `<section id>` of `provide.xml`'s `provide`
chapter at every nesting level. Three at depth 0 (`typcla`, `xcremet`, `classmeth`), the rest nested
under them. `unkno` and `reqstr` are rows, which is what the half exists for. The extractor panics on
a section with no `id` or with no title of its own, rather than emitting a row that cannot be keyed.

A section of another chapter is not a row — the span is `<chapter id="provide">` to `</chapter>`, and
a unit test with a second chapter pins it.

### `hierarchy-edges.txt` — 57 rows

`child<TAB>parent<TAB>level<TAB>line`. The `chi` `<simplelist>`, five literal `&nbsp;` per level, a
level-0 member's parent being `Object` from the section's own opening sentence. 59 live members, and
the two rules that drop a row take exactly `Object → Object` and `RexxInfo → Object`. Every child and
every parent is a name `class-set.txt` carries, asserted.

### `class-set.txt` — 63 rows

`name<TAB>entry<TAB>section<TAB>book:line<TAB>status<TAB>reason`. Every `cls*` section across the
four books, minus `RegularExpression`, with `RexxInfo` as an `instance` row and `ArgUtil` added with
`provide.xml:838`'s XML comment as its citation. 38 `covered`, 23 `not-covered`, 2 `unreachable`.

The line numbers reproduce Task 2's committed `cls*` block exactly (`clsClass` `fundclasses.xml:51`
through `clsStream` `streamclasses.xml:389`), which is an independent route to the same span.

### `class-methods.txt` — 1345 rows

`class<TAB>method<TAB>arm<TAB>status<TAB>section<TAB>origin<TAB>reason`, over 62 classes. 793
`covered`, 545 `not-covered`, 7 `unreachable`; 132 `class`-arm and 1213 `instance`-arm.

### `directive-options.txt` — 79 rows

`directive<TAB>keyword<TAB>position<TAB>side<TAB>evidence`. 62 `both`, 15 `parser-only`, 2
`cross-reference`. Per directive: `::ANNOTATE` 6, `::ATTRIBUTE` 13, `::CLASS` 8, `::CONSTANT` **0**,
`::METHOD` 12, `::OPTIONS` 33, `::REQUIRES` 2, `::RESOURCE` 2, `::ROUTINE` 3.

---

## The hard cases, and how each is handled

Every one is a named branch in `src/docs/classes.rs`'s `names_of` or in the module beside it, and
every one has a unit test.

**Entity-reference prefixes, with no DTD-resolving escape.** `xml::split_revision_marker` strips
`&added50;`, `&added51;`, `&added52;` and `&changed50;` off the front of a title. The whole extractor
family is text-level for the reason the spec gives and I re-checked: `provide.xml` uses `apos`,
`mdash`, `nbsp` and `quot`, which `rexxref.ent` does not define, and the DOCTYPE names the DTD by an
`http` URL. That is stated in `xml.rs`'s module doc, so a later reader meets the reason before the
mechanism.

**Comment stripping, everywhere and not only in `chi`.** `xml::blank_comments` overwrites a comment's
bytes with spaces and keeps its newlines, so byte offsets and line numbers survive. That matters
because every row cites a `file:line`; deleting comment bytes would renumber the document being
cited. Measured against the ledger's figures, the blanked four books give 250 / 346 / **371** / 45
`mth*` sections against 250 / 346 / 372 / 45 raw — the one difference being
`utilityclasses.xml`'s `mthSupplierInit` inside a comment opened at `:10061`. So the live figure is
1012, not the spec's 1013, exactly as the ledger's finding 6 says.

**Group-heading titles, and the operator methods they document.** A title of `Comparison Methods`,
`Arithmetic Methods`, `Concatenation Methods` or `Logical Methods` yields no name. The names come
from the class table's own text after the `<xref>` — `mthObjectComparisonMethods` is followed by
`= == &lt;> >&lt; \= \==`. A group-heading member with no operator list **panics** rather than
silently emitting nothing, and a unit test witnesses that.

**The two concatenation placeholders.** `(abuttal)` and `(blank)` are not method names. Measured on
the oracle from a fresh empty directory: `.Object~instanceMethods` contains an entry named `""` and
one named `" "`, and `.Object~method("")` and `.Object~method(" ")` both answer `The Method class`.
The extractor maps them to those names and writes them back as the placeholders, since neither
survives a tab-separated line; `render_method`/`parse_method` are the pair.

**Method sets pulled in by `xi:include`.** Neither of the two rules the spec warns about is used. A
class's methods are the `<member><xref linkend="mth...">` entries of its **own generated class
table** — the span before its first nested `<section>` — plus the members of every
`*classmethods.xml` that table `xi:include`s. Spot-checked: `Array` gets `difference`, `subset`,
`union` and `xor` from `mthCollection*`, and **no** `String` row comes from an `mthStringTable*`
section (0 of 135).

**`objectclassmethods.xml`, which nothing includes.** Re-checked rather than remembered:
`/bin/grep -arn 'objectclassmethods' oodocs/rexxref/en-US/` finds it only in the file itself, exit 1
for everything else. Following the includes is therefore the correct rule and globbing the directory
is not — the two differ by `mthObjectInit`, which `clsObject`'s own table comments out with
`<!-- see new() -->` and which `objectclassmethods.xml` carries live. The extractor **derives** the
unincluded set and writes it into `class-methods.txt`'s header, so it is policed by the
both-directions check rather than asserted in prose: if upstream starts including that file, the
header moves and the check reddens.

**The arm suffix.** `TITLE_PARENTHETICALS` is the whole list, matched case-insensitively so
`(Class method)` and `(Class Method)` are one case, with `(Inherited Class Method)` class-side and
`(Abstract Method)` / `(Private Method)` / `(Attribute)` instance-side. **A parenthetical the list
does not know panics** — an unknown one is either a kind marker whose arm nobody has decided or a
gloss that has to come off the name, and guessing either way gives a name the oracle has on neither
arm.

**`? (inline if)`, and a correction to how I first read the spec.** The spec calls it "one
parenthetical that is not a kind marker at all", and I first implemented that as "leave it alone",
which produced the method name `? (inline if)`. The oracle has `?` and not that — measured,
`.String~new("a")~hasMethod("?")` is `1`. So it is in the list, carrying **no** arm: it comes off the
name like every other parenthetical, and it says nothing about the arm.

**The two bare-`new` titles.** `BARE_NEW_CLASS_SIDE` is `mthClassNew` and `mthRexxQueueNew` — a named
list of two, not a rule about the word `new`. The unit test pins the negative too: `mthObjectNew`
with the same bare title is instance-side by this rule, because its own title carries
`(Class Method)` in the book and the exception list is not what decides it.

**Slashed titles.** `center/centre`, `canceled/cancelled` (twice) and `delete / delStr` split on `/`
with each half trimmed.

**`(no class or instance methods)`.** `clsSetCollection` and `clsInputOutputStream` print it. Both
still get rows, from the mixin sets their tables include, so no exception is needed to produce rows —
but the literal is what lets the extractor assert that every other class's table names at least one
method rather than silently emitting an empty class.

---

## The `covered` sweep

**Result: 793 rows asked, neither-arm set empty.**

Generated one program per `covered` class from the committed `class-methods.txt` (placeholders mapped
back to `""` and `" "`), then run from a fresh empty directory with absolute paths, three descriptors
read separately:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout -s KILL 10 /home/moritz/dev/repos/ooRexx/build/bin/rexx <ABS>/<Class>.rex )
```

Each program hoists `o = .Class~new`, then for each name prints its index and both
`.Class~hasMethod(n)` and `o~hasMethod(n)`. Printing the index rather than the name is deliberate:
two of the names are the empty string and a blank.

37 runs (the 38th `covered` class is `ArgUtil`, which the books document nowhere and which therefore
has no method rows), **every one rc 0 with empty stderr**, none hitting the 10 s bound.

| | |
|---|---|
| rows asked | 793 |
| answered on both arms | 64 |
| class arm only | 75 |
| instance arm only | 654 |
| **neither arm** | **0** |

### And a second sweep the brief does not require, which is the one that could still have found something

The gated sweep is scoped to `covered`, and the spec is right that an unscoped one is unmeetable. But
the **class arm** works without an instance, so every `not-covered` and `unreachable` row can be
asked on it, and a bogus name would answer `0` there too. Run the same way over the other 25 classes,
552 rows:

* **180 answered `1`. Every single row whose own arm is `class` answered `1` — 0 exceptions.**
* 372 answered `0`, and **every one of them is an `instance`-arm row**, which is the `Alarm` shape:
  the class has no instance, so the class arm is not the question that row asks.

That is the strongest independent corroboration available without construction programs: across
**both** sweeps, every class-arm row in the whole file answers, and every instance-arm row on a class
that has an instance answers.

---

## The `ArgUtil` assertion, seen firing

`hierarchy::edges_from_members` asserts `ArgUtil` is absent from the derived edge set. It fires in two
places, and both were run rather than reasoned about:

* `src/docs/hierarchy.rs`'s `skipping_the_comment_blanking_fires_the_argutil_assertion` — a
  `#[should_panic]` over a sample carrying the same comment shape.
* `tests/extract_docs.rs`'s test of the same name — over the **real `provide.xml`**: derive once from
  the blanked text (no `ArgUtil`, passes) and once from the raw text, catching the panic and checking
  its message. That is the assertion firing on the actual book, not on a model of it.

**Control F**, run: replacing the assertion's condition with `true || …` turns
`docs::hierarchy::tests::skipping_the_comment_blanking_fires_the_argutil_assertion` red
(`25 passed; 1 failed`). Restored, `26 passed`.

---

## Every control, run

Each of these was applied to the tree, run, observed, and reverted.

| control | what was changed | what happened |
|---|---|---|
| A | deleted `Array append` from `class-methods.txt` | `rows derived and not committed (1)`, FAILED |
| B | appended an invented `Array zznosuch` row | `rows committed and no longer derived (1)`, FAILED |
| C | rewrote a stamp to `r13100` | `left: ["# derived-at: … r13100"] right: [… r13198]`, FAILED |
| D | pointed the test at an absent `oodocs/` | FAILED, naming the `svn checkout` line and `REXX_DOCS_LESS=1` |
| E | the same, with `REXX_DOCS_LESS=1` | `7 passed` — the three docs-reading tests return early, the four that read only committed files still run |
| F | disabled the `ArgUtil` assertion | the unit test FAILED |
| G | moved `UNCONSTRUCTIBLE`'s `Buffer` citation from `:429` to `:430` | `utilityclasses.xml:430-430 no longer reads "can only be created using…"; it reads "The method will raise"`, FAILED |
| H | renamed `("ARRAY", "new")` in `CONSTRUCTION` to `("ARRAYZZ", …)` | `the class set the books produce and the swept .environment class entries disagree`, FAILED |

A and B together are the both-directions property, each seen from its own side. D and E together are
D56's rule: absent **fails**, absent-and-marked skips.

---

## Two departures from the brief's stated derivation, both because running it produced wrong rows

The brief and the spec give table D's parser side as *"`DirectiveParser.cpp`'s per-function
`SUBDIRECTIVE_*` arms"* / *"the `SUBDIRECTIVE_*` arms of its own `switch`"*. Applied literally that
produces **five wrong rows**, and I widened it. `src/docs/directives.rs`'s module doc and
`directive-options.txt`'s header both say so, and the `evidence` column records each arm's family
(`SUBDIRECTIVE` / `SUBKEY`) and form (`case` / `comparison`), so the narrower set is recovered by
filtering rather than lost.

1. **`SUBKEY_*` arms.** `::OPTIONS`' `FORM` and `NUMERIC` values reach the parser as `SUBKEY_*`
   (`DirectiveParser.cpp` `:1010`, `:1015`, `:1342`, `:1347`), not `SUBDIRECTIVE_*`. Under the
   literal rule `SCIENTIFIC`, `ENGINEERING`, `INHERIT` and `NOINHERIT` are documented-only and would
   be marked `cross-reference` — four rows saying the directive's own parser has no arm for a keyword
   it plainly handles. It also breaks the spec's own stated purpose for the `position` column: the
   spec says *"the parser's nesting is what settles it"*, and the nesting that settles those four is
   a `SUBKEY_*` switch inside `case SUBDIRECTIVE_FORM:`.
2. **A token tested outside a `switch`.** `::RESOURCE`'s `END` is
   `if (token->subDirective() != SUBDIRECTIVE_END)` at `DirectiveParser.cpp:2295` — not a `case`. And
   `dire.xml`'s `resourced` section documents it only in a railroad SVG and in an example
   (`::resource greyCat end "-"`), with no `<option>END</option>` and no `END subkeyword` indexterm.
   So under the literal rule `::RESOURCE END` gets **no row at all**, though it is both documented
   and implemented. It is now a `parser-only` row with `comparison` in its evidence.

The doc side was cross-checked by an independent `awk` pass over the nine section line ranges: 6, 13,
8, 0, 12, 18, 2, 1 and 3 distinct keyword names for `annotd`…`routd`, and every one appears in the
committed file.

---

## What the extraction found that the plan and the spec do not carry

**1. `::CONSTANT` has no table D row at all.** Neither `dire.xml`'s `constantd` section nor
`constantDirective()` names a subkeyword — correct, since `::CONSTANT` takes a name and a value — so
table D has nothing to probe for that directive. The spec's "nine directive sections" is true, and
the row set covers eight of them.

**2. `dire.xml:1140` files a `LIBRARY subkeyword` indexterm under `::RESOURCE`**, with the secondary
reading `in a RESOURCE directive`. `::RESOURCE` has no `LIBRARY` subkeyword; `::REQUIRES` does
(`DirectiveParser.cpp:2836`). The row comes out `cross-reference` with the evidence naming
`::REQUIRES`, which is the mechanism working — but it is an upstream documentation defect, not the
`::CLASS`/`CLASS` shape the spec describes (a genuine cross-reference in running prose).

**3. Three `mth*` sections document a method their class's own generated table omits.** Derived, and
written into `class-methods.txt`'s header as named exceptions:

* `mthObjectInit` (`fundclasses.xml:2883`) — commented out of `clsObject`'s table with
  `<!-- see new() -->`, live in the unincluded `objectclassmethods.xml`. Measured,
  `.Object~new~hasMethod("INIT")` and `.Object~hasMethod("INIT")` are **both `1`**, so this is a real
  method with no gate row.
* `mthPropertiesNew` (`collclasses.xml:5759`), titled `&changed50;new (Class method)` — `Properties`
  is `covered`, so a `Properties new` row would pass; it is simply not in the table.
* `mthStackFrameExecutable` (`utilityclasses.xml:9488`), titled `executable`.

Making any of them a row needs an `mth<Class>` name-prefix rule, which is exactly the rule the spec
rules out for colliding. **They are named exceptions rather than rows, and that is a decision a
reviewer should look at**, because the alternative reading is that the class table is not quite the
complete denominator the spec takes it for. The other five unreferenced sections are
`RegularExpression`'s, which follows from that class being out of the set.

**4. The four classes ruled `not-covered` by the ledger's finding 10 keep a documented citation.**
`RexxContext` (`:7545`), `RexxInfo` (`:7942`), `StackFrame` (`:9407`) and `VariableReference`
(`:12556`-`:12557`) carry a sentence of the same force as `Buffer`'s and `Pointer`'s but naming a
Rexx-level route, so their reason cites the sentence and their status is `not-covered`. Only `Buffer`
(`:429`) and `Pointer` (`:6910`) are `unreachable`. Each citation is **re-read at the lines it names
on every run**, with the span joined, tag-stripped and whitespace-collapsed first, because a quoted
span may cross a line break and may carry inline markup. (Corrected in fix round 2: this sentence
first said "three of the six sentences wrap across a line break and one carries an `<xref>`", and
neither half holds. Measured over the six cited spans -- the lines each row's own quoted text
occupies, which is the span the row carries and not the book's whole sentence -- **one** crosses a
line break, `VariableReference` at `:12556`-`:12557`, and **one** carries inline markup, the same
span's `<methodname>`; no cited span carries an `<xref>`. Under the other reading, the book's whole
sentence, **four** cross a line break -- `Buffer`, `Pointer`, `RexxInfo`, `VariableReference` --
which is not three either.)

**5. The class-set/hierarchy disagreement is exactly what the spec records.** Asserted rather than
observed: the class-set names with no documented edge are `ArgUtil`, `EventSemaphore`,
`MutexSemaphore`, `Object`, `RexxInfo` and `Singleton`. `Object` and `RexxInfo` are rules 2 and 3;
`ArgUtil`'s member is commented out; the trio is the spec's "documented in a `cls*` section but absent
from the hierarchy list".

**6. `utilityclasses.xml:4809` has a `<section>` with no `id`** (`<title>Examples</title>`). It is
harmless here — no `cls*`/`mth*` extraction keys on it — but a depth-tracking scanner is needed
rather than a per-line pattern, and `provide.rs` panics on an id-less section inside its own chapter.

**7. `62 / 38 / 24` reproduced a third time.** Re-run for this task on the current oracle: 62
`.environment` entries answering `~isA(.Class)`, 38 constructing on a bare `~new`, 24 raising
(`93.901` ×12, `93.967` ×5, `93.903` ×3, `88.901` ×3, `97.1` ×1). The full list is committed as
`CONSTRUCTION` in `src/docs/classes.rs`, and the extractor asserts it is exactly the class set the
books produce (control H).

---

## What I could not check

* **The `covered` sweep is a measurement in this report, not a standing test.** Making it standing
  needs the bounded oracle harness Task 1 landed, which lives in `crates/rexx-exec/tests/support/`;
  `rexx-extract` has no oracle support and `oracle_root()` is a hard-coded absolute path that would
  become a second copy in a second crate. Table C is Task 5's, it is in the crate that has the
  harness, and that is where this belongs. **Until then, a row set regenerated by a later hand could
  reintroduce a neither-arm row and nothing would catch it.**
* **The both-directions check cannot see a regeneration** — both sides moving in one commit. That is
  a diff for a human, deliberately, and it is why these files are reviewed as artifacts.
* **None of CI's five platforms has `oodocs/`**, so every check in `tests/extract_docs.rs` that reads
  it fires on one developer machine per gate. The four that read only the committed files run
  everywhere.
* **`CONSTRUCTION` is not re-measured by anything.** The spec rules that a class's membership of the
  constructible set is not re-checked by the gate tables. What *is* checked is that its names are
  exactly the class set the books produce, which catches the books and the image drifting apart but
  not a `~new` outcome changing.
* **The `arm` column is not policed.** The sweep accepts either arm by design, so a row classified
  `instance` that is really class-side passes. Nothing here would catch it; `.Class`'s rows are the
  place I would look first, since `Class`'s instance methods are every class's class methods.
* **`mth*` titles were read through the extractor's own pattern**, not by eye. A `mth*` section whose
  `<title>` is not the first element in its body would be invisible to it; the extractor asserts
  `title_is_immediate` for every `mth*` section it resolves, so such a section would panic rather
  than pass silently — but a section that no class table names is not resolved at all and would only
  appear in the derived named-exception list.
* **I did not run `REXX_PHASE_GATE=5a …`.** Re-checked for this task:
  `/bin/grep -arn 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/crates/` **exits 1**, so neither exists in
  the tree and the command would exit 0 for a reason unrelated to what it checks. That is Task 24's.

---

## Gates

All five run from `rust/` **on the final tree at `fd8cb5cd0`**, each status read on its own and
unpiped.

```
cargo fmt --all --check                                          exit 0
cargo clippy --workspace --all-targets -- -D warnings            exit 0
cargo test --release --workspace                                 exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace              exit 0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  exit 0   106 of 106 matching
```

`memcap` is present at `/home/moritz/.local/bin/memcap`, checked with `command -v` rather than
assumed. The last two commands have **zero** `test result: FAILED` lines each.

Every measured claim written into a committed header was re-run for this task rather than inherited:
`.Array~hasMethod("OF")` `1` against `.Array~new~hasMethod("OF")` `0`;
`.Alarm~hasMethod("CANCEL")`, `("TRIGGERED")` and `("SCHEDULEDTIME")` all `0` with `.Alarm~new`
raising `93.901`; `::requires "rxregexp.cls"` rc 213 with
`Error 43.901: Could not find file "rxregexp.cls" for ::REQUIRES`; and `.Object~method("")` and
`.Object~method(" ")` both `The Method class`.

Clippy needed three fixes on the way: `needless_lifetimes` on `class_sections`, `manual_strip` in the
C++ scanner, and `err_expect` in the test.

**No benchmark sitting, and the reason is structural rather than a judgement.** The changes are
`rust/corpus/docs/` (new data), `crates/rexx-extract/src/`, `crates/rexx-extract/tests/`. `rexx-extract`
is not `rexx-exec`, `rexx-core`, `rexx-classes` or `rexx-lib`, and nothing outside it changed except
one `//!` paragraph in `rexx-extract/src/lib.rs`. The release binary the axes measure is
byte-identical, so a sitting would measure noise.

**No corpus program was added**, so `rust/corpus/phase-5a.txt` is untouched and the corpus stays 106.

---

# Fix round 1

Base `fd8cb5cd0`, one commit: `c0fda0a4438c737c446a2335dbc410c94d75e665`. One medium, four low, one
nit; F5 needed nothing from me and the controller closed its plan half at `c36f52881`.

**New `class-methods.txt` row count: 1347**, up from 1345. The two rows added are

```
DateTime	new	class	covered	mthDateTimeInit	utilityclasses.xml:1775
TimeSpan	new	class	not-covered	mthTimeSpanInit	utilityclasses.xml:10700
```

`covered` rows go 793 → **794**; the `covered` sweep re-run over the regenerated file is 37 classes,
37 oracle runs at rc 0 with empty stderr, **neither-arm set empty**.

## F1 — MEDIUM. `xrefstyle` is now read, the two missing rows are emitted, and the rule is guarded

**Reproduced first.** The reviewer's pattern, run verbatim over the four books plus every
`*classmethods.xml` with comments blanked, groups by the part before the first `:` as **`select`
1168, `template` 5** — the same five members at the same lines. Separately: **every** `<member>`
carrying an `mth*` `<xref>` has an `xrefstyle`, so there is no third case.

**One correction to the review's account, which does not change the finding.** Three of the five come
out right, but not all for the reason given. `mthTraceObjectNew` (`:11472`) and
`mthTraceObjectNotifySet` (`:11492`) are **not class-table members at all** — they are `<member>`
elements used as wrappers around an `<xref>` in running prose, inside a `<note>` inside a `<para>`
inside the `mthTraceObjectCollectorSet` section, well past `clsTraceObject`'s head (`:11301`-`:11395`).
The extractor reads only a class section's head, so it never saw them and their titles never came into
it. Their `select:title` twins in the actual class table are what produced `TraceObject new class` and
`TraceObject notify= class`. So the three that "come out right" are: one because its template and its
title agree (`mthRexxQueueNew`), two because they are never read.

**That matters for the exception list.** `TEMPLATE_MEMBERS` holds the three that *are* class-table
members — `mthDateTimeInit`, `mthRexxQueueNew`, `mthTimeSpanInit` — and not the two prose ones. If
upstream ever moves one of those into a table, the assertion fires and a person decides, which is the
behaviour wanted.

**What the fix does.** `xml::Member` gains an `xrefstyle` field, read off the same `<xref>` as the
`linkend`. `classes::displayed_names` is the new seam: a member whose style starts `select:title`
yields the target's title, and a member in `TEMPLATE_MEMBERS` yields **both** the title's name and the
displayed one. Both are right — measured on the oracle, `.DateTime~hasMethod("NEW")` is `1` with
`.DateTime~new~hasMethod("NEW")` `0`, `.DateTime~hasMethod("INIT")` and `.DateTime~new~hasMethod("INIT")`
are both `1`, `.TimeSpan~hasMethod("NEW")` is `1`, and `.RexxQueue~hasMethod("NEW")` is `1` with the
instance arm `0`. `RexxQueue`'s template and title agree, so its row deduplicates to the one that was
already there.

**Anything else panics**, and the displayed text is part of the key, so an upstream edit to it is a
different member rather than a silent re-read of the old one.

**Seen firing on the production path over the real book, not only on a sample:**

| control | what I changed | what happened |
|---|---|---|
| I | deleted `mthDateTimeInit` from `TEMPLATE_MEMBERS` | `utilityclasses.xml names mthDateTimeInit with xrefstyle "template:new (Inherited Class Method)", which neither displays the target section's own title nor is a member this extractor has decided about…`, FAILED |
| J | changed `mthTimeSpanInit`'s expected text to `template:new (Class Method)` | same panic naming `mthTimeSpanInit`, FAILED |

Four unit tests cover the synthetic cases: an ordinary member, a `template:` member yielding both
names with the right arms, an unrecognised style, and a member with no style at all.

**Both-directions re-run on the regenerated file, in both directions and on the new header list:**

| control | what I changed | what happened |
|---|---|---|
| A | deleted the new `DateTime new class` row | `rows derived and not committed (1)`, naming that row, FAILED |
| B | appended an invented `DateTime zznosuch` row | `rows committed and no longer derived (1)`, FAILED |
| C | edited the new derived list's `ArgUtil` line in the header | FAILED, with both row lists empty and the message saying the header moved |

**A second measurement worth recording, because the report's own caveat was wider than the truth.**
For every one of the 794 `covered` rows where exactly **one** arm answers, the declared `arm` is that
arm — **0 exceptions**. The 64 rows answering on both arms are unpinned by construction. The
reviewer found the same on 793 rows independently. So "the `arm` column is not policed" stands as
written — nothing *checks* it — but within the `covered` set it is, measured, correct.

## F2 — LOW. `hashCode` recorded, and why it is not a derived list

`fundclasses.xml:102` comments out `<member><xref linkend="mthClassHashCode"/>` with the reason
*"won't document hashCode"*; no `mthClassHashCode` section exists anywhere; measured,
`.Class~hasMethod("HASHCODE")` is `1`. So the name is in neither the row set nor the derived
unreferenced-section list — the `ArgUtil` shape one level down, exactly as the ledger's finding 11
says. **`Class` is `not-covered`, so a row would be gated on nothing, and no row is added.**

**I built the derived version of this and rejected it, and the measurement is the reason.** Class-table
members that sit inside an XML comment, over the four books: **41**, of which **38** have no live
`mth*` section anywhere. Several of those 38 are alternate ids for methods that *are* documented under
a slashed title — `mthStringCentre` beside `center/centre`, `mthAlarmCancelled` and
`mthTickerCancelled` beside `canceled/cancelled` — and nine are `mthStream!*` internal entry points.
A header list captioned "these produce no row" would be true and would read to a reasonable person as
"these methods are undocumented", which is false for at least three of its members. **A derived list
whose caption cannot be made true of every row is worse than the sentence it replaces**, so this stays
a recorded decision here rather than an artifact.

## F3 — LOW. The explanation is now inside the committed artifact, and derived

`class-methods.txt`'s header gains a **third derived list**: classes `class-set.txt` carries that have
no row in it at all, each with why its method set is empty. Today it is one line —
`ArgUtil -- the books document it nowhere; its only citation is provide.xml`. Derived rather than
written, so it is policed in both directions like every other header line (control C above), and so a
second such class appearing later cannot arrive unexplained.

## F4 — LOW. Both statements corrected

* The `objectclassmethods.xml` evidence. **"finds it only in the file itself, exit 1 for everything
  else" is wrong on both halves**: re-run, `/bin/grep -arn 'objectclassmethods' oodocs/rexxref/en-US/`
  prints nothing and exits 1 — the string occurs in no file at all, the file itself included, and one
  `grep` invocation has one exit status, not one per file. The conclusion holds by the reviewer's
  better route, which I re-ran: **16** `*classmethods.xml` files exist in `rexxref/en-US`,
  `/bin/grep -arho 'href="[a-z]*classmethods.xml"'` over the four books finds **55** occurrences of
  **15** distinct hrefs, and `objectclassmethods.xml` is the one absent — which is also what the
  extractor's derived list (1) says.
* CI. **"The four that read only the committed files run everywhere" is wrong.** Re-run:
  `/bin/grep -arn 'cargo\|rust' .github/workflows/` matches only two `svn checkout` lines, so **CI runs
  no `cargo` at all** and **none** of `extract_docs.rs`'s tests runs on any CI platform. The correct
  statement is the one the test's own module doc already makes: these checks fire on a developer
  machine, not continuously. The four that need no `oodocs/` would run wherever the suite runs; nothing
  runs the suite in CI.

## F6 — NIT. The two subset counts are gone

`UNCONSTRUCTIBLE`'s doc no longer says "the two rows" and "the other four"; it states the property —
which sentence grounds `unreachable` and which grounds `not-covered` — and points at each row's own
`Status`, which is the enumeration and cannot rot. `unconstructible_index`'s doc no longer says "three
of the six sentences wrap". (The replacement text was itself corrected in fix round 2 -- it named
`<xref>` as the mid-sentence tag, which no cited span carries.) The rest of the crate's pre-existing
counts are untouched, as ruled.

## Verification

Five gates from `rust/`, each status read on its own, unpiped:

```
cargo fmt --all --check                                             exit 0
cargo clippy --workspace --all-targets -- -D warnings               exit 0
cargo test --release --workspace                                    exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 exit 0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  exit 0   106 of 106 matching
```

`rexx-extract`'s own suite is 32 unit tests plus 7 integration tests, all passing. All five row sets
re-derive byte-identically (`--check` exit 0). `REXX_PHASE_GATE=5a` stays unrun:
`/bin/grep -arn 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/crates/` still exits 1.

**No sitting.** The change touches `rust/corpus/docs/` and `crates/rexx-extract/{src,tests}` only;
`rexx-extract` is none of the four measured crates.

## What I could not close

* **The `covered` sweep is still a measurement in this report, not a standing test**, for the reason
  the first round gave: the bounded oracle harness lives in `rexx-exec`'s test support behind a
  hard-coded `oracle_root()`, and Table C is Task 5's, in that crate. Unchanged by this round.
* **The `xrefstyle` guard covers class-table members only.** A `template:` `<member>` in running prose
  — the two `TraceObject` ones — is outside every span this extractor reads, so the guard says nothing
  about it. That is correct today and would stop being correct if the book moved one into a table,
  which is the case the assertion is for.

---

# Fix round 2

Base `c0fda0a44`. Two commits:

* `658514a42232c34a9dfbbd45d4a0868d0365738c` -- N1, N2 and N3.
* `094fbbdd87ff955f3dd61003cfb1c1f8aa4f3388` -- two horizontal-ellipsis characters that went
  into the new test's doc comment; this tree writes ASCII.

One low, two nits.

**Counts in this section, with their predicate stated.** Every figure below is **data rows** — lines
that are neither a `#` comment nor blank. `class-methods.txt` is 1432 total lines = 84 comment
lines + 1 blank line + **1347 data rows**, unchanged by this round. The five files are 21, 57, 63,
1347 and 79 data rows.

## N2 — the artifact now carries the rule that explains its own strangest rows

Two changes, and the second is the one the re-review asked for by name.

**`origin` now cites where the row's *name* came from**, which for a row whose class-table member
overrides the displayed title is the member, not the section:

```
DateTime	init	instance	covered	mthDateTimeInit	utilityclasses.xml:1775
DateTime	new	class	covered	mthDateTimeInit	utilityclasses.xml:1281
TimeSpan	init	instance	not-covered	mthTimeSpanInit	utilityclasses.xml:10700
TimeSpan	new	class	not-covered	mthTimeSpanInit	utilityclasses.xml:10492
```

`:1281` and `:10492` are the `<member><xref … xrefstyle="template:new (Inherited Class Method)"/>`
lines the re-review named. Following `DateTime new`'s citation now lands on the text that displays
`new`. `section` still names the section that *documents* the method, and the header says so.

**The header states the override rule**, so `/bin/grep -a 'xrefstyle'` over the committed file now
matches: what `select:title` and `template:<text>` do, that an override yields both names, that the
override row cites the member, `clsDateTime` worked through with its two line numbers and the oracle
measurements, and that an unrecognised style is a hard error rather than a fallback.

**A defect I introduced and the check that now catches it.** My first version took the member's line
straight from `members(head)` — but `head` is a *slice* of the book, so the line counted from the
section's opening tag: `DateTime new` came out citing `utilityclasses.xml:48`. I caught it by reading
the output, which is not a control. So `extract_docs.rs` gains
`every_method_rows_origin_line_names_its_section`: every row's `origin` line must contain the row's
`section` id, which holds for a `<section id="mth…">` line and for a `<member><xref linkend="mth…">`
line alike, so one assertion covers both kinds. **Seen firing**: with the offset restored to `0` it
reports

```
the row citing utilityclasses.xml:48 names mthDateTimeInit, and that line reads
"  <xi:include href=\"utilityclassesintro.xml\" …/>"
```

and FAILED. Restored, 8 of 8 integration tests pass.

**Both-directions re-run**, all three arms: a row removed → `rows derived and not committed (1)`; a
row added → `rows committed and no longer derived (1)`; a line of the new header prose edited →
FAILED with both row lists empty and the message naming the header.

## N1 — LOW. The comment is right and the report is brought along

**The measurement first**, since both prose versions were written without one. Over the six cited
spans — a span being **the lines that row's own quoted text occupies**, which is what the row carries
and is not the book's whole sentence:

* **one** crosses a line break: `VariableReference`, `:12556`-`:12557`;
* **one** carries inline markup, the same span's `<methodname>`; **no cited span carries an
  `<xref>`**;
* under the other reading — the book's whole sentence — **four** cross a line break (`Buffer`,
  `Pointer`, `RexxInfo`, `VariableReference`).

Three is neither. `Buffer` is the case that separates the two readings: its sentence runs
`:428`-`:429` and the row quotes the clause on `:429`, so the row cites `:429`.

`unconstructible_index`'s doc now states the span rule first — a row's span is the lines its own
quoted text occupies, and a span written as a range is one whose quotation crosses a line break — and
names `<methodname>` as the markup a quotation may carry. **The report's two copies of the deleted
count are corrected in place**, at the "what the extraction found" entry and at fix round 1's F6
entry, each saying what the sentence used to claim and what measures instead.

## N3 — NIT. The derived-list entry cites `file:line`

`classes_without_method_rows` had the line in the same `ClassRow` and did not use it. Both arms now
cite it:

```
ArgUtil  --  the books document it nowhere; its only citation is provide.xml:838
```

## Verification

All five row sets re-derive byte-identically (`--check`, exit 0). Five gates from `rust/` **on the
final tree at `094fbbdd8`**, each status read on its own, unpiped:

```
cargo fmt --all --check                                             exit 0
cargo clippy --workspace --all-targets -- -D warnings               exit 0
cargo test --release --workspace                                    exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 exit 0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  exit 0   106 of 106 matching
```

`rexx-extract`'s own suite is 32 unit tests and 8 integration tests. `REXX_PHASE_GATE=5a` stays
unrun. No sitting: `rust/corpus/docs/` and `crates/rexx-extract/{src,tests}` only.

## What I could not close

Nothing new. The two standing items are unchanged: the `covered` sweep is a measurement in this
report rather than a standing test, because the bounded oracle harness is in `rexx-exec` behind a
hard-coded `oracle_root()` and table C is Task 5's; and the `xrefstyle` guard covers class-table
members only, so a `template:` `<member>` used as prose markup — the two `TraceObject` ones — is
outside every span this extractor reads.
