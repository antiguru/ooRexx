# Review: Phase 9d plan (2026-10-18)

Plan: `docs/superpowers/plans/2026-10-18-phase-9d.md` at `b44ffe45f`.
Spec: `docs/superpowers/specs/2026-10-17-phase-9d-divergences-and-close-design.md`.
Parent: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md`.
Spec review: `.superpowers/sdd/2026-10-17-phase-9d-spec-review.md`.
Every claim below was checked against the tree at `b44ffe45f` by grep, by reading the file named, or by running the command quoted. Line numbers are `file:line`.

## Summary for Moritz

The plan covers all six spec goals with ten tasks, three of them templates the controller fills from tables the first three tasks produce, and the task order and dependencies are right.
Two findings block: the refusal enumeration (Task 3) is built from a table in which no `native_method` refusal appears, so it cannot find the four reached sites already on record, among them the two 9a owed to 9d that the spec names by line; and `native_method` is one constructor behind six sites with three different fates, so the "one GUARD row" the last Task 7 instance is told to write would be a false guard over a reachable, Deviation-ruled site.
Seven major findings are text fixes: the yaml witness needs `yaml.cls` on a path the corpus harness does not set, the harness keeps no output from which Task 1's `cause` column can be read, "the groups the touched files feed" names a map that does not exist (the whole suite runs in 4m38s and should be run instead), the per-task perf delta is measured against the wrong baseline and lands in `/tmp`, two test fixtures that pin Phase 9 ownership are named in no task, the Review Focus obligations are not in the templates, and nothing says the fix instances run one at a time.
Nothing contradicts your rulings; two Global Constraints over-constrain against `rust/CLAUDE.md` (semicolons, `--`).
With the two blocking fixes and the major text fixes, ready.

## Verdict

**Not ready.** Two blocking findings, both in the goal 2.3 chain (Task 3 feeds Task 7). Seven major, nine minor.

## Coverage of the spec goals

| Goal | Tasks | State |
|---|---|---|
| 2.1 The 9d set | 1, 6.n | Covered. One inverted sentence, m1. |
| 2.2 json, yaml, dateparser | 4 | Covered. The witness cannot run as written, M1. |
| 2.3 Reachable refusals | 3, 7.n | Not covered as specified: the enumeration misses the sites the spec names, B1; the last instance's GUARD is false, B2. |
| 2.4 Queued inventory | 2, 8.n | Covered. Template gaps, M6. |
| 2.5 No refusal names Phase 9 | 5, 7 (last), 9 | Covered. Two fixtures unnamed, M5. |
| 2.6 The close | 9, 10 | Covered. Script workability, m6. |

## Rulings check

| Ruling | Where the plan states it | Bent |
|---|---|---|
| 1. SysFork and named semaphores, Deviations 42 and 43 | :33-34, Task 5 | No. Entries exist at `phase-4-exclusions.txt:2313` and `:2327`, numbered `42.`/`43.`, which `deviation_citations` (`refusal_dispositions.rs:574-581`) resolves. |
| 2. json, yaml, dateparser in 9d | :35, Task 4 | No. |
| 3. Corpus and RxFunc* to Phase 10 | :36-37 | No. |
| 4. Coverage for reachable sites, ooTest not extended | :38-39, Task 3, Task 7 | Not bent, but not reachable through Task 3 as written (B1). |
| 5. Every queued item inventoried, one batch, 9d fixes only what he accepts | :40-41, Task 2 Step 3 | No. |
| 6. A Deviation only after his ruling, quoting him | :42, Task 6 Step 2 | No. |

## Blocking

### B1. Task 3's enumeration cannot find the sites the spec names

Task 3 Step 1 (:184-192): the candidate set is "the `loud` rows of `rust/corpus/method-bodies.txt` together with every method in `rust/corpus/docs/class-methods.txt` that has no body", per (class, method) pair, with `hash.rs` the one per-receiver exception.

Evidence.
* `grep -P '\tloud\t' rust/corpus/method-bodies.txt` prints 17 rows: `Message~result`, `Message~wait`, `EventSemaphore~wait` (a wait refusal) and `RexxQueue`'s LIBRARY REXX entry points (Phase 10). None is a `native_method` refusal.
* `grep -rn 'Loud::native_method' rust/crates/rexx-exec/src` finds six sites: `dispatch.rs:2500` (the fallthrough), `hash.rs:214`, `context.rs:1180` (`EXECUTABLE` on `StackFrame`), `object_protocol.rs:487` (`OBJECTNAME=`), `object_protocol.rs:658` (`COPY` on a non-Package primitive), and a comment at `lib.rs:1132`.
  Four of the five are inside methods that have bodies and refuse for some receivers only. A per-(class, method) derivation marks them as having a body.
* Spec 2.3 :79-81: "The two sites 9a recorded as owed to 9d are among them (`phase-9a-gate.md:287`, `:291`): `StackFrame~executable` on a library routine frame, and `copy` of the REXX package." Those are `context.rs:1180` and `object_protocol.rs:658`. The plan names neither anywhere (`grep -n 'executable\|REXX package' docs/superpowers/plans/2026-10-18-phase-9d.md` prints nothing).
* "The script states how a body is found" leaves the derivation to the implementer. A body is resolved at run time through four maps (`invocable`, `dispatch.rs:2472-2500`: `natives`, `method_bodies`, `generated_methods`, `native_externals`, `library_externals`), one of them filled from `CoreClasses.orx`. There is no static table to read, and the plan does not say which test or probe answers the question.
* Review Focus 5 (:123-126) expects the subclass probe to find what a plain one misses. The known reaches are not subclass-shaped: a library routine's frame, the REXX package, the condition object's directory, `.methods`.

Consequence. Task 7's instances are written from `refusals.tsv`. A table that misses these sites never implements them, the last instance removes `native_method`'s owner, and the Phase 9 close ships reachable refusals under a GUARD row, which anti-goal "No silent licences or guards" forbids.

Fix.
* Seed `refusals.tsv` from the six `Loud::native_method` sites, one row per site and receiver condition, with the four receivers already on record: a library routine frame (`phase-9a-gate.md:287`), the REXX package (`:291`), the condition object's directory and `.methods` (`queued/2026-10-01-condition-object-directory-methods.md`), plus `OBJECTNAME=` on a String or SmallInt (Deviation 36, `refusal-dispositions.tsv:51`).
* For the fallthrough at `dispatch.rs:2500`, state the derivation: every `class-methods.txt` row whose `(class, METHOD)` `lookup` resolves and `invocable` refuses, measured by a probe that sends each row's name to a receiver of each kind the class can have. Name the probe harness (`method_bodies.rs` already sends every row with no arguments; its receiver is one construction expression per class, `method-bodies.txt:3-4`).
* Keep the plain and subclass probes, and add the receiver kinds above.

### B2. One constructor, six sites, three fates: the last Task 7 instance writes a false GUARD

Task 3 Step 3 (:197-199) and Task 7 (:265-266): every `reached = no` row "becomes an internal guard ... with a GUARD row in `refusal-dispositions.tsv` citing `refusals.tsv`", and the last instance "removes `Loud::native_method`'s `Owner::Phase9`".

Evidence.
* `refusal_dispositions.rs` keys rows by constructor (`:642-650`, `seen` over `row.constructor`). `native_method` (`lib.rs:684`) is one constructor. It gets one row.
* GUARD means "no program reaches it" (`refusal-dispositions.tsv:12-13`). `object_protocol.rs:487` is reached by any program sending `OBJECTNAME=` to a String, and stays refused under a ruled Deviation (`phase-9a-gate.md:292`, `refusal-dispositions.tsv:51`). A GUARD row on `native_method` is false the moment it is written, and the checker cannot see it: "`refusal_dispositions.rs` checks that rows exist, not that they are true" (spec review B2).
* The fixture `the_scanner_reads_each_owner_form` (`refusal_dispositions.rs:783-788`) asserts `native_method` carries `Phase 9`. The last instance reddens it and no task names it (see M5).

Fix. The last Task 7 instance gives each site its own disposition: the Deviation-ruled site uses a constructor with a DEVIATION row (`object_name_of_a_value` already exists for this, `refusal-dispositions.tsv:51`); an implemented site deletes its call; `native_method` keeps only the sites `refusals.tsv` shows unreached, and its single GUARD row cites that table. The instance updates `refusal_dispositions.rs:783-788`.

## Major

### M1. Task 4's witness needs `yaml.cls`, which the corpus harness does not provide

Task 4 Step 1 (:216-217): "Add the repro as a corpus program." Review Focus 2 (:116): the repro "is independent of either copy".

Evidence.
* The repro (`final-review.md:82-88`) ends `::requires 'yaml.cls'`, and the review ran it "with the oracle's `build/bin/yaml.cls` beside them" (`:80`). It is not independent of either copy; it depends on whichever `yaml.cls` the search path finds.
* The corpus harness runs each side from the program's own directory (`tests/support/oracle.rs:333`) and sets no `PATH` (`grep -n PATH rust/crates/rexx-exec/tests/corpus.rs` prints only a comment at `:680`). `lay_class_files` (`suite.rs:342`) is the suite harness's. No `corpus/lang/*.rex` requires an external `.cls` today (`grep -h -i '::requires' rust/corpus/lang/*.rex` shows LIBRARY and inline-package shapes only).
* Two refusal texts are reached (`phase-9a-gate.md:488-490`, spec review B2): "a pass boundary with no loop open" from `run/loops.rs:1859` and "Store" from `ir/drive.rs:3782`. One repro reaches one.
* Task 4's files (:209-210) name `lib.rs:877`, which is the constructor, plus the two driver sites without their directories (`run/`, `ir/`). The queued item titles the defect a "compiler GUARD" (`2026-10-13-yaml-pass-boundary-guard.md:1`) and the repro fires at a `do while` end after a nested call returns (`final-review.md:90-91`). If the fix is in the compiler, the "Shared paths" rule (:104-105) makes the implementer stop.

Fix. Reduce each refusal to a pure-Rexx witness (the `do while` end after a routine returning a collection, and the Store shape), one program per text, in `rust/corpus/phase-9.txt`; the yaml groups under the suite harness stay the acceptance. List `rust/crates/rexx-exec/src/ir/` and `src/run/loops.rs` as files. Rewrite Review Focus 2 to say the witnesses are pure Rexx, and that the harness groups cover the oracle's copy.

### M2. Task 1's `cause` column has no source in the harness

Task 1 Step 3 (:144-146): "record the first divergence: the assertion that failed, and the crate's refusal text or wrong value."

Evidence. `GroupRun` (`suite.rs`, `pub struct GroupRun`) holds test-name sets, the `blocked` texts and counters; the suite writes four tables (`suite.rs:1061-1092`) and no per-group output. The 9c close's `tests.tsv` crate column is `passed`, `failed`, `blocked: <refusal>` or `not started`. An assertion message exists only in the framework's own stdout, which is discarded.

Fix. Say how the text is obtained: run the group's framework invocation by hand on each side (the commands are in `run_group_on_oracle_from`, `suite.rs:574`, and `run_group_on_crate_within`, `:639`), keeping stdout and stderr per side under `docs/superpowers/records/2026-10-18-phase-9d/causes/`; or add a log directory switch to the harness as part of Task 1 and name it.

### M3. "The groups the touched files feed" is not derivable, and the whole suite fits the bound

Task 6 Step 3 (:249-251) and Review Focus 1 (:109-112).

Evidence. No file maps source files to ooTest groups. The 9c close's suite step took 4m38s (`close/status.txt`: workspace exit at 19:55:57Z, suite exit at 20:00:35Z), inside the 5-minute per-task bound (:76).

Fix. Every Task 6, 7 and 8 instance runs the whole suite (`REXX_SUITE_GATE=1`, no `REXX_SUITE_GROUPS`) and reports every row whose result differs from `close/suite/tests.tsv`. Delete "the groups the touched files feed".

### M4. The per-task perf delta is taken against the wrong baseline and lands in `/tmp`

Global Constraints :87-94.

Evidence.
* "The task's own delta on each program is at most +0.05%, taken against the 9c close's figures." That is the first task's own delta. For every later task it is the sum of all 9d drift so far, and the bound trips on accumulation, not on the task.
* The 9c record has two blocks (`close/callgrind.txt`): `churn` and `parse` only in the `48fb59b51` block, `startup`, `sayloop` and `interpret` re-measured at `25197002a` (`startup` +0.0181 vs +0.0162). The plan does not say which figure is the baseline.
* The command has no `-o`, so `callgrind.sh` writes to `mktemp -d` (`callgrind.sh:44`), under `/tmp`, against :101 "Raw callgrind files go in `cg-9d/`".

Fix. Add `-o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/<task>`; the task copies `table.txt` to `docs/superpowers/records/2026-10-18-phase-9d/perf/<task>.txt`. The task's own delta is its `head d%` minus the previous perf-checked task's `head d%`; the first task's baseline is the `25197002a` block for `startup`, `sayloop`, `interpret` and the `48fb59b51` block for `churn`, `parse`.

### M5. Two fixtures that pin Phase 9 ownership are named in no task

Evidence.
* `refusal_dispositions.rs:783-793`, `the_scanner_reads_each_owner_form`, asserts `native_method` and `named_semaphore` carry `Phase 9`. Task 5 (`named_semaphore`) and the last Task 7 instance (`native_method`) each redden one assertion. Task 5's files say "the tests that pin the refusal texts" (:231-232); this one pins the owner, not the text.
* Task 5's text fixtures, for the record: `src/run/tests/directives.rs:321-355`. These are string literals under `src/`, so `closed_phases` would also scan them once Phase 9 is in `CLOSED`.
* Task 9 (:278-279) says "the rest named by the compiler". Not these: `refusal_dispositions.rs:843` and `:864` are `Owner::Phase9` inside fixture source strings the scanner parses, and `:945` (`hostemu\tREHOME\tPhase 9`) expects "REHOME to Phase 9, but the constructor", which changes once Phase 9 leaves `OPEN_PHASES` (`:663-666`). The test run finds them, the compiler does not.

Fix. Name `refusal_dispositions.rs:783-793` in Task 5 and in Task 7's last instance; name `:843`, `:864`, `:945` and `directives.rs:321-355` in Tasks 9 and 5.

### M6. The templates drop the Review Focus obligations and do not fit Task 8

Evidence.
* Review Focus 3 and 4 (:117-122) assign steps to "the streams cluster's instance" and "the traceback cluster's instance". The Task 6 template (:245-251) carries neither. An instance written from the template alone omits the `'80'x` name and the one-program-per-path witnesses.
* Task 8 (:268-272) "follows Task 6's steps". Step 3 runs "the cluster's tests" and removes lines from `suite-9d-set.txt`; a queued item has neither. "Takes the queued file's probe as its witness" is undefined where none exists (Task 2 Step 2: "when one exists").

Fix. The Task 6 template says: "an instance copies the Review Focus lines that name its cluster as steps". Task 8 says: "write the witness from the queued file's probe, or write one where the file has none; Step 3's suite check is the whole suite (M3)".

### M7. Shared files, and nothing says the instances run one at a time

Every Task 6, 7 and 8 instance writes `rust/corpus/refusal-dispositions.tsv`, `refusal-sites.tsv`, `phase-9.txt`, `suite-9d-set.txt` and `docs/superpowers/plans/phase-4-exclusions.txt`. The "Shared paths" rule (:104-105) covers files outside a task, not files every task owns. The spec review's decomposition said "one writer at a time". Fix: "Tasks 6, 7 and 8 instances run one at a time; Tasks 1, 2 and 3 may run in parallel."

## Minor

* m1. Task 1 Step 1 (:140-141): "Keep the 9b and 9c witness exclusions only where those tests still fail" is inverted. A witness test that still fails with no row belongs in the set (spec 2.1 :25-28). Measured over the 9c close's `tests.tsv`: the header's awk filter and the same filter without the exclusions both print 178 rows, and none of the excluded tests fails, so drop the exclusions. 14 rows of the current file pass at the 9c close.
* m2. Task 2 Step 1: `phase-9b-gate.md` has no carried list (`:118-121` is a note; spec review "New claims checked"). Step 4: of 157 queued files, 3 open with the bold `**CLOSED ...**` first line the plan asks for, and 24 carry a resolution word somewhere in their first twelve lines (`head -12 | grep -i 'CLOSED\|RESOLVED\|RE-HOMED\|DELIVERED'`); say that a file already resolved gets its first line rewritten into the shape, not a second marker.
* m3. Task 5 (:230-231): "the fork refusal at `:565`" is the owner line; `fork_child` is at `lib.rs:557`.
* m4. Task 9 Step 1: name the test, `the_literal_scan_finds_a_phase_inside_a_longer_text` (`closed_phases.rs:305`), and the predicate, `names()` in `tests/support/literals.rs:164-171`, which `phase_ten_reasons.rs` shares. The "Consumes" check `git grep -n '"Phase 9"'` misses a literal like `directives.rs:322` ("(Phase 9)" inside a longer string); the check that matters is `closed_phases` green with `"Phase 9"` in `CLOSED`.
* m5. Task 10 Step 1: `bggates.sh` re-runs fmt, clippy, the release build and the release tests that `run.sh` already ran (its runs took about 50 minutes each, `bg/*/status.txt`), its G3 and G5 builds carry no `memcap` where `run.sh`'s own build needed 16G, it writes under `.superpowers/sdd/2026-10-07-phase-6-1/bg/<sha>/`, and `tsan.sh` needs `CARGO_BUILD_JOBS=4 memcap 8G bash tsan.sh <target> <logs>` from `rust/` on nightly (`phase-6-1-gate.md:1266`), which the plan does not give. Fix: run G5-G9 only or cap G3/G5, state the TSan invocation, and copy both status files into `close/`.
* m6. Task 10 Step 3: add the set file's header derivation run over the close `tests.tsv` printing nothing (spec review M5), and the five extension groups' rows.
* m7. Task 9 Step 3 assumes a Task 6 instance fixed the INTERPRET traceback row. If the traceback cluster ends in a Deviation, the row closes with that note instead. Say "with the disposition the cluster's instance recorded".
* m8. Review Focus 5 names `enhanced`; Task 3 Step 2 probes plain and subclass only.
* m9. Global Constraints :67 "no structuring semicolons" and "no ` -- ` in new prose" contradict `rust/CLAUDE.md:132` ("Prefer `--` over an em-dash. There is no rule against semicolons in comments here"). Harmless over-constraint; delete both.

## Templates: what each table must hold

* `clusters.tsv` (`group`, `test`, `cluster`, `cause`) plus `clusters.md` (count, likely files, Deviation needed). Enough for a Task 6 instance once M2 makes `cause` obtainable and M3 removes the group map.
* `refusals.tsv` (`class`, `method`, `receiver`, `site`, `reached`, `probe`, `oracle`). Enough for a Task 7 instance once B1 seeds the sites and receivers. Add a `disposition` column (`implement`, `deviation:<n>`, `guard`) so the last instance (B2) is written from the table.
* `queued-inventory.tsv` (`item`, `kind`, `proposed`, `reason`, `ruled`). Enough for a Task 8 instance once M6 says what a row with no probe gets. Add a `probe` column (`file:line` or `-`).

## Per-task check and close: workability

* The per-task command list (:76-86) names real switches and binaries: `REXX_SUITE_GATE`, `REXX_SUITE_GROUPS`, `REXX_SUITE_TABLE` (`suite.rs:49-56`, `suite_rows.rs:54`), `REXX_REFUSAL_SITES_REFRESH` (`refusal_sites.rs:503`), the test binaries `owners`, `closed_phases`, `phase_ten_reasons`, `representation_refusals`, `refusal_dispositions`, `refusal_sites` (all under `rexx-exec/tests/`), `memcap` at `~/.local/bin/memcap`.
* `callgrind.sh` takes `-B`, `-p`, `-o` and `NAME=BINARY` (`callgrind.sh:5-12`); the five program names are in `PROGRAMS` (`:48-50`); the pin exists and its sha256 begins `c4b4e8c4`. Workable with M4's fixes.
* `close/run.sh` of 9c runs fmt, clippy, the capped build, the gated workspace and the suite, and the plan's additions (G1-G9, TSan) are real scripts; see m5 for what they cost and lack.
* The suite at the 9c close exited 101 with `finished`; the 9d close requires 0 (:324). A cluster that ends in a licence row needs `suite-licences.txt` (`rust/corpus/suite-licences.txt:1-3`, "added only by Moritz's ruling"), which Task 6 Step 2 reaches through the controller.

## Review Focus: is each line pinned?

| Line | Pinned by | Gap |
|---|---|---|
| 1. A fix reddens a test elsewhere | Task 6 Step 3, Task 10 Step 3 join | M3: the group set is undefined; run the whole suite. |
| 2. The yaml copy | Task 4 Step 4 (harness groups) | M1: the witness is not copy-independent; nothing runs the repository's copy. |
| 3. Byte names on stream paths | nothing | M6: not in the template. |
| 4. Conditions under INTERPRET and REPLY | nothing | M6: not in the template. |
| 5. A refusal reached only through a subclass or enhanced object | Task 3 Step 2 (subclass) | m8: `enhanced` unprobed; B1: the known reaches are receiver-shaped, not subclass-shaped. |

## Claims checked and true

* `suite-9d-set.txt`'s header holds the awk filter and names the 9a path; the 9c close's `tests.tsv` exists with the same columns; `25197002a` is the 9c close commit.
* `2026-10-11-output-lost-on-kill.md:1` has the `**CLOSED <date> by Phase 9c Task 7 (<sha>).**` shape.
* `phase_10_group` (`suite_rows.rs:278-289`) answers `Extension` by the `extensions/` prefix; `derived_rows` (`:633-656`) emits a `group` row per test of such a group; the `PASSING` fixture is at `suite.rs:1516` and `FAILING` at `:1544`; `extensions/rxmath/RxMath` is a group (`phase-9-exclusions.txt:922`); 453 derived rows name json, yaml or dateparser.
* `named_semaphore` at `lib.rs:540`, `Owner::Phase9` at `:548`, `:565`, `:689`; `op_not_driven` at `:877`; `loops.rs:1859`, `drive.rs:3782`.
* `Owner::Phase9` at `owner.rs:57,67,80,98-102`; `OPEN` at `native/tests.rs:150`; `PHASES` at `internal_routines.rs:107`; `CLOSED` at `closed_phases.rs:37`; `OPEN_PHASES` at `refusal_dispositions.rs:60`; the open `OWNER: Phase 9` row at `phase-4-exclusions.txt:6390-6391` (`:5830` and `:6454` are DELIVERED); roadmap row 9 at `2026-07-27-rust-rewrite.md:661`, rows 6 to 8 carry `**CLOSED <date>**`.
* `hash.rs:214` passes `method` with the receiver's class id; `queued/2026-10-01-condition-object-directory-methods.md` exists.
* `p61-gates/bggates.sh` and `tsan.sh` exist; `rust/tsan.supp` exists; nightly is installed.

## Re-review

Revision `3abf2e2ef` (2026-10-18), scoped to B1, B2, M1-M7 and m1-m9. Every new citation was checked against the tree at `3abf2e2ef`.

### Verdict

**Ready with fixes.** Both blocking findings are resolved. Three new citations point at the wrong lines or a wrong file and would send an implementer astray; five minor residues. No finding needs Moritz.

### First-round findings

| # | State | Where |
|---|---|---|
| B1 enumeration | Resolved. Seeded from the five `Loud::native_method(` sites (`grep` prints 5), each known receiver its own row, a `disposition` column, the fallthrough's reach by probe. Two citation errors, R1. | :212-235 |
| B2 one constructor, three fates | Resolved. The last Task 7 instance splits by disposition into separate constructors, citing `refusal_dispositions.rs:642-650`; the `:783-788` assertion moves with it. | :314-323 |
| M1 yaml witness | Resolved. Pure-Rexx witnesses with no `::requires`, one per refusal text; `src/ir/` listed. One wrong path, R2; one unmechanised step, R5. | :245-265 |
| M2 cause column | Resolved. By-hand runs as `suite.rs:574` and `:639` do (both build a `work_dir`), stdout and stderr kept under `causes/<group>/`. | :150-155 |
| M3 group map | Resolved. Every task runs the whole suite and reports changed rows. Bound residue, R4. | :82-87, :117-118 |
| M4 perf baseline | Resolved. `-o cg-9d/<task>`, delta against the previous perf-checked task, table copied to records. Residue, R8. | :90-99 |
| M5 fixtures | Resolved. `refusal_dispositions.rs:783-793` in Task 5, `:783-788` in Task 7's last instance, `:843`, `:864`, `:945` in Task 9. One wrong file, R3. | :272-276, :322-323, :348-349 |
| M6 templates | Resolved. Task 6 carries Review Focus 3 and 4 as steps; `clusters.md` marks which cluster they belong to; Task 8 writes a witness where the item has no probe and runs the whole suite. | :164-165, :288-301, :330-336 |
| M7 serial instances | Resolved. | :100-102 |
| m1 inverted exclusion sentence | Resolved. | :145-149 |
| m2 9b carried list, markers | Resolved. | :183-185, :193-196 |
| m3 `fork_child` line | Resolved (`:557`, owner `:565`). | :270-271 |
| m4 `names()` and the test named | Resolved. | :344-347, :355-360 |
| m5 bggates cost, caps, tsan | Resolved in substance (G5-G9 only, builds under 16G, tsan's invocation and `phase-6-1-gate.md:1266`). Mechanism residue, R6. | :382-389 |
| m6 header derivation at the close | Resolved. | :396-397 |
| m7 INTERPRET row's disposition | Resolved. | :361-363 |
| m8 `enhanced` | Resolved. | :131-132, :232-233 |
| m9 semicolon and `--` bans | Resolved (only "No em-dashes" remains). | :67 |

### Open findings

#### R1. Task 3's seed list swaps two 9a-gate citations (Major)

* :216-219, `OBJECTNAME=`: "`phase-9a-gate.md:291` has the history". `:291` is the `Object~copy` on Package row. The `OBJECTNAME=` row is `:292` ("on a String or a SmallInt it stays refused, Deviation 31").
* :220-221, `COPY`: "including the REXX package's copy (`phase-9a-gate.md:287`)". `:287` is the `StackFrame~executable` row. The REXX package's copy is `:291`.
* :214-215 (`:287` for `StackFrame~executable`) is right.

Fix: `:292` for `OBJECTNAME=`, `:291` for `COPY`.

#### R2. Task 4 names a file that does not exist (Major)

:246: "`src/drive.rs:3782`". `ls rust/crates/rexx-exec/src/drive.rs` fails; the site is `src/ir/drive.rs:3782`. Fix the path.

#### R3. Task 5 names a real file that is the wrong one (Major)

:275: "The texts pinned at `rust/crates/rexx-exec/src/directives.rs:321-355`". That file exists and holds directive handling; the pins are at `rust/crates/rexx-exec/src/run/tests/directives.rs:321-355`. The grep beside it (`grep -rn 'named semaphore\|forked child' rust/crates`) finds the right file, which is why this is not blocking, but an implementer who reads the cited lines first reads unrelated code. Fix the path.

#### R4. The per-task bound no longer holds (Minor)

:76 "at most 5 minutes in all" now contains the whole suite at 4m38s (:82) plus fmt, clippy, the touched binaries, the refusal tables and the corpus gate. Either raise the per-task bound (the foreground bound at :75 is 10 minutes) or name the suite as a second exception beside the perf check.

#### R5. The repository-copy yaml run has no mechanism (Minor)

:264-265: "Run the yaml group once more with `extensions/yaml/yaml.cls` in place of the oracle's copy." `lay_class_files` (`suite.rs:342-353`) copies every `.cls` from `oracle_root()/bin` on every run and takes no override. Say how: a by-hand run per Task 1 Step 2 with the repository's file laid in the crate side's `bin` directory, or drop the step and keep Review Focus 2 to the harness's copy, which is what spec 2.2 :58-60 requires.

#### R6. "The gates of `bggates.sh` that `run.sh` does not already cover" cannot be invoked as such (Minor)

:382-383 and :388. `bggates.sh` takes one argument, a sha, and runs G1-G9 over an archived tree; it has no switch for G5-G9. "Both gate scripts' status files" presumes it ran. Say that `run.sh` carries the G5-G9 commands copied from `bggates.sh` (its G5, G6, G7, G8 and G9 lines) under its own status file, and that only `tsan.sh`'s output is copied in beside it.

#### R7. Task 3 Step 2 re-measures `method-bodies.txt` and misses the class arm (Minor)

:228-231: "For every method in `class-methods.txt`, send it once to an instance of its class with no arguments." That is `method_bodies.rs` (D76), whose committed output `rust/corpus/method-bodies.txt` already holds that send per row and shows no `native_method` hit. `class-methods.txt` has class-arm rows (`awk -F'\t' '$3=="class"'` prints 134), which an instance send does not reach. Say: cite `method-bodies.txt` as the no-argument measurement, send per arm, and probe only the rows its header says it cannot see (classes whose zero-argument constructor raises).

#### R8. The first perf baseline is still ambiguous (Minor)

:95-96: "The 9c close's figures ... are the first baseline." `close/callgrind.txt` has two blocks: `startup` +0.0181 at `48fb59b51` and +0.0162 at `25197002a`; `churn` and `parse` exist only in the first block. Name the `25197002a` block for `startup`, `sayloop`, `interpret` and the `48fb59b51` block for `churn`, `parse`.

### New claims checked, true

* :82 "4m38s at the 9c close": `close/status.txt`, workspace exit 19:55:57Z, suite exit 20:00:35Z.
* :152 `suite.rs:574` and `:639` each build a `work_dir` for the run.
* :212-213 the grep prints five sites; `lib.rs:1132` is a comment without `(`.
* :224-225 `dispatch.rs:2472-2500` is `invocable` with its four maps and the refusal.
* :240-242 `phase_10_group` `:278-289`, `PASSING` `:1516`, `FAILING` `:1544`.
* :270-271 `named_semaphore` `:540`/`:548`, `fork_child` `:557`/`:565`.
* :272 `the_scanner_reads_each_owner_form` at `:783`, `named_semaphore` asserted at `:789-793`, `native_method` at `:785-788`.
* :315 `:642-650` is the `seen` map keyed by constructor.
* :341-351 every Task 9 line reference matches.
* :384-386 `tsan.sh` runs `cargo +nightly` itself; nightly is installed; `phase-6-1-gate.md:1266` gives the quoted invocation.

### Summary for Moritz

1. Both blocking findings are resolved: the refusal table is now seeded from the five actual `native_method` sites with their known receivers, and the last Task 7 instance splits each site by its fate so no constructor carries a false GUARD.
2. All seven major and nine minor findings are resolved in substance.
3. Three of the revision's new citations are wrong and would send an implementer to the wrong lines: two 9a-gate line numbers are swapped in Task 3, Task 4 names `src/drive.rs` (it is `src/ir/drive.rs`), and Task 5 names `src/directives.rs` where the pins are in `src/run/tests/directives.rs`.
4. Five minor residues: the 5-minute per-task bound now contains a 4m38s suite run, the repository-copy yaml run has no mechanism, `bggates.sh` cannot run G5-G9 alone, Task 3's probe duplicates `method-bodies.txt` and skips class methods, and the first perf baseline block is unnamed.
5. With R1-R3 fixed in text, ready to dispatch; R4-R8 can be fixed in the same edit.
