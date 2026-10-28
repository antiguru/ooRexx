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
`sendWith`'s SENDWITH, `Message~send`'s SEND) agree at base and head (`probes/frames/f1.rex`), and
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
