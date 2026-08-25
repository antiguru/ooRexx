# Task 3, fix round 1 — re-review

Scope `c36f52881..c0fda0a44`, one commit, re-reviewed at `c0fda0a44` with a clean working tree.
`oodocs/rexxref` and `oodocs/rexxpg` at r13198, `ootest/` at r13178, checked with `svn info` today.

**Verdict: APPROVE.** Every finding closes, the guard was seen firing on the production path over a
real book with a genuine sixth `template:` member introduced, and the both-directions test fails in
both directions. What is left is two nits and one low, none of which can make a gate pass while
meaning nothing.

## Findings

* **F1 — CLOSED.** `xrefstyle` is read, the two rows are emitted and are true on the oracle, and the
  rule is guarded by an assertion I saw fire at both of its call sites and on both of its arms
  (unknown style, and no style at all). The round's correction to the exception list is right at the
  source: `TEMPLATE_MEMBERS` correctly holds three.
* **F2 — CLOSED.** The `hashCode` decision is recorded, with a measurement behind the rejection of
  the derived-list alternative that reproduces exactly (41 commented class-table members over the
  four books, 38 with no live `mth*` section anywhere, 9 of those `mthStream!*`). Measured,
  `.Class~hasMethod("HASHCODE")` is `1`.
* **F3 — CLOSED.** The explanation is inside the committed artifact and derived, and it is policed:
  editing the `ArgUtil` line of the new list reddens with both row lists empty and the message
  naming the header. Independently, `comm` over the two committed files says `ArgUtil` is the only
  class-set name with no method row, which is exactly what the list carries.
* **F4 — CLOSED.** Both corrected evidences reproduce here: 16 `*classmethods.xml` files in
  `rexxref/en-US`, `/bin/grep -arho 'href="[a-z]*classmethods.xml"'` over the four books gives 55
  occurrences of 15 distinct hrefs with `objectclassmethods.xml` the one absent; and
  `/bin/grep -arn 'cargo\|rust' .github/workflows/` matches two lines, both `svn checkout`, across
  three workflow files — so CI runs no `cargo`.
* **F5 — n/a.** Closed by the controller at `c36f52881`; the task's half was already right.
* **F6 — CLOSED.** Both subset counts are gone from the two doc comments, and nothing else in the
  crate was swept. (One statement that replaced a count needs a second look — N1 below.)

## New defects, most severe first

### N1 — LOW. The replacement for one of the F6 counts states a criterion the report contradicts, and names a tag form no cited sentence has

`classes.rs`'s `unconstructible_index` doc now reads *"because a sentence in these books may wrap
across a line break or **carry an `<xref>` in the middle** … **that a row's span is a range rather
than one line is what says it wraps**"*. Measured, over the six `UNCONSTRUCTIBLE` rows at the lines
they cite, asking of each whether the raw cited lines already contain the stored sentence:

| row | span | raw substring test | tags inside the cited span |
|---|---|---|---|
| Buffer `:429` | one line | passes | none |
| Pointer `:6910` | one line | passes | none |
| RexxContext `:7545` | one line | passes | none |
| RexxInfo `:7942` | one line | passes | none |
| StackFrame `:9407` | one line | passes | none |
| VariableReference `:12556-:12557` | range | **fails** | `<methodname>` |

So the criterion in the second clause is true — exactly the ranged row is the one that wraps — but
the example in the first clause is not a form any cited span has: the one live mid-sentence tag is
`<methodname>`, not `<xref>`. The two clauses also use "sentence" in two senses: the book's full
sentence (of which `RexxInfo`'s does carry an `<xref>` mid-sentence, at `:7940`-`:7942`) and the
cited span. Under the full-sentence reading the criterion is false — `Buffer`'s sentence spans
`:428`-`:429` with a one-line citation.

**And the report was not brought along.** Section 3 item 4 still reads *"three of the six sentences
wrap across a line break and one carries an `<xref>` mid-sentence"* — the count the round removed
from the code. Measured, it is one under the cited-span reading and four under the full-sentence one
(`Buffer`, `Pointer`, `RexxInfo`, `VariableReference`); it is not three under either. So the round
deleted a wrong count from the source and left the same wrong count in the record, and the record
and the comment now state different tests.

**Consequence, and it is bounded.** A later task adding an `UNCONSTRUCTIBLE` row reads item 4,
believes several of these citations need line-joining, and cannot tell from the comment which sense
of "wraps" decides the span it should write. Nothing goes silently wrong: `unconstructible_index`
re-reads every citation on every run and a span that does not contain its sentence reddens. This is
reader cost, not a gate hole.

### N2 — NIT. The header's provenance sentence is false for the two new rows, and the artifact says nothing about the override

`class-methods.txt`'s header says *"`section` is the mth\* section **the name came from** and
`origin` is that section's `file:line`, so every row cites the book."* For the two new rows the name
came from the class-table member's `xrefstyle`, at `utilityclasses.xml:1281` and `:10492`, which no
row cites. Following `DateTime new class`'s own citation lands on
`utilityclasses.xml:1775`, which is `<section id="mthDateTimeInit"><title>init</title>` — no `new`
anywhere in it. `/bin/grep -a 'xrefstyle\|template\|displays' rust/corpus/docs/class-methods.txt`
matches nothing, so the artifact carries no rule that explains the two rows; `TEMPLATE_MEMBERS`'s
doc does, and it is in the crate rather than in the file.

**Why this is a nit and not the F3 shape again.** The looseness is pre-existing: the rows that come
from a group heading have the same property — `Object = instance` cites
`fundclasses.xml:2600`, whose title is `Comparison Methods`, and the name `=` comes from the class
table's trailing text. This round extends an existing convention by two rows rather than starting
one. One header sentence naming the override would settle both, in the shape the header already uses
for `RegularExpression` and for the placeholders.

**Consequence.** The spot-check the first review ran — "the `<section id>` is on the cited line, and
the method name is derivable from that section's title under the stated rules" — now has two rows it
fails on, against a header that states the rule they break. A Task 5 reviewer sampling them files a
defect or "fixes" a correct row.

### N3 — NIT. The new derived list is the one whose citation is a file rather than a `file:line`

List (3) renders `ArgUtil  --  the books document it nowhere; its only citation is provide.xml`,
while the `ArgUtil` row it is derived from carries `provide.xml:838` and both other derived lists in
the same header carry `file:line`. `classes_without_method_rows` has `r.book` in hand and the line
beside it in the same `ClassRow`. Consequence: the reader the list exists for has to search for the
commented-out member instead of jumping to it.

## What I ran

**The `xrefstyle` assertion, seen firing.** All on a faithful relocated copy of the crate outside the
repository — the same sources, with `oodocs/` and `interpreter/` beside it — because the tracked
trees are read-only for this review. The copy is `7 passed` before and after every control, checked
each time.

| control | what I changed | what happened |
|---|---|---|
| **B1 — the one the brief asked for** | a **sixth** `template:` member introduced in the book: `utilityclasses.xml:11346`, a `clsTraceObject` class-table member, `select:title` → `template:start` | FAILED. `utilityclasses.xml names mthTraceObjectActivate with xrefstyle "template:start", which neither displays the target section's own title nor is a member this extractor has decided about…` — it reddens, it does not drift |
| B2 | the `xrefstyle` removed from that same member | FAILED, `…in a <member> whose <xref> carries no xrefstyle` |
| B3 | `mthDateTimeInit` deleted from `TEMPLATE_MEMBERS` | FAILED, the same panic naming `mthDateTimeInit` (the round's control I, reproduced) |
| B4 | `mthTimeSpanInit`'s expected style text changed in the source | FAILED, the panic naming `mthTimeSpanInit` — the displayed text is part of the key (the round's control J, reproduced) |
| B5 | the **book** drops the override: `utilityclasses.xml:1281` `template:new (Inherited Class Method)` → `select:title` | FAILED the other way, `rows committed and no longer derived (1)` naming `DateTime new class`. So the row is held from the book side too, not only by the panic |
| B6 | the same mutation at the **other call site**: `collectionclassmethods.xml:49`, an `xi:include`d member | FAILED with the same panic naming `mthCollectionAtGet` — the guard covers both places members are read |

A note on my own probes: my first attempt at B5 edited `:1775`, the *section* line rather than the
member line, changed nothing, and the suite stayed green. Recorded because a no-op mutation reads
exactly like a control that could not fire; the run above is the one that diffs its own edit.

**The both-directions test, in both directions.**

| control | what I changed | what happened |
|---|---|---|
| A1 | deleted the new `DateTime new class` row from `class-methods.txt` | FAILED, `rows derived and not committed (1)` printing that exact row |
| A2 | appended an invented `DateTime zznosuch class` row | FAILED, `rows committed and no longer derived (1)` printing it |
| A3 | edited the new list (3)'s `ArgUtil` line in the header | FAILED with **both** row lists empty and `If both lists are empty the header moved` |
| A4 | deleted the last row of `directive-options.txt` | FAILED, `rows derived and not committed (1)` — the loop still reaches past the file this round touched |
| A5 (unplanned) | my own `.orig` backup left in `corpus/docs/` | `the_row_set_directory_holds_exactly_the_row_sets` FAILED naming it |

**Re-derived.** `rexx-extract-docs --check` from `rust/`: all five unchanged, exit 0, and
`class-methods.txt` reported at **1347 rows** by the extractor's own `set.rows.len()`. Re-run writing
to a scratch directory and `cmp`'d against the committed copies: **5 of 5 byte-identical**.

**The row count, by a predicate that means "a row".** `awk '!/^#/ && NF'` over
`corpus/docs/class-methods.txt` gives **1347**, agreeing with the extractor's own count;
`/bin/grep -avc '^#'` gives 1348, which is the trap in the brief — it counts the blank line that
separates the header from the data. `covered` rows, `awk -F'\t' '!/^#/ && NF && $4=="covered"'`:
**794**, over **37** distinct classes. `git show` on the data file: exactly two rows added,
`DateTime new class` and `TimeSpan new class`, and no other data line moved.

**The `covered` sweep, re-run independently of their generator** — programs written from the
committed file, placeholders `(abuttal)`/`(blank)` mapped back to `""` and `" "`, one program per
class, each run from a fresh empty directory under the wrapper with the three descriptors read
separately: **37 runs, all rc 0 with empty stderr, and the neither-arm set empty over all 794 rows.**
The arm split is **both 64, class arm only 76, instance arm only 654** — the first review's
64/75/654 plus the one new class-arm row — and for every row where exactly one arm answers, the
declared `arm` **is** that arm, 0 exceptions.

**The oracle claims the round rests on**, from a fresh empty directory, absolute paths, three
descriptors separately, rc 0 and empty stderr: `.DateTime~hasMethod("NEW")` `1` with
`.DateTime~new~hasMethod("NEW")` `0`; `.DateTime~hasMethod("INIT")` and `.DateTime~new~hasMethod("INIT")`
both `1`; `.TimeSpan~hasMethod("NEW")` `1`; `.RexxQueue~hasMethod("NEW")` `1` with the instance arm
`0`; `.TraceObject~hasMethod("NOTIFY=")` `1`; `.Class~hasMethod("HASHCODE")` `1`. Separately measured
that `hasMethod` is case-insensitive, so the uppercase spellings in the record are not load-bearing.

**The round's correction to the first review, checked at the source.** `clsTraceObject` opens at
`utilityclasses.xml:11301` and its first nested section is `mthTraceObjectActivate` at `:11396`, so
its head is `:11301`-`:11395` — and `Section::head` is the body cut at the first `<section`, which I
read rather than assumed. The two `template:` occurrences at `:11472` and `:11492` sit inside
`mthTraceObjectCollectorSet` (`:11451`-`:11508`), in running prose, one of them inside a `<note>`;
neither is in any span this extractor reads. `clsTraceObject`'s actual table, inside the head, names
`mthTraceObjectNew` and `mthTraceObjectNotifySet` with `select:title`. **So the round is right and
the first review's "come out right by accident" was right for the wrong reason on those two, and
`TEMPLATE_MEMBERS` correctly holds three.** Holding five would be worse: a prose member later moved
into a table would then be accepted silently instead of reddening.

**The negative, with the pattern beside it.** Over the four books plus every `*classmethods.xml` in
`rexxref/en-US`, comments blanked byte-for-byte, matching `<member(\s[^>]*)?>(.*?)</member>` and
inside each the first `<xref(\s[^>]*?)/?>` with a generic attribute reader (any order, either
quote — not the linkend-then-xrefstyle order the reviewed pattern assumes):

* **1186** `<member>` elements, against 1186 `<member` open tags and 1186 `</member>` — none
  self-closing, none carrying attributes, so nothing was skipped by the shape of the match;
* **1173** of them carry an `mth*` `<xref>`, and **0** of those lack an `xrefstyle`;
* the exact values are `select:title` **1168**, `template:new (Inherited Class Method)` **3**,
  `template:new` **1**, `template:NOTIFY` **1** — so `starts_with("select:title")` is exact today
  and there is no third case;
* the five `template:` members are at `utilityclasses.xml:1281`, `:8838`, `:10492`, `:11472`,
  `:11492`, and structurally: the first three are inside a `cls*` head (`clsDateTime`,
  `clsRexxQueue`, `clsTimeSpan`), the last two are not.

**One hole in the guard, measured to be empty.** `listed_members` filters on `linkend` before
`displayed_names` sees anything, so a member whose `<xref>` the scanner failed to parse is dropped
silently rather than reaching the panic. Over the same corpus: `<xref` not followed by whitespace or
`>` — **0**; members with more than one `<xref` — **0**; members with an `<xref` and no `linkend` —
**0**. So the hole has no instance today, and it is the pre-existing filter rather than anything this
round added.

**Gates**, run by me from `rust/` at `c0fda0a44`, each status read on its own, none piped into
another command:

```
cargo fmt --all --check                                             exit 0
cargo clippy --workspace --all-targets -- -D warnings               exit 0
cargo test --release --workspace                                    exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 exit 0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  exit 0   106 of 106 matching
```

Zero `test result: FAILED` lines in any of the three test runs. `extract_docs.rs`'s seven tests ran
under the plain release gate — read in the log, not assumed — and `rexx-extract`'s lib suite is 32
unit tests, which is the report's number. `REXX_PHASE_GATE=5a` correctly unrun:
`/bin/grep -arn 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/crates/` exits 1.

**Constraints.** `git show --stat` is four files — `corpus/docs/class-methods.txt` and three under
`crates/rexx-extract/src/`; `rexx-extract` is none of the four measured crates, so no sitting is
owed. `git show c0fda0a44 | /bin/grep -an '^+.*unsafe'` exits 1. The working tree is clean.

**Sampled against the book.** The two new rows at both ends: the member that names each
(`:1281`, `:10492`, both read in the source) and the section each cites (`:1775`, `:10700`, both
`<title>init</title>`). Plus `clsTraceObject`'s whole head, `mthTraceObjectCollectorSet`'s prose
members, the six `UNCONSTRUCTIBLE` citations at the lines they name, `mthObjectComparisonMethods` at
`fundclasses.xml:2600`, and `ArgUtil`'s `class-set.txt` row.

## What I could not check

* **Whether these row sets are the right shape for tables C and D.** Unchanged from the first review
  and still Tasks 4 and 5's to answer.
* **A regeneration** — both sides of the both-directions check moving in one commit. By design it
  cannot see one; my controls move one side at a time on purpose.
* **My scan shares one assumption with theirs**: text-level matching with comments blanked. I probed
  what that could hide for *this* rule specifically — attributed `<member>` tags, self-closing
  members, multi-`<xref>` members, unparseable `<xref` forms, members with no `linkend` — and all
  five sets are empty, but I did not parse any book with a real XML parser and the DTD is an `http`
  URL, so I could not.
* **The 64 `covered` rows that answer on both arms.** Their `arm` column is unfalsifiable by the
  readback instrument; nothing I ran narrows it, and the round's own claim about it is scoped to the
  rows where exactly one arm answers, which is the claim I reproduced.
* **`TimeSpan init instance`'s truth.** `TimeSpan` is `not-covered`, so its instance arm cannot be
  measured, and the new `TimeSpan new class` row is gated on nothing either. Only the `DateTime`
  half of F1's fix is inside anything that will be probed.
* **Any claim about upstream drift.** Everything here is r13198 for `oodocs/rexxref` and the tracked
  `interpreter/` at `c0fda0a44`.
