# Task 9 and Phase 8 close: final review

Reviewer: final8, 2026-09-28/29. HEAD `81bfd20c7`, worktree clean. Everything below was run, not
read, unless it says otherwise. `$S` = `scratchpad/review-final8`. Oracle runs use the standard
wrapper from a fresh `mktemp -d` per side, three descriptors compared separately (`$S/cmp.sh`).
Builds: `$S/target` (release `rexx-run` at HEAD, Compiling lines present), `$S/target-base`
(`git archive 2ae06085c` of `rust/ interpreter/ api/`, touched, Compiling lines present),
`$S/target-nc` (a `git archive HEAD` copy at `$S/nc`, used for every negative control and the
gate reruns), `$S/target-miri`.

## Verdicts

* **(A) Task 9: not approved as it stands.** The Directory work matches the oracle on every
  Directory and StringTable case probed, but it regresses `DirectoryPut` on a `Relation` (I2), and
  the closed-phase instrument it extended has a blind spot a Phase 8 owner walks straight through
  (I3). Two of its re-homes rest on reasons the oracle contradicts (I4, for the lead to rule).
* **(B) Phase 8 may not be called closed yet.** Row 8's "native-API ooTest groups pass" is not
  what the gate measures: `api_group_tests` asserts agreement with the oracle, and in the gate's
  environment `METHOD`'s `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` fails on **both** sides
  (I1). Section 10's sentence "the native-API ooTest groups this row owns pass, but for the one
  test row 10 owns" is false as written. Everything else in the close checks out: the status
  block is quoted byte for byte, my own reruns of the corpus gate and `api_group_tests` are green,
  Miri passes, the unsafe inventory holds, and no live document claims Phase 8 is open.

## Findings

### Important

**I1. `METHOD` does not pass; the gate measures agreement, and both sides fail one test.**
`docs/superpowers/plans/phase-8-gate.md` section 10, last paragraph ("the native-API ooTest groups
this row owns pass, but for the one test row 10 owns"); roadmap row 8
(`2026-07-27-rust-rewrite.md:544`); `rust/crates/rexx-exec/tests/api_group_tests.rs:360-460` never
asserts the oracle's outcome is `pass`.
Run: a copy laid out as `fresh_copy` does (`$S/oot`, `rxfuncquery` lines dropped), crate's
release `rexx-run testOORexx.rex -f ooRexx/API/oo/METHOD.testGroup -U -V 2 -S`: `Tests ran 320,
Assertions 1128, Failures 1, Errors 0`, rc 1; the failure is
`TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, `assertRc` expected 0 actual 127
(`METHOD.testGroup:3105`, `address "" "rexxc" ...`). The oracle on that one test, same
environment: `Failures 1`, `Actual: 127`, rc 1. `rexxc` is not on the gate's `PATH` (`which
rexxc` prints nothing; no `ooRexx/build/bin` entry), so both sides fail identically and the test
is invisible to `api_group_tests`. With the oracle's `build/bin` on `PATH` the oracle has a
`rexxc` and ours has none until row 9 ships one, so this is a second test the row does not pass,
owner Phase 9 by row 9's "`rexx`, `rexxc`, `rxqueue`, `rxsubcom` ship". CONVERSION: 97 tests, 0
failures, 0 errors. FUNCTION (ours only, rxapi rule): ends rc 120 at `TEST_REXXQUEUE`, as
recorded.
Fix: either record it like `TEST_REXXQUEUE` (with `rexxc` on the oracle's `PATH`, so the oracle
passes and ours is the recorded difference), and add an assertion that every non-recorded test's
oracle outcome is `pass`, which is what "pass" means; or narrow row 8 and section 10 to "agree
with the oracle".

**I2. `DirectoryPut` on a `Relation` regressed: the direct path replaces where the oracle adds.**
`rexx-api/src/callbacks.rs` `table_put` (the new `store_put` branch);
`rexx-exec/src/dispatch/library/surface.rs` `store_put` calls `hash::directory_put` (`insert`) for
every class `hash::owns` admits, which is all of `OWNED` (`hash.rs:103-112`: Table,
IdentityTable, Set, Relation, Bag, Directory, StringTable, Properties). The oracle's stub calls
the virtual `put`, which for a multi-valued collection adds.
Run: `$S/q/c_rel.rex` (`x = .relation~new`, `TestDirectoryPut(x,'p','P')`,
`TestDirectoryPut(x,'q','P')`, `say x~items`, ...): oracle `2 / q,p / q / q 1`, HEAD `1 / q / q /
q 0`. The same program on the base build (`2ae06085c`): **identical** (rc 0). So Task 9 turned an
agreeing case into a silent wrong answer. `Bag` (`c_bag.rex`) differs at HEAD too (oracle 2 items,
ours 1), and differed at base differently (93.949 from the `PUT` send). Remove is fine on both
(`c_rel_remove.rex` identical at HEAD and base). Nothing in the tree reaches this; it is an
extension passing a non-Directory to `DirectoryPut`, which the oracle handles through `put`'s
virtual dispatch.
Fix: have `store_put` use the collection's own put semantics (add for the multi-valued stores),
or restrict the direct path to the classes whose `put` is `DirectoryClass::put` /
`HashCollection::put`-replace, and add `c_rel.rex`'s lines to the witness.

**I3. `closed_phases` cannot see a Phase 8 owner written inside a longer literal, or in the
exclusions file.** `rust/crates/rexx-exec/tests/closed_phases.rs:78-95` matches only the exact
token `"Phase 8"` (quotes included). `rexx-api` builds refusals the other way too:
`rexx-api/src/ffi.rs:3468` `"RexxInstanceInterface.AttachThread is not implemented (Phase 9)"`
and `:3515` (`... from another thread is not implemented (Phase 9)`).
Controls, predictions written first (`$S/predictions-A3.md`), each in `$S/nc` and restored:
* NC-a, a row `OWNER: Phase 8` appended to `phase-4-exclusions.txt`: predicted nothing red.
  Result: `closed_phases`, `builtin_status`, `bif_assertions` all green; only the manual (A) grep
  prints it. As predicted.
* NC-b, `ffi.rs:3468` `(Phase 9)` -> `(Phase 8)`: predicted nothing red. Result: `closed_phases`
  green, every `rexx-api` test green, and section 10's (C) grep prints nothing (exit 1). As
  predicted.
* NC-c, `REFUSING_MEMBERS` gains `("RexxThreadInterface.Dummy", "Phase 8")`: red,
  `src/layout.rs:379: Phase 8`. As predicted.
* NC-d, `alarm_startTimer`'s deferred owner -> `"Phase 8"`:
  `every_deferred_entry_point_names_an_open_phase` red (`alarm_startTimer defers to "Phase 8",
  which is not one of ["Phase 6", "Phase 10"]`), and with `--no-fail-fast` `closed_phases` red
  on `src/dispatch/native.rs:116`. As predicted.
* NC-e, `class-set.txt` Alarm's method-owner 6 -> 8: report mode green; `REXX_CORPUS_GATE=1`
  `concept_and_class_gate_table` red, "gated by this run: 7 row(s)". As predicted.
* NC-f, `introspection-arity.tsv:196` `(Phase 9)` -> `(Phase 8)`:
  `the_table_matches_the_three_sides` red, `closed_phases` green. As predicted.
So the gate tables, the TSV, the registry and a bare owner literal are covered; a refusal text
literal naming a closed phase and an exclusions owner row are not. Fix: match
`\(Phase N\)` inside any literal (and `"Phase N"`), and turn the (A) exclusions grep into a test.
The same blind spot applies to Phase 7.

**I4. Two re-homes rest on reasons the oracle contradicts** (rulings 2a and 2c are binding; this
is for the lead).
* rcond POSITION -> Phase 9 (`phase-4-exclusions.txt:4727-4734`, "which entries a delivered
  condition object carries is the condition machinery's"). Run: `rcond.rex` reproduces (stdout
  differs by `  POSITION = 6`). But `$S/q/rcond_rexx.rex`, the same shape in Rexx (a method and a
  routine that `RAISE USER BAR` into a caller with `CALL ON USER BAR`), is **identical**, and both
  sides carry POSITION there. The condition machinery agrees; only the native `RaiseCondition`
  path adds a POSITION the oracle's native activation does not. That is the native API's
  surface, and row 9 says "No native-API ooTest group is this phase's".
* blocking member on an outer context -> Phase 6 (`phase-4-exclusions.txt:4757-4772`, "D3's
  frame-ownership question"). Run: `outer9.rex` reproduces. But `outer9` is one thread and one
  activity: `Outer9` stashes its own call context, sends `RUN` on the same thread, and
  `UseOuterVar` reads through it. D3 (`2026-07-27-rust-rewrite.md:170-196`) is about
  cross-activity access ("Cross-activity requests ... never through a foreign frame pointer").
  Nothing here is concurrent; row 6's exit criteria do not reach it.

**I5. The committed gate starts the oracle's rxapi.** `api_group_tests.rs`'s per-test loop runs
every test name on the oracle, `FUNCTION.TEST_REXXQUEUE` included (`RECORDED` only leaves it out of
the whole-group run), and that test does `.rexxqueue~create('ADDRESSWITH')`. `pgrep -a rxapi`:
pid 1892, `/home/moritz/dev/repos/ooRexx/build/bin/rxapi`, parent 1, started 2026-09-28 10:18:58,
before this review (my gate rerun reused it). This breaks the brief's "Never rxapi" oracle rule on
every gate run, and makes the oracle's side of that test depend on queue state persisted from the
previous run. I did not kill it. Fix: skip the recorded tests in the per-test loop too.

### Minor

**M1. The stash refusal's licence citation does not cover it.** `phase-4-exclusions.txt:5531-5539`
cites Moritz's 2026-09-01 GC-ordering ruling for "OWNER: none". That licence is over ordering
only: its SCOPE (5b plan `2026-08-27-phase-5b.md:646-655`, the SCOPE sentence at :654, and the licence's own record) allows no
exit-status or stderr difference. `stash.rex`: oracle `use stashed`, rc 0; ours rc 120 with
`rexx-exec: the handle is no longer held by this activation is not implemented`. And the
oracle's other ordering (a collection between the calls) is a read of a freed object, not a
correct answer, so there is no ordering whose outcome ours matches. Staying loud is right; the
reason should be the API contract (a local reference is invalid once its call returns) rather
than the licence. The text "... is not implemented" with no owner also reads as unbuilt work.

**M2. `ReleaseLocalReference` is now a no-op for the whole call.** Forged `RelLoop(2000000)`
(`$S/rel/relloop.cpp`, NEEDs nothing, no undefined Rexx symbol) creating and releasing a string
each pass: oracle maxrss 20104 KB; HEAD 430068 KB; base 348032 KB. At 8M passes under the 1 GB
cap, HEAD aborts rc 134. Most of the growth predates Task 9 (no collection inside a native call);
Task 9 adds about 40 bytes per released handle. Arguably inside the OOM licence; worth a sentence.

**M3. The witness has no `StringTableRemove`.** `library_directory_members.rex` declares no
`TestStringTableRemove`. `$S/q/q1.rex` (subclass with REMOVE overridden, plain hit, miss raising
88.900) is identical, so the behaviour is right, but a mutant of that path would pass the corpus.

**M4. The closing gate run compiled nothing.** `surface-9/gates/g3-build-release.txt` has 0
`Compiling` lines, and G4/G6 0, so the binaries came from an earlier build of the shared
`rust/target`. `ef62d6800..fc32f74aa` changes only a runtime-read `.txt`, so this is benign if the
previous build was `ef62d6800`'s, but section 10 does not say so. My rerun in a fresh target at
HEAD (Compiling lines present): `api_group_partition` 22 passed, `api_group_tests` 21 passed
(`every_test_of_the_phase_8_groups_matches_the_oracle_but_the_recorded ... ok`), corpus
`653 of 653 matching`, exit 0, with `REXX_CORPUS_GATE=1` (`$S/gate-api-corpus.txt`).

**M5. "`testbinaries/` compile unchanged" is witnessed by a 2026-08-05 build, not a build.** The
spec's instrument (`2026-09-14-phase-8-native-api.md:306`) is "a build of `testbinaries/` against
the frozen headers". Rerun: both `diff -rq` commands of section 9 exit 0; every CMake target's
product exists; and `g++ -fsyntax-only -Iapi -Iapi/platform/unix` on every `testbinaries/*.cpp`
succeeds, `orxclassic1.c` as C with `gcc`. So the claim holds today, by my run, not the tree's.

**M6. `LIBRARY rexxutil` -> Phase 10 is a weak owner fit** (ruling 4 binds). D11 is about the
`Sys*` functions, which answer here (`syssleep.rex` identical); what is missing is binding the
internal package name, an EXTERNAL-directive mechanism, which row 521 gave Phase 8.

Not findings: `DirectoryAt` on an `IdentityTable` finds an equal string the oracle misses
(`c_itable.rex`), which DEVIATION 4 (object identity not modelled) already licenses, and pure
Rexx shows the same (`it_pure.rex`). Pointer and Buffer instances, which the 5i plan handed to
Phase 8, now arrive from `orxmethod` and agree (`ptr.rex` identical).

## A. Task 9

### A1 Directory/StringTable native members

Run at HEAD, each identical on all three descriptors unless stated: the committed probes
(`task-9-probes/*.rex`: dirapi, ov, stale, constlib, constlib2, reqlib, sinunit, syssleep, extmeth2,
x431a-d identical; extmeth, extrout, mnew3, outer9, stash differ as recorded). Mine, in `$S/q/`:
q1 StringTable remove (subclass, hit, miss); q2 index case (Directory both cases, StringTable,
setMethod lower-case name, upper-case miss raising 88.900); q3 `.nil` value through put/at/remove
on Directory and StringTable, a method and an UNKNOWN answering `.nil`; q4 put over a setMethod
entry on a subclass, a subclass `unknown` method bypassed; q5 remove of a method-backed index, one
shadowing a contents entry, UNKNOWN on a missing index, removing twice; c_table, c_set, c_props,
c_cross (Directory members on a StringTable and the reverse, with methods and UNKNOWN), c_env
(`.environment`, `.local`, which take the send path): all identical. c_rel and c_bag differ (I2);
c_itable differs under DEVIATION 4. Non-hash receivers (Array, String) were not run: the oracle's
stub casts and calls a virtual on an object without that vtable slot.

### A2 ReleaseLocalReference

`stale.rex` identical rc 0; `stash.rex` rc 0 vs rc 120 as recorded. Licence: M1. Growth: M2.

### A3 closed_phases / OPEN / CLOSED_PHASES controls

I3. Predictions in `$S/predictions-A3.md`, written before the first control ran.

### A4 re-homed rows

| row | probe reproduces | reason true | owner fits |
|---|---|---|---|
| rcond POSITION -> 9 | yes | no (I4) | no: native-only path |
| blocking member -> 6 | yes | no (I4) | no: single thread |
| LIBRARY rexxutil -> 10 | yes (extrout, extmeth differ; extmeth2, syssleep identical) | yes | weak (M6) |
| Task 8 residuals -> 9 | yes (g3, if1, l6, pf1, ri1 each differ; all pure Rexx) | yes | yes |
| TEST_REXXQUEUE -> 10 | ours only (rxapi rule): `rexx_create_queue ... (Phase 10)`, rc 120 | yes | yes |
| Method~new 3rd arg -> 9 | yes (mnew3) | yes | yes |

### A5 Miri

Rerun: `RUSTUP_HOME=scratchpad/surface-4/rustup-home CARGO_TARGET_DIR=$S/target-miri cargo
+nightly miri test -p rexx-api --lib --offline` in `$S/nc` (HEAD), `MIRIFLAGS` unset (Stacked
Borrows): 55 passed, 0 failed, 8 ignored, exit 0, `a_released_local_reference_still_answers ... ok`
(`$S/miri.txt`).

## B. Phase close

### B6 row 8 exit criterion, gate reruns

Literal clauses: (1) "`testbinaries/` compile unchanged against frozen headers": true (M5); the
committed instrument is a pair of diffs plus an old build, not a compile. (2) "native-API ooTest
groups pass": not met as measured (I1). The partition to three groups is derived by
`api_group_partition` (22 passed in my rerun). Section 10's status block: `diff` of the quoted
block against `surface-9/gates/status.txt` prints nothing. G4 2753/0/4 and G6 2754/0/4 re-summed
from the logs agree; 653 of 653, the report-mode figures and "gated by this run: 0 row(s)" are
in both logs. Reruns: M4.

### B7 Phase 8 open/owner grep

Command (collapsed per file, since the records are hard-wrapped):

    for f in $(git ls-files docs rust | /bin/grep -a -v '^docs/superpowers/records/' \
                 | /bin/grep -a -E '\.(md|txt|rs|toml|tsv|rex)$'); do
      tr '\n' ' ' < "$f" | tr -s ' ' | /bin/grep -a -oiE '[^.]*phase[- ]?8[^.]*' \
        | /bin/grep -a -iE "open|pending|owe[sd]?\b|owner|owns|unbuilt|not (yet )?(closed|built|met)|still|remain|until|is phase 8's|phase 8's work|\(phase 8\)" \
        | sed "s|^|$f: |"; done

51 sentences (`$S/b7.txt`), each read. None claims Phase 8 is open or assigns it work: they are
history in dated plans and specs, the "Phase 8" owners of the 5i and Phase 7 documents (Pointer
and Buffer, which now agree, `ptr.rex`; native libraries, which load, e.g. `rxunixsys`,
`unixsys.rex` identical), the L2 record, and the close's own text. `git grep -i 'phase 8' --
rust/corpus` outside `oracle-crashes.txt` finds only `phase-8.txt`'s header.

### B8 unsafe inventory

    git grep -n -E '\bunsafe[[:space:]]*(\{|fn\b|impl\b|trait\b|extern[[:space:]]*"[^"]*"[[:space:]]*fn[[:space:]]+[a-z_])' \
      -- rust/crates | /bin/grep -a -v -E '^[^:]+:[0-9]+:[[:space:]]*(//|\*|/\*)' | cut -d: -f1 | sort | uniq -c

prints only `rexx-api/src/ffi.rs` (495), `rexx-api/src/load.rs` (46), `rexx-core/src/bytes.rs`
(1), `rexx-core/src/frame.rs` (7). The wider `-w unsafe` grep adds `rexx-api/src/layout.rs` and
`rexx-api/tests/layout.rs`, which hold `unsafe extern fn` pointer types and strings, no unsafe
operation. `#[allow(unsafe_code)]` sits on `ffi.rs`, `load.rs`, `frame.rs` and `lib.rs:23`
(`mod bytes`). `$S/safety.py` (each `unsafe {` needs a `// SAFETY:` in the comment run above it,
over at most three lines of the same statement): ffi.rs 290 blocks, load.rs 37, bytes.rs 1,
frame.rs 7, **0 without**; inverted on a copy of bytes.rs with its SAFETY renamed, it flags that
block.
