# Task 8 report: the directives that reach a library

Committed at `ff03676bc` (ruling 19), `08d232ecc` (the wiring),
`e427cf61e` (the call's own error paths), `6c5f4e8a8` (a comment round) and
`3245708c0` (a wrong answer the wiring introduced, closed), `8da8daecf`
(the corpus README and one early return) and `f07592242` (the collect-stress
gap). The report itself is not committed: `.gitignore:30` ignores `.superpowers/`, as Tasks 1
to 7 found.

What each commit touched, from `git show --name-only` and
`git diff --name-status ff03676bc..HEAD`.

`ff03676bc` modified `rexx-api`'s `src/values.rs`, `src/invoke.rs`,
`tests/values.rs`, `tests/invoke.rs` and `tests/context.rs`.

`08d232ecc`, `e427cf61e`, `6c5f4e8a8`, `3245708c0`, `8da8daecf` and
`f07592242` between them added
`rust/crates/rexx-exec/src/dispatch/library.rs`, `rust/corpus/phase-8.txt`,
nine `rust/corpus/lang/library_*.rex` with eight `.env` sidecars
(`library_requires_missing` needs none) and nine
`rust/crates/rexx-parse/tests/sourceline_oracle/library_*.txt`; and modified
`rexx-api`'s `src/load.rs` and `tests/load.rs`,
`rexx-core/src/body.rs`, `rexx-exec`'s `src/lib.rs`, `src/dispatch.rs`,
`src/dispatch/native.rs`, `src/dispatch/package.rs`, `src/error.rs`,
`src/run.rs` and `src/run/tests.rs`, `rexx-exec`'s `tests/corpus.rs`,
`tests/collect_stress.rs`, `tests/coverage.rs`, `tests/ir_recorded.rs`,
`tests/trace_oracle.rs` and `tests/support/arity.rs`, and
`rust/corpus/introspection-arity.tsv`, `rust/corpus/phase-4c.txt` and
`rust/corpus/README.md`.

---

## 1. Ruling 19: the shape the local-reference table took

```rust
// rexx-api/src/values.rs
pub trait Host {
    // ... the Task 5/6/7 methods, unchanged ...
    fn locals(&mut self) -> &mut Table;
}

pub struct Conversion<'a> {
    pub host: &'a mut dyn Host,
    pub strings: &'a mut CStringPool,
}
```

`Conversion::locals` is gone and every reader goes through `cx.host.locals()`.
No interior mutability was added anywhere: `Interp` answers
`&mut self.native_handles.last_mut().expect(..).locals`, which is a plain
field borrow of the same object the callback is already holding, so a
collection triggered from inside a callback reaches the table by the ordinary
route.

The change is mechanical everywhere else. The stand-in interpreters in
`src/invoke.rs`, `tests/invoke.rs`, `tests/values.rs` and `tests/context.rs`
each gained a `Table` field and the one-line accessor, and the `Conversion`
literals lost a field. No assertion changed:
`cargo test -p rexx-api` reported the same 105 `ok` result lines afterwards
that it reported before, with `context` still at 14.

**One thing ruling 19 does not fix.** `Activation::conversion` still takes a
`RefCell` for the length of one operation, so an extension re-entering the
interpreter while a conversion is in flight is still a panic. Ruling 19 was
about the *aliasing* of two `&mut Interp`, and that is what it removed.

---

## 2. How `impl Host for Interp` reaches the receiver and the scope

`Interp::native_handles` changed from `Vec<Table>` to `Vec<NativeFrame>`:

```rust
struct NativeFrame {
    owner: ObjRef,   // Interp::pool_owner of the receiver
    scope: ObjRef,   // Resolution::scope -- the running method's own scope
    locals: rexx_api::handles::Table,
}
```

`Interp::run_library_method` pushes one before the call and pops it after,
whatever the call did. `Host::cself`, `Host::set_object_variable` and
`Host::drop_object_variable` read the innermost frame; `Host::locals` answers
its table.

**The scope is the method's and not the receiver's class**, which is Task 7's
own oracle measurement (`NativeActivation.cpp:1878`) and is why the frame
carries `Resolution::scope` rather than deriving anything from the receiver.
`Interp::pool_owner` is the existing seam a generated `::ATTRIBUTE` accessor
already writes through, so a class receiver gets its class-variables object
exactly as it does for an accessor.

`Interp::object_roots` now walks each frame's `locals`, its `owner` and its
`scope`. The doc on the field says so, and
`a_native_activations_local_references_are_roots_only_while_it_lives` was
updated to build a frame.

The rest of the implementation:

| member | what `Interp` answers |
|---|---|
| `is_method` | `true`; every binding this task installs is a method |
| `string_value` | `Interp::required_string_value(..).ok()` |
| `string_bytes` | a small integer's and an inline string's rendering as `Cow::Owned`, a `Body::Text`'s and a `Body::Num`'s bytes borrowed, and `None` for everything else |
| `cself` | the frame's pool, at the frame's scope, name `CSELF`, unwrapped from the `.Pointer` body |
| `constants` | `.nil`, `counted(1)`, `counted(0)`, `text(b"")` |
| `set_object_variable` | `set_pool_variable`/`clear_pool_variable` on the frame's owner and scope, under `pool_variable_name` |
| `whole_number` | the small integer, or a text value where the number is too wide for one |
| `new_pointer` | `Body::pointer` with the class the registry answers for `"Pointer"`, pushed as a temp root |
| `locals` | the innermost frame's table |

`pool_variable_name` upcases and answers nothing for an empty name, one
holding `.`, or one starting with a digit; a write the interpreter's own
`getVariableRetriever` would refuse reaches the API only as "the write did
nothing", which is Task 7's finding restated in code.

**`Host::string_value` narrows a condition to a refusal.** The trait answers
`Option<ObjRef>` and `required_string_value` answers `Result`, so an object
whose `makeString` raises is reported as the conversion's own 88.909 rather
than as the condition it raised. Nothing in reach produces one -- `rxregexp`
converts strings the program already holds -- and widening the trait is a
change to Task 5's interface for a case no witness reaches. It is recorded
here rather than fixed.

---

## 3. The one resolution path

```rust
// crates/rexx-exec/src/lib.rs
pub(crate) enum LibraryLoad { Loaded(Rc<rexx_api::load::Library>), Missing, Version }

impl Interp {
    fn library_search_path(&self) -> Vec<PathBuf>;
    pub(crate) fn resolve_library(&mut self, name: &[u8]) -> LibraryLoad;
    fn require_library(&mut self, name: &[u8]) -> Result<Rc<Library>, Failure>;
    fn resolve_directive_library(&mut self, program: &Rc<Program>, directive: &Directive)
        -> Result<(), Failure>;
    fn library_binding(&mut self, external: Option<&MethodExternal>, key: &[u8])
        -> Option<LibraryBinding>;
}
```

`resolve_library` is `PackageManager::loadLibrary`
(`interpreter/package/PackageManager.cpp:229`). It holds its answer in
`Interp::libraries`, keyed by the name **byte for byte** -- measured, oracle,
`::method m external "LIBRARY RXREGEXP RegExp_Init"` is
`98.903 Unable to load library "RXREGEXP".` where the same directive spelling
the name in lower case loads. A miss is held as well as a hit.

The five entry points that reach it, and nothing else opens a library:

| site | how it uses the answer |
|---|---|
| `Interp::resolve_directive_library` | 98.903 / 98.982 / 90.998 / 90.999 at directive-install time |
| `Interp::library_binding` | the `InstallBody::Library` a dictionary key installs with |
| `Interp::load_required_packages` | `::REQUIRES ... LIBRARY`, raising the same 98.903 |
| `dispatch::native_load_external` | `.nil` for a miss, a `Method`/`Routine` object for a hit |
| `dispatch::package::load_library` | `1` or `0` |

Derived with `grep -rn 'resolve_library\|require_library' crates/*/src`. Its
output is those five sites, the two definitions with `require_library`'s own
call and the doc line naming it, and six lines inside
`dispatch::library::tests`.

The sites still naming Phase 8, from `grep -rn 'Phase 8' crates/*/src` at the
tip: in `rexx-api`, `layout.rs:381` (a context member no task has filled) and
`values.rs:166` (a conversion row Task 5 left unfilled); in `rexx-exec`,
`lib.rs:422` (`loadExternalRoutine` on the `REXX` package), `:560` (a call to
a library-backed `::ROUTINE EXTERNAL`), `:574` (a call to a routine a
`::REQUIRES ... LIBRARY` registered), `:836` (a `::ROUTINE EXTERNAL` naming
`REXX` or `REGISTERED`), `dispatch/library.rs:131` (an unfilled conversion
reaching the boundary), `dispatch/native.rs:836` (the `OPEN` list itself), and
`run/tests.rs:7100`, `:7384` and `:7417` (the expectations pinning three of
them). **None of them is `handle_set`.**

**The search path.** `rexx_api::load::open` grew a `search: &[PathBuf]`
parameter, tried before the undecorated `dlopen` and before `/usr/lib`.
`Interp::library_search_path` fills it from `LD_LIBRARY_PATH` **in the
interpreter's own environment**. The process loader reads that variable once
at start-up, so a value this interpreter was handed afterwards -- by
`Invocation::with_environment`, or by a corpus sidecar -- reaches the search
no other way, and writing it back to the process is forbidden here and would
reach every other interpreter in the process besides.
`a_search_directory_resolves_a_name_the_undecorated_attempt_misses` in
`rexx-api/tests/load.rs` carries its own control: the same name with an empty
search answers nothing.

**"Loaded once" is asserted on the side effect.**
`a_library_named_twice_is_opened_once` compares the two answers with
`Rc::ptr_eq`, because two `Loaded` answers look alike whether or not the
second re-opened the library. Its own control is a second `Interp`, which
opens its own and whose `Rc` is therefore not the first one's.

---

## 4. `handle_set`

**It never needed the loader.** What it needs is `from_raw_fd` to adopt a
descriptor the interpreter did not open, and D-U1 grants `unsafe` to
`rexx-api/src/ffi.rs` and `rexx-api/src/load.rs` alone. Nothing about a
`HANDLE:` stream reaches a shared library; it was parked on Phase 8 because
Phase 8 was the next open phase when the stream family was built, which the
`deferred` doc comment beside it already said.

Re-homed to **Phase 10**, which is the phase that owns RXAPI and the external
queues -- the rest of the surface that has to adopt descriptors this process
did not open, and therefore the phase that widens the same grant. `"Phase 8"`
stays in `every_deferred_entry_point_names_an_open_phase`'s `OPEN` list, as
the dispatch requires.

**That list is a whitelist, and after the move no entry point names Phase 8.**
Derived with
`grep -oE 'deferred\("[a-zA-Z_]+", Family::[A-Za-z]+, "Phase [0-9]+"\)' crates/rexx-exec/src/dispatch/native.rs | grep -oE '"Phase [0-9]+"' | sort | uniq -c`:
eleven rows name Phase 10 and five name Phase 6. The test asserts every
deferred row's owner is in the list, not that every list member is used, so
it stays green and the entry costs nothing; it is there because the surface
half of Phase 8 still owes work, which the sites listed in section 3 name.
`rexx-core/src/body.rs`'s `StreamState::handle` doc now names D-U1 as the
obstacle rather than "needs `unsafe`".

---

## 5. The witnesses, and the oracle transcript each reproduces

Every oracle run used the standard wrapper from a fresh empty directory made
with `mktemp -d` under the session scratchpad, three descriptors kept apart,
never `2>&1`. `rust/corpus/oracle-crashes.txt`'s entry titles were read before
running anything; none of these is one of them.

```
( cd "$D" && ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/$f.rex" ) \
  >"$D/$f.out" 2>"$D/$f.err"
```

### `lang/library_method_external.rex` -- rc 0

```
parse 0
pos 3
match 1
lastpos 3
reparse 0
match 1
minimal 0
find 3
findpos 5
```

stderr empty. A `::CLASS` bound to `librxregexp.so`'s `RegExp_Init`,
`RegExp_Uninit`, `RegExp_Parse`, `RegExp_Match` and `RegExp_Pos`, with one
written method exposing `!POS` to read back what the extension stored. It is the end-to-end witness: `CSELF` across
sends, `SetObjectVariable` into the method's own scope, `WholeNumberToObject`
for `!POS`, and `RegExp_Pos`, whose result is pointer arithmetic between an
advanced `StringData` pointer and a second `StringData` call on the same
object -- so `find 3` is wrong unless one object answers one address for the
whole call.

### `lang/library_requires_missing.rex` -- rc 158, stdout empty

```
     5 *-* ::requires 'zorkolib' LIBRARY
Error 98 running <path>/library_requires_missing.rex line 5:  Execution error.
Error 98.903:  Unable to load library "zorkolib".
```

### `lang/library_method_entry_missing.rex` -- rc 166, stdout empty

```
     5 *-* ::method x external "LIBRARY rxregexp NoSuchEntry"
Error 90 running <path>/library_method_entry_missing.rex line 5:  External name not found.
Error 90.998:  Unable to find external method "NoSuchEntry".
```

### `lang/library_routine_entry_missing.rex` -- rc 166, stdout empty

```
     5 *-* ::routine x external "LIBRARY rxregexp NoSuchRoutine"
Error 90 running <path>/library_routine_entry_missing.rex line 5:  External name not found.
Error 90.999:  Unable to find external routine "NoSuchRoutine".
```

### `lang/library_attribute_entry_missing.rex` -- rc 166, stdout empty

```
     5 *-* ::attribute at external "LIBRARY rxregexp NoSuchGet"
Error 90 running <path>/library_attribute_entry_missing.rex line 5:  External name not found.
Error 90.998:  Unable to find external method "GETNoSuchGet".
```

The getter's own prefixed spelling, which is what says the attribute shape's
accessor derivation runs before the library lookup and not after.

### `lang/library_loads_once.rex` -- rc 0

```
requires ran
load 1
again 1
absent 0
method Method
nomethod The NIL object
nolib The NIL object
noroutine The NIL object
```

stderr empty.

### `lang/library_method_missing_argument.rex` -- rc 168, stdout empty

```
       *-* Compiled method "DOPARSE" with scope "RE".
     5 *-* say r~doparse()
Error 88 running <path>/library_method_missing_argument.rex:  Invalid argument.
Error 88.901:  Missing argument; argument 1 is required.
```

No `line N` on the major line, which is what control D shows the crate
getting wrong before the `lineless` fix.

### `lang/library_method_extra_arguments.rex` -- rc 168, stdout empty

```
       *-* Compiled method "DOPARSE" with scope "RE".
     4 *-* say r~doparse('aab', 'MAXIMAL', 'extra')
Error 88 running <path>/library_method_extra_arguments.rex:  Invalid argument.
Error 88.922:  Too many arguments in invocation; 2 expected.
```

### `lang/library_method_raises.rex` -- rc 218, stdout empty

```
       *-* Compiled method "INIT" with scope "RE".
       *-* Compiled method "NEW" with scope "Object".
     5 *-* r = .Re~new('[')
Error 38 running <path>/library_method_raises.rex line 5:  Invalid template or pattern.
```

`RaiseException0(Rexx_Error_Invalid_template)` is 38000
(`api/oorexxerrors.h:362`), which the caller's frame raises as 38.0 once the
call has returned. The extension runs on after the raise and stores its
`CSELF`, and the raise still wins.

**The `.env` sidecars.** The eight witnesses that name `rxregexp` carry
`LD_LIBRARY_PATH={oraclelib}`; `library_requires_missing.rex` names a library
nothing has and needs none. `{oraclelib}` is a new substitution in
`resolved_environment`, beside `{run}`, resolving to
`support::oracle::oracle_root().join("lib")` -- the same directory
`Oracle::wrapped` already puts on the spawned side, so the oracle's answer is
unchanged and the in-process side is given what the process loader could not
be told later.

That made `a_sidecar_changes_what_the_oracle_answers` red, correctly: the
sidecar is inert on the oracle and load-bearing here. The control was
generalised and renamed to
`a_sidecar_changes_what_one_of_the_interpreters_answers` -- where the oracle's
two runs agree, it now runs the in-process interpreter with the sidecar and
without and requires a difference there. It passes, so each of the eight is
load-bearing on this side; a run without the variable cannot load the library
at all and answers 98.903 where it should answer 90.998, or refuses where it
should run.

---

## 6. What the dispatch and the brief got wrong

1. **`loadExternalMethod` and `Package~loadLibrary` are not 98.978.** Measured
   2026-09-14 against the oracle, all at rc 0:
   * `.Method~loadExternalMethod('x', 'LIBRARY zorkolib RegExp_Parse')` and
     `.Method~loadExternalMethod('x', 'LIBRARY rxregexp NoSuchEntry')` both
     answer `The NIL object`;
     `.Method~loadExternalMethod('x', 'LIBRARY rxregexp RegExp_Parse')`
     answers a `Method`.
   * `.Routine~loadExternalRoutine` answers `The NIL object` for both misses
     and a `Routine` for `'LIBRARY rxmath RxCalcPi'`.
   * `.context~package~loadLibrary('rxregexp')` answers `1` and the same call
     naming `zorkolib` answers `0`.
   * `.Object~package~loadLibrary(...)` is **98.984**, "User additions are not
     allowed to the REXX package", for both a real and a missing library --
     the receiver check, not the load.

   `PackageManager.cpp:947` and `:968` are `resolveMethodEntry` and
   `resolveRoutineEntry`, which the file's own comments describe as "used on a
   restore or reflatten". Neither surface reaches them. **98.978 is raised
   nowhere this task can reach**, and no code here builds it.

2. **98.982 is carried but unwitnessed.** `LibraryLoad::Version` exists and
   `Raised::library_version` builds the condition, but `librxregexp.so` asks
   for 4.0.0 and this interpreter is 5.3.0, so no run produces it. Task 2's
   `a_package_asking_for_a_newer_interpreter_is_refused` covers
   `load::check_version` on a synthetic entry; the raise on top of it is not
   covered, and writing a witness needs an extension built to ask for a newer
   interpreter, which the read-only constraint forbids.

3. **The `98.903` for `::REQUIRES` upcases an unquoted name.** Measured,
   oracle, `::requires zzznolib library` is
   `Unable to load library "ZZZNOLIB".` while the same name inside an
   `EXTERNAL` string keeps its case. That is the scanner's, not the loader's,
   and the corpus witness quotes the name so the two are not conflated.

4. **The brief's file list misses several.** `rexx-api/src/load.rs` needed the
   search parameter and `Library::routine`; `rexx-exec/src/error.rs` needed
   the constructors for 98.903, 98.982, 90.999, 88.901 and 93.968;
   `src/run.rs` needed the refusal for a called library-backed routine;
   `src/dispatch/library.rs` is new; and the harnesses `corpus.rs`,
   `collect_stress.rs`, `coverage.rs`, `ir_recorded.rs` and `trace_oracle.rs`
   each had to learn `phase-8.txt`, because each asserts it reads every subset
   file on disk. `rexx-core/src/body.rs` and `tests/support/arity.rs` are the
   rest of the modified set, given above.

5. **`::ROUTINE EXTERNAL` is only half wired, deliberately.** The library and
   the procedure resolve at install, so `98.903` and `90.999` are the oracle's
   and both are witnessed. **Calling one refuses loudly**, naming Phase 8: a
   routine runs through `RexxRoutineEntry` and a `RexxCallContext`, which is a
   second two-call protocol Tasks 6 and 7 did not build.
   `a_library_backed_routine_installs_and_refuses_when_it_is_called` pins both
   halves. `::ROUTINE EXTERNAL` naming the `REXX` package still refuses at
   install, because the table it resolves against is `rexx_routines[]`
   (`runtime/InternalPackage.cpp:230`) and not the method registry -- measured,
   oracle, `::routine r external "LIBRARY REXX Filespec"` is rc 0 and the
   routine runs, so answering 43.1 for it would be a wrong answer.

6. **`::REQUIRES ... LIBRARY` runs no routine, and the first draft answered
   43.1 for one.** The oracle's `::requires 'rxmath' LIBRARY` makes
   `RxCalcPi()` answer `3.14159265` at rc 0. At `08d232ecc` ours loaded the
   library and stopped: run through `target/release/rexx-run`, the same
   program was `Error 43.1: Could not find routine "RXCALCPI".` at rc 213 --
   a wrong answer a program cannot tell from its own typo, and worse than the
   loud refusal the directive gave before this task touched it.

   `3245708c0` closes it. A successful `LIBRARY` requires records the routine
   names the library exports, and a call reaching one refuses loudly naming
   the routine, the library and Phase 8. The check sits after the internal
   packages and before the external file search, so a name both an internal
   package and the library export still runs the one this crate implements.
   `a_routine_a_required_library_exports_refuses_rather_than_answering_43_1`
   asserts the adjacent pair together: `zorkolo()` in the same program is
   still 43.1, so the refusal is about the library's own table and not about
   every unresolved name in a program that requires a library. Running the
   routine is still item 5's work.

7. **`Loud::external_entry_point` did not disappear**, as "delete the
   refusals" implies. It now covers `loadExternalRoutine` on the `REXX`
   package, which is item 5's, and the unreachable case of a library procedure
   that resolved at install and no longer does.

8. **`staged_gap` is gone.** With `::REQUIRES ... LIBRARY` no longer a gap, the
   `::REQUIRES`-stage call could never fire, and a walk that cannot answer is
   worse than no walk. Its measured stage-order table moved to a comment above
   `directive_gap`, which is what it documents.

---

## 7. Two derived tables moved, and one was already red

**`corpus/introspection-arity.tsv`.** Three rows moved from `send-differs` to
`agree`: `Method~loadExternalMethod`, `Package~loadLibrary` and
`Routine~loadExternalRoutine`. Refreshed with
`REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity`,
and the diff is those three rows and nothing else.

Two of the three needed a harness change first.
`support::arity::measured` spawns the crate's release binary and gives the
oracle child `LD_LIBRARY_PATH` through `oracle_command`, so a row whose method
loads one of the oracle's extensions was measuring the difference between a
child that was told where the libraries are and one that was not:
`Package~loadLibrary('rxmath')` answered `1` on one side and `0` on the other.
The crate's child is now given the same variable, which the phase's own
constraint names as the permitted way ("sets the variable on a child it
spawns"). Without that it would have been a silent wrong answer where a loud
refusal stood before.

**`corpus/refusal-sites.tsv` was already red and stays red.**
`the_table_holds_every_constructor_the_source_defines` disagrees with the
source on both directions of the set, which Task 7's report already recorded
("disagrees on every row"); the table was last written at `bacd0bae5` and the
line numbers have moved since. This task adds constructors to the
"in the source, not the table" side and so makes that list longer. **Not
re-derived here**: the table's last four columns are hand-measured probe
results that a re-derivation from source cannot produce, and joining them by
name with a scanner that is not the test's own would commit an artifact
derived by a different rule than the one that checks it.

`a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not`
in the same file **did** go red and was fixed rather than recorded. Its
`external_entry_point` row is witnessed by
`.Routine~loadExternalRoutine('Filespec', 'LIBRARY REXX')`, which the first
draft of `native_load_external` answered `.nil` for -- a silent wrong answer
where the oracle answers a `Routine`. That path refuses loudly again, and the
row's `answer` column is a substring of the refusal's own construction site,
which is what the test requires.

---

## 8. Controls

Predictions were written to `scratchpad/predictions-task8.md`, the first block
at 05:01:32 and the revised baselines with controls D and E at 05:18:22, each
before the control it describes ran. Baselines at `e427cf61e`:
`cargo test -p rexx-exec --lib` 809 passed 3 failed (the pre-existing
`ir::drive::tests` three), and the corpus gate 525 of 525 matching.

Each control was one edit undone by re-editing. For A to E, `sha256sum -c`
against the hashes taken beforehand reports `OK` for `dispatch.rs`, `lib.rs`,
`error.rs` and `dispatch/library.rs`, and `git status --short` was empty
afterwards. F edited `lib.rs` alone, whose sha256 is the same before and
after (`1b7b37a2...`); the `dispatch/library.rs` doc rewrite F prompted is
`f07592242` and deliberate.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| A | `Invocable::Library`'s arm refuses instead of calling `run_library_method` | 524 of 525, `library_method_external.rex` the one mismatch; `--lib` unchanged | 521 of 525: that program **and** `library_method_missing_argument`, `library_method_extra_arguments` and `library_method_raises`, each `rust rc 120` against the oracle's 0, 168, 168 and 218 | failing set **falsified in extent**, the named member **confirmed** |
| B | `resolve_library` drops its cache lookup | `a_library_named_twice_is_opened_once` alone fails, `--lib` 808/4; corpus unchanged at 525 | exactly that, and 525 of 525 | **confirmed**, both parts |
| C | `resolve_directive_library` returns before it looks anything up | 521 of 525, the four load-failure witnesses; three named `--lib` tests fail | 522 of 525 -- `library_method_entry_missing`, `library_routine_entry_missing` and `library_attribute_entry_missing`, **not** `library_requires_missing`; `--lib` 806/6, exactly the three named | corpus extent **falsified**, `--lib` set **confirmed** |
| D | `missing_native_argument` no longer sets `lineless` | 524 of 525, `library_method_missing_argument.rex` alone, on stderr alone; `--lib` unchanged at 809/3 | exactly that, `stderr differ` | **confirmed**, all three parts |
| E | `run_library_method` drops the condition the extension raised | 524 of 525, `library_method_raises.rex` alone | exactly that, `stdout, stderr, exit code differ` | **confirmed** |
| F | `Interp::object_roots` stops extending from `native_handles` | `a_library_call_answers_the_same_under_a_collection_at_every_allocation` **still passes**, because the receiver is a send temp and `new_pointer` pushes a temp | exactly that, and the failing set grows by `a_native_activations_local_references_are_roots_only_while_it_lives` alone: `--lib` 810/4 | **confirmed**, and it is the finding below |

**A's prediction was written for a corpus that had three fewer witnesses.**
The revised-baseline note updated the count and not the set, which is the
"correction rounds add false statements" shape: the three witnesses added at
`e427cf61e` all send to a library-bound method, so they belong in A's failing
set and the note should have said so.

**C is the one that found something.** `library_requires_missing.rex` stayed
green because `::REQUIRES ... LIBRARY` reaches `require_library` through
`Interp::load_required_packages` and not through
`resolve_directive_library` -- two of the five call sites in section 3, which
the prediction collapsed into one. The behaviour is right and the prediction
was not.

**No control could redden by aborting.** Every one of them is observed from
the corpus differential or from a test assertion after the call has returned,
which is the shape Task 7's fix round asked for: a panic inside a host
callback crosses an `extern "C"` frame and aborts the whole binary.

**F is the one that stopped a test claiming more than it holds.** The
collect-on-every-allocation harness reads `phase-8.txt` last and aborts on a
panic that predates this phase, so none of the new corpus programs reaches it;
`f07592242` runs the end-to-end witness through
`run_program_collect_every_alloc` directly instead. It was written as "the
native frame is rooted", and F says it is not that: with the frames dropped
from `object_roots` entirely it stays green, because the receiver is rooted by
the send's own temps and `Host::new_pointer` pushes what it mints as a temp
before handing it back, so neither object the frame is meant to hold is
reachable only through it. Its doc now says what it does hold -- the call is
answer-stable under collection -- and names
`a_native_activations_local_references_are_roots_only_while_it_lives` as the
test that does redden under F. The prediction for F was written before it ran
and is the outcome.

**A to E were run at `e427cf61e` and not re-run at the tip.** `6c5f4e8a8`
touches comments only. `3245708c0` adds a refusal whose control is inside its
own test rather than a separate edit:
`a_routine_a_required_library_exports_refuses_rather_than_answering_43_1`
asserts `zorkolo()` in the same program is still 43.1 at rc 213, so a version
that refused every unresolved name in a program requiring a library would
fail it. What that pair does **not** cover is the position of the check:
placed before `crate::internal_routines::lookup` instead of after it, a name
both an internal package and the library export would refuse where it now
runs, and no witness in reach names such a name.

---

## 9. Commands and exit statuses

Every status read unpiped, from `rust/`. The table is the run at
`3245708c0`; `fmt`, `clippy`, `--lib` and the corpus gate were re-run at the
tip `f07592242` and answer the same. The two commits after `3245708c0` add a
README paragraph, an early return on a lookup that cannot hit, and one test.

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |
| `cargo test -p rexx-exec --lib` | 101 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 |
| `cargo test -p rexx-exec --no-fail-fast` | 101 |
| `cargo test -p rexx-parse` | 0 |
| `REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity` | 0 |

* `cargo test -p rexx-api` reports **106** `ok` result lines, one more than
  Task 7's 105: the new one is
  `a_search_directory_resolves_a_name_the_undecorated_attempt_misses`.
  `context` is still at 14, `values` at 36, `invoke` at 10.
* `cargo test -p rexx-core --test unsafe_sites` reports 2 passed, so the
  granted set and the using set are both unchanged. **No `unsafe` was added
  anywhere**, and unlike Task 7's unchanged result that is not merely the
  test's file granularity speaking:
  `git diff ff03676bc^..HEAD -- rust/crates | grep '^+' | grep unsafe`
  answers one line, a doc comment in `rexx-core/src/body.rs` naming D-U1.
* `cargo test -p rexx-exec --lib` reports **811 passed, 3 failed**, the three
  pre-existing `ir::drive::tests` cases. The baseline before this task was
  803 with the same three, and the new ones are
  `a_library_call_answers_the_same_under_a_collection_at_every_allocation`,
  `a_directive_naming_a_library_that_is_not_there_is_98_903`,
  `a_library_backed_routine_installs_and_refuses_when_it_is_called`,
  `a_routine_a_required_library_exports_refuses_rather_than_answering_43_1`
  and the four in `dispatch::library::tests`.
* The corpus gate reports **525 of 525 matching**, 29 passed 0 failed 1
  ignored. The subset was 516 before this task and nine programs were added.
* `cargo test -p rexx-exec --no-fail-fast` fails in three binaries and **all
  three sets are the ones Task 7 recorded as pre-existing**: `--lib`'s three,
  `collect_stress`'s `a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`
  and `the_l0_subset_passes_again_under_collect_on_every_allocation`, and
  `refusal_sites`'s `the_table_holds_every_constructor_the_source_defines`.
  1476 `ok` result lines. Note that `collect_stress` now runs the nine new
  programs too, and its failing set did not grow.
* `cargo test -p rexx-parse` covers
  `sourceline_matches_the_interpreter_for_every_corpus_program`, which is what
  the nine new `sourceline_oracle` expectations are read by.
* The `--no-fail-fast` run was taken at `3245708c0`. An earlier one at
  `08d232ecc` gave the same three binaries and 1475 `ok` lines, one fewer
  because `a_routine_a_required_library_exports_refuses_rather_than_answering_43_1`
  did not exist yet.
* `REXX_INTROSPECTION_ARITY_REFRESH=1` was run at `08d232ecc`, before the
  three later commits; `introspection_arity` without the refresh is green in
  the `--no-fail-fast` run above, which is what says the table still matches.

---

## 10. Open, for whoever comes next

1. **The routine half of the two-call protocol.** `::ROUTINE EXTERNAL` naming
   a library installs and refuses at the call; a routine a
   `::REQUIRES ... LIBRARY` registered refuses at the call;
   `::ROUTINE EXTERNAL "LIBRARY REXX ..."` refuses at install. All three want
   `RexxRoutineEntry`, a `RexxCallContext_` and an `invoke::routine`, none of
   which Tasks 6 and 7 built. Phase 8's surface half owes them and the
   refusals name it.

2. **A `Routine` object `loadExternalRoutine` answered is not callable.** The
   object exists and `~class~id` answers `Routine`, which is what the oracle
   answers and what `introspection-arity.tsv` now records as agreeing.
   Sending it `~call` is item 1's.

3. **Nothing unloads a library.** `Interp::libraries` holds an `Rc<Library>`
   for the interpreter's whole life, so the constraint "a loaded library is
   not unloaded while anything it produced is reachable" holds trivially and
   for the wrong reason. Task 9's `UNINIT` is where the ordering becomes real.

4. **`RegExp_Uninit` never runs.** `library_method_external.rex` matches the
   oracle because the extension prints nothing when it frees an automaton, so
   the leak is invisible; that is Task 9's whole subject.

5. **98.982 is unwitnessed**, section 6 item 2.

6. **`Host::string_value` narrows a raised condition to 88.909**, section 2.

7. **`corpus/refusal-sites.tsv` is stale and got staler**, section 7.

8. **`Failure::Signature` maps to 93.968 and never to 40.918.**
   `Failure::error_number` takes an `is_method` flag that
   `Interp::run_library_method` does not consult, because every binding this
   task installs is a method. Whoever builds item 1 owes the call spelling.
