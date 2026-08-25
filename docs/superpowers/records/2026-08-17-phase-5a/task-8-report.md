# Task 8 report: `::CLASS ... METACLASS`, and the rest of `::CLASS`'s option surface

Base `d0b7bd151`. Everything below was run by me in this worktree, one process per oracle
invocation, from a fresh empty directory
(`/tmp/claude-1000/.../scratchpad/t8probe`) with absolute paths, three descriptors read
separately, never `2>&1`. `REXX_ENGINE` selects the crate's engine; where a row says "both
engines" it was run twice.

---

## 1. The five gate commands

Run from `rust/`, in this order, each status read **unpiped** from `$?` on its own line. The run
transcribed here was taken at the tree that became **`6754ae2a2`**, the last of this task's three
commits -- after the prose sweep of section 8,
after every control of section 6 had been reverted, and after the `gate_table_c.rs` control sentence
was sharpened following the team lead's ruling (section 7 item 7) -- so these are the statuses of the
tree as it stands and not of a state that preceded a mutation or an edit:

```text
cargo fmt --all --check                                              exit 0
cargo clippy --workspace --all-targets -- -D warnings                exit 0
cargo test --release --workspace                                     exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                  exit 0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast    exit 0
```

`command -v memcap` answered `/home/moritz/.local/bin/memcap`, so gate 5 is the `memcap` form and
not the `ulimit` substitute. Across the three test gates' captured output: `/bin/grep -cE
'^failures:'` is **0** and `/bin/grep -cE 'test result: FAILED'` is **0**, for each of the three
separately.

Clippy was run after `touch`ing every source file this task changed, so its green is a re-lint of
the committed content and not a reused per-crate result -- the hazard `rust/CLAUDE.md` records under "a green `clippy` is only
evidence if the linter re-examined the code".

**Corpus: 135 of 135.** 127 before this task, plus eight programs. The count comes from the gate 4
capture's own `135 of 135 matching` line.

**The pin's staleness test, run before the sitting.** `git merge-base --is-ancestor 15a1ffa98 HEAD`
succeeds; `sha256sum bench-baselines/pinned/rexx-run-15a1ffa98` is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`; and
`git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml` lists
**no commit this plan's `progress.md` does not name** -- checked mechanically, one `grep` per commit,
printing `LEDGER` or `***NOT IN LEDGER***` for each, and no commit printed the second. That is the
property the staleness test is about; the list's length is not, and moves with every commit of this
task (**28** at the base `d0b7bd151` when the test was run, 29 at `bb6d46466`, 32 at `8fb97527c`).

---

## 2. The metaclass witness, byte for byte, both engines

The brief's program, unchanged:

```rexx
say .K~classSideHi
::CLASS S MIXINCLASS Class
::METHOD classSideHi
  return "class-side hi"
::CLASS K METACLASS S
```

`od -c` of each descriptor, all three sides:

**The committed corpus program is a superset of this, not this.**
`corpus/lang/class_metaclass.rex:27` declares `K` as `::CLASS K METACLASS S ABSTRACT`, adding the
keyword so the program doubles as the neighbouring success for `class_abstract_metaclass.rex`. The
brief's own five lines are what was run here, unchanged, as their own file.

```text
oracle              rc 0   stdout: c l a s s - s i d e   h i \n     stderr: (0 bytes)
crate REXX_ENGINE=ir         rc 0   stdout: c l a s s - s i d e   h i \n     stderr: (0 bytes)
crate REXX_ENGINE=tree-walker rc 0  stdout: c l a s s - s i d e   h i \n     stderr: (0 bytes)
```

**The `CLASS`-spelling variant stays 97.1.** The same program with `::METHOD classSideHi CLASS`:

```text
oracle                        rc 159   stdout empty
crate REXX_ENGINE=ir          rc 159   stdout empty
crate REXX_ENGINE=tree-walker rc 159   stdout empty
```

stderr identical on all three, opening

```text
     1 *-* say .K~classSideHi
Error 97 running <path> line 1:  Object method not found.
Error 97.1:  Object "The K class" does not understand message "CLASSSIDEHI".
```

compared with `cmp` on the raw bytes, not by eye. That is the row D44 reads as prose if you take
"a metaclass donates methods to the class side" to mean the methods are written `CLASS`: a metaclass
donates its **instance** methods, and the class-side spelling lands in the metaclass's own class
dictionary where nothing merges it. `corpus/lang/class_metaclass_class_method_does_not_donate.rex`
is that program.

---

## 3. `PRIVATE` and `ABSTRACT`, and what is not reachable here

Re-measured on the build as committed, both engines, `cmp` on each descriptor:

| program | oracle | crate, both engines |
|---|---|---|
| `say 'main ran'` / `say .K` / `::CLASS K PRIVATE` | rc 0, `main ran\nThe K class\n`, stderr empty | rc 0, stdout and stderr byte-identical |
| the same with `::CLASS K ABSTRACT` | rc 0, same bytes | rc 0, byte-identical |
| the same with `::CLASS K PUBLIC` | rc 0, same bytes | rc 0, byte-identical |

**What is *not* reachable here, stated rather than implied.**

* **`PRIVATE` versus `PUBLIC` on a class is not covered by these rows.** The row above says the
  keyword parses and the directive installs; it does not say the access is honoured, and it cannot,
  because the difference between a private and a public class is only observable **across a package
  boundary** -- another file `::REQUIRES`-ing this one and finding the name, or not. `::REQUIRES` is
  refused here and is 5c's. The two programs above produce identical bytes under `PRIVATE` and
  `PUBLIC`, which is exactly the shape of a row that would read `agree` under a build that dropped
  the keyword on the floor.
* **`ABSTRACT`'s effect on `~new` is not covered either**, for the same kind of reason: the flag
  `makeAbstract` sets is read by `~new`, which is 5b's, and gate table C files abstract-*class*
  enforcement under `abscla` and 5b.
* **And table D's own `ABSTRACT` probe does not cover it either**, which this section said for
  `PRIVATE`/`PUBLIC` and failed to say for `ABSTRACT`.
  `corpus/gate-tables/directives/class__abstract__subkeyword.rex` is `say 'main'` followed by
  `::class k abstract` -- a program any `ABSTRACT`-ignoring build passes, exactly the shape Task 7
  had to fix for `MIXINCLASS` and `INHERIT` and that this task fixed for `METACLASS`. I did not give
  it a discriminator, and the reason is not that none exists: `::CLASS k MIXINCLASS Class ABSTRACT`
  discriminates, but it is a **refusal**, and every probe in that table must print exactly one line
  of `stdout` ([`expected_oracle_lines`]). The enforcement that a one-line probe could read lands in
  `~new`, which is 5b's and is `abscla`'s row. So the row stays weak, deliberately, and
  `corpus/lang/class_abstract_metaclass*.rex` is where `ABSTRACT` is actually pinned.
* What **is** covered, and is new, is section 5's `98.990`.

---

## 4. The refusal arm is deleted, and its in-crate rows go with it

`directive_gap`'s arm

```rust
DirectiveKind::Class(class) if class.metaclass.is_some() => {
    gap("::CLASS METACLASS", "Phase 5")
}
```

is **deleted, not narrowed** -- there is no `::CLASS METACLASS` string left in
`crates/rexx-exec/src/lib.rs`. The `::CLASS` arm that remains is the namespace one, which is about a
qualified target on any keyword that takes a class reference and names no keyword of its own.

The in-crate table that held its rows loses them **in the same commit**:
`crates/rexx-exec/src/run/tests.rs`'s
`every_directive_this_crate_cannot_install_refuses_before_the_first_clause` loses its
`::class foo metaclass zzznotaclass` row, and `a_class_keyword_gap_is_raised_inside_the_class_pass`
loses its `::class q metaclass zzzm` row. `/bin/grep -rn "CLASS METACLASS" rust/ docs/` returns
nothing outside `docs/superpowers/records/`. **With the positive control**, because an empty result
is a claim about the pattern: the same pattern over `rust/ docs/ .superpowers/` matches 30 lines,
every one of them in a report, ledger or review diff under
`.superpowers/sdd/` or `docs/superpowers/records/`.

**What instrument catches a regression, now that the refusal is gone.** The corpus differential,
and it can see both directions: every shape that was refused is now a differential row this crate
matches, so a build that stopped installing `METACLASS` -- or installed it wrongly -- reddens
`corpus/lang/class_metaclass*.rex`. That is a stronger answer than the constraint's fallback ("an
in-crate test only"), and section 6 is the evidence rather than the claim: every mutation listed
there reddens a named program, measured rather than reasoned about.

---

## 5. What is built

### The metaclass edge

`Interp::install_class_at` now follows `ClassDirective::install`
(`interpreter/instructions/ClassDirective.cpp:165`-`:249`) in order: resolve `METACLASS`, resolve
`SUBCLASS`/`MIXINCLASS`, create the class from the pair, walk the `INHERIT` list, apply `ABSTRACT`.
`Interp::install_class` takes the metaclass as a parameter rather than always passing `.Class`.

`ClassGraph::define_class` gained the two lines `RexxClass::subclass` opens with
(`ClassClass.cpp:1566`-`:1591`):

* a directive naming no `METACLASS` derives from **the superclass's own** metaclass. Measured:
  `::CLASS S MIXINCLASS Class` + `::CLASS K METACLASS S` + `::CLASS J SUBCLASS K` answers
  `.J~classSideHi` at rc 0 on the oracle, and `.J~class~id` is `S`;
* deriving from a metaclass makes the new class one **and overrides the metaclass the directive
  named**. Measured: `::CLASS S MIXINCLASS Class METACLASS M1`, with `M1` a metaclass carrying an
  instance `who`, is 97.1 for `.S~who` -- `S`'s metaclass is `.Class`, not `M1` -- while the same
  `M1` under `::CLASS S SUBCLASS Object METACLASS M1` answers `from M1` at rc 0.

`ClassDef` gained `is_metaclass`, the oracle's `META_CLASS` flag. Nothing derives `.Class` from a
metaclass, so `ClassGraph::bootstrap_metaclass` seeds it there, mirroring
`buildFinalClassBehaviour`'s `if (this == TheClassClass) setMetaClass();` (`ClassClass.cpp:744`).

### The merge position, and where the rebuild happens

`cascade_build`'s `Side::Class` arm already merged the metaclass's flattened **instance** behaviour
(D44, `createClassBehaviour:1116`-`:1129`); nothing there changed. What changed is the reason the
post-attach `refresh_class_behaviour` loop exists, and its comment now says so: a `METACLASS` whose
own `::METHOD` directives arrive after the class was created reaches the class side only once
something rebuilds it, which is exactly what `updateSubClasses`' comment means by building the class
behaviour second *"because the added methods may have an impact on metaclasses"* (`:1036`).

**The ordering question the dispatch asked about.** The loop still runs over `classes.values()` --
no order -- and the *reason* it may is different from the one the old comment gave. The class side
reads each ancestor's own dictionary (walked, not read as a built behaviour) **and** the metaclass's
flattened instance behaviour, which is a built behaviour. That second read is order-independent
anyway, because `ClassRegistry::add_instance_method` rebuilds the receiving class's instance side
and cascades to its subclasses on every call, so every instance behaviour in the file is final
before the loop starts. I did not move the loop into `order`: doing so would make it look as though
the order were load-bearing when it is not, and the honest thing is the comment that says why.

The `check_uninit` / `refresh_parent_has_uninit` pass Task 7 added is untouched. The metaclass edge
is not a superclass edge -- the oracle keeps `metaClass` out of `superClasses` -- so neither `UNINIT`
flag propagates along it.

### Three refusals, each measured

| shape | oracle | frame? |
|---|---|---|
| `::class k metaclass zzznometa subclass zzznosub` | 98.908 rc 158, `Metaclass "ZZZNOMETA" not found.` | none |
| `::class k metaclass object` | 99.927 rc 157, `"The Object class" is not a valid metaclass.` | none |
| `::CLASS S MIXINCLASS Class ABSTRACT` | 98.990 rc 158, `Class S is a metaclass and cannot be made ABSTRACT.` | none |

99.927 is the rc 157 row because it is a *translation* error number, raised from install all the
same -- `RexxClass::subclass` reports it before it builds anything.

Substitutions, each measured rather than inferred: 98.908 takes the target's **upcased spelling**,
and upcases even a quoted literal (`::class k metaclass "zzzm"` names `"ZZZM"`); 99.927 takes the
target's `~defaultName`; 98.990 takes the class's own `~id`, with **no quotes** around it in the
message, and the id is upcased because the directive's name is (`::class s mixinclass class
abstract` names `S`).

**On `blame_native_method`, and the rule has since changed.** None of the three takes a frame,
measured on all of them. I checked my callers against that function's stated rule -- "anything that
reaches a native method's body and raises from inside it owes this line" -- and **they did not fit
it as written**, which I reported rather than bent. `RexxClass::subclass` *is* the body of the
`~subclass` method, and 99.927 is raised inside it, so the rule read literally predicts a frame; the
oracle emits none, because `ClassDirective::install` **calls** `subclass()` rather than sending it,
and it is the activation a send pushes that contributes the line. Task 7's `INHERIT` is the
contrasting case in the same install: the oracle reaches it by `sendMessage` and its refusals do
carry the frame.

**The team lead owns that rule and ruled it over-predicts.** `121bc720d` amends the doc so the
condition is the **send** -- the line is owed wherever the oracle reached the body by a message
send, whatever put the send there -- with `ClassDirective::install` standing as the discriminating
pair by itself (`sendMessage` for `INHERIT` at `:230`; direct calls to `subclass()` and
`mixinClass()` at `:205` and `:200`). The stem-forwarded operator is kept as the illustration that a
*source-level* send term is not what is being asked about: `StemClass::processUnknown` reaches the
method with `value->messageSend` (`classes/StemClass.cpp:280`) and no `~` appears anywhere. The
measurement stays in `Raised::bad_metaclass`'s own doc as well, so a reader arriving from that side
still finds it.

### Error ordering, all measured on the oracle and matched

```text
::requires 'zzznosuchfile.rex' above ::class k metaclass zzzm     43.901 rc 213   (::REQUIRES stage first)
a ::class cycle above ::class q metaclass zzzm                    98.911 rc 158   (ordering before creation)
::class q metaclass zzzm above ::class a subclass zzznotaclass    98.908 rc 158   (source order)
::class a subclass zzznotaclass above ::class q metaclass zzzm    98.909 rc 158   (source order, reversed)
::class k metaclass zzznometa subclass zzznosub                   98.908 rc 158   (within the directive)
::CLASS S MIXINCLASS Class ABSTRACT INHERIT zzznotaclass          98.909 rc 158   (ABSTRACT applied last)
::CLASS S MIXINCLASS Class METACLASS Object                       99.927 rc 157   (checked before the override)
```

The crate matches every one of these byte for byte on both engines except the first, whose
`::REQUIRES` is this crate's standing declared gap (`rc 120`, `::REQUIRES is not implemented
(Phase 5)`) -- the refusal fires at the `::REQUIRES` stage, ahead of the class pass, which is the
staging behaviour it is meant to have and not a regression from this task.

---

## 6. The controls, run

Each mutation was applied to the tree, the workspace rebuilt `--release`, and
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus -- --nocapture` run; the files
were restored from a copy under the scratchpad, not from git, and `cmp` confirmed the restored files
byte-identical to the copies before the final gate run.

**Baseline: 135 of 135, corpus exit 0.**

### The brief's control: rebuilding only the instance behaviour

`ClassGraph::refresh_class_behaviour` changed from `rebuild_behaviour(class, Side::Class)` to
`Side::Instance`:

```text
corpus exit 101, 130 of 135 matching
  [UNCLASSIFIED] lang/class_subclass.rex: stdout, stderr, exit code differ
  [UNCLASSIFIED] lang/expose_two_scopes.rex: stdout, stderr, exit code differ
  [UNCLASSIFIED] lang/class_mixinclass.rex: stdout, stderr, exit code differ
  [UNCLASSIFIED] lang/class_inherit_order.rex: stdout, stderr, exit code differ
  [UNCLASSIFIED] lang/class_metaclass.rex: stdout, stderr, exit code differ
```

The metaclass witness reddens, which is what the brief asks for. Gate table D moves the
`::CLASS METACLASS` row from `agree` to `diverge-both` under the same mutation -- **and exits 0
while doing so**, because 5a is an open phase and a diverged 5a row is reported rather than gated.
So table D's row is a report line here and the corpus is what gates; I am stating that rather than
letting "the row is `agree`" carry more weight than it does.

### The rest

Every row was re-run against the suite as committed, so all denominators are 135:

| mutation | corpus | reddened |
|---|---|---|
| drop the metaclass override in `define_class` | 101, 134 of 135 | `class_metaclass_superclass_wins.rex` only |
| resolve `SUBCLASS` before `METACLASS` | 101, 134 of 135 | `class_metaclass_not_found.rex` only |
| apply `ABSTRACT` before the `INHERIT` loop | 101, 134 of 135 | `class_abstract_metaclass_after_inherit.rex` only |
| let only a `MIXINCLASS` inherit metaclass-ness | 101, 134 of 135 | `class_abstract_metaclass_subclass.rex` only |
| check the metaclass **after** the override | 101, 134 of 135 | `class_metaclass_not_a_metaclass.rex` only |
| delete the metaclass validity check | 101, 134 of 135 | `class_metaclass_not_a_metaclass.rex` only |
| the metaclass also donates its **class** methods | 101, 134 of 135 | `class_metaclass_class_method_does_not_donate.rex` only |
| drop `metaclass` from `class_references` | 101, 133 of 135 | `class_metaclass.rex` and Task 7's `class_metaclass_cycle.rex` |

**One program was deleted because of what these measured, not added.** I first wrote both
`class_metaclass_not_a_metaclass.rex` (`::CLASS M MIXINCLASS Object` + `::CLASS K METACLASS M`) and
a separate `class_metaclass_checked_before_override.rex`. Run against both the "check after the
override" and the "delete the check" mutations, the plain program caught nothing the corner one did
not, while the corner one caught a mutation the plain one missed. So the two were merged into one
program under the readable name, carrying the corner shape, and the plain one is gone -- the suite
is one program smaller and catches strictly more. This is `rust/CLAUDE.md`'s "'can fail' is not
'adds coverage'" applied by running it rather than by reasoning about it.

**What I could not construct a control for.** `class_metaclass.rex`'s `.M~baseClass` line -- a mixin
that also names a `METACLASS`, answering `The Object class`. Every mutation I could think of that
moves it also moves a line some other program already catches, so it earns its place as the
neighbouring-success check for combining the two keywords and not as a discriminator. Saying
"nothing, it was cheap" for it is the honest answer.

---

## 7. Beyond the brief, and what each discriminates

**1. `98.990`, `ABSTRACT` on a metaclass. This is a Task 7 regression closed in Task 8, and the
report says so rather than letting it read as new Task 8 work.** Three builds, both engines,
`::CLASS S MIXINCLASS Class ABSTRACT` and `::CLASS S MIXINCLASS Class` / `::CLASS T SUBCLASS S
ABSTRACT` -- the second column built for this measurement in a throwaway worktree at `d0b7bd151`
and removed afterwards, so every row here is re-runnable rather than recalled:

```text
                                  ::CLASS S MIXINCLASS Class ABSTRACT
oracle                            rc 158  98.990  Class S is a metaclass and cannot be made ABSTRACT.
pinned rexx-run-15a1ffa98         rc 120  rexx-exec: ::CLASS MIXINCLASS is not implemented (Phase 5)
Task 7's head, d0b7bd151          rc 0    main ran                          <-- the regression
HEAD, bb6d46466                   rc 158  98.990, byte-identical to the oracle
```

The `SUBCLASS` shape reads identically at every row, naming `T`. So the keyword went from a **loud
refusal** the pin gives, to a **silent wrong answer**, at the commit that landed `MIXINCLASS` --
which is what makes it a regression rather than a standing gap, and what makes the pinned build the
right before column: it shows the answer was never wrong until Task 7 made the shape reachable.

**Why Task 7's review did not catch it, which is worth recording and is not a criticism of it.**
That review measured a wide set of shapes the committed programs missed, and every one of them was a
`MIXINCLASS` or `INHERIT` shape. **Option *combinations* were not among them** -- `MIXINCLASS` plus
`ABSTRACT` is one keyword interacting with another, and nothing in that task's risk framing pointed
at the combination. The team lead has recorded that as a gap in how the task's risks were framed.

*What it discriminates:* `class_abstract_metaclass.rex` and `class_abstract_metaclass_subclass.rex`
are the only programs in the corpus that read the `ABSTRACT` keyword's effect at all -- section 3
says why every other shape of `ABSTRACT` produces bytes indistinguishable from a build that ignores
it. It cost one `if` over the `is_metaclass` flag the 99.927 check needs anyway.

**2. `class_abstract_metaclass_after_inherit.rex`.** *Discriminates:* the position of the `ABSTRACT`
check inside the directive. Measured to redden alone under "apply `ABSTRACT` before the `INHERIT`
loop", which `class_abstract_metaclass.rex` does not catch.

**3. `class_abstract_metaclass_subclass.rex`.** *Discriminates:* metaclass-ness travelling down
`SUBCLASS` and not only `MIXINCLASS`. Measured to redden alone under the "only a `MIXINCLASS`
inherits metaclass-ness" mutation.

**4. `class_metaclass_superclass_wins.rex`.** *Discriminates:* the `subclass()` override. Measured to
redden alone under "drop the metaclass override".

**5. `class_metaclass_class_method_does_not_donate.rex`.** *Discriminates:* the metaclass donating
its instance dictionary and not its class dictionary. Measured to redden alone under "the metaclass
also donates its class methods", which leaves `class_metaclass.rex` green.

**6. The table D probe rewrite.** `corpus/gate-tables/directives/class__metaclass__subkeyword.rex`
said `say 'main'`, which agrees with the oracle under any build that accepts `METACLASS` and ignores
it. It now says `.k~classSideHi`, one line, the row's own subject -- the same treatment Task 7 gave
the `MIXINCLASS` and `INHERIT` probes and for the same reason. *Discriminates:* it is what makes
gate table C's `xmetac` control ("ignore `METACLASS`") reddenable at all *in table D*; see below for
why it is not reddenable in table C yet.

**7. Two corrections in `crates/rexx-exec/tests/gate_table_c.rs`, both to false statements.**

* `xmetac`'s and `typcla`'s controls each named **Task 8** as the task that can first run them,
  which the field's own doc defines as the task where the row first reads `agree`. Both probes read
  `~id` and `~class~id`. Measured after my change, both are still `rc 120`, `rexx-exec: method "ID"
  of class "Class" is not implemented (Phase 5)`, so neither row can read `agree` here. The plan's
  Task 9 is "the Object and Class reflection protocol" and owns `~class`, `~id`, `~superClass`,
  `~superClasses`, `~metaClass`, `~isA`, `~isSubclassOf`. Both corrected to Task 9, with `xmetac`'s
  saying that `::CLASS ... METACLASS` itself installs from Task 8.
* `typcla`'s control also named `make ::CLASS ... ABSTRACT a no-op`. **That mutation cannot redden
  the row.** The probe's abstract line is `say 'abstract' .ab~id .ab~class~id`, and the oracle
  answers `abstract AB Class` where the plain class's line is `object OC Class` -- an abstract class
  answers `~class~id` exactly as a plain one does, so a build ignoring `ABSTRACT` entirely produces
  identical bytes. I struck that half and said in the arm where abstract-class enforcement actually
  lands (`abscla`, 5b). *Discriminates:* nothing runnable. It stops a later reader running a control
  at a task where the row cannot flip, which is the gate-criteria hazard this plan has a standing
  memory for.

  **The row needs no replacement discriminator, and the reason is stronger than the one I first
  gave.** I wrote that I did not believe one existed before `~new`. The team lead's ruling is the
  better argument and is the one to keep: **`abscla` already *is* the row for abstract enforcement**,
  filed at 5b because the check lives inside `~new`. A discriminator added to `typcla` would
  duplicate an existing row rather than cover a gap, so the question is settled by the table's own
  shape and not by what happens to be reachable this phase.

**8. `docs/superpowers/plans/phase-4-exclusions.txt`.** The `::class foo metaclass zzznotaclass` row
moved out of the "refused here" list (it now matches), an installing `METACLASS` form joined the
"runs here too" list, and the "METACLASS is still refused" paragraph became a CLOSED DEFECT entry in
that file's own house form. *Discriminates:* nothing; it is a record that would otherwise state
something false about the tree. Note that this file's entries are deliberately historical -- the
whole document is a record of where the boundary moved -- so the no-historical-framing rule that
governs source comments does not apply to it, and I kept its form rather than inventing a new one.

**9. `collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS` and `coverage.rs`'s `EXPECTED_SUBSET_5A`.** Both
are committed lists that a new corpus program must join; the first went red on the first run and
told me exactly which programs it wanted. The two 97.1 programs got their own comment there rather
than being filed under one of the existing reasons, because neither existing reason is true of them:
they reach their first clause and raise, and the report's substitutions are rendered out of the
registry and the plan rather than built as values.

---

## 8. The prose sweep

Swept everything I wrote this task against the two rules, by re-reading the added lines rather than
against a list of instances. `git diff | grep '^+'` filtered for historical framing turned up one
candidate ("instance behaviours are final before this loop starts"), which survives the deciding
test -- strike nothing, and it still says what the code does about ordering. Filtered for set sizes
and enumerations, it turned up **more than I would have guessed**, and I struck each:

* "`[ClassGraph::bootstrap_metaclass`] is the one other writer" -- a call-site count, which
  `rust/CLAUDE.md` forbids outright. Replaced with what the function is for.
* "The class side reads two things: ..." -- a cardinality immediately followed by its own
  enumeration. The enumeration stays, the count goes.
* "a directive owing two of them" -> "owing more than one"; "`RexxClass::subclass`'s own first two
  acts" -> "own opening"; "which is why the two share this function" -> "why they share".
* Three phase-boundary claims, which `rust/CLAUDE.md` names as the mutable aggregate that actually
  rots: "the refusal is the whole of `ABSTRACT` that is observable here" in `lib.rs`, "ABSTRACT's one
  effect a program can reach before `~new`" in `phase-5a.txt`, and "ABSTRACT's one reachable effect"
  in `coverage.rs`. Each restated as the property (`a flag whose reader is ~new`) rather than as a
  claim about where the implemented boundary sits.
* "Last of the directive's refusals" in `error.rs` -> "Raised after the directive's other refusals",
  since the first form is a claim about a set of refusals in this crate.

**And the sweep found a false comment that was not mine.** `corpus/lang/class_metaclass_cycle.rex`
(Task 7's) reads "The directive is otherwise refused here, which is why this shape and not an
installing one is what witnesses the edge" -- which this task makes false. Corrected to say what the
program pins now (the ordering runs before any class is created). The same pass caught my own
`class_metaclass.rex` opening naming `~class`, `~metaclass` and `~isMetaClass` as "later work",
which is the same defect in a file where it is *expensive*: editing any byte of a `corpus/lang/*.rex`
reddens `sourceline_matches_the_interpreter_for_every_corpus_program`, so a later task correcting
that sentence would have had to regenerate an expectation file to do it. Both rewritten, and both
expectation files regenerated with the sanctioned `.Package~new` driver from the repository root;
`git status` was clean of stray files afterwards, checked.

New expectation files were generated for every added program the same way. Counts against
`wc -l` of each source file: 37/37, 12/12, 12/12, 6/6, 9/9 (`class_metaclass_cycle`), 7/7, 8/8, 6/6,
12/12.

---

## 9. The performance sitting

Owed, because this task lands code in `src/` of both `rexx-exec` and `rexx-classes`. Run after the
code commit, so the rows carry the commit they measure. Exit **0**, 312 rows appended to
`bench-baselines/phase-5a-arms.tsv`, tagged `task=8 commit=bb6d46466`, against
`bench-baselines/pinned/rexx-run-15a1ffa98` whose sha256 was checked against `PINNED.md` first
(section 1).

**`instructions:u`, `pinned>head`, `across_builds`, k=5** -- every cell is exactly `1.000000` in
median, min and max except the ones below:

```text
alloc4c   tw small   median 1.000000  min 1.000000  max 1.000001
alloc4c   ir large   median 1.000000  min 0.999999  max 1.000000
arith     tw small   median 1.001006  min 1.001006  max 1.001006
arith     ir small   median 1.001283  min 1.001283  max 1.001284
arith     tw large   median 1.001036  min 1.001036  max 1.001036
arith     ir large   median 1.001294  min 1.001294  max 1.001294
strings   tw small   median 1.001990  min 1.000000  max 1.004423
```

**`arith` reproduces what Task 6 and Task 7 each recorded**, and is 0.13% at worst -- under the
threshold and not a finding. Three of its four cells are identical to six decimals across the three
sittings; **`ir small` is not, and claiming an identity there was wrong**: it reads `1.001283` here
against `1.001284` in Task 7, and its own min and max this sitting are `1.001283` and `1.001284`.
That cell is bimodal across those two values and has been in three sittings now, which is a fact
about the cell rather than about anything a task changed -- the same standing this plan gives
`alloc4c ir small`. `/bin/grep -c "::" bench-programs/arith.rex` is
**0**: the axis's program contains no `::` token at all, so nothing this task added to the directive
install can execute on it.

**`strings tw small` is the cell that was not moving on the previous sitting, and it does not
reproduce.** 0.44% at worst, under the threshold and so not a finding by the rule either -- but a
cell going from flat to bimodal is worth a measurement rather than a sentence. Three things, all
measured:

* `/bin/grep -c "::" bench-programs/strings.rex` is **0** as well, so the same argument applies:
  no code this task added is on that axis's path.
* the raw rows say the per-iteration work did not rise. Head's `per_pass tw` is
  `9026.521621 [9004.520977..9044.520759]` against pinned's `9044.521199 [9044.520374..9044.521397]`
  -- head's *median* per-pass is **lower**, and it is the fixed term that swings, head's `fixed tw`
  reading `[-346057.999998..119653548.000000]` against pinned's `[-347362..-344508]`. One round of
  five measured a large fixed component.
* **a second sitting on the same two builds reads `1.000000` in median, min and max** for all four
  `strings` cells, while reproducing `arith`'s four values exactly. Run to a throwaway baseline file
  under the scratchpad, tagged `bb6d46466-confirm`, so the committed TSV holds exactly one sitting
  for this commit.

So the `strings` row is this sitting's own artifact and not the change, established by re-running
rather than by argument -- which is the same instrument the `alloc4c ir small` bimodal cell got, and
`alloc4c` here shows the same thing at a smaller magnitude (`tw small` max `1.000001`, `ir large`
min `0.999999`).

**`cycles:u` across the same cells ranges `0.918153` (`arith ir large`) to `1.020535`
(`alloc4c tw large`)** on axes whose `instructions:u` is `1.001294` and `1.000000` respectively.
That is the pattern the constraint says a `cycles:u` figure carries on its own, so it is recorded
and not read as a result.

---

## 10. What I could not close

* **`::CLASS K METACLASS Singleton` diverges, and it is the `.Singleton` deferral rather than
  anything about `METACLASS`.** Measured: the oracle runs it at rc 0 (`.Singleton~isMetaClass` is
  `1` there); this crate answers 98.908 `Metaclass "SINGLETON" not found.` I checked that this is
  pre-existing and keyword-independent by running `::class k subclass singleton` on the build at
  Task 7's head: 98.909 `Class "SINGLETON" not found.` against the oracle's rc 0, the same shape.
  `.Singleton` is a class this registry does not build and `say .Singleton` is its own loud refusal,
  `rexx-exec: environment symbol ".SINGLETON" is not implemented (Phase 5)`.
* **A sweep of `.environment` for metaclasses found `.Class` and `.Singleton` and nothing else** --
  and I am recording the *pattern* beside the claim, because that is as wide as the claim goes: the
  probe iterates `.environment~allItems`, keeps `item~isA(.Class)`, and asks `item~isMetaClass`. It
  says nothing about a class not registered in `.environment`. What this crate's registry does is
  decided by `ClassGraph::define_class` reading the superclass each class in `native_classes()` is
  given, and every class there but `.Class` is given `.Object`.
* **The `xmetac` and `typcla` concept rows do not move to `agree` here**, for the reason in
  section 7 item 7. D44's *program-level* witness is delivered -- the brief's program runs and
  matches -- but its *table C* row waits on Task 9.
* **`~class`, `~metaClass` and `~isMetaClass` are not implemented**, so the `metaclass` field and
  the `is_metaclass` flag are observable to a program only through the class-side donation and
  through the 98.990 refusal. That is enough for every fact this task pins, and it is why the
  corpus programs are shaped the way they are rather than as reflection queries.
* **No `ir_dual_cases` stanza was added. The reason I first gave was wrong** -- I wrote that
  `message-sends` already runs this dispatch path on both engines, and it does not: no
  `ir_dual_cases` stanza contains `::class` or `::method` at all, so no user class exists in any of
  them and `message-sends` sends only to primitives. Checked with a positive control, because a zero
  is a claim about the pattern: the same `/bin/grep -rlE "::class|::method"` matches **38** files
  under `corpus/lang/` and **0** under `ir_dual_cases/`.

  **The reviewer's reason is the right one and I verified it rather than adopting it.** The
  instrument is the table D probe rewrite together with `run_on_both_engines`, which asserts the two
  engines agree on all three descriptors and says in its own message that a disagreement is *a
  structural failure and not a verdict*. That matters because `corpus.rs` runs `Invocation::none()`,
  which is the IR engine, so the corpus differential sees one engine only. I checked the assertion is
  genuinely unconditional rather than gated: `directive_option_gate_table` appears as `ok` in the
  **ungated** `cargo test --release --workspace` capture as well as the gated one, so the
  `class__metaclass__subkeyword.rex` engine comparison runs with no gate variable set.

  Beside that, every program in this task was run on **both** engines against the oracle and compared
  with `cmp` on all three descriptors (sections 2, 3 and 5).

---

## 11. Files changed

Source: `crates/rexx-classes/src/class_graph.rs`, `crates/rexx-classes/src/registry.rs`,
`crates/rexx-classes/src/native_classes.rs`, `crates/rexx-exec/src/error.rs`,
`crates/rexx-exec/src/lib.rs`.

Tests and tables: `crates/rexx-exec/src/run/tests.rs`, `crates/rexx-exec/tests/collect_stress.rs`,
`crates/rexx-exec/tests/coverage.rs`, `crates/rexx-exec/tests/gate_table_c.rs`.

Corpus: eight new `corpus/lang/class_*metaclass*.rex` programs with their
`crates/rexx-parse/tests/sourceline_oracle/*.txt` expectations, an edit to
`corpus/lang/class_metaclass_cycle.rex` and its expectation, a rewrite of
`corpus/gate-tables/directives/class__metaclass__subkeyword.rex`, and `corpus/phase-5a.txt`.

Records: `docs/superpowers/plans/phase-4-exclusions.txt`.

Commits: `bb6d46466` (the code and the corpus), `96773c448` (the sitting rows), `6754ae2a2` (the
`gate_table_c.rs` control sentence), `121bc720d` (`blame_native_method`'s rule), `8fb97527c` (four
C++ citations). The last three follow team lead rulings and run no sitting: one is `tests/`-only and
the other two are comments, so the release binary the axes measure is byte-identical and a sitting
would measure noise.

**The citations `8fb97527c` moved**, all four printed before and after the edit:

```text
inherit_mixin           ClassDirective.cpp:224 -> :230   :224 is `{`; :230 is the sendMessage
class_not_found         :214-:219 -> :222 and :230       the old range was the loop header and a
                                                         comment; the sentence is about the lookup
                                                         preceding the send, so both are named
abstract_metaclass      :246-:249 -> :247-:249           :246 is the comment above `if (isAbstract())`
install_class_at        ClassClass.cpp:1753 -> :1754     :1753 is the ` */` above the function
```

The first two are Task 7's and the first was wrong by six lines. The last two are mine and neither
was wrong -- each range contained its mechanism -- but a range opening on the comment above the line
it is about is the shape that becomes wrong at the next insertion above it, which is why the team
lead asked for them too.

---

# Fix round 1

Base `8fb97527c`, landed as **`8a88dc63d`**. Same conventions as above: one process per oracle run,
fresh empty directory, absolute paths, three descriptors read separately.

## FR1.1 The latent bug: `~metaClass` and `~class` are two fields

**Re-measured before building anything, because the finding arrived as a claim.** One program, oracle
rc 0:

```text
::CLASS M1 MIXINCLASS Class            T metaclass S        T class     M1
::CLASS S  MIXINCLASS Class            S metaclass Class    S class     Class
::CLASS T  SUBCLASS S METACLASS M1     K metaclass M1       K class     M1
::CLASS K  METACLASS M1                P metaclass Class    P class     Class
::CLASS P
```

The reviewer's row reproduces exactly.

### The exact rule, and the false one it replaces

**THE RULE: `~metaClass` and `~class` part iff the superclass is a metaclass and is not the
named-or-inherited metaclass.** Where they part, `~metaClass` is the superclass and `~class` is the
named-or-inherited metaclass.

**The sentence this round replaces was false, it was the team lead's phrasing, and I lifted it
without testing it** -- *"the split is a property of deriving from a metaclass, and naming
`METACLASS` only chooses which value the `~class` side holds"*. Necessity holds; **sufficiency does
not**. One program, oracle rc 0, `MC` and `M1` both metaclasses:

```text
::class MC MIXINCLASS Class           ~metaClass Class  ~class Class   same
::class Z  SUBCLASS Class             ~metaClass Class  ~class Class   same
::CLASS M3 SUBCLASS MC METACLASS MC   ~metaClass MC     ~class MC      same
::CLASS T  SUBCLASS MC METACLASS M1   ~metaClass MC     ~class M1      part
::CLASS T2 SUBCLASS MC                ~metaClass MC     ~class Class   part
::CLASS K  METACLASS M1               ~metaClass M1     ~class M1      same
```

The first three rows each derive from a metaclass and each coincide, because in each the superclass
*is* the metaclass in play. The false rule predicts a divergence for all three.

**The counterexample was already in this report and in the committed test's own program**, and
neither of us saw it: FR1.1's own table two lines above the sentence read `S metaclass Class  S class
Class`, and `::class S mixinclass class` is the first directive the test installs. A refuting row
sitting inside the evidence for the claim it refutes is the reason this is written as an `iff` with
the coinciding rows kept beside it, rather than as the shape of the divergence.

**The mechanism, and the report had the write order backwards.** Printed, `ClassClass.cpp:1586`-
`:1615` in the order the lines run:

```text
:1579   new_class = meta_class->sendMessage(NEW, class_id)   metaClass := meta_class (newRexx)
:1586   if (isMetaClass())            -- `this` is the SUPERCLASS
:1590       new_class->metaClass = this                       metaClass := the superclass
:1613   new_class->createClassBehaviour(...)                  reads metaClass, already overridden
:1615   new_class->behaviour->setOwningClass(meta_class)      the LOCAL, never reassigned
```

So `:1590` runs **before** `:1615`, not after. The two do not race over one location -- they write
two different ones, and the field write at `:1590` never touches the local `meta_class` that `:1615`
reads. My text said the opposite in six places; the team lead's brief had it right and this round is
what inverted it. **`8a88dc63d`'s commit message carries a copy of the inverted order and cannot be
edited** -- the tree is right and the history is wrong, which is the disposal this plan uses for an
uneditable false statement.

**Fixed at the type level, as ruled.** `ClassDef` now carries `owning_class` beside `metaclass`;
`define_class` binds `owning_class` to the metaclass it was passed **before** applying the override
and `metaclass` after it; `ClassGraph::owning_class` is the new accessor and `ClassRegistry::class_of`
reads it. `ClassRegistry::metaclass` is unchanged, and so is `cascade_build`'s merge, which reads
`metaclass` because `createClassBehaviour` does (`:1123`-`:1127`) -- the merge follows the override,
which is what `class_metaclass_superclass_wins.rex` already pins.

`class_of`'s doc carried the false sentence and now carries the measured shapes instead, including
the pair where the two agree, so a reader cannot take "they differ" as the whole rule either.

### The instrument, and why it is in-crate

`crates/rexx-exec/src/lib.rs`'s `a_class_objects_metaclass_and_its_class_are_separate_fields`. It
goes through **`install_directives`**, not a hand-built graph -- `installed()` parses and installs a
real six-directive program -- so it cannot pass over a layer no program reaches, which is the
Task 7 failure this plan already paid for. Nothing differential can witness it: every observer needs
`~class`, which is Task 9's.

It asserts four classes, and the two that agree are as load-bearing as the two that differ -- without
them the test would admit a build that simply answered different things.

**Shown failing, both collapses, each at a different row:**

```text
class_of reads the metaclass field (the state before this fix)
    FAILED  assertion `left == right` failed: T~class

define_class never applies the override, so both fields hold the named metaclass
    FAILED  assertion `left == right` failed: T~metaClass

restored                                              1 passed; 0 failed
```

Each run reported `running 1 test`, so neither was a filter that matched nothing.

The rows are named by their assertion labels rather than by line number: an earlier draft cited
`lib.rs:5467`/`:5466`, and editing the doc comment above them moved the assertions to `:5469`/`:5468`
before the round was over. A line number into a file this round is still editing is the one citation
that cannot be checked once and left alone.

## FR1.2 The false sentence in the tracked document

`phase-4-exclusions.txt` said "`.Class` is then the metaclass whatever the directive said". The
superclass becomes the metaclass; `.Class` is merely the common case where the superclass *is*
`.Class`. Rewritten to say that, with the measurement, and extended to name the second field --
since the sentence's neighbourhood is exactly where a reader would otherwise conclude that `~class`
follows the override too.

## FR1.3 The set size

`phase-5a.txt` said "The two 97.1 programs are the boundaries of the donation" and then enumerated
them. Replaced with what puts a program in that group -- its send resolves to nothing under the
donation rule the first program measures -- so adding one needs no edit here.

## FR1.4 Minors

Folded into the sections above rather than listed twice: the `:189`/`:223` citation is now
`:191`/`:225`; the `ir_dual` reason is replaced with the reviewer's, verified rather than adopted;
the commit count is restated over the property with its own stamp; the `arith ir small` identity
claim is corrected to the bimodal cell it is; table D's `ABSTRACT` probe is now said to be weak and
why it stays that way; and the committed witness is noted as a superset of the brief's program.

**One I found rather than was given.** `lib.rs`'s `a_bare_class_directive_subclasses_object` doc read
"the only shape that reaches this far, since `SUBCLASS`/`METACLASS`/`INHERIT` are still
`directive_gap` above". All three install -- two by Task 7 and one by me -- so my own commit
falsified it and the review did not catch it either. The clause is gone; the rest of the sentence is
still true and still says what the test pins.

## FR1.5 Two facts from the review, re-measured rather than quoted

**The amended frame rule is discriminated by the pair it predicts.** `.Object~subclass('X', .Object)`
on the oracle:

```text
       *-* Compiled method "SUBCLASS" with scope "Class".
     1 *-* say .Object~subclass('X', .Object)
Error 99 ...  Translation error.
Error 99.927:  "The Object class" is not a valid metaclass.
rc 157
```

The same `RexxClass::subclass` body, the same 99.927, **sent instead of called** -- and the frame
appears. That is the rule's own condition demonstrated from the other side, and it is the strongest
single piece of evidence for the amendment.

**The reviewer independently caught three of the four citations `8fb97527c` fixed**, from the C++
rather than from my report. It missed `inherit_mixin`'s `:224` only because that hunk sits outside
the reviewed diff -- which is the argument for fixing another task's known-wrong citation rather than
leaving it to be rediscovered.

## FR1.6 The sitting question, and the standing method it produced

**Ruled: no sitting, and the predicate is `.text` identity rather than whole-file identity.** The
team lead's original predicate -- two equal whole-file sha256 sums -- cannot hold for this workspace
at all: with `debug = true` the build embeds `.debug_info`, `.debug_line` and a build-id note whose
content varies, so equal sums would have been a result that could not have been true. What follows
is the method that replaces it, written as a procedure because the next comments-only commit in
`src/` should run it rather than re-derive it.

### The method, four parts

1. **Build both commits in one worktree at one path**, `git checkout --detach` between them. Separate
   worktrees at different paths embed different absolute source paths under `debug = true` and
   manufacture a difference out of nothing. I did it the wrong way first.
2. **Run a reproducibility control before reading anything into a difference.** At the later commit,
   `touch` a source file of the crate and rebuild -- a genuine recompile and relink -- and confirm the
   sum reproduces. Without this a difference is indistinguishable from a non-deterministic build.
   Confirm `Compiling <crate>` appears in each build's output, or a cache hit will masquerade as a
   result.
3. **Compare `.text` and the section table, not the whole file.** Whole-file identity is the wrong
   question; what an axis can see is the mapped image.
4. **Extract sections with `objcopy --dump-section=`, never `objcopy -O binary --only-section=`.**
   The latter emits nothing for a non-allocated section, so every `.debug_*` comparison silently
   compares two empty files.

### What it measured here, for `121bc720d` against `121bc720d~1`

```text
121bc720d~1 (6754ae2a2)  59537abfc8abefa6281e3907334b6511fb3f8eeabbe26f590ce2430a0dde948b
121bc720d                9c1946644738ec8b9d727a170ab8fc88baf87f0c941c5dc7138b90531b899dae
```

The sums differ. Everything else says why that does not matter:

```text
file size                 18176528 bytes, both
section table             identical -- every name, address, offset and size, 43 sections
program headers           identical -- every segment offset, vaddr, filesz and memsz
.text                     identical, 1205001 bytes, same sha256
.rodata .data .data.rel.ro .eh_frame .gcc_except_table .got .plt   identical
.debug_info               5178351 bytes both, content differs   (no A flag: never mapped)
.debug_line                843651 bytes both, content differs   (no A flag: never mapped)
.note.gnu.build-id              36 bytes both, content differs   (the linker's hash of its inputs)
```

**So nothing moves.** `instructions:u` cannot change, because `.text` is byte-identical and the same
instruction stream executes. And the `cycles:u` half is stronger than "cycles carries nothing on its
own": the mapped image is identical at identical addresses, so file layout is not a live variable
either. The only differing bytes that are mapped at all are 36, in a note nothing reads at run time.

### The probe defect this round produced, which is the part worth keeping

**A probe that returns a pass by not looking.** My first section sweep used `objcopy -O binary
--only-section=`. That flag emits only allocated sections, so `.debug_line` and `.debug_info` each
extracted as a **zero-length file** and `cmp` reported them identical. The sweep would have told the
team lead that every section matched while the whole files differed -- and those two things cannot
both be true.

**What caught it was the incoherence, not the flag.** Noticing that a result contradicts something
already known is the check that generalises; knowing this particular `objcopy` flag does not. It is
the same family as a `grep` whose pattern matches nothing being read as an absence, which this task
had already hit twice.

**A second slip in the same investigation, recorded beside it.** The first `readelf -S -W` name parse
silently dropped every single-digit section index, because `-W` renders those as `[ 1]` and awk
splits that into two fields -- so a sweep that reported 33 sections had quietly skipped the
low-numbered ones, `.text` among them. Both slips were in the instrument rather than in the subject,
and both produced a confident wrong answer rather than an error.

### Proposed for `rust/CLAUDE.md`, under Gates -- NOT APPLIED

The team lead's ruling: `CLAUDE.md` instructs every agent in this tree and is not edited on a peer's
request, so this stays a draft here for them to put to Moritz. Written in that file's own house
style so it can be lifted verbatim:

> * **A comments-only commit in `src/` owes no sitting, and what proves it is `.text` identity --
>   not whole-file identity, which `debug = true` makes unattainable.** Build both commits in **one
>   worktree at one path**: separate paths embed different absolute source paths and manufacture a
>   difference out of nothing. Run a reproducibility control first -- `touch` a source file of the
>   crate, rebuild, confirm `Compiling <crate>` appears and the sum reproduces -- because without it
>   a difference is indistinguishable from a non-deterministic build. Then compare `.text` and the
>   section table, not the file. Extract sections with `objcopy --dump-section=`, **never** `objcopy
>   -O binary --only-section=`: the latter emits nothing for a non-allocated section, so every
>   `.debug_*` comparison silently compares two empty files and passes by not looking. Measured
>   2026-08-21: a doc-comment commit left `.text` byte-identical at 1205001 bytes with the section
>   table and program headers identical section for section, while a commit adding one struct field
>   moved it to 1205177.

**Not re-run for `8fb97527c`**: it is comments only and the argument transfers.

**And it does NOT transfer to this round's `8a88dc63d`, which is the error worth recording.** I
wrote that "the same `.text` argument covers it". It cannot: FR1.1 adds a field to `ClassDef` and
repoints `class_of`, so the instruction stream changes by construction. The argument was built for a
comments-only commit and stayed attached to the sentence after the commit stopped being one --
**a stale *reason* rather than a stale list, which is harder to see, because the sentence carrying it
stays well-formed and stays true of the thing it was originally written about.** The check that
catches it is the one this task learned for citations, one level up: state the predicate and apply
it, rather than restating the conclusion it produced last time.

**A copy of it is in `8a88dc63d`'s own commit message and cannot be edited.** That message closes
"No sitting. The release binary's `.text` is byte-identical across a comments-only change", and the
very next commit, `c9523023c`, is the sitting that commit turned out to owe -- so the history
contains a claim its own successor refutes. The tree is right and the message is wrong, which is
this plan's standing disposal for an uneditable false statement, and it is the second such copy this
task has produced.

**Measured rather than reasoned, because a predicate only ever asserted to hold is not doing work.**
Both sides of `8a88dc63d` built at one path, both recompiled:

```text
8a88dc63d~1  .text  5c4df2d110950beb385943d10d7e5f0072e2711f3eb6a9e62bdfc3fc936beb3b  1205001 bytes
8a88dc63d    .text  d0aa335f64ea2406fd6803518c3a55ba6c3a1cc1f6407e8d1a20a04ee2ec235e  1205177 bytes
```

176 bytes more instruction stream. So the predicate **held** for `121bc720d` and **fails** here --
one pass and one failure, which is what makes it a rule rather than a way of not running sittings.
FR1.8 is the sitting it required.

## FR1.7 The five gate commands

Run from `rust/`, each status read unpiped:

```text
cargo fmt --all --check                                             exit 0
cargo clippy --workspace --all-targets -- -D warnings               exit 0
cargo test --release --workspace                                    exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 exit 0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   exit 0
```

Zero `failures:` blocks and zero `test result: FAILED` in each of the three test captures,
counted separately. Corpus **135 of 135** -- unchanged, and it should be: this round adds no corpus
program, because nothing it changes is expressible as a differential row. The in-crate test of
FR1.1 is the whole instrument, which is what FR1.1 says and what its two collapse runs demonstrate.

Clippy was run after `touch`ing every source file this round changed, so its green is a re-lint.

## FR1.8 The sitting this round owed

Run against `bench-baselines/pinned/rexx-run-15a1ffa98`, sha256 checked against `PINNED.md` and the
staleness test re-run, tagged `task=8 commit=8a88dc63d` -- this plan's own convention, where a fix
round's rows carry the task number and the commit column tells them apart (Task 7's fix round is
`7`/`c35ba11cc`). Exit **0**, 312 rows.

**`instructions:u`, `pinned>head`, `across_builds`, k=5: every cell exactly `1.000000` in median,
min and max except these.**

```text
alloc4c   tw small   median 1.000000  min 1.000000  max 1.000001
alloc4c   ir small   median 1.000000  min 0.999999  max 1.000000
arith     tw small   median 1.001006  min 1.001006  max 1.001006
arith     ir small   median 1.001284  min 1.001283  max 1.001284
arith     tw large   median 1.001036  min 1.001036  max 1.001036
arith     ir large   median 1.001294  min 1.001294  max 1.001294
```

**The field split moves no axis.** `arith` is the standing pair of cells, and `alloc4c`'s two
single-ulp cells are the artifact this plan already names for that axis. Nothing is at or above 1%,
so there is no finding and no tie-breaker control is owed.

**`arith ir small` reads `min 1.001283, max 1.001284` in every sitting from `c351fa473` onward** --
`c351fa473`, `56c842cb0`, `6aa432f19`, `c35ba11cc`, `bb6d46466`, `8a88dc63d` -- with the *median*
flipping between those two adjacent values across them, while the two sittings before it
(`84f3e580b` at `1.000542`, `211763aaa` at `1.000000`) sat elsewhere entirely. That span and that
commit list are the claim; **an earlier draft said "fourth sitting for that cell" and "three earlier
sittings", both wrong, and a copy of the wrong count is in `c9523023c`'s commit message and cannot
be edited.** I re-derived the list from the TSV myself rather than taking the corrected number,
which is the point: the count is the part that keeps going wrong, and the span is what anyone
actually needs.

That the split moves nothing is the expected shape, and worth saying why rather than only that it
happened. **Narrowed to what was measured:** none of the six axes this sitting ran -- `alloc4c`,
`arith`, `compound`, `emptyloop`, `strings`, `varlookup` -- contains a `::` directive at all,
checked with `/bin/grep -c '^::'` over each of the six, and with `bench-programs/dispatch.rex` as
the positive control at **4**, which is also the counterexample to the wider claim an earlier draft
made: `dispatch.rex` is in `rexx-bench`'s `PROGRAMS` and does install a `::CLASS`. It is simply not
one of the axes this guard measures. So `ClassGraph::define_class` runs only over the bootstrap
classes, once, before any measured pass, and the extra field write lands in the regression fit's
fixed term rather than in `per_pass`.

**`strings tw small` reads `1.000000` in median, min and max**, a third independent reading of the
cell that was `1.001990/1.004423` in the first Task 8 sitting and `1.000000` in the confirmation
one. That closes it as that sitting's artifact rather than anything about the change.

**`cycles:u` ranges `0.946087` (`alloc4c tw small`) to `1.019660` (`compound ir small`)**, both on
axes whose `instructions:u` is `1.000000`. Recorded, not read as a result.

---

# Fix round 2 -- prose only

Base `c9523023c`, landed as **`4729e5d3a`**. **No code changed**: `git diff -U0 -- rust/crates` filtered to lines that are not
`//` or `///` is empty, checked rather than claimed.

**Which side of the predicate this round falls on, stated rather than concluded.** Comments only, so
the FR1.6 predicate applies and says no sitting is owed. I did not re-run the `.text` pair for it --
the round that *did* owe one is the round that changed a struct, and that pair is already in FR1.6
and FR1.8. Saying "the predicate says no" is only worth anything because the predicate has a
recorded failure as well as a recorded pass; on its own it would be the same unfalsifiable claim
this task made once already.

## What the round corrected

Both of the substantive corrections were **errors in the team lead's text that I adopted without
testing**, and both are now stated in the exact measured form with the counterexample beside them:

* **The general rule (N2)** -- FR1.1's "The exact rule, and the false one it replaces". Deriving from
  a metaclass is necessary and **not sufficient**; the refuting rows were already inside my own
  report's table and inside the committed test's own first directive.
* **The oracle's write order (N1)** -- inverted in six places. `:1590` runs before `:1615`; they
  write two different locations and the field write never touches the local the later line reads.
  The C++ is printed in FR1.1 in execution order rather than summarised.

The remaining items are folded in where they belong rather than listed twice: the "discarded
altogether" contradiction in `phase-4-exclusions.txt` (N2b); `native_classes` and `.Object`'s `None`
superclass (N4); `~request` as a second oracle-side reader of `owningClass`, `ObjectClass.cpp:1916`
(N5); the axis claim narrowed to the six measured, with `dispatch.rex` named as the counterexample
that made the wider claim false (N6); the uneditable commit-message copies in `8a88dc63d` and
`c9523023c` (N1, N3, N7); the assertion rows renamed from line numbers to their labels (N8); and
"Either way" replaced (N9).

**Three counts were wrong across two rounds and all three are gone rather than corrected.** The
sitting count, the "fourth sitting" claim, and "three earlier sittings" have been replaced by the
span `[1.001283..1.001284]` and the commit list it holds across, re-derived from the TSV here rather
than taken from the correction. That is the third time on this task that deleting a count beat
fixing it.

## The five gate commands

```text
cargo fmt --all --check                                             exit 0
cargo clippy --workspace --all-targets -- -D warnings               exit 0
cargo test --release --workspace                                    exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 exit 0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   exit 0
```

Each read unpiped. Zero `failures:` blocks and zero `test result: FAILED` in each of the three test
captures. Corpus **135 of 135**. Clippy re-run after `touch`ing every file the round changed.
