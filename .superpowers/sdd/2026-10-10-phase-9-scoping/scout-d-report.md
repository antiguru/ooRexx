# Scout D report: refusals and queued items Phase 9 inherits

HEAD `a98238be5` (`git rev-parse --short HEAD`). `rexx-run` rebuilt at HEAD before probing:
`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -p rexx-exec --bin rexx-run`, exit 0, 1m19s.
Probes are under `/tmp/claude-1000/p9scope/d/` (`cmp.sh` runs one file on both sides from a fresh
directory: crate under `memcap 2G`, oracle under `ulimit -v 1048576`, `timeout 30` on both).
"ran" means I ran it on both sides at HEAD. "inferred" means I read it and did not run it.

## 1. Refusals naming Phase 9

Enumeration commands, all run:

* `grep -rn '"Phase 9"' rust/crates/*/src` gives 6 hits: five `layout::REFUSING_MEMBERS` rows
  (`rexx-api/src/layout.rs:379-383`) and one `Loud::native_method` (`rexx-exec/src/lib.rs:612`).
* `grep -n 'OWNER: Phase 9' docs/superpowers/plans/phase-4-exclusions.txt` gives 4 hits: lines 5284,
  5542, 6114, 6179.
* Two more places print the text "(Phase 9)" without a `"Phase 9"` literal, found by
  `grep -rn 'Phase 9' rust/crates/rexx-api/src`: `ffi.rs:4187` (`AttachThread` from a thread that is
  not the activity's) and `ffi.rs:4229` (`AddCommandEnvironment` from another thread). Both are
  assert-panics, not `Loud`, and not in `REFUSING_MEMBERS`. The spec should count them.
* `rust/corpus/refusal-sites.tsv` has no owner column. Its only `native_method` row is line 217. The
  roadmap row cites the file by the 6.1 convention only.
* `rexx-inventory`'s table (`builtins::EXCLUDED`) names only Phase 10 (RXQUEUE, RXFUNCADD,
  RXFUNCDROP, RXFUNCQUERY, QUEUED). No Phase 9 entry.
* `rust/corpus/refusal-dispositions.tsv` has no REHOME row:
  `awk -F'\t' '!/^#/ && NF>=4 {print $2}' ... | sort | uniq -c` gives 28 GUARD, 10 LIMIT, 5 DEVIATION.
  Nothing there re-homes to Phase 9.

### 1a. `Loud::native_method`: what it covers

`native_method` has seven construction sites (command:
`grep -rn 'Loud::native_method' rust/crates/rexx-exec/src`):

| Site | What it refuses | Probe, both sides | Oracle |
|---|---|---|---|
| `dispatch.rs:2409` (`invocable`, the fall-through for any resolved method with no native entry, body, generated, external or library binding) | a method with no code | `.rexxinfo~executable`, `.rexxinfo~libraryPath` ran: ours rc 120 `method "EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9)` | rc 0, prints `/home/moritz/dev/repos/ooRexx/build/bin/rexx` and `.../build/lib` |
| `dispatch/object_protocol.rs:621` | `Object~copy` on every receiver kind but Instance, Message, Array, Stem, String, SmallInt | ran: Method, Routine, Package copy ours rc 120, oracle rc 0. `.context~copy` ours rc 120, oracle rc 163 93.970. `.rexxinfo~copy` ours rc 120, oracle rc 163 | see left |
| `dispatch/object_protocol.rs:466` | `OBJECTNAME=` on a String, SmallInt, Object, Array, VariableReference or Stem receiver | `'abc'~objectName = 'x'` ran: ours rc 120, oracle rc 0 prints `x` | rc 0 |
| `dispatch/array.rs:700` | `MAKEARRAY` for an `~UNKNOWN` argument list that is not an Array (`unconverted_array_argument`) | `.environment~unknown('ARRAY','y')` ran: ours rc 120 `MAKEARRAY of class "String"`, oracle rc 0 `The Array class` | rc 0 |
| `dispatch/hash.rs:213` (`refusal_outside`, called by `not_this_task`, 30 call sites in `hash.rs` and `hash/relation.rs`) | a hash-collection method on a native receiver that has no store: the condition object from `Message~errorCondition` (a `Body::Native` Directory) and `.methods` (a StringTable made by `::METHOD` directives) | condition object `c~items` ran: ours rc 120 `ITEMS of class "Directory"`, oracle rc 0 prints `13` | rc 0 |
| `dispatch/class_protocol.rs:577` | `SUPPLIER` in `~defineMethods` on a value it cannot walk | inferred, not probed | |
| `dispatch/class_protocol.rs:774` | a metaclass with its own `NEW` class method | `::CLASS MyMeta SUBCLASS Class` + `::METHOD new CLASS` + `forward class (super)` ran: ours rc 120, oracle rc 0 prints `id k` | rc 0 |
| `dispatch/object_protocol.rs:1220` | `Message~notify` in an unnamed shape | `m~notify(.message~new(...))` ran: agrees, rc 0 both. Site not reached by that shape; inferred otherwise | |
| `dispatch/construct.rs:102` (`unbuilt_new`) | `NEW` on a class this crate cannot construct: the subclass-of-String/Stem/Method/Routine/Message route (section 3) | ran, section 3 | |

The set of methods with no native code at all is derived, not listed in prose. The committed
method-body table `rust/corpus/method-bodies.txt` sends every documented method with no arguments
to a real receiver. `awk -F'\t' '!/^#/ && $4=="loud" && $5 ~ /is not implemented/' rust/corpus/method-bodies.txt`
would list rows whose refusal text is the `native_method` text. What I ran:
`awk -F'\t' '!/^#/ && $4!="answers"' rust/corpus/method-bodies.txt` gives 37 non-answering rows. Of
them only `RexxInfo executable` and `RexxInfo libraryPath` (instance arm) name `native_method`. The
rest are 6 `uncomparable` (oracle segfaults), 5 `unstable`, 5 `unanswered` (Pointer), and `loud` rows
that are Phase 10's (14 `RexxQueue` rows naming `rexx_*_queue`) or Deviation 20 (`Message result`,
`Message wait`, `EventSemaphore wait`). So the "primitive method with no code" set is two rows. The
bulk of `native_method`'s real reach is the wrong-receiver and wrong-shape sites above. Caveat: the
table's receivers come from `class-set.txt`, so a class with no construction expression
(`unanswered` rows) is not covered.

Whole-group effect, from the recorded `whole_groups` table
(`python3 -I` over `rust/crates/rexx-exec/tests/concurrency_tests.rs`, 24 groups, 76 rows): six whole
groups end at a Phase 9 refusal:

| Group | Refuses at | Site |
|---|---|---|
| `base/class/Message` | TEST_REPLYWITH_NOT_ARRAY | `MAKEARRAY` (array.rs:700) |
| `base/class/Method` | TEST_NEW_FOUR_ARGS | `Method NEW with an INIT argument` (subclass NEW) |
| `base/class/Object` | TEST_RUN_ARRAY_ARGUMENT | `MAKEARRAY` |
| `base/class/RexxContext` | TESTCOPY01 | `RexxContext COPY` (oracle 93.970, ours rc 120) |
| `base/directives/METHOD` | TESTPACKAGE | `.methods~items` StringTable |
| `doc/rexxref/chapter5/Section1` | TEST_OBJECT_OBJECTNAMEEQUALS | `OBJECTNAME=` |

`CONSTANT` also refuses at TEST_EXPRESSION_ACTIVATE, `Routine NEW` (Phase 6.1 gate record line 575;
not in the table). The suite beyond those 24 groups has not been run whole against this crate by any
committed gate: `find ootest -name '*.testGroup' | wc -l` is 409.

Size of the native_method family: medium. `Object~copy` for Method, Routine, Package is small
(side tables are keyed by ObjRef). `OBJECTNAME=` on String/SmallInt needs an object-variable pool
on values that have none (a design question, medium). `MAKEARRAY` and `ITEMS` are small each. The
condition object becoming a real store Directory is medium (identity under re-raise,
`Activity::reraiseException`).

### 1b. Other Phase 9 rows

| Item | Probe, both sides | Oracle | Size |
|---|---|---|---|
| Conditions and stack frames under INTERPRET (`phase-4-exclusions.txt:6114`, OWNER Phase 9) | all five rows ran: g3 (untrapped `raise propagate` in a SYNTAX handler: ours adds an `interpret 'raise propagate'` line, oracle does not), l6 (traceback line indent: ours `*-*   interpret`, oracle `*-* interpret`), ri1 (`raise user foo return 7` in `interpret`: oracle runs the handler, ours skips it), if1 (`.routine~new(...)~call`: ours `.context~stackframes~items` 2, oracle 3; first name `CALL` vs `x`), pf1 (after SIGNAL ON SYNTAX trap of INTERPRET: ours names the PROGRAM frame by file, oracle `""`) | differs on all five | medium. Frame and traceback bookkeeping, 4 INTERPRET + 1 `Routine~call` level. pf1 may be a licence candidate. Queued items `2026-10-10-prologue-frames-missing-loading-levels`, `...propagated-condition-extra-traceback-line`, `2026-10-07-interpret-syntax-traceback-line`, `2026-10-03-interpret-translation-error-traceback` are the same family with no owner |
| Second interpreter instance (`phase-4-exclusions.txt:5284`, OWNER Phase 9: unloader and termination UNINIT run on the loader's thread context, one context per interpreter here) | not probeable without a forged extension, inferred from the row's own measurement (ours `same 1`/`use 42`, oracle `same 0`, and rc 139 on the oracle's own variant) | oracle rc 0 or 139 | large. There is no `RexxCreateInterpreter` or `RexxStart` in `rexx-api` at all: `grep -n 'RexxCreateInterpreter\|RexxStart' rust/crates/rexx-api/src/*.rs` finds nothing. `rexx-api` is the extension surface only. The embedding API, `Terminate`, `Halt`, `SetTrace`, `HaltThread`, `SetThreadTrace`, `AttachThread`, and the from-another-thread `AddCommandEnvironment` are all unbuilt. Owners also queued here: `2026-10-04-aborted-run-leaks-interp` ("the interpreter API needs the run to cancel or wait out in-flight calls and free the Interp"), `2026-10-05-two-interpreters-signal-race` ("for the interpreter API phase") |
| `rexxc` (`phase-4-exclusions.txt:5542`, OWNER Phase 9) | ran: oracle `rexxc a.rex a.rxo` rc 0 (1316 bytes). `.routine~newFile('a.rxo')~call`: oracle prints `hi`, ours rc 243 `Error 13.1 Incorrect character in program "&1" ('&2'X)` (a wrong message, and the "Phase 5" refusal the record describes is gone). `rexx-run a.rxo` the same | rc 0 `hi` | large. No `rexxc`, `rexx`, `rxqueue`, `rxsubcom` binary exists: `ls rust/crates/rexx-exec/src/bin` is `rexx-ir.rs rexx-run.rs`, and there is no `rexx-cli` crate though the crate tree names it. The `.rxo` format (`Envelope`, `ProgramMetaData`) is unread. The oracle's `build/bin` also has `rxapi` and `rexximage` |
| `ReleaseLocalReference` (`:6179`, OWNER Phase 9) | ran: forged routine from the queued file's text, `say RelLoop(2000000)`: ours `maxrss 429788 KB`, oracle `20276 KB`, both `done` | `removeLocalReference` drops from the save list | medium. Needs a collection inside a native call or a save-list removal |
| `REFUSING_MEMBERS` x5 | inferred, no probe: needs an embedder or a forged extension that reaches the instance table | | each small, but they sit behind the missing embedding API |
| `Phase 9` text on `AttachThread` and `AddCommandEnvironment` | inferred | | small each, same dependency |

## 2. Refusals naming Phase 10, 11 or none that a non-Phase-10 group reaches

* `grep -rn '"Phase 5"|"Phase 6"|"Phase 7"|"Phase 8"|"Phase 11"' rust/crates/*/src` each give 0 hits.
  `"Phase 10"` gives 61. So no refusal names 11 and no closed phase is still named.
* **17 non-extension, non-API groups use routines this crate refuses as Phase 10.** The refused set
  is `grep 'util("Sys' rust/crates/rexx-exec/src/internal_routines.rs` (37 names, all
  `Package::RexxUtil`). The groups: `/bin/grep -rlaiE --include='*.testGroup' "\b(<those names>)\b" ootest`
  gives 17: ADDRESS, File, Macrospace, SecurityManager, SysDumpVariables, SysFileDateTime,
  SysFileSearch, SysFileXXX, SysFormatMessage, SysGetMessage, SysIsFileDirectory, SysSearchPath,
  SysStemCopy, SysStemDelete, SysStemInsert, SysStemSort, tilde. Ran four of the routines on both
  sides: `SysStemSort`, `SysIsFileDirectory`, `SysSearchPath`, `SysTempFileName` each give ours rc 120
  `routine "..." is not implemented (Phase 10)` against oracle rc 0. Only Macrospace is RXAPI's
  (D7); the other 16 need no daemon. The roadmap's D11 says "subset in Phase 7, rest in Phase 10",
  and row 10 does not list RexxUtil. So Phase 9's "full suite minus enumerated groups" either
  excludes 16 groups that are core-conformance by name (File, ADDRESS, tilde among them), or pulls
  the remainder of RexxUtil forward. This is the finding most likely to change the scope.
* `phase-4-exclusions.txt` OWNER lines: 31 "OWNER: none" plus the Deviation 9-30 block
  (22 numbered paragraphs: `python3 -I` over the file, split on blank lines, filter `OWNER: none`).
  None names Phase 9.
* `LIBRARY rexxutil` as a package name (ours 98.903 rc 158, oracle works) is Phase 10 by the
  D11 line (`phase-4-exclusions.txt`, row beginning "`LIBRARY rexxutil` NAMES"); not probed here.

## 3. `NEW` on a subclass of String, Stem, Method, Routine, Message

* `git apply --check .superpowers/sdd/2026-10-07-phase-6-1/scout-d-prototype.patch`: **fails**, exit 1,
  on 3 of 8 files (`dispatch/class_protocol.rs:302`, `dispatch/construct.rs:168`,
  `environment.rs:1239`). `git apply --check --3way` reports conflicts in the same three and applies
  `body.rs`, `dispatch.rs`, `hash/stem.rs`, `string.rs`, `value.rs` cleanly. Commits touching the
  three files since the patch's base `fc04e3857`:
  `git log --oneline fc04e3857..HEAD -- <file> | wc -l` is 7, 2, 9. Working tree was clean after.
* Probes ran, ours vs oracle, five subclass shapes: String (`.K~new('abc')`), Stem, Method, Routine,
  Message all ours rc 120 `method "NEW" of class "K" is not implemented (Phase 9)`, oracle rc 0
  (`K abc | 3`, `K`, `K 3`, `K 1`, and Method rc 159 only because my probe called `~call`, which
  Method lacks).
* 6.1 scout D's own sizing (`scout-d-report.md`): String M (upper end), Stem M, Method+Routine M,
  Message S/M, total L (~600 lines with tests). The prototype is 176 inserted lines, no tests, no docs.
  What the prototype lacks, from that report: String `sign`, hash (`hashcode`, Table keys), the
  17+4+6+2 `return this` identity answers audit; Stem string value (`say x` answers `S.`),
  `request('STEM')`, `empty`, `copy`; Method/Routine `scope` and `annotations` for an instance,
  `addRoutine` accepting a descendant, the INIT send, `copy`; base-class `Method~copy` and
  `.method~new(n, s, context)`.
* Since that report landed in 6.1 (ran): `.VariableReference~new` now agrees (93.967 both), and
  `setMethod` from a class method on a class object agrees (`42` both). Still open: `Class~enhanced`
  skips `NEW` (ran `.array~enhanced(d, 3)`: ours rc 163 93.902, oracle rc 0 `0 Array`; queued
  `2026-10-10-class-enhanced-skips-new`, "one-line change once subclass NEW exists"). Method~new with
  a context argument agrees now.
* Also in the same family and open, queued: `2026-10-10-subclass-package` (97.1 vs the oracle
  answering), `2026-10-10-subclass-rexxinfo` (98.909 vs 99.949), `2026-09-30-stringtable-subclass-new-arg`,
  `2026-09-30-hash-subclass-init-capacity`. These are subclass-of-primitive gaps beside the five named.

## 4. Queued items

125 files in `.superpowers/sdd/queued/`, 112 without a RESOLVED/CLOSED line by
`grep -qi '^RESOLVED...'` (a loose test: briefs and spikes included). Items naming Phase 9 or the
interpreter API, and items bearing on an ooTest group:

| File | Owner it names | Blocks a group? |
|---|---|---|
| `2026-09-28-phase-5-refusal-labels` | "owner" = none; says Phase 9 relabel of `native_method` happened at Phase 8 | no |
| `2026-10-09-wrong-type-native-rows-name-phase-9` | RESOLVED by `ae86f4d04` | no |
| `2026-10-10-class-enhanced-skips-new` | Phase 9 row (subclass NEW) | not a recorded group |
| `2026-10-01-condition-object-directory-methods` | "owner not checked against any plan row" | yes, via the Message group's `errorCondition` tests (inferred) |
| `2026-09-29-release-local-reference-memory` | Phase 9 (exclusions `:6179`) | no (memory only) |
| `2026-10-04-aborted-run-leaks-interp`, `2026-10-05-two-interpreters-signal-race` | "the interpreter API (embedding)" | no |
| `2026-09-28-rexxqueue-in-function-group` | Phase 10 | FUNCTION TEST_REXXQUEUE |
| `2026-09-28-compiled-source-parse-errors`, `...condition-object-divergences-found-in-fix-round-a`, `2026-09-28-use-arg-message-term` | "not assigned" | METHOD/RAISE/USE ARG rows (inferred) |
| `2026-10-10-prologue-frames-missing-loading-levels`, `...propagated-condition-extra-traceback-line`, `...reply-propagated-condition-traceback-line`, `2026-10-03-interpret-translation-error-traceback`, `2026-10-07-interpret-syntax-traceback-line`, `2026-10-02-frame-executable-invocation-and-native-levels` | none | traceback and frame family, same as 1b |
| `2026-10-10-reply-continuation-trace-entry`, `2026-10-02-traceobject-variable-and-collector` | none | TRACE TEST_TRACE_LABEL_WITH_FORWARD; TraceObject collector tests (gate record lines 139, `Class` whole rows) |
| `2026-10-10-debug-*` (8 files), `2026-10-07-debug-*` (2) | none | interactive-debug rows |
| `2026-10-10-subclass-package`, `...subclass-rexxinfo`, `...variable-reference-concatenation`, `...native-routine-arguments-not-strings`, `...enhanced-object-name-in-messages`, `...string-translate-empty-tables`, `...compiled-method-package-in-trace`, `...security-manager-command-rc-requires`, `...misplaced-class-attribute-error`, `...leave-iterate-error-not-trappable`, `...call-on-nostring-insert`, `...beep-wrong-type-message`, `...translation-error-inserts` | none | silent or message-text divergences, a differential would meet them |
| `2026-10-10-upstream-do-to-identity`, `...upstream-zero-pass-do-reexecute` | upstream candidates, owner none | no |
| `2026-10-10-emptyloop-wall-clock`, `...rexxcps-task-6-round-2-delta`, `...list-remove-copies-per-call`, `2026-09-17..2026-09-22` perf files, `2026-10-04-send-site-cache-self-customization` | perf | bear on "every benchmark at parity" |

Task 12 report, "For Moritz": four open questions: emptyloop +4.6% cycles at -0.63% Ir (accept or
open an item), the two upstream candidates (file on SourceForge or not), and the oracle-judged
seeded gate (closed as not wanted by commit `a98238be5`). Also states `phase-4-exclusions.txt`'s row
"Method~new AND Routine~new REFUSE ANY THIRD ARGUMENT" is false at 6.1's HEAD and "the final review
should delete it". Check, run: `grep -n "REFUSE ANY THIRD" docs/superpowers/plans/phase-4-exclusions.txt` prints nothing, so it is deleted.
Gate record `phase-6-1-gate.md` hands forward:
Method whole at TEST_NEW_FOUR_ARGS (Phase 9), RexxContext whole at TESTCOPY01 (Phase 9), CONSTANT at
Routine NEW (Phase 9), METHOD whole at TESTPACKAGE (Phase 9), TRACE_LABEL_WITH_FORWARD and the
TraceObject collector (no phase).

## 5. Licensed divergences affecting the suite gate

* `rust/crates/rexx-exec/tests/licensed_divergences.rs` holds four executable rows:
  `driven-collection-reaches-a-new-object` (UNINIT order after `call gc 'force'`: oracle
  `start / after-gc / uninit ran`, ours `start / uninit ran / after-gc`), and
  `do-compare-computed-true`, `do-compare-array-one`, `do-compare-array-many` (DO TO/BY tests treat
  a computed `'1'` as true; the oracle compares to TheTrueObject by pointer).
  Command: `grep -n '^        name: "' rust/crates/rexx-exec/tests/licensed_divergences.rs`.
* `phase-4-exclusions.txt` numbered Deviations 9-30, OWNER none. The ones a suite test can meet
  (by subject; none was run against a group by me): 10 (SIGNAL halts where the oracle dies or hangs),
  13 (no other activity runs during a default-input read), 20 (a wait nothing can end is refused:
  `Message result`/`wait`, `EventSemaphore wait` are `loud` in `method-bodies.txt`), 22 (one
  schedule where the oracle's threads race), 25 (DO TO/BY truth), 28 (native method on another
  class's object refused; ooTest tests that deliberately run one primitive's method on another's
  receiver hit this), 29 (value-less entry-method assignment), 30 (a user `STRING` method answering an
  object with no string value). Deviation 21-24 concern termination UNINIT activities, kept contexts.
* Seeded-gate judging: ruling R1, the crate's own invariants with oracle differences reported (not
  gated), `oracle-is-a-build-tool` memory; the oracle-judged variant was closed as not wanted.
* Oracle crashes are not suite-reachable: `rust/corpus/oracle-crashes.txt` lists programs that kill
  the oracle, `uncomparable` rows (6 String operators, `Stem hasItem`, `Stem index`).
* Whole-group harness licences: `whole-groups/alone.txt` ("tests outside the derived list that fail
  alone too"), the clock=real exemptions (R6 in the 6.1 ledger), ticker output (`-U`, patched
  `ooTest.frm`: both `rxfuncquery` tests removed from the instrument's copy, per
  `phase-4-exclusions.txt:5542` row's trailing paragraph).

## Items not run or not derived

* `REFUSING_MEMBERS` and the two `ffi.rs` assert-panics: no embedding harness, inferred.
* The Phase 9 SUPPLIER site and the `Message~notify` site: not reached by any probe I tried.
* The full 409-group suite: not run by me; the 17-group count is a text match for names, so a group
  that mentions a name only in a comment is counted. Unverified per group.
