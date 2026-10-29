# Phase 9d Task 8.7b report: security, native API, frames and install

Status: DONE_WITH_CONCERNS. Base `504ff0743` (dispatch commit `5de63cee6` on top). Code `ffba1e60f`,
`127a59431`, `67e9a22e9`, `5053fbd9b`; records, closings and inventory rows `6d5a8dc11`, with this
report; the witness fixture rename and the workspace result in the commit after. The closing lines and inventory rows name `5053fbd9b`, the last code commit. Records:
`docs/superpowers/records/2026-10-18-phase-9d/t8.7b/` (`probes/<item>/` with each probe's oracle
`.o.*`, base `.b.*` and head `.h.*` outputs under `out/`, `probes/summary-1.txt` and `summary-2.txt`
the three-way verdicts, `suite/`, `checks-*.txt`, `callgrind-final.txt`).

## What changed, per item

**2026-10-13-class-named-drops-raised-condition.md (closed).** `Interp::class_named`
(`dispatch/library/surface.rs`) holds the lookup's condition for the native call (`held`) instead of
dropping it, as the `class_in_program` branch does. Red shape: `TestFindClass` installed by
`setMethod` (scope not a program class) with the package local's `UNKNOWN` raising 93.900: the
oracle reports 93.900 at position 9, base 91.999 at position 1 (`probes/class-named-c/c.rex`).
Witness `native_find_class_raised_condition.rex`. The routine form (`TestFindContextClass`) agreed
at base (`class-named-b`): the oracle reports 88.900 there.

**2026-10-10-security-manager-command-rc-requires.md, 2026-10-22-security-manager-set-mid-activation.md
and half 1 of 2026-10-13-security-manager-and-library-key.md (closed).** Rule, from the oracle's
source (rexxref `secman.xml` names the checkpoints and says nothing on when a manager takes effect):
`PackageClass::setSecurityManager` installs a new `SecurityManager` object
(`classes/PackageClass.hpp:156`); a `RexxActivation` copies its code's package's manager into its
settings when it is built (`RexxActivation.cpp:176`, `:315`), and an internal call or INTERPRET copies
its caller's settings. COMMAND, CALL, METHOD and STREAM ask that kept manager
(`getEffectiveSecurityManager`); a `::REQUIRES` is processed in the requiring package's own installing
activation (`RexxActivation.cpp:508`, `PackageManager.cpp:720`), so it asks that package's manager;
LOCAL and ENVIRONMENT ask the running code's package's manager as it is now
(`PackageClass::findClass`, `PackageClass.cpp:1135`). So the two sites the 11a table marked n/a were
both the mid-activation case: the oracle consults neither a command nor an external call's
`::REQUIRES` because the activation running them began before the manager was set, and the external
file's own package has none. Here: `install_security_manager` (`security.rs`) first gives every live
activation of the package (every activity's stack) the manager it began under
(`ActivationCold::security_manager`, rooted, carried by `inherited` to internal calls and handlers);
`effective_security_manager` reads it before the package's; `package_security_manager` serves
LOCAL/ENVIRONMENT (`environment.rs`); a `::REQUIRES` directive asks
`installing_security_manager(from)` (`install.rs`, `load_requires`'s `directive`). The untraced,
unmanaged path is unchanged (the `security_armed` gate). Witnesses
`security_manager_activation_boundary.rex` (internal call, INTERPRET, routine, method, a protected
method, a stem's UNKNOWN, a manager replaced mid-routine, LOCAL/ENVIRONMENT seeing it at once) and
`security_manager_requires_installing.rex` (an external call's `::REQUIRES` unasked under either
manager, `.Package~new` source unasked, `loadPackage` asked twice, a REQUIRES-given manager asking for
the loaded package's own `::REQUIRES`). `collect_stress`'s package-held-objects test (`x = .local`
after a mid-activation set, oracle `mgr local`) caught the first version reading LOCAL from the
activation; `67e9a22e9` fixed it.

**Half 2 of 2026-10-13-security-manager-and-library-key.md (closed).** `GlobalNames::REXXUTIL` is
`rexxutil` (`memory/GlobalNames.h:201`); `libraries.rs` now keys the internal package so. Measured:
no Rexx-visible observable differs (`probes/rexxutil-key/`: `findRoutine('SYSVERSION')~package~name`
REXX and `LIBRARY REXXUTIL` 98.903 agree at base and head); the key's only effect is which bucket it
heads, which moves the point the table expands at, and so the unload order of user libraries once
enough of their names collide. Test `libraries::tests::the_table_starts_with_the_internal_packages_keys`.

**2026-10-17-newmethod-parent-package.md and gate:phase-9c-gate.md:144 (closed).** Rule:
`NewMethod`/`NewRoutine` call `LanguageParser::createMethod(name, buffer)` and
`createRoutine(name, buffer, OREF_NULL)` (`api/ThreadContextStubs.cpp:529`, `:547`): no source
context, so the package has no parent; a method's buffer is never restored, a routine's is restored
or parsed. `compile_buffer` (`dispatch/library/surface.rs`) now builds every method and routine
directly through `method_in_package`/`routine_in_package` (`class_protocol.rs`) with no parent; the
`.Method~new` send fallback is left for package source only. Witnessed through the C API test
library (`orxmethod`'s `TestNewMethod`, `TestNewRoutine`): `native_new_executable_no_parent.rex`
(43.1 for a routine only the caller defines, `.T` unresolved, directives and source lines of both,
and `.Method~new`/`.Routine~new` still inheriting).

**2026-10-22-live-frames-omit-nested-primitives.md, 2026-10-22-directory-subclass-unknown-frame.md
(closed).** Rule: every primitive runs under an `InternalActivationFrame` (`CPPCode.cpp:140`), so a
live `stackFrames` lists it while Rexx code it reaches runs. `Interp::primitive_level`
(`dispatch.rs`, cold) pushes a method level looked up by name for the duration of such a body:
`Object~run`, `Array~sort`/`stableSort`, `Array~hasItem` with an argument not of a base class,
`Object~request`'s `MAKE` send, a hash collection's `put`/`[]=` with an argument not of a base class
(`[]=` now its own function so the level has its name), `Directory`/`StringTable` `UNKNOWN` when sent
(a subclass's, or an explicit send; the base-class `processUnknown` stays frameless), and the class
`NEW` of `Method`, `Routine` and `Package` while they compile and install. Witnesses
`live_frames_nested_primitives.rex` and the `NEW` rows of `requires_install_levels.rex`.

**2026-10-02-frame-executable-invocation-and-native-levels.md (closed).** fexe1 and fexe3 agree at
base and head. fexe2: a fragment's frame now keeps one invocation (`FragmentLevel::invocation`,
minted on the first ask, shared by the live frame and the condition's INTERPRET frame;
`build_ended_frame` takes it). Witness `interpret_frame_invocation_indent.rex`.

**2026-10-14-native-method-frames.md (closed: probe agrees).** Its open parts (`Routine~call`'s CALL,
`sendWith`'s SENDWITH, `Message~send`'s SEND) agree at base and head (`probes/frames/f1.rex`'s `send`,
`sendwith`, `osend`, `call` and `callwith` rows; its `run`, `sort`, `eq`, `makearray` and `hash` rows
differ at base and are the live-frames item's), and
StackFrame `TESTNATIVECODEFRAME` passes in `suite/tests.tsv`.

**2026-10-22-interpret-clause-traceback-indent.md (closed).** Rule: an activation running INTERPRET
is stopped on the INTERPRET clause, at that clause's indent; the fragment's frame is stopped on the
fragment's clause. Here they were swapped: `clause_of` now gives an activation with fragments its
outermost fragment level's indent at any depth; a fragment's frame takes the owner's next fragment's
INTERPRET indent, else the owner's live clause or the clause it left for a call; and the INTERPRET
step restores the clause indent when the fragment ends (`run.rs`). Witness
`interpret_frame_invocation_indent.rex`. The probes' remaining differences are the oracle's
under-indent after a completed loop (Concerns 3).

**2026-10-28-rexx-package-source.md (closed).** `Method~package`/`Routine~package` of a library
program's code answers the REXX package (`dispatch/executable.rs`), whose `source` has no lines; it
answered a package named after the running program with the library's 1010 lines. Witness
`method_package_rexx_library.rex`.

**2026-10-26-compile-failure-condition-names-caller.md (closed).** Rule: rexxref's condition object
PROGRAM and PACKAGE name the program where the condition was raised, which is the source being
translated. `Interp::untranslated_origin` (`run.rs`, cold) makes a package for a named source that
failed (its lines, nothing it declares, named as the source) the failure's origin. Witness
`compile_failure_condition_origin.rex` (a called file's 35.1, its fixture named `.cls` so the parse tests skip it, and 99.903, `.Routine~new`'s 35.1). An
INTERPRET's failure is not this item: Concerns 2.

**2026-10-26-main-program-level-during-requires-install.md (closed).** Rule: a package installs in an
activation of its own, so while its `::REQUIRES` load it is a level stopped on that directive.
`push_installing_level` (`install.rs`) pushes that level for a package with no running activation
around each `::REQUIRES` load; it pops before the failure is blamed, so tracebacks are unchanged
(`probes/main-level/h1`-`h4`). Witness `requires_install_levels.rex` (a required prologue two
deep, a `::CONSTANT`, a trapped failure's frames and traceback).

**2026-10-22-executable-at-quadratic-depth.md (closed).** `frame_at` indexes instead of walking
(`activation.rs`); `executable_at` walks an internal-call chain once and gives each call in it the
answer (`ActivationCold::executable`). `corpus/lang/call_depth_cap_call.rex`: 1,201,334,529 Ir at
base, 355,537,591 at `67e9a22e9`, release (`probes/depth-cap/instructions.txt`); a dev-profile build
(`probes/depth-cap/debug-times.txt`) takes 1.60 s at head, and at base had not finished after 2269 s,
where it was killed: its `timeout 590` SIGTERM did not end it (Concerns 5).

**2026-10-26-library-rxmath-error-number.md (closed: probe agrees).** Every `rxmath` shape (`::requires
LIBRARY` quoted and bare, `EXTERNAL` routine and method binds, missing and present entry points)
agrees at base and head (`probes/rxmath/`). The 98.903 the review saw is this crate run without the
oracle's library directory on `LD_LIBRARY_PATH` (measured: `b3.rex` then 98.903 "Unable to load
library rxmath"); corpus programs carry it in a `.env` sidecar.

**2026-10-27-deadlock-report-shape.md (closed: probe agrees).** Outcome sets, path-normalised
(`probes/deadlock/classes.txt`, `classes-text.txt`, `outcomes.txt`): the oracle shows three classes
for each of `d2` (8 runs) and `g15` (40 runs): the started report naming its own line then the
waiter's, the waiter alone (`Compiled method "RESULT"`), and the two reports interleaved both naming
the waiter's line. This crate under `REXX_SWITCH_MODE` (seeds 1-6, `fifo`, `uniform:0.2`,
`pct:3,k=40`) produces the first two, byte-identical to the oracle's; the third is Deviation 67's
race.

**2026-10-10-compiled-method-package-in-trace.md and gate:phase-9a-gate.md:469 (closed: probe
agrees).** `t4c.rex` (package `M`), `.Method~new`/`.Routine~new`/`define` packages
(`probes/compiled-method-package/t4d.rex`), an external CALL under `::OPTIONS TRACE R` and
`.Routine~new`'s 35.1 message (`probes/gate-9a-469/g1.rex`) agree at base and head.

**2026-10-22-user-raised-in-own-handler.md (not closed: Deviation candidate).** Below.

## Deviation candidates (not written)

1. **A condition raised again while its CALL ON handler runs.** Rule: rexxref `condtra.xml`, CALL ON:
   a trap in the delayed state stays enabled; a condition other than ERROR and FAILURE raised again
   is delayed "until ... A RETURN is made from the subroutine. In this case, the condition trap is no
   longer delayed and the subroutine is called again immediately", or until a CALL ON for it, after
   which the call "takes place immediately". Both interpreters drop it: USER and HALT raised from a
   routine the handler calls run no second handler after RETURN (`probes/user-raised/u2.rex`), nor
   after a `CALL ON` re-issued in the handler (`u3.rex`). Candidate: follow the rule (redeliver at
   the handler's RETURN or at the new CALL ON), or license the shared answer.
2. **An INTERPRET's translation failure's PACKAGE.** The oracle's condition object's PACKAGE is a
   package object other than the running one, named as the program (`.context~package ==
   condition('o')~package` is 0, `probes/compile-condition/m2.rex`); here it is the running package
   (1). rexxref: the package of the program where the condition was raised, which for INTERPRET is
   the running program's. This crate follows the rule.

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file generated by the
test's driver from a copy outside the repository: `native_find_class_raised_condition` and
`native_new_executable_no_parent` (each with a `.env` naming the oracle's libraries),
`security_manager_activation_boundary`, `security_manager_requires_installing` (`.d`),
`live_frames_nested_primitives`, `interpret_frame_invocation_indent`,
`method_package_rexx_library`, `compile_failure_condition_origin` (`.d`),
`requires_install_levels` (`.d`). Each differs from the oracle at base and agrees at head on stdout,
stderr and rc (checked three-way from scratch copies before they were added; at head by the corpus
gate). I read each oracle stdout for the paths it claims: every security line names the checkpoint
or its absence as described; every frame listing shows the primitive's `METHOD <name> | The NIL
object` row and the plain Directory's lack of one; the invocation lines `1 2` twice; each traceback
line's indent; `REXX 0 | 0`; each untranslated source's name, line count and `0` identity; the
installing levels at the `::REQUIRES` lines.

## Perf

`callgrind-final.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b/final2 base=.../rexx-base-504ff0743
head=.../rexx-head-5053fbd9b`, every program, 3 rounds; no OVER or SPREAD flag (the table recomputed
with `-T` exits 0; the run was detached, so its own status was not captured). Libc subtracted, every
program is at most +0.0469% (startup); pingmsg +0.0449%, sayloop +0.0249%, interpret +0.0074%;
pingguard +0.0023%, pingsem -0.3381%; alloc -0.6422%, pkglocal -0.5655%, heapshape -0.2718%, rexxcps
-0.0250%. Whole process at most +0.4787% (interpret, +0.0074% libc subtracted). The base was built
from a `git archive` of `504ff0743` (`rust/` and `interpreter/`) in its own target dir.

The first full run, at `67e9a22e9` (`callgrind-67e9a22e9.txt`, exit 1), was OVER on pkglocal (+1.08%
libc subtracted: the LOCAL checkpoint's package-manager gate was a call per environment lookup) and
over budget on pingmsg (+0.21%: `executable_at`'s walk and prologue on each started message's spawner
frame), startup (+0.052%) and dirread (+0.092%). `5053fbd9b` inlines the gate, gives
`executable_at` a fast path for anything but an internal call, and checks only the hashed argument in
a hash put (`try2/`, `try3/` the one-round reruns).

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87b-target`; records in
`t8.7b/`.

| command | result |
|---|---|
| `cargo fmt --all --check` | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-api --all-targets -- -D warnings` | 0 (`checks-clippy.txt`, at `5053fbd9b`) |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage` (`checks-touched.txt`) | 0; lib 1202 passed (at `5053fbd9b`) |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 (at `67e9a22e9`; `5053fbd9b` touches no corpus file) |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`, at `5053fbd9b`) | 0, 74 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101 at `5053fbd9b`: the one mismatch the known `rexxutil_file_date_time.rex`. At `ffba1e60f` also `live_stack_frames.rex` (a fragment whose owner was suspended in a call took the callee's indent), fixed by `127a59431` |
| `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites` (`checks-unsafe.txt`) | 0 (at `67e9a22e9`) |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.7b/suite-final/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite-final/`, at `5053fbd9b`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `t8.7a/r1/suite/`. The earlier run at `67e9a22e9` (`suite/`) differed in two rows, neither this task's: SysFileTree `TEST_TREE_DIRECTORY` crate-side (the known flake) and RexxContext `TEST_INTERPRETER_THREAD_INVOCATION` failing on the oracle side; that group rerun alone three times (`suite-rexxcontext/`) was 15 of 15 on both sides each time |
| `memcap 16G cargo test -j 4 --release --no-fail-fast --workspace`, `rust/target/release/rexx` built first, default target dir (`close-workspace-tests.txt`) | 101 at `6d5a8dc11`: three failures. `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the known DST flake; `rexx-parse`'s `tiling` `every_corpus_program_tiles` and `variants` `every_variant_is_constructed_by_the_corpus_and_samples`, both on the fixture `compile_failure_condition_origin.d/perr.rex`, which those tests parse as a corpus program and which does not translate by design. The commit after renames it `perr.cls` (the witness calls `perr.cls`; its `sourceline_oracle` file regenerated); then `cargo test --release --no-fail-fast -p rexx-parse` (`checks-parse-all.txt`) exits 0 and the corpus gate is 101 on the known `rexxutil_file_date_time.rex` only |

## Concerns

1. **Primitive levels are per method, not general.** The oracle frames every primitive; this crate
   frames the ones the items name and the `NEW`s. Others that reach Rexx code (a `List`/`Table`
   `hasItem` or `index` with an object argument, `Array~index`, `removeItem`, a collection's `==`
   through `requestString`) still list no frame of their own. A general fix needs the running
   native's identity on every native call, which the send path would pay for.
2. Deviation candidate 2 above (INTERPRET translation failure's PACKAGE identity).
3. **The oracle's under-indent after a completed loop shows on stdout.** `~traceline` and
   `condition('o')~traceback` read inside INTERPRET after `do 2; nop; end` differ by two columns
   (`probes/interpret-indent/i1`-`i3`); Deviation 0 normalises this on stderr only. The witness
   avoids the shape.
4. The installing level makes a package's `::REQUIRES` load run with an activation on the stack
   where none was before (the main program's install). Corpus, suite and collect_stress agree; any
   reader of "no running activation" during a main program's requires install now sees that level.
5. **A busy interpreter outlives SIGTERM.** The base debug binary on `call_depth_cap_call.rex` ran on
   for 30 minutes after `timeout`'s SIGTERM and ended only on SIGKILL. Why was not established; at
   head the program finishes in 1.6 s.

## Fix round 1

Status: DONE_WITH_CONCERNS. Code `97ccdac9c`, `848403a01`, `10b0f5081`, `cca0956e0` (tests only); records
and this section in the commit after. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.7b/fix-r1/`
(`three.sh` the three-way runner: `o` oracle, `b` `5053fbd9b`, `h` this round; `checks-*.txt`, `suite/`,
`callgrind-final.txt`, `close-workspace-tests.txt`).

**Important 1 (primitive frame names the override).** `primitive_level` no longer looks the method up;
the level keeps no resolution and `build_method_level_frame` asks `Interp::running_primitive` when the
frame is built: the first method of the name in the receiver's search order that is a primitive, past
any Rexx override that forwarded to it (`dispatch.rs`, `dispatch/context.rs`; `MethodLevel::resolution`).
This also takes the hash put's by-name lookup off its path. Witness `corpus/lang/primitive_frame_past_override.rex`
(the review's `f2` and `rn2` shapes: `SORT`, `[]=`, `REQUEST`, a `Routine` subclass's class `NEW`, and the
trapped traceback beside them): oracle~`5053fbd9b` stdout differs, oracle~head equal on all three
(`fix-r1/f2/`, `fix-r1/rn/`).

**Important 2 (manager set during the main program's install).** `run_loaded` registers the package as
installing (`Interp::installing_managers`); `keep_security_manager` records there the manager the
package had before a set during the install; a directive's `::REQUIRES` asks it
(`installing_security_manager`), the installing level keeps it, and the activation that runs after the
install starts with it (`run_main`'s `kept_manager`). Witness
`corpus/lang/security_manager_set_during_install.rex` (the review's `sm1`: the main program's later
`::REQUIRES` and its command unasked, a routine begun after the set asked): oracle~`5053fbd9b` stdout
differs, oracle~head equal on all three (`fix-r1/sm1/`).

**Ruling 3 (a condition raised again while its CALL ON handler runs).** Derived rule, rexxref
`condtra.xml` (Action Taken when a Condition Is Trapped, CALL): while a trap is delayed, a condition other
than ERROR and FAILURE raised for it is held, and its action waits for the first of: a `CALL ON` or
`SIGNAL ON` for it, after which the call or signal takes place at once; a `CALL OFF` or `SIGNAL OFF`,
after which its default action occurs (only HALT's reports: 4.1); the handler's `RETURN`, after which the
handler is called again at once. An ERROR or FAILURE raised again is ignored. Here: `trap_for` answers a
delayed trap, so every raise site queues as for any `CALL ON` trap; delivery holds a condition whose trap
is delayed on the delaying activation's record (`ActivationCold::delayed`, `DelayedTrap`); a `RAISE ...
RETURN` in the handler, which queues on the delaying activation itself, is collected when the handler
returns; `run_pending_handler` returns what was held and `deliver_one_pending_trap` calls the handler again
for each, in raise order; `exec_condition_trap` releases the held conditions for a trap instruction naming
the delayed condition (`release_delayed`). A condition the debug pause's own input read queues for a
delayed trap is dropped rather than held (`debug_input_line`): rexxref's Notes set every trap OFF while
that input is processed, and the corpus programs whose `CALL ON NOTREADY` handler pauses at the end
of the input (`debug_input_eof_*`, `debug_handler_exit_at_iterate`, `debug_input_stdin_eof_call_on`)
were otherwise called again for ever (at `97ccdac9c` each ended rc 124 or 137 under `memcap 2G timeout
8`; at `848403a01` each agrees with `5053fbd9b` on stdout, stderr and rc). The oracle's answer: it drops the
condition (`u2.rex`, `u3.rex`: oracle and `5053fbd9b` equal, no second handler run), and segfaults on two
shapes (`corpus/oracle-crashes.txt` entries 64 and 65, never re-run). Witnesses:
`crates/rexx-exec/tests/delayed_trap_cases/crate` (test
`run::tests::conditions::a_condition_raised_while_its_handler_runs_is_held`): USER and HALT raised from a
routine the handler calls (the `u2` shape), a `RAISE ... RETURN` in the handler, a `CALL ON` re-issued in
the handler (the `u3` shape), a `SIGNAL ON` re-issued, a `CALL OFF` for USER (ignored; the adjacent case,
equal at `5053fbd9b`) and for HALT (4.1 at the `CALL OFF`), NOTREADY and `CALL ON ANY`, and ERROR and
FAILURE ignored (a FAILURE in its own handler no longer becomes an ERROR). The probes the cases come
from (`fix-r1/delayed/`, `b` and `h` outputs) each differ from `5053fbd9b` on stdout; the `CALL OFF
USER` half of `w2.rex` prints the same at both. Also
changed to the rule: `scheduler::tests::a_message_halt_inside_its_handler_runs_it_again_on_return` and
the `signals` tests `a_second_signal_in_a_call_on_halt_handler_runs_it_again`,
`a_second_signal_in_a_call_on_any_handler_runs_it_again` and `a_second_signal_ends_a_handlers_sleep`.
`2026-10-22-user-raised-in-own-handler.md` closed (`closed:10b0f5081`). The new oracle defects are
appended to `docs/superpowers/records/oracle-defects/defects.json`; its page is not republished.

**Minor 1.** `push_installing_level`'s doc names both `None` cases. **Minor 3.** The native-method-frames
closing (queued file and the paragraph above) names `f1.rex`'s rows that agree at base (`send`,
`sendwith`, `osend`, `call`, `callwith`) and those that are the live-frames item's.

### Perf

`fix-r1/callgrind-final.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o .../cg-9d/t8.7b/fix-r1-final
base=.../rexx-base-504ff0743 head=.../rexx-head-10b0f5081`, every program, 3 rounds, exit 0, no OVER or
SPREAD flag. Libc subtracted against base, at most +0.0312% (pingmsg); startup +0.0235% (was +0.0469% at
`5053fbd9b`), sayloop +0.0125%, interpret +0.0067%, pingguard -0.0076%, pingsem -0.3478%; whole process
at most +0.4708% (interpret, `_int_malloc` as before). The run at `848403a01`
(`fix-r1/callgrind-848403a01.txt`, exit 1) had pingguard +0.7743%, pingmsg +0.6539%, pingsem +0.4177%
libc subtracted, and interpret +0.8819% whole process: two `Vec`s added to `Activity`, which an activity
switch swaps whole (`switch_to`'s `mem::swap`); `10b0f5081` moves them to `ActivationCold` and `Interp`.

### Commands

From `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87b-target` unless named; at
`10b0f5081` unless named; outputs in `fix-r1/`.

| command | result |
|---|---|
| `cargo fmt --all --check` | 0 (at `cca0956e0`) |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 (at `cca0956e0`) |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage` (`checks-touched.txt`) | 0; lib 1203 passed |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`, `-2.txt`) | 101 then 0 (74 passed): the first run's one failure the known redirected-target row (`command_redirected_target_waits_elsewhere.rex`, `guard 0 3 waited 1` under stress) |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: the one mismatch the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../fix-r1/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `suite-final/` |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test signals` | 0 at `cca0956e0`, 42 passed |
| `memcap 16G cargo test --release -j 4 --no-fail-fast --workspace`, default target dir, `rust/target/release/rexx` built first (`close-workspace-tests.txt`) | 101 at `cca0956e0`: one failure, `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the known DST flake. The run at `10b0f5081` (`close-workspace-tests-10b0f5081.txt`) also failed the three `signals` tests above, which asserted the oracle's drop; `cca0956e0` changes them to the rule |

### Concerns

1. **A primitive level is now pushed whatever the receiver answers.** Where no method of the name in the
   receiver's search order is a primitive (a primitive reached with a receiver it was not found on), the
   frame is built with the NIL object as scope and no executable, where `5053fbd9b` pushed no level. No
   probe reaches it.
2. **The level's method is found when the frame is built**, from the receiver's methods then; a primitive
   defined into the receiver's search order between the push and a `stackFrames` read would be named
   instead.
3. **`Routine~call`'s level in `dispatch/executable.rs` keeps its by-name lookup.** Its receiver there is
   the program's own `Routine`; a `Routine` subclass's `newFile` routine reaches another path and agrees
   with the oracle at `5053fbd9b` (probed, not recorded as a witness).
4. **Rexxref-derived arms not in the ruling's text**: `SIGNAL ON` re-issued signals the first held
   condition with a condition object built at that clause (its rc and description, no ADDITIONAL);
   `CALL OFF`/`SIGNAL OFF` drop the held conditions, HALT's default 4.1 at that clause; the debug pause's
   own input conditions are dropped, not held. A handler that raises its own condition on every run is
   now called for ever.

## Fix round 2

Status: DONE_WITH_CONCERNS. Code `d775871f7`, `d1f8b387f`; records and this section in the commit after.
Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.7b/fix-r2/` (`four.sh` the runner: `o` oracle,
`b` `5053fbd9b`, `c` `cca0956e0`, `h` this round; probe directories with their `out/`; `red-cca0956e0*.txt`
the new tests run against `cca0956e0`'s code; `mem/figures.txt`; `checks-*.txt`, `suite/`,
`callgrind-final.txt`, `close-workspace-tests.txt`).

**Important 1 (NOTREADY raised while its trap is delayed).** Ignored, with ERROR and FAILURE
(`ignored_while_delayed`, `run/condition.rs`): a held or `RAISE ... RETURN`-queued NOTREADY is dropped,
and `raise_notready` raises none where its `CALL ON` trap, its own or `ANY`, is delayed by a running
handler or by a NOTREADY its clause already queued. rexxref: `condtra.xml:278`-`:280` ("ignored if the
NOTREADY condition trap is already in the delayed state"), `iostrms.xml:657`-`:667` (the trap enters the
DELAY state at the raise, and a NOTREADY raised in that state leaves it there and processing continues).
`d1/d1.rex`: `o`, `b` and `h` equal on all three descriptors, `c` runs the handler twice. Witnesses:
`a_condition_raised_again_while_its_call_trap_is_delayed`'s `notready` row (one handler run; red at
`cca0956e0`, `h NOTREADY` twice), the case file's "NOTREADY raised again is ignored" (red at `cca0956e0`,
`n = 2`, `red-cca0956e0-notready.txt`), and a new case "NOTREADY is ignored under a delayed CALL ON ANY
trap, and when raised twice in one clause" (`nr/nr.rex`: `c` calls the `ANY` handler again until the 2G
cap kills it, rc 137; `b` runs the NOTREADY handler twice for the one clause).
**The debug-pause drop of `848403a01` is removed**: its commit and comment justify it only by the `CALL
ON NOTREADY` handler pausing at the end of the input being called again for ever, which the ignore now
prevents where the NOTREADY is raised. `debug_input_line` is `5053fbd9b`'s again and
`drop_queued_for_delayed` is gone. What rexxref supports and stays: the NOTREADY ignore above, and Note
4 (`condtra.xml:489`-`:494`, traps OFF while interactive input is processed) as `ignored_in_debug_pause`
already applies it to the typed line. The corpus gate's `debug_input_eof_*`,
`debug_handler_exit_at_iterate` and `debug_input_stdin_eof_call_on` agree with the oracle.

**Important 2 (hidden primitive).** `live_levels` drops a primitive level whose method
`running_primitive` does not find, in a cold pass taken only while a primitive level is live, so such a
primitive lists no frame, as at `5053fbd9b`. `n5/`, `n6/`: `h` equal to `b` on all three; the oracle's
`METHOD HAS ... scope "MYARR"` row is the queue candidate `n2` of the re-review, at base too. Witness
`dispatch::tests::a_hidden_primitive_lists_no_frame`, red at `cca0956e0` (rc 120, the `lib.rs:730`
refusal).

**Important 3 (memory of a redelivery chain).** What each redelivery keeps is the chain of condition
objects, and the program can reach every one of them: a condition raised in the handler carries the
handler's frame in `STACKFRAMES`, whose `ARGUMENTS` is the condition the handler was called with.
`mem/walk.rex` walks m1's chain from the last handler run back to `first`, 2000 links at 2000. The same
graph built from ordinary deliveries, with no trap ever delayed (`mem/ctl3.rex`, each routine's frame
holding the previous condition object as its argument), retains as much here and on the oracle:

| program, N | `c` | `h` | oracle |
|---|---|---|---|
| `m1.rex` 2000 / 20000 | 40,804 / 218,944 KB | 38,932 / 218,916 KB | (drops the held condition) |
| `ctl3.rex` 2000 / 20000 | 40,180 / 226,640 KB | 41,580 / 226,744 KB | 20,248 / 99,104 KB |

So the redelivery keeps nothing of its own, and a chain cannot stay flat without collecting objects the
program can still read; an endless self-raising handler grows for the same reason. No bounded test: the
growth is the reachable graph's. The size per link against the oracle's is the condition object and
frame representation, on any delivery. Found instead: `redeliver_held`'s queue rooted nothing, so a
condition waiting while the handler runs for one before it was collectable. It is rooted as a
temporary now, and `release_delayed` roots the held object it reads. Witness
`conditions::conditions_waiting_for_their_handler_survive_a_collection` (the `w/g.rex` shape under
`run_program_collect_every_alloc`), red at `cca0956e0`: stdout ends after `h 2 second second-add`.

**Minor 1.** The `SIGNAL ON` released over a held condition takes its `ADDITIONAL` and `RESULT` from the
held condition object. The case file's `SIGNAL ON` case now asserts both (`sg two SIGNAL 10 two-add
two-res`); red at `cca0956e0` (`w/s.rex`: `sg two The NIL object The NIL object SIGNAL`).

**Minor 3.** `oracle-crashes.txt` entry 64 is titled for its program's two halves: a direct `RAISE ...
RETURN` in the handler, then a `SIGNAL ON` re-issued over a condition raised again.

**Deviation witnesses (tests only).** `run::tests::conditions::an_interpret_translation_failure_names_the_running_package`
pins `1 35.1` (oracle `0 35.1`, `w/a.rex`). `run::tests::indent::interpret_frames_after_a_completed_do_indent_by_block_structure`
pins the traceback and the `traceLine`s after `do 2; nop; end` inside `INTERPRET` (`w/b1.rex`,
`w/b2.rex`; the oracle's `return 1 / 0`, `x = f()` and `do f over` rows are two columns short). Both
pass at `cca0956e0` as well.

**New oracle crash, entry 66.** `nr/nr.rex` was run on the oracle by the four-way runner before its
second half was checked against the probes rule (one NOTREADY condition queued twice at one boundary):
SIGSEGV, rc 139. Entered in `oracle-crashes.txt`; `defects.json` is the controller's.

### Perf

`fix-r2/callgrind-final.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -r 3 -o .../cg-9d/t8.7b/fix-r2-final
base=.../rexx-base-504ff0743 head=.../rexx-head-d1f8b387f`, every program, no OVER or SPREAD flag (`-T`
over the outputs exit 0; every `cmd:` names one of the two binaries). Libc subtracted against base, at
most +0.0353% (pingmsg); startup +0.0235%, sayloop +0.0125%, interpret +0.0067%, pingguard -0.0076%,
pingsem -0.3481%. Whole process at most +0.4644% (interpret). The run at `d775871f7`
(`callgrind-d775871f7.txt`) had pingmsg +0.2051% libc subtracted: the primitive check ran on every
`live_levels` call, which every `START` makes for its spawner frame; `d1f8b387f` takes it only while a
primitive level is live.

### Commands

From `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87b-target` unless named; outputs
in `fix-r2/`.

| command | result |
|---|---|
| `cargo fmt --all --check` | 0 (at `d1f8b387f`) |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 (at `d775871f7`; again 0 at `d1f8b387f`) |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage --test signals` (`checks-touched.txt`) | 0 at `d775871f7`; lib 1207 passed, signals 42; lib again 1207 passed at `d1f8b387f` |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: the one mismatch the known `rexxutil_file_date_time.rex` |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../fix-r2/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `fix-r1/suite/` |
| the new tests against `cca0956e0`'s code (`red-cca0956e0.txt`, `red-cca0956e0-notready.txt`) | 101: red as named above; the two Deviation pins pass |
| `memcap 16G cargo test --release -j 4 --no-fail-fast --workspace`, default target dir, `rust/target/release/rexx` built first (`close-workspace-tests.txt`) | 101 at `d1f8b387f`: one failure, `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the known DST flake |

### Concerns

1. **Important 3 is not closable as briefed.** The retained memory is the reachable condition chain;
   flattening it would collect objects the program reads (`mem/walk.rex`). The outcome to rule is whether
   that is acceptable, or whether a redelivered handler should be called with something that does not
   link the chain.
2. **Entry 66 was run on the oracle once**, against the probes rule; it is recorded and never to be re-run.
3. **The SIGNAL release's other condition fields** (`POSITION` 10, the `SIGNAL ON` clause) are still the
   release clause's; only `ADDITIONAL` and `RESULT` are taken from the held condition.

## Fix round 3

Code `9cb989e54`, `376052dd5` (`refusal-sites.tsv` re-derived for `error.rs`'s shifted lines). Records
and this section are in the commit after them. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.7b/fix-r3/`
(`four.sh` is the runner: `o` oracle, `p` `d1f8b387f`, `c` `cca0956e0` (`rexx-head-10b0f5081`), `h` this
round (`rexx-head-9cb989e54`), stdin `/dev/null`. Probe directories hold their `out/`. A crate output
of more than 200k lines keeps its first 20 lines in `.head`. Also there: `red-d1f8b387f-*.txt`,
`checks-*.txt`, `suite/`, `callgrind-final.txt`, `close-workspace-tests.txt`). The oracle was run on
no probe this round.

**Important 1 (debug pause input).** `debug_input_line` again drops each condition its `LINEIN` send
queues for a trap that is delayed (`drop_queued_for_delayed`). The citation is rexxref `condtra.xml:489`-`:494`,
Note 4: every trap is OFF while interactive input is processed. NOTREADY is still ignored while
delayed (`ignored_while_delayed` is unchanged). Results:
* `dp2/`: `p` rc 137, OOM-killed at the 2G cap after 210840 stdout lines.
* `c` and `h` agree on all three descriptors (rc 0, `after 1`).
* `dp1/`: `p` prints `h 2 from input` to `h 4 from input`, then `after 4`. `c` and `h` agree with each
  other (`after 1`).

Witness: the case file's new case "a condition the debug pause's input read raises while its trap
is delayed is dropped" (`dp3/dp3.rex`). It is dp2's shape, bounded because the reader stops raising
after six reads. It is red at `d1f8b387f` (`red-d1f8b387f-debug.txt`): the handler runs six times and
the program prints `after 6`. At head it prints `after 1`.

**Minor 1 (SIGNAL ON release).** `release_delayed` now signals the held condition with the object it
was raised with. That object travels in `Activity::reraised_object` with the new `Delivery::held`
flag. The trap marks it `INSTRUCTION` `SIGNAL` (`trap_held_condition_object`) and changes nothing
else, so `POSITION`, `TRACEBACK`, `STACKFRAMES`, `ADDITIONAL` and `RESULT` all come from the raise.
SIGL still names the `SIGNAL ON` clause (`condtra.xml:677`-`:678`). The flag is cleared before the
condition becomes the activation's `active_condition`. A missing label clears the object through
`clear_failure_levels`. Results:
* `p8/`: `h` prints `sg two 12 27     12 *-*     raise user s2 description (dd) return`.
* `p` and `c` print `sg two 27 27     27 *-*   signal on user s2 name sg`.

Witness: the case file's `SIGNAL ON` case now also prints `POSITION` and the first `TRACEBACK` line
(`sg 5      5 *-*     raise ...`). It is red at `d1f8b387f` (`red-d1f8b387f-signal.txt`: `sg 10     10
*-*   signal on user sig name sg`).

**Minor 2 (NOTREADY under a CALL ON ANY delayed at the clause).** `raise_notready` treats the trap as
delayed when one of these was already queued at the trapping activation (`notready_trap_delayed_at`):
* a NOTREADY;
* where `CALL ON ANY` is the trap that takes NOTREADY, any condition that the `ANY` trap took.

Results:
* `p4/`: `h` prints `ha 1 USER U one`, `after a, m = 1`. `p` and `c` print `m = 2`.
* The second clause, where the NOTREADY comes first, is unchanged: one handler run for each condition.
* `p5/`: `p` and `h` agree on all three descriptors.

Witness: the case file's new case "NOTREADY is ignored under a CALL ON ANY trap a USER condition
queued at the same clause delayed". It is p4's first clause and runs on the crate only. It is red at
`d1f8b387f` (`red-d1f8b387f-notready.txt`: `ha 2 NOTREADY no_such_file_zz.txt`, `m = 2`).

Each red file comes from `d1f8b387f`'s `rust/` and `interpreter/` (`git archive`) with the case file
holding only the case concerned, through `cargo test --release -p rexx-exec --lib --
a_condition_raised_while_its_handler_runs_is_held`, exit 101 each. The signal file is the first
run, with the whole case file. Its first failure is the signal case.

**Regression probes.** The re-review's `d1`, `p1`, `p2`, `p3`, `p6` and `p7`: `p` and `h` agree on all
three descriptors for each.

**Item 4.** `oracle-crashes.txt` entry 67 is p4's program, with the reviewer's single run: rc 139,
stdout `ha 1 NOTREADY no_such_file_zz.txt`, stderr empty. `rust/CLAUDE.md`'s probe bullet gained a
sentence: two differently named conditions in one clause are not safe when one `CALL ON ANY` traps
both (entry 67).

### Perf

`fix-r3/callgrind-final.txt` was produced by `bench-programs/callgrind.sh -B 0.5 -j 4 -r 3 -o .../cg-9d/t8.7b/fix-r3-final
base=.../rexx-base-504ff0743 head=.../rexx-head-9cb989e54`, run on every program. It exited 0 with no
OVER or SPREAD flag. Every `cmd:` names one of the two binaries, 99 each.

| program | -libc d% against base |
|---|---|
| pingmsg (the largest) | +0.0366% |
| startup | +0.0238% |
| sayloop | +0.0125% |
| interpret | +0.0067% |
| pingguard | -0.0162% |
| pingsem | -0.3563% |

Whole process, the largest is +0.4644% (interpret, as in fix round 2).

### Commands

All from `rust/` with `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR=/tmp/claude-1000/t87b-target` unless named.
All at `9cb989e54`/`376052dd5`. Outputs are in `fix-r3/`.

| command | result |
|---|---|
| `cargo fmt --all --check` | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage --test signals` (`checks-touched.txt`) | 101 at `9cb989e54`. lib 1207 passed, signals 42. The one failure was `refusal_sites`: `error.rs` lines shifted. Re-derived with `REXX_REFUSAL_SITES_REFRESH=1` (`376052dd5`), then 0 (`checks-refusal-sites.txt`) |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101. The one mismatch is the known `rexxutil_file_date_time.rex` |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../fix-r3/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`checks-suite.txt`) | 0. `tests.tsv`, `table.tsv`, `verdicts.tsv` and `oracle-differences.tsv` are identical to `fix-r2/suite/` |
| the new cases against `d1f8b387f` (`red-d1f8b387f-*.txt`) | 101 each, red as named above |
| `memcap 16G cargo test --release -j 4 --no-fail-fast --workspace`, default target dir, `rust/target/release/rexx` built first (`close-workspace-build.txt`, `close-workspace-tests.txt`) | 101. One failure: `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the known DST flake |

### Concerns

1. **Note 4 is wider than the restored drop.** A condition that the input read raises in a level
   whose trap is not delayed is still delivered. Examples: dp2's first read in main, and `dp3`'s
   `h 2 from input` after `after 1`. `c` does the same, and this predates the task. The new debug case
   pins that output too, so a later fix of the rest of Note 4 changes its expected block.
2. **The SIGNAL release reuses `Activity::reraised_object`** with a `Delivery::held` flag. No probe
   reaches a path that sets `reraised_object` between the release and its trap.
