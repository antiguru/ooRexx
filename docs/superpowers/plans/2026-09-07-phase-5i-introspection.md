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
* **`memcap 8G` cannot wrap the release fast check on this machine, and it fails looking like a test
  failure.** Measured 2026-09-08: `memcap 8G cargo test --release --workspace --no-fail-fast` is
  exit **137** with **zero** `test result` lines, `memcap: OOM-killed at the 8G cap (peak 8.0G)`, and
  `Compiling rexx-exec` as the last thing on stderr -- the cap killed the *compile*, not a test.
  Without the cap it runs. G3 has no `memcap` and G4 does, which is the right split; the reason to
  write it down is that "exit 137, no test results" reads exactly like a red gate.
* Every family's witness sends a **short argument list as well as a good one**.
* Every family generates a `crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, and adds its file
  to `dispatch_seam.rs`'s `CLEARANCE_CONSUMERS` **only if it is a source file naming the `Cleared`
  seam token** -- a builtin in `builtin/state.rs` takes no clearance and must not be added.
* **A task commits the shared artifacts its change moves**: `corpus/method-bodies.txt`, the arity
  tables, `corpus/phase-5c.txt`, `corpus/unfiled.txt` and `EXPECTED_SUBSET_5C` in
  `crates/rexx-exec/tests/coverage.rs`. Refresh them, and record each moved row's verdict **before
  and after** your change in your report with the refresh command quoted -- that is the evidence,
  and it is required whether or not the table is where it is filed.
  (A constraint saying the controller owned those four was in force between `a85183fbb` and Task 1's
  pre-flight. It existed to let two implementers run at once; the decision to run implementers
  serially -- because two agents changing interpreter behaviour contaminate each other's local
  refresh whatever the commit boundary -- removed its only justification, and it stayed in force
  long enough to make one task's commit red by construction against five test binaries that assert
  the committed tables match the tree. Revoked.)
* **Your dispatch names the files you may touch; touch nothing outside it.**
* **Formatting, while a sibling is live in the same crate.** `cargo fmt -p <crate>` is package-wide
  and reformats a sibling's uncommitted file, which has already happened on this project. But bare
  `rustfmt <path>` is the wrong replacement **in this tree**: without an edition it applies the 2015
  `use`-ordering and rewrites `use super::{NONE, Receiver, program}`, which `cargo fmt --all --check`
  then rejects -- measured 2026-09-08, and it nearly shipped. Use **`rustfmt --edition 2024 <path>`**
  to format one file and `rustfmt --edition 2024 --check <path>` to check it. `cargo fmt --all
  --check` is safe beside a sibling because it only reads.
* **No task may cite the `answers` verdict of `corpus/method-bodies.txt` as a reason a row needs no
  work.** Task 0's instrument is what a later task reads. A row this phase declines to do is
  declined in a report sentence naming why, not by a green cell.
* **The full gate suite runs in the gate worktree `/home/moritz/dev/repos/ooRexx-5i-gates`, not in
  the working tree** -- `git -C <that> checkout --detach <your commit>`, then the seven commands
  from `<that>/rust`. Both its builds are warm. `rust/CLAUDE.md`'s ordering rule is unchanged and
  still binds: **commit first, then gate**. What changes is that the working tree no longer belongs
  to the run, because a pinned worktree cannot drift -- the property the freeze rule protects is
  stronger here, and the next task does not wait behind you. Your own `fmt`, `clippy` and
  `cargo test --release --workspace --no-fail-fast` stay in the working tree, before the commit.
  Never touch `/home/moritz/dev/repos/ooRexx-gates` or `-gates-b`.
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

**Ruled during execution, from this task's own pre-flight: the introspection tables carry an `arm`
column and the collection table does not.** `corpus/collection-arguments.tsv` is instance-arm only —
its 44 `Array` rows are exactly `class-methods.txt`'s 44 `Array` *instance* rows — and copying that
shape would leave five of this phase's own rows unsized: `Package~defaultOptions`,
`Method~loadExternalMethod`, `Method~newFile`, `Routine~loadExternalRoutine`, `Routine~newFile`, all
named verbatim in Tasks 4 and 7. The shared module takes the arm column as a layout flag, off for
the collection driver, so `corpus/collection-arity.tsv` keeps its four columns and its byte-identity
control. **`Pointer` and `Buffer` are excluded per (class, arm) rather than per class**: only their instance
arm is excluded by name, with the reference citation as its reason, and their class-arm `new` row is
measured on the refusal that IS its documented behaviour.

**This ruling was given a false reason, the correction was itself partly false, and the implementer
found the answer both had missed.** The sequence is recorded because it is the phase's clearest
example of a defect class this project keeps hitting -- a correct decision shipping with a wrong
justification, and then a correction round introducing a new wrong statement.

* The ruling said the class-arm row "is measurable with receiver `.Pointer` and sizes Task 2".
* The correction said that was false: the oracle **raises** `93.967` on `.Pointer~new`, so the
  probe's `SYNTAX` trap fires, `SENT` is never printed, and the harness rule -- a row's list is real
  only if the oracle completes the send -- refuses it as a data point. It concluded that the rows
  could only be `EXEMPT:`, and that **nothing this instrument can do sizes Task 2**.
* Both halves of that conclusion were wrong. An `EXEMPT:` row indeed cannot move -- but the harness
  rule can be **inverted rather than waived**. `REFUSED:` marks a row the oracle refuses *by
  design*; the row is still compared on all three descriptors, so it reads `send-differs` today
  (oracle `SYNTAX 93.967` against this crate's loud refusal) and becomes `agree` the moment Task 2
  makes this crate raise the same thing. It moves, and it sizes Task 2 after all.
* And the marker cannot become a hiding place, because the inversion is asserted:
  `every_refused_row_is_really_refused` fails on a `REFUSED:` row the oracle **does** complete.

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

**`dimension` is the second detail a naive implementation gets wrong, and it does NOT split on
"past the end" -- it splits on whether the position is 1.** `BuiltinFunctions.cpp:934` tests
`position == 1` **before** `:938`'s `position > size`, so position 1 always takes
`new_array(size, arglist)` and never the empty-array limb. Measured on the oracle 2026-09-08 from a
call with **no arguments at all**:

```
arg(1,'A')   size=0  items=0  dimension=1
arg(2,'A')   size=0  items=0  dimension=0
```

**And position 1 on an empty argument list is exactly the path all eight `of` rows take**, because
`corpus/method-bodies.txt` sends `of` with no arguments. So the two limbs are: position 1 gets
`native_array_of`'s own shape -- `dimensions: args.is_empty().then(|| [0])` -- and every other
position past the end gets `dimensions: None`. Witness both; a single rule for "past the end" gives
every row this task exists to close a `~dimension` of 0 where the oracle answers 1.

(An earlier version of this paragraph said the past-the-end case "wants `dimensions: None`" and
offered `arg(4,'A')` as its case. That is true for position != 1 and false for position 1, which is
the case the phase's own rows take. Task 1's pre-flight caught it before any code was written.)

The rest of `native_array_of` (`dispatch.rs:6509`) is the template -- `slots: args.to_vec()` copies
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

**The sub-number is `967` and it was measured, not read.** A survey of
`interpreter/messages/rexxmsg.xml` reported `Error_Unsupported_new_method` as `93.968` — that is
the *next* `<SubMessage>` block's subcode, read one entry down. Run it yourself before writing the
witness and use what the interpreter prints. `.StackFrame~new` shares this site; see Task 6.

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
`new_instance` and drops the referent on the floor, and its own doc comment says so.

**No `src/` file constructs a `Body::WeakRef`** -- but `rexx-core/tests/uninit.rs` builds one by
hand, so the weak-clearing pass is not untested; what is missing is any route to it from a Rexx
program. (An earlier version of this paragraph said "nothing in either crate constructs one". That
was a grep of `crates/*/src/` and it is false of the test tree. Task 2's implementer found it when
deleting the pass reddened `uninit.rs` as well as its own witness.) So:

* **The reference object stays a `Body::Instance`, and the weak cell goes in its scope pool.** Ruled
  2026-09-08 after Task 2's pre-flight measured that the obvious shape regresses a green case:
  `Body::WeakRef` carries no class, behaviour, `ScopePools` or name, and a `WeakReference` SUBCLASS
  keeps all four today -- `::class W subclass WeakReference` with an attribute answers `W`, `a W`
  and its own attribute, byte-identical on the oracle and both engines. So `new` additionally
  allocates a one-word cell whose body is `Body::WeakRef(referent)` and stores it in the instance's
  pool for the `WeakReference` scope. The instance's trace walks the pool, so the cell lives exactly
  as long as the reference object; the cell's own body traces nothing, so the referent is not
  marked; `heap.rs`'s `checkWeakReferences` rewrites it to `Body::WeakRef(ObjRef::NIL)` when the
  referent dies. **The already-written protocol runs unchanged and becomes reachable from a Rexx
  program for the first time**, and `value` reads the cell with no branch because `NIL` decodes as
  `.nil`. This is the crate's existing idiom, not an invention: `COLLECTION_STORES`
  (`dispatch.rs:7485`) keeps `Array`'s `ITEMS` and `Table`'s `HASHINDEXES` the same way.
* `receiver_kind` (`dispatch.rs:1923`) keeps its `Err("a weak reference")` arm, and it stays
  unreachable from a program because the cell is never handed out. Say why that is correct rather
  than a leftover.
* and the doc comment at `dispatch.rs:10364`-`:10368` states the old contract and becomes false --
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

**`name` is exactly the `PARSE VERSION` string, and this crate already has it in one place.**
Measured, `.RexxInfo~name = v` after `parse version v` is `1`. The constant is
`crates/rexx-exec/src/parse_template.rs:97`:

```rust
const VERSION: &[u8] = b"REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026";
```

**Read that, do not copy it.** Its own doc comment says why: it is a claim about the oracle
*binary*, its third field is the oracle's build date which nothing here can derive, and
`tests/parse_version_oracle.rs` is the only thing that can notice it going stale — it runs `parse
version` through both interpreters under `REXX_CORPUS_GATE`. A second copy of the string would go
stale on the next oracle rebuild with that harness still green for the first copy. Assert
`.RexxInfo~name = v` in this task's witness so the two readers can never drift apart.

**`version`, `majorVersion`, `release`, `modification`, `revision`, `languageLevel` and `date` are
fields of that same string** — `5.3.0`, `5`, `3`, `0`, `0`, `6.06`, `30 Jul 2026`. Decide whether
they are parsed out of it or whether the constant is restructured into components that the
`PARSE VERSION` renderer then assembles, say which you chose and why, and either way keep **one**
source. A literal `"5"` in the `majorVersion` body is the defect this plan's first NEW constraint
names: it agrees with the differential and is not an implementation.

**`revision` and `modification` are both `0` on this build**, which makes them indistinguishable
from each other and from a stub. Read `RexxInfo`'s C++ (`Setup.cpp:1735`-`:1737` builds the
instance; find the methods) and say which field of the version each one is, so the two are right
for a build where they differ rather than right by coincidence here.

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

**Plus one row Task 0 found and the scope note does not count: `Routine~new` on the class arm.**
`corpus/method-bodies.txt` calls it `answers`, because a zero-argument send agrees about an arity
error; `corpus/introspection-arity.tsv` calls it `send-differs` under
`.Routine~new('R9', 'return 1')`. Ruled into this task 2026-09-07 because this phase's own
witnesses are already written against it -- Task 8's `addRoutine` witness is
`.Routine~new('NEWR', 'return 42')`. **Twenty-five rows.**

**What exists.** `Interp::method_object` (`environment.rs:1369`) already builds a `.Method`
instance, sets its scope, and caches it keyed by (class, name); `compile_method_source`
(`dispatch.rs:5062`) builds one and `record_compiled_body` (`lib.rs:6501`) files the parsed program
under it; `method_scope` (`environment.rs:1686`) reads `NativeObject::scope`. `ANNOTATION`,
`ANNOTATIONS` and `SCOPE` already answer for `Method`; `ANNOTATION` and `ANNOTATIONS` for
`Routine`; both have a class-side `NEW` through `native_executable_new`.

**`Routine` is the asymmetric half and its rows cost more than `Method`'s.** Read against the tree
2026-09-07: a `Method` object reaches its body through `table_method_bodies`
(`lib.rs:3432`, filled at `environment.rs:875`-`:884` for `.methods` entries and at `lib.rs:6501`
for compiled ones), or through `method_objects` → `own_instance_slot` → `MethodSlot::Defined` →
`method_bodies` (`lib.rs:3364`). **A `Routine` object built by `.routines~<name>`
(`environment.rs:1906`) gets `body: None` and no `table_method_bodies` entry at all** — its only
link back to its directive is the annotation key. So `Routine~source`, `~call`, `~callWith` and
`~[]` need that link built, and that is the design work in this task. Confirm it before you plan
around it; if the link is there by another route, say which and the task gets smaller.

`Package`'s receiver is `.Class~package`, the REXX package — see the section before Task 7 — so
`Method~package` and `Routine~package` should be checked against **both** a program's package and
that one.

**The link from a `Method` object to its parsed body already exists, and that is what most of
these sixteen rows read.** `~source` is the body's own source lines as an Array; the seven `is*`
flags are properties of the directive that declared it; `~package` is the package object the
`method_object`/`package_object` pair already keys on. Confirm each of those claims against the
tree before writing, and report any that is not true — the row count depends on it.

**Measured on the oracle 2026-09-07.** On `class-set.txt`'s own receiver,
`m = .Object~method('objectName')` — a **native** method:

```
scope=Object   package~name=REXX   source=Array(0)
isAbstract=0  isAttribute=0  isConstant=0  isGuarded=1  isPackage=0  isPrivate=0  isProtected=0
```

and on a Rexx one, `mm = .K~method('M')` from `::method M unguarded private`:

```
scope=K   package~name=<the program's path>   source=Array(1)   source[1]=[  return 1]
isGuarded=0   isPrivate=1   isProtected=0
```

**Two things that block a body from agreeing by accident.** The measured receiver's `~source` is an
**empty Array** and every one of its seven flags but `isGuarded` is `0`, so a reader answering an
empty array and seven zeroes agrees with the oracle on eight rows — witness the Rexx method too,
where the source has content and the flags disagree with each other. And `isGuarded` is the one
flag whose default is `1`, which is what tells a reader of the flags apart from a reader of
nothing.

**The four setters return no result at all.** Measured, `say m~setGuarded` is
`91.999 Message "SETGUARDED" did not return a result.` at rc 165 — so the body answers `None`, not
a value, and a witness must send them as statements. Sent that way they work and are observable
through their readers: `setUnguarded` then `isGuarded` is `0`, `setGuarded` then `1`, `setPrivate`
then `isPrivate` `1`, `setProtected` then `isProtected` `1`.

**`Routine~call`, `~callWith` and `~[]` measured**, on `.routines~rr` where `rr` is
`use arg n; return n * 2`: `~call(3)` is `6`, `~callWith(.Array~of(4))` is `8`, `~'[]'(5)` is `10`,
and `~call(6)` **again** on the same object is `12` — that last is the second send this plan's NEW
constraint asks for, and it is what catches a body that consumes its program on first use.
`~source` is `Array(2)` with `source[1]` = `  use arg n`, leading blanks included.

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

**There are two loud sites for these six names, not one, and they are reached by different
syntax.** The operator path above is what `(.Array = .Array)` as an *expression* takes. A
**message send** of the same name — `.Array~'='(.Array)` — takes `invocable` instead, and the row's
own evidence in `method-bodies.txt` is `method "=" of class "Class"`, which is
`Loud::native_method` at `dispatch.rs:2529`. That is the site the eight rows in the table are
measured at, because the harness sends a message. `Setup.cpp`'s `Class` block gives `Class` its own
`AddMethod("=", RexxClass::equal, 1)` and siblings, so the name resolves to a **Class-scoped**
method id distinct from `Object`'s, and `NATIVE_METHODS` binds `=` only at `"Object"`
(`dispatch.rs:756`). **Close both, and assert both spellings in the witness** — a fix at one site
leaves the other loud and the row moves or does not depending on which the harness happens to use.

So the six comparisons answer `Object`'s **identity** test — note `.array = 'The Array class'` is
`0`, not a comparison of renderings — while every other operator is `97.1`, the oracle's own
no-such-method. This crate refuses **all** of them loudly today, and `eval.rs`'s existing test
around `:3705` pins that: it asserts `=`, `>`, `+` and `**` on `.array` are all loud with the noun
`a class object`. **That test has to change, and how it changes is the deliverable** — the six
comparisons stop being loud, and `>`, `+`, `**` must not silently become loud-for-a-different-reason
or start answering. Whether `97.1` for the arithmetic operators is in scope here or stays loud is
yours to measure and rule on; say which, and why, in the report.

*The five readers* — `isAbstract`, `isMetaclass`, `methods`, `queryMixinClass`, `subclasses`.
**Measured on the oracle 2026-09-07**, on `class-set.txt`'s own receiver `k = .Object~subclass('k')`
and on a `::class KK` with two methods:

```
k~isAbstract=0   k~isMetaclass=0   k~queryMixinClass=0   k~subclasses=Array(0)
k~methods=Supplier, 32 entries -- ALL of Object's, not k's own
.KK~methods=Supplier, 34 entries: M1 and M2 at scope KK, the other 32 at scope Object
.Object~subclasses~items=53
o~isInstanceOf(.Object)=1   o~isInstanceOf(.String)=0
o~instanceMethod('OBJECTNAME') -> a Method whose ~scope~id is Object
o~instanceMethod('ZZZ') -> The NIL object
o~instanceMethods -> Supplier, 32 entries; with .Object as the argument, also 32
```

**Three things that measurement settles.** `~methods` answers the **whole** resolved set including
everything inherited, not the class's own dictionary — a body that answers only own-scope methods
gives `2` for `.KK` where the oracle gives `34`. `~subclasses` on a freshly made subclass is empty
while `.Object`'s is 53, so the empty answer proves nothing on its own. And `instanceMethod` of a
name the receiver does not have is `.nil`, not a raise.

**`Class~enhanced` is a silent wrong answer and this task owns it.** Found by Task 0 re-running its
own `agree` rows with the result printed, and confirmed independently by the controller on both
engines:

```
k = .Object~subclass('K'); st = .StringTable~new
st['EXTRA'] = .Method~new('EXTRA','return 99')
o = k~enhanced(st); say o~string; say o~class~id; say o~extra

oracle : enhanced K / K / 99
ir     : a K        / K / 99
tree   : a K        / K / 99
```

**The method works** -- `extra` answers `99` on both sides -- and only the *rendering* is wrong: an
enhanced object renders as `enhanced <id>` where this crate gives it the ordinary `a <id>`. It is a
wrong answer at rc 0, not a missing one, and no instrument in this tree saw it:
`corpus/method-bodies.txt` records the row `answers rc 163` and `corpus/introspection-arity.tsv`
records `agree`, which means only that neither side raised. Fix it here, and give it a witness that
reads the rendering rather than the send's success.

**The `Supplier`'s ORDER is hash order and no witness may assert it.** Measured, the oracle hands
back `M1` and `M2` interleaved among `Object`'s entries in neither alphabetical nor definition
order. Project memory `oorexx-hash-iteration-order` is the rule: string keys reproduce across runs
of the *same* interpreter, and nothing makes two different interpreters agree. So a corpus witness
that prints the supplier in the order it comes will diverge for a reason that is not a defect —
**sort, or assert membership and count**, and say in the witness's own comment which of the two it
does and why.

**The subclass list already exists**: `class_graph.rs:128`'s `subclasses: Vec<ObjRef>`, written by
the same path for `subclass()` and `inherit()` the way the oracle's `addSubClass` is, and read out
by `Registry::subclasses` (`rexx-classes/src/registry.rs:387`). So this row is a reader over state
that is there, not a new structure.

**Its hazard is what the list does with a collected subclass.** The oracle's is weak; this crate's
is a plain `Vec<ObjRef>`. Check what `ClassClass`'s list does when a subclass becomes unreachable
and write the witness for whichever the oracle does — and note that spec D59 says classes are never
collected here, so the answer may be that the question does not arise. Say which it is rather than
leaving it unstated.

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
FUNCTION METHOD REQUIRES` — that is `PARSE SOURCE`'s vocabulary, and it is **not** what `~type`
answers. Measured on the oracle 2026-09-07, from inside a `::routine` called by a `::method` called
by an internal label called by the program — four frames, **innermost first**:

```
frames=4
1  type=ROUTINE       name=INNER2  line=12  invocation=1  target=The NIL object
2  type=METHOD        name=M       line=9   invocation=2  target=a KK
3  type=INTERNALCALL  name=OUTER   line=5   invocation=3  target=The NIL object
4  type=PROGRAM       name=<the program's path>  line=1  invocation=4  target=The NIL object
```

So: `~name` is the routine or method name **upcased**, and the program's *path* for the outermost
frame; `~line` is the clause each frame is currently executing, not the line it was entered at;
`~invocation` is the frame's depth counted from the innermost, starting at `1`; and `~target` is
the receiver, present only on a `METHOD` frame. The set of `~type` values still has to come from
the C++ (`StackFrameClass.cpp` and whatever fills it) rather than from those four — measure the
kinds this crate can produce and say which of the C++'s values it has no way to reach yet.

**`StackFrame`'s ten rows are a cascade today, and closing them by cascade would be the worst
hollowness in this phase.** Read from the tables 2026-09-07: `corpus/docs/class-set.txt:100` marks
`StackFrame` `not-covered` with **no construction expression** (`deferred-rexxcontext-stackframes`),
so `method-bodies.txt` falls back to a bare `~new` — and every one of the ten instance rows carries
the evidence `method "NEW" of class "StackFrame"`. The oracle refuses `.StackFrame~new` at the same
`93.967` site as `Pointer` and `Buffer`. **So making `new` raise would turn all ten rows green
without implementing a single StackFrame method**, and the next phase would read ten closed rows.

**Do both, in this order, and the second is not optional.** Raise on `new` — it is the oracle's
answer and it belongs here — and then **add a `RECEIVER_OVERRIDES` entry**
`("StackFrame", ".context~stackFrames[1]")` in `crates/rexx-exec/tests/method_bodies.rs:432`, which
is the mechanism this project already uses for exactly this (Phase 5g Task 7 and Phase 5h Task 6
each added a block of them, with the reasoning in the file). That turns ten rows measured about a
constructor into ten rows measured about their own methods, and it is the override that makes this
task's work visible. **Report the ten rows' verdicts before and after the override**; the ones that
are still loud after it are the honest count of what this task closed.

**A `StackFrame` outlives its frame and a `RexxContext` does not**, which is the opposite way round
from the guess. Measured on the oracle 2026-09-07, twice and independently — by Task 0's
implementer during its pre-flight and by the controller — with both objects captured inside a
routine and read after it returned:

```
c~name       -> Error 98.981  Target RexxContext is no longer active.
f~name       -> R
f~line       -> 19
f~traceLine  -> "    19 *-*   f = c~stackFrames[1]"
f~arguments~items -> 0
```

So a `StackFrame` is a **snapshot**, taken when `stackFrames` builds it, and a `RexxContext` is a
**live handle** that raises `98.981` once its activation is gone. Those are two different
representations and this task builds both. `98.981` is a row this task owes a witness for like any
other; find its message in `rexxmsg.xml` and assert the whole transcript.

**What the collector sees is still open** and is this task's remaining design question.
`Activation::object_roots` is named in `collect_now`'s comment as the other route for a parked
activation's objects; read it. Task 0's report carries both transcripts above under a heading for
this task.

---

## Both `Package` tasks share one problem: the receiver the rows are measured on

**Read from the tables 2026-09-07: `corpus/docs/class-set.txt:55` gives `Package`'s construction
expression as `.Class~package` — the REXX package, not a program's.** Every `Package` row in
`corpus/method-bodies.txt` is therefore measured against the interpreter's own built-in package,
and that object answers almost nothing. Measured on the oracle, rc 0:

```
name=REXX          sourceSize=0        source=Array(0)      sourceLine(1)=[]
classes=StringTable(67)                publicClasses=StringTable(62)
routines=StringTable(0)                publicRoutines=StringTable(0)
resources=StringTable(0)               namespaces=StringTable(0)
definedMethods=StringTable(0)          importedPackages=Array(0)
digits=9  form=SCIENTIFIC  fuzz=0      trace=[]   (the null string, not N)
prolog=The NIL object
options=::OPTIONS DIGITS 9 FORM SCIENTIFIC FUZZ 0 … PROLOG TRACE ?n/a?
```

**So an implementation that answers an empty `StringTable` to everything closes about fifteen of
the thirty-three rows and agrees with the oracle on every one of them.** That is this phase's
worst instance of `rust/CLAUDE.md`'s hollow-shell rule, and it is not hypothetical — it is what
the cheapest correct-looking implementation does.

**Two consequences, and both tasks are bound by them.**

* **`method-bodies.txt` is not the instrument for `Package`.** Task 0's
  `corpus/introspection-arity.tsv` is, because its receiver is a *program's* package with real
  directives behind it. Every witness in both tasks reads a program package; a row is reported
  closed only with its arity verdict beside its method-bodies verdict.
* **The two rows with real content on the REXX package are `classes` (67) and `publicClasses`
  (62)**, and they are Task 8's. `publicClasses` **is already implemented for program packages** —
  the loud site is `Loud::rexx_package_classes` (`dispatch.rs:5885`) and the row's evidence says so
  in words: `the REXX package's class table`. So that row is not "implement publicClasses", it is
  "give the REXX package a class table", and `classes` is the same job one method over.

Compare the same reads against a program package, measured on a 28-line file with a `::routine`, a
`::class` and a `::resource` (the figures in Task 7 and Task 8 below), and note which rows move.
Where the two disagree, both answers have to come out right.

---

## Task 7 — `Package`, the source and settings half

Twelve rows: `source`, `sourceLine`, `sourceSize`, `digits`, `form`, `fuzz`, `trace`, `options`,
`prolog`, `loadLibrary`, `setSecurityManager` on the instance arm, and `defaultOptions` on the class
arm.

**Plus one row Task 0 found and the scope note does not count: `Package~new` on the class arm**,
`send-differs` under `('p9.rex', .Array~of('return 1'))` -- so it does not even need a file on disk.
Ruled into this task 2026-09-07 for the same reason as `Routine~new` in Task 4: Task 8's
`addPackage` witness is written against it. **Thirteen rows.**

**Measured on the oracle 2026-09-07 on a program package** — the 28-line file described above:

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

**Four of these twelve answer differently on the two receivers, and each difference is a witness.**
Measured, program package against the REXX package: `trace` is `N` against the **null string**;
`prolog` is a `Routine` against **`The NIL object`**; `source` is `Array(28)` against `Array(0)`
and `sourceSize` `28` against `0`; and `options`' last field is `TRACE NORMAL` against
`TRACE ?n/a?`. A reader that answers the program package correctly and the REXX package by
accident will get at least one of those four wrong — assert all four pairs.

**`~options` takes an argument, and the plan's first measurement of it was incomplete.** Task 0
measured the missing half: `p~options` answers the whole `::OPTIONS ...` string,
`p~options('DIGITS')` answers `9`, and `p~options('DIGITS', 1)` also answers `9`. So it is a
**per-option reader with a no-argument summary form**, not a single string. `defaultOptions`
apparently ignoring its argument is still surprising; **re-measure that one yourself before
implementing**, with more than one option name and with a package carrying a real `::OPTIONS`
directive, and implement what you measure.
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

**`ADDCLASS`, `ADDPUBLICCLASS`, `LOCAL` and `NAME` already answer.** Read those four bodies first —
they are the worked example for where a package's state lives and how it is reached, and one of
them (`ADDCLASS`) is the write side of a table this task adds the read side of.

**Most of this state already exists on `Interp`, keyed by `ProgramId`.** Read against the tree
2026-09-07, `crates/rexx-exec/src/lib.rs`'s field list carries `package_options`, `routines`,
`package_public_routines`, `merged_public_routines`, `package_classes`, `package_public_classes`,
`merged_public_classes`, `package_namespaces`, `package_locals`, `class_packages`, `package_objects`
and `package_tables` (the last keyed by `(ProgramId, PackageTable::{UnattachedMethods, Routines,
Resources})`). A `Package` object reaches its `ProgramId` through `package_objects`' own key,
`Package::Program(ProgramId)` (`plan.rs:69`). **So for most of these rows the state is there and
what is missing is the reader** — confirm that per row and say which rows are the exception, because
those are the ones that carry the task's real cost.

**The measurement in the two blocks above is the one to distrust, and this task is where the
hollow-shell rule bites hardest.** On the REXX package nearly every count is 0; on the program
package nearly every count is 1. A body answering an empty table of the right class agrees with the
oracle on about half these rows, and a body answering a one-entry table agrees on most of the rest.
So: **the receiver Task 0 commits must carry more than one of each**, every witness here reads an
entry back out **by name**, and the report gives each row its arity verdict beside its
method-bodies verdict.

**The four writes, measured on the oracle 2026-09-07 with the read that sees each one:**

```
r = .Routine~new('NEWR', 'return 42')
p~addRoutine('NEWR', r)        -> a Package   ; p~routines      0 -> 1, index NEWR
p~addPublicRoutine('PUBR', r)  -> a Package   ; p~publicRoutines 0 -> 1, index PUBR
                                              ; p~findRoutine('NEWR') -> a Routine
op = .Package~new('other.rex')   -> a Package, ~name the resolved absolute path
p~addPackage(op)               -> a Package   ; p~importedPackages 0 -> 1
                                              ; p~importedClasses  -> OTHERC
                                              ; p~importedRoutines -> OTHERR
p~loadPackage('other.rex')     -> a Package
```

where `other.rex` holds `::routine OTHERR public` and `::class OTHERC public`. **All three `add*`
answer the receiving package itself, not the thing added**, and `addPackage` is what fills
`importedClasses` and `importedRoutines` — so those two rows cannot be witnessed at all without it,
which is the reason they are in this task and not Task 7. Witness each write by the read that
should see it, in the same program, and assert the before value as well as the after: a table that
was already non-empty makes "the write worked" indistinguishable from "the read answers something".

`.Package~new('<file>')` reads and installs a file, so a witness using it must live under the
corpus's own rules about files it creates — check what `rust/CLAUDE.md` says about never
instantiating `.Package~new` on a file inside the repository before writing one.

**The `find*` family and `resource`, measured on the oracle 2026-09-07** on a program with
`::routine RR public`, `::class K public` and a two-line `::resource R1`:

```
findClass('K')        -> the class K              findClass('ZZZ')  -> The NIL object
findClass('ARRAY')    -> the Array class          findRoutine('RR') -> a Routine
findPublicClass('K')  -> K                        findRoutine('ZZZ')-> The NIL object
findPublicClass('ARRAY') -> The Array class       findNamespace('X')-> The NIL object
findPublicRoutine('RR')  -> a Routine
findProgram('pf.rex')    -> the program's resolved absolute PATH, a String, not a Package
resource('R1')  -> Array(2), [1] = "line one"     resource('ZZ')    -> The NIL object
resources/classes/routines ~allIndexes -> R1 / K / RR
```

**The four rows once thought unmeasurable are measurable, and the receiver is how.** Task 0 reported
`importedPackages`, `importedClasses`, `importedRoutines` and `namespaces` as empty on both sides
and unreachable, on the ground that a `::requires` needs a file on this build's search path and
nothing is on it. That is falsified: a relative `::requires` resolves against the **calling
program's own directory**, and the arity harness already writes a fixture file into each temp probe
directory. Measured by the controller 2026-09-07, with `lib.rex` beside the probe holding
`::routine LIBR public` and `::class LIBC public`, and the probe carrying
`::requires 'lib.rex' namespace NS`:

```
namespaces        1   NS          importedClasses   1   LIBC
importedPackages  1               importedRoutines  1   LIBR
findNamespace('NS')       -> a Package
findNamespace('NS')~name  -> .../lib.rex
```

So all four have a non-empty witness, and `findNamespace` gains a hit case to go with its miss.
**A body answering an empty container is not allowed to pass any of them.**

**Two of those are the ones a guess gets wrong.** `findClass` and even `findPublicClass` reach past
the program's own table into the environment — `'ARRAY'` answers the `Array` class from both — so a
body that searches only the package's own `classes` table answers `.nil` where the oracle answers a
class. And `findProgram` answers a **path string**, not a package object. Read `PackageClass.cpp`
for each one's search order, measure the miss as well as the hit, and assert both.

---

## Task 9 — the sweep, and close

**No new bodies.** Its subject is what a per-task review structurally cannot see.

1. **Refresh both instruments and read them against the phase's claim.**
   `corpus/method-bodies.txt` under `REXX_METHOD_BODIES_REFRESH=1` and
   `corpus/introspection-arity.tsv` under Task 0's refresh variable. Report the `loud` count per
   class before and after the phase, and **name every row that did not close and why**. The scope
   note's figure to close against is 139.
2. **Re-run `fmt` and `clippy` from a CLEAN target directory**, in the gate worktree, at this
   boundary and nowhere else. `rust/CLAUDE.md` records a green
   `cargo clippy --workspace --all-targets -- -D warnings` that had not re-linted the code, repeated
   across a session including at two commits that fail the identical command when it is re-run, and
   its rule is that a same-session green is provisional. This phase's per-task runs are all warm --
   Task 0's second run reported G1 and G2 within seconds for a doc-comment-only diff, and its
   implementer flagged that itself. So the clean-target run belongs here, once, and its reading is
   the one the phase closes on.
3. **Run the gate's one rule in anger**: no row that was not diverging may have started diverging,
   and no row that was answering may have stopped. `RexxInfo~executable`/`~libraryPath` are the
   known exception and carry Task 3's decision.
4. **The hollowness sweep.** For every row this phase closed, ask the question
   `rust/CLAUDE.md`'s rule asks: what degenerate implementation satisfies its witness, and would
   deleting the state it reads leave it green? Report the rows where the answer is uncomfortable.
   The classes to look at hardest are the ones whose oracle answer is an empty container or a
   constant — `Package`'s tables, `RexxInfo`'s platform facts, the three `setSecurityManager` rows.
5. **Two things this phase found and declined, which the close must carry with their
   measurements.** `Class~new` on the class arm is `send-differs` under a real argument list and is
   **declined with a destination**: measured, `.Class~new('NEWCLS')` answers a class whose id is
   `NEWCLS` and whose superclass is `Object`, and whose instances then cannot be constructed at all
   -- `c~new` is `97.1 Object "The NEWCLS class" does not understand message "NEW"`. It is the raw
   metaclass primitive, class *creation* rather than class introspection, and belongs with the
   class-definition surface. And **turning the arity instrument's value comparison on for the
   collection driver as well is an open opportunity with a named cost**: it re-verdicts
   `corpus/collection-arity.tsv`'s `agree` rows, and whatever it finds is a Phase 5g or 5h defect
   rather than a 5i one. `Class~enhanced` is what that comparison found on this phase's own classes.
6. **The handover, which is a deliverable and not a formality.** It names, for the next phase to
   read: `Pointer` and `Buffer` as classes with no constructible instance and Phase 8 as the owner;
   `Message` and the two semaphores as Phase 6's; `Stream`, `File` and `RexxQueue` as Phase 7's,
   with `RexxQueue` additionally gated on D7; and any row this phase declined.
7. **Update the scope note in place** with what the phase actually cost against what it was scoped
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
