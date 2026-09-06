# Phase 5g — the ordered collections

Spec: `docs/superpowers/specs/2026-09-06-collections.md`. Read it first; this plan does not restate
its measurements. Survey: `docs/superpowers/plans/2026-09-06-collections-survey.md` at `2f041fd49`.

Classes: `Array`, `Queue`, `List`, `CircularQueue`. `Stem` and the mapped classes are Phase 5h;
`RexxQueue` is out of scope by D93.

**The thing to hold on to while reading this plan.** For these four classes the `answers` column of
`corpus/method-bodies.txt` is not evidence of anything. `Table~at` reads `answers` and refuses at
rc 120 when sent an argument; `CircularQueue~queue` reads `answers` and refuses the same way. That
is not a bug in the harness -- its own comment says a zero-argument send to a method needing
arguments is agreement about an arity error -- it is the reason Task 0 exists and the reason no
later task may cite a verdict as a reason to skip a row.

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

**(a) `corpus/collection-scopes.tsv`, derived and re-derived (D89).** A test in `rexx-exec`'s test
tree that reads `/home/moritz/dev/repos/ooRexx/interpreter/memory/Setup.cpp` and
`/home/moritz/dev/repos/ooRexx/interpreter/RexxClasses/CoreClasses.orx` and joins them against
`corpus/docs/class-methods.txt`, emitting one row per (class, method, arm) for every class in this
spec's scope: the scope that defines the method, whether that scope is native or Rexx, and for a
native scope the C++ entry point and the `Setup.cpp` arity operand verbatim (`A_COUNT` or the
literal). Follow `refusal_sites.rs`: the file is committed and the test re-derives it, so an
upstream edit is a red test rather than a stale document.

Three things it must get right, each of which is a way to be silently wrong:

* `StartClassDefinition(X)` blocks, and `InheritInstanceMethods(source)` copies the source's whole
  instance behaviour *before* the block's own `AddMethod` lines override part of it. A join that
  ignores the macro puts `Table`'s rows on nothing.
* Scope order is most-derived first, and for `CircularQueue` and `Properties` the most-derived scope
  is a **Rexx** one. Getting this backwards is what makes `CircularQueue~supplier` look native.
* A row whose scope resolves nowhere is a **failure**, not an empty cell. `RexxQueue`'s rows resolve
  nowhere and are excluded by name in the table's own scope list, so that the failure mode stays
  available for a class that is supposed to resolve.
* **The entry point is the token `Setup.cpp` writes, and two tokens can name one C++ function.**
  `Set`'s `HasItem` is written `IdentityTable::hasIndexRexx`, and `IdentityTableClass.hpp` declares
  no such member -- it is `HashCollection::hasIndexRexx` reached through C++ inheritance. So the
  table's entry-point column is a *citation*, not a body identity, and a task that counts distinct
  bodies from it overcounts. Say which the column is in the file's own header.

**(b) The re-probe instrument, `corpus/collection-arity.tsv`.** For every row of every class in
scope, send the documented name **with an argument list that the method could accept**, to a
receiver holding something, on both engines and on the oracle, and record the three descriptors'
agreement. This replaces the verdict column for this phase, and its whole purpose is to find the
rows that read `answers` and are wrong.

**Its red control, and the control is the point.** The instrument's failure mode is that it sends
argument lists nothing accepts, agrees about *that*, and reports a green table -- the same shape of
mistake one layer down from the one the survey found. So: `Bag~put` and `Set~put` must come out
**disagreeing** in the first run of this table, because D92 has already measured that they do
(`b = .Bag~new; b~put('x'); say b~items` is oracle rc 0 `1`, this crate rc 168). A first run that
puts those two rows in agreement means the harness is not sending an argument, and the table is
worthless until it does. Predict that before running it and record which way it came out.

Report, whether or not it is acted on: how many rows in scope read `answers` and disagree under a
real argument list. That number sizes 5h.

**(c) `crates/rexx-exec/src/dispatch/collection.rs`**, beside `dispatch/native.rs` and
`dispatch/string.rs`, holding its own `NATIVE_METHODS` slice chained into the registration loop.
**Land it with the slice empty.** Its red control is Phase 5f Task 0's, which worked: a row naming a
method the class does not answer must panic at `ObjectModel::build`, and a row binding a real loud
name to the wrong body must make the send reach that body. Remove both and confirm the name is loud
again.

---

## Task 1 — the contents protocol, proved on `Array`'s shared surface

`Array`'s store exists (`Body::Array`, `slots: Vec<Option<ObjRef>>` with `dimensions`), which is why
the protocol is introduced here and not against a store being invented in the same commit (D95).

Introduce the internal protocol — iterate (index, item) pairs in the store's order, look up by
index, put, remove by index, count — and write `Array`'s shared surface against it:
`allIndexes allItems supplier index makeArray empty isEmpty hasIndex hasItem remove removeItem`.

**`makeArray` is `RexxObject::makeArrayRexx`, the one body every class in both phases lands on**
(spec §3). Write it once, here, against the protocol.

Three traps, all upstream-visible:

* `Array`'s `hasIndex`, `next`, `previous` and `remove` are `A_COUNT`, not fixed — a multi-dimensional
  index arrives as several arguments. `Setup.cpp`'s operand is in Task 0's table; read it.
* A sparse `Array` has holes. `allItems` skips them, `allIndexes` skips them, `items` counts
  non-holes, and `size` does not. Any of those four written as "the length of the vector" is wrong
  and no zero-argument probe sees it.
* `supplier` answers a `Supplier` object, and `SupplierMixin` is one of the `CoreClasses.orx`
  mixins. Check whether `.Supplier` constructs in this crate today before assuming the row is
  reachable; if it is not, that is a finding for the report and possibly a task, not something to
  work around with a substitute object.

Witnesses: one corpus program for the enumeration surface over a populated, a sparse and an empty
`Array`; one refusals program.

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

`sort sortWith stableSort stableSortWith`, all four on `ArrayClass::stableSortRexx` and
`ArrayClass::stableSortWithRexx` — the interpreter maps the unstable names onto the stable bodies,
which Task 0's table shows and which means **there is one algorithm here, not two**.

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

## Task 6 — `~new` on a subclass of a native class, and `CircularQueue`

The mechanism of D90. The crate refuses in its own words today —
`rexx-exec: ~new on a subclass of Array is not implemented (Phase 5)` — so the refusal site is
already named and findable.

`CircularQueue` needs it and nothing else of its own: its bodies are Rexx, in `CoreClasses.orx`,
already installed and already running. `Properties` needs the same mechanism against `Directory`
and is 5h's, but if it falls out for free here, take it and say so.

**Scope check before starting** (spec §7): what a subclass instance carries as its `Body`, and
whether `Primitive` resolution moves from identity to descent, is not settled by this plan.
`dispatch.rs` documents `Primitive::Directory` as "identity and not descent" deliberately and
`Primitive::StringTable` as the opposite, so the two conventions already coexist and the reason is
written down there. Read that first. If the answer is large, split this task and say why — do not
widen it silently.

Its witness is ordinary user Rexx, not a library class: `::CLASS MyArr SUBCLASS Array` with a
method of its own that calls `self~append`, because that is the shape that proves the native body
and the Rexx subclass share a receiver.

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
