# Task 5 — `Object`'s three, and `Class`'s eleven

Status: **implemented.** BASE `8bc2df809`, taken as the HEAD found on release rather than rebased.

Dispatch was held at `5d833ec6d` while Task 4 still had `dispatch.rs` modified and two `cargo` runs
live in the working tree; that wait is what moved BASE forward twice, to `552d108b0` and then
`8bc2df809`.

## Pre-flight findings against the brief

### 1. The brief's central difficulty does not exist, and the arm must not be deleted

The brief states the six operators are loud at two sites needing two separate fixes, and that
`Interp::operator_operand_gap`'s class arm (`eval.rs:1819`) has to change. Both are wrong, and the
correction makes the change smaller.

`Interp::apply_binary` (`eval.rs:1882`) is "the one dispatch both engines enter" for every binary
operator except arithmetic, and at `:1901` it asks `Interp::operator_message_receiver` **before**
`compare_values` or `logical_values` is reached. The prefix path (`:1004`) and the arithmetic
operand path (`:1273`) each ask it one line before their own gap check. So every operator path
already sends the operator as a message whenever the receiver admits one; `operator_operand_gap` is
only the fallback for a receiver that does not.

`operator_message_receiver` returns `None` for a class handle at `eval.rs:1751`-`:1753`. That single
early-out is the whole reason the six rows are loud. `.nil`, two lines above it, returns `Some` and
gets exactly the semantics measured for a class object: identity for the six comparisons, 97.1 for
every other operator.

### 2. Deleting the gap's class arm would have caused three regressions the brief does not mention

The brief says the gap is asked at `:1007`, `:1276`, `:1575`, `:1692`. That is true of the operator
paths, but the same function is also asked at three **non-operator** sites that never ask
`operator_message_receiver`:

* `run.rs:8674` — `Interp::header_number`, a controlled `DO` header's `initial`/`TO`/`BY`
* `run.rs:10379` — `Interp::controlled_step_wide`, the control variable
* `run.rs:5417` — a `RAISE ADDITIONAL` value

Deleting the class arm would have dropped all three out of their loud refusal into a conversion,
answering 41.1 or worse — a loud gap turning into a silently wrong answer, which is the failure the
brief warns about for `>`/`+`/`**`. Leaving the arm untouched keeps all three as they are, and
`an_object_in_a_do_headers_numeric_position_is_loud` stays green without modification.

### 3. `Subclasses` is declared with `AddProtectedMethod`, and the module doc understates the parser

`Setup.cpp:475` is `AddProtectedMethod("Subclasses", RexxClass::getSubClasses, 0)`, not `AddMethod`.
`native_classes.rs`'s module doc enumerates the derived operations as
`AddMethod`/`AddClassMethod`/`InheritInstanceMethods`/`RemoveMethod`/`HideMethod`, which does not
include it. The parser is fine — `build.rs:199`-`:201` folds `AddProtectedMethod` into the
`AddMethod` arm — so `Subclasses` **is** in the derived `Class` definition and the registration
control will accept the row. Only the prose is incomplete.

### 4. `~methods` and `~instanceMethods` take a scope argument the brief does not cover

Measured on the oracle 2026-09-08, on a `::class KK` with two methods:

```
.KK~methods                      34      .KK~new~instanceMethods         34
.KK~methods(.KK)                  2      .KK~new~instanceMethods(.KK)     2
.KK~methods(.Object)             32      .KK~new~instanceMethods(.nil)    0
.KK~methods(.nil)                 2
```

Both funnel into one C++ helper, `behaviour->getMethods(class_object)`
(`ClassClass.cpp:1027`, `ObjectClass.cpp:358`): omitted means the whole resolved set, a class means
that scope alone, and `.nil` means the object-level methods only. The two differ solely because
`RexxClass::methods` maps `.nil` to `this` before the call (`ClassClass.cpp:1022`-`:1025`), which is
why `methods(.nil)` is `2` where `instanceMethods(.nil)` is `0`.

### 5. `Class~enhanced` needs a field outside this task's permitted files

`RexxObject::defaultName` (`ObjectClass.cpp:1763`-`:1767`) reads:

```cpp
RexxString *defaultname = behaviour->getOwningClass()->getId();
if (behaviour->isEnhanced()) return defaultname->concatToCstring("enhanced ");
```

It returns `enhanced K` with **no article**, returning before the `a`/`an` switch, and
`RexxClass::enhanced` sets the flag with `enhanced_object->behaviour->setEnhanced()`
(`ClassClass.cpp:1478`) *after* `setOwningClass(this)` has already restored the class to `K`. The
flag is therefore the only carrier of the answer; the class id cannot produce it.

`ObjectMethods` (`rexx-core/src/body.rs:465`) already carries an `enhanced` list, but it is **not** a
faithful proxy: measured, `k~enhanced(.StringTable~new)` with an empty table still answers
`enhanced K` for both `~string` and `~defaultName`, while that list would be empty. A faithful fix
needs a real boolean on `Body::Instance`, and `rust/crates/rexx-core/src/body.rs` is not in this
task's permitted set. Ruling requested.

Setting `Body::Instance.name` instead would answer `~objectName` and `~string` correctly and leave
`~defaultName` at `a K` — a new silent wrong answer of exactly the kind this task exists to remove,
so it will not be shipped.

Ruled: `body.rs` is on this task's list, with the flag to go on `ObjectMethods` if it fits there.
`ObjectMethods` is `#[derive(Clone, Debug, Default)]` over two `Vec`s and is reached through
`own: Option<Box<ObjectMethods>>`, so a `bool` on it is behind a `Box` and cannot move
`body.rs:665`'s `assert!(size_of::<Body>() <= 80)`. **The empty-table case is the wrinkle**: with no
enhancing methods `own` may never be created, so the flag has to materialise `ObjectMethods` rather
than assume it exists — the same case that ruled out deriving the flag from the list being
non-empty.

### 6. One brief figure corrected, harmlessly

`.Object~subclasses~items` is **51** on a bare program, not the brief's 53. `subclass()` registers
in the parent's list, measured 51 → 52 → 53 across two calls, so the brief's figure was taken after
its own two subclasses existed. No witness may assert this number in any case.

## Design

One mechanism closes both of the brief's two sites, because they become the same site:

* delete the `is_class_slot` early-out in `operator_message_receiver` (`eval.rs:1751`-`:1753`)
* add `=`, `==`, `\=`, `\==`, `<>`, `><` to `NATIVE_METHODS` under `"Class"`, bound to the existing
  `native_object_identical` / `native_object_different` (`dispatch.rs:4861`, `:4874`), which are
  `receiver == other` on `ObjRef` and need no change for a class handle
* leave `operator_operand_gap` entirely alone

The table is `NATIVE_METHODS` and not `NATIVE_CLASS_METHODS`: the doc at `dispatch.rs:955`-`:962`
records that `Setup.cpp`'s `AddMethod` rows on `.Class` belong in the former, which is where
`("Class", "ID", ...)` and `("Class", "SUBCLASS", ...)` already sit.

The expression spelling `(.Array = .Array)` and the message spelling `.Array~'='(.Array)` then
resolve to the same method id, so a fix at one cannot leave the other loud. Both are asserted in the
witness.

### The arithmetic operators are in scope, and they are fixed for free

Measured on the oracle 2026-09-08, three descriptors, one program each:

```
.array > .array    rc 159  97.1 Object method not found
.array + 1         rc 159  97.1        .array ** 1   rc 159  97.1
.array & 1         rc 159  97.1        -.array       rc 159  97.1
.array < .string   rc 159  97.1        \.array       rc 159  97.1
.array || 'x'      rc 0    The Array classx
```

`Setup.cpp`'s `Class` block declares `New Enhanced ID Inherit Method Methods MixinClass
QueryMixinClass IsMetaClass IsAbstract Subclass IsSubclassOf DefaultName Package Copy = == \= <> ><
\== HashCode Activate Annotations Annotation`, and its `Object` block declares `= == \= <> ><
\== || Copy Class HasMethod ... IsInstanceOf isNil IsA InstanceMethod InstanceMethods IdentityHash`.
Neither holds `+`, `>`, `**` or `&`, so once a class is an operator-message receiver those sends
miss every behaviour and reach the same 97.1 the test at `dispatch.rs:11676` already pins for
`Object "The K class" does not understand message`. They move from loud rc 120 to 97.1 rc 159 and
match the oracle.

**This is derived from an enumeration outside the repository and still has to be run before it is
claimed.** It is recorded here as a prediction, not a measurement.

### Argument handling, measured before implementing

Every witness carries a short argument list as well as a good one, per the global constraint.
Measured on the oracle 2026-09-08, one program each, three descriptors:

```
.Object~new~isInstanceOf            rc 168  88.901 Missing argument; argument class is required.
.Object~new~isInstanceOf(1)         rc 168  88.914 Argument class must be an instance of the Class class.
.Object~new~isInstanceOf(.Object,2) rc 163  93.902 Too many arguments in invocation of method; 0 expected.
.Object~new~instanceMethod          rc 163  93.903 Missing argument in method; argument 1 is required.
.Array~'='()                        rc 163  93.903
.Array~isAbstract(1)                rc 163  93.902
.Array~subclasses(1)                rc 163  93.902
.Array~queryMixinClass(1)           rc 163  93.902
.Array~isMetaclass(1)               rc 163  93.902
```

**`isInstanceOf` is the only row that type-checks its argument.** `methods` and `instanceMethods` do
not: `.Array~methods(1)` and `.Object~new~instanceMethods(1)` each answer a `Supplier` at rc 0, and
each supplier is **empty**. The argument is matched against method scopes rather than validated, so
a body that raised on a non-class would diverge at rc 0.

## The two predictions, run before a line was written

Both were held as predictions through pre-flight and settled **at BASE, before any edit**, because
the message-send spelling already takes the send path the change routes operators into. Predictions
were written down first, then run; both engines and the oracle, three descriptors.

| probe | predicted | measured at BASE |
| --- | --- | --- |
| `.Array~'>'(.Array)` | 97.1 rc 159, a `Miss` | **confirmed** — rc 159, `97.1 Object "The Array class" does not understand message ">"`, both engines |
| `.Array~'+'(1)` | 97.1 rc 159 | **confirmed** — rc 159, same shape |
| `.Array~'||'('x')` | `The Array classx` rc 0 | **confirmed** — rc 0, both engines |
| `.Array~'='(.Array)` | loud rc 120 naming `"=" of class "Class"` | **confirmed** — `rexx-exec: method "=" of class "Class" is not implemented (Phase 5)` |

The first two are what make the arithmetic ruling safe: `>` and `+` are names neither `Setup.cpp`
block declares, so the send misses every behaviour and answers the oracle's own 97.1 rather than a
refusal of this crate's. The third is what makes the receiver change safe for concatenation.

## The six operators

`operator_message_receiver` (`eval.rs:1751`) now answers `Some(value)` for a class handle instead of
`None`, and `NATIVE_METHODS` binds `=`, `==`, `\=`, `\==`, `<>` and `><` under `"Class"` to the
existing `native_object_identical` / `native_object_different`. **`operator_operand_gap` is
unchanged**, which is what leaves `header_number`, `controlled_step_wide` and `RAISE ADDITIONAL`
exactly as they were.

Measured after the change, both engines against the oracle, one program each, three descriptors:
**24 rows, 24 agree, 0 differ** — the six comparisons in both the expression and the message
spelling, the identity-not-rendering rows, `>` `<` `+` `**` `&` `-` `\` at 97.1 rc 159, and `||`,
abuttal and blank still answering `The Array classx`.

## The five readers

`isAbstract`, `isMetaclass` and `queryMixinClass` read `ClassRegistry`'s own predicates;
`queryMixinClass` needed one that did not exist and `ClassGraph::is_mixin` is it, over the
`ClassKind` the graph already stores. `subclasses` reads `ClassRegistry::subclasses`, the reverse
edge `subclass` and `inherit` already maintain — measured, a fresh subclass takes its parent's list
from 0 to 1 and the child is found in it by `~id`.

**What state each reads**, per the phase's constant-answer constraint: all five read the class graph
this crate builds as directives and `~subclass`/`~inherit` calls execute. None answers from a
constant.

`~methods` answers the **whole resolved set**. Measured against the oracle on a `::class KK` with two
methods of its own: 34 with no argument, 2 at scope `.KK`, 32 at scope `.Object`, 2 at `.nil`, and
**0 for a non-class scope** — the argument is matched, never validated, so `.KK~methods(1)` is an
empty `Supplier` at rc 0 and not a raise. A body that validated would have diverged silently.

## `Object`'s three

`isInstanceOf` is the only row that type-checks its argument, and it uses the existing
`class_argument` helper for exactly the pair the oracle raises: `88.901` with no argument and
`88.914` for a non-class, both rc 168. `instanceMethod` answers `.nil` for a name the receiver does
not have rather than raising, and its missing-argument error is `93.903` at rc 163 where
`Class~method`'s is `88.901` at rc 168 — an asymmetry measured rather than assumed.

`instanceMethods` and `Class~methods` share one body, because the C++ does
(`behaviour->getMethods`). They differ only in the `.nil` scope: `RexxClass::methods` maps it to the
receiving class, so `.KK~methods(.nil)` is 2 where `.KK~new~instanceMethods(.nil)` is 0.

## `Class~enhanced`

The rendering came from `RexxObject::defaultName` (`classes/ObjectClass.cpp:1763`-`:1767`), which
returns `<id>` prefixed with `enhanced ` and **no article**, ahead of the `a`/`an` choice, when
`behaviour->isEnhanced()`. `RexxClass::enhanced` sets that flag at `ClassClass.cpp:1478`, after
`setOwningClass(this)` has already restored the class, so the class id cannot produce the answer and
a flag is the only carrier.

The flag is a `bool` on `ObjectMethods` (`rexx-core/src/body.rs`), which is reached through
`own: Option<Box<ObjectMethods>>` and so is behind a `Box`. **`body.rs`'s
`const _: () = assert!(size_of::<Body>() <= 80)` is unmoved and still passes** — the field widens the
boxed `ObjectMethods`, not `Body`.

**Why the flag is real and not derived.** `ObjectMethods` already carries an `enhanced` list, and
"non-empty" looked like it would serve. It does not: measured, oracle rc 0,
`k~enhanced(.StringTable~new)` over an **empty** table installs no method and still answers
`enhanced K` for `~string` and `~defaultName` alike. The same case is why `mark_enhanced_instance`
materialises `ObjectMethods` rather than assuming the walk created it — with an empty table
`install_enhancing_object_methods` loops zero times and `own` stays `None`, so a flag written only
inside the walk would have been a silent no-op.

## Witnesses

`rust/corpus/lang/class_introspection.rex`, with
`rust/crates/rexx-parse/tests/sourceline_oracle/class_introspection.txt` (`count 207`, generated with
the sanctioned `.Package~new` driver from `sourceline_oracle.rs`'s module comment). Measured: oracle,
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` produce **byte-identical stdout at rc 0 with empty
stderr**.

`rust/crates/rexx-exec/src/dispatch/introspection.rs` is added to `dispatch_seam.rs`'s
`CLEARANCE_CONSUMERS`, which the global constraint requires for a new file naming the seam token.

**The supplier's order is asserted nowhere.** Every supplier row walks the whole supplier, reports
its `~class~id` and its length, and finds named entries by comparing `~index` — never printing one
entry before another. The witness says so in its own header and gives the reason: the oracle's order
is hash order, this crate's is `BTreeSet` order, and nothing makes two interpreters agree. This is
the "assert membership and count" of the two options, not the "sort" one.

**A count is used where it is a property of one class and avoided where it is a property of the
image.** `32` and `34` are `Object`'s and `KK`'s resolved method counts and move only if the class
set does. `.Object~subclasses~items` is not asserted at all — section E builds its own `BASE` class,
takes the list from 0 to 1, and asks whether `KID` is in it by `~id`.

**Every object-answering row is read through a second send**, per the ruling: `~methods` and
`~instanceMethods` through `~class~id` plus a walk, `~subclasses` through `~class~id` and `~items`
and an element's `~id`, and `instanceMethod` through `~class~id`, `~scope~id` and `~scope~class~id`.

**The short argument list sits beside the good one**: section J runs sixteen refusals under
`SIGNAL ON SYNTAX` so each is a row of output rather than the end of the program, covering every
reader's arity, `isInstanceOf`'s non-class argument, and every operator a class does not answer.

### The `eval.rs` test, which was a named deliverable

`an_operator_sent_to_an_object_is_loud` loses its `.array` rows and gains a comment saying why their
absence is the property rather than an omission. **Every non-class row in it is untouched** — the
array, `.environment`, `.methods`/`.routines` and package rows — and that is the control: a change
that had widened past class handles would have moved one of them.

The replacement is `a_class_objects_operators_are_sent_as_messages`, which asserts the six as values
in **both** spellings, the other operators at 97.1 rc 159, and concatenation unchanged.

`an_object_in_a_do_headers_numeric_position_is_loud` is unmodified and still green, which is the
second control.

### A neighbouring test the change falsified, and the oracle settled it

`an_object_reached_through_a_stem_default_is_loud` failed. Its own doc recorded that the oracle
answers `0` for `a. = .array; say (a. == 'The Array class')` and that the test had settled for a
loud refusal as the safe non-wrong answer. Measured after the change, all three of its operator rows
now match the oracle exactly — `0` at rc 0, and 97.1 at rc 159 for `a. + 1` and `a.zz + 1` — because
the stem-default and `>varref` redirects reach the send. The test is rewritten as
`an_object_reached_through_a_stem_default_answers_as_the_object` and asserts the agreement.

Its fourth row, `do i = 1 to a.`, still refuses at rc 120 where the oracle answers 97.1, and it is
kept as that test's explicit control with a comment naming the site. See Concerns.

## Negative controls

Three, each predicted before it was run, each run against a rebuilt binary, and each reverted from a
`cp` copy whose `sha256sum` was compared afterwards — never `git checkout --`. All three files came
back byte-identical and with zero mutation residue.

**Control A — the 5f/5g registration control.** Predicted: a row naming a method the class does not
answer compiles and then panics at `ObjectModel::build`. **Confirmed** — adding
`("Class", "NOSUCHMETHODNAME", ...)` built at rc 0 and every run died at rc 101,
`dispatch.rs:1303`, `NATIVE_METHODS names Class~NOSUCHMETHODNAME, which that class's behaviour does
not answer`.

**Control B — remove the bindings.** Predicted: both operator spellings go loud again and
`~subclasses` goes loud again. **Confirmed** — with the six `"Class"` operator rows and the
`SUBCLASSES` row deleted, `(.Array = .Array)` and `.Array~'='(.Array)` both returned to rc 120
`method "=" of class "Class" is not implemented`, and `.Array~subclasses` to rc 120 `method
"SUBCLASSES" of class "Class"`. `.Array~isAbstract` still answered `0`, which is the control on the
control: the removal was targeted and did not disable the module.

**It also separated the two halves of the change**, which was not predicted and is worth recording:
with the bindings gone, `(.Array > .Array)` was **still** 97.1 at rc 159. The 97.1 comes from the
receiver change, not from the bindings.

**Control C — revert only the receiver change**, leaving the bindings in place. Predicted: the
expression spelling goes loud with the *gap* message while the message spelling still answers.
**Confirmed exactly** — `(.Array = .Array)` and `(.Array > .Array)` both returned to rc 120
``rexx-exec: the operator `=` applied to a class object is not implemented``, while
`.Array~'='(.Array)` still answered `1` at rc 0 and `.Array~subclasses~items` still answered `0`.

**Controls B and C together are the empirical proof of the brief's central claim.** The two sites are
distinct and both had to be closed: B shows the bindings alone give the *values*, C shows the
receiver change alone gives the *97.1* and that a fix at only the binding site leaves every
expression-spelled operator refusing. Each half is independently load-bearing, and neither is
redundant.

## Shared artifacts, before and after

Four moved, and all four are committed with this change.

### `corpus/method-bodies.txt`

Refreshed with:

```
REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies
```

**Exactly fourteen rows moved, all of them this task's, every one `loud` -> `answers`.** Nothing
else in the file changed.

| row | before | after |
| --- | --- | --- |
| `Class = / == / <> / >< / \= / \==` (instance) | `loud  method "<op>" of class "Class"` | `answers  rc 163` |
| `Class isAbstract` | `loud  method "ISABSTRACT" of class "Class"` | `answers  rc 0` |
| `Class isMetaclass` | `loud  method "ISMETACLASS" of class "Class"` | `answers  rc 0` |
| `Class methods` | `loud  method "METHODS" of class "Class"` | `answers  rc 0` |
| `Class queryMixinClass` | `loud  method "QUERYMIXINCLASS" of class "Class"` | `answers  rc 0` |
| `Class subclasses` | `loud  method "SUBCLASSES" of class "Class"` | `answers  rc 0` |
| `Object instanceMethod` | `loud  method "INSTANCEMETHOD" of class "Object"` | `answers  rc 163` |
| `Object instanceMethods` | `loud  method "INSTANCEMETHODS" of class "Object"` | `answers  rc 0` |
| `Object isInstanceOf` | `loud  method "ISINSTANCEOF" of class "Object"` | `answers  rc 168` |

The rows landing on `rc 163` and `rc 168` are the harness's zero-argument probe meeting the method's
own arity refusal — `93.903` for `=` and `instanceMethod`, `88.901` for `isInstanceOf` — raised
identically on both sides, which is what `answers` records. Per the standing constraint this verdict
is **not** cited as evidence any row works; the differential and the witness are.

### `corpus/introspection-arity.tsv`

Refreshed with:

```
REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity
```

**Fifteen rows moved, all `send-differs` -> `agree`**: the fourteen above plus `Class enhanced`.
Nothing else moved.

| row | before | after |
| --- | --- | --- |
| `Class = / == / <> / >< / \= / \==` | `send-differs  oracle rc0 VALUE <0 or 1>; crate rc120 rexx-exec: method "<op>" of class "Class" is not implemented (Phase 5)` | `agree  rc0` |
| `Class isAbstract` / `isMetaclass` / `queryMixinClass` | `send-differs  oracle rc0 VALUE 0; crate rc120 ... not implemented` | `agree  rc0` |
| `Class methods` | `send-differs  oracle rc0 VALUE a Supplier; crate rc120 ...` | `agree  rc0` |
| `Class subclasses` | `send-differs  oracle rc0 VALUE an Array; crate rc120 ...` | `agree  rc0` |
| `Object instanceMethod` | `send-differs  oracle rc0 VALUE a Method; crate rc120 ...` | `agree  rc0` |
| `Object instanceMethods` | `send-differs  oracle rc0 VALUE a Supplier; crate rc120 ...` | `agree  rc0` |
| `Object isInstanceOf` | `send-differs  oracle rc0 VALUE 1; crate rc120 ...` | `agree  rc0` |
| **`Class enhanced`** | `send-differs  oracle rc0 VALUE enhanced K; crate rc0 VALUE a K` | `agree  rc0` |

The `enhanced` row is the silent wrong answer this task owned: it was the one row already at rc 0 on
both sides and differing only in its value, and it is now the instrument's own evidence that the
rendering matches.

**A note on reading this table's failure output.** When the committed table is stale the test prints
pairs, and the **first** element is the fresh measurement while the second is the committed row. I
read that pair backwards at first and briefly believed my own fix had not taken; a direct probe of
the harness's exact receiver (`r = .K` from `corpus/introspection-receivers.tsv`, not `.Array`)
settled it in one run.

### `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`

`lang/class_introspection.rex` added to both, under a `# Task 5:` comment naming what it witnesses.
They must change together or `phase_5c_subset_matches_the_committed_list` fails; before the addition
`every_lang_program_is_run_or_named_unfiled` failed naming the new file, which is that gate working.

`corpus/unfiled.txt` is **not** touched: the witness is filed, not excused.
`corpus/refusal-sites.tsv` is **not** touched: no `Loud`/`Raised` constructor was added or
re-surfaced — every refusal in this change reuses an existing one.

## Gates

**Fast checks, run in the working tree before the commit.** Each figure below is read from the
output of the command quoted beside it.

| check | command | result |
| --- | --- | --- |
| format | `cargo fmt --all --check` | exit 0, no output |
| lint | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no output |
| tests | `cargo test --release --workspace --no-fail-fast` | exit 0, **2282 passed, 0 failed** |

The test run above is the second. The first, before the shared artifacts were refreshed, was exit
101 with **2279 passed and 3 failed** — `every_lang_program_is_run_or_named_unfiled`,
`the_table_matches_the_three_sides` and `no_row_started_diverging_or_stopped_answering`. All three
are the artifact gates noticing rows this change moved, and all three are the reason the refreshes
above exist rather than separate defects.

Neither run was wrapped in `memcap`, per the ruling.

**Full gate suite**, run in `/home/moritz/dev/repos/ooRexx-5i-gates` pinned to this task's commit:

| gate | result |
| --- | --- |
| G1 | **G1** |
| G2 | **G2** |
| G3 | **G3** |
| G4 | **G4** |
| G5 | **G5** |
| G6 | **G6** |
| G7 | **G7** |

## Rulings table

Every ruling in the brief's `## Rulings` section and every ruling sent since, against what was done
with it. Re-read at the end, not from memory.

| ruling | what was done with it |
| --- | --- |
| **The arity instrument cannot see an object's class** — witness each object-answering row through a second send reading `~class~id` as well as a value | **Applied.** `~methods` and `~instanceMethods` are read through `~class~id` and a full walk, `~subclasses` through `~class~id`, `~items` and an element's `~id`, `instanceMethod` through `~class~id`, `~scope~id` and `~scope~class~id`. Section D2 also reads each supplier item's `~class~id`, so a `Supplier` of the wrong element class would fail. |
| **The live builtin-argument-run defect** — assign to a variable before nesting a send in a multi-argument builtin call | **Applied.** No send is nested inside a multi-argument builtin anywhere in the witness; the supplier rows pass one send as `CALL`'s second argument and immediately `parse arg` it, and every other row assigns first. Not hit once. |
| **`Class~enhanced` is a silent wrong answer and it is yours** — fix it, witness the rendering rather than the send's success | **Applied.** Fixed via the behaviour flag the C++ uses. The witness reads `~string`, `~defaultName` and `~objectName`, and asserts `~extra` answers `99` beside them so the rendering fix is not confused with the method working. The empty-table case is witnessed too. |
| **Two witnesses other tasks want, if they come cheap** — `.RexxInfo~package == .Array~package`, and any `Class` operator row a later task needs | **Half applied, and the halves separate cleanly.** The `~package` identity does **not** come cheap and is still loud: measured, oracle `1` at rc 0, ours rc 120 ``the operator `==` applied to one of the interpreter's own objects``. `Package` is a `Body::Native`, and this change made *class handles* operator receivers and nothing else, so Task 3's witness must keep `~name` as its second send. **But a class-to-class identity now is assertable** — measured, `(.Array~class == .Class)` is `1` at rc 0 on both sides where it was loud at BASE. Any later task wanting a `Class` operator row has it. |
| **The shared artifacts** — a task commits the ones its change moves, with each moved row's verdict before and after and the refresh command quoted | See *Shared artifacts, before and after*. |
| **`rustfmt --edition 2024 <path>`, never bare `rustfmt`; do not wrap the release fast check in `memcap`** | **Applied.** Every format run in this task was `rustfmt --edition 2024` over named paths; the fast check ran bare. |
| **A scripted multi-part edit that partially applies is worse than one that fails** — assert every replacement, verify what you wrote before writing the sentence describing it | **Applied.** Every edit was a single `Edit` call that fails loudly on a missed match; no multi-pattern script was used. The one citation written from memory (`Setup.cpp:479`-`:484`) was checked against the C++ before the build, found wrong, and corrected to `:485`-`:490`. |
| **Take your design** — delete the `is_class_slot` early-out in `operator_message_receiver`, bind the six under `"Class"` in `NATIVE_METHODS`, leave `operator_operand_gap` alone | **Applied exactly.** `operator_operand_gap` has no edit of any kind. |
| **The arithmetic operators are in scope — assert rather than argue, and run the two open claims first** | **Applied.** Both predictions were written down and then run at BASE before any edit; both confirmed. See *The two predictions*. |
| **`body.rs` is on your list, with two conditions: try `ObjectMethods` first, and report `body.rs:665`'s assertion state either way** | **Applied.** The flag is on `ObjectMethods`, behind the existing `Box`. `assert!(size_of::<Body>() <= 80)` is **unmoved and passing** — `Body` gained no field. The empty-table measurement that ruled out the derived version is in the report as the reason. |
| **BASE is the HEAD you find on release; no rebase** | **Applied.** BASE `8bc2df809`, stated at the top of this report. |

## Concerns

**1. Three sites still refuse loudly where the oracle answers 97.1, and this task deliberately did
not close them.** `Interp::header_number` (`run.rs:8674`), `Interp::controlled_step_wide`
(`run.rs:10379`) and `RAISE ADDITIONAL` (`run.rs:5417`) ask `operator_operand_gap` and never ask
`operator_message_receiver`, so a class object in a `DO` header, in a control variable, or in a
`RAISE ADDITIONAL` list is `rexx-exec: ... is not implemented (Phase 5)` at rc 120 where the oracle
is 97.1 at rc 159. **This is pre-existing and not a regression** — those rows were loud at BASE and
are loud now, unchanged and byte for byte. It is named here rather than left silent because closing
the operator arm makes it the last place a class object still refuses, and because it is a
loud-versus-97.1 gap and not a wrong answer. Closing it means teaching those three the send, which
is a change to `run.rs` that this task's file list does not cover and whose blast radius is the
controlled-loop path.

**2. Ruling 4's `~package` witness is still not assertable; its `Class` half now is.** Measured,
`(.RexxInfo~package == .Array~package)` and `(.Array~package == .String~package)` are both `1` at
rc 0 on the oracle and rc 120 here, ``the operator `==` applied to one of the interpreter's own
objects``. A `Package` is a `Body::Native`, and `operator_message_receiver` still answers `None` for
that body, so Task 3's witness must keep `~name` as its second send. The same mechanism would close
it — a `Body::Native` arm plus `==` bound where those classes declare it — but that is a different
receiver kind and was out of scope. **A class-to-class identity is assertable now**: `(.Array~class
== .Class)` answers `1` at rc 0 on both sides where it was loud at BASE.

**3. The supplier order divergence is real, licensed, and asserted nowhere.** The oracle's suppliers
are in hash order and this crate's are in `BTreeSet` name order. No witness asserts order, and the
witness says so in its own header. If a later task ever needs the orders to agree, this is where
that starts.

**4. `~subclasses` holds strong references where the oracle's list is weak, and the question does not
arise.** `RexxClass::getSubClasses` is `subClasses->weakReferenceArray()`; this crate's is a plain
`Vec<ObjRef>`. Spec D59 says classes are never collected here, so no entry can become unreachable
for the array to drop, and there is no program that distinguishes the two. This is recorded rather
than left unstated, as the brief asked: **the answer is that the question does not arise**, not that
the lists behave alike under collection.

**5. One file outside the dispatch list was edited, and it was required rather than chosen.**
`crates/rexx-exec/tests/dispatch_seam.rs` gains `"src/dispatch/introspection.rs"` in
`CLEARANCE_CONSUMERS`. The global constraint requires a new dispatch submodule to be added there;
without it that test fails. Flagging it because it is not in this task's own file list.
