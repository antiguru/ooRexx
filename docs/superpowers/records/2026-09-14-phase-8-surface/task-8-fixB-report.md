# Task 8 fix round B -- report

Implementer: fix-s8b. BASE `7f220b983`. Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/fix-s8b/`
(below, `$B`). Written first, filled as the work goes.

## Status

Code committed at `a9fd1a7ec`; gates green.

## I6 members

### METHOD.TESTCLASS01 -- Class~new

**Cause.** `.Class`'s one class method, `NEW` (`Setup.cpp:450`, `RexxClass::newRexx`,
`ClassClass.cpp:1776`), had a row whose body checked its arguments and refused
(`construct::native_class_new`, "builds no class"). The oracle clones the receiver: the clone's
class behaviour is a copy of the receiver's instance behaviour (so it answers `ID`, `SUBCLASS`,
`DEFINE` and no `NEW`), its instances derive from `Object` alone, no subclass list holds it, its
metaclass is `.Class` for a primitive receiver and the receiver otherwise, its package is the
receiver's, and `INIT` is sent with the arguments after the id.

**Fix.** `ClassGraph::define_cloned_class` (and the registry's) builds exactly that class;
`native_new_class` (class_protocol.rs) takes the row, answers the oracle's 93.901 and 88.901, copies
the receiver's package entry and runs the subclass factory's uninit steps and `INIT`. "Primitive" is
a class no program's install recorded a package for (`is_primitive_class`). The old refusing body
is gone.

**Witness** `class_new.rex` (`phase-8.txt`, sourceline file): the rec1 probe and every reader of the
clone, a subclass of it and its instance, a Rexx metaclass's `new` (its `INIT`, class, metaclass,
package), identity operators, `c~new` 97.1, the missing and omitted id, `INIT` refusing the extra
arguments, and ids from an Array and a number. `7f220b983`'s binary refuses at rc 120.

**Left, pre-existing and recorded before this round**: `~define` of a Method built from source text
refuses naming Phase 5 (exclusions, Task 2's items), so the witness defines nothing on the clone.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| K1 | `class_graph.rs` `define_cloned_class`: the class behaviour left empty (no copy of the receiver's instance behaviour) | corpus red on `class_new.rex` alone; instrument red, `newly failing: ["METHOD.TESTCLASS01"]` alone | as predicted: 639 of 640, `class_new.rex` (all three descriptors); instrument `newly failing: ["METHOD.TESTCLASS01"]`, `newly passing: []`. Restored |
| K2 | `class_protocol.rs` `native_new_class`: the metaclass always `.Class` | corpus red on `class_new.rex` alone, its `Viam` line (`META META` becoming `META Class`) | as predicted: 639 of 640, the `Viam` line (`META Class`, checked on a mutated build). Restored |
| K3 | `class_protocol.rs` `native_new_class`: the receiver's package not copied | corpus red on `class_new.rex` alone, stopping at the `Viam` line (`.nil~name`, 97.1) | **falsified in its mechanism**: 639 of 640, stdout only; a class with no package entry answers the REXX package, not `.nil`, so the line reads `REXX` for `class_new.rex` and nothing is raised (checked on a mutated build). Restored |
| K4 | `class_graph.rs` `define_cloned_class`: the clone pushed into `Object`'s subclass list | corpus red on `class_new.rex` alone, its first line (`0` becoming `1`) | as predicted: 639 of 640, the first line (`FooBar Object 1`, checked on a mutated build). Restored |

### EXPOSE of a single compound tail (TESTGETOBJECTVARIABLE01/02/03, TESTSETOBJECTVARIABLE01, TESTDROPOBJECTVARIABLE01)

**Cause.** `VariableTester~init` exposes `stem2.1`; `bind_exposed` refused a compound name
(`Loud::compound_expose`), as did `PROCEDURE EXPOSE`. The oracle makes the tail in the object's
stem (`StemClass::exposeCompoundVariable`, which gives it the stem's assigned default), makes the
local stem, and puts an element in it whose `realElement` is the object's
(`RexxCompoundVariable::expose`, `StemClass::expose`). Reads and writes through the variable,
`[]`, `[]=`, `hasIndex`, `remove` and the element accessors follow `realVariable`; the walks
(`items`, `allIndexes`, `supplier`, `copy`, `tailArray`) see the element's own value, which is
none; `empty` drops it.

**Fix.** `Body::Stem` gains `exposed`, the tails an EXPOSE made another stem's, each naming that
stem; the tail also sits in `tails` as a tombstone so it keeps its place and holds no value of its
own. `Body` stays at its size (measured: over 72 bytes before and after, under the existing 80
assertion). `expose_tail` (stem.rs) does both halves; `EXPOSE` finds or makes the object's stem in
the pool, `PROCEDURE EXPOSE` resolves the tail and finds the stem in the caller before the callee's
frame exists (`exposeLocalCompoundVariable`), and both trace `>C>`. The variable read, write and
drop, the `Stem` methods that follow `realVariable`, the redirection's element accessors, `empty`
and `copy` read `exposed`; the read path's cost is one test of a `None` on the stem already in
hand. `Loud::compound_expose` is gone with its refusal-sites row (re-derived; column 4 blanked, the
only difference is that row).

**Witnesses** (`phase-8.txt`, sourceline files): `expose_compound_tail.rex` (defaults, the indirect
list, `expose a. a.1`, tail order and ordinals, `[]`, `hasIndex`, `hasItem`, `index`, `copy`,
`remove`, `[]=`, `symbol`/`value`/`var`, a drop, a stem replaced after it, PROCEDURE EXPOSE
through a method's exposed tail and over the caller's stem, `procedure expose i a.i`, a local
`empty`, the object's `allIndexes` order after a tail was made by an EXPOSE),
`expose_compound_tail_trace.rex` (the `>C>` lines), `expose_compound_tail_redirect.rex` (`ADDRESS
WITH INPUT STEM` and `OUTPUT STEM`, replace and append). `7f220b983`'s binary refuses each at rc 120.

**Left**: `~empty` on an object's stem while a method's stem exposes one of its tails reads the
object's stem afresh here where the oracle's method stem keeps the old element (`$B/p/ex3.rex`);
recorded, owner Phase 9. Replacing the object's stem agrees.


| # | Mutation | Prediction | Result |
|---|---|---|---|
| E1 | `stem.rs` `stem_get_at`: the exposed test removed (reads the local tombstone) | corpus red on `expose_compound_tail.rex` and `expose_compound_tail_redirect.rex` (its `run` line) and on nothing else; instrument green (`init` writes through, and the tests read the object's stem) | as predicted at the file level: 637 of 639, `expose_compound_tail.rex` and `expose_compound_tail_redirect.rex`; instrument green. **The redirect detail was wrong**: its differing line is `run2` (`OUT.2` for `z`), not `run`, because the REPLACE in `run` drops the exposure before `run` reads (checked on a binary built with the mutation). Restored |
| E2 | `stem.rs` `stem_set_at`: the exposed test removed (writes stay local) | corpus red on `expose_compound_tail.rex` and `expose_compound_tail_redirect.rex`; instrument red with `newly failing: ["METHOD.TESTGETOBJECTVARIABLE01"]` alone | as predicted: 637 of 639, both files; instrument `newly failing: ["METHOD.TESTGETOBJECTVARIABLE01"]`, `newly passing: []`. Restored |
| E3 | `stem.rs` `expose_tail`: the tail not made in the object's stem | corpus red on `expose_compound_tail.rex` alone, its `order` line | **falsified**: 639 of 639. The `order` line does not see it: inserting 3 before 1, 5, 2, 4 and after them builds the same balanced tree. Added a witness case the creation does show (a defaulted object stem whose tails a method exposes: the object's `items` counts them, oracle `count 3 1,9,2`), then E3b |
| E4 | `run.rs` `exec_procedure`: the collected tails never exposed | corpus red on `expose_compound_tail.rex` and `expose_compound_tail_trace.rex` (the procedure's `>C>` line) | as predicted: 637 of 639, `expose_compound_tail.rex` (stdout) and `expose_compound_tail_trace.rex` (stderr). Restored |
| E5 | `dispatch/hash/stem.rs` `native_stem_empty`: `exposed` kept | corpus red on `expose_compound_tail.rex` alone, its `locempty`/`after` lines | as predicted: 638 of 639, `expose_compound_tail.rex`, its `locempty` and `after` lines (checked on a mutated build). Restored |
| E6 | `dispatch/object_protocol.rs` `native_copy`: `exposed` kept on the copy | corpus red on `expose_compound_tail.rex` alone, its `copy` line | as predicted: 638 of 639, `expose_compound_tail.rex`, its `copy` line (`copy 3 e 1`, checked on a mutated build). Restored |
| E3b | E3 again, over the witness with the defaulted stem (the prediction was made before running but written here after, a lapse) | corpus red on `expose_compound_tail.rex` alone, its `count` line | as predicted: 638 of 639, the `count` line (`count 1 9` for `count 3 1,9,2`). Restored |

### Package tables and Routine~new over directives (TESTGETPACKAGECLASSES01, ...PUBLICCLASSES01, ...PUBLICROUTINES01, ...ROUTINES01)

**Cause.** Two refusals, each test hitting both. `Package~classes`, `~publicClasses`, `~routines`,
`~publicRoutines` and the other readers `string_table_of` serves answered a `Body::Native`
StringTable, which refuses `ITEMS`, `HASINDEX`, `ALLINDEXES`, `SUPPLIER` and `MAKEARRAY` (the phase
5i register's row 16); the oracle answers a copy of the installed `StringTable`
(`PackageClass::getClassesRexx`, `classes/PackageClass.cpp:1542`). And `Routine~new` refused a
source carrying a directive, where `LanguageParser::generateRoutine` (`parser/LanguageParser.cpp:624`)
installs the directives into the routine's own package, parent the caller's, with no prologue.

**Fix.** `string_table_of` builds a store-backed StringTable (`hash::new_string_table`, the store
`.StringTable~new` gives, filled through `store_insert` in name order). `new_file_executable`'s
second half is now `install_executable`, which `compile_routine_source` also takes for a source with
directives, recording the package under the routine's name as the no-directive path does. The
cached `.METHODS`/`.ROUTINES`/`.RESOURCES` tables are untouched (still `Body::Native`, row 16's).

**Witness** `package_tables_routine_source.rex` (`phase-8.txt`, sourceline file): the rec3 and rec4
probes, every table the four tests read and what they ask of it, a class of the new package
instantiated, a routine of it resolving its caller's `::ROUTINE`, `allIndexes`, `makeArray`, `DO
OVER`. `7f220b983`'s binary refuses at rc 120.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| P1 | `environment.rs` `string_table_of`: a `Body::Native` table again (`native_instance`, `set_entry`) | corpus red on `package_tables_routine_source.rex` alone, loud at its first line (rc 120, `ITEMS`); instrument `newly failing` the four tests | as predicted: 640 of 641, loud `ITEMS` on the witness; instrument `newly failing` the four. Restored |
| P2 | `class_protocol.rs` `compile_routine_source`: the directive branch refuses again | corpus red on the witness alone, loud at its third line; instrument `newly failing` the four tests | as predicted (the first run's mutation did not apply after `cargo fmt` rejoined the line; re-run with the formatted text): 640 of 641, loud `a routine source that carries a directive`; instrument `newly failing` the four. Restored |
| P3 | `class_protocol.rs` `compile_routine_source`: no parent for the new package | corpus red on the witness alone, at its `helper b` line (43.1, rc 213 or thereabouts, stderr) | as predicted: 640 of 641; on a mutated build, `Error 43.1 Could not find routine "HELPER"`, rc 213, at the `helper b` line. Restored |

### FUNCTION.TEST_INPUT_OUTPUT_STREAM -- the stream-object input reader

**Cause.** `read_stream_lines` sent `LINEIN` and then `STATE`, and `ArrayInputStream` has no
`STATE` (97.1). The oracle sends `LINEIN` alone, under a `RedirectionDispatcher`
(`instructions/InputRedirector.cpp:62`, `:313`): any condition the send raises ends the input and
reaches no trap of the issuing activation; a SYNTAX condition is handed to the command's callout and
raised once the command completes. Inside the per-test run (no `io` handler) the oracle fails the
test with RC(30); the reader now leaves the crate the same failure, and the whole-group run, where
FUNCTION's `TEST` registers the handler, passes on both.

**Fix.** `input_dispatch` names the activation the reader sends from; a `RAISE ... RETURN` whose
caller is that activation, and a `raise_notready` running in it (a native `LINEIN`), are taken by
the dispatcher (`input_dispatcher_takes`), which ends the input; a SYNTAX failure from the send is
kept (`input_dispatch_syntax`) and raised after the command completes, re-raised
(`reraise_failure_levels`). `STATE` is no longer sent.

Found on the way and fixed with it, because the kept SYNTAX needed it: a re-raised condition's
`POSITION` and report line are the re-raising level's (`Activity::reraiseException`). The earlier
rule (SYNTAX and not a boundary raise) mis-classed a failure raised by a native argument's
conversion, which the oracle does not re-raise (`library_string_argument.rex` went red); an
extension's re-raise is now the condition it raised while running (`native_reraise`, set where a
held or pending condition becomes the call's failure), and `reraise_failure_levels` makes the left
levels' lines traceback-only. The Task 5 row's untrapped `sendthrow` line now agrees
(`$B/p/sendthrow2.rex`).

**Witnesses** (`phase-8.txt`, `.env`, sourceline files): `input_stream_object_reader.rex` (blank
line kept, `CALL ON NOTREADY` not run, a USER raise ending the input, one raised a level deeper not
ending it, the `io` handler, a SYNTAX kept past the command, trapped and untrapped),
`native_reraise_position.rex` (orxmethod's `TestSendMessage0` over a failing method, trapped and
untrapped). `7f220b983`'s binary differs on both.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| S3 | `redirect.rs` `read_stream_lines`: a SYNTAX from the send propagates at once | corpus red on `input_stream_object_reader.rex` alone, its `trapped` line (`out~items` 0) and the untrapped report | as predicted: 642 of 643, `input_stream_object_reader.rex` (stdout and stderr). Restored |
| S4 | `run.rs` `reraise_failure_levels`: the origin reset kept, the lines not made traceback-only | corpus red on `native_reraise_position.rex` (report `line 19`) and `input_stream_object_reader.rex` (report `line 70`), stderr only | as predicted: 641 of 643, both witnesses, stderr only. Restored |
| S5 | `run/condition.rs` `exec_raise`: the dispatcher test on the RETURN path removed | corpus red on `input_stream_object_reader.rex` alone, which does not finish (the reader never stops) | **falsified, and the mutation was not the one described**: `input_dispatcher_takes(caller) && false` still evaluates the take (which sets the flag) and only drops the early return, so the reader still stops; 642 of 643, stdout only, the one difference being `notready handler ran` (checked on a mutated build) -- which is what the early return guards: the issuing activation's `CALL ON` never sees the condition. The mutation predicted here (no take at all) would leave the reader looping and was not run |
| S6 | `dispatch/library.rs` `raise_held_condition`: `native_reraise` not set | corpus red on `native_reraise_position.rex` alone (POSITION absent, report line 19) | as predicted: 642 of 643, `native_reraise_position.rex` (stdout and stderr). Restored |

## I2 -- a trapped SYNTAX describes the raising activation

**Cause.** A trapped condition's object was built at the trap from the stack as it stood then, so
every level the failure had already left was missing: `STACKFRAMES` and `TRACEBACK` started at the
trapping activation, `POSITION` was its line and `PROGRAM` the main program's path
(`self.program_path`). The oracle builds the object at the raise
(`Activity::generateProgramInformation`, `concurrency/Activity.cpp:1080`): every frame, `POSITION`
and `PROGRAM`/`PACKAGE` from the first frame that has a package, and a native call re-raises a
SYNTAX condition in its caller with the caller's `POSITION` and program
(`NativeActivation::checkConditions` `:1787`, `Activity::reraiseException` `:1330`).

**Fix.** Each level a failure leaves now contributes its `StackFrame` beside the traceback line it
already contributed (`failure_frame`/`failure_frames` beside `failure_site`/`failure_sites`, sealed
together): an activation's is built before it is popped (`capture_activation_frame`, the method
send, the resumed reply and `invoke_call`), an `INTERPRET` fragment's is the interpret frame
(`capture_fragment_frame`, `build_interpret_frame`), and a native level's is built where it is
blamed (`build_native_level_frame`; a routine's context is the caller's, a method's `.nil`,
measured). `failure_origin` keeps the first level with a package and its line; a native level
with a package that did not re-raise is that origin with no line, so `POSITION` is absent
(measured: `EXTERNAL` routine and method boundary raises, `filespec()` naming `REXX`); a
re-raised SYNTAX clears it. `offer_to_trap` hands all of it to `build_trapped_condition_object`,
which puts the unwound frames and lines first. An internal call a tail-less `RAISE` left reports no
arguments, as the oracle's does (measured, `args2.rex`).

Found on the way and fixed with it, because it is the `79 *-*`/`69 *-* else` line of
`TEST_BUFFERED_INPUT`'s report: an external program's run left the callee's clause state behind,
so a prologue its `::REQUIRES` ran rewrote the line its caller's `StackFrame` reports, and
`.context~line` after the call returned read the callee's last line. `run_loaded` now restores the
caller's clause when it ends.

The seven `DETAIL_DIFFERS` members then matched: `REXX_CORPUS_GATE=1 cargo test --release -p
rexx-exec --test api_group_tests` went red with `detail no longer differing` naming six of them
(`$B/api1.log`), and after the clause-state fix the seventh (`FUNCTION.TEST_BUFFERED_INPUT`,
`$B/t1.sh`). `DETAIL_DIFFERS` is now empty and the instrument green (`$B/api2.log`, 60.24 s),
including `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` back in the whole-group run.

**Witnesses** (`phase-8.txt`, sourceline files): `trapped_condition_raising_frames.rex` (a method
error trapped by its caller, a `RAISE SYNTAX` through internal calls, directly and through
`INTERPRET`, a `Message~send` native frame with an interpret frame, a trap after its own
activation's fragment), `trapped_condition_raising_program.rex` with `.d/` (a required package's
error names that package; an external call's caller frame and `.context~line` across a
prologue), `trapped_condition_native_levels.rex` with the `.env` (`SUBSTR`, `SysSleep`, `filespec()`,
`EXTERNAL` routine and method boundary raises, a SEND with a Rexx method below it). All three agree
on three descriptors and differ from `7f220b983`'s binary on stdout.

**Records.** Removed: the `DETAIL_DIFFERS` block and B12 ("THE CONDITION OBJECT A TRAP READS NAMES
THE RUNNING PROGRAM FOR A BOUNDARY RAISE"; re-run as `b12.rex` and a plain-method `b12b.rex`, both
identical now). Narrowed: the Task 5 rows on the re-raised native condition (native frame now
agrees, PROPAGATED does not; `cond3.rex`, `sendthrow.rex` re-run against the forge built by
`build.sh` into `$B/forge`) and `RaiseCondition` (SIGNAL ON now agrees, CALL ON does not;
`rcond.rex`); the compiled-source row's item count. Added: "WHAT A TRAPPED CONDITION STILL DOES NOT
DESCRIBE", owner Phase 9 (an `INTERPRET` fragment still running pushes no frame, the phase 5i
register's row 6; an external program called as a routine is a PROGRAM frame here; a failure in
an external program's directive install), each measured (`$B/p/sf5.rex`, `$B/p/ext`,
`$B/p/inst`).

`collect_stress.rs`: the programs that ended untrapped through a native level or a called
activation now allocate the frame as that level ends, so they left `NO_ALLOCATION_PROGRAMS`
(the assertion printed exactly those; none diverged under the stress mode).

### Controls (predictions written 2026-09-28 before any ran)

Each mutation is applied to a copy-checked file, the binary rebuilt, and
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` run; the file is restored and
`cmp`-checked after.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| D1 | `run.rs` `capture_activation_frame` returns at once | corpus red on `trapped_condition_raising_frames.rex` (the RAISER, B4/A4, B6/A6, N frames missing), `trapped_condition_raising_program.rex` (`main.rex` for `raiser.cls`, position 4 kept only by chance of the same line: predicted red on the program name) and `trapped_condition_native_levels.rex` (t6: INNER missing, position 40); any other red is a program reading a trapped condition's frames after an unwind. Instrument: `detail newly differing` holds at least `FUNCTION.TEST_BUFFERING` | as predicted: 633 of 636, the three witnesses (stdout); instrument `detail newly differing` all seven former members. Restored, `cmp` clean |
| D2 | `run.rs` `capture_fragment_frame` returns at once | corpus red on `trapped_condition_raising_frames.rex` alone (the INTERPRET frames of r4, viaSend, inPlace) | as predicted: 635 of 636, `trapped_condition_raising_frames.rex`; restored |
| D3 | `dispatch.rs` `blame_native_level`: no frame built (`failure_frame` left unset) | corpus red on `trapped_condition_raising_frames.rex` (SEND frame) and `trapped_condition_native_levels.rex` (every case's native frame) alone | as predicted: 634 of 636, `trapped_condition_raising_frames.rex` and `trapped_condition_native_levels.rex`; restored |
| D4 | `dispatch.rs` `blame_native_level`: the re-raise arm removed, so a re-raised SYNTAX keeps the native package as its origin | corpus red on `trapped_condition_native_levels.rex` alone, its t2 line (`88.902 The NIL object 0 REXX` for `88.902 20 1 ...`) | as predicted: 635 of 636, `trapped_condition_native_levels.rex`; its one differing line is t2's, `88.902 The NIL object 0 REXX 3` (checked on a binary built with the mutation); restored |
| D5 | `lib.rs` `run_loaded`: the caller's clause not restored after `install_directives` | corpus red on `trapped_condition_raising_program.rex` alone, the callee's caller line | **falsified**: 636 of 636. That restore turned out to be redundant: the nested `run_loaded` a prologue runs in restores its own caller's clause at its end, so the witness cannot see the first one. Removed from the code, which keeps the end restore alone; D5b tests it |
| D6 | `run.rs` `capture_activation_frame`: `arguments` always true | corpus red on `trapped_condition_raising_frames.rex` alone, the A6 frame's argument count | as predicted: 635 of 636, `trapped_condition_raising_frames.rex`; restored |
| D5b | `lib.rs` `run_loaded`: the caller's clause not restored at its end (the prediction was made before running but written into this file after, a lapse) | corpus red on `trapped_condition_raising_program.rex` alone (the caller frame's line and `.context~line`) | as predicted: 635 of 636, `trapped_condition_raising_program.rex`; restored |


## DETAIL_DIFFERS

Emptied by the I2 commit; every former member's detail agrees (`$B/api2.log`, then each later
instrument run). No member had another cause: six needed the raising levels' frames, and
`TEST_BUFFERED_INPUT` needed those and the caller-clause restore after an external program's run.
None was kept for a licence or a timestamp.

## m4 -- refusal texts naming Phase 5

The review names the refusals behind rec1 and rec3-rec6. rec5 and rec6 went with round A.
rec1's (`Class NEW`, `construct::native_class_new`) and rec4's (`a routine source that carries a
directive`) went with their fixes above. rec4's sibling, `a method source that carries a directive
is not implemented (Phase 5)`, goes too: `Method~new` over directives takes `install_executable`
as `Routine~new` does (`generateMethod`), witness `method_source_directives.rex`; the
`run/tests/directives.rs` case asserting its text is removed, the oracle and this crate both
running `.k~define("m", ('return 1', '::class zz'))` at rc 0.

rec3's text is the generic `Loud::native_method` ("method X of class Y ... (Phase 5)"), which
`.METHODS`, `.ROUTINES` and `.RESOURCES` (cached `Body::Native` tables, the phase 5i register's
row 16) still reach on `ITEMS` and its siblings, and which every unimplemented native method
shares. Raised with the lead; see Concerns.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| M1 | `class_protocol.rs` `compile_method_source`: the directive branch refuses again | corpus red on `method_source_directives.rex` alone, loud at its first line | as predicted: 643 of 644, loud on the witness. Restored |

## Roadmap row 8

One sentence added at `7b694a0e4`: `FUNCTION`'s `TEST_REXXQUEUE` is row 10's, its `RexxQueue`
living in rxapi. Nothing else in the roadmap changed. The same commit rewrites the exclusions
block's header, which said the run was `-V 0` and that no member was Phase 8's.

## Final RECORDED and DETAIL_DIFFERS

`RECORDED`: `FUNCTION.TEST_REXXQUEUE` (Phase 10). `DETAIL_DIFFERS`: empty. Queued files removed:
`class-new`, `expose-compound-tail`, `package-tables-and-directive-sources`,
`input-stream-object-reader`; `rexxqueue-in-function-group` stays.

## Performance

Callgrind `Collected`, each binary run the same way from a fresh directory by `$B/cg/run.sh NAME
BINARY PROGRAM` (`valgrind --tool=callgrind`), BASE = `7f220b983` (`$B/rexx-run-BASE`), final =
`7b694a0e4` (`$B/rexx-run-FINAL`):

| program | BASE | final | change |
|---|---|---|---|
| `rexxcps.rex` (`rust/bench-rexxcps`) | 19,440,602,071 | 19,450,930,078 | +0.05% |
| the reviewer's trap loop (`review-s8/cg/t.rex`: 5000 SYNTAX errors trapped in the raising activation) | 549,960,141 | 550,736,911 | +0.14% |
| the same trapped one level up (`$B/cg/tu.rex`) | 1,076,130,219 | 1,176,823,634 | +9.36% |
| `compound.rex` at 200000 passes (`$B/cg/compound.rex`) | 505,760,299 | 506,857,744 | +0.22% |

The unwound loop pays for one `StackFrame` per level left, which is the object the oracle builds at
the raise. No per-clause work was added: the changes on hot paths are a test of a `None` on a stem
already in hand (compound read, write, drop) and, on an activation's exit, a branch on the result.

## Commits

* `61381d68a` Give a trapped condition the frames of the levels it left (I2, DETAIL_DIFFERS)
* `1287827e5` Expose a single compound tail (five METHOD tests)
* `5eab077dc` Build the class Class~new answers (TESTCLASS01)
* `5170f0290` Answer a package's tables as StringTables, and Routine~new over directives (four
  METHOD tests)
* `6f61d38dd` Read a redirection's input stream object as the oracle's dispatcher does
  (TEST_INPUT_OUTPUT_STREAM)
* `629d93089` Compile Method~new over a source with directives, as Routine~new does (m4)
* `7b694a0e4` Name TEST_REXXQUEUE as row 10's in roadmap row 8
* `a9fd1a7ec` Refresh introspection-arity.tsv for Class~new (found by the first gate run, below)

Before each commit: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
warnings` (own target dir), and `REXX_CORPUS_GATE=1 memcap 8G cargo test --release` over
`rexx-classes`, `rexx-core`, `rexx-exec`, `rexx-api`, `rexx-parse`, built first outside the cap
(an 8G-capped build was OOM-killed once, `$B/full-i2.log`'s first attempt): 0 failed each time,
1848 passed at the first (rexx-exec and rexx-api only), 2342 at the second, 2400 at the rest.
Refusal sites re-derived at each commit that moved a constructor; the only row change outside
column 4 is `compound_expose`'s removal.

## Gates

**First run, at `7b694a0e4`** (`$B/gates-r1/`): G4 and G6 exit 101, both on one test,
`introspection_arity::the_table_matches_the_three_sides`. `Class new class` now agrees where the
table recorded `send-differs`. It is a refresh, not a finding. My per-commit runs had not seen it:
the table's crate side runs the `rexx-run` binary under `rust/target`, and those runs were built in
`$B/tgt`, which left the old binary there. Refreshed with
`REXX_INTROSPECTION_ARITY_REFRESH=1`, one row changed, committed as `a9fd1a7ec`, and the whole gate
re-run.

**Second run, at `a9fd1a7ec`**, `$B/gates/status.txt`; the tree was not touched while it ran:

```
a9fd1a7ece6808ed6b96981ab399f004ee3a8b98
started 2026-09-28T18:14:22+02:00
load at start 5.25 7.00 8.62 6/2881 1993522
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 5.63 7.00 8.59 4/2852 1995511 2026-09-28T18:14:34+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 4.69 6.83 8.21 4/2854 2123751
G5 debug build (test --no-run) exit 0
load G6 4.69 6.83 8.21 5/2854 2123764 2026-09-28T18:20:09+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 5.52 8.15 8.57 4/2898 2252011
a9fd1a7ece6808ed6b96981ab399f004ee3a8b98
finished 2026-09-28T18:26:01+02:00
```

Summed `test result` lines: G4 2752 passed / 0 failed / 4 ignored, G6 2753 / 0 / 4. The corpus
reads `644 of 644 matching` in both, and
`every_test_of_the_phase_8_groups_matches_the_oracle_but_the_recorded ... ok` in both. G2's clippy
checked every workspace crate (`Checking rexx-exec` among them).

## Concerns

* **m4's generic text remains.** `Loud::native_method` ("method X of class Y is not implemented
  (Phase 5)") is what rec3's `ITEMS` came from; the Package readers no longer reach it, but
  `.METHODS`, `.ROUTINES` and `.RESOURCES` (cached `Body::Native` tables, the phase 5i register's
  row 16) still do, and every unimplemented native method shares the constructor. I left it and
  asked the lead whether to relabel it or make those three tables store-backed; no answer had
  arrived when the gates started.
* **Residual divergences recorded for Phase 9**: an `INTERPRET` fragment still running pushes no
  frame (`.context~stackframes` inside one, or a trap in an activation it called); an external
  program called as a routine is a PROGRAM frame; a trapped failure of an external program's
  directive install; `~empty` on an object's stem while a method's stem exposes one of its tails.
  Each is measured and in `phase-4-exclusions.txt`. None is a member of the three groups.
* **A condition trapped one level up costs +9.4%** on callgrind (one `StackFrame` per level left);
  the in-activation trap loop the reviewer measured is +0.14%.
* **Scope went past the listed members where the fix needed it**, each witnessed: the caller's clause
  across an external program's run, a native argument's conversion not being a re-raise (which
  also fixed the Task 5 row's untrapped `sendthrow` line), `Method~new` over directives (m4), and a
  SYNTAX from `LINEIN` kept until the command completes.
* **Control mishaps, reported in their rows**: D5, E3, K3 and S5 were falsified or ran a different
  mutation than described; the E3b and D5b predictions were made before running but written after.
* **Pre-existing, found, not changed**: `~define` of a Method built from source text refuses
  naming Phase 5 (recorded as Task 2's item, "NO OWNER"), which a `c~define` on a clone or on a
  `Method~new` over directives reaches; the `RaiseCondition` taken by `CALL ON` still carries
  `POSITION`, and `PROPAGATED` is still 0, both Task 5 rows owned by Phase 8.
