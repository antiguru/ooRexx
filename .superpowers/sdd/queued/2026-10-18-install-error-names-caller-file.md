**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** gap.

**OPEN: found by Phase 9d Task 6.6, not yet ruled.** An install error in a called program names the caller's file in `Error N running <file>`.

# An install error in a called program names the caller's file

Found 2026-10-18 by Phase 9d Task 6.6 (`task-6.6-report.md`, Concerns). When a program reached by
`CALL` or `::REQUIRES` fails to install, the crate's `Error N running <file>` line names the
calling program's file, where the oracle names the called one. Probes:
`docs/superpowers/records/2026-10-18-phase-9d/t6.6/probes/superclass/superclass_e`,
`superclass_g` and `superclass_l`. `superclass_l` is a plain 98.909 that differs at Task 6.6's
parent commit too.

The Task 6.6 re-review (`task-6.6-rereview.md`, Minor 3) found a likely second instance. An error
in a required file's `::CONSTANT` expression names the requiring file: with main
`say .c~k` / `::requires "r.cls"` / `::options novalue syntax digits 4` and `r.cls` holding
`::class c public` / `::constant k (1/3 + nv)`, the oracle says `Error 41 running DIR/r.cls line 2`
and the crate says `DIR/main.rex line 2`. The traceback and rc 215 match. `.routine~newfile` and
inheriting variants show the same.

The round-2 re-review (`task-6.6-rereview-r2.md`, finding 2) adds an executable's own constant
error. With `.routine~new("x", ...)` holding `::constant k ('a' + 1)`, the oracle says
`Error 41 running x line 4` and the crate `Error 41 running DIR/main.rex line 4`. With
`.routine~newfile("r.rex")` the oracle names `r.rex` and the crate `DIR/main.rex`.

The Task 6.11 review (`task-6.11-review.md`, Minor 4) adds a class cycle inside `.Package~new`:
the error names the main file instead of the package.
