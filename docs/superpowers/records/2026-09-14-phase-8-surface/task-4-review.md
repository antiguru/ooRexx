# Task 4 review (Phase 8 surface): interpreter-lifetime thread context, package hooks

Reviewer: task reviewer, 2026-09-26. Subject: e6dcc65d0, 61be5ecfb (base 91f6afaba).
Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s4/`
(`$R` below). Every build is from a `git archive` of 61be5ecfb (`$R/tree`) or 91f6afaba (`$R/old`),
files touched after extraction, `Compiling rexx-api`/`rexx-exec` lines seen on each build. The forge
libraries were rebuilt from `task-4-forge/forge.cpp` into `$R/forge/lib`; every one has `NEEDED`
`libc.so.6` only and no undefined `Rexx` symbol. Probes run with `$R/run.sh` (standard oracle
wrapper, fresh `mktemp -d`, three descriptors separately).

Status: COMPLETE. No check hung. Target dirs deleted after the run.

## Verdicts

- **Spec: PASS.** Steps 1 to 3 are implemented as the brief asks and the oracle agrees on every
  loader and unloader observable I measured.
- **Quality: PASS, with one Important record fix.** The code is sound as far as Miri and the probes
  reach. The report states a false equality (I1).

## Findings

**Important**

- **I1. Library closes run in random order, and the report says the forge lines match.** The report
  (Oracle measurements, last paragraph) says the forge lines are "identical once the `same=` field is
  dropped". They are not: the `destructor` lines differ on `order`, `order2`, `order3`, `orderA/B/C`,
  `unloadraise*` and `uref`. The implementer's own `$S/forge/oracle` and `$S/forge/fix1` outputs show
  the same thing. The oracle closes each library straight after its unloader
  (`interpreter/package/LibraryPackage.cpp:178-181`, `lib.unload()`). The crate closes them all
  later, when `Libraries`' `HashMap` drops (`rexx-exec/src/lib.rs:3216` in the backtrace), so the
  order changes from run to run. Four runs of `$R/probes/order3.rex` gave four different
  destructor orders (`f e c a b d`, `c a d b e f`, `a e f c b d`, `b a c f e d`), against the
  oracle's fixed `a b c d e f`, where each close follows its own unloader. The HashMap drop is older
  than this task. No library this tree may load has a destructor that shows anything, so no corpus
  program or ooTest group can see it. Fix: correct the report's sentence, and record the close order
  as a divergence with an owner. The cheap fix is to close in `in_unload_order`, after each
  unloader.

**Minor**

- **M1. Nothing tests that `execute` calls `terminate`.** Plant: `lib.rs:3180`
  `for loud in interp.terminate()` changed to `interp.run_termination_uninits()`, which is the
  pre-task line. Result: `cargo test -p rexx-exec --no-fail-fast` shows no new red (`$R/plant3.txt`).
  The only failures are in `collection_arity`, 2 tests, which also fail on the unplanted tree (they
  need `rust/target/release/rexx-run` at a fixed path). The fix-round witness covers `terminate` and
  `settle_library`, but not the edge from the program's end. D1/D2 reproduced: deleting the loader
  call (`install.rs:1941`) reddens `a_loaded_librarys_loader_and_unloader_run` at `tests.rs:705`.
  Deleting `run_package_unloaders` from `terminate` reddens the same test at `tests.rs:711`. Nothing
  else changes (849 passed).
- **M2. `load::hooks_only` is production API**: `pub`, `#[doc(hidden)]`, not `cfg(test)`
  (`load.rs:220`). A test in another crate cannot use `cfg(test)`. It is sound, since the hooks are
  safe `extern "C" fn`s and the entry names no table or string, and it is narrow. A dev-only cargo
  feature would take it off the public surface. Acceptable as is.
- **M3. The no-frame abort matches the oracle on exit status only.** `dtor.rex`: both sides exit rc
  134. The oracle's stderr is `terminate called after throwing an instance of 'NativeActivation*'`.
  The crate's is the Rust panic naming the member, then `panic in a function that cannot unwind`
  and a full backtrace. The crate's stdout loses `main`/`stashed` (the report's Concern 2). The abort
  is reached only through extension code the interpreter did not call. Every `Contexts` is built
  inside `ThreadContext::enter`, and `invoke::hook` runs inside `enter` too. The oracle aborts on the
  same probe, at the same point in the destructor.
- **M4. Concerns 1 and 2 need no new owner.** Concern 1 (the unloader gets the loader's context
  where the oracle's is `same=0`) can be seen only by an extension comparing the two pointers. The
  shipped extensions that declare an unloader are `hostemu` and `orxinvocation`, and both NEED
  `librexx`, so neither can be loaded here. Concern 2 (buffered `SAY` against the extension's
  `printf`) follows from Phase 7 spec A1 (`docs/superpowers/specs/2026-09-11-phase-7-streams-platform.md:12`).
  The loadable shipped extensions (`rxsock`, `rxunixsys`) print nothing, so ooTest and the corpus
  cannot see it. Record both beside I1 in the divergence entry.

## Checks run

1. **Soundness, Miri.** Used the implementer's scratch nightly (`surface-4/rustup-home`, rustc
   1.100.0-nightly), with target dirs deleted between trees. On 61be5ecfb, `cargo +nightly miri test -p rexx-api
   --lib`: Stacked Borrows exit 0, 29 passed and 7 ignored (`$R/miri-new.txt`). Tree Borrows
   (`-Zmiri-tree-borrows`) gives the same (`$R/miri-new-tb.txt`). The nested test
   (`a_nested_call_hands_the_thread_context_back_to_the_outer_call`) and the no-frame path are
   covered. The no-frame test spawns a child, so it is Miri-ignored and I ran it plainly: green.
   The hook tests are Miri-ignored because they `dlopen` the running image. I read every SAFETY note
   (`Home::drop`, `ThreadContext::new`/`pointer`/`enter`/`constants`, `innermost_activation`,
   `run_hook`, `hooks_only`, `library_of`), and each states an invariant the code keeps. `Entered`
   restores on unwind. The table's data members are written only before the first handout.
2. **Kept-context test on the old design.** I ported the test to 91f6afaba's `Contexts::new` API in
   `$R/old`. Miri reports `Undefined Behavior: in-bounds pointer arithmetic failed: alloc63253 has
   been freed` at the `(*(*thread).functions)` read. The allocation is at the old `Contexts::new`
   line and the free at that frame's closing brace (`$R/miri-old.txt`). The failure comes from Miri,
   so it is deterministic. On 61be5ecfb the test passes plainly and under both Miri models.
3. **No-frame abort.** See M3.
4. **Loader/unloader vs the oracle.** I ran the implementer's 17 probes plus 8 of my own: `order3`
   (six libraries required in reverse order), `unloadraise2/3` (the raising unloader first in table
   order and among four others), `loadraise3/4` (a runtime `loadLibrary` raise, and a trapped one),
   `vhook` (version-refused with a raising loader), `lref2`, and `uref2`. Two groups of probes are
   left out of the comparison: `dtor` (M3), and the four refusal probes `lref`, `lref2`, `uref` and
   `uref2`, where the crate refuses loudly with rc 120 by design (point 7). On every other probe the
   crate matches the oracle on rc, on stderr (byte for byte), and on the Rexx line sequence. On every
   probe but `uref2`, `dtor` included, the loader and unloader line sequences match, with `same=`
   dropped. On `uref2` the crate's refusal ends the unloader walk, so `forgee` and `forgef` never
   unload.
   Specifically: loaders run after `register_package_routines`, since
   `loadraise2`'s trapped raise is followed by `UseStash()` answering `x 42`. A version-refused
   library never runs its loader (`version`, `vhook`: rc 158, unloader only). A raise in a loader
   becomes the load's error and the library stays held (`again 1`). The unloader order matches for
   every load order tried. A raise in an unloader stops the rest and leaves rc 0, including when the
   raising unloader is first in the table (`unloadraise2`, where no other unloader runs). The
   crate's cites print as claimed: `LibraryPackage.cpp` 166-174 (`unload`), 232-235 (version),
   237-245 (routines then loader); `PackageManager.cpp` 79-89 and 642-650; `Activity.cpp` 3461-3474;
   `Interpreter.cpp` 279/281; `oorexxapi.h:257-258`; `ActivationApiContexts.hpp:64-68`;
   `Activity.hpp:458`, where `getApiContext` is `topStackFrame` (`Activity.hpp:318`). The report's
   table of shipped hooks agrees with the package entries in `extensions/`.
5. **Plants.** See M1: D1 and D2 are red, and the `execute` edge survives.
6. **`Raised::library_version`.** Construction sites: at 91f6afaba, `construct.rs:298`,
   `package.rs:433` and `install.rs:2017`; at e6dcc65d0, only `install.rs:1953`; at 61be5ecfb,
   `construct.rs:298`, `package.rs:433` and `install.rs:2027`. So the send sites are back.
   `git diff --stat 91f6afaba 61be5ecfb -- rust/corpus/refusal-sites.tsv` is empty. The send
   path raises: `verload.rex` gives `code 98.982`, `again 1` and `ext The NIL object`, rc 0, the
   same as the oracle.
7. **Refusal inside a hook.** `layout.rs` `REFUSED` doc names both readers. A plant making
   `invoke::hook` ignore the record reddens `a_hook_that_reaches_an_unwritten_member_is_refused`
   only (35 passed, 1 failed; restored and checked with `cmp`). Run: `lref`, `lref2` (a loader
   refusal under `signal on syntax`), `uref` and `uref2` (the refusing unloader in the middle of the
   table) all exit 120 with `rexx-exec: RexxThreadInterface.NewStringFromAsciiz is not implemented
   (Phase 8)`. The refusal cannot be trapped, and it ends the unloader walk.
8. **Implementer's concerns.** See M4.
9. **Unsafe and NEEDED.** `cargo test -p rexx-core --test unsafe_sites`: 2 passed. The files
   containing `unsafe` are the same at 91f6afaba and 61be5ecfb (`ffi.rs`, `layout.rs`, `load.rs`,
   `bytes.rs`, `frame.rs`). The new tests load only the running image (`Library::this()`), and the
   probes load only the forge libraries and `rxsock`/`rxunixsys`, whose NEEDED lists are
   `libstdc++`, `libgcc_s` and `libc` for the one, and `libc` alone for the other.
