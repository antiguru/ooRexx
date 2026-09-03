# Phase 5d: the foundation Phase 5 still owes

**Status:** draft, 2026-09-03. Continues the Phase 5 numbering; new decisions here are **D75**
onward (5c ended at D74).

**Depends on:** 5c, implemented at `64258d712`, **not closed**. D75 is the reason and the fix.

**This document replaced an earlier draft of the same name on 2026-09-03**, which scoped 5d as
"every remaining constructor plus the stream read path — 95 of table C's 110 open rows". A plan
review and the measurements below killed that scope: **not one of the 95 rows can move without doing
work the tree already files under Phase 6 or Phase 7.** The old scope is not kept for comparison
because nothing in it survives; what replaced it is below.

---

## Why the scope changed

**The crate's own registry assigns the work**, with a reasoned comment
(`crates/rexx-exec/src/dispatch/native.rs:136`):

```rust
Family::Timer  => "Phase 6",                                 // .Alarm, .Ticker
Family::Stream | Family::Queue | Family::File => "Phase 7",  // stream_init, file_qualify
```

> **Timers are Phase 6's because nothing reaches them on one activity.** `Alarm~init` is `reply` and
> then `self~!startTimer(...)`, so the entry point runs on the activity the `REPLY` split off.

| the 110 open rows | what construction needs | filed under |
|---|---|---|
| `File` 50 | `MutableBuffer` bodies **and `file_qualify`** | Phase 5 **and 7** |
| `Stream` 24, `StreamSupplier` 8 | `stream_init`, `stream_uninit`, the read path | Phase 7 |
| `Alarm` 7, `Ticker` 6 | operator dispatch, `String~sign`, **and the timers** | Phase 5 **and 6** |
| `StackFrame` 10 | a `RexxContext` body and a `StackFrame` model | unowned |
| `Pointer` 5 | nothing; the refusal is correct | never |

**And the timers genuinely need Phase 6, measured rather than assumed.** This crate's `REPLY` is a
deferred queue drained at program end (`Interp::run_deferred_replies`, `lib.rs:7130`); the oracle
continues the body on an activity that can preempt the caller.

**This is already a deterministic, silent divergence, with no `Alarm` involved.** One program: a
method that replies and then prints, and a caller that busy-loops about 2M iterations after the send.

```
oracle          body-ran, main-done      8 of 8 runs
crate, both engines   main-done, body-ran      3 of 3 runs
```

rc 0 and empty stderr on both sides — only stdout order differs, which is the worst defect class this
project recognises. A shorter caller hides it: with a caller that finishes before it can be
preempted, the oracle gives the crate's order most of the time, which is why the first measurement of
this only interleaved 1 run in 3 and looked like a race.

**`REPLY`'s scheduling is Phase 6's** by the crate's own comment (`run.rs:3700`, "SCHEDULING, and
Phase 6 owns it"), so the fix is not 5d's — but the divergence exists today and is recorded here
rather than discovered again. An alarm exists to fire **while** the main program runs, and a queue
drained at program end structurally cannot. `alarm_startTimer` also has to block on a timed wait that
`cancel` can signal — activities, the kernel lock, and a waitable semaphore, which is Phase 6's
roadmap row verbatim.

**Chasing those rows would have repeated 5c's error in a new costume**: a row count picked the scope,
and the rows turned out to be owned by phases that have not started.

---

## What Phase 5 actually still owes

Each is Phase 5's by the crate's own refusal message, and each was measured 2026-09-03 at
`7816fc1bc` against the oracle, both engines, three descriptors read separately, from a fresh empty
directory.

### 1. Operator methods on instances

```
o = .DateTime~new + .TimeSpan~fromSeconds(5)
crate   rc 120   the operator `+` applied to an instance of a user class is not implemented (Phase 5)
oracle  rc 0     DateTime
```

`DateTime` defines `::METHOD "+"` in the bootstrap and this crate never dispatches to it. One
mechanism, and it is under `Comparable`, `Orderable`, every `TimeSpan` arithmetic path, and any user
class that defines an operator.

### 2. The method bodies, and the fact that nothing measures them

**Sampled 2026-09-03** by sending every documented instance-arm name to a real receiver, one program
each, and reading the refusal:

| class | instance rows | refuse *"is not implemented"* | reached a body |
|---|---|---|---|
| `String` | 118 | **80** | 38 |
| `Array` | 44 | **27** | 17 |
| `Directory` | 31 | **18** | 13 |
| `MutableBuffer` | 51 | **49** | 2 |
| `Method` | 17 | **14** | 3 |
| `TimeSpan` | 47 | 4 | 43 |
| | **308** | **192** | 116 |

**192 of 308 have no body, and every one of those rows is `agree` in gate table C** — the rows are
`hasMethod` readbacks, so a hollow class scores full marks.

**192 is exact, not a floor, and the reason matters for how the instrument gets built.** No arguments
were passed, and the refusal fires **before** any argument handling: `'abcdef'~substr` with no
arguments is rc 120 `method "SUBSTR" of class "String" is not implemented`, while an *implemented*
method sent at the wrong arity raises an ordinary ooRexx error instead —
`.Array~of(1,2)~at` is rc 163 `Compiled method "AT" with scope "Array"`, which contains no
"is not implemented". So a zero-argument send classifies `loud` cleanly in both directions and needs
no knowledge of any method's signature.

`TimeSpan` is the tell at 4 of 47: its bodies are Rexx in `CoreClasses.orx`. The natively-backed
classes are 60–96% hollow.

### 3. The package mechanism

The binding Phase 5 spec (`2026-08-17`, §"5c — the package, and the class library") files under 5c:
`::REQUIRES` with `LIBRARY` and `NAMESPACE`, namespace-qualified class references, `::OPTIONS`,
`::RESOURCE`, `::ROUTINE`'s options, `Package~local`, the two environment-search steps that cross a
package boundary, and the `>N>` trace prefix. **5c delivered `::OPTIONS`, `::RESOURCE` and
`::ROUTINE`'s options and left the rest.** Measured, each refusing with a message naming Phase 5:

```
::requires 'lib.rex'              crate 120   oracle rc 0, public routine and class both reachable
::requires 'lib.rex' namespace w  crate 120   oracle rc 0
say w:Widget~new~describe         crate 120   a namespace-qualified class lookup is not implemented
.context~package~local            crate 120   oracle rc 0, a Directory
trace i + a qualified lookup      not emitted oracle emits  >N>   W:WIDGET => "The WIDGET class"
```

`>N>`'s coverage entry still reads `Coverage::Owned("Phase 5")` in `trace_oracle.rs`, and it is a
by-product of the namespace work rather than a task of its own.

### 4. D73's guard, specified in 5c and never built

5c's spec made it a gate criterion: *"for every class `class-set.txt` marks `unreachable`, a
committed test that its documented construction route still refuses"*, and required the guard be
shown to fail. **It does not exist** — no test in `gate_table_c.rs` or `extract_docs.rs` reads the
`unreachable` status. 5d adds no constructors, so the risk is lower than it was in 5c; the guard is
owed because it was promised and because the classes it protects are about to be re-owned.

---

## D75. A method row's owner is a property of its class, and table D needs the same fix

**5c's gate cannot close and neither can any successor.** Measured in a `git archive` extract of
`64258d712`: adding `"5c"` to `CLOSED_PHASES` gives `gated by this run: 110 row(s)`, rc 101.
`gate_table_c.rs:591` is `const METHOD_PHASE: &str = "5c";` and files **all 1347 method rows** under
one phase.

### "Never expected to agree" is already in the data, and it is not per class

`class-methods.txt` carries `status` per **(class, method, arm)**, and the seven `unreachable` rows
split by arm, cleanly:

| rows | arm | verdict today |
|---|---|---|
| `Buffer~new`, `Pointer~new` | `class` | **`agree`** |
| `Pointer` `=` `==` `\=` `\==` `isNull` | `instance` | `unanswered` |

A class arm asks `hasMethod` of the class object, which answers whether or not an instance can exist.
So `unreachable` alone is the wrong predicate — applied per class it would mark two rows that agree
today as never able to agree. **The predicate is `status == unreachable && arm == instance`**, from
two columns that already exist: no new data, no second list of the same two class names, and a class
that becomes `unreachable` later is covered by existing.

### The new column carries the phase, and the values are not 5d's

`File`, `Stream`, `StreamSupplier` → **7**. `Alarm`, `Ticker` → **6**. `StackFrame` → a value naming
the `RexxContext` work it waits on. Everything else keeps `5c`.

**`StreamSupplier` is not `unreachable`, ruled here** because 5c's handover assigned 5d that ruling
and required the sentence be cited rather than paraphrased. `utilityclasses.xml:9754` says it
*"provides a snapshot of the stream at the point in time it is created"* — semantics, not
constructibility — and carries no "only using the native code application programming interfaces"
sentence, which is the criterion `class-set.txt`'s header sets. `Stream~supplier` is a documented
Rexx route (`streamclasses.xml:1685`). It stays `not-covered` and becomes Phase 7's.

### Table D files `::REQUIRES` under 5c and D75 cannot reach it

`gate_table_d.rs:251` is `("::OPTIONS" | "::RESOURCE" | "::REQUIRES" | "::ROUTINE", _) => Some("5c")`.
Running `REXX_PHASE_GATE=5c` on table D gives **rc 101, `gated by this run: 2 row(s)`** —
`requires__library__subkeyword.rex` and `requires__namespace__subkeyword.rex`. Table D has no
classes, so a per-class column cannot own them.

**They are re-owned in `owning_phase` directly**, where `("::ROUTINE", "EXTERNAL") => Some("7")` on
the line above is the precedent: `::REQUIRES LIBRARY` → **7**, because it needs a native library
loader that none of this build's eight libraries currently satisfies; `::REQUIRES NAMESPACE` → **5d**,
because this phase builds it.

**The control:** give one class an owner naming a closed phase while a row of it is open, and confirm
the gate reddens. `verdict_is_gated` with a never-agrees value is gated by neither disjunct and trips
no assertion — confirmed live, `deferred-parse-error-rendering` printed and was absent from the gated
list in an rc-101 run.

---

## D76. The method-body instrument, and what it gates

**Ruled by Moritz, 2026-09-03.** The sample above becomes a full sweep of all ~1347 documented rows,
committed as a generated table beside `class-methods.txt`, with one row per (class, method, arm)
classified as:

* **`answers`** — the send reached a body and agreed with the oracle;
* **`loud`** — this crate refused at rc 120 naming the method, which is the current safety property;
* **`diverge`** — the send answered and disagreed with the oracle.

**The gate is one rule: no row may move `loud` → `diverge`.** A hollow method that starts answering
*wrongly* is the failure this table exists to catch, and it is exactly what table C cannot see. A
`loud` → `answers` move is progress and needs no gate.

**Why this and not a count.** Counting implemented bodies would reward breadth; the project's real
invariant is that an unimplemented method is loud rather than wrong. The table makes that invariant
checkable for the first time, and it sizes whatever phase takes the bodies on.

**It must be shown to fail**, by making one `loud` row answer something the oracle does not and
confirming the gate reddens.

**The sweep is in two passes, and only the second needs the oracle.** Because the refusal precedes
argument handling, a zero-argument send classifies `loud` exactly — so pass one is crate-only over
every row, and pass two runs the oracle **only for the rows that are not `loud`**, sending both sides
the same arguments so no signature source is needed. On the sample's ratio that is roughly a third of
the rows reaching the oracle rather than all of them.

**A signature source is needed only to make `answers` a meaningful claim**, not to make the gate
work: a method that answers at zero arguments where the oracle also answers is compared honestly, and
one that needs arguments is compared on the same wrong ones. That is a weaker `answers` than the
table could carry, and it is a deliberate trade — the gated rule does not rest on it.

**A table nobody can afford to regenerate rots**, and a stale one asserting `loud` over a row that
now diverges is worse than none, so the refresh path is part of the deliverable.

---

## D77. Phase 5 does not implement Phases 6 and 7

**No task in this phase implements a `Family::Timer`, `Family::Stream`, `Family::File` or
`Family::Queue` entry point**, and none flips a `class-set.txt` row for `File`, `Stream`,
`StreamSupplier`, `Alarm` or `Ticker` to `covered`. Those classes' 95 rows leave 5d exactly as they
entered it, correctly owned and correctly open.

**The temptation is specific and worth naming.** `MutableBuffer`'s bodies are Phase 5's and they are
four-fifths of what `File`'s 50 rows need; `String~sign` is Phase 5's and it is most of what
`Ticker`'s 6 need. Landing them will move those rows *closer* without moving them, and the report
must not present that as progress against a row count.

---

## D78. `MutableBuffer` and `String` bodies enter as themselves, not as unblockers

The bodies D76 measures are Phase 5's own work, and 5d lands the ones its other tasks need —
`String~sign` is reached by `TimeSpan~sign`, and `MutableBuffer`'s are reached by `File~init`, which
5d does not build. **A body is added here because the documented method should work**, not because a
row it unblocks is nearly green.

Every body added in this phase gets a differential corpus program. The rc-120 safety property is what
D76's gate protects, and this phase is the first to add bodies while that gate exists.

---

## The gate

1. **5c closes**, on both tables: `CLOSED_PHASES` names `"5c"`, and
   `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1` over table C **and** table D exits 0. The table D half is
   the one an earlier draft missed.
2. **5d's own rows agree** under `REXX_PHASE_GATE=5d`, which after D75 means table D's
   `::REQUIRES NAMESPACE` row and nothing in table C.
3. **The per-class owner and the never-agrees value have each been shown to gate**, by the controls
   in D75.
4. **The method-body table exists, is complete over the documented set, and its gate has been shown
   to fail** by making one `loud` row diverge (D76).
5. **The `unreachable` guard exists, passes, and has been shown to fail** (D73, owed since 5c and
   never built).
6. **Operator methods on instances work**, witnessed by a corpus program covering a user-defined
   `"+"`, the bootstrap's `DateTime`/`TimeSpan` arithmetic, and the refusal that remains when no
   operator method is defined.
7. **`::REQUIRES` of a program file works**, witnessed by a corpus program: a public routine and a
   public class from a required file, the namespace form, a qualified reference, `>N>` under
   `trace i`, and the four-route search order.
8. **The five gates each 0**, and the corpus headline at its full count — **331 of 331** today, plus
   whatever this phase commits and the three 5c witnesses Task 1 folds in.
9. **Nothing that previously refused now answers wrongly** — D76's gate, over the whole documented
   set rather than over the rows a task touched.

**Not a criterion:** the number of method bodies implemented, or any movement in `File`, `Stream`,
`StreamSupplier`, `Alarm` or `Ticker`'s 95 rows. D77 puts them out of scope; a phase that moved them
would have done Phase 6 and 7's work.

---

## Risks

| risk | why it bites | mitigation |
|---|---|---|
| the bodies get chased by row count again | 192 hollow rows are all green, and the 95 open ones are nearly-green for the wrong reasons | D77 names the temptation; D76 replaces the count with an invariant |
| the sweep is too expensive to re-run | a stale table asserting `loud` over a diverging row is worse than none | D76's two passes: `loud` needs no oracle run, so only the non-`loud` third reaches it |
| `::REQUIRES` grows into a package rewrite | it is the package mechanism: search order, prologue-once, circularity, namespaces, `Package~local` | split across two tasks with the file-loading half first and witnessed on its own |
| the search path is narrowed | the oracle finds a required file by cwd, program directory, `REXX_PATH` and `PATH`, plus extension appending; a narrower search turns an honest refusal into a loud wrong answer | criterion 7 names the four routes |
| operator dispatch reaches further than expected | it is under `Comparable`, `Orderable` and every bootstrap arithmetic path | its own task, its own corpus witness, and the benchmark axes read after it |
| closing 5c ratifies its unbuilt half | `CLOSED_PHASES` means "its rows are gated", not "its spec is done" | this document is where the unbuilt half is recorded, and it is 5d's scope |

---

## What 5d hands on

* **To Phase 6, and one of them is a live defect rather than absent work:** `REPLY` continues the
  method body at **program end** here and on a preemptable activity in the oracle, which is a
  deterministic stdout-ordering divergence at rc 0 with empty stderr on both sides — measured 8 of 8
  against 3 of 3, no `Alarm` involved, both engines. It is Phase 6's by `run.rs:3700`'s own
  attribution, and it is the reason `Alarm` and `Ticker` cannot be stubbed. Plus their 13 rows and
  the timer entry points. `Ticker~init`'s loop runs until cancelled, so a construction cell is safe only
  while the target's `triggered` is ABSTRACT (`CoreClasses.orx:735`) — measured, a working
  `triggered` gives rc 137 at a 6-second bound.
* **To Phase 7:** `File` 50, `Stream` 24, `StreamSupplier` 8; `stream_*`, `file_*` and `qualify`; the
  stream read and write paths and `~open`'s option grammar (`StreamCommandParser.cpp`, 253 lines);
  the stream and platform BIFs, unchanged in `phase-4-exclusions.txt`; and `::REQUIRES LIBRARY`,
  which needs a native library loader.
* **Unowned, and each a real divergence** — inherited from 5c's handover and untouched here:
  `RootSet::promote` leaks one cell per referenced variable instance; `~unknown`'s argument list does
  not send `MAKEARRAY`; a `Directory` subclass's entry writes refuse where the oracle answers;
  `identityHash` is licensed rather than matched.
* **`StackFrame`'s 10 rows**, needing a `RexxContext` method body and a `StackFrame` object model.
* **The corpus-coverage gap**, which Task 1 closes rather than hands on: a `corpus/lang/*.rex` named
  in no phase subset file is silently unrun. **342 on disk, 319 filed, so 23 unrun** — the earlier
  figure of 11 came from subtracting the differential's whole count of 331, only 319 of which are
  `lang/`. **Twelve of the 23 are `directive_options*`**, which is the Deviation 7 stderr family, so
  filing them without carrying the stderr-comparison scope across brings that flake straight back.
  The other eleven — `call_procedure`, `condition_syntax`, `do_variants`, `gate_variants`,
  `keyword_as_variable`, `source_arg`, `string_builtins`, `whitespace_significant`, and 5c's three
  witnesses — have been silently unrun for longer than 5c.
