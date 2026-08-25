# Task 18 report: the install passes, and class-object initialization

**Sections 1 to 11 are the first round, unchanged except where the review corrected a figure.
Section 12 is fix round 1.**

**Status: DONE_WITH_CONCERNS.** Both discriminators match byte for byte on stdout, stderr and exit
status under `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`; both required controls were run rather
than argued; every gate is green. The concerns are three, and none of them is a defect in what
landed: the brief's own discriminator program contains a construct this crate refuses at rc 120 and
I dropped that line, the machine's `rustc` moved out from under the performance pin, and `clippy`
was already red at BASE for that same reason.

BASE `dd83eecdc`. Commits, oldest first:

| SHA | what |
|---|---|
| `f880a4753` | the pre-existing `clippy` failure, fixed in a commit of its own |
| `34cd90a4a` | the implementation |
| `423526a86` | the six corpus programs and their registration |
| `88bab13f1` | the guard-axis sitting |

---

## 1. What the two discriminators did at BASE, re-measured by me

Both reproduce exactly as `task-18-controller-notes.md` describes. Fresh directories, absolute
paths, three descriptors read separately, both sides bounded.

**The `INIT`/`ACTIVATE` pair was a silent wrong answer.** Oracle rc 0 with three lines; crate rc 0
on both engines with `prologue` alone and stderr empty. The crate ran the file, exited clean, and
fired neither method.

**The constant forward reference was a wrong raise.** Oracle rc 0 `from B`; crate rc 159 on both
engines, `Error 97.1: Object "The B class" does not understand message "M".`, blaming
`3 *-* ::CONSTANT c (.B~m)` then `4 *-* ::CLASS B`.

## 2. What landed

`Interp::install_directives` now has the shape `PackageClass::processInstall` has: one walk of the
dependency-ordered class list to create every class (`classes/PackageClass.cpp:1281`), a second to
resolve every `::CONSTANT` expression (`:1290`), a third to send `ACTIVATE` (`:1299`). Each walk
finishes before the next begins.

A class's own `::METHOD`, `::ATTRIBUTE` and `::CONSTANT` directives are attached **inside that
class's own install**, which is the half that makes the pair observable. `class_members` computes
the positional attachment (R9) once; `install_class_at` then follows `ClassDirective::install`'s own
order: create the class with its **class-side** members already in it (the enhancing methods
`subclass`/`mixinClass` take, `ClassDirective.cpp:200`/`:205`), `checkUninit`, the `INIT` send
(`ClassClass.cpp:1628`, `:1631`), the parent-`UNINIT` propagation, the `INHERIT` sends, the
**instance-side** members (`defineMethods`, `ClassDirective.cpp:237`), then `ABSTRACT`.

`("Object", "INIT")` and `("Class", "ACTIVATE")` are new `NATIVE_METHODS` rows, both bound to one
no-op `native_no_op` because `memory/Setup.cpp:520` and `:497` bind one C++ function
(`RexxObject::initRexx`) under both names. Without them every `::CLASS` would refuse at rc 120 --
measured at BASE, `say .K~init` under a lone `::CLASS K` is already
`method "INIT" of class "Object" is not implemented (Phase 5)`.

**Reading a `::CONSTANT` back is new with it, because the second discriminator needs it.** `say
.A~c` was 97.1 at BASE for every form of the directive. One directive now installs a
`GeneratedKind::Constant` accessor on each dictionary side, as
`ClassDirective::addConstantMethod` does (`instructions/ClassDirective.cpp:520`-`:524`);
`Interp::constant_values` holds the value, keyed by the directive and rooted through
`RootSet::add_global`. A literal value is recorded while the class is built and an expression's in
the second pass, so a class-side `INIT` can read the first and not the second -- which is the
oracle's own 97.4, `Constant "C" of object "The A class" has not been initialized.`, and not a name
miss. That report is reachable and is now a corpus row.

### Two things the restructure removed or moved, each re-measured

* **`check_uninit` and `refresh_parent_has_uninit`** moved from a pass after the directives into
  each class's own install, at `RexxClass::subclass`'s two points. **Only the first is still
  witnessed**, re-measured by deleting each on its own against
  `the_uninit_flags_are_set_for_the_classes_a_file_declares`: dropping `check_uninit` fails the
  `has_uninit(kid)` row; dropping `refresh_parent_has_uninit` leaves the test green, because under
  this order `ClassGraph::define_class` already computes the parent flag from a finished parent and
  the call is a second computation of the same answer. I kept the call -- it is `ClassClass.cpp:1634`
  and costs nothing -- and said so in the test's doc rather than leaving a false "each call is
  witnessed separately". **This is a decision worth the controller's ruling: the alternative is
  deleting the call, which leaves `ClassRegistry::refresh_parent_has_uninit` and
  `ClassGraph::refresh_parent_has_uninit` with no caller anywhere and no test.**
* **The class-behaviour rebuild loop** after the directives is gone; its premise ("this crate creates
  every class the file declares before it attaches any method") is false now. Both cases its own
  comment named were re-measured against the oracle after the removal and match byte for byte on
  both engines: a class method on a superclass declared *later* in the file reaches `.c` through
  `::class c subclass b` / `::class b subclass a` / `::class a`, and a `METACLASS` whose own
  `::METHOD` follows the class reaches `.K~who`. The corpus rows
  `class_metaclass_class_method_does_not_donate.rex` and `class_metaclass_superclass_wins.rex` cover
  the same edge and stayed green.

Three doc comments in `rexx-classes` and one in `behaviour_wiring.rs` described the old install
order and were corrected rather than left to rot.

## 3. The deviation from the brief's own program, and why

**The brief's discriminator carries `self~init:super`; the program I committed does not.**

`Interp::message_term` refuses **every** message scope override, not only `SUPER`'s:

```
rexx-exec: a message scope override on "The Class class" is not implemented (Phase 5)
```

rc 120, both engines, measured on the brief's program verbatim once `INIT` began firing. The
refusal is unconditional at `dispatch.rs`, it is pinned by an in-crate test
(`a_scope_override_is_loud_on_a_class_object_and_88_914_on_anything_else`) and by the corpus row
`message_send_scope_override.rex`, and its own comment says this phase implements neither the 93.957
subclass check the construct needs nor the `SUPER` sends that would want it. So the construct is
another task's, and the controller's note that "the old plan's Task 5 landed it" is true of
`Interp::resolve`'s `start_scope` parameter and not of the expression that would reach it -- the
mechanism exists and nothing can call it.

**The transcript is unchanged without the line.** Measured on the oracle, the program with
`self~init:super` removed:

```
K init,     hasMethod MM = 0
K activate, hasMethod MM = 1
prologue
```

byte for byte the brief's own transcript, rc 0. That send exists in the brief only because it stands
in for the spec's `forward class (super) continue`, whose job was to chain the superclass `INIT`;
`K`'s superclass is `.Object`, whose `INIT` is a no-op, so chaining to it changes nothing this
program asks about. The discriminating property -- `0` from `INIT` and `1` from `ACTIVATE` -- is
untouched.

I did not implement the scope override. It is a construct with its own refusal ladder, its own
witnesses and its own task, and nothing in this task's goal needs it.

## 4. The two required controls, run and read

Each was applied as a mutation of the committed tree, with `REXX_CORPUS_GATE=1 cargo test --release
-p rexx-exec --test corpus` read afterwards and the tree restored from a copy.

**Control 1 -- fire `ACTIVATE` before the merge.** The third pass deleted and the send moved into
`install_class_at` immediately after `INIT`, before the `INHERIT` loop. **193 of 196.**

```
lang/class_init_activate_inherit_merge.rex: stdout differ
  rust:   "K init,     hasMethod MM = 0\nK activate, hasMethod MM = 0\nprologue\n"
  oracle: "K init,     hasMethod MM = 0\nK activate, hasMethod MM = 1\nprologue\n"
```

plus `class_init_activate_order.rex` and
`class_activate_failure_blames_the_last_installed_class.rex`.

**Control 2 -- resolve constants inside the class pass.** The `resolve_constants` call moved into the
class loop, immediately after each `install_class_at`, with the blame target left alone so that only
the ordering moves. **194 of 196.**

```
lang/class_constant_expression_later_class.rex: stdout, stderr, exit code differ
  rust:   rc 159, "    17 *-* ::CONSTANT c (.B~m)\n    23 *-* ::CLASS D\nError 97 ... "
  oracle: rc 0,   "from B\nfrom B through D\n"
```

The refusal under that mutation is `Error 97.1: Object ".B" does not understand message "M".` --
the receiver is the literal `.B`, an unresolved environment symbol, because with the mutation applied
to *this* install order `A` is constructed before `B` exists at all. The dispatch brief quotes
`The B class` for it; that is the **BASE** reading, from the order in which every class was created
before any method was attached.

plus `class_init_activate_order.rex`.

**No program that predates this task reddens under either mutation.** That is the "adds coverage"
check rather than the "can fail" one: the mismatching set under each control is exactly the new
programs, so the suite without them would not have caught either.

## 5. The six corpus programs

Registered in both `corpus/phase-5a.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5A`, with a
`sourceline_oracle/<name>.txt` fixture each, generated with the driver in
`sourceline_oracle.rs`'s own module comment.

| program | the rule it pins, that no other pins |
|---|---|
| `class_init_activate_inherit_merge.rex` | `INIT` before the `INHERIT` merge and `ACTIVATE` after it |
| `class_constant_expression_later_class.rex` | a constant expression reaching a class declared later, with a control naming an earlier one |
| `class_init_activate_order.rex` | the construction order every pass walks, from a file written leaf first |
| `class_constant_values.rex` | each form of the directive's value, the quoted name beside the symbol one, and the accessor's argument bound |
| `class_constant_uninitialized.rex` | 97.4 from a class-side `INIT`, with a literal constant read at the same moment as its neighbour |
| `class_activate_failure_blames_the_last_installed_class.rex` | the class a failing `ACTIVATE` is blamed against |

`class_activate_failure_blames_the_last_installed_class.rex` joined `collect_stress.rs`'s
`NO_ALLOCATION_PROGRAMS`; the both-directions assertion measured it at zero collections and the
other five out of the list, so nothing there was added by assumption.

## 6. Probes beyond the two discriminators, all matching on both engines

Each run from a fresh directory, three descriptors separate, both sides bounded.

| probe | oracle and crate |
|---|---|
| `INIT` chain across a scrambled `SUBCLASS` chain | `init A` / `init B` / `init C` / `activate A` / `activate B` / `activate C` / `main`, rc 0 |
| a failing class-side `INIT` | rc 214, the method's clause then `2 *-* ::CLASS A` |
| a failing class-side `ACTIVATE`, construction order not source order | rc 214, the method's clause then `15 *-* ::CLASS leaf SUBCLASS middle` |
| `::METHOD init CLASS PRIVATE` | rc 159, 97.2, the `::CLASS` clause alone |
| `::METHOD init CLASS PACKAGE` and `activate CLASS PACKAGE` | rc 0, both bodies run |
| the constant value forms, `hasMethod`, `~class~id` | rc 0, `5` / `some text` / `C3` / `c4` / `-5` / `5` / `String` / `1` |
| `.A~c(1)` | rc 163, 93.902 `0 expected` |
| a literal constant read from `INIT` and from `ACTIVATE` | rc 0, `5` from each |
| an expression constant read from `INIT` | rc 159, 97.4 |
| an expression constant read from `ACTIVATE` | rc 0, `5` |
| `::options digits 12` beside a failing `::CLASS` | rc 158, the `::CLASS` line -- the row that fixes where the source-order gap walk sits |
| a class method on a superclass declared later | rc 0, `from a` through `.c` |
| a `METACLASS` whose own methods follow the class | rc 0, `from M` |
| `::class a` / `::routine r` / `::method m class` | rc 0, the method still attaches to `a` |

## 7. A pre-existing divergence found while writing a comment, not fixed

A **duplicate member name inside one `::CLASS`** is a translation error on the oracle and is not
detected here. Measured, oracle rc 157 in both shapes, echoing the second directive:

```
::CLASS A / ::METHOD m CLASS (twice)      Error 99.902: Duplicate ::METHOD directive instruction.
::CLASS A / ::CONSTANT c / ::METHOD c CLASS   the same 99.902
```

This crate runs both at rc 0 and answers the last member installed. **It predates this task** --
`install_method` overwrote the dictionary key the same way before -- and the second shape is only
newly *expressible*, not newly wrong: `.A~c` answered `method` before this task too, because the
constant installed no accessor at all. Nothing in `rexx-parse` or `install_directives` detects the
duplication. Stated in `install_class_members`'s own doc so the next reader does not assume the
parser owns it, and left for whichever task owns 99.902.

## 8. What a check here could not see

* **The corpus gate cannot see the scope-override refusal getting worse or better.** A refusal the
  oracle does not share is not expressible as a differential row. Its instruments are
  `message_send_scope_override.rex`, which pins the refusal's own bytes, and the in-crate test named
  in section 3. Neither would notice the *reason* for the refusal changing.
* **The corpus gate cannot see the duplicate-member divergence at all**, for the same reason in
  reverse: the crate answers rc 0 where the oracle is rc 157, so a corpus row would fail today. There
  is no instrument for it; if it silently changed shape, nothing here would notice.
* **The uninit deletion measurement can only see what the one test asserts.** Dropping
  `refresh_parent_has_uninit` left it green; had the call mattered on some shape that test does not
  build, the measurement would have looked the same.
* **`add_global` is a linear scan by key**, so one entry per `::CONSTANT` makes install quadratic in
  the number of constants in a file. Nothing in the corpus or the bench axes is near that; no
  measurement was taken and none is claimed.
* **`read_constant`'s `NOMETHOD`-trap arm has no corpus witness**, found by the review and not by me.
  The arm is reachable and right -- measured, `signal on nomethod` around a class-side read of an
  unresolved constant expression is oracle rc 0 `caught NOMETHOD COMPUTED`, matched on both engines
  -- but no corpus program combines `::CONSTANT` with `nomethod`, so nothing pins it and the corpus
  gate would not see it degrade to the plain 97.4.
* **The claim that the `NOMETHOD` condition carries the constant's name rather than the send's
  spelling is not discriminable from Rexx**, also the review's. The accessor is keyed by the upcased
  constant name, so no send that reaches it can spell anything else; `CPPCode.cpp:450` is the only
  evidence and nothing runnable separates the two readings.

## 9. Gates, run by me at `88bab13f1`, tree clean

```
cargo fmt --all --check                                        FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings           CLIPPY_EXIT=0   (cold CARGO_TARGET_DIR, zero warning/error lines)
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   RELEASE_GATE_EXIT=0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   DEBUG_GATE_EXIT=0
```

Both test gates: **98** `test result: ok` blocks, **zero** `FAILED` lines, **zero** `panicked` lines,
and the corpus report reads

```
rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
196 of 196 matching
```

Corpus 190 -> 196.

## 10. The performance sitting, and a control whose referent moved

Two builds against `bench-baselines/pinned/rexx-run-15a1ffa98`, five rounds, six axes, at
`423526a86`. Staleness test run first: `git log --oneline 15a1ffa98..HEAD -- rust/crates
rust/Cargo.toml rust/Cargo.lock Cargo.toml` lists only commits this plan's ledger names, and the
pin's sha256 is still `141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`.

`pinned>head`, `instructions:u`, `size` kept as a column because dropping it turns four cells into
two:

```
axis       arm  small     large
alloc4c    tw   1.003773  1.003676
alloc4c    ir   1.004806  1.004622
arith      tw   0.997137  0.997099
arith      ir   0.989742  0.989572
compound   tw   1.006965  1.006966
compound   ir   1.010472  1.010473
emptyloop  tw   0.995435  0.995434
emptyloop  ir   0.992023  0.992022
strings    tw   1.009178  1.009178
strings    ir   1.013411  1.013411
varlookup  tw   0.997161  0.997161
varlookup  ir   0.994260  0.994260
```

**Four cells at or above 1% once `size` is kept as a discriminator: `compound` `ir` and `strings`
`ir`, at both sizes. None is this change, and the tie-breaker is a comparison rather than an
argument.** Task 17's rows against the same pin,
already in `phase-5a-arms.tsv`, read `compound` `ir` 1.010472 and 1.010473 and `strings` `ir`
1.013411 and 1.013411 -- identical to every digit the file records. Task 14's read the same two cells
at 1.010472/1.010473 and 1.013409/1.013410, so the drift has stood since then. The six axes declare
no class, send no message and evaluate no directive; nothing this task added is on their executed
path.

The sitting was run twice, before and after the commit, and reproduces to five decimals on every
cell. `cycles:u` is in the TSV beside it and is not a result on its own.

**The concern.** `bench-baselines/PINNED.md` records the pin as built under
`rustc 1.97.1 (8bab26f4f 2026-07-14)`. This machine now runs `rustc 1.98.0 (88d9e12ae 2026-08-18)`,
and `rust-toolchain.toml` pins the `stable` channel rather than a version, so the bump arrived
without any commit. Every `head` build since is a different compiler's output from the pin's, and
**the staleness test the plan states is phrased over commits and cannot notice that** -- it is the
same shape as the defect the whole global-constraints document was swept for, on the same artifact,
one axis over. It does not affect the reading above, which rests on a head-to-head with Task 17's
rows rather than on the pin's absolute numbers. It does mean a rebuild of the pin at `15a1ffa98`
will no longer reproduce its recorded sha256, which is the plan's own check for whether a pin is
still the artifact it claims to be. **Rebuilding the pin and restarting the baseline is a plan-level
decision and I did not take it.**

## 11. The `clippy` failure that was already there

`cargo clippy --workspace --all-targets -- -D warnings` fails at BASE `dd83eecdc`:

```
error: useless use of `format!`
    --> crates/rexx-extract/src/docs/classes.rs:1014:9
     = note: `-D clippy::useless-format` implied by `-D warnings`
```

`clippy::useless_format` fires from `rust-1.98.0`'s lint set on a file untouched since BASE --
`git diff --stat dd83eecdc -- crates/rexx-extract/` is empty. Fixed in `f880a4753`, a commit of its
own, by replacing the `format!` wrapper with `.into()` as every neighbouring row in the same array
already does; the string itself is unchanged. Without that, the gate cannot be read at all.

---

# 12. Fix round 1

**Status: DONE.** Head `fe8d51cb0`. Three commits: `98d608364` (the duplicate-member check and three
corrected claims), `d7020ed9f` (its six corpus programs), `fe8d51cb0` (the eight-axis sitting and the
contribution arm). Corpus 196 -> 202.

## 12.1 The required fix: the sitting the plan actually names

The plan's guard command is eight axes; the first sitting ran six, because `global-constraints.md`
carried the stale form. The controller has corrected that file. Re-run at `d7020ed9f`, five rounds,
`pinned>head`, `instructions:u`, `size` kept:

```
axis           arm  small     large
alloc4c        tw   1.003773  1.003676
alloc4c        ir   1.004806  1.004622
arith          tw   0.997138  0.997099
arith          ir   0.989742  0.989572
compound       tw   1.006965  1.006966
compound       ir   1.010472  1.010473
emptyloop      tw   0.995434  0.995434
emptyloop      ir   0.992023  0.992022
strings        tw   1.009178  1.009178
strings        ir   1.013411  1.013411
varlookup      tw   0.997161  0.997161
varlookup      ir   0.994260  0.994260
dispatchclass  tw   1.016293  1.016325
dispatchclass  ir   1.018375  1.018412
rexxcps        tw   1.016600  --
rexxcps        ir   1.020307  --
```

**Ten cells at or above 1% with `size` kept**: `compound` `ir` and `strings` `ir` at both sizes, all
four `dispatchclass` cells, and both `rexxcps` cells. The two loudest axes are the two the six-axis
command omitted, and `dispatchclass` is the send-path axis this change most needed.

**On the six cells the reviewer measured independently, the two sittings agree to five decimals on
four of them and to four decimals on `dispatchclass` `tw` `small` (1.016303 against 1.016293) and
`rexxcps` `tw` `small` (1.016592 against 1.016600)** -- the two arms the paragraph below shows are
bimodal, so four is the honest figure and not a discrepancy owing an explanation. **The two sittings
did not measure the same binary**: the reviewer's head was `4761541c0` and mine is `d7020ed9f`, which
carries the duplicate-member check. Agreement across a code change these axes cannot see is a
stronger result than a reproduction, and that is what it is.

**The contribution arm, which the plan's ruling names as the gate quantity.** Base built from
`dd83eecdc` with `git archive` into a scratch tree and `interpreter/` symlinked in unchanged
(`git diff dd83eecdc -- interpreter/` empty); same eight axes, five rounds, `base>changed`,
`instructions:u`, `size` kept:

```
alloc4c arith compound emptyloop strings varlookup   1.000000 on every arm and both sizes
dispatchclass  tw 0.998019 small / 0.999997 large    ir 0.999987 small / 0.999997 large
rexxcps        tw 1.000004                           ir 0.999981
```

Nothing at or above 1%, and nothing above 0.2%. The absolutes say it without a fit: `dispatchclass`
`tw` `small` is 13,122,125,431 base against 13,096,076,134 changed, per-run ranges overlapping, and
`rexxcps` `ir` is 19,380,638,873 against 19,380,760,087.

**One decomposition in that run is unusable and is named rather than quoted.** `dispatchclass` `tw`
fits `fixed` at 101,395,232 for base and 49,339,046 for changed, and `per_pass` at 6510.365 against
6523.373 -- a 0.2% per-pass gap pointing the opposite way to the `across_builds` cell over the same
data. **That axis arm is bimodal at roughly 0.2% and both builds sample both modes**: the base row
is median 13,122,125,431 with min 13,096,103,237 and max 13,156,140,568 over five rounds, so three
of its five runs sit at or above the median, and the changed row mirrors it at median
13,096,076,134 with max 13,122,006,485. Which mode carries the median is what the 0.998019 cell
reports, and a two-point fit over a bimodal sample is not a decomposition. What carries the flat
reading is the overlapping per-run ranges -- base min 13,096,103,237 against changed max
13,122,006,485 -- and the seven quiet axes, not that split. `base>changed` is changed over base, so
0.998019 says the changed build used *fewer* instructions on the noisy arm.

The accumulated `pinned>head` drift on the two added axes is a position the plan has already ruled
on: over the line at Task 13 for `dispatchclass`, at Task 15 for `rexxcps`, climbing since.

## 12.2 The ruling: duplicate member names

**It was contained, so I built it.** The change lives in `install_directives`' first walk,
`install_method`, `install_attribute` and `error.rs` -- all code this task already touched -- and the
corpus was green before the new rows were added.

`Interp::check_member_keys` is `LanguageParser::checkDuplicateMethod`
(`parser/DirectiveParser.cpp:507`-`:530`). Every placement was measured rather than reasoned:

| program | oracle and crate |
|---|---|
| `::method m` twice, then `::class B subclass zzznotaclass` | rc 157, 99.902 -- translation before install |
| `::method m` then `::method m external "LIBRARY nosuchlib nosuchfn"` | rc 157, 99.902 -- the duplicate beats the `EXTERNAL` |
| `::constant c 5` then `::constant c (1+2)`, no `::CLASS` | rc 157, 99.932 -- the duplicate check runs ahead of 99.906 |
| `::constant sep (1+2)` twice, no `::CLASS` | rc 157, 99.906 on the **first** -- nothing has duplicated yet |

**The reverse of that pair is not evidence for the placement and is not offered as any.** With the
`EXTERNAL` first the oracle is 98.903 at rc 158 and this crate is
`rexx-exec: ::METHOD EXTERNAL is not implemented (Phase 7)` at rc 120 on both engines -- a Phase 7
gap `directive_gap` takes whichever check the walk reached first, so neither number bears on where
the duplicate check sits. The placement rests on the forward direction, which matches byte for byte.
The first version of this row put the reverse direction in a column headed `oracle and crate` and
implied the crate answers 98.903; it does not, and never has for any `EXTERNAL` directive.

**The keys a directive claims are now one enumeration.** `method_dictionary_keys` and
`attribute_dictionary_keys` came out of `install_method` and `install_attribute` unchanged; the check
and the install both read them, exactly as `methodDirective` calls `checkDuplicateMethod` once per
name it is about to add. That is what makes `::METHOD "p="` beside `::ATTRIBUTE p` the attribute's
99.931: the setter key nothing in the file spells is what collides.

**The `CLASS`-keyword-with-no-`::CLASS` refusal came with it**, slightly beyond the letter of the
ruling and for a stated reason: it is the same C++ function's other arm, and leaving it out would
mean inventing a rule for a loose class-side member that the oracle never reaches. Measured, both
rc 157: `::METHOD m CLASS` and `::ATTRIBUTE p CLASS` alone in a file each report `Error 99.905: CLASS
keyword on ::METHOD directive requires a matching ::CLASS directive.`

**Fourteen shapes measured against the oracle on both engines, all byte-identical**, including the
two that must stay rc 0: `::METHOD m` beside `::METHOD m CLASS` in one class, and `::ATTRIBUTE p GET`
beside `::ATTRIBUTE p SET`.

**Six corpus programs, each shown to add coverage rather than merely to be able to fail.** Each
mutation applied to the committed tree, the corpus gate read, the tree restored from a byte-identical
copy (`sha256` of `lib.rs` `5297769b8247c2bfc942145e30c560e56cfa86556820b2c15f6589064c324b35` before
and after every one):

| mutation | result | what reddened |
|---|---|---|
| the check removed entirely | 197 of 202 | the five refusal programs |
| every duplicate reports 902 | 200 of 202 | the `::ATTRIBUTE` and `::CONSTANT` rows alone |
| a member's keys forget which side | 200 of 202 | the negative control and the 99.905 row |
| the `CLASS`-with-no-class arm removed | 201 of 202 | the 99.905 row alone |
| a `::CONSTANT` claims only one side | 201 of 202 | the constant-and-method row alone |

**No program that predates this round reddens under any of them.** A crude sixth mutation -- the key
ignoring the side for every directive kind -- reddens eight earlier constant programs as well,
because a `::CONSTANT` claims both dictionaries and then collides with itself; the narrower
third row is the one that isolates the negative control.

The five refusals joined `NO_ALLOCATION_PROGRAMS`, measured by the both-directions assertion; the
negative control is absent from it because it runs and allocates.

## 12.3 The three corrected claims

* `record_literal_constants` cited `DirectiveParser.cpp:1875` for a literal constant's value with
  `::constant c 5` as its example, and `:1875` is the value-**omitted** arm. `/bin/grep -n
  'value = token->value();'` puts the literal arm at `:1911`; both are now cited where they belong,
  with the `isEndOfClause` test at `:1873` that chooses between them.
* `install_constant` said `createConstantGetterMethod` calls `setUnguarded` "and nothing else". It
  also calls `setConstant` at `:2525`. The substantive claim now rests on
  `MethodClass::isSpecial()` reading neither (`classes/MethodClass.hpp:118`, checked).
* `ClassGraph::refresh_parent_has_uninit`'s doc justified the function by a caller that does not
  exist. It now says what is true, including the reviewer's argument that the redundancy holds under
  every input rather than only the corpus's, that nothing witnesses the call, and why it is kept
  anyway.

## 12.4 What this round could not see

* **The duplicate-member check has no instrument outside the corpus rows above.** Every shape it
  refuses is one the oracle also refuses, so each is expressible as a differential row and the six
  are it. What no instrument covers is a *new* member-directive form later gaining a dictionary key
  that `member_dictionary_keys` does not list: the check and the install would both miss it together,
  silently and consistently, which is the cost of sharing one enumeration.
* **The contribution arm cannot see a cost that is not on a bench axis.** The check runs once per
  directive at install; none of the eight axes declares more than a handful of directives, so a
  per-directive cost would have to be enormous to show. `add_global`'s linear scan, section 8's last
  bullet, is in the same blind spot.
* **The base build is a `git archive` of `dd83eecdc`, not that commit's own `target/`.** If a build
  script read something outside the archive and `interpreter/`, the base binary would differ from
  what `dd83eecdc` produced in place. Checked: `rexx-classes` and `rexx-inventory` are the only
  crates with a `build.rs`, and every `read_to_string` in them takes a path under
  `../../../interpreter/`, which the symlink resolves to the unchanged tree.

## 12.5 Carried forward, not this round's

The re-review found a duplicate `::CLASS` directive to be the same shape as the divergence this
round closed, one subcode below it: oracle rc 157 `Error 99.901: Duplicate ::CLASS directive
instruction.`, this crate rc 0 on both engines. **Pre-existing** -- `install_directives`' `declared`
map has always taken first-wins with no refusal, and the reviewer's `dd83eecdc` build answers rc 0
too.

**Ruled on and closed in fix round 3, along with `::RESOURCE`'s 99.942** -- see section 13.

## 12.6 Fix round 2

Three sentences, all measured false by the re-review, none of them behaviour. `14479a7fa` carries
the source one; the other two were report-only and are corrected in place above.

* **`check_member_keys`' doc claimed the reversed duplicate/`EXTERNAL` pair is 98.903 at rc 158**,
  unlabelled in a paragraph whose subject is this crate's walk. Re-measured: oracle 98.903 at rc 158,
  this crate `rexx-exec: ::METHOD EXTERNAL is not implemented (Phase 7)` at rc 120 on both engines.
  The clause was also offered as the *reason* for the ordering, and in that direction it is evidence
  for nothing -- `directive_gap` refuses the file whichever check the walk reaches first. The doc now
  rests the placement on the forward direction and states the reverse as the divergence it is.
  Section 12.2's table carried the same row under an `oracle and crate` heading and is corrected.
* **The `dispatchclass` `tw` sample was described as one high run among four low ones.** The TSV row
  is median 13,122,125,431, min 13,096,103,237, max 13,156,140,568 over five rounds, so three of the
  five sit at or above the median; the `changed` row mirrors it. The arm is bimodal at roughly 0.2%
  and both builds sample both modes. Corrected in 12.1, along with the direction of the ratio:
  `base>changed` is changed over base, so 0.998019 is the changed build using *fewer* instructions.
* **"Reproduce the reviewer's to five decimals on every one" was four on two cells** -- the same two
  bimodal arms -- and the two sittings measured different binaries (`4761541c0` against
  `d7020ed9f`). Both stated in 12.1.

## 12.7 Gates, run by me on the tree that became `14479a7fa`, tree clean

```
cargo fmt --all --check                                              FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                CLIPPY_EXIT=0   (cold CARGO_TARGET_DIR, zero lines starting `warning` or `error`, and the log carries the whole workspace's own `Checking` lines)
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   RELEASE_GATE_EXIT=0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   DEBUG_GATE_EXIT=0
```

Both test gates: **98** `test result: ok` blocks, **zero** `FAILED`, **zero** `panicked`, and

```
rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
202 of 202 matching
```

---

# 13. Fix round 3: the duplicate-directive family

**Status: DONE.** Head `9a0f249e9`. Three commits: `ac2e92bf0` (the two checks),
`0baa2ccae` (their witnesses), `9a0f249e9` (the sitting). Corpus 202 -> 204.

## 13.1 Where the family stood, measured before anything was built

Every row oracle against `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, three descriptors compared
separately, fresh directory per probe.

| code | shape | oracle | this crate, before |
|---|---|---|---|
| 99.901 | `::CLASS A` twice | rc 157 | **rc 0, answering the second class** |
| 99.902 | `::METHOD m CLASS` twice in one class | rc 157 | matches (fix round 1) |
| 99.903 | `::ROUTINE r` twice | rc 157 | matches |
| 99.931 | `::ATTRIBUTE p` twice in one class | rc 157 | matches (fix round 1) |
| 99.932 | `::CONSTANT c` twice in one class | rc 157 | matches (fix round 1) |
| 99.942 | `::RESOURCE d` twice | rc 157 | **rc 0, keeping both bodies** |

Two silent wrong answers left, both the same shape as the four that already work: a table keyed by
the upcased name, checked in the walk that answers the rest. **Contained, so both were built.**
Nothing needed machinery beyond that walk, so nothing was parked.

## 13.2 What the checks are

`isDuplicateClass` compares `commonString(name->upper())` (`parser/DirectiveParser.cpp:347`, `:349`)
and `resourceDirective` compares the same (`:2277`, `:2316`). Each kind keeps its own table --
`classDependencies`, `resources`, `unattachedMethods` and the per-class dictionaries -- so a name
shared across kinds is not a collision.

**Twenty-three shapes measured against the oracle on both engines, all byte-identical.**

*Nine refusals:* `::CLASS A` twice; `::CLASS a` beside `::CLASS A`; `::CLASS a` beside
`::CLASS "A"`; `::CLASS "a"` twice; a non-adjacent pair; `::CLASS A SUBCLASS Object` beside a bare
`::CLASS A`; `::RESOURCE d` twice; `::RESOURCE d` beside `::RESOURCE D`; `::RESOURCE "d"` beside
`::RESOURCE d`.

*Eight that must stay rc 0 and do*, probed as hard as the refusals: `::CLASS A` beside `::CLASS B`;
`::CLASS r` beside `::ROUTINE r`; `::RESOURCE d` beside `::ROUTINE d`; `::RESOURCE d` beside
`::CLASS d`; `::RESOURCE d` beside `::RESOURCE e`; one member name in two different classes; one
`::CONSTANT` name in two different classes; and `::ROUTINE r` with `::RESOURCE r` and `::CLASS r`
in one file.

*Six orderings*, each the source order the single walk gives: a duplicate `::ROUTINE` pair above a
duplicate `::CLASS` pair is 99.903 and the blocks swapped is 99.901; a duplicate `::CLASS` pair above
a duplicate `::METHOD` pair is 99.901 and swapped is 99.902; and either duplicate pair above a
`::CLASS` naming an unresolvable superclass is the duplicate, not the class error.

**The controller's caution, answered by running.** With 99.901 in place a second `::CLASS` of one
name cannot reach the `declared` map, so its `or_insert` first-wins can only ever insert; the whole
gated suite is green either way, which is what says nothing in the tree depended on which duplicate
won. The comment on that map now says so instead of describing a tie-break that cannot happen.

## 13.3 The witnesses, and the two that did not work first time

`::RESOURCE` gets no corpus row and **cannot**, measured rather than inherited: a `::RESOURCE`
program placed in `corpus/lang/` fails `every_corpus_program_tiles` byte by byte -- `byte 'o' at
offset 45 sits after the last clause span and belongs to no node`. So `run/tests.rs`'s
`a_duplicate_resource_name_is_refused_and_a_distinct_one_is_not` is its **sole instrument**, and it
carries the refusal and the distinct-name pair together.

**Both corpus programs were wrong on the first attempt, and the mutations are what said so.** Each
mutation applied to the committed tree, corpus gate read, tree restored from a byte-identical copy
(`sha256` of `lib.rs` `b256ab483217cbd3bb1011eecbb4f65ed4ce8f1762885ceddcbb4c6cf7202f31` before and
after every one):

* The `::CLASS` row first paired `::CLASS a` with `::CLASS "A"`. That does not discriminate: the
  tokenizer upcases a symbol, so **both directives store `A`** and a key that is the stored name
  still collides. Keying on the stored name left the suite at **204 of 204** -- a witness that could
  not fail. The pair is now `::CLASS "a"` and `::CLASS A`, stored as `a` and `A`.
* The control's comment said `R` named a class and a routine; the routine was named `f`. Merging the
  class and routine tables left the suite at **204 of 204**. Both are named `R` now.

The four mutations as they stand, with no pre-existing program reddening under any:

| mutation | result | what reddened |
|---|---|---|
| the `::CLASS` check removed | 203 of 204 | `class_duplicate_class.rex` alone |
| its key is the stored name | 203 of 204 | `class_duplicate_class.rex` alone |
| one table for classes and routines | 203 of 204 | `class_directive_names_are_their_own_table.rex` alone |
| the `::RESOURCE` check removed | **204 of 204** | nothing in the corpus; the in-crate test fails |

The last row is why "sole instrument" was worth writing down rather than assuming.

## 13.4 What this round could not see

* **Nothing in the corpus can see the `::RESOURCE` refusal**, by construction. If that check
  regressed and the in-crate test were deleted with it, every gate would stay green.
* **The duplicate checks are keyed by name and nothing checks the key against the C++ mechanically.**
  The four tables are separate here because they are separate there, read and cited; a future
  directive kind sharing a table with an existing one would be a divergence no test in this tree
  could find.
* **The rc-0 neighbours are probes, not rows.** Six of the eight have no corpus witness; only the
  class/routine split and the per-class member names do. A check that started refusing
  `::RESOURCE d` beside `::CLASS d` would pass every gate.

## 13.5 The sitting

Eight axes, the plan's list, and I checked `global-constraints.md`'s copy against the plan's before
running -- they now agree. `pinned>head`, `instructions:u`, `size` kept, five rounds at `0baa2ccae`:

```
axis           arm  small     large
alloc4c        tw   1.003773  1.003676
alloc4c        ir   1.004807  1.004622
arith          tw   0.997138  0.997099
arith          ir   0.989742  0.989572
compound       tw   1.006965  1.006966
compound       ir   1.010472  1.010473
emptyloop      tw   0.995435  0.995434
emptyloop      ir   0.992023  0.992022
strings        tw   1.009178  1.009178
strings        ir   1.013411  1.013411
varlookup      tw   0.997161  0.997161
varlookup      ir   0.994260  0.994260
dispatchclass  tw   1.016290  1.016326
dispatchclass  ir   1.018380  1.018408
rexxcps        tw   1.016583  --
rexxcps        ir   1.020294  --
```

Ten cells at or above 1% with `size` kept, the same ten as fix round 1's, and the accumulated
position the plan has ruled on. The contribution arm, base from `dd83eecdc` so it spans the whole of
Task 18 including both duplicate rounds, `base>changed`, `instructions:u`:

```
alloc4c arith compound emptyloop strings varlookup   1.000000, every arm, both sizes
dispatchclass  tw 1.000002 small / 0.999993 large    ir 0.999994 small / 0.999995 large
rexxcps        tw 0.999987                           ir 1.000017
```

Nothing above 0.002%. **This settles fix round 1's noisy cell**: that sitting read `dispatchclass`
`tw` `small` at 0.998019 with a `per_pass`/`fixed` fit disagreeing with its own `across_builds`
figure, and the same arm here is 1.000002 over a build carrying strictly more code. That is what a
bimodal sample looks like when the modes fall the other way, and not what a 0.2% effect looks like.

## 13.6 Gates, run by me at `9a0f249e9`, tree clean

```
cargo fmt --all --check                                              FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                CLIPPY_EXIT=0   (cold CARGO_TARGET_DIR, zero lines starting `warning` or `error`)
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   RELEASE_GATE_EXIT=0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   DEBUG_GATE_EXIT=0
```

Both test gates: **98** `test result: ok` blocks, **zero** `FAILED`, **zero** `panicked`, and

```
rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
204 of 204 matching
```
