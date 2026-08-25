# Task 5 review -- gate table C, the concept and class surface

Scope: `fe4c95913..0e605eb77`, two commits. Reviewed on the committed tree at `0e605eb77`, `git
status` clean at the start and at the end of the review.

**Spec compliance: APPROVE.** Everything the brief asks for is built, the counts are right, M9 is
disposed of, the three structural controls fire, the four verdict mutations are named against their
owning tasks in files those tasks' authors will read, and all five gate commands exit zero with the
corpus at 106 of 106.

**Task quality: REWORK.** Seven findings, two of them major. Both majors are places where a row or a
whole family of rows reads `agree`, or passes a structural check, with nothing having been asked --
and both are demonstrated by a control I ran rather than argued from the code. The most important is
that a **class wiring row and a hierarchy edge row read `agree` when both interpreters fail
identically**, which I showed live: a synthetic row for a class that exists in neither interpreter
came out green and was counted as a satisfied 5a row.

---

## Findings, most severe first

### 1. MAJOR -- a wiring row, an edge row and a concept row read `agree` when both sides fail identically, and nothing checks that the oracle answered

`run_probe` hands three descriptors to `verdict()` and nothing anywhere asks whether the oracle
produced the output the probe was derived to produce. For the **method** family the task built
exactly that check -- `expected_oracle_lines` at `crates/rexx-exec/tests/gate_table_c.rs:630`, whose
own doc says "without it a probe that silently stopped asking half its names would leave those rows
comparing an absent line against an absent line, which reads `agree` for a question nobody asked".
The reasoning is not applied to the class, edge or concept families, where the derived text pins the
expected line count exactly: six `say`s for a class row, four for an edge row.

**Demonstrated, control 12/13 below.** I appended a synthetic `Zork` class row, a `Zork <- Object`
edge row and three `Zork` instance method rows to the committed row sets, with correctly derived
probes. `.Zork` is not a class in either interpreter, so `.Zork~id` raises `97.1 Object ".ZORK" does
not understand message "ID"` on both sides -- byte-identical stderr, identical rc 159, empty stdout.
The table exited **0** and reported:

```
  agree          loud=no  5a   Zork                   class not-covered fundclasses.xml:1  zork.rex
  agree          loud=no  5a   Zork <- Object         provide.xml:1                        zork__object.rex
  Zork           instance  loud=no  5c   3 row(s): agree=3                                 zork__instance.rex
  5a: 137 rows, 135 not yet `agree`
```

Two gated 5a rows went green over a class that does not exist, and three 5c method rows went green
without a single `hasMethod` question being asked.

**Concrete consequence.** Task 9's "Done when" (plan line 1054) is *"the wiring rows for the classes
this crate registers read `agree`"* and Task 24's (line 1879) is *"every 5a row in both tables reads
`agree`"*. A `class-set.txt` row naming a class this oracle build does not ship satisfies both
criteria while neither interpreter can answer any of the six wiring questions. That is not
hypothetical: `class-set.txt`'s own header records `RegularExpression` being **removed by hand** for
exactly that reason ("`::requires "rxregexp.cls"` ... not on this build's search path, measured
43.901 rc 213"). Had it been left in, its wiring row would be green today. The same holds for a
documented hierarchy edge whose child the build does not ship.

**Not introduced here.** Table D has the same property -- `gate_table_d.rs:404` builds its verdict
from `compare_raw` alone, and `corpus/gate-tables/README.md` states the intent ("the oracle side
shows the program ran rather than that it produced nothing") with no check behind it. What makes it
this task's to answer is that Task 5 multiplied it by 120 wiring rows and 21 concept rows *and*
wrote the counter-measure for its own fourth family, so the omission is both visible and a
three-line generalisation: the class and edge probes' derived text already fixes the expected line
count, so the same `oracle_lines.len() != expected` structural push applies unchanged.

**What the check I ran would have done had the claim been false:** the synthetic rows would have
appeared as `diverge-*`, or the run would have exited 101 on a structural failure. It exited 0 and
printed `agree`.

---

### 2. MAJOR -- `check_probe_text` silently skips any probe whose bytes are not valid UTF-8, defeating the task's headline contribution

`gate_table_c.rs:822`:

```rust
let Ok(committed) = fs::read_to_string(&path) else {
    // Missing is the set check's case, already reported there.
    return;
};
```

The comment names one of the reasons `read_to_string` fails. It also fails on **invalid UTF-8** and
on a permission error, and in both of those the file *exists*, so the set check passes and the text
check reports nothing at all.

**Demonstrated, control 6 below.** I replaced `corpus/gate-tables/classes/array.rex` with the same
header, a body reading `say 'id' .String~id`, and one trailing `0xff` byte. The table exited **0**.
That is precisely the mutation the commit message singles out as the reason the derivation exists --
"`classes/string.rex` holding `say .Array~id` passes a path check, agrees with the oracle byte for
byte, and reports String green while nothing has asked about String" -- and it is defeated by a
single non-ASCII byte, silently.

**Concrete consequence.** The derivation check is the only thing standing between a wiring row's
verdict and a program about a different class. One byte turns it off for that file with no
diagnostic, and the probe still runs and still produces a verdict. `run_on_both_engines` reads bytes
(`fs::read`), so the program executes normally; the hand-run provenance sweep would not catch it
either, because the program runs fine.

The fix is local: match on the error and push a `Structural` for anything that is not
`ErrorKind::NotFound`.

A weaker sibling, same shape, worth fixing in the same edit: `probe_set` at `gate_table_c.rs:796`
filters directory entries through `entry.file_name().into_string().ok()`, so a probe file whose
**name** is not UTF-8 is dropped from the listing and can never be reported as an orphan.

---

### 3. MEDIUM -- `expected_oracle_lines` turns the `status` column into a structural assertion about the oracle that the column is documented not to make

`expected_oracle_lines(arm, status, rows)` reads `status == "covered"` as "the oracle's bare `~new`
constructs and the probe prints one line per row" and anything else as "the oracle's `~new` raises
and prints none". Both directions are stronger than what the column carries:

* `corpus/docs/class-set.txt`'s header: "`not-covered` says no construction program is committed and
  **CARRIES NO CLAIM ABOUT THE ORACLE**". The plan says the same at line 615.
* `covered` is a **disjunction**: "a class in the set a bare `~new` constructs, **or one opted in with
  a committed construction program**" (plan line 614; header: "none is committed today").
  `method_probe_text` derives a bare `o = .X~new` for every `covered` class, with no route for a
  construction program.

**Concrete consequence.** The first task to commit an opt-in construction program for a class whose
bare `~new` raises -- the case `covered`'s definition exists for -- flips that class to `covered`,
leaves the derived probe on a bare `~new`, and the oracle then prints zero lines where the table
expects `rows`. That is a **structural** failure: red in every mode, including
`cargo test --release --workspace` and both corpus-gate commands the global constraints require to
exit zero. The message a maintainer meets ("the program and the row set have drifted") points away
from the cause.

`expected_oracle_lines`'s doc restates `covered` as "a class a bare `~new` constructs", which is one
limb of the definition presented as the whole of it.

**Related, and disclosed but understated.** 497 method rows across 23 groups (measured: instance-arm
rows whose class is not `covered`) sit on the `expected == 0` path, where `agree` means only that the
two sides raised alike at `~new`. The report says these rows "measure the constructor, not the method
set", which is right; what it does not say is that the visible outcome is **`agree`** -- so when 5b
lands `~new`-parity those 497 rows go green at once and a reader of the verdict summary sees a third
of the method surface turn green with no `hasMethod` question asked. Control 12 demonstrates the
mechanism (3 of 3 synthetic rows `agree`).

---

### 4. MEDIUM-LOW -- the "first differing line" diagnostic is empty for whitespace-only differences, which is the shape the report says control 4 fixed

`first_difference` compares `str::lines()`, which strips the line terminator and a trailing `\r` and
ignores whether the file ends in a newline. So a committed probe that differs from its derivation
*only* in line endings or in a final newline produces:

```
gate-tables/classes/array.rex: the committed probe is not what its row derives. ...
    line 13 derived:   ""
    line 13 committed: ""
```

**Demonstrated twice**, controls 9 and 10: stripping the final newline, and converting the file to
CRLF. Both exit 101 with that message.

The report describes fixing exactly this failure mode ("a check that ran, exited non-zero, and told a
reader nothing -- the shape worth catching"). The fix closed the case where a line's *text* differs
and left open the case where the difference is invisible to `lines()`. There is no `.gitattributes`
anywhere in this repository, so a CRLF checkout would fail all 216 derived probes at once with this
non-message.

---

### 5. LOW -- "All 32 `agree` rows in this table exist because of that split" is false, and one copy of it is in an uneditable commit message

Report ("Design decisions worth a reviewer's attention") and commit `0e605eb77`'s message:
"**All 32 `agree` rows in this table exist because of that split** -- under the letter of the rule the
table would read 0 `agree` out of 1488".

Measured: `Buffer`, `Singleton` and `Validate` have **class-arm rows only** -- no instance arm exists
in `class-methods.txt`. So under the letter of "one program per class", `buffer.rex` is byte-identical
to today's `buffer__class.rex`; there is no `~new` to poison it. And the `Buffer new` row is one of
the 32 that agree (oracle `class 1`, crate `class 1`, verified by running the probe on both sides).
The table would read at least 1 `agree`, not 0.

The decision is right and the measured reason for it (Array's 46 rows) stands; the justification is
overstated by one row. Recording it because this project tracks the "false justification rides a
correct decision" pattern and because one copy is now unamendable.

---

### 6. LOW -- "every one of the six [5b concept rows] because its section's distinguishing claim needs an instance" is false for `obdes`

Report, "What each row asks". Five of the six do need an instance (`objcla`, `abscla`, `usesem`,
`creo`, `methodsbyclass`). `obdes.rex` deliberately has none -- its own header says "A class object is
destroyed like any other object, so a class-side UNINIT fires with no instance anywhere" -- and the
committed `CONCEPTS` authority for it gives a different reason entirely ("spec enumeration: object
destruction and uninitialization ... is 5b"). The committed code is right; the report's summary of it
is not.

---

### 7. LOW -- the `reason` column is interpolated into a Rexx block comment with no escaping

`method_probe_text`'s not-covered arm writes `class-set.txt`'s free-text `reason` into a
`/* ... */` comment. No reason contains `*/` today (checked). A future one that did would close the
comment early and leave the rest of the header being parsed as Rexx. Because the derivation *is* the
definition, the committed file would match and `check_probe_text` would pass; and for a not-covered
instance group `expected_oracle_lines` is 0, so a program that now prints nothing on both sides would
leave every row of that group reading `agree` -- the same path as finding 1.

---

## What is right, and how I checked it

**Counts, re-derived.** Predicate: lines that are neither blank nor start with `#`, which is what
`read_table` implements.

| file | rows |
|---|---|
| `provide-sections.txt` | 21 |
| `class-set.txt` | 63 |
| `hierarchy-edges.txt` | 57 |
| `class-methods.txt` | 1347 |
| **total verdict rows** | **1488** |

Probe programs: `concepts/` 21, `classes/` 63, `hierarchy/` 57, `methods/` 96 -- **237**. (`directives/`
holds 79 more; those are table D's and are not in this count. `find` over `gate-tables/` returns 316,
which is 237 + 79.) 96 groups matches the 96 distinct (class, arm) pairs exactly. 62 distinct classes
have method rows; `ArgUtil` is the only class-set row with none, which its own header records.

**No filename collisions and no unhandled name shapes.** No two class names collide after
lower-casing; no two (child, parent) pairs do; no duplicate (class, method, arm) row; no method name
contains a double quote; only two arm values (`class` 134, `instance` 1213). The two placeholder
names are handled and I verified the mapping against the oracle: `.Object~method("")` and
`.Object~method(" ")` both answer `The Method class`, and `"abc"~hasMethod("")` and `(" ")` are both
`1`.

**The 32 agreeing rows redden under mutation -- all of them, not a sample.** See mutations A and B
below. They are also not vacuous agreements: every one answers `1` on both sides (I ran all 17
agreeing class-arm groups through both interpreters), and the crate discriminates --
`.Array~hasMethod("zork")` is `0` on both sides.

**Per-row attribution inside a shared program is exact.** Mutation B perturbed one method name only
and precisely the rows for that name reddened, in four separate groups, with the row-by-row listing
naming the right method.

**The three structural controls the brief names all fire, with the messages the report quotes** --
re-run by me, controls 1, 2, 3a and 3b. So do the two extra ones (4 and 5). Control 2 is a genuine
two-engine divergence in the crate's own `say_evaluated`, and it reddens on a table C row before any
verdict exists.

**The `ArgUtil` assertion fires, and in the shape it exists for it is the only thing that fires.**
Control 5b: with the `ArgUtil <- Object` edge added *and* a correctly derived probe committed for it,
the assertion is the sole structural failure. I confirmed the premise on the oracle:
`.ArgUtil~superClasses` is `The Object class` and `~hasItem(.Object)` is `1`, so an edge row for it
would indeed read `agree` over a wrong member set. The other half fires too (control 11).

**The four verdict mutations are named where their owners will meet them**, which matters because
`.superpowers/` is git-ignored (`git check-ignore` confirms) and the report is therefore invisible to
the next task's author. Each is named in the **committed** plan and, for the three that fire on a
table C row, in the **committed** `CONCEPTS` table's `control` field:

| mutation | plan | code |
|---|---|---|
| 1 drop a class from the registry | Task 9's "Done when" (line 1054) plus its bulleted controls; Task 21 named there for the deferred classes | `classmeth` row's `control` |
| 2 flattened all-scopes `~method` | Task 9, same bullet list, explicitly *not* on a table C row | `xscope` row's `control` |
| 3 `makeString` answering the wrong string | Task 14: "table C's **mutation 3**, which Task 5 also owns to this task", in its "Done when" | `reqstr` row's `control` |
| 4 drop the operator-frame line | Task 6: "Negative control, and it fires on the corpus rather than on table C", in its "Done when" | module doc, "The operator-frame traceback line ... Its instrument is Task 6's corpus programs" |

D50's two deletion controls are likewise carried by Task 12's and Task 14's own "Done when" clauses.

**M9's disposal is right.** `corpus/lang/primitive_classes.rex` is gone; `corpus/gate-tables/concepts/classmeth.rex`
is the `classmeth` row's probe and is run by the table. The rename diff shows a header comment added
and nothing else, so the **pinned instruction count of 31 is unchanged and still asserted**
(`the_corpus_programs_parse` passes in the workspace run). The parse-fixture table's generalisation is
correct: keys are corpus-relative paths with extensions, all three messages read `corpus/{name}`, and
`the_keyword_as_variable_corpus_parses_every_keyword_as_a_variable`'s `CORPUS[0].0` assertion was
updated to match. The only remaining textual reference to the old name is in
`corpus/gate-tables/README.md`, where it is deliberate history.

**The gate numbers.** Confirmed independently, each status read unpiped:

| command | exit | note |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| `cargo test --release --workspace` | 0 | |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 | corpus **106 of 106** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 | corpus **106 of 106** |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101** | expected; stops at gate table C |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --test gate_table_c` | **101** | **135** gated rows |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --test gate_table_d` | **101** | **10** gated rows |

135 = 15 concept rows owned by 5a + 63 class + 57 edge. Table D reports 5a 36 rows / 10 open. Total
171 / 145, as the report says. `rexx-diff --cpp X --rs X --corpus corpus`: **440 programs, 0
divergences, exit 0**.

**No sitting is owed.** Neither commit touches `src/` of `rexx-exec`, `rexx-core`, `rexx-classes` or
`rexx-lib`. `crates/rexx-parse/src/instruction/tests.rs` is reached through
`crates/rexx-parse/src/instruction.rs:2714`'s `#[cfg(test)]`, so the release binary the axes measure
is unchanged.

**Global constraints.** No non-ASCII byte in `gate_table_c.rs`, in `corpus/gate-tables/README.md`, or
in any of the 237 probes. No comment in `gate_table_c.rs` names the size of a set -- the only numerals
in its doc comments are measurements, phase labels, task numbers and one quotation of the roadmap
criterion being replaced. No `unsafe`.

**The concept probes I read are faithful to their sections.** I read all 21 and checked `typcla` and
`objcla` against `oodocs/rexxref/en-US/provide.xml` at the cited lines. Several are the section's own
worked example (`chsrod`'s Account/Savings, `creo`'s `.savings~new(1000.00, 6.25)`, `usingcl`'s
Persistence/myarray, `methodsbyclass`'s `matrix[2, 3] = 0`). I re-measured the three claims the report
leans on hardest, three descriptors read separately: `obdes` is oracle `main\nuninit ran\n` against
crate `main\n`, both rc 0, both stderr empty; `unkno` is oracle rc 0 `unknown: ZORK with 2
argument(s): 1 2` against crate rc 159 with a 97.1; `reqstr` is oracle rc 0 opening `K says hello`
against crate rc 120 `The K class`. All three match the report.

---

## Every control and mutation I ran, and what it returned

The real repository was never modified. All mutations were applied to a copy of `rust/` at
`.../scratchpad/t5rev/repo/rust` with a symlink supplying `interpreter/` for `rexx-classes`'s build
script. The copy's unmutated baseline reproduces the release run's verdicts exactly (agree 32 /
diverge-both 1434 / diverge-stdout 22 / loud 1432), so it is a valid platform; the copy is a debug
build, which is stated rather than implied. After every control the file was restored from a
scratchpad copy and the baseline re-run; the final restored run reads 32 / 1434 / 22 again.

| # | what was changed | result |
|---|---|---|
| baseline (real repo, release) | -- | exit 0; agree 32, diverge-both 1434, diverge-stdout 22, loud 1432 |
| baseline (scratch copy, debug) | -- | identical |
| **A** | `native_has_method` (`dispatch.rs:952`) returns `usize::from(!answers)` | exit 0; **agree 32 -> 21**. Every one of the 32 reddened; the 21 now agreeing are exactly the 21 that diverged before (String 16, plus `of` on Directory, IdentityTable, Relation, StringTable, Table). diverge-stdout 22 -> 33 |
| **B** | `let answers = answers && name != "OF";` | exit 0; **agree 32 -> 28**. Array, Bag, List and Set each became `agree=1 diverge-stdout=1`, and the row-by-row listing names `of` as the diverging row and `new` as the agreeing one in all four |
| hand probe | `.Array~hasMethod("zork")` | `0` on both sides -- the crate discriminates rather than answering `1` |
| hand probe | all 17 agreeing class-arm groups, both interpreters | every agreeing line is `class 1` on both sides; no `0 == 0` agreement |
| **1** | `rm corpus/gate-tables/classes/array.rex` | exit **101**, `gate-tables/classes/array.rex: a row has no probe program...` |
| **2** | `say_evaluated` gains `if matches!(self.engine, Engine::Ir) { self.out.push(b'!'); }` | exit **101**, `the two engines disagree on .../concepts/xmeths.rex`, tree-walker `"own-class sub m\n..."` vs ir `"own-class sub m!\n..."` -- on a table C row, before any verdict exists |
| **3a** | append `Zork ... covered ...` to `class-set.txt` | exit **101**, `gate-tables/classes/zork.rex: a row has no probe program...` |
| **3b** | delete `Array<TAB>OrderedCollection` from `hierarchy-edges.txt` | exit **101**, `gate-tables/hierarchy/array__orderedcollection.rex: a probe program no row names, so nothing runs it` |
| **4** | `sed 's/\.Array~id/.String~id/' classes/array.rex` | exit **101**, `line 7 derived: "say 'id' .Array~id" / line 7 committed: "say 'id' .String~id"` |
| **5** | append `ArgUtil<TAB>Object<TAB>0<TAB>843` to `hierarchy-edges.txt` | exit **101**; the ArgUtil assertion fires with its own message, co-firing with a missing-probe structural |
| **5b** | same, **plus** a correctly derived `hierarchy/argutil__object.rex` | exit **101** with the **ArgUtil assertion as the only failure** -- the shape it exists for. Oracle confirms the premise: `.ArgUtil~superClasses` is `The Object class`, `~hasItem(.Object)` is `1` |
| **11** | delete the `ArgUtil` row from `class-set.txt` | exit **101**, the assertion's other half: "`ArgUtil` is not in class-set.txt..." |
| **7** | delete `Array<TAB>of<TAB>class` from `class-methods.txt` | exit **101**, two independent instruments: the derivation check (`line 7 derived: "" / committed: say 'class' .Array~hasMethod("of")`) **and** the line-count check ("the oracle answered 2 line(s) where the row set says 1") |
| **8** | delete the `unkno` row from `provide-sections.txt` | exit **101**, `CONCEPTS arm unkno: names a section provide-sections.txt does not carry` plus the orphan-probe report |
| **6** | `classes/array.rex` rewritten to ask `.String~id`, with one `0xff` byte appended | **exit 0 -- finding 2.** The derivation check never ran |
| **9** | strip the final newline from `classes/array.rex` | exit 101 with `line 13 derived: "" / line 13 committed: ""` -- **finding 4** |
| **10** | convert `classes/array.rex` to CRLF | exit 101 with the same empty diagnostic -- **finding 4** |
| **12** | synthetic `Zork` class row (`not-covered`) + three `Zork` instance method rows, with correctly derived probes | **exit 0.** `Zork` wiring row `agree` (5a, gated); `zork__instance.rex` `agree=3` -- **finding 1** |
| **13** | plus a synthetic `Zork <- Object` edge row and its derived probe | **exit 0.** Edge row `agree` (5a); `5a: 137 rows, 135 not yet agree` -- **finding 1** |
| restore | all files restored, baseline re-run | exit 0; agree 32, diverge-both 1434, diverge-stdout 22 -- the copy is back where it started |

---

## Which probes and rows I sampled

**Concept probes** -- read all 21 in full: `typcla`, `objcla`, `xmixin`, `abscla`, `xmetac`,
`xcremet`, `usingcl`, `xscope`, `usesem`, `methna`, `xmeths`, `unkno`, `chsrod`, `pubpri`, `creo`,
`obdes`, `reqstr`, `concurr`, `classmeth` (head), `chi`, `methodsbyclass`. Checked `typcla` and
`objcla` against `provide.xml` at the lines `provide-sections.txt` cites. Ran `obdes`, `unkno` and
`reqstr` on both interpreters with the three descriptors read separately.

**Derived probes** -- `classes/array.rex`, `classes/rexxinfo.rex`; `hierarchy/alarm__object.rex`,
`hierarchy/array__orderedcollection.rex`; `methods/array__class.rex`, `class__class.rex`,
`string__class.rex`, `directory__class.rex`, `method__class.rex`, `buffer__class.rex`;
`methods/object__instance.rex` (a `covered` class), `methods/rexxinfo__instance.rex` (a `not-covered`
class). Each was checked against the corresponding branch of `class_probe_text`, `edge_probe_text` or
`method_probe_text`: the question asked matches the row's subject in every one, the name order
matches the row set's order, and the header's not-covered arm reproduces the row's `status` and
`reason`.

**Rows run on both interpreters** -- all 17 class-arm groups that contain an agreeing row; the
`array` wiring row; `concepts/obdes`, `concepts/unkno`, `concepts/reqstr`; plus hand probes for
`hasMethod` discrimination, the abuttal/blank placeholders, `.ArgUtil~superClasses`, and a synthetic
`.Zork~new`.

---

## What I could not check

* **Whether each hand-written concept probe is faithful to its section's prose.** I read all 21 and
  judged them against their section titles, their own headers and their committed `authority`
  strings, and I read two sections in `provide.xml` itself. For the other 19 I did not re-read the
  prose at the cited line. This is the same limit the report states, and it is real: a concept probe
  has no derivation, so nothing in the tree would notice a body that drifted from its section. It is
  also the live route by which a later task could turn `unkno` or `reqstr` green by editing the probe
  rather than the interpreter -- no structural check stands in the way.
* **Whether any row's owning phase is right.** Nothing checks it, by design (R32). I read the
  `authority` strings and they are internally consistent with the plan's handover; I did not
  re-derive them from the spec's enumeration.
* **Whether the row sets themselves are correctly derived from the books.** That is Task 3's output
  and Task 3's review. I checked only that this table reads them faithfully.
* **Whether the mutations behave the same under `--release`.** The mutation platform is a debug build
  of a copy. Its unmutated baseline reproduces the release run's verdict counts exactly, which is the
  evidence I have; I did not repeat the twelve controls under `--release`.
* **Performance.** No sitting was run and none is owed: both commits touch only `tests/`, `corpus/`,
  READMEs and a `#[cfg(test)]` module, so the release binary is unchanged.
* **Whether the 497 not-covered instance rows will in fact all flip to `agree` when 5b lands.** I
  demonstrated the mechanism on a synthetic group rather than on a real one, because no real instance
  group agrees on all three channels today.
* **CI.** `support::oracle::locate()` asserts rather than skips when the oracle binary is absent, so
  gate table C cannot pass on a machine without `/home/moritz/dev/repos/ooRexx/build`. That is
  inherited from `corpus.rs` and table D and is outside this task's scope; I did not read the CI
  configuration to see whether these targets run there.
