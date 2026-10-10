# Task 11a fix round 1 (controller)

Read `task-11a-review.md` in this directory. Its probes are in `/tmp/claude-1000/p61/t11ar/probes/`. Fix each item below with its own commit and test. The carry's rules still hold:
* run mutants in the tree and restore them by an exact edit, never with checkout, stash or reset;
* the per-task check includes the feature clippy;
* use callgrind on the eight programs against base61 if any hot path changes;
* stop and message the controller if a program goes over +0.5%.

* **I1, fixed in this round.** `directive_class` must resolve a `::CLASS ... SUBCLASS` superclass through the installing package's own classes and then its parent chain, in the order of `findClass`. Witness `routsub.rex` against the oracle in 3 runs. The reviewer found the same mechanism in two more routes: `.package~new(name, src, ctx)`, and a `::CLASS` in a file reached by an external `call`. If one fix at `directive_class` covers them, add a witness for each. If either needs a separate site, message the controller before touching it.
* **I2.** Add a test that fails when the `imported_class` parent arm is disabled, using the `a2/req.rex` shape (a parent that got its public class through `::REQUIRES`). Show it go red under the mutation.
* **I3. Ruling:** put the seven sites in the `.nil` set: `signal value`, stem tail, arithmetic, `numeric digits`, `do` count, host command and `interpret`. Where the oracle's message names the original object ("a SN" in 41.1, 26.5 and 26.2), keep the original object. Each row gets a witness that matches the oracle in at least 2 runs. The principle stays R11 narrowed: refuse only where the oracle reads `.nil` through the string layout, giving garbage, a crash or 88.909 through REXX-package frames. Correct Deviation 30, the R11 line in the spec and the report's claim about which sites read bytes, so that none of them says every other consumer reads garbage. Commit the enumeration command and its full per-site classification table to the report.
* **I4.** Check NOSTRING before the refusal in `reqstr.rs`. If NOSTRING is trapped, raise it with `The NIL object` as the description, and refuse only when it is not trapped. Witness `b3/ns_sigbytes.rex`.
* **m1 and m2:** no change. The controller records both.

When done, append a `## Fix round 1` section to `task-11a-report.md` and run the full per-task check, `whole_groups` and the seeded gate in release, the corpus pair, and callgrind if any hot path changed. Commit, then return only the status, the commits, the test line, the perf line and any concerns.
