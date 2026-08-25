# Task 19: `::CONSTANT`'s instance method and class method

**Status: DONE.** Base `9a0f249e9`, head `a39a91ebd`, tree clean, all five gate commands exit 0,
corpus **207 of 207**.

| commit | what |
|---|---|
| `59d320c04` | the implementation, and the `input_oracle.rs` row that is its sole instrument |
| `3aae756b7` | three corpus witnesses, their registrations, and their bookkeeping |
| `13081bf80` | the `phase-4-exclusions.txt` KNOWN GAP moved to CLOSED DEFECTS, in its own commit |
| `a39a91ebd` | the sitting, both arms, eight axes |

---

## 1. The shape this turned out to have, against what the notes predicted

The controller's notes said this might be verification, coverage and bookkeeping, because the brief's
six crate-side readings had all closed under Task 18. **I re-measured all six myself and every one
matches** -- so that part of the notes is confirmed, not taken on trust.

But the brief's **Build** list has an item the notes did not measure: *"the parenthesised form
running as a method against the class object with `self` bound (`resolveConstants`' `setScope`)"*.
That was not built, and probing it found **four divergences from the oracle**, all present at BASE on
both engines. So this task is not bookkeeping: it landed code, and the code closed real wrong
answers.

The lesson worth carrying: the notes were right about every reading they took and wrong about the
task's shape, because the readings were of the *observations the brief listed as diverging* and not
of the *mechanism the brief listed as owed*. A mechanism that is not built shows up only in the
probes nobody wrote down yet.

## 2. What diverged, measured at BASE `9a0f249e9`

Each run from a fresh empty directory holding only that program, absolute paths, three descriptors
read separately, oracle under `ulimit -v 1048576` and `timeout -s KILL 10`, crate under `memcap 1G`
and the same timeout, `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` both.

| program | oracle | crate, both engines at BASE |
|---|---|---|
| `::CLASS K` / `::CONSTANT c (self~id)`, read `.K~c` | rc 0, `K` | rc 159, 97.1 `Object "SELF" does not understand message "ID".` |
| `::CONSTANT c (self)` and `(self~class~id)` | rc 0, `The K class` / `Class` | rc 0, **`SELF` / `String`** -- a silent wrong answer |
| `::CLASS K` / `::CONSTANT a (super~id)` | rc 0, `Class` | rc 159, 97.1 on `"SUPER"` |
| `::METHOD p CLASS PRIVATE` reached as `::CONSTANT c (self~p)` | rc 0, the method answers | rc 159, 97.1 on `"SELF"` |
| `::CONSTANT c (self~hasMethod("D"))` with `::CONSTANT d 7` | rc 0, `1` | rc 0, **`0`** -- a silent wrong answer |
| a failure inside a class method the expression calls | rc 214, the method's clause echo above the two directive echoes | rc 159, 97.1 on `"SELF"` |
| program invoked with one argument, `::CONSTANT c (arg() "\|" arg(1))` | rc 0, `0 \| ` | rc 0, **`1 \| hello`** |

Two of those are silent -- rc 0 with wrong output -- which is the class this project treats as worse
than a refusal.

**Two shapes in the same area were already right at BASE and stayed right**: a forward reference to a
*calculated* constant is the oracle's own 97.4 (`Constant "B" of object "The K class" has not been
initialized.`) on both sides, and variables in a constant expression are uninitialised on both sides
(`::CONSTANT a (x)` answers `X`). Neither needed building.

## 3. What was built

`crates/rexx-exec/src/lib.rs`, three functions:

* `install_directives`'s constants loop passes `classes[index]` -- the class object the constants
  attach to -- into `resolve_constants`, which now carries it beside the blame target it already
  carried. **They are different classes whenever a file declares more than one**, which is the whole
  reason for the pair: the blame target is the class installed last and `SELF` is the constant's own.
* `eval_constant_expression` takes that class and, in the directive frame, binds `SELF` to it and
  `SUPER` to `class_super_scope(class, class)`, and replaces the calling convention with one whose
  receiver is the class object and whose argument list is empty, restoring the caller's on the way
  out.

`SUPER` reuses the function a class-side `::METHOD` already reads through. That reuse is checked
rather than assumed: `::METHOD s CLASS` returning `self~id "and" super~id` answers `J and K` for
`.J~s` and `K and Class` for `.K~s`, byte-identical to the oracle **before** this change, and those
are exactly the strings the constant form has to produce.

The C++ this follows, every line re-read with `/bin/grep -n` on the symbol before it was cited:

* `instructions/ClassDirective.cpp:271` builds the `MethodClass`, `:273` is `code->setScope(classObject)`,
  `:276` is `code->run(activity, classObject, GlobalNames::CONSTANT_DIRECTIVE, NULL, 0, dummy)`.
* `memory/GlobalNames.h:84` is `GLOBAL_NAME(CONSTANT_DIRECTIVE, "::CONSTANT")`.
* `classes/ObjectClass.cpp:616` reads the sender, `:617`-`:620` is the arm my private-method example
  takes (sender and receiver are the same object), `:622`-`:626` is the no-receiver refusal it does
  **not** take. Both are named in the comment, with which one the example takes.
* `classes/ClassClass.cpp:984` is `RexxClass::method`'s signature and `:991` the
  `instanceMethodDictionary->getMethod` retrieval.
* `execution/RexxActivation.cpp:535`-`:536` are the `SELF`/`SUPER` `setLocalVariable` calls.

**One citation I wrote was wrong and I caught it before committing**: the doc referred to
`Interp::invoke_method`, which does not exist -- the function that binds `SELF`/`SUPER` is
`Interp::enter_method_body` (`dispatch.rs:1469`). `/bin/grep -n "fn invoke_method"` found nothing
and the only hits were my own two new lines. A rustdoc intra-doc link to a missing item is not a
compile error here, so neither gate would have caught it.

## 4. The control, which is what the task turns on

**It fires.** The brief's control, run exactly as specified: install the `::CONSTANT` getter on the
class side alone, by guarding `install_class_members`' `Constant` arm with `if class_side`.

```
rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
206 of 207 matching
mismatches (1):
  [UNCLASSIFIED] lang/class_constant_instance_method.rex: stdout, stderr, exit code differ
      rust:   stdout="" stderr="       *-* Compiled method \"METHOD\" with scope \"Class\".\n    25 *-* say .K~method(\"C\")\n..." exit=159
      oracle: stdout="a Method\na Method\nthe class-side method raised 97\n" stderr="" exit=0
```

`.K~method("C")` raises 97.1 with the `Compiled method "METHOD" with scope "Class".` frame where the
oracle answers `a Method`. That is the brief's wording, met.

**"Can fail" is not "adds coverage", so the count is the point.** 206 of 207 with exactly one
mismatch means every other corpus program passes under the mutation -- including
`class_constant_values.rex`, whose `.A~c1` and `.A~hasMethod("C1")` answer identically with only the
class-side getter installed. That is the brief's reason `.K~c` cannot be the control's subject,
confirmed by running it rather than by argument. Across the whole workspace the mutation reddens
`corpus_differential` and `collect_stress.rs`'s zero-collection set (an allocation-accounting side
effect, not a behavioural witness) and nothing else.

### Every mutation run, with the corpus count each produced

| # | mutation | corpus | what reddened |
|---|---|---|---|
| A | the `::CONSTANT` getter installed class-side only (**the brief's control**) | 206 of 207 | `class_constant_instance_method.rex` alone, on `.K~method("C")` |
| B | the instance-side getter only for a value already recorded, so the expression form has none | 206 of 207 | the same program, on its `::CONSTANT c (1+1)` row |
| C1 | `SELF` not bound | 205 of 207 | `class_constant_expression_self.rex` and `class_constant_expression_method_failure.rex` |
| C2 | `SUPER` bound to `.nil` | 206 of 207 | `class_constant_expression_self.rex` |
| C3 | the calling convention's receiver left `None` | 206 of 207 | the same program, on its `PRIVATE` row alone |
| C5 | `SELF` and `SUPER` read from the class installed last, which is the rule the blame target follows | 206 of 207 | the same program, stdout alone: `J / K` twice |
| D | the expression inherits the running program's arguments | 207 of 207 | **nothing in the corpus**; `input_oracle.rs`'s `command_line_arguments_and_the_console_agree_with_the_oracle` alone, across the whole workspace |

Every mutation was applied to a copy-restored `lib.rs` (`cp` from a pristine copy, never
`git checkout`), rebuilt, run, and the file restored from that copy. The tree is clean at
`a39a91ebd` and `target/release/rexx-run`'s sha256 is
`8d4a633df899e26ac39f006d1f95c8c77ea2768fd81141cba4069bc036fdd810`, distinct from the BASE build
`59ceb09cd36c91876ac9b1e6dc3a564798e1f94ae4c4abd97c19935a2bd4a939` -- so no mutated or stale binary
outlived its revert.

Row D is the one worth reading twice: it is a real oracle divergence with **no corpus expression at
all**, because the corpus harness passes no arguments and with none the expression and the main body
read alike. That is why the witness lives in `input_oracle.rs`, which is the one differential in this
tree that runs a program with a command line.

## 5. The corpus, 204 -> 207

**`corpus/lang/class_constant_instance_method.rex`** -- M7's own witness. `~method` retrieves from
the class's own `instanceMethodDictionary`, so `.K~method("C")` answering `a Method` says the getter
is on the instance behaviour with no `~new` anywhere. Rows: the literal form, the expression form,
and a class-only `::METHOD` under the same `::CLASS` reached under `SIGNAL ON SYNTAX`, which is what
makes the constant's answer mean *which dictionary it came out of*.

The class-only contrast is not new to the tree -- `class_method_own_dictionary.rex` pins the
dictionary rule for a `::METHOD` and an `::ATTRIBUTE`. What is new is pairing the two member kinds
**inside one class**: no program there declares a `::CONSTANT`, so none of them can tell a build that
put constants somewhere else from one that did not.

**`corpus/lang/class_constant_expression_self.rex`** -- the expression's own activation. Two classes,
`::CLASS J SUBCLASS K`, each with `::CONSTANT c (self~id "/" super~id)`: `K` answers `K / Class` and
`J` answers `J / K`. Both halves need the pair, because a build reading either out of one fixed class
agrees with at most one row -- and mutation C5 is that build, measured. The third row is a `PRIVATE`
class method reached as `::CONSTANT p (self~secret)`, which is the receiver half and which `SELF`
alone does not reach (mutation C3).

**`corpus/lang/class_constant_expression_method_failure.rex`** -- the failure path through that
activation. A condition raised inside a class method the expression calls echoes the method's clause
above the `::CONSTANT` and the `::CLASS`, rc 214, stdout empty.
`directive_constant_expression_fails.rex` beside it has those two directive echoes and no method
echo. **Fix round 1 corrected this paragraph**: it claimed the program was the only one putting a
real activation between the raising clause and the directives, and
`class_activate_failure_blames_the_last_installed_class.rex` is a counterexample already in the
corpus -- oracle rc 214, `13 *-* x = 1/0` then `15 *-* ::CLASS leaf SUBCLASS middle`, which I ran.
What the program actually adds is a method echo above a `::CONSTANT` echo **and** a `::CLASS` echo
together.

Registered in `corpus/phase-5a.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5A` together, as required.

### Bookkeeping two other gates demanded, which the brief could not know about

* `collect_stress.rs`'s zero-collection set gains `class_constant_expression_method_failure.rex`.
  Its two siblings allocate and are deliberately absent; the comment says so.
* `crates/rexx-parse/tests/sourceline_oracle/` gains one `.txt` per program, captured with the
  `.Package~new` driver in that test's own module comment -- the sanctioned exception to the
  never-instantiate-a-repository-file rule. The failure program takes the driver's fallback path
  (its install raises), which is faithful here because the file has a trailing newline, no CRLF and
  no embedded `CTRL-Z`, the three shapes that module names as making the fallback unfaithful.
  **Editing any byte of a corpus program means regenerating its `.txt`, and I did it again after
  every comment edit** -- the last regeneration is after fix round 1's wording change.
  **Fix round 1 turned that argument into an assertion**; section 11 has why.

## 6. `phase-4-exclusions.txt`, in its own commit (`13081bf80`)

The KNOWN GAP -- a `::CONSTANT` expression cannot send to a class declared later in the same file --
moves to CLOSED DEFECTS. I ran its own transcript program: **rc 0 `main` on the oracle and on both
engines**, three descriptors, 2026-08-24.

The row keeps its transcript, its `62de43c0f` provenance paragraph and its verdict, and gains what
this file's CLOSED DEFECTS convention requires: what closed it (`34cd90a4a`, Task 18's three install
passes -- confirmed with `git log -S"fn resolve_constants"`, which names exactly that commit) and
what would go red without it (`class_constant_expression_later_class.rex`).

**One thing the move broke and I fixed with it**: the paragraph beneath it opened *"THE BLAME TARGET
OF A FAILING `::CONSTANT` IS A DIFFERENT QUESTION"*, whose referent had just moved out of the
section. It now names both questions and says which section holds the other. A moved row can leave a
dangling *"a different question"* behind it, and nothing in the file's own gate can see that.

I searched the tree for other references to the moved row: the only ones outside the SDD workspace's
own historical reports are the plan and the brief, which are records of what was true when written.

## 7. The sitting

Eight axes. **I checked `global-constraints.md`'s guard block against the plan's before running**:
they agree now, both eight axes, `dispatchclass` and `bench-rexxcps/rexxcps.rex` included.

**Staleness test.** `git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock
Cargo.toml` lists this phase's own task commits and nothing foreign; the three most recent
(`0baa2ccae`, `ac2e92bf0`, `14479a7fa`) are named in `task-18-report.md`, which `progress.md`'s Task
18 entry closes on. `sha256sum bench-baselines/pinned/rexx-run-15a1ffa98` is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`.

**The contribution arm, the gate quantity.** `base` built from `9a0f249e9`'s crate source (only
`lib.rs` differs under `src/` between BASE and head, and no `Cargo.toml`/`Cargo.lock` does, checked
against a full `git diff --stat` rather than a pathspec), `changed` from the branch head,
`base>changed`, `instructions:u`, five rounds:

```
alloc4c arith compound emptyloop strings varlookup   1.000000, every arm, both sizes
dispatchclass  tw 1.000003 small / 1.000006 large    ir 0.999996 small / 0.999993 large
rexxcps        tw 1.000009                           ir 1.000006
```

Nothing above 0.001%. That is what a change on a path no bench axis reaches looks like: no axis
declares a class, so no `::CONSTANT` expression runs on any of them.

**The accumulated `pinned>head` position** matches the previous sitting's to five or six digits on
every axis except one cell: `dispatchclass ir small` reads **1.020427** against that sitting's
**1.018380**, while `dispatchclass ir large` reads 1.018412 against its 1.018408.

**Two corrections fix round 1 made to this paragraph.** The sitting those figures come from is task
**`18-fixround-3`** (`0baa2ccae`), not task `18`: task `18`'s own rows in this TSV carry six axes and
no `dispatchclass` at all, checked. And **`dispatchclass ir small` is not one of the cells the ledger
records as bimodal** -- those are `alloc4c ir small` and `dispatchclass tw small`. So the record
supports only that this axis has produced such a cell on its other arm, not that this cell is one.
**The tie-breaker the plan names is the contribution arm, and it is flat to a millionth**, which is
what the conclusion rests on either way.

`cycles:u` on the contribution arm spans **0.979953** to **1.035493** over the same pair of builds
whose `instructions:u` is flat to a millionth. That is a third instance of the pattern the plan's
constraint was written for, on builds that differ by a function nothing on those axes calls.

## 8. Gates, run by me at `a39a91ebd`, tree clean

```
cargo fmt --all --check                                              FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                CLIPPY_EXIT=0
  and again with a fresh CARGO_TARGET_DIR                            COLD_CLIPPY_EXIT=0
cargo test --release --workspace                                     REL_NOGATE_EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                  REL_GATE_EXIT=0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   DEBUG_GATE_EXIT=0
```

Both gated runs: **98** `test result: ok` blocks, **zero** `FAILED`, **zero** `panicked`. Neither
was piped when its exit code was read. The cold clippy checked `rexx-exec` and `rexx-parse` among
its 66 `Checking` lines and printed zero lines beginning `warning` or `error`.

The corpus matching line, verbatim from the release gate:

```
rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
207 of 207 matching
```

### One red I saw once and could not reproduce

The **first** gated workspace run after the code change reported `error: 1 target failed: -p
rexx-exec --lib`, with the failure names cut off by a `tail -60` I should not have piped. Against the
same source and the same binaries I then ran the lib suite three times through cargo (728 passed each
time), the full workspace gate three times, and each of the two `rexx_exec` lib test binaries
directly ten times: **exit 0 every time, no failure, 26 runs**. I could not reproduce it and I do not know
what it was; other agents were active in this session and share this checkout's `target/`, which
would explain a test that reads a binary mid-relink, but that is a hypothesis I did not test.
Recording it rather than dropping it, because "any red is a regression" and an unreproduced one is
still a red somebody saw.

## 9. Debt, stated so it is not implied to be covered

**The instance-side reading is 5b's, and `~method("C")` does not cover it.** `~method` says the
getter is *installed* on the instance behaviour; sending the constant to an *instance* needs `~new`,
which this phase does not have. The plan already files this under 5b's list -- *"the instance-side
reading of every 5a limit measured only on a class object -- the old plan's Task 7 debt, this plan's
Task 13 (`PRIVATE`), Task 19 (`::CONSTANT`) and Task 21"* -- so it needs no new row, and this
paragraph exists so the report cannot be read as claiming otherwise.

## 10. What these checks could not see

* **The corpus cannot see the argument-list rule.** Row D above is the proof: the mutation that
  reinstates the old behaviour leaves the corpus at 207 of 207. `input_oracle.rs` is the sole
  instrument, and it runs the **default engine only** (`run_rust` sets no `REXX_ENGINE`). I measured
  that program on both engines by hand and they agree byte for byte; nothing in the tree re-checks
  the tree-walker on it.
* **`SUPER`'s value is checked against the class-side `::METHOD` path, not against the C++
  independently.** If `class_super_scope` were wrong, the constant form and the class-method form
  would be wrong together and both corpus rows would still agree with each other. What stops that is
  that both are compared against the oracle, not against each other.
* **The `::CONSTANT` message name is written into the calling convention and nothing reads it back.**
  The failure path I have a witness for prints no method name. If the oracle exposes it somewhere I
  did not probe, this crate would answer `::CONSTANT` there by construction, but that is not a
  measured agreement.
* **Nothing checks that the constants pass runs each class's expressions in one activation.** The
  oracle accumulates a class's expressions into one `MethodClass`; this crate runs each expression in
  its own frame. No expression can assign a variable, so I found no observable that separates them --
  but "I found no observable" is weaker than "there is none", and I did not enumerate the ways an
  activation can be observed.
* **The mutations are of my own choosing.** They cover each half of what I built. A build wrong in a
  way I did not think to write down would pass all seven.

---

# 11. Fix round 1

Commit `7130b222e`. Gates re-run there, tree clean: `cargo fmt --all --check` 0;
`cargo clippy --workspace --all-targets -- -D warnings` 0 with zero lines beginning `warning` or
`error`; `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` 0, 98
`test result: ok`, zero `FAILED`, zero `panicked`, `207 of 207 matching`; and
`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` 0 on the same bytes. **No
sitting**: this round changes comments and adds test assertions, so the release binary the axes
measure is unchanged in behaviour and a sitting would measure noise.

All three review defects are the same shape -- a claim that was true before this change and that this
change's own new witness contradicts. None is behavioural and none is caught by any gate. Below is
what each turned out to be when I went to fix it, because two were wider than the finding.

## D1 -- `resolve_constants`' Err arm said there is no real activation nesting here

True before this task; `class_constant_expression_method_failure.rex` is the counterexample the same
commit added. The comment now says the two directive clauses stand in for nesting **between
themselves, and only between themselves**, and that a method the expression calls is a real
activation that has already sealed its own site below them.

**The transcript I quote is a probe, not the corpus file, and that was deliberate.** The finding's
own transcript quotes `22`/`23`/`20`, which are line numbers in
`class_constant_expression_method_failure.rex` -- a file D2 was about to edit. Quoting them would
have made the D1 fix false the moment the D2 fix landed. The comment instead extends the probe the
paragraph above it already uses (`::class K` / `::constant c (1/0)`, echoes at `4` and `3`) by moving
the divide into a class method, and quotes that: `5 *-* return 1/0` / `6 *-* ::constant c (self~m)` /
`3 *-* ::class K`. I ran both probes to confirm the base numbering and the extension.

## D2 -- a false universal in the corpus program, and in the report

Confirmed by running the reviewer's counterexample myself:
`class_activate_failure_blames_the_last_installed_class.rex`, oracle rc 214, `13 *-* x = 1/0` then
`15 *-* ::CLASS leaf SUBCLASS middle` -- a method activation with its clause above a directive echo,
already in the corpus. My sentence was false.

The comment now names both neighbours and what each has:
`directive_constant_expression_fails.rex` two directive echoes and no method echo; the ACTIVATE
program a method echo above one directive echo; and this program a method echo above a `::CONSTANT`
echo **and** a `::CLASS` echo together. The same correction is made in section 5. The program's
`sourceline_oracle` expectation was regenerated afterwards.

## D3 -- and it was much wider than one file

The comment said the driver takes the fallback path on `trace_numeric_request.rex` "and the primary
path everywhere else". I ran that module's own driver over **every** program in `corpus/lang` with
the path it took reported, and **90 of 213 take the fallback** -- `directive_constant_expression_fails.rex`,
every `class_duplicate_*`, every `operator_frame_stem_*`, `class_method_own_dictionary.rex`, and so
on. Constructing a package runs the file's prolog and installs its directives, so *every witness
program whose purpose is a failure* takes it. My new file is one contributor to a sentence that was
already wrong about most of the corpus.

**That figure is 90 and commit `7130b222e`'s message says 91.** I counted the listing by eye the
first time and then counted it with `wc -l` over the same walk, twice, both 90. The message cannot be
amended, so the correction lives here; nothing committed carries the number, because the module
comment deliberately says "a large fraction" instead of a count.

**So a prose fix would have been the same defect one file later**, which is why I did more than the
minimal edit and am flagging it:

* The module comment no longer lists which programs take the fallback. It states the rule (a file
  takes it when constructing its package raises), says a large fraction of the corpus does, and moves
  the load-bearing part to the condition that has to hold: no CR byte, no `CTRL-Z`, and a final
  terminator on every file except the one whose name says it has none.
* `sourceline_matches_the_interpreter_for_every_corpus_program` now **asserts** that condition over
  every corpus program. Measured before writing it: zero CR bytes and zero `CTRL-Z` bytes across
  `corpus/lang`, and exactly one file without a trailing newline.
* The `saw_unterminated_final_line` boolean became the set of names, asserted equal to
  `["no_trailing_newline"]`. It keeps the original assertion's guarantee -- the criterion's edge case
  is present among the files -- and adds that nothing else lacks a terminator.

**All three new assertions inverted, to prove them live**, each restored from a `cp` afterwards:
the CR assertion reddens naming `condition_syntax`, the `CTRL-Z` assertion reddens naming
`condition_syntax`, and the set equality reddens with `left: ["no_trailing_newline"] right: []`.
`sourceline_oracle.rs` is byte-identical to its pre-inversion copy, sha256
`437b00bcefbdb749e1f437988001f60c7e388fb2ac763e0e1c65926d07d9f8bf`.

`no_trailing_newline.rex` takes the **primary** path -- it is absent from the 91 -- and its
expectation reads `count 7` over a file with 6 newline bytes, so the "7 lines under `~source`, not 6"
claim I restated is one I checked rather than copied.

## The two minors

* **(a) The ledger's coverage sentence.** Corrected. Every row of the two new rc-0 programs has a
  mutation that reddens that row alone; the failure program is reached only by the `SELF` mutation,
  which reddens its sibling too, so its marginal coverage over that sibling is not demonstrated by
  any mutation I ran. That is the reviewer's own combination-witness finding, now stated in the
  ledger rather than only in their report.
* **(b) `lib.rs`'s "runs before either engine's own instruction loop starts".** Corrected, and I am
  confident: a constant expression can call a `::ROUTINE` (it always could) and now a method, and
  both run under the selected engine. The sentence now separates the two claims -- the expression's
  own operators reach the shared `Interp::eval` that both loops call, and a body it calls does enter
  a loop, with both engines agreeing with the oracle on that body.

## The single-engine statement, moved into the tree

`run_rust`'s doc now says it sets no `REXX_ENGINE`, so every row in that file compares the default
engine alone, **and that `corpus.rs` does the same with `Invocation::none()`** -- checked,
`corpus.rs:246`, and no `REXX_ENGINE` anywhere in that file. That is the reviewer's correction to my
framing: row D is no worse covered than any corpus row, and my report's section 10 made it sound
uniquely exposed. The case's own `why` points at `run_rust` for it.

## Two claims of the reviewer's I checked rather than accepted

* The counterexample in D2 -- ran it, it is real.
* "task `18`'s own rows carry six axes and no `dispatchclass`" -- `/bin/grep -cP "^18\t.*dispatchclass"`
  on the TSV answers `0`, and the figures I had compared against carry `18-fixround-3` and
  `0baa2ccae`. The reviewer is right and my report and commit message both said "Task 18's". The
  report is corrected; the commit message stands as written, since amending is out.

## What this round could not see

* **Nothing checks that a corpus program's expectation was captured on the path the module says.**
  The new assertions check the *conditions* under which either path is faithful, not which path any
  file took. A file meeting all three conditions and captured wrongly for some fourth reason would
  pass.
* **The 91-program figure is a measurement of today's corpus**, taken with a modified copy of the
  module's driver that reports its path. It is in this report and deliberately not in the comment,
  because it is exactly the kind of in-repo aggregate that rots.
* **No mutation was run this round**, because no behaviour changed. The claim that nothing behavioural
  moved rests on the gates and on `git diff` showing only comments plus the new assertions.
