# Phase 5i, Tasks 7 and 8 (merged): `Package`'s thirty-three rows

BASE `59d25e938`. Code commit `2ebeba1c8`, records `3c53fdbc6` and `bb5d69274`,
fix round 1 `1a9c2efce`, fix round 2 the commit this line is in.

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
| 2026-09-08 — sorted comparison for the table rows, and it needs a stdout mode first: one implementation, one control, opt-in per program by path, a named licence entry | **Built.** `StdoutComparison`, `stdout_multiset` and `descriptor_diff_modes` in `tests/support/oracle.rs`; `HASH_ORDERED_STDOUT` and `stdout_mode` in `tests/corpus.rs`; **DEVIATION 8** in `docs/superpowers/plans/phase-4-exclusions.txt`. The control is `the_sorted_stdout_licence_covers_an_ordering_difference_and_nothing_else`, and it holds the list in **both** directions — an entry's raw stdout must differ (so a program that does not need the relaxation cannot sit there) and its sorted stdout must agree (so a content difference is not hidden) — plus Deviation 7's empty-stdout guard. The list holds `lang/package_writes.rex` and nothing else; every other `Package` witness -- `package_settings`, `package_options`, `package_tables`, `package_find` and `package_rexx` -- is compared byte for byte, for the reasons below. |
| 2026-09-08 pre-flight, 1 — the 33 rows are confirmed | Confirmed against the tree before starting. |
| 2026-09-08 pre-flight, 2 — the arity table cannot see order or contents; an `agree` row is not evidence the contents match | **Applied.** No row here is reported closed on its arity verdict alone; every collection row has a corpus witness that reads its entries back by name. |
| 2026-09-08 pre-flight, 3 — `source` and `resource` answer Arrays whose order the oracle specifies and must never opt in; `importedPackages` is unknown, report what you find | **Applied.** Neither opts in, and neither program printing them is on the list. **`importedPackages`' order is specified**: `PackageClass::addPackage` appends to `loadedPackages` and `getImportedPackagesRexx` copies it, so it is insertion order — `::REQUIRES` in source order, then `~addPackage`/`~loadPackage` in call order. It is read by position here and compared byte for byte. |
| 2026-09-08 pre-flight — `publicClasses` is the `agree` row most likely to be silently wrong | **It was wrong, and *why* is sharper than the ruling that predicted it.** The row was green not merely because the instrument cannot see inside a collection, but because **it probes a receiver on which the defect does not arise**: the arity receiver is a *program's* package, where both sides answered a one-entry `StringTable` and this crate's body was correct. The defect lived on the other receiver — on the REXX package the crate refused outright (`Loud::rexx_package_classes`) — and no argument list could have reached it, because the receiver is fixed by `corpus/introspection-receivers.tsv` and not by the argument. **A blind instrument can be sharpened; a receiver that cannot exercise the bug cannot.** `corpus/lang/package_rexx.rex` is the answer: a witness whose receiver is the one the row could not reach, reading all 67 class names back by name and asserting the 62/5 public split. |

The rulings and the brief did not conflict anywhere except on
`setSecurityManager`'s justification, noted in the table above; the ruling's
*decision* was followed.

## A finding about what an `agree` row is worth, and it is not only this row's

**`Package~publicClasses` read `agree` because the arity probe's receiver is a
program's package — a receiver on which the defect cannot arise.**

That is stronger than "the instrument cannot see inside a collection", which is
what this phase had recorded. Both are true here, but only the second one is
fixable by sharpening the instrument. The body was **correct** for the receiver
the row probes: on a program's package this crate answered the same one-entry
`StringTable` the oracle did, contents and all. The defect was on the other
receiver — on the REXX package the crate refused outright
(`Loud::rexx_package_classes`), 67 entries where it had none — and **no
argument list could have reached it**, because the receiver is fixed by
`corpus/introspection-receivers.tsv` and the argument file only varies what is
sent.

**A blind instrument can be sharpened. A receiver that cannot exercise the bug
cannot.** So the reading generalises past `Package`: an `agree` row in
`corpus/introspection-arity.tsv` says the crate matches the oracle *for one
receiver*, chosen once per class, and says nothing at all about any other
receiver that class can have. Where a class has two receivers that behave
differently — and `Package` has exactly that, four readers parting between them
— half its rows are unexamined however many arguments are tried.

The answer here was a corpus witness whose receiver is the one the row could
not reach: `corpus/lang/package_rexx.rex` reads all 67 class names back by name
and asserts the 62/5 public split, and `package_settings.rex` asserts all four
of the readers that part.

## `corpus/refusal-sites.tsv`: the before and after

The Global Constraints require each moved row's verdict before and after with
the refresh command quoted. The refresh is
`cargo test --release -p rexx-exec --test refusal_sites`, whose
`the_table_holds_every_constructor_the_source_defines` re-derives columns 1–4
from `src/` and fails against a stale file; the file is then edited to match
and the test re-run. It was run twice, because three constructors were added
after the first refresh.

`git diff 59d25e938 -- corpus/refusal-sites.tsv` moves 142 lines. **Seven rows
are new, one is deleted, and 64 rows keep their verdict and move only their
`definition` line**, which is the shift every earlier row takes when a
constructor is inserted above it in `error.rs`.

**Added** — verdict/reached/answer as committed:

| kind | name | surface | verdict | reached | answer |
| --- | --- | --- | --- | --- | --- |
| `Loud` | `package_from_source` | send | recorded | yes | a loadPackage source array |
| `Loud` | `package_option_write` | send | recorded | yes | a package settings write |
| `Raised` | `argument_not_an_instance` | send | agrees | yes | 88.914 |
| `Raised` | `argument_not_convertible` | send | agrees | yes | 93.953 |
| `Raised` | `method_argument_not_in_list` | send | agrees | yes | 93.914 |
| `Raised` | `method_argument_not_whole` | send | agrees | yes | 93.905 |
| `Raised` | `method_user_defined` | send | agrees | yes | argument 2 must not be empty |

**Deleted**: `Loud rexx_package_classes`, which at BASE read
`send / diverges / yes / the REXX package's class table`. It is gone because
the REXX package now has one — that row **was** the `publicClasses` refusal,
and deleting it is the task's headline expressed in the table. Its witness in
`run/tests.rs`'s `the_refusals_this_task_leaves_where_the_oracle_answers_still_fire`
went with it, since the send it asserted now answers.

**The header's own claim was rewritten and the rewrite was measured.** The file
records which rows its checks cannot tell apart, and `argument_not_an_instance`
joins `argument_not_a_class` and `scope_override_not_a_class` at 88.914, making
a group of three where the header said a pair. `SHARED_ANSWERS` in
`refusal_sites.rs` was widened to match, and
`the_answers_more_than_one_row_shares_are_the_recorded_ones` is what holds the
two together. **The header's claim that a transposition inside such a group
leaves every test green was re-measured for the new member rather than assumed**:
swapping `argument_not_a_class`'s `answer` and `witness` with
`argument_not_an_instance`'s left every test in `refusal_sites.rs` passing, and
the file was restored from a copy afterwards.

## The witnesses

Six corpus programs, in `corpus/phase-5c.txt` where Phase 5i's other tasks
put theirs, plus two `.cls` helpers so that no corpus scan reads them as
programs of their own. Each helper declares **two** public classes and **two**
public routines, and the main receiver carries two of every kind, so a body
answering an empty or a one-entry table cannot pass.

| program | what it pins |
| --- | --- |
| `package_options.rex` | the same settings readers on a package that **declares** seven of them — `::options digits 13 form engineering fuzz 2 numeric inherit novalue syntax error syntax trace l` — each read back through the method that reports it and printed beside the REXX package's, plus `~options('X')`, which answers the subkeywords the directive **named** rather than the values in force. It is the pair `package_settings.rex` needs: every answer there is a language default, so a reader that ignored its receiver passed it. |
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
collect-on-every-allocation, and the witnesses that enumerate a package table
-- `package_tables.rex`, `package_writes.rex` and `package_rexx.rex` -- tripped
it; the ones that do not, `package_settings.rex`, `package_options.rex` and
`package_find.rex`, cannot reach it. Re-measured over long-named copies at
`1a9c2efce`: all three panic, and all three answer rc 0 with the durable root
added, which is the documented revert exercised rather than assumed. A
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

## The argument sweep, and the count I got wrong

Every `Package` row, on **both** receivers where the row has an instance arm,
under twelve argument lists — none, `()`, `(.nil)`, `(1)`, `('X')`,
`('X','Y')`, `('X','Y','Z')`, `(,'Y')`, `('DIGITS','Y')`, `('R','Y')`,
`('X','')` and `('I','Y')` — three descriptors and exit status compared against
the oracle, both engines.

**My report said 12 refusals over 768 cases. The reviewer measured 17. Both
numbers are right and they are over different sets; my report's sentence around
mine was wrong.** Re-run 2026-09-08, after the review, with the enumeration
published rather than the count:

| set | cases | wrong answers | loud refusals | engines |
| --- | --- | --- | --- | --- |
| the narrower set published below, 2 receivers × 12 lists | 768 | **0** | **12** | agree on all |
| every `Package` row — each instance-arm row × 2 receivers, plus each class-arm row — × 12 lists | 936 | **0** | **17** | agree on all |

**The narrower set, published rather than described**, because describing it is
how the first account went wrong. It is exactly the instance-arm method names
the sweep's own list carried:

    addPackage       addPublicRoutine   addRoutine         classes
    definedMethods   digits             findClass          findNamespace
    findProgram      findPublicClass    findPublicRoutine  findRoutine
    form             fuzz               importedClasses    importedPackages
    importedRoutines loadLibrary        loadPackage        namespaces
    options          prolog             publicClasses      publicRoutines
    resource         resources          routines           setSecurityManager
    source           sourceLine         sourceSize         trace

**That is the instance-arm `send-differs` rows plus `publicClasses`**, which
reads `agree` at BASE and is in the list because it is the row this task's
headline is about. **It is not "the 33 rows"**: `defaultOptions` and `new` are
the class-arm rows and never entered this matrix -- I ran the class arm in a
*separate* loop with a different set of argument lists. My report's sentence
said "every one of the 33 methods, on both receivers", which was false of the
matrix that produced 768, and the arithmetic is what caught it.

The wider set is every row of `corpus/introspection-arity.tsv`'s `Package`
block, instance and class, which is where the reviewer's 17 comes from.
**Neither figure was adjusted toward the other**; both were re-measured here
and each is reported with its set beside it.

**The five cases the wider set adds**, all loud, none a wrong answer:

    .Package~defaultOptions('DIGITS', 'Y')  a package settings write (D12, Phase 7)
    .Package~new('X', 'Y')                  a command is not implemented (Phase 7)
    .Package~new('DIGITS', 'Y')             a command is not implemented (Phase 7)
    .Package~new('R', 'Y')                  a command is not implemented (Phase 7)
    .Package~new('I', 'Y')                  a command is not implemented (Phase 7)

The first is the declined settings write. **The four `~new` cases are not this
task's**: a string is a valid source, so the oracle compiles `'Y'` as a
one-line program and runs it as a command clause, reporting `+++ "RC(127)"`;
this crate refuses because a command clause is Phase 7's. `.Package~new` itself
is correct — it reaches `method_source_lines` and compiles.

**The twelve of the narrower set**, unchanged from the first run:

    p~loadLibrary(1)               p~loadLibrary('X')
    p~loadPackage('X', 'Y')        p~loadPackage('DIGITS', 'Y')
    p~loadPackage('R', 'Y')        p~loadPackage('X', '')
    p~loadPackage('I', 'Y')        p~options('DIGITS', 'Y')
    p~setSecurityManager(.nil)     p~setSecurityManager(1)
    p~setSecurityManager('X')      rp~options('DIGITS', 'Y')

**The zero is credible because the sweep caught its own author.** It found
three defects in this task's *first* implementation and they are why the final
run is clean, not evidence that the first one was: `~loadLibrary` checked its
receiver ahead of its name, so `.Class~package~loadLibrary` answered 98.984
where the oracle answers 88.901; `~loadPackage` named its first argument where
the oracle numbers it, so a missing name was 88.901 against the oracle's
93.903; and `~options('X','')` took the read-only arm ahead of the
empty-argument check, 93.902 against the oracle's 93.900. A sweep that catches
nobody has not been shown to be able to catch anyone.

## What would be observably different if a claim here were false

**This section was wrong in the review's most valuable direction: it listed the
blind spots it knew about and missed four it did not.** The reviewer mutated
the change four ways and every one stayed green — including the gate. It is
rewritten from those mutations, and each is now covered by something that goes
red. **Predictions were written down before any mutation was applied**
(`mutation-predictions.txt` in the task's scratch directory); all four were
confirmed, and one reddened more than predicted.

| mutation | before fix round 1 | after, and what catches it | predicted / observed |
| --- | --- | --- | --- |
| `stdout_multiset` body → `String::new()` | whole corpus binary green, **gate included** | `support::oracle::tests::the_stdout_multiset_comparison_discards_ordering_and_nothing_else` FAILS — sorting must accept a reordering and still catch a changed, missing, added or duplicated line and a lost final newline | predicted FAIL for that test, and pass for `only_the_multiset_stdout_mode_ignores_ordering`, `the_sorted_stdout_licence_covers_an_ordering_difference_and_nothing_else`, `the_multiset_stdout_mode_is_selected_for_exactly_the_licensed_list` and `corpus_differential`; **observed exactly that** |
| `StdoutComparison::Raw` made to compare multisets — the licence leaking to every program | green at 462/462 | `support::oracle::tests::only_the_multiset_stdout_mode_ignores_ordering` FAILS — the raw mode must report a stdout difference on a pure reordering | predicted FAIL for that test alone; **observed exactly that** |
| `settings_of` made to ignore its package | gate, whole `rexx-exec` suite and `introspection_arity` all green | `corpus_differential` FAILS on `lang/package_options.rex`, the new witness whose package declares seven settings: `digits 9` against the oracle's `13`, `SCIENTIFIC` against `ENGINEERING`, `trace [N]` against `[L]` | predicted FAIL on that program; **observed exactly that** |
| `importedPackages` order reversed | green at 462/462 | `corpus_differential` FAILS on `lang/package_writes.rex`, which now reads `~importedPackages[1]` and `[2]` **by position** | predicted FAIL on the differential; **observed that AND the Deviation 8 control**, which reported the same program differing as a multiset — one more than predicted, because reversing the order also changes which name each line carries |

**A fourth control, for the other direction of the licence.**
`the_multiset_stdout_mode_is_selected_for_exactly_the_licensed_list` walks the
whole subset and asserts `stdout_mode` answers `Multiset` **iff** the entry is
on `HASH_ORDERED_STDOUT`, and that the list is neither empty nor the whole
subset. Nothing above implies it: a `stdout_mode` returning `Multiset` for
every path passes the licence control and fails this one.

**And a panic the review found in code this task wrote.** `resource_span`
called `slice::windows(marker.len())` with an empty marker, which
`::RESOURCE d END ''` produces and both interpreters accept at rc 0.
`the_resource_span_is_total_over_the_markers_the_parser_produces` covers the
three marker shapes the parser emits; with the guard removed it fails with
exactly `window size must be non-zero`.

**What still would not be noticed, stated rather than papered over:**

* The per-option validation inside the `~options` write — `p~options('DIGITS',
  'Y')` is 93.905 on the oracle and a Loud here. It is a refusal, never a wrong
  answer, and no test asserts the oracle's number for it.
* The *order* of `package_writes.rex`'s stdout, which is exactly what
  Deviation 8 licenses and what its two controls bound.
* **Register row 16**: the tables these rows answer are half-real
  `StringTable`s. Every collection-returning row in this phase can read `agree`
  while `~items`, `~hasIndex`, `~allIndexes`, `~supplier` and `~makeArray` on
  the answered object are rc 120. Nothing here would notice, because nothing
  here sends them — inherited from Phase 5h and now flagged for Task 9.

## Gates

Run in the gate worktree `/home/moritz/dev/repos/ooRexx-5i-gates`, detached at
the commit above.

**At the code commit `2ebeba1c8`, all seven read exit 0**, from
`gate-status.txt`, whose first line is that sha and whose last line is
`finished`:

* **G1** `cargo fmt --all --check` — exit 0
* **G2** `cargo clippy --workspace --all-targets -- -D warnings` — exit 0
* **G3** `cargo test --release --workspace --no-fail-fast` — exit 0
* **G4** `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` — exit 0
* **G5** `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` — exit 0
* **G6** `cargo test --release -p rexx-exec --test collect_stress` — exit 0
* **G7** `cargo test --release -p rexx-exec --test method_bodies` — exit 0

**Fix round 1 changed code, so the seven were re-run at `1a9c2efce`. All seven
read exit 0 there too**, from `gate-status-r1.txt`, whose first line is that
sha and whose last line is `finished`: `fmt`, `clippy`,
`cargo test --release --workspace --no-fail-fast`,
`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`,
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`,
`collect_stress` and `method_bodies`.

**Fix round 2 changes prose only** -- the counts this project keeps shipping
wrong, deleted rather than corrected -- so the readings above stand for it.

Before the commit, in the working tree:

* `cargo fmt --all --check` — exit 0, no diff printed.
* `cargo clippy --workspace --all-targets -- -D warnings` — exit 0, no
  `error`/`warning` line.
* `cargo test --release --workspace --no-fail-fast` — exit 0, **2284 passed,
  0 failed** summed over every `test result` line of that run.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` —
  **462 of 462 matching**, printed by that binary's own report.
