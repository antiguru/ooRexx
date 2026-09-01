# Phase 5b — an instance exists

**Status:** design.
**Entry:** met. Phase 5a is closed and gating: `CLOSED_PHASES` holds `"5a"`, and the gate tables
report `5a: 135 rows, 0 not yet agree` for table C and `5a: 36 rows, 0 not yet agree` for table D.
**Blocks:** 5c. The instance-side arm of 5c's method criterion cannot be measured until `~new`
exists, which D46 already states and this document does not re-argue.
**Decided:** 2026-08-27. Written against `538f35310`.
**Parent:** `docs/superpowers/specs/2026-08-17-phase-5-object-model.md`, which is binding. This
document adds to it and supersedes nothing in it. Decisions D57-D69 continue its numbering, plus
D59a, which the adversarial review forced out of D59 and which is numbered beside it because it is
D59's licence rather than a decision of its own.

## What this document adds to the Phase 5 spec

The parent spec cut Phase 5 into three phases along documented mechanisms and filed each mechanism
in an enumeration. 5b's cells in that enumeration are a list of mechanism names. What they are not
is a specification of the phase: they do not say what the mechanisms' observable rules are, they do
not say what the phase's gate is, and they do not settle the questions that only appear once you
try to build them.

Three of those questions turned out to have measured answers that a reader would not have guessed,
and all three are load-bearing:

* **An instance dispatches against the behaviour its class had when the instance was created**, and
  the two families of class mutator differ on whether an existing instance sees them. `~define`
  does not reach it; `~inherit` does. Measured below.
* **The oracle's ordering of `UNINIT` at process termination is not reproducible.** Twenty runs of
  a two-instance program gave two different orders. A gate row that depends on that order is a
  flaky row, and one such row already exists in shape if not in fact.
* **Both of gate table D's 5b rows are green today over a mechanism that is not implemented.**
  Measured: they install a `DELEGATE` directive and never send the delegated message.

And one question is settled here by decision rather than by measurement, because it is a design
choice this project is making rather than a property of ooRexx: **class objects are not collected.**
That is [its own section](#object-destruction-and-what-collection-is-here), because it is the one
place where this phase deliberately declines to reproduce the oracle. What that costs is wider than
`UNINIT`: `~subclasses` and `WeakReference` observe a class's collection directly, and a program that
answers on the oracle can be OOM-killed here. The licence names all four consequences rather than the
one that was noticed first.

**This document has been through one adversarial review, and the review changed it.** Six findings
were high: the sweep order was not creation order, the licence did not hold its stated bound in two
places, the evidence offered for the restricted-private check belonged to a different check, instance
`UNINIT` had no instrument at all, and a printed transcript asserted a racy value. Where a sentence
below says "an earlier draft said", that is the record of what the review found, kept rather than
quietly repaired.

## Authorities

Unchanged from the parent spec's list, in the same order: the documentation at `oodocs/`, then the
implementation at `/home/moritz/dev/repos/ooRexx/interpreter/`, then `ootest/`. The sections this
phase answers to:

* `rexxref/en-US/provide.xml` — `objcla` (Object Classes), `abscla` (Abstract Classes), `usesem`
  (Defining Instance Methods with SETMETHOD or ENHANCED), `creo` (Initialization), `obdes` (Object
  Destruction and Uninitialization), and `xmeths`'s first step. Each is a row of gate table C.
* `rexxref/en-US/fundclasses.xml` — `mthObjectNew`, `mthObjectInit`, `mthObjectCopy`,
  `mthObjectRun`, `mthObjectSend`, `mthObjectSendWith`, `mthObjectStart`, `mthObjectStartWith`,
  `mthObjectSetMethod`, `mthObjectUnsetMethod`, `mthClassEnhanced`.
* `rexxref/en-US/dire.xml` — `DELEGATE` on `::METHOD` and on `::ATTRIBUTE`.
* `rexxref/en-US/instrc.xml` `keyForward` — the `FORWARD` instruction.
* `rexxpg/en-US/classes.xml` `uninit` — "Uninitializing and Deleting Instances Using UNINIT".

**`UNINIT` has no `mth*` section anywhere in `fundclasses.xml`.** Searched by id across
`rexxref/en-US/*.xml`: the matches are `mthStreamUninit`, `mthEventSemaphoreUninit` and
`mthMutexSemaphoreUninit`, all per-class overrides, plus `rexxpg`'s `uninit` section and an
unrelated `condtra.xml` term id. So `UNINIT` is specified by `provide.xml` `obdes` and by the
Programming Guide, and it is not a member of Object's documented method set. That matters for 5c's
criterion, which is derived from the `mth*` sections, and it is recorded here rather than
discovered there.

## The mechanism set

One row per mechanism, with the authority that pins it and what it does **today**, measured at
`538f35310` on both engines against the oracle. "loud" means
`rexx_exec::NOT_IMPLEMENTED_EXIT` rc 120 with a `rexx-exec: ` line on stderr; "silent" means
matching exit status and stderr with differing stdout.

| mechanism | authority | today |
|---|---|---|
| `~new`: allocate, `checkAbstract`, snapshot the behaviour, register `UNINIT`, send `INIT` | `provide.xml` `creo`; `ClassClass.cpp:1882` `completeNewObject` | loud — `method "NEW" of class "Object" is not implemented (Phase 5)` |
| `INIT`, its arguments, and `self~init:super` chaining | `provide.xml` `creo`; `fundclasses.xml` `mthObjectInit` | unreachable behind `~new` |
| the behaviour snapshot: an instance keeps the methods its class had at creation | `provide.xml` `objcla`, `xmeths` step 2 | unreachable behind `~new` |
| abstract-**class** enforcement inside `~new` | `provide.xml` `abscla`; `checkAbstract` | installs; the check is unreachable |
| per-object methods: `setMethod`, `unsetMethod`, `enhanced`, and the scope they create | `provide.xml` `usesem`; `fundclasses.xml` `mthObjectSetMethod`, `mthObjectUnsetMethod`, `mthClassEnhanced` | loud |
| the per-object first step of the method search order | `provide.xml` `xmeths` step 1 | unreachable behind the above |
| the restricted-private check on `run`, `setMethod`, `unsetMethod` | the three `fundclasses.xml` sections' Notes; `ObjectClass.cpp:697` `checkRestrictedMethod` | unreachable |
| `FORWARD` | `instrc.xml` `keyForward`; `provide.xml` `creo` uses it to define multi-`INIT` | loud |
| `DELEGATE` on `::METHOD` and `::ATTRIBUTE`, defined as `expose` plus `forward to()` | `dire.xml` | installs; the delegated send is unreachable |
| `~copy` | `fundclasses.xml` `mthObjectCopy` | unreachable behind `~new` |
| `~run`, `~send`, `~sendWith`, `~start`, `~startWith` | `fundclasses.xml`, one section each | unreachable behind `~new` |
| `UNINIT` for an instance: its delivery from a collection **and** at termination (D69) | `provide.xml` `obdes`; `rexxpg` `uninit` | unreachable, and `obdes`'s probe has no instance in it |
| `UNINIT` for a class object | `provide.xml` `obdes`, which says "any object" | **silent** — see [the licence](#the-licence-and-its-bound) |
| `~subclasses` and `WeakReference` observing a class's collection | `fundclasses.xml` `mthClassSubclasses`; `utilityclasses.xml` `clsWeakReference` | loud today; **licensed to diverge** by D59a, and 5c owes the witnesses |
| the instance-side reading of every 5a limit that could only be measured on a class object | the parent spec, which calls it Task 7's debt | open |

Several of these carry a split the parent spec's enumeration already names and owns on both sides,
which is what D46 requires: `ABSTRACT` enforcement (the method arm 5a's, the class arm here),
`~enhanced` (5a builds the class-side machinery, 5b the instance it returns), `DELEGATE`, `~copy`,
the UNINIT propagation flags (carried by 5a's code, delivered here) and `~start` (5b builds the
Message object and `~result`; **Phase 6**, not 5c, replaces the scheduling). D46's own text adds
`xmeths`'s per-object first step. The directions differ and that matters: `ABSTRACT`'s other half is
already built in 5a, `~start`'s goes forward to Phase 6, and only the split
[named below](#the-native-new-split) -- which this document adds -- defers anything to 5c.

## Instance construction, in the order `completeNewObject` fixes

`RexxClass::completeNewObject` (`ClassClass.cpp:1882`) is four steps in a fixed order, and the order
is observable:

1. `checkAbstract()`
2. `obj->setBehaviour(getInstanceBehaviour())`
3. `if (hasUninitDefined()) obj->requiresUninit()`
4. `obj->sendMessage(INIT, initArgs, argCount, result)`

That the abstract check precedes `INIT` is visible: an abstract class with an `INIT` that prints
never prints. That `~new`'s frame is still on the traceback stack while `INIT` runs is visible in
the transcript of an argument error, measured:

```
say 'a'
o = .Object~new(1)
say 'b'

  oracle rc 163
  stdout  a
  stderr         *-* Compiled method "INIT" with scope "Object".
                 *-* Compiled method "NEW" with scope "Object".
               2 *-* o = .Object~new(1)
          Error 93 running …:  Incorrect call to method.
          Error 93.902:  Too many arguments in invocation of method; 0 expected.
```

Two frames, `INIT` inner and `NEW` outer, both named `Object`. 5a landed the native-send half of
that traceback line; this is the shape 5b's construction path must produce.

**Arguments to `~new` are arguments to `INIT`, and nothing chains them automatically.** Measured, a
subclass whose `INIT` does not send `self~init:super` leaves the superclass's `INIT` unrun and its
exposed variable uninitialised:

```rexx
o = .Sub~new
say 'a' o~report                    -- a b=[B]
::CLASS Base
::METHOD init
  expose b
  b = 'base-init'
  say 'base-init-ran'               -- never printed
::METHOD report
  expose b
  return 'b=[' || b || ']'
::CLASS Sub SUBCLASS Base
::METHOD init
  expose s
  s = 'sub-init'
  say 'sub-init-ran'
```

Oracle rc 0, stdout `sub-init-ran` then `a b=[B]`. `b=[B]` is the ordinary Rexx uninitialised-value
rendering, so the discriminator is not the missing print alone.

**An instance's default name follows the class.** Measured: `.Object~new~string` and its
`~defaultName` and `~objectName` are all `an Object`; an instance of a user class `K` is `a K`;
`~objectName =` overwrites what `~string` answers. 5a built `~objectName=`; 5b is where it first
has a non-class receiver.

## The behaviour snapshot, and the asymmetry between the two mutators

`provide.xml` `objcla` and `xmeths` step 2 both state the rule, wording its tail differently.
`xmeths` step 2: "an object acquires the instance methods of the class to which it belongs at the
time of its creation. If a class gains additional methods, objects created before the definition of
these methods do not acquire these methods." `objcla` ends the same sentence "do not acquire the new
or changed methods."

Measured, and the second and third lines are the ones a reader would not predict:

```rexx
k = .Object~subclass('K')
o = k~new
k~define('LATE', .methods~late)
say 'a' o~hasMethod('LATE')      -- a 0
say 'b' k~hasMethod('LATE')      -- b 0
p = k~new
say 'c' p~hasMethod('LATE')      -- c 1
signal on syntax name h
say 'd' o~late
h: say 'e trapped' rc            -- e trapped 97
exit 0
::METHOD late
  return 'late-ran'
```

Oracle rc 0. `b 0` is the class object being asked about its own class-side behaviour, which
`~define` does not touch. The floating `::METHOD late` is load-bearing: without it `.methods~late`
answers nothing and the program is 97.1 at rc 159 rather than the transcript above.

**`~inherit` behaves the opposite way, and this is the rule an implementation is most likely to get
wrong**, because the natural reading of "snapshot" makes both mutators invisible to an existing
instance:

```rexx
k = .Object~subclass('K')
o = k~new
k~inherit(.Mx)
say 'a' o~hasMethod('MXM')       -- a 1
say 'b' k~new~hasMethod('MXM')   -- b 1
say 'c' o~mxm                    -- c mixin-ran
::CLASS Mx MIXINCLASS Object
::METHOD mxm
  return 'mixin-ran'
```

Oracle rc 0.

The mechanism is in the C++ and its own comment states it. `RexxClass::defineMethod` at
`ClassClass.cpp:860`-`:862` reads

> `// make a copy of the instance behaviour so any previous objects aren't enhanced`
> `setField(instanceBehaviour, (RexxBehaviour *)instanceBehaviour->copy());`

`defineMethods` at `:530`-`:533` carries the same comment and the same call; `delete` at `:962`-`:964`
carries the call under a comment that says it a second way -- "we work on a copy of the instance
behaviour so that this changed does not suddenly show up in existing instances of this class."
`RexxClass::inherit` (`:1287`) has no such copy: it appends to `superClasses` and ends with
`updateSubClasses()` (`:1361`, defined at `:1036`), which clears and rebuilds **the existing
behaviour object** in place; `~uninherit` does the same at `:1413`. So an instance holding a pointer
to that object sees the rebuild. `updateInstanceSubClasses` (`:1071`) is the *copying* family's
helper, called on the fresh copy -- a branch `inherit` never takes.

This is therefore not a rule to be enforced by remembering it at each mutator. It is a property of
**where the behaviour an instance dispatches against lives**: the copying mutators must produce a
new one and leave existing holders on the old, and the rebuilding mutators must mutate the one
existing holders already point at. An implementation in which an instance names its class and the
lookup walks the registry live gets the `~inherit` case right and the `~define` case wrong, at rc 0,
with no refusal anywhere. Gate table C's `objcla` row already names exactly that mutation as its
negative control.

**The parent spec's enumeration row "`~define` copies the behaviour, `~inherit` mutates it in
place" is therefore a 5b requirement as well as a 5a one**, and 5a could not witness it: with no instances,
the two are indistinguishable.

## The object's own scope

`usesem` says the methods and object variables defined with `SETMETHOD` or `ENHANCED` "form a
separate scope, like the scopes the class hierarchy defines." `mthObjectSetMethod` adds a third
argument selecting `OBJECT` or `FLOAT` scope, `FLOAT` being the default, and describes `OBJECT` as
sharing "the scope with other, potentially statically defined, methods of the object it is attached
to."

Measured, which turns that sentence into a discriminator:

```rexx
o = .K~new
o~go
say 'class-sees' o~peek        -- class-sees class-v
say 'float-sees' o~oneoff      -- float-sees [V]
o~go2
say 'object-sees' o~oneoff2    -- object-sees [obj-v]
say 'class-sees2' o~peek       -- class-sees2 obj-v
::CLASS K
::METHOD init
  expose v
  v = 'class-v'
::METHOD peek
  expose v
  return v
::METHOD go
  self~setMethod('ONEOFF', 'expose v; return "["v"]"')
::METHOD go2
  self~setMethod('ONEOFF2', 'expose v; v = "obj-v"; return "["v"]"', 'OBJECT')
```

Oracle rc 0. `float-sees [V]` is a `FLOAT`-scope method exposing a `v` that is not the class's — it
renders as its own name, so the pool is genuinely a different one. `class-sees2 obj-v` is the class
method reading a write made by an `OBJECT`-scope method, so `OBJECT` scope shares the class's pool.
The crate already has the shape this needs: `Body::Instance(ScopePools)` is a list of pools keyed by
scope, and it is what a class object's own variables already use.

`unsetMethod` removes a `setMethod` definition and reveals the class's again, measured — `self~mm`
answers `object-mm` after `setMethod` and `class-mm` after `unsetMethod`, rc 0 — and `hasMethod`
answers `1` for the one-off in between.

**A one-off method does not reach the class's other instances**, which is `usesem`'s own claim and
what the existing probe tests.

**`~copy` copies the object's own scope with it.** Measured: a copy of an object carrying a
`setMethod` definition answers that method and reports `hasMethod` `1` for it, and the copy's
exposed variables have the receiver's values. `mthObjectCopy`'s note bounds it — the copy is
shallow, objects referenced by the target are not copied — and `(o == c)` is `0`, so the copy has
its own identity.

## The restricted-private trio

`run`, `setMethod` and `unsetMethod` each carry the same Note in `fundclasses.xml`: a private method
"with the additional restriction that it can only be called from an instance method of the receiving
object itself, or from a class method in the receiving object's inheritance chain", and each is also
protected.

`RexxObject::checkRestrictedMethod` (`ObjectClass.cpp:697`) is the check, and its own comment says
why it exists separately from `checkPrivate`: because these three are defined by Object, the ordinary
private rule "essentially means the only restriction ... is that it must be called from another
method", which was not the intent. The rule it implements, read from that function: take the top
stack frame's receiver; allow if it is this object; refuse if there is none, which is a routine or
program context; allow if it is a class object this object is an instance of.

**Two refusals are reachable and they are not the same one.** From a program context the ordinary
private-method check -- one of D53's three -- refuses first, at dispatch, before the native body runs:

```
o = .Object~new ; o~setMethod('MM', 'return 42')
  oracle rc 159
  stderr  Error 97.2:  Object "an Object" cannot accept private message "SETMETHOD" from this context.

o~run('expose v; return v*10')     on an instance of a user class
  oracle rc 159
  stderr  Error 97.2:  Object "a K" cannot accept private message "RUN" from this context.
```

`checkRestrictedMethod`'s own refusal is a different error at a different exit status, and reaching
it needs a caller that gets past the private check -- a class method of a class the object is **not**
an instance of:

```rexx
o = .K~new
say 'r' .Other~poke(o)
exit 0
::CLASS K
::CLASS Other
::METHOD poke CLASS
  use arg obj
  obj~setMethod('NN', 'return "x"')
  return 'ALLOWED'
```

```
oracle rc 158, stdout empty
stderr         *-* Compiled method "SETMETHOD" with scope "Object".
             8 *-* obj~setMethod('NN', 'return "x"')
             2 *-* say 'r' .Other~poke(o)
        Error 98 running ...:  Execution error.
        Error 98.991:  Method SETMETHOD may only be invoked from a method of the same object or one of its classes.
```

`RUN` gives the same shape with its own name. **The `*-* Compiled method "SETMETHOD" with scope
"Object".` frame is the tell**: the 97.2 transcript has no method frame at all, because that send
never entered the body. An earlier draft of this section offered the 97.2 as `checkRestrictedMethod`'s
evidence, which would have built a fourth check that answers the third check's error.

The allowing arm is measured too: a class method of a class the object *is* an instance of may send
`setMethod`, oracle rc 0.

`send` and `sendWith` are **not** in the trio and answer from a program context, measured: `o~send('M', 3)`
and `o~sendWith('M', .Array~of(4))` both return, rc 0.

This is a fourth access check beside D53's three, with its own refusal text and its own predicate.
5a's Task 5 made `resolve` the chokepoint that `PRIVATE`, `PACKAGE` and `PROTECTED` pass through;
this check reads the *caller's receiver* rather than the caller's scope or package, so it is a new
input to that chokepoint rather than a new value of an existing one.

## FORWARD, and DELEGATE

`FORWARD` is 5b's because `provide.xml` `creo` uses it to define multi-`INIT` chaining and because
`dire.xml` defines `DELEGATE` as its equivalent. Measured, a plain forward returns from the
forwarding method rather than continuing it:

```rexx
o = .Sub~new
say 'a' o~m(1,2)             -- sub-entered, then a base 1 2
::CLASS Base
::METHOD m
  use arg x, y
  return 'base' x y
::CLASS Sub SUBCLASS Base
::METHOD m
  use arg x, y
  say 'sub-entered'
  forward class (super)
```

`dire.xml` gives `DELEGATE` twice, once under `::METHOD` and once under `::ATTRIBUTE`, each time as
a stated equivalence rather than as a description:

```
::method name delegate delegateName
```
is equivalent to
```
::method name
  expose delegateName
  forward to(delegateName)
```

and the `::ATTRIBUTE` form generates the same pair for `name` and `name=`. The equivalence is the
specification, which is why `DELEGATE` is here with `FORWARD` and not with its directive.

## The alternative invocation paths

`mthObjectSend` and `mthObjectSendWith` both state that the message name may instead be an array
whose first item is the name and whose second is "a class object to use as the starting point for the
method search" — the same override `chsrod` gives as `~m:scope`, reached dynamically. 5a built the
static form. Documented, not measured here.

`~start` and `~startWith` answer a Message object. Measured:

```rexx
o = .K~new
m = o~start('M', 5)
say 'a' m~class~id      -- a Message
say 'b' m~completed     -- b 0 OR 1: NOT reproducible, 30/40 and 10/40 over forty runs
say 'c' m~result        -- c 6
say 'd' m~completed     -- d 1
say 'e' m~hasError      -- e 0
```

**`~completed` sampled before `~result` is the interleaving**, not a separate property, and the `b`
line is printed here only so that a plan reading this transcript does not turn it into a corpus row.
Everything else on it is reproducible.

**`~start`'s interleaving with the program that started it is not reproducible.** Measured, six runs
of a program whose started method prints and whose result is never asked for:

```
before after end inside-m       5 runs
before after inside-m end       1 run
```

So a gate row may test what `~result` forces and may not test when the body ran. That is the concrete
content of the parent spec's "minus their concurrency": Phase 6 replaces the scheduling, and 5b owes
the object, the result, and the completion flag.

## Object destruction, and what collection is here

### What the documentation asks for

`provide.xml` `obdes`: "Object destruction is implicit. When an object is no longer in use, Rexx
automatically reclaims its storage. ... If an object has more than one UNINIT method (defined in
several classes), each UNINIT method is responsible for sending the UNINIT method up the object
hierarchy." `rexxpg`'s `uninit` section adds the chaining spelling, `self~uninit:super`, and titles
itself "Uninitializing and Deleting **Instances**".

### What is already in the crate

`rexx-core`'s heap already implements the resurrection half. `Heap::collect` returns
`CollectStats::pending_uninit`, the unreachable objects whose `has_uninit` flag is set, which are
reported rather than swept so a finalizer does not see a half-collected graph; `Heap::set_uninit` is
the only writer of `true` and `Heap::clear_uninit_all` is how a caller reports the finalizer has run.
`Interp::collect` (`rexx-exec/src/lib.rs:6446`) carries the matching `debug_assert!` that the list is
empty, with a comment naming itself as the site that owes delivery on the day something sets the
flag. 5b is that day.

`rexx-classes` already carries 5a's half of the propagation. `ClassGraph::check_uninit` sets a
class's `has_uninit` when its flattened instance behaviour answers `UNINIT`; `parent_has_uninit`
propagates through `subclass`, `mixinClass` and `inherit`, matching the oracle's
`hasUninitDefined() || parentHasUninitDefined()` pair. Its doc comment already names what is missing
and why: the oracle's `checkUninit` goes on to `if (hasUninitMethod()) requiresUninit();`, which
enters the **class object itself** in the collector's uninit table, and "this crate has no such
table."

### The decision: class objects are not collected

**D59 below.** A class identity in this crate is not an arena object. It comes from
`rexx_core::CLASS_SLOT_BASE`, is `generation == 0` by construction, and names no slot the sweeper
can reach; `ClassRegistry::reserve_id` counts `next_id` upward and no path removes an entry from the
graph or from any of the registry's name tables. The collector has never seen a class and this
decision is that it never will.

**What it costs, measured.** Two hundred thousand `~subclass` calls, on this machine, at
`538f35310`:

```
do i = 1 to 200000 ; k = .Object~subclass('t') ; end ; say 'done' i

/usr/bin/time -v, oracle:  Maximum resident set size 111816 kB   elapsed 1:05.37
/usr/bin/time -v, crate:   Maximum resident set size 4086536 kB  elapsed 0:06.87
```

Both print `done 200001` at rc 0. The crate holds about thirty-six times the resident set and takes
about a tenth of the wall clock. The oracle's time on this program is largely the collection this
decision declines, and that is the trade being made rather than an incidental benefit.

**What it costs transitively, and the cost is behavioural as well as memory.**
`Interp::pool_owner` (`rexx-exec/src/run.rs:3086`-`:3095`) allocates a class object's variable pools
as an arena object and roots it through `RootSet::add_global`, whose entries are never removed. So
anything a class-scope instance variable holds is permanently live too. In 5a that could only be a
value; in 5b it can be an instance, and a class-side `EXPOSE` accumulating instances retains all of
them.

The memory half is obvious. The behavioural half is not, and it is the one that bites: a permanently
rooted instance is never unreachable, so it never reaches `CollectStats::pending_uninit`, so a
delivery driven by collection alone never runs its `UNINIT` at all. That would be an *absence*, not a
late delivery. **D69 is what closes it** -- the termination sweep runs over every live object with a
`UNINIT`, not only over the class registry -- and the oracle is the reason it has to, measured, rc 0:

```rexx
say 'start'
do i = 1 to 2
  o = .K~new(i)
end
drop o
call gc 'force'
say 'after-gc'
exit 0
::CLASS K
::METHOD init
  expose n
  use arg n
  .K~keep(self)
::METHOD keep CLASS
  expose bag
  use arg obj
  if var('bag') = 0 then bag = .Array~new
  bag~append(obj)
::METHOD uninit
  expose n
  say 'instance uninit' n
```

```
start / after-gc / instance uninit 1 / instance uninit 2
```

The forced collection reaches neither instance -- the class-scope `bag` holds both -- and both
`UNINIT`s fire at termination anyway. **That is a consequence of this decision, not of `~new`**, and
it is stated here so the plan owns it rather than discovers it.

**Why the alternative is not taken.** Collecting classes means class identities stop being a
monotone registry outside the arena, which is the shape 5a's whole class model is built on —
`ObjRef::class_id` asked before the arena on every send, `is_class_slot` as the receiver-kind
discriminator, the registry's name tables keyed by identity. The measured benefit is resident set on
a program that creates classes in bulk, which no benchmark axis has and which is not a shape real
Rexx programs take. The measured cost of the current arrangement is one observable divergence, below,
and it is narrow.

### The licence, and its bound

**A class object's `UNINIT` never fires from a collection.** Measured, and this is a silent
divergence -- matching exit status, empty stderr on both sides, differing stdout:

```rexx
say 'start'
k = .Object~subclass('K', .Meta)
say 'built' k~id
drop k
call gc 'force'
say 'after-gc'
exit 0
::CLASS Meta SUBCLASS Class
::METHOD uninit
  say 'CLASS UNINIT RAN'
```

```
oracle rc 0:  start / built K / CLASS UNINIT RAN / after-gc
crate  rc 0:  start / built K / after-gc
```

`provide.xml` `obdes` says "any object", and a class object is an object, so this is a divergence
from documented behaviour and is called one here rather than argued away.

**The `UNINIT` half of the licence is narrower than that transcript makes it look, and the
measurement that narrows it is the same program with the forced collection removed.** Five runs of
five, oracle rc 0:

```
say 'start' / k = .Object~subclass('K', .Meta) / say 'built' k~id / drop k / say 'dropped' / exit 0

oracle:  start / built K / dropped / CLASS UNINIT RAN
crate:   start / built K / dropped
```

So the oracle does **not** collect a dropped class promptly either. Left alone it defers the
`UNINIT` to termination, which is exactly where D60 puts it. For `UNINIT` the divergence is therefore
about *when* it runs, not *whether*: **every class object that defines a class-side `UNINIT` gets that
`UNINIT` run**, because under D59 no class object is ever discarded and under D60 the termination
sweep reaches all of them.

**But `UNINIT` is not the only observer of a collection, and the first draft of this section wrongly
said it was.** Two ordinary mechanisms read "this class object is gone" directly, and under D59 both
must answer the other way. Each is a silent wrong answer -- rc 0, empty stderr, differing stdout --
mentioning no `UNINIT` and defining no metaclass. Both reproduce three runs of three.

```rexx
say 'start' .Object~subclasses~items
k = .Object~subclass('K')
say 'after-create' .Object~subclasses~items
drop k
call gc 'force'
say 'after-drop' .Object~subclasses~items
```
```
oracle:  start 51 / after-create 52 / after-drop 51
D59:     start 51 / after-create 52 / after-drop 52
```

```rexx
k = .Object~subclass('K')
w = .WeakReference~new(k)
say 'before' (w~value == .nil)
drop k
call gc 'force'
say 'after' (w~value == .nil)
```
```
oracle:  before 0 / after 1
D59:     before 0 / after 0
```

`~subclasses` does not even need the forced collection: replacing it with
`do i = 1 to 200000; z = .Array~new; end` gives the same three answers, so ordinary allocation
reaches it. And `WeakReference`'s documented purpose *is* to report collection, so it is not an
exotic shape. Both are loud on the crate today, which is why they are named here rather than after
the methods land.

**The licence, stated over the observable rather than over `UNINIT`.** D59 makes "a class object was
collected" unobservable in this crate. Three consequences are licensed, and each owes a witness:

| observable | oracle | here | who owes the witness |
|---|---|---|---|
| a class's `UNINIT` after a driven collection | at the collection point | in the termination sweep | 5b |
| `~subclasses` after a class dies | drops it | keeps it | 5c, which lands `~subclasses` |
| a `WeakReference` to a dead class | `.nil` | the class | 5c, which lands `WeakReference~new` |

**A fourth observer of class collection is a new licence and needs its own decision, not this one.**

**And the memory consequence is a fourth divergence, not a case of an existing licence.** The first
draft rested it on this project's standing best-effort licence for memory pressure
(`docs/superpowers/plans/2026-08-17-phase-5a.md:273`-`:278`), whose own text is about *both sides
failing differently under a bound the harness imposes*. D59's shape is not that: one side answers and
the other dies, inside the project's own standard wrappers. Measured:

```
do i = 1 to 100000 ; k = .Object~subclass('t') ; end ; say 'done' i

oracle ( ulimit -v 1048576; ... )            rc 0    stdout  done 100001
crate  ( memcap 1G env REXX_ENGINE=ir ... )  rc 137  stdout empty
                                             stderr  memcap: OOM-killed at the 1G cap (peak 1.0G)
```

At about twenty kilobytes of resident set per class, that is the cost D59 buys the wall clock with.
It is licensed **explicitly here** rather than by reference: a program that answers on the oracle and
is killed here because class objects accumulate is an accepted divergence of this decision.

**The existing gate row stays gated, which is what D60 is for.**
`corpus/gate-tables/concepts/obdes.rex` is a class-side `UNINIT` on a directive-installed class and
drives no collection:

```rexx
say 'main'
::class k
::method uninit class
  say 'uninit ran'
```

Oracle rc 0, stdout `main` then `uninit ran`, five runs of five. Under D60 that row is reachable and
must agree. **The row is not re-authored and not re-filed to another phase** -- closing a gate row by
narrowing what the gate covers is a move this project has already declined once, and the decision
that makes the row passable is a better answer than the decision that makes it go away. Its committed
negative control does need an edit, and the plan owes it: `gate_table_c.rs`'s text is "stop running
`UNINIT` before the object's storage is reclaimed", and under D59 a class object's storage is never
reclaimed, so the runnable mutation becomes "skip the termination sweep".

**The termination sweep's order is NOT creation order, and an earlier draft of this section said it
was.** That claim was measured over two directive classes alike in everything but declaration order,
which is the one shape that cannot discriminate creation order from any other order agreeing with it
on that pair. A third class falsifies it. All four classes below are directive-installed, so creation
order is `Meta`, `M1`, `D1`, `M2`:

```rexx
say 'start'
exit 0
::CLASS Meta SUBCLASS Class
::METHOD uninit
  say 'uninit-meta' self~id
::CLASS M1 METACLASS Meta
::CLASS D1
::METHOD uninit class
  say 'uninit D1'
::CLASS M2 METACLASS Meta
```

Oracle, rc 0, empty stderr, three runs of three: `start` / `uninit-meta M1` / `uninit-meta M2` /
`uninit D1`. **`D1` is created before `M2` and fires after it.** So an implementation that did what
that draft said -- walk the class registry in creation order -- would emit a silent wrong answer, at
rc 0 with empty stderr, produced by following the spec.

What is established is weaker and is what D60 now says: the order is reproducible run to run, it
holds within a homogeneous group, and it is not creation order across groups. **Characterising it is
a task the plan owes**, and until it is characterised D61's prohibition covers class objects as well
as instances. `obdes.rex` is unaffected: it has exactly one class with a `UNINIT`.

### What is reproducible about UNINIT and what is not

Three measurements, and they do not agree with each other, which is the point.

**Termination, two instances: not reproducible.** Twenty runs of

```rexx
say 'start'
a = .S~new('one')
a = .S~new('two')
say 'exiting'
exit 0
::CLASS S
::METHOD init
  expose t
  use arg t
::METHOD uninit
  expose t
  say 'uninit' t
```

gave `uninit two` before `uninit one` sixteen times and the reverse four times, always rc 0. Note
also that overwriting `a` does **not** run the first object's `UNINIT` at that point: both fire at
termination.

**Termination, two class objects: reproducible.** Twenty runs of two directive classes each with a
`::METHOD uninit CLASS` gave declaration order, `uninit k1` then `uninit k2`, twenty times.

**A forced collection, one instance: reproducible.** Five runs of

```rexx
say 'a'
o = .S~new('one')
say 'b'
drop o
call gc 'force'
say 'c'
```

gave `a b uninit one c` five times.

So the delivery point is testable and the shutdown ordering across several instances is not. **D61**
turns that into a rule the row set has to obey.

## The gate

### The rows 5b owns today

From `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d`
at `538f35310`, whose phase summaries read

```
table C    5b: 6 rows, 6 not yet `agree`
table D    5b: 2 rows, 0 not yet `agree`
```

| table | row | verdict | loud |
|---|---|---|---|
| C | `objcla` | diverge-both | yes |
| C | `abscla` | diverge-both | yes |
| C | `usesem` | diverge-both | yes |
| C | `creo` | diverge-both | yes |
| C | `obdes` | **diverge-stdout** | **no** |
| C | `methodsbyclass` | diverge-both | yes |
| D | `::METHOD DELEGATE subkeyword` | **agree** | no |
| D | `::ATTRIBUTE DELEGATE subkeyword` | **agree** | no |

`obdes` is the only silent one, and it is the class-side `UNINIT` row the previous section settles.

The oracle transcripts of the table C probes 5b owns, recorded here so a later reader can tell a
row that changed from a row whose probe changed:

```
objcla          rc 0    before 0 / after 0 / fresh 1
abscla          rc 158  stdout  declared AB Class
                        stderr  *-* Compiled method "NEW" with scope "Object".
                                4 *-* say 'instance' .ab~new
                                Error 98 running …:  Execution error.
                                Error 98.989:  Class AB is ABSTRACT and cannot be directly created.
usesem          rc 0    setmethod one-off / not-shared 0 / enhanced enhanced / still-not-shared 0
creo            rc 0    type a savings account / balance 1000.00 / rate 6.25
obdes           rc 0    main / uninit ran
methodsbyclass  rc 0    element 0 / dimensions 2 / methods 1 1 / environment Array Directory StringTable
```

### The two rows that cannot fail

Both of table D's 5b rows are `agree` and neither exercises `DELEGATE`. The probes, in full:

```rexx
say 'main'                       say 'main'
::class k                        ::class k
::attribute d                    ::attribute d
::method m delegate d            ::attribute at delegate d
```

Each prints `main` at rc 0 on both sides because the directive installs and the delegated message is
never sent. Measured, a probe that does send it diverges:

```rexx
say 'main'
k = .K~new
say 'len' k~length
::CLASS K
::ATTRIBUTE d
::METHOD init
  expose d
  d = 'abcdef'
::METHOD length DELEGATE d
```

```
oracle rc 0:    main / len 6
crate  rc 120:  main    stderr  rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)
```

So the two rows are green over an unimplemented mechanism, and would stay green if `DELEGATE` were
never built. **D62** replaces them. Worth naming the shape, because it is not the one this project
has caught most often: not a criterion over an empty set and not a negative control that cannot
redden, but a witness whose program stops one send short of the thing it names.

Replacing them is not narrowing: the installation arm that the current probes test is kept, and the
send arm is added, so the row covers strictly more than before.

**Two constraints on the replacements, both found by running them.** First, `gate_table_d.rs` bounds
a non-refusing row's oracle stdout at exactly one line -- `expected_oracle_lines` is
`usize::from(!oracle_refuses(row))` -- so a two-line probe reports `unanswered` and can never reach
`agree`. Both arms fit on one line. Second, the `::ATTRIBUTE` form generates **two** methods, `name`
and `name=`, so a probe that sends only the getter leaves the setter arm green over an unbuilt one.

```rexx
say 'main' .K~new~length          -- main 6
::class k
::attribute d
::method init
  expose d
  d = 'abcdef'
::method m delegate d
::method length delegate d
```

```rexx
k = .K~new                        -- main via-set
k~at = 'via-set'
say 'main' k~peek~at
::class inner
::attribute at
::class k
::attribute d
::method peek
  expose d
  return d
::method init
  expose d
  d = .Inner~new
::attribute at delegate d
```

Both oracle rc 0, empty stderr. **The second one reads the inner object back rather than `k~at`, and
that is not a stylistic choice.** A draft of it said `say 'main' k~at`, which passes whether or not
`DELEGATE` does anything: with `::attribute at delegate d` replaced by a plain `::attribute at`, the
program still prints `main via-set`. Reading through `k~peek~at` separates them -- measured, the
plain-attribute control prints `main AT`, the uninitialised rendering. That is the third witness in
this document's own neighbourhood that stopped one send short of what it named, and the only reason
it is not the fourth defect shipped is that the control was run.

### The `methodsbyclass` row stays here

Its owning-phase authority in the table, quoted in full, is "no row in the spec enumeration; the
probe needs a multidimensional Array instance **for the section's own `matrix[2, 3] = 0` note**, and
the plan's handover puts instance construction in 5b". So the authority does name the mechanism; what
the *probe* lacks is the message-send arm. The section it names -- "Class Library Notes" -- reads
like 5c's subject, and the row stays 5b anyway for two reasons. Re-filing a row to a later phase to
get it out of a closing phase's way is the move Task 24 declined, and the reason it declined it
applies unchanged here. And the section's own testable claim is the equivalence it states -- that
`matrix[2, 3] = 0` and `matrix~"[]="(0, 2, 3)` are both valid and equivalent -- which needs the
instance either way. The row's committed negative control already names that half ("route
`matrix[2, 3] = 0` to a single-index `[]=`, so the element read back is not the one written"), so the
probe is the only piece missing. **D63** keeps the row and adds the equivalence arm, so the row tests
its section rather than merely requiring an instance to exist.

The consequence is that 5b owes multidimensional `Array` construction and indexing. `Body::Array` is
a flat `Vec<Option<ObjRef>>` today.

### The native `~new` split

`~new` for a native class is one mechanism and it is split, so under D46 the split is named here and
both halves are owned.

**5b owns the classes its own rows construct**: `Object` and every user class deriving from it
(`objcla`, `abscla`, `creo`, `usesem`, `obdes`), `StringTable` (`usesem`'s `enhanced` argument),
`Array` including the multidimensional form (`methodsbyclass`), and `Message` (`~start`'s answer).

**5c owns the rest**, as part of the documented per-class method sets it already owns. Their probes
are the `*__instance.rex` files under `corpus/gate-tables/methods/`, whose rows are reported under
`METHOD_PHASE`, which is `"5c"`. Checked at `538f35310` with `^o = \.[A-Za-z]+~new` over that
glob: every one of them opens with a bare construction, so none of them needs 5b to pass a
constructor argument.

**But "5b unblocks them" is false for a large minority of them**, and an earlier draft said it of all.
The probe writing a bare `~new` says nothing about whether the class accepts one -- most of those
probes say so in their own headers -- and measured on the oracle, twenty-three of the fifty-nine
classes refuse a bare `~new`: `93.901` for a missing required argument (Alarm,
CaselessColumnComparator, CircularQueue, Class, ColumnComparator, File, InvertingComparator, Message,
Stream, Ticker, TimeSpan), `88.901` (Method, Package, Routine), `93.903` (String, Supplier,
WeakReference), `93.967` (Pointer, RexxContext, StackFrame, VariableReference) and `97.1` (RexxInfo,
StreamSupplier). For those rows the blocker is 5c's own per-class `~new`, which must raise the
oracle's error from a native constructor; 5b changes nothing about them. **5b owes only the
constructions its own rows make**, and unblocks the remainder.

### What 5b owns that no gate-table row measures

**The rule is not "has no concept row"; it is "has no concept row whose probe exercises it."** An
earlier draft used the first, which is wrong in the direction that matters: instance `UNINIT` has a
concept row -- `obdes` -- and `obdes.rex` contains no instance, so every criterion in D65 could be
satisfied with instance `UNINIT` never delivered at all. Over `rust/corpus/`, the only `.rex` files
mentioning `uninit` are `obdes.rex` and three `*__instance.rex` method rows whose whole use of it is
`say 'instance' o~hasMethod("uninit")` -- a name, never a delivery, and 5c's rows besides.

So the mechanisms needing a witness in the ordinary differential corpus, which **D64** puts in
`corpus/phase-5b.txt` in the shape `phase-5a.txt` has, unioned by the same `read_subset`:

* `~copy`, `~run`, `~send`/`~sendWith`, `~start`/`~startWith`, and `FORWARD` as an instruction --
  no `provide.xml` section of their own; their table C rows are method rows, which are 5c's.
* **the restricted-private check, both refusals** -- the 97.2 the private check raises from a program
  context and `checkRestrictedMethod`'s own 98.991 at rc 158.
* **instance `UNINIT` delivery, twice**: the reproducible collection shape (`drop` then
  `call gc 'force'`, one instance, five runs of five) and a termination-delivery program with exactly
  one instance, so D61 is satisfied.
* **`~inherit` and `~uninherit` reaching an existing instance.** D58 calls this the arm an
  implementation is most likely to get wrong, and `objcla`'s committed control covers only the
  `~define` arm -- it is "rebuild an existing instance's method lookup from its class on every send",
  which is exactly the opposite outcome. Over `rust/corpus/`, no committed `.rex` combines `~inherit`
  with `~new`. An implementation that copied the behaviour on *both* families leaves every gate row
  green and fails D58's `~inherit` case at rc 0 with empty stderr.
* **`unsetMethod` and `setMethod`'s `OBJECT`/`FLOAT` third argument.** `provide.xml` `usesem` does not
  mention `UNSETMETHOD` at all and `usesem.rex` exercises neither, so by this section's own rule both
  belong here. D67 is the scope argument's entire subject and would otherwise have no witness.
* **the per-object first step's precedence**, by the program below, which carries `unsetMethod` with
  it.

**The per-object first step needs a witness of its own and does not have one.** `xmeths`'s probe
deliberately omits the arm and its row says so, and `usesem` is the row that nominally carries it --
but `usesem.rex` defines `EXTRA` only in the object's scope, with no class-level `EXTRA` to lose to,
so an implementation that searched the class first and the object second produces byte-identical
output for it. "First" is a precedence claim and the probe has nothing to order. This is the same
shape as the `DELEGATE` rows two sections up, found in the row this document had just called
sufficient. The discriminating program is one method longer, measured, oracle rc 0:

```rexx
o = .K~new
say '1' o~probe
::CLASS K
::METHOD mm
  return 'class-mm'
::METHOD probe
  r = 'before=' || self~mm
  self~setMethod('MM', 'return "object-mm"')
  r = r 'after=' || self~mm 'has=' || self~hasMethod('MM')
  self~unsetMethod('MM')
  r = r 'unset=' || self~mm
  return r
```
```
1 before=class-mm after=object-mm has=1 unset=class-mm
```

It carries `unsetMethod` and the precedence step together. **D64 takes it**, rather than extending
`usesem.rex`, so that the concept row keeps testing its own section's claim.

### The exit criterion

**D65.** 5b closes when, at one commit, with `REXX_CORPUS_GATE=1` and both engines:

1. every row whose owning phase is `5b` in either gate table is `agree`, with no structural failure
   in either table;
2. `corpus/phase-5b.txt` exists and every program in it agrees on all three descriptors on both
   engines;
3. D59a's licensed divergences that 5b owns have committed witnesses **the harness runs**, whose
   expected divergence is asserted -- as a numbered DEVIATION in the shape
   `docs/superpowers/plans/phase-4-exclusions.txt` already uses and which the parent spec's D41
   already cites, not as a prose file. A `.txt` nobody executes satisfies "committed" and rots
   silently;
4. `CLOSED_PHASES` gains `"5b"` and the gates pass with it there;
5. the five gates pass, with the command that printed each figure quoted beside the figure.

The instance-arm method rows of table C are **reported and not gated** by this, exactly as they are
today. They are 5c's criterion and 5b's landing is expected to move most of them; a plan that treats
their count as its own progress measure is measuring 5c.

## New decisions

* **D57. Phase 5b is the phase in which an instance exists**, with the mechanism set enumerated
  above. It takes nothing back from 5a, and it defers nothing of its own beyond the splits the parent
  spec's enumeration already names and the native-`~new` split named here.
* **D58. An instance dispatches against the behaviour its class held at construction, and the two
  families of class mutator differ on whether an existing instance sees a change.** `~define`,
  `~defineMethods` and `~delete` replace the class's instance behaviour with a copy, so existing
  instances keep the old one; `~inherit` and `~uninherit` rebuild the existing behaviour in place, so
  existing instances see the change. All three arms measured. The implementation must make this a
  property of where the behaviour lives, not a rule restated at each mutator: a live walk of the
  class graph on every send satisfies the `~inherit` case and silently fails the `~define` case at
  rc 0, and a copy on both families fails the `~inherit` case the same way.
* **D59. Class objects are not collected.** Class identities stay outside the arena, the registry
  stays monotone, and `Interp::class_variables` stays a permanent root -- so what a class-scope
  instance variable holds is permanently live too. Measured cost and measured benefit above. This is
  a decision about this implementation and it is expected to outlive Phase 5.
* **D59a. D59's licence is stated over the observable "a class object was collected", not over
  `UNINIT`.** Four consequences are licensed and no more: a class's `UNINIT` runs later than the
  oracle's; `~subclasses` keeps counting a dropped class; a `WeakReference` to a dropped class keeps
  answering it; and a program that answers on the oracle can be OOM-killed here because class objects
  accumulate, measured at 100,000 `~subclass` calls inside this project's own wrappers. Each of the
  first three owes a committed witness -- the first from 5b, the other two from 5c, which is the
  phase that lands `~subclasses` and `WeakReference~new`. **A fifth observer of class collection is a
  new licence and needs its own decision, not this one.**
* **D60. A class's `UNINIT` runs in a termination sweep and never from a collection.** Every class
  object that defines one gets one, so D59 costs a *later* `UNINIT` and not a missing one -- measured,
  the oracle also defers it to termination unless a collection is driven. That keeps
  `corpus/gate-tables/concepts/obdes.rex` reachable and gated. **The sweep's order is not creation
  order**: an earlier draft said it was, on a two-class measurement that could not discriminate, and a
  three-class program falsifies it three runs of three. What holds is that the oracle's order is
  reproducible and agrees with creation order within a homogeneous group. **Characterising it is a
  task this phase's plan owes**, and until it is characterised D61 covers class objects too.
* **D61. No row of any gate table, and no program in `corpus/phase-5b.txt`, may depend on the order
  in which `UNINIT` runs at termination** -- for instances, because the oracle does not reproduce it
  (two instances, two orders over twenty runs); for class objects, because D60 has not characterised
  it yet. A program needing more than one `UNINIT` to fire must force each with `drop` followed by
  `call gc 'force'`, which is reproducible five runs of five, or must not exist. `obdes.rex` is
  unaffected: it has exactly one class with a `UNINIT`.
* **D62. Gate table D's two `DELEGATE` rows are replaced by probes that send the delegated message**,
  in the two one-line shapes above -- one line because `expected_oracle_lines` bounds a non-refusing
  row's stdout at one, and for `::ATTRIBUTE` both generated methods, `name` and `name=`, because the
  directive generates the pair. The setter arm must be read back through the delegate object, not
  through the delegating one, or the probe passes with `DELEGATE` unimplemented; the control that
  shows this is recorded above and must be recorded as run.
* **D63. `methodsbyclass` stays a 5b row and gains the `[]`/`[]=` equivalence its section states.**
  5b therefore owes multidimensional `Array` construction and indexing.
* **D64. The mechanisms with no concept row *whose probe exercises them* get their witnesses in
  `corpus/phase-5b.txt`**, in the format and by the reader `phase-5a.txt` uses. The list is the one
  enumerated above, and it includes instance `UNINIT` delivery, the `~inherit`/`~uninherit` arms of
  D58, `unsetMethod`, `setMethod`'s scope argument and the per-object precedence step -- each of which
  a criterion built on "has a concept row" would have left unbuilt.
* **D65. 5b's exit criterion is the numbered list above.**
* **D66. The restricted-private check is a fourth access check, not a fourth value of D53's three, and
  it has its own error.** `checkRestrictedMethod` reads the *caller's receiver* -- allow if it is the
  object, refuse if there is none, allow if it is a class the object is an instance of -- where
  `PRIVATE`, `PACKAGE` and `PROTECTED` read the caller's scope, package and the security manager. It
  applies to `run`, `setMethod` and `unsetMethod` and not to `send` or `sendWith`. **Its refusal is
  `98.991 ... may only be invoked from a method of the same object or one of its classes.` at rc 158**,
  with a `Compiled method` frame on the traceback -- not the `97.2 ... cannot accept private message`
  at rc 159 that the private check raises first from a program context. Both are reachable and 5b owes
  both.
* **D67. `setMethod`'s third argument selects which variable pool the one-off method's `EXPOSE`
  reaches.** `FLOAT`, the default, is **one pool per object**, shared by all of that object's `FLOAT`
  one-offs and separate from the class's; `OBJECT` shares the class's. The per-object part is not
  determined by the single-method transcript above and is measured separately, with two `FLOAT`
  one-offs on one object reading each other's writes and a second instance seeing an uninitialised
  name. An implementation reading `FLOAT` as one pool per *method* is silently wrong at rc 0.
* **D68. `~start`'s interleaving is not a property any check may assert**, and `~completed` sampled
  before `~result` is part of that interleaving: measured, forty runs split thirty to ten. `~completed`
  *after* `~result` is `1` and may be asserted. 5b owes the Message object, `~result` and `~completed`;
  Phase 6 owes the scheduling.
* **D69. Every object with a live `UNINIT` gets it at termination, instance or class, whether or not
  it was ever unreachable.** The oracle does this, measured, including for instances a class-scope
  variable retains -- which under D59 is a shape the crate can produce and the collector can never
  reach. So the termination sweep runs over every live object carrying the flag, not only over the
  class registry, and a delivery driven by collection alone is not sufficient.

## Handover from 5a

Four items, each with a site.

1. **The `SELF` rooting hazard.** `Interp::pool_owner`'s doc comment (`rexx-exec/src/run.rs:3077`)
   records that a running send's receiver is rooted only by the `SELF` slot, which a method body may
   assign over, and names "the task that creates instances (`~new`)" as the one that can settle it.
   Today no `::METHOD` body reaches a non-class receiver, so nothing can hit it. 5b makes it
   reachable, and it is a use-after-free rather than a wrong answer. The instrument that finds a
   missed root is `run_program_collect_every_alloc`, which collects on every allocation.
2. **`Interp::collect`'s `debug_assert!`** (`rexx-exec/src/lib.rs:6446`) that `pending_uninit` is
   empty, with the comment naming itself as the delivery site. 5b replaces the assertion with the
   delivery.
3. **`receiver_kind`'s refusal** (`rexx-exec/src/dispatch.rs:1123`),
   `Body::Instance(_) => Err("an instance of a user class")`. That is the seam 5b opens, and the
   `Body::Instance(ScopePools)` behind it is the same shape a class object's variables already use.
4. **The instance-side reading of every 5a limit that could only be measured on a class object**,
   which the parent spec calls Task 7's recorded debt. It is an audit rather than a build: every
   refusal, error text and traceback 5a pinned with a class as the receiver now has a second receiver
   kind, and a limit that was measured on one and assumed for the other is a claim, not a
   measurement.

## Risks

| risk | consequence | response |
|---|---|---|
| the snapshot is implemented as a live walk of the class graph | `~define` after construction reaches an existing instance; rc 0, empty stderr, wrong stdout | D58 makes it a property of where the behaviour lives; `objcla`'s negative control is exactly this mutation and must be recorded as run |
| a `UNINIT` row, or a `~completed` row, is written from a racy transcript | the row passes and fails at random | D61 and D68, and the reproducible shapes are named with their measurements |
| D59's licence widens by drift | a further silent divergence arrives under cover of the named ones | D59a enumerates the four and says a fifth needs its own decision; each owed witness names its phase |
| the class-scope root makes an instance's `UNINIT` an *absence* rather than a late delivery | a silent wrong answer with no gate row anywhere near it | D69 puts the termination sweep over every live object carrying the flag, not over the class registry; D64 gives it a witness |
| the class UNINIT sweep is implemented in creation order, as an earlier draft of D60 said | a silent wrong answer produced by following the spec | D60 retracts it and D61 covers class objects until the order is characterised; the falsifying program is in the licence section |
| the class-scope root retains instances | a long-running program's resident set grows without bound where the oracle's does not | stated in D59 rather than left to be found; no ordinary path pays for it, and the shape that provokes it is a class-side `EXPOSE` accumulating instances |
| the instance-arm method rows move a lot when `~new` lands | 5b's progress is read off 5c's criterion | D65 gates 5b on 5b's rows and says explicitly that the method rows are reported, not gated |
| 5b's row set is small enough to close while a named mechanism is unbuilt | the phase closes over a gap | D64's corpus subset is the denominator for the mechanisms with no concept row, and the exit criterion names it |

## What I could not check

* **Whether a single instance's `UNINIT` at termination is reproducible.** The instability was
  measured with two instances. The single-object shapes that were re-run were a class object at
  termination and an instance under a forced collection, not an instance left to the termination
  sweep, so the phase where that instance's `UNINIT` lands relative to other output is untested.
* **`enhanced`'s argument requirements beyond a `StringTable` and a `Directory`.**
  `mthClassEnhanced` says any collection supporting `supplier`; only those two were run.
* **`FORWARD`'s full option surface.** `forward class (super)` and the bare `forward to()` shape in
  `dire.xml`'s `DELEGATE` equivalence were measured; `ARGUMENTS`, `ARRAY`, `MESSAGE`, `CONTINUE` and
  their interactions were not, and `instrc.xml` `keyForward` was located but not read in full.
* **What `~start` does with a method that raises**, and what `~hasError` and the Message error
  surface then answer. `hasError` was measured only on a clean result.
* **What rule the oracle's class-`UNINIT` termination order actually follows.** D60 records that it
  is reproducible and that it is not creation order, and no more. The two falsifying programs both
  fire metaclass-`UNINIT` classes before `::METHOD uninit CLASS` ones, and runtime-built classes
  before directive-installed ones, but two witnesses do not make a rule and the plan owes the
  characterisation.
* **Whether any documented mechanism outside this list becomes reachable once `~new` exists.** The
  parent spec's enumeration was the source for what 5b owns; a mechanism it filed under 5a or 5c and
  that turns out to need an instance would surface as a red row rather than as a gap in this
  document, which is the property the derived row sets exist for.
* **`.Message`'s own construction path.** `~start` answers one; whether `Message~new` is in 5b's
  scope was not settled, and its method rows are 5c's.
