# Task 24 -- the 5a gate

**Status: DONE_WITH_CONCERNS.** The gate was measured rather than asserted, and the measurement
says **5a is not closed**. Five rows of the two gate tables, owned by 5a, do not `agree` with the
oracle, and each stops at a 5a mechanism that none of this plan's twenty-three preceding tasks
built. `5a` is therefore **not** in `CLOSED_PHASES`; the one line that puts it there, the five rows
that must go green first, and the cost of that call being wrong are recorded in the plan.

Commits, oldest first:

| sha | what |
|---|---|
| `cd4e6894a` | Ask the hierarchy edge rows their question with 5a's own machinery |
| `5b4302f59` | Replace Phase 5's class count with the criterion a command answers |
| `ae7d291f0` | Record what the gate reddens, and why the flip stays out of the tree |
| `059b38124` | Give the amended Phase 5 note the fraction its own numbers say |
| `433ac60fc` | Hand 5b and 5c the two boundary items the gate found and the lists missed |

None of the five touches `rust/crates/*/src`, `rust/Cargo.toml` or `rust/Cargo.lock` --
`git diff --stat 571275f0b..433ac60fc -- 'rust/crates/*/src' 'rust/Cargo.toml' 'rust/Cargo.lock'`
prints nothing -- so the release binary the guard's axes measure is byte-identical to the one at the
task's base and **no sitting is owed**. The guard's standing is reported below anyway, because the
brief asks for it.

### Where each item the brief asks for is answered

| brief item | answer | section |
|---|---|---|
| the five gate commands | **all five exit 0** at `433ac60fc`, each status read unpiped | 13 |
| both tables: every 5a row `agree` | **not met** -- 4 in table C, 1 in table D | 1, 3 |
| the 5b and 5c rows with their verdicts | measured lists, both tables | 3 |
| the wiring half of the class-set criterion | 62/63 class, 57/57 edge, `ArgUtil` held | 4 |
| `RexxInfo`, both halves | out of the class set **and** required as an instance entry; the second half fails | 4 |
| `Queue`, `Stem`, `VariableReference` | all three `agree` | 4 |
| surviving `native_classes.rs` deferrals | 3 stand; 2 outside the criterion, `RexxInfo` inside | 4 |
| D56's re-derivation run | `oodocs/` present at r13198; the run is in section 13 | 5, 13 |
| the corpus, and what `phase-5a.txt` gained | `phase-5a.txt` 51 -> 193; the gate's own figure is in section 13 | 6, 13 |
| unsafe blocks, and `deny` vs `forbid` | 1 block, 1 opt-in; no crate root carries either | 7 |
| the guard's standing per axis | the shipped sitting's `pinned>head`, both arms, both sizes, **and the control build's `per_pass` rows beside it** | 12 |
| cold start and D2 | 3.85x the oracle, and the bootstrap costs 148.3M `instructions:u` | 14 |
| every committed table this plan edited | six tables, commits and standing | 8 |
| the roadmap amendment | landed, with both measurements | 9 |
| what 5a did not cover, for 5b and 5c | the boundary, re-measured where it could be, plus Task 23's handed-forward cost decision and one divergence no gate row sees | 10, 16 |
| the flip's negative control | recorded, and it discriminates | 11 |

---

## 1. The decision, and what it cost

**The brief says "Add `5a` to `CLOSED_PHASES` in this commit".** I did apply that edit, ran the
gate tables under it, and then took it back out. The reason is a measurement and not a preference.

With the gating arm live the two tables exit **101** on five rows:

```
$ cd rust && REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a cargo test --release -p rexx-exec \
      --test gate_table_c --test gate_table_d --no-fail-fast
rc=101
gated by this run: 4 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
gated by this run: 1 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
```

and, separately, with `pub const CLOSED_PHASES: &[&str] = &["5a"];` applied at
`rust/crates/rexx-exec/tests/gate_tables/mod.rs:344` and **no** `REXX_PHASE_GATE` set at all:

```
$ cd rust && REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
      --test gate_table_c --test gate_table_d --no-fail-fast
rc=101
gated by this run: 4 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
gated by this run: 1 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
4 row(s) of gate table C owned by a closing or closed phase do not `agree` with the oracle; the first
  of them are ["gate-tables/concepts/usingcl.rex", "gate-tables/concepts/xscope.rex",
  "gate-tables/concepts/methna.rex", "gate-tables/classes/rexxinfo.rex"].
1 row(s) of gate table D owned by a closing or closed phase do not `agree` with the oracle:
  ["gate-tables/directives/attribute__external__subkeyword.rex"].
```

The two runs redden the identical rows, which is the check that `CLOSED_PHASES` **is** the switch
the brief believes it is and not a constant nothing reads.

`mod.rs` was restored from a scratchpad copy and diffed with `sha256sum`
(`adda5cfa0a483c734d3a79645aa7ab11afdc8bd03603450cb111c72eec418373`, both sides).

**Both flip runs were at `cd4e6894a`, and the five rows are the same at the reported commit.** The
commits after it are documentation only, and gate 4 at `433ac60fc` printed the identical
tallies from the same two tables -- table C `5a: 135 rows, 4 not yet agree`, table D
`5a: 36 rows, 1 not yet agree`. Those tallies are the input the gating arm reads, so re-running the
flip would redden the same five.

### The five rows, and the mechanism each stops at

Each was also run by hand from a fresh empty directory, three descriptors:

| row | the crate's `stderr`, rc 120 | the task this plan gave it to |
|---|---|---|
| `gate-tables/concepts/usingcl.rex` | `method "MIXINCLASS" of class "Class" is not implemented (Phase 5)`, with `method "SUBCLASS" of class "Class"` behind it | **Task 7**, per the row's own recorded owning-phase authority: "spec enumeration: `mixinClass()` and `inherit()` are 5a, and `~subclass` is the same factory protocol reached by message rather than by directive" |
| `gate-tables/concepts/xscope.rex` | `method "SCOPE" of class "Method" is not implemented (Phase 5)` | **Task 9**: "the method dictionary's one entry per scope, and `Method~scope`, are 5a" |
| `gate-tables/concepts/methna.rex` | `a method built from source text is not implemented (Phase 5)` -- this is `~define(name, 'source')` | **Task 21**: "the probe needs `~define` and `~method`, which the enumeration files under 5a" |
| `gate-tables/classes/rexxinfo.rex` | `environment symbol ".REXXINFO" is not implemented (Phase 5)` | the standing `RexxInfo` deferral in `rust/crates/rexx-classes/src/native_classes.rs` |
| `gate-tables/directives/attribute__external__subkeyword.rex` | `::ATTRIBUTE EXTERNAL is not implemented (Phase 7)` | **Task 22**, which moved `::METHOD ... EXTERNAL 'LIBRARY REXX name'` into 5a (`rust/crates/rexx-exec/src/lib.rs:1510`-`:1511`) and left the `::ATTRIBUTE` spelling of the same `LIBRARY REXX` form at Phase 7 (`:1527`) |

The last one is worth separating from the other four, because its filing is arguable and I did not
change it. Table D's `owning_phase` sweeps `::ATTRIBUTE` into 5a with a catch-all
(`rust/crates/rexx-exec/tests/gate_table_d.rs:236`-`:247`, the catch-all at `:244`) whose doc enumerates every exception it
makes -- `::ROUTINE EXTERNAL` to Phase 7, `DELEGATE` to 5b, the two cross-reference rows -- and
never mentions `::ATTRIBUTE EXTERNAL`. So the row is filed 5a by omission rather than by decision.
**Re-filing it to Phase 7 would close a gate row by narrowing what the gate covers, and I declined
to do that.** The probe names `LIBRARY REXX`, which is present, and that is the exact spelling D37
moved into 5a for `::METHOD`. Measured, the whole reachable behaviour of the form on this build is
one error: `/bin/grep -n "INTERNAL_METHOD(GET" interpreter/runtime/NativeMethods.h` and the `SET`
form both match nothing, so no `::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX x'` can resolve `GETx` on
either side. That makes it a small and *complete* piece of work for whoever picks it up.

### Why the flip is not committed

`global-constraints.md` says, in terms: "There is no standing red and no exception to read past, so
the two corpus gate commands exit zero and 'the corpus is green' means what it says ... **any red is
a regression**: a task that finds one stops and diagnoses it rather than matching it against a name
it was told to expect." Committing the flip would put five standing reds under both corpus gate
commands and make that sentence false for every task after this one -- and the first thing a later
task would then need is a list of names to read past, which is the mechanism the constraint exists
to prevent.

**Cost if this call is wrong:** the line is not in the tree and someone has to remember it. Against
that, `docs/superpowers/plans/2026-08-17-phase-5a.md`'s Task 24 section now carries the exact line,
its file, the five rows that must agree first, and this reasoning -- so the thing to remember is
written where the next reader of that task looks. The alternative cost is a gate every later task
reads past, which this project has paid before.

---

## 2. The hierarchy edge rows: an instrument that could not answer its own phase

This is the one piece of work in the task beyond measuring, and it is not one of the five above.

**What was wrong.** All 57 hierarchy edge rows of gate table C -- rows of a **5a** criterion -- read
`diverge-both`, loud, at `rexx-exec: method "HASITEM" of class "Array" is not implemented
(Phase 5)`. The derived probe asked `.Child~superClasses~hasItem(.Parent)`. `Array hasItem instance`
is a **5c** row (`rust/corpus/docs/class-methods.txt:486`) and `ArrayClass::hasItemRexx` is native
(`/bin/grep -n 'AddMethod("HasItem", ArrayClass::hasItemRexx' interpreter/memory/Setup.cpp` -> 740),
so a 5a criterion could not be answered until 5c landed. That inverts the phase order, and it is a
defect in the instrument rather than in the build.

**The criterion was already met, measured before the change.** With the `documented-edge` line
stripped from each of the 57 probes so this crate reaches the rest of each program, **all 57 agree
with the oracle on all three descriptors** (57 identical, 0 differing). So every documented parent
was already present in its child's `~superClasses`, and the whole list already rendered identically.
That is what makes this an instrument change and not a narrowing.

**What it asks now**, derived by `edge_probe_text` and compared against the committed file in both
directions on every run:

```rexx
say 'child' .Array~id
say 'parent' .OrderedCollection~id
supers = .Array~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .OrderedCollection~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .Array~superClasses~makeString('L', ' ')
```

`~items`, `[]` and `~id` are all answered by this build; the walk needs nothing 5c owns. Both engines
are covered without my having to run them separately: `run_on_both_engines` runs every probe twice,
through `Invocation::with_engine`, and asserts **before any verdict exists** that the two agree on
all three descriptors -- an unconditional, structural assertion no gate mode can relax. So the 57
`agree` verdicts are 57 rows on which the oracle, the `ir` engine and the tree-walker all match. I
also ran the shape by hand on one edge against all three, at rc 0 with identical output, before
regenerating the 57.

**What it gives up, said plainly.** `~id` equality is not object identity. It cannot see a
`~superClasses` holding a different object whose `~id` is the documented parent's. Two things stand
behind that. First, nothing else in this family or the class family asks identity either --
`~superClasses~makeString` renders each member as its `~id` inside `The ... class`, and the class
rows ask `~id` directly -- so the walk asks membership at the fidelity the row already had, and the
`superclasses` line is still a byte-for-byte comparison of the whole rendered list. Second, the
identity question is a mechanism this phase **refuses**. Measured, `if c == .OrderedCollection`
inside a `DO OVER` over `.Array~superClasses`, rc 120:

```
rexx-exec: the operator `==` applied to a class object is not implemented (Phase 5)
```

`an_operator_sent_to_an_object_is_loud` (`rust/crates/rexx-exec/src/eval.rs:3458`) is what pins that
refusal. A wiring row phrased over identity would wait on a later phase for the same reason
`~hasItem` did.

**Effect.** `cargo test --release -p rexx-exec --test gate_table_c`, report mode, before and after:

| | before (`571275f0b`) | after (`cd4e6894a`) |
|---|---|---|
| 5a rows | 135 | 135 |
| 5a rows not yet `agree` | **61** | **4** |
| hierarchy edge rows: `agree` / not | 0 / 57 | **57 / 0** |
| class wiring rows: `agree` / not | 62 / 1 | 62 / 1 |
| concept rows: `agree` / not | 12 / 9 | 12 / 9 |

---

## 3. Both tables, every row, by phase

Read from the report both tables printed inside **gate 4 at `433ac60fc`**
(`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`, exit 0). Both tables print
the same text under the gate as in report mode; only the exit status differs. With `CLOSED_PHASES`
empty and no `REXX_PHASE_GATE` set, both printed:

```
gated by this run: 0 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
```

### Gate table C -- 1488 rows

| phase | rows | `agree` | `diverge-both` | `diverge-stdout` | `unanswered` |
|---|---:|---:|---:|---:|---:|
| **5a** | 135 | **131** | 4 | 0 | 0 |
| 5b | 6 | 0 | 5 | 1 | 0 |
| 5c | 1347 | 134 | 716 | 0 | 497 |

The 135 5a rows are **15 of the 21 concept rows** (12 `agree`; the other 6 concept rows are 5b's),
**all 63 class wiring rows** (62 `agree`) and **all 57 hierarchy edge rows** (57 `agree`). The
`ArgUtil` assertion, which has no verdict channel by design, reads
`held: ArgUtil is a class row and emits no hierarchy edge`.

**The 5b rows, named, because 5b starts from this list.** All six are concept rows:

| row | verdict | what the crate stops at |
|---|---|---|
| `objcla` Object Classes | `diverge-both` | `method "NEW" of class "Object"` |
| `abscla` Abstract Classes | `diverge-both` | `method "NEW" of class "Object"` |
| `usesem` SETMETHOD / ENHANCED | `diverge-both` | `method "NEW" of class "Object"` |
| `creo` Initialization | `diverge-both` | `method "NEW" of class "Object"` |
| `obdes` Object Destruction and Uninitialization | **`diverge-stdout`** | nothing: rc 0 and empty `stderr` on both sides. The oracle prints `main` / `uninit ran`, this crate prints `main`. **A silent divergence** |
| `methodsbyclass` Class Library Notes | `diverge-both` | `method "NEW" of class "Array"` |

**The 5c rows** are the 1347 method rows, sharing 96 programs. 497 of them were asked of neither
side because their group's probe raised at its constructor first, which is why they are
`unanswered` rather than `agree`.

**What the 1222 loud rows of table C are waiting on** -- the report prints this list
alphabetically; it is sorted here by row count, which is the order 5b and 5c would want to read
it in:

```
   422  method "NEW" of class "Object"
   118  method "NEW" of class "String"
    90  method "NEW" of class "Queue"
    70  method "NEW" of class "Directory"
    51  method "NEW" of class "MutableBuffer"
    45  method "NEW" of class "Array"
    38  method "NEW" of class "Package"
    38  method "NEW" of class "List"
    31  method "NEW" of class "Class"
    29  method "NEW" of class "StringTable"
    29  environment symbol ".REXXINFO"
    28  method "NEW" of class "Relation"
    28  method "NEW" of class "Bag"
    27  method "NEW" of class "Stem"
    24  method "NEW" of class "Table"
    24  method "NEW" of class "Set"
    24  method "NEW" of class "IdentityTable"
    20  method "NEW" of class "Message"
    17  method "NEW" of class "Method"
    15  method "NEW" of class "Supplier"
    15  method "NEW" of class "RexxContext"
    10  method "NEW" of class "StackFrame"
     8  method "NEW" of class "Routine"
     5  method "NEW" of class "Pointer"
     5  method "NEW" of class "EventSemaphore"
     4  method "NEW" of class "VariableReference"
     3  method "NEW" of class "MutexSemaphore"
     1  method "SCOPE" of class "Method"
     1  method "NEW" of class "WeakReference"
     1  method "MIXINCLASS" of class "Class"
     1  a method built from source text
```

`method "HASITEM" of class "Array": 57` was on this list at `571275f0b` and is not on it now.

### Gate table D -- 79 rows

| phase | rows | `agree` | `diverge-both` |
|---|---:|---:|---:|
| **5a** | 36 | **35** | 1 |
| 5b | 2 | 2 | 0 |
| 5c | 38 | 3 | 35 |
| 7 | 1 | 0 | 1 |
| `deferred-parse-error-rendering` | 2 | 0 | 2 |

The one 5a row is `::ATTRIBUTE EXTERNAL`, above. The 35 non-`agree` 5c rows are 33 `::OPTIONS` rows,
`::REQUIRES LIBRARY` and `::REQUIRES NAMESPACE`; the three 5c rows that do `agree` are
`::RESOURCE END`, `::ROUTINE PRIVATE` and `::ROUTINE PUBLIC`. The Phase 7 row is
`::ROUTINE EXTERNAL`, and the two `deferred-parse-error-rendering` rows are `::CLASS CLASS` and
`::RESOURCE LIBRARY`, both filed under a deferral nobody owns and which is not Phase 5 work.

---

## 4. The wiring half of the class-set criterion

**The set.** `rust/corpus/docs/class-set.txt` holds **63** rows -- 62 `class`, 1 `instance` -- and
is derived by `rexx-extract-docs` from every `<section id="cls...">` across `fundclasses.xml`,
`collclasses.xml`, `utilityclasses.xml` and `streamclasses.xml`. `crates/rexx-extract/tests/extract_docs.rs`
re-derives it from `oodocs/` and compares it against the committed file in both directions -- that
is the D56 run in section 5. Its three departures from the sections are each named in its own
header rather than mechanical: `RegularExpression` is **out** (delivered by
`::requires "rxregexp.cls"`, not on this build's search path, measured 43.901 rc 213), `ArgUtil` is
**in** (an `.environment` class the books document nowhere, its only citation the XML comment at
`provide.xml:838`), and `RexxInfo` is an `instance` row rather than a `class` row.

**The standing**, from gate table C's report inside gate 4 at `433ac60fc`:

* **62 of 63 class wiring rows `agree`** -- each asked what its `.environment` entry renders as, what
  its class is, and then `~id`, `~class`, `~superClass`, `~superClasses`, `~metaClass` and
  `~isA(.Class)`, byte-identically on three descriptors against a live oracle launch.
* **57 of 57 hierarchy edge rows `agree`** -- every documented edge present in the child's
  `~superClasses`, and the whole rendered list identical.
* the `ArgUtil` assertion holds.
* **1 class row does not `agree`: `RexxInfo`.**

### `RexxInfo` carries both halves, and only the second one fails

The first half is that `RexxInfo` is **out of the class row set**. Verified in the C++:
`EndSpecialClassDefinition(RexxInfo);` is `interpreter/memory/Setup.cpp:1285`, and the macro at
`:396`-`:397` expands to `addToSystem(#name, currentClass);` -- not `addToEnvironment` -- so no
environment symbol reaches the class object. `class-set.txt` files it `instance`, which is that half
honoured.

The second half is that its `.environment` entry is **required to be an instance whose `~class~id`
is `RexxInfo`**, and it is required because `interpreter/memory/Setup.cpp:1737` reads
`addToEnvironment("REXXINFO", info);` over a `RexxInfo *info = new RexxInfo;`. Writing only "minus
`RexxInfo`-as-an-environment-entry" would drop that and read as dropping the row.

Measured, `corpus/gate-tables/classes/rexxinfo.rex`, three descriptors:

```
oracle rc=159  stdout: "entry a RexxInfo\nclass-of-entry RexxInfo\n"
               stderr:     10 *-* say 'id' .RexxInfo~id
                       Error 97 ... Object method not found.
                       Error 97.1:  Object "a RexxInfo" does not understand message "ID".
crate  rc=120  stdout: ""
               stderr: rexx-exec: environment symbol ".REXXINFO" is not implemented (Phase 5)
```

The row's oracle bound is the two entry questions, which the oracle answers before raising; the six
class questions are then a divergence the two sides must share. This crate answers none of it.

### The deferrals in `native_classes.rs` that still stand

Three: `RexxInteger`, `NumberString`, `RexxInfo`.

**Two of the three are outside the criterion and one is inside it.** `RexxInteger` and
`NumberString` are not named by any `cls*` section -- `/bin/grep -aiE "^(Integer|NumberString|RexxInteger)\t"
rust/corpus/docs/class-set.txt` matches nothing -- so no wiring row asks about them and the
criterion neither covers them nor is weakened by their absence. Their recorded reason is the same
one in both cases and is about a mechanism rather than a name: `CLASS_CREATE_SPECIAL` makes an
`Integer` value's `~class` answer `String`, so modelling them needs a per-value class-identity
override this registry does not build.

**`RexxInfo` is inside the criterion**, is the one class row that does not `agree`, and is one of
the five rows blocking the flip. What it needs is an `.environment` entry that is an instance of a
class whose `~id` is `RexxInfo`, rendering `a RexxInfo`, and answering `~id` with 97.1 -- not the
class object, which by `addToSystem` no symbol may reach.

### `Queue`, `Stem` and `VariableReference`

All three are `class` rows of `class-set.txt` and all three read **`agree`**. They were Task 21's,
and Task 21's own "Done when" excepted them; they are not deferrals, and this report does not note
them in passing -- they are measured members of the criterion that pass.

`ArgUtil` also reads `agree`. So does every other class row: the tally is 62 `agree` against one
`diverge-both`, and the one is `RexxInfo`.

---

## 5. D56's re-derivation run

`oodocs/` and `ootest/` are present in this worktree and at the revisions the constraints stamp:

```
$ svn info oodocs/rexxref | grep ^Revision:   ->  Revision: 13198
$ svn info oodocs/rexxpg  | grep ^Revision:   ->  Revision: 13198
$ svn info ootest         | grep ^Revision:   ->  Revision: 13178
```

The re-derivation is `crates/rexx-extract/tests/extract_docs.rs`, which derives every row set from
`oodocs/` and `interpreter/` and compares each against its committed file **in both directions**. It
is not behind an env var and it **fails** when `oodocs/` is absent unless the run is explicitly
marked `REXX_DOCS_LESS=1`, so a silent skip is not one of its outcomes.

**Run at `433ac60fc`, inside gate 3** (`cargo test --release --workspace --no-fail-fast`), against
the present `oodocs/`:

```
     Running tests/extract_docs.rs (target/release/deps/extract_docs-49150e50b8630822)
test every_row_set_is_exactly_what_its_extractor_derives_today ... ok
test every_row_set_is_stamped_with_the_revision_the_checkout_is_at ... ok
test skipping_the_comment_blanking_fires_the_argutil_assertion ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

**The diff against each committed row set is empty**, which is what the first of those tests
asserts, and the revision stamp is checked by the second -- `derived-at: oodocs/rexxref/en-US
r13198` is the line each row set carries, and r13198 is what `svn info` answers today. This is the
only place the derivations are exercised at a phase boundary: CI's five platforms have no `oodocs/`
and never run any of it.

---

## 6. The corpus, and what `phase-5a.txt` gained

**The brief asks for "106 of 106 with no red". 106 is the corpus as it stood when the plan was
written, and the constraint the plan actually states is that the count "must stay total and grow".**
Both numbers are the same union, measured at two commits:

```
$ for c in 15a1ffa98 HEAD; do
    for f in phase-4a phase-4b phase-4c phase-5a; do git show $c:rust/corpus/$f.txt; done \
      | /bin/grep -av '^#' | /bin/grep -av '^[[:space:]]*$' | sort -u | wc -l
  done
```

| | `phase-4a` | `phase-4b` | `phase-4c` | `phase-5a` | union |
|---|---:|---:|---:|---:|---:|
| `15a1ffa98` (the plan's base) | 31 | 12 | 12 | **51** | **106** |
| `433ac60fc` (here) | 31 | 12 | 12 | **193** | **248** |

**`phase-5a.txt` gained 142 programs**, and the corpus grew from 106 to 248 entirely through it. The
gate figure and the command that printed it are in section 13.

This task adds no corpus program. The 57 hierarchy edge probes it regenerates live under
`corpus/gate-tables/`, which the phase subset files do not list and `corpus.rs` does not read; they
are gate-table probes, run by `gate_table_c.rs`. They are also outside
`sourceline_matches_the_interpreter_for_every_corpus_program`, whose corpus is `corpus/lang/`
(`crates/rexx-parse/tests/sourceline_oracle.rs:143`), so no `sourceline_oracle/*.txt` needed
regenerating.

---

## 7. `unsafe`, and where the level is set

**One `unsafe` block in the workspace's crate sources**, at
`rust/crates/rexx-core/src/bytes.rs:184`:

```
$ /bin/grep -rc "unsafe {" --include=*.rs rust/crates/*/src/ | /bin/grep -av ":0$"
rust/crates/rexx-core/src/bytes.rs:1
```

**One opt-in**, `#[allow(unsafe_code)]` at `rust/crates/rexx-core/src/lib.rs:23`. Both sets are
asserted rather than described, by `crates/rexx-core/tests/unsafe_sites.rs`, which holds them as two
separate lists because the compiler couples them only one way.

**No crate root carries `deny` or `forbid`.** `/bin/grep -rn "^#!\[deny\|^#!\[forbid" --include=lib.rs
--include=main.rs rust/crates/` matches nothing. The level is set once, at `rust/Cargo.toml:31`, as

```toml
unsafe_code = "deny"
```

and all ten crates inherit it through `[lints]` / `workspace = true` in their own `Cargo.toml` --
checked one file at a time, ten of ten. So the answer to "the crate roots carrying `deny` rather
than `forbid`" is: **none carry either; the workspace carries `deny`, and every crate inherits it.**
`deny` rather than `forbid` is what lets an approved module carry an inner `#[allow]`; `forbid`
cannot be overridden, and Cargo refuses a crate that overrides `workspace.lints`. This task adds no
`unsafe` and no opt-in.

---

## 8. Every committed table this plan edited

Measured with `git log --oneline 15a1ffa98..HEAD -- <path>`, run from the repository root. (Run from
`rust/` the same pathspec answers 0 for everything, because a git pathspec is relative to the
working directory -- the trap the plan's own staleness note records walking into once.)

| table | commits this plan made to it | standing now |
|---|---:|---|
| `crates/rexx-exec/tests/assertions.rs` `EXEMPT` | 2 | **35 rows -> 13.** 22 retired; every remaining row is `"Phase 5"`. `git show 15a1ffa98:...` and the working file, counted by `/bin/grep -ac '^    ExemptRow {'` -- **anchored**, because the unanchored needle also matches `struct ExemptRow {` at `assertions.rs:313` and reads one too many at both ends |
| `corpus/bif-exempt.txt` | **0** | 76 rows: 51 `4c`, 22 `Phase 5`, 3 `ANOMALY`. Its set assertion runs under a plain `cargo test` and is policed **in both directions**, so a `Phase 5` row that had started passing would be red -- none is, and that is the check behind "unchanged", not an absence of looking |
| `corpus/bif-exempt.txt`'s attribution column | **0** | the column is not hand-written for a row that fails loudly: it is `rexx-exec`'s own owning-phase message, re-derived on every run, so it cannot drift from `instruction_owner`/`expr_owner` |
| `crates/rexx-exec/tests/owners.rs` | 3 | `ebe91e39b` (GUARD/REPLY), `071835c9c` (`RAISE ADDITIONAL`), `f4b21eadb` (the Array and Directory this phase sends to) |
| `crates/rexx-exec/tests/coverage.rs` | 40 | `EXPECTED_SUBSET`, moved by nearly every task in the plan |
| `crates/rexx-exec/tests/loud.rs` | 2 | the witness tables, the fourth of the five pinned items |
| `crates/rexx-exec/src/lib.rs` | 52 | carries `instruction_owner`/`expr_owner`, the fifth pinned item; the count is the whole file's, not those two functions' |
| `corpus/keyword-exempt.txt` | **0** | 8 rows: 2 `4c`, 3 `Phase 7`, 3 `RAISED`, and **no Phase 5 row at all** -- so this plan had nothing here to move, which is why 0 is the right number rather than a suspicious one. Same both-directions policing as `bif-exempt.txt` |
| `corpus/builtin-status.txt` | **0** | 81 rows: 66 `implemented`, 15 `excluded`. Not hand-maintained -- `builtin_status.rs` derives every row by running both interpreters and asserts equality in both directions, so an unchanged file is a measured statement that the builtin boundary did not move in 5a |
| `crates/rexx-exec/tests/trace_oracle.rs` `PREFIX_COVERAGE` | **0** | 19 prefixes: 15 `Witnessed`, 2 `WitnessedLive`, `+++` `Owned("Phase 7")`, **`>N>` `Owned("Phase 5")`**. `>N>` is the one Phase 5 row and it is 5c's, per the handover below |
| `docs/superpowers/plans/phase-4-exclusions.txt` | 15 | rows moved out as constructs started agreeing, most recently by `c59a80672` (`::METHOD EXTERNAL`) and the two fix rounds after it |

---

## 9. The roadmap amendment

`docs/superpowers/plans/2026-07-27-rust-rewrite.md:453`'s Phase 5 exit clause read
"**32 classes exist and respond**". Both halves of why that is not a criterion, measured:

```
$ /bin/grep -acE "^::[Cc][Ll][Aa][Ss][Ss]" /home/moritz/dev/repos/ooRexx/interpreter/RexxClasses/CoreClasses.orx
32
```

and, on the shipped oracle, iterating `.environment` and counting the entries answering
`~isA(.Class)`:

```
classes 62
entries 69
```

So the clause was `CoreClasses.orx`'s own `::CLASS` count: a build satisfying it holds 32 classes
where the shipped oracle's `.environment` holds **62**, and it can satisfy it without ever running
`StreamClasses.orx`. And "respond" named no question at all.

It is replaced (commit `5b4302f59`) by the wiring criterion gate table C already runs, with
`RexxInfo`'s two halves both stated, and with the documented per-class **method** sets named as 5c's
-- cited to the spec's own 5c section (`2026-08-17-phase-5-object-model.md:328`) with D48 (`:1245`)
as the class half, D48 being the decision that names line 453 as its amendment target. A paragraph
under the table records what the old clause was and what it was satisfiable by, so the amendment is
checkable rather than a silent edit.

---

## 10. What 5a did not cover -- the boundary 5b and 5c start from

Every claim in this section that could be run was run, from a fresh empty directory, both
interpreters, three descriptors. Where a claim was inherited from the plan and re-measured, the
measurement is beside it.

### 5b's

* **`~new`, `init`, and `self~init:super` chaining as instance construction.** This is the single
  largest item in the tree by row count: **1190 of gate table C's 1222 loud rows** name a
  `method "NEW" of class "X"` refusal, `Object` alone accounting for 422.
* **`UNINIT` itself, and everything that reads the propagation flags.** The flags are Task 7's and
  are carried by its constructors. **Travelling with that row: the class-side witness Task 7
  measured and could not use.** A class object is destroyed like any other object, so a
  `::METHOD uninit CLASS` fires with no `~new` anywhere. Measured here, today, all three programs:

  | program | oracle | this crate |
  |---|---|---|
  | a plain `::CLASS` carrying `::METHOD uninit CLASS` | rc 0, `main` / `uninit ran` | rc 0, `main` |
  | a `SUBCLASS` of one carrying it | rc 0, `main` / `uninit ran` / `uninit ran` | rc 0, `main` |
  | an `INHERIT` of a mixin carrying it | rc 0, `main` / `uninit ran` / `uninit ran` | rc 0, `main` |

  **All three are silent** -- rc 0 and empty `stderr` on both sides, differing on `stdout` alone.
  The plan predicted the third would still be loud until the `MIXINCLASS` directive refusal lifted;
  it has lifted (`6aa432f19`), so 5b inherits three runnable differentials rather than two, and does
  not have to construct an instance to see `UNINIT` fire.
* **Per-object methods** -- `SETMETHOD`, `ENHANCED`, `unsetMethod` -- and the object-own scope they
  create. Concept row `usesem` is the gate row.
* **`FORWARD`, and therefore `DELEGATE`.** Table D files both `DELEGATE` rows 5b for that reason.
* **Abstract-*class* enforcement**, because the check lives inside `~new`. Concept row `abscla`.
  The abstract-*method* half is 5a's and has no arm of that probe.
* **`~copy`; `~run`, `~send`/`~sendWith`, `~start`/`~startWith` minus concurrency; the per-object
  first step of the method search order.**
* **The instance-side reading of every 5a limit measured only on a class object** -- the old plan's
  Task 7 debt, this plan's Task 13 (`PRIVATE`), Task 19 (`::CONSTANT`) and Task 21 (D43's two
  orderings).
* **A pre-existing `USE STRICT ARG` defect that is not 5a's, and that Task 23's handover records as
  now reachable from 52 library class-methods** (that count is the handover's, not re-derived here).
  The defect itself was re-measured here, on a purely user-declared class method:

  ```
  oracle rc=163   Error 93 ... Incorrect call to method.
                  Error 93.901:  Not enough arguments for method; 1 expected.
  crate  rc=216   Error 40 ... Incorrect call to routine.
                  Error 40.3:  Not enough arguments in invocation of M; minimum expected is 1.
  ```

  Wrong condition, wrong sub-number, wrong exit status. **Run here against the pinned pre-5a binary
  `bench-baselines/pinned/rexx-run-15a1ffa98`, the same program answers rc 216 and 40.3 as well**, so
  the defect predates this phase and none of it is 5a's. It belongs to whoever owns the
  method-versus-routine split at the `USE STRICT ARG` site, and it is in this boundary because 5b is
  the first phase that will trip over it at volume.

### 5c's

* **`::REQUIRES` with `LIBRARY` and `NAMESPACE`, and namespace-qualified class references.** Both
  table D rows are `diverge-both`, loud at `::REQUIRES is not implemented (Phase 5)`.
* **`>N>`**, whose only route is one of those references. It is the one `Owned("Phase 5")` row of
  `trace_oracle.rs`'s `PREFIX_COVERAGE`, and this hands it to 5c.
* **`::OPTIONS` and the `OPTIONS` instruction.** 33 of table D's 35 non-`agree` 5c rows.
* **`::RESOURCE` and `.RESOURCES`.**
* **`::ROUTINE`'s option surface, and whatever of `.ROUTINES` Task 17's widening left.**
  **The readback of an `::ANNOTATE ROUTINE` is not among them**, and that is measured rather than
  asserted: `.routines["R"]~class~id` and `.routines~r~class~id` both answer `Routine` on both
  interpreters, byte-identically at rc 0, and `corpus/lang/directive_annotate_targets.rex` is the
  committed witness that Task 20 reads the annotation back through both.
* **`Package~findRoutine` is still unbuilt and is still 5c's.** Measured: oracle
  `The NIL object` at rc 0; this crate
  `rexx-exec: method "FINDROUTINE" of class "Package" is not implemented (Phase 5)` at rc 120.
* **`Package~local`, and environment search steps 3 and 5.** Measured: `.Array~package~local~class~id`
  is `Directory` at rc 0 on the oracle and
  `rexx-exec: method "LOCAL" of class "Package" is not implemented (Phase 5)` at rc 120 here.
  `.Array~package~name` already agrees on both sides (`REXX`, rc 0), so it is `~local` and the two
  crossing search steps that are outstanding, not the package object.
* **`PACKAGE`'s cross-package refusal arm.**
* **`~identityHash`'s identity semantics, and D41.** Note that the whole of object identity is here:
  measured, `==` sent to a class object is loud at rc 120, which is why section 2's edge walk asks
  `~id` and not identity.
* **The documented per-class method sets**, which is the acceptance set Phases 6, 7 and 8 enter on.
  Table C holds them as 1347 method rows: **134 `agree`, 716 `diverge-both`, 497 `unanswered`** --
  the last because their group's probe raised at its constructor before any documented name was
  asked, on either side. Most of those 497 unblock with 5b's `~new` rather than with 5c's work.

### Three that belong to neither list, and that this task did not move

**1. A measured, undecided cost: loading the library costs every allocating axis per pass.** This is
**Task 23's Concern 1, handed forward rather than reopened.** Its own words: "Loading the library
costs every allocating axis between +3.59% and +31.35% per pass, attributed to the collector by a
control build ... it is a question about the collector's cost model with a large resident set." The
control build is committed (`23-attempt-2-control-no-bootstrap`, `cfffc7899`) and section 12 reads
its rows: on `strings`, ir arm, `per_pass` `instructions:u`, the pin is 5,369.53, the same code with
the bootstrap suppressed is 5,406.48, and the shipped build is 7,136.86. So the measurement is
settled and what is open is a **decision** -- what a large resident set should cost, now that every
process carries the library. Nothing in 5a's scope was blocked by it and nothing in 5b's or 5c's is
either; it is handed to both because it is the standing cost either will measure against.

**Two things about that attribution, because Task 23's report says both.** Its Concern 1 names the
collector; its body's "What that does not establish" says the control "does not separate *the
collector traces a bigger live set* from any other consequence of a larger resident heap -- a longer
allocation path, a bigger root set walked per allocation", and that no collection count was read. So
**what the control settles is that the cost arrives with the resident library and is per pass**;
which consequence of a larger resident heap it is remains open, and section 14's zero-collections
result does not bear on it -- that is about the bootstrap's own execution, not about a later
program's collections.

**2. `Directory` iteration is a loud refusal on membership grounds, not order.** Measured:
`do i over .local` counting entries answers `local-entries 10` at rc 0 on the oracle, and on this
crate is
`rexx-exec: one of the interpreter's own objects as a DO header's OVER target is not implemented
(Phase 5)` at rc 120. The refusal is about *what may be an `OVER` target*, so it is not an ordering
question and closing it is not an ordering decision.

**3. `::ROUTINE ... EXTERNAL 'LIBRARY REXX name'` diverges, and no row of either gate table can see
it.** Section 1 audits the row filed *into* 5a by omission; this is the row filed *out* of it, and
the audit ran in only one direction until the review pointed at it. `owning_phase` sends
`("::ROUTINE", "EXTERNAL")` to Phase 7, and a row's identity here is (directive, keyword, position),
so **one row covers both spellings** and the probe picks the shared-library form. The `LIBRARY REXX`
form is therefore never asked. Measured, three descriptors:

| program | oracle | this crate |
|---|---|---|
| `say 'main'` + `::routine r external 'LIBRARY REXX Filespec'` | rc 0, `main` | rc 120, empty, `rexx-exec: ::ROUTINE EXTERNAL is not implemented (Phase 7)` |
| the same, plus `say r('/tmp/x.txt')` | rc 168, `main`, then `Error 88.901` from inside `Filespec` -- the routine **ran** | rc 120, empty, the same refusal |

The bare declaration is the sharper witness: the oracle runs the program clean at rc 0 and this
crate refuses it. This is the same `LIBRARY REXX` spelling D37 moved into 5a for `::METHOD`, and the
same argument section 1 uses to keep `::ATTRIBUTE EXTERNAL` in 5a; applied symmetrically it is at
least arguable that this half is 5a's too. **Nothing is concealed** -- Phase 7 does own
`::ROUTINE EXTERNAL`, and `gate_table_d.rs`'s own doc records that the row spans a boundary and
cannot hold both sides of it. What is worth stating at a phase boundary is that **this divergence
has no gate row anywhere**, where `::ATTRIBUTE EXTERNAL`'s does. Closing it needs
`corpus/docs/directive-options.txt` to distinguish the two forms, which is that file's shape and not
`gate_table_d.rs`'s.

---

## 11. The negative control

**The question a control has to answer here**: with the gating arm live, does a 5a row that
currently `agree`s actually go red when its mechanism is reverted -- or is the arm reporting a
number nothing can move?

**The control.** With `CLOSED_PHASES = &["5a"]` applied, `native_superclasses`
(`rust/crates/rexx-exec/src/dispatch.rs:3116`) was changed to answer only the first element of the
class's superclass list:

```rust
     let items: Vec<Option<ObjRef>> = interp
         .classes()
         .superclasses(class)
         .iter()
+        .take(1)
         .map(|class| Some(*class))
         .collect();
```

**It is a mechanism revert and not a bootstrap break**, and that is read off the result rather than
argued from the code: the bootstrap still ran and 1448 of the table's 1488 rows were unchanged. The
distinction matters, because the previous task recorded three controls that reddened *everything* --
each having broken the bootstrap itself -- and a control that reddens everything witnesses far less
than one that reddens a nameable set.

**What it did.** `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test
gate_table_d --no-fail-fast`, exit **101**:

| | flip alone | flip + control | delta |
|---|---:|---:|---:|
| gated 5a rows, table C | 4 | **40** | +36 |
| gated 5a rows, table D | 1 | 1 | 0 |
| concept rows `agree` | 12 | 10 | -2 |
| class wiring rows `agree` | 62 | 45 | -17 |
| hierarchy edge rows `agree` | 57 | 40 | -17 |

**It discriminates, and the split is the one the mutation predicts.** 17 of the 57 edge rows redden
and **40 stay green**. The 17, read out of the report rather than reasoned about, are every edge
whose documented parent is a *second* superclass:

```
Bag/Directory/IdentityTable/Relation/Set/Stem/StringTable/Table <- MapCollection
Array/List/Queue                                               <- OrderedCollection
DateTime/File/String/TimeSpan                                  <- Comparable
InputOutputStream                                              <- InputStream
Message                                                        <- MessageNotification
```

`.Array~superClasses` is `The Object class The OrderedCollection class`, so truncating to the first
element drops the documented parent; an edge whose parent is already first is untouched. That is a
control that breaks one mechanism rather than a wall of red. It also reddens the
two concept rows that ask about the hierarchy (`xmixin`, which asks `~baseClass`, and `chi`, the
class-hierarchy row) and 17 class wiring rows, all on `stdout` alone.

**And it exercises the row the instrument change touched.** The 17 reddened edge rows fail on the
`documented-edge` line that section 2 rewrote as well as on the `superclasses` line, so the walk is
load-bearing and not decoration.

**What this control could not see.** It moves `~superClasses`, so it says nothing about whether the
gating arm would fire for a row whose mechanism is elsewhere -- a concept row, a directive row. What
covers that is the flip run itself: five rows in three different families are already red under the
arm, at exit 101, which is the same channel.

**Restored.** `dispatch.rs` and `gate_tables/mod.rs` were both copied to the scratchpad before
mutation and restored from those copies, `sha256sum` equal on both sides
(`1e93e79a858ccf39c40ed418cec12c77945ce51f3865e1e35b650f6dcf9ef73e` and
`adda5cfa0a483c734d3a79645aa7ab11afdc8bd03603450cb111c72eec418373`), and `git status` clean.

**A hazard this control produced, worth recording.** `cp -a` from the backup restores the file's
**original mtime**, so `cargo build --release --workspace` answered
`Finished ... in 0.03s` and left the *control* binary in `target/release/`. Checked rather than
assumed: `say .Array~superClasses~makeString('L', ' ')` still answered `The Object class` after the
revert, with `git status` clean. `touch`ing the two restored files and rebuilding fixed it, and the
same probe then answered `The Object class The OrderedCollection class`. A revert verified only by
`git status` and a re-run of `cargo build` would have carried the mutation into every measurement
after it.

---

## 12. The performance guard's standing

### This task owes no sitting, and here is the check rather than the claim

```
$ git diff --stat 571275f0b..433ac60fc -- 'rust/crates/*/src' 'rust/Cargo.toml' 'rust/Cargo.lock'
(no output)
```

All commits here are under `rust/crates/rexx-exec/tests/`, `rust/corpus/gate-tables/hierarchy/` and
`docs/`, so the release binary the axes measure is byte-identical to the base's and a sitting would
measure noise. `global-constraints.md` states that exemption in those words.

**Proved rather than argued, by rebuilding.** The `.text` section of `target/release/rexx-run` was
hashed, every crate source `rexx-run` depends on was `touch`ed, and the binary was rebuilt -- seven
crates recompiled from source (`rexx-inventory`, `rexx-lib`, `rexx-num`, `rexx-core`, `rexx-parse`,
`rexx-classes`, `rexx-exec`), read from the build's own `Compiling` lines rather than assumed:

```
$ objcopy -O binary --only-section=.text target/release/rexx-run /dev/stdout | sha256sum
450ff4b7568cc95950d846ac75e3d38c5743baff205c734f45fc2fa65df70f12    (before)
450ff4b7568cc95950d846ac75e3d38c5743baff205c734f45fc2fa65df70f12    (after the forced rebuild)
```

Identical, so no codegen moved and there is nothing for a sitting to measure. This is stronger than
the `git diff` above: a diff says the source did not change, and the hash says the *machine code*
did not.

### The pin is not stale

* **Ancestry.** `git merge-base --is-ancestor 15a1ffa98 HEAD` -> true.
* **Identity.** `sha256sum rust/bench-baselines/pinned/rexx-run-15a1ffa98` ->
  `141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, which is the hash
  `bench-baselines/PINNED.md` records for it.
* **Foreign commits.** `git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml
  rust/Cargo.lock Cargo.toml`, **run from the repository root**, lists **124** commits. (From
  `rust/` the same command answers 0, because a git pathspec is relative to the working directory --
  that is the trap the constraints file records, and it is still there.)

  **The staleness test as written is not literally satisfiable, and this is the number rather than
  an assertion.** It says "every commit this lists is one this plan's ledger records". Grepping each
  short SHA against `progress.md`, **88 of the 124 appear and 36 do not** -- the ledger records
  tasks and the commits it needs to cite, not every commit. So the test was re-read as the thing it
  is for, "is any of these foreign": all 36 were authored on this branch between 2026-08-22 and
  2026-08-25 with subjects naming this plan's own tasks. Exactly one is this task's own,
  `cd4e6894a`; its other four commits touch only `docs/` and do not appear in the list at all, which
  is why the two counts read 124 and 36 unchanged at `cd4e6894a`, `059b38124` and `433ac60fc`. **No
  foreign commit, so the pin stands and the baseline is comparable.** What this check could not see:
  a foreign commit whose subject happened to read like one of this plan's.

### The standing per axis

`bench-baselines/phase-5a-arms.tsv` ends with **two** sittings, and both matter. The last 156 rows
are task **`23-attempt-2-control-no-bootstrap`** at commit `cfffc7899` -- a control build, and
`git diff --stat cfffc7899..HEAD -- 'rust/crates/*/src' 'rust/Cargo.toml' 'rust/Cargo.lock'` is
empty, so it is a control build of this tree's source. The sitting before it is `23-attempt-2`, the
shipped build, and that is the one the table below reads.

`pinned>head` is head over pinned, so above 1.000 is this tree retiring more instructions than the
pin; `instructions:u`, median of 5 rounds:

| axis | ir small | ir large | tw small | tw large |
|---|---:|---:|---:|---:|
| `alloc4c` | 1.2313 | 1.1794 | 1.1546 | 1.1219 |
| `arith` | 1.0473 | 1.0356 | 1.0507 | 1.0421 |
| `compound` | 1.1494 | 1.1339 | 1.1042 | 1.0934 |
| `emptyloop` | 1.0236 | 1.0078 | 1.0225 | 1.0090 |
| `strings` | 1.3476 | 1.3384 | 1.2102 | 1.2047 |
| `varlookup` | 1.0122 | 1.0032 | 1.0060 | 1.0016 |
| `dispatchclass` | 1.0906 | 1.0869 | 1.0897 | 1.0840 |
| `bench-rexxcps/rexxcps.rex` | 1.1654 | -- | 1.1320 | -- |

**Every axis is above the pin, and the control build in the same file already separates the two
halves of it.** These are whole-process counts, and the pin is the crate as it stood *before* the
phase, so the delta is 5a's entire body. Part of it is a fixed per-process cost: the two axes that
allocate least, `emptyloop` and `varlookup`, sit nearest 1.000 and move toward 1.000 at the larger
size, which is the same added instructions divided by more work. **`strings` at 1.348 is not that
shape**, and the control rows say why. `per_pass`, `instructions:u`, ir arm:

| axis | pin | head, bootstrap suppressed | head as shipped |
|---|---:|---:|---:|
| `strings` | 5,369.53 | 5,406.48 (+0.69%) | **7,136.86 (+32.9%)** |
| `alloc4c` | 3,594.69 | 3,606.50 (+0.33%) | **4,067.06 (+13.1%)** |
| `emptyloop` | 376.00 | 373.00 | 373.00 |

**The per-pass cost arrives with the resident library, not with the bootstrap's fixed 148M and not
with the new code merely existing.** The control build carries the new code and does not run the
bootstrap, and it is flat; the shipped build runs it, and every *allocating* axis moves while
`emptyloop` does not. The column settles the "fixed or per-pass" question by construction: these
figures are already divided by the pass count, so **a fixed per-process cost cannot move them at
all** -- the arms bench derives `per_pass` from the difference between the two sizes precisely so
that the fixed part cancels.

**What a sitting on these axes witnesses, and what it does not.** Seven of the eight are classic
Rexx: `alloc4c`, `arith`, `compound`, `emptyloop`, `strings`, `varlookup` and
`bench-rexxcps/rexxcps.rex` contain no message send and no `::` directive at all -- checked, and
`alloc4c`'s only `~` characters are inside its opening comment. **`dispatchclass` is the exception
and was added for exactly this reason**: it declares a `::CLASS` with a `::METHOD ... CLASS` body
and sends `.Counter~bump` once per pass, so the class-side send path has an axis. `dispatch.rex`,
the same dimension through an instance, is still blocked on `~new`.

So a green sitting says the classic paths and the class-side send did not get slower, and says
nothing about the cost of directives, instance construction, or the rest of what this phase built.
That is worth stating because a phase that added an object model and measured mostly classic axes
could otherwise be read as having measured its own work.

---

## 13. The five gate commands

**All five run at `433ac60fc`, the commit this report names, from `rust/`, in one chained script so
no commit could land between them.** They were run twice: once at `059b38124` and again after the
fix round's only tracked commit, and every status and every figure below is the second run's. The script printed the commit and the working-tree state before
running anything, and each status is `$?` read unpiped straight after its command:

```
commit: 433ac60fc Hand 5b and 5c the two boundary items the gate found and the lists missed
tree:   0 modified paths

G1 cargo fmt --all --check                                            exit 0
G2 cargo clippy --workspace --all-targets -- -D warnings              exit 0
G3 cargo test --release --workspace                                   exit 0
G4 REXX_CORPUS_GATE=1 cargo test --release --workspace                exit 0
G5 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast exit 0

ALL FIVE FINISHED at 17:09
```

`--no-fail-fast` was added to G3, G4 and G5 so a failure anywhere could not hide a failure later --
`cargo test` otherwise stops at the first failing binary, which is how a figure from an earlier run
can read identically to one just observed. `memcap` is present in this session
(`command -v memcap` -> `/home/moritz/.local/bin/memcap`), checked rather than assumed.

**The figures, each beside the command that printed it.**

| figure | value | printed by |
|---|---|---|
| `test result: ok` lines | **102** | `/bin/grep -ac '^test result: ok'` on G3's stdout |
| the same, gated release | **102** | the same grep on G4's stdout |
| the same, gated debug | **102** | the same grep on G5's stdout |
| `FAILED` | **0** in each of G3, G4, G5 | `/bin/grep -ac 'FAILED'` on each stdout |
| `panicked at` | **0** in each of G3, G4, G5 | `/bin/grep -ac 'panicked at'` on each stdout |
| ignored tests | **4** in each | `/bin/grep -ao '[0-9]* ignored'`, summed |
| **the corpus** | **248 of 248 matching** | G4's stderr, under `rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt` / `mode: STRICT (the gate) -- REXX_CORPUS_GATE is set` |
| the corpus again, debug | **248 of 248 matching** | G5's stderr, same report |
| gate table C, gated rows | **0** | G4's and G5's stderr, the `gated by this run:` line of table C's report |
| gate table D, gated rows | **0** | the same line from table D's report |

**Zero gated rows is the correct reading of an unflipped gate, not a green 5a.** With `CLOSED_PHASES`
empty and `REXX_PHASE_GATE` unset, both tables print
`mode: STRICT (the gate) -- REXX_CORPUS_GATE is set, no closing phase named, so no verdict is gated`
and report rather than gate. What they report at this commit is section 3's table: table C `5a: 135
rows, 4 not yet agree`, table D `5a: 36 rows, 1 not yet agree`. Section 1 is what those five become
under the arm.

**`git status` is clean at `433ac60fc`** -- no probe, control or scratch file left behind.

---

## 14. Cold start and D2

`rexx-bench/src/bin/rexx-time.rs --warmup 10 --runs 50`, on `bench-programs/startup.rex`
(`/* Cold-start dimension: interpreter launch, source load, first clause. */` and `say 1`), each
side inside the same `/bin/sh` wrapper `rexx-bench` uses --
`cd "$1" || exit 111; ulimit -v "$2" || exit 112; shift 2; exec "$@"` at
`ADDRESS_SPACE_LIMIT_KIB` = 8388608 -- so both sides carry one `exec` equally.

| side | min | median | mean | max |
|---|---:|---:|---:|---:|
| oracle, `build/bin/rexx` | 5.084 ms | **7.437 ms** | 7.308 ms | 9.413 ms |
| this crate, `target/release/rexx-run` | 26.181 ms | **28.657 ms** | 28.848 ms | 32.550 ms |
| the pin, `rexx-run-15a1ffa98`, no bootstrap | 0.925 ms | **0.970 ms** | 1.036 ms | 1.670 ms |

**Cold-start ratio: 3.85x**, this crate's median over the oracle's. Re-run with the order reversed
-- crate first, oracle second -- it is 28.102 / 7.254 = **3.87x**, so the figure is not an artifact
of which side ran first. `rexx-time` cannot interleave, and running the two orders is the closest
thing to it this instrument allows.

### The comparison the brief asks for, and why the recorded number does not transfer

The named subsection -- **"Fixed per-process offset (`startup.rex`)"** inside **"The first counted
baseline, measured 2026-08-20 at `d90de68e3`"** (`perf-baseline.md:1388`, its section heading at
`:1337`) -- reports the oracle at **14.380 ms** median. **I could not reproduce that figure in
either configuration, and the reason is the harness rather than the machine.** That baseline counts
every run: `rexx-bench-suite` builds its offset measurement with the same `Wrapper` as its axes, and
that wrapper carries `Counted::User`, so the offset is timed *through* `perf stat`. Measured here,
same wrapper, `perf stat -x, -e cycles:u,instructions:u` in the chain:

| side | median, no `perf` | median, through `perf stat` |
|---|---:|---:|
| oracle | 7.437 ms | 17.983 ms |
| this crate | 28.657 ms | 39.662 ms |

`perf stat` costs about 10.5 ms of fixed launch on either side here, and 14.380 ms sits between my
two oracle readings rather than matching either. So **the ratio in this report is taken from my own
two sides measured the same way, not by dividing my crate figure by that file's oracle figure**, and
the recorded 14.380 ms is cited as the thing that did not transfer rather than used as a
denominator. Wrapped in `perf` the ratio would read 2.21x, which is flattery: the fixed overhead
inflates the denominator.

**Two things the comparison must carry, and both hold.**

* **The named subsection reports the pre-5a crate as *not comparable* precisely because it has no
  bootstrap** -- its own words are "Not comparable, and not a pass. This crate has no
  `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does
  at startup." That is measured here rather than repeated: the pin, `rexx-run-15a1ffa98`, starts in
  **0.970 ms** against the oracle's 7.437 ms, and retires 0.588M instructions where the oracle
  retires 9.12M. **Task 23 is what turns this into a real comparison
  rather than a formality**: with the bootstrap landed, the reason that subsection gave for refusing
  the comparison no longer holds, and 3.85x is a ratio between two interpreters doing the same
  startup work.
* **`build/` is a `RelWithDebInfo -O2 -g` oracle, not `-O3`**, which `rust/CLAUDE.md` requires for a
  performance claim. The 3.85x is reported with that named rather than dropped. **Which way an `-O3`
  oracle would move it is not measured**, and I am not guessing: `rust/CLAUDE.md`'s `-O0`-versus-`-O3`
  figures are for `samples/rexxcps.rex`, a throughput axis, and say nothing about process startup.
  So 3.85x is a ratio against *this* oracle build and is not a published figure.

### `instructions:u`, and the attribution the previous task withdrew

`perf stat -x, -e cycles:u,instructions:u` on the same program, three runs each, from a fresh
directory:

| side | `instructions:u` (three runs) | `cycles:u` (three runs) |
|---|---|---|
| oracle | 9,124,478 / 9,132,563 / 9,110,449 | 8,865,701 / 7,335,972 / 6,529,980 |
| this crate | 148,922,066 / 148,912,085 / 149,059,110 | 64,573,388 / 55,679,674 / 65,403,900 |
| the pin, no bootstrap | 587,955 / 587,102 / 587,671 | 661,885 / 647,095 / 692,287 |

**The bootstrap costs 148.3M `instructions:u` per run**, and the subtraction is tight rather than
approximate: the pre-bootstrap crate retires **0.588M for the entire process**, so at most 0.588M of
the 148.3M delta can be anything other than the bootstrap. This crate retires **16.3x** the oracle's
instructions on a cold start where the pre-5a crate retired **0.064x** -- it started fast by not
doing the work.

### **The fixed cost is not collection, and that is now measured rather than withdrawn**

The previous task withdrew "this is the collector" as unproven, and named reading the collection
count as what would settle it. **Read here: the library bootstrap performs zero collections.**

`Interp::bootstrap_library` sets `collections_before_program` from `heap.collections_performed()` at
the moment the bootstrap ends, and `Outcome::collections` is deliberately counted *from* it, with a
comment saying why. **No binary here surfaces the pre-program value**: checked by grepping
`crates/rexx-exec/src/bin/` and `crates/rexx-lib/` for `.collections` (no match) and `rexx-run.rs`
for its environment variables (only `REXX_ENGINE`). A temporary `eprintln!` beside that assignment
answers it, on `startup.rex`:

```
T24-BOOTSTRAP-COLLECTIONS 0     (REXX_ENGINE=ir)
T24-BOOTSTRAP-COLLECTIONS 0     (REXX_ENGINE=tree-walker)
```

**Inverted, so a zero cannot be the probe failing to read anything.** A second temporary marker on
the total, run against `collect_policy.rs`'s own `TRANSIENT_THEN_CHURN` program:

```
T24-BOOTSTRAP-COLLECTIONS 0
T24-TOTAL-COLLECTIONS 6 PROGRAM 6
ok abcdefghij1000000
```

6 is the number that test's doc records for the current policy, so the accessor moves and reads what
it should.

**So none of the 148.3M is collection.** What it *is* has not been profiled by me and this report
does not name it: the previous task's own profile of the install cascade pointed at
`MethodDict::add_method`'s hashing rather than at the parse or the removal walk, and that is its
measurement to stand behind, not mine. What is settled here is the negative, which is the half the
withdrawn attribution turned on.

**What this does not settle, and what is settled elsewhere.** The `strings` axis's per-pass
movement is a *different* cost and these figures cannot speak to it: this is a program with one
clause, and a per-pass effect is not visible in it. **It does not need a new control build, because
one is already committed** -- section 12 reads it out of the same file the guard's baseline lives
in, and it shows the per-pass movement arriving with the resident library rather than with the
bootstrap's fixed 148M. Task 23 handed the *decision* forward -- and its own body records that the
control cannot say **which** consequence of a larger resident heap the cost is. Section 10 carries
both halves.

**And the two findings are complementary rather than in tension.** The bootstrap performing no
collection is a fact about the fixed cost -- the install itself never triggers the collector. What
happens afterwards is a different question: a program that allocates enough to collect at all now
traces a live set the library sits in. Nothing here measures which of a larger live set's costs the
per-pass movement is.

`lib.rs` was restored from a `cp -a` copy, `sha256sum` equal on both sides
(`49c4669116abe052e86e9e0638a66501253af86fc831340a386eb763a920280b`), `touch`ed and rebuilt --
section 11's hazard, avoided deliberately this time -- and the marker is gone from a fresh run.
`git status` is clean.

---

## 15. Findings, and what each check could not see

**F1. Three of this plan's tasks left the gate row their own work was supposed to turn green.** The
concept rows carry, beside each row, the task the plan named as its owner. `usingcl` is Task 7's,
`xscope` is Task 9's, `methna` is Task 21's. All three tasks reported done; all three rows are still
`diverge-both`, loud. Nothing between the task and this gate was in a position to notice, because a
5a row is a *progress report* until the phase closes and a progress report exits zero. **This is the
gate finding**: the tables were built so that a task's own row would redden if the task did not do
its work, and the arm that makes that an exit status was never switched on until now.

*What this could not see:* a task whose row passes for a reason other than the task's work. The row
is a differential against a live oracle, so that would need this crate and the oracle to agree by
coincidence on every one of three descriptors.

**F2. A gate row can name a mechanism a later phase owns, and nothing about the row says so.** The
57 hierarchy edge rows are section 2. The failure mode is not that they were red -- it is that they
were red for a reason no reading of the row would surface, and 57 identical failures read as one big
problem rather than as an instrument defect. What found it was asking, of each row, *whose* method
the probe stops at, and looking that method up in the row set that assigns phases.

*What this could not see:* the same defect in the other direction -- a 5c row whose probe reaches
only 5a machinery, which would pass early and pass for the wrong reason. I did not look for that.

**F3. `owning_phase`'s catch-alls file rows nobody decided, and an audit of them has two
directions.** `::ATTRIBUTE EXTERNAL` is 5a because
`("::ANNOTATE" | "::ATTRIBUTE" | "::CLASS" | "::METHOD", _) => Some("5a")` catches it, and the arm's
own doc lists every exception it makes without mentioning this one. Gate table D's module doc
already names "a row filed under the wrong phase" as something the table cannot see; this is an
instance of it, found only because the phase gate made the row bite.

**I audited only the rows filed *into* 5a, and the review found the other direction.**
`("::ROUTINE", "EXTERNAL") => Some("7")` files a row *out*, and because a row's identity is
(directive, keyword, position) that one row covers the `LIBRARY REXX` spelling too -- the spelling
D37 moved into 5a for `::METHOD`, and the spelling my own argument for keeping `::ATTRIBUTE
EXTERNAL` in 5a turns on. Section 10 carries the measurement. **The asymmetry is what generalises**:
a row swept *in* by a catch-all is reported at every run and eventually bites; a row filed *out* is
invisible to every row of both tables, so nothing will ever surface it. An audit of phase filings
that only checks the inbound direction checks the half that can announce itself.

**F4. `cp -a` defeats cargo's freshness check, so a control build outlives its own revert.** Section
11 carries it. `git status` was clean, `cargo build --release --workspace` said
`Finished ... in 0.03s`, and the binary in `target/release` was still the mutated one. The check
that caught it was running a program that exercises the mutated mechanism, not any state of the
repository.

**F5. The bootstrap's *fixed* cost is not collection, and the counter that says so is not
surfaced.** The dispatch carried "this is the collector" as withdrawn for want of a collection
count. It is zero for the bootstrap, on both engines -- section 14. **This is about the fixed 148.3M
only**, and it does not touch the *per-pass* cost, which the committed control build attributes to
the resident library; the two are different costs and section 14 says so. Getting the number needed a temporary `eprintln!` in `src/`,
because `Outcome::collections` is deliberately counted from the end of the bootstrap and no shipped
binary exposes the value before that point. That is a reasonable design for a per-program instrument
and it makes the per-*bootstrap* question unanswerable from outside the crate; a task that wants to
watch bootstrap cost over time would need a way to read it.

*What this could not see:* what the 148.3M **is**. A zero collection count rules one thing out and
names nothing.

**F6. The staleness test's own wording does not match what the ledger records.** Section 12. The
test says "every commit this lists is one this plan's ledger records"; 36 of the 124 are not in the
ledger by SHA, and all 36 are this plan's own. The test still answers the question it exists for,
but reading it literally fails it here, and would fail it at any boundary where the ledger cites
tasks rather than every commit.

**F7. I wrote a negative claim about a file I was reading in the same section, and never looked.**
Section 12 read `phase-5a-arms.tsv` for the guard's standing and section 14 said the control build
"this task did not run" -- while the last 156 rows of that same file **are** the control build, at a
commit whose `src/` diff against HEAD is empty. One `awk -F'\t' '{print $1}' | tail -1` over the
file already open would have falsified it. The claim was not checked at all, so it was exactly as
wide as the pattern that looked for it, and no pattern did. **The rule this project already has --
record the pattern beside the claim -- would have caught it, because there was no pattern to
record.** Found by the reviewer, not by me, and it cost the report its most consequential handover:
Task 23's Concern 1 came forward as an open *measurement* question when the measurement was already
committed and what was open was a decision.

## 16. Concerns handed back

1. **A decision is outstanding, not a measurement: what a large resident set should cost.** This is
   **Task 23's Concern 1**, and it arrives at this gate unanswered. The measurement is done and
   committed -- the control build
   (`23-attempt-2-control-no-bootstrap`, `cfffc7899`) shows every allocating axis moving **per
   pass** once the library is resident, `strings` from 5,369.53 to 7,136.86 instructions per pass on
   the ir arm, while the same code with the bootstrap suppressed reads 5,406.48. Task 23's Concern 1
   names the collector; its body records that the control cannot separate collection tracing from a
   longer allocation path or a bigger root set, so the mechanism is open too. A phase gate is where
   an open, measured, undecided cost gets handed on, so it goes to 5b and 5c as the standing cost
   either will measure against. **It is not blocking any of the five rows above.**

2. **5a is not closed, and the five rows are not this task's to build.** Each is a mechanism-sized
   piece of work: `Class~subclass` and `Class~mixinClass` as message sends, `Method~scope`,
   `~define` from source text, `.RexxInfo` as an instance entry, and
   `::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX name'`. The last is small and complete; the first two are
   runtime class creation.

3. **The `USE STRICT ARG` condition defect is pre-existing and now reachable at volume.** 93.901 rc
   163 on the oracle against 40.3 rc 216 here. Not 5a's, but 5b will meet it on every library class
   method it constructs against.

4. **`perf-baseline.md`'s recorded oracle cold-start figure does not reproduce, and the reason is
   that it was timed through `perf stat`.** 14.380 ms recorded; 7.437 ms here unwrapped and 17.983 ms
   here wrapped. The number is not wrong -- it is a `perf`-wrapped measurement, and the file does not
   say so at the point a reader takes the number. Anyone comparing a future cold start against it
   has to wrap their own side the same way or they will divide two different quantities. That is the
   same class of problem as the pin that went stale under a control that could not tell.

5. **`obdes` is a silent divergence, and it is 5b's -- so the 5a flip would not catch it.** rc 0 and
   empty `stderr` on both sides; only `stdout` differs. It is a gate-table row rather than a corpus
   program, and a gate-table row is a progress report until *its own* phase closes. It is named here
   so 5b starts knowing it is already red rather than discovering it at its own gate.
