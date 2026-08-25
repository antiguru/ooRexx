# Task 5, fix round 1 -- re-review

Scope: `0e605eb77..938916aa2`, one commit. Reviewed on the committed tree at `938916aa2`, `git
status` clean at the start and at the end. Every mutation was applied to a copy of `rust/` at
`.../scratchpad/t5rr/repo/rust`, with symlinks supplying `interpreter/` and `oodocs/`; the real
repository was never modified.

**Verdict: REWORK.** All seven findings are closed and both demonstrations now fail. The reason for
REWORK is one new defect: **the defect class this round exists to close survives in the two arms
where the new check's expected line count is zero** -- table C's `entry != "class"` class row and
table D's seven `ORACLE_REFUSES` rows -- and I demonstrated an `agree` verdict over a question nobody
asked in **both** tables, one of them on a gated 5a row. The remaining work is small and local; the
round's substance is right.

## The seven findings

| # | | |
|---|---|---|
| 1 | **CLOSED** | `OracleShape` is applied to all four families; the synthetic `Zork` rows now exit 101 and read `unanswered`. See N1 for the two arms it does not reach. |
| 2 | **CLOSED** | The `0xff` probe exits 101 and two instruments name it, the first saying `its bytes cannot be read as text: stream did not contain valid UTF-8`. A missing file still reports exactly once, through the set check. The non-UTF-8 **filename** sibling reports too. |
| 3 | **CLOSED** | `expected_oracle_lines` is gone from table C; the method bound is `Exactly(rows)` on the class arm and `AllOrNothing(rows)` on the instance arm, keyed on `arm` and not on `status`. Constructed the case that used to break: no structural failure, rows read `unanswered`. The 497-row property is in the verdict summary, on stderr, on a green run. |
| 4 | **CLOSED** | `split_inclusive('\n')`. Both shapes reproduce with a message that says something. |
| 5 | **CLOSED** | The report reads "All 32 `agree` rows in this table but one", and I verified the replacement is true (below). No copy of the false sentence survives under `rust/`. The commit was not amended, as ruled, and the report flags it. |
| 6 | **CLOSED** | The `obdes` paragraph now gives that row's own reason and says the earlier version was false for it. |
| 7 | **CLOSED** | `check_interpolated_text` fires and names the row rather than the probe. |

## New defects, most severe first

### N1. MEDIUM -- where the new check's expected count is zero it cannot tell "refused for the documented reason" from "nothing there at all", and that is finding 1 unchanged. Demonstrated in both tables.

The round built two guards for the method family and one for the other three. The second guard --
`asked`, in `gate_table_c.rs:1573`, which withholds a verdict from a row whose own line is absent
from **both** sides -- exists precisely because `AllOrNothing(n)` admits `0` and the shape check
alone left the reviewer's three synthetic method rows green. **The round then created two more arms
whose admitted count is zero and gave neither of them that second guard.**

**Table C, the `entry` column.** `gate_table_c.rs:1444` reads
`OracleShape::Exactly(if row.entry == "class" { asked } else { 0 })`. For a row whose `entry` is not
`class`, "the oracle printed nothing" **is** the expectation, and it is also what a name neither
interpreter has produces.

Demonstrated (control **A'** below): the reviewer's synthetic `Zork` row, filed with `entry` reading
`instance` instead of `class`, with the correctly derived probe committed and no edge or method rows.
The table exited **0**, the row read **`agree`**, and the phase line read `5a: 136 rows, 135 not yet
agree` -- **a gated 5a row green over a class that exists in neither interpreter**, which is the
reviewer's original demonstration, reproduced after the fix with the row's `entry` column reading
`instance`.

The row this is live for today is `RexxInfo`, the one `instance` row in `class-set.txt`. It reads
`diverge-both` now, and its false green needs the **oracle** to stop shipping `.RexxInfo` -- at which
point both sides raise `Object ".REXXINFO" does not understand message "ID"` alike, the shape check
admits it, and the row goes green. That is the `RegularExpression` scenario, which
`OracleShape`'s own doc cites as the reason the type exists.

Two things sharpen it. `row.entry` is read at exactly two decision sites and validated nowhere, so a
value that is not `class` -- a fourth entry kind, a typo -- silently exempts its row from the check
rather than reddening. And `check_edge_endpoints` excludes `instance` rows from being edge endpoints,
so the cross-family referential integrity the round added deliberately does not reach this row
either.

**Table D, `ORACLE_REFUSES`.** The same shape, and here the exemption is a committed list. For the
seven rows named there `expected_oracle_lines` is `0`, so `answered == expected` holds and a verdict
is computed from `compare_raw` exactly as before the round. `agree` is reachable for those rows with
**no clause of the probe executed on either side**.

Demonstrated (control **D3**): replacing `directives/attribute__external__subkeyword.rex` -- a gated
5a row -- with a program reading `nop` moves the row from `diverge-both` to **`agree`**, takes the
5a open count from 10 to 9, and the table exits **0** with no structural failure. Table D derives
only the probe's **path**, so nothing else in the tree sees the body change either.

**The concrete consequence, in both tables.** Task 24's "Done when" is *every 5a row in both tables
reads `agree`*. Three rows can satisfy it while neither interpreter answered a question:
`RexxInfo`'s wiring row, and table D's `::ATTRIBUTE EXTERNAL` and `::METHOD EXTERNAL`, which the
oracle refuses at install time because the named shared library is not on this machine. That is the
criterion the round's own commit message calls "a satisfied row of its phase" with nothing asked.

**And the ruling's fallback for table D is half-done.** It said: apply the invariant if one exists,
and *"if it genuinely does not, record it in that table's header as a named gap, with the reason and
the task that could close it."* The round applied an invariant to 72 of 79 probes and left 7
exempt -- so the fallback applies to those 7. `ORACLE_REFUSES`'s doc gives the **reason** each row is
refused. It does not say that those rows' verdicts still carry finding 1's property, the
`# What this table cannot see` section -- which is where this table names its gaps, and which the
round did not touch -- does not mention them, and no task is named.

**What would close it.** Table C: for an `entry != "class"` row, ask one question the documented
entry does answer, so the bound is non-zero -- `.RexxInfo~class~id` is one line the environment
entry answers and an absent name does not. Table D: the ruling's second branch, in
`# What this table cannot see`.

**What the check I ran would have done had the claim been false.** Control A' would have exited 101
with the class row reading `unanswered` and the 5a line reading `136 rows, 136 not yet agree`, which
is what control A (the same row filed `class`) does. D3 would have exited 101 or left the row
`diverge-both`. Both exited 0 with the row `agree`.

*(Honesty check on A': `rexx-extract`'s `every_row_set_is_exactly_what_its_extractor_derives_today`
does catch a hand-added `class-set.txt` row -- I ran it, and it names `Zork` -- so that particular
route to the false green is closed by a different target. The route that matters is not hand
editing: it is a row the extractor derives correctly from the books naming something this build does
not ship, which is `RegularExpression`'s case and is exactly what finding 1 was about.)*

### N2. LOW-MEDIUM -- a sentence this round added to a tracked README is false, measured

`corpus/gate-tables/README.md`: *"`directives/` has the same check, against the one line each of its
probes prints."* Measured, every directives probe run through the oracle wrapper with the three
descriptors read separately: **72 print one line and 7 print none**, and the 7 are exempted from the
check rather than checked against one line. The same paragraph opens *"Every row's oracle side is
checked for having answered at all, before any verdict exists"*, which is false for those 7 and for
the `RexxInfo` class row.

Unlike the report, the README is tracked and is what a reader consults to learn what protects a
row's verdict. The paragraph's other three clauses are accurate.

A smaller inaccuracy in the same sentence: for `classes/` the bound is the derived probe text
**gated on the `entry` column**, not the derived text.

### N3. LOW -- the summary sentence the ruling demanded be stated can be false

`gate_table_c.rs:1773` computes `constructor_raised` from `program.oracle_stdout.is_empty()` and
prints *"method rows whose group's probe raised at ~new on the oracle, so no documented name was
asked **on either side**: 497"*. The `unanswered` verdict uses a both-sides predicate; this sentence
uses an oracle-only one. Today both read 497, so nothing is wrong now. A group whose oracle raised at
`~new` while this crate answered would be counted by the sentence and would **not** be `unanswered`,
and the reader would meet a false claim in the line the ruling asked for specifically because a
reader meets it. One line: sum the method rows whose verdict is `None`.

### N4. LOW -- `verdict_label` and `UNANSWERED` are duplicated verbatim in both test binaries

Both files `mod gate_tables`, and `Verdict`, `verdict()`, `Report` and `Structural` already live
there. `verdict_label` and the `UNANSWERED` label are now defined identically in
`gate_table_c.rs:1840`/`1848` and `gate_table_d.rs:362`/`358`; `gate_table_d.rs`'s
`stdout_line_count` is a third copy of table C's "empty input is no lines" rule. Nothing makes the
two labels stay the same string, and a report consumer reading both tables would not see them drift.

### N5. LOW -- two new comments carry the historical framing the plan's own rule strikes

The rule ruled at Task 1: an account of what the code did **before** is history; the test is to
strike the historical framing and see whether the sentence still says the same thing about the code
as it is.

* `check_probe_text`'s error arm: *"Measured: one `0xff` byte appended to a probe whose body asked
  about another class left the table exiting 0."* The three sentences before it already state the
  contract in full.
* `first_difference`: *"so a file differing from its derivation only in line endings produced two
  identical empty strings and a message that told the reader nothing."* The `lines` explanation and
  the `.gitattributes` sentence around it carry the whole justification.

`Measured::verdict`'s "Dropping it would ..." is not this shape -- it is a rejected route stated as
the reason the current type is `Option`, which the same rule keeps.

### N6. LOW -- the module doc's absolute claim now has an exception it does not name

`gate_table_c.rs`'s `# The table types no expected bytes` says *"no oracle answer is recorded
anywhere in this file or beside it"*. `Concept::oracle_lines` is a committed observation of what the
oracle prints. The field's own doc meets the objection squarely ("it holds no byte the oracle
answered", "it can only make a row **fail**") and I checked the field is not a rubber stamp -- for
six of the concept probes the committed count differs from the probe's own top-level `say` count,
because some `say`s raise and one prints from a method body, so the number carries information the
text does not. The section heading was simply not qualified when the field landed.

## Observations, no action asked

* **The permission-denied limb of finding 2 is implemented but never observable.**
  `check_probe_text` does push a `Structural` for it, but `run_on_both_engines` panics at
  `gate_tables/mod.rs:196` before `assert_no_structural_failures` runs, so what a maintainer meets is
  `cannot read .../classes/array.rex: Permission denied (os error 13)`. Red, and it names the file,
  so there is no correctness consequence -- recorded because the report's description of that arm is
  not what the run shows.
* **`run_probe` still drops a row entirely on its two failure paths**, which is the shape the round's
  own `Measured::verdict` doc argues against. Both paths are structural-red in every mode, and the
  doc's sentence is scoped to "a row that fails the check", so nothing here is false. Pre-existing.
* **The ledger does not carry finding 5's note about the uneditable commit message.** The ruling put
  that half on the ledger rather than on the report, and the report does flag the deviation, so this
  is the controller's half rather than the task's.

## Every control and mutation I ran, and what it returned

The debug build of the scratch copy reproduces the release run's verdicts exactly -- `agree 32 /
diverge-both 937 / diverge-stdout 22 / unanswered 497`, `5a: 135 rows, 135 not yet agree` -- so it is
a valid platform. `plainc.sh`/`plaind.sh` run the table with no environment set, so **exit 0 means no
structural failure** and exit 101 means there is one. The corpus was restored from a pristine copy
after every control.

### The five gate commands, on the committed tree, each status read unpiped

| command | exit | note |
|---|---|---|
| `cargo fmt --all --check` | **0** | |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** | |
| `cargo test --release --workspace` | **0** | |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0** | corpus **106 of 106** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** | corpus **106 of 106** |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101** | expected |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --test gate_table_c` | **101** | **135** gated |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --test gate_table_d` | **101** | **10** gated |
| `cargo test --release --test gate_table_c` / `--test gate_table_d` | **0** / **0** | structurally clean |

**Both gated counts, re-derived from the report's own row lines rather than from its summary.**
Predicate: a row line whose owning-phase field reads `5a` and whose verdict label is not `agree`.
Table C: **135** such lines, of which **0** read `agree` -- 135 gated, matching the summary. Table D:
**36** 5a row lines, **26** reading `agree` -- **10** gated, matching. The two tables were run
separately, because `cargo test` without `--no-fail-fast` stops at the first failing target.

### The two demonstrations

| | before (reviewer) | now |
|---|---|---|
| **A** -- synthetic `Zork` class row, `Zork <- Object` edge row and three `Zork` instance method rows, all with correctly derived probes | exit **0**, all five `agree`, `5a: 137 rows, 135 not yet agree` | exit **101**. Class and edge rows `unanswered`, method group `unanswered=3`; two structural failures naming `classes/zork.rex` (`the oracle answered 0 line(s) where this row's probe asks for exactly 6`) and `hierarchy/zork__object.rex` (`for exactly 4`); `5a: 137 rows, 137 not yet agree`. **The gated count moves the way the round says: the two new rows are counted, not dropped.** |
| **B** -- `classes/array.rex` rewritten to ask `.String~id` with one `0xff` byte appended | exit **0**, the derivation check never ran | exit **101**, two instruments: `the probe is listed in the directory but its bytes cannot be read as text: stream did not contain valid UTF-8`, and `the oracle answered 0 line(s) where this row's probe asks for exactly 6`. **The message names the cause and not "missing".** |

### Everything else

| # | what was changed | result |
|---|---|---|
| baseline, real repo, release | -- | exit 0; agree 32, diverge-both 937, diverge-stdout 22, unanswered 497, loud 1432 |
| baseline, scratch copy, debug | -- | identical, including `5a: 135 rows, 135 not yet agree` |
| **A'** | **the same synthetic `Zork` class row, filed `entry = instance`**, derived probe committed, no edge or method rows | **exit 0. The row reads `agree`. `5a: 136 rows, 135 not yet agree`** -- **N1** |
| A'-check | `cargo test -p rexx-extract --test extract_docs` with that row present | FAILED, naming the `Zork` row as committed and no longer derived -- the hand-edit route is closed elsewhere |
| **D3** | `directives/attribute__external__subkeyword.rex` (a gated 5a row in `ORACLE_REFUSES`) replaced by `nop` | **exit 0. The row moves `diverge-both` -> `agree`; 5a open 10 -> 9** -- **N1** |
| C1 | `rm classes/array.rex` | exit 101, reported **once**, by the set check -- the `on_disk` guard holds |
| C2 | `chmod 000 classes/array.rex` | exit 101, but as a panic at `gate_tables/mod.rs:196` (`Permission denied`) before the structural channel is reported -- see Observations |
| C3 | final newline stripped from `classes/array.rex` | exit 101, `line 12 derived: "say 'isa-class' .Array~isA(.Class)\n" / committed: "...(.Class)"` -- finding 4 |
| C4 | `classes/array.rex` converted to CRLF | exit 101, `line 1 derived: "...questions\n" / committed: "...questions\r\n"` -- finding 4 |
| **C5** | `Alarm` flipped to `covered` in both row sets **and its instance probe regenerated to the covered derivation** -- finding 3's construct | **exit 0, no structural failure**; the group reads `unanswered=7`. The failure whose message pointed away from the cause is gone |
| C5b | the same flip with the probe left alone | exit 101, and the message is the derivation check naming the probe and showing the two headers -- it points at the row/probe pair |
| C6 | `RexxInfo Object` edge appended with its derived probe | exit 101, `its child, RexxInfo, is not a class row of class-set.txt` |
| C7 | `Alarm RexxInfo` edge appended with its derived probe | exit 101, the same check on the **parent** end -- `check_edge_endpoints` is not vacuous at either end |
| C8 | ` */ oops` appended to `Alarm`'s `reason` | exit 101, `class-set.txt row Alarm: its reason contains */, which ends the Rexx block comment...` -- finding 7 |
| C9 | `say 'extra'` inserted into `concepts/unkno.rex` | exit 101, `the oracle answered 2 line(s) where this row's probe asks for exactly 1` |
| C10 | `unkno.rex`'s send retargeted at an undefined class | exit 101, `the oracle answered 0 line(s) ... for exactly 1` |
| C11 | `Alarm Object` edge rewritten to `Alarm Comparable` with its derived probe | exit 101, `the oracle does not confirm the documented edge Alarm <- Comparable: its documented-edge answer is Some("0"), not "1"` |
| C14 | a file with a non-UTF-8 name created in `classes/` | exit 101, `a file in the probe directory whose name is not valid UTF-8` |
| E1 | `ArgUtil Object` edge appended **with a correctly derived probe** | exit 101 with **the ArgUtil assertion as the only structural failure** -- the round did not shadow it with `check_edge_endpoints` |
| E2 | `Array of class` deleted from `class-methods.txt`, probe left | exit 101, two instruments: the derivation check (`line 7 derived: "" / committed: "say 'class' .Array~hasMethod(\"of\")\n"`) and `the oracle answered 2 line(s) where this row's probe asks for exactly 1` |
| D0 | table D baseline, scratch copy | exit 0; agree 31, diverge-both 48; `5a: 36 rows, 10 not yet agree` |
| D1 | `("::METHOD", "EXTERNAL")` removed from `ORACLE_REFUSES` | exit 101, `the oracle answered 0 line(s) on stdout where this row expects 1 ... this row is not named there`. The row **keeps its place** and reads `unanswered` |
| D2 | `("::CLASS", "PUBLIC")` added to `ORACLE_REFUSES` | exit 101, `the oracle answered 1 line(s) ... expects 0 ... this row is named there`. The row moves `agree` -> `unanswered`, **the 5a total stays 36**, so no row vanishes |
| oracle sweep | all directives probes run through the wrapper, three descriptors separately, from a fresh empty directory | **72 print one line, 7 print none**; the 7 are exactly `ORACLE_REFUSES`'s members. The committed list is correct and complete against this build |
| finding 5 | which agreeing rows belong to a class-arm-only class | Buffer, Singleton and Validate are the only class-arm-only classes; of the 32 agreeing rows exactly **one** is theirs (`Buffer`'s). The corrected sentence and its "the table would read 1, not 0" are true |
| finding 3, second half | is the summary line on a **green** run? | yes -- `emit_uncaptured` writes the report to an inherited stderr, and `cargo test --release --test gate_table_c` (exit 0) carries the 497 line |
| concept counts | committed `oracle_lines` against each probe's own `say` count | 15 match, **6 differ** (`abscla`, `obdes`, `pubpri`, `reqstr`, `xmeths`, `xscope`), so the field is not derivable from the text and is not a rubber stamp |
| constraints | non-ASCII bytes in the three changed files; numerals in added comment lines | 0 non-ASCII lines in each; 5 added comment lines carry a digit, all of them an error code, an encoding name or a measurement -- no set cardinality. No `unsafe` |
| restore | corpus and tests restored, real repo checked | `git status --porcelain` empty, `HEAD` at `938916aa2` |

## What I could not check

* **Whether any concept probe is faithful to its section's prose.** Unchanged from the review, and
  the round's own "could not close" says the same. `oracle_lines` narrows it -- a probe that stopped
  reaching its mechanism reddens -- but a probe edited together with its count is not caught.
* **Whether the `RexxInfo` wiring row could go green on a build that still ships `.RexxInfo`.** I did
  not construct that; I showed the mechanism on a synthetic `instance` row and confirmed the oracle's
  current answer for `classes/rexxinfo.rex` (rc 159, `Object "a RexxInfo" does not understand message
  "ID"`, empty stdout), against which this crate today produces its own refusal. So the row is not
  falsely green **now**; N1 is about the direction the fix exists to cover.
* **Whether table D's `::ATTRIBUTE EXTERNAL` and `::METHOD EXTERNAL` will in fact be made `agree` by
  a later task.** I demonstrated the mechanism with a `nop` probe rather than by making this crate
  reproduce the oracle's install-time failure.
* **Whether the mutations behave the same under `--release`.** The mutation platform is a debug build
  of a copy whose unmutated baseline reproduces the release verdicts exactly; I did not repeat the
  controls under `--release`.
* **Performance.** None owed: the commit touches `tests/` and a README only, so the release binary
  the axes measure is byte-identical.
* **Whether any row's owning phase is right.** Nothing checks it, by design (R32), and this round did
  not change that.
* **CI.** `support::oracle::locate()` still asserts rather than skips when the oracle binary is
  absent; inherited, out of scope, not read.
