# A loaded file's prologue sees no frames for the levels that loaded it

Found by Phase 6.1 Task 5 (a KNOWN GAP row in `docs/superpowers/plans/phase-4-exclusions.txt`, "A PROLOGUE'S LIVE STACK FRAMES LACK THE LEVELS THAT LOADED IT", NO OWNER) and widened by the Task 5 re-review (`.superpowers/sdd/2026-10-07-phase-6-1/task-5-rereview.md`: nested and `newFile` shapes miss the intermediate frames too). Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both.

Probe `frames.rex`, run from a fresh empty directory:

    p = .Package~new('frames/p.cls')
    say '=='
    m = .Method~newFile('frames/mid.cls')

with `p.cls`:

    do f over .context~stackframes; say f~type f~name f~line; end
    say '--'

Oracle, rc 0:

    [stdout]
    ROUTINE frames/p.cls 1
    METHOD NEW The NIL object
    PROGRAM frames/frames.rex 1
    --
    ==
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    ROUTINE frames/p.cls 1
    PROGRAM frames/frames.rex 1
    --
    ==
    [stderr]
    (empty)

Probe `frames2.rex`, run from a fresh empty directory:

    call 'mid2.cls'

with `mid2.cls`:

    return
    ::requires 'q.cls'

and `q.cls`:

    do f over .context~stackframes; say f~type f~name f~line; end

Oracle, rc 0:

    [stdout]
    ROUTINE frames/q.cls 1
    ROUTINE mid2.cls 2
    PROGRAM frames/frames2.rex 1
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    ROUTINE frames/q.cls 1
    PROGRAM frames/frames2.rex 1
    [stderr]
    (empty)

Suspected site: the frames pushed when a prologue runs (`install.rs`, `Package~new` and `::REQUIRES` loading): the oracle has a `METHOD NEW` frame and the requiring file's `ROUTINE` frame.
