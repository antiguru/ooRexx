# Re-review of the residual round (cf92ff4fb..233d2766d)

Reviewer copy: `git archive 233d2766d rust` extracted to
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/residual-rereview/tree/`,
built with `CARGO_TARGET_DIR` under the same scratch. Worktree untouched.

Written first, appended as work proceeds. Every item below is marked run or inferred.

## Status of this report

Complete. Counts: Critical 0, Important 1, Minor 3. Sections: 0 setup; 1 X1 (priority 1);
2 X2/X3/X4/Y1 probes and the hunt past them (priority 2); 3 X5's instruments (priority 3);
4 prose against code and runs (priority 4); 5 per original finding; 6 findings by severity;
Not reached. Sections stand in the order the work ran, so 2.4 (results) follows 3.2
(predictions); every prediction block precedes its results block in the file.

## 0. Setup (run)

`git archive 233d2766d rust` extracted to `residual-rereview/tree/rust`, the repository's read-only
directories symlinked beside it, `CARGO_TARGET_DIR=residual-rereview/target`, `-j 4`, `--locked`.
`cargo build --release -p rexx-exec` exit 0, `target/release/rexx-run` sha256 prefix
`4d9785fdabb027cb` (`residual-rereview/build-release.txt`). `cargo test -p rexx-api --no-fail-fast`
exit 0: lib 18, context 14, handles 5, invoke 9, layout 17, load 10, values 38, doctests 1 + 4
(`residual-rereview/api-tests.txt`). Worktree HEAD `233d2766d`, `git status` clean at the start
(the gitStatus snapshot); nothing in the worktree is written but this file.

## 1. X1 (`69a579370`): safe paths to UB through `rexx-api` (priority 1)

### 1.1 Reading (before any run)

Every function member of every `interface!` table is typed `unsafe extern "C" fn` by
`entry_type!` (`layout.rs:388-398`), so `METHOD_CONTEXT` (`ffi.rs:115`),
`METHOD_CONTEXT_INTERFACE` (`layout.rs:726`), `RexxThreadInterface::REFUSING` and
`MethodContextInterface::REFUSING` (the `populated` tables) and any table a program builds by hand
all need `unsafe` to call a slot. The `entry_stub!` stubs are safe `extern "C" fn` items coerced
into the unsafe slots, and are local to a block inside a `const`, so nothing can name them.
`owner_of` and `value_of` are `pub unsafe fn`. `MethodContext` has private fields, `as_ptr` is
`pub(crate)`, `bare` is `cfg(test)`; the only public constructor is `Contexts::method(&mut self)`,
whose answer borrows the `Contexts` mutably for its lifetime (`PhantomData<&'a mut _>` bound to
`&mut self`), and `Contexts<'a, 'h>` carries `PhantomData<&'a Activation<'h>>`, so neither the
wrapper nor the activation can go away under a context. `NativeMethodEntry` and
`NativeRoutineEntry` keep `entry_point` private, `Library`'s fields are private and only
`open`/`open_path` build one, so the only stub `invoke::method` can reach is one a `dlopen`ed
library published. Every other `pub fn` that takes a raw pointer (`Table::resolve`/`remove`,
`Activation::set_object_variable`/`string_data`/`string_length`/`new_pointer`,
`CStringPool::bytes_at`) compares or stores the address and dereferences nothing; `check_version`
reads one integer field of a struct the caller owns. `activation_of` forms only shared references
from `owner`, which `Contexts::new` took from a shared reference, so a `Contexts` built over one
activation and an `invoke::method` handed another aliases nothing (it would be a semantic mix-up,
not UB). I did not find a safe path. The probes below are what that reading is checked against.

**Predictions (written before the runs).**
* P1: the boundary re-review's `zz_bare_context_probe.rs` copied verbatim into my tree,
  `cargo test -p rexx-api --test zz_bare_context_probe --no-run`: exit 101, `error[E0133]` at
  the probe's two call sites (`:18:5` and `:34:5`), no other error.
* P2: my own `zz_neighbours_probe.rs` under `#![forbid(unsafe_code)]`, one `fn` per shape:
  (a) `(METHOD_CONTEXT_INTERFACE.GetSelf)(ptr)`; (b) `(RexxThreadInterface::REFUSING.HaltThread)(null)`;
  (c) a `MethodContextInterface` copied out of `METHOD_CONTEXT` by value and one slot called;
  (d) `let f: extern "C" fn(..) = METHOD_CONTEXT.SetObjectVariable;` (a safe fn-pointer binding);
  (e) `MethodContext { pointer: .., wrapper: .. }` built by hand; (f) `NativeMethodEntry { .. }`
  built by hand; (g) `row.entry_point` read off a `Library::method` answer; (h) a call to a
  `RexxPackageEntry.loader` the program sets to a safe `extern "C" fn`. Expected: nothing
  compiles; (a)(b)(c)(h) E0133, (d) E0308 (mismatched types, an unsafe fn pointer is not a safe
  one), (e) E0451 (private field) or E0423, (f) E0451, (g) E0616.
* P3: slice A's `zz_offsets_probe.rs` copied into my tree, run with `--nocapture`: its printed
  lines byte-identical to `final-a/offsets.cpp.txt` (111 lines); a fresh run of the compiled
  `final-a/offsets` also byte-identical to that file.
* P4: `cargo test -p rexx-core --test unsafe_sites` in my tree: 2 passed.

**Results (run; outputs under `residual-rereview/x1/`, statuses in `x1/status.txt`).**
* P1 exit 101: `error[E0133]` at `zz_bare_context_probe.rs:18:5` and `:34:5`, nothing else.
  Confirmed.
* P2, first as one file per group: `zz_neighbours_unsafe_probe.rs` exit 101, four `E0133` at
  (a) `:22:13`, (b) `:28:5`, (c) `:39:5`, (h) `:58:5`, nothing else. Confirmed.
  `zz_neighbours_priv_probe.rs` exit 101: (d) `E0308` at `:12:9`, (g) `E0616` at `:48:60`, and
  **no error for (e) or (f)**: rustc stopped at its type errors before the privacy pass that
  reports a private field in a struct literal, so those two were unwitnessed by that file.
  Re-run alone (`zz_neighbours_ef_probe.rs`): exit 101, `E0451` "fields `pointer` and `wrapper`
  of struct `MethodContext` are private" at `:17:9` and `E0451` "field `entry_point` of struct
  `NativeMethodEntry` is private" at `:29:9`. Confirmed in outcome; the first prediction's
  "one file" was wrong in mechanism, corrected by the split.
* P3 exit 0: the probe's 111 printed lines `diff` empty against `final-a/offsets.cpp.txt`, and a
  fresh run of `final-a/offsets` `diff` empty against the same file. Confirmed: the ABI slice A
  measured is unchanged.
* P4 exit 0: 2 passed. Confirmed.
* The probe files were removed from my tree afterwards (`tests/` lists the six committed files).

So: no safe path to `owner_of`, to a stub, or to a table slot was found by reading, and every
shape I could write under `#![forbid(unsafe_code)]` is refused at the type or privacy level.

### 1.2 The `SAFETY:` notes X1 added or changed (read against the code)

| where | invariant named | who establishes it | holds? |
|---|---|---|---|
| `set_object_variable` `# Safety` (`ffi.rs:255-257`) and its note (`:263-265`) | the context is one a live `Contexts` handed out, used during that call; `name` is NUL-terminated; `invoke::method` holds no conversion state across the call | the C caller, which is the stub `invoke::method` called; `invoke::method` takes `cx.conversion()` per argument and drops it before `entry.call` (`invoke.rs:84`, `:94`, `:100`) | yes; the conversion-state clause guards a `RefCell` panic (an abort in an `extern "C"` frame), not UB |
| `drop_object_variable`, the five thread callbacks (`:273-331`) | as above, the thread context of a live `Contexts` | `Contexts::method` writes `threadContext` from the whole thread wrapper (`ffi.rs:207`) | yes; section 3 mutates that line |
| `dropping_stub`'s new note (`:424-426`) | the context is one a `Contexts` handed this call, the name a literal | the only test using it builds `Contexts::new(&activation)` (`invoke.rs:532-533`) | yes |
| `an_unbuilt_entry_refuses_loudly` (`:515-516`) | the stub reads none of its arguments | `entry_stub!` binds every parameter to `_` and calls `refuse` (`layout.rs:405-416`) | yes |
| `owner_of`'s `# Safety` (`:33-39`, unchanged text) | "the guarantee is the interpreter's, which mints every context it hands out and never hands out one it did not build" | after X1 this is true as a description of who can reach it: every caller is now `unsafe` | yes, now |

Commit message `69a579370` checked: "aborts on stable at `ffi.rs:225`" is the boundary re-review's
measurement (P1/P2 there); the layout claim is P3 here; the doctest claim is the report's C2/C3/C3b,
whose saved outputs (`residual-fix/x1-C2.txt`, `x1-C3.txt`, `x1-C3b.txt`) say what the report says
(stable: `METHOD_CONTEXT (line 105) - compile fail ... FAILED` with the mutant; nightly: 4 passed
as written, and `Some expected error codes were not found: ["E0425"]` with the code changed). Read,
not re-run.

## 2. X2, X3, X4, Y1: the original probes and the hunt past them (priority 2)

### 2.1 The original probes (run, `residual-rereview/cmp.sh` and `cmpsep.sh`, my build `4d9785fdabb027cb`)

`f4b`, `f4a3`, `f4e`, `pk-retry`, `x4-dup`, `x4-entry`, `x4-req`, `r7-routine`: SAME x3 each.
`f5-t3` (cmpsep): SAME x3 (`first raised 98.903` / `second loaded` / `k class The K class` / `k 0`).
`f5-t2` through the scratch-only `final-fix/ext/libforgever.so`: SAME x3. `r7-unbound`: lines 1-3
SAME (`package of m The NIL object` / `prolog ran` / `answered The Package class x`), then the
crate stops at `~routines~items` with rc 120 `method "ITEMS" of class "StringTable" is not
implemented (Phase 5)` where the oracle prints `routines 0`: the pre-existing loud refusal the
report's Y1 section already records for the base, not this round's.

### 2.2 The committed witnesses, replayed on the before-binaries and the saved mutants (run)

The four new corpus programs copied with their `.d/` fixtures into `residual-rereview/src/w-*`.
* X2 `library_routine_package`: HEAD SAME x3; `bins/rexx-run-x1` (before) DIFF on lines 2, 4, 6;
  `bins/rexx-run-MX2a` DIFF on 2, 4, 6; `bins/rexx-run-MX2b` DIFF on line 4 alone. The commit
  message's "differed on three before", "exactly those three lines", "exactly the second
  binder's line" hold.
* X3 `library_bound_before_refusal`: HEAD SAME x3; `bins/rexx-run-x2` DIFF on lines 2, 3, 4 and
  SAME on line 5. Holds.
* X4 `library_package_retried` (cmpsep): HEAD SAME x3; `bins/rexx-run-x3` DIFF on lines 2, 8, 9,
  stderr and rc 159; `bins/rexx-run-MX4b` DIFF on line 5 alone (`its class The K3 class`). Holds.
* Y1 `library_context_package`: HEAD SAME x3; `bins/rexx-run-x5` stops at line 8 with 93.953,
  rc 163; `bins/rexx-run-MY1` SAME through line 8, then 43.1 at `call pkroutine`, rc 213, the last
  three lines lost. Holds.

### 2.3 Past the witnesses: predictions (written before the runs)

Sources under `residual-rereview/src/<name>/`; runs under `p/<name>`.
* `x2-lower`: `pk.cls` binds `rxcalcsqrt` in lower case; the program asks `loadExternalRoutine`
  before and after under three spellings and calls the binder's routine. Both sides: `early nil` /
  `binder pk.cls` / `loaded exact pk.cls` / `loaded upper pk.cls` / `loaded lower pk.cls` /
  `call 5`, rc 0.
* `x34-routine` (cmpsep): `pk.cls` binds `RxCalcSqrt`, declares `::class K`, then names
  `LIBRARY yyregexp`, which is missing on the first ask (X3 binds the routine, X4 then drops the
  package). Both sides: `first raised 98.903` / `routine after dropped binder pk.cls` (bound to a
  package neither side keeps) / `again loaded` / `binder routine pk.cls` / `loaded routine pk.cls`
  / `call 4` / `class The K class`.
* `x34-method`: `pk.cls` has a routine, a class, a bound method and a missing-entry method, loaded
  twice (both fail); an `early` `loadExternalMethod` answer reports `pk.cls` after each, and is
  then the context of a `.Package~new` whose prolog resolves nothing. Both sides: `first raised
  90.998` / `method after dropped binder pk.cls` / `again raised 90.998` / `method after second
  drop pk.cls` / `prolog under a dropped binder` / `package answered The Package class`.
* `x34-method-lookup`: the same, then a `.Package~new` whose prolog is `call pkr`, where `pkr`
  is a routine the dropped `pk.cls` declared before its failing directive. I cannot predict the
  oracle from the C++ without reading where a translating package's routine table is attached; the
  candidates are `pkr ran` (the table is live on the discarded package) or 43.1, or a crash. The
  crate: unknown, this is the probe.
* `x4-two-requirers`: `a.cls` and `b.cls` each `::requires 'bad.cls'`, whose translation fails on a
  duplicate `::routine`; then the program rewrites `bad.cls` through a stream and asks again. Both
  sides: `a first raised 99.903` / `b first raised 99.903` / `bad direct raised 99.903` /
  `bad again loaded` / `kb class The KB class` / `a again loaded` / `b again loaded`, rc 0. The
  crate side is the open question: whether the retry re-reads the file rather than a parse it kept.
* `x4-prolog`: a package whose prolog raises 42.3 after its class installed, asked twice. Both
  sides: `prolog runs` / `first raised 42.3` / `second loaded` / `k class The K class` (the
  `load_requires` comment's "or in its prologue stays cached").
* `y1-more` (a shape not in the report): an unbound method as context of a `.Package~new` whose
  prolog calls a routine the same source defines, then `findRoutine` of it, `name`, `sourceSize`.
  Oracle: `prolog ran` / `r ran` / `findRoutine The Routine class` / `name x` / `sourceSize 4`,
  rc 0 (a local hit returns before the parent is consulted). Crate SAME x3.
* `y1-class-parent`: a bound method as context, the prolog resolving a class through the parent
  (`say .Re`); the same with `.context~package` as context resolving the caller's own `::class K`;
  the same with the loaded package as context; and `Routine~newFile` over a file saying `.Re`.
  Oracle: `bound pk.cls` / `The Re class` / `answered The Package class` / `The K class` /
  `The Re class` / `The Re class` / `newFile 1`. Crate: `run.rs:3842` is the only reader of
  `package_parents`, so I predict the crate walks parents for routines only and prints `.RE`,
  `.K`, `.RE`, `.RE` on those four lines. Whether that predates the round is checked on
  `final-b/target-head/release/rexx-run` (`e64202ae7`).

## 3. X5's instruments (priority 3)

### 3.1 Miri over the thread-table test: predictions (written before the runs)

Toolchain: the previous implementer's scratch `RUSTUP_HOME` (`final-fix/rustup-home`, read only),
a copy of `residual-fix/xdg-cache` as my `XDG_CACHE_HOME`, `cargo +nightly miri test --offline -j 4`,
Stacked Borrows (no flags). Trees under `residual-rereview/miri/`: `base` (`git archive 233d2766d`),
`m-method` (one line, `ffi.rs:210`: the method context pointer taken from `self.method.context`
rather than the whole `self.method` wrapper, the method-side twin of the report's M5b and of
pre-F8 A4), `m5b` (one line, `ffi.rs:207`: the thread context pointer taken from
`self.thread.context`, the report's M5b replayed). `diff` against `base` shown: one line each.
* B1 `base`, `-p rexx-api --lib`: 16 passed, 0 failed, 2 ignored (`an_unbuilt_entry_refuses_loudly`,
  `a_routine_is_found_by_its_exact_spelling_before_its_case`), exit 0.
* B2 `m-method`, narrowed to `a_stub_reaches_its_activation_through_the_thread_context`: exit 1,
  `Undefined Behavior: attempting a read access using <tag> at alloc[0x18], but that tag does not
  exist in the borrow stack for this location` at `ffi.rs:46:14`, the tag created by a
  SharedReadWrite retag at offsets `[0x0..0x18]` at `ffi.rs:210`; backtrace
  `owner_of::<RexxMethodContext_, Activation>` <- `activation_of` <- `set_object_variable` <-
  `thread_table_stub`: the five thread callbacks the stub calls first succeed (the thread pointer
  is still the whole wrapper) and the first `SetObjectVariable` through the method table is the
  read Miri reports. Under stable the same mutant passes (the bytes are where the read lands).
* B3 `m5b`, the same narrowing: exit 1, the same report at `ffi.rs:46:14` with the retag at
  `ffi.rs:207` over `[0x20..0x30]` (the thread context sits after the 24-byte method context and
  its owner inside `Contexts`, as the report's M5b showed) and backtrace
  `owner_of::<RexxThreadContext_, Activation>` <- `activation_of` <- `string_length` <-
  `thread_table_stub`.

### 3.2 The per-variable sidecar control and `ir_recorded`'s assertion: predictions (written before the runs)

Both in my tree, my stable target, corpus files and `ir_recorded.rs` edited in the copy only and
restored from `git show 233d2766d:<path>` with `cmp` afterwards. Outputs under `residual-rereview/x5/`.
* S0, pristine, `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus -- sidecar`:
  `a_sidecar_changes_what_one_of_the_interpreters_answers ... ok` and
  `every_sidecar_names_a_program_the_subset_runs ... ok`, 2 run.
* S1, a live variable's line put back as inert elsewhere: `LD_LIBRARY_PATH={oraclelib}` (live in
  `library_routine_package.env`) appended to `lang/external_trace.env`, whose program loads no
  library: the control FAILED at `corpus.rs:733` naming `lang/external_trace.rex` and
  `Variable("LD_LIBRARY_PATH")`; the other test ok.
* I0, pristine, `cargo test --release -p rexx-exec --test ir_recorded`: 28 passed, exit 0.
* I1, `compare`'s sidecar replaced by `Sidecar::default()` (the report's C-R10b shape on the
  current tree): `every_population_runs_without_a_refusal` FAILED, `11 of 10884 programs did not
  run cleanly` (the corpus is one program larger than at X5), every line `stopped at 98.903 for
  rxregexp, which the oracle's build directory holds`, the eleven being
  `library_method_entry_missing`, `library_routine_entry_missing`,
  `library_attribute_entry_missing`, `library_method_external`, `library_loads_once`,
  `library_method_missing_argument`, `library_method_extra_arguments`, `library_method_raises`,
  `library_uninit_collected`, `library_uninit_termination`, `library_uninit_twice`; and none of the
  round's four new witnesses among them, because without their `.d/` fixtures each stops at a
  `.nil~package` 97.1 or a trapped 43.901 before any library is asked for by a directive (read
  from their sources), which the assertion does not see.

### 2.4 Past the witnesses: results (run)

* `x2-lower`: lines 1-5 SAME (`early nil` / `binder pk.cls` / `loaded exact pk.cls` / `loaded
  upper pk.cls` / `loaded lower pk.cls`); line 6, `p~findRoutine('SQ')~call(25)`, is the crate's
  loud rc 120 `a call to a ::ROUTINE EXTERNAL is not implemented (Phase 8)` where the oracle prints
  `call 5`. The same refusal on the base `e64202ae7` (`p/x2-lower-base`, where lines 1, 3, 4, 5 are
  the base's `REXX`), so pre-existing and loud; `run/tests.rs:7384` pins the message. Confirmed
  for the five lines predicted; the call line was not predicted and is not this round's.
* `x34-routine` (cmpsep): lines 1-5 SAME (`first raised 98.903` / `routine after dropped binder
  pk.cls` / `again loaded` / `binder routine pk.cls` / `loaded routine pk.cls`), then the same
  loud rc 120 at `call`. Confirmed for the routine-package lines.
* `x34-method`: SAME x3. Confirmed.
* `x34-method-class` (added after the next result): SAME x3, `.K` on both. So the dropped
  package's classes are absent on both sides.
* **`x34-method-lookup`: DIFF, silent.** Oracle: `first raised 90.998` / `method after dropped
  binder pk.cls`, then `Error 43.1: Could not find routine "PKR"` at `fs2 line 1`, rc 213. Crate:
  the same two lines, then `pkr ran` / `package answered The Package class`, rc 0. On the base
  `e64202ae7` the crate answers 43.1 rc 213 too (the parent is `REXX` there); on `bins/rexx-run-x2`
  (X2, before X3) the crate stops at 93.953 (the object is unbound); on `bins/rexx-run-x5` (after
  X3 and X4, before Y1) it prints `pkr ran` at rc 0. So the divergence is X3 meeting X4: X3 binds
  the method's shared code to `pk.cls` while `pk.cls` is being translated; the translation then
  fails and X4 drops `pk.cls` from `required_packages` but leaves the `routines` record its first
  walk made under that `ProgramId`; the bound object's package is that id, `.Package~new` takes it
  as the parent (`executable_context_package`, since F4 for a bound object), and
  `installed_routine`'s walk (`run.rs:3821-3848`) finds `PKR` in `self.routines` for the dropped
  program. The oracle's discarded package has no routine table, so its parent walk finds nothing.
  Finding 1 below.
* `x4-two-requirers`, first run through `cmp.sh`: invalid, since both sides share one directory and
  the oracle's run had rewritten `bad.cls` before the crate ran (the crate printed `loaded` where
  the oracle raised). Re-run with a directory per side (`cmpsep.sh`, `p/x4-two-requirers-sep-{o,r}`):
  SAME x3, `a first raised 99.903` / `b first raised 99.903` / `bad direct raised 99.903` / `bad
  again loaded` / `kb class The KB class` / `a again loaded` / `b again loaded`. Confirmed: the
  crate re-reads a file whose translation failed, and keeps the two requirers that translated.
* `x4-prolog`: SAME x3 as predicted. Confirmed: "or in its prologue stays cached" holds.
* `y1-more`: SAME x3 as predicted. Confirmed.
* The report's non-crashing Y1 controls re-run (`p/y1-pkg-realcontext`, `y1-pkg-nocontext`,
  `y1-method-newfile`, `y1-routine-newfile-plain`): SAME x3 each. `y1-routine-new-plain`,
  `y1-method-new`, `y1-method-new-realcontext`: oracle rc 0, crate rc 120 `method "NEW" of class
  "Routine"`/`"Method" is not implemented (Phase 5)`, the loud pre-existing refusal the report
  leaves. Confirmed.
* `y1-class-parent`: DIFF on four lines, oracle `The RE class` / `The K class` / `The RE class` /
  `The RE class` against the crate's `.RE` / `.K` / `.RE` / `.RE`; lines 1, 3, 7 SAME (`bound
  pk.cls`, `answered The Package class`, `newFile 1`); stderr and rc SAME. The base `e64202ae7`
  prints the same four (`p/y1-class-parent-base`, differing from HEAD only on line 1, `bound
  REXX`), so a class resolved through a package parent (`findPublicClass`,
  `classes/PackageClass.cpp:1042-1049`) is a pre-existing silent gap, reached with a real package
  as the context just as with a bound library object: `run.rs:3842` is the only reader of
  `package_parents`, and it serves routines. Predicted; not this round's. Finding 4 below.

## 3. X5's instruments: results (run)

### 3.1 Miri (`residual-rereview/miri/`, statuses in `miri/status.txt`)

* B1 exit 0 (`miri/B1.txt`): `16 passed; 0 failed; 2 ignored`, the two ignored
  `an_unbuilt_entry_refuses_loudly` ("spawns a process") and
  `a_routine_is_found_by_its_exact_spelling_before_its_case` ("opens the running image"), the two
  stub tests ok. Confirmed.
* B2 exit 1 (`miri/B2.txt`): `Undefined Behavior: attempting a read access using <184714> at
  alloc57341[0x18], but that tag does not exist in the borrow stack for this location` at
  `ffi.rs:46:14`; `<184714> was created by a SharedReadWrite retag at offsets [0x0..0x18]` at
  `ffi.rs:210:22`; frames `ffi.rs:46` <- `:238` (`activation_of`) <- `:266`
  (`set_object_variable`) <- `:485` (the stub's first `SetObjectVariable`, after its five thread
  calls) <- `invoke.rs:94` <- `:557` (the test). Confirmed exactly. The same mutant on stable
  (`miri/B2-stable.txt`): the test passes. Confirmed.
* B3 exit 1 (`miri/B3.txt`): the same report at `alloc57341[0x30]`, retag `[0x20..0x30]` at
  `ffi.rs:207:45`, frames `:46` <- `:238` <- `:313` (`string_length`) <- `:476` (the stub's
  first thread call). Confirmed exactly, the report's M5b replayed.

So the committed thread-table test pins both wrapper pointers under Stacked Borrows: each one-line
"field rather than wrapper" mutant is reported at `owner_of` from the first callback that uses the
mutated pointer, and stable sees neither.

### 3.2 The sidecar control and `ir_recorded` (`residual-rereview/x5/`, statuses in `x5/status.txt`)

* S0 exit 0: both sidecar tests ok, `2 passed; 28 filtered out`. Confirmed.
* S1 exit 101 (`x5/S1.diff`: one added line): `a_sidecar_changes_what_one_of_the_interpreters_answers
  ... FAILED` at `corpus.rs:733:13`, `lang/external_trace.rex: neither interpreter answers
  differently without its Variable("LD_LIBRARY_PATH"), so that part of the sidecar is not
  load-bearing ...`; `every_sidecar_names_a_program_the_subset_runs ... ok`. File restored, `cmp`
  clean. Confirmed exactly: the per-variable control catches a live line carried to a program that
  does not need it.
* I0 exit 0: 28 passed. Confirmed.
* I1 exit 101 (`x5/I1.diff`: the five-line sidecar lookup replaced by `Sidecar::default()`):
  `every_population_runs_without_a_refusal` FAILED, `11 of 10884 programs did not run cleanly`,
  every line `[corpus] corpus lang/<name>: stopped at 98.903 for rxregexp, which the oracle's
  build directory holds`, the eleven exactly the set predicted, none of the round's four
  witnesses, none of `library_method_package`, `library_search_path_fixed`,
  `library_load_retried`. File restored, `cmp` clean. Confirmed exactly, count included.

### 3.3 `collect_stress`'s L0 test (run)

`cargo test --release -p rexx-exec --test collect_stress -- the_l0_subset` in my copy: exit 101,
`thread 'rexx-interp' panicked at crates/rexx-exec/src/dispatch.rs:1506:9`, no `library_` or
phase-8 program named in the output (`x5/stress.txt`). As the report says: unchanged, blocked before
`phase-8.txt`.

## 4. Comments, doc comments, commit messages and the report (priority 4)

**Every C++ citation the seven commits add or change, printed from the worktree's `interpreter/`
(section 1 of this run's outputs), each landing on the construct its sentence names:**
`LibraryPackage.cpp:270-299` (`loadRoutines`; `:291` is `routines->put(routine, routineName)`),
`:320-325` (`locateMethodEntry`'s caseless compare), `:344-363` (`locateRoutineEntry`; `:355` the
compare), `:374-400` (`resolveMethod`; `:383` the cache read, `:391-392` the build and `put` under
the name asked), `:410-435` (`resolveRoutine`; `:420` the exact `get`, `:428` the `get` under the
entry's spelling), `:232-237` (the version check, then `loadRoutines`);
`HashContents.cpp:245-256` (`setValue` on a present key); `RexxUtilCommon.cpp:2161` and `:2176`
(`SysUtilVersion` twice, spelled alike); `DirectiveParser.cpp:1381-1388` (`resolveMethod` then
`setPackageObject` inside `createNativeMethod`), `:2684-2693` (`resolveRoutine`,
`routine->setPackageObject(package)` with its answer discarded at `:2691`, `setEntry` at `:2693`);
`PackageManager.cpp:824-838` (`getRequiresFile`; `:828` `createPackage`, `:836`
`addToRequiresCache`); `BaseExecutable.cpp:121-125`, `:274-277`, `:343-346` (`getPackage`'s
`resultOrNil` and its two consumers); `PackageClass.cpp:202-205`, `:680-684`, `:836-839`,
`:878-881`, `:931-934`, `:997-1000`, `:1042-1049` (the parent walks); `LanguageParser.cpp:603`,
`:637`, `:669` (`inheritPackageContext`); `Activity.hpp:503`, `MethodContextStubs.cpp:374`,
`Activity.cpp:1844-1849`, `ActivationApiContexts.hpp:58-95` (the `ffi.rs` citations X1's
neighbourhood keeps). No citation misses.

**Checked and true (read against code, or run where a run is named):** X1's `entry_type!` and
`METHOD_CONTEXT` docs and every `# Safety`/`SAFETY:` in section 1.2; X1's message (sections 1.1,
1.2); X2's `library_codes` doc (the `setPackageObject` in-place-then-copy rule and the `::ROUTINE`
discard, `NativeCode.cpp:130-140` was printed by the previous re-review, `DirectiveParser.cpp:2691-2693`
here), `library_routine_codes` doc, `LibraryCodeKey.procedure` doc, the edited
`library_method_package.rex` header, and the message's three "exactly" claims (section 2.2); X3's
comment in the first walk and its message (2.2); X4's `translated` doc, the `load_requires`
comment including "or in its prologue stays cached" (`x4-prolog`), the message's two-requirer and
retry behaviour (`x4-two-requirers-sep`) and its "exactly the base class line" (2.2); X5's
`library_open_attempts` doc and the test's two comments, `invoke::method`'s `# Errors` (read against
`values::to_native` `:961-995`), the `ir_recorded` comment and test doc (3.2), `sidecar.rs`'s and
`corpus.rs`'s docs (3.2), `library_search_path_fixed.env`'s comment against its program (it copies
`WIDEN_FROM` under `libzzregexp.so` into `WIDEN_TO` and writes that into `LD_LIBRARY_PATH`);
Y1's `executable_context_package` doc, the witness header and `.env` comment, the `phase-8.txt`
rows; Y2's `Library::routine` doc and message, with its extent claim re-derived (section 2.2:
routine rows under `extensions/`, `interpreter/`, `testbinaries/` grouped by file and lower-cased
name: the only repeated name is `RexxUtilCommon.cpp`'s `SysUtilVersion`, twice, spelled alike);
`oracle-crashes.txt` entry 15 against `residual-fix/src/y1-pkg-dotclass/main.rex` (the program
byte for byte), `p/y1-pkg-dotclass/o.{out,err,rc}` (`prolog ran`, 0 bytes, 139), `p/y1-pkg-find`
(`prolog ran` / `findRoutine CALLERROUTINE`, 139) and the three `-again` runs (139 each): the text
matches what was measured; "an object that is neither the routine nor `.nil`" is inferred from the
printed string value, which neither a Routine (`a Routine`) nor `.nil` (`The NIL object`) has.

**Statements found false or overreaching:**
* `residual-fix-report.md`, "Not done": "After a translation failure the uncached program's
  per-program records (its `routines`, `package_options`, `required_paths`) stay keyed by an id
  nothing reaches again." Run: `x34-method-lookup` reaches `routines` for that id through a
  library object the program bound before it failed (finding 1). This is the sentence that hid the
  defect: it was written as a reason not to clear those records.
* `rust/crates/rexx-exec/src/lib.rs` (`LibraryCodeKey`'s doc, X2): "a routine's is built once per
  routine table entry when the library loads and found without regard to case
  (`LibraryPackage::loadRoutines` and `::resolveRoutine`, `:270-299`, `:410-435`)". Y2's own
  doc and message on `Library::routine`, in the same range, say `resolveRoutine` finds the exact
  spelling first and only then a caseless match; the crate now does the same, and the key
  `native_load_external` and `resolve_directive_library` build is that row's name. For every table
  the C++ tree builds the two readings coincide (no two names differ only in case), so no shipped
  answer is wrong; the sentence describes the cited function as X2 read it, not as Y2 corrected it.
  Read.
* `9c0d44d8e`'s message: "every shape the oracle answers is answered the same". The oracle answers
  `.Method~new('mm', 'return 43', m)` and `.Routine~new(...)` with such a context (rc 0) where the
  crate refuses loudly (the message's own next paragraph says so), and for a bound object it answers
  `The RE class` for a class resolved through the parent where the crate prints `.RE`
  (`y1-class-parent`, pre-existing). Read as "every shape in which nothing is resolved through an
  unbound object's parent", which is what the report's table covers, it holds; as written it does
  not. Run.

## 5. Per original finding

| finding | status | witness here |
|---|---|---|
| boundary 1 (safe code reaches `owner_of` through the tables) -> X1 | **closed** | P1, P2 (eight shapes refused at the type or privacy level), P3 (ABI unchanged), P4; section 1.1 reading of every `pub` item |
| boundary minor 3 (no Miri-runnable thread-table test) -> X5 | **closed** | B1 passes; B2 (my method-side mutant) and B3 (the report's M5b) both reported at `owner_of` under Stacked Borrows, stable blind to B2 |
| boundary minor 4 (`# Errors` pre-F9) -> X5 | **closed** | read against `to_native` |
| R1, R2 (routine keyed per spelling; second binder) -> X2 | **closed** | `f4b`, `f4a3`, `x2-lower` lines 1-5, the witness on HEAD and on three saved binaries |
| R3 (bind only after every directive resolved) -> X3 | **closed**, with a new defect beside it | `f4e`, the witness and its before-binary; finding 1 is the binding X3 makes meeting X4's drop |
| R4 (retry after a library failure) -> X4 | **closed** | `f5-t3`, `f5-t2`, `pk-retry`, `x4-dup/entry/req`, the witness on HEAD, before-binary and mutant; `x4-two-requirers-sep`, `x4-prolog` |
| R5 (inert `LD_LIBRARY_PATH` line, per-variable control) -> X5 | **closed** | S0, S1 |
| R7 (`executable_package` doc; the unbound context regression) -> X5, Y1 | **closed** for every shape the oracle answers without resolving through the parent; the class-through-parent gap is pre-existing (finding 4) | `r7-routine`, the Y1 witness, `y1-more`, the four non-crashing controls |
| R8 (`library_opens`) -> X5 | **closed** | read |
| R10 (`ir_recorded` cannot see a lost sidecar) -> X5 | **closed** for a 98.903 on a library the oracle ships, and for nothing else, as the report says | I0, I1 (eleven, the predicted set) |
| `Library::routine` match order -> Y2 | **closed** | the unit test in my `-p rexx-api` run (lib 18), the extent claim re-derived |

## 6. Findings by severity

**Critical:** none.

**Important**
1. `rust/crates/rexx-exec/src/lib.rs:3297-3304` (`load_requires`' drop of a package whose first walk
   did not finish) with the first walk's routine records and `run.rs:3821-3848`
   (`installed_routine`'s parent walk): **a routine declared by a package whose translation later
   failed is still found through that package as a parent.** X3 binds a library procedure's shared
   code to the package while it translates; when a later directive of that package refuses, X4
   removes the package from `required_packages` but not the `routines` entry the first walk made
   under its `ProgramId`; the bound `loadExternal*` object still reports that id as its package
   (`pk.cls` by name, as the oracle's discarded package does), and as the context of `.Package~new`
   (or `newFile`) it becomes the parent the new code's routine calls walk. Run,
   `residual-rereview/p/x34-method-lookup`: oracle `Error 43.1: Could not find routine "PKR"`,
   rc 213; crate `pkr ran` / `package answered The Package class`, rc 0, stderr empty. Silent.
   Introduced by this round: base `e64202ae7` 43.1 (parent `REXX`), `bins/rexx-run-x2` 93.953
   (unbound), `bins/rexx-run-x5` `pkr ran`. The report's "keyed by an id nothing reaches again" is
   the false premise. Not built: the fix shape is to drop the first walk's per-program records with
   the cache entry, or to give the dropped program an empty routine table, which is what the
   oracle's discarded package has; the witness shape is the probe (`library_bound_before_refusal.d/pk.cls`
   plus one `::routine`, and the bound method as a `.Package~new` context whose prolog calls it).
   The classes half is already right (`x34-method-class`, `.K` on both sides).

**Minor**
2. `residual-fix-report.md`, "Not done", the sentence quoted under finding 1: a false statement
   about the tree, and the reason the defect was not looked for. Run.
3. `rust/crates/rexx-exec/src/lib.rs`, `LibraryCodeKey`'s doc (X2, `0e57202cd`): "found without
   regard to case" for `resolveRoutine`, which Y2 (`233d2766d`) in the same range documents and
   implements as exact spelling first. Unobservable through any shipped extension. Read.
4. `run.rs:3842` is the only reader of `package_parents`, so a class resolved through a package
   parent (`PackageClass::findPublicClass`, `classes/PackageClass.cpp:1042-1049`) answers the
   unresolved symbol: `y1-class-parent`, oracle `The RE class` / `The K class` against `.RE` / `.K`
   with a bound library object, the caller's own package and a loaded package as the context, and
   through `Routine~newFile`. Pre-existing (the base prints the same), silent, rc 0. The Phase 7
   survey record `docs/superpowers/records/2026-09-12-phase-7/survey/E-external-resolution.md:305-312`
   names `parentPackage` and `findClass`; the files carrying a KNOWN GAPS block have no line
   mentioning a parent, so it wants a KNOWN GAPS row in the docs pass. `9c0d44d8e`'s "every shape
   the oracle answers is answered the same" overreaches by this and by the loud
   `Method~new`/`Routine~new` refusal. Run.

**Loud, pre-existing, not counted:** `~call` on a `::ROUTINE ... EXTERNAL "LIBRARY"` routine object
is rc 120 `a call to a ::ROUTINE EXTERNAL is not implemented (Phase 8)` on HEAD and on the base
(`x2-lower`, `x34-routine`); `Method~new`/`Routine~new` with any third argument is rc 120 "not
implemented (Phase 5)"; `~routines~items` is rc 120 (`r7-unbound`); the crate's traceback lacks the
oracle's `Compiled method "NEW" with scope "Package".` line (`x34-method-lookup` on the base).

## Not reached

Silence below is not coverage.
* **Tree Borrows** was not run; every Miri run is Stacked Borrows.
* **The gates**: no fmt, clippy, debug gate or full corpus gate was run in my copy (the controller's
  gate run in the worktree is that); here only `-p rexx-api`, `unsafe_sites`, the two sidecar
  tests, `ir_recorded` and `collect_stress`'s L0 test ran.
* **Finding 1's other records**: `package_options`, `required_paths`, `reqstr_armed`,
  `lostdigits_armed`, `merged_public_routines` of a dropped program were not probed for a route that
  reaches them; only `routines` was, and the oracle's side of finding 1 is read off the 43.1, not
  off where `LanguageParser` attaches a routine table.
* **Identity** (`==`) between a dropped package and its re-translation, on either side.
* **A version-refused library** through the X3/X4 path beyond `f5-t2`; an `::ATTRIBUTE` whose
  `GET` resolves and `SET` does not; a routine table with two names differing only in case on a
  forged extension (Y2's unit test is the only witness); concurrency on any of the round's paths.
* **Class lookup through a parent** (finding 4) was not traced further than `run.rs:3842`; which
  crate site would own it was not read.
* **The corpus witnesses under the harness** were run through `cmp.sh`/`cmpsep.sh`, not through
  `tests/corpus.rs`'s sidecar plumbing, except as `ir_recorded` and the sidecar control exercise
  them.
* **Scratch state**: worktree HEAD `233d2766d`, `git status --short` empty at the end (this file is
  under the ignored `.superpowers/`, `git check-ignore -v`); my tree copy has its probe files
  removed and its two edited files restored (`cmp` clean); the Miri trees hold their one-line
  mutants; `p/x4-two-requirers` (the invalid shared-directory run) is kept beside
  `p/x4-two-requirers-sep-{o,r}`.
