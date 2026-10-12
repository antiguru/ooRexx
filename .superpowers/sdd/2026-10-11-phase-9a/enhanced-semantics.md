# `Class~enhanced` and metaclass `NEW`: oracle semantics, crate state, mechanism

Scout report for Task 8b, 2026-10-12. Read-only; nothing in `rust/` was edited.

**Method.** Every rule below comes from a probe run on the oracle with the standard wrapper
(`ulimit -v 1048576`, `timeout -k 5 20`, from the empty directory `/tmp/claude-1000/p9/scout-enh/run`).
Probe files are kept in `/tmp/claude-1000/p9/scout-enh/probes/` under the names given (`pN.rex`).
"Crate" columns are from `rexx-run` built at **988bf1e08** (committed HEAD; built from a `git archive`
copy, because Task 8's implementer had uncommitted edits in the tree), run under `memcap 2G timeout 20`.
stdout, stderr and rc were captured on separate descriptors; where a rule quotes only stdout, the
other two were empty and rc 0 unless stated.

**Oracle source.** `RexxClass::enhanced` (`interpreter/classes/ClassClass.cpp:1440`-`1480`) in order:
1. arg checks (`:1443`-`:1450`);
2. `subclass(OREF_NULL, "Enhanced Subclass", OREF_NULL, OREF_NULL)` (`:1453`), which is the full
   `RexxClass::subclass` (`:1562`-`:1640`): `NEW` sent to the receiver's metaclass, rewiring, the
   weak entry in the receiver's subclass list (`addSubClass`, `:1626`), `checkUninit`, and an `INIT`
   sent to the new class;
3. the enhancing methods compiled with scope `.nil` and merged into the dummy's instance dictionary,
   instance behaviour rebuilt (`:1456`-`:1463`), `checkUninit` (`:1466`);
4. `NEW` sent to the dummy with the remaining arguments (`:1470`);
5. on whatever `NEW` answered: `behaviour->setOwningClass(receiver)` and `setEnhanced()` (`:1474`-`:1476`).

`RexxBehaviour::superScope` (`behaviour/RexxBehaviour.cpp:549`-`:557`): a method of scope `.nil`
sees the behaviour's owning class as `SUPER`.

## 1. Observable rules

### R1. The enhanced object reports the receiver as its class, and renders `enhanced <id>`
`p1.rex` (`::class Thing`, `o = .Thing~enhanced(d)`):
```
class: THING
same class: 1
superclass: Object
isA Thing: 1 isA Object: 1
isInstanceOf Thing: 1
string: enhanced THING
defaultName: enhanced THING
objectName: enhanced THING
```
Error messages use the same rendering (`p3.rex`, last line):
`Error 97.1:  Object "enhanced THING" does not understand message "NOPE".` rc 159.
Crate: agrees on all of these (p1 crate run, identical lines).

### R2. The dummy class is visible in the receiver's `~subclasses` while it lives
`p1.rex`: `.Thing~subclasses~items` is `1` with one live enhanced object; the entry's
`id | superclass~id | class~id | isMetaClass` is `Enhanced Subclass | THING | Class | 0`.
`p3.rex` with two live enhanced objects: `2`. `p6.rex`: two `.string~enhanced` calls give
`.string~subclasses~items` = `2`, each id `Enhanced Subclass`. The entry is weak: after 200000 calls
in a loop (`mem200000.rex`) only `1` remained.
A failed `enhanced` still leaves its dummy behind (`p10.rex`, method source `return (`):
`trapped 35.1 subclasses: 1`.
Crate: `0` in every one of these. **Diverges.**

### R3. The class INIT runs for the dummy, before the methods are compiled
`p4.rex` (`::method init class` on Thing prints `self~id`):
```
Thing class INIT for THING
Thing class INIT for Enhanced Subclass
Thing INIT args: 3 1 3
```
Once per `enhanced` call (a second call prints a second `Thing class INIT for Enhanced Subclass`).
`p10.rex`: with a bad method source, `class INIT Enhanced Subclass` is printed and then 35.1 is
raised; the user `NEW` is not run.
Crate: the dummy class INIT never runs (`p4` crate: only `Thing class INIT for THING`;
`p10` crate: `class INIT THING` then `trapped 35.1 subclasses: 0`). **Diverges.**

### R4. NEW is sent to the dummy, so a user class-side NEW runs, with `self` the dummy
`p5b.rex` (Thing has `::method new class` that prints `self~id` and forwards):
```
Thing NEW class, self= Enhanced Subclass args 1
Thing INIT 1
o: enhanced THING foo THING
```
A metaclass instance-level `NEW` (the class-side `NEW` of every class built from that metaclass)
runs too, `p7b.rex`:
```
Meta2 instance NEW self= Enhanced Subclass args 1 earg
T INIT 1 earg
e: enhanced T foo
```
Crate: neither `NEW` line is printed; the rest agrees. **Diverges.**

### R5. INIT runs once, reached through NEW, with the arguments after the first
`p2.rex`, `.Thing~enhanced(d, 'A1', 'B2')` with an enhancing `INIT`: `enh INIT args: 2 A1 B2`, and
Thing's own `INIT` does not run (the enhancing one overrides it). Without an enhancing INIT the
class's INIT gets all of them (`p4.rex`, `enhanced(d, 1, 2, 3)`: `Thing INIT args: 3 1 3`).
An enhancing `INIT` that forwards to its superclass reaches Thing's (`p4.rex`:
`enh init 2` then `Thing INIT args: 2 x y`).
An extra argument when no INIT takes it: `93.902 Too many arguments in invocation of method; 0 expected.`
(`p9b.rex`).
Crate: once, same arguments, agrees, **except** the forwarding case: `forward class (super)` in the
enhancing INIT raises `88.914 Argument SCOPE must be an instance of the Class class.`, rc 168 (R9).

### R6. During NEW and INIT the object is an instance of the dummy; the class flips afterwards
`p11.rex`:
```
enh INIT: class= Enhanced Subclass string= an Enhanced Subclass isA Thing 1
Thing INIT: class= Enhanced Subclass string= an Enhanced Subclass
after: class= THING string= enhanced THING
stored during INIT: THING 1
```
The object saved during INIT is the same object, and reports `THING` once `enhanced` returns.
Crate: `enh INIT: class= THING string= enhanced THING isA Thing 1`, then 88.914 from
`self~init:super` (R9). **Diverges.**

### R7. Enhancing methods are `.nil`-scoped behaviour methods, listed at scope `.nil`, not removable
`p2.rex`, `p3.rex`, `p9e.rex`:
* `o~instanceMethod('FOO')~scope` is `The NIL object`; a `Method` object passed in keeps its identity
  (`x~instanceMethod('M') == .methods~mm` is `1`).
* `o~instanceMethods(.nil)` lists the enhancing names, and later `setMethod` names join them
  (`UM BASE SM FOO`, then `... BAZ`).
* `o~instanceMethods(.Thing)` lists only Thing's own (`BASE INIT`).
* `o~instanceMethods` (no argument) lists every entry of the behaviour, enhancing names included,
  with shadowed entries repeated (`INIT` appears three times: Object's, Thing's, the enhancing one).
* `unsetMethod` of an enhancing method is a no-op: `after unsetMethod(FOO) hasMethod foo: 1`, and
  after `unsetMethod('BASE')` the enhancing `BASE` still runs. `unsetMethod` of a `setMethod` name
  removes it (`p4.rex`: `after unset BAZ: 0`).
* `setMethod` on one enhanced object is not seen by another from the same receiver
  (`second object ... hasMethod baz: 0`).
* `setMethod`/`unsetMethod` are private: sent from outside, `97.2 ... cannot accept private message
  "UNSETMETHOD" from this context.` (`p2.rex` first version, rc 159).
* A `.nil` item hides the inherited method (`p9a.rex`: `nil hides: 0` for `d~base = .nil`).
* `EXPOSE` in an enhancing method binds in the object's `.nil` scope, shared by all enhancing and
  `setMethod` methods (`p2.rex`: `getv: A1`, then `set` after `foo` assigns it).

Crate: `unsetMethod`/`setMethod`/scope/EXPOSE agree. Diverges on:
`instanceMethod('FOO')` answers `.nil` (then `97.1 Object "The NIL object" does not understand message
"SCOPE"`, rc 159, `p2` and `p9e`); `instanceMethods` (no argument) omits every enhancing name;
a `.nil` item is refused with `93.974 The method source argument must be a string, array, or method
object.` (`p9a`, rc 163).

### R8. `copy` keeps the enhancing methods and the rendering
`p3.rex`: `copy: enhanced THING foo THING 1` and `copy hasMethod um: 1`. `p6.rex` on an array: `copy: foo Array enhanced Array`.

### R9. `SUPER` in a `.nil`-scoped method is the object's owning class
`p12.rex`:
```
enhancing method SUPER: super= The THING class
setMethod method SUPER: super= The THING class
setMethod on plain: super= The THING class
```
Per R6 this is the dummy while NEW/INIT run and the receiver afterwards. `self~base:super` from an
enhancing method runs Thing's `BASE` (`p3.rex`: `base: enh base then base in Thing`).
Crate: `super= The NIL object` on all three lines; `forward class (super)` and `~m:super` raise 88.914.
**Diverges, and not only for `enhanced`**: every `setMethod` method has the defect.

### R10. Primitive receivers build a subclass instance and keep the primitive behaviour
`p6.rex`:
```
array: 0 3 Array foo Array enhanced Array
array put: 1 x
string: hello 5 foo String String
concat: hello! arith on enhanced 5:
6 10 5x String
derived has foo: 0 String
n string: 5 n defaultName: enhanced String n objectName: enhanced String
datatype: NUM 1
directory: 1 1 foo Directory Directory enhanced Directory
stem: dflt1 foo Stem Stem dflt
object: foo Object enhanced Object
```
Derived values (`n + 1`) are plain Strings with no enhancing methods. A String's `~string` is its value, not `enhanced String`.
Crate: `.array~enhanced(d, 3)` raises `93.902 Too many arguments in invocation of method; 0 expected.`,
rc 163, the size going to `INIT`. **Diverges** for every primitive receiver that takes a `NEW` argument.

### R11. An abstract receiver can be enhanced
`p9b.rex`: `.abs~enhanced(.directory~new) -> ok enhanced ABS` (the dummy is not abstract).
Crate: `98.989 Class ABS is ABSTRACT and cannot be directly created.` **Diverges.**

### R12. Arguments and errors
`p9b.rex`, `p9c.rex`, `p9d.rex`:

| call | oracle |
|---|---|
| `.Thing~enhanced` | 93.901 `Not enough arguments for method; 1 expected.` |
| `.Thing~enhanced(,'x')` | 88.901 `Missing argument; argument methods is required.` |
| `.Thing~enhanced('notacollection')` | 97.1 `Object "notacollection" does not understand message "SUPPLIER".` |
| `.Thing~enhanced(.array~of(1,2))` | ok, methods named `1` and `2` |
| an item `5` | ok (source `5`) |
| an item `.object~new` | 93.974 `The method source argument must be a string, array, or method object.` |
| an item `'return ('` | 35.1 `Incorrect expression detected at "(".`, traceback `Compiled method "ENHANCED" with scope "Class".`, `Error 35 running A line 1`, rc 221 when untrapped |
| an empty Directory | ok, `enhanced THING` |
| lower-case index (`StringTable['lower']`) | method `LOWER` |

Crate: agrees on every row but the source error, where the message reads `Incorrect expression detected at "&1".`
(insert not filled; probably a general method-source defect, not specific to `enhanced`).

### R13. `enhanced` on an enhanced object's class is ordinary
`o~class` is the receiver (R1), so `o~class~enhanced(...)` is `.Thing~enhanced(...)`
(`p9a.rex`: `enhanced THING`). Both sides agree.

### R14. A metaclass's own `NEW` (class-side) builds every class made from it, and the metaclass INIT runs twice
`p7.rex` and `p14.rex`:
```
MyMeta NEW class, self= MYMETA args 1 k
MyMeta instance INIT self= k args 0
MyMeta instance INIT self= k args 0
k: k MYMETA
```
The class INIT a metaclass gives its instances runs **twice** per class, for `~subclass` and for a
`::CLASS ... METACLASS` directive alike (`p14.rex`: `K` twice before `start`, `dyn` twice after),
because `Class~new` sends INIT and `subclass` sends it again (`ClassClass.cpp:1846` and `:1631`).
This holds with no `NEW` override at all.
Crate: `p7.rex` refuses (`rexx-exec: method "NEW" of class "MYMETA" is not implemented (Phase 9)`,
rc 120); `p14.rex` runs the metaclass INIT once per class. **Both diverge.**
A metaclass `NEW` answering a non-class is oracle crash entry 8 (`rust/corpus/oracle-crashes.txt:254`); not probed.

### R15. The documented `.class~enhanced(methods, "Remote Class")` builds a class
`p15.rex`:
```
id: Remote Class class: Class string: The Remote Class class
dispatch: dispatched Remote Class
isMetaclass: 0 superclass: Object metaclass: Enhanced Subclass
```
then `rc_~new` is 97.1, the same as for `.class~new('x')~new` (`p16.rex`; crate agrees on `p16`).
Crate: `p15` raises 93.902 (the id goes to INIT). **Diverges.**

### R16. A user NEW that answers some other object: the oracle damages that object's class
`p8.rex` (`Odd`'s NEW answers an ordinary `Odd`; `Odd2`'s answers `'abc'`):
```
o: enhanced ODD 0
fresh plain Odd made after: enhanced ODD
q: abc ODD2
a new string: ODD2 enhanced ODD2
```
`setOwningClass`/`setEnhanced` write to the behaviour the answered object shares, so afterwards
**every** `Odd` renders `enhanced ODD` and **every String** reports class `ODD2`. This is an oracle
defect, not a semantic. **Recommend licensing the divergence**: hand the answer back untouched when it
was not built from the dummy. Crate today: `o: enhanced ODD 1` (it never runs the user NEW).

### R17. Memory and identity
Each call makes a new class (R2), and the dummy is collectable in the oracle. Oracle peak RSS,
measured with `/usr/bin/time -v env LD_LIBRARY_PATH=... rexx FILE` (the subject itself, not `timeout`):

| calls (`memN.rex`, one string source) | 20000 | 50000 | 100000 | 200000 |
|---|---|---|---|---|
| oracle max RSS kB | 30052 | 44140 | 65864 | 114572 |
| oracle wall | 0:00.52 | 0:03.90 | 0:17.47 | 1:22.78 |

`memplain.rex` (200000 `.Thing~new`): 20032 kB. So **the oracle is not flat** (about 0.47 kB per
call, the dead weak entries left in the subclass list), and its time is superlinear.

Crate at 988bf1e08 (no class per call), same files, direct run:

| calls | 20000 | 50000 | 200000 |
|---|---|---|---|
| crate max RSS kB | 392820 | 924760 | 3658776 |

That growth is **method-source compilation, not `enhanced`**: `memcompile.rex` (50000
`.method~new('foo', "return 'foo'")`, no enhanced) reaches 921808 kB against the oracle's 20324;
`memmeth.rex` (50000 enhanced calls with a Method object) is 45444 kB, and `memempty.rex`
(empty table) 27016 kB. **"Memory flat across a loop of enhanced calls" cannot pass with a string
source under any `enhanced` mechanism until compiled sources are reclaimed.** That defect is
separate, and Task 8b should not own it.

Cost of a class in the crate, `subN.rex` (`.Thing~subclass('x')` in a loop):

| calls | 20000 | 50000 | 200000 |
|---|---|---|---|
| crate max RSS kB | 431224 | 1044364 | 1399512 |
| live at end | 20000 | 50000 | 4113 |

Oracle `sub200000.rex`: 116300 kB, `1` alive. A crate class weighs about 20 kB until collected
(two flattened dictionaries), and no collection ran inside 50000 creations: the collection trigger
does not see class weight (open item in `docs/superpowers/plans/phase-5j-gate.md` §6; the 6.1 byte
trigger did not close it for classes, by this measurement). Each class also leaves two emptied
`Behaviour` slots behind for good (`ClassGraph::expunge`, `rexx-classes/src/class_graph.rs:308`).
Note that the brief's premise "classes are never collected in this crate" is out of date: Phase 5j
made `~subclass` classes collectable (819de57fe), and `sub200000` shows it working.

## 2. What ooTest exercises

Command: `grep -rn -i "~enhanced(" ootest` (inside the worktree) and
`grep -rniE "::method\s+['\"]?new['\"]?\s+class" ootest` (no output).

| site | rules |
|---|---|
| `ootest/ooRexx/base/class/Class.testGroup:261`, `:272` | R5 (INIT gets `tmpArg` via `test_c`'s INIT), R7 (`.methods` floating methods, EXPOSE in the `.nil` scope) |
| `ootest/ooRexx/base/class/Object.testGroup:1053` | UNINIT added by an enhancing method runs (dummy `checkUninit`, `ClassClass.cpp:1466`) |
| `ootest/ooRexx/base/class/Object.testGroup:1062` | R7 `.nil` item hides an inherited UNINIT. The crate refuses `.nil` (R7) |
| `ootest/ooRexx/base/class/Object.testGroup:1154` | `.Object~enhanced(.StringTable~of(("hashCode", "")))`, 91.999 from a HASHCODE that returns nothing |
| `ootest/ooRexx/base/class/Method.testGroup:234`, `:360` | `.object~enhanced(.directory~of(("m", m)))` with a Method object |

No ooTest file defines a class-side `NEW` (the second grep), so neither R4's class-side case nor
R14's `NEW` override is exercised. R14's double metaclass INIT is reached by
`Class.testGroup:533`-`555` (`.object~mixinclass("test_01", ...)` over the `counter` and `singleton`
metaclasses, whose instance INITs at `:1294` and `:1309` set `counter=0` / `singleton=.nil`); running
those twice is idempotent, so the tests cannot tell one INIT from two. `Class.testGroup:1163` (`test_subclasses_gc`) counts
`.DefineTester~subclasses`; it never calls `enhanced` on `.DefineTester`, so R2 does not reach it.

## 3. The crate today

`native_enhanced` (`rust/crates/rexx-exec/src/dispatch/class_protocol.rs:1108`) builds the object
with `new_instance(receiver)`, compiles the methods into a second object-level layer
(`ObjectMethods::enhanced`, `rust/crates/rexx-core/src/body.rs:823`), marks it, and sends INIT.
No dummy class, no NEW send. `factory_metaclass` (`class_protocol.rs:770`-`790`) refuses any
metaclass whose `NEW` does not resolve to `.Class`'s; `class_factory` builds the class in place
instead of sending NEW, and sends INIT once.

| rule | crate |
|---|---|
| R1 identity and rendering | agrees |
| R2 dummy in `~subclasses` | diverges |
| R3 class INIT on the dummy | diverges |
| R4 user / metaclass-instance NEW | diverges |
| R5 INIT once, arguments | agrees, except the forwarding case (R9) |
| R6 dummy class during INIT | diverges |
| R7 method level | `unsetMethod`, `setMethod`, EXPOSE agree; `instanceMethod`, `instanceMethods()`, `.nil` item diverge |
| R8 copy | agrees |
| R9 SUPER of `.nil` scope | diverges, also for every `setMethod` method |
| R10 primitive receivers | diverges (93.902) |
| R11 abstract receiver | diverges (98.989) |
| R12 arguments | agrees but the 35.1 insert |
| R13 | agrees |
| R14 metaclass NEW, double INIT | diverges (rc 120; single INIT) |
| R15 `.class~enhanced` | diverges (93.902) |
| R16 foreign NEW answer | oracle defect; recommend licence |
| R17 memory | string sources grow by about 18 kB per call from compilation, independent of `enhanced` |

## 4. Recommended mechanism: NEW is a real send, at both levels

One rule serves both items: **an object is obtained by sending `NEW` to a class, and the
caller then rewires what came back.** That is what `RexxClass::subclass` does to the metaclass's
answer, and what `RexxClass::enhanced` does to the dummy's answer.

**Class level (metaclass NEW, R14).** `class_factory` (and the directive install path,
`Interp::install_class_at`) sends `NEW(id)` to the metaclass instead of constructing in place. For
`.Class` and every metaclass without an override that lands in the existing `native_new_class`
(`class_protocol.rs:632`), which already builds a bare class and sends its INIT. The factory then
checks the answer is a class (oracle entry 8 crashes; raise rather than match), rewires it into a
subclass of the receiver (superclass, superclass list, metaclass flag, rebuild both behaviours,
subclass entry, enhancing class methods), runs `check_uninit`, and sends INIT a second time. This
needs one new registry operation, "reparent a freshly built class" (`define_cloned_class` today
fixes the superclass at `.Object`). The double INIT of R14 then falls out rather than being added.

**Instance level (`enhanced`, R1-R13, R15).** `native_enhanced` becomes the oracle's sequence:
1. dummy = `class_factory(receiver, "Enhanced Subclass", no metaclass, no class methods)`. This
   gives R2, R3 and R4's metaclass case for free.
2. compile the methods and define them in the dummy's own instance dictionary **with scope `.nil`**
   (a `define`-with-scope registry call; `MethodSlot::Defined` already carries a scope, and `.nil`
   items become `MethodSlot::Hidden`), rebuild its instance behaviour, `check_uninit(dummy)`.
3. send `NEW` to the dummy with `args[1..]`. User NEW, metaclass-instance NEW, primitive `NEW`
   arguments (R10), abstract (R11) and `.class` (R15) all follow from an ordinary send.
4. if the answer is a `Body::Instance` built from the dummy (its behaviour handle is the dummy's),
   set its `class` field to the receiver, mark it enhanced, and record the dummy in its
   `ObjectMethods` so the object keeps the dummy alive (a new traced `Option<ObjRef>` beside
   `enhanced_instance`). Otherwise hand it back untouched (R16 licence). A class answer (R15)
   needs the same reset on the class side (a reported-class override on the class definition); it
   is the documented example but no ooTest site uses it, so it may be refused loudly by name in 8b
   and done later.

`ObjectMethods::enhanced`, `ObjectMethods::enhance` and `ObjectMethodWrite::Enhance` are deleted:
the enhancing methods live in the dummy's behaviour, where `lookup_at`, `has_method_at`,
`method_names_at` and `instanceMethod` already look, which fixes R7's `instanceMethod` and
`instanceMethods()` rows. `instanceMethods(.nil)` (`introspection.rs:241`, `object_level_entries`)
must add the handle's `.nil`-scoped entries to the `setMethod` level.

**R9 is a separate one-line fix that 8b needs.** In `Interp::super_scope_in`
(`rust/crates/rexx-exec/src/dispatch.rs:2355`) a `.nil` scope answers the behaviour's owner
(`Behaviour::Instance { owner, .. }`), as `RexxBehaviour::superScope` does. Without it R5's
forwarding INIT and every `~m:super` from an enhancing or `setMethod` method raise 88.914.

### Costs

* **Per-send path.** Method lookup is unchanged (`lookup_at` on the object's handle, as today).
  `ObjectMethods::get` loses its second-layer search, so objects with own methods do slightly less work.
  The R9 fix adds one comparison to `Interp::super_scope_in`, called from method activation
  (`dispatch.rs:3194`-`3197`) on every Rexx method call. It sits beside a dictionary lookup that already
  runs there, but per the per-clause-branch record it must be measured (callgrind, rexxcps), not assumed.
* **Per-allocation path.** None. `new_instance`, `instance_over_store`, `array_of_class` and
  `native_new` are untouched; the dummy is an ordinary class to them. No check is added to them.
* **Class creation path** (`class_factory`, `install_class_at`): one extra message send (NEW) and a
  second INIT send per class. Directive-heavy startup pays this per `::CLASS`; it should be measured
  on the `startup` axis.
* **Class retention.** One class per `enhanced` call (the oracle makes one too), collectable through the 5j
  machinery once the object and its dummy die. Each leaves two emptied `Behaviour` slots behind for good.
  Each costs about 20 kB until collected, and the trigger does not see that weight (R17: 1.4 GB peak
  over 200000 `~subclass` calls). So **memory flat** across an enhanced loop needs the collection
  trigger to charge a class's dictionaries, which also fixes the existing `~subclass` peak. Charge it
  in `class_factory` (class creation), not on allocation.
* **Witnesses that change.** No oracle-compared corpus output should change; the code under them
  does: `lang/enhanced_scope.rex`, `enhanced_unset.rex`, `enhanced_constant_method.rex`,
  `enhanced_native_method.rex`, `context_executable_enhanced.rex`, `class_introspection.rex`,
  `method_source_shapes.rex`, `method_source_messages.rex`, `define_methods_supplier.rex` (the
  files `grep -rln -i "~enhanced(" rust/corpus` lists). New output appears where the crate
  diverged: R2, R3, R4, R6, R10, R11, R14 (the double INIT changes every program with a metaclass
  INIT), and `corpus/method-bodies.txt:71` (`Class enhanced ... rc 163`, the 93.902 of R10) should
  flip to `rc 0` once `NEW` takes the argument. The `factory_metaclass` refusal and
  its scout-D row (`class_protocol.rs:788`) go away.

### Rejected
* **Option A, a hook in `new_instance`** that copies pending enhancements onto the object: puts a check
  on every allocation and still cannot give R2, R3, R4 or R6 (there is no class for NEW to run on).
* **Option C, a cached dummy per receiver**: R2 counts one dummy per call, R3 runs class INIT per call,
  and the enhancing methods differ per call, so one cached behaviour cannot hold them.
* **Keep `class` = dummy and redirect `~class`/rendering readers**: every reader of an object's class
  (`isA`, `defaultName`, error inserts, `class_of_value`) would need the redirect; resetting the field
  once matches `setOwningClass` and touches no reader.
* **Keep the object-level `enhanced` layer and add the dummy only for NEW**: the object NEW builds
  must already answer to the enhancing INIT (R5) before `enhanced` regains control, so the methods
  have to be in the dummy's behaviour anyway; two homes for the same methods is the R7 `instanceMethod` bug.
