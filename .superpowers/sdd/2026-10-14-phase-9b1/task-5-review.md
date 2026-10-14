# Phase 9b1 Task 5 review: packages are collected

Reviewed `62a1a9de6..8294a7908` (code `dd5017a8e`). Probes ran on a `git archive` of `8294a7908`
under `/tmp/t5rev` with its own `CARGO_TARGET_DIR`, through a throwaway `examples/stress.rs` that
calls `run_program_collect_every_alloc` (debug and release builds), each from a fresh `mkdir`'d
directory under `memcap 4G`. Oracle: the standard wrapper, the filler a 3000-pass
`zz = copies('y', 100000)` loop. Crate filler: `do zi = 1 to 3; zz = 'f' zi; end`. The copy is
deleted.

## Spec Compliance

❌ Issues found.

1. **Review Focus 1: a package is collected while the interpreter is still using it**
   (`rust/crates/rexx-exec/src/lib.rs:2349`-`:2351`, `enter_external_program`, unchanged code made
   wrong by this task). After an external call returns, the callee's activation is gone and nothing
   reaches its package. `package_object(caller)` then builds the caller's package if it was never
   asked for, that allocation collects, and `package_object(program_id)` finds the callee's package
   dead. Debug: the new `debug_assert` panics. Release: it rebuilds an empty package and merges
   nothing, so a public class the callee added at run time is lost.

   ```
   -- p09e.rex
   v = ext9e()
   say 'p09e' v .dk~id
   -- ext9e.rex
   .context~package~addPublicClass('DK', .object~subclass('dk'))
   FILL
   return 'v' || 1
   ```

   Oracle: `p09e v1 dk`, rc 0. Crate, collect-every-allocation, release: `Error 97.1: Object ".DK"
   does not understand message "ID"`, exit 159. Debug: panic `Program(ProgramId(4))'s package was
   asked for after it was collected` at `identities.rs:156`, rc 101 (backtrace: `package_object` <-
   `enter_external_program` at `lib.rs:2350`). Plain run: `p09e v1 dk`. Before this task, the
   global root kept every package, so this task introduces it. The same panic fires for any
   external function called from a main program that has not asked for its own package yet
   (`p01`: `say ext1()`, where `ext1.rex` adds a routine to its own package; `p06`: an external
   routine using INTERPRET). Fix: hold the callee's package across its return and the merge, for
   example by building and rooting it as a temp before `run_loaded`. That adds an allocation to
   every external call, so measure extcall.

2. **The waiting-activity shape is not pinned (Review Focus 1, Step 3).** The walk of activities
   that are not running (`scheduler.rs:411`-`:415`, `Activities::running_programs`) is
   load-bearing, but no test goes red without it (report Concern 2, mutation M1). Here is a shape
   where only a parked activity's frame reaches the package, because the package is built inside
   an internal routine whose temps are popped when it returns:

   ```
   -- p10.rex
   sem = .EventSemaphore~new
   ready = .EventSemaphore~new
   msg = .starter~new~start('go', sem, ready)
   ready~wait
   FILL
   sem~post
   say 'p10' msg~result
   ::class starter
   ::method go
     use arg sem, ready
     return ext10(sem, ready)
   -- ext10.rex
   use arg sem, ready
   h = pk()
   ready~post
   sem~wait
   return (.local~w10~value \== .nil) (.context~package~identityHash == h) .context~package~findRoutine('Z7')~call
   pk:
     .local~w10 = .WeakReference~new(.context~package)
     .context~package~addRoutine('Z7', .routine~new('z', 'return 7'))
     return .context~package~identityHash
   ```

   Oracle `p10 1 1 7`, rc 0. Head, debug and release, under collect-every-allocation: `p10 1 1 7`.
   With M1 (the loop body replaced by `let _ = out;`): debug panics at `identities.rs:156` from
   `begin_invoke` after the wake; release gives `Error 97.1: Object "The NIL object" does not
   understand message "CALL"`, exit 159. Add this program to `a_package_in_use_is_never_collected`
   (it needs `ext10.rex` in the run directory, or the routine inlined some other way that keeps
   the package unbuilt outside `pk`).

⚠️ Cannot verify from the diff: Step 5's memory bar, moved to Task 6 by ruling.

Everything else in the brief is met. Step 1: only the REXX package keeps a global root
(`identities.rs:186`-`:193`). Step 2: weak cache (`lib.rs:3144`-`:3151`) with its witness.
Step 4: the record shows TESTRECLAIM01 passed. Step 6: no Task 5 `ALLOWED` rows, and the owner
is removed.

## Strengths

- The holder table in the report matches the code. Each holder I mutated or probed behaves as
  claimed.
- `Bound::Code`/`ScopedCode` hold the executable's package without growing `NativeObject`
  (`body.rs:1027`, `:1047`). The bug in the first draft, `.nil` used as "no scope", was found and
  fixed with separate variants.
- The debug assertion in `package_object` is what made finding 1 and the M1 gap visible.
- Re-requiring is witnessed with an oracle measurement taken first, including Task 4's binary as
  the negative.

## Issues

### Critical (Must Fix)

None.

### Important (Should Fix)

1. Finding 1 above (`lib.rs:2349`-`:2351`). A package is collected mid-use: a debug panic and a
   release wrong answer.
2. Finding 2 above (`scheduler.rs:411`). A new root with no red test, against the global GC-rooting
   constraint. `p10` is the test.
3. **The replied-continuation branch of `Activity::running_programs` has no red test either**
   (`activity.rs:505`-`:511`). Mutant: `continuation.activity.running_programs(out);` replaced by
   `let _ = continuation;`. `p05c` and `p11`, where a method installed with `setMethod` does
   `reply 1`, the method is then unset, and it waits on a semaphore (in `p11` the package is built
   inside an internal routine), both stay at `1 1 7` under that mutant, as on the oracle. Witness
   it, or delete it and say what holds the continuation's package. This is the ruling Task 3 got for
   unwitnessed roots.
4. **The release fallback in `package_object` turns a rooting bug into a silent wrong answer**
   (`identities.rs:155`-`:162`). A rebuilt package has a new identity and none of the dynamic
   state, which is how finding 1 ships in release. Consider a loud failure instead of the rebuild,
   or a test that runs the probes above in debug as well as release. The debug collect_stress run
   is the only gate that sees this class.

### Minor (Nice to Have)

1. `copy_class_package`'s `class_holds_package` (`identities.rs:79`-`:82`) cannot be observed:
   the original class, which is never collected (D59), already holds the same package. It is
   harmless, but no test can witness it.
2. The doc comment on `Activity::running_programs` is pasted between `object_roots`'s doc comment
   and `object_roots` itself (`activity.rs:489`-`:515`). `object_roots` now has no doc, and the
   new function starts with `object_roots`'s first two lines ("Appends every `ObjRef` the activity
   must hand the collector to `out`, and names every field..."). This is the insertion-orphans-doc
   defect.

## Probes (collect-every-allocation, debug and release; oracle 3000-pass filler)

| shape | probe | oracle | crate dbg / rel |
|---|---|---|---|
| running activation of an external routine's code, package built in its own frame | p01 | `1 1 7` twice | dbg panic (finding 1, second call's merge), rel `1 1 7` |
| `.context~package` in a variable, from a `Package~new` routine and a `Routine~new` | p02 | `1 1 7 1`, `1 8` | same |
| routine object (with a private helper) and a floating `::method` via `definedMethods` + `setMethod`, package dropped | p03 | `1 1 7`, `1` | same |
| object of a class from a `Package~new`, and a subclass of a `::requires`'d class | p04 | `1 1 7`, `1 1` | same |
| GUARD WHEN on a started activity | p05a | `1 1` | same |
| external routine parked on `EventSemaphore~wait` | p05b | `1 1 7` | same (green under M1 too: its own frame's temp holds the package) |
| reply-continued `setMethod` method, then unset, waiting | p05c, p11 | `1 1 7` | same (green under M1 and under the continuation mutant) |
| external routine in `SysSleep` on another activity | p05d | `1 1 7` | same |
| INTERPRET in an external routine, a class method, a `Package~new` routine | p06 | `1 1 1 7`, `1 1`, `1 1 7` | dbg panic (finding 1), rel same |
| only `.local`, `.environment`, `addPackage(p, 'NS')` + `ns:nr()` | p07 | `1 1 7`, `1 1`, `1 1` | same |
| `Message` to a routine of a dropped package and to a `setMethod` method | p08 | `1`, `1` | same |
| external call merging public routines/classes | p09, p09b-d, p09f-g | match | same |
| external call merging a runtime public class, caller package unbuilt | p09e | `v1 dk`, rc 0 | dbg panic, rel 97.1 rc 159 (finding 1) |
| only a parked frame reaches the package | p10 | `1 1 7` | same at head; red under M1 (finding 2) |
| package annotations: main, external, required | p12 | `blue 1`, `big`, `dark` | same |

## Checks

1. **Concern 2 (the walk over activities that are not running) is reachable and load-bearing.**
   See finding 2 and `p10`.
2. **Weak requires cache.** `corpus/lang/package_rerequire_collected.rex`: the oracle gives
   `first answered / collected 1 / again answered 1 / prologs 2 / cached 1 prologs 2`, rc 0, the
   same in 2 of 2 runs. The crate under collect-every-allocation, debug and release (113
   collections), and plain release (1 collection) print the same lines.
3. **TESTRECLAIM01.** `task-5/suite/tests.tsv:433`: `base/class/Package TESTRECLAIM01 - passed
   passed`. Sorted, `task-4/suite/tests.tsv` differs from it in that row alone (it was `blocked:
   timeout after 60 s`). The test's own wait is `loop until package~value == .nil`
   (`ootest/ooRexx/base/class/Package.testGroup:179`), so a pass means the package cleared.
4. **Perf.** Head rebuilt from the archive (`rexx-run-9b1-t5-review`, sha256 `d6ee89ff...`):
   `timeout 590 bench-programs/callgrind.sh -B 0.5 -j 8 -p "startup alloc hashnew pkglocal" -o
   /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-5-review
   pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run
   head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t5-review`, from `rust/`, exit 0.
   Whole process: startup +0.3156%, alloc -1.4278%, hashnew -0.0545%, pkglocal -7.6359%. This
   reproduces the report to four decimals (hashnew -0.0544 there).
5. **Mutations.** M1 (`Activities::running_programs` walks nothing) stayed green. It matters:
   `p10` kills it in both builds. A second unwitnessed branch, the continuation walk, is
   Important 3. Both mutants were applied in the archive copy and restored from saved copies, and
   `git diff --no-index --stat` against the repository files was empty after each.

## Assessment

**Task quality:** Needs fixes

**Reasoning:** Packages are collected and the holders are mostly right. The re-require, the
TESTRECLAIM01 and the perf claims reproduce. But an external call's merge reads its callee's
package after the collector may have freed it: a debug panic, and a release wrong answer against
oracle rc 0. The parked-frame root and the continuation root have no red test, and `p10` supplies
the first one.
