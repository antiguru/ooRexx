# Task 2, fix round 4 — re-review

Scope `84b62a596..5a466236b`, one commit, one file
(`docs/superpowers/plans/phase-5-reading-ledger.md`, 254 insertions / 49 deletions, no code).
`HEAD` is `5a466236b`, the working tree is clean, and `git show HEAD:…` of the ledger is
byte-identical to the file on disk — so every run below was against the committed object.

**Verdict: APPROVE.** All seven findings are closed, and each was closed by fixing the mechanism
rather than the sentence. The two substantive fixes do what the round claims, measured directly
rather than argued: the committed extractor pulled from the file's raw bytes reproduces its committed
block **byte for byte** with no `UNPLACED`, and its label column is **one string over all 24
orderings** of the four lists where the previous round's script gives eight and moves nine rows under
full reversal. Both negative controls fire; the new `cls*` extractor reproduces byte for byte and its
cross-width assertion fires when broken. **Every one of the seven rules the script states as
load-bearing was broken in turn and behaves exactly as written**, including the one stated as a
guard. The round's own two extra findings are both real and both check out at the C++ source.

Two new defects, **both LOW, neither one I would block on**, and one of them is report-side on an
untracked file.

---

## Per-finding

| # | | verdict |
|---|---|---|
| N1 | fix the extractor, not the sentence | **CLOSED** — the carry crosses a line break, cleared at a Markdown block boundary or by a bare filename; `ClassClass.hpp:180` and `:189` now appear; the other two wrapped tokens deduplicate onto earlier rows; `07:06` stays suppressed, and I re-measured it returning when the block reset is removed |
| N2 | qualify the match by file | **CLOSED, and by the property rather than the symptom** — `Setup.cpp:1285` and `createInstanceBehaviour :1148` read `(a)`, both confirmed correct at the C++ source; output invariant over all 24 orderings |
| N3 | delete five words | **CLOSED** — `/bin/grep -ac 'extension set above'` exits 1; the block now names the unfiltered form, and all nine patterns re-run in it and reproduce |
| N4 | state the rule as the guard it is, and verify it now does something | **CLOSED** — narrowing the extension set now gains exactly the five rows named; I confirmed independently that narrowing it in the *previous* script changes no byte |
| N5 | the D3 measurement does not support its conclusion | **CLOSED in substance, with a new defect in the replacement** — see D1 |
| N6 | repair the orphaned join | **CLOSED** — `about \`init\`.** And the \`:137\` pattern …` |
| N7 | finish applying the structural ruling | **CLOSED** — the four-books row reads `done against a stated extractor`; extractor and 63-row output committed, reproduce byte for byte, assertion fires when broken |
| beyond-1 | `Setup.cpp:348-351` was a wrong file on a committed row | **CONFIRMED at the source** — see below |
| beyond-2 | the case-sensitivity pair was a negative without its pattern | **CONFIRMED** — both halves reproduce, including the counter-pattern |
| gates | five commands, corpus | **CLOSED** — all `EXIT=0`, `106 of 106 matching` in both corpus gates, no `FAILED` line, `REXX_PHASE_GATE=5a` correctly not run |

---

## New defects, most severe first

### D1 — LOW, report side. N5's replacement reasoning has a stale enumeration and a search narrower than the word it licenses

`task-2-report.md:707`-`:713`: *"What supports the conclusion instead, **measured on the object this
round commits**: no line containing an ellipsis falls inside a fenced block … and each of the ten
ellipsis lines was read. They are an `svn` error message quoted as text (`:38`), four elisions inside
quoted Rexx test bodies (`:637`, `:640`, `:642`, `:869`), four quotations from the reference books
(`:734`, `:755`, `:757`, `:1034`), and one quotation of this document's own phrase (`:1023`)."*

**The ten line numbers do not resolve against the object the sentence names.** Computed on
`5a466236b`'s ledger, the lines carrying U+2026 are `:38`, `:644`, `:647`, `:649`, `:741`, `:762`,
`:764`, `:876`, `:1030`, `:1041`. Every number the report gives except `:38` is exactly **seven**
short of one of these, so the enumeration was taken against an intermediate draft; `:637`, `:734` and
`:1034` are **blank lines** in the committed file. The count of ten is right and the reading holds —
I read all ten, and none is inside a fence or is part of a command — but a reader auditing D3 follows
the numbers to blank lines.

**And "ellipsis" is measured as U+2026 only.** The pattern beside this negative is
`'…' in line`; under the ASCII form the word equally licenses, ledger `:65` — *``# Every `<section
id="cls...">` in the four class books``* — is a line containing an ellipsis that **is** inside a
fenced block, and it is a line **this round added**. So the stated support, "the fenced spans and the
ellipsis lines are disjoint", is false on the reading the sentence invites and true only on the
narrower one, which is not stated. That is the recurring defect of this task on an eighth axis, the
character class this time.

**It costs no conclusion**, and that is worth saying plainly: `:65` is a Python comment, and I ran
the script it sits in from the file's raw bytes — it reproduces byte for byte. The conclusion "no
ellipsis in this document breaks a procedure a reader re-runs" is true, checked by reading each of
the ten and by running both committed scripts.

**Consequence.** Confined to `.superpowers/`, which is untracked, so no later task reads it; the
committed ledger carries no ellipsis claim (`/bin/grep -ac 'ellipsis'` on the ledger exits 1).
**I would not block on it.** A one-line fix is to state the pattern (`U+2026`) beside the claim and
re-take the ten numbers against `5a466236b`.

### D2 — LOW, cosmetic. The committed script's header comment drops two words

Ledger `:297`-`:298`, inside the fenced `python` block:

```
# one measured to change the output is the list item at spec :1400 -- and by any filename with no :N
# own. Those two are what keep a `07:06` timestamp from being read as a citation and a roadmap
```

*"any filename with no `:N` own"* — "of its" is missing across the line break. The same rule is
worded correctly in the prose at `:254` (*"any filename that has no `:N` of its own"*) and in the
inline comment at `:317` (*"a filename with no `:N` of its own ends the citation before it"*), so
this is the wrap swallowing two words in the third copy, not a disagreement about the rule.

**Consequence.** A reader of the script's own account of its reset rule gets a sentence that does not
parse; nothing functional, and the rule itself is correct and measured. **I would not block on it.**

**The pattern beside both negatives.** D1's is *"no line matching `…` (U+2026) in the committed
ledger falls inside a `^```` `-delimited span"*, computed by toggling on fence lines — not "no
ellipsis anywhere", which is the sentence and is false at `:65`. D2's is *"the three copies of the
reset rule at `:254`, `:297` and `:317` do not read the same"*, checked by reading all three, not by
a grep.

---

## The extractor diff, the permutation test, the control runs, and every command I re-ran

`ootest` r13178, `oodocs/rexxref` and `oodocs/rexxpg` r13198, `svn info` run today; `oodocs` itself
`E155007`, as the ledger says. Every pattern and script below was pulled from the ledger's raw bytes
with `sed -n`, never retyped, and run from the repository root (gates from `rust/`). Controls were
run on copies outside the repository; no file under `docs/`, `rust/`, `interpreter/`, `ootest/` or
`oodocs/` was edited.

### Both committed outputs reproduce byte for byte

```
sed -n '287,381p' docs/superpowers/plans/phase-5-reading-ledger.md > spec-cpp-citations.py
python3 spec-cpp-citations.py docs/superpowers/specs/2026-08-17-phase-5-object-model.md \
                              docs/superpowers/plans/phase-5-reading-ledger.md | sort -k1,1 -k2,2V
```

92 rows; `cmp` against `sed -n '393,484p'` — the committed block — reports **no difference**.
`/bin/grep -ac 'UNPLACED'` is 0, exit 1. Label tally 76 `(a)`, 9 `(b)`, 3 `(c)`, 4 `found-wrong`,
matching the prose at `:389`-`:390` exactly. `ClassClass.hpp :180 spec:1361 continuing (a)` and
`:189` are present — the tokens N1 was about.

```
sed -n '65,83p' docs/superpowers/plans/phase-5-reading-ledger.md > cls-sections.py
python3 cls-sections.py oodocs/rexxref/en-US/fundclasses.xml … streamclasses.xml
```

63 rows, `cmp` against `sed -n '94,156p'` reports **no difference**; per-book tally 7 / 17 / 35 / 4.

### The permutation test — the property the finding was about

Patched each script's search order by inserting one reorder line immediately before `def where(`
(`PLACES` in the new script, `BOUNDS` in the old), leaving everything else untouched, and ran all 24
permutations of the four lists through the same `sort`:

| script | distinct outputs over 24 orderings | rows moving under full reversal |
|---|---|---|
| committed at `5a466236b` | **1** — and it is byte-identical to the committed block | 0 |
| `84b62a596`'s, for comparison | **8** | **9** |

The nine that move on the old script are `ClassClass.cpp:984`, `ClassClass.hpp:180-189`,
`ClassDirective.cpp:257`, `Setup.cpp:331`, `:332`, `Setup.cpp:1285`, `createInstanceBehaviour :1148`,
`inherit :1287`, `mixinClass() :1514`. Both figures the round states are exact.

The old script's 90-row block also reproduces byte-identically at `84b62a596`, so the comparison is
between two working artifacts and not against a broken one.

### The seven rules, each broken against the committed script and the committed ledger

| rule as stated | broken how | result |
|---|---|---|
| the carry crosses a line break | `carry = None` restored at the head of the per-line loop | 90 rows, **43 `UNPLACED`**, loses `ClassClass.hpp:180` and `:189` — exactly as written |
| cleared at a Markdown block boundary | `if BLOCK.match(line)` → `if False` | gains `(via RexxString::compareToRexx) :06 spec:1401`, the `07:06` timestamp |
| cleared by a filename with no `:N` of its own | the `if bare:` reset removed | gains `PackageClass.cpp :482 spec:1366` and `:492 spec:1365`, the roadmap citations |
| that clearing fires on any filename | `ANYFILE` narrowed to `(?:cpp\|hpp)` | gains **five** rows: `DirectiveParser.cpp :66 :73 :92 :93` and `(via Invocation::with_engine) :244` — the four `.orx` and the `corpus.rs` citation, as written. **N4 is now live**, and I confirmed the same narrowing on `84b62a596`'s script changes **no byte** |
| a `Class::method` may be a third-form antecedent | `carry = None` in the `if qual:` branch | `:348-351` dropped with **no row printed** — the silent form |
| only the row's own citation cell counts | `entries()` returns whole rows | `createInstanceBehaviour :1148` flips to `found-wrong` |
| the explicit form is matched by file | `key = '*'` unconditionally | **no change**, which is what the script and the prose both say. Stated as a guard, correctly |

I also tested the **block-boundary rule one alternative at a time**, because the script's comment
claims *"the one measured to change the output is the list item at spec `:1400`"*: removing the blank
line, numbered-list, heading, table and fence alternatives each changes nothing, and only
`[-*+>][ \t]` changes the output. The comment is exact.

### The negative controls, run rather than asserted

* **reword one list's heading** (`### In-tree Rust —` → `### In tree Rust —`, ledger `:532`):

  ```
  AssertionError: [('## Citations found wrong', 1), ('**(a) ', 1), ('**(b) ', 1),
                   ('**(c) ', 1), ('### In-tree Rust', 0)]
  EXIT=1
  ```

* **delete the `ClassClass.cpp:988`-`:990` table row** (ledger `:221`): exactly two lines of the
  output change, both to `** UNPLACED **`, and nothing else moves.

* **narrow the `cls*` pattern to `clsA`**: `AssertionError: oodocs/rexxref/en-US/fundclasses.xml`,
  `EXIT=1`. Fires, naming the first book, exactly as the round reports.

### The round's own two findings, at the source

* `MethodDictionary.cpp:348` is `void MethodDictionary::hideMethod(RexxString *methodName)`, `:350`
  is `put(TheNilObject, methodName);`, `:351` its closing brace — the spec's quotation at spec `:126`
  verbatim. `Setup.cpp:348` is the comment line *"// local variable scope that identify which
  behaviours we are working with."*, `:349` opens `#define StartClassDefinition(name)` and `:350`-`:351`
  are its body: **not** `hideMethod`, and not what the spec cites. Run against `84b62a596`'s script
  the row printed is `Setup.cpp :348-351 spec:126 continuing (a)`; against this one it is
  `(via MethodDictionary::hideMethod) :348-351 spec:126 continuing (a)`. It was *placed* because
  ledger `:202` — list (a) — carries ``MethodDictionary.cpp`` ``:164 :348 :434 :594``. Every clause
  of the claim holds.
* the case-sensitivity pair: over `ootest/framework/` for `#!/usr/bin/env rexx`,
  `--include='*.cls'` exits **1** and `--include='*.[cC][lL][sS]'` returns `OOREXXUNIT.CLS`, which is
  **2289** lines; under `OOREXXUNIT` both forms exit **0**, the case-sensitive one returning
  `FileUtils.cls` and `WinUtils.cls`. The counter-pattern is real and the paragraph now names it.

### My own probes, run independently of the document's

* **every `:\s*\d+` token in the spec classified against the committed extractor.** Emitted 105
  tokens; the occurrences the extractor drops are `.orx` (`CoreClasses.orx`, `StreamClasses.orx`),
  `.xml` (`provide.xml`, `utilityclasses.xml`), `.rs` (`corpus.rs`, `oracle.rs`, `lib.rs`,
  `tests.rs`), `.md`/`.txt` roadmap and exclusions citations, the roadmap's `:482`/`:492`, and the
  `07:06` timestamp. **None is a C++ citation**, so the "no `UNPLACED`" verdict is not hiding a
  dropped C++ token this time — which is the check N1 said could not see itself.
* **the fifth-citation-form axis the previous re-review left open.** No line of the spec matches
  `\bline[s]? [0-9]+`, and the only `.cpp`/`.hpp` mention not carrying its own `:N` is spec `:1365`'s
  bolded `**ClassClass.cpp**`, which is prose naming the file for two symbolic citations already in
  the block. Within that shape the form does not exist in this spec.
* **the `cls*` pattern's blind spots, tested rather than conceded.** Every `<section` in the four
  books carries its `id=` on the same line (the single exception, `utilityclasses.xml:4809`
  `<section><title>Examples</title>`, has no id at all and so cannot be a `cls*`), and no line
  carries two `<section` tags — so neither the line-based match nor the one-match-per-line
  `.search` has a live instance to miss. The three pattern widths give 7 / 17 / 35 / 4 at every
  width.
* **every first nested `<section` inside a `cls*` span is an `mth*`**, which is what the row's
  stopping-point sentence claims; the two spans not bounded by one (`clsSetCollection`,
  `clsInputOutputStream`) are bounded by their own `</section>`, which is the other half of the rule.
* the ellipsis and fenced-span computation of D1 above.

### Commands stated in the document, re-run

| group | n | result |
|---|---|---|
| the C++ extractor, its invocation, `cmp`, `UNPLACED` count | 3 | 92 rows, byte-identical, no `UNPLACED` |
| the `cls*` extractor, its invocation, `cmp` | 3 | 63 rows, byte-identical, 7 / 17 / 35 / 4 |
| the three negative controls | 3 | all fire, as quoted above |
| the seven rule-breaks, plus six block-alternative removals, plus the old script's `ANYFILE` narrowing | 14 | every one as the document states |
| all 24 orderings on both scripts | 48 | 1 distinct output against 8 |
| the nine `\|`-block patterns, unfiltered, `--exclude-dir=.svn` | 9 | `:124` exit 1; `:126` its four files; `:126-names` piped over `:126`'s hits exit 1; `:137` exit 1; `:153` its three; `:154` `Package.testGroup`; `:143`/`:152` their files; `:169` its files |
| the four block-pattern counts on the files the rows name | 4 | `:143` **32**, `:152` **12**, `:154` **52**, `:169` **60** |
| the case-sensitivity pair and its counter-pattern | 4 | as above; `OOREXXUNIT.CLS` 2289 lines |
| mapping-table counts, sampled across six rows | 15 | `MIXINCLASS` 52 / 67, `METACLASS` 50 / 58, `ABSTRACT` 16 / 12, `::annotate` 76, `~annotations?\b` 70, `\.methods\b` 3 / 4, `::attribute` 118, `ABSTRACT` 50, `EXTERNAL[[:space:]]+['"]LIBRARY` 14, `::constant` 76, `ACTIVATE` 5 |
| comment-stripping pairs, 7 member files + 4 section files | 22 | 255/249, 354/345, 391/374, 54/45, 25/24, 34/33, 9/8; sections 250/250, 346/346, 372/**371**, 45/45; sums 1013 and 1012, and `mthSupplierInit` at `utilityclasses.xml:10062` inside `<!-- see new()` at `:10061` |
| item 6a | 3 | tree-wide returns only the two `#define`s at `Setup.cpp:360`/`:361`; non-preprocessor count 0, exit 1; `RemoveMethod(` **9**, `HideMethod(` **12**, running `:792` to `:1404` |
| row `:141`'s citations | 4 | `USELOCAL.testGroup:119`-`:120` builds the five assignments, `:131` asserts `"RESULT RC SELF SUPER SIGL"`; `FUNCTION.testGroup:1053` is `::method TestGetAllVariables1`; `base/special.variables/` holds only `RESULT_RC_SIGL.testGroup` |
| the `cls*` block's cross-checks at the source | 6 | `clsBuffer` opens `utilityclasses.xml:421` and its first nested is `:459`; `clsPointer` `:6902`/`:6943`; `clsClass` `fundclasses.xml:51`, first nested `:134`; ledger `:552`-`:553` and `:1036`-`:1037` cite `:429` and `:6910`, both read and both carrying the native-API sentence |
| the two flipped labels at the C++ source | 2 | `Setup.cpp:1285` is `EndSpecialClassDefinition(RexxInfo);`; `ClassClass.cpp:1148` is `void RexxClass::createInstanceBehaviour(…)`; `PackageClass.cpp:1276`/`:1285`/`:1294` are three `for` loops |
| `svn info` on the three trees and on `oodocs` | 4 | r13178 / r13198 / r13198, `E155007` |
| the five gate commands | 5 | below |

```
cargo fmt --all --check                                              EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                EXIT=0
cargo test --release --workspace                                     EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                  EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast    EXIT=0   106 of 106 matching
```

No `FAILED` line in any of the three test logs. `REXX_PHASE_GATE=5a …` not run and not asked for.
No performance sitting owed: `docs/` only. The commit message is ASCII only and carries both required
trailers.

---

## What I could not check

* **Whether the authorities the ledger lists are all there are.** Unchanged and unchangeable from
  inside the document.
* **Whether the readings happened.** "Read end to end" is not checkable by any instrument; only the
  derived facts are, and those reproduce.
* **Whether a `cls*` section could exist that neither pattern width reaches.** I closed the two
  shapes I could test — a tag split across lines, and two tags on one line — and neither exists in
  these books. A section whose id does not begin `cls` is outside every width, as the round says, and
  nothing inside the document can see it.
* **The order-invariance of the label column against *future* content.** It is measured on today's
  citations, not enforced: the permutation harness is mine and is not committed. It does not need to
  be — `at == sorted(at)` pins the document order the block was derived under, so a re-derivation is
  deterministic — but a later citation whose line number collides across two lists would reintroduce
  the ambiguity silently, and the ledger's residue paragraph is where that is disclosed rather than
  checked.
* **The report's commands-run count itself.** I re-ran what is listed above and everything in it
  reproduced; I cannot audit the 127/125 tally as a number.
* **The rows this round did not touch.** Their counts were re-run where cheap and reproduce; their
  prose was approved in earlier rounds and is not reopened. No oracle run — nothing in the diff moves
  an oracle-derived claim, and the release binary is byte-identical.

No file was edited except this review. No subagents were dispatched.
