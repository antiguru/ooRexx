# Task 5 report -- gate table C, the concept and class surface

**Status: complete.** `crates/rexx-exec/tests/gate_table_c.rs` is a second `#[test]` sharing Task 4's
harness, over 1488 verdict rows and one assertion row, with 237 committed probe programs. M9 is
disposed of by promotion. The three structural controls are recorded as run, and two more beside
them. Each of the four verdict mutations is named against the task that owes it. No sitting is owed:
nothing landed in `src/` of `rexx-exec`, `rexx-core`, `rexx-classes` or `rexx-lib` -- the only file
outside `tests/`, `corpus/` and `docs/` that this task touched is
`crates/rexx-parse/src/instruction/tests.rs`, which is `#[cfg(test)]` and does not reach the release
binary the axes measure.

---

## Commits

| SHA | what |
|---|---|
| `e00ac9daf` | M9's disposal: `corpus/lang/primitive_classes.rex` becomes `corpus/gate-tables/concepts/classmeth.rex`, with the parse fixture's keys and messages fixed so a fixture outside `lang/` names itself |
| `0e605eb77` | gate table C: `crates/rexx-exec/tests/gate_table_c.rs`, 236 new probe programs, and both corpus READMEs |

## The five gate commands, each with its own exit status

Run from `rust/`, on the committed tree, each status read unpiped.

| command | exit |
|---|---|
| `cargo fmt --all --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `cargo test --release --workspace` | **0** |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0**, corpus **106 of 106** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0**, corpus **106 of 106** (`memcap` present at `~/.local/bin/memcap`, checked with `command -v`) |

And the phase-gate command the global constraints add, which is **expected to be non-zero** and is:

| command | exit |
|---|---|
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101** |

Because `cargo test` without `--no-fail-fast` stops at the first failing target, that run reports
only whichever table it reached first. The two tables were therefore also run separately under the
same environment, so that both numbers are measured rather than one inferred: `--test gate_table_d`
exits 101 with **10** gated rows, `--test gate_table_c` exits 101 with **135**.

---

## The row classes, their counts, and the predicate behind each count

Every count below is over **non-comment, non-blank lines** of the named committed file -- the same
predicate every other reader of a corpus list uses, and the one `read_table` implements.

| row class | rows | row set | probe path | probe text |
|---|---|---|---|---|
| concept | 21 | `corpus/docs/provide-sections.txt` | `gate-tables/concepts/<id>.rex` | hand-written |
| class wiring | 63 | `corpus/docs/class-set.txt` | `gate-tables/classes/<lower name>.rex` | derived |
| hierarchy edge | 57 | `corpus/docs/hierarchy-edges.txt` | `gate-tables/hierarchy/<child>__<parent>.rex` | derived |
| method | 1347 | `corpus/docs/class-methods.txt` | `gate-tables/methods/<class>__<arm>.rex` | derived |
| the `ArgUtil` assertion | 1, **with no verdict channel** | `class-set.txt` and `hierarchy-edges.txt` together | none -- see below | n/a |

**Probe programs committed: 237.** 21 + 63 + 57 + 96, the last being one program per (class, arm)
rather than one per method row, which is the spec's own rule and the reason it gives: every row costs
an oracle process launch, and 1347 of them inside `cargo test --release --workspace` is not a table,
it is an outage. Oracle launches per run of this table: 237.

**Verdicts on the commit that creates the table**, over all 1488 verdict rows -- the predicate is
`Verdict::Agree`, which is exit status, `stdout` and `stderr` all equal with `stderr` compared
**raw**, between this crate (run in process on both engines, which must agree with each other first)
and the C++ oracle (run in a subprocess on the same run):

```
agree: 32
diverge-both: 1434
diverge-stdout: 22
loud (this crate declined rather than answered): 1432
```

**By owning phase**, `rows / rows not yet agree`:

```
5a: 135 / 135
5b:   6 /   6
5c: 1347 / 1315
```

All 32 `agree` rows are method rows on the **class arm** -- `.X~hasMethod("M")` for a class this
crate already registers. There is no `agree` row in 5a or 5b, which is what the plan predicted:
`.array~id` is rc 120 today and the first wiring question stops there.

### The 5a non-`agree` count across both tables

**145 of 171**, with the predicate: a row whose committed owning-phase column is `5a` and whose
verdict is not `agree`, summed over gate table C and gate table D, measured on this task's commit.

| table | 5a rows | not yet `agree` |
|---|---|---|
| D (`gate_table_d.rs`) | 36 | 10 |
| C (`gate_table_c.rs`) | 135 | 135 |
| **total** | **171** | **145** |

Table D's 10 is unchanged by this task and was re-measured rather than quoted. This task is the
second of the two that *establish* the number rather than hold it, exactly as the global constraints
say; every task after this one reports against 145.

`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` therefore exits non-zero,
naming 145 gated rows. **That is the design and not a regression**, and the two commands the global
constraints list as gates -- the two without `REXX_PHASE_GATE` -- both exit zero.

---

## What each row asks

**Concept rows.** One program per `provide.xml` section, exercising that section's mechanism, plus a
committed `phase` / `authority` / `control` triple per section in `CONCEPTS`. The assignment is
checked against `provide-sections.txt` **in both directions**: a section with no arm is structural,
and so is an arm naming a section the row set does not carry.

Fifteen sections are 5a and six are 5b: `objcla`, `abscla`, `usesem`, `creo` and
`methodsbyclass` because each one's distinguishing claim needs an instance, and **`obdes` for a
different reason** -- its probe deliberately uses no instance at all (a class object is destroyed
like any other, so a class-side `UNINIT` fires with no `~new` anywhere), and it is 5b because the
spec's enumeration files object destruction and uninitialization there. The committed `authority`
string for that row says so; an earlier version of this paragraph did not, and was false for it. The two rows the spec
exists for are 5a and are written class-side so that they can be: `unkno` is `say .k~zork(1, 2)` with
`::METHOD unknown CLASS`, and `reqstr` is `say .k` with `::METHOD makeString CLASS` beside
`~request`, `~string` and a trapped and untrapped NOSTRING. Measured today, `unkno` is oracle rc 0
`unknown: ZORK with 2 argument(s): 1 2` against crate rc 159 `97.1 does not understand "ZORK"`, and
`reqstr` is oracle `K says hello` against crate `The K class`.

**Wiring rows, classes.** `~id`, `~class`, `~superClass`, `~superClasses~makeString('L', ' ')`,
`~metaClass` and `~isA(.Class)`, in that order, on `.X`.

**Wiring rows, edges.** `.Child~superClasses~hasItem(.Parent)` -- **present in**, never the whole
answer -- with `~id` on both ends and the whole list printed beside it for the report. No line of any
edge probe asserts the list's length.

**Method rows.** `.X~hasMethod("M")` on the class arm; `o = .X~new` then `o~hasMethod("M")` on the
instance arm. Which of those two shapes the instance arm takes comes from the committed `status`
column and from nothing measured in the test: for a class `class-set.txt` records as `covered` a bare
`~new` constructs, and for the rest the probe asks anyway and the raise is the row's evidence.

---

## The probe corpus is derived, and checked in both directions

Table D derives each row's probe **path**. Three of table C's four families derive the probe's
**text** as well: `class_probe_text`, `edge_probe_text` and `method_probe_text` are the definition of
what those programs contain, and every run compares the committed file against the re-derivation.

**Why that is worth the extra machinery, stated as what the path-only check cannot see.** A path
check answers "does a file with this name exist". It does not answer "does the file ask about this
row's subject". `classes/string.rex` containing `say 'id' .Array~id` passes the path check, agrees
with the oracle byte for byte, and reports the `String` row as green while nothing anywhere has asked
about `String`. Control 4 below is that exact mutation, run.

The concept probes keep table D's path-only property, because nothing derives a program that
exercises a documented mechanism. What stands in for the derivation there is the `CONCEPTS`
both-directions check on the section id, which catches a lost or invented row but not a probe whose
body drifted from its section.

---

## The three structural controls, recorded as run

Each was applied to the tree, run, observed, and restored from a copy in the scratchpad (not from
git). `git status` after each shows the file back.

| control | what was changed | what happened |
|---|---|---|
| **1. delete a probe program** | `rm corpus/gate-tables/classes/array.rex` | FAILED, structural: `gate-tables/classes/array.rex: a row has no probe program...` |
| **2. make one engine's answer differ from the other's** | `run.rs`'s `say_evaluated` gained `if matches!(self.engine, Engine::Ir) { self.out.push(b'!'); }` | FAILED, structural, **on a table C row**: `the two engines disagree on .../concepts/xmeths.rex`, with `tree-walker: stdout="own-class sub m\n..."` against `ir: stdout="own-class sub m!\n..."` |
| **3a. a row appears in the committed row file** | appended `Zork class clsZork fundclasses.xml:1 covered ...` to `class-set.txt` | FAILED, structural: `gate-tables/classes/zork.rex: a row has no probe program...` |
| **3b. a row disappears from the committed row file** | deleted the `Array OrderedCollection` line from `hierarchy-edges.txt` | FAILED, structural: `gate-tables/hierarchy/array__orderedcollection.rex: a probe program no row names, so nothing runs it` |

Control 2 is the one worth reading twice. The mutation is in the **crate's own engine dispatch**, not
in the harness: `say_evaluated` is the single place a `SAY` reaches `self.out`, and gating one byte
of it on `Engine::Ir` is a genuine two-engine divergence rather than a harness that was told to
report one. It reddens **before any verdict exists**, which is the property Task 4 built and this
table inherits.

### Two further controls, run because they cover channels the three do not

| control | what was changed | what happened |
|---|---|---|
| **4. a committed probe asks about a different subject** | `sed -i 's/\.Array~id/.String~id/' corpus/gate-tables/classes/array.rex` | FAILED, structural: `the committed probe is not what its row derives ... line 7 derived: "say 'id' .Array~id" / line 7 committed: "say 'id' .String~id"` |
| **5. the `ArgUtil` assertion** | appended `ArgUtil Object 0 843` to `hierarchy-edges.txt` | FAILED, structural, with the assertion's own message: `hierarchy-edges.txt carries an edge naming ArgUtil: ArgUtil <- Object (provide.xml:843) ... **No verdict row can see this**` |

Control 4 also produced a fix. The first run of it printed two *identical-looking* excerpts, because
`excerpt`'s bound cuts at 96 characters and every program in these families opens with a comment
naming its own row. The message now reports the **first differing line** from each side, which is
what the table above shows. That is a check that ran, exited non-zero, and told a reader nothing --
the shape worth catching, and only running the control surfaced it.

---

## The four verdict mutations, each named against the task that owes it

**None of them is runnable in this task, and the reason is not scheduling.** A mutation control
demonstrates a row going from green to red. Measured, `.array~id` is rc 120 today, so every wiring
row and every 5a concept row is red on the commit that creates it, and a red row cannot demonstrate
anything. Each control is therefore owned by the task where the thing it checks **first reads
`agree`**.

| # | mutation | owner | fires on |
|---|---|---|---|
| 1 | drop a class from the registry -- its wiring row cannot answer and reddens | **Task 9** (and again at **Task 21** for `Queue`, `Stem` and `VariableReference`, which `native_classes.rs` defers) | a table C **wiring row** |
| 2 | answer an own-scope `~method` query from a flattened all-scopes dictionary | **Task 9** | **not a table C row.** The plan builds no scope row class, because the documentation supplies no expected answer for the scope question -- it says a class documents a method, never at which scope it is defined. The instrument is the `~method` corpus programs Task 9 commits to `phase-5a.txt`, and the classes that task names are the whole of the guard |
| 3 | implement a section's mechanism silently wrongly -- `makeString` answering the **wrong string** rather than not at all | **Task 14** | the **`reqstr` concept row**, at rc 0 with empty `stderr` on both sides, on `stdout` alone |
| 4 | drop the operator-frame traceback line | **Task 6** | **not a table C row.** The spec names it as one of exactly two mechanisms with no documented section, and table C's concept rows are one per `provide.xml` section id. Its instrument is the three corpus programs Task 6 commits to `phase-5a.txt` under `RAW_STDERR_COMPARISON`, where dropping the frame line reddens the corpus gate byte-exactly |

Beside those, **D50's two required deletion controls** are carried in the `CONCEPTS` table itself, on
the rows they fire on, so a reader of the code meets them where the row is:

* **delete the `UNKNOWN` step** -- the `unkno` row reddens. **Task 12**, which carries it in its own
  "Done when".
* **delete the `makeString` limb** -- the `reqstr` row reddens. **Task 14**, same.

Every one of the 21 concept rows carries a `control` string of this shape and an `authority` string
for its phase, and the report prints both under the row listing. That is the honest statement of what
a concept row reaches: a section is a page of prose and its row is one program.

---

## M9's disposal

`corpus/lang/primitive_classes.rex` is **promoted**, not deleted:
`corpus/gate-tables/concepts/classmeth.rex`, the probe for `provide.xml`'s `classmeth` section
("Overview of Classes Provided by Rexx"). Its content is what that section is about -- `~id` of every
class reachable as an environment symbol -- and it needed only a header comment naming the section.

The charge was that it looked like coverage and was reached only as a parse fixture. Both halves are
answered:

* it no longer sits in `corpus/lang/`, whose README table lists the differential programs, and its
  row there is gone. It now sits under `corpus/gate-tables/`, whose README's first sentence is **"Not
  a differential corpus"**;
* it is now **run** -- on both crate engines and against the oracle -- as the `classmeth` row's probe,
  which is coverage rather than the appearance of it.

It remains a parse fixture of `crates/rexx-parse/src/instruction/tests.rs`, which was never the
problem. That table's entries were keyed by bare stem (`"primitive_classes"`) with the messages
hard-coding `corpus/lang/{name}.rex`; the key is now the corpus-relative path with its extension
(`"lang/do_variants.rex"`, `"gate-tables/concepts/classmeth.rex"`) and the messages read
`corpus/{name}`, so a fixture outside `lang/` names itself correctly. The pinned instruction count is
unchanged at 31: only a comment moved.

`corpus/README.md`'s prose reference to "the first draft of `primitive_classes.rex`" now names the new
path, and both READMEs' `rexx-diff` self-test figure is re-measured: **440 programs, 0 divergences,
exit 0** (`--cpp X --rs X` over `corpus/`, exit status read unpiped), where it read 204 before this
task's probes landed.

---

## What this table cannot see

Said here rather than left for a control that could not fire.

* **Whether a method is defined at a class's own scope.** Measured: `.Array~hasMethod("ID")` and
  `.Array~hasMethod("DEFINE")` are both **1** though `id` and `define` are `Class`'s, and
  `.Array~method("ID")` raises **97.1** because `~method` reads the instance dictionary. So a build
  that moved a class method between scopes is not caught here. The place it is caught is a
  scope-override send (`~m:scope`), which has its own corpus programs. Also measured, and the reason
  `~instanceMethods` is not the missing readback: `.Array~instanceMethods` contains `DEFINE`, `ID`
  and `OF` and **not** `APPEND`.
* **The operator-frame traceback line**, for the reason in mutation 4's row above.
* **A concept row's section beyond the one program that probes it.** A `provide.xml` section is prose
  with several claims; its row is one program. `CONCEPTS`'s `control` field is where each row states
  what it does reach.
* **A row filed under the wrong phase.** As in table D, the owning phase is a committed assignment
  read by a human out of a diff. Nothing checks the assignment, and a row filed under the wrong phase
  escapes gating. What *is* checked is that every section has exactly one arm, in both directions.
* **A probe's *body* drifting from its section**, for the concept family only. The other three
  families' bodies are derived and checked; a concept probe's is not, and cannot be.
* **A method row of a class with no instance.** Its group's probe raises at `~new`, so all of that
  group's rows share one verdict -- the raise -- and the individual names are never asked. That is
  what the row set says there is to have (`class-methods.txt`'s own header scopes its completeness
  evidence to `covered` rows for exactly this reason), but it means those rows measure the
  constructor, not the method set.

---

## What the row sets turned out not to support

Four things, none of which this task changed, because the row sets are Task 3's committed output and
R32 makes them a plan task's output rather than a derivation restated elsewhere.

1. **`RexxInfo`'s instance arm asks the wrong receiver, and the row set is why.** `class-set.txt`
   marks it `entry = instance` -- the `.environment` entry **is** the instance -- and separately
   `status = not-covered`. The probe shape comes from `status`, so `rexxinfo__instance.rex` opens
   `o = .RexxInfo~new` and raises 97.1, and all 28 of its rows share that verdict. Asking
   `.RexxInfo~hasMethod("M")` directly would answer them. Changing that is a row-set change (it needs
   `entry` to drive the shape, or a `status` that says so), which is Task 3's and not this one's, and
   inventing it here would make the table assert something the row set does not say.
2. **`RexxInfo`'s class wiring row stops at its first question.** `.RexxInfo~id` raises 97.1, so the
   remaining five wiring questions are never asked -- and two of them would answer, since every object
   understands `~class` and `~isA`. The probe text is derived uniformly from the row, which is what
   makes a probe unable to drift from its subject; special-casing this one row would trade that
   property for one row's worth of extra questions.
3. **`ArgUtil` has a wiring row and no method rows.** `class-methods.txt`'s own header records this
   and gives the reason -- the books document it nowhere -- so it is a stated absence rather than a
   gap. Named here because a class with a wiring row and no method rows reads, from the report alone,
   exactly like a class whose method rows went missing.
4. **Six `provide.xml` sections have no row in the spec's mechanism enumeration**: `xcremet`,
   `usingcl`, `methna`, `classmeth`, `chi` and `methodsbyclass`. Their `authority` strings say so and
   fall back to the plan's handover. This is not a defect found in passing -- it is the concept-row
   denominator doing the job the spec says it is there for: "the enumeration has no denominator, and
   that is the honest answer rather than a gap to fill... every mechanism that **has** a `provide.xml`
   section is inside table C's denominator, so its absence from the enumeration still surfaces there."

---

## Two silent divergences this table found on the commit that created it

Both are `rc 0` with **empty `stderr` on both sides**, differing on `stdout` alone -- the shape the
spec was rewritten for, and the shape no existing harness reddens.

* **`obdes`** (5b): a class-side `::METHOD uninit CLASS` fires at interpreter termination on the
  oracle and not here. Oracle `main\nuninit ran\n`, crate `main\n`, both rc 0, both `stderr` empty.
* **16 of `String`'s 17 class-arm method rows, and `~of` on five map collections** (5c): measured,
  `.String~hasMethod("ALNUM")` and its 15 siblings, and `.Directory~hasMethod("OF")` and the same on
  `IdentityTable`, `Relation`, `StringTable` and `Table`, answer differently here from the oracle at
  rc 0 with empty `stderr`. They are 5c rows and are reported, not gated.

Neither is this task's to fix. Both are recorded because a silent divergence found by a table on its
first run is the table earning its keep, and because the second one would otherwise be read as noise
in a 1347-row block.

---

## Design decisions worth a reviewer's attention

**One probe per (class, arm), not one per class.** The spec says "one program per class, not one per
pair". This task splits the class arm from the instance arm -- 96 programs over 62 classes rather than
62 -- and the reason is measurable: `.Array~hasMethod("new")` and `("of")` **agree with the oracle
today**, and bundling them into the same program as `o = .Array~new` (rc 120, loud) would make the
program's status and `stderr` channels red and take all 46 of Array's rows down with them. The cost
argument the spec gives is about per-row launches (1347 of them); 96 preserves it. **All 32 `agree`
rows in this table but one exist because of that split.** Under the letter of the rule the table
would read **1** `agree`, not 0: `Buffer`, `Singleton` and `Validate` have class-arm rows only, so
their one-program-per-class file is byte-identical to today's class-arm file and has no `~new` to
poison it, and `Buffer new` is one of the 32. The decision and its measured reason stand; the
justification was overstated by one row. **A copy of the overstated sentence is in commit
`0e605eb77`'s message and cannot be edited** -- amending reviewed work to correct prose costs more
than the sentence does, so this paragraph is the correction and the commit message is left as it is.
Flagged as a deviation.

**A method row's `stdout` channel is one line of a shared program.** `status` and `stderr` are the
program's, which is right -- a program that died answered none of its rows. `stdout` is the row's own
line, and a row agrees only when the two sides produced the **same number of lines** and this row's
line matches. Without the count rule, a crate printing extra lines would let the rows before the
extra one read `agree` off an output that is wrong as a whole.

**A structural line-count check over the shared program.** The oracle's `stdout` must have exactly as
many lines as the group has rows (or none, for a class with no instance). Without it, a probe that
quietly stopped asking half its names would leave those rows comparing an absent line against an
absent line -- which reads `agree` for a question nobody asked. It fired during development, on the
first version of `stdout_lines`: `split` on an empty slice yields one empty slice, so a program that
printed nothing looked as though it had answered one row, and 23 groups reported "the oracle answered
1 line(s) where the row set says 0".

**The `ArgUtil` assertion is the one row with no verdict channel, and that is the point.**
`provide.xml` comments `ArgUtil`'s hierarchy member out, so an extractor that read the `<member>`s
without blanking comments first emits an `ArgUtil Object` edge -- and that edge is **true**, measured:
`.ArgUtil~superClasses` is `The Object class`. An edge row for it would read `agree` and the
end-to-end oracle run would stay at zero failures over a wrong member set. Only an assertion over the
committed files can see it, so that is what it is, and it is structural because there is nothing for
a gate mode to relax.

---

## What I could not check

* **That every concept probe exercises its section faithfully.** It is a judgement about prose, and no
  instrument in this repository can make it. What I did instead: read each section in
  `oodocs/rexxref/en-US/provide.xml` at the line `provide-sections.txt` cites, wrote the probe from
  the section's own claims (three probes are the section's own worked example -- `chsrod`'s
  Account/Savings `TYPE`, `creo`'s `.savings~new(1000.00, 6.25)`, `methodsbyclass`'s
  `matrix[2, 3] = 0`), and recorded per row what would falsify it. Had a probe been unfaithful, none
  of the five controls would have said so.
* **That the owning-phase assignment is right.** Nothing checks it, by design (R32's standing). It is
  in the diff with its authority beside it.
* **Whether the six sections with no enumeration row are 5a or later.** I assigned each from what its
  probe needs and named the handover clause; if the spec author disagrees, the arm is one line.
* **The 5b and 5c rows will never be gated by anything in this plan.** 5b and 5c are not planned yet,
  so those 1353 rows are reported for the rest of Phase 5a and gate nothing -- the same standing table
  D's `7` and `deferred-parse-error-rendering` rows have.
* **`unsafe`**: none added, none needed, no exception taken.

---

## Provenance

* `oodocs/rexxref` at **r13198**, `ootest/` at r13178, both checked with `svn info` on 2026-08-21,
  which is what `provide-sections.txt`'s and its siblings' `derived-at:` stamps say.
* Oracle: `/home/moritz/dev/repos/ooRexx/build/bin/rexx` under the standard wrapper
  (`ulimit -v 1048576`, `LD_LIBRARY_PATH=.../build/lib`, `timeout -s KILL 10`), three descriptors
  read separately, never `2>&1`, run from a fresh empty directory with absolute paths.
* **Every one of the 237 probes was run by hand through `rexx-run` under `timeout -s KILL 10`, on
  both engines, before commit**: 237 probes, 0 killed, 0 engine disagreements. That sweep was re-run
  on the final content, after the control-2 mutation was restored and the release binary rebuilt --
  `ls -l` on binary and source, and `say 'x'` under `REXX_ENGINE=ir` printing `x` and not `x!`, were
  both checked, because a clean `git status` says nothing about `target/`.
* **Determinism** over the final content is `rexx-diff`'s own self-test, `--cpp X --rs X` over
  `corpus/`, which walks the directory recursively and so reads every one of these: **440 programs,
  0 divergences, exit 0**, status read unpiped. During development the 21 concept probes were also
  run three times each and the 216 derived ones twice, and the derived ones were swept for exit
  status and for `stdout` line count against the row set.
* Nothing in `corpus/oracle-crashes.txt` was run.

---

# Fix round 1

**Commit `938916aa2`**, on top of `0e605eb77`.

**The five gate commands on the committed tree**, each status read unpiped: `cargo fmt --all --check`
**0**; `cargo clippy --workspace --all-targets -- -D warnings` **0**; `cargo test --release
--workspace` **0**; `REXX_CORPUS_GATE=1 cargo test --release --workspace` **0** at **106 of 106**;
`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` **0** at **106 of 106**. The
phase gate stays non-zero as designed: the workspace form exits **101**, and per table
`--test gate_table_c` exits 101 with **135** gated rows and `--test gate_table_d` exits 101 with
**10**.


All seven findings closed. Both majors were demonstrated by controls the reviewer ran; both
demonstrations were reproduced here, then re-run after the fix and **both now fail**.

## What replaced the defect, family by family

**Finding 1** is the generalisation the review asked for, and it needed a different bound per family
because only three of the four have a derivation to take one from. `OracleShape` is the type:
`Exactly(n)` for a program every `say` of which is reached, `AllOrNothing(n)` for one that constructs
first and can therefore answer everything or nothing and nothing between.

| family | bound | where it comes from |
|---|---|---|
| class wiring | `Exactly(say-count)` for an `entry = class` row, `Exactly(0)` otherwise | `derived_say_lines` over the derived text, gated on the `entry` column -- which **is** a claim about the `.environment` entry, and `class-set.txt`'s header states it for the one row that is not a class object |
| hierarchy edge | `Exactly(say-count)`, **plus** the oracle's own `documented-edge` answer having to read `1` | the derived text, and the row's own documented claim |
| concept | `Exactly(committed count)` | a new `oracle_lines` field on `CONCEPTS`, one per section |
| method | `Exactly(rows)` on the class arm, `AllOrNothing(rows)` on the instance arm | the probe's shape, **not** the `status` column -- see finding 3 |

**A row that fails the check keeps its place in the table.** It is reported as `unanswered`, counted
as not-`agree`, and counted as gated on a gated phase. Dropping it would be the vanishing-row shape
Task 4 fixed: one row fewer, a lower gated count, and a close criterion satisfiable by removing
evidence. `Measured::verdict` is therefore `Option<Verdict>` in both tables.

**And the same for a method row whose own line is absent from both sides.** `AllOrNothing` admits a
group that answered nothing, so the shape check alone left the reviewer's three synthetic method rows
green. A row whose line is on neither side was asked of neither, so it has no verdict either --
`agree` would be a claim about a question nobody put and `diverge` a claim about the constructor
rather than about the row. One side answering and the other not stays a real divergence and keeps its
verdict.

**Referential integrity, so the class row is the existence check for the other families.** Every
hierarchy endpoint must be a `class` row of `class-set.txt` (`check_edge_endpoints`); every method
row's class already had to be one. So a name this build does not have is caught once, at its wiring
row, rather than once per family or not at all.

## Table D: assessed, and the invariant exists

Measured over all 79 probes, three descriptors read separately: **72 print exactly one line on the
oracle and 7 print none.** The 7 are `::CLASS CLASS` and `::RESOURCE LIBRARY` (the row set's two
`cross-reference` rows, which are syntax errors) and `::ATTRIBUTE`/`::METHOD`/`::ROUTINE EXTERNAL`
plus `::REQUIRES LIBRARY`/`NAMESPACE`, whose subject -- a shared library, a package -- is not present
on this build, so the failure is at install time before the program's first clause. `side` does not
separate them: 5 of the 7 are `both` rows.

So the invariant exists but its exception set is not derivable from the row set. It is committed as
`ORACLE_REFUSES` in `gate_table_d.rs`, beside `owning_phase` and with the same standing -- a human
reads it out of a diff -- and it is **policed in both directions**, which is what stops it growing
quietly to cover a probe that stopped working. Both directions were run (controls below).

## Findings 2, 3, 4, 7

* **2.** `check_probe_text` now matches on the error instead of returning on any of them, and pushes a
  `Structural` guarded on the directory listing -- Task 4's N1 pattern, so a missing file still
  reports once through the set check. Its sibling is fixed too: a probe file whose **name** is not
  UTF-8 was dropped from `probe_set`'s listing and could never be reported as an orphan; it now
  reports.
* **3.** `expected_oracle_lines` is gone. The method bound comes from the probe's shape, so no claim
  about the oracle is read out of `status` at all -- which removes both halves of the finding: the
  `not-covered` direction, whose column says in capitals that it carries no claim about the oracle,
  and the `covered` direction, which is a disjunction. `method_probe_text`'s doc now states `covered`
  as the disjunction it is, says that only its first limb is derived, and says what the task
  committing the first construction program has to do here in the same change. **And the second half
  is now in the verdict summary rather than only in this report**: the summary line reads `method
  rows whose group's probe raised at ~new on the oracle, so no documented name was asked on either
  side: 497`, and those rows read `unanswered` rather than `agree`, so the block cannot turn green
  without an instance existing.
* **4.** `first_difference` splits with `split_inclusive('\n')` instead of `lines()`, so a terminator
  is part of the line it terminates. Both shapes now produce a message that says something: a
  stripped final newline reads `line 12 derived: "...isA(.Class)\n" / committed: "...isA(.Class)"`,
  and a CRLF conversion reads `line 1 derived: "...\n" / committed: "...\r\n"`.
* **7.** `check_interpolated_text` makes a `class-set.txt` `reason` containing `*/` a structural
  failure naming the row, rather than a probe whose block comment ends early. The fix belongs in the
  row file, so the check names the row instead of escaping the text and carrying on.

## The two demonstrations, re-run

| demonstration | before | after |
|---|---|---|
| synthetic `Zork` class row, edge row and three method rows, all with correctly derived probes | exit **0**, all five `agree`, `5a: 137 rows, 135 not yet agree` | exit **101**. The class and edge rows read `unanswered`; the method group reads `unanswered=3`; two structural failures name `classes/zork.rex` and `hierarchy/zork__object.rex` (`the oracle answered 0 line(s) where this row's probe asks for exactly 6` / `for exactly 4`); `5a: 137 rows, 137 not yet agree` |
| `classes/array.rex` rewritten to ask `.String~id`, with one `0xff` byte appended | exit **0**, the derivation check never ran | exit **101**, and **two** independent instruments fire: `its bytes cannot be read as text: stream did not contain valid UTF-8`, and `the oracle answered 0 line(s) where this row's probe asks for exactly 6` |

## Every control run in this round

Applied to the tree, run, observed, restored from a scratchpad copy, `git status` checked after each.

| control | what was changed | what happened |
|---|---|---|
| demo A | the reviewer's five synthetic `Zork` rows | exit 101, above |
| demo B | the reviewer's `0xff` probe | exit 101, above |
| concept count, high | `unkno.rex` gains a `say 'extra'` before its send | exit 101, `the oracle answered 2 line(s) where this row's probe asks for exactly 1` |
| concept count, low | `unkno.rex`'s send retargeted at an undefined class | exit 101, `the oracle answered 0 line(s) ... asks for exactly 1` |
| documented edge | `hierarchy-edges.txt`'s `Array OrderedCollection` row rewritten to `Array Comparable`, with its derived probe | exit 101, `the oracle does not confirm the documented edge Array <- Comparable: its documented-edge answer is Some("0"), not "1"` |
| edge endpoints | a `RexxInfo Object` edge appended (`RexxInfo` is an `instance` row) | exit 101, `its child, RexxInfo, is not a class row of class-set.txt` |
| interpolated text | `*/` inserted into `Alarm`'s `reason` | exit 101, `its reason contains */, which ends the Rexx block comment the derived probe writes it into` |
| final newline | stripped from `classes/array.rex` | exit 101, with the terminator visible in the diagnostic |
| CRLF | `classes/array.rex` converted | exit 101, with `\r\n` visible in the diagnostic |
| table D, direction A | `("::METHOD", "EXTERNAL")` removed from `ORACLE_REFUSES` | exit 101, `the oracle answered 0 line(s) on stdout where this row expects 1 ... this row is not named there` |
| table D, direction B | `("::CLASS", "PUBLIC")` added to `ORACLE_REFUSES` | exit 101, `the oracle answered 1 line(s) ... expects 0 ... this row is named there` |

## The counts after the round, and which earlier numbers they supersede

The verdict **labels** moved and the per-phase open counts did not. Superseding the table earlier in
this report:

```
agree: 32          (unchanged)
diverge-both: 937  (was 1434)
diverge-stdout: 22 (unchanged)
unanswered: 497    (new -- method rows whose group raised at ~new, previously counted diverge-both)
loud: 1432         (unchanged)
```

**5a non-`agree` is unchanged at 145 of 171** -- table C 135 of 135, table D 10 of 36, each measured
by running that table alone under `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1`. The predicate is
unchanged: a row whose committed owning-phase column is `5a` and whose verdict is not `agree`, where
`unanswered` is not `agree`.

Also superseded: this report's "What this table cannot see" says a method group with no instance
leaves "all of that group's rows sharing one verdict -- the raise". They now share **no** verdict;
they read `unanswered`.

## What I could not close

* **Whether each concept probe is faithful to its section's prose** -- unchanged, and no instrument
  here can make that judgement. The new `oracle_lines` field narrows the hole rather than closing it:
  a probe that stopped reaching its mechanism now reddens, and editing what a probe prints now
  requires changing a committed number in the same diff, which makes the edit visible. A probe edited
  *together with* its count is still not caught.
* **A method group whose class the build has but whose constructor stops constructing.** `AllOrNothing`
  admits it and the rows read `unanswered` rather than `agree`, so it cannot go green -- but nothing
  reddens either. The instrument that would see it is `class-set.txt`'s own `status`, which Task 3
  re-derives.
* **Whether the seven `ORACLE_REFUSES` rows stay refused on another build.** They are refused because
  a library or package is absent here. A build that shipped one would flip that row, and the
  both-directions check would redden and name it -- which is the intended behaviour, but it does mean
  the list is a fact about this machine, stated in the constant's own doc.

---

# Fix round 2

**Commit `772cb9852`**, on top of `938916aa2`. One medium, five low; all closed.

**The five gate commands**, each status read unpiped, on the content this commit carries:
`cargo fmt --all --check` **0**; `cargo clippy --workspace --all-targets -- -D warnings` **0**;
`cargo test --release --workspace` **0**; `REXX_CORPUS_GATE=1 cargo test --release --workspace` **0**
at **106 of 106**; `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` **0** at
**106 of 106**.

**5a non-`agree` is unchanged at 145 of 171** -- table C **135 of 135**, table D **10 of 36**, each
measured by running that table alone under `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1`, both exiting
**101**. Predicate unchanged: a row whose committed owning-phase column is `5a` and whose verdict is
not `agree`, where `unanswered` is not `agree`.

## N1 -- the two zero-count arms

The re-review's point is exact and worth restating as the general rule: **where the expectation is
"the oracle printed nothing", nothing there at all satisfies it.** `asked` was added for
`AllOrNothing`; the same round created `Exactly(0)` twice and guarded neither.

### Table C, the `entry` column

**The bound is now non-zero for every class row.** The probe opens with two questions any
`.environment` entry answers -- `say 'entry' .X` and `say 'class-of-entry' .X~class~id` -- before the
six only a class object answers. So an `entry = class` row's bound is all of them and an
`entry = instance` row's is the opening two, and neither is zero.

**That alone does not discriminate, and the ruling's suggested question was measured before it was
trusted.** The brief named `.RexxInfo~class~id` as "one line an entry answers and an absent name does
not". Measured, an absent name answers it too: an unresolved environment symbol evaluates to its own
name as a string, so `.Zork~class~id` is **`String`** -- one line either way, and both interpreters
agree on it. So the row would still have read `agree`. The bound is therefore paired with a check on
the **rendering**, which does separate them: measured, `.Array` renders as `The Array class`,
`.RexxInfo` as `a RexxInfo`, and `.Zork` as `.ZORK`. `check_entry_present` requires the oracle's
`entry` answer not to be the row's own name in that form -- the row's own claim that this name is an
entry, checked the way `check_documented_edge` checks an edge row's claim, and a failure makes the
row `unanswered`.

**Why not `.environment~hasIndex("NAME")`, which is the direct question.** Measured, it answers `1`
for `.Array`, `.ArgUtil` and `.RexxInfo` and `0` for `.Zork` -- but this crate refuses it today with
`a message send to one of the interpreter's own objects is not implemented`, which is **Task 17's**.
Putting it in every wiring probe would make all 63 rows unable to reach `agree` until Task 17, and
Task 9's "Done when" is that they read `agree`. The rendering check needs nothing the row did not
already need: `say .Array` agrees with the oracle on this crate today.

**And `entry` is validated.** `ENTRY_KINDS` is the recognised set and an unrecognised value is
structural. It was read at two decision sites and validated at neither, so a typo dropped the row out
of both -- the bound fell to the instance arm and `check_edge_endpoints` stopped seeing it.

### Table D, `ORACLE_REFUSES`

**Asked first, as ruled: a non-zero `stdout` bound does not exist, and that is measured.** All seven
probes already open with `say 'main'`, and the oracle prints nothing for them, because each refusal
is a translate-time or install-time failure and both precede the program's own first clause. There is
no "before the refusing directive" in a Rexx program's execution order.

**So the bound moved channel rather than falling back to a stated gap.** What those rows do answer is
the report the oracle writes on `stderr`: measured, each writes one (the shortest 244 bytes) while a
probe replaced by `nop` writes none. `refusal_answered` requires it, and a row that fails is
`unanswered`. What remains uncovered -- a probe rewritten to fail for an unrelated reason still
writes a report -- is now stated in that table's own `# What this table cannot see`, with the task
that could close it: one that gives table D's row set a column saying what each row's probe is
expected to produce, the standing gate table C's `entry` and `status` columns already have.

## N2, N3, N4, N5, N6

* **N2.** `corpus/gate-tables/README.md` -- which is tracked, unlike this report -- now says per
  directory what the bound is: the derived text **gated on `entry`** plus the entry-resolution check
  for `classes/`, the derived text for `hierarchy/` and `methods/`, a committed count for
  `concepts/`, and **one line for most `directives/` rows and none for the ones the oracle refuses,
  whose answer is on `stderr`**. It also says what happens to a row that fails: it stays, reads
  `unanswered`, and still counts against the gate.
* **N3.** The sentence is computed from the verdicts (`verdict.is_none()`), not from the oracle's
  output alone, so it cannot disagree with the column beside it. Both read **497** today; the
  predicates now cannot diverge.
* **N4.** `verdict_label`, `UNANSWERED` and the `stdout` line split live in `gate_tables/mod.rs`,
  which both binaries already `mod`. Nothing held the two copies of the label equal.
* **N5.** Both historical sentences struck. `Measured::verdict`'s "Dropping it would..." stays, as
  ruled -- it is the rejected route given as the reason the type is an `Option`.
* **N6.** The module doc's heading reads "The table types no expected bytes, **and one expected
  count**", and names `Concept::oracle_lines` as the exception with why the concept family is where a
  bound has to come from somewhere.

## The three controls, re-run

| control | before | after |
|---|---|---|
| the `Zork` row filed with `entry = instance` | exit **0**, row read **`agree`**, a gated 5a row green over a class in neither interpreter | exit **101**; the row reads **`unanswered`**, and the structural failure names the cause: `the oracle does not resolve .Zork to an .environment entry: its 'entry ' answer is Some(".ZORK"), which is how an unresolved environment symbol renders` |
| an unrecognised `entry` value (`Array` filed as `klass`) | silently exempted the row from its bound and from the edge referential check | exit **101**, with all three consequences visible: `its 'entry' column reads "klass", which is not one of ["class", "instance"]`, then `Array <- OrderedCollection: its child, Array, is not a 'class' row`, then `the oracle answered 8 line(s) where this row's probe asks for exactly 2` |
| a gated 5a table D row's probe replaced by `nop` | exit **0**, row moved to `agree`, 5a open count **10 -> 9** | exit **101**, `the oracle refuses this row's probe, so its answer is the report it writes on 'stderr' -- and it wrote none`, and the 5a open count stays at **10** |

## Re-verification the regeneration required

All 63 class probes changed, so the provenance checks were re-run rather than inherited: every one of
the 237 probes run by hand through `rexx-run` under `timeout -s KILL 10` on **both engines** -- 237
probes, 0 killed, 0 engine disagreements -- and `rexx-diff --cpp X --rs X --corpus corpus`, the
recursive determinism self-test, at **440 programs, 0 divergences, exit 0**, status read unpiped.

## What I could not close

* **A concept probe edited together with its committed count** -- unchanged from round 1.
* **A class the build has whose constructor stops constructing** -- unchanged from round 1: the rows
  read `unanswered` so they cannot go green, but nothing reddens.
* **A table D probe rewritten to fail for an unrelated reason** -- it still writes a report on
  `stderr`, so the new bound admits it. Stated in that table's own section with the task that could
  close it.
* **The rendering check reads one line of the oracle's output by a marker the derivation writes.** A
  probe that stopped printing that line reddens on the bound rather than here, which is why both
  checks run and neither replaces the other; but nothing outside this file pins the marker's spelling.

---

# Fix round 3

**Commit `fe0364c08`**, on `989fb26e1` (which touches only `docs/superpowers/plans/`).

**The five gate commands**, each status read unpiped, never through a pipe: `cargo fmt --all --check`
**0**; `cargo clippy --workspace --all-targets -- -D warnings` **0**; `cargo test --release
--workspace` **0**; `REXX_CORPUS_GATE=1 cargo test --release --workspace` **0** at **106 of 106**;
`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` **0** at **106 of 106**.
`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` per table: gate table C exits **101** with **135** gated
rows, gate table D exits **101** with **10**. **5a non-`agree` unchanged at 145 of 171.**

## A -- the impossibility claim, and the construction that tested it

The claim was that a line printed before the refusing directive **does not exist**, because the
refusal is a translate-time or install-time failure and both precede the program's first clause.
Reproduced the re-review's measurement rather than taking it: it is **false for the `::REQUIRES
NAMESPACE` row**, because installing a `::REQUIRES` runs the required program -- install time is not
before Rexx code runs, it is Rexx code running.

The construction, run against every one of the seven, from a fresh empty directory with absolute
paths and three descriptors read separately:

```
say 'main'
::requires 'helper.rex'          -- helper.rex is:  say 'helper-ran'
<the row's own directive>
```

| row | rc | `stdout` |
|---|---|---|
| `::REQUIRES NAMESPACE` | 213 | `helper-ran` |
| `::ATTRIBUTE EXTERNAL` | 166 | empty |
| `::METHOD EXTERNAL` | 166 | empty |
| `::ROUTINE EXTERNAL` | 158 | empty |
| `::REQUIRES LIBRARY` | 158 | empty |
| `::CLASS CLASS` | 231 | empty |
| `::RESOURCE LIBRARY` | 231 | empty |

**The bound stays on `stderr`**, and its reason is now what that construction measures: a report
there is the answer **every** one of these rows gives, not that no other answer could exist for any
of them. Both places that carried the old reason -- the module doc and `refusal_answered`'s doc --
now carry the construction and its per-row result instead.

**The `::REQUIRES NAMESPACE` row could carry a `stdout` bound of its own**, on the evidence above. It
is not built here: a per-row bound is the shape that needs a row-set column saying what each probe is
expected to produce, which is the task the module doc already names.

## B -- "every probe here already opens with `say 'main'`"

False as written: measured, the `options__*` probes open with `say digits()`, `say form()` and
`say fuzz()`. **Narrowed to the rows it is an argument about, and asserted rather than written
down**, because it is load-bearing -- a program with no `SAY` before its first directive prints
nothing whether or not anything refuses it, so the `stderr` bound would rest on a `stdout` carrying
no information. `check_refusing_probe_says` requires a `say` clause before the first directive of
every row `ORACLE_REFUSES` names. The README's "like the rest" is gone with it.

## C, D, E

* **C.** `refusal_answered`'s history sentence (`replacing such a row's probe with a program reading
  nop did exactly that and took the 5a open count down by one`) is struck. Applying the deciding test
  to its neighbours: the two sentences before it say the same thing about the code as it is with the
  historical framing removed, so they stay.
* **D.** Three enumerations brought to what the code does: `gate_table_c.rs`'s module-doc wiring
  bullet and its `OracleShape` paragraph, and `corpus/gate-tables/README.md`'s `classes/` bullet. All
  three now say the probe asks what the entry renders as and what its class is before the questions
  only a class object answers, and the module-doc paragraph names the pairing with
  `check_entry_present` and why a count alone cannot do it.
* **E.** `check_concept_line_counts` makes a committed `oracle_lines` of zero structural. It is the
  third arm of the same family: a committed number where the others derive one, and zero is met by a
  program that produced nothing at all.

## Controls, all five run

| control | result |
|---|---|
| a concept row committed at `oracle_lines: 0` | exit **101**, `CONCEPTS arm abscla: its 'oracle_lines' is zero, and a bound of zero lines is met by a program that produced nothing at all`, co-firing with the bound itself |
| a refusing row's probe with no `say` before its directive | exit **101**, `'ORACLE_REFUSES' names ::METHOD EXTERNAL ... This one has no 'say' clause there` |
| the `Zork` row filed as `entry = instance` | exit **101**, row reads **`unanswered`**, `the oracle does not resolve .Zork to an '.environment' entry: its 'entry ' answer is Some(".ZORK")` |
| an unrecognised `entry` value (`Array` filed as `klass`) | exit **101**, the kind check plus both consequences it named -- the edge referential check and the bound |
| `nop` on a gated 5a table D row | exit **101**, now on **two** instruments (the new `say` guard and the `stderr` bound), and the 5a open count stays at **10** |

## Which round-2 claims these edits change

* The round-2 report's sentence *"a non-zero `stdout` bound does not exist, and that is measured: all
  seven probes already open with `say 'main'` and print nothing, because the refusal is a
  translate-time or install-time failure and both precede the program's first clause"* is
  **superseded**. Both halves were wrong: the impossibility does not hold for `::REQUIRES NAMESPACE`,
  and "all seven ... like the rest" overstated a property that is true of those seven and false of
  the `options__*` probes. What survives is the conclusion -- the bound is on `stderr` -- and the
  measurement that `nop` writes none where each of the seven writes a report.
* Round 2's commit message carries the same false reason. As in round 1's finding 5, it is left
  unamended and corrected here.
* No other round-2 claim changes. The counts, the verdicts and the controls are as reported.

## What I could not close

Unchanged from round 2, minus nothing and plus nothing:

* a concept probe edited together with its committed count;
* a class the build has whose constructor stops constructing;
* a table D probe rewritten to fail for an unrelated reason -- it still writes a report on `stderr`;
* the entry-rendering check reads one oracle line by a marker only this file pins.

And one this round names rather than closes: **the `::REQUIRES NAMESPACE` row's `stdout` bound**,
available on the evidence above and left to the task that gives table D's row set an expected-output
column.
