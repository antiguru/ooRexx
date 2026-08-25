# Task 20 review: `::ANNOTATE`'s six targets, and the readback

Range `7130b222e..196da092d`. Every probe below is from a fresh empty directory, absolute paths,
three descriptors read separately, both sides bounded, and both `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker` unless a row says otherwise.

## Verdicts

**Spec compliance: PASS.** All six targets install and all six readbacks agree with the oracle byte
for byte on both engines; the unknown-target case is the oracle's own 99.945 rc 157 for every
keyword that takes a name and for both kind filters; the `lib.rs` carrier of the generalisation is
retired rather than re-commented and the other two were already superseded; the sibling row is in
`phase-4-exclusions.txt`; the sitting is both arms, eight axes, five rounds, re-run at the commit
that ships, with nothing at or above 1% on the contribution arm. The controller's ruling was
honoured as written: `Method` and `Routine` became receivers by the arms `.Package` already had, and
`Body::Instance(_)` still answers `Err("an instance of a user class")` -- no general
instance-receiver arm was added.

**Task quality: CHANGES REQUIRED.** Three false statements ship in the tree, two of them created by
this task's own late fix falsifying prose it did not sweep, and one of them in the records file the
task was sent to correct. Beside them: a false count in a corpus header, three C++ citations pinned
to a brace or a blank line, and a new run-to-run nondeterminism in the shipped binary.

## Findings, one line of basis each

**F1. `crates/rexx-exec/src/environment.rs:907` and `:958` both say a `Method` object is rebuilt per
`~method` send. It is cached.** `301842eaa` added `Interp::method_object` and the `method_objects`
map and asserts the opposite in `a_class_answers_one_method_object_per_dictionary_entry`; both
sentences are the *justification* for keeping the annotation table and for rooting it as a global,
so a correct decision now carries a false reason. Must fix.

**F2. `docs/superpowers/plans/phase-4-exclusions.txt:2520`-`:2526`, the new `WHAT REMAINS REFUSED`
block, is false in all three of its claims.** Measured on both engines, byte-identical to the
oracle at rc 0: `.K~method("M")~objectName = "x"` then `say .K~method("M")` prints `renamedM`,
`~objectName` reads it back, and `.K~method("P")` still prints `a Method`; the same holds through
`.routines["R"]`, `.methods~u` and `.K~package`. "This crate builds a Method object per send" is F1's
claim again, and `dispatch.rs`'s `native_object_name_set` now *stores* on
`Primitive::Method | Primitive::Routine` and its own comment carries no such measurement. The block
landed in `6808a1901` and `301842eaa` did not sweep it. Must fix.

**F3. `corpus/lang/directive_annotate_targets.rex:2` says the six readbacks reach "four different
C++ implementations" and then names three.** `/bin/grep -rn "::getAnnotation(\|::getAnnotations(" `
over `interpreter/` finds `RexxClass`, `PackageClass` and `BaseExecutable` as readback bodies;
`ClassDirective::getAnnotations` (`instructions/ClassDirective.cpp:565`) is the directive-side
accumulator and no readback reaches it. Wrong count, and a set cardinality besides.

**F4. Three C++ citations land on a brace or a blank line.**
`run/tests.rs`'s refusal-test doc cites `isAttribute()` at `parser/DirectiveParser.cpp:2164`, which
is `}`; the calls are `:2140`, `:2146`, `:2156`, `:2162`. `lib.rs`'s `annotation_target` doc cites
`processAttributeAnnotations` at `:2160`, a blank line inside it; it is defined at `:2131`.
`lib.rs`'s `is_attribute_method` doc cites `createMethod`'s `true` at `:1774`, which is `{`; the call
is `:1775`. Marginal: `processAnnotation`'s put is cited at `:2258`, the comment above
`table->put(value, name)` at `:2259`. Everything else I checked lands correctly, including
`Setup.cpp:1111` and `:1140` being in the `Method` and `Routine` blocks respectively,
`StringHashCollection::entry` actually upcasing (`HashCollection.cpp:824`-`:827`), `isConstant()` at
`:2081`, `ClassDirective.cpp:243`, `:455`, `:520`-`:524`, `ClassClass.cpp:325`/`:357`/`:374`/`:984`/
`:991`, `BaseExecutable.cpp:378`/`:411`, `PackageClass.cpp:1791`, `RexxErrorMessages.h:715` and all
five `syntaxError` sites.

**F5. The shipped binary is no longer run-to-run deterministic where annotation tables are
allocated.** `lib.rs`'s leftover loop `for (target, pairs) in staged` drains a std `HashMap`, so the
tables are allocated in a per-process random order and `~identityHash` answers the handle. Measured:
one program printing four annotation tables' `~identityHash`, twelve runs of
`target/release/rexx-run`, ten distinct outputs. Controls, both 12/12 identical: the same program
with the `::ANNOTATE` directives removed, and the annotated *objects*' own `~identityHash` values in
the annotated program. No corpus program prints an `identityHash` value today (the one that names it,
`corpus/lang/environment_methods_join.rex:30`, prints a comparison), so nothing is flaky yet. The fix
is to drain `staged` in a deterministic order; `attach_directive_annotations`' `sides` map is the
same shape.

**F6. Set cardinalities in comments, which `rust/CLAUDE.md:87` forbids outright.** `dispatch.rs`'s
`NATIVE_METHODS` comment ("the three C++ bodies differ only in which field they reach for, so the
four classes share one implementation here"), `environment.rs`'s `Annotated` doc ("the three C++
bodies behind those rows"), `environment.rs`'s `annotations_of` doc ("an object of each of the last
three"), and `run/tests.rs` ("five keywords need five programs, and the two kind filters below need
two more"). F3's "four" is the same rule.

**F7. `crates/rexx-exec/tests/coverage.rs`'s
`every_directive_keyword_is_correctly_admitted_or_refused` doc still says "moving *any* one of the
six refused variants", and this commit edited the match it describes.** That match now refuses three
(`Options`, `Requires`, `Resource`); at `7130b222e` it refused four, so the count was already wrong
and the change moved it further from true.

**F8. Historical framing in a comment.** `lib.rs`'s `staged_gap` doc: "**A resolvable `::ANNOTATE`
target is no longer one of them**". Strike "no longer" and the sentence says the same thing about the
code as it stands, which is `global-constraints.md`'s own test for decoration.

**F9. Task 21's brief still claims the `PACKAGE` readback this task delivered, in two places.** Its
Build list: "`~annotation`/`~annotations`, which is Task 20's `PACKAGE` target readback arriving at
the task that builds the object it needs"; its "Done when": "Task 20's `PACKAGE` annotation readback
agrees byte for byte". Both are satisfied before Task 21 starts, so the second is a criterion that
cannot fail there. What Task 21 does still own is measured and still rc 120 here:
`.context~package~annotation("A")` is a loud send gap and `.K~package~findRoutine("R")` is a loud
`NATIVE_METHODS` gap, against oracle rc 0 for both.

**F10. The live plan's own Task 20 section still carries the falsified ownership split** ("the
`PACKAGE` target ... **Task 21 owns its readback**", "the `ROUTINE` target's readback is 5c's"). The
implementer disclosed this and corrected only the two forward-looking rows; controller's call whether
a reader of `docs/superpowers/plans/2026-08-17-phase-5a.md` should meet those paragraphs unmarked. The
sentence that replaced the reading-table row is also garbled -- "The ownership this row drew from that
did not survive Task 20's own measuring".

## Priority 1: the reconstructed `environment.rs`

**Nothing the file asserted at `7130b222e` has silently gone.** `git diff 7130b222e --
crates/rexx-exec/src/environment.rs` is 243 insertions and 28 deletions, and every deletion is
accounted for by the change: the old `TableValue::Instance(&'static str)` doc and variant (replaced),
the `method_value()` helper and its doc (replaced by a closure that takes the annotation key), and
eight signature or call-site lines. A deletion the reconstruction had dropped would appear in that
diff, so the diff is a complete witness for that half of the risk.

**The `env_seam` seam is untouched.** The diff changes no line inside `mod env_seam` (`:95`) and no
line at `env_seam::admit(` (`:521`) or `env_seam::directory(` (`:561`); `tests/environment_seam.rs`,
which pins the module's items and both call spellings crate-wide, passed in the controller's gate run.

**Cross-file doc references still say true things.** `eval.rs:30`, `lib.rs:742`, `lib.rs:4865`,
`value.rs:819`, `dispatch.rs:877` and `dispatch.rs:883` all name items that exist. `dispatch.rs:883`
is the interesting one: it says `environment.rs` builds a `Method` "one of per instance dictionary
entry a program asks for", which is the *correct* post-fix statement -- so `301842eaa` updated that
neighbour and missed the two inside `environment.rs` itself, which is F1.

The residual risk the controller named -- mid-task work that was lost and never restored -- cannot be
seen by any diff. What can be said is that the shipped file's behaviour is covered by the three
corpus programs, three in-crate tests and the ~60 differential probes below, and that the only prose
defects it carries came from the later fix rather than from the reconstruction.

## Priority 2: is the identity fix complete?

**Yes on every route I could reach.** Byte-identical to the oracle on both engines: `~identityHash`
self-equality through `.K~method("M")`, `.routines~r`, `.routines["R"]` against `.routines~r`,
`.K~package`, `.methods~u`, and the `~annotations` table of each of those; `~objectName=` then read
back through a *second* fetch on a `Method`, a `Routine`, an unattached `Method` and a `Package`, with
`.K~method("P")` unrenamed as the control; `.K~package` and `.J~package` the same object and
`.Array~package` a different one.

`==` and `=` between two fetches are **loud**, not wrong: `rexx-exec: the operator `==` applied to one
of the interpreter's own objects is not implemented (Phase 5)`, which pre-dates this task and applies
to every interpreter object.

## Priority 3: the new receivers' surface

I took the `Method` and `Routine` instance sets from
`crates/rexx-classes/tests/native_classes_wiring.rs` and sent every name on both sides.

* Every name with a `NATIVE_METHODS` row matches the oracle byte for byte: `CLASS`, `STRING`,
  `OBJECTNAME`, `OBJECTNAME=`, `ISNIL`, `ISA`, `HASMETHOD`, `REQUEST`, `INIT`, `ANNOTATION`,
  `ANNOTATIONS`.
* Every name without one is a loud rc 120 Phase 5 gap naming the receiver's class -- on `Method`:
  `ISABSTRACT`, `ISATTRIBUTE`, `ISCONSTANT`, `ISGUARDED`, `ISPACKAGE`, `ISPRIVATE`, `ISPROTECTED`,
  `PACKAGE`, `SCOPE`, `SOURCE`, `SETGUARDED`, `SETUNGUARDED`, `SETPRIVATE`, `SETPROTECTED`; on
  `Routine`: `CALL`, `CALLWITH`, `PACKAGE`, `SOURCE`, `SETSECURITYMANAGER`, `[]`.
* A name in neither dictionary is 97.1 rc 159 on both sides (`~zzznosuch` on each).

So no name that should be refused now answers, and the widening produced no wrong answer. The
`Primitive::Routine` doc's own measurement holds exactly: `.routines~r~annotation()` reports
`Compiled method "ANNOTATION" with scope "Routine".` where `.K~method("M")~annotation()` reports
`"Method"`, both rc 168 and byte-identical here.

Twenty further differential probes on the resolution rules all matched on both engines: an
`::ANNOTATE` with no pairs, `::annotate method "A="` naming the setter alone, the same method name
under two classes, a subclass not inheriting the base's annotation, `::attribute a get` with only one
half, package and class annotations of the same name not colliding, a quoted target name, a quoted
method name containing spaces, unattached `::ATTRIBUTE` and `::CONSTANT` read through `.METHODS`, two
routines not crossing tables, a class-side `::METHOD` target installing, the shared `::CONSTANT`
table staying live, and `.Array~package~annotations` answering an empty table.

## Priority 4: the retired generalisation

`/bin/grep -rn "ANNOTATE naming a target"` over the repository finds no hit under `rust/` -- the
`directive_gap` arm is gone, not re-commented, and the measurement now sits at
`Raised::missing_annotation_target`. The other two carriers need nothing: the 2026-08-15 spec's own
header says `**SUPERSEDED 2026-08-17 ... Do not plan against this file.**` and the 2026-08-15 plan's
header says the 2026-08-17 spec wins where they disagree.

`phase-4-exclusions.txt:420`'s `::annotate routine nosuchrtn` row was **removed** rather than kept.
That is right and is not a loss of the measurement: the list it sat in is headed "REFUSED BY THE
ORACLE BEFORE main ... **and refused here, loudly, naming Phase 5**", and the second half stopped
being true, so leaving the row would have been a false statement. The measurement survives in the
`CLOSED DEFECTS` entry, and the sibling row the brief asked for is in the "runs here too" list above
it.

A sample of the six shapes the records carried as standing costs, all byte-identical to the oracle on
both engines: `::routine r`/`::annotate routine r`/failing `::CLASS` at rc 158; `::class a`/
`::constant kk (1/0)`/`::routine r`/`::annotate routine r` at rc 214; the duplicate-`::ROUTINE` and
class-less-`::CONSTANT` pairs at rc 157; and both unresolvable-target shapes at rc 157.

## Priority 5: the sitting

`bench-baselines/phase-5a-arms.tsv` holds task 20 at `6808a1901` and again at `301842eaa`, the last
commit that touches `src/`; the two commits after it are records and plan only, so the release binary
is byte-identical and the re-run is at the tree that ships. Both arms are present at each commit
(`pinned`, `base`, `pinned>head`, `base>head`), all eight axes, both sizes, both engines, both
instruments, `value_rounds` 5 on every row.

Contribution arm, `instructions:u`, at `301842eaa`: six axes are `1.000000` on every cell;
`dispatchclass` is 0.999077 / 0.999082 / 0.999090 / 1.001081 across its four cells and `rexxcps` is
0.999984 / 1.000011. Nothing at or above 1%, so there is nothing to attribute.

Against the pin, the four axes above 1% are `compound`, `dispatchclass`, `strings` and `rexxcps`, and
they read the same at task 19 to five decimal places (`strings` large ir 1.013411 at both; `compound`
large ir 1.010473 at both; `rexxcps` small ir 1.020310 against 1.020293) -- inherited, as reported.
`cycles:u` is recorded and is not quoted as a result. The pin checks pass: `15a1ffa98` is an ancestor
of `196da092d`, its sha256 is `141c3fa96...74b` matching `PINNED.md:16`, and
`git log --oneline 15a1ffa98..196da092d -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml`
lists only this phase's own task commits.

## Constraints

No `unsafe` and no non-ASCII byte in any added line under `rust/`. The three new corpus programs
carry no CR and no `CTRL-Z` byte and each ends with a newline. `cargo doc --workspace --no-deps` exits
0 and its four `unresolved link` warnings are in `rexx-bench` and `rexx-classes`; none is in
`rexx-exec` or `rexx-core`, which confirms the report's one-time check and clears the intra-doc-link
defect for this range. Comment defects are F6, F7 and F8.

## What I ran, and what I did not

Ran: `cargo doc --workspace --no-deps` (exit 0). About sixty differential probes across nine batches,
oracle plus both engines, three descriptors separately, each from a fresh directory under the
scratchpad. Twelve-run determinism sweeps with two controls. `/bin/grep -n` or `sed -n` on every C++
line cited by an added comment.

Did not: re-run the five gate commands, per the controller's instruction that they are already
verified; re-run the three mutation checks the report lists, so "caught by the new test only" is
reported rather than confirmed here.

## Tree state left

`git status --porcelain` is empty; `HEAD` is `196da092d`. No file in the repository was created,
edited or deleted by this review except this report. Every probe lives under the session scratchpad.
`cargo doc` wrote `rust/target/doc` and dev-profile artifacts into the shared `target/`.
