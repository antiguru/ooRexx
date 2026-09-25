# Task 4 report: a thread context that lives as long as its interpreter

Base: `91f6afaba`. Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-4/`
(below, `$S`). The forged extension, the derivation scripts and every probe program are committed
beside this report in `task-4-forge/`; its two runner scripts still name `$S` paths.

## Status

Steps 1 to 3 implemented, the fast checks green; the full gates are in the Gates section.

## Commits

The Gates section's first line names the sha the gates ran on.

## Oracle measurements

### The forged extension

`task-4-forge/forge.cpp`, built into `$S/forge/lib/lib<TAG>.so` by

    g++ -shared -fPIC -O1 -I/home/moritz/dev/repos/ooRexx/api \
        -I/home/moritz/dev/repos/ooRexx/api/platform/unix -DTAG='"<name>"' [flags] forge.cpp

with flags `-DLOADER_RAISES`, `-DUNLOADER_RAISES`, `-DLOADER_REFUSES`, `-DUNLOADER_REFUSES`,
`-DREQUIRED=0x7fffffff`, `-DDESTRUCTOR_CALLS` for the variants. Every built library's `readelf -d`
`NEEDED` is `libc.so.6` only and `nm -D --undefined-only` shows no `Rexx` symbol, so none maps an
interpreter into the process. Its loader and unloader print their tag, call `WholeNumberToObject`
through the context they are handed, and print whether the answer was null and (unloader) whether
the context is the one the loader was handed; routines `Stash`, `UseStash` (answers
`WholeNumberToObject(42)` through the kept context) and `Same`. The first build called `CString`,
which is an aborting slot here, so every forge line below is from the second build, which prints
`null=` instead; both sides were rerun on it.

Oracle runs: `task-4-forge/oracle.sh`, the standard wrapper from a fresh `mktemp -d`, with the forge
directory appended to `LD_LIBRARY_PATH`; three descriptors separately. Crate runs:
`task-4-forge/crate.sh`, the same with `ulimit -v 4194304`.

### No native frame in flight

In the oracle every path that runs extension code pushes a native activation first (a method or
routine call, or `Activity::run` for a hook, `interpreter/concurrency/Activity.cpp:3461-3474`), so
the only way to call through a thread context with none in flight is from code the interpreter did
not call. `probes/dtor.rex` keeps the context in a routine call and the library's destructor, run
by the `dlclose` at termination, calls `WholeNumberToObject` through it. Oracle: stdout ends
`forgedtor destructor`, stderr `terminate called after throwing an instance of 'NativeActivation*'`,
**rc 134**. The crate now answers rc 134 too, the panic naming
`RexxThreadInterface.WholeNumberToObject was called with no native call in flight`. The stdout
differs: the crate's buffered Rexx `say` output (`main`, `stashed`) is lost to the abort where the
oracle had already written it (Concerns).

### Loader: when it runs, what it is handed

* `probes/order.rex` (`::requires` b, a, c, d) and `order2.rex` (d, c, a, b): the loaders run in
  `::requires` order, all before the program's first clause.
* `probes/runtime.rex`: `Package~loadLibrary` runs the loader inside the call, before it answers `1`.
* `probes/x42.rex`: the context kept by `Stash` and used by `UseStash` forty Rexx levels deeper
  prints `x 42`, rc 0; and `order.rex`'s `Same` shows a routine's `threadContext` is the loader's
  (`same 1`).
* `probes/loadraise.rex`: a loader that raises 40.0 runs on after the raise (`loader after raise`),
  then the `::requires` reports `Error 40 ... Incorrect call to routine.` at its line, rc 216.
  `loadraise2.rex` traps it: condition code `40.0`, `UseStash()` then answers `x 42`, so the
  routines had registered before the loader ran, the library stays held, and a second
  `loadLibrary` answers `1` without running the loader again.
* `probes/version.rex` (`requiredVersion` above the interpreter): 98.982, rc 158, and the loader
  never runs.

### Unloader: order and a raise inside one

* Every held library's unloader runs at termination, after the `UNINIT`s
  (`runtime/Interpreter.cpp:279-281`), including a version-refused library's (`version.rex` prints
  `forgever unloader`) and one whose loader raised.
* Order is `PackageManager::packages`' iteration order: a `StringTable` seeded with `REXX` and
  `REXXUTIL`, bucket by bucket, each chain in insertion order, with a load that finds nothing
  counted as a put and a remove. `task-4-forge/model.py` is that geometry, and it predicted every
  measured order before any Rust was written: the two above (both `forgea forgeb forgec forged`),
  and `orderA`/`orderB`/`orderC`, whose names were chosen so that one bucket's chain empties the
  free chain, and where `orderB` and `orderC` differ only by a trailing load of a missing library,
  whose put expands the table (`seq.txt` holds the names and the predictions).
* `probes/unloadraise.rex`: an unloader that raises runs on after the raise, then the walk ends:
  the libraries after it in table order run no unloader, stderr is empty, rc 0.
* The unloader's context is not the loader's in the oracle (`same=0` on every unloader line):
  termination runs on an instance created for it. Here it is the same context (`same=1`).

Crate against oracle on `e6dcc65d0`'s binary, every probe above except `dtor`: rc equal, stderr
byte-identical, the Rexx lines identical, and the loader and unloader lines identical once the
`same=` field is dropped. **The destructor lines were not identical**, which this paragraph first
claimed they were (the review's I1): the oracle closes each library right after its unloader, and
this crate closed them all when its `HashMap` dropped, in an order that changed from run to run.
Fix round 2 fixes the order; the other differences are `same=` and where the Rexx lines fall
relative to the extension's own `printf` lines (Concerns).

## Which shipped extensions declare a hook

Derived from the oracle build's binaries, not the sources: `task-4-forge/hooks.py` reads each
`RexxGetPackage`'s answer out of the disassembly and GOT relocation, then the dynamic relocations at
`loader` (+32) and `unloader` (+40) of that `RexxPackageEntry`; no library is loaded. The method was
checked on `librxregexp.so` (name, version and methods relocated, routines not) and `librxmath.so`
(routines relocated, methods not). Over every `build/lib/*.so` exporting `RexxGetPackage`:

| library | loader | unloader | NEEDs an interpreter library |
|---|---|---|---|
| `libhostemu.so` | yes | yes | yes, `librexx.so.4` and `librexxapi.so.4` |
| `liborxinvocation.so` | yes | yes | yes, through `liborxexits.so` |
| `librxsock.so` | yes | no | no |
| `librxunixsys.so` | yes | no | no |
| every other | no | no | |

The sources agree (`extensions/rxsock/rxsock.cpp:703`, `extensions/platform/unix/rxunixsys/rxunixsys.cpp:1740`,
`extensions/hostemu/platform/unix/hostemu.cpp:857-858`, `testbinaries/orxinvocation.cpp:532-533`).
So no library this tree may load declares an unloader, and the loaders it may load are empty on
this platform (`rxsock.cpp:617-626` is Windows-only, `rxunixsys.cpp:1658`), so nothing either does
is observable. Both hooks are witnessed in-repo only by unit tests; `probes/sock.rex` runs the
shipped loaders and agrees with the oracle, on the base binary as well as the new one.

## Design

* `ffi::ThreadContext` owns, through an `Rc`, a heap allocation made with `Box::into_raw` that
  holds the `RexxThreadContext_` and `RexxInstance_` wrappers, the thread table and an `Innermost`
  cell; the extension is handed an address into it, and nothing forms a reference to the whole of
  it. `Interp` holds one for its lifetime.
* `ThreadContext::enter(activation, body)` makes `activation` the innermost call for `body` and
  puts the previous one back on the way out, including by unwind; the method and call contexts are
  per call as before, built inside it, so no `Contexts` exists outside an `enter`. That is a
  closure rather than a guard object because `mem::forget` of a guard would leave `Innermost`
  dangling from safe code.
* The thread callbacks read `Innermost` through the context's `owner`, which is what
  `contextToActivation` does for a thread context; none in flight panics inside the `extern "C"`
  frame, so the process aborts, the oracle's rc 134.
* The table's data members are written at the first `enter` and compared, in a debug build, with
  every later call's registered constants.
* Each `Library` holds a clone, declared after its handle, so the allocation outlives every
  `dlclose` and a destructor calling through a kept context gets the abort, not a freed read.
* `load::Library` carries the entry's `loader` and `unloader`, private as `entry_point` is; a
  version-refused entry keeps no loader. `invoke::hook` runs one inside `recording_refusals`.
* `Interp::settle_library` runs the loader after `register_package_routines`; its raise or refusal
  becomes `LibraryLoad::Raised`, which replaces `LibraryLoad::Version`, so every caller raises it as
  it raised 98.982. `execute` runs `Interp::run_package_unloaders` after the termination
  `UNINIT`s. `Libraries` keeps `PackageTable`, the oracle's table geometry, for that order.

## Step 2: the kept-thread-context test, and how it fails under the old design

`invoke::tests::a_thread_context_kept_from_one_call_reaches_the_next`: `ffi::stashing_stub` keeps
its method context's `threadContext` in one call made from a frame that has returned, and
`stash_using_stub`, called forty frames further down, answers `WholeNumberToObject(42)` through the
kept pointer. The answer converts only if the callback registered it in the second call's own
local references.

Under the old design the kept pointer addresses the first call's stack `Contexts`, so the read is a
use-after-return and a plain build's outcome is not determined. Measured by porting the test to the
old API over a `git archive` of `91f6afaba` in `$S/old-tree/`:

* plain `cargo test -p rexx-api --lib`, three runs: red all three, `left: Err(StaleHandle)`: the
  stale read reached some activation, which registered the 42 in a table that is not the second
  call's. That is this build's accident, not a property;
* `cargo miri test` (Stacked Borrows): `Undefined Behavior: in-bounds pointer arithmetic failed:
  alloc61488 has been freed, so this pointer is dangling` at the `(*(*thread).functions)` read,
  allocated at the test's `Contexts::new` line and freed at that function's closing brace
  (`$S/miri-old.txt`).

## Step 3: a refusal recorded inside a hook

`invoke::hook` reads the record `recording_refusals` answers and returns
`Failure::UnfilledSlot`, forgetting any condition raised after it, as `invoke::run` does;
`Interp::run_package_hook` turns that into the `Loud` "not implemented (Phase 8)" refusal. The
record cell's doc in `layout.rs` names both readers. Measured on the new binary with
`probes/lref.rex` and `uref.rex` (`NewStringFromAsciiz` in the loader, and in the unloader of the
middle one of three libraries): each exits 120 with
`rexx-exec: RexxThreadInterface.NewStringFromAsciiz is not implemented (Phase 8)`, the loader case
before the program's first clause. The unloader case also overrides the program's `exit 3`, as a
refusal in a termination `UNINIT` does.

## Negative controls

Each prediction below was written before its run. Every mutated file was restored from a copy and
compared with `cmp`.

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| C1 | `Libraries::missed` does nothing | `unloaders_run_in_the_oracles_table_order` red on the fourth case only (the miss that expands is the last ask); the fifth stays right because `vakh`'s put expands the table anyway | red on the fourth case, answering `orderC`'s order. datadriven stops at the first mismatch, so the fifth case's half of the prediction is unmeasured |
| C2 | expansion to `bucket_size(buckets * 2)` instead of `* 4` | red on the fourth and fifth cases, the first three green | red on the fourth case (first mismatch; the fifth unmeasured, as C1) |
| C3 | `string_hash` over unsigned bytes | green: every case name is ASCII, so this is a blind spot, not a witness | green |
| C4 | `Entered::drop` writes null instead of the previous call | `a_nested_call_hands_the_thread_context_back_to_the_outer_call` ends the test binary by abort (the outer raise finds no call in flight), a red with no name | as predicted: panic at the no-call-in-flight assertion inside that test, binary ended by SIGABRT |
| C5 | `Entered::drop` puts nothing back, leaving the finished nested call innermost | plain: `a_nested_call_hands_the_thread_context_back_to_the_outer_call` red or crashing, since the outer raise lands on the dropped inner activation (use-after-free, so the plain outcome is not determined); Miri: that test reports the dangling read; every other test green under both | plain: that test red, `left: None, right: Some(40001)`, the rest green; Miri: `constructing invalid value of type &values::Activation<'_>: encountered a dangling reference (use-after-free)` at `innermost_activation`'s final dereference |
| C6 | `invoke::hook` ignores the refusal record | `a_hook_that_reaches_an_unwritten_member_is_refused` red, answering `(Ok(()), Some(40001))`; the other hook test green | as predicted, the rest green |
| C7 | `library_of` keeps the loader of a version-refused entry | `a_library_refused_for_its_version_keeps_only_its_unloader` red at its first `has_hook(Loader)` assertion | as predicted (`load.rs:848`), the rest green |
| C8 | `settle_library` never runs the loader | every in-repo test green: no library this tree may load declares a loader that does anything observable, so only the scratch forge sees it | `cargo test -p rexx-exec --lib` 849 passed, 0 failed, and `--test corpus` green; the forge's `loadraise.rex` on the mutant binary is rc 139 where the oracle is 216 |

## Miri

Nightly `rustc 1.100.0-nightly (f7575a9da 2026-09-24)` with `miri` and `rust-src`, installed by
`RUSTUP_HOME=$S/rustup-home CARGO_HOME=$S/cargo-home rustup toolchain install nightly --profile
minimal --component miri,rust-src` (its last line is the same unrelated `rustup is not installed at
<CARGO_HOME>` error the L2 slice recorded; the toolchain is in place).

`cargo miri test -p rexx-api --lib --offline` over the working tree, Stacked Borrows (no
`MIRIFLAGS`), `CARGO_TARGET_DIR=$S/target-miri`: **exit 0, 29 passed, 7 ignored** (`$S/miri-new.txt`).
Ignored: the child-process tests, and those that `dlopen` the running image, which Miri cannot
(the two hook tests among them, so `Library::run_hook`'s call is not under Miri; the thread
context it hands over is, through the other tests). The kept-context and nested tests pass under it.

A first run reused the old tree's test binary: both trees are the same workspace at different
paths, cargo's artifact names do not depend on the path, and the old tree's dep-info mtimes made
its binary look fresh. Its report is kept as `$S/miri-new-WRONG-reused-old-binary.txt`; the target
directory was deleted and the run above is from an empty one.

## Gates

First run, at `e6dcc65d0` (`$S/gates/status.txt`): G1 fmt 0, G2 clippy from an empty target dir 0,
G3 `cargo build --workspace --release` 0, **G4 137**, G5 debug build 0, **G6 101**. G4 was
OOM-killed by `memcap` at 8G while compiling `rexx-api` and `rexx-exec`, because G3 built the
libraries and not the test targets `cargo test` needs. G6's one failure was
`refusal_sites::the_table_holds_every_constructor_the_source_defines`; see Fix round 1.

Second run, at `61be5ecfb` (`$S/gates2/status.txt`, G3/G5 now `cargo test --no-run`): G1 0, G2 0,
G3 0, G4 0 with 0 `Compiling` lines (load 9.73 / 11.13 before), G5 0, G6 0 with 0 `Compiling`
lines (load 4.05 / 7.45 before); the tree was clean and at the same sha at the end.

## Concerns

1. **The unloader's thread context is the loader's here and a fresh one in the oracle.** Visible
   only to an extension that compares the two pointers; recorded, not matched.
2. **Rexx stdout is buffered to the end of the run and an extension's `printf` is not**, so the
   relative order of the two differs from the oracle, and an abort loses the Rexx lines (the
   `dtor` probe). This predates the task; it becomes visible once an extension prints.
3. **The loader and unloader wiring in `rexx-exec` has no in-repo witness** (C8): the unit tests
   cover `invoke::hook`, `Library` and the table order, and the forge probes in `task-4-forge/`
   are the only check that `settle_library` and `execute` call them.
4. `PackageTable` models the names this crate resolves. `REXX` and `REXXUTIL` are seeded, but a
   `loadLibrary('REXX')` here still goes to `dlopen` where the oracle finds its internal package;
   that predates this task and does not change the order of any library this crate holds.
5. A non-ASCII library name is not covered by any case (C3).

## Fix round 1

### Predictions for the new witness, written before the runs

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| D1 | `settle_library` answers `Loaded(held)` without running the loader | `a_loaded_librarys_loader_and_unloader_run` red at "loading ran no loader", `left: []` | as predicted: `left: []`, `right: [("loader", true)]` |
| D2 | `Interp::terminate` does not run the unloaders | the same test red at "terminating ran no unloader", `left: [("loader", true)]` | as predicted: `left: [("loader", true)]`, `right: [("loader", true), ("unloader", true)]` |

Both mutated files were restored from copies and checked with `cmp`.

### G4 recompiled

`cargo build --workspace --release` builds neither the unit-test harness of each library nor the
integration-test targets, so G4's `cargo test` compiled them inside the cap. G3 and G5 are now
`REXX_CORPUS_GATE=1 cargo test --workspace [--release] --no-fail-fast --no-run`, the command G4 and
G6 run without `--no-run`, outside the cap; the script records how many `Compiling` lines G4 and G6
printed (`$S/gates2.sh`). No `build.rs` in the workspace reads `REXX_CORPUS_GATE` or declares
`rerun-if-env-changed`; the environment is matched anyway.

### `Raised::library_version` left the send surface

The scanner in `tests/refusal_sites.rs` tags a constructor `send` where a construction site is in
`dispatch.rs` or under `dispatch/`. Sites:

| | `91f6afaba` | `e6dcc65d0` | `61be5ecfb` |
|---|---|---|---|
| `install.rs` `require_library` | yes | no | yes |
| `install.rs` `settle_library` | no | yes | no |
| `dispatch/construct.rs` `native_load_external` | yes | no | yes |
| `dispatch/package.rs` `load_library` | yes | no | yes |

Folding the version refusal into `LibraryLoad::Raised` had moved its only construction into
`settle_library`, so the row's surface read `body` where the table says `body+send`. The behaviour
through a send did not change: the refusal was still raised, from the load under the send. The fix
restores the classification rather than re-deriving the table: `LibraryLoad::Version` is back and
constructed where it was, and `LibraryLoad::Raised` carries only a loader's failure. `git diff
e6dcc65d0 61be5ecfb -- rust/corpus/refusal-sites.tsv` is empty, and
`cargo test -p rexx-exec --test refusal_sites` is 5 passed.

The row's witness run, `probes/verload.rex` (`.context~package~loadLibrary('forgever')` under
`signal on syntax`, then again, then `.Routine~loadExternalRoutine` naming it), on `61be5ecfb`'s
release binary and on the oracle: both rc 0, stdout `try`, `code 98.982`, `again 1`,
`ext The NIL object`, stderr empty (the forge lines aside, which interleave as before).

### The in-repo witness for the hook calls

The ruling asked for the test in `ffi.rs` or `load.rs`. The registration path is `rexx-exec`'s
(`Interp::settle_library`, and termination), which `rexx-api` cannot call, so the seam is in
`load.rs` and the test is in `rexx-exec`: `load::hooks_only(loader, unloader)` reads an in-memory
`RexxPackageEntry` through the same `library_of` an opened library takes, over the running image,
and takes the hooks as safe `extern "C" fn`s, so it adds no `unsafe` outside `load.rs` and no
unsound public API. `dispatch::library::tests::a_loaded_librarys_loader_and_unloader_run` loads it
through `settle_library` and then calls `Interp::terminate`, which now holds the termination
`UNINIT`s and the unloaders that `execute` used to call one after the other; each hook records that
it ran and whether it was handed a context. D1 and D2 above are its controls.

## Fix round 2

### Predictions, written before the runs

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| E1 (M1) | `execute` calls `interp.run_termination_uninits()` where it calls `interp.terminate()` (`lib.rs`) | `dispatch::library::tests::a_programs_end_runs_its_librarys_unloader` red, `left: ["loader"]`; `a_loaded_librarys_loader_and_unloader_run` green | as predicted: `left: ["loader"]`, `right: ["loader", "unloader"]`; the other test green |
| E2 | `run_package_unloaders` does not close a library after its unloader | `a_loaded_librarys_loader_and_unloader_run` red at "terminating left the library open" | as predicted; `a_programs_end_runs_its_librarys_unloader` green |
| E3 | `Mapping::close` leaves `open` set | `a_closed_librarys_hook_does_not_run` red at `!library.is_open()` | as predicted (`invoke/tests.rs:1127`), 36 others green |

Each mutated file was restored from a copy and checked with `cmp`.

### I1: libraries close in the oracle's order

Measured first on the oracle (`probes/unloadraise{,4,5}.rex`, two runs each, identical): when an
unloader raises, no library is closed during the walk, the raising one included, and at process
exit the destructors run in the order the libraries loaded (`forgec forgeb forgeunload forged
forgea` for `unloadraise4`, whose table order is `forgeunload forgea forgeb forgec forged`). A
library whose unloader returns is closed straight after it; one with no unloader is closed at its
place in the walk (`LibraryPackage.cpp:166-181`).

The code: `load::Library` and every row copied out of it share an `Rc<Mapping>` holding the
`libloading` handle. `Library::close` drops the handle (so `dlclose` runs its destructors then),
after which a row's stub and a hook answer nothing; a close is refused while a call into the
library is in flight, so no safe code can close a mapping under a running stub. A closed method's
signature request answers `Failure::Signature`, reachable only by a call after termination.
`run_package_unloaders` closes each library after its unloader returns; `Drop for Libraries` closes
whatever is left in load order.

Forge lines, destructors included, `same=` dropped, oracle once against the crate three times,
before (`61be5ecfb`'s binary) and after (`fb860a6f2`'s tree):

| probe | before | after |
|---|---|---|
| order, order2, order3, orderA, orderB, orderC | 0 of 3 match | 3 of 3 |
| unloadraise | 1 of 3 | 3 of 3 |
| unloadraise2, unloadraise3, unloadraise4, unloadraise5 | 0 of 3 | 3 of 3 |
| uref | 0 of 3 | 3 of 3 |
| version, x42, loadraise, loadraise2, runtime, twoinst | 3 of 3 | 3 of 3 |

rc agrees on every row except `uref`, which is 120 here by design (a refusal). Sample, `order3`:

    oracle:     a.unloader a.destructor b.unloader b.destructor ... f.unloader f.destructor
    before, 1:  a..f unloaders, then e d f c b a destructors
    before, 2:  a..f unloaders, then a e f b d c destructors
    after:      the oracle's sequence

and `unloadraise3`: oracle and after `forgeunload.unloader`, then `forgee forgef forgec
forgeunload forgea` destructors; before, `forgeunload`'s destructor came first.

### M1: the program's end runs termination

`install::offer_library` (test only) lets a test offer a library under a name, which
`resolve_library` takes before searching for a shared object. `a_programs_end_runs_its_librarys_unloader`
runs `say 'main'` with `::requires 'hooked' LIBRARY` through `execute` on the interpreter thread
and asserts rc 0, stdout `main`, and that the loader and then the unloader ran. E1 is its control.

### M3 and M4: recorded

In `docs/superpowers/plans/phase-4-exclusions.txt`, beside the Phase 8 entries, as "THREE
DIVERGENCES THE SURFACE PLAN'S TASK 4 LEAVES" (commit `5a5c462ac`): the no-frame abort's stderr and
lost stdout (no owner, with the reason), the unloader receiving the loader's thread context
(Phase 9), and SAY buffering against an extension's `printf` (no owner; Phase 7 spec A1 is the
decision).

### Miri

`cargo miri test -p rexx-api --lib --offline`, Stacked Borrows, from an empty
`CARGO_TARGET_DIR=$S/target-miri` (deleted after): exit 0, 29 passed, 8 ignored (the new
`a_closed_librarys_hook_does_not_run` opens the running image). `$S/miri-fix2.txt`.

### Gates

(the gate run's results follow below once it has finished)

Third run, at `5a5c462ac` (`$S/gates3/status.txt`): G1 0, G2 0, G3 0, G4 0 with 0 `Compiling`
lines (load 14.53 / 9.75 before), G5 0, G6 0 with 0 `Compiling` lines (load 6.95 / 7.48 before);
the tree was clean and at the same sha at the end.

## Fix round 3

### Prediction, written before the run

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| F1 (m1) | `Drop for Libraries` does `let _ = library` instead of `library.close()` | `dropping_the_libraries_closes_what_termination_left_open` red at "dropping the libraries left one open"; every other `rexx-exec --lib` test green | as predicted: 851 passed, 1 failed, the panic at `tests.rs:773` naming it; restored and checked with `cmp` |

### m2

`phase-4-exclusions.txt`'s entry now reads "THE UNLOADER AND EVERY TERMINATION UNINIT RUN ON THE
LOADER'S THREAD CONTEXT", citing `runtime/Interpreter.cpp:277-281` (printed: `InstanceBlock
instance;`, `memoryObject.lastChanceUninit();`, `PackageManager::unload();`). Re-measured with
`probes/uninit4.rex` (`call stash`, then an object kept in `.local` whose `UNINIT` calls `Same` and
`UseStash`) and `uninit5.rex` (the same without `Stash`): oracle `same 0` and no `use` line, rc 0,
both; ours `same 1`, `use 42`, rc 0, both. The re-review's own variant of `uninit4` answered rc 139
on the oracle; the entry says so. Owner stays Phase 9.

### Checks

`cargo fmt --all --check` 0; `cargo clippy -p rexx-exec --all-targets -- -D warnings` 0 (warm
target dir); `memcap 8G cargo test --release -p rexx-exec --no-fail-fast` was OOM-killed at the cap
while compiling (rc 137) on the first try, so it was prebuilt with `--no-run` outside the cap and
rerun: exit 0, 0 `Compiling` lines, 1585 passed, 0 failed, 1 ignored (summed over its
`test result` lines).
