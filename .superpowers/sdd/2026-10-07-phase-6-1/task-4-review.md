# Task 4 review: a3c2c3c0a..3c87b21d9

Ours: `rexx-run` built in release from `git archive 3c87b21d9 rust interpreter` (every file touched,
own target dir, one `Compiling rexx-exec` line). Base: the same from `a3c2c3c0a`, in its own target
dir. Collect-every-alloc runs: a throwaway test `zz_probe_t4rev.rs` in the scratch copy only, which
runs each probe through `run_program` and `run_program_collect_every_alloc` and compares rc, stdout
and stderr. Oracle: the standard wrapper. Every probe ran from fresh empty directories through `cmp.sh`
(ours default, ours `REXX_SWITCH_MODE=every`, oracle; run paths normalised before the diff). Probes,
script and outputs:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t4rev/`
(`probes/`, `cmp.sh`, `out/<probe>[-tag]/`). Scratch target dirs deleted.

### Spec Compliance

- ✅ Spec compliant for what the brief lists. Every Step 1 program is in `rust/corpus/lang/` and
  `phase-6-1.txt`: b4's six, b5's thirteen, b6's two, b7's four, b8, b14's three, and both
  VariableReference `NEW` programs. Step 2's refusals are gone (`Loud::method_body`; the
  `method_from_source` sites at `class_protocol.rs` `:274`/`:777`/`:782`/`:1115`,
  `object_protocol.rs` `:488`/`:728`, `executable.rs` `:581`/`:622`; `object_method` at
  `object_protocol.rs:531` and `dispatch.rs:2082`). Pins `run/tests/directives.rs:264`/`:270` are
  deleted, and `scheduler/tests/native.rs:301` is repointed. Step 3's record is in the gate file.
- ⚠️ b7 ("answer the Method object") is wrong for one installer the brief does not name,
  `Directory~setMethod`. See Critical 1 and Important 1. The controller should decide whether b7 covers
  it. It was a loud refusal at base, so this task's change turned it into a panic and a wrong answer.

### Strengths

- `ExecutableRecord::installed` now carries the identity for every Method object, and the
  `table_method_bodies` side table is deleted. `scoped_method` plus `copy_method_identity`
  (`dispatch.rs:2304`) puts every installer (`setMethod`, `run`, `enhanced`, `define`,
  `defineMethods`, `defineClassMethod`, enhancing class methods) behind one rule, copying every row a
  send reads.
- setMethod does `newScope(.nil)` before the checks and `newScope(target)` after
  (`object_protocol.rs:470-515`). That gives the oracle's identity answers. Checked beyond the
  witnesses by `d1_setmethod_twice` (`e == m` is `0`, scope `.nil`, on both engines) and
  `d2_run_thrice` (`1`, `0`, `0`).
- The rooting of fresh copies holds. `method_new_scope` pushes its copy as a temp, `native_run` pushes
  the executable, and `ObjectMethod.executable` and `ActivationCold.executable` are traced. No
  collect-every-alloc difference except Critical 1.
- The three `SourceTaker` raises and the `requestArray`/`makeString` order match the oracle on every
  source shape I tried (below).

### Issues

#### Critical (Must Fix)

1. **Panic, rc 101: `.context~executable` in a method `Directory~setMethod` compiled from source
   panics once a collection has run.** The panic is at `environment/identities.rs:301`.
   - Program (`probes/f6_dir_setmethod_alloc.rex`, no `gc` call):
     ```
     d = .directory~new
     d~setMethod('x', 'return .context~executable~source[1]')
     do i = 1 to 200000; z = .array~new(10); z[1] = 'abcdefghijklmnop' i; end
     say d~x
     ```
     Oracle: rc 0, prints the source line. Ours, both engine modes: rc 101,
     `panicked at crates/rexx-exec/src/lib.rs:2943:18: class_owns on a handle that is not a live class object`.
     Base `a3c2c3c0a`: rc 120, the loud
     `a message send to a method context whose scope no longer defines it ... (Phase 5)`.
     `f3_dir_setmethod_gc_force.rex` (`call gc 'force'` in place of the loop) panics the same way. So does
     `f2` under collect-every-alloc, and `s1_stress_mix` under collect-every-alloc.
   - Mechanism:
     - `native_directory_set_method` drops the Method object (`dispatch/hash.rs:1934`,
       `let (_, method) = super::run_method(interp, source)?`), so nothing holds it.
     - Once it is swept, `running_method_executable` misses its own-entry branch (a Directory entry is
       not an `ObjectMethod`), its class branch (the scope is `.nil`) and its standing branch (the object
       is dead). It falls through to the rebuild.
     - The rebuild calls `attach_annotations(object, Annotated::Member(scope, ...))` with `scope ==
       .nil` (`identities.rs:301`). `annotation_table` then calls `class_owns(.nil, table)`, which panics.
   - Fix shape, not checked:
     - Hold the executable beside the Directory entry and answer it, which also fixes Important 1.
     - Keep the rebuild from treating a non-class scope as `Annotated::Member`.
     - Add a crate test with a forced collection that asserts it reaches the rebuild site, per the
       defect rule.

#### Important (Should Fix)

1. **`.context~executable` in a `Directory~setMethod` method answers the wrong object when it was
   given a scoped Method object.** No collection is needed.
   - `probes/f1_dir_setmethod_executable.rex`: `m = .u~method('M')`, `d~setMethod('x', m)`, and `e =
     d~x`, where `M` returns `.context~executable`. `say (e == m) e~scope` prints `0 The NIL object` on
     the oracle (the `.nil`-scoped copy). Ours prints `1 The U class` in both engine modes. Base: rc 120,
     loud.
   - `f5` (`.context~executable~scope` after `call gc 'force'`) is the same: the oracle answers
     `The NIL object` and ours `The U class`. `s1_stress_mix.rex` line 3 is the same difference.
   - Cause: the same discarded object (`hash.rs:1934`). `running_method_executable`'s standing branch
     (`identities.rs:252-256`) then picks the lowest identity running the body, which is `U`'s own entry.
   - Report concern 4 says the Directory case differs only "after a collection". That understates it:
     a scoped Method object is wrong at once, and a source-compiled one panics after a collection
     (Critical 1).
   - The witness `directory_setmethod_method_object.rex` never reads `.context~executable`, so the
     corpus cannot see either defect.

#### Minor (Nice to Have)

1. **False sentence, `lib.rs:596-597`.** The `Loud::object_method` doc says `SETMETHOD` sent to "a
   `Directory`" is "private, 97.2 on both engines". `Directory~setMethod` is public. `f4`'s
   `d~setMethod('x', 'return "ran"')` from the main program answers `ran` on both. The Array and
   String halves of that sentence are true.
2. **Stale record, `docs/superpowers/plans/phase-4-exclusions.txt:5574-5587`.** It says
   `Directory~setMethod` and `self~setMethod` of a `loadExternalMethod` answer give "stdout empty, rc
   120, ... a one-off method whose body this crate does not hold". That is no longer true.
   - With rxregexp resolvable (`LD_LIBRARY_PATH` set to the oracle's lib), ours prints `set` and ends
     with 88.901 at rc 168, as the oracle does. The traceback carries the extra
     `Compiled method "UNKNOWN"` line (report concern 3).
   - Base gave the recorded rc 120 (`out/g3`).
   - Per the prose rule the false sentence goes. No witness covers a `loadExternalMethod` answer
     through `setMethod`.
3. **Unrecorded oracle crash.** `.method~new('m', x)`, where `x`'s `MAKEARRAY` answers a string
   (`probes/e7_5.rex`), is SIGSEGV rc 139 on the oracle, 5 runs of 5. Ours raises 93.961. It is not in
   `rust/corpus/oracle-crashes.txt`, whose entry 28 is the FORWARD form only.
4. **Unwitnessed divergence: a `MAKESTRING` that answers a non-string.**
   `class_protocol.rs:307-308` takes the `REQUEST('STRING')` answer only if it is a string, and
   otherwise raises 93.961. The oracle uses the answer's string value as the source line.
   - In `e7_7.rex`, `MAKESTRING` answers `.array~of('return 1')`. The oracle compiles the line
     `an Array`, which runs as a command (`/bin/sh: 1: AN: not found`, then 91.999). Ours raises 93.961.
   - This is an edge case, and arguably the oracle's type confusion. It needs a ruling and a row, or a
     witness either way.
5. **Witness gaps** (the behaviour agrees; there is just no corpus program for it):
   - The program's own Routine: EXIT inside the re-run, recursion through `.context~executable~call`,
     a call from a method, identity of `.context~executable` inside the re-run (`d6`), a trapped SYNTAX
     inside, and an untrapped one reaching the caller's trap (`h3`). `~start`/`~result` and GUARD WHEN
     waits inside it agreed 5 runs of 5 each (`h1`, `h2`).
   - Source shapes: a non-empty Directory, Table, Set, Bag, StringTable, an Array subclass, an
     `enhanced` object's MAKESTRING, a sparse array and a number (`e7_1`-`e7_12` except `_5` and `_7`).
     `loadPackage` with a List, a MAKEARRAY object and an empty Directory (`d8`).
6. `borrowed_run_row.rex:6` installs a `SETMETHOD` row as `sm` that nothing sends, so that line tests
   nothing. In `load_package_object_source.rex`, the last stdout line is `s2:` falling through and
   reprinting `s1`'s condition, not a second raise.
7. Every `.context~executable` in a `::ROUTINE` now runs `routine_object_running`
   (`dispatch/context.rs`), a linear scan of `executable_sources`, before the table lookup it did
   before. Not measured. No benchmark program reads it.

### Checks run

1. **Reaching `object_method` (`dispatch.rs:2114`) and `expose_receiver` (`run.rs:1546`).** Neither
   was reached. Every probe below agrees with the oracle on all three descriptors unless noted.
   - Subclasses of Array, Bag, CircularQueue, Directory, IdentityTable, List, Monitor, MutableBuffer,
     Properties, Queue, Relation, Set, Stream, StringTable, Supplier and Table. Each instance method did
     EXPOSE, `self~setMethod` and `self~unsetMethod` (`r1_sub_*`).
   - Subclasses of String, Stem and Message instead stop at `method "NEW" of class "K_..." is not
     implemented (Phase 9)` (spec R2, pre-existing).
   - Entry methods on a Directory, StringTable, Properties and `.local` doing EXPOSE,
     `setMethod:.object`, `unsetMethod:.object` and `self~run` of a floating method that EXPOSEs
     (`r2_*`). The RUN-row traceback name difference is report concern 3.
   - A borrowed Object `RUN` row on a Directory (`r3_borrowed_dir_run`, 88.901 on both, traceback
     differs per concern 3).
   - `.array~enhanced` with EXPOSE and setMethod (`r3_enhanced_prim`).
   - `send`, `sendWith`, `start` and a `Message` delivering `RUN` or `SETMETHOD` to a String, `.nil`
     and an Array (`r4_*`). All are 97.2 on both.
   - The borrowed-RUN guard probe on a waiting guarded method: `h4_run_guard_wait`, 5 runs, `inner 1 1`
     on both.
2. **Rooting.** Every `d*`, `e7_*`, `f*`, `r2_*` and `s2_stress_mix_nodir` probe matched plain vs.
   collect-every-alloc, with 12 to 450833 collections each. Long strings were used throughout.
   - `s2_stress_mix_nodir` covers a class-object `setMethod` with a scoped Method object under OBJECT
     scope, a class EXPOSE pool, `run` of a List source, MAKESTRING, `defineMethods`, an enhanced
     EXPOSE and `Routine~new`. 115 collections, same.
   - The exceptions are `f2`/`f3`/`s1`, which hit Critical 1.
   - The 53 witnesses are already in `collect_stress.rs`'s subset through `phase-6-1.txt`.
3. **Witnesses, and more than one send.**
   - All 53 Task 4 programs (`git diff --name-only` over `rust/corpus/lang/`, Task 3's
     `do_object_compare_array` excluded) are identical to the oracle in both engine modes. I read each
     stdout and each reaches its claimed path; the exceptions are noted in Minor 6.
   - Double sends:
     - `d1`: setMethod then two sends, two receivers, a re-set, an unset (97.1).
     - `d2`: run three times, a source run twice, a scoped Method run twice.
     - `d3`: define, redefine, define with a Method object, delete then 97.1, old and new instances.
     - `d4`: two `enhanced` objects from one table.
   - All agree.
4. **Widened scope.**
   - `define` bodies: `d3` agrees.
   - The main Routine call: `d6`, `h1`, `h2` and `h3` agree.
   - The `.NIL` spelling: `d10` (a `.METHODS` method erroring under setMethod) and `d11` (`run`'s
     traceback) agree. In `d9`, `>I>`/`<I<` with scope `.NIL` and `Class` agree; only the package-name
     difference of concern 3 remains.
   - The `requestArray`/`makeString` path: `e7_*` and `d8` agree except Minor 3 and Minor 4.
   - Coverage gaps: Minor 5.
5. **Refusal tables and prose.**
   - In the scratch copy, `refusal_sites` passes (5 passed), and `REXX_REFUSAL_SITES_REFRESH=1` rewrites
     `refusal-sites.tsv` byte-identical to the commit.
   - `git grep` at `3c87b21d9` for each deleted refusal's text found nothing in tests except the
     whole_groups lines (item 6).
   - It also found the stale exclusions record (Minor 2) and historical plan files.
   - The gate record's "earlier tasks' pads printed 0.0000% on these programs" holds: Task 1 padded the
     five send programs and Task 2 the three loop programs.
   - False prose: Minor 1, Minor 2. Report concern 4 understates Important 1.
6. **Stale `whole_groups` lines.**
   - `tests/concurrency_tests.rs:2926-2934` (TRACE_TraceObject `whole`, normal and every) still expect
     `rexx-exec: a message send to a method context whose scope no longer defines it ... rc 120, last
     started TEST_OBJECT_AND_SCOPE`.
   - That refusal can no longer fire. `w1_traceobject_scope.rex`, the test's resource with its loop,
     completes rc 0 with `items 0` (the oracle has 19). The collector gap is Task 2's.
   - So TEST_OBJECT_AND_SCOPE becomes an assertion failure, `assertEquals(arr~items, 19)`, and the
     whole-mode run continues past it. The line would become a failure or error status with
     TEST_OBJECT_AND_SCOPE among the failing tests.
   - I do not have the exact text. My scratch run of `whole_groups` (release, gated) was OOM-killed at
     the `memcap 8G` cap.
   - The Class group's `TEST_CLASS_DEFINE` lines (`:2534`, `:2541`, a `define` Phase 9 refusal) are
     likely to move as well; not run.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The identity design and the witnesses are sound and agree with the oracle broadly,
including under collect-every-alloc. But `Directory~setMethod` throws away the Method object it
installs. `.context~executable` there now panics after a collection (Critical 1) or answers the wrong
object (Important 1), where base refused loudly.

Findings: Critical 1, Important 1, Minor 7.
