# Task 8 review -- METHOD, CONVERSION and FUNCTION

Reviewer: review-s8. Range `391242b7e..2b26b6970`. Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s8/`
(below, `$R`). Written first, filled as the work goes.

## Verdict

**Changes needed.** No Critical finding. Six Important: two silent divergences the fixes
introduced (I1, I3), an instrument that cannot see a failure's detail or an assertion count (I7),
an unrecorded divergence that instrument hid (I2), a record whose stated cause is not what its
tests need (I4), an unmeasured +88% on trapped conditions (I5), and an owner assignment the roadmap
text contradicts (I6).

## Findings

### Critical

None.

### Important

* **I1. `condition('O')` answers the condition object itself; the oracle answers a copy.**
  `rust/crates/rexx-exec/src/builtin/state.rs:335` against `BuiltinFunctions.cpp:2681`
  (`conditionobj->copy()`). `$R/p/co2.rex`: `(c == condition('O'))` oracle 0, crate 1;
  `c['CODE'] = 'changed'` then `condition('O')['CODE']` oracle `41.1`, crate `changed`.
  `co4.rex`: a replaced `ADDITIONAL` reaches `condition('A')` on the crate only. At BASE the same
  program ended loudly (`==` refused, and the store family refused on the `NativeObject`), so the
  store-backed condition object (`condition.rs:89`) made a loud gap silent. The witness
  `condition_object_directory.rex` reads, never writes.
* **I2. A trapped SYNTAX condition's POSITION, STACKFRAMES and TRACEBACK describe the trapping
  activation, not the raising one; unrecorded.** Pre-existing (same at BASE), not made by this
  task, but on every failure report the Phase 8 groups print. `$R/p/sf3.rex`: oracle frames
  `RAISER 14`, `TRAP 10`, program, POSITION 14, 3 traceback lines; crate `TRAP 10`, program,
  POSITION 10, 2. `tb1.rex`/`tb2.rex`: a trapped `RAISE SYNTAX`'s traceback loses the unwound
  internal calls' lines. In the groups: `$R/sv2/d/FUNCTION.TEST_BUFFERING` (`Line: -1` against
  `1300`), `FUNCTION.TEST_BUFFERED_INPUT` (`Line: -1`, `Program:` printed, raising line and
  `Compiled method "SEND"` line missing). `phase-4-exclusions.txt` has the traceback family
  (`:5019`-`:5179`) but no entry for this shape (`grep -n -i stackframes` finds only `:4696`, a
  different one). Needs a record with an owner.
* **I3. The shared Routine object carries the first directive's annotations; the oracle's
  carries every directive's, a later one winning per key.** `environment/identities.rs:654-664`
  (`.min()` over the bound directives). `$R/p/gr1.rex`, `gr1b`, `gr1c`: two directives on
  `orxfunction TestGetRoutine`, the second annotated `k 'v2' j 'j2'`: oracle `v2 j2`, crate
  `v1 The NIL object`; `gr2.rex` (order swapped): oracle `v1 j2`, crate `v2 j2`. With one
  annotated directive (`gr1d`, the witness's shape) both agree, which is all the report's
  "measured: annotations answer v1 on both sides" covered. The comment states the wrong rule.
* **I4. `TESTNEWMETHOD02`/`TESTNEWROUTINE02`'s record names a cause the tests do not need.** See
  section 4: both tests check only the code `36.901`, which the crate already computes; the fix
  looks local and the ruling puts local runtime fixes in Task 8.
* **I5. Trapped conditions cost +88% instructions, unmeasured.** Section 5. Every trapped
  condition now builds a store-backed Directory with one `PUT` send per entry
  (`condition.rs:89`, `:201`).
* **I6. Owners conflict with roadmap rows 8 and 9.** Section 4. Needs the rows amended or the
  members moved; the lead's call.
* **I7. The instrument is count-only; `-V 0` is not forced.** `api_group_tests.rs:153`, `:254`,
  `:288`. `-V 0` prints tests, failures and errors, no assertion count and no failure detail
  (`ooTest.frm:609-616`). M2 is green with a test the crate fails for a different reason and a
  test the crate runs with fewer assertions. The `rxfuncquery` the report cites as the reason
  sits at `ooTest.frm:725`/`:728` in a copy the harness already rewrites; deleting `:724-729` in
  the copy runs `-V 2` on both sides (`$R/sweep.sh`), and the only extra lines to mask are the
  timings and the `[failure]`/`[error]` timestamps. Measured at HEAD, `-V 2` changes no verdict
  outside the tests failing on both sides, so this hid I2 and nothing Phase 8 owns.

### Minor

* **m1.** A red per-test part hides the whole-group part: `assert!` at `api_group_tests.rs:314`
  panics before `:318` is evaluated (M1).
* **m2.** The record check (`api_group_tests.rs:231-237`) is `exclusions.contains(test)`: the bare
  test name anywhere in the file, not the group, not the owner.
* **m3.** Set sizes in new comments, `eval.rs:1189` and `api_group_tests.rs:22`.
* **m4.** Refusal texts name Phase 5 for members recorded as Phase 9 (`rec1`, `rec3`-`rec6`).
* **m5.** `USE ARG o~a` still refuses (`a message send is not implemented`, `$R/p/pm8.rex`,
  oracle `x`). Outside this change; noted because the queued item asked for the target forms.

## 1. The instrument

Build: `git archive 2b26b6970` into `$R/head`, every file touched, own `CARGO_TARGET_DIR=$R/tgt`
(`Compiling rexx-exec` seen). `ootest/` in the archive is a scratch copy of the real one, which
is only read. Unmutated: `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test
api_group_tests`, ok, 60.14 s (`$R/inst-base.out`).

### Mutation predictions (written before any ran)

Group-copy mutations: three test methods added to `$R/head/ootest/ooRexx/API/oo/METHOD.testGroup`
before `-- test class for SendMessageScoped`, each reading `condition('O')~stackframes[1]~name`
of a SYNTAX error raised in `reviewRaiser` and trapped in its caller `reviewTrap`. Amended before any instrument run: a sanity run of the three alone showed
that inside the group the oracle answers `REVIEWTRAP` (not `REVIEWRAISER`, as in the standalone
`$R/p/sf3.rex`) and the crate `TESTREVIEWA`, so A and C compare against `REVIEWTRAP`:

* `testReviewA`: `assertSame('REVIEWTRAP', name)`: oracle passes, crate fails.
* `testReviewB`: `assertSame('never', name)`: fails on both sides, with different actuals.
* `testReviewC`: `if name == 'REVIEWTRAP' then assertTrue(.true)`: passes on both, one
  assertion on the oracle, none on the crate.

| # | Mutation | Prediction |
|---|---|---|
| M1 | A, B and C added | red; `newly failing: ["METHOD.TESTREVIEWA"]` only, `newly passing: []`; the whole-group METHOD line differs |
| M2 | B and C only | green: `-V 0` prints neither a failure's detail nor an assertion count |

Results:

* **M1: as predicted for the per-test part**: `newly failing: ["METHOD.TESTREVIEWA"]`, `newly
  passing: []` (`$R/m1.out`). The whole-group half of the prediction is **unverified**: the
  per-test `assert!` (`api_group_tests.rs:314`) panics before the whole-group one (`:318`) is
  evaluated, so a run red in both parts reports only the first.
* **M2: green, as predicted** (`$R/m2.out`, 49.42 s). A test the crate fails for a different
  reason than the oracle, and a test in which the crate performs fewer assertions, are both
  invisible. Sanity runs of the same three at `-V 2` show each (`Actual: REVIEWTRAP` against
  `Actual: TESTREVIEWB`; `Assertions: 1` against `0`).

### Are `-U` and `-V 0` justified?

* `-U`: yes. The ticker is a `REPLY` thread waiting on `GUARD ON WHEN`; the crate refuses (Phase
  6), and a `GUARD ... WHEN` whose expression is false is oracle-crashes entry 7's territory.
* `-V 0` avoids `printSummary` (`ooTest.frm:709`), whose `rxfuncquery("SysWinVer")`
  (`ooTest.frm:725`) reaches `RexxQueryFunction` on the oracle (`PackageManager.cpp:618-635`,
  read: `SysWinVer` is not a loaded routine). The premise is true. The conclusion is not the only
  one: the harness already rewrites its copy of the group file, and deleting `ooTest.frm:724-729`
  in the copy (both `rxfuncquery` tests) lets every verbosity run on both sides. Measured,
  `$R/sweep.sh basev sv2 2`: every test the oracle lists, alone, at `-V 2`, with the four timing
  lines stripped. The crate prints `Interpreter:`, `OS Name:`, `SysVersion:` byte-identically.
  What `-V 0` drops is exactly the assertion count and every failure's detail (`printBrief`,
  `ooTest.frm:609-616`).
* **Does skipping them hide a Phase 8 failure?** No member of the three groups changes verdict at
  `-V 2`: the tests differing at `-V 2` are the recorded ones plus the ones that fail on both
  sides alone (`METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` and the FUNCTION io tests),
  and every other test's assertion count agrees (`/bin/grep -a -v ' same ' $R/sv2/verdicts.txt`).
  What the fail-on-both tests' detail shows is a real divergence, not Phase 8's: every failure
  and error report says `Line: -1` on the crate against the test's line on the oracle
  (`$R/sv2/d/FUNCTION.TEST_BUFFERING`), and an error's traceback lacks the raising line and the
  `Compiled method "SEND"` line (`$R/sv2/d/FUNCTION.TEST_BUFFERED_INPUT`). Finding I2.


## 2. The framework fixes

Probes in `$R/p/`, each run by `$R/cmp.sh` (the standard wrapper from a fresh `mktemp -d`, stdin
`/dev/null`, three descriptors compared separately; the crate is `$R/tgt/release/rexx-run` built
at `2b26b6970`; BASE runs use surface-8's `base-target/release/rexx-run`).

* **String~copy** (`cp1`, `cp5`-`cp8`): values, classes and arithmetic agree; every enhanced or
  renamed string refuses loudly on the crate (`OBJECTNAME=`, `Table~supplier`). Identity: the
  oracle's copy is a distinct object at every length (`IdentityTable~hasIndex`, `cp5`); the
  crate's copy of a short, handle-carried string is the same object. That is the "identity is not
  modelled" licence (`phase-4-exclusions.txt:975-996`), as the report says. Note for the next
  reader: comparing two `identityHash` values with `=` under DIGITS 9 is the trap
  `phase-4-exclusions.txt:1015` describes; `cp5`'s `lit36`/`cmp36` lines fell into it.
* **makeArray with a separator** (`ma1`-`ma5`): empty receiver, separator absent, equal to or
  longer than the receiver, overlapping (`'aaa'`/`'aa'`, `'aaaa'`/`'aa'`), trailing, a numeric
  receiver and separator, the null separator, a non-string separator, too many arguments: all
  identical.
* **Message~new / send / sendWith** (`ms1`, `me1`-`me14`): 'I' with omitted positions, 'A', a new
  receiver kept for later sends, new arguments replacing old ones, a scope override and its 93.957,
  every argument error of `newRexx` and `sendWithRexx`: identical. `Message~target` refuses
  loudly (`me13`).
* **Operators on Array and native objects** (`op1`-`op13`): `==`, `=`, `\=`, `<>`, `><`, `\==`
  on Array, Method, Routine, Package, Directory, MutableBuffer, WeakReference, Bag, Queue,
  CircularQueue, Table, Relation, List, StringTable, IdentityTable, `.context`, `.rexxinfo`,
  `.stdout`, a Supplier and a Message; `<`/`>`/`+`/prefix `-`/`\` for 97.1 or 41.1;
  concatenation: identical.
* **RAISE SYNTAX offered at the nearest non-internal activation** (`rs1`-`rs3`, `tb1`, `tb2`):
  internal calls skipped even when they hold their own `SIGNAL ON SYNTAX`, INTERPRET (nested)
  inside an internal call and inside a `::ROUTINE`, the raising method's own trap, `CALL ON ANY`
  declining, the EXIT form, a trap already fired: which trap fires, the variables it sets and
  the program's end agree. One divergence on the newly reachable path, pre-existing: the trapped
  condition object's `TRACEBACK` lacks the lines of the internal calls unwound (`tb1`: oracle 4
  items, crate 2). `tb2` shows the same divergence at BASE for a trap in the outermost activation,
  so the change widened it rather than caused it. Folded into finding I2.
* **RAISE ... ARRAY's ADDITIONAL** (`ra1`, `ra2`, `ra5`): the items themselves (identity kept),
  omitted positions as holes (`size` 3, `items` 2), an empty ARRAY, a SYNTAX message still
  substituted from the rendering, and the `>K>` trace line: identical.
* **Condition objects as store-backed Directories** (`co1`-`co4`): the Directory surface
  (`items`, `allIndexes`, `hasEntry`, `entry`, `put`, `remove`, `copy`, `supplier`,
  `makeArray`, unknown-entry NIL) agrees. **One silent divergence, new: finding I1.**
* **PARSE into message terms** (`pm1`-`pm8`): attributes, `[]=` on Array/Stem, a setter with
  arguments (omitted ones too), a scope override `o~a:.t`, PARSE VAR/UPPER/LOWER/CASELESS/ARG/
  PULL, positional patterns, a repeated target, `.nil~foo`'s 97.1, and the `>=>`/`>A>` trace
  lines under `TRACE R` and `TRACE I`: identical. `USE ARG o~a` still refuses loudly (`pm8`,
  `a message send is not implemented`); USE is a different instruction and not in this change.


## 3. GetRoutine's shared object

* Identity and `==`: `findRoutine`, `.routines`, `GetRoutine` in either order, repeated, and
  after 2000 allocating iterations and a collection loop (`gr3`): identical. The object is rooted
  by `add_global` under `library_routine_root_key` (`identities.rs:650`) before it enters
  `routine_objects`, so the closure note does not apply: nothing allocated here waits in a Rust
  local across an allocation.
* **Annotations: finding I3.** With one annotated directive (`gr1d`, the witness's shape) both
  sides agree. With two directives naming the procedure and each annotated, the oracle's one
  object carries every directive's annotations, a later directive's value winning per key
  (`gr2`: `k` from the second directive, `j` from the first); the crate carries the first
  directive's alone (`gr1`, `gr1b`, `gr1c`, `gr2`).


## 4. The recorded failing set

Every record's probe re-run at `2b26b6970` (`$R/p/rec1.rex`-`rec6.rex`): each reproduces as
written (oracle `FooBar Object 0 The REXX Package`, `5`, `0`, `1`, `36.901 1` for Routine and
Method; the crate's refusals as quoted). `TEST_INPUT_OUTPUT_STREAM`'s reader is shared by a shell
command (`redirect.rs:688`), so it is ADDRESS WITH's, not the API's. `TEST_REXXQUEUE` is RXAPI's
queue. No record names Phase 8 (`/bin/grep -a -n -i 'owner: phase 8'` over the new block: none).

* **`TESTNEWMETHOD02` and `TESTNEWROUTINE02`: finding I4.** Both tests are
  `self~expectSyntax('36.901')` and nothing else (`METHOD.testGroup:1833-1837`, `:1850-1854`), and
  `checkCondition` (`OOREXXUNIT.CLS:1352-1366`) compares only the code when no inserts are given.
  The record's cause, "a substituted message, POSITION inside the source and a traceback line of
  it, none of which rexx-parse keeps", names three things these tests do not read. The crate
  already has the code: its refusal carries `36.901` (`rec5`, `rec6`). Raising SYNTAX 36.901 from
  the compile path looks local, and the message text is Phase 3's standing licence (roadmap row 3:
  "message text and substitutions deliberately not reproduced"). Not measured: whether a SYNTAX
  raised inside `NewMethod` reaches the test through the native frame as the oracle's does.
* **Owners against the roadmap: finding I6.** Row 8 (`2026-07-27-rust-rewrite.md:544`) makes
  "native-API ooTest groups pass" Phase 8's exit, and row 9 (`:545`) says "No native-API ooTest
  group is this phase's". The records give METHOD and FUNCTION members to Phase 9 by cause. The
  causes are plausibly Phase 9's (Class~new, EXPOSE of a tail, package tables, a stream reader);
  the text of both rows is then wrong, and nothing in the diff amends them.
* The refusal texts behind four records still name Phase 5, which is closed, while the records say
  Phase 9 (`rec1`, `rec3`-`rec6`). Minor m4.


## 5. Performance

* No file under `src/ir/` changed (`git diff --stat 391242b7e..2b26b6970`). In `run.rs`, the new
  arm of `assign_expr_target` is reached only for a message-term target; in `run/condition.rs` both
  changes are on the raise path; `eval.rs` adds one arm to a match on the heap body. No per-clause
  work: agreed.
* **Finding I5: the raise path is not flat.** The three benches the report measured raise no
  condition. A trapped SYNTAX error in a loop (5000 iterations of `call f` with `f` trapping
  `1 + 'a'`), callgrind `Collected` (profiles and program in `$R/cg/`): BASE (surface-8's `base-target`) 627,591,878, HEAD
  1,178,416,998, **+88%**. Inclusive `build_condition_object_from`: 462,478,061 against
  1,010,052,824, about 92k against 202k instructions per condition object; 623,353,736 of HEAD's
  are `dispatch::hash::native_hash_put`, the store-backed Directory's `PUT` that
  `condition.rs:201` sends once per entry (the send was there at BASE; its receiver is what
  changed at `condition.rs:89`). Wall clock, 50000 iterations, three runs each: 0.59/0.59/0.62 s
  against 1.08/1.52/1.07 s. No bench axis covers this, so no gate would see it.


## 6. Quality

* No `unsafe` added (`git diff ... -- 'rust/crates/*.rs' | grep -c '^+.*unsafe'`: 0). No em-dash
  (U+2014/U+2013) in any added line.
* Records re-derived and green at HEAD in my build: `REXX_CORPUS_GATE=1 cargo test --release -p
  rexx-exec --test corpus --test method_bodies --test refusal_sites` (630 of 630 matching; 29, 23
  and 5 passed) and `-p rexx-parse --test sourceline_oracle` (1 passed).
* Set sizes in new comments: `eval.rs:1189` ("`Object`'s six and `Pointer`'s four") and
  `api_group_tests.rs:22` ("Two options"). Minor m3.


## Housekeeping

`$R/tgt` deleted after the runs (every binary figure above was taken before). `$R/head` (the
archive and its `ootest/` copy, `METHOD.testGroup` restored, `cmp` clean against `ootest/`) and
the probe outputs remain in `$R`. Nothing in the repository was edited.
