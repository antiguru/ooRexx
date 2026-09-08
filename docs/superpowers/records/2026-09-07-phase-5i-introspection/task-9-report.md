# Task 9 — the sweep, and the close of Phase 5i

BASE `532cbe2cf`, which is the phase tip and the commit every measurement below was taken at.
Baseline for the perf work is `bc9d991cc`, the parent of the scope note. This task added **no method
bodies and touched no `.rs` file**; its commit is this report, the scope note's close section, and
the rows `Object~copy`, the zero-length `Rc`, the identity operators and `.RexxInfo~class~new`
added to the found-and-not-fixed register.

---

## 1. The phase's central result: every defect it found returned success

**Not one defect Phase 5i found changed an exit status.** `Class~enhanced` answering `a K` at rc 0
with the method working; the arity probe measuring its own crate's `CONDITION option "O"` gap;
`Package~findProgram` rewriting the table on every refresh; the `LIBRARY` name comparison being
caseless; the setters' flag writes lost across `~define`'s copy; `RexxContext~package` answering a
`Package` where the oracle raises `98.981`. Every one is rc 0 on both sides, or rc 0 against a row
whose recorded evidence does not move.

A gate that reads exit codes and a reviewer that reads code are both blind to that whole class.
Only a differential that compares **the answer** sees any of it.

### The four ways an instrument here went green over a wrong answer

These are properties of the instruments, not a list of rows that have since been fixed. **Every
green cell this phase produced is qualified by all four**, including the ones quoted as evidence in
this phase's own rulings and reports.

1. **A gate reading exit statuses cannot see a wrong answer at rc 0.** `RexxContext~package`
   answered a `Package` where the oracle raises `98.981`; its `corpus/method-bodies.txt` row is
   **byte-identical before and after the correctness fix**, because the row records the status both
   sides gave and both gave the same one.
   *Owner: the harness. Not fixable by sharpening a verdict — the row's evidence column is a status
   by construction.*

2. **A row comparing `vv~string` records a class description, not contents.** `a StringTable` is
   what a real one and a wrong one both render as, so **every collection-returning row in
   `corpus/introspection-arity.tsv` compares equal the moment the send succeeds**.
   *Owner: the harness. Sharpenable in principle — see §9 for the named cost of turning the value
   comparison on for the collection driver.*

3. **A row probes one instance per class, fixed as an expression.**
   `corpus/introspection-receivers.tsv` gives a class at most a `class` arm and an `instance` arm —
   the class object and *one* instance — never two alternative instances, and some classes there carry only
   one of the two: `RexxContext`, `RexxInfo` and `StackFrame` have no class arm, `Pointer` and
   `Buffer` no instance arm (§8's table marks each). `Package`'s is
   `r = .context~package`, a program's package, so `.Class~package`, the REXX package with empty
   tables and `sourceSize 0`, **is unreachable by any argument list**: the arguments file varies only
   what is *sent*. `Package~publicClasses` read `agree` while the crate refused outright on the REXX
   package, because the receiver could not exercise the defect. A blind instrument can be sharpened;
   a receiver that cannot exercise a defect cannot.
   *Owner: a design property of the arity table. §8 bounds it per class.*

4. **The object a row answers can be half-real — the right class with the wrong body.** A package
   table is a `Body::Native`; `.StringTable~new` is a store-backed `Body::Instance`; the collection
   methods are bound to the store-backed shape. `~items`, `~hasIndex`, `~allIndexes`, `~supplier`
   and `~makeArray` are **rc 120 on a package table** and answer on a `.StringTable~new`. The
   refusal is **process-terminating**: `signal on syntax` does not see it, so it is not a refusal a
   program can route around.
   *Inherited from Phase 5h, undisclosed until Tasks 7+8's review. Owner: Phase 5h's. Register
   row 16.*

**Task 9 did not attempt to fix any of the four**, per the ruling: three are Phase 5h's or the
harness's and the fourth is a design property. Its job was to state them, bound them, and give each
an owner, which is what the four paragraphs above do.

**Two more instances of the same shape were found by this task's own sweep**, both of them
blindness 3 biting rows outside this phase: `Object~copy`, and the six identity operators. See §5.

---

## 2. Both instruments refreshed at the tip

### `corpus/method-bodies.txt`

```
REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies
  test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 96.70s
  regressions this run: 0. other drift from the committed table: 0.
```

The file's mtime moved (14:44:12) and `git status --short -- corpus/` was empty afterwards, so the
refresh **rewrote the table and rewrote it identically**. That is a stronger reading than "the
committed table was left alone": it says the committed table is what a fresh run produces.

The `bc9d991cc` column is that commit's **committed** table read with `git show`, not a fresh run
taken there; the `532cbe2cf` column is the refresh above.

| verdict | at `bc9d991cc` | at `532cbe2cf` |
| --- | --- | --- |
| `loud` | 204 | 67 |
| `answers` | 1058 | 1190 |
| `unanswered` | 71 | 76 |
| `diverge` | 2 | 2 |
| `unstable` | 4 | 4 |
| `uncomparable` | 8 | 8 |

`loud` rows per class, before and after — every class the table names whose count moved, plus every
class that still refuses:

| class | before | after | |
| --- | --- | --- | --- |
| `Package` | 33 | 0 | |
| `RexxInfo` | 28 | **2** | `executable`, `libraryPath` — declined |
| `Message` | 17 | 17 | out of scope, Phase 6 |
| `Method` | 16 | 0 | |
| `RexxContext` | 14 | 0 | |
| `RexxQueue` | 14 | 14 | out of scope, Phase 7 + D7 |
| `Class` | 11 | 0 | |
| `StackFrame` | 10 | 0 | |
| `Routine` | 8 | 0 | |
| `Pointer` | 6 | 0 | five became `unanswered`, see below |
| `Stream` | 25 | 25 | out of scope, Phase 7 |
| `EventSemaphore` | 4 | 4 | out of scope, Phase 6 |
| `File` | 3 | 3 | out of scope, Phase 7 |
| `Object` | 3 | 0 | |
| `MutexSemaphore` | 2 | 2 | out of scope, Phase 6 |
| `Buffer`, `Directory`, `IdentityTable`, `MapCollection`, `Properties`, `Relation`, `Stem`, `StringTable`, `Table`, `WeakReference` | 1 each | 0 each | the mapped-collection `of` rows, plus `Buffer~new` and `WeakReference~value` |

### `corpus/introspection-arity.tsv`

```
REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity
  test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.23s
```

Same reading: mtime moved (14:47:08), `git status` empty afterwards.

| verdict | as first committed (`031b7a6f0`, Task 0) | at `532cbe2cf` |
| --- | --- | --- |
| `agree` | 58 | 164 |
| `send-differs` | 116 | **9** |
| `setup-differs` | 10 | 0 |
| `exempt` | 6 | 6 |
| `no-value` | — | 11 |
| `unstable` | — | 2 |

**The two columns are not a clean delta and should not be read as one**: `no-value` and `unstable`
did not exist when the table was created, so rows now carrying them were carrying something else
then. What does compare cleanly is `setup-differs`, which went to zero: **every receiver in the
table now builds on both sides**, which was the phase's headline measurement when it started.

---

## 3. The gate's one rule, run in anger

**No row that was not diverging started diverging, and no row that was answering stopped.**

Measured two ways, not one:

* The refresh run's own `no_row_started_diverging_or_stopped_answering` passed, and its writer
  refuses to write a regression at all.
* Independently, a row-by-row join of the table at `bc9d991cc` against the table at `532cbe2cf`:
  **the only transitions in the entire file are 132 `loud` -> `answers` and 5 `loud` ->
  `unanswered`.** Nothing left `answers`, nothing entered `diverge`, and the two `diverge` rows are
  the same two — `DateTime~date` and `~timeOfDay`, register row 1's builtin-argument-run defect.

The five are `Pointer`'s instance rows: `=`, `==`, `\=`, `\==`, `isNull`. That is not a regression
but the verdict becoming honest — `.Pointer~new` now raises `93.967` on both sides, so the probe's
receiver is never built and the row records that it learned nothing rather than recording a refusal.
The scope note predicted exactly this cascade when it took Pointer in.

`RexxInfo~executable` and `~libraryPath` are the known exception and carry Task 3's decision: they
answer a `.File` and `File` is Phase 7's, so they are **declined**, not missed.

---

## 4. The hollowness sweep

The question `rust/CLAUDE.md` asks of every assertion: *what degenerate implementation satisfies
this, and would deleting the state it reads leave it green?*

### `Package`'s tables — answered by running, not by reading

A mutation **pair**. **What is attested and what is not**: the prediction text is at
`scratchpad/t9-close/mutation-prediction.txt`, and its file mtime (15:01:34) is **after** the mutated
binary's (14:50:52), because the second prediction below was appended to the same file later. So the
file cannot attest that the first prediction preceded the build — it attests only that both
predictions exist and that neither was edited after the last append. Read the ordering claim as
unattested rather than as evidence.

Mutation: `fn classes` (`crates/rexx-exec/src/dispatch/package.rs:538`) answers an empty entry set
for every receiver, with the `package_of` receiver check left in place. Built in its own target
directory; `sha256` differs from the unmutated binary and its mtime moved.

| | `package_tables.rex` | `package_rexx.rex` |
| --- | --- | --- |
| predicted, unmutated | GREEN | GREEN |
| **measured, unmutated** | **GREEN**, rc 0 | **GREEN**, rc 0 |
| predicted, mutated | RED | RED |
| **measured, mutated** | **RED**, differing from the first stdout line (`classes [KA] is a Class`) | **RED**, differing from the first stdout line, and the crate's run ends early at rc 159 where the oracle runs to the end at rc 0 |

All four cells as predicted. **The witnesses read the state the row answers**; an empty-container
implementation does not survive them.

**"Can fail" is not "adds coverage", so the same mutation was run against the whole corpus gate**
— `REXX_CORPUS_GATE=1 memcap 8G cargo test --profile mutation -p rexx-exec --test corpus
--no-fail-fast`, in its own target directory. Prediction written first (same scratch file):
`package_tables.rex` and `package_rexx.rex` to fail; `package_writes.rex`, `class_package.rex` and
`package_find.rex` possibly; everything else to pass.

```
STRICT (REXX_CORPUS_GATE) mode: 3 of 463 corpus programs disagree with the oracle
  lang/package_tables.rex: stdout differ
  lang/package_find.rex:   stdout, stderr, exit code differ
  lang/package_rexx.rex:   stdout, stderr, exit code differ
exit 101
```

Both predicted failures **confirmed**; of the three "possibly", `package_find.rex` **failed** and
`package_writes.rex` and `class_package.rex` **passed** — they read a package table but not the one
this mutation empties. So `Package~classes` is load-bearing in three corpus programs, and the
mutation pair above was not measuring the gate but the two programs directly. Both readings are in
the report because they answer different questions.

### `RexxInfo`'s platform facts — a constant is genuinely right, and the readers say which constant

`RexxInfo::initialize` fills every field of the C++ instance from a compile-time constant or a
platform call, at image-build time, and there is one instance; nothing a program does moves any of
them. So the hollowness question has no state to delete, and the constraint the phase's own global
rule states is **which** constant. Read at the tip, the bodies name shared ones rather than literals
copied out of a differential: `architecture` is `size_of::<*const ()>() * 8`, `digits` is
`rexx_num::Settings::default().digits()`, `internalDigits` is `rexx_num::ARGUMENT_DIGITS`,
`maxArraySize` is `MAX_FIXED_ARRAY_SIZE`, `maxPathLength` is `PATH_MAX`, `platform` is
`parse_template::PLATFORM` — the same source as `PARSE VERSION`.

**What settles it observationally is variation, and it was run.** A degenerate reader that answered
the running setting, or one that answered a frozen 9 regardless, are told apart by changing the
setting and reading both:

```
default digits  .RexxInfo~digits form fuzz = 9 SCIENTIFIC 0
numeric digits 15; numeric form engineering; numeric fuzz 3
changed digits  .RexxInfo~digits form fuzz = 9 SCIENTIFIC 0     <- default, correctly unmoved
live settings   digits() form() fuzz()      = 15 ENGINEERING 3
ctx defaults    .context~digits form fuzz   = 9 SCIENTIFIC 0
ctx changed     .context~digits form fuzz   = 15 ENGINEERING 3  <- live, correctly moved
```

Oracle and both engines byte-identical on three descriptors. So the crate gets **both** halves of a
distinction that a single reading cannot see: `RexxInfo`'s three are the defaults and
`RexxContext`'s three are the settings in force.

**What was not done**: no mutation was built to prove the *coupling* between, say,
`.RexxInfo~maxArraySize` and the limit the array code enforces. The claim that they share a constant
rests on reading the two sites, not on moving one and watching the other follow.

### `setSecurityManager` on `Method`, `Routine` and `Package`

The hollowness question does not arise: they are **not closed**. Each is `send-differs`, refusing
loudly, and are declined because implementing them without interception would ship rc 0
where the oracle raises. That is the right shape — a method that exists and does nothing is not
implemented.

---

## 5. The adversarial hunt

Run deliberately, per the ruling, because *the witness a task writes is drawn from its own
implementation, so it cannot be the instrument that finds what the implementation did not think of*.
Each candidate class the ruling named was turned into oracle-versus-both-engines probes over three
descriptors, from a fresh empty directory per probe.

| candidate class | what was run | result |
| --- | --- | --- |
| an object-valued row's **class** | every object-valued introspection answer's `~class~id`: the `RexxContext`, `Package`, `Method`, `Routine`, `Class`, `StackFrame` and `RexxInfo` readers, plus the same rows reached through **built-in class receivers** (`.Array~package`, `~methods`, `~method('ITEMS')~source`/`~scope`/`~package`) | **all agree** |
| **case sensitivity** in a parsed name | `findClass`, `findRoutine`, `hasMethod`, `method`, `instanceMethod`, `findProgram`, and index reads on `classes`, `definedMethods`, `resources`, each in upper, lower and mixed case | **all agree**, including the asymmetry that `findClass` upcases and `classes['k']` does not |
| a **copy** carrying half of the original | `~copy` on every Phase 5i class and on ordinary values | **a divergence — see below** |
| an **empty container** answering where the oracle answers something | `importedPackages`, `importedClasses`, `importedRoutines` on a package that really has a `::requires`, which the arity table's receiver cannot be | **agree** — the tables carry the imported names and their objects |
| a reader that is a **constant** right on this machine | §4's variation probe | **agree**, both halves |
| *(added by a probe, not by the ruling's list)* the **identity operators** on an introspection object | `=`, `==`, `<>`, `><`, `\=`, `\==` over the same receiver shapes | **a divergence — see below** |

### The finding: `Object~copy` refuses on a plain String, untrappably

`native_copy` (`crates/rexx-exec/src/dispatch.rs:7558`) answers for `Primitive::Instance`,
`Primitive::Array` and `Primitive::Stem`, and is `Loud::native_method(b"COPY", "Object")` for
everything else.

The sweep that bounds it: the `Object` sends `class`, `copy`, `defaultName`, `objectName`, `string`,
`isNil`, `instanceMethods`, `hasMethod`, `isA`, `request` and `send`, over the receiver shapes in the
table below, one program per combination so no refusal hides another, oracle and both engines on
three descriptors. **178 of 187 combinations agree, and every one of the 9 that differ is `copy`.**
That is the finding and also its bound.

| receiver | oracle | this crate |
| --- | --- | --- |
| `'abc'`, `42` | rc 0, copies a `String` | **rc 120** |
| `.K~method('MM')`, `.routines['R']`, `.context~package`, `.context~stackFrames[1]` | rc 0 | **rc 120** |
| `.context~package~classes` | rc 0 | **rc 120** |
| `.methods` — which at the top level is the String `.METHODS`, not a table, and which is **not one of the 187 combinations**; it was run as its own program, re-measured 2026-09-08 from a fresh directory: oracle rc 0 `copied String`, both engines rc 120 | rc 0 | **rc 120** |
| `.RexxInfo` | rc 163 | **rc 120**, different message |
| `.nil` | rc 159 | **rc 120** |
| `.Object~new`, `.array~of(1,2)`, `.StringTable~new`, `.Directory~new`, a `WeakReference`, a `Supplier`, both class arms | — | agree |

It is **untrappable**: a `signal on syntax` around the send does not see it and the process ends at
rc 120 — the same cost as register row 16.

**`corpus/method-bodies.txt` records `Object copy instance answers rc 0`.** The row's receiver is
`.Object~new`, the one instance shape that works. That is blindness 3 from §1, on a row this phase
never touched, and it is why the sweep found it and no review did.

**What is new and what is not.** `corpus/refusal-sites.tsv`'s `native_method` row already records
this site as `diverges` and already carries `'abc'~copy` as one of its witness expressions. What is
new is the reach — every Phase 5i class's instance, plus a String and an integer — the
untrappability, and the fact that a green `answers` row sits over it. **Recorded as register row 17;
not fixed here** (Task 9 adds no bodies, and the owner is the phase that owns `Object~copy`).

### The second finding: the identity operators refuse on every introspection instance, and on an `Array`

Found by asking whether `.RexxInfo` is the only `RexxInfo` a program can get. It is — the oracle
refuses `.RexxInfo~class~new` with `93 SYNTAX` — but the probe's own first line,
`(.RexxInfo == .RexxInfo)`, was rc 120 here and `1` on the oracle.

Swept the same way as `copy`: the six operators `=`, `==`, `<>`, `><`, `\=`, `\==` over the same
receiver shapes, one program per combination. **42 of 90 combinations agree; the 48 that differ are
exactly the six operators applied to `Array`, `Method`, `Routine`, `Package`, `RexxInfo`, a package
table, `StackFrame` and `RexxContext`.** The oracle answers `1` or `0` at rc 0 for every one; this
crate is rc 120. `.Object~new`, a String, a `.StringTable~new`, a user class, a built-in class, a
`Supplier` and a `WeakReference` agree.

The site is `eval.rs`'s operand check rather than the operator send (`eval.rs:1831`,
`dispatch.rs:2108`), so this is register row 3's family: **closing it means teaching the evaluator
the operator send**. Row 3's sentence — that those sites are "the last places a class object
refuses" — is true and reads narrower than the fact, because a `Body::Native` **instance** and an
`Array` refuse there too.

**It is older than the phase.** The same two programs give byte-identical refusals under a binary
built at `bc9d991cc`, so nothing in Phase 5i introduced it.

**It was disclosed, twice, and still needed the sweep.** Task 3 recorded that it could not assert
`.RexxInfo~package == .Array~package` for this reason; Task 5 recorded that its change made class
handles operator receivers and nothing else. What neither could see from inside its own row is the
width, or that **both tables are green over it**: `corpus/method-bodies.txt` records
`Object = instance answers rc 163` and `corpus/introspection-arity.tsv` records all six operator
rows `agree rc0` — both probing `.K~new`, the one receiver shape on which they work. Blindness 3
and blindness 1 together, on a row this phase never touched. **Register row 19.**

**A probe error worth recording, because it nearly became a finding.** The first version of this
sweep generated `say 'V['(o = o)']'`, in which a quoted string immediately followed by `(` is a
*function call* on the literal name — so both sides failed with error 43 and the run reported 62
divergences that were the probe's. The tell was that the oracle failed too. The numbers above are
from the corrected form, which assigns the comparison to a variable first.

---

## 6. Cumulative performance: the phase total

Per-step thresholds cannot see a sum. Phase 5a passed every one of them under 1–2% and totalled
**+32%**, which is why this measurement exists.

`bc9d991cc` against `532cbe2cf`, `instructions:u`, **interleaved** round by round, 5 rounds,
medians. Each revision built in **its own target directory** from its own detached worktree:

```
target-base/release/rexx-run  sha256 33e15b13c3b4911b96b402c5f7b190a24c462ba74cba8c16d48d9bf7bd38fbb5  31,741,904 bytes
target-tip/release/rexx-run   sha256 17c8373300d0b762194353c27183dd722bcc03930bb6c5935c0ce0129c745a5f  34,521,288 bytes
```

The two hashes differ, both mtimes moved during the builds, and both build logs show `Compiling`
lines rather than a bare `Finished`.

| axis | `bc9d991cc` | `532cbe2cf` | phase total | Task 6's own step |
| --- | --- | --- | --- | --- |
| `emptyloop` | 9,496,957,362 | 9,547,166,038 | **+0.529%** | +0.527% |
| `varlookup` | 16,784,980,658 | 16,861,189,548 | **+0.454%** | +0.453% |
| `dispatch` | 30,019,354,237 | 30,369,565,262 | **+1.167%** | +1.166% |
| `dispatchclass` | 24,903,770,310 | 25,175,969,974 | **+1.093%** | +1.092% |

**Null control**, the baseline binary interleaved against itself, same harness, 5 rounds:
`emptyloop` +0.0000%, `dispatch` −0.0001%. Predicted under ±0.05% and confirmed, so the four figures
above are three to four orders of magnitude above the measurement floor.

**The finding: the phase total IS Task 6's single step, on all four axes.** Every other task in
Phase 5i — the `ARG` fix, weak references, `RexxInfo`, `Method`, `Class`, `Package`, `Routine`,
`StackFrame` — cost nothing these counters can see. This is the outcome the ruling named as "about
what this one step suggests", not the several-times-it outcome that would have needed an owner
before the phase closed.

**Why it was a real risk and not a formality**: none of these numbers could have been obtained by
adding up the per-task figures, because each was taken against its own BASE on a machine in an
unknown state. Both ends were re-measured here.

Task 6's residue, from its own isolation builds, is where a later redesign starts: removing the
clause snapshot from `push_activation` is worth 0.08pp, removing the per-activation convention
snapshot entirely is worth 0.38pp, so the rest is the `Rc` conversion of `CallContext.arguments` and
the wider `Activation`. **That cost is paid by programs that never touch `.context`**, which is the
argument for paying it down rather than accepting it.

---

## 7. The zero-length `Rc::from` sweep

`Rc<[T]>::from(&[])` allocates a header even for a zero-length slice, so an "empty default" is a
`malloc`/`free` pair. Task 6 found two instances and each cost real time on the send axes; two
instances of one shape is a class, so it was swept at `HEAD` with the tree stable:
`(Rc|Arc)::from\(` over `crates/*/src`, plus the literal `Rc::from(&[][..])`.

**A candidate is not a defect**: the trap costs only where the empty case is common *and* the site is
hot.

| site | zero-length case | how hot | verdict |
| --- | --- | --- | --- |
| `run.rs:1446`, `Rc::from(&self.call_context.name[..])` | **the common case**, from the second execution of a function-form call site onward | **the call path** | **a live hit — see below** |
| `lib.rs:4620`, `Interp::shared_arguments` | guarded: `values.is_empty()` returns the shared empty first | the call path | Task 6's fix, working |
| `lib.rs:4715`, `empty_arguments: Rc::from(&[][..])` | always | once per interpreter | note |
| `lib.rs:7258`, `::CONSTANT`'s `Rc::from(&[][..])` | always | once per `::CONSTANT` directive | note — the literal form, on a directive-install path |
| `lib.rs:4893`, `Rc::from(arguments)` | never (one element) | once per program load | not the shape |
| `lib.rs:8160`, `Rc::from(&[Some(value)][..])` | never (one element) | once per program | not the shape |
| `run.rs:4614`, a `Trap`'s label | never (a label has a name) | per `SIGNAL ON` executed | not the shape |
| `activation.rs:277`, `AddressState::set_bytes` | an `ADDRESS` name is not empty in practice | per `ADDRESS` change | not the shape |
| `selector.rs:98`, `SelectorTable::intern` | never | once per new spelling | not the shape |
| `activation.rs:1998` | — | `#[cfg(test)]` | not code |

### The live hit, and why the fix is not at the site

`run.rs:1446` builds the call-name snapshot for every **non-method** activation. Register row 13 is
that, from the second execution of one function-form call site onward, the IR engine — the default —
passes an **empty** name into `CallContext.name`. So at that site the common case on a repeated call
is a zero-length `Rc`.

Re-measured at the tip, oracle rc 0 printing `RTN` six times:

```
do i = 1 to 3; say i '[' || rtn() || ']'; end     REXX_ENGINE=ir         -> RTN, <empty>, <empty>
                                                  REXX_ENGINE=tree-walker -> RTN, RTN, RTN
```

The empty name the program prints **is** the Rc that was allocated.

**What retires with row 13 is the zero-length case, not the allocation.** An earlier draft of this
section and of register row 18 said "fixing row 13 removes the allocation". That was an unrun
mechanism claim and it is wrong: row 13's fix makes the name **non-empty**, so `run.rs:1446` still
builds an `Rc<[u8]>` per non-method activation — the same `malloc`/`free` pair, now with a payload to
copy. What the fix removes is the *trap's shape* (an allocation for nothing), not its cost. An
`is_empty()` guard at the site would skip the allocation only while the defect is present, would
leave the wrong answer, and would never fire once row 13 is fixed — so the guard is not the fix
either way.

**Removing the cost belongs with Task 6's residue** (§6), which already names "the `Rc` conversion of
`CallContext.arguments` and the wider `Activation`" as where a redesign starts; the method path
already avoids it by reading the name from `method_identity`. **The cost at this site was not
measured** — no A/B was built for it — so this is a named candidate, not a quantified one. Recorded
as **register row 18**.

---

## 8. Blindness 3, bounded: which classes have a second instance different in kind

Per the ruling, this is answered by **reading the receiver expressions and asking what instance the
expression cannot produce** — not by re-running the tables, which cannot see it. For each, whether a
**corpus witness** reached that second instance, since a witness can reach what the arity table
cannot.

| class | the receiver the table probes | a second instance different in kind | reached by a witness? |
| --- | --- | --- | --- |
| `Package` | `r = .context~package`, a program's package | **the REXX package** (`.Class~package`: empty `routines`, `publicRoutines`, `resources`, `definedMethods`, `namespaces`, `sourceSize 0`), and a package with a real `::requires` | **yes** — `package_rexx.rex` and `class_package.rex` reach `.Class~package`; the `::requires` case was reached by this task's probe and agrees. This is the one the phase found the hard way. |
| `Method` | `r = .K~method('MM')`, a Rexx-coded method | **a native method** (`.Object~method('objectName')`), whose `~source` is an empty Array — the receivers file says in its own header that the native one cannot discriminate a body that reads the source from one that answers an empty Array | **yes** — `method_introspection.rex` sends the flag and line rows to `.Object~method('OBJECTNAME')` |
| `Class` | `r = .K`, a user class with an inherited mixin | **a built-in class** (`.Array`, `.String`), whose identity lives outside the arena in `rexx-classes` | **yes** — `class_introspection.rex` sends the operator rows to `.Array`, and this task's probe sent `~package`, `~methods`, `~method('ITEMS')~source`/`~scope`/`~package` to `.Array`; all agree |
| `Routine` | `r = .routines['R']`, a directive's routine | **a routine built at run time** (`.Routine~new`, `~newFile`), and an external routine | **yes** for `.Routine~new` — `routine_introspection.rex`, `package_rexx.rex`, `package_writes.rex`. **No** for an external routine: `loadExternalRoutine` is declined to Phase 7. |
| `RexxContext` | `r = .context` **inside a call**; **no class arm** | **the top-level context** (no caller), and a context inside a method | **yes** for the top level — `rexx_context.rex` reads `.context~name` and `~line` at the top level |
| `StackFrame` | `r = .context~stackFrames[1]` inside a call; **no class arm** | a `PROGRAM` frame, a `METHOD` frame, an `INTERPRET` frame | **partly** — `stack_frames.rex` prints `~type` for the frames it walks; the `INTERPRET` and `COMPILE` values are **unreachable here** and are register row 6 |
| `WeakReference` | `r = .WeakReference~new(o)` over a live referent | **a reference whose referent has been collected** — the state the class exists for | **yes, but not from a corpus program**: the witness's own header says the clearing half is not observable from Rexx, and it is asserted in `collect_stress.rs`'s `a_weak_reference_clears_only_when_its_referent_becomes_unreachable` |
| `Object` | `r = .K~new`, a user object | **a String, an integer, a class object, `.nil`**, and any object whose body is a `Body::Native` | **no, before this task — and it is where the sweep found both of §5's divergences**, `Object~copy` and the six identity operators |
| `RexxInfo` | `r = .RexxInfo`; **no class arm** | **there is no second instance a program can build** — both sides refuse `.RexxInfo~class~new`, so the limitation does not arise, **but they refuse differently**: see below | n/a |
| `Pointer`, `Buffer` | **class arm only** — no instance arm at all, named absent by the receivers file | any instance — none is constructible from Rexx | n/a; Phase 8 owns making one |

**An undeclared loud-versus-loud difference, found while establishing that row.** Re-measured
2026-09-08 from a fresh directory, three descriptors, both engines: `.RexxInfo~class~new` is
**oracle rc 163**, `Error 93 … Incorrect call to method.` under a
`Compiled method "NEW" with scope "RexxInfo".` trace line, and **this crate rc 120**,
`method "NEW" of class "RexxInfo" is not implemented (Phase 5)`. Both refuse, so it is safe; the
status and the text differ, so it is a divergence. **No table covers it**: `RexxInfo` has only an
`instance` arm in `corpus/introspection-receivers.tsv`, and neither `corpus/method-bodies.txt` nor
`corpus/introspection-arity.tsv` carries a `RexxInfo new` row or any `RexxInfo` class arm. Homeless
in the same way `.VariableReference~new` was, and declared here rather than left in a probe log —
**register row 20**.

**The pattern**: the classes where a second instance is different *in kind* are the ones whose two
instances come from different machinery — a program's package versus the interpreter's, a Rexx
method versus a native one, a user class versus a built-in, a live weak reference versus a cleared
one. Where that is true, the corpus witness is the instrument that reaches it and the arity table is
not.

---

## 9. Declined rows, and two things found and declined with their measurements

### Declined, each with its reason

| row | reason |
| --- | --- |
| `RexxInfo~executable`, `RexxInfo~libraryPath` | they answer a `.File`, and `File` is Phase 7's |
| `Method~setSecurityManager`, `Routine~setSecurityManager`, `Package~setSecurityManager` | implementing them without interception ships rc 0 where the oracle raises |
| `Method~loadExternalMethod`, `Routine~loadExternalRoutine` naming a non-`REXX` library | Phase 7, the boundary `directive_gap` already draws |
| `Package~loadLibrary` | the same boundary: a native library load is Phase 7's |
| `Method~newFile`'s package-context argument | Phase 7 |
| `Class~new` | the raw metaclass primitive — see below |

### The rest of `corpus/introspection-arity.tsv`'s non-`agree` rows

The declined rows above are its `send-differs` set, with one qualification: `Method~newFile`'s
package-context argument is a declined **argument form**, not a `send-differs` row — `Method~newFile`
and `Routine~newFile` both read `agree rc0`. Everything else in that table is `send-differs`, and
every `send-differs` row is in that table.

The remaining verdicts are not declines and were wrongly folded into a single sentence in an earlier
draft, which named `Object`'s "three" `exempt` rows — there are more, and they are named below — and
omitted the `no-value` ones entirely. Enumerated from the committed table:

* **`exempt`**, all on `Object`: `(abuttal)`, `(blank)`, `new` (class arm), `run`, `setMethod`,
  `unsetMethod`. Each carries a committed reason in the table's own evidence column instead of an
  argument list.
* **`unstable`**, both on `Object`: `hashCode` and `identityHash` — licensed to diverge, and policed
  by `every_unstable_row_is_really_unstable`.
* **`no-value`**: `Class~activate`, `~define`, `~defineMethods`, `~delete`, `~inherit`,
  `~uninherit`; `Method~setGuarded`, `~setPrivate`, `~setProtected`, `~setUnguarded`; and
  `Object~objectName=`. These are **not declines and need no destination** — the send completes
  identically on both sides and returns nothing, so there is no value to compare, which is what the
  verdict says. What they need instead is a **side-effect** witness, and all but one have a corpus
  program that reads the effect back: `method_introspection.rex` for the `Method` setters,
  `class_behaviour_snapshot_delete`/`_inherit`/`_subclass`/`_uninherit` and
  `class_mutator_define_methods_supplier` for `define`, `defineMethods`, `delete`, `inherit` and
  `uninherit`, and `class_context_identity.rex` for `objectName=`.
* **The exception is `Class~activate`, which has no side effect to read** — and that is correct
  rather than hollow: it is bound to `native_no_op`, because `memory/Setup.cpp:497`'s own comment
  says "this is a NOP by default, so we'll just use the object init method as a fill in", and
  `RexxObject::initRexx` (`classes/ObjectClass.cpp:2546`-`:2549`) takes no arguments, does nothing
  and answers `OREF_NULL`. A no-op is the oracle's behaviour, not an unimplemented shell.

So every `send-differs` row has a destination, and no other non-`agree` verdict is a row owing
work.

### `Class~new` on the class arm — declined **with a destination**, and measured

`.Class~new('NEWCLS')` answers a class whose id is `NEWCLS` and whose superclass is `Object`, and
whose instances then cannot be constructed at all: `c~new` is
`97.1  Object "The NEWCLS class" does not understand message "NEW"`. It is the raw metaclass
primitive — class **creation** rather than class introspection — and it belongs with the
class-definition surface, not here.

### Turning the arity instrument's value comparison on for the collection driver

An **open opportunity with a named cost**. `corpus/introspection-arity.tsv` compares the answered
value; `corpus/collection-arity.tsv` does not. Turning it on there re-verdicts that file's `agree`
rows, and **whatever it finds is a Phase 5g or 5h defect rather than a 5i one**. `Class~enhanced` is
what that comparison found on this phase's own classes, which is the evidence that it finds things.
It also does not reach blindness 4: a `vv~string` comparison reads `a StringTable` for a real table
and a half-real one alike.

---

## 10. Two properties no corpus program can witness

Named here as **structurally unwitnessable rather than unmeasured**. Each rests on a probe in
Task 4's report.

* **That `newFile` installs a loaded file's directives.** A corpus program has no second file, and
  loading itself would re-run the program from inside itself.
* **That a library *name* is case-sensitive.** A program containing `LIBRARY rexx` cannot agree on
  three descriptors: the oracle answers `.nil` and this crate refuses.

---

## 11. The `diverges` rows of `corpus/refusal-sites.tsv`, with owners

That file's header says every `diverges` row is in this report with an owner. They are refusals that
are safe — loud, never a wrong answer — and diverge because the oracle answers where this crate
refuses. Owners are assigned by the **subject each witness names**; none was re-measured here.

| constructor | witness | owner |
| --- | --- | --- |
| `Loud::accessor_variable` (`lib.rs:1177`) | `say o~"A.B"` over `::attribute "A.B"` | whoever owns directive-generated accessors for non-symbol names |
| `Loud::builtin_option_object` (`lib.rs:1230`) | `.context~condition` inside a handler | whoever implements `CONDITION('O')` — register row 10 |
| `Loud::deferred_send` (`dispatch/native.rs:512`) | `.Stream~new('x')~lineIn(1,2,3,4,5)` | Phase 7 |
| `Loud::delegate_variable` (`lib.rs:1195`) | `::method m delegate a.b` on an instance | whoever owns `DELEGATE` |
| `Loud::method_from_source` (`lib.rs:1014`) | `self~setMethod('Z', .nil, 'BOGUS')` | whoever owns `setMethod`'s source forms |
| `Loud::native_method` (`lib.rs:820`) | `m~dimensions`, `'abc'~copy`, `.Message~new` | split: `'abc'~copy` is **register row 17**, `.Message~new` is Phase 6, `m~dimensions` is the mapped-collection surface |
| `Raised::no_method` (`error.rs:2157`) | `.DN~new~zzz` under a `::method defaultName` | whoever owns the `97.1` rendering of a defaultName |
| `Loud::object_method` (`lib.rs:1029`) | `self~setMethod('ZZ', "return 'x'")` inside `::method cm class`, then `.K~zz` | whoever owns class-scope `setMethod` |
| `Loud::receiver_class` (`lib.rs:697`) | `a. = 'dflt'` then `a.~length` | whoever owns a stem's default-value receiver |
| `Loud::unreadable_collection` (`lib.rs:894`) | `.K~defineMethods(.local)` from a method body | whoever owns `.local` as a readable collection |

---

## 12. The D59 exit obligation

**Moritz corrected D59 on 2026-09-08.** It was a *temporary licence* taken to unblock the object
model without weak references, recorded in the spec in the voice of a permanent decision. **Built-in
classes may have a static lifetime; every other class is collected like any other object. No
permgen.**

**The licence's premise expired inside this phase.** Task 2 made `Body::WeakRef` constructible and
`rexx-core/src/heap.rs`'s weak protocol reachable from a Rexx program for the first time — so the
reason to hold D59 went away in the phase that was still citing it. **Nothing in this report cites
D59 as licensing anything.**

**Four things turn from licensed back into defects**, and D60 reopens with D59:

* `~subclasses` holds **strong** references where the oracle's list is weak — Task 5 shipped that row
  recording the difference as "does not arise under D59". **It arises.** This is a defect, not an
  observable difference.
* A `WeakReference` to a dropped class must clear; D59a licensed it answering the dropped class.
* `Interp::class_variables` is a permanent root, so a class-scope instance variable retains what it
  holds for ever — a real leak.
* A class's `UNINIT` runs later than the oracle's.

**Blast radius, which is why it is not a row-sized change.** The registry is monotone and class
identities live **outside the arena**, in `crates/rexx-classes/`. Collecting user classes means
moving those identities into the arena and giving up the monotone registry — structural work in that
crate, not a body. It also reaches `~subclasses`' list representation, the weak protocol's treatment
of a class referent, `class_variables`' rooting, and `UNINIT` ordering at termination.

**When: after Phase 5i closes and before any new work starts.** Not implemented here; carried as
register row 2.

---

## 13. Two observable differences this phase named rather than hid

* **The `Supplier` from `~methods`/`~instanceMethods` iterates in a different order** — hash order on
  the oracle, name order here. **This is an observable difference, not a defect**: matching the
  oracle would mean replicating its hash function, and nothing in this phase decided to. Its
  resolution is a method rather than a divergence — sort both sides' output and compare the sorted
  output, which Tasks 7+8 built for stdout as `StdoutComparison`/`stdout_multiset`, licensed as
  **DEVIATION 8** with a control that holds the opt-in list in both directions. That control took
  two of the three candidate programs back off the list, and the licence ends the phase with one
  entry, `lang/package_writes.rex`.
* **`~subclasses` holds strong references where the oracle's list is weak.** **This is a defect**,
  per §12, and it is recorded as one.

---

## 14. Why the coverage gaps were all found by reviewers

Every task in this phase disclosed a great deal, and **every coverage gap was found by a reviewer,
never by the implementer**. That is not a failure of candour. Task 5's own account of it, which
generalises across the whole phase:

> Everything I did disclose was something I had tripped over — a failing neighbour test, a ruling I
> could only half-apply, a reading error. This one never failed anything, so nothing made me look.
> Coverage gaps are invisible by construction, and asking "what constant satisfies this row?" is a
> question I have to ask deliberately rather than wait to be prompted by a red.

A gap emits no signal. This is an argument **for** the review seat, not against the implementers —
and the same argument applies one level up to this report, which found `Object~copy` only because
the close was told to hunt rather than to summarise.

**Two controls lied in this phase's favour**, which is the same lesson from the other side: one did
not run at all (a `str.replace` asserting against text `rustfmt` had rewrapped, so the script wrote
nothing and the unmutated binary reported STILL GREEN four times), and one ran, mutated real code,
and was green because the three witness rows it should have reddened all fail on an *arity* check
and never reach the branch that was removed. **Assert every scripted edit; for a green control, name
the row that should have reddened and why it did not.** §4's mutation pair follows that: the edit
was applied by an assert-then-replace script, the binary's `sha256` was checked against the
unmutated one, and the prediction was written before the build.

---

## 15. The handover

What the next phase reads.

### Classes with no constructible instance — **Phase 8**

* **`Pointer`** and **`Buffer`.** `.Pointer~new` and `.Buffer~new` raise `93.967` on both sides, and
  that documented refusal is all Phase 5i owns. `corpus/docs/class-set.txt` gives neither a
  construction expression because the reference says instances come only from native code. Phase 8
  owns making one that holds something; until then `Pointer`'s five instance rows are `unanswered`
  by construction, not by neglect.

### **Phase 6** (*Concurrency*)

* **`Message`**, 17 rows. Eight of them — `start`, `startWith`, `reply`, `replyWith`, `wait`,
  `notify`, `halt`, `messageComplete` — are concurrency whatever else is true.
* **`EventSemaphore`** and **`MutexSemaphore`**. Decided rather than measured: whether
  `post`/`isPosted`/`reset` need concurrency would not change where they belong.
* **`~thread` is wrong now, not in Phase 6** — register row 11. `Object~start` is not refused, so a
  `::method` that `REPLY`s is on another system thread on the oracle and the same one here.

### **Phase 7** (*Streams & platform*)

* **`Stream`** 25 rows, **`File`** 3. The refusal is the `stream_uninit` LIBRARY entry point, which
  names Phase 7 itself.
* **`RexxQueue`** 14 rows, **additionally gated on D7**, which is reopened. `RXQUEUE`, cross-process
  `QUEUED` and this class share that one dependency.
* **Arriving with them**: `RexxInfo~executable` and `~libraryPath` (they answer a `.File`),
  `Package~loadLibrary`, `Routine~loadExternalRoutine` and `Method~loadExternalMethod` for a
  non-`REXX` library, and `Method~newFile`'s package-context argument.

### Rows this phase declined that belong to no phase above

* **`Class~new`** — the raw metaclass primitive, to the class-definition surface (§9).
* **`StackFrame~executable`** and **`RexxContext~copy`** — `Setup.cpp` rows in neither table, left
  loud and named rather than missed. Register row 8.
* **`.VariableReference~new`** — one line with Task 2's helper, and no table's row, which is why it
  was homeless. Register row 4.

### The register is the handover's other half

`found-not-fixed-register.md` carries what this phase measured and does not own. **Read it before
scoping anything**: the ones that bite first are the D59 removal (row 2, owed *before* new work),
the builtin-argument-run panic (row 1), the `DO OVER` use-after-free (row 15), the half-real package
tables (row 16), and the rows this task added — `Object~copy` (17), the zero-length `Rc` (18), the
identity operators (19), and `.RexxInfo~class~new` (20).

---

## 16. What this close does **not** close

**Phase 5's own exit gate has never been assessed**, and the scope note records that closing
**Phase 5** is blocked on that rather than on these rows. It is not this plan's work and was not any
task's. The parent plan's exit clauses with no delivery evidence are:

* **security-manager interception points (D12)** — and this phase declined the `setSecurityManager`
  rows on `Method`, `Routine` and `Package` precisely because that machinery does not exist;
* **cold start measured against C++ (D2)**;
* **rung L2**.

Two earlier drafts of this paragraph were wrong in the same way, and the second was the fix for the
first. It said "every closed phase has a gate document" — a universal over a set nobody had
enumerated — and the correction replaced it with a **hand-written** list, which missed four files.
A hand-written list looks like evidence and is only a memory. So here is the command and its output,
run 2026-09-08 from the repository root:

```
$ find docs/superpowers -iname '*gate*' | sort
docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md
docs/superpowers/plans/phase-2-gate.md
docs/superpowers/plans/phase-3-gate.md
docs/superpowers/plans/phase-4a-gate.md
docs/superpowers/plans/phase-4b-gate.md
docs/superpowers/plans/phase-4c-gate.md
docs/superpowers/plans/phase-4d-gate.md
docs/superpowers/plans/phase-4e-gate.md
docs/superpowers/plans/ppwizard-gate.md
docs/superpowers/records/2026-07-28-phase-3-parser/gate-report.md
docs/superpowers/records/2026-08-15-phase-5-object-model/review-r2-gate.md
docs/superpowers/records/2026-08-26-phase-5a-gate-close
docs/superpowers/records/2026-08-27-phase-5b/gate-parallelization.md
docs/superpowers/records/2026-09-07-phase-5h-mapped-collections/gates.md
```

**So "none for the 5b-5h sub-phases" was false**:
`records/2026-09-07-phase-5h-mapped-collections/gates.md` opens `# Phase 5h - gate readings`. Each of
the four the hand-written list missed was opened and read rather than judged by its name: the Phase 3
one is an implementer's report on that phase's gate closure and points at `plans/phase-3-gate.md` for
the assessment itself; the 5b one is about **parallelizing gate runs**, not about a phase's gate; the
5h one is per-task gate readings; and `records/2026-08-15-phase-5-object-model/review-r2-gate.md` is
a **reviewer's review of the criteria the Phase 5 spec proposes** -- its own opening says the lens is
"find a criterion that cannot fail" -- not an assessment of whether Phase 5 met them.

**The load-bearing conclusion narrows to what that command can support, and survives**: of the files
`find` returns, **none assesses Phase 5's own exit clauses**. How each was ruled out, so the ruling
can be checked rather than trusted -- the `plans/phase-2`, `-3` and `-4a` through `-4e` files name a
different phase in their own filenames; `plans/ppwizard-gate.md` opens
"PPWIZARD as a gate ... parked the same day"; `plans/2026-08-26-phase-5a-gate-close.md` opens
"# Phase 5a gate close" and is the sub-phase; the four `records/` files are the ones read above. The
nearest thing to an assessment is `review-r2-gate.md`, which reviews the criteria rather than
recording delivery evidence for D12, D2 or rung L2.

The wider claim -- that no such assessment exists anywhere under any filename -- is the scope note's,
taken from its own search, and was **not** re-derived here; `find -iname '*gate*'` cannot support it.
**Closing Phase 5i is not blocked on that. Closing Phase 5 is.**

---

## 17. Gates

### Fast checks, in the working tree, before the commit

Read unpiped, each from output on screen. The commit these precede touches no `.rs` file, so the
code state they measured is the code state the commit carries.

| check | command | result |
| --- | --- | --- |
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| clippy, **clean target directory** | the same command with a fresh `CARGO_TARGET_DIR`, per `rust/CLAUDE.md`'s phase-boundary rule | exit 0, and the log shows `rexx-core`, `rexx-parse` and `rexx-exec` were checked rather than reused |
| tests | `cargo test --release --workspace --no-fail-fast` | exit 0, 116 `test result: ok`, 2332 passed, no `FAILED` |

### The full suite, at `1284ef198`

Run in the gate worktree `/home/moritz/dev/repos/ooRexx-5i-gates`, `git checkout --detach`ed to this
report's own commit, which nobody edits while it runs. **`532cbe2cf` was never gated**; this run is
what establishes the phase's final gated state. Every cell is read from a `gate-status.txt` whose
first line is `1284ef198501e2f210e1df0ce63e97ca534aea06`, written as each command exited —
`started 2026-09-08T15:04:08+02:00`, `finished 15:15:50`, with G8 appended after at the same pinned
commit. **One irregularity in that file, stated rather than tidied**: G1–G3 and G5–G7 each have a
`--- Gn: <command>` header line above their status because they went through the script's `run`
helper, and **G4 has only its `G4 exit 0` line** — it was spelled out inline to carry
`REXX_CORPUS_GATE=1` and `memcap`, so its command is recorded here and not there.

| gate | command | result |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | exit 0, 116 `test result: ok`, 2332 passed, no `FAILED` |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0, 116 `test result: ok`, 2333 passed, no `FAILED`, `mode: STRICT`, `463 of 463 matching` |
| G5 | `cargo test --release -p rexx-exec --test collection_arity` | exit 0, 25 passed |
| G6 | `cargo test --release -p rexx-exec --test introspection_arity` | exit 0, 27 passed |
| G7 | `cargo test --release -p rexx-exec --test introspection_scopes` | exit 0, 24 passed |
| G8 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | exit 0, 27 passed, `mode: STRICT`, `463 of 463 matching` |

`corpus.rs` reports `1 ignored` in G3, G4 and G8. It is `probe_emit_uncaptured_marker`, whose own
line says why: *ignored, run only by `demonstrate_the_report_reaches_a_plain_cargo_test`, as a child
process*. It is ignored by design, not skipped.

G4 reports one more passing test than G3 because it is the **debug** run: `[profile.release]` sets
`debug = true` for symbols and does not set `debug-assertions`, so every `debug_assert` in the
workspace — `Interp::enter_clause`'s tripwire among them — is compiled out of everything a
`--release` gate runs.

### Which instrument asserts the differential, and in which profile

**G5, G6 and G7 here are not the same three commands earlier tasks in this phase ran.** They ran the
STRICT corpus differential, `collect_stress` and `method_bodies`; this run puts the two arity tables
and the scopes table in those slots, because they are the instruments this task refreshed and read.
So the substitution is stated rather than left to be noticed, and the coverage it might have dropped
was checked by reading the logs rather than assumed:

* **`collect_stress` and `method_bodies` ran inside both G3 and G4** as workspace members —
  confirmed from the `Running tests/…` lines: `collect_stress` 9 passed in each,
  `method_bodies` 23 passed in each, `corpus.rs` 27 passed in each.
* **The differential is asserted in both profiles.** G4 is the debug STRICT run
  (`463 of 463 matching`) and **G8 is the release STRICT run** (`463 of 463 matching`), added for
  exactly this reason. G3's own `corpus.rs` carries no `REXX_CORPUS_GATE`, so it runs in **report
  mode and exits 0 on a divergence** — its `463 of 463 matching` line is a reading, not an
  assertion, and nothing here rests on it.

So every gate the earlier tasks asserted is asserted here, and the three tables this task refreshed
are asserted as well.

**These cells are the readings taken at `1284ef198` and were not re-run.** The fix rounds after them
changed this report, the found-and-not-fixed register and `2026-09-07-phase-5i-scope.md` — docs only,
nothing under `rust/` — so a re-run would measure the same code against the same corpus.

**Where this table lives, relative to the commit it describes.** The gated commit is `1284ef198`.
Filling these cells is itself an edit, so the commits carrying the filled table and the fix round
are that commit's **docs-only children** — a report cannot contain the results of the run that
measured it. They change docs and no `.rs`, so `1284ef198` remains the state the seven gates and G8
were taken at, and they are not separately gated. That is the same shape as the delta the phase
inherited at `532cbe2cf`, stated here rather than left for the next reader to discover.

---

## 18. Every ruling, against what was done with it

| ruling (all 2026-09-08) | what was done |
| --- | --- |
| **The close runs an adversarial hunt, not only a sweep** — name the class a defect would share and enumerate spellings of it against the oracle | **Applied.** Each candidate class the ruling named became a three-way probe: object-valued answers' classes, case sensitivity in every parsed name, copies, an empty container against a package with a real `::requires`, and a constant reader varied. It found two defects — `Object~copy` and the identity operators — neither of which any amount of code-reading produced. §5. |
| **Every defect this phase found returned success** — say it in the close, as the phase's central finding about its own instruments | **Applied.** §1 opens with it and states the four blindnesses as properties of the instruments, with an owner each, and says plainly that every green cell the phase produced is qualified by all four. |
| **A control's green needs the same suspicion as its red** — assert every scripted edit; for a green control, name the row that should have reddened and why it did not | **Applied.** Every scripted edit in this task was an assert-then-replace (`assert s.count(old)==1`). The mutation's binary was checked by `sha256` against the unmutated one and by mtime. No control in this task came back green where a red was predicted, so the "name the row that should have reddened" clause had nothing to bite on — the two lying controls the ruling cites are quoted in §14 as the reason the discipline exists. |
| **Carry the declined rows and the found defects with their measurements** | **Applied.** §9 lists every declined row with its reason and shows that the declined set exhausts the arity table's non-`agree` rows; §15's handover repeats them by destination; the found-and-not-fixed register carries the builtin-argument-run panic and `.VariableReference~new`, and the `::ATTRIBUTE` upstream candidate stays at one signal. |
| **Two properties no corpus program can witness** — name them as structurally unwitnessable rather than unmeasured | **Applied**, §10: `newFile` installing a loaded file's directives, and a library name's case sensitivity. |
| **Two observable differences Task 5 named rather than hid** — the `Supplier` order is a difference, `~subclasses`' strong references are a **defect**, and D59 licenses nothing | **Applied**, §13. The `Supplier` order is stated as a difference with the reason (matching it means replicating a hash function), and its resolution as DEVIATION 8's sorted comparison. `~subclasses` is stated as a defect. **No sentence in this report cites D59 as licensing anything.** |
| **Run a coverage mutation as a pair, and the green run is the finding** | **Applied in the form this task could take, and the difference is stated.** Task 9 added no witness, so there is no "with and without the new test" pair to run. What was run instead is the mutation against the **whole corpus gate**, which is the "adds coverage" question in its other form: which programs catch it. Prediction written first, in the scratch file. §4. |
| **Why coverage gaps are the hardest thing to self-disclose** — say so in the close, because it argues for the review seat | **Applied**, §14, with Task 5's own account quoted and the same argument turned on this report. |
| **Cumulative perf drift is Task 9's, and it has named axes** | **Applied**, §6. Four axes, interleaved, medians, separate target directories, `sha256` checked, plus a null control. **The phase total is Task 6's single step on all four axes** — the "about what this one step suggests" outcome, not the several-times-it one. |
| **Sweep the zero-length `Rc::from` trap, after the tree is stable** — a candidate is not a defect; a cold-path hit is a note | **Applied**, §7. Swept at `HEAD` with the tree clean. One live hit (`run.rs:1446`, common-case empty **because of** register row 13, on the call path), two cold notes, the rest not the shape. No code changed, and the mechanism claim was corrected: fixing row 13 removes the **zero-length case**, not the allocation — the same `Rc` is then built with a payload — and a guard would leave the wrong answer and never fire afterwards. The cost belongs with Task 6's residue and was not A/B'd. |
| **The perf baseline commit, pinned to `bc9d991cc`; do not use a stored figure as the baseline side** | **Applied.** Both ends re-measured from their own detached worktrees; no per-task figure was used as a baseline. Task 6's numbers appear only in a comparison column, labelled as its own step. |
| **The arity instrument probes one instance per class, and that bounds every green row** — say it, name the classes whose second instance differs in kind, do not re-derive it by re-running the tables | **Applied**, §8. Derived by reading the receiver expressions, not by re-running; every class in the table is named with its second instance in kind and whether a corpus witness reached it. Two of the "no" answers turned into this task's findings. |
| **Pre-flight against the tree; do not cite a line number you have not just checked; the blindness count is four, not three; and Task 9 must not attempt to fix any of them** | **Applied.** Every line number in this report was re-checked at `532cbe2cf` immediately before it was written (`dispatch.rs:7558`, `run.rs:1446`, `package.rs:538`, `eval.rs:1831`, `dispatch.rs:2108`). The four blindnesses are §1's spine. **Nothing was fixed**: this task changed no `.rs` file. |
