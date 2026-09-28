# Task 8 fix round A -- report

Implementer: fix-s8a. BASE `2b26b6970` (Task 8 BASE `391242b7e`). Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/fix-s8a/`
(below, `$F`; the reviewer's is `$R`). Written first, filled as the work goes.

## Status

Code committed at `7f220b983`; gates below.

## I1 -- condition('O') answers a copy

Done together with I5, because the representation is the same decision. A trapped condition's
object is built as BASE built it (a `Body::Native` `Directory`, entries written directly, no
`PUT` send); `CONDITION('O')` answers `condition_copy`, a new store-backed `Directory` holding the
same items (`BuiltinFunctions.cpp:2681`, `conditionobj->copy()`: shallow, items shared). A native
call's object (`build_native_condition_object`, handed to the extension as it is) stays
store-backed; `condition_copy` of a store-backed object sends `COPY`. `condition_entry` reads a
native object's map directly.

Found on the way and fixed with it: the store-backed object iterated in an order the oracle's does
not (`allIndexes`, unsorted), because the entries were put in this crate's build order. The copy
now puts them in `Activity::createExceptionObject`'s order for `SYNTAX` and
`Activity::createConditionObject`'s for the rest, `INSTRUCTION` last (`RexxActivation.cpp:2524`).
`$F/p/coorder.rex`, `coorder2.rex`: SYNTAX, ERROR, FAILURE, NOVALUE, a SYNTAX from a called
routine, all unsorted `allIndexes` byte-identical; at HEAD `2b26b6970` the SYNTAX and ERROR lines
differed.

Witness `rust/corpus/lang/condition_object_copy.rex` (STRICT, `phase-8.txt`, sourceline file):
`==` between copies, a write to one copy (`CODE`, `remove('MESSAGE')`, `ADDITIONAL` replaced, `RC`)
unseen by the next `CONDITION('O')` and `CONDITION('A')`, an item mutated in place seen by both
(shared), and the unsorted `allIndexes` of a SYNTAX and an ERROR condition. Oracle and fix
identical; HEAD's binary differs on stdout (`$F/p/w_copy.rex`, `cocopy.rex` run with
`RR=$F/rexx-run-HEAD`).

Controls (predictions written before running):

| # | Mutation | Prediction | Result |
|---|---|---|---|
| C1 | `state.rs` `'O'` arm answers the object itself, no copy | corpus red on `condition_object_copy.rex` and `condition_object_directory.rex` (the native object refuses the store-only methods); any other red is a program sending a store-only method to `condition('O')` | as predicted: 629 of 631, `condition_object_directory.rex` (`ITEMS`) and `condition_object_copy.rex` (`REMOVE`), both loud rc 120; restored, `cmp` clean |
| C3 | `state.rs` `'O'` arm: the first ask copies and installs the copy as the activation's object, and a later ask answers that store-backed object itself (HEAD's shape: one shared object) | corpus red on `condition_object_copy.rex` alone, silently (stdout, rc 0) | as predicted: 630 of 631, `condition_object_copy.rex`, stdout only (`1 1 1`, `changed 0 replaced`); restored, `cmp` clean |
| C2 | `key::order` always answers `ORDER` | corpus red on `condition_object_copy.rex` alone (its SYNTAX `allIndexes` line) | **falsified**: 593 of 631. `ORDER` has no `CODE`, `ERRORTEXT` or `MESSAGE`, so the copy, which puts only the keys the order names, dropped them from every SYNTAX copy (`condition_object_syntax.rex`: `code The NIL object`). I did not see that the order is also the copy's key set. Restored, `cmp` clean |
| C2b | `SYNTAX_ORDER` replaced by HEAD's build order (`CONDITION DESCRIPTION INSTRUCTION PACKAGE POSITION PROGRAM PROPAGATED STACKFRAMES TRACEBACK RC RESULT CODE ERRORTEXT MESSAGE ADDITIONAL`: same keys, HEAD's order) | corpus red on `condition_object_copy.rex` alone, stdout, its SYNTAX `allIndexes` line | as predicted: 630 of 631, `condition_object_copy.rex`, stdout, the SYNTAX line in HEAD's order; restored, `cmp` clean. No other corpus program saw the order |

## I3 -- the shared Routine object merges every directive's annotations

The oracle resolves a library `::ROUTINE` to the procedure's one object at parse time, and
`::ANNOTATE ROUTINE` writes into that object's own table (`DirectiveParser.cpp:2003-2014`,
`getAnnotations()` then `processAnnotation`), so the rule is **`::ANNOTATE` order**, not
`::ROUTINE` order: measured, `$F/p/gr5.rex` (routines A then B; annotate B `k 'b' j 'j2'`, then A
`k 'a' m 'm1'`, then B `m 'm2'`) answers `a j2 m2` on the oracle.

The install walk now also keeps every write to a `Directive` target in `::ANNOTATE` order, and
after the per-site tables are recorded replays the writes whose `::ROUTINE` is library-bound into
one table per procedure, `Annotated::LibraryRoutine(code)`, which `library_routine_object`
attaches. A later program's directives write into the same table, which the object already
carries. `$R/p/gr1`, `gr1b`, `gr1c`, `gr1d`, `gr2`, `gr3` and `$F/p/gr5`, `gr6` identical on all
three descriptors.

Also changed by it: a library `Routine` object no directive binds
(`.Routine~loadExternalRoutine`) now has an empty table, so `~annotation('k')` answers `.nil` as
the oracle's lazily created one does (`BaseExecutable::getAnnotations`, `:378-387`), where HEAD
refused loudly (the Task 8 report's pre-existing concern). `$F/p/gr7.rex`: oracle and fix `Routine
The NIL object`, HEAD rc 120.

Witnesses: `library_routine_shared_object.rex` extended (two more `::ANNOTATE`s, one naming the
earlier `::ROUTINE` after the later one's: `k` answers `v2`, `j` `j1`; HEAD answers `v1`), and
`library_routine_unbound_annotation.rex` (new, with the same `.env`). Sourceline files
regenerated for both.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| C4 | `install.rs`: `directive_writes` sorted by `::ROUTINE` directive (stable) before the replay | corpus red on `library_routine_shared_object.rex` alone, stdout: its last line `j2 j2 m1` for `j1 j1 m1`, `k` still `v2` | as predicted: 631 of 632, `library_routine_shared_object.rex`, stdout `j2 j2 m1`; restored, `cmp` clean |
| C5 | `identities.rs` `library_routine_object`: the table attached only where some directive binds the procedure | corpus red on `library_routine_unbound_annotation.rex` alone, loud rc 120 | as predicted: 631 of 632, loud `a message send to a value that carries no annotations`, rc 120; restored, `cmp` clean |

## I4 -- NewMethod/NewRoutine raise 36.901

`compile_method_source` and `compile_routine_source` (`dispatch/class_protocol.rs`), the one path
`Method~new`, `Routine~new`, `define` and the API's `NewMethod`/`NewRoutine` compile through, now
raise the parse error's own SYNTAX condition (`Raised::from(&ParseError)`, as `INTERPRET` does)
with `POSITION` the line within the source (`source_syntax`, `ProgramSource::line_of` over the
error's byte), where they refused loudly.

Measured first whether the SYNTAX reaches the tests through the native frame: with the change and
`RECORDED` unchanged, `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test
api_group_tests` went red with `newly failing: []`, `newly passing: ["METHOD.TESTNEWMETHOD02",
"METHOD.TESTNEWROUTINE02"]` (`$F/api1.log`). Both are removed from `RECORDED`, from the record
block in `phase-4-exclusions.txt`, and the queued file
`.superpowers/sdd/queued/2026-09-28-compiled-source-parse-errors.md` now holds only the residual
below.

Trapped probes, identical on three descriptors (`$F/p/np4.rex`, `np5.rex`, `rec5`/`rec6` for
`POSITION`): `.Routine~new` over `return arg(1` (36.901, POSITION 1), `.Method~new` over a
two-line array (36.901, POSITION 2), `.k~define` over `this is not rexx +++` (35.901) and over a
source with a line feed (13.1), a four-line routine source failing on line 3 (35.929, POSITION 3).
Witness `rust/corpus/lang/compiled_source_syntax.rex` (those five, trapped, printing code, RC,
POSITION, `condition('E')`, `CONDITION`); HEAD's binary refuses at rc 120.

What still differs, recorded in `phase-4-exclusions.txt` ("A SOURCE THAT DOES NOT PARSE, COMPILED
BY ...") with no owner assigned by this round (see Concerns): the message keeps its `&1`
placeholders (roadmap row 3's licence), and the compiled source's own frame is missing -- trapped,
`TRACEBACK` has the caller's clause alone where the oracle has `1 *-* return arg(1`, the `Compiled
method "NEW"` line and the caller's clause (`$F/p/np6.rex`); untrapped, the oracle reports
`Error 36 running Testing line 1` under that first line and this crate the caller's path and line
(`$F/p/np1.rex`, `np3.rex`). `ParseError` carries the clause's start byte and no end, so there is
no clause text to echo.

`run/tests/directives.rs`: the two `define` rows asserting the old refusal text are removed; the
witness covers both sources. `refusal-sites.tsv` re-derived
(`REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites --
--test-threads=1`); changes outside column 4: `Raised::from` moves from `body` to `body+send`
(its new construction site is under `dispatch/`) and `source_syntax` is a new `send` row; both
filled `agrees`/`yes` with the probe above, the answer being text the constructor's own source
holds (`syntax(error.code`, `line_of(error.byte)`), as the `condition` row does, because neither
builds a literal `M.N`.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| C6 | `source_syntax` leaves `position` at 0 | corpus red on `compiled_source_syntax.rex` alone, stdout: every POSITION reads the `interpret` clause's line, 11 (amended before running from 12, a miscount of the file's lines) | as predicted: 632 of 633, stdout, every POSITION 11; restored, `cmp` clean |
| C7 | both compile functions back to HEAD's loud refusal (`$F/ctl/class_protocol.rs.head`'s two `map_err`s) | corpus red on `compiled_source_syntax.rex` alone, loud rc 120; instrument red, `newly failing: ["METHOD.TESTNEWMETHOD02", "METHOD.TESTNEWROUTINE02"]`, `newly passing: []` | as predicted: 632 of 633 (loud rc 120 on the witness) and the instrument's `newly failing` exactly those two, `newly passing: []`; restored, `cmp` clean |

## I5 -- trapped-condition cost

A trapped condition's object is `Body::Native` again, its entries written with
`hash_entry_write` (no send); the store-backed `Directory` is built only when a program observes
it (`CONDITION('O')`, see I1) or when a native call asks for it, and then through
`dispatch::hash::store_insert` (the store's `insert`, no `PUT` send and none of `native_hash_put`'s
class-name lookups).

Callgrind `Collected`, the reviewer's program (`$F/cg/t.rex` = `$R/cg/t.rex`: 5000 iterations of
`call f`, `f` trapping `1 + 'a'`), each binary run the same way from a fresh directory by
`$F/cg/run.sh NAME BINARY` (`valgrind --tool=callgrind --callgrind-out-file=... BINARY t.rex`):

| binary | Collected | against BASE |
|---|---|---|
| BASE `391242b7e` (surface-8's `base-target/release/rexx-run`) | 627,571,736 | -- |
| HEAD `2b26b6970` (`$F/rexx-run-HEAD`) | 1,178,851,776 | +87.8% |
| fix (`$F/rexx-run-fix2`, the committed code; two comments were edited after the build) | 548,017,174 | **-12.7%** |

The observed case, the same loop with `o = condition('O')` in the handler (`$F/cg/t2.rex`,
`T=t2 $F/cg/run.sh`): BASE 632,379,408 (BASE answered the native object itself, no copy), HEAD
1,183,132,531, fix 835,320,496 (+32% against BASE, -29% against HEAD). That is the copy the oracle
also makes; before `store_insert` it cost 1,295,674,722 through `PUT` sends
(`$F/rexx-run-fixI1`).

Wall clock, 50000 iterations, interleaved, three rounds (`/usr/bin/time -f %e`): BASE 0.56 /
0.55 / 0.54 s, HEAD 1.04 / 1.03 / 1.04 s, fix 0.52 / 0.50 / 0.51 s.

## I7 -- per-test comparison of outcome and detail

`rust/crates/rexx-exec/tests/api_group_tests.rs`, committed at `7f220b983`:

* Both sides run at `-V 2` (per-test, listing and whole-group runs). At that verbosity
  `printSummary` prints `Assertions:` and `print` prints every failure's and error's detail
  (`ooTest.frm` `printFailureInfo`, `printErrorInfo`). `-V 1` prints the same detail as far as I
  read (`print`, `ooTest.frm:644-672`), but I kept the reviewer's measured `-V 2` rather than claim
  the lowest.
* **The framework patch, and why.** `printSummary` (every verbosity above 0) calls
  `rxfuncquery("SysWinVer")` and `rxfuncquery("SysLinVer")` (`ooTest.frm:725`, `:728`); the
  crate refuses `RXFUNCQUERY` (Phase 10) and the oracle answers it through `RexxQueryFunction`,
  i.e. rxapi (`PackageManager.cpp:618-635`), which the rules forbid. The harness already lays out
  its own copy of `ooTest.frm`; `without_rxfuncquery` deletes each `rxfuncquery` test with the
  comment above it and the assignment it guards, asserting that shape, so an upstream edit to those
  lines fails loudly rather than silently leaving the call in. Neither name is registered on
  either side, so the removed code printed nothing: the oracle's `-V 2` summary shows
  `SysVersion:` alone (`$F/sanity/*.o.out`), as the reviewer found. `ootest/` itself is only read.
* Masked on both sides before comparing: the four timing lines' durations and the timestamp after
  `[failure]`/`[error]`. Everything else in stdout, and stderr and status, is compared raw.
* **Classification.** A test differing anywhere is split: if the outcome class (pass, failure or
  error, from `Failures:`/`Errors:`), the assertion count, stderr and status all agree, only a
  failure's or error's detail differs and it belongs to `DETAIL_DIFFERS`; otherwise it belongs to
  the failing set. Each set must equal its list exactly, and each named test must have a record
  line. The report line per member gives both sides' status, outcome class, assertion count, stderr
  excerpt and the first differing stdout line.
* **What `-V 2` found, measured before adding the list** (`$F/api2.log`): the recorded set
  unchanged, plus seven tests that fail on both sides alone and whose failure line reads `Line: -1`
  here (`FUNCTION.TEST_BUFFERED_INPUT`, `TEST_BUFFERING`, `TEST_FILE_INPUT`, `TEST_GLOBAL_SETTING`,
  `TEST_SIMPLE_WITH`, `TEST_WRITE_BUFFER`, `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`),
  and the whole-group METHOD run differing by the last one's `Line:` (3105 against -1). The
  framework takes that line from the condition object's `STACKFRAMES`, the frame named after the
  test (`OOREXXUNIT.CLS:900-905`), which is the review's I2. **I was told not to change `RECORDED`'s
  membership and that I2 belongs to round B**, so these seven are a second list, `DETAIL_DIFFERS`,
  asserted exactly and recorded in `phase-4-exclusions.txt` ("TESTS WHOSE FAILURE OR ERROR DETAIL
  DIFFERS", OWNER: Phase 8, the fix round owning I2); the one whose detail differs in the
  whole-group run too is flagged `true` and left out of that run. When round B fixes I2 the
  per-test part goes red with "detail no longer differing", which forces the list's removal.
  The lead may prefer a different shape; see Concerns.
* Result at `7f220b983`: `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test
  api_group_tests` ok, 50.68 s (`$F/api5.log`).

### The reviewer's M2, repeated

Built from `git archive 7f220b983` into `$F/head` with `ootest/` copied in (`diff -rq` against the
real one: identical), every file touched, own `CARGO_TARGET_DIR=$F/tgt` (`Compiling rexx-exec`
seen). Mutated file: `$F/head/ootest/ooRexx/API/oo/METHOD.testGroup`, methods inserted before
`-- test class for SendMessageScoped`, original kept at `$F/METHOD.testGroup.orig`. The methods
(`$F/addABC.txt`) are the reviewer's, compared against `REVIEWTRAP` as the reviewer's amendment
says. Sanity runs alone at `-V 2` first (`$F/sanity/`): A oracle pass, 1 assertion / crate
failure, `Actual: TESTREVIEWA`; B failure on both, 0 assertions, `Actual: REVIEWTRAP` / `Actual:
TESTREVIEWB`, `Line: 3198` / `-1`; C pass on both, 1 assertion / 0.

Predictions, written before either instrument run:

| # | Mutation | Prediction | Result |
|---|---|---|---|
| M2 | B and C only (`$F/addBC.txt`) | red, one panic carrying all of: `newly failing: ["METHOD.TESTREVIEWC"]`, `newly passing: []` (C's assertion count differs), `detail newly differing: ["METHOD.TESTREVIEWB"]`, `detail no longer differing: []` (B fails on both, detail differs), and the whole-group line for METHOD (its `Assertions:` line differs) | as predicted, all four in one panic (`$F/m2.log`): `newly failing: ["METHOD.TESTREVIEWC"]` (oracle 1 assertion, ours 0), `newly passing: []`, `detail newly differing: ["METHOD.TESTREVIEWB"]` (`Line: 3195` against `-1`), `detail no longer differing: []`, and `METHOD: ... assertions 1072, ... assertions 1071` |
| M1 | A, B and C | as M2 with `newly failing: ["METHOD.TESTREVIEWA", "METHOD.TESTREVIEWC"]` | as predicted (`$F/m1.log`): that `newly failing`, the same detail line, and `METHOD: ... assertions 1073, ... assertions 1071`. Group file restored from `$F/METHOD.testGroup.orig`, `cmp` equal to the real one |

## m1, m2, m3, m5

* **m1** (`7f220b983`): the per-test sets and the whole-group run are each checked into a list of
  problems and one `assert!` reports them all. Evidence: M2 and M1 above, whose predictions need
  both parts in one panic.
* **m2** (`7f220b983`): a test's record must be a line of `phase-4-exclusions.txt` equal to the
  test's name after trimming, the way every record names its tests. Evidence: while the
  `DETAIL_DIFFERS` records were not yet written the instrument stopped at `FUNCTION.TEST_BUFFERED_INPUT
  has no record in phase-4-exclusions.txt` (`$F/api3.log`).
* **m3** (`7f220b983`): `eval.rs` "`Object`'s six and `Pointer`'s four" is now "`Object`'s and
  `Pointer`'s"; the instrument's "Two options" paragraph is rewritten without a count. The older
  "the six" two lines above in `eval.rs` predates Task 8 (`git blame -w`: `6e41107d88`) and was
  left.
* **m5**: not fixed; `.superpowers/sdd/queued/2026-09-28-use-arg-message-term.md` holds the probe.

## Commits

* `91e7b4b24` Answer a copy of the condition object, and build a trapped one cheaply (I1, I5)
* `1e737cbf8` Give a library procedure's Routine object one annotation table (I3)
* `71bc06c18` Raise a compiled source's parse error as its SYNTAX condition (I4)
* `7f220b983` Compare each API group test's outcome and failure detail, not its counts (I7, m1,
  m2, m3)

Queued (under `.superpowers/`, not committed): `2026-09-28-use-arg-message-term.md` (m5),
`2026-09-28-compiled-source-parse-errors.md` (rewritten to the residual),
`2026-09-28-condition-object-divergences-found-in-fix-round-a.md`.

Test runs before each commit: `cargo fmt --all --check`, `cargo clippy -p rexx-exec --all-targets
-- -D warnings`, and `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec` for `--lib`,
`corpus`, `api_group_tests`, `refusal_sites`, `method_bodies` (and `collect_stress` for the first
two), plus `-p rexx-parse --test sourceline_oracle`; all green, corpus 631, 632 and 633 of the
same (`$F/t2.log`, `t3.log`, `t4.log`).

## Gates

(pending)

## Concerns

* **`DETAIL_DIFFERS` is a second list the instrument asserts.** `-V 2` makes seven tests that fail
  on both sides alone differ in their failure line (I2), and I was told neither to change
  `RECORDED`'s membership nor to fix I2. The list is exact in both directions, its tests are
  recorded with owner "Phase 8, the fix round owning I2", and it goes red when that fix lands. If
  the lead would rather have them in `RECORDED`, or have round B's I2 fix land first, the list is
  one `const` and one record block.
* **The FUNCTION io tests' detail is compared only alone**, where they fail on both sides for want
  of the handler; in the whole-group run, where they pass, only
  `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` is left out.
* **I4's residual has no owner.** The compiled source's own frame (traceback line and the
  untrapped `running <name> line <n>`) is recorded with "OWNER: not assigned by the fix round";
  `ParseError` has no clause end to echo. The controller should give it one.
* **I5's observed case costs more than BASE**: a handler that reads `condition('O')` pays for the
  copy the oracle also makes, +32% on the loop against BASE (which answered the uncopied object),
  -29% against HEAD. The unobserved loop the finding named is -12.7%.
* **I3 changed a loud answer to a silent one**: a library `Routine` object no directive binds now
  answers `~annotation` with `.nil` (the oracle's), witnessed by
  `library_routine_unbound_annotation.rex`.
* **Found, not fixed**: a NOTREADY condition object's entry set, and a CALL ON handler's missing
  argument 1 (the condition object), both pre-existing and unrecorded; `.context~condition` could
  now answer `condition_copy`. Queued, see Commits.
