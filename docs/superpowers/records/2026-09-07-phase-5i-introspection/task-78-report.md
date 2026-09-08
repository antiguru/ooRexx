# Phase 5i, Tasks 7 and 8 (merged): `Package`'s thirty-three rows

BASE `59d25e938`. Commit `<SHA>`.

## Outcome

**31 of the 33 `send-differs` rows now read `agree` in
`corpus/introspection-arity.tsv`. Two are declined with their measurements and
stay `send-differs`:**

| row | why it is declined |
| --- | --- |
| `loadLibrary` | `PackageManager::loadLibrary` is a `dlopen`. Measured, oracle rc 0 in one run: `~loadLibrary('rxmath')` and `~loadLibrary('rexxutil')` answer `1`, `~loadLibrary('nosuchlib_zzz')` answers `0`. The answer is a property of this machine's shared libraries; this crate loads no native library at all, so `1` and `0` would both be invented. Refused as `a native library load … (Phase 7)`. |
| `setSecurityManager` | The 2026-09-08 ruling, applied unchanged. The no-argument form is implemented and answers `1`; the argument-taking form is refused as `a security manager … (D12, Phase 7)`, and the arity row's argument list is `.Object~new`, so the row stays `send-differs`. |

`corpus/method-bodies.txt` moved all 33 `Package` rows from `loud` to
`answers`. **That column is not the instrument** — most of those rows are at
rc 168, an arity error, which is what its own header warns about. The arity
table above is the reading that counts.

## What was implemented

`crates/rexx-exec/src/dispatch/package.rs` (new) carries every body.

**Settings and source**: `~source`, `~sourceLine`, `~sourceSize`, `~digits`,
`~form`, `~fuzz`, `~trace`, `~options`, `~prolog`, and `Package~defaultOptions`
and `Package~new` on the class arm.

**Tables and lookup**: `~classes`, `~publicClasses`, `~importedClasses`,
`~routines`, `~publicRoutines`, `~importedRoutines`, `~definedMethods`,
`~resources`, `~namespaces`, `~importedPackages`, `~resource`, the six `find*`
readers, and the four writes `~addRoutine`, `~addPublicRoutine`, `~addPackage`
and `~loadPackage`.

### What state was already there, and what was added

The brief's expectation held for most rows: the state was on `Interp` and what
was missing was the reader. `package_classes`, `package_public_classes`,
`merged_public_classes`, `routines`, `package_public_routines`,
`merged_public_routines`, `package_namespaces`, `package_options`,
`package_tables` and the program source all already existed.

**Three rows carried the task's real cost, and each needed new state:**

* `~importedPackages` — nothing recorded which packages a program had
  imported. New: `Interp::package_imports`, appended by `::REQUIRES` (which is
  `PackageClass::loadRequires`' own trailing `addPackage`) and by
  `~addPackage`, de-duplicated the way `PackageClass::addPackage` returns
  early on a package it already holds.
* `~routines` / `~publicRoutines` / `~importedRoutines` and the two
  `find*Routine` readers — the name tables hold a `(program, directive)` pair,
  not a `Routine` object, and the object's **identity is observable**:
  measured, oracle rc 0, after `p~addRoutine('ADDA', r)` both
  `p~routines['ADDA']` and `p~findRoutine('ADDA')` are the object added. New:
  `Interp::routine_objects`, keyed by the installed routine rather than by a
  package and a name, because `~addPublicRoutine` files an object under any
  name the caller gives and `mergeRequired` carries that name into the
  importing package.
* the REXX package's class table — `Loud::rexx_package_classes` refused it and
  is now deleted. `Interp::rexx_package_class_table` answers the native class
  registry plus what the shipped `.orx` files' own `::CLASS` directives
  installed. Measured against the oracle, both sides: **67 classes and 62
  public ones, the same names**, and the five that part are `BAGMIXIN`,
  `LOCALSERVER`, `MANYITEMMIXIN`, `SETMIXIN` and `SUPPLIERMIXIN`.

`package_imports` and `package_namespaces` were widened from `ProgramId` to
`crate::plan::Package`, because **the interpreter's own package can be
imported** — the arity table caught this: its `addPackage` argument list is
`.Class~package`, and the first implementation refused it with 88.914 where
the oracle answers a `Package`. Measured, oracle rc 0 and now this crate byte
for byte: `p~addPackage(.Class~package)` leaves `~importedPackages` at one
entry whose `~name` is `REXX`, fills `~importedClasses` with 62 entries and
`~importedRoutines` with none.

### What was declined inside an implemented row

Each is a refusal, not a wrong answer, and each is a *write* whose effect this
crate cannot carry:

* **`~options(name, value)` and `~defaultOptions(name, value)`** — the setting
  forms. Measured, oracle rc 0: `p~options('DIGITS', 5)` answers the previous
  `9` and leaves `p~digits` at `5`, so the write reaches every activation of
  that package's code. Refused as `a package settings write (D12, Phase 7)`.
  The refusals the oracle raises *around* the write are reproduced, because
  they fire before it: 93.902 for a second argument to the read-only `I`, `R`
  and `X`; 93.900 for an empty second argument, which stands ahead of the
  per-option switch; 93.914 naming the setting form's own list, which is a
  **different list** from the no-value form's (it has `A[ll]` and no `X`);
  93.903 for a value with the name omitted; and 93.905 for a non-whole
  `defaultOptions('C', …)`.
  **Not reproduced**: the per-option validation *inside* the write —
  `p~options('DIGITS', 'Y')` is 93.905 on the oracle and the Loud here.
* **`~loadPackage(name, source)`** — the source-array form, which builds a
  package out of lines rather than a file. Refused as
  `a loadPackage source array (Phase 7)`.
* **`.Package~new(name, source, context)`** — a parent context the new package
  resolves names against, refused for `native_new_file`'s reason. A third
  argument that is none of `Method`, `Routine` or `Package` is the oracle's own
  93.953 and is reproduced.

## The rulings, and what each produced

| ruling | action |
| --- | --- |
| 2026-09-08 — `Package~setSecurityManager`: no-argument form only, refuse the with-argument form naming D12 and Phase 7; the row stays `send-differs` and is declined with the measurement | **Applied.** The no-argument form answers `1`. **The ruling's reason does not transfer, and the code says so:** `PackageClass::setSecurityManagerRexx` ends `return TheTrueObject` (`classes/PackageClass.cpp:2049`) whatever it was given, so on a `Package` receiver the answer *is* a constant, unlike the `0`/`1`-by-code-kind reader Task 4 measured on `Method` and `Routine`. Measured, oracle rc 0: `p~setSecurityManager` is `1`. The decision is unchanged; only its justification is. The row is declined, not closed. |
| 2026-09-08 — the arity instrument cannot see an object's class; witness each row through a second send reading `~class~id` as well as a value | **Applied throughout.** Every collection row prints each entry's own `~class~id`; `~source` and `~resource` are read by position; `~prolog`'s `Routine` is asked for its `~source~items`; `~findProgram`'s answer is asked whether it is a `String` and an absolute path. It found nothing wrong here — every class matched — but it is what makes the rows evidence rather than agreement about a rendering. |
| 2026-09-08 — a live defect your witnesses can trip over: a send inside a multi-argument builtin's argument list corrupts the argument run; assign to a variable first | **Applied, and it fired.** `package_rexx.rex`'s first draft had `right(rexx~findProgram('package_rexx.rex'), 16)` and reported `40.12 RIGHT argument 2 must be a whole number; found "<the path>"` where the oracle answered. Every such site is now assigned first. |
| 2026-09-08 — sorted comparison for the table rows, and it needs a stdout mode first: one implementation, one control, opt-in per program by path, a named licence entry | **Built.** `StdoutComparison`, `stdout_multiset` and `descriptor_diff_modes` in `tests/support/oracle.rs`; `HASH_ORDERED_STDOUT` and `stdout_mode` in `tests/corpus.rs`; **DEVIATION 8** in `docs/superpowers/plans/phase-4-exclusions.txt`. The control is `the_sorted_stdout_licence_covers_an_ordering_difference_and_nothing_else`, and it holds the list in **both** directions — an entry's raw stdout must differ (so a program that does not need the relaxation cannot sit there) and its sorted stdout must agree (so a content difference is not hidden) — plus Deviation 7's empty-stdout guard. **The list has one entry**, `lang/package_writes.rex`; see below for why the other four are compared byte for byte. |
| 2026-09-08 pre-flight, 1 — the 33 rows are confirmed | Confirmed against the tree before starting. |
| 2026-09-08 pre-flight, 2 — the arity table cannot see order or contents; an `agree` row is not evidence the contents match | **Applied.** No row here is reported closed on its arity verdict alone; every collection row has a corpus witness that reads its entries back by name. |
| 2026-09-08 pre-flight, 3 — `source` and `resource` answer Arrays whose order the oracle specifies and must never opt in; `importedPackages` is unknown, report what you find | **Applied.** Neither opts in, and neither program printing them is on the list. **`importedPackages`' order is specified**: `PackageClass::addPackage` appends to `loadedPackages` and `getImportedPackagesRexx` copies it, so it is insertion order — `::REQUIRES` in source order, then `~addPackage`/`~loadPackage` in call order. It is read by position here and compared byte for byte. |
| 2026-09-08 pre-flight — `publicClasses` is the `agree` row most likely to be silently wrong | **It was wrong, and *why* is sharper than the ruling that predicted it.** The row was green not merely because the instrument cannot see inside a collection, but because **it probes a receiver on which the defect does not arise**: the arity receiver is a *program's* package, where both sides answered a one-entry `StringTable` and this crate's body was correct. The defect lived on the other receiver — on the REXX package the crate refused outright (`Loud::rexx_package_classes`) — and no argument list could have reached it, because the receiver is fixed by `corpus/introspection-receivers.tsv` and not by the argument. **A blind instrument can be sharpened; a receiver that cannot exercise the bug cannot.** `corpus/lang/package_rexx.rex` is the answer: a witness whose receiver is the one the row could not reach, reading all 67 class names back by name and asserting the 62/5 public split. |

The rulings and the brief did not conflict anywhere except on
`setSecurityManager`'s justification, noted in the table above; the ruling's
*decision* was followed.

## The witnesses

Five corpus programs, in `corpus/phase-5c.txt` where Phase 5i's other tasks
put theirs, plus two `.cls` helpers so that no corpus scan reads them as
programs of their own. Each helper declares **two** public classes and **two**
public routines, and the main receiver carries two of every kind, so a body
answering an empty or a one-entry table cannot pass.

| program | what it pins |
| --- | --- |
| `package_settings.rex` | the source and settings readers on **both** receivers, and the four pairs that part between them: `~trace` `N` against the null string, `~prolog` a `Routine` against `.nil`, `~source`/`~sourceSize` this file against `Array(0)`/`0`, and `~options` ending `TRACE NORMAL` against `TRACE ?n/a?`. Plus every `~options` name, `~defaultOptions`' two, `.Package~new`'s array form running its prologue, and ten argument errors. |
| `package_tables.rex` | the ten table readers, each entry read back by name with its own class; that each answer is a fresh copy a write does not reach; and `~resource`'s Array read by position, its upcasing lookup and its miss. |
| `package_find.rex` | the six `find*` readers on a hit and a miss each, including the two a body searching only the package's own tables gets wrong (`~findClass('ARRAY')` and `~findPublicClass('ARRAY')` both answer the `Array` class), `~findPublicRoutine` answering a non-`PUBLIC` routine because the C++ stub calls `findRoutine`, and `~findProgram` answering a path `String` rather than a `Package`. |
| `package_writes.rex` | the four writes, each with its before value as well as its after, the object-name tag that says all three `add*` answer the **receiving** package and `~loadPackage` the loaded one, that the object added is the object the tables answer, and that importing the REXX package fills `~importedClasses` with 62 and `~importedRoutines` with none. |
| `package_rexx.rex` | the interpreter's own package: 67 class names read back by name with the 62/5 public split, seven empty tables, the four readers that part from a program's package, and the seven writes that answer 98.984. |

## A defect found, not owned, and worked around

**`DO OVER` over one of the interpreter's own `Body::Native` tables binds a
dead handle once the index string is longer than seven bytes — and that
seven-byte boundary is where the defect becomes *visible*, not where it
begins: a shorter name is not an allocation at all.** Written up in
`found-defect-do-over-native-table-rooting.md` beside this file, with a
reproduction that uses `.methods` and no `Package` method at all, **run at
BASE `59d25e938` in the gate worktree**: the long-name form panics at
`Interp::not_in_arena` and the same program with three-letter method names
prints every entry at rc 0.

It surfaced because `collect_stress.rs` runs the corpus subset under
collect-on-every-allocation, and three of the five witnesses tripped it. A
durable root fixes it (measured); wrapping the items in one `Array` and
`push_temp`ing that does not (also measured), which suggests the converted-
target branch beside it is equally unprotected and passes only because its
items are reachable from the target the source named. A real fix is
`RootSet::park`/`release` tied to `LoopState::OverItems`' lifetime, in both
engines — loop-control work in `run.rs`, outside this task.

**What was done instead**: `package_tables.rex`, `package_writes.rex` and the
two helpers key every entry under a name of at most seven bytes, so the bound
index sits in the handle; each program's comment says why and points at the
write-up. `package_rexx.rex` cannot choose the REXX package's class names, so
it reads all 67 back **by name** from a list it carries and uses `DO OVER`
only to count — which is the stronger witness of the two, since it says which
names must be there.

**A consequence worth stating**: shortening those names made
`package_tables.rex`'s raw stdout agree with the oracle's, so it came **off**
`HASH_ORDERED_STDOUT` — the licence's own both-directions control is what
caught that, exactly as intended. Only `package_writes.rex` still needs it.

## Two harness gaps this task had to widen

Both are the same shape: no corpus program had ever carried a `::RESOURCE`.

* `tests/coverage.rs`'s `is_admitted_directive_kind` refused
  `DirectiveKind::Resource`. Admitted now — a `::RESOURCE` owns lines rather
  than instructions, so `each_instruction`'s own `_ => None` already walks all
  of it there is to walk. **Its negative control lost its subject**: every
  variant of the enum is admitted now, so
  `an_unadmitted_directive_still_panics` would have iterated over nothing. The
  assertion was split so the control passes a stand-in predicate that refuses
  `Resource`, keeping the panic path witnessed while the production path uses
  the real predicate.
* `rexx-parse`'s `tests/tiling.rs` had no span for a `::RESOURCE`'s body or
  its terminator line, so every byte of one read as "belongs to no node". The
  body lines come from `Resource::lines`; the terminator's position is not
  recorded anywhere, so `resource_span` derives it from the marker the
  directive named.

## Measurements this task made that the brief did not have

* **`Package~defaultOptions` reads a class-level static, not the receiver**:
  `psOverridePackageSettings` and `overrideCount`
  (`classes/PackageClass.cpp:2655`). Measured, oracle rc 0: a program carrying
  `::options digits 13` reads `13` from `~options` and `DIGITS 9` from
  `.Package~defaultOptions('DIGITS')`. **Only the first byte of the name is
  read** — `D` is `DefineDefaultOptions` and `C` is `CountOverrides` — which is
  why `'DIGITS'` answers at all, and why `'FORM'` is 93.914. So the brief's
  "apparently ignoring its argument" is not a mystery.
  **What this reader reads, and where a constant is right**: the override
  settings and the override count. Nothing but `defaultOptions`' own second
  argument moves either, and that argument is refused, so for the whole of a
  run they are the language defaults and zero. The string is built from
  `PackageOptions::default()` rather than written out, so it cannot drift from
  what `~options` renders; the `0` mirrors `overrideCount`'s own initial value.
* **`~options` is a per-option reader over fifteen names**, and three of them
  (`I`, `R`, `X`) are read-only. `X` answers the subkeywords a `::OPTIONS`
  directive actually named, which needed new state:
  `PackageOptions::explicit`, a flag per subkeyword recorded as each option is
  applied. Measured, oracle rc 0: `::options all syntax trace l noprolog form
  engineering` renders
  `FORM ERROR FAILURE LOSTDIGITS NOSTRING NOTREADY NOVALUE NOPROLOG TRACE`,
  a file with no `::OPTIONS` renders the null string, and `PROLOG` and
  `NOPROLOG` are two flags rather than one.
* **`~findPublicRoutine` is not a narrower `~findRoutine`.** The C++ stub calls
  `findRoutine` (`classes/PackageClass.cpp:2032`). Measured, oracle rc 0: it
  answers a `Routine` for a `::ROUTINE` declared without `PUBLIC`.
* **`~loadPackage` numbers its name argument where every other reader names
  it.** Measured at rc 163: `p~loadPackage` is 93.903 `argument 1 is required`
  and `p~loadPackage(.nil)` is 88.909 `Argument 1 must have a string value.`
* **`~loadLibrary` validates its name before it checks the receiver.**
  Measured at rc 168: `.Class~package~loadLibrary` is 88.901, not 98.984.
  `~loadPackage`'s `checkRexxPackage` stands between the name and the source
  array, so `.Class~package~loadPackage('X','Y')` is 98.984.
* **`Package~new`'s array form does not resolve the name.** Measured, oracle
  rc 0: `.Package~new('inmem.rex', <lines>)~name` is `inmem.rex` where the file
  form answers the resolved absolute path. Its prologue runs either way.
* **`~findProgram` resolves under `RESOLVE_DEFAULT`**, not
  `RESOLVE_REQUIRES` (`classes/PackageClass.cpp:955`), whose one difference is
  the `.cls` extension tried ahead of every other. `require::candidates` gained
  a `requires` parameter and a test that pins the two orders apart.

### An oracle hazard found

**`::options trace ?<letter>` blocks the oracle forever.** It enters
interactive debug and waits on a terminal that never answers; a probe using it
has to be killed. Not added to `corpus/oracle-crashes.txt` because it does not
crash — it hangs — but it costs a session the same way.

## The argument sweep

Every one of the 33 methods, on **both** receivers, under twelve argument
lists — none, `()`, `(.nil)`, `(1)`, `('X')`, `('X','Y')`, `('X','Y','Z')`,
`(,'Y')`, `('DIGITS','Y')`, `('R','Y')`, `('X','')` and `('I','Y')` — three
descriptors and exit status compared against the oracle, both engines.

**768 cases: zero wrong answers, 12 loud refusals, all of them the deliberately
declined surfaces above. The two engines agree on all 768.**

This sweep is what found the two argument-order defects the first
implementation shipped (`~loadLibrary`'s receiver check ahead of its name
check, and `~loadPackage`'s named-instead-of-numbered argument), both of which
`method-bodies.txt` also caught, and the `~options('X','')` ordering.

## What would be observably different if a claim here were false

* Every closed row: its arity verdict moves off `agree` on the next refresh,
  and its corpus witness diverges on stdout.
* The identity claims (`~routines` answers the object added, `~findRoutine`
  answers the same one): `package_writes.rex` prints the object's
  `~objectName` through each reader, which the oracle prints too. A body
  building a fresh `Routine` per ask would print the default rendering.
* The 67/62 class table: `package_rexx.rex` names every one of the 67 and
  asserts the 62/5 split. A body answering a table with the wrong members
  diverges on the line naming the missing one.
* **Where nothing here would notice**: the per-option validation inside the
  `~options` write, named above; and any difference in the *order* of
  `package_writes.rex`'s stdout, which is what Deviation 8 licenses. Both are
  stated rather than papered over.

## Gates

Run in the gate worktree `/home/moritz/dev/repos/ooRexx-5i-gates`, detached at
the commit above.

* **G1** `cargo fmt --all --check` — `**G1**`
* **G2** `cargo clippy --workspace --all-targets -- -D warnings` — `**G2**`
* **G3** `cargo test --release --workspace --no-fail-fast` — `**G3**`
* **G4** `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` — `**G4**`
* **G5** `cargo test --release -p rexx-exec --test corpus` under `REXX_CORPUS_GATE=1` — `**G5**`
* **G6** `cargo test --release -p rexx-exec --test collect_stress` — `**G6**`
* **G7** `cargo test --release -p rexx-exec --test method_bodies` — `**G7**`

Before the commit, in the working tree:

* `cargo fmt --all --check` — exit 0, no diff printed.
* `cargo clippy --workspace --all-targets -- -D warnings` — exit 0, no
  `error`/`warning` line.
* `cargo test --release --workspace --no-fail-fast` — exit 0, **2284 passed,
  0 failed** summed over every `test result` line of that run.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` —
  **462 of 462 matching**, printed by that binary's own report.
