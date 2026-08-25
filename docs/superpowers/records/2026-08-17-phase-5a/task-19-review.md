# Task 19 review: `::CONSTANT`'s instance method and class method

Range `9a0f249e9..a39a91ebd`. Everything below was run by me from this worktree.

**Spec compliance: PASS.**
**Task quality: PASS with three comment-accuracy defects, none behavioural.**

---

## 1. The control (brief item 1). Both halves confirmed.

Mutation A applied to `install_class_members`' `Constant` arm
(`DirectiveKind::Constant(constant) if class_side =>`), release rebuild,
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`:

```
206 of 207 matching
mismatches (1):
  [UNCLASSIFIED] lang/class_constant_instance_method.rex: stdout, stderr, exit code differ
      rust:   stderr="       *-* Compiled method \"METHOD\" with scope \"Class\".\n    25 *-* say .K~method(\"C\")\n..." exit=159
      oracle: stdout="a Method\na Method\nthe class-side method raised 97\n" exit=0
```

The reddening half is the brief's wording met exactly. The staying-green half is
the count: one mismatch means `class_constant_values.rex` passes with only the
class-side getter installed, and `class_constant_values` appears nowhere in the
mutation's output. The brief's reason `.K~c` cannot be the control's subject is
therefore measured, not argued. **The instance half is real and this task's
coverage is real.**

## 2. Six of the seven reported mutations reproduced (brief item 2).

Each applied to a `cp`-restored `lib.rs`, rebuilt, run, restored from the same
pristine copy.

| # | mutation | corpus | reddened |
|---|---|---|---|
| A | getter class-side only | 206 of 207 | `class_constant_instance_method.rex` alone, `.K~method("C")` |
| B | instance getter only for an already-recorded value | 206 of 207 | same program alone, **line 26 `say .K~method("D")`** -- the expression row |
| C1 | `SELF` not bound | 205 of 207 | `class_constant_expression_self.rex` **and** `class_constant_expression_method_failure.rex` |
| C3 | receiver left `None` | 206 of 207 | self program alone, on `::constant p (self~secret)` |
| C5 | `SELF`/`SUPER` from the last-installed class | 206 of 207 | self program alone, **stdout only**: `J / K` twice |
| D | expression inherits the program's arguments | 207 of 207 | nothing in the corpus |

Every count and every named program matches the report. I did not run C2.

**No witness that cannot fail.** The two I judged most at risk were the self
program's extra rows -- the `J` row and the `PRIVATE` row -- and the second row
of the instance-method program. C5 reddens the `J` row alone and stdout alone,
C3 reddens the `PRIVATE` row alone, B reddens the expression row alone. All
three rows earn their place.

**One coverage limit the report states and I confirm:** under the implementer's
own mutation set `class_constant_expression_method_failure.rex` never reddens
alone -- only C1 reaches it, and C1 reaches the self program too. It can fail
(C1 proves that), and it pins a three-echo traceback no other program pins, but
its *marginal* coverage over its sibling is not demonstrated by a mutation. That
is the combination-witness case, not a defect.

## 3. `input_oracle.rs` is the sole instrument, confirmed workspace-wide (brief item 3).

Under mutation D I ran the **whole workspace**, `REXX_CORPUS_GATE=1 cargo test
--release --workspace --no-fail-fast`: exactly one failing test in 98 blocks,
`command_line_arguments_and_the_console_agree_with_the_oracle`, failing on
`constant-expression-arguments [stdout]`. Corpus stayed 207 of 207. Sole-instrument
status is measured, not asserted.

**The single-engine limitation is not stated in the tree** -- not in
`input_oracle.rs`'s module doc, not in the case's `why`. It lives only in the
report's section 10. Two things soften it and one sharpens it:

* `run_rust` (`input_oracle.rs:388`) sets no `REXX_ENGINE`, but so does
  `corpus.rs:246` (`Invocation::none()`). **The entire corpus gate is
  single-engine.** The report's framing makes row D sound uniquely exposed; it
  is not.
* No `ir_dual_cases` stanza declares a `::CONSTANT`, so **nothing automated in
  this tree runs any of Task 19's behaviour on the tree-walker.** I ran all
  three new corpus programs and the argument program on `REXX_ENGINE=tree-walker`
  and `REXX_ENGINE=ir` against the oracle by hand: byte-identical on all three
  descriptors in every case. The behaviour is right; the automation is
  single-engine, and that is the phase's position rather than this task's.

## 4. `self` binding at its edges (brief item 4). Thirteen probes, all byte-identical.

Fresh empty directory per program, absolute paths, three descriptors read
separately, oracle under `ulimit -v 1048576`, crate under `memcap 1G`, both
engines. Every one matched the oracle byte for byte on stdout, stderr and exit
code.

| probe | oracle |
|---|---|
| constant calling a class method that itself reads `self`/`super`, on `K` and on `J SUBCLASS K` | rc 0, `self=K super=Class` / `self=J super=Class` |
| `::constant c (super)` on a plain class | rc 0, `The Class class` |
| constant using `self` **and** reaching a class declared later (Task 18's second pass) | rc 0, `A -> from a later class, self=LATER` |
| `::constant c (self~c)`, self-reference | rc 159, 97.4 `Constant "C" ... has not been initialized.` |
| `::constant c (self~inst)` reaching an **instance**-side method | rc 159, 97.1 `Object "The K class" does not understand message "INST".` |
| `arg()` inside a method the constant calls | rc 0, `args=0` |
| `SELF` leaking: `(self~id)` on K, `(self~id "|" x)` on J, `say self` in the main body | rc 0, `K` / `J \| X` / `SELF` |
| an **inherited** constant: `.J~c`, `.J~hasMethod("C")`, `.J~method("C")` | rc 0 `K`, `1`, then 97.1 -- J's own dictionary does not hold it |
| **install order != source order**: `J SUBCLASS K` and `M SUBCLASS J` declared above `K` | rc 0, `J / K`, `K / Class`, `M / J` |
| a failure **two** method activations deep from a constant | rc 214, four echo lines in the right order |
| a `::ROUTINE` called from a constant reading `self` | rc 0, `routine sees self as: SELF` |
| the argument-list divergence, program invoked with `hello` | rc 0, `constant [0][]` / `program  [1][hello]` |
| the three new corpus programs themselves | as recorded |

The install-order probe is the one that could have broken silently: it pairs
`classes[index]` against `members.get(index)` under a dependency order that
differs from source order, and each class answers with its own identity.

## 5. `phase-4-exclusions.txt` (brief item 5). Faithful.

* Moved in its own commit, `13081bf80`, as the plan assigns.
* The gap is genuinely closed: its own transcript program is rc 0 `main` on the
  oracle and on both engines (re-measured by me).
* `WHAT CLOSED IT` names `34cd90a4a`; `git log --oneline -S"fn resolve_constants"
  -- rust/crates/rexx-exec/src/lib.rs` returns exactly that one commit.
* The description of the fix -- three passes over the dependency-ordered class
  list -- matches `install_directives` as written (`lib.rs:4193`, `:4207`, `:4213`).
* `WHAT WOULD GO RED` names `corpus/lang/class_constant_expression_later_class.rex`,
  which exists. Naming the test without a measured count matches the neighbouring
  entry at `:3728`; it is the section header's own requirement.
* The dangling `"IS A DIFFERENT QUESTION"` referent left by the move was found
  and repaired in the same commit. That was a real catch.

## 6. The sitting (brief item 6). Correct.

760 rows, task `19` (`pinned>head`) and task `19-contribution` (`base>changed`),
**eight axes each**, both engines, `instructions:u` and `cycles:u`, `size` on
every row. Projections keeping `instrument` and `size`:

* Contribution arm, `instructions:u`: every cell of `alloc4c`, `arith`,
  `compound`, `emptyloop`, `strings`, `varlookup` reads exactly `1.000000`;
  `dispatchclass` reads `1.000003`/`1.000006` (tw) and `0.999996`/`0.999993`
  (ir); `rexxcps` `1.000009` (tw) / `1.000006` (ir). Matches the report and the
  commit message digit for digit.
* `cycles:u` on the same arm spans `0.979953` to `1.035493`. Correctly not taken
  as a result.
* The `dispatchclass ir small` anomaly: `1.020427` here against `1.018380`, and
  `ir large` `1.018412` against `1.018408`. Those two comparison figures are
  **task `18-fixround-3`'s** rows, not task `18`'s -- task `18`'s own rows in
  this TSV carry six axes and no `dispatchclass` at all. The numbers are exactly
  right; the label "Task 18's" is imprecise about which sitting.
* The attribution to the recorded bimodal cell is one step wider than the record:
  `progress.md:4881` records `dispatchclass **tw** small` as bimodal, and this is
  `ir small`. The contribution arm being flat to a millionth is the tie-breaker
  the plan names, and it is stated as such, so the conclusion stands.

## 7. Constraints and citations.

* **No `unsafe`** added. Zero added lines match `\bunsafe\b`.
* **ASCII only, no em-dashes**: zero non-ASCII bytes among the 366 added lines.
* **No historical framing in source comments**: the single hit for past-tense
  framing is in `phase-4-exclusions.txt`, which is the file history belongs in.
* **No set cardinality**: the counting words present (`Two classes`, `Both value
  forms`) describe the program's own content rather than the size of a set the
  code could enumerate, which is the settled reading in this tree -- the same
  shape appears throughout the existing corpus comments.
* **Every C++ citation verified with `/bin/grep -n` / `awk` and checked against
  the path its own example takes:**
  * `ClassDirective.cpp:271` `new MethodClass(GlobalNames::CONSTANT_DIRECTIVE, ...)`,
    `:273` `code->setScope(classObject)`, `:276` `code->run(activity, classObject,
    ..., NULL, 0, dummy)` -- all inside `ClassDirective::resolveConstants`, which
    opens at `:257`. The `NULL, 0` at `:276` is the argument-list claim's own source.
  * `GlobalNames.h:84` is `GLOBAL_NAME(CONSTANT_DIRECTIVE, "::CONSTANT")`.
  * `ObjectClass.cpp:616` `sender = activation->getReceiver()`, `:617`-`:620` the
    `sender == this` return, `:622`-`:626` the `sender == OREF_NULL` refusal --
    inside `RexxObject::checkPrivate`, opening at `:609`. For
    `::CONSTANT c (self~p)` sender and receiver are both the class object, so the
    example takes `:617`-`:620` exactly as the comment says.
  * `ClassClass.cpp:984` `MethodClass *RexxClass::method(RexxString *)`, `:991`
    the `instanceMethodDictionary->getMethod` retrieval.
  * `RexxActivation.cpp:535`-`:536` the `SELF`/`SUPER` `setLocalVariable` calls.
  * `dispatch.rs:1469` `fn enter_method_body`, inside `impl Interp` at `:697`.

  **No wrong-line citation this round.** That is the first task in four without one.
* **Intra-doc links resolve.** `cargo doc --no-deps -p rexx-exec
  --document-private-items` reports 15 unresolved links, all pre-existing, none in
  `lib.rs` lines 5000-5199 and none naming `enter_method_body`, `slot_of` or
  `push_directive_activation`. The self-caught `Interp::invoke_method` slip is
  genuinely absent from the tree.
* **The three `sourceline_oracle` recordings are current**: each `count` matches
  its program's line count and each body is byte-identical to the `.rex`.

## 8. Defects found

**D1. `lib.rs:4372`-`:4374`: a comment this task falsified, inside the function
this task changed.** The `resolve_constants` Err arm says the two directive
echoes are sealed with the same mechanism real activations use, "-- there is no
real activation nesting here, only the two directives' own clauses standing in
for it." After this task there can be: `class_constant_expression_method_failure.rex`
puts a real method activation under those two echoes, which is the whole reason
the commit added it. Measured, oracle and both engines:
`22 *-* return 1/0` / `23 *-* ::constant c (self~m)` / `20 *-* ::class K`.
Fix is one clause saying the standing-in applies to the two directive echoes
themselves.

**D2. `corpus/lang/class_constant_expression_method_failure.rex:8`-`:9`: a false
universal.** "this program is the only one that puts a real activation between
the raising clause and the directives." Counterexample already in the corpus:
`class_activate_failure_blames_the_last_installed_class.rex`, whose ACTIVATE
method raises. Measured just now, oracle and both engines, rc 214:
`13 *-* x = 1/0` then `15 *-* ::CLASS leaf SUBCLASS middle` -- a real method
activation, its clause above a directive echo. The program's true unique
property is the **three**-echo shape: a method clause above both a `::CONSTANT`
and a `::CLASS` echo. The same sentence is repeated in the report's section 5.

**D3. `crates/rexx-parse/tests/sourceline_oracle.rs:45`-`:46`: a neighbouring
comment this task falsified and did not update.** It reads "Verified this driver
taking the fallback path on `trace_numeric_request.rex` and the primary path
everywhere else." The new failure program takes the fallback path too. Measured
with that module's own driver against the oracle: `class_constant_expression_method_failure.rex`
FALLBACK, `trace_numeric_request.rex` FALLBACK, the other two new programs
PRIMARY. The report *knows* this -- section 5 says the failure program takes the
fallback path and argues correctly that it is faithful here (I checked: trailing
newline present, zero CR bytes, zero `CTRL-Z` bytes) -- but the comment that
says it is the only such file was left standing.

All three are the same shape: a claim that was true before the change and that
the change's own new witness contradicts. None affects behaviour; none is caught
by any gate.

**Minor, not defects.** (a) The ledger's "each half of what was built has one
that reddens it and nothing else" is loose for the `SELF` half: C1 reddens two
programs, as the report's own table correctly shows. (b) The pre-existing
sentence at `lib.rs:5083` -- the constant expression "runs before either engine's
own instruction loop starts" -- was already loose (a constant could always call
a `::ROUTINE`) and is looser now that it can call a method; not introduced here,
and I measured both engines agreeing anyway.

## 9. Gates I ran at `a39a91ebd`, after restoring the tree

```
cargo fmt --all --check                                     FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings        CLIPPY_EXIT=0, zero warning/error lines
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   GATE_EXIT=0
   98 "test result: ok", 0 FAILED, 0 panicked, 0 lines starting "error"
   207 of 207 matching
```

## 10. Tree state I leave

Clean at `a39a91ebd`; `git status --short` and `git diff --stat` both empty.
`crates/rexx-exec/src/lib.rs` restored **byte-identically** from a pristine `cp`
taken before the first mutation -- sha256 `4e4b93cc672b8dc92179e6b361f59f1a164548179d886fa4e4a09487e4692ed3`,
the same value it had before I touched it. `target/release/rexx-run` rebuilt to
sha256 `8d4a633df899e26ac39f006d1f95c8c77ea2768fd81141cba4069bc036fdd810`, the
head value the report records, so no mutated binary outlived its window.

Six mutation windows, each mutation applied, built, run and restored before the
next. Two shared-tree side effects to disclose: I `touch`ed `lib.rs` once
(content unchanged) to force a rustdoc re-check, and every mutation rebuilt
shared `target/release` artifacts. The final state is a clean rebuild at head
with all gates green.
