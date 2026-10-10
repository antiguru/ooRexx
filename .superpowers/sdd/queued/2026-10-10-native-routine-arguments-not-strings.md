# Native routine arguments that are not strings are rendered, not refused

Found by the Phase 6.1 final review (finding 2, `probes/s2/native_routine_args.rex`, `probes/s3/s3_62.rex`, `s3_69.rex`); pre-existing at the 6.1 base. Queued by the Phase 6.1 final fix (2026-10-10). Silent: rc 0 here where the oracle raises 88.909.

Neither engine sends `STRING` for a native routine's argument, so this is not an R11 site. The oracle converts with `requiredString(position)` and raises 88.909 through the routine's frame; this crate renders the object's default name or reads it as a missing file. The review found the same answers on base61.

Probe `natargs.rex`, run from a fresh empty directory:

    o = .object~new
    say 'filespec' filespec('name', o)
    say 'sysfileexists' sysfileexists(o)
    say 'directory' directory(o)
    say 'sysfiletree' sysfiletree(o, 'f.')

Oracle, rc 168:

    [stdout]
    (empty)
    [stderr]
           *-* Compiled routine "FILESPEC".
         2 *-* say 'filespec' filespec('name', o)
    Error 88 running REXX:  Invalid argument.
    Error 88.909:  Argument 2 must have a string value.

This crate (`rexx-run` built from the final fix's finding 1 tree), rc 0:

    [stdout]
    filespec an Object
    sysfileexists 0
    directory 
    sysfiletree 0
    [stderr]
    (empty)

`say 'directory' directory(o)` alone: oracle rc 168, `Compiled routine "DIRECTORY"`, 88.909 Argument 1; this crate rc 0, `directory `.

Suspected site: the native routine bodies' argument reads, `builtin/platform.rs` (`directory`, `filespec`) and `builtin/rexxutil.rs` (`file_exists`, the `SysFileTree` body), which call `Interp::to_text` where the oracle calls `requiredString(position)`.
