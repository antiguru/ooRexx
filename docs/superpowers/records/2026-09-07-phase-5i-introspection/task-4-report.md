# Task 4 — `Method` and `Routine`

Twenty-five rows across two classes. BASE `acf617abd`, confirmed HEAD with a clean tree at the
start of the task.

## Status

**All twenty-five rows are implemented** and agree with the oracle on all three descriptors under
both engines, with two arms declined by ruling and one refused by choice, each with its measurement:
`setSecurityManager`'s argument-taking form, `loadExternalRoutine`, and `newFile`'s second argument.
The report below was written as the work went and its early sections say "twenty-one" where four
rows were still blocked on a file boundary the controller then granted; the last section is where
those four land.

## Rulings applied

Re-read from `.superpowers/sdd/2026-09-07-phase-5i-introspection/task-4-brief.md`'s `## Rulings`
section before writing this line.

* **`Routine~new` is in this task** (2026-09-08). Implemented; `corpus/lang/routine_introspection.rex`
  section D is its witness.
* **`setSecurityManager`: the no-argument form only, and the row is DECLINED** (2026-09-08).
  Implemented that way: the no-argument form answers `0`/`1` by code-object kind, and any supplied
  argument -- `.nil` included, which the oracle treats as a manager and not an absence -- is
  `Loud::security_manager`, `a security manager is not implemented (D12, Phase 7)`.
* **`loadExternalMethod`/`loadExternalRoutine`: the `LIBRARY REXX` arm only** (2026-09-08).
  Implemented, once `error.rs` was granted -- and the refused half turned out to include
  `loadExternalRoutine`'s own `LIBRARY REXX` arm, for the reason `directive_gap` keeps every
  `::ROUTINE EXTERNAL` form.
* **`error.rs` is on the touch list** (2026-09-08). Two constructors added,
  `Raised::executable_file_unreadable` and `Raised::bad_external_specification`, and
  `corpus/refusal-sites.tsv` refreshed with them.
* **The synthetic directive for `newFile`, with an assertion** (2026-09-08). Done, and the assertion
  is `no_written_directive_has_an_empty_clause_span`.
* **Report every row the two behaviour changes move** (2026-09-08). Done; both tables' moved rows
  are listed below and **every one of them is a `Method` or `Routine` row** -- 21 rows over 42
  changed lines in the arity table and 20 over 40 in `method-bodies.txt` on the first refresh, and
  4 over 8 and 4 over 8 on the second, so no row belonging to another task moved.
* **The live builtin-argument-run defect** (2026-09-08). Every message send in both witnesses is
  assigned to a variable before it reaches a multi-argument builtin. Neither witness calls one.
* **The arity instrument cannot see an object's class** (2026-09-08). Every row that answers an
  object is witnessed through a second send reading `~class~id` as well as a value: `~package`
  answers `Package` and then its `~name`, `~source` answers `Array` and then its `~items` and its
  elements, and `Routine~new`'s answer is asked its class, then called twice, then asked its source
  and its package.
* **The shared artifacts** (2026-09-08) and **formatting, the fast check, and the gate worktree**
  (2026-09-08). Followed; see the sections below.
* **Commits** (2026-09-08): split into two or three with one gate run at the tip.

## Baseline

All twenty-five rows are loud today. Measured at BASE, both engines, three descriptors, on the
probes below: `PACKAGE`, `SOURCE`, `CALL` and `SETSECURITYMANAGER` each report
`rexx-exec: method "<NAME>" of class "<Method|Routine>" is not implemented (Phase 5)` at rc 120,
where the oracle runs the same programs at rc 0.

## Pre-flight findings

### PF1 — `setSecurityManager` is a real reader, and it *is* witnessable through behaviour

The brief says the row "cannot be witnessed through behaviour" and that both forms answer `1`.
Measured on the oracle, both halves are wrong.

**The answer is not a constant.** `interpreter/execution/BaseCode.cpp:133` returns
`TheFalseObject`; `interpreter/execution/RexxCode.cpp:245`-`:249` calls
`package->setSecurityManager(manager)` and returns `TheTrueObject`. So the answer is `0` for every
receiver whose code object is not Rexx code and `1` for one that is. Measured, oracle rc 0
(`s4.rex`):

```
.Object~method('objectName')~setSecurityManager          0     (native, CPPCode)
.Object~method('objectName')~setSecurityManager(.Object~new)  0
.K~method('AA')~setSecurityManager    0   ::attribute AA        (AttributeGetterCode)
.K~method('CC')~setSecurityManager    0   ::constant CC 5       (ConstantGetterCode)
.K~method('BB')~setSecurityManager    0   ::method BB abstract  (AbstractCode)
.K~method('MM')~setSecurityManager    1   ::method MM           (RexxCode)
.routines~rr~setSecurityManager       1   ::routine rr          (RexxCode)
```

That is the same discriminator `~source` and the seven `is*` flags read, so the row is a reader
over real state rather than a constant.

**It is observable, and the control is red.** The manager goes on the method's *package*, and
`SecurityManager::checkLocalAccess` sends it `LOCAL` at the next environment-symbol lookup.
Measured, oracle:

```rexx
say 'a' .routines~rr~class~id
m = .K~method('MM')
say 'ssm' m~setSecurityManager(.Object~new)
say 'b' .routines~rr~class~id
```

is rc 159 — `a Routine` / `ssm 1` on stdout, and on stderr
`Error 97.1: Object "an Object" does not understand message "LOCAL".` blaming line 4. The
identical program with the `setSecurityManager` line deleted is rc 0 and prints `b Routine`.

**The no-argument form intercepts nothing.** Measured, after `m~setSecurityManager` with no
argument, `.routines~rr` still answers at rc 0 — `SecurityManager` is constructed over a null
manager and `callSecurityManager` never fires.

So storing the object and answering `1` is wrong on the answer for four receiver kinds, and
diverges at the next `.name` lookup for the with-argument form. **Question for the controller**,
and it is the one the brief asked me to raise.

### PF2 — `loadExternalMethod`/`loadExternalRoutine` are `directive_gap`'s Phase 7 gap under another name

Task 0's note that both "complete on the oracle" is true and hides the answer. Measured, oracle
rc 0:

```
.Method~loadExternalMethod('M9','LIBRARY rexxutil SysCurPos')      The NIL object
.Method~loadExternalMethod('M9','LIBRARY nosuchlib nosuchproc')    The NIL object
.Routine~loadExternalRoutine('R9','LIBRARY rxmath RxCalcPi')       a Routine
```

`/home/moritz/dev/repos/ooRexx/build/lib` holds `librxmath.so` and no `rexxutil`, so **the answer
is a property of the machine's shared libraries** — a `.nil` where the library will not load and a
real object where it will. A malformed spec is raised before any load:
`.Method~loadExternalMethod('M9','garbage')` is
`Error 99.917: Incorrect external name specification "garbage".` at rc 157 under a
`Compiled method "LOADEXTERNALMETHOD" with scope "Method".` frame.

`lib.rs:1589`'s `directive_gap` already declares `::ROUTINE EXTERNAL` and
`::METHOD EXTERNAL naming a library other than REXX` **Phase 7** gaps for exactly this reason.
These two class methods are the same capability reached through a different door.

One arm is answerable here. Measured, oracle rc 0:

```
.Method~loadExternalMethod('M9','LIBRARY REXX file_separator')   a Method
   ~source~items 0   flags 0 0 0 1 0 0 0   ~package~name REXX   ~setSecurityManager 0
.Method~loadExternalMethod('M9','LIBRARY REXX nosuchentry')      The NIL object
.Routine~loadExternalRoutine('R9','LIBRARY REXX Filespec')       a Routine
```

which is `dispatch::native`'s existing entry-point registry.

### PF3 — the asymmetry is on the `Method` side, not the `Routine` side

The brief's "design work" is the `Routine`-to-body link. Confirmed against the tree that the link
is missing — `environment.rs:1906` builds the `.ROUTINES` entry as
`TableValue::Instance("Routine", Annotated::Routine(id, index), None)` — but the fix is one
argument: that variant's third field is `Option<(ProgramId, usize)>` and it is what fills
`table_method_bodies` at `environment.rs:882`-`:885`, and the annotation key beside it already
carries the same `(id, index)` pair. So the `Routine` half is cheaper than the brief expects.

**The expensive half is the one the brief calls already done.** `Interp::method_object`
(`environment.rs:1369`) stores a scope and an annotation site and nothing else; it files no
`table_method_bodies` row, and `run_method_body`'s own doc says so in as many words — "one that
came from `Class~method` names a dictionary entry and carries no body a send could enter". So
`~source`, `~package` and the seven flags need a resolver the tree does not have:

  object → `(class, name)` → `ClassGraph::own_instance_slot` → `MethodSlot::Defined { method }`
  → whichever of `natives`, `method_bodies`, `generated_methods`, `native_externals` holds it.

`method_objects` is keyed `(class, name) -> object`, so the first step is a reverse lookup this
crate does not keep. That is the design work in this task, and it is on the `Method` side.

A fourth producer needs the same row and is easy to miss: `Interp::method_new_scope`
(`environment.rs:1425`) *clones* the `NativeObject` into a fresh `ObjRef` for a method that
already had a scope, so `.K2~define("X", .K~method("M"))` then `.K2~method("X")` is an object with
no entry in any of the tables above.

### PF4 — the four setters are not flag bookkeeping; `setPrivate` changes dispatch

Measured, oracle rc 0 then rc 159 in one program: `o~mm` answers `1`; then
`.K~method('MM')~setPrivate`; then `m~isPrivate` is `1` and the same `o~mm` raises 97. The object
`Class~method` answers *is* the dictionary's method, so the flag is the live one.

This crate keeps access scopes in `special_methods` (`lib.rs:6575`-`:6590`), a `Vec` appended in
mint order with a `debug_assert` that every push is past the last key, and read by
`access_scope_of` in `Interp::resolve` (`dispatch.rs:2124`). A setter therefore needs a mutable
override, not a push.

`setGuarded`/`setUnguarded` and `setProtected` have **no reachable behaviour** here and I will not
claim one: this crate runs one activity, so both arms of the C++'s `isGuarded()` split are the
same read (`read_attribute`'s own doc states this), and `protected` is consulted only by the
security-manager seam, which nothing in this phase installs. For those three the readback is the
whole observable, and the report will say so rather than assert a behaviour witness.

### PF5 — `~source` is a directive-to-directive line range, and the C++ rule is in hand

`RexxCode::getSource` (`interpreter/execution/RexxCode.cpp:220`) is
`package->extractSource(location)` over the block location `LanguageParser::translateBlock`
computed (`interpreter/parser/LanguageParser.cpp:1190`-`:1193` for the start, `:1626`-`:1671` for
the end): **start is the line after the directive clause; end is the line before the next
directive clause, or the file's last line; nothing between the two means an empty Array**
(`blockLocation.setLineNumber(0)` at `:1643`, which `ProgramSource::extractSourceLines:205` turns
into `new_array(0)`).

Measured on the oracle, and this is what tells the rule apart from "the first clause to the last
clause":

```
::method PAD          -> Array(5): "", "  /* leading comment */",
  <blank>                          "  return 1  /* trailing */", "", "  /* after */"
  /* leading comment */
  return 1  /* trailing */
  <blank>
  /* after */
::method TAIL         -> Array(1): "  say 1; say 2   /* tail comment */"
  say 1; say 2   /* tail comment */
::method EMPTY        -> Array(0)   (last directive, nothing after it)
```

and `::method BB abstract`, `::attribute AA` and `::constant CC 5` each answer `Array(0)`.

`rexx_parse::Directive::clause_span` gives both ends, so the range is derivable — but
`CodeBody` carries no span, so it has to come from the directive list, not from the body.

### PF6 — scope

Twenty-five rows, of which six are separate pieces of real work rather than table entries: the
`Method` body resolver (PF3), the `Routine` run path, the four setters (PF4), `Routine~new`,
`newFile`, and `loadExternal*` (PF2). Two further measurements that size them:

* `.Routine~new('NEWR','return 42')~package~name` is **`NEWR`** — the executable's own name, the
  shape `Interp::compiled_method_names` already keeps — not a file path.
* **`newFile` loads a whole package, not a body.** Measured, oracle rc 0: a file whose text is
  `return helper(3)` above a `::routine helper` answers `33` from `~call`, so its directives
  install; `~source` is `Array(1)`, the main section alone. `.Method~newFile('nope.rex')` is
  `Error 3.1: Failure during initialization: File "nope.rex" is unreadable.` at rc 253 under a
  `Compiled method "NEWFILE" with scope "Method".` frame, and `.Routine~newFile` is the same. The
  path is answered back unabsolutised: `~package~name` for `newFile('nf/body.rex')` is
  `nf/body.rex`.

## What each row reads, and where that state comes from

**Every row here reads one thing: which directive declared this executable.** `crate::ExecutableRecord`
is the row `Interp::executable_sources` keeps per `Method` and `Routine` object, and its `source`
field is a `crate::ExecutableSource` -- `Directive { program, directive }` for one a directive
declared, `Main { program }` for a body compiled from source text, and `Native` for a primitive.
Nothing here answers from a constant; the four writers are `Interp::method_object` (which resolves
the dictionary's `MethodId` through `Interp::installed_executable_source`), `environment.rs`'s
`.METHODS`/`.ROUTINES` table build, `Interp::record_compiled_body`/`record_compiled_routine`, and
`Interp::method_new_scope`, which carries the row across the copy `~define` makes.

| row | what it reads |
| --- | --- |
| `isAbstract` `isAttribute` `isConstant` `isGuarded` `isPackage` `isPrivate` `isProtected` | the declaring directive's own keywords, plus the flag word `::CONSTANT` sets outright (`setUnguarded`, `setConstant`); `Native` answers the flag word `new MethodClass` leaves, which is every bit clear |
| `source` | physical lines of the declaring program, from the line after the directive to the line before the next one; `Native` and every directive with no `CodeBody` answer `BaseCode::getSource`'s empty array |
| `package` | `Package::Program` of the declaring program, `Package::Rexx` for `Native` |
| `setGuarded` `setUnguarded` `setPrivate` `setProtected` | write `Interp::method_flag_writes`, read back over the directive's own value; `setPrivate` also writes `Interp::special_methods`, which is what a send reads |
| `setSecurityManager` | whether the declaring directive owns a `CodeBody` and carries no `EXTERNAL` -- the `RexxCode`/`BaseCode` split |
| `call` `callWith` `[]` | run the `::ROUTINE` directive `ExecutableRecord::routine` names, as a `SUBROUTINE` in that routine's own package |
| `new` | compiles the source into a program of its own, whose sole `::ROUTINE` is the body |

**The one place the answer could have been a constant, and is not.** `~package~name` for a compiled
executable is the executable's own name and not a path, so it had to come from
`Interp::program_display_name` rather than `Interp::package_path` -- one line in
`environment.rs`'s `package_name`. Measured, oracle rc 0:
`.Routine~new('NEWR','return 42')~package~name` is `NEWR`; and, independently,
`.K~define('MM', "say .context~package~name; return 1")` prints `MM` from inside the body, so the
same line moves a second reader toward the oracle.

## Three edges of `~source` that a plausible implementation gets wrong

Each cost a measured correction, and each has a row in `corpus/lang/method_introspection.rex`.

1. **The range is directive-to-directive, not first-clause-to-last-clause.** `::method PAD` over a
   blank line, a comment line, `return 8  /* trailing */`, a blank line and a second comment line
   answers `Array(5)` -- all five, in order, the trailing comment included.
2. **An end that lands on an empty line steps back one** (`ProgramSource::extractSourceLines`,
   `parser/ProgramSource.cpp:224`-`:233`). Measured across `n` trailing blank lines for `n` in
   0..3, at the end of the file and before a following directive: a body plus one blank answers the
   body alone, plus two answers the body and one blank; and a `::method` with nothing after it but
   one blank line answers `Array(0)`.
3. **An `::ATTRIBUTE ... GET` with a body starts one line late**, because
   `LanguageParser::hasBody` consumes the first clause and `reclaimClause()`s it without restoring
   the scanner's line position, so `translateBlock` records a start past it. Measured, a
   three-line attribute body answers `Array(2)` beginning at its second line, where the `::method`
   beside it answers both of its own. This is mirrored rather than corrected: the differential is
   the standard.

## Two behaviour changes outside the twenty-five rows

Both move toward the oracle and both are measured.

* `Package~name` for a program compiled from source text, above.
* `Interp::method_new_scope` carries the executable record across the copy it makes, so
  `.K2~define("X", .K~method("M"))` then `.K2~method("X")~source` answers `M`'s lines.
  `table_method_bodies` is deliberately **not** copied with it -- that would change what
  `Object~setMethod` and `Object~run` accept, which is a different row.

## Declined and blocked

**`setSecurityManager`'s argument-taking form is DECLINED**, per the 2026-09-08 ruling, and the
measurement is why: the manager lands on the method's *package* and
`SecurityManager::checkLocalAccess` sends it `LOCAL` at the next environment-symbol lookup.
Measured, oracle: `m = .K~method('MM'); say m~setSecurityManager(.Object~new)` then
`say .routines~rr~class~id` is rc 159 with
`Error 97.1: Object "an Object" does not understand message "LOCAL".` on stderr; the identical
program with that one line deleted is rc 0 and prints `Routine`. **D12 owns the interception
points and Phase 7 the rest.** The no-argument form is provably inert -- measured, `.routines~rr`
still answers after it -- and is implemented, answering `0` or `1` by code-object kind.

**`loadExternalMethod`, `loadExternalRoutine`, `Method~newFile` and `Routine~newFile` were blocked
here on a file boundary** -- all four need a `Raised` constructor and `Raised`'s constructors live in
`crates/rexx-exec/src/error.rs`, which the dispatch did not list. **The controller granted it**, and
the four are landed; see the last section.


## Witnesses

Two corpus programs, both filed in `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C` beside Task 3's,
both with a `crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, and
`crates/rexx-exec/src/dispatch/executable.rs` added to `dispatch_seam.rs`'s
`CLEARANCE_CONSUMERS`.

**`corpus/lang/method_introspection.rex`.** Nine sections. Its receiver set is one directive of
every kind that makes a method -- a primitive, a plain `::METHOD`, `UNGUARDED PRIVATE`, `PACKAGE`,
`PROTECTED`, `GUARDED`, `ABSTRACT`, `::ATTRIBUTE`, its generated setter, `::CONSTANT`,
`::METHOD ... ATTRIBUTE`, `::ATTRIBUTE ... GET` with a body, and `::METHOD ... EXTERNAL
'LIBRARY REXX ...'` -- because the receiver a first draft reaches for answers an empty `~source`
and six zeroes with `isGuarded` `1`, and a body answering exactly that agrees with the oracle on
nine rows. Section E is the `setPrivate` dispatch witness: `o~mm`, then the setter, then the same
send trapped at 97, with an explicit `THE SEND WAS NOT BLOCKED` / `exit 1` on the path where it is
not. Section I sends a one-argument list to three zero-argument readers.

**`corpus/lang/routine_introspection.rex`.** Seven sections. `~call`, `~callWith` and `~'[]'` are
each sent twice to one object, and `.Routine~new`'s answer is asked its class, called twice, and
then asked its `~source`, `~package` and `~annotations`. Section B is what says the call is a
`SUBROUTINE` in the routine's own package: a `::ROUTINE` that reports `PARSE SOURCE`'s first two
words and `arg()`, and a second that calls a third routine of this file. Section G sends `callWith`
with no argument, with an empty list, and `call` with a long list and a short one.

**No `::ROUTINE EXTERNAL` in either.** It is a Phase 7 gap that refuses at *install* time, so one
such directive makes every row above it unreachable here; the external arm of `~source` and
`~setSecurityManager` is witnessed on the `Method` side instead, where
`::METHOD ... EXTERNAL 'LIBRARY REXX ...'` installs.


## Shared artifacts — before and after

Each refreshed with the command quoted, from `rust/`.

**`corpus/introspection-arity.tsv`** --
`REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity`,
exit 0, `25 passed; 0 failed`. Twenty-one rows moved, all of them `Method` or `Routine`:

| rows | before | after |
| --- | --- | --- |
| `Method` `isAbstract` `isAttribute` `isConstant` `isGuarded` `isPackage` `isPrivate` `isProtected` `package` `source` | `send-differs` | `agree` |
| `Method` `setGuarded` `setPrivate` `setProtected` `setUnguarded` | `send-differs` | `no-value` |
| `Routine` `call` `callWith` `[]` `package` `source` `new` | `send-differs` | `agree` |
| `Method` `setSecurityManager`, `Routine` `setSecurityManager` | `send-differs` | `send-differs` |

The last row is the declined one and its *evidence* moved rather than its verdict: from
`crate rc120 rexx-exec: method "SETSECURITYMANAGER" of class "Method" is not implemented (Phase 5)`
to `crate rc120 rexx-exec: a security manager is not implemented (D12, Phase 7)`.

**`corpus/method-bodies.txt`** --
`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`, exit 0,
`21 passed; 0 failed`. Twenty rows moved `loud` -> `answers`: `Method`'s seven flags, `package`,
`source`, the four setters and `setSecurityManager`; `Routine`'s `call`, `callWith`, `[]`,
`package`, `source` and `setSecurityManager`. `Routine~new` did **not** move -- it was already
`answers` there before this task, because that table sends no arguments and a zero-argument send
agreed about an arity error, which is the blindness the 2026-09-08 ruling ruled the row in over.
`setSecurityManager` moved to `answers` for the same reason and it is not evidence the row is done:
the form that table sends is the no-argument one, which is the form this task implements.

**`corpus/refusal-sites.tsv`** -- re-derived by `cargo test --release -p rexx-exec --test
refusal_sites`, which failed first (`4 passed; 1 failed`,
`the_table_holds_every_constructor_the_source_defines`) and passes after (`5 passed; 0 failed`,
exit 0). Forty-seven rows moved by line number alone -- every `Loud` constructor defined below
`Loud::security_manager` in `lib.rs` -- and one row is new:
`Loud security_manager send crates/rexx-exec/src/lib.rs:977 recorded yes a security manager`, whose
witness column is `say .K~method('MM')~setSecurityManager(.Object~new)`. `recorded` is the verdict
the table defines as "a divergence the phase plan records as not to be built", which is exactly what
the 2026-09-08 ruling made it.

**`corpus/phase-5c.txt`** and **`EXPECTED_SUBSET_5C`** -- the two witness paths appended after Task
3's, under a comment naming what they cover. Before, `cargo test --release --workspace` failed
`every_lang_program_is_run_or_named_unfiled` naming exactly those two files; after, it does not.

**`corpus/unfiled.txt`** -- unchanged. Neither witness is a debt: both are filed in the subset and
both are compared against the oracle.


## Commits

Two, per the 2026-09-08 ruling, and the split is the one that was available rather than the one that
would read best. **A deeper split is red by construction here**: `corpus/method-bodies.txt`, the
arity table and `corpus/refusal-sites.tsv` are each asserted against the tree on every test run, so
a commit landing half the code with the whole refresh is red and one landing half the code with half
a refresh needs its own refresh cycle -- two more table runs, each ~10 minutes, and each a place to
get a hand-written column wrong. The code, the witnesses and the artifacts they move go together;
the report follows.

## Gates

Fast checks, in the working tree, before the commit:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0,
  `Finished \`dev\` profile`. (It was exit 101 once, `clippy::redundant_guards` on this file's
  own `Some(text) if text.is_empty()`; fixed to `Some([])` and re-run.)
* `cargo test --release --workspace --no-fail-fast` -- exit 0, 116 `test result:` lines, none
  `FAILED`. Run again before each of the three commits, exit 0 each time; the third adds
  `no_written_directive_has_an_empty_clause_span`, confirmed to exist by
  `cargo test --release -p rexx-exec --lib no_written_directive` reporting `1 passed` rather than
  `0 passed; 787 filtered out`. Two earlier runs of the same command were red and each named its own
  fix:
  `every_lang_program_is_run_or_named_unfiled` before the two witnesses were filed in
  `corpus/phase-5c.txt`, and `the_table_matches_the_three_sides` before the arity table was
  refreshed.

| gate | command | result |
| --- | --- | --- |
| G1 | | **G1** |
| G2 | | **G2** |
| G3 | | **G3** |
| G4 | | **G4** |
| G5 | | **G5** |
| G6 | | **G6** |
| G7 | | **G7** |

## Negative controls

**Predictions written before the run**, one line each, and marked below with what the run said.
The script is `controls.sh` in this task's scratch: it applies one mutation, rebuilds
`--release --bin rexx-run`, runs both witnesses on both engines against the saved oracle
transcripts, and restores from a `cp` backup -- never `git checkout --`.

| control | mutation | predicted |
| --- | --- | --- |
| BASE | none | both witnesses green on both engines |
| C1a | a `NATIVE_METHODS` row naming a name `Method` does not answer | `ObjectModel::build` panics, rc 101 |
| C1b | the `ISGUARDED` row removed | `~isGuarded` is loud again, rc 120 |
| C2 | `declared_flag` answers the primitive's flag word for every directive | method witness RED on the nine section-A rows that are not `0 0 0 1 0 0 0`; routine witness green |
| C3 | the empty-last-line step back deleted | method witness RED on exactly `step` (2 -> 3) and `tail` (1 -> 2); routine witness green |
| C4 | an `::ATTRIBUTE` body starts where a `::METHOD` body does | method witness RED on `attribute-with-body` (1 -> 2); routine witness green |
| C5 | `setPrivate` writes the readback and not the dictionary | method witness RED at `send-after`, which answers instead of raising; routine witness green |
| C6 | `Routine~call` is a `FUNCTION` call | routine witness RED on `describe`; method witness green |
| C7 | `Package~name` is the path again for a compiled executable | routine witness RED on `new-package-name`; method witness green |
| C8 | `setSecurityManager` answers `1` for every receiver | method witness RED on section H's five zero rows; routine witness green |
| C9 | `method_new_scope` does not carry the record across its copy | method witness RED at section J's `copy-flags`, loud; routine witness green |

**What the run said**, from `controls.out` and `controls2.out`:

* **BASE and RESTORED** -- both witnesses green on both engines, before and after the whole pass.
  The restore is a `cp` from a backup taken before the first mutation, and the binary is rebuilt
  after it.
* **C2 CONFIRMED.** Method witness red on both engines at
  `unguarded-private`, `package`, `protected`, `abstract`, `attribute-get`, `attribute-set`,
  `constant`, `method-attribute` and `attribute-with-body`, each collapsing to
  `0 0 0 1 0 0 0`. Routine witness green.
* **C3 FALSIFIED, and the control is wider than the prediction rather than narrower.** It reddens
  `written` (1 -> 2), `pad` (5 -> 6) and `tail` as predicted, and also `step`, **and the routine
  witness** (`source-items` 3 -> 4). The prediction was wrong about my own witness files: they
  separate directives with a blank line, so nearly every block's last candidate line is empty and
  the step-back is exercised at almost every row rather than at the two written for it. The two
  written for it are still the only rows that would survive removing the blank separators.
* **C4 CONFIRMED exactly.** Method witness red on `attribute-with-body` alone, `1 -> 2`, the added
  line being `  a = 7`. Routine witness green.
* **C5 CONFIRMED.** Method witness red from `send-after` onward at rc 1 -- the send answers where
  the oracle raises, so the witness takes its own `THE SEND WAS NOT BLOCKED` / `exit 1` path and
  every section after E is lost. Routine witness green.
* **C6 CONFIRMED exactly.** Routine witness red on `describe` and `describe-again`,
  `LINUX SUBROUTINE 0` -> `LINUX FUNCTION 0`. Method witness green.
* **C7 CONFIRMED exactly.** Routine witness red on `new-package-name` (`NEWR` -> this file's
  absolute path) and on the projection row `new-package-name-is-the-source` (`0` -> `1`). Method
  witness green -- every package it reads is the running file's or `REXX`.
* **C8 CONFIRMED exactly.** Method witness red on section H's `native`, `abstract`,
  `attribute-get`, `constant` and `external`, each `0 -> 1`. Routine witness green, because both of
  its receivers are Rexx code and already answer `1`.
* **C1a CONFIRMED exactly.** With `("Method", "NOSUCHNAME", Arity::Fixed(0), is_abstract)` planted,
  every program is rc 101 and
  `panicked at crates/rexx-exec/src/dispatch.rs:1269:21: NATIVE_METHODS names Method~NOSUCHNAME,
  which that class's behaviour does not answer`.
* **C1b CONFIRMED exactly.** With the `ISGUARDED` row removed, the method witness stops after its
  first heading at rc 120 with
  `rexx-exec: method "ISGUARDED" of class "Method" is not implemented (Phase 5)` -- the loud
  refusal these rows close, back again.
* **C9 CONFIRMED exactly.** Method witness red at rc 120, last stdout line `copy-scope K2` and
  stderr `rexx-exec: a message send to an executable this crate did not build is not implemented
  (Phase 5)` -- so section J does rest on the record crossing `method_new_scope`'s copy and not on
  something else. Routine witness green.

## Concerns and hand-offs

1. **CLOSED.** The four rows are landed; `error.rs` was granted. What is left of them is three
   deliberate refusals, each named in the last section with its measurement:
   `loadExternalRoutine` (the routine entry-point table this crate does not keep),
   `loadExternalMethod` naming any other library (a `dlopen`), and `newFile`'s second argument (the
   package context the loaded file would resolve names against).
2. **`setSecurityManager`'s argument form stays a divergence, by ruling.** The oracle installs a
   manager and raises `97.1` at the next environment-symbol lookup; this crate refuses loudly. The
   row is `send-differs` in `corpus/introspection-arity.tsv` and will stay so until D12's
   interception points exist. The same decision is owed to `Package~setSecurityManager`, which is
   Task 7's.
3. **An upstream oddity this task mirrors rather than corrects.** An `::ATTRIBUTE ... GET`/`SET`
   with a body reports its `~source` starting one line late, because `LanguageParser::hasBody`
   (`parser/DirectiveParser.cpp:189`) consumes the first clause and `reclaimClause()`s it without
   restoring the scanner's line position. It is unfiled; three signals are not in hand, and one
   measurement is one.
4. **CLOSED, and it was a defect rather than a gap.** Both `~source` edges can land mid-line and
   the first commit answered whole lines only. Measured, fixed, witnessed and controlled -- see
   *A divergence found after the first commit* at the end of this report. It is the reason there
   is a third commit and a second gate run.

## A divergence found after the first commit, and fixed

**Concern 4 and concern 5 above were not gaps; they were a defect, and probing them found it.**
Both edges of `~source`'s range can land mid-line, and the first commit answered whole lines only.
Measured on the oracle, four shapes, all of them rc 0 and all of them wrong here at `546bd9ed6`:

```
::attribute AB get / "  a = 1;b = 2"        Array(2): "b = 2", "  return a + b"     -- was Array(1)
::attribute AB get / "  a = 1;   b = 2"     Array(2): "   b = 2", ...               -- was Array(1)
::attribute AB get / "  a = 1;"             Array(2): "", "  return a"              -- was Array(1)
::method MSEMI; return 7                    Array(1): " return 7"                   -- was Array(0)
::method A / "  return 1; ::method B"       Array(1): "  return 12; "               -- was Array(0)
```

**The rule, and it is one rule at each edge.** A clause the parser ended with `;` leaves the scanner
on its own line at the byte after it, where a clause ended by the line itself leaves it at the start
of the next -- `blockLocation.setStart(lineNumber, lineOffset)`
(`parser/LanguageParser.cpp:1193`) records whichever. And a terminating directive clause that does
not begin its line cuts that line short at the `::` --  `translateBlock`'s `else` arm at `:1657`
against the `getOffset() == 0` arm above it. `block_first_line` and `block_last_line` are the two,
and `block_source_lines` now works in absolute byte offsets and clamps each line to them, which is
`extractSourceLines`' own shape.

A continued first clause was measured too and agrees either way: `a = 1 +,` over `      2` answers
`Array(1)`.

**Four rows added to `corpus/lang/method_introspection.rex`**, and control **C10 CONFIRMED exactly**:
with both rules reverted to whole lines, the method witness is red on
`semicolon-in-an-attribute-body` (2 -> 1), `semicolon-on-the-directive-line` (1 -> 0) and
`a-directive-cutting-a-line-short` (1 -> 0), `after-that-cut` is unchanged, and the routine witness
is green. **The first attempt at C10 did not run at all** -- its second `str.replace` asserted
against text `rustfmt` had since rewrapped, so nothing was written and the unmutated binary reported
STILL GREEN four times. The assertion is what caught it; a script that had written the first
replacement and skipped the second would have reported a half-mutation as a whole one.

No shared table moves for this fix: `cargo test --release -p rexx-exec --test introspection_arity
--test refusal_sites --test coverage` is exit 0 (25, 5 and 20 passed) and
`cargo test --release -p rexx-parse --test sourceline_oracle` is exit 0 against the regenerated
expectation.

## The hunt that found it, and what else it cleared

The defect above was not found by review. It was found by naming the class this task's own defects
would share -- **range arithmetic over a program's source, and the calling convention a `~call`
enters under** -- and running shapes of it against the oracle rather than reasoning about them.
Each of these was measured on the oracle and on both engines after the fix; every one agrees on all
three descriptors.

*Source-range shapes.* A directive clause **continued** onto a second line (`::method CONT ,` over
`  unguarded`) -- the block starts on the third line, `Array(1)`. A body's last line ending in `;`
-- the whole line is kept, `<  return 2;>`, because that edge is the next directive's and not the
`;`'s. A `::CONSTANT` whose parenthesised expression spans two lines -- `Array(0)`, a
`ConstantGetterCode`. A `.Method~new` over a two-element `Array` -- `Array(2)`, the elements
verbatim. The **first** directive after its `::CLASS`. And the two the previous section fixed.

*Package shapes.* A method declared in a `::REQUIRES`-loaded file, read from the requiring program:
its `~package~name` is **not** the requiring file (`0` against `PARSE SOURCE`'s third word), and its
`~source` and flags are the required file's own. A `~define` with a source string: `~package~name`
is `DS`, the name `~define` was given, and `~setSecurityManager` is `1`. A `~defineMethods(.methods)`
entry, which goes through `Interp::method_new_scope` **twice**: `~source` is the unattached
`::METHOD`'s own line and `~scope` is the receiving class.

*Call-convention shapes.* `exit 55` inside a routine reached by `~call` answers `55` to the caller
and lets the program run on, rather than exiting it. A failure inside one reports the routine's own
clause, then `Compiled method "CALL" with scope "Routine".`, then the sending clause, at rc 214 --
stderr byte-identical. And a routine that calls itself through `.routines~R~call` five deep answers
`15`.

## The last four rows, after the controller granted `error.rs`

`Method~newFile`, `Routine~newFile`, `loadExternalMethod` and `loadExternalRoutine` are landed. The
two `Raised` constructors the 2026-09-08 ruling granted are `executable_file_unreadable` (3.1) and
`bad_external_specification` (99.917); both messages were already in `rexx-inventory`'s generated
catalogue, so neither is hand-transcribed.

**`newFile` loads a package and files its main section as a synthetic directive**, the shape the
2026-09-08 ruling approved. Three walks skip a directive whose clause span is empty --
`Interp::install_directives`, `class_members` (without which the main section would attach to the
file's **last class**) and `environment.rs`'s `package_table_entries` (without which it would appear
in that package's `.METHODS`). **The assertion the ruling asked for is
`no_written_directive_has_an_empty_clause_span`**, which parses every `.rex` under `corpus/` and
fails if any written directive carries one; it also asserts the walk found more than a hundred
directives, so an empty walk cannot pass it.

**Measured, oracle rc 0, and matching on both engines**: `newFile('body.rex')` answers a `Method`
with `~scope` `.nil`, `~package~name` `body.rex` (the argument as written, unabsolutised),
`~source` the file's two lines, default flags and `~setSecurityManager` `1`; the same on `Routine`,
whose `~call` answers `5` and answers it again from a file that is `return 5` above a
`::routine helper`. `newFile('nope.rex')` is `Error 3.1: Failure during initialization: File
"nope.rex" is unreadable.` at rc 253, **byte-identical on all three descriptors**.

**`loadExternalMethod`'s `LIBRARY REXX` arm is answered and everything else is refused**, per the
ruling. The refused half grew by one arm the pre-flight had not separated: **`loadExternalRoutine`
cannot answer even `LIBRARY REXX`**, because a routine entry point resolves against
`rexx_routines[]` and this crate keeps only the method registry -- measured, oracle rc 0,
`loadExternalRoutine('Filespec', 'LIBRARY REXX')` answers a `Routine` where the same entry point is
absent from `dispatch::native`'s table. That is the same boundary `directive_gap` draws when it
keeps **every** `::ROUTINE EXTERNAL` form, the `LIBRARY REXX` spelling included, on Phase 7's list.

Answered and matching: a known entry by name or as the executable's own name (case included --
`loadExternalMethod('file_separator', 'LIBRARY REXX')` answers where the same call under `M9` does
not), `.nil` for an unknown REXX entry, and 99.917 for a descriptor that is not one -- `garbage`,
`LIBRARY` alone, `REGISTERED junk`, a fourth word, and the empty string, each byte-identical.

**The second `newFile` argument is refused rather than accepted**, and that is a deliberate
divergence: it is the package context the loaded file resolves names against, so accepting it
without honouring it would answer at rc 0 where the resolution differs.

**One property is measured but NOT witnessed in the corpus, and it is stated rather than papered
over**: that `newFile` installs the loaded file's directives. A corpus program has no second file to
load, and loading itself and calling what that answers would re-run the program from inside itself
-- so both witnesses call `newFile` on their own file and never call the result. The evidence for
the property is the probe above, `return helper(3)` over a `::routine helper` answering `33`.

### Controls for these rows

| control | mutation | predicted | outcome |
| --- | --- | --- | --- |
| C12 | every descriptor is a valid external specification | method witness RED on the descriptor rows | **FALSIFIED first, then confirmed** -- see below |
| C13 | `loadExternalMethod` files no executable record | method witness RED at `external-source`, loud | CONFIRMED, rc 120, `a message send to an executable this crate did not build` |
| C14 | an unknown REXX entry point still answers an object | method witness RED at `external-unknown-entry` | CONFIRMED exactly, `The NIL object` -> `a Method` |
| C15 | an unreadable file is a loud refusal rather than 3.1 | both witnesses RED at their `newFile-a-file-that-is-not-there` row | CONFIRMED, rc 120 on both |
| C16 | `newFile` files a native record rather than its own program | both witnesses RED at `file-has-source` | CONFIRMED, and it also flipped `file-package-name-is-the-source` and `file-ssm` |

**C12 is the one worth reading.** It came back STILL GREEN on both witnesses and both engines, and
the mutation was real. The three descriptor rows the witness had -- `garbage`, `LIBRARY`, and the
empty string -- fail on the descriptor's **arity** and never reach the `LIBRARY` keyword test, so
removing that test changed nothing they could see. This is "can fail is not adds coverage" from the
other side: three red-able rows, none of them covering the branch they appeared to. Two rows were
added -- `REGISTERED junk` (two words, wrong keyword) and a four-word descriptor -- and C12 is red
on both engines with them, at the first of them.
