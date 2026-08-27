# Adversarial review — `docs/superpowers/specs/2026-08-27-phase-5b-instances.md`

Subject at `be1cf430b`; the spec says "written against `538f35310`" and `git diff --stat 538f35310..HEAD`
is that spec file and nothing else, so every measurement below is against the tree the spec was written
against. All probes run from `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/specrev5b-probes/`
with the process CWD an empty directory, descriptors read separately, oracle under
`( ulimit -v 1048576; LD_LIBRARY_PATH=… timeout -s KILL 10 … )` and the crate under
`( memcap 1G env REXX_ENGINE=… timeout -s KILL 20 … )` on both engines. No `cargo` was run.

## Verdict

The measured half of this document is unusually solid: of the twenty-odd transcripts I re-ran, all but
three reproduce byte for byte, the 200k `~subclass` figures reproduce at the same factor, the two
instability claims (two-instance `UNINIT`, `~start` interleaving) reproduce, the class-object
reproducibility claim reproduces twenty of twenty, and every C++ and Rust citation but one is exact.
The argument built on top of it is weaker than the measurements are, and it fails in the two places the
document itself flags as load-bearing. **D60's "in creation order" is false** — it was measured on the
one shape that cannot discriminate, and a three-class all-directive program contradicts it ten runs of
ten (F1). **D59's licence does not hold its bound** — `.Object~subclasses~items` and `.WeakReference`
each observe "a class was collected" with no `UNINIT` anywhere in the program, so the divergence is not
"about *when* a class's `UNINIT` runs" (F2); and a class-scope `EXPOSE` turns an instance's `UNINIT`
into an *absence*, which line 475 says the licence does not cover, because no decision in the document
requires an instance termination sweep at all (F3). Separately, the one refusal D66 offers as its
measurement belongs to a different check: `checkRestrictedMethod` raises **98.991 at rc 158**, and the
97.2 the spec prints is the ordinary private-method refusal firing before it (F4). On the gate side the
document catches one criterion that cannot fail (the `DELEGATE` rows) and leaves at least four more:
instance `UNINIT`, the `~inherit` arm of D58, `unsetMethod` and `setMethod`'s scope argument, and the
per-object *precedence* step — each named in "The mechanism set" and each satisfiable while unbuilt.
Twenty-three findings: 6 high, 11 medium, 6 low. None of the document's decisions is unsalvageable; D60 needs
its rule replaced, D59 needs its bound restated and D66 needs its evidence replaced.

---

## F1 — D60's "creation order" is falsified; the measurement was over the one shape that cannot discriminate

**SEVERITY: high**

Line 496-500:

> **The termination sweep's order is a requirement, not an accident.** Measured, two directive classes
> each with a `::METHOD uninit CLASS` fire in declaration order on the oracle, twenty runs of twenty.
> So the sweep has to walk the class registry in creation order […]

and D60, line 713-719:

> **D60. A class's `UNINIT` runs in a termination sweep of the class registry, in creation order,
> and never from a collection.** […] Creation order is measured on the oracle, twenty runs of twenty,
> and is part of this decision rather than an implementation detail.

**What is wrong.** Two classes that are alike in every way except declaration order cannot separate
"creation order" from any other order that happens to agree with it on that pair. Add a third class
whose `UNINIT` arrives by a different route and the oracle does not use creation order.

**Evidence.** All three classes below are directive-installed, so creation order is the declaration
order `Meta`, `M1`, `D1`, `M2`:

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

Oracle, ten runs of ten, rc 0, empty stderr:

```
start | uninit-meta M1 | uninit-meta M2 | uninit D1
```

`D1` is created before `M2` and fires after it. A second witness, mixing directive and runtime classes,
ten of ten:

```rexx
say 'start'
r1 = .Object~subclass('R1', .Meta)
r2 = .Object~subclass('R2', .Meta)
say 'built'
exit 0
::CLASS Meta SUBCLASS Class
::METHOD uninit
  say 'uninit' self~id
::CLASS D1
::METHOD uninit class
  say 'uninit D1'
::CLASS D2
::METHOD uninit class
  say 'uninit D2'
```

```
start | built | uninit R1 | uninit R2 | uninit D1 | uninit D2
```

`D1` and `D2` are installed before the program's first clause; `R1` and `R2` are created by it; the
runtime pair fires first. I also re-ran the spec's own shapes and both reproduce: two `::METHOD uninit
CLASS` directive classes give `uninit k1` then `uninit k2` twenty of twenty, and two *runtime* classes
built through a metaclass give `uninit A` then `uninit B` twenty of twenty. So creation order holds
*within* a homogeneous group and not across groups.

An implementation that does what D60 says — "walk the class registry in creation order" — emits
`uninit D1, uninit D2, uninit R1, uninit R2` for the second program: rc 0, empty stderr, wrong stdout
order. A silent wrong answer, produced by following the spec.

**Smallest correct replacement.** Delete "in creation order" from D60 and from line 498, and replace the
sentence at 496-500 with what is actually established: *the order in which class `UNINIT`s fire at
termination is reproducible run to run but is not creation order — measured, a three-class program in
which a `::METHOD uninit CLASS` class declared second fires after a metaclass-`UNINIT` class declared
third, ten runs of ten. Until the rule is characterised, D61's prohibition covers class objects too, and
`corpus/gate-tables/concepts/obdes.rex` stays gated because it has exactly one class with a `UNINIT`.*
That keeps the row D60 was written to protect, and stops the document promising an order the oracle does
not use.

---

## F2 — D59's licence does not hold its bound: two programs observe "a class was collected" with no `UNINIT` in sight

**SEVERITY: high**

Line 468-473:

> So the oracle does **not** collect a dropped class promptly either. Left alone it defers the
> `UNINIT` to termination, which is exactly where D60 puts it. **The divergence D59 buys is therefore
> about *when* a class's `UNINIT` runs, not about *whether* it runs**, and it is observable only in a
> program that drives a collection […]

and line 475: "That is the whole licence".

**What is wrong.** `UNINIT` is not the only thing that can see a collection. Two ordinary mechanisms read
"this class object is gone" directly, and under D59 both must answer the other way.

**Evidence, escape 1 — `~subclasses`.** Oracle, three runs of three, rc 0, empty stderr:

```rexx
say 'start' .Object~subclasses~items
k = .Object~subclass('K')
say 'after-create' .Object~subclasses~items
drop k
call gc 'force'
say 'after-drop' .Object~subclasses~items
exit 0
```

```
start 51 | after-create 52 | after-drop 51
```

Under D59 — "`ClassRegistry::reserve_id` counts `next_id` upward and no path removes an entry from the
graph" (line 399, verified: `crates/rexx-classes/src/registry.rs` and `class_graph.rs` have no removal of
a class) — `after-drop` must answer 52. rc 0, empty stderr, differing stdout: a silent wrong answer that
mentions no `UNINIT` and defines no metaclass. It does not even need the forced collection; replacing
`call gc 'force'` with `do i = 1 to 200000; z = .Array~new; end` gives the same `51 / 52 / 51`, three
runs of three, so ordinary allocation reaches it.

**Evidence, escape 2 — `WeakReference`.** Oracle, three runs of three, rc 0, empty stderr:

```rexx
say 'start'
k = .Object~subclass('K')
w = .WeakReference~new(k)
say 'before' (w~value == .nil)
drop k
call gc 'force'
say 'after' (w~value == .nil)
exit 0
```

```
start | before 0 | after 1
```

Under D59 `after` must be `0`. `WeakReference`'s whole documented purpose is to report collection, so this
is not an exotic shape.

Both are loud on the crate today (`rexx-exec: method "SUBCLASSES" of class "Class" is not implemented
(Phase 5)`, `… "NEW" of class "WeakReference" …`), so nothing is red now — which is exactly why the
licence needs to name them before those methods land, rather than after. The Risks table's own row
("D59's licence widens by drift | a second silent divergence arrives under cover of the first") describes
a future event; these two exist at authoring time.

**Smallest correct replacement.** Replace lines 468-477 with a bound stated over the *observable* rather
than over `UNINIT`: *D59 makes "a class object was collected" unobservable in this crate. Three
consequences are known and licensed: a class's `UNINIT` runs later than the oracle's; `~subclasses`
keeps counting a dropped class; and a `WeakReference` to a dropped class keeps answering it. Each has a
committed witness. A fourth observer of class collection is a new licence and needs a decision, not this
one.* And since `~subclasses` and `WeakReference` are 5c method rows, say which phase owes their witnesses.

---

## F3 — a class-scope `EXPOSE` makes an instance's `UNINIT` *absent*, not late; and no decision requires an instance termination sweep at all

**SEVERITY: high**

Line 417-422:

> `Interp::pool_owner` […] allocates a class object's variable pools as an arena object and roots it
> through `RootSet::add_global`, whose entries are never removed. So anything a class-scope instance
> variable holds is permanently live too. In 5a that could only be a value; in 5b it can be an instance,
> and a class-side `EXPOSE` accumulating instances retains all of them.

against line 475:

> That is the whole licence, and it does not cover an absence.

**What is wrong.** The two sentences contradict each other, and the Risks row for the first
(line 773: "a long-running program's resident set grows without bound") describes only the memory half.
The behavioural half is that a permanently-rooted instance is never unreachable, so it never appears in
`CollectStats::pending_uninit`, and D60's sweep is over *the class registry*. Nothing in the document
delivers that instance's `UNINIT`. That is an absence.

Worse, the same hole exists without any class-scope `EXPOSE`: **no decision in this document says an
instance still live at termination gets its `UNINIT` at all.** D60 is about class objects; D61 regulates
the *ordering* of instance `UNINIT`s at termination without anything requiring the event; the mechanism
set's row is "`UNINIT` for an instance, and its delivery **from a collection**" (line 81). The oracle
fires them, measured.

**Evidence.** Oracle, rc 0, empty stderr — the instances are retained by a class-scope variable and the
forced collection does not reach them, but termination does:

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
start | after-gc | instance uninit 1 | instance uninit 2
```

And the plain case, the spec's own two-instance program (line 518-531), where `a` still holds the second
object at `exit 0`: both `UNINIT`s fire at termination, reproduced twenty runs of twenty here.

**Smallest correct replacement.** Add a decision beside D60: *every object with a live `UNINIT`, instance
or class, gets it at termination, whether or not it was ever unreachable — the oracle does this, measured,
including for instances a class-scope variable retains.* Then correct line 475 to say the licence covers
class objects' `UNINIT` timing and does not extend to instances, and correct the Risks row at 773 to name
the behavioural consequence as well as the resident set.

---

## F4 — D66's measured refusal is the wrong check's; `checkRestrictedMethod` raises 98.991 at rc 158

**SEVERITY: high**

Line 277-290:

> **Only the refusal arm was measured**, below; the two allowing arms are read from the C++.
>
> Measured from a program context:
> ```
> o = .Object~new ; o~setMethod('MM', 'return 42')
> oracle rc 159
> stderr Error 97.2: Object "an Object" cannot accept private message "SETMETHOD" from this context.
> ```

and line 295: "This is a fourth access check beside D53's three, **with its own refusal text and its own
predicate**."

**What is wrong.** The 97.2 transcript reproduces exactly — but it is not `checkRestrictedMethod`'s
refusal. It is the ordinary private-method refusal, one of D53's three, firing at dispatch before the
native body runs. `checkRestrictedMethod` is called from *inside* `setMethodRexx`/`unsetMethodRexx`/`run`
(`/home/moritz/dev/repos/ooRexx/interpreter/classes/ObjectClass.cpp:1876`, `:1896`, `:2241`) and both of
its refusing arms raise `Error_Execution_private_access`, which is `98991`
(`interpreter/messages/RexxErrorCodes.h:659`; text at `messages/RexxErrorMessages.h:661`). So the section
that argues this is a *fourth* check offers one of the *existing three* as its evidence, and an
implementation built from it will answer 97.2/rc 159 where the oracle answers 98.991/rc 158.

**Evidence.** The refusal `checkRestrictedMethod` actually produces — reached from a class method of a
class the object is *not* an instance of, which is the only way past the private check:

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
oracle rc 158
stdout  (empty)
stderr         *-* Compiled method "SETMETHOD" with scope "Object".
             8 *-* obj~setMethod('NN', 'return "x"')
             2 *-* say 'r' .Other~poke(o)
        Error 98 running …:  Execution error.
        Error 98.991:  Method SETMETHOD may only be invoked from a method of the same object or one of its classes.
```

Same for `RUN`: `Error 98.991: Method RUN may only be invoked from a method of the same object or one of
its classes.`, rc 158. The `*-* Compiled method "SETMETHOD" with scope "Object".` frame is the tell — in
the 97.2 transcript there is no method frame at all, because the send never entered the body.

**Also measured, closing the gap line 277 leaves open** — the allowing arm the spec read only from the
C++ does hold: a class method of a class the object *is* an instance of may send `setMethod`, oracle rc 0,
`from-class-method set-from-class-method`.

**Smallest correct replacement.** In the "restricted-private trio" section, keep the 97.2 transcript but
label it as the private-method check refusing first from a program context, and add the 98.991 transcript
above as `checkRestrictedMethod`'s own refusal. In D66, replace "It applies to `run`, `setMethod` and
`unsetMethod` and not to `send` or `sendWith`, measured" with the same plus "and its refusal is
`98.991 … may only be invoked from a method of the same object or one of its classes.` at rc 158, which is
not the 97.2 the private check raises from a program context; both are reachable and 5b owes both."

---

## F5 — instance `UNINIT` delivery has no instrument anywhere; D65 closes with it unbuilt

**SEVERITY: high**

Mechanism set, line 81:

> | `UNINIT` for an instance, and its delivery from a collection | `provide.xml` `obdes`; `rexxpg` `uninit` | unreachable |

and line 665-671:

> `~copy`, `~run`, `~send`/`~sendWith`, `~start`/`~startWith`, the restricted-private check, and
> `FORWARD` as an instruction have no `provide.xml` section of their own and therefore no table C
> concept row. […] **D64** puts them in `corpus/phase-5b.txt`

**What is wrong.** The rule "has a concept row ⇒ is instrumented" is the step that fails. Instance
`UNINIT` has a concept row — `obdes` — and `obdes`'s probe has no instance in it:

```rexx
say 'main'
::class k
::method uninit class
  say 'uninit ran'
```

(`rust/corpus/gate-tables/concepts/obdes.rex`, verbatim; the spec prints it itself at line 483-488.)
So every criterion in D65 can be satisfied with instance `UNINIT` never delivered: criterion 1 is
`obdes`, which is class-side; criterion 2 is `phase-5b.txt`, whose contents D64 enumerates without it;
criteria 3-5 are about the licence, `CLOSED_PHASES` and the five gates.

**Evidence.** `/bin/grep -rail uninit rust/corpus/` returns, among `.rex` files, only
`gate-tables/concepts/obdes.rex` and `gate-tables/methods/{stream,eventsemaphore,mutexsemaphore}__instance.rex`
— and all three of the latter contain exactly `say 'instance' o~hasMethod("uninit")`, never a delivery,
and are `METHOD_PHASE` = `"5c"` rows anyway (`crates/rexx-exec/tests/gate_table_c.rs:524`). No corpus
program under `lang/` mentions `UNINIT` as a method at all.

**Smallest correct replacement.** Add instance `UNINIT` to D64's list explicitly, with the reproducible
shape the document already established (line 540-551: `drop` then `call gc 'force'`, one instance, five
runs of five — I reproduce `a b uninit one c` five of five) plus a termination-delivery program with
exactly one instance so D61 is satisfied. Amend line 665 to say the rule is "no concept row **whose probe
exercises the mechanism**", since that is the distinction that does the work.

---

## F6 — the `~start` transcript asserts a value that is not reproducible

**SEVERITY: high**

Line 343-353:

> `~start` and `~startWith` answer a Message object. Measured:
> ```rexx
> o = .K~new
> m = o~start('M', 5)
> say 'a' m~class~id      -- a Message
> say 'b' m~completed     -- b 0
> …
> ```

and D68 (line 741-742), which exempts only the *interleaving*: "`~start`'s interleaving is not a property
any check may assert. […] 5b owes the Message object, `~result` and `~completed`."

**What is wrong.** `m~completed` sampled before `~result` *is* the interleaving. The spec's transcript is
the only measurement of `~completed` in the document, and it prints the racy value as if it were the
answer. D64 puts `~start`/`~startWith` in `corpus/phase-5b.txt`; a program written from this transcript
is a flaky corpus row, which is precisely what D61 exists to prevent.

**Evidence.** Forty runs of the spec's program, oracle, stdout only:

```
26  a Message|b 0|c 6|d 1|e 0|
14  a Message|b 1|c 6|d 1|e 0|
```

(The spec's other `~start` claim — the interleaving instability at line 355-361 — does reproduce: twenty
runs gave `before after end inside-m` nineteen times and `before after inside-m end` once.)

**Smallest correct replacement.** Drop the `say 'b' m~completed` line from the transcript, or annotate it
`-- b 0 or 1, not reproducible: 26/40 and 14/40 over forty runs`. Extend D68's sentence from "interleaving"
to "`~completed` sampled before `~result` is part of that interleaving and no row may assert it; `~completed`
after `~result` is `1` and may be."

---

## F7 — D58's `~inherit` and `~uninherit` arms — the ones the spec calls most likely to be got wrong — have no instrument

**SEVERITY: medium**

Line 172-174 and D58 (line 699-708):

> **`~inherit` behaves the opposite way, and this is the rule an implementation is most likely to get
> wrong** […]
> `~inherit` and `~uninherit` rebuild the existing behaviour in place, so existing instances see the change.

and the instrument the spec names, line 208-209:

> Gate table C's `objcla` row already names exactly that mutation as its negative control.

**What is wrong.** `objcla`'s control is *"rebuild an existing instance's method lookup from its class on
every send, so a method defined after the instance was created answers -- 5b"*
(`gate_table_c.rs:327`-`:328`) — verified, and it does cover the `~define` arm exactly. It cannot cover the
`~inherit` arm, because that arm requires the opposite outcome and `objcla.rex` contains no `~inherit`.
An implementation that copies the behaviour on *both* families leaves every gate row green and fails
D58's `~inherit` case at rc 0 with empty stderr.

**Evidence.** No committed program combines the two. Over `rust/corpus/`: the only `.rex` files matching
`~inherit|uninherit` are `gate-tables/concepts/usingcl.rex` (no `~new`),
`gate-tables/methods/class__instance.rex` (a 5c method row), and thirteen `lang/class_*` programs — and
`for f in lang/*.rex; do grep -q '~inherit' && grep -q '~new'; done` matches **none** of them. D64's list
does not include it.

**Smallest correct replacement.** Add to D64's list: a program carrying the spec's own `~inherit` probe
(line 176-186, which reproduces exactly: `a 1 / b 1 / c mixin-ran`, oracle rc 0) and the `~uninherit`
arm D58 asserts. I confirm the `~uninherit` arm: `k~inherit(.Mx)`, `o = k~new`, `k~uninherit(.Mx)` gives
`a 1 / b mixin-ran / c 0 / e trapped 97` at rc 0.

---

## F8 — "none of them needs a constructor argument" is false for 23 of 59, and "5b unblocks them" is false for the same 23

**SEVERITY: medium**

Line 660-663:

> Checked at `538f35310` with `^o = \.[A-Za-z]+~new` over that glob: every one of them opens with a bare
> construction, so `~new` is what blocks all of them and **none of them needs a constructor argument**.
> 5b unblocks them; it does not owe them.

**What is wrong.** The grep is correct and I reproduce it — all 59 `*__instance.rex` files match
`^o = \.[A-Za-z]+~new`, none fails to. The *inference* does not follow: the probe writing a bare `~new`
says nothing about whether the class accepts one. Most of these probes say so in their own headers
("no construction program is committed; a bare ~new raises 93.901 on the oracle").

**Evidence.** Every class named by those 59 probes, bare `~new`, on the oracle:

```
23 of 59 raise.  93.901 (missing required argument): Alarm, CaselessColumnComparator, CircularQueue,
Class, ColumnComparator, File, InvertingComparator, Message, Stream, Ticker, TimeSpan.
88.901: Method, Package, Routine.   93.903: String, Supplier, WeakReference.
93.967: Pointer, RexxContext, StackFrame, VariableReference.   97.1: RexxInfo, StreamSupplier.
The other 36 answer at rc 0.
```

So fourteen of them raise *because an argument is required*, and — more damaging to the split — for all
twenty-three, "5b unblocks them" is false. Once `~new` exists for `Object` and user classes, `.Stream~new`
still has to raise `93.901` from a native `Stream` constructor that 5c owns. Those rows are not unblocked
by 5b; they are unchanged by it.

**Smallest correct replacement.** Replace the sentence with: *Checked with `^o = \.[A-Za-z]+~new` over
that glob: every one opens with a bare construction, so none of them needs 5b to pass a constructor
argument. Twenty-three of the fifty-nine classes refuse a bare `~new` on the oracle — 93.901, 88.901,
93.903, 93.967 and 97.1 — so for those the row is unblocked by 5c's own per-class `~new`, not by 5b's.
5b owes only the constructions its own rows make.*

---

## F9 — `inherit` calls `updateSubClasses` (`:1036`, from `:1361`), not `updateInstanceSubClasses` (`:1071`)

**SEVERITY: medium**

Line 199-201:

> `RexxClass::inherit` (`:1287`) has no such copy: it appends to `superClasses` and calls
> `updateInstanceSubClasses` (`:1071`), which clears and rebuilds **the existing behaviour object** in
> place.

**What is wrong.** The mechanism conclusion is right; the function and the line are not. `RexxClass::inherit`
ends with `updateSubClasses();` at `classes/ClassClass.cpp:1361`. `updateInstanceSubClasses` (`:1071`) is
what `defineMethod` (`:867`), `defineMethods` (`:540`) and `deleteMethod` (`:970`) call — i.e. it is the
*copying* family's helper, called on the fresh copy. Citing it for `inherit` points a reader at the branch
`inherit` never takes.

**Evidence.**

```
/home/moritz/dev/repos/ooRexx/interpreter/classes/ClassClass.cpp
:1036  void  RexxClass::updateSubClasses()          // clears behaviour AND instanceBehaviour, rebuilds both
:1071  void RexxClass::updateInstanceSubClasses()   // clears and rebuilds instanceBehaviour only
:1361      updateSubClasses();                      // inside RexxClass::inherit
:1413      updateSubClasses();                      // inside RexxClass::uninherit
```

Both do rebuild `instanceBehaviour` in place, so D58 survives unchanged.

**Smallest correct replacement.** "…and calls `updateSubClasses` (`:1361`, defined at `:1036`), which
clears and rebuilds the existing behaviour object in place; `~uninherit` does the same at `:1413`."

---

## F10 — the `~define` snapshot probe, as printed, does not run

**SEVERITY: medium**

Line 156-169. The program is printed in full and followed by "Oracle rc 0." with `a 0 / b 0 / c 1 /
e trapped 97` as inline annotations. It has no `::METHOD late` directive, so `.methods~late` has nothing
to answer.

**Evidence.** The program exactly as printed:

```
oracle rc 159
stdout  (empty)
stderr       3 *-* k~define('LATE', .methods~late)
        Error 97 running …:  Object method not found.
        Error 97.1:  Object ".METHODS" does not understand message "LATE".
```

With `::METHOD late / return 'late-ran'` appended it gives exactly what the spec claims: `a 0 / b 0 /
c 1 / e trapped 97`, rc 0. So the *claim* is right and the *witness* is not runnable — which matters
because this is the probe D58's whole "where the behaviour lives" argument rests on, and because the
programs on either side of it (`~inherit` at 176-186, `setMethod` at 225-244) are complete and do run.

**Smallest correct replacement.** Append the two lines to the printed program.

---

## F11 — "Two of these carry a split" undercounts by at least four

**SEVERITY: medium**

Line 85-88:

> Two of these carry a split named in the parent spec's enumeration and owned on both sides, which is
> what D46 requires: `~enhanced` […] and `~start` […]. This document adds one further split, named below.

and D57 (line 697): "defers nothing of its own to 5c beyond the two splits named here and in the parent
spec's enumeration."

**What is wrong.** The parent spec's D46 (line 1233-1235 of
`docs/superpowers/specs/2026-08-17-phase-5-object-model.md`) enumerates its own two-phase cells:

> the enumeration carries two-phase cells for `ABSTRACT` enforcement, `~enhanced`, `DELEGATE`, `~copy`,
> environment search steps 3 and 5, `Package~local`, the UNINIT propagation flags, `PROTECTED`,
> `REPLY`/`GUARD` and `~start`.

Six of those ten appear in 5b's own mechanism set: `ABSTRACT` enforcement (5b's table row 4; the
abstract-*method* half is 5a's), `~enhanced`, `DELEGATE`, `~copy`, the UNINIT propagation flags (5b's rows
12-13, and the "already in the crate" section says so), and `~start`. A seventh is the `xmeths` per-object
first step, which D46 names in the sentence immediately before. So the count is six or seven, not two,
and D57's "beyond the two splits" is wrong for the same reason. D57 also conflates directions: `~start`'s
other half goes to Phase **6**, `ABSTRACT`'s to 5**a** (already built), and only the native-`~new` split
actually defers to 5c.

**Smallest correct replacement.** "Six of these carry a split already named in the parent spec's
enumeration — `ABSTRACT` enforcement, `~enhanced`, `DELEGATE`, `~copy`, the UNINIT propagation flags and
`~start` — plus `xmeths`'s per-object first step, which D46 names in its own text. This document adds one
further split, named below." And in D57: "defers nothing of its own beyond the splits the parent spec's
enumeration already names and the native-`~new` split named here."

---

## F12 — `unsetMethod` and `setMethod`'s scope argument have no witness, and D64's own rule puts `unsetMethod` in `phase-5b.txt`

**SEVERITY: medium**

Mechanism set, line 74:

> | per-object methods: `setMethod`, `unsetMethod`, `enhanced`, and the scope they create | `provide.xml` `usesem`; … | loud |

**What is wrong.** `usesem` is offered as the authority that instruments all four. `provide.xml`'s
`usesem` section does not mention `UNSETMETHOD` at all — it reads "The methods and the object variables
defined on an object with SETMETHOD or ENHANCED form a separate scope…" — and `usesem.rex` exercises
`setMethod`, `enhanced` and the not-shared property, and neither `unsetMethod` nor `setMethod`'s third
argument. So by D64's own rule ("no `provide.xml` section of their own and therefore no table C concept
row"), `unsetMethod` belongs in `corpus/phase-5b.txt` and is not there. `setMethod`'s scope argument is
D67's entire subject and has no witness either; its method row `mthObjectSetMethod` is 5c's and D65
explicitly does not gate method rows.

**Evidence.** `rust/corpus/gate-tables/concepts/usesem.rex` in full (10 lines of code): `o~addExtra`,
`o~extra`, `.k~new~hasMethod("EXTRA")`, `.stringtable~new`, `.k~enhanced(t)`, `e~enhanced`,
`.k~new~hasMethod("ENHANCED")`. `/bin/grep -c unsetMethod` over `rust/corpus/gate-tables/concepts/` is 0.
Both mechanisms measure cleanly and are ready to be witnesses: `unsetMethod` reveals the class method
again (`before=class-mm after=object-mm has=1 unset=class-mm`, rc 0), and the scope argument is the
spec's own transcript at 225-244, which reproduces exactly.

**Smallest correct replacement.** Add `unsetMethod` and `setMethod`'s `OBJECT`/`FLOAT` argument to D64's
list at line 667.

---

## F13 — the per-object *first* step is claimed covered by `usesem`, whose probe cannot discriminate it

**SEVERITY: medium**

Line 673-675:

> The per-object first step of the search order is **not** in that list: `xmeths`'s own probe
> deliberately omits that arm and its row says so, and `usesem` is the row that carries it. The step is
> covered, by a row that already exists.

**What is wrong.** "First" is a precedence claim: the object's own scope is searched *before* the class's.
`usesem.rex` defines `EXTRA` only in the object's scope — there is no class-level `EXTRA` to lose to. An
implementation that searched the class first and the object second produces byte-identical output for
that probe. This is the shape the spec itself names two paragraphs earlier as the one the project keeps
shipping: "a witness whose program stops one send short of the thing it names" (line 630-631).

**Evidence.** `usesem.rex` line 16-18 is `::class k` / `::method addExtra` / `self~setMethod("EXTRA", …)`;
`k` has no `EXTRA`. The discriminating shape exists and is one line longer — measured, oracle rc 0:

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

**Smallest correct replacement.** Either extend `usesem.rex` with a one-off that shadows a class method
of the same name, or move the step into D64's list. The sentence "The step is covered, by a row that
already exists" must not stand as written.

---

## F14 — D62's replacement probe breaks table D's one-line structural check

**SEVERITY: medium**

Line 611-626 shows the replacement shape, whose oracle stdout is two lines (`main` / `len 6`), and D62
(line 726-728) adopts it.

**What is wrong.** `gate_table_d.rs` bounds every non-refusing row's oracle output at exactly one line:
`fn expected_oracle_lines(row: &Row) -> usize { usize::from(!oracle_refuses(row)) }` (`:317`-`:319`), and
the module doc at `:46`-`:51` explains why. A two-line replacement reports the row as `unanswered`,
never as `agree` — so D62 as written cannot satisfy D65's criterion 1 without also amending
`expected_oracle_lines`, which the spec does not mention.

**Evidence.** Both arms fit on one line, measured, oracle rc 0, empty stderr:

```rexx
say 'main' .K~new~length            -- main 6
::class k
::attribute d
::method init
  expose d
  d = 'abcdef'
::method m delegate d
::method length delegate d
```

```rexx
k = .K~new                          -- main via-set
k~at = 'via-set'
say 'main' k~at
::class inner
::attribute at
::class k
::attribute d
::method init
  expose d
  d = .Inner~new
::attribute at delegate d
```

**Smallest correct replacement.** Use one-line probes as above, and say so in D62. The second one also
fixes F15.

---

## F15 — D62's `::ATTRIBUTE` replacement must send the setter, which `dire.xml` says the directive generates

**SEVERITY: medium**

D62, line 726-728: "Gate table D's two `DELEGATE` rows are replaced by probes that send the delegated
message. […] The installation arm is kept and the send arm is added."

**What is wrong.** The document itself records (line 333) that "the `::ATTRIBUTE` form generates the same
pair for `name` and `name=`", and `dire.xml:297`-`:310` shows both `::method name` and `::method "NAME="`
in the equivalence. The only replacement the spec prints is a `::METHOD DELEGATE` getter. A replacement
that sends only the getter leaves the `name=` half of the `::ATTRIBUTE` row green over an unbuilt arm —
the same defect D62 was written to fix, one level down.

**Evidence.** The setter arm works and is observable through the delegate, oracle rc 0:
`k~val = 'set-through-delegate'` then `k~val` and `k~inner~val` both answer `set-through-delegate`.

**Smallest correct replacement.** D62: "…the send arm is added, and for `::ATTRIBUTE` that means both
generated methods, `name` and `name=`, since the directive generates the pair."

---

## F16 — the memory licence is stretched past its own text, and the divergence is reachable under the project's standard wrapper

**SEVERITY: medium**

Line 507-510:

> The second-order divergence -- a program that exhausts memory here and not on the oracle because
> classes accumulate -- is already covered by this project's standing licence that matching the oracle
> under memory pressure is best-effort (`docs/superpowers/plans/2026-08-17-phase-5a.md:274`).

**What is wrong.** The cited licence is about *both sides failing differently under a bound the harness
imposes* — "Under the same `ulimit -v`, the oracle raises a clean Rexx error and this crate aborts, so
the two sides fail *differently*" (`2026-08-17-phase-5a.md:273`-`:278`). D59's consequence is a different
shape: **one side answers and the other dies.** That is not "failing differently"; it is a wrong answer
with a signal, and it is reachable inside the project's own standard wrapper.

**Evidence.** Under exactly the wrappers this project uses:

```
do i = 1 to 100000 ; k = .Object~subclass('t') ; end ; say 'done' i

oracle ( ulimit -v 1048576; … )     rc 0    stdout  done 100001
crate  ( memcap 1G env REXX_ENGINE=ir … )  rc 137  stdout  (empty)
       stderr  memcap: OOM-killed at the 1G cap (peak 1.0G)
```

50,000 still passes on the crate, so the boundary is between the two. The spec's own 200k figures
reproduce: oracle 116268 kB / 1:10.19 (spec: 111816 kB / 1:05.37), crate 4086940 kB / 0:06.71 (spec:
4086536 kB / 0:06.87) — about 20 kB of resident set per class.

**Smallest correct replacement.** Either widen the licence explicitly ("*and a program that answers on
the oracle and is OOM-killed here because classes accumulate; measured, 100,000 `~subclass` calls*"), or
name a class budget the crate stays inside. Do not leave it resting on a licence whose own text describes
the symmetric case.

---

## F17 — the `methodsbyclass` authority is quoted with the clause that carries the mechanism elided

**SEVERITY: medium**

Line 638-640:

> Its owning-phase authority in the table is "the probe needs a multidimensional Array instance", which
> is a fact about how the probe was written rather than about a mechanism […]

**What is wrong.** The committed authority (`gate_table_c.rs:503`-`:505`) reads:

> "no row in the spec enumeration; the probe needs a multidimensional Array instance **for the section's
> own `matrix[2, 3] = 0` note**, and the plan's handover puts instance construction in 5b"

The elided clause is precisely the mechanism reference the paragraph then says is missing, so the
quotation manufactures the gap the paragraph closes. D63 is still a real addition — the probe does
`matrix[2, 3] = 0` and `matrix[2, 3]` but never `matrix~"[]="(0, 2, 3)`, and `provide.xml:1006`-`:1033`
does state the equivalence ("even though the latter is valid and equivalent") — but the argument for it
should not rest on a truncated quote. Note also that the row's committed negative control already names
the equivalence half: *"route `matrix[2, 3] = 0` to a single-index `[]=`, so the element read back is not
the one written -- 5b"*.

**Smallest correct replacement.** Quote the authority in full and say that what it lacks is the
message-send arm, not a mechanism.

---

## F18 — the `~new(1)` traceback is printed against a program that does not produce it

**SEVERITY: low**

Line 104-113:

```
say 'a' ; o = .Object~new(1) ; say 'b'
  oracle rc 163
  …
               2 *-* o = .Object~new(1)
```

**Evidence.** The one-liner as printed gives `1 *-* o = .Object~new(1) ;` — line 1, and the oracle echoes
the trailing `;`. `Error 93 running … line 1:`. The three-line form
(`say 'a'` / `o = .Object~new(1)` / `say 'b'`) gives exactly the spec's rendering. Everything else in the
transcript — rc 163, the two frames in that order, 93.902 "Too many arguments in invocation of method;
0 expected." — reproduces byte for byte.

**Smallest correct replacement.** Print the three-line program.

---

## F19 — `Heap::set_uninit` is not the only writer of the flag

**SEVERITY: low**

Line 382: "`Heap::set_uninit` is the only writer of the flag and `Heap::clear_uninit` is how a caller
reports the finalizer has run."

**Evidence.** `crates/rexx-core/src/heap.rs:322` `object.has_uninit = true;` (in `set_uninit`),
`:346` `object.has_uninit = false;` (in `clear_uninit`), `:411` and `:434` `has_uninit: false` at
construction. The sentence contradicts its own second half.

**Smallest correct replacement.** "`Heap::set_uninit` is the only writer of `true`".

---

## F20 — one quotation attributed to two sections that word it differently

**SEVERITY: low**

Line 150-152 attributes a single sentence to both `objcla` and `xmeths` step 2.

**Evidence.** `provide.xml:91`-`:94` (`objcla`) ends "…do not acquire **the new or changed methods**.";
`provide.xml:502`-`:504` (`xmeths`) ends "…do not acquire **these methods**.)". The spec quotes the
`xmeths` wording for both.

**Smallest correct replacement.** "both state the rule; `xmeths` step 2 words it: '…'".

---

## F21 — D67's "a pool of its own" does not determine what the transcript shows

**SEVERITY: low**

D67, line 739-740: "`FLOAT`, the default, is a pool of its own; `OBJECT` shares the class's."

**What is wrong.** The transcript has one `FLOAT` method, so it cannot distinguish "one FLOAT pool per
object" from "one pool per FLOAT method". The doc says FLOAT "shares the same scope with methods that
were defined outside of a class" (`fundclasses.xml`, `mthObjectSetMethod`), which is the first. An
implementation reading D67 as the second is silently wrong at rc 0.

**Evidence.** Two `FLOAT` one-offs on one object share their pool, and the pool is per object — oracle
rc 0:

```
1 [float-set]     -- FA writes v
2 [float-set]     -- FB reads what FA wrote
3 class-v         -- the class's own v is untouched
4 [V]             -- a second instance's FB sees an uninitialised v
```

**Smallest correct replacement.** "`FLOAT`, the default, is one pool per object shared by all of that
object's `FLOAT` one-offs and separate from the class's; `OBJECT` shares the class's. Measured above, and
measured with two `FLOAT` methods."

---

## F22 — `obdes`'s committed negative control describes a reclaim that D59 abolishes

**SEVERITY: low**

`gate_table_c.rs:457`-`:459` holds the control *"stop running `UNINIT` before the object's storage is
reclaimed, which is **silent**…"*. Under D59 a class object's storage is never reclaimed, so the control's
own description names an event that does not occur; the runnable mutation becomes "skip the termination
sweep". The spec keeps the row (line 490-494, correctly) but does not notice that the control's text
stops matching.

**Smallest correct replacement.** A line in the licence section: *"`obdes`'s committed control text —
'stop running `UNINIT` before the object's storage is reclaimed' — becomes 'skip the termination sweep'
under D60, and the plan owes that edit."*

---

## F23 — D65 criterion 3's witness has no instrument, and the project already has a stronger mechanism for this

**SEVERITY: low**

D65 criterion 3 (line 685-686): "the licensed class-object-`UNINIT` divergence has its committed witness,
and that witness is in no subset file". Line 503-505 models it on `corpus/oracle-crashes.txt`.

**What is wrong.** `oracle-crashes.txt` is prose because its programs *must not be run*; this one can be,
and is rc 0 on both sides. As proposed, the criterion is satisfied by committing a file with any content
— nothing re-runs the transcripts, so the witness rots silently, and "in no subset file" is trivially
true for a `.txt`. The project already has a stronger vocabulary for exactly this: numbered DEVIATIONs in
`docs/superpowers/plans/phase-4-exclusions.txt`, whose header says "the gate asserts the SET of both
sections rather than a count", each row carrying IMPLEMENTED / SCOPE / WHY and pinned witnesses, and
which the parent spec already uses ("licensed as deviation 4", D41).

**Smallest correct replacement.** Make the licence a numbered DEVIATION in the existing file (or its
Phase 5 successor), and make criterion 3 read "…and its witness is a committed program the harness runs,
whose expected divergence is asserted".

---

## Asserted about the world without being checked — every one run

| line | claim | verdict |
|---|---|---|
| 53-56 | `UNINIT` has no `mth*` section in `fundclasses.xml`; the id matches are `mthStreamUninit`, `mthEventSemaphoreUninit`, `mthMutexSemaphoreUninit`, `rexxpg`'s `uninit`, and an unrelated `condtra.xml` term | **HOLDS, exactly.** `grep -oiE 'id="[^"]*uninit[^"]*"'` over `oodocs/rexxref/en-US/*.xml` gives those four and nothing else; `condtra.xml:289` is `<term id="uninit">NOVALUE`, unrelated as stated; `rexxpg/en-US/classes.xml` has the fifth |
| 4-5 | `CLOSED_PHASES` holds `"5a"` | **HOLDS.** `gate_tables/mod.rs:344` |
| 564-565, 568-577 | table C `5b: 6 rows`; table D `5b: 2 rows`; the eight row verdicts | **HOLDS.** Six `phase: "5b"` in `gate_table_c.rs` (`:323, :342, :387, :445, :454, :502`); `owning_phase` maps `("::METHOD"\|"::ATTRIBUTE", "DELEGATE")` to `"5b"` and exactly two probe files carry that pair. I also re-derived table C's `5a: 135` as 57 hierarchy + 63 class probes + 15 5a concept rows, and table D's `5a: 36` as 39 `annotate/attribute/class/method` probes minus `::CLASS CLASS` and the two `DELEGATE` rows |
| 382 | `Heap::set_uninit` is the only writer of the flag | **FAILS as written** — see F19 |
| 399-401 | no path removes an entry from the graph or the registry's name tables; the collector has never seen a class | **HOLDS for class identities.** `registry.rs` has no `remove`/`clear`/`retain`/`pop` at all; `class_graph.rs` never removes from `self.classes`. Two `remove` calls exist (`:841`, `:850`) and take entries out of a class's *superclass* and *subclass* lists, not out of the graph — worth the word "class" in the sentence |
| 419 | `RootSet::add_global`'s entries are never removed | **HOLDS.** `roots.rs:193`-`:196` push or replace in place; `globals` is touched at `:52, :132, :194, :196, :572, :595` and nowhere else |
| 579 | `obdes` is the only silent one of the eight | **HOLDS.** Re-ran all eight: `objcla`, `usesem`, `creo`, `methodsbyclass` differ on status, stdout and stderr; `abscla` differs on status and stderr with stdout matching (`declared AB Class` both sides) — all `diverge-both`; `obdes` is rc 0/rc 0, stderr empty both sides, stdout `main\nuninit ran` vs `main` — `diverge-stdout`; the two D rows `agree` |
| 662-663 | every instance probe opens with a bare construction / none needs a constructor argument | **first half HOLDS** (59 of 59 match `^o = \.[A-Za-z]+~new`, 0 fail); **second half FAILS** — see F8 |
| 475-477 | every class object defining a class-side `UNINIT` gets one | **HOLDS under D59+D60** by construction, but it does not extend to instances — see F3 |
| 85 | "Two of these carry a split" | **FAILS** — see F11 |
| 697 | "supersedes nothing"; "takes nothing back from 5a" | **HOLDS for D1-D56**: the parent spec contains no decision about collecting classes or about class lifetime; the only friction is line 211-213 making a 5a enumeration row (`:130`, marked "built (Task 2)") also a 5b requirement, which the document states openly |
| 414-415 | "The oracle's time on this program is largely the collection this decision declines" | **SUPPORTED.** 50,000 `~subclass` dropped is 2.80 s / 46 MB; the same 50,000 *retained in an array* is 1.55 s / 870 MB — retaining is faster; and 50,000→200,000 takes 2.80 s→70 s, four times the work for twenty-five times the time. 50,000 `.Array~new` is 0.01 s, so construction is not the cost |
| 751 | "Today no `::METHOD` body reaches a non-class receiver, so nothing can hit it" | **not falsified, not proved.** Every route I tried is loud first (`~new`, `~run`, `setMethod`, `.StringTable~new` for `~enhanced`). I could not enumerate the negative |

---

## Verified and found CORRECT

Re-run from my own probe directory, byte for byte against what the spec prints, unless noted:

* **Line 118-141, `INIT` chaining.** `sub-init-ran` then `a b=[B]`, rc 0. Exact.
* **Line 143-146, default names.** `.Object~new`'s `~string`/`~defaultName`/`~objectName` all `an Object`;
  a user class instance is `a K`; after `k~objectName = 'zed'`, `~string` and `~objectName` answer `zed`
  and `~defaultName` still answers `a K`. Exact, and the last point is more than the spec claims.
* **Line 176-188, `~inherit` reaches an existing instance.** `a 1 / b 1 / c mixin-ran`, rc 0. Exact.
* **Line 190-201, the C++ mechanism.** `ClassClass.cpp:860`-`:862` carries the comment and the
  `instanceBehaviour->copy()`; `:531`-`:533` the same in `defineMethods`; `:962`-`:964` the same in
  `deleteMethod` under "we work on a copy of the instance behaviour so that this changed does not
  suddenly show up in existing instances of this class."; `RexxClass::inherit` is at `:1287` and has no
  copy. Only the name and line of the rebuild helper are wrong (F9).
* **Line 225-248, `FLOAT` vs `OBJECT` scope.** `class-sees class-v / float-sees [V] / object-sees
  [obj-v] / class-sees2 obj-v`, rc 0. Exact.
* **Line 252-254, `unsetMethod`.** `before=class-mm after=object-mm has=1 unset=class-mm`, rc 0.
* **Line 259-263, `~copy`.** The copy answers the one-off, `hasMethod` 1, exposed variables carry the
  receiver's values, `(o == c)` is 0. All four.
* **Line 265-278, `ObjectClass.cpp:697`.** `checkRestrictedMethod` is at exactly that line; its comment
  says what the spec quotes; the predicate the spec reads out of it — top frame's receiver, allow if it
  is this object, refuse if none, allow if it is a class this object is an instance of — is accurate.
  Only the refusal *number* is wrong (F4).
* **Line 282-293.** All three transcripts reproduce: 97.2/rc 159 for `setMethod` and `run` from a program
  context (and `unsetMethod` likewise), `o~send('M', 3)` → 4 and `o~sendWith('M', .Array~of(4))` → 5 at
  rc 0.
* **Line 90-116, `completeNewObject`.** `ClassClass.cpp:1882` is the function; the four steps are in the
  order given. The abstract check preceding `INIT` is visible: an `ABSTRACT` class with an `INIT` that
  prints gives `98.989 Class AB is ABSTRACT and cannot be directly created.` at rc 158 and never prints.
* **Line 300-318, `FORWARD`.** `sub-entered` then `a base 1 2`, rc 0. Exact.
* **Line 320-334, `dire.xml`.** Both equivalences are at `dire.xml:297`-`:310` (`::ATTRIBUTE`, generating
  `name` and `"NAME="`) and `:732`-`:741` (`::METHOD`), worded as the spec renders them.
* **Line 336-341, `mthObjectSend`.** "If *messagename* is an array object, its first item is the name of
  the message and its second item is a class object to use as the starting point for the method search."
  Verbatim.
* **Line 355-361, `~start` interleaving instability.** Reproduced: 19 `before after end inside-m` and
  1 `before after inside-m end` over twenty runs (spec: 5 and 1 over six).
* **Line 377-393, the crate's half.** `Heap::collect`/`CollectStats::pending_uninit`/`set_uninit`/
  `clear_uninit` all exist as described; `Interp::collect`'s `debug_assert!` is at
  `rexx-exec/src/lib.rs:6446` with the comment at `:6439`-`:6445` naming itself as the delivery site;
  `ClassGraph::check_uninit`'s doc (`class_graph.rs:371`-`:385`) says exactly what the spec quotes,
  including "This crate has no such table."
* **Line 397-401.** `rexx_core::CLASS_SLOT_BASE` is `handle.rs:88`; `is_class_slot` is
  `generation == 0 && slot >= CLASS_SLOT_BASE` (`:104`); `ClassRegistry::reserve_id` is `registry.rs:86`
  and increments `next_id`.
* **Line 403-415, the 200k figures.** Oracle 116268 kB / 1:10.19 against the spec's 111816 kB / 1:05.37;
  crate 4086940 kB / 0:06.71 against 4086536 kB / 0:06.87. Both print `done 200001` at rc 0. The stated
  ratios ("about thirty-six times", "about a tenth") hold at 35.2× and 9.6×.
* **Line 417, `pool_owner`.** `run.rs:3081` is the function; the allocation is `:3088` and the
  `add_global` `:3093`-`:3094`, so the cited `:3086`-`:3095` starts one line early on the cache-hit
  return but brackets both. The doc comment at `:3077` says what handover item 1 quotes.
* **Line 434-453, the licensed divergence.** Exact, both sides: oracle `start / built K / CLASS UNINIT
  RAN / after-gc`, crate `start / built K / after-gc`, rc 0 on both, stderr empty on both.
* **Line 458-466, the no-forced-collection variant.** `start / built K / dropped / CLASS UNINIT RAN` on
  the oracle, five runs of five; `start / built K / dropped` on the crate. This is the load-bearing
  measurement behind D60 and it holds.
* **Line 480-494, `obdes.rex`.** The committed file is exactly as printed; oracle `main` + `uninit ran`
  five runs of five; crate `main` only, rc 0, stderr empty.
* **Line 516-535, two-instance instability.** 15 `uninit two`-first and 5 `uninit one`-first over twenty
  (spec: 16 and 4). The note that overwriting `a` does not fire the first `UNINIT` at that point holds —
  both lines follow `exiting`.
* **Line 537-538, two class objects reproducible.** `uninit k1` then `uninit k2`, twenty of twenty. (The
  rule it is used to induce is what fails — F1.)
* **Line 540-551, forced collection, one instance.** `a b uninit one c`, five of five.
* **Line 581-595, the five other table C oracle transcripts.** All five reproduce byte for byte,
  including `abscla`'s full traceback and `methodsbyclass`'s `element 0 / dimensions 2 / methods 1 1 /
  environment Array Directory StringTable`.
* **Line 597-626, the two `DELEGATE` rows.** Both probes are exactly as printed and both `agree`
  (`main`, rc 0, empty stderr, on the oracle and on both engines). The sending probe diverges exactly as
  printed: oracle `main / len 6` rc 0; crate rc 120 with `rexx-exec: method "NEW" of class "Object" is
  not implemented (Phase 5)` on both engines.
* **Line 643-644, the `methodsbyclass` equivalence.** `provide.xml:1006` is titled "Class Library Notes"
  and its second note states `matrix[2, 3] = 0` "rather than" `matrix~"[]="(0, 2, 3)` "even though the
  latter is valid and equivalent."
* **Line 648, `Body::Array`.** `Array(Vec<Option<ObjRef>>)` at `rexx-core/src/body.rs:119`.
* **Line 657-659, `METHOD_PHASE`.** `"5c"` at `gate_table_c.rs:524`.
* **Line 670-671, D64's shape.** `read_subset` and `SUBSET_FILES` exist as described
  (`tests/corpus.rs:215`, `:653`), and `the_differential_reads_every_phase_subset_file` (`:688`) makes a
  new `corpus/phase-5b.txt` red until it is wired in — a real control for criterion 2.
* **Line 703-706, D58's `~uninherit` arm.** `a 1 / b mixin-ran / c 0 / e trapped 97`, rc 0.
* **Line 734-738, D66's negative half.** `send` and `sendWith` carry no such check; rc 0 from a program
  context.
* **Line 754-759, handover items 2 and 3.** `lib.rs:6446` and `dispatch.rs:1123`
  (`Body::Instance(_) => Err("an instance of a user class")`) are both exact; `Body::Instance(ScopePools)`
  is `body.rs:122`.
* **Line 509, the cited licence.** `2026-08-17-phase-5a.md:273`-`:278` says what the spec cites it for,
  for the case it was written about (F16 is about the case it was not).
* **`NOT_IMPLEMENTED_EXIT`** is `120` (`rexx-exec/src/lib.rs:161`), matching every "loud" transcript.

Two extra measurements the document does not have, offered because they strengthen its decisions:
`~subclass`-created classes fire in creation order among themselves, twenty of twenty; and the
restricted-private *allowing* arm the spec read only from the C++ is real — a class method of a class the
object is an instance of may send `setMethod`, oracle rc 0.

---

## What I could not check

* **"0 not yet `agree`" for 5a in either table, and the `5b: 6 rows, 6 not yet agree` / `5b: 2 rows, 0
  not yet agree` summaries as printed output.** These come from
  `REXX_CORPUS_GATE=1 cargo test …`, and I was told not to run cargo. I verified the row *counts*
  structurally (see the table above) and re-derived every 5b row's verdict by running its probe on both
  sides by hand; the "not yet agree" tallies follow from those verdicts but I did not see the harness
  print them.
* **The negative controls, as mutations.** Each 5b row's control is a source mutation; running one means
  editing and rebuilding. I checked each control's *text* against its probe and can say whether the probe
  could see it (F13 is the one that cannot), but I did not execute any.
* **Line 751's negative** — "no `::METHOD` body reaches a non-class receiver today". Every route I tried
  is refused earlier, but the claim is a universal over the whole reachable surface and I could not
  enumerate it.
* **Whether `~enhanced`'s class-side half is built in 5a.** `usesem`'s shape is blocked at
  `.StringTable~new` (`rexx-exec: method "NEW" of class "StringTable" is not implemented (Phase 5)`), so
  I could not separate the two halves of that split from the outside.
* **The mechanism behind F1.** I report the observable — the order is reproducible and is not creation
  order — and deliberately do not name a site in the C++ for it; that would need reading the uninit
  table's iteration and I did not.
* **`.Message`'s construction path**, which the spec also lists as unchecked. `.Message~new` raises
  93.901 on a bare send, so 5b's `Message` obligation is whatever `~start` answers and nothing more; the
  document's split section (line 657) and its last "could not check" bullet (line 794-795) read
  differently about this and I did not resolve which is intended.
