# Task 4 fix round 2 re-review (9956c0ac6..5a5c462ac)

Reviewer: re-review, 2026-09-26. Subject: fb860a6f2 (close order), 5a5c462ac (divergences).
Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/rereview-s4/`
(`$R`). Tree: `git archive 5a5c462ac` into `$R/tree`, every `.rs` touched, `Compiling rexx-api` and
`Compiling rexx-exec` seen in the release build. Forge rebuilt from `task-4-forge/forge.cpp` into
`$R/forge/lib` (flags inferred per tag; all 41 libraries `cmp`-identical to the implementer's
`surface-4/forge/lib`; `NEEDED` libc.so.6 only, no undefined Rexx symbol). Probes: the implementer's
22 plus the first review's 4 (`$R/p`), and 6 of my own (`$R/p2`). `$R/run.sh` is the standard
wrapper, fresh `mktemp -d`, three descriptors separately. Target dirs deleted after the run.

Status: COMPLETE. Nothing hung.

## Verdict

All of I1, M1, M3 and M4 are addressed. Two new Minor findings, no Critical or Important.

## 1. I1: destructor order vs the oracle -- FIXED

Oracle twice, crate three times, on all 26 probes in `$R/p`. Every forge line was compared,
destructors included, with `same=` dropped. The oracle was the same on both runs. The crate matched
it on all three runs of every probe: `order`, `order2`, `order3`, `orderA/B/C`, `unloadraise` and
`unloadraise2`-`5`, `uref`, and the rest. rc agreed except on the refusal probes `lref`, `lref2`
and `uref`, which are 120 by design. The Rexx lines agree except on `dtor`, `lref` and `lref2`,
where the differences are known.

What happens to a library whose unloader raised or was refused matches the oracle. It stays open
through the rest of the walk, as does every library after it in table order, and the leftovers
close in load order: `unloadraise3` gives `forgee forgef forgec forgeunload forgea` on both sides,
and `uref` gives `forgea forgeb forgeuref`. My `nohook1` checks a library with no hooks between two
that have them. On both sides `forgenone` closes at its place in the walk, after `forgea`'s close
and before `forgeb`'s unloader.

The plant proves `Drop for Libraries` does the work, and that glibc's exit order is not what
matches. I replaced `library.close()` in the drop with `let _ = library` and built
`$R/rexx-run.nodrop`. The order then went wrong: `unloadraise3` gave
`forgef forgee forgec forgeunload forgea`, `unloadraise4` gave `forgeunload forged forgea forgec forgeb`
(the oracle's is `forgec forgeb forgeunload forged forgea`), and `nohook2` was wrong too. See m1.

## 2. Close soundness -- SOUND

Oracle sequence, printed: `runtime/Interpreter.cpp:277-281` runs `InstanceBlock`, then
`memoryObject.lastChanceUninit()`, then `PackageManager::unload()`. `LibraryPackage.cpp:166-181`
runs the unloader, then `lib.unload()`. Ours has the same order: `Interp::terminate`
(`dispatch/library.rs:214-218`) runs `run_termination_uninits`, then `run_package_unloaders`.
- A row is never called once its library is closed. `invoke::method` and `invoke::routine`
  (`invoke.rs:93`) request a signature on every call, and `stub()` answers `None` for a closed
  mapping, so the call fails with `Failure::Signature` before `call_stub` runs. `run_hook` checks
  `open` before calling. Between the `open` check and `hold`, no code runs.
- `hold` counts a call as in flight, and `close` refuses while one is. Rows live inside the
  `Library` and hold the same `Rc<Mapping>`. They are found through `LibraryCodeKey` by library
  name and are never copied out.
- Probes `uninit1`, `uninit2` and `uninit4` call forge routines from termination `UNINIT`s,
  including after `Stash`. The `UNINIT`s run before any unloader, so every library is still open.
  No crash, and the lines are deterministic across three runs. The same probes turned up m2.

## 3. M1: terminate witness -- FIXED

Plant `lib.rs:3180` `interp.terminate()` -> `interp.run_termination_uninits()`, then
`cargo test -p rexx-exec --lib --no-fail-fast`. Result: 850 passed, 1 failed, and the failure is
`a_programs_end_runs_its_librarys_unloader`, `left: ["loader"]`, `right: ["loader", "unloader"]`
(`$R/plant-m1b.txt`). Unplanted: 851/0 (`$R/base-exec.txt`). The file was restored and checked
with `cmp`.

## 4. M3/M4 divergence entries -- ACCURATE, placed right

`phase-4-exclusions.txt:4642-4671` (entries at 4650, 4660, 4665). The entries sit beside the Phase 8 native-API entries, which is
where the file records accepted divergences. Each has an owner, or `OWNER: none` with a reason,
which follows the file's own rule ("its owner, or the reason it has none"). Phase 9 is the owner
the file already names for the embedding API. I checked the entries against `dtor` on both sides:
rc 134 on each, the oracle's `terminate called ... 'NativeActivation*'`, and our panic followed by
`panic in a function that cannot unwind`, with `main`/`stashed` missing from our stdout.
`Interpreter.cpp:277` confirms the fresh instance. The entry for the second context is narrower
than its cause (m2).

## 5. Miri (Stacked Borrows) -- GREEN

Command: `RUSTUP_HOME=surface-4/rustup-home cargo +nightly miri test -p rexx-api --lib --offline`,
using rustc 1.100.0-nightly (f7575a9da 2026-09-24) and a fresh `CARGO_TARGET_DIR` (`$R/miri.txt`).
Result: exit 0, 29 passed, 8 ignored. The new test, `a_closed_librarys_hook_does_not_run`, is among
the ignored ("opens the running image"), so Miri never runs `Mapping::hold`/`close` against a real
mapping. Those two are safe code around the same `unsafe` calls as before.

## 6. Unsafe / SAFETY -- PASS

The diff adds no `unsafe` operation. The five existing blocks were moved into `hold` closures, and
`method_table` and `routine_table` gained a parameter. Each moved block's note now says "in a
mapping that is open and that a close leaves open while this call is held". That is true: `stub()`
or `run_hook` checks `open`, `hold` counts the call, and `close` refuses while the count is above
zero. The field-order doc on `Library` still holds: the rows' `Rc` clones are dropped with
`methods` and `routines` before `mapping`, and `mapping` before `thread`.
`cargo test -p rexx-core --test unsafe_sites`: 2 passed.

## New findings

**Minor**

- **m1. Nothing in the tree witnesses `Drop for Libraries`.** Plant: `libraries.rs:96`
  `library.close()` -> `let _ = library`. `cargo test -p rexx-exec --no-fail-fast` still gives
  851/0 on the lib tests. The integration reds in `$R/plant-drop.txt` come from my archive, which
  lacks `ootest/` and `rust/target/release/rexx-run`. All of them panic on those missing paths.
  The forge sees this plant (section 1). The report's E1-E3 controls do not cover it. Fix: a unit
  test that keeps an `Rc<Library>` from `hold`, drops the `Libraries`, and asserts
  `!is_open()`.
- **m2. The second-instance entry names only the unloader, but termination `UNINIT`s show the same
  cause.** `$R/p2/uninit4.rex`: `call stash`, then `.local~keep = .thing~new`, and the `uninit`
  calls `same()` and then `usestash()`. Oracle: `uninit local same 0`, then a segfault with rc 139.
  Ours: `same 1`, `use 42`, rc 0. `uninit5.rex` has no `Stash`, so the kept context is the
  loader's. There the oracle prints `same 0`, drops the `usestash` line without a message and
  exits rc 0, while ours prints `use 42`. This predates the round: the 61be5ecfb-era
  `rexx-run.fix1` answers the same as 5a5c462ac. Fix: widen the entry at
  `phase-4-exclusions.txt:4660` to say "the unloader and every termination UNINIT", and cite
  `Interpreter.cpp:277-279`. The owner stays Phase 9.
