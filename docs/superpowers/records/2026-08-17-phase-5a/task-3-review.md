# Task 3 review — the extractors, and the committed row sets

Scope `68a44aa1c..fd8cb5cd0`, two commits, reviewed at `fd8cb5cd0` with a clean working tree.

**Verdicts.**

* **Spec compliance: APPROVE.** Every hard case the brief enumerates is a named branch with a unit
  test, and each one I re-derived independently from `oodocs` agrees with what the code does.
* **Task quality: REWORK**, on one finding. The row sets are right everywhere I could push them —
  every derived count reproduces from an independent pass, the oracle sweeps reproduce row for row,
  and about sixty sampled rows agree with the book at the line they cite. But the extractor reads a
  class table's `<xref>` without reading its `xrefstyle`, and the book overrides the displayed name
  on five members. Two of those five are the documented constructor entries of `DateTime` and
  `TimeSpan`, and the rows they should have produced are absent from the method denominator.

---

## Findings, most severe first

### F1 — Medium. `xrefstyle="template:…"` is never read, so two documented class-table entries produce no row, and nothing can see it

`classes.rs`'s `method_rows` keys a method to a class by the class table's `<member><xref
linkend="mth…">` entries and then takes the name from that section's `<title>`. The book does not
always display the section's title. Five class-table members carry
`xrefstyle="template:<text>"`, which replaces the rendered name outright:

```
utilityclasses.xml:1281   mthDateTimeInit          template:new (Inherited Class Method)
utilityclasses.xml:8838   mthRexxQueueNew          template:new (Inherited Class Method)
utilityclasses.xml:10492  mthTimeSpanInit          template:new (Inherited Class Method)
utilityclasses.xml:11472  mthTraceObjectNew        template:new
utilityclasses.xml:11492  mthTraceObjectNotifySet  template:NOTIFY
```

**The pattern, so the negative is checkable.** `<member>\s*<xref linkend="(mth[^"]*)"\s*xrefstyle="([^"]*)"`
over the four books plus every `*classmethods.xml`, comments blanked, grouped by the part before the
first `:` — `select` 1168, `template` 5. Those are the five.

Three of the five happen to come out right anyway: `mthTraceObjectNew`'s title is
`&added51;new (Class Method)` and `mthTraceObjectNotifySet`'s is `&added52;notify= (Class Method)`
(measured, `.TraceObject~hasMethod("NOTIFY=")` and `("NOTIFY")` are both `1`), and `mthRexxQueueNew`
is one of the two names in `BARE_NEW_CLASS_SIDE`. Two do not:

* `DateTime`'s class table names its constructor as `new (Inherited Class Method)` pointing at
  `mthDateTimeInit`. The row set carries `DateTime init instance` and **no** `DateTime new class`.
* `TimeSpan` is the same shape at `:10492`.

Measured on the oracle from a fresh empty directory: `.DateTime~hasMethod("NEW")` is `1`,
`.DateTime~new~hasMethod("INIT")` is `1`, `.TimeSpan~hasMethod("NEW")` is `1`. So the emitted row is
**true** — this is a missing row, not a wrong one.

**Concrete consequence.** Table C is Task 5's and its denominator is this file. `DateTime` is
`covered`, so its rows are the ones the gate actually probes — and the entry the book's own class
table displays as DateTime's constructor is not among them. That is precisely the failure the
both-directions check cannot see: it compares the extractor with itself, so the row was never
derived, has no arm to disagree about, and the file and the derivation agree perfectly on its
absence. This is the same shape as rule 1 of the hierarchy derivation, which the task *did* guard —
an extractor that misreads the book in a direction the oracle confirms or ignores. Rule 1 got an
assertion; this rule got none, and if upstream adds a sixth `template:` member the row set drifts
from the book with every check still green.

**What would close it**, in the shape the task already uses elsewhere: assert in `method_rows` that
every class-table member's `xrefstyle` begins `select:title`, with a named exception list for the
`template:` members whose displayed name the task has decided about — the `ArgUtil` assertion's
pattern, applied to the one remaining unguarded rule.

### F2 — Low. Ledger finding 11 (`hashCode`) is addressed nowhere

Task 2's ledger, an input this task was told to treat as one, ends its finding 11 with *"The
class-set criterion has an explicit `ArgUtil` row for the class-level case; the method half has the
same case and no equivalent, and the both-directions check cannot see it."* `fundclasses.xml:102`
comments out `<member><xref linkend="mthClassHashCode"/>` with the reason *"won't document
hashCode"*, and there is no `mthClassHashCode` section anywhere — so the name appears in neither the
row set nor the derived unreferenced-section list. Measured, `.Class~hasMethod("HASHCODE")` is `1`.

Neither the report, the code, nor `class-methods.txt`'s header mentions it.

**Consequence, and it is bounded.** `Class` is `not-covered`, so a `Class hashCode` row would have
been gated on nothing; the gap costs no coverage today. What it costs is a decision: the next reader
of this row set who reaches the ledger has to re-derive the question from scratch, and the report's
own section 3 ("what the extraction found that the plan and the spec do not carry") is the place a
reader would expect to find it already answered — it lists three near-identical cases and not this
one. Recording the ruling is the whole fix.

### F3 — Low. `ArgUtil` is the one class in `class-set.txt` with no rows in `class-methods.txt`, and neither artifact says so

This is the 38-versus-37 gap the brief asks about. **It is correct and it is explained**: the
`covered` set is 38 classes; `ArgUtil` is `covered` (`.ArgUtil` is `The ArgUtil class` and a bare
`~new` constructs it) but the books document it nowhere, so it has no `mth*` sections and no method
rows, leaving 37 classes for the sweep. `comm` over the two files confirms `ArgUtil` is the only
such name. Nothing went missing.

But the explanation lives only in the task report, and `.superpowers/` is git-ignored — the records
are copied to `docs/superpowers/records/` when the plan closes, not now. `class-methods.txt`'s header
explains `RegularExpression`'s absence and says nothing about `ArgUtil`, and
`every_method_rows_class_and_status_come_from_the_class_set` only checks methods ⊆ class-set, never
the reverse.

**Consequence.** A later task diffing the two committed artifacts finds a `covered` class with zero
method rows and no stated reason, and either files it as a defect or "fixes" it. One header line —
or the same derived-list treatment `objectclassmethods.xml` already gets — settles it inside the
artifact.

### F4 — Low. Two statements in the report do not reproduce

Both conclusions are right; the stated evidence is not.

* *"`/bin/grep -arn 'objectclassmethods' oodocs/rexxref/en-US/` finds it only in the file itself,
  exit 1 for everything else."* Re-run, that command prints **nothing** and exits 1 — the string
  occurs in no file, `objectclassmethods.xml` included, and one grep invocation has one exit status.
  The conclusion holds and I confirmed it another way: 16 `*classmethods.xml` files exist in
  `rexxref/en-US`, `/bin/grep -arho 'href="[a-z]*classmethods.xml"'` finds 15 distinct hrefs, and
  `objectclassmethods.xml` is the one absent — which is also what the derived header list says.
* *"The four that read only the committed files run everywhere."* CI runs no `cargo` at all:
  `/bin/grep -arn 'cargo\|rust' .github/workflows/` matches only two `svn checkout … --trust-server-cert`
  lines, and all three workflows check out `ootest` and nothing else. None of the seven tests in
  `extract_docs.rs` runs on any CI platform, not four of them. (The test's own module doc says
  "none of CI's five platforms has it", which is true and is the claim that matters.)

### F5 — Low. The spec and the plan still carry the narrow parser-side rule the task departed from

`docs/superpowers/specs/2026-08-17-phase-5-object-model.md:857` still reads *"the `SUBDIRECTIVE_*`
arms of its own `switch`"*, and the plan repeats it at `:618`.

**Both departures are correct** — I checked each against the C++ rather than against the report:

* `DirectiveParser.cpp:999` `case SUBDIRECTIVE_FORM:` opens a `switch (token->subKeyword())` whose
  arms are `SUBKEY_SCIENTIFIC` (`:1010`) and `SUBKEY_ENGINEERING` (`:1015`); `:1331`
  `case SUBDIRECTIVE_NUMERIC:` likewise gives `SUBKEY_NOINHERIT` (`:1342`) and `SUBKEY_INHERIT`
  (`:1347`). Under the literal rule those four documented, implemented keywords come out
  `cross-reference` with evidence reading "no directive function names SUBDIRECTIVE_SCIENTIFIC" —
  four false rows, and it breaks the spec's own stated purpose for the `position` column.
* `DirectiveParser.cpp:2295` is `if (token->subDirective() != SUBDIRECTIVE_END)`, not a `case`, and
  `dire.xml`'s `resourced` section has no `<option>END</option>` and no `END subkeyword` indexterm
  (confirmed by my own pass: that section's whole documented set is `{LIBRARY}`). Under the literal
  rule `::RESOURCE END` gets no row at all.

**And they are recorded where a reader will look**: `directives.rs`'s module doc, the committed
`directive-options.txt` header, the commit message, and the report — four places, one of them the
artifact itself, and the `evidence` column records each arm's family and form so the narrower set is
recoverable by filtering. That is the right standard.

**Consequence of the residue.** A Task 4 reviewer checking table D against the spec's stated rule
finds a mismatch the committed artifact explains and the spec does not. This project has a name for
that shape; the cheap answer is a line in the plan pointing at the header.

### F6 — Nit. Two comments count subsets of a table a later task will edit

`classes.rs:188-189`: *"`unreachable` belongs to the **two** rows whose sentence says instances come
only from native code … The other **four** say the user cannot construct one"*, above
`UNCONSTRUCTIBLE`; and `:393-394`, *"**three** of the **six** sentences wrap across a line break"*.
Most of the counts in these files sit immediately above the set they count and name it — "the four
class books" above `CLASS_BOOKS`, "the three row statuses" above `Status`, "the nine directive
functions" above `DIRECTIVES` — which is what the constraint asks for. These two count *subsets by
status* of a table whose whole point is that a later task may add a row to it, and adding a seventh
falsifies both sentences while the code stays right. The rest of the crate's pre-existing comments
use counts the same way, so this is the tree's practice rather than a deviation this task
introduced.

---

## What I re-derived, and what I sampled against the source

**Re-derived in full — the extractors' own output.** `rexx-extract-docs --check` reports all five
unchanged, exit 0. Re-run writing to a scratch directory and `cmp`'d against the committed copies:
**5 of 5 byte-identical**.

**Re-derived in full — independently of their code.** Each of these is a pass I wrote, over
`oodocs` at r13198 and `interpreter/parser/DirectiveParser.cpp`, not a re-run of theirs:

| what | independent result | committed |
|---|---|---|
| `<section id>` strictly inside `provide.xml`'s `provide` chapter (lines 46–1047), none id-less | 21 | 21 rows |
| `<member>` lines in the `chi` `<simplelist>` (840–903); comments inside that span | 60 lines, exactly one comment and it is the `ArgUtil` member at 843–845 | 60 − 1 commented − `Object` (887) − `RexxInfo` (893) = 57 rows |
| `<section id="cls…">` per book, and anywhere outside the four books | 7 / 17 / 35 / 4, none elsewhere | 63 − `RegularExpression` + `ArgUtil` = 63 rows |
| `mth*` sections with an immediate `<title>`, comments blanked | 1012 (raw 250/346/**372**/45, the difference being `mthSupplierInit` inside the comment opened at `utilityclasses.xml:10061`) | 1004 distinct sections referenced + 8 named exceptions in the header = 1012 |
| distinct trailing parentheticals over all `mth*` titles | `(Class Method)` 118, `(Class method)` 3, `(Abstract Method)` 24, `(Abstract method)` 1, `(Attribute)` 3, `(Private Method)` 3, `(Inherited Class Method)` 1, `(inline if)` 1 — nothing else | `TITLE_PARENTHETICALS` covers exactly those, case-insensitively |
| `mth*` titles that are bare `new` | `mthClassNew`, `mthRexxQueueNew` | `BARE_NEW_CLASS_SIDE` is exactly those two |
| `mth*` titles ending in `Methods` (group-heading shape) | `Comparison` 5, `Arithmetic` 3, `Concatenation` 2, `Logical` 1 — nothing else | `GROUP_HEADINGS` is exactly those four |
| `mth*` titles containing `/` | `center/centre`, `canceled/cancelled` ×2, `delete / delStr` | all four split |
| members with text after the `<xref/>` that are **not** group headings | 0 | the trailing text is read only for group headings |
| `<member `, `<option `, `<primary ` with attributes (which the text-level scanner would skip) | none anywhere | — |
| `dire.xml` documented names per section (`<option>` ∪ `NAME subkeyword` primaries), all 45 primaries inside the nine sections | 6 / 13 / 8 / 0 / 12 / 18 / 2 / 1 / 3 | matches the report's independent `awk` pass and the committed rows |
| `SUBDIRECTIVE_*`/`SUBKEY_*` tokens per directive function, comments and literals stripped | 6 / 13 / 7 / 0 / 12 / 33 / 2 / 1 / 3 = 77, exactly one not a `case` (`SUBDIRECTIVE_END`, `:2295`); all 77 tokens in the file are inside the nine functions | 77 parser rows + 2 `cross-reference` = 79 rows |
| the reference's native-code sentence, searched over all four books **and** `provide.xml` | exactly 2 occurrences: `utilityclasses.xml:429` and `:6910` | `unreachable` is exactly `Buffer` and `Pointer` |
| other cannot-construct sentences (`cannot be directly created`, `cannot be created`, `not allowed`, `created by the user`, `cannot be instantiated`, `are not created directly`, plus a `new method` × negation sweep) | `:7545`, `:7942`, `:9407`, `:12557` — the four the ledger's finding 10 names, and nothing else | all four `not-covered` with their sentence cited, none `unreachable` |

**Re-derived in full — the oracle measurements.** All from a fresh empty directory, absolute paths,
three descriptors read separately, under the wrapper.

* **The `.environment` construction sweep**, re-run with my own program (`signal on syntax` in a
  `::routine`, not inside the loop): **62** class entries, **38** constructing, **24** raising, rc 0,
  empty stderr, well inside the bound. Compared entry by entry against `CONSTRUCTION`: same 62 names,
  **zero outcome disagreements**, same code histogram (`93.901` ×12, `93.967` ×5, `93.903` ×3,
  `88.901` ×3, `97.1` ×1). This is the one input nothing re-checks, and it holds.
* **The `covered` sweep**, regenerated from the committed file rather than from their generator
  (placeholders mapped back to `""` and `" "`): 37 classes, **793 rows**, 37 runs at rc 0 with empty
  stderr. **both arms 64, class arm only 75, instance arm only 654, neither arm 0.** Identical to the
  report's table.
* **The arm column, which the report lists as unpoliced.** For every one of the 793 rows where
  exactly one arm answers, the declared `arm` **is** the arm that answers — 0 exceptions. The 64
  rows answering on both arms are unpinned by construction. So the caveat is narrower than the report
  states, for the `covered` set.
* **The class-arm sweep over the other 25 classes**, 552 rows: **180 answered `1`, 372 answered `0`,
  and 0 of the zeros is a `class`-arm row.** Identical to the report. (125 of the 180 are
  `instance`-arm rows that also answer on the class arm, which is the class object carrying `Object`'s
  instance methods — expected, and not an arm error.)
* Header claims re-measured: `.Alarm~hasMethod("CANCEL")`, `("TRIGGERED")`, `("SCHEDULEDTIME")` all
  `0`; `.Array~hasMethod("OF")` `1` against `.Array~new~hasMethod("OF")` `0`; `.RexxInfo` prints
  `a RexxInfo` and `~isA(.Class)` is `0`; `.ArgUtil` prints `The ArgUtil class`;
  `::requires "rxregexp.cls"` is rc 213 with `Error 43.901: Could not find file "rxregexp.cls" for
  ::REQUIRES`.

**Sampled against `oodocs` at r13198.** Every sampled row was checked by opening the file at the line
the row cites.

* `provide-sections.txt` — 9 rows: `typcla` 75, `objcla` 84 (depth 1, parent `typcla`), `xmixin` 105,
  `xmeths` 483, `unkno` 526, `reqstr` 723, `classmeth` 824, `chi` 828 (depth 1), `methodsbyclass` 1006.
* `hierarchy-edges.txt` — the whole 840–903 block read in the source. Specifically the nesting chain
  `Collection` 848 → `MapCollection` 849 (5 `&nbsp;`) → `Directory` 851 (10) → `Properties` 852 (15),
  and `StringTable` 857 → `TraceObject` 858; both dropped members (`clsObject` 887, `clsRexxInfo`
  893); the commented `clsArgUtil` member at 844 and its reason comment at 838.
* `class-set.txt` — 12 rows drawn at random, each verified to sit at `<section id="cls…">` on the
  cited line (`Comparator` 566, `Validate` 12201, `NumericComparator` 1139, `TraceObject` 11301,
  `StackFrame` 9396, `CaselessComparator` 639, `DescendingComparator` 888, `InputStream` 103,
  `EventSemaphore` 3142, `Bag` 2566, `Array` 1267, and `ArgUtil`'s comment citation at
  `provide.xml:838`), plus all six `UNCONSTRUCTIBLE` sentences located by search rather than by
  following the citation.
* `class-methods.txt` — **25 rows drawn at random**, each verified twice: the `<section id>` is on
  the cited line, and the method name is derivable from that section's title under the stated rules
  (0 disagreements; the sample included entity-prefixed titles, `(Abstract Method)`, `(Class Method)`,
  and four operator rows from three different group headings). Plus, read in full against the book:
  `Object`'s 32 rows against `clsObject`'s table at `fundclasses.xml:2519` (including `mthObjectNew`
  titled `new (Class Method)` → the `class` arm, `mthObjectInit` commented out with `<!-- see new() -->`
  → no row, and `(abuttal) || (blank)` at `:2549` → `""`, `||`, `" "`); `Supplier`'s 8 rows against a
  table whose `mthSupplierInit` member *and* section are both commented out; `TraceObject`'s 16 rows;
  the 8 `(class, method)` pairs carrying both arms, of which I traced `String lower/space/upper` and
  `File separator` to their separate `*ClsMth` sections; `SetCollection` 19 and `InputOutputStream` 11,
  the two classes whose own table prints `(no class or instance methods)` and whose rows come from
  the mixin sets they `xi:include`; `Array`'s 7 `mthCollection*` rows; and `String` drawing **0** of
  its 135 rows from an `mthStringTable*` section against `StringTable`'s 22.
* `directive-options.txt` — **14 rows drawn at random**, each verified at both cited lines (the C++
  line carries that token and its `case`/`comparison` form, and the `dire.xml` line carries that
  `<option>` or `NAME subkeyword` primary; 0 disagreements). Plus, read in full in the source: the
  `::OPTIONS` `NOVALUE`/`ERROR` nesting at `:1081`–`:1135`, the `FORM` and `NUMERIC` `SUBKEY` blocks,
  `::RESOURCE`'s `END` `if` at `:2295`, and the `LIBRARY subkeyword` indexterm filed under
  `resourced` at `dire.xml:1140` whose secondary reads "in a RESOURCE directive".

**Controls run.** All in a faithful relocated copy of the crate — same sources, same test file, only
the paths moved — since the repository's tracked files are read-only for this review. The copy is
green before and after every control.

| control | what I changed | what happened |
|---|---|---|
| A × 5 | deleted the last data row of **each** of the five files, one at a time | each reddened: `rows derived and not committed (1)` / `…no longer derived (0)`, FAILED |
| B × 5 | appended an invented `zznosuch` row to **each** of the five, one at a time | each reddened the other way: `derived and not committed (0)` / `committed and no longer derived (1)`, FAILED |
| C | rewrote `class-set.txt`'s stamp `r13198` → `r13100` | FAILED, naming `svn info oodocs/rexxref` and refusing the hand edit |
| D | moved `oodocs/` aside | FAILED, printing the `svn checkout` line and `REXX_DOCS_LESS=1` |
| E | the same, with `REXX_DOCS_LESS=1` | `7 passed` — the three docs-reading tests return early |
| F | **removed the comment blanking from the production `derive()` path** over the real `provide.xml` | the `ArgUtil` assertion fired, naming `provide.xml:844` and saying the oracle confirms that edge |

So the both-directions check **fails in both directions, on every one of the five files** — not on
one of them — and the `ArgUtil` assertion fires on the production path over the actual book, not only
on the sample.

**Gates**, run by me from `rust/` on `fd8cb5cd0`, each status read on its own:

```
cargo fmt --all --check                                             exit 0
cargo clippy --workspace --all-targets -- -D warnings               exit 0
cargo test --release --workspace                                    exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 exit 0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   exit 0   106 of 106 matching
```

Zero `test result: FAILED` lines in either corpus-gate run. `extract_docs.rs`'s seven tests run under
the plain release gate — confirmed in the log, not assumed. `REXX_PHASE_GATE=5a` is correctly not
run: `/bin/grep -arn 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/crates/` matches nothing, so the variable
has no referent yet.

**No sitting, correctly.** The two commits touch `rust/corpus/docs/`, `crates/rexx-extract/src/`,
`crates/rexx-extract/tests/` and one `//!` paragraph in `crates/rexx-extract/src/lib.rs` —
`git show --stat` on both confirms nothing else. `rexx-extract` is none of the four measured crates.

---

## What I could not check

* **Whether these row sets are the right *shape* for tables C and D.** I checked that they say what
  the books and the C++ say. Whether a `(class, method, arm)` triple is the right probe key, and
  whether table D's `position` column carries what Task 4 needs, are Tasks 4 and 5's to answer.
* **A regeneration** — both sides moving in one commit. By design the check cannot see it, and my
  controls could not manufacture it in a way that would prove anything the design does not already
  concede.
* **My independent passes share one assumption with theirs**: text-level scanning with comments
  blanked. A defect in that assumption would be invisible to both instruments. I probed the specific
  things it could hide — attributed `<member>`/`<option>`/`<primary>` tags (none exist), sections and
  members inside comments (found and accounted for: `mthSupplierInit`, `mthObjectInit`,
  `mthClassHashCode`, the `clsArgUtil` member), and `<title>` elements carrying attributes (nine in
  `provide.xml`, none of them a section title, which the extractor's own panic proves) — but I did
  not parse any book with a real XML parser, and the DTD is an `http` URL, so I could not.
* **The `evidence` and `reason` prose beyond the citations it carries.** I verified every sampled
  `file:line` resolves and says what the row claims. I did not read all 1345 reasons.
* **The 64 `covered` rows that answer on both arms.** Their `arm` column is unfalsifiable by the
  readback instrument, as the report says; nothing I ran narrows it.
* **`CONSTRUCTION` staying true.** I re-measured it once, today, and it matched exactly. Nothing in
  the tree re-measures it, by the spec's decision; what is checked is that its names are the class
  set the books produce.
* **Any claim about upstream drift.** Everything here is r13198 for `oodocs/rexxref` and the tracked
  `interpreter/` at `fd8cb5cd0`.
