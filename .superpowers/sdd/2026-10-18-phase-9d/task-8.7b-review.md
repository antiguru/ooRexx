# Task 8.7b review: security manager, native API, frames and install

Reviewed `5de63cee6..04d155abc` (code BASE `5de63cee6`, tree identical in code to `504ff0743`; code
HEAD `5053fbd9b`; records HEAD `04d155abc`).

**Spec compliance: COMPLIANT.** **Quality: CHANGES REQUESTED.** Critical 0, Important 2, Minor 3.

* Important 1: a primitive's new live frame is looked up by name from the receiver, so under a
  subclass override that forwards to super it names the override's method and scope.
* Important 2: a security manager set during the main program's `::REQUIRES` install, which the
  new installing level makes reachable, is asked by the main program's later `::REQUIRES` and
  commands, though the main activation began before it was set.

## What I ran

* Builds: `git archive` of `504ff0743` and `04d155abc` (`rust/` and `interpreter/`) into fresh
  directories, each `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4` into
  `/tmp/claude-1000/t87b-review-target-base` and `-head`, exit 0. The `.text` section of each is
  byte-identical to `ooRexx-p9-pin/cg-9d/t8.7b/rexx-base-504ff0743` and `rexx-head-5053fbd9b`
  (`objcopy -O binary --only-section=.text`, `cmp`), so the pinned binaries carry the named commits.
  All builds, the worktree below and the callgrind outputs are deleted.
* Probes: each in its own directory under `t87b-review-scratch/`, run by a three-way script
  (oracle under the standard wrapper, base and head under `memcap 2G timeout -k 5 20`, the oracle's
  `build/lib` on `LD_LIBRARY_PATH` for all three), stdout, stderr and rc compared separately.
* Every new `corpus/lang` witness, copied with its `.d` fixtures into a fresh directory: each
  differs from the oracle at base on stdout and matches it at head on stdout, stderr and rc.
* Every "probe agrees" closing re-run three-way (below).
* Perf: `bench-programs/callgrind.sh -B 0.5 -j 4 -r 3 -p "startup pingmsg sayloop interpret dirread
  pkglocal" base=<my base> head=<my head>`, exit 0; the head `cmd:` line of every `.cg` file in the
  implementer's `final2/`.
* Memory and time: loops at two counts; callgrind on a depth program at two depths.
* Workspace test at HEAD: a detached `git worktree` of `04d155abc` under the scratchpad with its own
  `rust/target`, the untracked `oodocs`, `ootest` and `build` linked in, `rust/target/release/rexx`
  built first, then `CARGO_INCREMENTAL=0 memcap 16G cargo test --release -j 4 --no-fail-fast
  --workspace`: exit 101 (below).

## Witnesses and closings

All of `native_find_class_raised_condition`, `native_new_executable_no_parent`,
`security_manager_activation_boundary`, `security_manager_requires_installing`,
`live_frames_nested_primitives`, `interpret_frame_invocation_indent`, `method_package_rexx_library`,
`compile_failure_condition_origin` and `requires_install_levels` print
`out: oracle~base X oracle~head =`, `err: = =`, `rc: = =`.

Probe-agrees closings, re-run three-way, all three streams equal at base and at head:
`gate-9a-469/g1`, `compiled-method-package/t4c`, `t4d`, `rxmath/b1` to `b6` (with the oracle's
`build/lib`), `frame-invocation/fexe1`, `fexe3`. `frames/f1.rex`, which the native-method-frames
closing cites, differs from the oracle at base: the differing rows are the live-frames item's
(`run METHOD | RUN`, `sort ... SORT`, `eq ... HASITEM`, `makearray ... REQUEST`, `hash ... PUT`),
and the CALL, SENDWITH and SEND rows that item left open are equal at base, so the closing holds
(Minor 3).

Deadlock (`d2.rex`): the oracle over 122 sequential runs gave two classes (`054c3a2f` 39, the
interleaved pair; `72772f0f` 83, the waiter alone); 120 more runs six at a time gave the third
(`622997ce`, the started report then the waiter's) once. This crate under `REXX_SWITCH_MODE`
(`sim:1,fifo`, `sim:2..6`, `uniform:0.2`, `pct:3,k=40`, `uniform:0.5`) gave `622997ce` and
`72772f0f` only, both oracle classes. `g15.rex` likewise. Confirmed.

`queued-inventory.tsv` columns 4 and 6, and each queued file's first line, read `closed:5053fbd9b`
or `closed:probe agrees` with its probe for every closed item; `2026-10-22-user-raised-in-own-handler`
stays `9d` with its OPEN line. `t8.7b/closed.tsv` holds `gate:phase-9c-gate.md:144 closed:5053fbd9b`
and `gate:phase-9a-gate.md:469 closed:probe agrees`, in t8.7a's format; the gate lines are
`2026-10-17-newmethod-parent-package.md` and the external CALL under package TRACE R.

The `executable_at` fix: `d3.rex` (internal-call recursion to depth `n`, then a trapped 42.3 whose
`stackFrames` are counted), callgrind totals: base 239,102,410 Ir at 2500 and 1,039,932,771 at 9000;
head 186,924,561 and 355,382,890. Base grows 4.3 times over 3.6 times the depth, head 1.9 times.

## Findings

### Important 1: a primitive's live frame names the override that forwarded to it

`Interp::primitive_level` (`rust/crates/rexx-exec/src/dispatch.rs:401`) finds the method it lists by
`self.lookup(receiver, name, None)`, a lookup from the receiver's class. A subclass that overrides
`SORT`, `[]=`, `REQUEST` or class `NEW` and forwards to super reaches the primitive with that
receiver, so the frame's `executable` and scope are the override's. The oracle lists the primitive's
own method. Probe `t87b-review-scratch/f2/f2.rex` (`.myarr` subclass of Array with
`::method sort; forward class (super)`; `.mytab` subclass of Table forwarding `[]=`; `.myk`
forwarding `request`; a comparator, `hashCode` and `makefoo` printing the frame with no line):

```
oracle:   METHOD SORT array sort 1 myarr sort 0 ...        *-* Compiled method "SORT" with scope "Array".
head:     METHOD SORT array sort 0 myarr sort 1 ...        *-* Compiled method "SORT" with scope "MYARR".
oracle:   METHOD []= ... tab 1 0 ...        *-* Compiled method "[]=" with scope "Table".
head:     METHOD []= ... tab 0 1 ...        *-* Compiled method "[]=" with scope "MYTAB".
oracle:   METHOD REQUEST ... req 1 0        *-* Compiled method "REQUEST" with scope "Object".
head:     METHOD REQUEST ... req 0 1        *-* Compiled method "REQUEST" with scope "MYK".
```

and `rn/rn2.rex` (a Routine subclass whose class `NEW` forwards, the routine's `::REQUIRES`
prologue printing its frames' tracelines): oracle `*-* Compiled method "NEW" with scope "Routine".`,
head `... with scope "MYR".` Base listed no such frame. The trapped condition's traceback in the same
probe already names `scope "Array"` at both, so only the new live frames are wrong. Fix: look the
method up from the primitive's own defining class (the scope the native row is registered under)
rather than from the receiver.

### Important 2: a manager set during the main program's install reaches the main activation

The rule the report derives: an activation keeps the manager its package had when it began. The
oracle's main activation begins before its package installs (`RexxActivation::run` calls
`code->install(this)`, `RexxActivation.cpp:508`), so a manager set during the install is asked by
neither the main program's remaining `::REQUIRES` nor its commands. The new installing level
(`push_installing_level`, `install.rs:499`) makes the main package reachable from a required file's
prologue through `.context~stackFrames`, and two things then disagree with the rule:
`installing_security_manager` (`security.rs:148`) reads the package's manager as it is now, and the
main activation, pushed after the install, starts from the package's current manager, while the
installing level that kept the old one is already popped. Probe `t87b-review-scratch/sm1/`:
`main.rex` runs `address system 'exit 5'`, calls a `::routine`, and requires `a.cls` then `b.cls`;
`a.cls`'s prologue takes `f[f~items]~executable~package` (the main package) and sets a manager whose
`command` sets rc 42 and whose `unknown` prints its message.

```
oracle                     head
pkg main.rex               pkg main.rex
set                        set
b loaded                   MGR REQUIRES b.cls
                           MGR REQUIRES /.../sm1/b.cls
                           b loaded
main start                 main start
main rc 5                  MGR command exit 5
                           main rc 42
MGR command exit 6         MGR command exit 6
rt rc 42                   rt rc 42
```

stderr and rc agree. Base printed only one frame and `pkg a.cls` here, so it never reached the main
package. The shape is contrived, but it is the area the task closed and the answer contradicts its
own rule. Fix: give the main activation the manager its package had before its install began (as
`keep_security_manager` does for a live activation), and have a directive's `::REQUIRES` ask that
kept manager.

### Minor 1: `push_installing_level`'s doc gives one of its two `None` cases

`install.rs:498`: "`None` outside an install." It also answers `None` where `id` already has a
running activation (`install.rs:506`).

### Minor 2: a failing translation now builds and loads a program

`untranslated_origin` (`run.rs:3533`) copies the source's lines into a new program and
`load_program`s it for every failure with a name. `m1/m1.rex` (a routine trapping `.Routine~new`
of `say (`, `n` times): base 37,200 KB 0.10 s at 2000 and 37,436 KB 0.76 s at 20000; head 53,968 KB
0.15 s and 53,408 KB 1.22 s. Memory is bounded (flat over ten times the count), the time is a cold
error path at about 60% more. Recorded, not a request.

### Minor 3: the native-method-frames closing's probe is not equal at base

As above: `frames/f1.rex` differs at base in other items' rows. The closing line could say which rows
agree at base.

## Report concerns and Deviation candidates

1. **Primitive levels are per method.** Confirmed. `f3/f3.rex` (an object whose `==` prints the
   second frame): oracle `METHOD HASITEM` for List, Table and Queue `hasItem`, `METHOD INDEX` for
   `Array~index`, `METHOD REMOVEITEM`; head lists the frame for Array and Queue `hasItem` only, and
   the PROGRAM row for the rest; base listed the PROGRAM row for every one. The absence is now inconsistent between
   sibling collections; the closed item names only the shapes it fixed. Queue candidate.
2. **INTERPRET translation failure's PACKAGE.** Confirmed: `compile-condition/m2.rex` last line,
   oracle `35.1 m2.rex 0`, head `35.1 m2.rex 1`.
3. **The oracle's under-indent after a completed loop.** Confirmed as the known traceIndent defect:
   `t1/t1.rex` under `trace r` differs on stderr only at the `return` after a completed `do over`
   (`25 *-*       return` oracle, `25 *-*         return` base and head); stdout, including every
   traceline and traceback read inside nested INTERPRETs, equal at head.
4. **The installing level.** Probed: a trapped condition's frames and traceback in a required
   prologue (`i1`), an untrapped error two deep, `ADDRESS` and rc, `::OPTIONS TRACE` and `trace r`
   in required files with the frames' tracelines (`i2`), HALT and EXIT in a prologue (`i3`),
   `.Routine~new` with a failing `::REQUIRES` prologue (`rn/rn.rex`): all equal at head. The one
   reader that answers wrong is the security manager (Important 2).
5. **SIGTERM.** Measured on release builds: `do forever; nop; end` under `timeout 2` ends at 2.0 s
   with `Error 4.1: Program interrupted with HALT condition.`, rc 124, at base and head. So SIGTERM is
   delivered as HALT at the next clause boundary; a debug run inside the quadratic frame walk, which
   is one clause, never reaches one. Inferred for the debug run, not re-run.

Deviation candidate 1 (condition raised again in its CALL ON handler): confirmed; `u2` and `u3` are
equal on all three; rexxref `condtra.xml:441-461` reads as the report quotes. The probes raise from an
internal routine (`call rp`, `RAISE ... RETURN`) rather than in the handler itself; a direct `RAISE
USER` in the handler (`u4/u4.rex`) printed nothing at all, rc 0, on both interpreters, which I did
not explain. Worth settling which shape the ruling covers before Moritz rules.

Deviation candidate 2: confirmed as concern 2.

## Security manager per activation: what agrees

`sm2/sm2.rex` (a method that replies and waits on a guard while the main program sets a manager,
three runs): oracle and head `main rc 5`, `replied rc 9`, `replied inner rc 8`, `MGR command exit 7`
for a method begun after; base checked all four. `sm3/` (a routine replacing the manager mid-run then
calling another routine; a callback from another package's routine setting a manager on that
package, with commands before and after and in a further routine there; INTERPRET of a CALL): all
equal at head. `rq/rq2.rex` (`request` reaching a private `MAKE` from inside the object): equal at head.

## Perf

Own run, libc subtracted, base to head: startup 57,318,956 to 57,345,731 (+0.0467%), pingmsg
+0.0465%, sayloop +0.0252%, interpret +0.0074%, dirread -0.1279%, pkglocal -0.5655%; spreads at most
0.0023%. The implementer's table has startup 57,318,877 to 57,345,772 (+0.0469%) and pingmsg
+0.0449%. The startup delta is deterministic: +26,775 Ir here and +26,895 there, spread 0.0003%
against a 0.047% delta, from independently built binaries. Function self costs put all of it in the
hash put's new path: `primitive_level` (`hash_put`'s closure), `is_base_class`, `index_only`,
`MethodDict::slot` (+18,094, the level's lookup by name) and `system_lookup`, on the startup library
code's puts whose index is not of a base class. Within budget, with little margin; the lookup by
name is also what Important 1 replaces. Interpret's whole-process +0.4786% is `_int_malloc` (+23 M
Ir) with `malloc`'s own count unchanged (138.3 M Ir both), allocator layout rather than more
allocations; libc subtracted it is +0.0074%. Every head `.cg` in `final2/` names
`rexx-head-5053fbd9b` (99 of 99) and every base one `rexx-base-504ff0743`.

## Workspace test at HEAD

Exit 101. Failing targets: `rexx-api --test context`, `--test invoke`, `--test load` and most
`rexx-exec --lib` library and scheduler tests, all panicking on a missing `build/lib`
(`the worktree's build/lib is three directories above this crate`, `the extension publishes
RexxGetPackage`): `build/` was linked into the worktree only after they had run. Re-run with it:
the three `rexx-api` targets `14 passed`, `16 passed`, `11 passed`, exit 0; `--lib` `1201 passed;
1 failed`, the failure `a_move_across_file_systems_copies_and_unlinks`, which asserts its two
directories are on different file systems and here both were on `/tmp` (`Invalid cross-device
link` expected, `Ok(())` got): my worktree's location, not this task. `datetime_zone`'s
`the_file_date_time_witness_reads_the_zone_the_oracle_does`: the known DST flake. Every other target
passed, including `rexx-parse` (`tiling`, `variants`) after the `perr.cls` rename.

## Queue candidates

* Primitives that reach Rexx code with no live frame: List, Table `hasItem`, `Array~index`,
  `removeItem` (concern 1, `f3/f3.rex`).
* A hash collection's lookup sends `==` to the identical index object; the oracle compares identity
  first. `idq/idq.rex` (a `.k` with a constant `hashCode` and a printing `==`): `t[k] = 2` and `t[k]`
  after `t[k] = 1` print `== called` once at base and head and never on the oracle. Predates the task.

## New oracle defects

None.
