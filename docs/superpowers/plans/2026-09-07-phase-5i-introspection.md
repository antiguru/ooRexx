# Phase 5i — the introspection surface

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` to
> execute this plan task by task.

Scope: `docs/superpowers/plans/2026-09-07-phase-5i-scope.md`, which was written before any work and
which settles what is in, what is out, and where each excluded class goes. Read it first; this plan
does not restate its argument.

Classes: `Package`, `RexxInfo`, `Method`, `RexxContext`, `Class`, `StackFrame`, `Routine`,
`Object`'s three introspection rows, `Pointer`, `Buffer`, `WeakReference`, and the `of` row of the
eight mapped classes. **139 `loud` rows of `corpus/method-bodies.txt`.**

`Message` and the two semaphores are Phase 6; `Stream`, `File` and `RexxQueue` are Phase 7. The
scope note carries the measurement behind each.

**The thing to hold on to while reading this plan.** This phase's subject is a program asking the
interpreter about *itself* — its own source, its own package, its own call stack, its own classes
and methods. Almost every row reads state the interpreter already has and does not currently hand
out. So the work is rarely a new algorithm and almost always a new *reader*, and the risk is
correspondingly different from 5g's and 5h's: the danger here is not a wrong data structure, it is
a reader that answers something plausible which the oracle does not say.

**And the trap this phase is most exposed to** is `rust/CLAUDE.md`'s "a method that exists and does
nothing is not implemented". A `Package` that answers `~source` with the right lines and `~classes`
with an empty StringTable has moved two rows and implemented one method. Every task below says
which of its rows must answer from real state and which may be a documented refusal; a row closed
any other way is a defect, and the close report names it.

BASE for Task 0 is the commit this plan lands in.

---

## Global constraints

Every task, no exceptions. Inherited from Phase 5g's plan
(`2026-09-06-phase-5g-ordered-collections.md`, *Global constraints*) unchanged except where marked
NEW. They are restated here rather than cited because a rule that lives only in another document is
prose the executor never reads.

* **Commit before the full gate run, not after**, per `rust/CLAUDE.md`'s Gates section: fast checks
  yourself, commit code and report together with the report's gate cells left as `**G1**`–`**G7**`,
  start the suite in the background with the commit sha as the status file's first line and a
  pidfile beside it, report `committed at <sha>, gates running, statuses at <path>` and **stop**.
  The tree belongs to the gate run until its status file says `finished`.
* **The fast checks are `cargo test --release --workspace --no-fail-fast`, not a chosen subset.**
* **Every claim gets a red control**, predicted before it is run and marked confirmed, falsified or
  unobservable. A control that varies something *about* the change proves nothing.
* **Both engines every time**: `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` — that exact spelling.
* Three separate descriptors, never `2>&1`, `$?` captured immediately.
* Oracle runs from a fresh empty directory you `mkdir` yourself, wrapped as
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
* Every family's witness sends a **short argument list as well as a good one**.
* Every family files both witnesses in `corpus/phase-5c.txt` **and** `EXPECTED_SUBSET_5C`
  (`crates/rexx-exec/tests/coverage.rs`), generates a
  `crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, refreshes `corpus/method-bodies.txt`
  under `REXX_METHOD_BODIES_REFRESH=1`, and adds its file to `dispatch_seam.rs`'s
  `CLEARANCE_CONSUMERS` if it is a new one.
* **No task may cite the `answers` verdict of `corpus/method-bodies.txt` as a reason a row needs no
  work.** Task 0's instrument is what a later task reads. A row this phase declines to do is
  declined in a report sentence naming why, not by a green cell.
* **Wait in ~1-hour stretches**, never ten-minute polls. A backgrounded command re-invokes its
  caller when it exits; the timer is the fallback against a hang, not the mechanism.

**NEW, and both are this phase's own shape:**

* **A reader that answers from a constant is a defect unless the oracle's own answer is a
  constant.** This phase is full of methods whose right answer on the machine that runs the gate is
  a fixed string — `RexxInfo~platform`, `RexxInfo~version`, `Package~name`. The differential cannot
  tell "reads the interpreter's build settings" from "returns the literal the differential was
  written against". So for every such row the task states, in one sentence in its report, **what
  state the reader reads and where that state comes from**, and where a constant is genuinely
  right, it says so and names the C++ constant it mirrors. `RexxInfo` has a section of its own
  below because it is where this bites hardest.
* **A row whose answer is an object of a class this phase also builds must be witnessed through a
  second send.** Project memory `one-send-to-a-fresh-receiver`: a harness that sends once to a
  freshly built receiver cannot see a defect that needs two sends, and four such defects shipped
  past eight green gates. `Package~classes` answering a `StringTable` proves nothing until
  something reads an entry out of it; `Object~instanceMethod` answering a `Method` proves nothing
  until that `Method` is asked its `~scope`.

---

## Task 0 — extend the arity instrument to this phase's classes

**Lands alone, ahead of every other task, and adds no method body.**

The scope note's *Prerequisite* section is this task: `corpus/collection-arity.tsv` covers the
collections only, so every class in this phase currently has **no arity row anywhere**, and the
only instrument that has run them is `corpus/method-bodies.txt` — whose own header says a
zero-argument send to a method needing arguments records agreement about an *arity error*. Sizing
this phase from that column is the same mistake that once got a phase planned around gate table
C's `hasMethod` readbacks.

**Do not widen `corpus/collection-arity.tsv`, and do not rename it.** Its header carries a
load-bearing sentence scoped to the collections — "THIS TABLE REPLACES `corpus/method-bodies.txt`'s
verdict column for these classes" — and twenty-two committed plan and record documents cite the
file by name. Widening the file silently widens that sentence over classes it was never measured
for; renaming it dangles every one of those citations.

**Instead: one harness, two tables.**

**(a) Factor the probe out of `crates/rexx-exec/tests/collection_arity.rs` into a shared module**
under `crates/rexx-exec/tests/support/`, parameterised by the four paths it reads and writes
(receivers, arguments, scopes, output table), the refresh environment variable, and the header
text. `collection_arity.rs` becomes a thin driver over the existing four files and **its committed
table must not move by one byte** — that is this half's red control, and it is available for free:
run the refresh before and after the refactor and `diff` the table. A refactor that changes a
verdict has changed the instrument, not the code under it.

**(b) A second driver, `crates/rexx-exec/tests/introspection_arity.rs`,** over new files
`corpus/introspection-receivers.tsv`, `corpus/introspection-arguments.tsv` and
`corpus/introspection-scopes.tsv`, writing `corpus/introspection-arity.tsv`, for the classes of
this phase: `Package`, `RexxInfo`, `Method`, `RexxContext`, `Class`, `StackFrame`, `Routine`,
`Object`, `WeakReference`.

**`Pointer` and `Buffer` are excluded by name, with the reason in the header**, the way
`collection_scopes.rs` excludes `RexxQueue`. `corpus/docs/class-set.txt` gives them no construction
expression — the reference says instances come only from native code
(`utilityclasses.xml:429`, `:6910`) — so no receiver exists to send to and every instance row of
theirs would be `setup-differs` on the oracle's own side. Naming them keeps "a row with no receiver
is a failure" available as a signal for a class that is supposed to have one.

**The harness rule carries over unchanged and is the thing that makes the table mean anything:** a
row's argument list is real only if the **oracle** completes the send, which the probe reports by
printing `SENT`. An oracle run that does not reach it is a harness failure for that row and not a
data point. `the_oracle_completes_every_send` is the test that enforces it; the introspection
driver gets its own copy. The two ways this instrument is defeated while reading green — fill in
lists for two rows and leave the rest empty, or send two arguments to everything and let both sides
agree on 93.902 — are closed by that rule plus `every_row_is_sent_something_its_arity_needs`, which
reads the scopes table's `arity` column.

**The receivers are the hard part of this task and they are where the thinking goes.** Each has to
be an expression the *oracle* can evaluate, leaving `r` holding a receiver that is worth sending
to. Sketches, to be measured rather than trusted:

* `Package` — `r = .context~package`, inside a probe file that has a `::routine`, a `::class` with
  a method, and a `::requires` if one can be satisfied on this build's search path, so that
  `~classes`, `~routines` and `~importedPackages` have something to answer. A `Package` with an
  empty program behind it makes every table row answer an empty table on both sides and the row
  says nothing.
* `RexxContext` — `r = .context`.
* `StackFrame` — `r = .context~stackFrames[1]`, and the probe must be **inside a routine called
  from the main program**, so the frame is not the only one on the stack.
* `Method` — `r = .methods['M']` with a `::method M` directive in the probe, or `.Method~new`.
  Whichever is used, the receiver must have a **scope** for `~scope`-shaped rows to mean anything,
  which means it has to have been installed on a class.
* `Routine` — `r = .routines['R']` with a `::routine R` in the probe.
* `Class` — `r` a class with a subclass, a mixin, and at least one method defined on it, so
  `~subclasses`, `~queryMixinClass` and `~methods` are not all empty.
* `Object` — `r = .Object~new`, and separately something with methods on it so `~instanceMethods`
  is non-empty.
* `WeakReference` — `class-set.txt`'s own expression, `.WeakReference~new(.Object~new)`.

**The receiver file's existing header rule applies and is why these are shaped that way: the setup
is the richest receiver the ORACLE can build, not the richest both sides can.** A class this crate
cannot build a receiver for fails at setup and the probe reports `setup-differs`, which is the
phase's headline measurement and not a harness fault.

**Two things the existing receiver format cannot express and that this task must decide, not
discover:** the probe writes one file and runs it, so a receiver needing `::routine`, `::method` or
`::class` **directives** needs those in the same file — the current format is a `|`-separated list
of statements only. And `StackFrame`'s receiver needs the send to happen inside a call. Decide the
mechanism (a per-class *prolog* column holding directive text appended after the send, a per-class
*wrapper* that puts the send inside a routine, or both) and say in the file's header what it is.
Ask before inventing a third file.

**(c) `corpus/introspection-scopes.tsv`, derived from the oracle** the way
`collection_scopes.rs` derives its own: `receiver~instanceMethod(NAME)~scope~id` sent to an
*instance*, never to the class object, which answers `.nil` for inherited names. Read `Setup.cpp`
keyed by (scope, name) for the entry-point token and the arity operand. A row whose scope resolves
nowhere is a **failure**, not a blank.

**Deliverable of this task, and it is what sizes every later one:** the number of rows in this
phase's classes that read `agree` under a real argument list, and the number that read
`send-differs`. Report both, per class, in the task report. If a class's row count under the new
instrument differs materially from its `loud` count in the scope note, **say so and stop** — that
is a re-slice, and carrying a stale size through the phase is what the scope note exists to
prevent.

---

## Task 1 — `ARG` option `"A"`, and the eight `of` rows it unblocks

The scope note's cheapest first move, and the only group in this phase with a measured single
cause. **Eight `loud` rows of `corpus/method-bodies.txt`** — `of` on `MapCollection`, `Directory`,
`IdentityTable`, `Properties`, `Relation`, `Stem`, `StringTable` and `Table` — plus one
`send-differs` row of `corpus/collection-arity.tsv`, `Properties~setLogical`, which is the scope
note's ninth.

`crates/rexx-exec/src/builtin/state.rs:417` refuses `arg(n,'A')` with
`Loud::builtin_option_object("ARG", b'A', "an Array")`. Every one of the eight mapped classes'
`of` rows fails there, because `MapCollection~of` is Rexx and its first statement is `args = arg(1,
'a')` (`interpreter/RexxClasses/CoreClasses.orx:1238`-`:1239`).

**The oracle's semantics, read from `interpreter/expression/BuiltinFunctions.cpp:873`
(`BUILTIN(ARG)`), case `'A'`:**

* position `1` → the whole argument list as an Array, `new_array(size, arglist)`;
* position greater than the count → an **empty** Array, `new_array()`;
* otherwise → the sub-array from that position, `new_array(size - position + 1, &arglist[position - 1])`.

The position is validated by `positive_integer` **before** the option switch, exactly as the
existing `'E'`/`'O'` arms are, and `state.rs`'s own doc comment above `arg` records the four
measured orderings — they do not change.

**The load-bearing detail, and it is the one a naive implementation drops.** `arglist` holds
`OREF_NULL` for an omitted argument, and `new_array` copies those pointers straight through, so
`arg(1,'a')` on a call with an omitted middle argument answers an Array with a **hole** — not
`.nil`, and not a shortened list. `Body::Array`'s `slots: Vec<Option<ObjRef>>` already models
exactly this, and `corpus/method-bodies.txt`'s own `Array` rows record that an empty slot and a
slot holding `.nil` are different values. `MapCollection~of` reads that distinction directly:
`if \args~hasIndex(i) then raise syntax 93.903 array(i)` (`CoreClasses.orx:1253`-`:1254`). So a
witness that only calls with every argument supplied **cannot see the defect**, and one that calls
`.Directory~of(.Array~of('k','v'), , .Array~of('k2','v2'))` can.

**Measured on the oracle 2026-09-07**, from a routine called as `call r 'a', , 'c'`:

```
arg(1,'A')   size=3  items=2  dimension=1  hasIndex(2)=0   [1]='a'  [3]='c'
arg(2,'A')   size=2  items=1  dimension=1
arg(3,'A')   size=1              dimension=1
arg(4,'A')   size=0              dimension=0   class=Array
arg(99,'A')  size=0              dimension=0
arg()        3
```

**`dimension` 0 for the past-the-end case is the second detail a naive implementation gets wrong**,
and the crate has a live example of the wrong answer to copy from: `native_array_of`
(`dispatch.rs:6509`) sets `dimensions: Some([0])` when its argument list is empty, because
`.array~of()` answers `~dimension` `1`. `arg(n,'A')` past the end answers `0`, so it wants
`dimensions: None`. The rest of `native_array_of` is the template — `slots: args.to_vec()` copies
the holes through exactly.

**Witnesses this task owes:**

* `arg(1,'A')` from a routine called with three arguments, the middle one omitted: assert
  `~size`, `~items`, `~hasIndex(2)`, and the values at 1 and 3.
* `arg(n,'A')` for `n` = 1, an interior position, exactly the count, count + 1, and a large
  position: assert `~size` for each.
* `arg(0,'A')` and `arg('x','A')`: the 40.14 and 40.12 raises, unchanged by this task and asserted
  so that the ordering stays pinned.
* Each of the eight `of` rows end to end — `.Directory~of(.Array~of('k','v'))` and its siblings —
  and, per this plan's second NEW constraint, **a second send that reads an entry back out** of the
  collection `of` built.
* `MapCollection~of`'s omitted-argument raise, `93.903` (`CoreClasses.orx:1253`-`:1254`). It comes
  from the Rexx body once `arg` works, so this task's job is to run it and confirm the crate
  reaches it — if it does not, the finding is in whatever `raise syntax N array(...)` does here,
  not in `arg`.

**The two raises this task cannot witness, and why that is not a gap in it.** The 88.923 and 88.924
arms of the same body send `.context~name` (`CoreClasses.orx:1258`, `:1263`), and measured
2026-09-07 that is `method "NAME" of class "RexxContext" is not implemented (Phase 5)` at rc 120
here against the oracle's rc 0. **Task 6 owns those two witnesses** and its text says so. The eight
`of` rows still close here, because `method-bodies.txt` sends `of` with no arguments at all:
`arg(1,'a')` is empty, `arg() == 0`, and the body returns the empty collection at
`CoreClasses.orx:1246` without ever reaching a raise. Say in the report that the rows closed on the
zero-argument path and that the argument-error path is Task 6's, so the next reader does not read
eight closed rows as a fully exercised body.

**Walk `CoreClasses.orx:1238`-onward statement by statement before writing code** and confirm every
message it sends already works here: `self~new`, `arg()`, `args~last`, `args~hasIndex`, `args[i]`,
`arg~isA(.array)`, `.context~name`, and whatever the rest of the body does with the pairs. A single
unimplemented send in that body means this task's row count is wrong; report it rather than
absorbing it.

**Also in scope, because it is the same cause:** `Properties~setLogical` is the last non-stream
`send-differs` row in `corpus/collection-arity.tsv` and the scope note attributes it to this fix.
Confirm it moves. If it does not, the attribution was wrong and the report says so.

**Red control:** revert the `'A'` arm to the `Loud` refusal and confirm the eight `of` witnesses go
red. Then, separately, make the `'A'` arm answer a *hole-free* array (`.nil` in place of an empty
slot) and confirm the omitted-argument witness — and only that one — goes red. The second control
is the one that proves the witness set can see the hole, and per project memory a green run cannot
witness a failure message.

---

## Task 2 — the three classes with no constructible instance

Eight rows, one error site, and the smallest task in the phase.

**`Pointer~new` and `Buffer~new` raise `93.967`.** Measured on the oracle 2026-09-07:

```
Error 93 running <file> line 1:  Incorrect call to method.
Error 93.967:  NEW method is not supported for the Pointer class.
```

and the same with `Buffer`. The trace line above it reads
`*-* Compiled method "NEW" with scope "Pointer".` — assert the whole three-descriptor transcript,
not the error number alone.

Find the C++ that raises it and cite it, so the substitution (the class's id) is read rather than
guessed, and so it is clear whether any other class in the environment shares the site. Both
`Pointer` and `Buffer` are documented as native-code-only in the reference
(`utilityclasses.xml:429`, `:6910`).

**Pointer's other five rows — `=`, `==`, `\=`, `\==`, `isNull` — close by cascade and not by
implementation.** The instance arm's receiver is `.Pointer~new`, so with `new` raising identically
on both sides the row records agreement about the construction. **That is honest here and it is not
hollowness**, because the oracle has no path to a Pointer either; Phase 8 owns making one that
holds something. Say exactly that in the report, and say it again in the phase's close, because
`rust/CLAUDE.md`'s hollow-shell rule requires a phase that ships one to name it in the handover.
**Do not add bodies for those five.** A body nothing can reach is the shell that rule forbids.

**`WeakReference~value` is a real body, and it is bigger than one row.** Measured on the oracle,
rc 0: `.WeakReference~new(.Object~new)~value` answers `an Object`, and with the referent held in a
live variable `~value~class~id` is `Object`.

**The state and the collector already exist and nothing reaches them.** `Body::WeakRef(ObjRef)` is
declared (`rexx-core/src/body.rs:235`), `body.rs:741` gives it a trace arm that walks nothing, and
`rexx-core/src/heap.rs:166`-`:202` implements the whole weak protocol — `checkWeakReferences`
before `checkUninit`, clearing a dead referent to `Body::WeakRef(ObjRef::NIL)`. But
`native_weak_reference_new` (`dispatch.rs:10369`) builds a **plain instance** through
`new_instance` and drops the referent on the floor, and its own doc comment says so. Nothing in
either crate constructs a `Body::WeakRef`. So:

* `new` has to build a `Body::WeakRef` instead of a plain instance, which makes `heap.rs`'s weak
  path **live for the first time**;
* `receiver_kind` (`dispatch.rs:1923`) must stop answering `Err("a weak reference")` for it;
* and the doc comment at `dispatch.rs:10364`-`:10368` states the old contract and becomes false —
  correct it rather than leaving it, per `rust/CLAUDE.md`'s rule about comments that state
  something false.

**The witness is the one the type exists for**: a reference whose referent is collectable answers
`.nil` after a collection, and one whose referent is still held answers the object. `collect_stress`
is the harness. **A test that only ever asks a live reference is satisfied by an implementation
that ignores weakness entirely** — that is the degenerate implementation this row exists to
exclude, and it is the shape this crate would have shipped, since the plain-instance version passes
every live-reference probe.

Run the dead-referent witness against the code **before** your change as well: it should fail
differently (the row is loud today), and if it passes, the witness is not testing what you think.

---

## The shape every remaining task shares, and where its loud refusal lives

Read this once; the tasks below do not repeat it.

**Every class in this phase is already registered and already has an instance representation.**
`crates/rexx-classes/src/native_classes.rs`'s `CHECKLIST_TO_DEFINITION` carries `Pointer`, `Buffer`,
`Method`, `Routine`, `Package`, `RexxContext`, `WeakReference`, `StackFrame` and `RexxInfo`, and
`crates/rexx-exec/src/environment.rs:408`'s `build_environment` wires them. `.RexxInfo` is already
an **instance** — built at `environment.rs:475`-`:482` as
`Body::Native(NativeObject::new(rexx_info_class, b"a RexxInfo"))` and put under `REXXINFO` at
`:489` — which is the parent plan's Phase 5 clause, and measured on both engines today
`.RexxInfo~class~id` is `RexxInfo`.

**So no task here creates a class. Every one of them adds readers.**

**The instance state they all share is one struct**: `NativeObject` (`rexx-core/src/body.rs:579`) —
a class, a rendered name, a `HashMap<Box<[u8]>, ObjRef>` of entries, an optional annotations table,
and an optional scope. That is the *whole* of what a `Package`, `Method`, `Routine`, `RexxContext`
or `RexxInfo` object carries today. **Where a task needs state that struct does not hold, adding it
is part of that task and it is the part to think about first**, because the collector walks these
objects and a field it cannot see is a use-after-free no small test produces (project memory
`oorexx-rust-fresh-allocations-in-a-closure`, and 5g's own D101 witness `collect_stress`).

**The loud refusal every row in this phase hits is one site**: `invocable`
(`crates/rexx-exec/src/dispatch.rs:2504`) falls through natives, method bodies, generated and
native-externals to `Err(Loud::native_method(name, &scope))` at `:2529`. A row closes by adding an
entry to `NATIVE_METHODS` (`dispatch.rs:255`) or `NATIVE_CLASS_METHODS` (`:951`) that binds the
name to a body. Two rows in this phase do *not* reach that site and are called out where they
occur: `Class`'s six operators, and `WeakReference~value`, whose receiver kind
(`receiver_kind`, `dispatch.rs:1893`) currently returns `Err` for `Body::WeakRef` at `:1923`.

**The 5f/5g registration control is available to every task and each one runs it.** A row naming a
method the class does not answer must panic at `ObjectModel::build` (`dispatch.rs:1200`, `:1205`),
and a row binding a real loud name to the wrong body must make the send reach that body. Remove
both and confirm the name goes loud again.

---

## Task 3 — `RexxInfo`

Twenty-eight rows, no dependencies, and the largest single-class block in the phase. It runs early
because nothing waits on it and because it is where this plan's first NEW constraint bites hardest.

**Measured on the oracle 2026-09-07, all twenty-eight at rc 0**, on this machine:

```
architecture=[64]                 majorVersion=[5]
caseSensitiveFiles=[1]            maxArraySize=[100000000000000000]
date=[30 Jul 2026]                maxExponent=[999999999]
debug=[0]                         maxPathLength=[4096]
digits=[9]                        minExponent=[-999999999]
directorySeparator=[/]            modification=[0]
endofline=[<a newline>]           name=[REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026]
executable=[…/ooRexx/build/bin/rexx]   package=[The REXX Package]
form=[SCIENTIFIC]                 pathSeparator=[:]
fuzz=[0]                          platform=[LINUX]
internalDigits=[18]               release=[3]
internalMaxNumber=[999999999999999999]     revision=[0]
internalMinNumber=[-999999999999999999]    version=[5.3.0]
languageLevel=[6.06]
libraryPath=[…/ooRexx/build/lib]
```

**`digits`, `form` and `fuzz` are the DEFAULTS and not the settings in force.** Measured: after
`numeric digits 5; numeric form engineering; numeric fuzz 2`, `.RexxInfo~digits .RexxInfo~form
.RexxInfo~fuzz` still prints `9 SCIENTIFIC 0`. An implementation that reads the activation's
current `NUMERIC` state agrees with the oracle on every probe that does not change them, which is
every probe anyone writes by accident. **The witness must change them first**, and that witness is
this task's proof that the reader reads the right state.

**`name` is exactly the `PARSE VERSION` string.** Measured, `.RexxInfo~name = v` after
`parse version v` is `1`. This crate already produces that string; read it from wherever `PARSE
VERSION` reads it rather than writing a second copy, and assert the equality in the witness so the
two can never drift.

**`version`, `majorVersion`, `release`, `modification`, `revision`, `languageLevel` and `date` come
out of the same version constants.** Find them, cite them, and read them. A literal `"5"` in the
`majorVersion` body is the defect this plan's first NEW constraint names: it agrees with the
differential and is not an implementation.

**Two rows cannot agree with the oracle, and this task must not pretend otherwise.**
`executable` and `libraryPath` answer the path of the *running interpreter's own file*. The oracle
answers its `bin/rexx` and its `lib`; this crate would answer `rexx-run` and wherever its library
lives. They are different files, so the honest implementation **diverges by construction**.

`corpus/method-bodies.txt`'s gate is one rule — *a row that was not diverging may not start* — so
implementing these two correctly turns two `loud` rows into two `diverge` rows and reddens the
gate. **This is a decision, not a coding problem, and it is Moritz's.** Put it to him before
writing either body, with these three options and your recommendation:

1. leave both rows `loud`, declined in the report with this reason, and note them in the close as
   the phase's only declined rows;
2. implement both and license the divergence by name in `method_bodies.rs`, in the shape of the
   existing `ORACLE_CRASHING_SENDS` list — which is guarded so that an entry naming a row that does
   not exist is itself a failure, and any new list must carry the same guard;
3. implement both and have the harness compare something other than the raw string, which asserts
   nothing and is named here only so it is rejected explicitly rather than by silence.

**Everything else in the list is a real reader.** `platform`, `architecture`, `directorySeparator`,
`pathSeparator`, `endofline`, `caseSensitiveFiles`, `maxPathLength` are platform facts — find the
C++ that produces each (`SystemInterpreter`, the `Rexx*.h` platform headers) and mirror the source
of the value, not the value. `internalDigits`, `internalMaxNumber`, `internalMinNumber`,
`maxExponent`, `minExponent`, `maxArraySize` are numeric-core limits this crate already enforces
somewhere — read them from the same constants, and assert in the witness that the answer agrees
with what the interpreter actually does (`.RexxInfo~maxArraySize` against what `.Array~new` of one
more refuses at, `maxExponent` against the exponent an overflow reports).

`debug` and `package` are the two to measure rather than reason about.

**Red control:** for each of `digits`, `form`, `fuzz`, change the reader to the activation's live
setting and confirm the `numeric`-changing witness goes red and nothing else does. For `name`,
break the shared source and confirm both the `PARSE VERSION` corpus programs and this witness go
red — one reader, two consumers.

---

## Task 4 — `Method` and `Routine`

Twenty-four rows, and the task the three after it depend on.

`Method` (16): `loadExternalMethod` and `newFile` on the class arm; `isAbstract`, `isAttribute`,
`isConstant`, `isGuarded`, `isPackage`, `isPrivate`, `isProtected`, `package`, `setGuarded`,
`setPrivate`, `setProtected`, `setSecurityManager`, `setUnguarded`, `source` on the instance arm.
`Routine` (8): `loadExternalRoutine` and `newFile` on the class arm; `[]`, `call`, `callWith`,
`package`, `setSecurityManager`, `source` on the instance arm.

**What exists.** `Interp::method_object` (`environment.rs:1369`) already builds a `.Method`
instance, sets its scope, and caches it keyed by (class, name); `compile_method_source`
(`dispatch.rs:5062`) builds one and `record_compiled_body` (`lib.rs:6501`) files the parsed program
under it; `method_scope` (`environment.rs:1686`) reads `NativeObject::scope`. `ANNOTATION`,
`ANNOTATIONS` and `SCOPE` already answer for `Method`; `ANNOTATION` and `ANNOTATIONS` for
`Routine`; both have a class-side `NEW` through `native_executable_new`.

**So the link from a `Method` object to its parsed body already exists, and that is what most of
these sixteen rows read.** `~source` is the body's own source lines as an Array; the seven `is*`
flags are properties of the directive that declared it; `~package` is the package object the
`method_object`/`package_object` pair already keys on. Confirm each of those claims against the
tree before writing, and report any that is not true — the row count depends on it.

**The `set*` family is where the trap is.** `setGuarded`, `setUnguarded`, `setPrivate`,
`setProtected` change a flag that something must *observe*, and the observable is the matching
`is*` reader plus the interpreter's own behaviour. **Pair every setter with its reader in one
witness** — set, then read back, then send the method and see the behaviour change where it can be
seen. A setter whose only witness is "it did not raise" is the shell `rust/CLAUDE.md` forbids.

**`setSecurityManager` is the one row in this family that cannot be witnessed through behaviour.**
Measured on the oracle: `p~setSecurityManager` with no argument and `p~setSecurityManager(.Object~new)`
both answer `1`, and nothing in the language reads the manager back. D12's interception points are
Phase 5's exit-gate work with no delivery evidence, and Phase 7 adds the rest. **Store the object on
the receiver and answer `1`** — real state, and the reason the state is unobservable today goes in
the report naming D12 as its owner. If you think that is the forbidden shell rather than a kept
state nothing yet reads, **ask before implementing**; it is a fair reading and the decision should
be made once, out loud, for all three classes that carry the row.

**`Routine~call` and `~callWith` actually run code**, and `~[]` is `call` under another name. They
are the only rows in this phase that are not readers. Their witnesses assert the return value, the
argument passing including an omitted argument, and — the second send this plan's NEW constraint
requires — that calling the same `Routine` object twice works, since a body that consumes its
program on first use passes a one-send harness.

**`newFile`, `loadExternalMethod` and `loadExternalRoutine` read the file system.** Measure what
the oracle does for a file that does not exist before deciding what these answer; if the answer
needs a stream this phase does not have, that is a Phase 7 hand-off and the report says so with the
measurement attached rather than the row being quietly closed.

---

## Task 5 — `Object`'s three, and `Class`'s eleven

Fourteen rows. Depends on Task 4 for `Method` objects.

**`Object~instanceMethod`, `~instanceMethods`, `~isInstanceOf`.** The first answers a `Method`
object, the second a `Supplier` over the receiver's behaviour. Both are what
`crates/rexx-exec/tests/collection_scopes.rs` sends to the **oracle** to derive its scope column —
so this crate implementing them changes nothing about that instrument, and the instrument is a
worked reference for what the answers mean. Per this plan's second NEW constraint, the witness for
`instanceMethod` sends a **second** message to the `Method` it gets back (`~scope~id` is the
obvious one, and its answer is measurable on the oracle).

**`Class`'s eleven split into two unrelated halves, and only one of them is introspection.**

*The six operators* — `=`, `==`, `<>`, `><`, `\=`, `\==` — are loud at a different site from every
other row in the phase, and it is **one** site: `Interp::operator_operand_gap`
(`crates/rexx-exec/src/eval.rs:1812`) answers `Some("a class object")` at `:1819` for every class
handle, and every operator path asks it first (`:1007`, `:1276`, `:1575`, `:1692`). So the six rows
are one arm, not six bodies.

**The arm cannot simply be deleted, and that is this half's whole difficulty.** Measured on the
oracle 2026-09-07:

```
.array = .array                 1        .array \== .array          0
.array = 'The Array class'      0        .array <> .string          1
.array == .string               0        .array >< .string          1
.array > .array      97.1  Object "The Array class" does not understand message ">".
.array + 1           97.1  Object "The Array class" does not understand message "+".
```

So the six comparisons answer `Object`'s **identity** test — note `.array = 'The Array class'` is
`0`, not a comparison of renderings — while every other operator is `97.1`, the oracle's own
no-such-method. This crate refuses **all** of them loudly today, and `eval.rs`'s existing test
around `:3705` pins that: it asserts `=`, `>`, `+` and `**` on `.array` are all loud with the noun
`a class object`. **That test has to change, and how it changes is the deliverable** — the six
comparisons stop being loud, and `>`, `+`, `**` must not silently become loud-for-a-different-reason
or start answering. Whether `97.1` for the arithmetic operators is in scope here or stays loud is
yours to measure and rule on; say which, and why, in the report.

*The five readers* — `isAbstract`, `isMetaclass`, `methods`, `queryMixinClass`, `subclasses`.
Measured on the oracle: `.Array~isMetaclass` is `0`, `.Object~isAbstract` is `0`,
`.Object~subclasses` is an **Array**, `.Array~methods` is a **Supplier**. `subclasses` and
`methods` are the two that need real state — a class's subclass list and its method table — and
both must be witnessed by a second send that reads an element out.

**`subclasses` has a hazard worth naming before it is discovered.** A class's subclass list is a
set the collector must not keep alive by itself, and the oracle's own list is weak; check what
`ClassClass`'s subclass list does with a collected subclass and write the witness for whichever it
is. If this crate has no subclass list at all, building one is the bulk of this task and the report
says so.

---

## Task 6 — `RexxContext` and `StackFrame`

Twenty-four rows. Depends on Task 4: `RexxContext~executable` answers a **`Routine`**.

**Measured on the oracle 2026-09-07**, from inside an internal routine `R` called with two
arguments from a program whose `.context` is also read:

```
args=Array(2)   condition=The NIL object   digits=9  form=SCIENTIFIC  fuzz=0
executable=Routine   interpreter=String   invocation=String   line=11   name=R
rs=The NIL object    stackFrames=Array(2)  thread=1   variables=Directory
```

and for `.context~stackFrames[1]`:

```
arguments=Array   context=RexxContext   invocation=String   line=17   name=R
target=The NIL object   type=INTERNALCALL
traceLine / string / makeString = "    17 *-*   f = c~stackFrames[1]"
```

**Four things that measurement settles and that a task written without it would get wrong:**

* `RexxContext~digits`/`form`/`fuzz` are the settings **in force at that context**, unlike
  `RexxInfo`'s, which are the defaults. Two classes, two meanings, one method name — the witness
  changes them and asserts both classes in the same program.
* `~name` is the *routine's* name inside a routine and the **program's file path** at the top
  level. Measured both. This is the row `Task 1` is blocked on, below.
* `~stackFrames` is an Array of `StackFrame`, and a `StackFrame`'s `~traceLine`, `~string` and
  `~makeString` are the same string — the caller's clause as `TRACE` would print it. This crate
  already builds those lines (`corpus/lang`'s trace programs and the `sourceline_oracle` fixtures
  depend on them); read them from the same place rather than re-rendering.
* `~variables` is a `Directory` of the context's variable pool, and `~args` an Array with the same
  hole semantics as Task 1's `arg(1,'A')`. Send them a **second** message.

**This task owes Task 1 two witnesses.** `MapCollection~of`'s two argument raises —
`raise syntax 88.923 array(.context~name, i, arg)` and `88.924`, `CoreClasses.orx:1258`, `:1263` —
send `.context~name`, which is loud until this task lands. Task 1 closes the eight `of` rows without
them because the zero-argument send never reaches those lines; **this task writes them**, and its
report says whether the crate reaches the oracle's messages once `~name` answers.

**The activation stack exists and is already walkable — do not build a second one.** Read against
the tree 2026-09-07:

* `Interp::running: Option<Box<Activation>>` and `Interp::suspended: Vec<Box<Activation>>`
  (`crates/rexx-exec/src/lib.rs:3038`) are the stack, **oldest first**, so `suspended.last()` is the
  running activation's own caller — the field's own doc says so. `Interp::collect_now`
  (`lib.rs:~7162`) already walks exactly that chain.
* **Each activation already carries its own `RexxContext` object**: `Activation::context_object`
  (`crates/rexx-exec/src/activation.rs`, the field around `:816`), created by
  `Interp::context_object`, and `collect_now`'s comment calls it "the one object an activation owns
  outright". `RexxContext~package` already answers through this path (`dispatch.rs:849`-`:854`).
* `Activation` (`activation.rs:405`) carries `program`, `program_id`, `body`, `plan`, `entry`,
  `call_type: CallType`, `method_identity`, `settings: Settings`, `pc`, `trace_entry` and
  `condition`. Between them those are the fields every row in this task reads. Find each and cite
  it in the report rather than adding a parallel field.

**`CallType`'s spellings are not `StackFrame~type`'s.** `CallType` (`activation.rs:~904`) is
`Command`, `Subroutine`, `Function`, `Method`, `Requires`, rendering as `COMMAND SUBROUTINE
FUNCTION METHOD REQUIRES` — that is `PARSE SOURCE`'s vocabulary. The oracle's `StackFrame~type` for
an internal routine call is `INTERNALCALL`. So there is a **mapping** to work out, and the set of
values `~type` can take has to come from the C++ (`StackFrameClass.cpp` and whatever fills it), not
from `CallType`. Measure at least a program frame, an internal-call frame and a method frame.

**The two questions this task still has to answer, and neither is answered above:** what a
`StackFrame` answers when it outlives the frame it describes, and what the collector sees of it.
Measure the oracle for the first — keep `.context~stackFrames[1]` in a variable, return from the
routine, read it — before choosing a representation. `Activation::object_roots` is named in
`collect_now`'s comment as the other route for a parked activation's objects; read it.

---

## Task 7 — `Package`, the source and settings half

Twelve rows: `source`, `sourceLine`, `sourceSize`, `digits`, `form`, `fuzz`, `trace`, `options`,
`prolog`, `loadLibrary`, `setSecurityManager` on the instance arm, and `defaultOptions` on the class
arm.

**Measured on the oracle 2026-09-07** on a 28-line program with a `::routine`, a `::class` and a
`::resource`:

```
source=Array(28)   sourceLine(1)=[p = .context~package]   sourceSize=28   sourceLine(99)=[]
digits=9  form=SCIENTIFIC  fuzz=0  trace=N
options=String, rendering as
  "::OPTIONS DIGITS 9 FORM SCIENTIFIC FUZZ 0 NUMERIC NOINHERIT ERROR CONDITION FAILURE
   CONDITION LOSTDIGITS CONDITION NOSTRING CONDITION NOTREADY CONDITION NOVALUE CONDITION
   PROLOG TRACE NORMAL"
prolog=Routine
loadLibrary('rxmath')=1     setSecurityManager()=1     setSecurityManager(.Object~new)=1
.Package~defaultOptions('DIGITS') = the same ::OPTIONS string as ~options
```

`~options` answering a String and `defaultOptions` apparently ignoring its argument are both
surprising; **re-measure them yourself before implementing**, with more than one option name and
with a package that carries a real `::OPTIONS` directive, and implement what you measure.
`defaultOptions` also raises `88.901 Missing argument; argument optionName is required` when sent
none.

**`~source`, `~sourceLine` and `~sourceSize` read the program's source text, which this crate
already keeps** — `SOURCELINE` answers from it, and `crates/rexx-parse/tests/sourceline_oracle/`
holds a fixture per corpus program. Read the same store. `sourceLine` past the end answers the null
string, not a raise; `~source` is an Array, so send it a second message.

`~prolog` answers a `Routine`, which Task 4 built.

`setSecurityManager` is Task 4's decision, applied here unchanged. `loadLibrary` answers `1` for a
library on this build's path; measure what it answers for one that is not there before deciding
whether it is a Phase 7 hand-off.

---

## Task 8 — `Package`, the tables and lookup half

Twenty-one rows: `classes`, `definedMethods`, `importedClasses`, `importedPackages`,
`importedRoutines`, `namespaces`, `publicRoutines`, `resources`, `routines`, `resource`,
`findClass`, `findNamespace`, `findProgram`, `findPublicClass`, `findPublicRoutine`, `findRoutine`,
`addPackage`, `addRoutine`, `addPublicRoutine`, `loadPackage` and `publicClasses` on the instance
arm. Check that list against `method-bodies.txt`'s `loud` rows yourself before starting and correct
the plan if it has drifted.

**`publicClasses` is the odd one and it is worth reading first.** It is the only `Package` row whose
`loud` evidence is not `method "X" of class "Package"`: the table records
`the REXX package's class table`, so the name **is** bound (`dispatch.rs:820`-`:845`) and the body
refuses for this receiver. Find out what receiver it does answer for and what it refuses on, before
writing anything — that answer is likely to be the shape of the state the other twenty rows need.

**Measured on the oracle 2026-09-07**, same program:

```
classes=StringTable(1)          definedMethods=StringTable(0)
importedClasses=StringTable(0)  importedPackages=Array(0)
importedRoutines=StringTable(0) namespaces=StringTable(0)
publicClasses=StringTable(1)    publicRoutines=StringTable(1)
resources=StringTable(1)        routines=StringTable(1)
name=<the program's path>
```

Every table is a `StringTable` except `importedPackages`, which is an **Array**. All of them are
classes Phase 5h built, so the containers exist; what this task adds is the state behind them.

**`ADDCLASS`, `ADDPUBLICCLASS`, `LOCAL`, `NAME` and `PUBLICCLASSES` already answer.** Read those
five bodies first — they are the worked example for where a package's state lives and how it is
reached, and one of them (`ADDCLASS`) is the write side of a table this task adds the read side of.

**The measurement above is the one to distrust.** Every count is 0 or 1, so a body answering an
empty table of the right class agrees with the oracle on six of these rows. **The receiver Task 0
commits must carry more than one of each**, and every witness here reads an entry back out by name
— that is this plan's second NEW constraint and this is the task it was written for.

`addPackage`, `addRoutine`, `addPublicRoutine` and `loadPackage` are writes; each is witnessed by
the read that should see it, in the same program.

`findProgram`, `findNamespace` and the four `find*` lookups have a defined search order that is not
guessable — read `PackageClass.cpp` for each, and measure the miss case as well as the hit.

---

## Task 9 — the sweep, and close

**No new bodies.** Its subject is what a per-task review structurally cannot see.

1. **Refresh both instruments and read them against the phase's claim.**
   `corpus/method-bodies.txt` under `REXX_METHOD_BODIES_REFRESH=1` and
   `corpus/introspection-arity.tsv` under Task 0's refresh variable. Report the `loud` count per
   class before and after the phase, and **name every row that did not close and why**. The scope
   note's figure to close against is 139.
2. **Run the gate's one rule in anger**: no row that was not diverging may have started diverging,
   and no row that was answering may have stopped. `RexxInfo~executable`/`~libraryPath` are the
   known exception and carry Task 3's decision.
3. **The hollowness sweep.** For every row this phase closed, ask the question
   `rust/CLAUDE.md`'s rule asks: what degenerate implementation satisfies its witness, and would
   deleting the state it reads leave it green? Report the rows where the answer is uncomfortable.
   The classes to look at hardest are the ones whose oracle answer is an empty container or a
   constant — `Package`'s tables, `RexxInfo`'s platform facts, the three `setSecurityManager` rows.
4. **The handover, which is a deliverable and not a formality.** It names, for the next phase to
   read: `Pointer` and `Buffer` as classes with no constructible instance and Phase 8 as the owner;
   `Message` and the two semaphores as Phase 6's; `Stream`, `File` and `RexxQueue` as Phase 7's,
   with `RexxQueue` additionally gated on D7; and any row this phase declined.
5. **Update the scope note in place** with what the phase actually cost against what it was scoped
   at, and say which of its predictions were wrong. The prediction most likely to be wrong is "the
   cheapest first move is `ARG` option `A`, nine rows on one fix" — it is measured, so if it did not
   hold, that is the finding.

---

## What this plan does not cover

**Phase 5's own exit gate has never been assessed**, and the scope note records that closing the
phase is blocked on that rather than on these rows. It is not this plan's work and not any task
below. The close report names it as the remaining blocker, with the parent plan's exit clauses that
have no delivery evidence: security-manager interception points (D12), cold start measured against
C++ (D2), and rung L2.
