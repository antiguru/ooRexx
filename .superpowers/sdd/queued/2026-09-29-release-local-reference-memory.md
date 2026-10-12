# Queued: `ReleaseLocalReference` frees nothing until the call ends

Found by the Phase 8 final review (M2, `scratchpad/review-final8/task-9-review.md`); ruled
"do not fix, queue" in the final fix round (progress.md, "Ruling (final fix round)"). Not a
Phase 8 exit item.

## What happens

Since Task 9 (`da17061ba`), `ReleaseLocalReference` keeps the handle resolvable and rooted until
the native call ends, so that an extension returning a released local reference is answered as
the oracle answers it (`a_released_local_reference_still_answers`). No collection runs inside a
native call either. So an extension that allocates and releases in a loop holds every object
until it returns. The oracle's release drops the object from the activation's save list
(`NativeActivation::removeLocalReference`, `interpreter/execution/NativeActivation.cpp:1219`),
and its next collection takes it.

## The probe

A forged routine. It NEEDs no `librexx.so` or `librexxapi.so` and has no undefined Rexx symbol.
It creates and releases a heap-sized string on each pass:

    #include "oorexxapi.h"
    RexxRoutine1(RexxObjectPtr, RelLoop, int, n)
    {
        for (int i = 0; i < n; i++) {
            RexxObjectPtr s = context->String("some string that is long enough to be boxed on the heap");
            context->ReleaseLocalReference(s);
        }
        return context->String("done");
    }
    RexxRoutineEntry routines[] = {
        REXX_TYPED_ROUTINE(RelLoop, RelLoop),
        REXX_LAST_ROUTINE()
    };
    RexxPackageEntry relloop_package_entry = {
        STANDARD_PACKAGE_HEADER
        REXX_INTERPRETER_4_0_0,
        "relloop", "1.0.0", NULL, NULL, routines, NULL
    };
    OOREXX_GET_PACKAGE(relloop);

Built with `g++ -shared -fPIC -O1 -static-libstdc++ -I<oracle>/api -I<oracle>/api/platform/unix
relloop.cpp -o librelloop.so`. `readelf -d` lists no NEEDED entry at all, and
`nm -D --undefined-only | grep -i rexx` prints nothing. The program, run from a fresh empty
directory with the library's directory on `LD_LIBRARY_PATH`:

    say RelLoop(2000000)
    ::requires 'relloop' LIBRARY

## The figures

Measured 2026-09-29 at `c0a12eed8` (no change to this path since `81bfd20c7`), release
`rexx-run`, `/usr/bin/time -f %M` run on the interpreter binary itself, not through a wrapper:

| side | cap | passes | maxrss | outcome |
|---|---|---|---|---|
| oracle | `ulimit -v 1048576` | 2,000,000 | 20296 KB | `done`, rc 0 |
| crate | `ulimit -v 4194304` | 1,000,000 | 224648 KB | `done`, rc 0 |
| crate | `ulimit -v 4194304` | 2,000,000 | 429752 KB | `done`, rc 0 |
| crate | `ulimit -v 1048576` (the oracle's cap) | 2,000,000 | 513144 KB at the abort | `memory allocation of 55 bytes failed`, SIGABRT |

The review measured the same program at `81bfd20c7`: oracle 20104 KB, this crate 430068 KB, and
348032 KB at Task 9's BASE `2ae06085c`. Most of the growth predates Task 9, because nothing
collects inside a native call. Task 9 adds about 40 bytes for each released handle.

## Whether it is licensed

This is arguably inside the OOM licence: behaviour under memory pressure is best effort, and no
ordinary path pays to match the oracle there (`docs/superpowers/plans/2026-08-17-phase-5a.md:274`). The ordinary path here is
an extension looping inside one call, which the oracle runs in constant memory. A fix would let
a collection run inside a native call and let a released handle go, while still answering a
released local that the call returns.
