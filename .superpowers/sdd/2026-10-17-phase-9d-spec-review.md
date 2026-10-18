# Review: Phase 9d spec (2026-10-17)

Spec: `docs/superpowers/specs/2026-10-17-phase-9d-divergences-and-close-design.md` at `e81ed234b`.
Parent: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md`, sections 2, 3, 7, 8, 9, 10.
Every claim below was checked against the tree at `e81ed234b` by grep or by reading the file named.
Line numbers are `file:line`.

## Summary for Moritz

9d closes Phase 9.
It takes the 178 ooTest tests that still fail on the crate and pass on the oracle (38 groups) and makes each one pass or carry a Deviation row you have ruled.
It moves json, yaml and dateparser off the Phase 10 list and makes their five groups green.
It removes every refusal that names Phase 9, relabels SysFork and the named semaphores to Deviations 42 and 43, deletes `Owner::Phase9`, and makes `closed_phases` police Phase 9.
It then runs the close: the whole suite green, G1-G9, TSan, and callgrind within +0.5% of the pin.
It does not do PPWizard or the Rexx Parser (a handover note to Phase 10), `RxFuncAdd`/`RxFuncQuery`/`RxFuncDrop`, rxapi, the C extensions, the embedding API, Windows groups, performance work or language extensions.
Against the tree: json and dateparser already pass every test on the crate at the 9c close, yaml passes 48 of 50 and the other two are blocked by compiler/driver refusals already queued with owner 9d.
Of the 9d set's 193 rows, 15 already pass and 178 fail.
The spec's weakest parts are where it states a mechanism the harness rejects, hands a Moritz-level decision to the controller, or cites a measurement no record holds.

## Verdict

**Not ready.**
Five places would send a subagent into work the harness rejects, work on the wrong file, or a decision only Moritz can make.

## Rulings check

| Ruling | Stated faithfully | Where it is bent |
|---|---|---|
| 1. Done = all but rxapi and some C interfaces, deviations documented | Yes, :10-12 | B3 and B4: generic refusals and queued divergences stay undocumented at the close |
| 2. Larger corpus to Phase 10 | Yes, :12-15, :68-73 | B5: the handover cites measurements that have no record |
| 3. json, yaml, dateparser in 9d, C extensions in 10 | Yes, :34-41 | B2: the mechanism and the real work are not the ones stated |
| 4. RxFunc* to Phase 10 with the daemon, no in-process version | Yes, :74-76, matches `rexx-inventory/src/lib.rs:28-35` | none |
| 5. SysFork Deviation 42, named semaphores Deviation 43, both recorded | Yes, :47-48, :83; entries at `phase-4-exclusions.txt:2313` and `:2327` | M4: the relabel's mechanical obligations are missing |
| 6. A new Deviation needs Moritz's ruling first | Yes, :113-114 | B3: "(Controller's call)" at :56 and :116 decides what stays unimplemented |

## Blocking issues

### B1. The `scclient`/`sfclient` re-home names a row the harness rejects, on a cause nobody measured

Spec :30-33: "`samples/scclient` and `samples/sfclient` need the `rxsock` C library (`extensions/rxsock/socket.cls:40`). They leave the set as `Phase 10 Extension` rows in `phase-9-exclusions.txt`. Parent criterion 2 allows Phase 10 rows there."

Evidence.
* A `group` row's owner check is `phase_10_group` (`tests/support/suite_rows.rs:278-289`): `extensions/` prefix, `ROW_10_GROUPS`, `R1_GROUPS`, else `Err("the roadmap does not make the group Phase 10's")` (`:521-527`).
  `samples/scclient` matches none, so the row the spec asks for reddens `the_row_files_hold_only_checked_rows` (`tests/suite.rs:1111-1135`).
* A `refusal:` row needs a captured loud refusal naming Phase 10 with a D3 reason.
  The 9c close has none: `close/suite/tests.tsv:24520` and `:24524` read `failed`, not `blocked: ...`.
* The recorded cause is different and already fixed.
  `phase-9a-gate.md:426-428`: the tests fail because `rexx-run` did not resolve `scclient` to `scclient.rex`, "which is 9c's front end".
  9c delivered that front end, and the tests still fail at the 9c close, so the present cause is unmeasured.
* "Need the rxsock C library" is not a cause on its own.
  `phase-9a-gate.md:421-424`: the crate side's child `rexx` inherits `LD_LIBRARY_PATH`, "so extension libraries load there: the crate passes `extensions/rxregexp/rxregexp` and `extensions/rxmath/RxMath` whole in this run".
  `socket.cls:40` is `::requires "rxsock" LIBRARY`, and nothing in the tree shows the crate failing to load it.
* Anti-goal :95-96 forbids exactly this: "A test moves to a Phase 10 row only when it reaches rxapi, a C library or the embedding API, and the row names which."
  No row shape names a library, and no measurement shows the reach.

Fix.
* Add a measurement task before any row: run both groups under the harness at HEAD (`REXX_SUITE_GROUPS`, alone, per `suite.rs`'s `ALONE`), record the failing assertion and the child `rexx`'s three descriptors.
* Then one of: fix it in 9d, or, if the failure is the `rxsock` load, make the `LIBRARY` load of a library the crate does not carry refuse loudly with `Phase 10 Extension`, so the harness derives a `refusal:` row, and say so in the spec.
  If instead the group itself is to be Phase 10's, the spec must name the code change (`phase_10_group` gains a named group list) and the roadmap row 10 edit that makes the check's error message true.
* Either way this needs Moritz's word: parent section 2 made these two the Environment candidates that "need Moritz's ruling, recorded in the file", and this spec re-homes them without one.

### B2. Goal 2 states the wrong mechanism and omits the actual work

Spec :34-41: "`phase-9-exclusions.txt` holds as `Phase 10 Extension` today. Their rows leave that file. ... These packages are Rexx source, so they load through `::REQUIRES` like any program. A test that reaches a C library or rxapi from inside them keeps a Phase 10 row naming that library."

Evidence.
* The rows are derived, not written: `derived_rows` (`suite_rows.rs:633-656`) emits a `group` row for every test of every `extensions/` group, and `the_row_files_hold_only_checked_rows` asserts each is present.
  Deleting the 453 rows reddens that test until `phase_10_group` (`suite_rows.rs:279`) stops answering `Extension` for those three directories.
  The fixture `"extensions/json/json\tTEST_INVALID_EMPTY\tPhase 10 Extension\tgroup"` under `PASSING` (`tests/suite.rs:1516`) reddens too.
* The groups already run on the crate side, and the result is known.
  `close/suite/tests.tsv` at the 9c close: `json` 33 of 33 passed, `json_02` 227 of 227, `DateFormatter` 31 of 31, `DateParser` 112 of 112, `yaml` 48 of 50.
  The two are `TEST35_DIRECTIVES`, `blocked: rexx-exec: a compiled a pass boundary with no loop open op has no driver arm`, and `TEST47_MULTILINEFLOWCOLLECTIONS`, `blocked: rexx-exec: a compiled Store op has no driver arm`.
  `phase-9a-gate.md:488-490` recorded the same two at the 9a close.
  `.superpowers/sdd/queued/2026-10-13-yaml-pass-boundary-guard.md:9` says "Owner: Phase 9d (engine divergence)".
  The spec names none of this.
* Those refusals have GUARD rows that are now false.
  `rust/corpus/refusal-dispositions.tsv:52` `op_not_driven GUARD - driver invariant: every op the compiler emits has a driver arm`.
  A GUARD row means "no program reaches it" (`:12-13`).
  `yaml.cls` reaches it, and `refusal_dispositions.rs` checks that rows exist, not that they are true, so nothing goes red.
* Which `yaml.cls` is the subject is ambiguous.
  The crate side resolves `::requires 'yaml.cls'` through `PATH` to `bin-rexx/yaml.cls`, a copy of `/home/moritz/dev/repos/ooRexx/build/bin/yaml.cls` laid by `lay_class_files` (`tests/suite.rs:342-353`), which is 5093 lines.
  This repo's `extensions/yaml/yaml.cls` is 5176 lines, 248 diff lines apart, and its `yaml.testGroup` has 54 tests where the ooTest copy has 50.
  A subagent reproducing against the repo's copy may not reach the same op.
* Dependencies are as the spec says: all three packages are pure Rexx (no `LIBRARY`, `EXTERNAL`, `RxFuncAdd` or RexxUtil call in `json.cls`, `yaml.cls`, `dateparser.cls`), and no test in the five groups reaches a C library, rxapi or `.RexxQueue`.
  Two tests (`json_02` `test_xml_xslt_roundtrip`, `yaml` `test37_XsltRoundtrip`) run `xsltproc` and `rexx runXSLT.rxj` through `ADDRESS SYSTEM` and self-skip when neither answers.

Fix, replacing :34-41.
* "`phase_10_group` (`tests/support/suite_rows.rs`) names the C-extension directories (`extensions/hostemu`, `extensions/platform`, `extensions/rxmath`, `extensions/rxregexp`, `extensions/rxsock`) in place of the `extensions/` prefix, the derived rows for `extensions/json`, `extensions/yaml` and `extensions/dateparser` leave `phase-9-exclusions.txt`, and the fixture at `tests/suite.rs:1516` moves to a C-extension group."
* "The work is `yaml`'s `TEST35_DIRECTIVES` and `TEST47_MULTILINEFLOWCOLLECTIONS`, blocked by the refusals quoted in `phase-9a-gate.md`. The subject is the `yaml.cls` the harness lays beside the crate's `rexx`, `/home/moritz/dev/repos/ooRexx/build/bin/yaml.cls`. The fix closes `queued/2026-10-13-yaml-pass-boundary-guard.md` and re-states the `refusal-dispositions.tsv` rows of every constructor those two tests reached, with the yaml probe named."
* Keep the pure-Rexx statement, cite the grep that showed it, and drop "a test that reaches a C library or rxapi from inside them", which describes nothing in the five groups.

### B3. The `Loud::native_method` sites and the generic fallthrough are left to "the controller's call", against the 9a record and ruling 1

Spec :49-56.

Evidence.
* The callers are as the spec lists them, but two are not "fixed subject".
  `dispatch/hash.rs:214` passes `method: &[u8]` with the receiver's class id, and `dispatch.rs:2500` passes `name` with `scope_id(resolution.scope)`.
  Both are reached by any method no table holds, for a class of receivers.
* 9a already measured two of these sites as reached and owed them to 9d.
  `phase-9a-gate.md:287`: "a library routine frame's [`~executable`] stays refused, 9d".
  `:291`: "the REXX package's own copy stays refused, 9d".
  `:474-476` carry "`instanceMethods`' repeated shadowed entries (9d)" and "the REXX package's own copy; a library routine frame's `~executable` (9d)".
  The spec's "either implemented as the oracle answers, where a probe reaches it, or becomes a refusal naming no phase" lets a subagent write a GUARD row for a site 9a showed is reached.
* `OBJECTNAME=` at `object_protocol.rs:487` refuses on class "Object".
  `phase-9a-gate.md:292` says `OBJECTNAME=` "on a String or a SmallInt stays refused, Deviation 31", and `refusal-dispositions.tsv:51` carries `object_name_of_a_value DEVIATION - Deviation 36`.
  So that site's disposition may already be a Deviation, not an implement-or-GUARD choice, and the spec does not say which receivers reach it after 9b1.
* The generic fallthrough's acceptance test, "no ooTest test reaches it at the close", is unsound as written.
  A test in a group with Phase 10 rows can be blocked by a refusal without reddening anything (`judge`, `tests/suite.rs:887-947`, checks rowed tests only for `refusal:` and `NotRun` rows).
  The yaml case in B2 is the proof: two loud refusals sat hidden under `Phase 10 Extension` rows across the 9a, 9b1 and 9c closes.
* Ruling 1 says "everything implemented except rxapi and some C interfaces, with deviations documented".
  A refusal path reachable by any program for any unimplemented native method, carrying a GUARD row that cites a suite run, is neither implemented nor a documented Deviation.
  Ruling 6 reserves that decision for Moritz, and the spec gives it to the controller twice (:56, :116).

Fix.
* Enumerate statically what reaches `invocable`'s refusal and `refusal_outside`: the crate's native method tables against the oracle's method list per class (`rust/corpus/method-bodies.txt` and the `.methods` lists already exist for this).
  The spec states the enumeration command, and each pair is then implemented, a Deviation Moritz rules, or a Phase 10 row with a D3 reason.
* For the four sites, write what 9a recorded: the library routine frame's `~executable` and the REXX package's copy are implemented in 9d, `OBJECTNAME=` names its Deviation, `hash.rs`'s receivers are enumerated with the rest.
* Replace "no ooTest test reaches it at the close" with "no `blocked:` value in the close run's `tests.tsv`, over every row including rowed ones, names a `method ... of class ...` refusal".
* Ask Moritz: may a residual GUARD refusal for unimplemented native methods survive the Phase 9 close, or does ruling 1 require every reachable pair to be implemented or a Deviation?

### B4. Queued and carried divergences have no disposition at the Phase 9 close

The spec never mentions `.superpowers/sdd/queued/` or any gate's carried list.

Evidence.
* `.superpowers/sdd/queued/` holds 151 `.md` files.
  None is named in the 9d spec.
  Many are resolved in place, and many are open with "owner: not assigned", "Owner: none yet", or no owner line.
* `phase-9c-gate.md:137-147` lists eight items "Carried, filed during 9c", among them `2026-10-17-routine-call-frame-name.md` ("A known-gap row sits after Deviation 41"), `2026-10-16-stream-command-open-missing-file.md` and `2026-10-17-newmethod-parent-package.md`.
* `phase-9a-gate.md:452-490` carries thirty items to "9b-9d", three tagged 9d (`:132`, `:137`, `:172`), one of them "TEST_SLEEP_DURATION, Flaky candidate: the S2 list in `concurrency_tests` has no licence path (9d)".
  `phase-9b1-gate.md` has a "Carried forward" section.
* Parent section 3 names five queued traceback items as 9d's cluster.
  This spec says "its clusters stand as written there", which reaches those five by reference and none of the others.
* Ruling 1's "deviations documented" has no mechanism here.
  A divergence no ooTest test reaches is in neither the 9d set nor any row file, and the close is green with it open.

Fix.
* Add a close task: every open file in `queued/` and every item in the 9a, 9b, 9b1 and 9c carried lists gets one of: fixed in 9d, a numbered Deviation Moritz rules, a Phase 10 owner with a D3 reason, or a "known gap, no owner" line Moritz accepts, and `phase-9-gate.md` lists the result.
* Ask Moritz whether "deviations documented" covers queued divergences that no test reaches, or only the row files.

### B5. The handover note is to be written from measurements that exist nowhere in the tree

Spec :14-15: "The real programs probed so far, PPWizard and the Rexx Parser, both stop at `RxFuncAdd` and load C libraries."
Spec :69: "The 9d close writes Phase 10's handover note with what was measured on 2026-10-17."
Spec :131: "Corpus sources are fetched by script and never committed | controller, following Moritz's ruling for the rx2cpp probe".

Evidence.
* No record under `docs/superpowers/records/`, `.superpowers/sdd/` or `docs/superpowers/plans/` is dated 2026-10-16 or 2026-10-17 for either program.
  `grep -rln -i 'ppwizard|rexx parser|josep'` over `docs`, `.superpowers`, `rust/corpus` finds no run, invocation or error line for the Rexx Parser anywhere.
* The only PPWizard measurement is `ppwizard-gate.md:83`, "We stop at the first `RxFuncAdd`, `ppwizard.rex:426`", dated 2026-08-12 (`:3`).
  `rust/CLAUDE.md` says any figure before 2026-08-20 was taken against the 5.0 oracle and may not reproduce.
* `rx2cpp` appears in no file but this spec.
* A subagent at the close has nothing to cite for "the Rexx Parser's standalone suite invocation and the blockers seen".

Fix.
* Before 9d starts, the controller commits a record of today's runs: the fetch commands, the invocations, the three descriptors at the stop, and the rx2cpp ruling in Moritz's words.
  The spec then cites that path.
* Or the spec drops "what was measured on 2026-10-17" and scopes the note to what the tree holds plus a re-run task in 9d with a fetch script.

## Major, non-blocking

### M1. The per-task +0.05% bound has no instrument and does not bound the sum

Spec :105-109 sets +0.05% per task on `startup`, `interpret` and `parse`.
Spec :110-112 lists each task's checks as fmt, clippy, test binaries and ooTest groups, "about 5 minutes", with callgrind "only at the 9d close".
Nothing measures the per-task figure, so the bound cannot fail.
The budget is cumulative: `churn` sits at +0.165% and `sayloop` at +0.145% of +0.5% (`close/callgrind.txt`, `phase-9c-gate.md:96`), and `startup`, `interpret`, `parse` are not the programs nearest the line.
`phase-9a-gate.md:467` already carried "cumulative per-task perf: all for the Phase 9 close callgrind".
Fix: delete the per-task bound, or require a three-program callgrind per task with a running sum against the headroom on every program, named in the checks list with its cost.

### M2. The 9d set has no partition, and the stopping rule blocks the close on rulings

The set is 178 failing tests in 38 groups (`close/suite/verdicts.tsv`, 38 red rows), and parent section 3's six clusters were drawn on the 249-test TSV before 9a.
Groups outside those clusters: `SecurityManager` 5, `VarRef` 5, `TRACE` 7, `TRACE_TraceObject` 5, `CLASS` 5, `IdentityTable` 6, `RAISE` 4, `Validate` 3, `Object` 3, `Class` 2, `Package` 2, and singletons.
"Its clusters stand as written there" is not a partition a planner can derive tasks from.
Constraint :115-116 sends each unfixed cluster to Moritz after three rounds, and goal 1 requires his ruling before a row exists, so the close waits on every escalation.
Fix: the re-derivation task emits the cluster table (group, tests, suspected cause, owner) as a committed file, and the spec says the close waits on rulings and that escalations go to Moritz as one batch.

### M3. The close's artifacts are not named

Parent criterion 1 puts the table and the oracle-difference report "in `phase-9-gate.md`", which does not exist.
Parent section 9 gives each sub-phase its own gate record, so `phase-9d-gate.md` is owed too.
Roadmap rows 6, 6.1, 7 and 8 (`2026-07-27-rust-rewrite.md:657-660`) carry "**CLOSED <date>**, `phase-N-gate.md`"; row 9 (`:661`) needs the same edit and the spec lists no roadmap edit.
Fix: name the two gate files, say whether 9d's close record and Phase 9's are one file, and add the row 9 edit to goal 4.

### M4. The Deviation 42/43 relabel and the `Owner::Phase9` deletion have unlisted mechanical obligations

* `refusal_dispositions.rs` requires a row for every constructor that can build an ownerless message (`refusal-dispositions.tsv:3-7`).
  `named_semaphore` (`lib.rs:540`) and `fork_child` (`lib.rs:557`) will need `DEVIATION` rows citing 43 and 42.
* Fixtures that quote the current texts: `src/run/tests/directives.rs:322-355`, `tests/concurrency_tests.rs:4984-5012`, `tests/suite.rs:1382` and `:1587`.
* Deleting the variant breaks `tests/internal_routines.rs:114`, `src/dispatch/native/tests.rs:159`, `tests/refusal_dispositions.rs:480-481`, and the string fixtures at `:843` and `:864`; `owner.rs:98-102`'s test goes too.
* The spec's "with the match arms that name them" covers only the first of these.

### M5. "The 9d set is empty" is not what any test checks

No Rust code reads `rust/corpus/suite-9d-set.txt`.
The 9c close's suite step exited 101 (`phase-9c-gate.md:22-23`) and the gate record accepted it in prose.
The checkable statement is: the close run's `l3_core_suite` exits 0, every group green, and the derivation command in the set file's header, run over the close `tests.tsv`, prints nothing.
Fix: state both, and say the close's suite step is exit 0 where 9a, 9b1 and 9c accepted 101.

### M6. The `names()` fix has no witness

`tests/support/literals.rs:164-171` stops only at a following digit, so "Phase 9" matches "Phase 9.1" and "Phase 9b1".
No literal under `src/` names "Phase 9.1" today, so the fix cannot be seen to work.
The parent's "adds `Phase 9.1` where a row may name it" is dropped without a sentence.
Fix: require the predicate change to come with a unit case beside `the_literal_scan_finds_a_phase_inside_a_longer_text` (`closed_phases.rs:305-320`, which already has the "Phase 80" shape).

### M7. Two Phase 10 decisions sit in 9d's Decided table, one on the controller's authority

Rows :130-131 decide PPWizard's patch policy and the corpus fetch rule.
Both are Phase 10's, the second is attributed to the controller "following Moritz's ruling for the rx2cpp probe", and that ruling is recorded nowhere.
Fix: move both into the handover note's content, and attribute the fetch rule to Moritz only if his words are quoted.

## Minor

* :49-50 calls `hash.rs`'s caller a "fixed subject"; its subject is the `method` parameter (`hash.rs:203-214`).
* :43 "The three `Owner::Phase9` sites" is a set size in prose (`rust/CLAUDE.md`, "A comment may not name the size of a set"); true today at `lib.rs:548`, `:565`, `:689`.
* :84 "under `restrictOS`": the reason form is `restrictOS line N` (`suite-not-run.txt:2-3`).
* The two XSLT tests (B2) depend on `xsltproc` or a BSF4ooRexx `rexx runXSLT.rxj`; on the crate side `rexx` resolves to the crate's binary through `bin-rexx` on `PATH`. They pass by self-skip today. The close record should say so.
* :5-6 "pushed at `71ab69cfe`" is right (`origin/plan/rust-rewrite` is `71ab69cfe`); the spec commit `e81ed234b` is ahead by one.
* :107 "`churn` at +0.165%" and :106 "sha256 beginning `c4b4e8c4`" are right (`close/callgrind.txt`, `phase-9a-gate.md:42`).
* :45-46 "Only the INTERPRET traceback row is open" is right: `:6390` open, `:5830` and `:6454` resolved.
* :58 "`closed_phases` scans `rexx-core`" is right (`closed_phases.rs:75-81`).

## Decomposition assessment

The spec gives no units, and the plan will have to derive them from section 7's order.
The derivable units and their state against the tree:

* Re-derive the 9d set: a docs edit with no code reader, cheap, but it is where the cluster table (M2) must be produced.
* Extension packages: one predicate change, one fixture, two yaml compiler/driver fixes, two or three disposition rows (B2). Small, and the spec's framing points away from it.
* The 9d clusters: 178 tests, unpartitioned, with per-cluster escalation to Moritz. This is the phase's bulk and the spec sizes none of it.
* Labels: delete `Owner::Phase9`, relabel two refusals to Deviations, re-disposition `native_method`'s callers (B3), update five fixture files and two test vocabularies (M4).
* Close: gate records, roadmap row, queued triage (B4), handover (B5), callgrind.

Two collisions.
* `refusal-dispositions.tsv`, `phase-4-exclusions.txt` and `suite-licences.txt` are shared state every cluster task writes.
  One writer at a time, or the plan merges rows at the end.
* The labels task deletes `Owner::Phase9` while cluster tasks may still be constructing refusals with it.
  Section 7's order (labels after clusters) is right and should be stated as a dependency, not a suggestion.
