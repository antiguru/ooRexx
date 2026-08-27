# Consolidated review -- Phase 5a gate close (Tasks 1-6, `8b4a7459d`..`743dd4109`)

Reviewed at `743dd4109`, tree clean, `rust/target/release/rexx-run` current:
`find crates -name '*.rs' -newer target/release/rexx-run` printed nothing, so
the binary post-dates every source file under `rust/crates`.

Not re-verified: the five gate rows, the five gates, Task 6's negative
control. Those were the controller's instrument and this review is what that
instrument cannot see.

## Verdict per dimension

| dimension | verdict |
| --- | --- |
| silent wrong answers outside the rows | **clean** |
| cross-task interactions | **clean** |
| prose | **5 findings** |

Findings by severity: **0 high** (nothing that could produce a wrong answer a
user would believe), **2 medium**, **3 low**.

---

## 1. Silent wrong answers -- clean

**Method.** 89 probe programs, each run against the oracle and against both
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, stdout, stderr and exit status
`cmp`-ed separately, every run from its own freshly created empty directory
with absolute paths. `ls $P/zzp_*.rex | wc -l` -> `89`. Oracle wrapper and
crate wrapper exactly as the constraints spell them. No program in
`corpus/oracle-crashes.txt` was run; the metaclass-`NEW` shape (entry 8) was
deliberately left alone.

**Every divergence found was loud.** Not one probe produced rc 0 with wrong
stdout, and not one produced a wrong error *number* at a matching exit code.
Every disagreement was either `rexx-exec: ... is not implemented (Phase N)` at
rc 120, or one of the divergences the ledger already records.

What was probed past the rows, by task:

* **Task 1, `::ATTRIBUTE EXTERNAL`.** A bound getter actually answering
  (`.k~at` -> `/`), a bound setter answering through `~"AT="()`, arity
  refusals on both accessors (`88.922 ... 0 expected`, rc 168, byte-identical
  including the `Compiled method "AT=" with scope "K".` frame),
  `~hasMethod` on both halves of a `GET`-style pair, and the bound accessor
  reached from a `::METHOD` body (`self~at`). All agree.
* **Task 2, `Method~scope`.** `::CONSTANT`, both `::ATTRIBUTE` accessors, a
  library class's own method, a method donated between dictionaries
  (`.Directory~method('AT')`), an overriding pair, a `~define` that fills a
  scope in place, a `~define` that copies an already-scoped object, a
  `~delete` then re-`~define` (the method-object cache's drop path), and
  `~scope` on a name reached only through `~inherit` (97.1, correct). All
  agree.
* **Task 3, `~subclass` / `~mixinClass`.** `~id`, `~string`, `~baseClass`,
  `~superClass`, `~superClasses`, `~metaClass`, `~class`, `~isA`,
  `~isSubclassOf`, `~package`; a runtime metaclass built by
  `.class~subclass`; two classes sharing one id; nested subclassing; a
  `~mixinClass` inherited into a `::CLASS`-declared class; the class-side
  `INIT` firing once per install and once per `~subclass`; an enhancing
  `INIT` running *inside* the send; a failure raised inside an enhancing
  `INIT` (rc 214, `Compiled method "SUBCLASS" with scope "Class".` frame
  byte-identical); the argument ladder (`88.901` with the `NEW`+`SUBCLASS`
  frame pair, `88.909`, `93.902`, `99.927` for `.Object` and for `.nil`,
  `97.1 ... "SUPPLIER"`); `~define` on what it built succeeding while
  `.array~define` is still `98.985`; `~subclass` called from inside a class
  method body. All agree.
* **Task 4, `.RexxInfo`.** The instance and the class object; `~objectName=`
  changing every rendering; `~isA`; `~hasMethod` on present and absent names;
  a program declaring its own `::class RexxInfo` (shadows, 97.1 on the
  program's class -- agrees); `.RexxInfo~class~subclass("kk")`. All agree.
  Independently re-measured the committed instance-method set:
  `do idx over .RexxInfo~class~methods(.nil)~allIndexes~sort; say idx; end`
  on the oracle gives 29 names, `diff` against
  `rexxinfos_own_instance_methods_are_setup_cpps_and_carry_no_id`'s list is
  empty.
* **Task 5, `~define` from source.** The string arm and the **array** arm
  (reachable in this phase only through `.resources[...]`, `~superClasses`
  and `.methods~put`, all three exercised); a `#!` first line; an empty
  string; a `\n` and a Ctrl-Z inside an element; a body that does not parse;
  a body carrying a directive; upcasing on both `~define`'s argument and the
  dictionary key; `~defineMethods`; and the annotation question below. All
  agree or refuse loudly.

**The one place a silent wrong answer was most likely, and it is closed.**
`Interp::method_object` attaches `Annotated::Member(class, false, name)` to
every method object it mints. Had `~define` with source text left the
dictionary entry to be re-minted that way, `.k~method("M")~annotation(...)`
after a `~define` over a name an `::ANNOTATE METHOD` already annotated would
have answered the *directive's* pairs at rc 0 where the oracle answers
`.nil`. Measured on the oracle and both engines: `before b` / `after The NIL
object`, byte-identical. `Annotated::Compiled(count)` is why, and its doc says
so.

**Memory.** `compile_method_source` allocates the method object and then calls
`attach_annotations`, which allocates again before anything roots the object.
That is safe because `Interp::native_instance` pushes a temporary root
(`environment.rs:836`); no gap. `Interp::method_object` roots as a global
before attaching, for the same reason. Nothing here needs changing.

## 2. Cross-task interactions -- clean

Pairs that were never exercised together by any single task's acceptance test,
each run as a differential probe:

| pair | probe | result |
| --- | --- | --- |
| 1 x 2 | `::attribute at get external 'LIBRARY REXX file_separator'` then `.k~method("AT")~scope~id` | `K`, agrees |
| 2 x 3 | `.object~subclass("kk")` then `~define` then `~method(...)~scope~id` | `kk`, agrees |
| 2 x 5 | compiled method's scope; `.methods~z~scope` before and after each installer | agrees |
| 3 x 5 | enhancing table holding source text (loud), holding a bad **array** (`93.952` byte-identical, rc 163), holding a `~define`-produced method object (loud) | agrees |
| 3 x 5 | `.methods~z~scope` after `.object~subclass("k", .Class, .methods)` -> `The kk class` | agrees |
| 3 x 4 | `.RexxInfo~class~subclass("kk")` -> id `kk`, superclass `The RexxInfo class` | agrees |
| 3 x 2 | `~subclass`-built mixin `~inherit`ed, then `~method` on the inherited name | 97.1, agrees |
| 4 x pre-existing | `::CLASS K SUBCLASS RexxInfo` | diverges -- **and so does `ENDOFLINE`**, which this plan did not touch, byte for byte the same way; so the divergence is general and `.RexxInfo` added nothing |

**The registry split was checked for collateral.** `define_system_class` moves
a class off the `.environment`-reachable path, so a second class taking that
route would silently lose a `.NAME`.
`/bin/grep -an 'EndSpecialClassDefinition' interpreter/memory/Setup.cpp` names
exactly `RexxInfo`, `Integer` and `NumberString`; the latter two are in
`DEFERRALS` and `continue` before the `system_only` branch is reached. So
`RexxInfo` is the only class that takes it, and `.environment` lost no name.

**No row was re-filed to dodge the gate.** The diff changes no `phase:` field
in `gate_table_c.rs`, and table D's row source file is not in the diff at all
(`git diff --name-only` under `docs/` names only the plan and
`phase-4-exclusions.txt`). The plan's Task 6 prohibition was honoured.

**The oracle SIGSEGV is real and the crate provably cannot reach it.**
`corpus/oracle-crashes.txt` entry 8 is well bounded -- the neighbour that
forwards is rc 0 and the direct `.MyMeta~new("k")~id` send is a clean 97.1, so
the cast and not the overridden `NEW` is the cause. `factory_metaclass`
refuses any metaclass whose `NEW` resolves anywhere but `.Class`, and
`a_metaclass_with_its_own_new_is_loud` pins both arms (the refusal *and* the
answering neighbour, so a build that refused every named metaclass fails).

## 3. Prose -- 5 findings

### F1 (medium). `method_scope.rex` states as a property of `~defineMethods` what is only a property of an already-scoped object -- and contradicts a sibling this plan committed

`rust/corpus/lang/method_scope.rex`, header bullet:

> `~defineMethods`, which copies for the reason its two `newScope` calls give
> (...), leaving the object it was handed alone

Measured, oracle and both engines, rc 0, three descriptors identical:

```
say 'a' .methods~z~scope        ->  a The NIL object
.k~defineMethods(.methods)
say 'b' .methods~z~scope        ->  b The K class
```

A scope-less object is filled **in place** by `~defineMethods`, exactly as by
`~define`. The file's own `define-methods-copies` row only witnesses a copy
because `m`'s scope was filled two rows earlier by `.k2~define('Y', m)`.

`rust/corpus/lang/method_from_source_table.rex`, committed by Task 5 of this
same plan, states the mechanism correctly and oppositely: "the table's own
object comes away carrying the scope the first of those two calls set on it".
Two committed corpus programs disagree about the same C++ function.

The code is right on both sides; this is prose only. **Prefer deleting**: the
two bullets above it already state the `newScope` conditional for `~define`,
so the clause "which copies ... leaving the object it was handed alone" can go
and the bullet keep only its citation. The row label
`define-methods-copies` would be more honest as
`define-methods-does-not-rescope`.

### F2 (medium, confirms and enlarges a known open item). The stale task-number citations are 26 lines, not two rows, and none is resolvable

```
/bin/grep -ac 'Task [0-9]' crates/rexx-exec/tests/gate_table_c.rs   ->  25
/bin/grep -ac 'Task [0-9]' crates/rexx-exec/tests/gate_table_d.rs   ->   1
/bin/grep -ac '2026-08-17-phase-5a\|superpowers/plans' crates/rexx-exec/tests/gate_table_c.rs  ->  1
```

The ledger parks two (`xscope` "Task 9", `usingcl` "Task 7"). The set is
larger and spans at least three plan generations -- Tasks 6-18, Tasks 21-22,
Task 24 -- across `authority` fields, `control` fields and the module doc, and
exactly one line in the file names a plan path at all, so a reader cannot
resolve any of them to a document.

This matters more after the flip than before it, and that is the whole reason
to raise it: a `control` field is now the instruction for reproducing a red on
a **gating** row, which is precisely the moment someone follows the citation.

`corpus/lang/class_method_class_side_raises.rex`, the one file-path citation in
the neighbourhood, does exist (`ls` confirms), so this is task numbers only.

### F3 (low, confirms a known open item). Both "no table C row exercises the scope question" sentences are false, and one is false twice

* `crates/rexx-exec/tests/gate_table_c.rs`, "What this table cannot see":
  "The plan builds no scope row class -- the documentation supplies no
  expected answer for the scope question -- and the instrument is instead the
  `~method` corpus programs Task 9 commits."
* `corpus/lang/class_method_own_dictionary.rex`, opening comment: "There is no
  table C row for it ... so this program and `class_method_class_side_raises.rex`
  beside it are the whole of the protection."

The `xscope` row is a scope row. Its own header quotes `provide.xml`'s
`xscope` section -- "...and Method~scope answers which" -- which is the
documentation supplying the expected answer; its first two lines read
`Method~scope`; and its third, `.sub~method("BASEONLY")`, is the
inherited-name discrimination both sentences say is not in the table. Since
the flip, `xscope` gates, so "the whole of the protection" is wrong as well.

`corpus/gate-tables/concepts/xscope.rex` is unchanged since `0e605eb77`, so
neither sentence is a regression this plan introduced -- the ledger's reading
is right. But `class_method_own_dictionary.rex` was edited by Task 2 (a false
block was deleted from its tail) and this sentence four lines from the top was
left standing, which is the [[correction-rounds-introduce-false-statements]]
shape: the neighbourhood was open and only the named finding was fixed. The
flip is what makes both load-bearing. Prefer deleting the clauses rather than
rewriting them.

### F4 (low). `rexxinfo_entry.rex`'s opening line is a false cardinality claim

`rust/corpus/lang/rexxinfo_entry.rex`, line 1:

> `.RexxInfo`: the one `.environment` entry that is a pre-built instance
> rather than a class object.

Measured, oracle rc 0:

```
.environment['ENVIRONMENT']   ->  The Environment Directory
.environment['LOCAL']~class   ->  The Directory class
.environment['NIL']           ->  The NIL object
.environment['ENDOFLINE']~class -> The String class
.environment['TRUE']          ->  1
```

Several `.environment` entries are pre-built instances rather than class
objects. The distinguishing property the file actually means is in its very
next paragraph and is correct (`EndSpecialClassDefinition` routes the class to
`addToSystem` while only the instance is `addToEnvironment`'d). Deleting "the
one" from the first line costs nothing.

The same shape, weaker, in `crates/rexx-exec/src/environment.rs`: "where every
other entry built from a class renders as `The X class`". `.ENVIRONMENT` is an
entry built from the `Directory` class and renders `The Environment
Directory`. Under the reading "every other entry that *is* a class object" the
sentence is true; under the natural reading of "built from a class" it is
false. Ambiguous rather than wrong; listed here only so it is not rediscovered.

### F5 (low). The new `methna` control text attributes lookup-upcasing to the wrong function

`gate_table_c.rs`, `methna`'s `control`, added by this plan:

> `MethodDict::replace_method`, which upcases every key on insert and on
> lookup

Measured: `replace_method` upcases on **insert** only
(`crates/rexx-classes/src/method_dict.rs:161`). The lookups upcase in their
own functions -- `:176`, `:194`, `:204`, `:215`, `:362`.

The control still works exactly as written, and Task 6's run proves it: with
`replace_method`'s insert upcase dropped alongside `method_name_pair`'s, the
lookups still upcase, which is what makes both `~method("TYPE")` and
`~method("type")` miss and reddens exactly `methna`. So this is an imprecise
attribution and not a broken criterion. Worth a word because the sentence
would send the next reader to `replace_method` for a lookup that is not there.

## Confirmations of the ledger's other open items

* **`RexxInfo~copy`**: loud rc 120 here against the oracle's rc 163
  `93.970`. Confirmed; a refusal, not a wrong answer.
* **`::CLASS K SUBCLASS <non-class environment entry>`**: confirmed
  pre-existing and general. `ENDOFLINE`, untouched by this plan, gives the
  identical oracle `99.949` rc 157 against this crate's `98.909` rc 158, so
  `.RexxInfo` joined an already-divergent set and changed nothing.
* **`Class~queryMixinClass`**: still a loud rc 120 (`.object~mixinclass("m1")~queryMixinClass`,
  oracle `1`). Confirmed.
* **The unbounded class allocation**: not re-measured; the ledger's figures
  stand and the axis is the licensed OOM one. Worth noting that `~subclass` is
  now reachable from inside a method body and from a loop, so the count is
  fully under a user program's control -- which the ledger already says.
* **`INTERPRET`'s parse transcript** and **`condition('A')`**: not re-probed.
  Both are pre-existing gaps owned by nobody and neither is a wrong answer.
* **The `93.952` "method source" position string** is correct on the oracle
  for both `~defineMethods` and `~subclass`'s table (measured, rc 163 each),
  but neither is reachable from a Rexx program through `~[]=` on a
  `StringTable` -- that method is unbuilt. `.methods~put` is the route the
  corpus uses, and it works. Not a defect; noted so nobody reads the
  in-crate test as unreachable.

## What I did not do

* Did not re-run the gate rows, the five gates, or Task 6's control.
* Did not modify the working tree beyond writing this file. Two mutations
  would sharpen findings if you want them run: (a) dropping only
  `MethodDict::replace_method`'s insert upcase, to confirm F5's reading that
  the row still reddens; (b) nothing else -- F1 through F4 are prose and need
  no mutation.
