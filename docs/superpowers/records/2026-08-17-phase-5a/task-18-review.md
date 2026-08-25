# Task 18 review: the install passes, and class-object initialization

Range `dd83eecdc..4761541c0`. Everything below that says "measured" was run by me, from a fresh
directory, absolute paths, three descriptors read separately, both sides bounded.

**Spec compliance: FAIL, one required fix.** The task's goal is met and discriminated -- three
passes, `INIT` before the `INHERIT` merge and `ACTIVATE` after it, both discriminators byte for byte
on both engines, both required controls really run. **The sitting is not the one the plan
specifies.** The plan's guard command (`docs/superpowers/plans/2026-08-17-phase-5a.md:311`-`:315`,
already there at BASE) names eight axes; the sitting ran six. The two omitted axes are the loudest
in the whole set, and one of them is this task's own path.

**Task quality: PASS, six findings, none of them a defect in behaviour.** The restructure matches the
oracle's shape at every point I could check by running; the citations are overwhelmingly exact; the
concerns are honestly raised rather than buried. The findings are prose, coverage and record-keeping.

---

## 1. The two required controls, run by me

Both applied as mutations of the committed tree, corpus gate read afterwards, tree restored from a
byte-identical copy and rebuilt. `sha256` of `crates/rexx-exec/src/lib.rs` before and after each
mutation: `748f44cf3060fa73b198d2d3d191cdcb0000662ebdc0715e2e167e107e926b09`, and
`git status --porcelain` empty.

**Control 1 -- fire `ACTIVATE` before the merge.** Third pass deleted, the send moved into
`install_class_at` immediately after `INIT`. **193 of 196**, and the mismatching set is exactly
`class_init_activate_inherit_merge.rex`, `class_init_activate_order.rex` and
`class_activate_failure_blames_the_last_installed_class.rex`. The discriminator reads

```
rust:   "K init,     hasMethod MM = 0\nK activate, hasMethod MM = 0\nprologue\n"
oracle: "K init,     hasMethod MM = 0\nK activate, hasMethod MM = 1\nprologue\n"
```

**Control 2 -- resolve constants inside the class pass.** `resolve_constants` moved into the class
loop, blame target untouched. **194 of 196**, mismatching set exactly
`class_constant_expression_later_class.rex` and `class_init_activate_order.rex`.

**Every program that reddens under either mutation is one this task added**, so the suite without
them would have caught neither. The report's claim on this point reproduces.

**A correction to the dispatch brief, not to the report.** The brief says control 2 raises "97.1 on
`The B class`". It does not. Read in full off the mutated binary:

```
    17 *-* ::CONSTANT c (.B~m)
    23 *-* ::CLASS D
Error 97.1:  Object ".B" does not understand message "M".
```

`The B class` is the **BASE** measurement, where every class was created before any method was
attached, so `.B` existed and was empty. Under the mutation applied to the *new* install order, `A`
is constructed before `B` exists at all, so `.B` is an unresolved environment symbol and the receiver
is the literal `.B`. The report's own section 4 does not make the brief's claim; it elides that line.

## 2. The sitting: the required fix

The plan's guard command is eight axes:

```
--axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
--axis dispatchclass --axis bench-rexxcps/rexxcps.rex
```

`phase-5a-arms.tsv` has no `dispatchclass` and no `rexxcps` row under task `18`. The paragraph
immediately below that command block is about exactly this: *"a task that did not happen to inherit
that prose would have run a narrower guard and reported green over an axis it never measured."*

**Mitigating, and it needs fixing too:** `global-constraints.md`, which the implementer was handed as
the binding constraints file, carries the **stale six-axis** form of the command at its line 37. It
is untracked, so nothing versioned catches the drift. The next task inherits the same trap.

**I ran the two missing axes.** `pinned>head`, `instructions:u`, `size` kept:

```
dispatchclass  ir   small  1.018376      dispatchclass  ir   large  1.018414
dispatchclass  tw   small  1.016303      dispatchclass  tw   large  1.016322
rexxcps        ir   small  1.020306      rexxcps        tw   small  1.016592
```

All six above the 1% line, and above every cell the task did report. `dispatchclass` is the send-path
guard axis, and this task adds two message sends per class at install plus a new arm in the
generated-method dispatch match -- it is the axis this change most needs.

**I also ran the contribution arm the plan's own ruling names as the gate quantity**, `base>changed`
with base built from `dd83eecdc` and changed the tree's `target/release/rexx-run`, all eight axes,
five rounds, `instructions:u`, `size` kept:

```
alloc4c/arith/compound/emptyloop/strings/varlookup   1.000000 on every arm and both sizes
dispatchclass  ir 0.999997 (both sizes)   tw 0.999994 (both sizes)
rexxcps        ir 1.000012               tw 0.999995
```

**So the omission hid nothing: this task's own contribution is flat on every axis, the two missing
ones included.** The accumulated `pinned>head` on `dispatchclass` was already over the line at Task
13 (1.0093) and has climbed monotonically since (14: 1.0118, 15: 1.0123, 16: 1.0165, now 1.0184);
`rexxcps` likewise (15: 1.0186, 16: 1.0188, now 1.0203). Under the plan's compiler ruling that
accumulation is a position and not a gate.

**The fix is recording, not re-litigating**: run the eight-axis sitting and commit its rows, and
correct `global-constraints.md`'s command to match the plan. The numbers above are what it will find.

Two checks the report claimed and I confirmed: the pin's sha256 is still
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, and `15a1ffa98` is an ancestor of
`HEAD`.

**One loose noun.** "Two cells at or above 1%" is four once `size` is kept as a discriminator. The
sentence says "both sizes" and the table keeps the column, so the content is right and only the noun
is wrong -- this is the same shape as Task 17's finding, caught much earlier.

## 3. The C++ citations

Every citation added in this range was read with `/bin/grep -n` / `sed -n` against
`/home/moritz/dev/repos/ooRexx/interpreter/`. All resolve to the line the sentence describes, with
the two exceptions below.

Spot-confirmed exact: `PackageClass.cpp:1281`/`:1290`/`:1299` (the three walks, in that order, each
finishing before the next); `ClassDirective.cpp:171`, `:200`, `:205`, `:237`, `:288`, `:501`-`:511`,
`:520`-`:524`; `ClassClass.cpp:1602`-`:1607`, `:1613`, `:1628`, `:1631`, `:1634`-`:1637`, and that
`1562` is where the four-argument `RexxClass::subclass` starts so all of those sit inside it;
`Setup.cpp:497` (including its own quoted comment, verbatim) and `:520`; `ObjectClass.cpp:2546`-`:2549`;
`CPPCode.cpp:438`-`:454` and `:450`; `ActivityManager.hpp:509`-`:515`; `DirectiveParser.cpp:610`-`:623`,
`:355`, `:1865`, `:1933`, `:2520`; `LanguageParser.cpp:1112`.

Two negative claims checked rather than read: `setCurrent` appears **only** at `ClassDirective.cpp:171`
and `resolveConstants` does not call it, so `resolve_constants`'s blame reasoning holds; and the only
assignments to `activeClass` anywhere in `parser/` are `DirectiveParser.cpp:355` and two clears,
`LanguageParser.cpp:843` (inside `initializeForDirectives`, called at `:1110`) and `:1112` -- both
before the directive walk, so "355 is the only assignment after that" holds.

**Finding 1: a citation on a branch its own example never takes.** `record_literal_constants`'s doc
says a literal constant's value is fixed at parse time, *"reached with `value` already set from the
token (`:1875`)"*, and its example is `::constant c 5`. `:1875` is `value = name;` -- the
**value-omitted** arm. `::constant c 5` takes `:1911`, `value = token->value();`. The paragraph
*below* it is the one about the omitted form, and `:1875` belongs there. This is the third task
running to ship this shape.

**Finding 2: a "nothing else" that is false as written.** `install_constant`'s doc says
`createConstantGetterMethod` *"calls `setUnguarded` and nothing else (`parser/DirectiveParser.cpp:2523`)"*.
`:2525` calls `method->setConstant()`. The substantive claim -- no access scope and no protection, so
`record_access_scope` files no row -- is correct; the universal is not.

## 4. Concern 4: `refresh_parent_has_uninit`

**It is genuinely redundant, not an unwitnessed case, and I can say which rather than only that the
suite stays green.**

Measured: with `self.classes().refresh_parent_has_uninit(id);` deleted,
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` exits **0**, 98 `test result: ok`,
zero FAILED, zero panicked, **196 of 196 matching**. That is wider than the report's measurement,
which was against one test.

The argument that closes it: at the call site the class's `superclasses` list holds exactly the one
entry `ClassGraph::define_class` already read, because the `INHERIT` loop that pushes more runs
after; `refresh_parent_has_uninit` is `any(sup) uninit_reaches(sup)` over that one entry, and
`define_class` computed `uninit_reaches` of the same class. Nothing between them can change that
parent's two flags: `has_uninit` is only written by `ClassGraph::define` and `check_uninit`, and
`parent_has_uninit` only by `define_class` and `ClassGraph::inherit`, all of which are driven by
`install_directives`, which finishes one class before starting the next -- and no Rexx construct
reachable in 5a mutates a class that is already built. So it is a second computation of the same
answer under every input, not merely under the inputs the corpus builds. `ClassGraph::inherit` sets
`parent_has_uninit` itself at its own tail, so putting the refresh before the `INHERIT` loop loses
nothing.

Keeping the call, at `ClassClass.cpp:1634`'s own position, is defensible: deleting it leaves
`ClassRegistry::refresh_parent_has_uninit` and `ClassGraph::refresh_parent_has_uninit` as the only
callers of each other, with no caller and no test anywhere. `lib.rs:4406` is the sole call site,
checked.

**Finding 3: the function's own doc justifies it by a caller that does not exist.**
`class_graph.rs`'s `refresh_parent_has_uninit` now reads *"This recomputes it from the class's
current superclass list, which is what a caller that builds a class before the classes it derives
from needs."* No such caller is in the tree -- the one caller builds in dependency order, which is
precisely why the call is redundant. The plain statement the brief asks for ("nothing witnesses this")
is in `lib.rs`'s test doc and is right there; the graph function's own doc says the opposite-shaped
thing, offering a purpose with no referent.

## 5. Concern 5: the duplicate-member divergence

**Confirmed, both shapes, both engines.** Oracle rc 157 in each, echoing the second directive:

```
::CLASS A / ::METHOD m CLASS twice          Error 99.902: Duplicate ::METHOD directive instruction.
::CLASS A / ::CONSTANT c / ::METHOD c CLASS the same 99.902, echoing the ::METHOD
```

Crate rc 0 on both engines, answering `second` and `method` respectively -- the last member
installed.

**Pre-existing, as claimed.** The range removes no duplicate check and `install_method` is untouched;
the oracle's own detection is `checkDuplicateMethod` at `DirectiveParser.cpp:1926`-`:1930` and there
has never been a counterpart here.

**Finding 4: it is recorded only in a source comment and in the report.** Nothing in
`docs/superpowers/plans/2026-08-17-phase-5a.md` mentions 99.902, and `progress.md` stops at Task 17.
Worse, the report's disposal -- *"left for whichever task owns 99.902"* -- names a task that does not
exist: no task in the plan owns duplicate-directive detection for `::METHOD`. This is a **silent wrong
answer**, this project's worst failure mode, with no instrument and no owner. It belongs in the ledger
entry the controller writes for this task, and the plan needs an owner for it or an explicit park.

## 6. The six corpus programs

Each registered in both `corpus/phase-5a.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5A`, each with a
`sourceline_oracle` fixture, and `phase_5a_subset_matches_the_committed_list` pins the pair.

**All six measured by me against the oracle on both engines**, three descriptors compared separately:
stdout, stderr and exit status identical in all twelve runs. The transcripts are the ones the plan and
the report record, including `Error 97.4: Constant "COMPUTED" of object "The A class" has not been
initialized.` and the `::CLASS leaf SUBCLASS middle` blame line.

The shipped `class_init_activate_inherit_merge.rex` -- the discriminator with the chaining line
dropped -- gives the oracle transcript the plan pins, and I confirmed the ruling's premise
independently: the same program **with** `self~init:super` is oracle rc 0 with the identical three
lines, while the crate refuses it rc 120 on both engines with
`a message scope override on "The Class class" is not implemented (Phase 5)`.

## 7. Report and comment claims re-run

Every one of these matched the oracle on both engines, and the BASE column is off a `rexx-run` I built
from `dd83eecdc` in a scratch tree:

| claim | oracle | head (both engines) | BASE |
|---|---|---|---|
| `say .K~init` under a lone `::CLASS K` | rc 165, `No result object.` / `Message "INIT" did not return a result.` | identical | rc 120, `method "INIT" of class "Object" is not implemented (Phase 5)` |
| the brief's discriminator with `self~init:super` | rc 0, the three lines | rc 120, scope-override refusal | rc 0, `prologue` alone |
| `::METHOD init CLASS PRIVATE` | rc 159, 97.2, `::CLASS` clause alone | identical | rc 0, `main` |
| `::METHOD init CLASS PACKAGE` + `activate CLASS PACKAGE` | rc 0, both bodies run | identical | rc 0, `main` |
| `::class a` / `::routine r` / `::method m class` | rc 0, method still attaches | identical | same |
| a lone `::METHOD` under `say 'main ran'` | rc 0 | identical | same |

Three edges of my own, which the restructure could plausibly have broken and did not -- all byte for
byte on both engines:

* a class-side `INIT` that prints, followed by a `::CLASS` naming an unresolvable superclass: rc 158,
  stdout carries `init A ran` **before** the 98.909;
* a class-side `INIT` that raises, with a later `::CLASS` whose `INIT` would print: rc 214, the later
  one never runs;
* a failing `::CONSTANT` expression in a file whose two `INIT`s both print: rc 214 with **both**
  `init` lines on stdout, which is the second pass running after every class is built.

And the two cases the removed class-behaviour rebuild loop existed for, re-measured directly rather
than through their corpus rows: a class method on a superclass declared later reaches `.c` (`from a`),
and a `METACLASS` whose own `::METHOD` follows the class reaches `.K~who` (`from M`).

**Finding 5: one arm of `read_constant` has no witness, and section 8 does not say so.** The
`trap_for(b"NOMETHOD")` branch -- the 97.4 degrading to a `NOMETHOD` condition -- is reachable and
correct: measured, `signal on nomethod` around a class-side read of an unresolved constant expression
is oracle rc 0 `caught NOMETHOD COMPUTED`, matched on both engines. But no corpus program combines
`::CONSTANT` with `nomethod` (checked across `corpus/lang/`), so nothing pins it. The report's "what a
check could not see" lists four things and not this one.

Note also that the doc's claim that the condition carries "the **constant's** name ... and not the
name the send spelled" cannot be discriminated from Rexx at all: the accessor is keyed by the upcased
constant name, so no send that reaches it can spell anything else. The C++ at `CPPCode.cpp:450`
supports the claim; nothing runnable can.

## 8. Constraints

* **No `unsafe`** added, and no comment claims the lint forbids anything -- neither word appears in
  the added lines under `rust/crates`.
* **ASCII only, no em-dashes**: zero non-ASCII bytes among the added lines under `rust/`.
* **No historical framing**: the three `rexx-classes` doc comments and the `behaviour_wiring.rs` one
  that described the old install order were rewritten to describe the code as it stands, and no added
  comment line carries a historical marker. Finding 3 above is the one that goes wrong, and it goes
  wrong by describing a hypothetical caller rather than a past one.
* **No set cardinality**: no added comment names the size of a set. The "two sides", "the pair" and
  "the two senders" phrasings all name their members in the same sentence.

## 9. Gates, run by me at `4761541c0`, tree clean

```
cargo fmt --all --check                                              FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                CLIPPY_EXIT=0, zero warning/error lines
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   EXIT=0
```

98 `test result: ok`, zero `FAILED`, zero `panicked`, **196 of 196 matching**.

**The clippy run was proved live rather than assumed.** Inserting an unused function containing
`v.len() == 0` into `dispatch.rs` makes the same command exit **101** with `length comparison to zero`
and `function ... is never used`; removing it returns it to 0. This matters because the run reports in
about a second off a warm cache, which reads exactly like a check that did not look.

## 10. State the tree is left in

`git status --porcelain` empty at `4761541c004a51faf8d7cc4e1678bed638d4ad0a`.
`crates/rexx-exec/src/lib.rs` and `crates/rexx-exec/src/dispatch.rs` restored from byte-identical
copies after each of the three mutations and the clippy probe, and `target/release/rexx-run` rebuilt
from the restored source afterwards. `bench-baselines/phase-5a-arms.tsv` was **not** written to: both
of my sittings went to scratch files. The BASE build lives in the scratchpad and touches nothing here.
