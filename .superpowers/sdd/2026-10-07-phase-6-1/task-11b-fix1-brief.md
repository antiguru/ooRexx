# Task 11b fix round 1 (controller)

Read `task-11b-review.md`. Its probes are under `/tmp/claude-1000/p61/t11br/`. The carry's rules still hold. Commit each fix with its test.

* **I1.** Add the two drain points the brief's rule names: an INTERPRET returning, and a native activation returning (`NativeActivation.cpp:1361`). Native activations here include builtins such as SAY, LINEOUT and LINEIN, and Stream methods. Cite the oracle's exact sites, and copy the oracle's placement, not more. Show that each of the reviewer's I1 probes reaches bounded memory and matches the oracle's `done` order of magnitude. Native returns are hot, so keep the check to one predictable test with an out-of-line `#[cold]` drain. Run callgrind on the eight programs against base61. If any program goes over +0.5%, stop and message the controller with the cgdiff.
* **M1.** Add a test that fails when the pool's shrink back to its bound (`pool.rs:262`) is disabled. Show it going red.
* **M2.** No change. The controller queues it, because it predates the task.
* **M3.** Make `Activation::object_roots` fail to compile when a new field is added to the cold part, for example with an exhaustive destructure (`let Cold { a, b, .. }` without `..`). Only do this if it costs nothing at run time. If it does not fit, say why.

Append a `## Fix round 1` section to the report. Run the per-task check, the corpus pair, and whole_groups plus the seeded gate in release. Return the status, commits, test line, perf line and concerns.
