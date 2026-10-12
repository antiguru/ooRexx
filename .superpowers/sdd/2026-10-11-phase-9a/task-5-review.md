# Phase 9a Task 5 review (BASE 6a8c461fd, HEAD d2f89f559)

**Spec verdict: PASS.** Every Step 1 probe item is implemented, or left as a refusal under the
controller's Task 5 rulings (the five non-mechanical items moved to Task 8b). Step 3's SUPPLIER site
was reached, so it is implemented. `Message~notify` was not reached, so it builds
`Loud::environment_symbol(.., None)` with no phase, under a GUARD row that names nt1, nt2 and info.
The tables are refreshed. The per-task groups ran. The path-valued and `bin` layout rulings are
implemented as ruled.

**Quality verdict: PASS WITH FIXES.** I1 to I3 below need fixing.

## What I checked, and how

* **Witnesses against the oracle.** I built `rexx-run` at HEAD into
  `/tmp/claude-1000/p9/t5-review-target`, which is now removed. I ran all five `phase-9.txt`
  witnesses once on each side, each from its own fresh `mktemp -d` under
  `/tmp/claude-1000/p9/t5-review-probes/`. `oracle-crashes.txt` was read first. All five give rc 0
  on both sides, with stdout and stderr byte-identical. I read each stdout. Each witness reaches the
  paths it names: S4 COMPILE answers `.nil`; `INTERNALCALL The Method class 1 1` and `RR
  INTERNALCALL ROUTINE 1 1` exercise the `executable_at` InternalCall arm; the unknown witness's
  `s.~unknown('POS','b')` = 2 shows a string really converts.
* **Callers of the shared `arrayArgument`.** I wrote `/tmp/claude-1000/p9/t5-review-probes/callers.rex`.
  It sends five values through sendWith, startWith, run, Message~new, callWith and both Supplier~new
  positions: an Object, a MAKEARRAY answering an array, a MAKEARRAY answering `.nil`, a string, and
  a 2-D array. The two sides agree on every line except two (I1). The stderr also differs once (the
  last bullet under Minor).
* **Cost of the `.context~executable` fix.** I wanted to know whether the new recursion over
  internal-call chains, run for every live frame on every condition, blows up. It does not. The
  crate alone (oracle entry 20 forbids the oracle run) took 0.09 s for a trapped SYNTAX 3000
  internal calls deep (3001 frames), 0.14 s for unbounded internal recursion to 11.1 (rc 245), and
  0.13 s for `lang/call_depth_cap_call.rex`.
* **Stale hard link.** `interpreter_bin` keeps `bin-rexx/rexx` only when its (dev, ino) equals the
  current `rexx-run`'s. A rebuild gives the binary a new inode, and a copy never shares one, so a
  stale binary cannot run. I list every `interpreter_bin` caller's run directory: each is unique
  per pid and counter or per unit, so no two threads link into one `bin`.
* **`check_path`.** It still refuses a foreign ELF, because any bytes other than the crate binary's
  are refused. The fixture still asserts that a `rexxc` copy is refused.
* **`PATH_VALUED`.** It covers exactly two rows, `RexxInfo executable` and `libraryPath`, in both
  `method_bodies.rs` and `support/arity.rs`. A wrong existing path would pass the table probe, but
  the corpus witness also tells a file from a directory.
* **Pins of deleted refusals.** `git grep` finds the deleted refusal texts only in the 9a baseline
  records. No test or table pins them.

## Critical

None.

## Important

**I1. `Supplier~new` now gives a wrong answer where it used to refuse.**
`dispatch/collection/supplier.rs:196-199` converts with `request_array`, then hands the *converted*
value to `array_argument`. Now that `array_argument` raises 98.913 instead of refusing, it renders
that converted value. Failure scenario, measured with probe `callers.rex`:

```rexx
signal on syntax
say .supplier~new(.kn~new, .array~of(1))~item
exit
syntax: say condition('O')['MESSAGE']
::class kn
::method makearray; return .nil
```

* Oracle: `Unable to convert object "a KN" to a single-dimensional array value.`
* Crate: `... object "The NIL object" ...`

The second Supplier~new position behaves the same way. At BASE this was a loud refusal; at HEAD it
is a silent stdout divergence.

Fix: drop the `request_array` step and let `array_argument` convert the original value. It needs a
form that answers the converted array as well as its slots, because the supplier keeps the array.

**I2. The change to the shared `arrayArgument` path is witnessed for only one of its other callers.**
`lang/unknown_argument_list.rex` says it covers "the other arrayArgument sites". It exercises only
`sendWith`, and only the success path. The failure paths of sendWith, startWith, run, Message~new
(both the `construct.rs:135` and the `object_protocol.rs:1646` routes), callWith (both
`executable.rs` sites) and Supplier~new used to refuse and now raise 98.913 or 88.913. Nothing in
the corpus covers them. Without I1's probe, the Supplier~new divergence would have shipped unseen.

Fix: add `callers.rex`, or an equivalent, to `phase-9.txt` once I1 is fixed. Every other line of it
already agrees with the oracle. Then narrow the witness header to what the witness runs.

**I3. Two refusal-table sentences became false in this task.**
`Method~copy` now answers, and the `lib.rs` doc on `unknown_receiver` was corrected to match. Two
copies of the old claim were not:

* `rust/corpus/refusal-dispositions.tsv:66` still says "Task 7 probes rc4 (a Method copy refuses at
  COPY first)".
* `rust/corpus/refusal-sites.tsv:323` still gives the reason "a copy of a Method object refuses at
  COPY before it can be passed on".

The GUARD disposition's evidence now asserts behaviour the crate no longer has. Delete both clauses,
per the prose rule.

## Minor

* **An unmatched `REXX_WHOLE_GROUPS_ONLY` passes green.** At `concurrency_tests.rs:3539`, a name that
  matches no group file (a typo, or a missing `.testGroup` suffix) leaves `files` empty. Nothing
  runs and the test still passes. Assert that every named group was found.
* **The copy fallback in `interpreter_bin` can truncate the cargo binary.** At
  `support/group_runner.rs:2807-2809` (diff lines) the `remove_file` error is ignored. If the remove
  fails and `rexx` is already a hard link of the same inode, `fs::copy(ours, &rexx)` truncates
  `target/release/rexx-run` itself. No caller can reach this today (run directories are unique).
  Not ignoring the remove error, or never copying onto an existing file, closes it.
* **`check_path` reads both copies of the binary every iteration.** It reads the whole ~58 MB binary
  twice per crate run (`suite.rs:2589-2594`). Comparing (dev, ino) first, and reading bytes only for
  a copy, would avoid that.
* **The `frame_executable` doc names the wrong set of refusing frames.** Frames built by
  `capture_site_frame` (`lib.rs:2410`, `install.rs:1094`, `install.rs:2556`) and by `run.rs:3371`
  pass `None`, so `~executable` refuses on them as well. They never call `executable_at`, so the doc
  sentence at `context.rs:1509-1512` ("nor one [`executable_at`] could not answer") misdescribes
  them, and the report says only native frames refuse. The refusal is loud, so this is about prose.
* **Each `Method~copy` grows side tables that are never pruned.** Every copy mints a `MethodId` and
  adds entries to `executable_objects`, `method_bodies` and the other side tables
  (`identities.rs:2100-2111`). Those tables are weak, so this is safe, but they are never pruned:
  `do forever; c = m~copy; end` grows memory without bound. `newScope` and `define` already work
  this way; this task makes a plain `~copy` reach it.
* **Groups this task unblocked were not run.** The `RexxInfo~executable` refusal used to block
  `base/class/RexxInfo`, `base/bif/CHAROUT`, `base/class/Stream`, `base/keyword/ADDRESS`,
  `base/source.file/SourceFile` and `base/keyword/VarRef` (source: `suite-baseline/oracle-differences.tsv`).
  None of them went through the harness. The brief named only six groups, so this is not a spec
  miss. But `TEST_REXXINFO_EXECUTABLE` asserts `~name` is `rexx`, which only the hard-link ruling
  makes true, and that is unverified.
* **Not this task's: startWith's error report.** In the startWith error report, the oracle's first
  report reads `Error 93 running FILE line 18:` and the crate's reads `Error 93:`. That path, a
  MAKEARRAY answering an array, is unchanged by this task, so the difference predates it.
* **Prose.** No em-dashes in added lines. No set sizes and no future mechanisms in the added
  comments or records that I could find.
