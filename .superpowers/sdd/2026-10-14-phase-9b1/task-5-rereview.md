# Phase 9b1 Task 5 fix round 1: re-review

Reviewed `8294a7908..04e57a918` (fix code `728d715da`); the tree at `f7988619e` adds only the
comment on `Activities::running_programs`. Probes ran on a `git archive` of `f7988619e` in the
scratchpad (`rr/`), through a throwaway `examples/stress.rs` calling
`run_program_collect_every_alloc` (debug and release), each from a fresh empty directory under
`memcap 4G`. Oracle: the standard wrapper with a 3000-pass `copies('y', 100000)` filler. Crate
filler under stress: `do zi = 1 to 3; zz = 'f' zi; end`. No edits, no commits.

## Finding verdicts

- **I1 (external call merge frees the callee's package)** -- ADDRESSED for the reported shapes.
  `lib.rs:2340`-`:2350` builds the caller's and callee's packages before `run_loaded` and roots
  them. p09e, p01, p06, p10 match the oracle in debug and release under collect-every-allocation
  (`p09e v1 dk`, `p01 1 1 7` twice, `p06 1 7`, `p10 1 1 7`, all rc 0). The two extra holders hold:
  `load_requires` roots a cache hit (`install.rs:634`-`:643`) and `package_from_source` builds and
  roots before the prologue (`install.rs:1004`). **The grep for further holders found one that the
  fix missed: see Open finding 1.**
- **I2 (waiting-activity shape unpinned)** -- RULED (kept as defence in depth, unwitnessed, with
  the comment at `scheduler.rs:409`-`:414`). Not re-litigated. p10 stays green.
- **I3 (continuation branch unwitnessed)** -- ADDRESSED. The branch is deleted
  (`activity.rs:489`-`:507`). `spawn_continuation` (`dispatch.rs:3381`-`:3410`) moves the replying
  activation into the continuation's `running` and the replying activation stays on the original
  stack until then. p05c and p11 (a `setMethod` method that does `reply 1`, is unset, drops, and
  waits on a semaphore; p11 builds the package in an internal routine) print `1 1 1 7`, as the
  oracle does, in debug and release under collect-every-allocation. Note: `setMethod` is private, so
  the probes install through a `::class box` method.
- **I4 (release fallback rebuilds silently)** -- ADDRESSED for `package_object`, NOT for the
  `held_package_object` readers.
  - `package_object` (`identities.rs:174`-`:187`) returns `Err(Loud::unknown_receiver("a package
    whose object is no longer live"))` after the debug assert. Seen live in release:
    `rexx-exec: a message send to a package whose object is no longer live is not implemented`,
    rc 120.
  - The ALLOWED row (`representation_refusals.rs:117`-`:122`) is tagged as an internal guard, Task 8.
  - Readers of `held_package_object`: `rexx_package_object` (REXX package is a global root:
    unreachable), `enter_library_program` (a fresh program id, never built before: unreachable),
    `record_constant_value` (the package is built at `install.rs:129` before any `::CONSTANT`
    runs, rooted for the install: unreachable), `build_package_string_table`'s `expect` (copies
    build their shares at copy time, so no entries remain to build: probed with copies of a
    package with a routine, a resource and a method, h5 and h6, match the oracle),
    `program_string_table` from `rexx_variable` (running program) and `context.rs:480` (a live
    frame): unreachable. **`program_string_table` from `routine_object`
    (`environment.rs:1262`) is reachable with a dead package: Open finding 1, release panic.**
- **Minors** -- ADDRESSED. `copy_class_package` drops the unobservable branch with a one-line
  reason (`identities.rs:77`-`:80`); `object_roots` has its doc comment back and
  `running_programs` has its own (`activity.rs:489`-`:513`).

## Open finding 1 (Important): an external callee's public routines are merged into the caller, and nothing holds the callee's package

`enter_external_program` roots the callee's package as a temp of the *calling frame*
(`lib.rs:2345`-`:2347`). When that frame is popped, nothing holds the package, but the caller's
package keeps `MergedRoutine::Installed { program: callee, .. }` for each public routine
(`install.rs:361`-`:366`, `lib.rs:1752`) and no `add_imported_package(into, from)` (compare
`install.rs:594`-`:595`, the `::REQUIRES` path). Classes are fine (a class holds its package,
e2 matches). A later call of the routine runs in the dead program, and `findRoutine` goes
through `routine_object` into `held_package_object`.

```
-- main.rex (e5)                        -- ext5.rex
call sub                                return 1
do zi = 1 to 3000                       ::routine pubr public
  zz = copies('y', 100000)                return .context~package~name~right(8)
end
say 'e5' pubr()
exit
sub: procedure
x = ext5()
return
```

Oracle: `e5 ext5.rex`, rc 0. Crate, plain `rexx-run` release at the fix head, no stress mode:
`rexx-exec: a message send to a package whose object is no longer live is not implemented`, rc 120.
Under collect-every-allocation, with `say 'e6' .context~package~name~right(8) pubr()` in main, the
debug build panics at `identities.rs:178` and release gives the same rc 120. The same program run
on the Task 4 binary (`rexx-run-9b1-t4b`, `-t4c`) and on the Task 5 review binary
(`rexx-run-9b1-t5-review`) prints the oracle's line, rc 0: Task 5 made the package collectable, and
the first version's silent rebuild masked the missing holder. The I4 ruling made it loud, which
is correct; the holder is still missing.

Reachable panic (e7, plain release, 3000-pass filler, same ext file):

```
call sub
<filler>
r = .context~package~findRoutine('PUBR')
say 'e7' (r == .nil)
...
```

Oracle: `e7 0` then `ext7.rex`. Crate: `panicked at identities.rs:161:23: Program(ProgramId(4))'s
package was collected while it was held`, from `routine_object` (`environment.rs:1262`) ->
`program_string_table` (`environment.rs:1093`). So the reader's "only ever meets a held package"
claim is false for a merged external routine. Fix: have the caller's package hold the callee's
(`add_imported_package(into, from)` where it merges), then re-check this with a test in
`collect_stress.rs` (`e5`, with an external file, real filler or an allocating one: the crate's
`'f' zi` filler allocates nothing because the strings are inline, so it never collects here;
the e6 shape does). Measure extcall after the change.

## New breakage in the fix diff

Open finding 1 is the only one: the fix did not add the holder, and the loud fallback turns it
from a silent wrong package into rc 120 / a panic. Nothing else: the `Result` plumbing is
mechanical, `into` is pushed twice when it was freshly built (harmless), and the callgrind figures
in the report (extcall -13.26% vs pin) were not re-run.

## Out-of-scope observations

None.

## Verdict

**Fix round:** findings remain open -- Open finding 1 (Important): the callee's package of an
external call is not held by the caller that merged its public routines; plain release run rc 120
vs oracle rc 0 (e5), and a release panic via `Package~findRoutine` (e7). I1's reported shapes, I3,
and the I4 `package_object` half are resolved.
