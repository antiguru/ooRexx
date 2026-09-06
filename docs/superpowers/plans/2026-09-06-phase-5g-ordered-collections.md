# Phase 5g — the ordered collections

Spec: `docs/superpowers/specs/2026-09-06-collections.md`. Read it first; this plan does not restate
its measurements. Survey: `docs/superpowers/plans/2026-09-06-collections-survey.md` at `2f041fd49`.

Classes: `Array`, `Queue`, `List`, `CircularQueue`. `Stem` and the mapped classes are Phase 5h;
`RexxQueue` is out of scope by D93.

**The thing to hold on to while reading this plan.** For these four classes the `answers` column of
`corpus/method-bodies.txt` is not evidence of anything. `t['k'] = 'v'` on a `Table` reads `answers`
for `[]=` and refuses at rc 120 on both engines; `.Bag~new~put('x')` reads `answers` and is rc 168
against the oracle's rc 0. That is not a bug in the harness -- its own comment says a zero-argument
send to a method needing arguments is agreement about an arity error -- it is why Task 0 exists and
why no later task may cite a verdict as a reason to skip a row.

**Two examples that look like they belong here and do not.** `.Table~new~at('k')` answers
`The NIL object` at rc 0 on the oracle *and* on both engines -- the read is fine, it is the write
that refuses. And `.CircularQueue~new(3)~items` refusing says nothing about `CircularQueue`: plain
`.Queue~new~items` refuses identically, because `Queue` has no store. Spec D90 has the measurements.

BASE for Task 0 is the commit this plan lands in.

---

## Global constraints

Every task, no exceptions. Inherited from Phase 5f's plan unchanged except where marked NEW.

* **Commit before the full gate run, not after**, per `rust/CLAUDE.md`'s Gates section: fast checks
  yourself, commit code and report together with the report's gate cells left as `**G1**`–`**G7**`,
  start the suite in the background with the commit sha as the status file's first line and a
  pidfile beside it, report `committed at <sha>, gates running, statuses at <path>` and **stop**.
  The tree belongs to the gate run until its status file says `finished`.
* **The fast checks are `cargo test --release --workspace --no-fail-fast`, not a chosen subset.**
  It is 12 minutes and it is the whole of what G3 runs.
* **Every claim gets a red control**, predicted before it is run and marked confirmed, falsified or
  unobservable. A control that varies something *about* the change proves nothing.
* **Both engines every time**: `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` — that exact spelling.
* Three separate descriptors, never `2>&1`, `$?` captured immediately.
* Oracle runs from a fresh empty directory, wrapped as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
* **Never `git checkout -- <path>`** on a file you edited; `cp` to your scratch directory and
  restore from the copy, then `touch` it.
* No `unsafe`; the workspace lint is `deny` and it is Moritz's call per site.
* Never `git add -A`; never amend; `git commit -F <file>` with paths named; confirm `Cargo.lock` is
  not staged unless the task deliberately changed it.
* **A method that exists and does nothing is not implemented.** Do not add a name that answers
  nothing to close a row.
* **The corpus control that can go red is `REXX_CORPUS_GATE=1`.** The plain `--test corpus` binary is
  report mode and exits 0 on a divergence. Cite the STRICT run and its matching line.
* **Mutation runs build with `--profile mutation`**, under `memcap 8G`, with `--no-fail-fast`.
* Every family's witness sends a **short argument list as well as a good one**; the shared
  missing-argument raiser makes those rows agree in `method-bodies.txt` for free.
* Every family files both witnesses in `corpus/phase-5c.txt` **and** `EXPECTED_SUBSET_5C`, generates
  a `crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, refreshes `corpus/method-bodies.txt`
  under `REXX_METHOD_BODIES_REFRESH=1`, and adds its file to `dispatch_seam.rs`'s
  `CLEARANCE_CONSUMERS` if it is a new one.
* **NEW — no task may cite the `answers` verdict as a reason a row needs no work.** The instrument
  Task 0 commits is what a later task reads. A row this phase declines to do is declined in a report
  sentence naming why, not by a green cell.
* **NEW — a witness may not depend on an iteration order the oracle does not itself reproduce.**
  Project memory `oorexx-hash-iteration-order` measured that identity-keyed iteration does not
  reproduce across runs of the oracle. That is 5h's problem mostly, but `makeArray` and `allItems`
  on an ordered collection have a *defined* order and their witnesses must assert it; a witness that
  sorts before comparing has thrown away the property it was written for.

---

## Task 0 — the instrument, the scopes table, and the module

**Lands alone, ahead of every other task, and adds no method body.**

Three deliverables, and the first is the phase's load-bearing measurement.

**(a) `corpus/collection-scopes.tsv`, derived and re-derived (spec D89).** One row per (class,
method, arm) for every class in scope: the scope that defines it, native or Rexx, and for a native
scope the C++ entry point and the `Setup.cpp` arity operand verbatim.

**The scope column comes from the oracle, not from a file scan**, and this is the plan's single
biggest correction from review:

```rexx
inst["ARRAY"] = .Array~new  ...  inst["CIRCULARQUEUE"] = .CircularQueue~new(5)
if arm == "instance" then m = inst[cls~upper]~instanceMethod(nm)
                     else m = .environment[cls~upper]~instanceMethod(nm)
say cls nm arm m~scope~id
```

Sent to an **instance** this resolves every inherited scope, over all 471 documented rows with no
misses. Sent to the class object it answers `.nil` for inherited names, which is what sent the first
version of this plan down a file-scanning path. `Class~method` is own-scope only; do not use it.

Then read `Setup.cpp` keyed by (scope, name) for the token and for native-or-Rexx: present in that
scope's post-`InheritInstanceMethods` table means native with that token, absent means Rexx. **The
Method object cannot answer that question** -- `~source~items` is 0 for `CircularQueue`'s Rexx
`queue` and `Array`'s native `[]` alike, and `~package~name` is `REXX` for both.

Follow `refusal_sites.rs`: the file is committed and the test re-derives it, so an upstream edit is
a red test rather than a stale document. A row whose scope resolves nowhere is a **failure**, not a
blank; `RexxQueue` is excluded by name so that failure mode stays available.

**If you build the scope column from the files anyway, these three will each attribute rows to the
wrong body**, and the first version of this plan named none of them:

* `RemoveMethod` — `Setup.cpp:792-804` strips `Dimension Dimensions Fill sort sortWith stableSort
  stableSortWith makeString toString` from `Queue` *after* `InheritInstanceMethods(Array)` copied
  them in. Nine documented rows across `Queue` and `CircularQueue` move to Rexx bodies.
* `HideMethod` — `Setup.cpp:1399-1404` on `Stem`. Moves no documented row today; would matter the
  moment `Stem`'s operator rows enter the table.
* The prolog's phony inherits at `CoreClasses.orx:80-87` —
  `.set~inheritInstanceMethods(.SetMixin)` and its four neighbours put `Set~union`, `Bag~union`,
  `Relation~union` and others at the class's own scope on a **mixin** body. `SetMixin~union` copies
  then adds items; `Collection~union` is single-valued. Attributing those to `Collection` is the
  wrong body, not just the wrong scope name.

**And the entry-point column is a citation, not a body identity.** Two tokens can name one C++
function (`Set`'s `HasItem` is written `IdentityTable::hasIndexRexx`; that header declares no such
member), and one token can reach three behaviours selected by the receiver's contents class. Say
which the column is in the file's own header.

**(b) The re-probe instrument, `corpus/collection-arity.tsv`.** For every row of every class in
scope, send the documented name with an argument list the method could accept, to a receiver holding
something, on both engines and on the oracle, and record the three descriptors' agreement. This
replaces the verdict column for this phase.

**The rule that makes it mean anything: a row's argument list is real only if the ORACLE answers
rc 0 to it.** An empty list, or one the oracle rejects, is a harness failure for that row and not a
data point. Without it the instrument is defeated two ways that both read green — fill in argument
lists for the control rows only and leave the rest empty, or send two arguments to everything and
let both sides agree on 93.902.

**Its red control, and its tree.** `Bag~put` and `Set~put` must come out **disagreeing**: with no
argument both sides give rc 163, with one argument the oracle is rc 0 `1` and this crate rc 168. But
spec D92's commit turns that row **loud**, and in a harness shaped like `method_bodies.rs` — whose
own doc says pass one is crate-only and pass two runs the oracle only for rows pass one did not
classify — a loud row never reaches the oracle, so "disagreeing" stops being a state it can be in
and the control becomes unfalsifiable rather than red. **Decide the order before writing the
prediction, and say in the table's header what verdict a loud row takes.**

Report, whether or not it is acted on: how many rows in scope read `answers` and disagree under a
real argument list. That number sizes 5h.

**(c) `crates/rexx-exec/src/dispatch/collection.rs`**, beside `dispatch/native.rs` and
`dispatch/string.rs`, with its own `NATIVE_METHODS` slice chained into the registration loop.
**Land it with the slice empty.** Its red control is Phase 5f Task 0's, which worked: a row naming a
method the class does not answer must panic at `ObjectModel::build` (`dispatch.rs:1200,1205`), and a
row binding a real loud name to the wrong body must make the send reach that body. Remove both and
confirm the name is loud again.

---

## Task 1 — the contents protocol, `Supplier`, and `Array`'s shared surface

`Array`'s store exists (`Body::Array`, `slots: Vec<Option<ObjRef>>` with `dimensions`), which is why
the protocol is introduced here and not against a store being invented in the same commit.

Introduce the internal protocol — iterate (index, item) pairs in the store's order, look up by
index, put, remove by index, count — and write `Array`'s shared surface against it:
`allIndexes allItems supplier index makeArray empty isEmpty hasIndex hasItem remove removeItem`.

**`Supplier` is this task's, and it blocks thirteen rows across both phases** (spec D99).
`Collection~supplier` is `.supplier~new(self~allItems, self~allIndexes)` (`CoreClasses.orx:753`),
and `Supplier` is unimplemented here:

```
.Supplier~new(.Array~of('x1','x2'), .Array~of(1,2))~available
    oracle rc 0            crate rc 120  method "AVAILABLE" of class "Supplier"
```

Its `Available Index Next Item Init` are `SupplierClass::*` (`Setup.cpp:1624-1628`). No `supplier`
row in either phase can be witnessed until they exist, so they are written here rather than
discovered by whichever task first tries to close one.

**`makeArray` is written per store, not once** (spec D98). The token `RexxObject::makeArrayRexx` is
shared because `ObjectClass.cpp` defines it as `return makeArray();`, a virtual, and the answers
differ: `Array` and `List` give **items**, hash collections give **indexes**, `Stem` gives its tail
array. Measured on the oracle:

```
Array     makeArray: p r      allIndexes: 1 3
Directory makeArray: k1 k2    allItems:   v1 v2
```

Write `Array`'s here. Do not write a shared one — the first version of this plan said "write it
once, here, against the protocol", which produces items where every 5h class needs keys.

Two more traps, both upstream-visible:

* `Array`'s `hasIndex`, `next`, `previous` and `remove` are `A_COUNT`, not fixed — a
  multi-dimensional index arrives as several arguments. The operand is in Task 0's table; read it.
* A sparse `Array` has holes. `allItems` skips them, `allIndexes` skips them, `items` counts
  non-holes, `size` does not. Any of the four written as "the length of the vector" is wrong and no
  zero-argument probe sees it.

Witnesses: one corpus program for the enumeration surface over a populated, a sparse and an empty
`Array`; one for `Supplier` driven to exhaustion; one refusals program.

---

## Task 2 — `Array`'s own surface

`append delete fill insert section dimensions first last firstItem lastItem next previous`.

Navigation and structure. The section family and `insert`/`delete` shift indexes, so the witness
must show the *whole* array after the operation, not the return value —
`assert-the-side-effect-not-the-return-value`. `fill` and `dimensions` interact with the
multi-dimensional case that `Body::Array` already carries.

Witnesses: one program per half (navigation, structure) plus refusals.

---

## Task 3 — the sort family

**`Array`'s four, and `Array`'s only.** All four land on `ArrayClass::stableSortRexx` and
`ArrayClass::stableSortWithRexx` — the interpreter maps the unstable names onto the stable bodies,
so **there is one algorithm here, not two**. `Queue` and `CircularQueue` do *not* share them:
`Setup.cpp:799-802` removes all four from `Queue` after it inherits `Array`'s behaviour, and their
rows fall through to `OrderedCollection`'s Rexx bodies, which route through `makeArray`. Task 0's
table shows this; a task that assumes the `Array` bodies serve all three is writing for one class
and claiming three.

`sortWith` takes a `Comparator`, and the comparator classes are Rexx-level in `CoreClasses.orx` and
already installed — `DescendingComparator`, `CaselessComparator`, `ColumnComparator`,
`InvertingComparator`, `NumericComparator` and the caseless column one. So this task's real content
is calling back into a Rexx method from a native body, per element. **Check that this crate can do
that before planning the sort**: if a native body cannot send `compare` to a Rexx object and get an
answer, that is the task, and the sort is what comes after it.

Sorting is comparison-visible: a witness must sort a list whose oracle order distinguishes stable
from unstable, or it has not tested the property in the method's name.

---

## Task 4 — `Queue`

Upstream, `Queue` copies `Array`'s whole native behaviour and then overrides. Task 0's table says
which rows are `ArrayClass::*` reached through the copy and which are `QueueClass::*`:
`push queue pull peek put []= delete remove` are `Queue`'s own -- and `delete` and `remove` are one
body upstream, as `size` and `items` are -- while `QueueClass::validateIndex` is a different index
rule from `Array`'s.

**The decision this task makes and must state in its report**: whether `Queue` reuses `Body::Array`
or gets its own store. Upstream shares the behaviour, not necessarily the layout; a deque with an
O(1) `pull` is not a `Vec` with `remove(0)`. Correctness first — this phase is not a performance
phase (spec §6) — but the choice is recorded, not drifted into.

If the protocol from Task 1 does not make `Queue`'s shared surface nearly free, **that is the signal
the protocol is wrong**, and it is cheaper to find it here than in 5h where seven classes depend on
it. Say so in the report either way.

---

## Task 5 — `List`

The one ordered class whose store this crate does not have, and the one whose semantics are not the
obvious ones: **a `List` index is not a position.** It is a handle that stays valid as the list is
inserted into and deleted from, `index`/`at`/`put`/`next`/`previous`/`section` all speak it, and an
implementation that makes it a subscript passes every single-element test and fails the first
`insert`.

`ListClass::validateIndex` upstream is the authority. Write the store to make the handle stable, and
the witness must insert and delete around a held index and show that it still reaches the same item.

`sort` and `stableSort` on `List` are Rexx-level, on `OrderedCollection` rather than on any native
body -- they route through `makeArray`, so they close when Task 1's `makeArray` and this task's
store meet. Confirm that by running, not by reading. `CircularQueue`'s own Rexx-level rows are
Task 6's; outside that task these two are the only ones.

---

## Task 6 — one instance carrying a store and a variable pool, and `CircularQueue`

**Not "`~new` on a subclass of a native class is unimplemented".** That was the first version's
framing and spec D90 records why it is wrong: three different mechanisms are in play and one of them
is not a subclass effect.

* `.Queue~new~items` refuses identically to `.CircularQueue~new(3)~items`, so **nothing observed
  about `CircularQueue` is evidence about subclass construction.** Its Rexx `init` and `size` run
  today.
* A `StringTable` subclass already carries a working store — `.MyST~new; s['k']='v'; say s['k']` is
  `v` at rc 0 on both engines — via `Primitive::StringTable(ObjRef)` (`dispatch.rs:1366`), and
  `TraceObject subclass StringTable` has shipped on it.
* A `Directory` subclass is **deliberately** given a plain instance so `expose` works:
  `native_directory_new` (`dispatch.rs:7839`) branches on the exact class, pinned by the test
  `a_directory_subclass_keeps_the_instance` (`dispatch.rs:11786`). That is why `Properties`' writes
  refuse, and it is a design to change knowingly, not a gap to fill.
* `Array` refuses at `~new` itself (`dispatch.rs:12068`).

**The subject is therefore one instance carrying a native store *and* an object variable pool**, and
the witness that both are needed on one object is `CircularQueue~init`'s `expose size`
(`CoreClasses.orx:1726-1728`). Read `dispatch.rs`'s own account of why identity and descent coexist
before changing either.

`CircularQueue` needs this and a `Queue` store, and nothing else of its own: its bodies are Rexx,
already installed and running. Note from Task 0's table that its `sort`/`stableSort` are
`OrderedCollection`'s, not `Array`'s — `RemoveMethod` stripped them from `Queue`.

`Properties` is 5h's; if it falls out here, take it and say so.

Its witness is ordinary user Rexx — `::CLASS MyArr SUBCLASS Array` with a method of its own that
calls `self~append` and one that `expose`s an instance variable, because the two together are the
property this task exists for.

---

## Task 7 — receiver overrides, the sweep, and close

**(a) `RECEIVER_OVERRIDES` for the four classes** (D91). The 5c follow-up's open sweep and Phase 5f
Task 0's measurement both land here: with the stores working, a populated receiver sharpens rows
that an empty one cannot discriminate. Add the entries, refresh `method-bodies.txt`, and report how
many rows changed verdict as a result — that number is the sweep's answer, and it retires the
follow-up's question 2.

**(b) Re-run Task 0's instrument** and diff it against the run Task 0 committed. Every row this
phase claims to have closed must have moved in that table. **A row that moved in
`method-bodies.txt` and not in the instrument has not been implemented**, it has been given a
signature.

**(c) The close report** enumerates the rows the phase moved by diffing `method-bodies.txt` against
BASE — not by adding up what the task reports claimed — and states, per class, what is left and who
owns it.

No `CLOSED_PHASES` change and no `class-set.txt` edit (D86 inherited): `5g` is these documents' name,
not a value anything reads.

---

## What this plan does not cover

Phase 5h's task list. Its sizing is Task 0(b)'s output, and writing it now against the verdict
column is the mistake the survey exists to prevent. Its shape is the spec's §3 and §4: one
object-keyed store with the four key-semantics variants `HashContents.hpp` names, the shared surface
over the Task 1 protocol, `Stem` possibly separately, and `Properties` on Task 6's mechanism.
