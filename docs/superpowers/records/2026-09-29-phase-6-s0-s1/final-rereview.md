# Final fix round re-review (70030c6ed..1a81353e3)

Verdict: all items Addressed; no new problem found. Read-only; built `1a81353e3` (release `rexx-exec` bin `rexx-run`), probes under `memcap 8G`.

## Rulings

- A1 Addressed. `run_loaded` wraps `run_activation` in `pinned!(self, nested.then_some(PinKind::Program), ...)` (lib.rs diff); `nested` is `running_activation().is_some()`. `FRAME_PROBES` has the `Program` case; `report_of` now runs in a directory holding `extf.rex`. Pinning doc has Program and Delegate rows. (Report evidence of 70030c6ed `[]` vs head `[Program]` taken from the fix report's red/green files, not re-run, per instruction not to re-run gates.)
- A-B1 Addressed. Check is in `run_loaded` before `install_directives` and before any push. Probe: 9000 x (`ext('ret')`, `ext('sig')`, `ext('exit')`) then in-program recursion: depth 9990 and 9995 answer `ok`; depth 10005 raises `11.1` (caught by SIGNAL ON SYNTAX), rc 0. A Rust test (`spike.rs`) pins 11.1 at 9998 for self-recursing `extf`.
- B1-B10 Addressed. Each diff hunk matches the fix text in final-review-b.md; the code-side edits (B5-B8) touch only comments/docs; D-U2 citations `:86 :95 :157 :168` match the review's own list.

## Requested checks

1. Depth release. `activation_depth()` is `suspended.len() + running`, derived from the stack, and `run_loaded` has no `?`/return between `push_activation` and `pop_activation` (the only early returns, Err at the cap/install failure and `Ok(None)`, precede the push). Probes, all with 9000 non-nested external calls then in-program recursion to 9000: normal return, error unwound to a trap in an internal routine (`safe`, 9000 times), EXIT in callee, SIGNAL out of callee's select into its own label, in-callee SYNTAX trap (`ext2`): each prints `loops done` / `deep ok`, rc 0; oracle prints the same.
2. `::REQUIRES` at start: `nested` is false (no running activation), so no pin and no count. Probes `::requires` of a library, and a library that itself requires a library (rq3), run correctly (`hi`); the program's own external call after it works.
3. Prose in diff: no false, forward-looking or cardinality sentence found. The new `lib.rs` comment ("bounded by the same count") is true.

## Not checked
Oracle comparison for the depth-cap probes (oracle depth differs, licensed). Pinning `[Program]` on head not re-run.
