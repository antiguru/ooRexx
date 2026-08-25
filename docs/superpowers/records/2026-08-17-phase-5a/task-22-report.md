# Task 22 report: the native entry-point registry

**Status: DONE_WITH_CONCERNS.** Everything the brief's "Done when" names is built, run and
recorded, including the control, run live and inverted. The concerns are inherited debts and stated
scope limits, not unfinished work; they are listed at the end.

**Commits**, all on `plan/rust-rewrite`, base `7c23bf891`:

| SHA | what |
|---|---|
| `c59a80672` | the registry, the eager bind, the refusal split, the corpus rows and the tests |
| `592967125` | the eight-axis sitting plus a contribution arm |
| `ee083bd7c` | one wrong exit status in a doc table, corrected |

Tree clean at `ee083bd7c`.

---

## Which `EXTERNAL` form this task moves, and which it does not

The brief asks for this explicitly and warns that the old plan's Task 8 fix rounds produced a wrong
answer in exactly this neighbourhood. Before this task, **one message covered all three forms and
named Phase 7 for each**. It now splits:

| form | before | after |
|---|---|---|
| `::METHOD m EXTERNAL 'LIBRARY REXX [entry]'`, no `ATTRIBUTE` | `::METHOD EXTERNAL is not implemented (Phase 7)` | **implemented**: binds at install |
| `::METHOD m EXTERNAL 'LIBRARY <other>'` | the same message | `::METHOD EXTERNAL naming a library other than REXX is not implemented (Phase 7)` |
| `::METHOD m ATTRIBUTE EXTERNAL '...'` | the same message | `::METHOD ATTRIBUTE EXTERNAL is not implemented (Phase 7)` |
| `::ATTRIBUTE a EXTERNAL '...'` | `::ATTRIBUTE EXTERNAL is not implemented (Phase 7)` | unchanged |
| `::ROUTINE r EXTERNAL '...'`, **every spelling, `LIBRARY REXX` included** | `::ROUTINE EXTERNAL is not implemented (Phase 7)` | unchanged |

`dispatch::native::method_external` is the single function that draws that boundary.
`directive_gap` reads it to decide which message to emit, and `Interp::install_directives` reads the
same function to decide what to resolve, so the form that binds and the forms that are refused
cannot come apart in a later edit.

**The instrument that catches a regression here** is
`run/tests.rs`'s `every_directive_this_crate_cannot_install_refuses_before_the_first_clause`, which
gained three rows -- `::ROUTINE`, `::ATTRIBUTE` and `::METHOD ... ATTRIBUTE`, each naming the
library `REXX` **and a real entry point that package exports**, so the only thing left to refuse
them is the form itself. The rows that were already there name `LIBRARY nosuchlib`, which refuses
whatever the form; without the new rows, moving one of the other forms by accident would have left
every test in the file green. This is an in-crate test only, said plainly: the oracle answers each
of these differently from this crate, so none of them is expressible as a corpus row.

### Why `::ROUTINE EXTERNAL` stays whole, measured

`::ROUTINE ... EXTERNAL 'LIBRARY REXX <name>'` does **not** resolve against the method table this
task builds. A routine resolves against `rexx_routines[]`
(`interpreter/runtime/InternalPackage.cpp:230`, expanded from `NativeFunctions.h`), which exports
`Directory`, `Filespec` and `Beep` and nothing else on this platform. Measured against the oracle
on 2026-08-25:

```
::routine r external 'LIBRARY REXX file_separator'   90.999 rc 166, "Unable to find external routine"
::routine r external 'LIBRARY REXX Filespec'         rc 0, and the routine runs
::routine r external 'LIBRARY zzznolib zzzr'         98.903 rc 158
```

So the `LIBRARY REXX` routine form is a *third* table, not a second use of this one, and moving it
would have been a separate build. It stays Phase 7's.

### Why `::ATTRIBUTE EXTERNAL` stays whole

`::ATTRIBUTE` and `::METHOD ... ATTRIBUTE` are one mechanism: both build `GET`- and `SET`-prefixed
procedure names and resolve a method for each (`parser/DirectiveParser.cpp:867`, `:1678`). The
prefix is **prepended**, not appended -- `concatToCstring` appends the receiver to its argument
(`classes/StringClass.cpp:1405`-`:1416`). Measured, oracle, both at rc 166:
`::attribute a external 'LIBRARY REXX file_separator'` and
`::method m attribute external 'LIBRARY REXX file_separator'` are each
`90.998 Unable to find external method "GETfile_separator"`.

`rexx-parse`'s `ExternalSpec` doc said "with `GET` or `SET` appended". That sentence was false in
the tree before this task; it is corrected with the measurement beside it.

---

## What was built

**A registry** (`crates/rexx-exec/src/dispatch/native.rs`) of the entry points the `REXX` package
exports to `::METHOD ... EXTERNAL`, its names and order taken from
`interpreter/runtime/NativeMethods.h` (which `InternalPackage.cpp:213` expands into `rexx_methods[]`
and `:241` hands to the package named `REXX`). `interpreter/platform/unix/SysNativeMethods.h` adds
nothing on this platform; its whole body is the comment "Unix doesn't currently have any of these."

Each row carries the entry point's name, the family it belongs to, and either a body this phase runs
or `Deferred`.

**Resolution** happens in `Interp::install_directives`' first walk -- the same walk that answers
every other translation-time refusal -- because the oracle resolves while the directive is being
translated (`createNativeMethod` raises from inside `methodDirective`,
`parser/DirectiveParser.cpp:1385`). The lookup is a caseless linear scan, which is
`locateMethodEntry`'s own shape (`package/LibraryPackage.cpp:313`, compare at `:324`). The library
name is compared **exactly**, which is `packages->get(name)` against the key
`loadInternalPackage(GlobalNames::REXX, ...)` put there (`package/PackageManager.cpp:233`, `:86`).

**Binding** records `MethodId -> &'static NativeExternal` in `Interp::native_externals`, filled by
`install_one_method` through a new `InstallBody` enum that replaced its `Option<GeneratedKind>`
parameter. `dispatch::Interp::invocable` reads that table **last**, after the primitives, the body
table and the generated methods, so no send that reached a table before this task pays for it.

**Two entry points are implemented**, the stated scope addition from Phase 7:
`file_separator` answers `/` and `file_path_separator` answers `:`
(`platform/unix/SysFileSystem.cpp:1358`-`:1361`, `:1369`-`:1372`). `StreamClasses.orx:546`-`:549`
is why: its `::CONSTANT separator` and `::CONSTANT pathSeparator` send them while the package is
still installing.

**Everything else binds and refuses at the send**, naming the entry point and its owning phase:

```
rexx-exec: the LIBRARY REXX entry point "stream_chars" is not implemented (Phase 7)
```

**Argument counts.** A `LIBRARY REXX` entry point checks its own count from inside its activation
and reports **88.922**, not the 93.902 a primitive method's `CPPCode::run` reports, and the report
carries the program name with **no source line** -- because the condition object has no `POSITION`
when the raise comes from a native activation (`concurrency/Activity.cpp:1453`-`:1459`). That
needed a new `Delivery::lineless`. Measured, oracle and both engines byte-identical:
`.k~sep(1)` is rc 168, `Compiled method "SEP" with scope "K".`, the sending clause, then
`Error 88 running <path>:  Invalid argument.` and `Error 88.922: Too many arguments in invocation;
0 expected.` **Without this the arguments would have been silently ignored and `/` answered, which
is the silent-wrong-answer class the dispatch brief calls the worst defect here.**

---

## The family split and its phase assignment

The C++ tree partitions the entry points by translation unit, checked by grepping each name's
`RexxMethodN` definition; the partition is exact and each entry lands in exactly one:

| family | C++ | owner | authority |
|---|---|---|---|
| timer | `platform/unix/TimeSupport.cpp` | **Phase 6** | `Alarm~init` is `reply` then `self~!startTimer(...)` (`RexxClasses/CoreClasses.orx:1555`, `:1557`), so the entry runs on the activity the `REPLY` split off, and this crate already spells its `REPLY` refusals `(Phase 6)` |
| stream | `streamLibrary/StreamNative.cpp` | Phase 7 | the roadmap's Phase 7 notes: "Phase 7's stream model is a subsystem, not file I/O" |
| queue | `classes/RexxQueueMethods.cpp` | Phase 7 | D7: "Blocks: Phase 10 (and partially Phase 7 -- external queues)" |
| file | `streamLibrary/FileNative.cpp` | Phase 7 | D11: "`.File` ... Phase 7 owns it alongside the file-system `Sys*` calls" |

**Decision recorded, with its cost if wrong.** The stream and file assignments are the roadmap's own
words. The **timer** and **queue** assignments are mine: the roadmap never names `alarm_*`/`ticker_*`
or `rexx_*_queue`. Cost if wrong: one word in one refusal message per family. `Family::owner` is the
only place either string is written, `tests/native_entries.rs` derives the expected message from it
rather than holding a copy, so changing an assignment is a one-line edit that nothing else has to
follow. Queue is the weaker of the two -- D7 splits the daemon between Phases 7 and 10 and the
`RexxQueue` class methods sit on the Phase 7 side of that split as I read it, but "partially" is the
word the roadmap uses and it does not enumerate.

---

## Verification: what ran

### The five gate commands, from `rust/`, at `ee083bd7c`

| gate | result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --release --workspace` | no failures |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **238 of 238 matching**, no failures |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **238 of 238 matching**, no failures |

**Corpus count 232 -> 238.** The 238 is the gate's own line. The 232 is **not** a gate line I
read: I did not run the gate before changing the tree, so I counted the subset files at
`7c23bf891` instead (`git show <base>:rust/corpus/phase-4{a,b,c}.txt` and `phase-5a.txt`, blank and
comment lines dropped), which is the number the gate would have printed, and it agrees with the
controller's dispatch note. Six rows added.

### The oracle-differential rows (the brief's item 2)

All five new `corpus/lang/` programs match the oracle byte for byte on all three descriptors, on
both engines:

* `directive_method_external_bind.rex` -- `StreamClasses.orx:546`-`:549`'s own shape (a `private
  class` external method read back through a `::CONSTANT` expression), plus the default entry name,
  a case that differs from the exported one, a `self~sep` from inside a `::METHOD` body, a `Method`
  object over an external instance method, and `~hasMethod` on three names. rc 0.
* `directive_method_external_missing.rex` -- **the eager-bind failure**: rc 166, stdout **empty**,
  the directive's own line 6 echoed, `Error 90.998: Unable to find external method
  "no_such_entry_point_xyz".`
* `directive_method_external_arguments.rex` -- the 88.922 above, rc 168.
* `directive_method_external_duplicate_wins.rex` -- a duplicate dictionary key beats the entry
  point: 99.902 rc 157.
* `directive_method_external_source_order.rex` -- the entry point beats a duplicate `::ROUTINE`
  pair standing later: 90.998 rc 166.

Plus `gate-tables/directives/method__external__subkeyword.rex`, gate table D's own `::METHOD
EXTERNAL` probe, whose row moved from `diverge-both` to **`agree`** and which therefore joins the
subset, per `corpus/gate-tables/README.md`'s own rule. It stays in `directives/` because table D's
row set still names it.

Determinism: each of the three programs whose output could in principle vary was run twenty times
against the oracle and hashed, twice; one hash each time. `rexx-diff`'s self-test over the whole
corpus (same binary twice) reports **575 programs, 0 divergences, exit 0**, which is the figure the
gate-tables README carried as 440 and which was already stale before this task.

### "All three `.orx` files install with no unresolved external" (the brief's item 1)

`dispatch::native::tests::every_bootstrap_external_binds_to_an_entry_point_this_registry_holds`
**parses** `CoreClasses.orx`, `StreamClasses.orx` and `platform/unix/PlatformObjects.orx` with this
crate's own parser and asks `method_external` about every directive. Every `EXTERNAL` in the three
files is the moved form and every one resolves. It also asserts that no `::ROUTINE` or
`::ATTRIBUTE` in those files carries an `EXTERNAL`, because "the bootstrap uses only the `::METHOD`
form" is the premise this task's scope rests on and it is checked rather than written down.

`the_bootstrap_files_and_the_registry_name_the_same_entry_points` asserts the two sets are equal in
**both directions**, caselessly. The reverse direction is deliberate: a name the registry holds and
no `.orx` declares is how a hand-written table's drift from `NativeMethods.h` would otherwise go
unnoticed.

Running the three files directly through `rexx-run` now gets **past** every `EXTERNAL`:
`PlatformObjects.orx` is rc 0, `StreamClasses.orx` reaches `98.909 Class "COMPARABLE" not found` at
its own line 506 (`Comparable` lives in `CoreClasses.orx`, which a standalone run has not loaded),
and `CoreClasses.orx` reaches a message scope override refusal. Both of those are Task 23's and 5b's
respectively, not unresolved externals.

### "Invoking an unimplemented entry is loud and names its phase" (the brief's item 3)

`crates/rexx-exec/tests/native_entries.rs`, over one committed program per family under
`corpus/gate-tables/native-entries/`:

* the family set the registry names and the programs on disk are the same set, both directions;
* each program's entry point is **read out of the program's own directive clauses**, then checked
  against the registry -- so a program repointed at an implemented entry point fails;
* each runs on **both engines** (through `run_on_both_engines`, which also checks the two agree and
  that neither refused a body to the other);
* `stdout` must be exactly the line the program prints **before** its send. That is what separates
  "the bind succeeded and the send refused" from "the install refused": a regression moving the
  refusal back to install time empties `stdout` and reddens the row;
* `stderr` must be exactly the message derived from the registry row, entry point and owner;
* and no such program may appear in any `phase-*.txt`.

That last one was **inverted live**: adding `gate-tables/native-entries/file.rex` to
`phase-5a.txt` reddens `no_family_program_is_a_corpus_row`; restored from a scratchpad copy,
`sha256` equal, tree clean, test green again.

`the_implemented_entry_points_are_the_ones_the_plan_names` pins the implemented set by name, so
implementing a further entry point and deferring one of these are both failures.

**What this cannot see.** It does not say the refusal is the *right* answer for the entry point --
only that this crate declines rather than answering. A program rewritten to fail for an unrelated
reason before its send would print nothing and fail on `stdout` rather than on its message, so it is
caught, but by the wrong assertion. And it says nothing about entry points no program names.

### Why none of this is a corpus row

The oracle answers where this crate declines. Measured: `.k~probe` on a class method bound to
`stream_chars` is the oracle's `48.1 Failure in system service: Stream not initialized.` at rc 208,
against `rexx-exec: ...` at rc 120 here. A refusal the oracle does not share is not expressible as a
differential row, so **the corpus gate cannot see one of these refusals become a wrong answer**, and
`native_entries.rs` is the only instrument that can. Said plainly, as the constraint requires.

---

## The control, run and inverted

The brief's control: **binding lazily instead of eagerly** turns the missing-entry program from rc
166 with empty stdout into a run that prints its prologue.

1. `crates/rexx-exec/src/lib.rs` copied to the scratchpad, `sha256`
   `ac329627f47a0dc71732f56d84821999ff0450c2c08f9d9c274c050672fd6d14`.
2. The eager-bind block in `install_directives` disabled (`if false && ...`), release rebuild.
3. `directive_method_external_missing.rex` on both engines: **rc 0, stdout `prolog ran`**, stderr
   empty -- against the oracle's rc 166 and empty stdout. The corpus differential went
   **238 -> 235 of 238**, reddening `directive_method_external_missing.rex`,
   `directive_method_external_source_order.rex` and
   `gate-tables/directives/method__external__subkeyword.rex`.
   `directive_method_external_duplicate_wins.rex` correctly stayed green: the duplicate check
   answers first either way.
4. Restored from the scratchpad copy; `sha256` equal, `git diff HEAD` empty for that path, tree
   clean. Rebuilt (a clean `git status` says nothing about `target/`) and re-ran the corpus gate:
   **238 of 238**.

---

## The sitting

Pin staleness test first: `15a1ffa98` is an ancestor of `HEAD`, `bench-baselines/pinned/rexx-run-15a1ffa98`
hashes to `141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b` as `PINNED.md` records,
and every commit `git log 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml`
lists is one of this plan's own task commits. The guard block in `global-constraints.md` and the one
in the plan agree on all eight axes; both were checked before running.

Eight axes, five rounds, `task=22`, `commit=c59a80672`, appended to
`bench-baselines/phase-5a-arms.tsv`. `instructions:u`, `pinned>head`:

```
alloc4c        1.00305 tw / 1.00371 ir      compound   1.00697 tw / 1.01047 ir
arith          0.99608 tw / 0.98867 ir      emptyloop  0.99544 tw / 0.99202 ir
strings        1.00831 tw / 1.01194 ir      varlookup  0.99716 tw / 0.99426 ir
dispatchclass  1.01450 tw / 1.01741 ir      rexxcps    1.01506 tw / 1.01869 ir
```

**Four of those are at or above 1%, and none of it is this change.** The same file's rows at
`21-fixround-2-inlined` and `21-fixround-3` carry the same figures to four decimals, taken before
this commit existed -- `dispatchclass` was 1.015507 tw and `rexxcps` 1.015346 tw there. It is
accumulated drift from Tasks 1 to 21 against the phase's own pin.

**The direct instrument for that** is a contribution arm, recorded as `task=22-contribution`: the
base build at `7c23bf891` (built from `git archive` into a scratch tree, its own `target/`) against
this one, interleaved by `rexx-arms` in one sitting. `instructions:u`, `base>head`:
`dispatchclass` **0.99985** on both engines, `alloc4c` **1.000001**, `rexxcps` **0.99972** tw and
**0.99967** ir. So the fourth table read in `invocable` and the fourth arm in `invoke` cost the send
path nothing measurable -- which is what putting the new table last was for. `cycles:u` figures are
in the TSV and are not a result on their own.

**No sitting for `ee083bd7c`, and that is measured rather than argued.** It edits a `//!` doc
comment in `src/`, so the plan's "only `tests/`, `corpus/` or `docs/`" exemption does not apply on
its face. The whole-binary hash does change -- the edit shifts line numbers and therefore
debuginfo -- but the `.text` sections of the two release builds are byte-identical
(`32c2549a40c437f2abb3d63f325d2c6395c066dd491765190db6b8fc2c98a4bf` for both), and a rebuild with no
source change reproduces its own hash, so the build is deterministic and the difference is debug
information alone.

---

## Prose in the neighbourhood that this task falsified, and corrected

Adding the witness made sentences beside it false. Each was re-run before rewriting:

* `method_body_gap`'s doc said "`EXTERNAL` never reaches a send -- `directive_gap` refuses it while
  the package is still installing". The moved form does reach a send; it never reaches
  `method_body_gap` because `invocable` answers it from `native_externals` first.
* `GeneratedKind`'s doc said `DELEGATE` and `EXTERNAL` are both absent because "this crate refuses
  them". Now true of `DELEGATE` only.
* `directive_gap`'s `::ROUTINE EXTERNAL` comment said "98.903 rc 158 ... in both shapes". True of the
  shared-library spellings; the `LIBRARY REXX` ones are 90.999 rc 166 and rc 0.
* `staged_gap`'s stage table: every `EXTERNAL` row in it names a shared library, which is now
  load-bearing and is said.
* `Invocable`'s doc and `invocable`'s doc both counted the kinds ("both invocable kinds", "neither
  way"); already stale before this task, now phrased over the set.
* `gate_table_d.rs`'s `ORACLE_REFUSES` doc said `::ATTRIBUTE`, `::METHOD` and `::ROUTINE`'s
  `EXTERNAL` all "name a shared library". Two of those three probes name `LIBRARY REXX` and refuse
  on the *entry point*; that was false before this task and is corrected with the measurement.
* `gate_table_d.rs`'s `owning_phase` note asked the task that draws the `::ROUTINE` boundary to
  decide it. Decided, and the consequence recorded there: `::routine r external 'LIBRARY REXX
  Filespec'` is oracle rc 0 and crate rc 120, a divergence **no row of that table sees**.
* `rexx-parse`'s `ExternalSpec` doc: `GET`/`SET` are prepended, not appended, and to the procedure
  rather than always to the upcased method name.
* `phase-4-exclusions.txt` gained the narrowing and the table of what stays refused.

**One false number shipped and was corrected in `ee083bd7c`**: the registry's own doc table said
`::attribute a external 'LIBRARY REXX file_separator'` is rc 158. It is rc 166; 158 is the
shared-library rows' status, carried down the column. Found by re-running every numeric claim in the
diff after committing. Every other measured figure in the diff was re-run and holds.

---

## Concerns and inherited debts

1. **`::ATTRIBUTE EXTERNAL` is now the only gate table D row owned by `5a` that does not `agree`**
   (the table reads `5a: 36 rows, 1 not yet agree`). `5a` is not a gated phase today, so nothing
   fails. The oracle's answer for `'LIBRARY REXX <x>'` is always `90.998` naming `GET<x>`, because
   the REXX package exports no `GET*`/`SET*` entry, so the fix is small: resolve `GET`+procedure
   against this registry and raise 90.998 on the miss, the `SET` half after it. **I did not build
   it**, because the brief moves one form and warns that this neighbourhood produced a wrong answer
   when an earlier task widened. Cost of leaving it: one reported-not-gated row, and whichever task
   closes 5a's gate has to take it.
2. **`::ROUTINE ... EXTERNAL 'LIBRARY REXX <routine>'` is a silent-status divergence with no gate
   row.** Oracle rc 0 and the routine runs (`Filespec` measured end to end); this crate refuses at
   rc 120. It is Phase 7's by the brief, and table D's row for `::ROUTINE EXTERNAL` is filed against
   Phase 7 and probed with a shared library, so nothing measures the `LIBRARY REXX` spelling.
   Closing it needs `corpus/docs/directive-options.txt` to distinguish the two forms.
3. **The two implemented entry points answer the unix values unconditionally.** `/` and `:` are
   hardcoded from `platform/unix/SysFileSystem.cpp`; there is no `cfg(windows)` arm, because writing
   one nothing on this machine can run would be unverified code. This matches Task 23's stated scope
   (it embeds `platform/unix/PlatformObjects.orx` and states the windows file is unread), but a
   Windows build of this crate would answer wrongly rather than loudly.
4. **The timer and queue phase assignments are mine, not the roadmap's.** See the table above for
   the reasoning and the cost if wrong.
5. **The registry is a hand-written copy of `NativeMethods.h`.** Nothing derives it. What guards it
   is the both-directions equality against what the three `.orx` files declare, which fails if
   either side moves -- including if upstream adds an entry point no bootstrap file uses. That is a
   true drift signal, but it means a future upstream sync reddens that test rather than the file
   that changed.
6. **`Family::ALL` was dropped** during the seam fix; the family set is derived from the rows.
7. **The seam.** `dispatch::seam::Cleared` is `pub(super)`, so a child module of `dispatch` *can*
   name it, which `tests/dispatch_seam.rs`'s
   `the_seam_token_is_named_only_by_the_dispatch_module` does not allow for -- its doc says
   `dispatch.rs` is the widest scope. Rather than weaken that test I moved the two entry-point
   bodies into `dispatch.rs` beside the other natives, so `native.rs` names no `Cleared` **in
   code** and the assertion is untouched and still true. **The "in code" is load-bearing and this
   sentence did not carry it in the first draft**: `native.rs` does name `Cleared`, in the doc
   comment on `ExternalBody`, and the assertion passes because `code_occurrences` skips a line
   whose trimmed start is `//`. What the move actually bought is that no *consumer* of the token
   is written outside `dispatch.rs`, which is what the test is for. `tests/dispatch_seam.rs`'s own
   module doc says that scan counts a mention inside a comment; it does not, and that half is
   pre-existing and untouched. **The doc's claim about the module tree is still
   imprecise** (`pub(super)` reaches descendants), and a later task adding another `dispatch/` child
   that writes a native body will meet this again.
8. **A `debug_assert!` guards the one combination `install_method` cannot decide** -- a directive
   that is both bound to an entry point and generating a method. The parser makes `EXTERNAL` and
   `ABSTRACT` mutually exclusive today, so it cannot fire; it is there because if that ever changed,
   silently preferring one over the other would drop the other. It fires only in the debug gate.

**What a reviewer should not read into the green.** The corpus gate's 238 of 238 says nothing about
the deferred entry points: none of them is a corpus row and none can be. The `.orx` test says every
external in those files resolves; it does not say the files install, which is Task 23's.

---

# Fix round 1

**Commit `0ced3f020`.** Comments, tests and two corpus rows. Spec compliance was PASS and no
behaviour changed: the release binary's `.text` section is
`32c2549a40c437f2abb3d63f325d2c6395c066dd491765190db6b8fc2c98a4bf` before and after, over a forced
rebuild (`touch` on all four edited `src/` files), so no sitting is owed. The whole-binary hash does
differ, in debug information only.

All five gates green. **Corpus 238 -> 240**, read off the gate's own line.

## The four majors

**M1, the orphaned doc block.** `pub struct NativeEntryPoint` and `pub fn native_entry_points` were
inserted between `run_program`'s doc comment and `run_program`, so that block documented the struct
and the public entry point had none. They moved below `render_ir` under a heading of their own.

**The sweep that found whether there were others**, since fixing the one I was handed is not a
sweep: over every hunk of `7c23bf891..HEAD` that adds an item declaration, walk back through the
*new* file's lines and report what precedes it. Every added item is preceded by its own doc, its own
attribute, or nothing at all; `run_program` was the only item that lost one.

**M2, the falsified caveat.** `staged_gap`'s second row claimed to be reasoned rather than probed
because "no library in this tree loads". I reproduced the reviewer's probe:
`::class a` / `::constant kk (1/0)` / `::routine r external 'LIBRARY REXX Filespec'` is **oracle rc
214** at the constant's own divide against **rc 120** here, both engines. The row is now a probe in
both tables and the caveat is gone from both.

**The correction I wrote last round was false and is gone**; so is the one I wrote to replace it in
this round's first draft. That second one read "`LIBRARY REXX` is what makes the second row runnable
-- it is the one library this binary can resolve". Before committing it I probed the obvious
near-miss and it fell over: **`::method m external 'LIBRARY rexxutil SysFileTree'` is 90.998 rc 166,
the entry-point error**, so `librexxutil` opens; only the upper-case spelling `LIBRARY REXXUTIL` is
98.903 rc 158. A shared library in this tree does load, which means the *original* caveat's ground
was false before `LIBRARY REXX` was ever bound. That measurement is in the exclusions record. What
is left in the source is the row and no reason at all, which is Task 21's rule applied.

**M3, the deleted refusal message.** Found with a collapsed-comment scan, not grep: the string is
hard-wrapped across two `///` lines and `grep` over the tree returns only two historical entries in
`docs/superpowers/records/`, which correctly record what the crate said in August and are not
touched. Exactly one live instance, in `check_member_keys`' doc.

**The repair is not a repaired quote.** The sentence said the reverse order "is not evidence for
that placement", because this crate never answered it the oracle's way. On the moved form it now
does: `::method m external "LIBRARY REXX no_such_entry_point_xyz"` above a plain `::method m` is
**90.998 rc 166 echoing the `EXTERNAL` directive, oracle and both engines**. So the reverse order is
evidence, and `corpus/lang/directive_method_external_before_duplicate.rex` is that program. The
`LIBRARY nosuchlib` half keeps the old argument with its message corrected to what it now emits.

**M4, the positional claim.** "No `blame_native_method` below this arm" is narrowed to the arms it
is about (`Rexx` and `Generated`) and says explicitly that `External`'s implemented half sits
between them and does blame.

## The two corpus rows this round added

Both byte for byte on three descriptors, both engines, and both reached from findings rather than
invented:

* `directive_method_external_before_duplicate.rex` -- M3's own program. 90.998 rc 166.
* `directive_method_external_not_a_staged_gap.rex` -- `::class a` / `::constant kk (1/0)` /
  `::method m external 'LIBRARY REXX file_separator'`, **42.3 rc 214**: a bound `::METHOD EXTERNAL`
  is not a gap, so it no longer preempts an install-time failure standing above it. The same file
  with a `::ROUTINE` in that position is M2's row and is still refused here.

Both perform zero collections and joined `NO_ALLOCATION_PROGRAMS`; both have sourceline
expectations; both are in `phase-5a.txt` and `EXPECTED_SUBSET_5A`.

## The seven minors

* **MINOR 1** swept by running a collapsed-comment search over `dispatch.rs` rather than fixing the
  list handed to me. Four more sentences, one an entire module-doc section. All of them are now
  phrased over the set: the module doc's clearance paragraph points at `Invocable` instead of
  enumerating, `Cleared`'s "both invocable kinds" becomes "every function that runs a resolved
  method", and `clear`'s two are widened to every invocation.
* **MINOR 2** `Arity`'s doc now says where each consumer's count comes from and which error each
  refusal raises.
* **MINOR 3** the four cardinalities struck, plus the ones the same scan found beside them: a
  `Family` variant count, a `::ROUTINE`/`::ATTRIBUTE` count, a corpus-row count in a `.rex` header,
  a "two forms" in `gate_table_d.rs`, a "two entry points" in `ast.rs`, and two more in `lib.rs` and
  `native_entries.rs`.
* **MINOR 4** `Activity::displayCondition` corrected to `Activity::display`, checked with
  `/bin/grep -n`: `display` opens at `:1414` and `displayDebug` at `:1492`, so `:1453`-`:1459` is
  inside `display`; `displayCondition` is at `:536`.
* **MINOR 5** the stray `--` is gone and the flat "Nothing here belongs in `phase-*.txt`" is
  replaced by the rule that already qualified it, naming the file that is now the counterexample.
* **MINOR 6** corrected in place above, in concern 7.
* **MINOR 7** the guard's limitation is now stated in the test's own doc, and it is stated because
  the fix failed. **I wrote a per-file version, ran it inverted, and it did not fire**: dropping
  `platform/unix/PlatformObjects.orx` from `bootstrap_files` left the test green, because the walk
  and the expectation both come from that one function -- it compared a list with itself, which is
  the witness-that-cannot-fail shape. Struck. What *does* catch a file dropping out is
  `the_bootstrap_files_and_the_registry_name_the_same_entry_points`, through the names the file
  contributes: **inverted live**, dropping `StreamClasses.orx` reddens it, restored from a
  `sha256`-checked copy and green again. The file that contributes no names is the one nothing can
  speak for, and the doc now says so.

## What this round could not check

The `.text` comparison says no executable code changed. It does not say the *reasoning* in the
comments is right, only that it is now consistent with what I ran. Every claim added or edited in
this round was run before it was written down, including the two that turned out false and were
struck rather than rewritten.

---

# Fix round 2

**Commit `5b054d4b5`.** Comments, one doc file and a corpus header. All five gates green,
**240 of 240**. The release binary's `.text` section is
`32c2549a40c437f2abb3d63f325d2c6395c066dd491765190db6b8fc2c98a4bf` before and after a forced
rebuild (`touch` on both edited `src/` files), byte-compared with `cmp`, so no sitting is owed.

## The methodological finding, which is worth more than the fix

Round 1's report said MINOR 1 was "swept by running a collapsed-comment search over `dispatch.rs`
rather than fixing the list handed to me". **The search found exactly the four it was handed and
missed a fifth**, in the doc of the very function whose `match` that round edited.

**Why.** The needle was
`both invocable|invocable kinds|either a .NativeMethod|native or Rexx-bodied|both entry points|two kinds|neither runs`
-- every alternative in it was lifted from a string the review had quoted. A pattern built out of
its own input can only return its input. The missing sentence spelled the same idea as
`either kind`, which no alternative covered.

**What replaces it.** A needle derived from the *concept*: over every collapsed comment block in
`crates/`, flag any quantifier (`both|either|neither|two|three|four|all`) within eighty characters
of `kind`/`kinds`/`invocable`. It knows nothing about which instances exist.

**And it is checked against something, which is the part that was missing.** Run over `dispatch.rs`
at three points:

| tree | flagged in `dispatch.rs` |
|---|---|
| `7c23bf891`, this task's base | 6, including the `either kind` sentence and the two `both invocable kinds` |
| `0ced3f020`, after round 1 | 4, including the `either kind` sentence |
| HEAD | 3, all of them sentences that name their members instead of counting: `Invocable`'s doc, and two pre-existing ones about `Primitive::SmallInt` and about receivers answering different kinds |

So the instrument fires where the defect was and is quiet where it is not. **The base-commit run is
the useful one**: the `either kind` sentence was already false before this task -- there were three
kinds then, `Generated` among them -- so this was an inherited defect that two reviews and one
narrow sweep walked past.

**It also found one nobody had named.** `tests/dispatch_seam.rs`'s module doc called `Cleared` a
bound on "both things a resolved method can be", naming a `NativeMethod` and
`Interp::enter_method_body`; the generated accessors take one too, and did before this task. It now
says every function that runs a resolved method takes one, keeping the `error[E0423]` measurement
that is the paragraph's actual evidence. Pre-existing, last touched at `7c23bf891` by another task.

## The must-fix items

**1. "Two do." struck.** The re-review measured eight shared libraries that open and the controller
spot-checked three; the number was wrong as a count of those, and counting `LIBRARY REXX` in
contradicts the next clause of its own sentence, which says that one is compiled in rather than
loaded. `Shared libraries in this tree do load` carries the whole argument and cannot rot. The
`librexxutil` measurement beside it stays, and it is the one that makes the point.

**This is where round 1 abandoned its own rule.** The strike-do-not-replace rule was applied in
`lib.rs` and not in `phase-4-exclusions.txt`, and the new falsehood was exactly there. The source
copy states the loss, names `::ROUTINE` as its cause, cites the arm, and offers no reason -- that is
the shape, and the doc copy now matches it.

**2. `Interp::invoke`'s doc.** `None` is not one kind's property, and a third kind does it:
`write_attribute` returns `Ok(None)`. Measured rather than read off the code -- `.k~a = 5` on
`::attribute a class` is rc 0 as a whole clause, and `say .k~'A='(5)` is
`91.999 Message "A=" did not return a result.` at rc 165, oracle and both engines.

**3. The falsified neighbour.** "The row below matched the oracle byte for byte at 62de43c0f and
refuses now" had two rows under it after round 1, and as written it extended an unmeasured
historical claim to a row first probed on 2026-08-25. It now says which row that clause is about.

**The blemishes**: the 97-column line rewrapped (the paragraph is now ≤78 throughout, checked by
measuring every line), and `_not_a_staged_gap.rex` moved into alphabetical order in
`NO_ALLOCATION_PROGRAMS`.

**The judgement call the re-review raised as a note, taken**: "no longer preempts" became "does not
preempt" in the corpus header, its sourceline fixture and `phase-5a.txt`, and
`method_body_gap`'s "and no longer for one reason" was struck outright. Precedent in other corpus
headers does allow the past tense; it buys nothing here and has a tense that can rot.

## A hazard that cost a source file, and nearly cost a false claim

**The scratchpad is shared between the agents on this task, and another agent's `collapse.py`
overwrote mine.** Theirs takes `(root, outfile)` and opens the second argument for **writing**;
mine takes a list of files and prints. My invocation passed a file list, so the second file in it --
`rust/crates/rexx-bench/src/arms.rs` -- was truncated to zero bytes, and the walk over a
non-directory then produced nothing.

Two consequences, and the second is the dangerous one:

* `arms.rs` was destroyed in the working tree. HEAD was intact, so it was restored with
  `git show HEAD:<path> > <path>` -- not `git checkout --`, which is forbidden here -- and
  `git diff` against HEAD is empty for it.
* **The run that did the damage printed no hits, and "no hits" is exactly what a clean sweep looks
  like.** Had I recorded that zero as evidence, this report would carry a vacuous claim with a
  destroyed file behind it. What caught it was `git status` showing a file I never edited.

Both tools now live under a uniquely-named directory of my own. The general rule: a scratchpad
script is another agent's namespace, and a tool that writes is indistinguishable from a tool that
reads until you check what changed.

---

# What a future task inherits from Task 22

## Reachable now, and what runs it

* **A registry of the `REXX` package's exported method entry points**
  (`crates/rexx-exec/src/dispatch/native.rs`), names and order taken from
  `interpreter/runtime/NativeMethods.h`, partitioned into four families by the C++ translation unit
  that defines each name. `dispatch::native::method_external` is the single place the
  `EXTERNAL`-form boundary is drawn; `directive_gap`, `install_directives` and `install_method` all
  read it, so the form that binds and the forms that are refused cannot come apart.
* **`::METHOD ... EXTERNAL 'LIBRARY REXX [entry]'` binds at install.** A miss is the oracle's own
  90.998 at rc 166 with stdout empty. `file_separator` and `file_path_separator` answer; every other
  entry binds and refuses at the send, naming its owning phase.
* **`Interp::native_externals`**, read last of `dispatch::Interp::invocable`'s four tables, so the
  send paths that existed before pay nothing for it. `Invocable::External` is the arm.
* **`Delivery::lineless`** in `error.rs`: a raise from inside a native activation renders
  `Error 88 running <path>:` with no ` line <n>`. Anything else raising from a native body wants it.
* **`rexx_exec::native_entry_points()`**, a flattened public projection of the registry, so a test
  can walk it without the seam token escaping `dispatch`.

## What Task 23 needs from this and gets

Every `EXTERNAL` the three bootstrap files declare is the moved form and every one resolves, checked
by parsing the files with this crate's own parser rather than scanning them
(`every_bootstrap_external_binds_to_an_entry_point_this_registry_holds`), with the registry and the
declared set held equal in both directions. Running the three files through `rexx-run` now stops at
Phase 5b and Task 23 work, not at an unresolved external.

## Debts, each with what closes it

1. **`::ATTRIBUTE EXTERNAL` is the only `5a`-owned gate table D row that does not agree.** The
   oracle's answer for `'LIBRARY REXX <x>'` is always 90.998 naming `GET<x>`, because the package
   exports no `GET*`/`SET*` entry, so the close is: resolve `GET`+procedure against this registry
   and raise 90.998 on the miss, the `SET` half after it. Deliberately not built -- the task moves
   one form.
2. **`::ROUTINE ... EXTERNAL 'LIBRARY REXX <routine>'` is a silent-status divergence with no gate
   row.** Oracle rc 0 and the routine runs (`Filespec` measured end to end); this crate refuses at
   rc 120. Table D files `::ROUTINE EXTERNAL` under Phase 7 and probes it with a shared library, so
   nothing measures the `LIBRARY REXX` spelling. Closing it needs
   `corpus/docs/directive-options.txt` to distinguish the two forms.
3. **The two implemented entry points hardcode the unix `/` and `:`.** No `cfg(windows)` arm; a
   Windows build answers wrongly rather than loudly. Matches Task 23's unix-only embedding.
4. **The timer and queue phase assignments are judgement, not roadmap text.** `Family::owner` is the
   only place either string is written and the test derives the expected message from it, so
   re-assigning either is a one-line edit.
5. **The registry is a hand-written copy of `NativeMethods.h`.** Its guard is the both-directions
   equality against the bootstrap files, so an upstream addition that no `.orx` uses reddens that
   test rather than the file that changed.
6. **`corpus/gate-tables/native-entries/` is not a gate table**, and the oracle answers every
   program in it, so none can ever be a `phase-*.txt` row. `tests/native_entries.rs` asserts that.
7. **`tests/dispatch_seam.rs`'s module doc still claims `code_occurrences` counts a mention inside a
   comment.** It does not -- it skips any line whose trimmed start is `//`. Pre-existing, untouched,
   and the reason the seam's bound is about *consumers* of the token rather than about the string.
8. **`corpus/phase-5a.txt:155`** carries a pre-existing phase-status comment of the kind
   `rust/CLAUDE.md` says to assert rather than write down. Not this task's.

## Two instruments this task added that generalise

* **The `.text`-section hash** (`objcopy --only-section=.text -O binary` plus `sha256sum`, over a
  forced rebuild) is the right no-codegen proof for this profile. The whole-binary hash is unusable,
  not merely noisy: `debug = true` puts DWARF in the binary, so a comment edit shifts line numbers
  and rewrites `.debug_line`. And because `.rodata` sits *before* `.text` at a lower file offset with
  RIP-relative references into it, a byte-identical `.text` also rules out a `.rodata` size change --
  the layout artifact this project has measured as worth several percent on a bench axis.
* **The collapsed-comment scan.** Hard-wrapped prose defeats `grep`; collapsing each `//`/`///`/`//!`
  run to one logical line first is what makes a phrase searchable. The tools are
  `t22-collapse.py` and `t22-kindsweep.py`. Two rules learned the hard way and stated where the next
  task can use them: **derive the needle from the concept, not from the instances a review handed
  you**, and **run the needle against a tree where the defect exists**, or a zero is
  indistinguishable from a broken instrument.
