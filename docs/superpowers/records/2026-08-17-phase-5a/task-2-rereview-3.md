# Task 2, fix round 3 — re-review

Scope `1b536079a..84b62a596`, one commit, one file
(`docs/superpowers/plans/phase-5-reading-ledger.md`, 247 insertions / 30 deletions, no code).
`HEAD` is `84b62a596`, the working tree is clean, and `git show HEAD:…` of the ledger is
byte-identical to the file on disk — so every run below was against the committed object.

**Verdict: REWORK.** The round's central deliverable works. The committed extractor, pulled from the
file's raw bytes, reproduces its committed 90-row output **byte for byte**, with no `UNPLACED`; both
negative controls were run and both fired; the fifth defect the round found itself is real and every
header fact it states holds at the source; D1–D4 are closed. The rework is smaller than any previous
round and is all in the derivation the round added: the extractor's stated reach is wider than its
real one on a form the spec actually uses, two of the six `found-wrong` labels in the committed
output are wrong and flip when the search order is permuted, and one instruction still tells a reader
to apply the extension filter this round deleted.

---

## Per-finding

| # | | verdict |
|---|---|---|
| D1.1 | place `ClassClass.cpp:988`-`:990` | **CLOSED** — a list (b) row; `:984` and the three-line comment verified at the source, double space after `behaviour.` included |
| D1.2 | widen the extractor to the third citation form | **CLOSED in substance** — both tokens come out `continuing`; but the sentence stating the reach is wider than the reach (N1) |
| D1.3 | mark the C++ row against what the check reaches | **CLOSED** — the row reads `done against a stated extractor`, names the reach, and says a fourth form would be invisible |
| D2 | extension enumeration and case-sensitive globs | **CLOSED, and better than asked** — the filter is gone rather than widened; census, case-sensitivity pair and every negative re-run and reproduce |
| D3 | no ellipsis in any command | **CLOSED** — no ellipsis in the file sits inside a command; row `:137`'s command runs as written. The report's *stated measurement* for it does not reproduce (N5) |
| D4 | `:137`'s single hit disqualified explicitly | **CLOSED** — `nop` at `:14`, `:35`, `:51` read at the file; the sentence is there. Its join to the next clause is broken (N6) |
| 5th | `ClassClass.hpp:176-186` placed by no entry | **CLOSED and independently confirmed** — in `1b536079a` the string occurs once, in prose at `:123`, in no list; every header fact holds |
| gates | five commands, corpus | **CLOSED** — all `EXIT=0`, `106 of 106 matching` in both corpus gates, no `FAILED` line, `REXX_PHASE_GATE=5a` correctly not run |

---

## New defects, most severe first

### N1 — MEDIUM. The spec writes the third form across a line break, three times, and those tokens are dropped silently rather than reported `UNPLACED`

Ledger `:139`-`:140`: *"The extractor reaches the **three** citation forms the spec uses"*, and `:149`:
the carried filename *"**resets at every line**"*, justified solely by the `07:06` → `Setup.cpp:06`
false positive.

The spec's prose is hard-wrapped: 451 of its non-table lines end between columns 95 and 100, and the
wrap point falls wherever it falls. So form 3 — a bare `:N` continuing an earlier citation in the
same parenthesis — sometimes has its filename on the **previous** line. **The per-line reset drops
every such token, and drops it invisibly**: no row is printed at all, so the "no `UNPLACED`" verdict
cannot see it. Three are present in the committed spec:

| spec | the citation | the continuation, on the next line |
|---|---|---|
| `:1349`-`:1350` | `Setup.cpp:185` … `:396` | `:1285` (`RexxInfo`) |
| `:1359`-`:1360` | `Setup.cpp:325-329` for `SELF`/`SUPER` | `:331`/`:332` |
| `:1360`-`:1361` | `ClassClass.hpp:176-186` for the class flags | `:180`-`:189` |

Confirmed by hoisting `carry = None` out of the per-line loop: the output gains
`ClassClass.hpp :180 spec:1361 continuing` and `:189 spec:1361 continuing` — tokens the committed
script never emits — alongside the `Setup.cpp:06` artifact the rule exists to suppress.

**Demonstrated in isolation**, same text, differing only in where it wraps, run against the committed
ledger:

```
=== A: wrapped, exactly as the spec wraps at :1359-:1360 ===
ClassClass.cpp             :9001        spec:1     explicit    ** UNPLACED **
=== B: identical text on one line ===
ClassClass.cpp             :9001        spec:1     explicit    ** UNPLACED **
ClassClass.cpp             :9002        spec:1     continuing  ** UNPLACED **
ClassClass.cpp             :9003        spec:1     continuing  ** UNPLACED **
```

**It costs no conclusion today**, and that is worth saying plainly: all three dropped tokens are
cited elsewhere in the spec in a form the extractor does reach, so the residue is empty and the
committed output is right. **The defect is the sentence and the blind spot, not the answer.**

**Consequence.** The ledger's honest paragraph at `:231`-`:234` says *"a fourth form nobody has
thought of is invisible to it exactly as the third was"* — framed as hypothetical. It is actual, it is
in the spec now, and it is the **likeliest shape of the next citation added**, because a wrap is not a
choice the author makes about the citation. When that happens the check prints 91 rows with no
`UNPLACED` and the reader who runs nothing sees a clean derivation. The round exists to stop exactly
that. The fix is one line — carry the filename across a line break but reset it on a blank line or on
any intervening filename — or, if the reset must stay, say in the reach paragraph that a wrapped
continuation is out of reach and name the three.

### N2 — MEDIUM. Two of the six `found-wrong` labels in the committed output are wrong, and both flip when the search order is permuted

`where()` searches `BOUNDS` in the fixed order found-wrong, (a), (b), (c) and returns the first
section whose entries carry a matching `:N` — with no file association. Two rows lose that race:

* `Setup.cpp :1285 spec:371 explicit found-wrong`. `Setup.cpp:1285` is
  `EndSpecialClassDefinition(RexxInfo);` and the spec cites it at `:371` for the `RexxInfo` row. It is
  in list (a)'s `Setup.cpp` run at ledger `:92` — the list headed *"Resolve to what the spec's cell
  names them as"*, i.e. the ledger's own record that this citation is **correct**. It is labelled
  `found-wrong` because the found-wrong table's `processInstall` row names `PackageClass.cpp`'s loops
  at `:1276`, **`:1285`**, `:1294`.
* `(via createInstanceBehaviour) :1148 spec:117 symbolic found-wrong`. `ClassClass.cpp:1148` is in
  (a) at ledger `:86` and the found-wrong table names it only as the correct sibling.

The found-wrong table has four rows, and none of them is about `Setup.cpp` at all.

**Measured, by reordering `BOUNDS` to put (a) first and changing nothing else** — exactly two rows
move, both to `(a)`:

```
< Setup.cpp                  :1285        spec:371   explicit    found-wrong
> Setup.cpp                  :1285        spec:371   explicit    (a)
< (via createInstanceBehaviour) :1148        spec:117   symbolic    found-wrong
> (via createInstanceBehaviour) :1148        spec:117   symbolic    (a)
```

**Consequence.** The committed block is the artifact a later task diffs, and its label column is what
a reader uses to answer "which of the spec's citations did this ledger find wrong?". For
`Setup.cpp:1285` the answer it gives contradicts the list the same document places that citation in.
The disclosure at `:233` — *"a token that happens to appear in a list under an unrelated file counts
as placed"* — covers the mechanism but stops one step short of the round's own ruling: it asserts the
limitation instead of naming the two rows where it fired. Naming them in a sentence beside the block
costs nothing and closes it; qualifying the match by file would close it properly.

### N3 — MEDIUM. `:484` still routes the `|`-block patterns through the extension filter this round deleted

Ledger `:400`: *"**There is no extension filter, and that is the fix rather than a wider one.** Every
negative below is run over the whole checkout"*, and `:418`: *"A filter narrow enough to write down is
narrow enough to be wrong. So there is none."*

Ledger `:484`, introducing the nine patterns that cannot live in a table cell: *"All are run over
`ootest/` **with the extension set above**, at r13178."*

There is no extension set above any more. The only thing "above" that could answer to the phrase is
the five-extension list at `:411`, which `:407`-`:416` exists to condemn as the way this table was
wrong the second time.

**Consequence.** These nine are the patterns for rows `:124`, `:126`, `:137`, `:143`, `:152`, `:153`,
`:154` and `:169`, including three of the document's negatives. A reader following `:484` literally
re-runs them under `--include='*.testGroup' --include='*.rex' --include='*.cls' …` — the narrowing
whose removal is this round's D2 fix — and, on `:137` in particular, under a filter that cannot see
`class.testgroup.cls`, the file the row is about. Deleting five words fixes it.

### N4 — LOW. The rule "resets on any filename, not only a C++ one" is stated as load-bearing and is inert against the committed script

Ledger `:150`-`:152` names two rules as *"load-bearing"* and gives each a worked failure. The first
reproduces (N1 above). The second — *"without that a bare `:453` following an `.orx` name is
attributed to the last `.cpp` seen"* — does not: narrowing `ANYFILE` to `(?:cpp|hpp)` and changing
nothing else leaves the output **byte-identical**, 90 rows, same labels.

The reason is that the per-line reset already covers it: the carry is only set by a filename adjacent
to a `:N`, and the spec has no line where a `.cpp:N` citation is followed on the *same line* by a
non-C++ `file:N` and then a bare `:N`. Break **both** rules together and the ledger's own example
appears — `ClassClass.cpp:453 spec:1239` among 54 new false rows — so the claim was true of a draft in
which the carry crossed lines, and is not true of what is committed. `:453` itself occurs in the spec
only under a non-C++ file.

**Consequence.** Small, and the same family as this round's subject: a rule asserted to be exercised
by the committed output when nothing in that output depends on it. It should read as a guard that
becomes load-bearing if the reset is ever relaxed — which N1's fix would do.

### N5 — LOW. The report's D3 measurement does not reproduce as stated, and its search is narrower than the sentence

`task-2-report.md`: *"Measured on the committed object: no line containing an ellipsis also contains
`grep`, `awk`, `find`, `python3` or `perl`."* Run as written it returns a hit: ledger `:33` contains
`<section id="cls…">` and the word **find**ings.

The conclusion still holds — I checked every ellipsis in the file individually (`:33`, `:38`, `:440`,
`:443`, `:445`, `:536`, `:557`, `:559`, `:671`, `:825`, `:836`) and none is inside a command; `:671`'s
`.methods[…]` is an honest elision of `[("string_cls_" || name)~upper]`, verified at
`CoreClasses.orx:73`. But the five-word list omits `sed`, `svn`, `cargo` and `python`, all of which
appear in commands in this document, so the check is narrower than the sentence it licenses even
where it runs — the recurring defect, on the report side this time.

### N6 — LOW. The D4 insertion orphaned the clause after it

Ledger `:455`: *"…asserts nothing whatever about `init`.** and the `:137` pattern in the block below
the table returns nothing, exit 1."* The word-diff shows why: the old text read *"…also matches
`::method[[:space:]]+.?activate`**;** and the `:137` pattern…"*, and the new bolded sentence replaced
the semicolon with a full stop, leaving a sentence that begins with a lower-case `and`.

### N7 — LOW. The structural ruling was applied to the C++ row only

The brief: *"Same rule anywhere else the ledger says 'every', 'all', or 'none remain'."* The
`fundclasses.xml` / `collclasses.xml` / `utilityclasses.xml` / `streamclasses.xml` authority row still
reads **done** — *"every `<section id="cls…">`'s prose from its opening tag to its first nested
section, **extracted mechanically** and read"*: a completeness claim naming a mechanical extraction,
with neither the extraction nor its output committed. It is the same shape the C++ row just shed.

**I checked whether it costs anything and it does not, today.** The denominator is stable under three
widths of the pattern — `<section id="cls`, `<section[[:space:]]+id=["']cls` and a case-insensitive
`<section[^>]*id=["']cls` all give 7 / 17 / 35 / 4 across the four books — so a stated extractor would
have produced the same set however it was written. The neighbouring `rexxpg/classes.xml` row is
already checkable and checks out: `:46` is `<chapter id="classes"><title>A Closer Look at
Objects</title>`, `:1500` is the file's only `</chapter>` and its last line.

---

## The extractor diff, the negative controls, and every command I re-ran

`ootest` r13178, `oodocs/rexxref` and `oodocs/rexxpg` r13198, `svn info` run today; `oodocs` itself
`E155007`, as the ledger says. Every pattern and script below was pulled from the ledger's raw bytes
with `sed -n`, never retyped, and run from the repository root (gates from `rust/`).

### The extractor's committed output reproduces byte for byte

```
sed -n '145,196p' docs/superpowers/plans/phase-5-reading-ledger.md > spec-cpp-citations.py
python3 spec-cpp-citations.py docs/superpowers/specs/2026-08-17-phase-5-object-model.md \
                              docs/superpowers/plans/phase-5-reading-ledger.md | sort -k1,1 -k2,2V
```

90 rows. `cmp` against `sed -n '208,297p'` of the ledger — the committed block — reports **no
difference**. `/bin/grep -ac 'UNPLACED'` is 0, exit 1. The label column tallies 72 `(a)`, 9 `(b)`,
3 `(c)`, 6 `found-wrong`, matching the prose at `:115`-`:116` exactly (two of the six are N2).

### Both negative controls fired, run rather than asserted

Both on copies of the ledger outside the repository; no file under `docs/` was edited.

* **Reword one list's heading** (`### In-tree Rust —` → `### In tree Rust —`):

  ```
  AssertionError: [('## Citations found wrong', 1), ('**(a) ', 1), ('**(b) ', 1),
                   ('**(c) ', 1), ('### In-tree Rust', 0)]
  EXIT=1
  ```

  Fires, names the marker, count zero — as described.

* **Delete the `ClassClass.cpp:988`-`:990` table row** (ledger line 107):

  ```
  ClassClass.cpp             :988         spec:946   continuing  ** UNPLACED **
  ClassClass.cpp             :990         spec:946   continuing  ** UNPLACED **
  ```

  Fires, and only those two rows change.

### The four load-bearing rules, tested by breaking each

| rule | broken how | result |
|---|---|---|
| carried filename resets at every line | `carry = None` hoisted out of the per-line loop | 90 → 95 rows; gains `Setup.cpp :06 spec:1401` from the `07:06` timestamp, plus `PackageClass.cpp:482`/`:492` from a roadmap citation and the two `ClassClass.hpp` rows of N1. **Load-bearing, exactly as stated** |
| resets on any filename, not only a C++ one | `ANYFILE` narrowed to `(?:cpp\|hpp)` | output **byte-identical**. **Not load-bearing alone** (N4); broken together with the reset it produces the ledger's own `:453` example among 54 new rows |
| fenced blocks blanked before markers are searched | `re.sub` removed | `AssertionError: [(…, 2), ('**(a) ', 2), ('**(b) ', 2), ('**(c) ', 2), ('### In-tree Rust', 3)]`, `EXIT=1`. **Load-bearing** — and the assertion now catches it loudly rather than silently truncating (c), which is the guarded version of the failure the comment describes |
| only the lists' table rows count, not the prose | `body = span` for every list | the delete-a-row control **stops firing**: the deleted `:988`/`:990` come back `(c)`, placed by the paragraph that discusses them. Three further rows mis-attribute — `inherit() :1322`, `RexxClass::subclass :1631`, `PackageClass::findClass :1086` flip to `found-wrong`, which is precisely the re-attribution the report claims the tightening fixed. **Load-bearing, and the report's account of it is exact** |

### The fifth defect, at the source

`ClassClass.hpp:176` is `static RexxClass *classInstance;`, `:178` `protected:`, `:180` `typedef enum`
through `:189` `} ClassFlag;`; the cited range ends at `:186` `PRIMITIVE_CLASS,` and stops before
`:187` `PARENT_HAS_UNINIT,` and `:188` `ABSTRACT,` — every clause of the (c) row and of the spec's
correction at `:1360`-`:1362`. That it was placed nowhere before is independently confirmed:
`git show 1b536079a` of the ledger contains `ClassClass.hpp:176-186` once, in prose at `:123`.
`ClassClass.cpp:984` is `MethodClass *RexxClass::method(RexxString *method_name)` and `:988`-`:990`
is the quoted comment, double space after `behaviour.` and all.

### Commands stated in the ledger, re-run: 109. Reproduced: 109.

| group | n | result |
|---|---|---|
| the citation extractor and its invocation | 1 | 90 rows, 0 `UNPLACED`, byte-identical to the committed block |
| the extension census, `find … \| sed \| sort \| uniq -c \| sort -rn` | 1 | `testGroup` 409, `rex` 49, `cls` 9, `testUnit` 7, `oodTestGroup` 2, **`CLS` 1**, `norex` 1, `other` 1, `test1` 2, `test2` 1 — and four extensionless files, which I enumerated: `test_sysfile`, `test_sysfile_readonly`, `search_order`, `lineout` |
| the case-sensitivity pair on `ootest/framework/` | 2 | `--include='*.cls'` **exit 1**; `--include='*.[cC][lL][sS]'` returns `OOREXXUNIT.CLS`, 2289 lines, first line `#!/usr/bin/env rexx` |
| the nine `\|`-block patterns, tree-wide, `--exclude-dir=.svn`, no filter | 9 | `:124` exit 1, `:126` its four files, `:126-names` over their 21 hit lines exit 1, `:137` exit 1, `:143`/`:152` their files, `:153` its three, `:154` `Package.testGroup`, `:169` its files |
| the block counts on the files the rows name | 6 | `:143` 32, `:152` 12, `:154` 52, `:169` 60, `\breply\b` 33, Stem `hasMethod` 0 (exit 1) |
| the inline row negatives, tree-wide | 7 | `inheritInstanceMethods` exit 1; `:139` its three files; `:146` its four; `:170` its one file and 6 lines; `:137` **29** files, intersection exactly `base/class/class.testgroup.cls` |
| the mapping table's `-aciE` counts | 50 | every one matches its stated figure, including the four `::method[^;]*\bX\b` arms 12 / 12 / 6 / 9 |
| comment-stripping pairs, 7 member files + 4 section files | 22 | 255/249, 354/345, 391/374, 54/45, 25/24, 34/33, 9/8; sections 250/250, 346/346, **372/371**, 45/45, sums 1013 and 1012. `mthSupplierInit` is `utilityclasses.xml:10062` inside `<!-- see new()` at `:10061` |
| the `awk` block, run as a script from the file | 1 | `CoreClasses.orx:1590` and `:1618` under `::CLASS 'Alarm'`, `:1690`-`:1692` under `::class Ticker`, `StreamClasses.orx`'s block under `Stream` / `RexxQueue` / `File` (`:510`, `:546`, `:547` among them), nothing from `PlatformObjects.orx` |
| the spec's enumeration-`ootest` check | 1 | 0, exit 1 |
| item 6a, the class-side twins | 4 | tree-wide returns only the two `#define`s at `Setup.cpp:360`/`:361`; non-preprocessor count 0; `RemoveMethod(` 9 and `HideMethod(` 12, running `:792` to `:1404` |
| `svn info` on the three trees | 4 | r13178 / r13198 / r13198, and `E155007` on `oodocs` |
| the five gate commands | 5 | below |

```
cargo fmt --all --check                                             EXIT=0
cargo clippy --workspace --all-targets -- -D warnings               EXIT=0
cargo test --release --workspace                                    EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   EXIT=0   106 of 106 matching
```

No `FAILED` line in any of the three test logs. `REXX_PHASE_GATE=5a …` not run and not asked for.
No performance sitting owed: `docs/` only.

### My own probes, run independently of the ledger's

* the four rule-breaks and the `BOUNDS` reorder above;
* the fourth-form demonstration (two one-line spec files differing only in a wrap);
* an instrumented copy of the extractor's regex over the spec, classifying **every** `:\s*\d+` token
  in the file. The regex sees all of them; 35 are dropped for want of a carry, and reading all 35 with
  their surrounding two lines is how N1's three were found. The other 32 are `.orx`, `.xml`, `.rs` and
  roadmap citations that are correctly not C++;
* the `cls` section census at three pattern widths (N7);
* `rexxpg/classes.xml`'s chapter bounds;
* `interpreter/` reads for every C++ fact this round added.

**Before every negative above, the pattern.** N4's negative is *"narrowing `ANYFILE` to
`(?:cpp|hpp)` changes no byte of the 90-row output"* — not *"the rule is useless"*, which it is not
once the reset is relaxed. N5's is *"no ellipsis in the ledger sits inside a command"*, checked by
reading all eleven ellipsis lines, not by the report's five-word grep. N7's is *"the `cls` section set
is invariant across `<section id="cls`, `<section[[:space:]]+id=["']cls` and a case-insensitive
`<section[^>]*id=["']cls`"* — three widths, not one.

---

## What I could not check

* **Whether the authorities the ledger lists are all there are.** Unchanged and unchangeable from
  inside the document; its own header says so better than a reviewer can.
* **Whether a fifth citation form exists.** I enumerated every `:\s*\d+` in the spec and classified
  each, so within *that* token shape the answer is complete. A citation written without a colon —
  "`Setup.cpp`, line 1809", or a symbol named with no number at all — is outside it, and I did not
  search for those. That is the same axis the whole task keeps being wrong on, and I am as exposed to
  it as the document.
* **Whether the ledger's readings happened.** "Read end to end" is not checkable by any instrument;
  only the derived facts are, and those reproduce.
* **`class.testgroup.cls`'s assertions under ooTest.** I read the file and re-derived its `nop`
  bodies; I did not run the framework.
* **The rows this round did not touch.** `:116`-`:123`, `:125`, `:127`-`:136`, `:138`, `:140`-`:145`,
  `:147`-`:152`, `:154`, `:155`, `:160`, `:169`, `:171` were re-run only as counts; their prose was
  approved in earlier rounds and is not reopened. The three in-tree Rust citations and the oracle
  reproductions at `:440` and `:470` are unchanged in this diff and were not re-run.
* **No oracle run this round.** Nothing in the diff moves an oracle-derived claim; the word-diff of
  every changed mapping row shows the substantive edits are confined to the search form.

No file was edited except this review. No subagents were dispatched.
