# Task 7 report: `~copy`, `~run`, `~send`/`~sendWith`, `~start`/`~startWith`

BASE `cabc32418`. Report written first and appended to as work proceeds.

## Status

COMPLETE. Five gates exit 0, phase gate as expected at BASE, committed at `1f57c3d02`
(24 paths, tree clean afterwards).

---

## 1. The brief's "what is already there" paragraph, re-derived

Every claim re-measured rather than taken.

**Confirmed.** The native method tables in `crates/rexx-exec/src/dispatch.rs` carry no `COPY`,
`RUN`, `SEND`, `SENDWITH`, `START` or `STARTWITH` row, on any class. Derived by parsing the three
tables (`NATIVE_METHODS` `:246`-`:486`, `NATIVE_CLASS_METHODS` `:496`-`:510`, `SETUP_METHODS`
`:527`-`:540`) with comments stripped, rather than read by eye:

```
python3 - <<'PY'   # over dispatch.rs lines 246-486, comments stripped
Object : CLASS, DEFAULTNAME, HASMETHOD, IDENTITYHASH, INIT, ISA, ISNIL, OBJECTNAME,
         OBJECTNAME=, REQUEST, SETMETHOD, STRING, UNSETMETHOD
Array  : AT, ITEMS, MAKESTRING, SIZE, TOSTRING, []
```

`Array` indeed has no `[]=`, `OF` or `NEW`. `.Array~of('M','x')` is measured rc 120,
`rexx-exec: method "OF" of class "Array" is not implemented (Phase 5)`, on both engines, against
oracle rc 0 / `2`.

**Two corrections, both small and neither in the direction that shrinks the task.**

* `NEW` is a row of `NATIVE_CLASS_METHODS` (`("Object", "NEW", Arity::Counted, native_new)`), not of
  `Object`'s instance table. The brief lists it among `Object`'s instance rows.
* **`.Message` exists.** The brief says "there is no `Message` class at all -- not the class". It is
  in the registry (`crates/rexx-classes/src/native_classes.rs:169`, `("MessageClass", "Message")`)
  and answers. Measured, `say .Message~id` / `say .Message~class~id`: oracle rc 0 `Message` /
  `Class`, and **both crate engines agree on all three descriptors at BASE**. What is missing is the
  *methods* -- `RESULT`, `COMPLETED`, `HASERROR` and the rest of `Setup.cpp`'s Message block -- and
  any way to make an instance of it. So "enough of `Message`" is method rows and an instance, not a
  class.

## 2. The Done-when correction the brief names, checked on both engines

The brief measured the array-literal spelling on `ir` only. Measured here on both:

```
$SP/probe.sh $SP/progs/p3_array_literal.rex     # a = ('M','x') ; say a~items ; say a[1]
ORACLE rc=0 -> "2" / "M"
CRATE[ir] rc=0  AGREES on all three descriptors
CRATE[tree-walker] rc=0  AGREES on all three descriptors
```

and the send program itself, `o~send(('M', .Base))`:

```
$SP/probe.sh $SP/progs/p5_send_override.rex
ORACLE rc=0 -> "plain sub" / "array base"
CRATE[ir] rc=120, CRATE[tree-walker] rc=120  (SEND not implemented -- this task's subject)
```

So the literal spelling is reachable at BASE on both engines and `.Array~of` is not. The plan's
Done-when is corrected in `docs/superpowers/plans/2026-08-27-phase-5b.md` accordingly.

## 3. The plan's own measurements, re-measured

Each ran under `probe.sh`, which runs the oracle under the standard wrapper from a fresh empty
directory and both crate engines, three descriptors read apart.

| plan's claim | re-measured | verdict |
|---|---|---|
| `~copy` write-through: `o~get` `orig`, `c~get` `changed`, `(o == c)` `0` | oracle rc 0, `same 0` / `copy-init orig` / `o orig` / `c changed` | holds |
| `o~run(...)` from a program context is 97.2 at rc 159 | oracle rc 159; **both crate engines already AGREE on all three descriptors at BASE** | holds, and confirms the brief's warning that a refusal-only witness is vacuous |
| from a method, `self~run('use arg x; return "ran" x', 'I', 5)` answers `ran 5` | oracle rc 0, `ran 5`; crate rc 120 | holds |
| D67: `self~run('expose v; return v*10')` is 41.1 at rc 215 where the class's `v` is 7 | oracle rc 215, stdout `class v 7`, stderr `Error 41 running RUN line 1` / `41.1 Nonnumeric value ("V")`, with a `Compiled method "RUN" with scope "Object".` frame | holds |
| `o~start('M',5)` answers a Message; `m~result` the value; `m~completed` after it `1` | oracle rc 0, `class Message` / `result ran 5` / `completed 1` / `haserror 0` | holds |

---

## 4. `~run`'s option surface, enumerated, and what this phase owes

`oodocs/rexxref/en-US/fundclasses.xml` `mthObjectRun` (line 3160) gives the shape; every row below
was then measured on the oracle rather than read off the prose. `RexxObject::run`
(`classes/ObjectClass.cpp:2185`) is the implementation and its step order is what the last rows pin.

### The `method` argument

| arm | oracle | owed or deferred |
|---|---|---|
| a source string | rc 0, `none=ran` | **owed, built** |
| an `Array` of source strings | rc 0, `array-source=lines 9` | **owed, built** |
| a `Method` object this crate compiled | -- | **owed, built** (`compile_method_source`'s own objects are in `table_method_bodies`) |
| a `Method` object from `Class~method` | rc 0, `from method object` | **deferred, loud** |
| anything else (`.nil`) | `93.974 The method argument must be a string, array, or method object.` rc 163 | **deferred, loud** |

**The `Class~method` deferral.** A `Method` object here is a `Body::Native` handle onto a dictionary
entry (`environment.rs:1193`, `Interp::method_object`) and carries no route to a body a send can
enter; only an object `compile_method_source` built has a `table_method_bodies` row. Making
`.K~method('M')` runnable needs a `Method`-object-to-`MethodId` route that no method in this crate
has, and it is not this task's subject. It refuses loudly rather than running under the wrong body.
Measured, oracle rc 0 / crate rc 120 on both engines, `self~run(.K~method('MK'))`.

**The 93.974 deferral is Task 3's, inherited.** `compile_method_source`'s own doc already refuses a
source that is neither a string nor an array loudly, with the reason: the oracle *converts*
`.environment` to a source and this crate models that directory as a subset, so raising 93.974 where
the oracle compiles would be a wrong answer. `~run` reaches the same function and takes the same
refusal.

### The option argument

| arm | oracle | built |
|---|---|---|
| absent | rc 0, no arguments | yes |
| `I`/`Individual`, any following text | rc 0, remaining arguments in order | yes |
| `A`/`Array` + a single-dimensional array | rc 0 | yes |
| `A` + a value needing `requestArray` | rc 0, `got x` for `'x'` | **deferred, loud** |
| omitted (`, , 5`) | `88.901 ... argument argument style is required.` rc 168 | yes |
| `.nil` | `88.909 Argument argument style must have a string value.` rc 168 | yes |
| `''` or any other text | `93.915 Method option must be one of "AI"; found "".` rc 163 | yes |
| `A` with no array | `88.901 ... argument argument array is required.` rc 168 | yes |
| `A` with a fourth argument | `93.902 ... 3 expected.` rc 163 | yes |

The `requestArray` deferral is the crate's standing one for `arrayArgument`, taken for
`native_hash_unknown`'s reason (`dispatch.rs`, `unconverted_array_argument`): the conversion is a
`MAKEARRAY` send this crate answers for no receiver. `~sendWith` and `~startWith` take it too.

### `send` and `sendWith`'s array form, measured before building it

The plan says this was not measured while writing the spec. Measured now, oracle, `::class Sub
subclass Base` with both defining `M`:

```
o~send('M')            -> sub      rc 0
o~send(('M', .Base))   -> base     rc 0
o~sendWith(('WITHARGS', .Base), (7, 8))  -> base 7 8   rc 0
o~start(('M', .Base), 7)~result          -> base 7     rc 0
```

`RexxObject::decodeMessageName` (`classes/ObjectClass.cpp:2119`) is shared by `send`, `sendWith`,
`start`, `startWith` and `Message~new`, so one function here serves all four this task builds.

---

## 5. A defect the enumeration caught: the message array's element count

The first build read the message-name array's **slot count**. `RexxObject::decodeMessageName` reads
`messageArgCount()`, which is `lastItem` (`classes/ArrayClass.hpp:305`) -- the position of the last
filled slot. The two differ for any literal with a trailing empty slot, and `('M',)` is one:

```
oracle:      o~send(('M',))  ->  93.946  rc 163
first build: o~send(('M',))  ->  88.914  rc 168   (both engines)
```

`('M',)` is `~size` 2 and `~items` 1, so the slot count passed the two-element test and the empty
second slot was then read as a missing class. It was found by `corpus/lang/object_send_refusals.rex`,
which was written before the fix, and the same rule governs the argument arrays: measured, oracle rc
0, `~sendWith` into a method reporting `arg()` passes `1` for `(5,)`, `3` for `(5, , 7)` with the
second omitted, and `2` for `(, 6)` with the first omitted. `message_argument_count` is the fix and
`message_argument_slots` applies it to the three argument arrays.

## 6. What was built

`crates/rexx-exec/src/dispatch.rs` gains ten native rows and their implementations:

* `Object~COPY` -- `Body::Instance` only; every other receiver is loud.
* `Class~COPY` -- 93.970, `Setup.cpp:483`'s override, which is where a class object stops.
* `Object~RUN`, `Object~SEND`, `Object~SENDWITH`, `Object~START`, `Object~STARTWITH`.
* `Message~RESULT`, `Message~COMPLETED`, `Message~HASERROR`.

`Primitive::Message` and `ObjectModel::message` are the receiver arm the last three resolve through.
`Interp::message_outcomes` records what each message's send ended with; the result itself is an entry
on the message's own `Body::Native`, which the collector walks, so a row in that table holds no
`ObjRef`.

`crates/rexx-exec/src/error.rs` gains three constructors, each for a `rexxmsg.xml` row already in the
generated catalogue: `copy_not_supported` (93.970), `message_name_shape` (93.972) and
`message_array_shape` (93.946).

### `~copy`'s scope, and what it does not do

`Object~copy` answers for an instance and refuses loudly for a string, an array, `.nil` and every
`Body::Native`. The oracle answers all of those -- measured, `'abc'~copy` is `abc` and
`(1,2)~copy~items` is `2`, rc 0. **Deferred rather than built**, and the reason is that each added
arm needs a witness and the identity half of one is not free: a short string in this crate lives in
the handle rather than in the arena, so what a copied string's `~identityHash` should answer is a
question this task would have had to measure and settle to witness the arm at all. No Done-when item
needs it, and a loud refusal cannot be a silent wrong answer.

### `Message`'s scope

Built: `~result`, `~completed`, `~hasError`, which is what the plan asks for. Every other row of
`Setup.cpp`'s `Message` block -- `NEW`, `~send`/`~sendWith`, `~start`/`~startWith`, `~reply`/
`~replyWith`, `~notify`, `~wait`, `~halt`, `~target`, `~messageName`, `~arguments`,
`~errorCondition`, `~hasResult`, `~messageComplete`, `~triggered` -- refuses loudly. So does
`.Message~new`, which is the one a program can reach without `~start`: measured, oracle rc 0,
`.Message~new(o, 'M', 'i', 5)~~start~result` answers, and this crate is rc 120.

**`.Message` itself already existed** -- see section 1 -- so what "enough of `Message`" turned out to
mean is three method rows, a receiver arm, and an instance nothing but `~start`/`~startWith` builds.

### Two divergences `~start` carries, both licensed by D68 and both recorded

`~start` runs its send before it answers, where the oracle runs it on an activity of its own. Neither
consequence is reachable by a check this phase may write, and both are measured:

1. **A started method's output.** It lands before the starting program's next clause here; the oracle
   interleaves the two and D68 says the order is not reproducible. No corpus program of this task
   writes to stdout from a started method.
2. **A started method that raises.** Measured, `o~start('M')` on a body of `1/0` then `m~result`:
   oracle **rc 214** with the 42.3 report on stderr **twice**, and this crate rc 214 with it
   **once**, at `~result`, under a `Compiled method "RESULT" with scope "Message".` frame. **The
   oracle's transcript does not reproduce**: four runs of that one program gave two orderings --
   three interleaving the two reports into one block whose `running ... line` lines both name the
   `m~result` clause, and one giving two complete reports naming the method's own clause and then
   `m~result`'s. Its `~hasError` sampled before `~result` reads `0` as readily as `1` (measured:
   `haserror 0 completed 0` on one such run), which is the same race D68 names. So the arm is
   asserted against this crate alone, in
   `run/tests.rs`'s `a_started_method_that_raises_has_an_error_and_reraises_at_result`.

   **An earlier draft of this report, of the plan's own paragraph and of `native_start`'s doc all
   said the first copy carries no `running <file>` line.** That is false for this shape and was
   carried over from the *unknown-message* start, where it is true; the four-run measurement above
   was taken because the claim was load-bearing in three places at once, and all three are
   corrected.

Only `Failure::Raised` is caught into a message. `Loud` is this crate saying it cannot run something
and has to reach the program; `Failure::Exited` is not a failure. Both propagate out of `~start`.

---

## 7. The witnesses

Eight programs, added to `rust/corpus/lang/` and to `rust/corpus/phase-5b.txt` with
`EXPECTED_SUBSET_5B` in the same commit. Each agrees with the oracle on stdout, stderr and exit
status on `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, read as three descriptors and never
`2>&1`, run from a fresh empty directory under
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`.

| program | what it carries | oracle rc |
|---|---|---|
| `object_copy.rex` | the write-through, the identity through `~identityHash`, the copied `setMethod` dictionary and FLOAT pool, and one forced finalizer per object | 0 |
| `object_copy_class_refusal.rex` | `Class~copy`'s 93.970 with its substitution and its `Compiled method "COPY" with scope "Class".` frame | 163 |
| `object_run.rex` | the allowing arm: every option, both source shapes, `parse source`, the FLOAT pool D67 puts it in, and the class-method caller | 0 |
| `object_run_refusals.rex` | the option table by exit status and sub-number, and the step order -- 93.915 before 98.991 | 0 |
| `object_send.rex` | `~send`/`~sendWith`, the array form's starting-class override, upcasing, arguments in order, a send with no value, `UNKNOWN` | 0 |
| `object_send_refusals.rex` | every message-name shape by exit status and sub-number, and the opposite argument order of `sendWith` and `startWith` | 0 |
| `object_send_name_refusal.rex` | 93.972 with its substitution, beside the same send spelled with a string | 163 |
| `object_start.rex` | `~start`/`~startWith`, the array form, `~result` then `~completed` then `~hasError` on every message, and a method answering nothing | 0 |

`object_copy.rex` and `object_start.rex` were each run **three times** on all three sides with
identical output, since both touch something D68 or D61 bounds.

**The two Done-when additions.** `object_start.rex` asserts `~hasError` after `~result` on all four
of its messages, and `object_send.rex` carries the starting-class override in the array-literal
spelling -- `o~send('M')` `sub`, `o~send(('M', .Base))` `base` -- which the plan is corrected to.

## 8. The controls, each recorded as run

Every control was applied to the source, rebuilt, and read two ways: the witness under `probe.sh`,
and

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus
```

whose mismatch list says **which** corpus programs catch it. In every case exactly one program did,
which is the "adds coverage" check as well as the "can fail" one: no other program in the union of
the five phase subsets sees the mutation, so deleting the witness would leave it uncaught. Sources
were restored from a scratchpad copy, never with `git checkout --`, and `md5sum` confirmed the
restore each time.

### Control 1 -- `~copy` shares the receiver's scope pools (the plan's named control)

`native_copy` records `copy -> receiver` in `Interp::class_variables` and `Interp::pool_owner`'s
instance arm consults it, so the copy's `EXPOSE` reads and writes the receiver's pools. This is the
plan's control exactly: identity and initial values are untouched and only the write-through moves.

```
322 of 323 matching
  [UNCLASSIFIED] lang/object_copy.rex: stdout differ

  rust:   ... copy-init orig float FV / one-off one-off / receiver changed float-changed / ...
          ... uninit changed / copy gone / uninit changed / receiver gone
  oracle: ... copy-init orig float FV / one-off one-off / receiver orig FV / ...
          ... uninit changed / copy gone / uninit orig / receiver gone
  exit=0 on both sides
```

`same 0 1` and `copy-init orig` are byte-identical under the mutation, so the plan's claim holds as
written: **only** the write-through lines redden it, and it reddens at rc 0 with empty stderr.

A second, blunter mutation was run first and is recorded because it is the one that would have
flattered the witness: emptying the copy's pools instead of sharing them is caught by `copy-init`
alone (`copy-init V float FV`), which is the non-discriminating half. A witness built from the
initial values would have passed control 1 and failed only this one.

### Control 2 -- the `~run` body's scope is the receiver's class rather than `ObjRef::NIL`

```
322 of 323 matching
  [UNCLASSIFIED] lang/object_run.rex: stdout differ

  rust:   ... float=FV shared=first run wrote class-pool=class-pool / class-pool self=K
  oracle: ... float=one-off wrote shared=first run wrote class-pool=V / class-pool self=K
  exit=0 on both sides
```

D67's own reading: with the class's scope the run body reads the class's `v` (`class-pool`) and loses
the FLOAT one-off's write.

### Control 3 -- `~send` reads the array's second item and never uses it as a start scope

`dynamic_send` validates the scope and then passes `None` to `send_message`.

```
322 of 323 matching
  [UNCLASSIFIED] lang/object_send.rex: stdout differ

  rust:   plain sub sub / scope sub sub / ... / scope-args sub 7 8
  oracle: plain sub sub / scope base sub / ... / scope-args base 7 8
  exit=0 on both sides
```

### Control 4 -- the `Message` does not keep the value its send answered

```
322 of 323 matching
  [UNCLASSIFIED] lang/object_start.rex: stdout differ

  rust:   result The NIL object / with-result The NIL object / scope-result The NIL object
  oracle: result sub 5 / with-result sub 6 / scope-result base 7
  exit=0 on both sides
```

The `novalue-result The NIL object` line is unmoved under this mutation, which is what makes the
three value-bearing rows the ones doing the work.

### Controls 5 and 6 -- the two refusal witnesses, run together

Applied in one build because they redden disjoint programs, and the mismatch list says which.

```
321 of 323 matching
  [UNCLASSIFIED] lang/object_run_refusals.rex: stdout differ
  [UNCLASSIFIED] lang/object_send_refusals.rex: stdout differ
  exit=0 on both sides
```

* **5**: the message array's element count is its slot count. `o~send(('M',)) -> rc 88 error 88.914`
  against the oracle's `rc 93 error 93.946`. This is section 5's defect, reintroduced.
* **6**: the restricted check runs before the option is read. `option 98.991 then 98.991` against the
  oracle's `option 93.915 then 98.991`, which is the sentence the program's own comment predicts.

### Controls 7 and 8 -- the two untrapped refusal witnesses, run together

```
321 of 323 matching
  [UNCLASSIFIED] lang/object_copy_class_refusal.rex: stderr differ
  [UNCLASSIFIED] lang/object_send_name_refusal.rex: stderr differ
  exit=163 on both sides
```

* **7**: `Class~copy` substitutes the class id rather than the object's string value, so the message
  reads `for object K` where the oracle reads `for object The K class`.
* **8**: the name-shape refusal substitutes a fixed string, so it reads `found "a value"` where the
  oracle reads `found "The NIL object"`.

### The three controls re-run against the final witnesses

`object_copy.rex`, `object_send.rex` and `object_send_refusals.rex` each changed after their controls
first ran -- a comment, one line, and two lines -- so controls 1, 3 and 5 were applied again together,
in one build, since they redden disjoint programs:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus
320 of 323 matching
mismatches (3):
  [UNCLASSIFIED] lang/object_copy.rex: stdout differ
  [UNCLASSIFIED] lang/object_send.rex: stdout differ
  [UNCLASSIFIED] lang/object_send_refusals.rex: stdout differ
```

One program per control and no others, against the bytes that are being committed. Controls 2, 4, 6,
7 and 8 target programs whose bytes did not change after they ran.

### The one control this crate cannot express, said plainly

The ordering asymmetry between `~sendWith` and `~startWith` was first written into
`object_send_refusals.rex` as `o~sendWith(.nil, .nil)` against `o~startWith(.nil, .nil)`, which is
how the oracle shows it -- 93.972 against 98.913. **That pair cannot be a corpus row here**: the
`startWith` half reaches `arrayArgument`'s `requestArray` conversion, which is the deferred arm of
section 4, so this crate is rc 120 and the program dies there. Measured, both engines. The witness
uses the shape that shows the same asymmetry through an arm both sides answer -- one call spelling,
two catalogue rows:

```
o~sendWith(.nil)  -> 93.972   (the name is decoded first)
o~startWith(.nil) -> 93.903   (the array is read first, and it is missing)
```

oracle rc 0 under the trap, and both engines agree. The two doc comments that had cited the `.nil,
.nil` pair were corrected to this one, because a comment naming a spelling the crate refuses sends a
reader at an rc 120.

---

## 9. Corrections made to the plan

Made in `docs/superpowers/plans/2026-08-27-phase-5b.md` itself, not in this report or the brief.

1. **`.Array~of` out of the Done-when**, which is what the brief names. The behaviour is real; the
   spelling is Task 9's and is rc 120 here, measured on both engines. The array literal is the
   equivalent route, measured on both engines at BASE, and the Done-when now reads
   `o~send(('M', .Base))`.
2. **`o~sendWith('M', .Array~of(4))` in the same paragraph** became `o~sendWith('M', (4,))`. The
   trailing comma is load-bearing and this is the correction to my own first attempt at the fix:
   `(4)` is a parenthesised expression and its `~class~id` is `String`, measured, while `(4,)` is an
   `Array` of one item.
3. **`(o == c)` cannot be the `~copy` witness's identity line.** An instance as an operator's left
   operand is a loud refusal this phase keeps on purpose -- `eval.rs`'s
   `a_named_instance_is_never_an_operators_left_operand` asserts it and
   `corpus/lang/instance_named_operands.rex` is the half the oracle answers -- so `(o == c)` is rc
   120 on both engines. The plan now says so and names
   `(o~identityHash == c~identityHash)` as the spelling, with the receiver-against-itself line beside
   it so the `0` is a difference rather than a comparison that never holds.
4. **What `~start` does with a raising method is now measured in the plan**, so that the reason for
   not asserting it is a measurement rather than its absence: oracle rc 214 with the report written
   twice. The sentence still says it is not asserted.

## 10. What I did not do

* **`Object~copy` for a string, an array, `.nil` or any `Body::Native`.** The oracle answers all of
  them; this crate refuses loudly. Section 6 has the reason.
* **`.Message~new`, and every `Message` row but `~result`, `~completed` and `~hasError`.** Loud.
* **`~run` given a `Method` object from `Class~method`.** Loud; section 4 has the reason.
* **`arrayArgument`'s `requestArray` conversion** for `~sendWith`, `~startWith` and `~run`'s `A`
  option. Loud, and the crate's standing position for that helper.
* **93.974 for a `~run` method argument that is neither string, array nor method.** Loud, inherited
  from `compile_method_source`, which is Task 3's decision and its reason.
* **`~start`'s scheduling.** D68 gives it to Phase 6. The send runs before `~start` answers and the
  two consequences are recorded in section 6.
* **Any change to a gate-table row.** The six phase-gate counts reproduce Task 6's exactly (section
  11), so no probe path moved into `corpus/phase-5b.txt` on that ground.
* **I did not verify per-row that no 5c gate-table row moved.** What was measured is the count, which
  is unchanged; a pair of rows moving in opposite directions would not show in it.

## 11. The phase gate

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

exit **101**, which is by design: `CLOSED_PHASES` plus `REXX_PHASE_GATE=5b` gates 5b's rows, and the
one Task 8 owns is red.

```
table C   5a: 135 rows, 0 not yet `agree`
          5b:   6 rows, 1 not yet `agree`     (methodsbyclass, Task 8's)
          5c: 1347 rows, 912 not yet `agree`
table D   5a:  36 rows, 0 not yet `agree`
          5b:   2 rows, 0 not yet `agree`
          5c:  38 rows, 35 not yet `agree`
          7:    1 rows, 1 not yet `agree`
```

All six 5a/5b/5c figures reproduce Task 6's report exactly, so nothing this task did moved a
gate-table row and no probe path moves into `corpus/phase-5b.txt` on that ground. The 5b numbers are
the brief's expectation for BASE: table C **1**, table D **0**.

---

## 12. A citation audit, and what it found

Every `interpreter/` line number this task added was printed and read, rather than trusted:

```
git diff | grep -oE 'classes/[A-Za-z]+\.(cpp|hpp):[0-9]+' | sort -u \
  | while IFS=: read -r f n; do sed -n "${n}p" /home/moritz/dev/repos/ooRexx/interpreter/$f; done
```

Seven were wrong and are fixed:

* **`RexxClass::copyRexx` is `classes/ClassClass.cpp:166`, not `:497`.** `:497` is
  `RexxClass::defineMethods`. The wrong number was in three places -- the table row's comment,
  `native_class_copy`'s doc and `Raised::copy_not_supported`'s -- because it was written once and
  copied.
* `validateScopeOverride` inside `sendWith` is `:1989`, not `:1986`.
* `startWith`'s `requiredArgument`/`arrayArgument` pair is `:2049`-`:2053`, not `:2052`-`:2057`.
* `run`'s option block is `:2207`-`:2235`, not `:2209`-`:2236`.
* `MessageClass::dispatch` is `:421` and `RexxObject::decodeMessageName` is `:2125`; both had been
  written at the line of the closing `*/` above them.
* `MessageClass`'s two flags are `MessageClass.hpp:61`-`:62` and its `condition` is a field at
  `:135`; the citation had put all three in one range that holds only the flags.

Every remaining citation lands on the declaration or statement it names.

## 13. Two false statements this task wrote and then caught

Both were caught by re-reading the neighbourhood of a change rather than by the change itself, which
is the failure mode this project keeps measuring.

1. **"once from the message's own activity with no `running <file>` line."** Written into the plan,
   into `native_start`'s doc and into this report at the same time, and false for the `1/0` shape --
   it was carried over from the *unknown-message* start, where the first report genuinely has no
   `running` line. The four-run measurement in section 6 was taken because the claim was
   load-bearing in three places, and all three now say what four runs actually show.
2. **A cross-reference that a trim had emptied.** `Primitive::Message`'s doc said "[`native_start`]
   names which of those rows this phase builds" after `native_start`'s own enumeration had been
   deleted in the same session. Found by reading the diff, not the file.

A third, smaller one: `corpus/phase-5b.txt` said the write-through was "the only line" that separates
a copied pool from a shared one. Control 1's transcript moves two of the program's output lines, and
naming a set's size is against this project's own comment rule twice over. Deleted.

---

## 14. The five gates

Run from `rust/` by a detached script that wrote each exit status to its own file as it finished, so
that no status is read from inside the turn that started the run. Statuses read back from those
files, unpiped:

| # | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

`memcap` was present (`command -v memcap` -> `/home/moritz/.local/bin/memcap`), so gate 5 ran under
it rather than under the `ulimit -v` substitute.

Gates 3, 4 and 5 each have **104** `test result: ok` lines and **no** `FAILED` line
(`grep -c` over each log). Gates 4 and 5 both print `323 of 323 matching` for the differential
corpus, which is BASE's 315 plus this task's eight witnesses, and `4246 of 4259 matching` for the
trace-surface report -- the same figure Task 4's fix report recorded, and its own output says it is
report mode and not the gate.

### The run certifies the tree that is being committed, and the hash is not what proves it

`tree-before.sha` was taken as

```
{ git status --porcelain; echo "---"; git diff; } | sha256sum
```

before gate 1 and re-run after gate 5: `08563f379a97...` both times, byte for byte. (The controller
computed `4b134e01b2...` for the same tree with the `echo "---"` omitted; that is a different
composition of the same two commands, not a moved tree, and re-running *this* command is what settles
it. The separator is not recorded anywhere the controller could read, which is the defect in how the
check was written down rather than in the check.)

**The hash's coverage is narrower than "the tree", and eight of this task's own files fall in the
gap.** `git status --porcelain` lists an untracked path but not its contents, and `git diff` does not
read untracked files at all -- so the eight `corpus/lang/object_*.rex` witnesses and their eight
`sourceline_oracle/*.txt` expectations were **named** by the hash and never **read** by it. An edit
to any of them during the run would have passed the check silently. Recorded rather than patched
after the fact, because patching the instrument now would not re-certify a run that has already
happened.

**What does certify them is the mtimes**, which cover contents and not just paths. The gate run
started at 01:20:26 (`g1.log`) and ended at 01:36:59 (`DONE`); every one of the 24 changed paths has
an mtime **before** the start, the newest being `dispatch.rs` at 01:19:38 and the oldest
`environment.rs` at 00:35:21:

```
git status --porcelain | awk '{print $2}' \
  | while read -r p; do printf '%s  %s\n' "$(date -r "$p" '+%F %T')" "$p"; done | sort
```

So nothing was written during the run, on a reading that covers the untracked files the hash does
not. This is the correction the check needs: hash the untracked files' own bytes as well, or say in
the same breath what it does not cover.

