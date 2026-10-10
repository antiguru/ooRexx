# Phase 6.1 Task 11a report: deferred items, small

Base `fbdf615e8` (code `cc21b5ae3`, binary `fr1`). Head `849f3dfb0`. Paths are under
`rust/crates/rexx-exec/` unless they say otherwise. Scratch: `/tmp/claude-1000/p61/t11a/`. Oracle
runs use the global constraints' command, each from a fresh `mktemp -d` (`tools/both.sh`, which runs the
crate under `memcap 2G timeout -k 5 20`). "N/N" means matching runs out of runs made, per engine.
"Fails at base" means the base binary (`/tmp/claude-1000/p61/hsr/bin/fr1/rexx-run`) was run on the
witness and the result is quoted. For a crate test that names a symbol absent at base, the
behaviour it pins was checked by a mutation, which is stated.

## Step 0: Task 11 review Minors (`c472613dd`)

* `src/scheduler/tests/mutants.rs`: M9's test runs every spec before it judges, with
  `pre:2,k=40` and `uniform:0.3` first. Under the M9 mutant (`when_parked = false;` deleted in
  `cancel_wait`, `scheduler.rs:1190`), both of those answer `caught 11.1 / result The NIL object / main
  halted`, which is the halt route. `uniform:1` at two seeds refuses at rc 120 (log
  `s0-m9.txt`). Unmutated: 4 passed (`s0-unmut.txt`). The mutation was restored from a copy, and
  `git diff` was empty afterwards.
* `tests/concurrency_tests.rs`: M7's comment says the one-clause comparator gives B an odd count of
  pinned visits, and that per-key totals need an odd count.
* `src/environment.rs` `set_native_entry`: the dead-handle counter reads with `heap.peek`.
* `task-11-report.md`: the M9 row is corrected.

## Step 1: empty-body `TIME('E')` loop (`e25925ad8`, `a8593f6e6`)

* Change: `src/run/loops.rs:1959` sets `clock_stale` before an `UNTIL` test, and `:2158` before a
  `WHILE` re-test after the first pass. The first test shares the `DO` clause's reading. `:1479`
  (`run_repeating`) does the same after the body; no probe reached it, because every repeating loop
  takes the flat path (`flat_loop_start` falls back only for a labelled `Simple`). Removing that store
  left the witness green. The first version (`e25925ad8`) stored on every pass and cost emptyloop
  +2.25% Ir, so `a8593f6e6` moved the store to the two test sites.
* Test: `corpus/lang/time_elapsed_empty_loop.rex` (plain WHILE, controlled WHILE, UNTIL, inside
  INTERPRET). At base it does not finish: rc 124, 4.1 at line 5, which is a structural failure in the corpus
  harness. After the change it matches 5/5. Removing the flat-path store brings rc 124 back. The scout's
  `s1_while`, `s1_ctl`, `s1_interp` and `s1_nop` probes match the oracle, 1/1 each.
* Perf (`callgrind.sh -r 3 -p "emptyloop rexxcps decloop"`, base against `a8593f6e6`): emptyloop
  -0.0000%, rexxcps -0.0016%, decloop +0.0000%.

## Step 2: `Message~halt` after `~send` (`23716a5ec`)

* Change: `src/scheduler.rs` gains `Activities::senders` (`MessageClass`'s `startActivity`), plus
  `record_message_sender` (`:2389`), called from `dispatch_held_message`
  (`src/dispatch/object_protocol.rs:1009`). `halt_message` (`:2354`) queues the halt on that
  activity's running activation and answers 1 where the activity has ended (`Activity::halt`).
  `collect_now` prunes swept messages (`src/lib.rs:3030`, `prune_senders`).
* Tests: `corpus/lang/message_halt_after_send.rex` and `message_halt_sender_ended.rex`. At base they
  give rc 0 with stdout `... 0 ...`; now each matches 5/5. Unit test
  `scheduler::tests::a_sender_record_lives_and_dies_with_its_message`: deleting the prune turns it red
  (left holds both handles).
* Witnesses, 5 runs per engine: the scout's `pend4` (REPLY) gives crate 5/5 rc 252 with the oracle's
  stderr. Its stdout is one of the oracle's two orders: the oracle gave `v,1,rest...` 3 times and
  `v,rest 1,1,...` 2 times. `sendhalt`, `nosend`, `trapped` and `other` match 5/5. Under
  `REXX_SWITCH_MODE=every`, `pend4`, `sendhalt`, `trapped` and `other` match 5/5 with the oracle's
  descriptors.

## Step 3: `EXIT` in a CALL ON handler at the REPLY clause's end (`04c46cb3b`)

* Change: `src/run.rs:2521` `exit_after_reply` makes the reply check on the activation the `EXIT`
  ends, which is the innermost that is not an internal call (`exitFrom`,
  `RexxActivation.cpp:1402-1441`). A bare `EXIT` there replaces the pending reply's value with none, and
  an `EXIT` with a value is 98.937.
* Tests: `corpus/lang/reply_handler_exit.rex` (oracle rc 165, 91.999),
  `reply_handler_exit_value.rex` (rc 158, 98.937 raised to the sender) and
  `reply_handler_exit_after_split.rex` (98.937 on the continuation). At base all three give rc 0 with no
  error; now each matches 5/5. `s3_lastreply`, `s3_callexit`, `s3_fallreply` and `s3_lastclause`, the
  neighbours that already agreed, still match 5/5.

## Step 4: a user STRING answer (`5bb6510af`, `9102f57aa`, `0517efbfb`)

* Change: `src/dispatch/reqstr.rs:244` `string_answer_text` reads the STRING answer the way
  `primitiveMakeString` does (`ObjectClass.cpp:1285`). A string or number is itself. An Array or Array
  subclass is its items joined by a newline, read through `array::collection_store`. A MutableBuffer
  is its text. Anything else (a Queue, `.nil`, a Directory, an instance) is "no string value". Per
  controller ruling (c), a consumer that judges the answer as an object reads it as `.nil`:
  * truth: `eval.rs:1479`, and through it every condition context;
  * `& | &&`: `eval.rs:1324`, `required_string_operand`, where the operator is read only on the cold
    path;
  * the native logical argument: `src/dispatch/library.rs:1503`.
  These answer as the oracle does: 34.x with found "The NIL object". Every other consumer refuses with
  `Loud::string_answer_not_a_string` (`src/lib.rs:515`, owner none, Deviation 30). The enumeration
  command for both sets, and its output, is below. Deviation 30 is in
  `docs/superpowers/plans/phase-4-exclusions.txt`. R11's line in the spec is amended. The refusal
  tables are re-derived (`REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites`), and the new row is
  in `refusal-dispositions.tsv`.
* Consumer enumeration:
  `grep -rn "required_string_value(\|required_string_arguments(\|required_string_or_nil(\|required_string_operand(" --include=*.rs src | grep -v "fn required_string\|/tests\|tests.rs"`
  gives 34 sites. The object set is `eval.rs:1324` (logical operators only), `eval.rs:1479` and
  `dispatch/library.rs:1503`. Every other site reads bytes. `say` is in the bytes set: its oracle answer is
  88.909 with a traceback through the REXX package's `Stream~SAY` and `Monitor~UNKNOWN`, which this
  crate has no frames for.
* Tests:
  * `corpus/lang/string_answer_array.rex` covers say, concatenation, length, truth, an Array
    subclass and NOSTRING. At base it gives rc 222, 34.1 found "an Array"; now 5/5.
  * `tests/truth/values` gains `.sa~new` (an Array answer, true in each context) and `.sn~new` (a
    Directory answer, 34.x in each context). At base the harness reports 34.1 found "an Array".
  * `dispatch::tests::a_string_answer_reads_as_primitive_make_string_reads_it` pins the Array,
    subclass, buffer and number answers, the refusal under concatenation and `length()` for a
    Directory, Queue, `.nil` and Object, and the `if` and `&` answers for each of those four.
* Witnesses: the scout's `i7a` and `i7d`, and `s4_array`, `s4_arraysub`, `s4_buffer`, `s4_number`,
  `s4_truth`, `s4_truth0` and `s4_nostring`, each matches 1/1, re-run at `849f3dfb0`. `truth_dir`, `truth_nil`, `truth_queue`,
  `truth_inst`, `found_nil` and `and` match 5/5. All 19 truth-table contexts over `.sa~new` and over
  `.sn~new` match 1/1 each, except `do-to` and `do-by`. Those two diverge for `.s1~new` as well
  (`do-to-s1.rex`): it is the oracle's DO TO identity test. The bytes probes `bytes_dir`, `bytes_nil`,
  `bytes_queue` and `bytes_obj` give, on the oracle (2 runs each), a `length` that differs per run and
  then Error 5. The crate refuses, rc 120.
* `corpus/lang/library_native_object_arguments.rex` (`t~logical(.S~new(.nil))`) was broken by the
  blanket refusal in `5bb6510af`. It matches again from `9102f57aa` (corpus gate below).

## Step 5: wrong-type native rows (`ae86f4d04`)

* Change: `src/dispatch/hash.rs:190` `not_this_task` goes through `refusal_outside` (`:202`): a
  receiver whose class is not a hash class refuses with `Loud::receiver_class("a value that is not a
  hash collection")`. A hash-class receiver with no store keeps `native_method`; a condition object's
  Directory is the case that has one, queued `2026-10-01-condition-object-directory-methods`. Stem
  (`hash/stem.rs`, `stem_refusal`) refuses with `... not a stem`. MutableBuffer (`dispatch/buffer.rs:396`)
  refuses with `... not a mutable buffer`. Message SEND, START and REPLY
  (`object_protocol.rs:1001,1115,1197`) refuse with `... not a message`. The MutableBuffer helpers no
  longer carry the method name, which only the old refusal used. Deviation 28's prose is updated. The
  refusal tables re-derived without change.
* Test: `dispatch::tests::a_native_row_on_a_receiver_of_another_class_refuses_its_type`. It covers
  each row, the condition object's refusal (unchanged) and a MutableBuffer subclass whose INIT does
  not forward (answers `0 / xy`). At base every row gave `method "X" of class "..." is not implemented
  (Phase 9)` (probes `probes/s5/w_*.rex`). The oracle is not run on these, because of
  `oracle-crashes.txt` entry 33.

## Step 6: `Routine~new` and `Method~new` take a context (`98caea3de`)

* Change: `src/dispatch/construct.rs:173` accepts a third argument and refuses a fourth (INIT
  arguments). `executable_context` (`:225`) resolves it for both `new` (NEW, position 3) and `newFile`
  (NEWFILE, 2). `compile_routine_source` and `compile_method_source_in`
  (`dispatch/class_protocol.rs`) install directives under it.
* Harness: `tests/assertions.rs` `program_for` runs a row that sends to `self` as a method of a test
  case. The test case carries `Literals.testGroup`'s `q`, `hex` and `bin` and the framework's
  `runDynamicSource`, and the row compares its two values in hex, because `test_string_range` holds
  every byte. EXEMPT is empty.
* Tests: `corpus/lang/routine_new_context.rex`; at base it gives rc 120 with the Phase 9 refusal, now 5/5.
  `REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec --test assertions`: 5 passed. At base the eleven
  Literals rows did not pass, so an empty EXEMPT is red there.
* Witnesses: `i9b` gives `AB A` 1/1. `ctx.rex` matches 1/1, including 40.904 for `'xyz'`, and `ctx5.rex`
  (`.nil` context) matches 1/1.

## Step 6b: a compiled source resolves through its package's parent (`8b95afe79`, `8c587bc1f`, `849f3dfb0`)

The controller ruled this in after Step 6 turned six TEST_NEW_CONTEXT_* refusals into wrong answers.

* Oracle rule: `generateRoutine` and `generateMethod` give the executable's own package the source
  context as its parent (`LanguageParser.cpp:603`, `:637`; `PackageClass.cpp:680-684`). The context
  defaults to the caller's package (`BaseExecutable.cpp` `processNewExecutableArgs`;
  `MethodClass.cpp:474-484`). `findInstalledClass` recurses to the parent (`PackageClass.cpp:982-1003`).
  `findPublicClass` checks installed public, then merged public, then the parent's (`:1014-1049`).
* Change: `src/install.rs` `record_compiled_body` and `record_compiled_routine` record the parent.
  `set_package_parent` (`environment/identities.rs:761`) also sets a `parented` bit (`src/lib.rs`).
  `installed_class` and `imported_class` (`environment.rs:682`, `:702`, `#[inline]`) keep their local
  lookup, and after a miss they test the bit. Only when it is set do they call the cold
  `parent_installed_class` or `parent_public_class` (`identities.rs:780`, `:795`). Those walk
  `package_lineage`, which `find_routine` shares. `Package~findClass` and `findPublicClass`
  (`installed_class_of`, `package_public_class_of`) take the same walks.
* Tests: `corpus/lang/routine_new_package_parent.rex` covers routine and class, no context and
  context, Method, a source with directives under a context, and the REXX package context (43.1).
  `corpus/lang/package_find_through_parent.rex` covers findClass, findPublicClass, findRoutine and
  findPublicRoutine. Both match 5/5. Crate tests:
  `dispatch::tests::a_compiled_source_resolves_through_its_package_parent` (reverting the class walk
  gives 97.1 on `.HIDDEN`; reverting the parent record gives 43.1 on `HELPER`) and
  `a_compiled_sources_package_finds_through_its_parent`. At `0517efbfb`
  `routine_new_package_parent.rex` gives rc 213.
* Witnesses: `ctx2`, `ctx3`, `vis`, `dir` and `find` each match 5/5.
* Perf: walking the chain on every miss cost alloc +2.70% (`8b95afe79`). The cold walk behind a
  bit cost +0.87% (`8c587bc1f`), because `installed_class` stopped inlining. With `#[inline]`
  (`849f3dfb0`) alloc is +0.336% against base61; `dot_variable` +48,000,696 Ir over `0517efbfb`, 8 Ir
  per local miss.
* whole_groups: every TEST_NEW_CONTEXT_* and TEST_NEWFILE_CONTEXT_* passes. Method rest is `error,
  assertions 85 ... failing [TEST_NEW_ARRAY_FROM_FILE TEST_NEW_FILE_COMPILED]`, and both tests failed
  at base `fbdf615e8` and at `0517efbfb` in the same way. Run alone at `fr1` and at `849f3dfb0`
  (`testOORexx.rex -f ooRexx/base/class/Method.testGroup -U -V 2 -t <test>`), each answers `SYNTAX
  98.971 raised unexpectedly. External command "rexxc ..." failed with return code 127` (lines 221 and
  352): `rexxc` is not on the run's PATH. The base expectation line lists both.

## Step 7: collections without copying slots (`e8a19b2e6`)

* Change: `src/dispatch/collection.rs:40` `read_slots` borrows the store's slots for a read.
  `append_slot` (`:263`) reads the last item and the length through it, and growth still charges
  through `array_grow`. `last_item`, `Array~first`/`last`, `isEmpty`, `empty`, `hasIndex`, `remove`,
  the `sortWith` counts and Queue's item count use it too. `List`'s handle and free-stack reads borrow
  (`collection/list.rs`). `Array~items` already borrowed; it is still an O(n) count per call.
* Test: `dispatch::tests::appends_and_reads_copy_slots_linearly` counts slots copied with a test-only
  counter in `array_slots_owned` and `occupied`, at 2000 and 4000 appends plus an unwind 500 and 1000
  deep. It requires fewer than 2.5x as many copies at twice the size. Reverting `append_slot` gives
  2,008,002 and 8,016,002, which is red. Reverting List's handle read gives 2,258,502 and 9,017,002, also
  red. The stdout it asserts (`h 501`) is the oracle's, 1/1.
* Scaling, wall clock (elapsed/user s, 3 interleaved runs each; base = `fr1`, head = `e8a19b2e6`):

| program | base | head | oracle |
|---|---|---|---|
| append 1e3 | 0.05-0.06 / 0.02-0.03 | 0.03-0.06 / 0.01-0.02 | 0.00-0.01 / 0.00 |
| append 1e4 | 0.08-0.11 / 0.07-0.08 | 0.05-0.06 / 0.02-0.03 | 0.00-0.01 / 0.00 |
| append 1e5 | 5.35-5.37 / 5.30-5.34 | 0.07-0.08 / 0.03-0.04 | 0.02 / 0.01 |
| listappend 1e4 | 0.06-0.08 / 0.03-0.04 | 0.04-0.07 / 0.02 | 0.01 / 0.00 |
| listappend 4e4 | 0.27-0.28 / 0.23-0.25 | 0.05-0.07 / 0.02-0.03 | 0.01 / 0.00 |
| unwind 2000 | 0.06-0.07 / 0.02-0.04 | 0.06-0.08 / 0.02-0.03 | 0.01 / 0.00-0.01 |
| unwind 4000 | 0.06-0.09 / 0.03-0.04 | 0.06-0.08 / 0.02-0.03 | 0.02 / 0.01-0.02 |
| unwind 8000 | 0.08-0.12 / 0.05-0.06 | 0.07-0.10 / 0.03-0.04 | 0.03 / 0.02-0.03 |

* Callgrind, one run each (total Ir / libc / rest less ld): unwind 2000: 235.8M / 147.7M / 87.7M
  base, 173.3M / 85.5M / 87.4M head. Unwind 4000: 479.5M / 362.2M / 117.0M base, 224.2M / 107.3M /
  116.5M head. Unwind 8000: 1352.7M / 1176.8M / 175.6M base, 325.6M / 150.5M / 174.7M head. Append 1e3:
  140.8M base, 122.9M head. Append 1e4: 1784.2M base, 139.1M head. List append 1e4: 952.3M base,
  149.6M head. Every run stdout-identical.

## Step 8: COPIES by doubling (`e4064dac0`, `86f84f2f1`)

* Change: `src/builtin/string.rs:730` writes the string once and then doubles with
  `extend_from_within`. The output buffer is sized once, as before.
* Test: `builtin::string::tests::copies_fills_every_count_exactly` (counts 2 to 33 of `abc`). The
  behaviour is unchanged by design, so it does not fail at base; the pin is the measurement below.
* Callgrind with libc included (total / libc / rest less ld, one run each, base against `e4064dac0`):
  loop999 1e5: 1470.2M / 779.4M / 690.5M against 509.3M / 235.7M / 273.3M. `copies('-', 80)` x1e6:
  4127.1M / 1734.4M / 2392.3M against 2116.1M / 638.4M / 1477.3M. `copies('abcdefgh', 2)` x3e5, the worst
  case: 592.6M against 593.5M (+0.15%).
* `86f84f2f1`: `tests/signals.rs` `sigint_halts_after_a_long_{builtin,method}` used
  `copies('ab', 30000000)` as the long builtin. It now takes about 3 ms, so the program ended before
  SIGINT (1 s after start). The workload is now `countstr('ab', copies('ab', 30000000))` and
  `'ab'~copies(30000000)~countStr('ab')`: 7.09 s and 7.03 s unsignalled for 30 passes with the
  test-profile `rexx-run`. Both tests passed 3 of 3.

## Step 9: checks, gate, perf

Per-task check at `849f3dfb0` (status file `ws6-status.txt`, each read unpiped):

* `cargo fmt --all --check`: 0
* `cargo clippy --workspace --all-targets -- -D warnings`: 0
* `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`: 0
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: 0, 3134 passed, 0 failed
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test ir_recorded_oracle`:
  0, 29 passed and 21 passed
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests
  whole_groups` (includes `sim_gate::the_seeded_gate` in release): 0, 16 passed

whole_groups lines rewritten (`98d0d4ccd`, `849f3dfb0`): CONSTANT whole now reaches the known
TEST_EXPRESSION_NOVALUE_ERROR failure, which the rest part has too (41.1 against the oracle's 98.986,
the same at base). Method whole stops at TEST_NEW_FOUR_ARGS. Method rest is as in Step 6b. Object rest
asserts 250. The first seeded-gate run at `0517efbfb` had 60 reds, all CONSTANT whole
TEST_EXPRESSION_NOVALUE_ERROR, which these lines settle.

Perf, callgrind against base61 (`cg17`, command in `phase-6-1-gate.md` `## Task 11a`, every spread
0.0000%):

| program | base % | t11a % |
|---|---:|---:|
| pingmsg | +0.0096 | -0.0000 |
| pingguard | -0.3663 | -0.3836 |
| pingsem | -0.2494 | -0.2664 |
| alloc | +0.2330 | +0.3362 |
| alloc4c | +0.0607 | +0.0606 |
| heapshape | +0.0525 | +0.0524 |
| rexxcps | +0.4409 | +0.4394 |
| emptyloop | -0.3191 | -0.3191 |

Wall clock, 5 interleaved runs (`wallclock.sh -r 5`), median against base61 for t11a, with base in
brackets:
rexxcps -1.27% (+0.15%), emptyloop +5.12% (+3.72%), alloc -2.44% (-2.32%), alloc4c -0.88% (-1.24%),
heapshape -1.62% (-1.62%), pingmsg +1.46% (+0.08%), pingsem -1.61% (+0.00%), pingguard +3.39%
(+5.08%). The 1-minute load average was 1.04 at the end of the run. emptyloop runs the same
instructions as base.

RESOLVED lines are added to `2026-10-01-time-elapsed-loop-hang`, `2026-10-02-halt-after-replied-send`,
`2026-10-02-exit-in-handler-at-reply`, `2026-10-09-wrong-type-native-rows-name-phase-9`,
`2026-10-09-literals-rows-need-a-test-case`, `2026-10-01-syntax-unwind-superlinear` and
`2026-10-10-routine-new-no-package-parent`.

## Concerns

* `run_repeating`'s clock store (Step 1) is not reached by any probe: no repeating loop takes the
  nested path. It is kept so that the two loop drivers agree.
* `say` over a STRING answer with no string value refuses rather than giving the oracle's 88.909,
  whose traceback runs through REXX-package frames this crate does not build (Deviation 30).
* Wall clock: emptyloop is +5.12% and pingguard +3.39% against base61. Both have flat instruction
  counts, and base shows the same pattern (+3.72% and +5.08%).
* The Routine~new caller-routine lookup, deferred to Task 12 by an earlier ruling, was fixed here as
  Step 6b by a later ruling. Its probes `probes/s6/ctx2.rex` and `ctx3.rex` are left in place.

## Fix round 1

Brief `task-11a-fix1-brief.md`, review `task-11a-review.md`. Head `2021f173f`. Probes:
`/tmp/claude-1000/p61/t11ar/probes/` (reviewer), `/tmp/claude-1000/p61/t11a/probes/{f1,f1c,f1d}/` (mine).
`N/N` is matching runs per engine, through `tools/both.sh`.

### I1: a `::CLASS` superclass resolves through the installing package (`8af5344dd`)

* Change: `src/environment.rs` `directive_class` is `findClass` on the installing package
  (`ClassResolver::lookup`, `ClassDirective.cpp:185`). It checks `installing`'s installed classes,
  then `parent_installed_class`, then `installing`'s merged public classes, then
  `parent_public_class`. Only after those does `.environment` come, as before. It used to read the
  running program's installed classes and walk no parent.
* The one site covers all three routes, and each has a witness that fails at `849f3dfb0` and
  matches 3/3 now:
  * `corpus/lang/routine_new_directive_subclass.rex` (reviewer `a4/routsub.rex`);
  * `package_new_directive_subclass.rex` (`a6/pkgsub.rex`);
  * `external_directive_subclass.rex`, with `external_directive_subclass.d/lib2.rex` (`a8`): at
    `849f3dfb0` it prints `loaded`, and the oracle and now the crate give `error 98.909`.
* The reviewer's `a1/shadow`, `a2/req` and `a3/pkgctx` still match 3/3.

### I2: a test for the parent's required public classes (`9bb5f3603`)

* Witness: `corpus/lang/routine_new_parent_required.rex` with fixture `.d/lib.rex`, which is the
  reviewer's `a2/req.rex`. Oracle and crate print the same four lines (1/1 by hand; 3/3 in the
  review's run of `req.rex`).
* Mutation: M2 (`if false && self.has_package_parent(program)` in `imported_class`), run as
  `REXX_CORPUS_GATE=1 ... --test corpus corpus_differential`. It goes red: `[UNCLASSIFIED]
  lang/routine_new_parent_required.rex: stdout, stderr, exit code differ`. The file was restored
  from a copy, and `git diff` was empty afterwards.

### I4: a trapped NOSTRING comes before the refusal (`807913226`)

* Change: `src/dispatch/reqstr.rs` takes a STRING answer with no string value through the NOSTRING
  checks first, with "The NIL object" as the readable. It refuses (bytes consumer) only after them.
* Witness: `corpus/lang/string_answer_nostring_trapped.rex` (reviewer `b3/ns_sigbytes.rex`) prints
  `h NOSTRING [The NIL object] 3`, 2/2. At `849f3dfb0` it gives rc 120. `ns_sig.rex` still matches
  2/2. `ns_call.rex` (`call on nostring`) differs on the `&1` insert, the reviewer's pre-existing
  item.

### I3: `.nil` wherever the oracle's answer is defined (`438f90b9e`, `2021f173f`)

* Change: the sites marked `.nil-set` below call `required_string_or_nil`. `apply_binary` passes the
  operator into the cold path (`required_string_operand`): every operator except concatenation is
  in the `.nil` set. After the conversion, `.nil` gets the equality rule (`primitiveIsEqual`). Where
  the oracle's error names the operand, the crate keeps naming the original object: `numeric_operand`
  keeps the original text when the conversion is `.nil`, and the semaphore timeout names
  `string_value_text` of the original. The other sites already rendered the original for the
  error. Deviation 30 and R11's line in the spec are corrected.
* Witnesses, each failing at `849f3dfb0` (rc 120) and matching the oracle 2/2:
  * `corpus/lang/string_answer_nil_consumers.rex`: SIGNAL VALUE, stem tail, arithmetic, `**`,
    DIGITS, FUZZ, DO count, DO FOR, FORM VALUE, semaphore timeout, OPTIONS, equality, ordering,
    PARSE VALUE, PARSE ARG, a host command and INTERPRET;
  * `string_answer_nil_native.rex`, with a `.env` sidecar for the oracle's libraries: native int,
    double, size, stem and ObjectToString.
* `2021f173f` adds that sidecar. Without it the workspace run (`438f90b9e`) failed
  `collect_stress` (the zero-collection list), `ir_recorded` (98.903 loading orxmethod) and the
  corpus gate on this one program.
* Enumeration command, from `rust/crates/rexx-exec/src`:
  `grep -rn "required_string_value(\|required_string_arguments(\|required_string_or_nil(\|required_string_operand(" --include=*.rs . | grep -v "fn required_string\|/tests\|tests.rs"`.
  Each site is classified below. "Probe" names the file. Oracle answers are 2/2 unless noted.

| site | consumer | class | oracle | probe |
|---|---|---|---|---|
| `install.rs:715` | security manager's REQUIRES `NAME` answer | n/a | the oracle never consults the manager here: the control (`NAME` a string) also differs, 43.901 against `in f` | `f1d/main.rex`, `main2.rex` |
| `redirect.rs:630` | ADDRESS ... WITH stream name | refused-garbage | 98.920 on the file "" | `f1c/c07_redirect` |
| `stem.rs:146` | compound tail | .nil-set | tail "The NIL object" | `b/b14_tail` |
| `run.rs:622` | `address (o) cmd` | refused-garbage | Error 5 | `f1c/c01_addrdyn` |
| `run.rs:751` | INTERPRET | .nil-set | interprets "The NIL object" | `b/b09_interp` |
| `run.rs:1031` | `call (o)` | refused-garbage | Error 5 | `f1c/c02_calldyn` |
| `run.rs:1201` | OPTIONS | .nil-set | rc 0 | `f1c/c03_options` |
| `run.rs:1458` | `expose (v)` | refused-garbage | SIGSEGV rc 139 | `f1c/c11_expose` |
| `run.rs:1595` | `procedure expose (v)` | refused-garbage | SIGSEGV rc 139 | `f1c/c05_procexpose` |
| `run.rs:1964` | SAY | refused-88.909 | 88.909 through `Stream~SAY` | `b/b02_say` |
| `run.rs:2203` | FORWARD MESSAGE (expr) | refused-garbage | Error 5 | `f1c/c19_forward` |
| `run.rs:2552` | QUEUE/PUSH | refused-88.909 | 88.909 | `b/b11_queue` |
| `eval.rs:785` | arithmetic right operand | .nil-set | 41.1, 26.8 naming "a SN" | `b/b12_arith`, `f1/p_say_2____o_` |
| `eval.rs:1324` | `& \| &&`, comparisons | .nil-set | 34.901; 0/1 | `b/b38_xor`, `b/b26`-`b28`, `f1/c_*` |
| `eval.rs:1324` | concatenation (`\|\|`, abuttal, blank) | refused-garbage | Error 5 | `b/b04_concr` |
| `eval.rs:1484` | truth value | .nil-set | 34.x found "The NIL object" | `s4c/truth_*` (5/5) |
| `builtin.rs:725` | builtin arguments | refused-garbage | Error 5 (`value()`, `upper()`) | `b/b21_value`, `b/b39_upper` |
| `dispatch/reqstr.rs:321` | the same, through `required_string_arguments` | refused-garbage | as above | as above |
| `dispatch/object_protocol.rs:284` | `"abc"~"\|\|"(o)` | refused-garbage | Error 5 | `b/b20_strcat` |
| `run/condition.rs:1088` | SIGNAL VALUE | .nil-set | 16.1 "The NIL object" | `b/b01_signal` |
| `command.rs:895` | security manager's command `RC` answer | n/a | the oracle runs the command and does not consult the manager: the control (`RC` 5) also differs | `f1c/c16_secrc`, `c18_secrc5` |
| `command.rs:936` | host command | .nil-set | runs "The NIL object", RC 127 | `b/b15_cmd` |
| `run/settings.rs:109` | TRACE VALUE | refused-garbage | 24.1 found "?" | `b/b07_trace` |
| `run/settings.rs:264` | ADDRESS VALUE | refused-garbage | 29.1 | `b/b05_address` |
| `run/settings.rs:386` | NUMERIC FORM VALUE | .nil-set | 25.11 naming "a SN" | `f1c/c06_form` |
| `run/settings.rs:426` | NUMERIC DIGITS, FUZZ | .nil-set | 26.5, 26.6 naming "a SN" | `b/b06_numeric`, `f1/p_numeric_fuzz_o_` |
| `parse_template.rs:445` | PARSE ARG | .nil-set | "The NIL object" | `f1/parsearg` |
| `parse_template.rs:631` | PARSE VALUE | .nil-set | "The NIL object" | `b/b10_parse` |
| `dispatch/semaphore.rs:197` | semaphore timeout | .nil-set | 88.902 naming "a SN" | `f1c/c08_sem` |
| `run/interpret.rs:155` | `drop (v)` | refused-garbage | SIGSEGV rc 139 | `b/b08_drop` |
| `run/loops.rs:685` | DO ... FOR | .nil-set | 26.3 naming "a SN" | `f1/p_do_i___1_for_o__end_` |
| `run/loops.rs:695` | DO count | .nil-set | 26.2 naming "a SN" | `b/b13_docount` |
| `dispatch/library.rs:1503` | native logical argument | .nil-set | 34.901 found "The NIL object" | `s4c/found_nil` (5/5) |
| `dispatch/library.rs:1733` | native numeric and stem arguments | .nil-set | 88.907, 88.921, 93.969 naming "a SN" | `f1c/c15_*` |
| `dispatch/library/surface.rs:149` | API ObjectToString | .nil-set | `.nil` | `f1c/c14_objtostr` |

The report's earlier sentence in Step 4, "Every other site reads bytes", is false. The table
replaces it.

### Checks at `438f90b9e`, and `2021f173f` for the three it failed

* `cargo fmt --all --check`: 0
* `cargo clippy --workspace --all-targets -- -D warnings`: 0
* `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`: 0
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast` at `438f90b9e`: 3132 passed, 2 failed
  (`collect_stress` and `ir_recorded`, both on the missing sidecar). At `2021f173f`,
  `cargo test -p rexx-exec --test collect_stress` passes 37 and `--test ir_recorded` passes 28.
* `REXX_CORPUS_GATE=1 ... --test corpus --test ir_recorded_oracle` at `2021f173f`: 29 passed and 28
  passed (`f1-c2.txt`). At `438f90b9e` the result was 28/29, the same program.
* `REXX_CORPUS_GATE=1 ... --release --test concurrency_tests whole_groups`, which includes the seeded
  gate: 0, 16 passed.

### Perf

`callgrind.sh -r 2 -j 3 -p "pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop"`
against base61, t11a = `438f90b9e` (`cg19`, every spread 0.0000%):

| program | t11a % |
|---:|---:|
| pingmsg | +0.0001 |
| pingguard | -0.3835 |
| pingsem | -0.2662 |
| alloc | +0.0118 |
| alloc4c | +0.0607 |
| heapshape | +0.0520 |
| rexxcps | +0.4267 |
| emptyloop | -0.3191 |

`2021f173f` adds only a sidecar file.

### Concerns

* Pre-existing, outside the task: with `.context~package~setSecurityManager`, the oracle consults
  the manager neither for a host command's `RC` nor for an external call's `::REQUIRES`, and the
  crate does both. The controls differ: `c18_secrc5` (`RC` 5) and `f1d/main2.rex`.
* Oracle SIGSEGV shapes, not in `oracle-crashes.txt`: `expose (v)` (`c11_expose`) and `procedure
  expose (v)` (`c05_procexpose`) with `v` an object whose STRING answers a Directory. They join the
  reviewer's `drop (v)`.
