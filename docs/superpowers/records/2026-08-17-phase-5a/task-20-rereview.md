# Task 20 re-review: fix round 1

Range `196da092d..4a519518d`. Probes from fresh directories under the session scratchpad, absolute
paths, three descriptors read separately, both sides bounded, both engines.

## Verdict

**CLOSE.** No load-bearing defect survives. All ten findings are addressed by a change I could
check against the tree rather than against the report, and the F5 control fails when inverted.

## What I checked, and what each rests on

**F5, the fix.** `staged` and `attach_directive_annotations`' per-name map are `BTreeMap`s at
`lib.rs:4232`, `:4505` and `:4520`. My own twelve-run sweep on the shipped release binary, the same
six-table `~identityHash` program, is **one distinct output over twelve on `ir` and one over twelve
on `tree-walker`, and the two engines agree**, against ten distinct outputs over twelve before.

**F5, the control, inverted live.** `cp` of `lib.rs` to the scratchpad, sha256
`680031f0...31432`; `sed` of the three `BTreeMap` sites back to `HashMap`;
`cargo test -p rexx-exec --lib two_runs_in_one_process_allocate_the_annotation_tables_alike`. Green
before, **FAILED** under the mutation, green again after restoring from the copy with the sha256
re-checked equal. So the control is not passing for the wrong reason.

**The load-bearing reasoning holds, and the failure output is the evidence rather than an appeal to
`std`'s internals.** Under the mutation the two sides differed as `8 12 16 20 28 24` against
`8 12 20 16 28 24` -- the same six tables at the same six arena indices in a different assignment,
which is an iteration-order difference and nothing else. Both sides were produced in one thread of
one process, so a second `HashMap` built in a thread does iterate differently from the first, which
is the claim the doc rests on.

**Nothing else in this range is exposed to an iteration order.** This range introduces no other map
walk. The two `Interp` maps this task added are reached only through `get` and `insert`
(`environment.rs:879`, `:886`, `:910`, `:916`, `:951`) and are never iterated, so neither can reach
allocation order. Empirically, each of the three new corpus programs is one distinct
`(rc, stdout, stderr)` triple over twelve runs on each engine. I did not attempt the general survey
of everything that answers from an arena index or a hash order -- that remains open, as reported.

**F1 and F2, checked against the tree.** Measured at `4a519518d`, oracle rc 0 and byte-identical on
both engines: `.K~method("M")~objectName = "renamedM"` then `say .K~method("M")` prints `renamedM`
through a second fetch, `~objectName` reads it back, `.K~method("P")` still prints `a Method`, and
`.routines["R"]`, `.methods~u` and `.K~package` behave the same -- which is what
`WHAT MAKING METHOD AND ROUTINE RECEIVERS ALSO CLOSED` now claims. The `WHAT REMAINS REFUSED` block's
witness is exact: `say .K~method("M")~source` is `  return 1` at oracle rc 0 and
`rexx-exec: method "SOURCE" of class "Method" is not implemented (Phase 5)` at rc 120 on both
engines, the message verbatim as the block quotes it. No corpus program prints an `~identityHash`
value; the only one that names it, `corpus/lang/environment_methods_join.rex:30`, prints a
comparison. `environment.rs:906`-`:908` and `:958`-`:962` no longer carry the per-send claim, and
what replaced them is true: every object that carries a handle on a table is rooted through
`add_global` (a `Method` object directly, a `Routine` or unattached `Method` as an entry of a rooted
package table, a `Package` object directly).

**F4's replacements, each verified with `sed -n` on the C++ and each on the branch its example
takes.** `parser/DirectiveParser.cpp:2139` is `findInstanceMethod(getterName)` and `:2140` is
`if (getterMethod != OREF_NULL && !getterMethod->isAttribute())` -- the instance getter's check, which
is the branch the test's `::class K / ::method a / ::annotate attribute a` row enters. `:2131` is
`void LanguageParser::processAttributeAnnotations(RexxString *getterName)`. `:1775` is
`createMethod(internalname, isClass, accessFlag, protectedFlag, guardFlag, true)`. `:2259` is
`table->put(value, name)`.

**F7's replacement is a property the assertion checks.** The count is gone; what stands is "moving
**any** variant across `is_admitted_directive_kind`'s arms in either direction reddens here
specifically, with the wrong keyword named in the failure", and `coverage.rs:249`-`:253` is
`assert_eq!(is_admitted_directive_kind(kind), *expected_admitted, "{expected_keyword} admission
disagrees with the committed expectation")` over a `cases` row per variant. Every one of the match's
nine variants has a row, so the statement is what the assertion holds and the assertion can fail.

**F3 and F6.** The corpus header names `ClassDirective::getAnnotations` as the directive-side
accumulator no readback reaches, which is right -- `/bin/grep -rn "::getAnnotation"` over
`interpreter/` finds `RexxClass`, `PackageClass` and `BaseExecutable` as readback bodies and that one
as the accumulator. The `dispatch.rs` and `environment.rs` cardinalities now name their members.

**The sitting.** `bench-baselines/phase-5a-arms.tsv` carries task 20 at `7e253fae1`, the last commit
in the range that touches `src/`; both arms (`pinned`, `base`, `pinned>head`, `base>head`), eight
axes, both sizes, both engines, both instruments, `value_rounds` 5 throughout. Contribution arm,
`instructions:u`: six axes bit-identical on every cell, `dispatchclass` between 0.999071 and
0.999085, `rexxcps` 0.999996 and 0.999998. Nothing at or above 1%.

**`4a519518d`.** The two overturned bullets each open by naming themselves as the claim
(`this paragraph claimed ... **Both halves are false and Task 20 delivered it**`;
`this paragraph deferred ... **Task 17 populated it**`), and the paragraph under them retracts the
whole passage explicitly (`nothing in them should be relied on`) and names the third premise as well,
so a reader of the section cannot take any of it as live guidance. Task 21's remaining scope is stated
correctly and I measured both halves of it at `4a519518d`: `.context~package~name` is oracle rc 0 and
`rexx-exec: a message send to one of the interpreter's own objects is not implemented (Phase 5)` at
rc 120 here, and `.K~package~findRoutine("R")` is oracle `a Routine` at rc 0 and
`method "FINDROUTINE" of class "Package" is not implemented (Phase 5)` at rc 120. Removing the
`PACKAGE` clause from the "Done when" is right -- it was satisfied before Task 21 starts. The reading
table's repaired sentence parses.

## Three notes, none of them defects and none needing a change

* The section's bolded lead-in ("Installing all six is this task's; reading all six back is not")
  is the strongest of the overturned claims and its marking arrives after the bullets rather than
  before them. It is one of "the paragraphs above" that the retraction covers by name, so a reader of
  the section is not misled; only a reader who stops at the bold sentence would be.
* Under the inversion the control reddens at `annotate_on_both_engines`' own engine-agreement
  assertion rather than at its later loop, so the message a maintainer would see names the engines
  and not the ordering. It fails on the right property -- the two stdouts differ only in which table
  got which index -- and the loop is redundant belt-and-braces rather than the thing that fires.
* Task 21's Build list still names `Package` `~name` among what it builds; measured,
  `.K~package~name` is rc 0 and byte-identical to the oracle today. Pre-existing text this commit did
  not touch, one clause from the row it did.

## Gates I ran

`cargo test -p rexx-exec --lib two_runs_in_one_process_allocate_the_annotation_tables_alike` three
times: ok, FAILED under the mutation, ok after restore. I did not re-run the five gate commands --
verified by the controller at `949c374f8`, and `4a519518d` is documentation only, so the release
binary is unchanged from that point. About twenty-five differential probes and four twelve-run
stability sweeps, all from fresh directories.

## Tree state left

`git status --porcelain` empty, `HEAD` `4a519518d`. `crates/rexx-exec/src/lib.rs` was mutated for the
inversion and restored from a scratchpad copy; its sha256 is `680031f0...31432`, equal to the copy
taken before the mutation, and `git status` confirms it. No other file in the repository was touched
except this report, which lives under the git-ignored `.superpowers/`. `cargo test` rebuilt
debug-profile artifacts under the shared `target/`; the release binary was not rebuilt.
