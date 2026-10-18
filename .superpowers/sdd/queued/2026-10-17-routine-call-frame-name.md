**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** probe still differs: ROUTINE MYNAME on the oracle, ROUTINE CALL here.

# Divergence: an activation entered through `Routine~call` is named `CALL`

Found 2026-10-17 by Phase 9c Task 9 and its review. Ruled by the controller in
Task 9's fix round 1: queued, not fixed in the restore task.

## The witness, from source

    r = .Routine~new('MYNAME', .array~of("f = .context~stackframes[1]", "return f~type f~name"))
    say r~call
    say .Routine~newFile('sf.rex')~call

where `sf.rex` holds the same two lines. Fresh empty directory, 1 run a side:

| side | rc | stdout |
|---|---|---|
| oracle | 0 | `ROUTINE MYNAME`, `ROUTINE sf.rex` |
| here | 0 | `ROUTINE CALL`, `ROUTINE CALL` |

The review's transcripts are in the Task 9 review's probe directory
(`rev9probe/oracle.txt` and `crate.txt`, `sfname.rex`). A traceback does not show
it for a source routine, because the clause is shown in place of the name. Task 9
also measured the frames of `x = .Routine~newFile('sfn.rex')~call`, where `sfn.rex`
prints each frame's type and name: the oracle lists `ROUTINE sfn.rex`,
`METHOD CALL`, `PROGRAM <path>`, and this crate lists `ROUTINE CALL`,
`PROGRAM <path>`, so the native `CALL` method's own frame is missing too.

## The restored-routine case

A restored routine has no source, so its traceback line names the routine.
`.Routine~newFile('fail.rxo')~call`, where `fail.rxo` is a `rexxc` image of a
program that raises 41.1 on its line 2:

    oracle:     2 *-* Routine DIR/fail.rex in package "fail.rxo" (no source available).
    here:       2 *-* Routine CALL in package "fail.rxo" (no source available).

Transcripts: `docs/superpowers/records/2026-10-15-phase-9c/task-9-step1/routine-object/`.
It is left out of `rust/crates/rexx-exec/tests/compiled.rs`, which asserts
identity with the oracle.

## The oracle's sites

* `RoutineClass::callRexx` runs the code under `executableName`
  (`interpreter/classes/RoutineClass.cpp:209`, also `:227` and `:248`), so the
  activation's message name is the routine's own name, not the message `CALL`.
* `executableName` is flattened with the routine (`RoutineClass.cpp:148`,
  `flattenRef(executableName)`), so a restored routine keeps the name `rexxc`
  gave it: the full path of the source it translated.
* `ProgramMetaData::restore` renames only the package (`ProgramMetaData.cpp:190-191`,
  `setProgramName`), never the routine.

## What a fix needs

* The activation a `Routine` object's `~call` (and `~callWith`) enters takes the
  routine's name, which `Activation::invoked_as` then answers for `StackFrame~name`
  and for the sourceless traceback line (`Interp::sourceless_line`).
* The native `CALL` method's frame, if the frame list is to match.
* Then the image must carry the routine's name: this crate's `.rxo` holds only
  the header and the source (`rust/crates/rexx-exec/src/compiled.rs`), so a
  restored routine has no name of its own to show until `rexxc` writes one, and
  that is a format version bump.

Owner: none yet. Phase 9c records it as a known gap in
`docs/superpowers/plans/phase-4-exclusions.txt`, beside Deviation 41.
