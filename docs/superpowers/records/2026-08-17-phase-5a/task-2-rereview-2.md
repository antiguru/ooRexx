# Task 2, fix round 2 — re-review

Scope `fa244be1a..1b536079a`, one commit, one file
(`docs/superpowers/plans/phase-5-reading-ledger.md`, 115 insertions / 31 deletions, no code).

**Verdict: REWORK.** The seven findings are closed and the round's own ruling holds where it was
aimed: every command the ledger now states was extracted from the file's raw bytes and re-run, and
all but one reproduce exactly. The rework is the promoted observation. It was ruled *"re-run the
conservation check against the spec's citations … and place every one"*; the re-run happened, the two
it found are right, and it still misses one — `ClassClass.cpp:988`-`:990`, cited by the spec at
`:946`, in no list — while the ledger now asserts in its own words that there is nothing left to
place.

---

## Per-finding

| # | | verdict |
|---|---|---|
| N1 | HIGH — `class.testgroup.cls` and activate ordering unnamed | **CLOSED** — every cited line resolves; ordering is now named; the false extension-bound sentence is gone |
| N2 | MEDIUM — `:137`'s intersection vacuous under BRE | **CLOSED** in substance — `-ariEl`, 27 files, exactly one intersection member, re-run; but the command as written will not paste (D3) |
| N3 | MEDIUM — `awk` printed cumulative `NR` | **CLOSED** — `FNR`, third path corrected, block run verbatim and its output matches |
| N4 | MEDIUM — row `:120`'s `(:167)` | **CLOSED** — `:146` with `:154`-`:157`; the `:1248` decoy correctly excluded |
| N5 | LOW — `:153`'s enumeration incomplete | **CLOSED** — three files with `:154`, `:160`, `:83`, all resolved |
| N6 | LOW — list (a)'s criterion false of its `Setup.cpp` members | **CLOSED** — criterion restated; the two range ends are pointed at the paragraph that covers them |
| N7 | LOW — `\|` in table patterns | **CLOSED by relocation** — no pattern or table row carries `\|`; all nine block patterns paste from raw bytes and run |
| obs | `Setup.cpp:325-329` in neither list | **OPEN** — the two named are placed and right; the re-run check still leaves one spec citation unplaced, and the ledger now claims none are (D1) |

---

## New defects, most severe first

### D1 — the conservation check is still narrower than the sentence it licenses, and the sentence is now absolute

Ledger `:120`-`:121`: *"With those placed, every C++ `file:line` the spec cites appears in (a), (b),
(c) or the found-wrong table, and the check that says so is run over the spec."* Authority row `:36`:
the stopping point is *"every `file:line` the spec cites, verified"*, status **done — every citation
resolved**.

Spec `:946` reads:

> `~method` reads `instanceMethodDictionary`, the methods "defined at this level"
> (`ClassClass.cpp:984`, comment at `:988`-`:990`).

`ClassClass.cpp:984` is in (a). **`:988`-`:990` is in no list.** Measured: `/bin/grep -an
':988\|:989\|:990'` over the ledger returns nothing, and a re-derived conservation check — harvest
every `(file, :N)` pair from the spec per line, carry the filename across the ledger's wrapped list
lines, subtract — leaves exactly this one after the two the round placed. So *"exactly two came back
unplaced"* is a count taken with an extractor that could not see this shape.

**Why it was missed, and the shape is the round's own.** The stated extraction is *"every
`File.(cpp|hpp):N` in the spec, plus every bare `name :N`"*. `:988`-`:990` is neither: it is a bare
`:N` continuation of an *earlier citation in the same parenthesis*, not of a function name. A third
form the pattern does not reach — the search narrower than the sentence, on a new axis, in the fix
for the previous instance of it.

I resolved it myself: `ClassClass.cpp:984` is `MethodClass *RexxClass::method(RexxString *method_name)`
and `:988`-`:990` is the three-line comment beginning `// we keep the instance methods defined at this
level in a separate`. **The citation is correct.** The defect is not the citation; it is that the C++
authority row is marked `done` against a stopping point that a complete run of its own check does not
reach.

**Consequence.** Task 3's staleness check and every later task that treats the C++ reading row as
closed inherit "every spec C++ citation verified" as a fact. One was not verified by this document,
and the next reader has no way to tell which — the completeness sentence is exactly what stops them
looking.

### D2 — the widening's own extension enumeration is wrong, and `--include=*.cls` misses a `.cls`

Ledger `:198`-`:201`: *"`ootest/` is not only `*.testGroup`. Measured … the test-carrying extensions
are `.testGroup` 409, `.rex` 49, `.cls` 9, `.testUnit` 7, `.oodTestGroup` 2."*

The `find`/`sed`/`sort`/`uniq` line reproduces those five figures exactly. The **enumeration** does
not survive its own output. The same command also prints, and the paragraph drops:

* `ootest/framework/OOREXXUNIT.CLS` — 2289 lines, the ooTest framework itself, first line
  `#!/usr/bin/env rexx`. **`grep --include='*.cls'` does not match it**: measured,
  `/bin/grep -aril --include='*.cls' 'testcase' ootest/framework/` exits 1 and the same command with
  `--include='*.CLS'` returns the file. The glob is case-sensitive and the stated set is written in
  lower case only;
* `ootest/ooRexx/base/rexxutil/Macrospace.norex` — first line `/* Macrospace.testGroup test routine */`;
* `ootest/ooRexx/base/keyword/search_order.other`, `API/oo/path/callPath.test1`, `test1.test1`,
  `test2.test2` — Rexx programs that are the fixtures for the CALL and `::REQUIRES` search-order
  tests, i.e. for row `:153`'s subject.

This is N1's mechanism restated one notch out: the fix for a too-narrow extension filter is a
slightly-less-narrow extension filter, stated as though it were the tree.

**I checked whether it costs any conclusion, and it does not.** Every negative in the mapping re-run
with **no `--include` at all** (`--exclude-dir=.svn`, patterns extracted from the raw file):
`:124` name and `:124` effect exit 1; `:137` phrasing exits 1; `:139` returns the same three files;
`:146` the same four; `:153` the same three; `:170` the same one; `:126` the same four with
`:126-names` exiting 1; and `:137`'s intersection goes 27 files → 29 with the same single member,
`class.testgroup.cls`. **So the widening genuinely happened and the negatives survive the widest
search available** — which is worth saying, because it is the thing the brief asked for and it is
true.

**Consequence.** The plan's three-signal rule makes signal 2 *"`ootest` does not pin it, checked at
`ootest/` directly"*, and this line is the stated procedure for that check. A later task applying it
searches a set that excludes the framework `.CLS` and the search-order fixtures, and believes it has
searched `ootest`. Naming the set rather than five globs — or `--include='*.[cC][lL][sS]'` and the
fixtures — is the fix.

### D3 — row `:137`'s command is the one in the ledger that does not run as written

Extracted from raw bytes, `:243` states:

```
/bin/grep -ariEl … '::method[[:space:]]+.?init[[:space:]]+class' ootest/
```

Every other row that takes the widened set writes the literal `<extensions>`, which the preamble at
`:208` names: *"Where a row writes `<extensions>` inside a command, substitute that line"*. This row
writes `…`. Pasted verbatim from the file it is

```
/bin/grep: ::method[[:space:]]+.?init[[:space:]]+class: No such file or directory
EXIT=2
```

— `…` is taken as the pattern and the pattern as a path. The preamble's substitution rule is keyed on
a token that is not present, so neither reading of the row produces a runnable command.

The report says *"Every command now in the ledger was run in that form before it was written down"*.
For this one that is not so; it was run with the includes and written with an ellipsis.

**Consequence** is small but is the round's own subject: exit 2 with a message naming the pattern as a
missing file is a distinct signal from the exit 1 the neighbouring negatives use, and it is the
document's one row where a paste fails.

### D4 — LOW. `:137`'s `INIT`-before-`INHERIT` negative does not say why its single hit does not count

The row narrows the intersection to exactly one file and then concludes *"no test found asserts the
spec's own discriminator, `self~hasMethod("MM")` answering 0 in `init` and 1 in `activate`"*. The
disqualifying fact is not stated: `class.testgroup.cls`'s three `::method init class` bodies are each
a bare `nop` (`:14`, `:35`, `:51`), so the one file that carries both directives asserts nothing about
`init`. Everywhere else the round's discipline is to state the search *and* what its hits are; here a
search that returns a hit is used to support a negative, and the reader has to open the file to learn
why.

---

## Every stated command I extracted and re-ran, and what it returned

All extracted with `sed -n` from `docs/superpowers/plans/phase-5-reading-ledger.md`'s raw bytes —
never from a rendered view — and run from the repository root. `ootest` r13178, `oodocs/rexxref` and
`oodocs/rexxpg` r13198 (`svn info`, checked today). Where a pattern lives in the `|`-block at
`:275`-`:283` it was pulled out of that block by `sed`, not retyped.

**Re-ran 26. Reproduced 25. One did not execute at all (D3).**

`<ext>` below is line `:205` verbatim:
`--include=*.testGroup --include=*.rex --include=*.cls --include=*.testUnit --include=*.oodTestGroup`.

| # | stated command, as extracted | stated | got |
|---|---|---|---|
| 1 | `find ootest -type f -not -path '*/.svn/*' \| sed 's/.*\.//' \| sort \| uniq -c \| sort -rn` | testGroup 409, rex 49, cls 9, testUnit 7, oodTestGroup 2 | **identical** — and the same output also names `CLS`, `json`, `norex`, `other`, `test1`, `test2` (D2) |
| 2 | `/bin/grep -aril <ext> 'inheritInstanceMethods' ootest/` | exit 1 | **exit 1** |
| 3 | `/bin/grep -ariEl <ext> '::method[[:space:]]+.?unknown' ootest/` | `Object`, `SecurityManager`, `SysUnicode` | **those three** |
| 4 | `/bin/grep -ariEl <ext> '::method[[:space:]]+.?makeString' ootest/` | `API/oo/METHOD`, `API/oo/FUNCTION`, `extensions/json`, `base/class/Array` | **those four** |
| 5 | `/bin/grep -ariEl <ext> 'Compiled method' ootest/` | one file, `incorrectCharacters.testGroup` | **that one** |
| 6 | the same, counted on that file | 6 lines | **6** |
| 7 | `/bin/grep -ariEl … '::method[[:space:]]+.?init[[:space:]]+class' ootest/` | 27 files | **exit 2, "No such file or directory"** as written (D3); **27** with `<ext>` substituted |
| 8 | the intersection, `/bin/grep -aqiE '::method[[:space:]]+.?activate'` over those | exactly one, the `.cls` | **exactly `ootest/ooRexx/base/class/class.testgroup.cls`** |
| 9 | `/bin/grep -aci 'hasMethod'` on `base/class/Stem.testGroup` | 0 | **0** (exit 1) |
| 10 | block `:124` `(supplier\|\.set\|\.bag\|\.relation)[^;]*~hasMethod\|hasMethod\("(ALLITEMS\|ALLINDEXES\|SUPPLIER)"` over `ootest/` | no file | **exit 1** |
| 11 | block `:126` `assertFalse\([^)]*hasm(ethod)?` over `ootest/` | four files | **`API/oo/METHOD`, `base/directives/ATTRIBUTE`, `base/class/Class`, `base/class/Object`** |
| 12 | block `:126-names` over #11's hits | exits 1 | **exit 1** (21 hit lines in, none matching) |
| 13 | block `:137` `init[^;]*before[^;]*(inherit\|activate)\|activate[^;]*after` over `ootest/` | no file | **exit 1** |
| 14 | block `:143` `~setEntry\|~entry\(` on `SecurityManager.testGroup` | 32 | **32** |
| 15 | block `:152`, same pattern, on `collections/directory.testGroup` | 12 | **12** |
| 16 | block `:153` `search *order\|searchord` over `ootest/` | `REQUIRES`, `CALL`, `Macrospace` | **those three**; `Macrospace` at `:83`, `:154`, `:160` exactly as the row says |
| 17 | block `:154` `addClass\|addPublicClass` on `Package.testGroup` | 52 | **52** |
| 18 | block `:169` `guard (on\|off)` on `GUARD.testGroup` | 60 | **60** |
| 19 | `\breply\b` on `base/keyword/REPLY.testGroup` | 33 | **33** |
| 20 | the `awk` block at `:331`-`:335`, run as a script straight from `sed -n '331,335p'` | `CoreClasses.orx:1590 :1618` under `Alarm`, `:1690`-`:1692` under `Ticker`, `StreamClasses.orx`'s block under `Stream`/`RexxQueue`/`File`, nothing from `PlatformObjects.orx` | **exactly that.** `FNR` now resolves: `StreamClasses.orx` `:510 :546 :547` in a file of 1010 lines, and `CoreClasses.orx` is 4193 — the old `:4703` is reproduced as an artifact of `NR` and is correctly described |
| 21 | `sed -n '114,172p'` over the spec `\| /bin/grep -ac ootest` | 0 | **0** (exit 1) |
| 22 | `cargo fmt --all --check` | EXIT=0 | **EXIT=0** |
| 23 | `cargo clippy --workspace --all-targets -- -D warnings` | EXIT=0 | **EXIT=0** |
| 24 | `cargo test --release --workspace` | EXIT=0 | **EXIT=0**, no `FAILED` line |
| 25 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | EXIT=0, 106 of 106 | **EXIT=0, `106 of 106 matching`** |
| 26 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | EXIT=0, 106 of 106 | **EXIT=0, `106 of 106 matching`** |

`REXX_PHASE_GATE=5a …` not run and not asked for. No sitting owed: `docs/` only.

**The `\|` fix, tested by pasting rather than by reading.** `/bin/grep -anF '\|'` over the ledger
returns two lines, `:268` and `:269` — both the prose *explaining* the convention, neither a table row
nor a pattern. Every pattern in the `:275`-`:283` block is free of `\|` and every one ran on the first
paste (#10-#18 above). A structural pipe count over every line beginning `|` gives 4 or 6 per row with
no odd value anywhere in the file, so no cell carries a stray unescaped pipe either.

### My own widening, run independently of the ledger's

Every negative above re-run with **no `--include` at all** and `--exclude-dir=.svn`. Nothing changes:
#2, #10, #13 still exit 1; #3 the same three; #4 the same four; #5 the same one; #11 the same four
with #12 still exit 1; #16 the same three. #7 goes 27 → 29 files with the intersection still exactly
`class.testgroup.cls`, and a bare `::method[[:space:]]+.?activate` search over the whole tree with no
filter returns 25 files of which that `.cls` is the only one also carrying an `init class`.

### N1's citations, resolved at the source

`ootest/ooRexx/base/class/class.testgroup.cls`, 70 lines, read in full. `::class class1 subclass
class3` `:12`, `::class class2` `:33`, `::class class3` `:49`; each with `::method init class`
(`:13`, `:34`, `:50`, every body `nop` at `:14`/`:35`/`:51`) and `::method activate class` (`:15`,
`:36`, `:52`). Existence `:25`-`:27`, `:43`-`:45`, `:59`-`:61` — each the three `~isa(.class)` checks
with their own message. Ordering `:20`-`:23`, `:38`-`:41`, `:54`-`:57`, and read as a graph they give
**class2 → class3 → class1** exactly as claimed: class2's activate asserts neither class3 nor class1
has run, class3's asserts class1 has not and class2 has, class1's asserts both have. Construction
inside activate `:16`-`:18` — `instance = self~new`, `-- this should not give an error`,
`instance~foo`. Prolog `:5`-`:9`, guarded by `assertFail \= .true` at `:5`. Comments `:11`, `:32`
("this class will run first because this is the first class without a dependency"), `:47`-`:48`.

`base/class/Class.testGroup`: `:962` `::method test_activate`, `:964` `.local~class.testgroup =
.directory~new`, `:966` `.context~package~loadPackage("class.testgroup.cls")`, `:968` the
`assertFail` assertion carrying `assertFailReason` as its message, `:970`-`:972` the three flags. All
exact.

**N4.** `:146` `::method test_issubclassof`, its four `.Amphibianvehicle~issubclassof` assertions at
`:154`-`:157` — the report's correction of the previous review's `:153`-`:156` is right, `:153` is
blank. `:167` is `test_issubclassof_non_class` (`expectSyntax(88.914)`, `.vehicle~issubclassof("object")`).
The decoy `:1248` sits under `::CLASS "WasserFahrzeug" SUBCLASS Fahrzeug` (`:1229`) with its own copy
at `:1255`-`:1258`. `:1276` is the directive-declared `AmphibianVehicle`; `:1245`'s `~show_off` is
inside `/*` `:1241` … `*/` `:1246`. `test_BASECLASS` `:172`, `:177`
`self~assertSame(.AmphibianVehicle, .AmphibianVehicle~baseclass)`.

### The (c) table and the conservation re-run

`interpreter/memory/Setup.cpp` `:325`-`:334` read: `:325`-`:326` the two-line comment, `:327`
`TheCommonRetrievers = new_string_table()`, `:328` blank, `:329`-`:330` the comment, `:331`/`:332` the
`SELF`/`SUPER` puts. The (c) row is exact and the spec's own correction is right.

`:790`-`:812` read: `:795` blank; `:796` `// to be consistent with our other Collections, also`;
`:797` `// - remove all four sort methods …`; `:798` `// - remove makeString, toString`; `:799`-`:804`
the six `RemoveMethod` calls; `:806` `CompleteMethodDefinitions();`. The spec's `:795`-`:797` does
start on a blank and does stop before the `makeString` line, and the ledger's correction to
`:796`-`:798` is right.

Re-derived independently, the residue is D1 and nothing else. Screening every citation in the spec's
enumeration table (`:114`-`:172`, implementation column) against every `:N` in the ledger produces no
miss; screening the spec's prose the same way produces `ClassClass.cpp:988` and `:990` and one false
positive from a `07:06` timestamp at spec `:1401`.

### Other sources touched

`interpreter/RexxClasses/StreamClasses.orx` `:510` `::method file_temporary_path class private
external "library REXX"`, `:546` `getSeparator … file_separator`, `:547` `getPathSeparator …
file_path_separator` — the double-quoted three, and the last two are indeed the pair D37 implements.
`interpreter/platform/unix/PlatformObjects.orx` is the single line `-- Nothing to do currently`.
`CoreClasses.orx` `:1590` `alarm_startTimer`, `:1618` `alarm_stopTimer`, `:1690`-`:1692` Ticker's
three. `ClassClass.cpp:984` and its `:988`-`:990` comment, quoted above.

---

## What I could not check

* **Whether the authorities the ledger lists are all there are.** Unchanged, and D1 is a small
  instance of it inside a set the document *can* enumerate. I widened only along the axes the round
  named; a citation form neither my extractor nor theirs recognises would look exactly like a placed
  one.
* **Whether `class.testgroup.cls`'s assertions pass upstream.** I read it and resolved its loader; I
  did not run ooTest. Its failures route through `call setError` and surface at `Class.testGroup:968`.
* **The rows this round did not touch** — `:116`-`:119`, `:121`-`:123`, `:125`, `:127`-`:135`, `:138`,
  `:140`, `:141`, `:142`, `:144`, `:145`, `:147`, `:148`, `:150`, `:151`, `:155`, `:160`, `:171`, and
  the counts the previous round re-ran and confirmed. Not reopened; I re-ran only counts whose row
  changed in this diff (`:126`, `:143`, `:152`, `:153`, `:154`, `:169`, `:170`).
* **The `Alarm` transcript prediction.** Not runnable in Phase 5 by construction, and labelled.
* **Whether the three in-tree Rust citations were right when the spec was written.** Same standing
  limit; unchanged by this round.
* **No oracle run this round.** The two the previous review reproduced (`:120`'s discriminator and
  `:153`'s `test_package_local`) are unchanged in the diff, so re-running them would measure the same
  thing twice.

No file was edited except this review. No subagents were dispatched.
