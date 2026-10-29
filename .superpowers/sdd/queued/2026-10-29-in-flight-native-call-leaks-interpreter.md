**OPEN, queued 2026-10-29 by the controller from an external (codex) review of 9af69058d; Moritz 2026-10-29 "lgtm" to the placement: proposed after 9d (needs a reproducing program first).**

# A run ending with a native call in flight leaks its interpreter

`rexx-exec/src/lib.rs:4404` `mem::forget`s the interpreter when `activities.in_flight()`, and a native call that later returns waits for a baton (`dispatch/library.rs:1346`) that no driver will lend. Repeated embedded runs ending this way would keep heaps and blocked workers. First find a program or API sequence that ends a run with a call in flight and measure what it keeps; then give the interpreter to a shutdown owner that services the outstanding completions.
